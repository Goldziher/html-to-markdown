use std::borrow::Cow;

use super::raw_text::track_svg_tag;

/// Normalize bogus HTML comment endings that confuse the `tl` parser.
pub fn normalize_bogus_comment_endings(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();

    if len < 5 || !bytes.windows(4).any(|w| w == b"<!--") {
        return Cow::Borrowed(input);
    }

    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;

    while idx + 3 < len {
        if !(bytes[idx] == b'<' && bytes[idx + 1] == b'!' && bytes[idx + 2] == b'-' && bytes[idx + 3] == b'-') {
            idx += 1;
            continue;
        }

        idx += 4;

        // ~keep `<!-->` (zero shared dashes) and `<!--->` (one shared dash) never
        // ~keep accumulate `consecutive_dashes >= 2` below, so the general scan can
        // ~keep never see them as closed -- it would run to EOF looking for an
        // ~keep unshared `-->` and swallow the rest of the document. Caught here,
        // ~keep before that scan starts.
        if bytes.get(idx) == Some(&b'>') {
            let out = output.get_or_insert_with(|| String::with_capacity(len));
            out.push_str(&input[last..idx - 4]);
            out.push_str("<!---->");
            idx += 1;
            last = idx;
            continue;
        }
        if bytes.get(idx) == Some(&b'-') && bytes.get(idx + 1) == Some(&b'>') {
            let out = output.get_or_insert_with(|| String::with_capacity(len));
            out.push_str(&input[last..idx - 4]);
            out.push_str("<!---->");
            idx += 2;
            last = idx;
            continue;
        }

        let mut consecutive_dashes: usize = 0;

        while idx < len {
            let b = bytes[idx];
            if b == b'-' {
                consecutive_dashes += 1;
                idx += 1;
            } else if b == b'>' && consecutive_dashes >= 2 {
                if consecutive_dashes > 2 {
                    let out = output.get_or_insert_with(|| String::with_capacity(len));
                    let close_start = idx - consecutive_dashes;
                    out.push_str(&input[last..close_start]);
                    out.push_str("-->");
                    idx += 1;
                    last = idx;
                } else {
                    idx += 1;
                }
                break;
            } else {
                consecutive_dashes = 0;
                idx += 1;
            }
        }
    }

    match output {
        Some(mut out) => {
            if last < len {
                out.push_str(&input[last..]);
            }
            Cow::Owned(out)
        }
        None => Cow::Borrowed(input),
    }
}

/// Normalize closing tags whose `>` appears on a subsequent line.
///
/// Some HTML formatters (JSX-style) write closing tags as:
///
/// ```html
/// </a
/// >
/// ```
///
/// The `tl` parser does not handle end-tags with a newline before the closing
/// `>`, leaving the element unclosed so all subsequent siblings become children
/// of the open element.  This pass collapses such patterns to a single-line
/// closing tag (`</a>`) before the document reaches `tl`.
///
/// Only the whitespace between the tag name and the closing `>` is normalised;
/// the rest of the document is untouched.
pub fn normalize_split_closing_tags(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();

    if len < 4 || !bytes.contains(&b'\n') {
        return Cow::Borrowed(input);
    }

    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;

    while idx + 2 < len {
        let Some(offset) = memchr::memchr(b'<', &bytes[idx..]) else {
            break;
        };
        idx += offset;
        if bytes.get(idx + 1) != Some(&b'/') {
            idx += 1;
            continue;
        }

        // ~keep Scan tag name: ASCII letters, digits, hyphens (HTML5 allows hyphens in custom elements)
        let name_start = idx + 2;
        let mut name_end = name_start;
        while name_end < len && (bytes[name_end].is_ascii_alphanumeric() || bytes[name_end] == b'-') {
            name_end += 1;
        }

        if name_end == name_start {
            idx += 1;
            continue;
        }

        let ws_start = name_end;
        let mut ws_end = ws_start;
        let mut has_newline = false;
        while ws_end < len && bytes[ws_end].is_ascii_whitespace() {
            if bytes[ws_end] == b'\n' || bytes[ws_end] == b'\r' {
                has_newline = true;
            }
            ws_end += 1;
        }

        if !has_newline || ws_end >= len || bytes[ws_end] != b'>' {
            idx += 1;
            continue;
        }

        let tag_name = &input[name_start..name_end];
        let out = output.get_or_insert_with(|| String::with_capacity(len));
        out.push_str(&input[last..idx]);
        out.push_str("</");
        out.push_str(tag_name);
        out.push('>');

        idx = ws_end + 1;
        last = idx;
    }

    match output {
        Some(mut out) => {
            if last < len {
                out.push_str(&input[last..]);
            }
            Cow::Owned(out)
        }
        None => Cow::Borrowed(input),
    }
}

