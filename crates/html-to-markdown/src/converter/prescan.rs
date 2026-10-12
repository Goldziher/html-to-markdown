//! Single-pass byte scanner that cleans HTML and emits signals
//! consumed by the tier-1/tier-2 router (added in M2).

use std::borrow::Cow;
use std::str;

/// Signals captured during the single prescan pass.
#[derive(Debug, Default, Clone)]
pub struct PrescanReport {
    /// Any tag-open whose name contains `-` (custom-elements heuristic).
    pub had_custom_elements: bool,
    /// Any occurrence of `<![CDATA[`.
    pub had_cdata: bool,
    /// Any `<` that the prescan escaped via the invalid-tag branch.
    pub had_unescaped_lt: bool,
    /// Saw `<script>` or `<style>` in the source.
    pub has_script_or_style: bool,
    /// SVG depth ever exceeded zero.
    pub has_svg: bool,
}

// ~keep Tags that are stripped of their content by the prescan.
const STRIP_CONTENT_TAGS: [&[u8]; 2] = [b"script", b"style"];

const SVG_TAG: &[u8] = b"svg";
const CDATA_START: &[u8] = b"<![CDATA[";
const DOCTYPE: &[u8] = b"doctype";
const EMPTY_COMMENT: &[u8] = b"<!---->";
const SELF_CLOSING: [(&[u8], &str); 3] = [(b"<br/>", "<br>"), (b"<hr/>", "<hr>"), (b"<img/>", "<img>")];

/// Run the prescan over `html`, returning the cleaned buffer and signals.
///
/// `Cow::Borrowed` is returned when no transformation was needed.
///
/// # Panics
///
/// Panics if a tag-name byte sequence encountered during script/style stripping
/// is not valid UTF-8 (this cannot happen in practice because it is always a
/// sub-slice of the valid UTF-8 input `html`).
#[must_use]
pub fn run(html: &str) -> (Cow<'_, str>, PrescanReport) {
    if html.is_empty() {
        return (Cow::Borrowed(html), PrescanReport::default());
    }
    Prescanner::new(html).scan()
}

struct Prescanner<'a> {
    html: &'a str,
    bytes: &'a [u8],
    report: PrescanReport,
    idx: usize,
    last: usize,
    output: Option<String>,
    svg_depth: usize,
}

impl<'a> Prescanner<'a> {
    fn new(html: &'a str) -> Self {
        Self {
            html,
            bytes: html.as_bytes(),
            report: PrescanReport::default(),
            idx: 0,
            last: 0,
            output: None,
            svg_depth: 0,
        }
    }

    fn scan(mut self) -> (Cow<'a, str>, PrescanReport) {
        while self.idx < self.bytes.len() {
            if self.bytes[self.idx] != b'<' {
                self.idx += 1;
                continue;
            }
            self.scan_markup();
        }
        self.finish()
    }

    fn scan_markup(&mut self) {
        if self.bytes[self.idx..].starts_with(CDATA_START) {
            self.report.had_cdata = true;
        }
        if self.replace_empty_comment() || self.replace_self_closing() || self.track_svg() {
            return;
        }
        if self.svg_depth == 0 && self.handle_outside_svg() {
            return;
        }
        if !is_valid_tag_start(self.bytes, self.idx) {
            self.escape_less_than();
            return;
        }
        self.idx += 1;
    }

    fn replace_empty_comment(&mut self) -> bool {
        if !self.bytes[self.idx..].starts_with(EMPTY_COMMENT) {
            return false;
        }
        self.flush_prefix();
        self.output.as_mut().expect("output initialized").push_str("<!-- -->");
        self.idx += EMPTY_COMMENT.len();
        self.last = self.idx;
        true
    }

    fn replace_self_closing(&mut self) -> bool {
        let Some((pattern, replacement)) = SELF_CLOSING
            .iter()
            .find(|(pattern, _)| self.bytes[self.idx..].starts_with(pattern))
        else {
            return false;
        };
        self.flush_prefix();
        self.output.as_mut().expect("output initialized").push_str(replacement);
        self.idx += pattern.len();
        self.last = self.idx;
        true
    }

    fn track_svg(&mut self) -> bool {
        if matches_tag_start(self.bytes, self.idx + 1, SVG_TAG) {
            let Some(open_end) = find_tag_end(self.bytes, self.idx + 1 + SVG_TAG.len()) else {
                return false;
            };
            self.svg_depth += 1;
            self.report.has_svg = true;
            self.idx = open_end;
            return true;
        }
        if !matches_end_tag_start(self.bytes, self.idx + 1, SVG_TAG) {
            return false;
        }
        let Some(close_end) = find_tag_end(self.bytes, self.idx + 2 + SVG_TAG.len()) else {
            return false;
        };
        self.svg_depth = self.svg_depth.saturating_sub(1);
        self.idx = close_end;
        true
    }

    fn handle_outside_svg(&mut self) -> bool {
        if self.strip_raw_text() || self.strip_doctype() {
            return true;
        }
        self.track_custom_element();
        false
    }

    fn strip_raw_text(&mut self) -> bool {
        let Some(tag) = STRIP_CONTENT_TAGS
            .iter()
            .find(|tag| matches_tag_start(self.bytes, self.idx + 1, tag))
        else {
            return false;
        };
        let Some(open_end) = find_tag_end(self.bytes, self.idx + 1 + tag.len()) else {
            return false;
        };
        self.report.has_script_or_style = true;
        let remove_end = find_closing_tag(self.bytes, open_end, tag).unwrap_or(self.bytes.len());
        self.flush_prefix();
        let output = self.output.as_mut().expect("output initialized");
        output.push_str(&self.html[self.idx..open_end]);
        output.push_str("</");
        output.push_str(str::from_utf8(tag).expect("tag names are valid UTF-8"));
        output.push('>');
        self.last = remove_end;
        self.idx = remove_end;
        true
    }