/// Collapse an empty HTML comment `<!---->` at `idx` to `<!-- -->` (a single interior space).
/// Returns `Some(new_pos)` when found, `None` otherwise.
///
/// Extracted from `preprocess_html`'s main scan loop — identical prefix match and rewrite,
/// unchanged.
fn collapse_empty_comment(
    input: &str,
    bytes: &[u8],
    idx: usize,
    last: usize,
    output: &mut Option<String>,
) -> Option<usize> {
    const EMPTY_COMMENT: &[u8] = b"<!---->";
    if !bytes[idx..].starts_with(EMPTY_COMMENT) {
        return None;
    }
    let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
    out.push_str(&input[last..idx]);
    out.push_str("<!-- -->");
    Some(idx + EMPTY_COMMENT.len())
}

/// Normalize an XML-style self-closing void element (`<br/>`, `<hr/>`, `<img/>`) at `idx` to its
/// HTML5 form (`<br>`, `<hr>`, `<img>`). Returns `Some(new_pos)` when matched, `None` otherwise.
///
/// Extracted from `preprocess_html`'s main scan loop — identical pattern table and rewrite,
/// unchanged.
fn normalize_self_closing_void(
    input: &str,
    bytes: &[u8],
    idx: usize,
    last: usize,
    output: &mut Option<String>,
) -> Option<usize> {
    const SELF_CLOSING: [(&[u8], &str); 3] = [(b"<br/>", "<br>"), (b"<hr/>", "<hr>"), (b"<img/>", "<img>")];
    for (pattern, replacement) in &SELF_CLOSING {
        if bytes[idx..].starts_with(pattern) {
            let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
            out.push_str(&input[last..idx]);
            out.push_str(replacement);
            return Some(idx + pattern.len());
        }
    }
    None
}

/// Lowercase the tag name of an HTML5 void element written in upper or mixed case.
///
/// The bundled astral-tl parser compares a tag's raw source bytes against an all-lowercase
/// void-element table, so `<META>` misses it and is pushed onto the open-element stack as a
/// *container*. The parent's close tag then cannot pop it and every following sibling is
/// absorbed as its child: for `<head><META ...></head><body>` that leaves `<body>` a
/// grandchild of `<head>`, out of reach of `converter::metadata`'s direct-child rescue, and
/// the whole document converts to an empty string (issue #467). Tier-1 lowercases tag names
/// before lookup and was never affected, so this also closes a tier divergence.
///
/// Only the name is rewritten -- attribute names and values are left byte for byte alone.
/// Non-void elements need no rewrite: the parser compares open and close tags against each
/// other, so `<DIV>...</DIV>` already matches.
///
/// Returns `Some(new_pos)` when a name was rewritten, `None` otherwise.
fn normalize_void_tag_case(
    input: &str,
    bytes: &[u8],
    idx: usize,
    last: usize,
    output: &mut Option<String>,
) -> Option<usize> {
    let name_start = idx + 1;
    if !bytes.get(name_start)?.is_ascii_alphabetic() {
        return None;
    }
    let name_end = crate::converter::main_helpers::scan_tag_name_end(bytes, name_start);
    let name = &bytes[name_start..name_end];
    // ~keep Uppercase check first: it is a byte scan, while the void-element test allocates.
    if !name.iter().any(u8::is_ascii_uppercase) || !crate::converter::main_helpers::is_html5_void_element(name) {
        return None;
    }
    let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
    out.push_str(&input[last..idx]);
    out.push('<');
    out.push_str(&input[name_start..name_end].to_ascii_lowercase());
    Some(name_end)
}

/// Strip a raw-text `<script>` or `<style>` element in `preprocess_html`'s pass, replacing its
/// content with an empty element (`<tag ...></tag>`) rather than removing it outright. A
/// `<script type="application/ld+json">` open tag is left alone (its JSON-LD body must survive
/// for metadata extraction), matching neither `TAGS` entry in that case — mirrored by the
/// `continue` below, which moves on to the next `TAGS` entry instead of stopping the scan.
///
/// Returns `Some(new_pos)` when a tag was stripped, `None` when `idx` does not start such a tag.
///
/// Extracted from `preprocess_html`'s main scan loop — identical tag matching and JSON-LD
/// exemption, unchanged.
fn strip_raw_text_tag_reopen(
    input: &str,
    bytes: &[u8],
    idx: usize,
    last: usize,
    output: &mut Option<String>,
) -> Option<usize> {
    const TAGS: [&[u8]; 2] = [b"script", b"style"];

    for tag in TAGS {
        if matches_tag_start(bytes, idx + 1, tag) {
            if let Some(open_end) = find_tag_end(bytes, idx + 1 + tag.len()) {
                if tag == b"script" && is_json_ld_script_open_tag(&input[idx..open_end]) {
                    continue;
                }
                let remove_end = find_closing_tag(bytes, open_end, tag).unwrap_or(open_end);
                let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                out.push_str(&input[last..idx]);
                out.push_str(&input[idx..open_end]);
                out.push_str("</");
                if let Ok(tag_str) = str::from_utf8(tag) {
                    out.push_str(tag_str);
                }
                out.push('>');

                return Some(remove_end);
            }
        }
    }

    None
}

/// Strip a `<!doctype ...>` declaration at `idx`. Returns `Some(new_pos)` when found and
/// terminated, `None` otherwise.
///
/// Extracted from `preprocess_html`'s main scan loop — identical whitespace-skip and
/// case-insensitive `doctype` match, unchanged.
fn strip_doctype(
    input: &str,
    bytes: &[u8],
    idx: usize,
    len: usize,
    last: usize,
    output: &mut Option<String>,
) -> Option<usize> {
    const DOCTYPE: &[u8] = b"doctype";

    if idx + 2 >= len || bytes[idx + 1] != b'!' {
        return None;
    }
    let mut cursor = idx + 2;
    while cursor < len && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }

    if cursor + DOCTYPE.len() > len || !bytes[cursor..cursor + DOCTYPE.len()].eq_ignore_ascii_case(DOCTYPE) {
        return None;
    }

    let end = find_tag_end(bytes, cursor + DOCTYPE.len())?;
    let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
    out.push_str(&input[last..idx]);
    Some(end)
}

/// Whether `<` at `idx` opens something that looks like real markup (a tag, a `<!...>`
/// declaration/comment, or a `<?...?>` processing instruction) rather than a bare `<` that must
/// be entity-escaped so a later HTML parse does not swallow following content as a bogus tag.
///
/// Extracted from `preprocess_html`'s main scan loop — identical byte-class checks, unchanged.
fn looks_like_tag_start(bytes: &[u8], idx: usize, len: usize) -> bool {
    if idx + 1 >= len {
        return false;
    }
    match bytes[idx + 1] {
        b'!' => {
            idx + 2 < len
                && (bytes[idx + 2] == b'-'
                    || bytes[idx + 2].is_ascii_alphabetic()
                    || bytes[idx + 2].is_ascii_uppercase())
        }
        b'/' => idx + 2 < len && (bytes[idx + 2].is_ascii_alphabetic() || bytes[idx + 2].is_ascii_uppercase()),
        b'?' => true,
        c if c.is_ascii_alphabetic() || c.is_ascii_uppercase() => true,
        _ => false,
    }
}

/// Preprocess HTML to normalize tags and fix common issues.
pub fn preprocess_html(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return Cow::Borrowed(input);
    }

    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;
    let mut svg_depth = 0usize;
    let last_gt = bytes.iter().rposition(|&byte| byte == b'>');

    while idx < len {
        if bytes[idx] == b'<' {
            if last_gt.is_none_or(|end| end <= idx) && looks_like_tag_start(bytes, idx, len) {
                break;
            }
            if let Some(new_pos) = collapse_empty_comment(input, bytes, idx, last, &mut output) {
                last = new_pos;
                idx = new_pos;
                continue;
            }

            if let Some(new_pos) = normalize_self_closing_void(input, bytes, idx, last, &mut output) {
                last = new_pos;
                idx = new_pos;
                continue;
            }

            if let Some(new_idx) = track_svg_tag(bytes, idx, &mut svg_depth) {
                idx = new_idx;
                continue;
            }

            if svg_depth == 0 {
                if let Some(new_pos) = normalize_void_tag_case(input, bytes, idx, last, &mut output) {
                    last = new_pos;
                    idx = new_pos;
                    continue;
                }

                if let Some(new_pos) = strip_raw_text_tag_reopen(input, bytes, idx, last, &mut output) {
                    last = new_pos;
                    idx = new_pos;
                    continue;
                }

                if let Some(new_pos) = strip_doctype(input, bytes, idx, len, last, &mut output) {
                    last = new_pos;
                    idx = new_pos;
                    continue;
                }
            }

            if !looks_like_tag_start(bytes, idx, len) {
                let out = output.get_or_insert_with(|| String::with_capacity(input.len() + 4));
                out.push_str(&input[last..idx]);
                out.push_str("&lt;");
                idx += 1;
                last = idx;
                continue;
            }
        }

        idx += 1;
    }

    if let Some(mut out) = output {
        if last < len {
            out.push_str(&input[last..]);
        }
        Cow::Owned(out)
    } else {
        Cow::Borrowed(input)
    }
}