    fn strip_doctype(&mut self) -> bool {
        if self.idx + 2 >= self.bytes.len() || self.bytes[self.idx + 1] != b'!' {
            return false;
        }
        let mut cursor = self.idx + 2;
        while cursor < self.bytes.len() && self.bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor + DOCTYPE.len() > self.bytes.len()
            || !self.bytes[cursor..cursor + DOCTYPE.len()].eq_ignore_ascii_case(DOCTYPE)
        {
            return false;
        }
        let Some(end) = find_tag_end(self.bytes, cursor + DOCTYPE.len()) else {
            return false;
        };
        self.flush_prefix();
        self.last = end;
        self.idx = end;
        true
    }

    fn track_custom_element(&mut self) {
        let tag_start = self.idx + 1;
        if tag_start >= self.bytes.len() || !self.bytes[tag_start].is_ascii_alphabetic() {
            return;
        }
        let mut name_end = tag_start;
        while name_end < self.bytes.len()
            && (self.bytes[name_end].is_ascii_alphanumeric()
                || self.bytes[name_end] == b'-'
                || self.bytes[name_end] == b'_')
        {
            name_end += 1;
        }
        self.report.had_custom_elements |= self.bytes[tag_start..name_end].contains(&b'-');
    }

    fn escape_less_than(&mut self) {
        self.report.had_unescaped_lt = true;
        self.flush_prefix_with_capacity(4);
        self.output.as_mut().expect("output initialized").push_str("&lt;");
        self.idx += 1;
        self.last = self.idx;
    }

    fn flush_prefix(&mut self) {
        self.flush_prefix_with_capacity(0);
    }

    fn flush_prefix_with_capacity(&mut self, extra: usize) {
        let output = self
            .output
            .get_or_insert_with(|| String::with_capacity(self.html.len() + extra));
        output.push_str(&self.html[self.last..self.idx]);
    }

    fn finish(self) -> (Cow<'a, str>, PrescanReport) {
        let cleaned = if let Some(mut output) = self.output {
            if self.last < self.bytes.len() {
                output.push_str(&self.html[self.last..]);
            }
            Cow::Owned(output)
        } else {
            Cow::Borrowed(self.html)
        };
        (cleaned, self.report)
    }
}

fn is_valid_tag_start(bytes: &[u8], idx: usize) -> bool {
    let Some(next) = bytes.get(idx + 1) else {
        return false;
    };
    match next {
        b'!' => {
            let Some(after_bang) = bytes.get(idx + 2) else {
                return false;
            };
            *after_bang == b'-' || after_bang.is_ascii_alphabetic()
        }
        b'/' => bytes.get(idx + 2).is_some_and(u8::is_ascii_alphabetic),
        b'?' => true,
        value => value.is_ascii_alphabetic(),
    }
}

fn matches_tag_start(bytes: &[u8], mut start: usize, tag: &[u8]) -> bool {
    if start >= bytes.len() || start + tag.len() > bytes.len() {
        return false;
    }
    if !bytes[start..start + tag.len()].eq_ignore_ascii_case(tag) {
        return false;
    }
    start += tag.len();
    matches!(
        bytes.get(start),
        Some(b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r') | None
    )
}

fn matches_end_tag_start(bytes: &[u8], start: usize, tag: &[u8]) -> bool {
    if start >= bytes.len() || bytes[start] != b'/' {
        return false;
    }
    matches_tag_start(bytes, start + 1, tag)
}

fn find_tag_end(bytes: &[u8], mut idx: usize) -> Option<usize> {
    let len = bytes.len();
    let mut in_quote: Option<u8> = None;
    while idx < len {
        match bytes[idx] {
            b'"' | b'\'' => {
                if let Some(current) = in_quote {
                    if current == bytes[idx] {
                        in_quote = None;
                    }
                } else {
                    in_quote = Some(bytes[idx]);
                }
            }
            b'>' if in_quote.is_none() => return Some(idx + 1),
            _ => {}
        }
        idx += 1;
    }
    None
}

fn find_closing_tag(bytes: &[u8], mut idx: usize, tag: &[u8]) -> Option<usize> {
    let len = bytes.len();
    let mut depth = 1usize;
    while idx < len {
        if let Some(next) = nested_open_end(bytes, idx, tag) {
            depth += 1;
            idx = next;
            continue;
        }
        if let Some(close) = nested_close_end(bytes, idx, tag) {
            depth -= 1;
            if depth == 0 {
                return Some(close);
            }
            idx = close;
            continue;
        }
        idx += 1;
    }
    None
}

fn nested_open_end(bytes: &[u8], idx: usize, tag: &[u8]) -> Option<usize> {
    (bytes.get(idx) == Some(&b'<') && matches_tag_start(bytes, idx + 1, tag))
        .then(|| find_tag_end(bytes, idx + 1 + tag.len()))
        .flatten()
}

fn nested_close_end(bytes: &[u8], idx: usize, tag: &[u8]) -> Option<usize> {
    (bytes.get(idx) == Some(&b'<') && matches_end_tag_start(bytes, idx + 1, tag))
        .then(|| find_tag_end(bytes, idx + 2 + tag.len()))
        .flatten()
}