/// Find the end of an unquoted attribute value starting at `start`: the first ASCII whitespace
/// or `>` byte, or `bytes.len()` if neither occurs before the end of `bytes`.
///
/// Extracted from `is_json_ld_script_open_tag`'s unquoted-value branch — identical scan,
/// unchanged.
fn scan_unquoted_attr_value_end(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < bytes.len() && !bytes[end].is_ascii_whitespace() && bytes[end] != b'>' {
        end += 1;
    }
    end
}

/// Check if a script tag is a JSON-LD script.
pub fn is_json_ld_script_open_tag(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    let mut idx = 0;
    while idx + 4 <= bytes.len() {
        if eq_ascii_case_insensitive(&bytes[idx..], b"type") {
            let before_ok = idx == 0
                || bytes
                    .get(idx.saturating_sub(1))
                    .is_some_and(|b| b.is_ascii_whitespace() || *b == b'<' || *b == b'/');
            let after_ok = bytes
                .get(idx + 4)
                .is_some_and(|b| b.is_ascii_whitespace() || *b == b'=');
            if !before_ok || !after_ok {
                idx += 4;
                continue;
            }

            let mut i = idx + 4;
            while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
                i += 1;
            }
            if bytes.get(i) != Some(&b'=') {
                idx += 4;
                continue;
            }
            i += 1;
            while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
                i += 1;
            }
            if i >= bytes.len() {
                return false;
            }

            let (value_start, value_end) = match bytes[i] {
                b'"' | b'\'' => {
                    let quote = bytes[i];
                    let start = i + 1;
                    let mut end = start;
                    while end < bytes.len() && bytes[end] != quote {
                        end += 1;
                    }
                    (start, end)
                }
                _ => {
                    let start = i;
                    (start, scan_unquoted_attr_value_end(bytes, start))
                }
            };

            // ~keep The decision reads the decoded value: `ld&#43;json` names the same type.
            let value = crate::text::decode_attribute_value_cow(&tag[value_start..value_end]);
            let media_type = value.split(';').next().unwrap_or(&value).trim();
            return eq_ascii_case_insensitive(media_type.as_bytes(), b"application/ld+json");
        }
        idx += 1;
    }
    false
}

/// Case-insensitive byte comparison for ASCII.
#[inline]
pub fn eq_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    haystack
        .iter()
        .zip(needle.iter())
        .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

/// Check if bytes match a tag start pattern.
pub fn matches_tag_start(bytes: &[u8], mut start: usize, tag: &[u8]) -> bool {
    if start >= bytes.len() {
        return false;
    }

    if start + tag.len() > bytes.len() {
        return false;
    }

    if !bytes[start..start + tag.len()].eq_ignore_ascii_case(tag) {
        return false;
    }

    start += tag.len();

    match bytes.get(start) {
        Some(b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r') => true,
        Some(_) => false,
        None => true,
    }
}

/// Find the end of an HTML tag (the position of '>').
pub fn find_tag_end(bytes: &[u8], mut idx: usize) -> Option<usize> {
    loop {
        idx += memchr::memchr3(b'"', b'\'', b'>', bytes.get(idx..)?)?;
        let delimiter = bytes[idx];
        idx += 1;
        if delimiter == b'>' {
            return Some(idx);
        }
        idx += memchr::memchr(delimiter, &bytes[idx..])? + 1;
    }
}

/// Find the closing tag for a given tag name.
pub fn find_closing_tag(bytes: &[u8], mut idx: usize, tag: &[u8]) -> Option<usize> {
    let len = bytes.len();
    let mut depth = 1usize;

    while idx < len {
        if let Some(next) = matching_open_end(bytes, idx, tag) {
            depth += 1;
            idx = next;
            continue;
        }
        if let Some(close) = matching_close_end(bytes, idx, tag) {
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

fn matching_open_end(bytes: &[u8], idx: usize, tag: &[u8]) -> Option<usize> {
    (bytes.get(idx) == Some(&b'<') && matches_tag_start(bytes, idx + 1, tag))
        .then(|| find_tag_end(bytes, idx + 1 + tag.len()))
        .flatten()
}

fn matching_close_end(bytes: &[u8], idx: usize, tag: &[u8]) -> Option<usize> {
    (bytes.get(idx) == Some(&b'<') && matches_end_tag_start(bytes, idx + 1, tag))
        .then(|| find_tag_end(bytes, idx + 2 + tag.len()))
        .flatten()
}

/// Check if bytes match an end tag pattern.
pub fn matches_end_tag_start(bytes: &[u8], start: usize, tag: &[u8]) -> bool {
    if start >= bytes.len() || bytes[start] != b'/' {
        return false;
    }
    matches_tag_start(bytes, start + 1, tag)
}
