//! HTML preprocessing and normalization.
//!
//! Functions for preprocessing HTML before conversion, including script/style stripping,
//! tag repair, and malformed HTML handling.

use std::borrow::Cow;
use std::str;

use super::markup::{find_tag_end, is_json_ld_script_open_tag, matches_end_tag_start, matches_tag_start};

/// If `idx` starts an `<svg`/`</svg` tag, adjust `svg_depth` accordingly and return the index
/// just past the tag. Returns `None` when `idx` does not start an svg tag (or the tag is
/// unterminated), in which case the caller's scan is unaffected.
///
/// Extracted from `strip_script_and_style_tags` — identical `<svg`/`</svg` detection and depth
/// bookkeeping, unchanged.
pub(super) fn track_svg_tag(bytes: &[u8], idx: usize, svg_depth: &mut usize) -> Option<usize> {
    if matches_tag_start(bytes, idx + 1, b"svg") {
        if let Some(open_end) = find_tag_end(bytes, idx + 1 + b"svg".len()) {
            *svg_depth += 1;
            return Some(open_end);
        }
    } else if matches_end_tag_start(bytes, idx + 1, b"svg") {
        if let Some(close_end) = find_tag_end(bytes, idx + 2 + b"svg".len()) {
            if *svg_depth > 0 {
                *svg_depth = svg_depth.saturating_sub(1);
            }
            return Some(close_end);
        }
    }
    None
}

/// Strip a `<script>…</script>` or `<style>…</style>` element starting at `idx` (which must
/// point at the `<`), appending everything before it to `*output` (lazily allocated) and
/// inserting a single space if collapsing the removed span would otherwise fuse two
/// non-whitespace characters together.
///
/// `open_tag_pattern` is the lowercase opening-tag prefix including `<` (e.g. `b"<script"`);
/// `close_tag_name` is the bare lowercase tag name for the closing-tag scan (e.g. `b"script"`).
/// `json_ld_exempt` additionally requires the opening tag not be a `type="application/ld+json"`
/// script — only meaningful for the script element, so callers stripping `<style>` pass `false`.
///
/// Returns `Some(new_pos)` when the element was stripped — the caller should set both `last`
/// and `idx` to it and `continue` the scan — or `None` when `idx` does not start a strippable
/// element of this kind, in which case the caller falls through unchanged.
///
/// Extracted from `strip_script_and_style_tags` — identical prefix/whitespace-boundary/
/// closing-tag-scan logic, unchanged.
struct RawTextElement<'a> {
    input: &'a str,
    bytes: &'a [u8],
    idx: usize,
    len: usize,
    last: usize,
    open_tag_pattern: &'a [u8],
    close_tag_name: &'a [u8],
    json_ld_exempt: bool,
}

impl<'a> RawTextElement<'a> {
    const fn new(
        input: &'a str,
        idx: usize,
        last: usize,
        open_tag_pattern: &'a [u8],
        close_tag_name: &'a [u8],
        json_ld_exempt: bool,
    ) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            idx,
            len: input.len(),
            last,
            open_tag_pattern,
            close_tag_name,
            json_ld_exempt,
        }
    }
}

fn strip_raw_text_element(element: RawTextElement<'_>, output: &mut Option<String>) -> Option<usize> {
    let prefix_len = element.open_tag_pattern.len();
    if element.idx + prefix_len >= element.len
        || !eq_ascii_insensitive(
            &element.bytes[element.idx..element.idx + prefix_len],
            element.open_tag_pattern,
        )
    {
        return None;
    }
    let after_tag = element.bytes[element.idx + prefix_len];
    if !(after_tag == b'>' || after_tag == b' ' || after_tag == b'\t' || after_tag == b'\n' || after_tag == b'\r') {
        return None;
    }

    let mut tag_end = element.idx + prefix_len;
    while tag_end < element.len && element.bytes[tag_end] != b'>' {
        tag_end += 1;
    }
    if tag_end >= element.len {
        return None;
    }
    tag_end += 1;

    if element.json_ld_exempt {
        let tag_content = &element.input[element.idx..tag_end];
        if is_json_ld_script_open_tag(tag_content) {
            return None;
        }
    }

    let close_idx = find_closing_tag_bytes(element.bytes, tag_end, element.close_tag_name)?;

    let out = output.get_or_insert_with(|| String::with_capacity(element.len));
    out.push_str(&element.input[element.last..element.idx]);
    if element.idx > 0
        && close_idx < element.len
        && !element.bytes[element.idx - 1].is_ascii_whitespace()
        && !element.bytes[close_idx].is_ascii_whitespace()
    {
        out.push(' ');
    }
    Some(close_idx)
}

/// Strip script and style tags and their content from HTML.
pub fn strip_script_and_style_tags(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();

    if len == 0 {
        return Cow::Borrowed(input);
    }

    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;
    let mut svg_depth = 0usize;

    while idx < len {
        let Some(offset) = memchr::memchr(b'<', &bytes[idx..]) else {
            break;
        };
        idx += offset;

        if idx + 1 < len {
            if let Some(new_idx) = track_svg_tag(bytes, idx, &mut svg_depth) {
                idx = new_idx;
                continue;
            }

            if svg_depth > 0 {
                idx += 1;
                continue;
            }

            // ~keep Check for </script or </style (closing tags first for safety)
            if bytes[idx + 1] == b'/' && idx + 2 < len {
                if idx + 9 <= len && eq_ascii_insensitive(&bytes[idx..idx + 9], b"</script>") {
                    idx += 9;
                    continue;
                }

                if idx + 8 <= len && eq_ascii_insensitive(&bytes[idx..idx + 8], b"</style>") {
                    idx += 8;
                    continue;
                }
            }

            if let Some(new_pos) = strip_raw_text_element(
                RawTextElement::new(input, idx, last, b"<script", b"script", true),
                &mut output,
            ) {
                last = new_pos;
                idx = new_pos;
                continue;
            }

            if let Some(new_pos) = strip_raw_text_element(
                RawTextElement::new(input, idx, last, b"<style", b"style", false),
                &mut output,
            ) {
                last = new_pos;
                idx = new_pos;
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

/// Upper bound on how far a closing-tag scan may travel from its start offset.
///
/// ~keep DoS guard: hostile input can contain an unterminated element followed
/// ~keep by hundreds of megabytes of text; the scan gives up rather than walking
/// ~keep the whole document for every such tag.
pub(super) const MAX_CLOSING_TAG_SCAN: usize = 100_000_000;

/// If `idx` starts a `</tag>` (or `</tag ...>`) closing tag matching `tag` (case-insensitively),
/// return the index just past its `>`. Returns `None` when `idx` does not start such a tag.
///
/// Extracted from `find_closing_tag_bytes`'s inner match — identical boundary checks and
/// closing-`>` scan, unchanged.
fn match_closing_tag_at(bytes: &[u8], idx: usize, len: usize, tag: &[u8]) -> Option<usize> {
    let tag_len = tag.len();
    if idx + 2 >= len || bytes[idx + 1] != b'/' {
        return None;
    }
    if idx + 2 + tag_len > len || !eq_ascii_insensitive(&bytes[idx + 2..idx + 2 + tag_len], tag) {
        return None;
    }
    let after_tag = idx + 2 + tag_len;
    if after_tag >= len || !(bytes[after_tag] == b'>' || bytes[after_tag].is_ascii_whitespace()) {
        return None;
    }

    let mut close_idx = after_tag;
    while close_idx < len && bytes[close_idx] != b'>' {
        close_idx += 1;
    }
    if close_idx < len { Some(close_idx + 1) } else { None }
}

/// Find the position of the FIRST closing tag in bytes, ignoring nesting.
/// Returns the position AFTER the closing tag (including the '>').
/// This is highly optimized for performance and uses a fast-path scan.
///
/// ~keep First-match (not depth-aware) is correct ONLY for the HTML raw-text
/// ~keep elements `<script>` and `<style>`: inside their bodies `<script` is
/// ~keep text, not a tag, so the first `</script>` is the spec terminator.
/// ~keep Counting depth here would treat `var s = "<script>"` as an opening tag
/// ~keep and run past the real close, swallowing the rest of the document.
/// ~keep Elements that can nest must use `find_closing_tag_bytes_nested`; do not
/// ~keep unify the two functions.
#[inline]
pub fn find_closing_tag_bytes(bytes: &[u8], start: usize, tag: &[u8]) -> Option<usize> {
    let len = bytes.len();
    let mut idx = start;

    while idx < len && (idx - start) < MAX_CLOSING_TAG_SCAN {
        if bytes[idx] != b'<' {
            if let Some(pos) = memchr::memchr(b'<', &bytes[idx..]) {
                idx += pos;
            } else {
                break;
            }
        }

        if let Some(close_pos) = match_closing_tag_at(bytes, idx, len, tag) {
            return Some(close_pos);
        }

        idx += 1;
    }

    None
}

/// Void elements that never carry a closing tag, so an occurrence of one inside
/// a subtree must not raise the nesting depth of a closing-tag scan.
const VOID_ELEMENT_NAMES: [&[u8]; 4] = [b"br", b"hr", b"img", b"input"];

/// Whether an opening tag closes itself: XHTML-style `<tag/>` or a void element.
///
/// ~keep Shared by `strip_hidden_elements` and `find_closing_tag_bytes_nested`
/// ~keep so the element the stripper treats as self-closing and the element the
/// ~keep depth counter refuses to count can never disagree.
#[inline]
pub(super) fn is_self_closing_tag(tag_slice: &[u8], tag_name: &[u8]) -> bool {
    tag_slice.ends_with(b"/>")
        || VOID_ELEMENT_NAMES
            .iter()
            .any(|void_name| tag_name.eq_ignore_ascii_case(void_name))
}

/// Byte sequences delimiting the regions of an HTML document whose bytes are
/// character data rather than markup.
const COMMENT_OPEN: &[u8] = b"<!--";
const COMMENT_CLOSE: &[u8] = b"-->";
const CDATA_OPEN: &[u8] = b"<![CDATA[";
const CDATA_CLOSE: &[u8] = b"]]>";

/// Elements whose body is raw text: `<` inside them never starts a tag.
const RAW_TEXT_ELEMENT_NAMES: [&[u8]; 2] = [b"script", b"style"];

/// Index just past the first occurrence of `terminator` at or after `from`,
/// or the input length when the region is never terminated.
///
/// ~keep An unterminated comment/CDATA region runs to EOF per HTML5, so
/// ~keep returning `len` (rather than `None`) keeps the caller from resuming a
/// ~keep tag scan inside text that will never be markup.
#[inline]
fn find_sequence_end(bytes: &[u8], from: usize, terminator: &[u8]) -> usize {
    let len = bytes.len();
    if from >= len {
        return len;
    }
    match bytes[from..].windows(terminator.len()).position(|w| w == terminator) {
        Some(offset) => from + offset + terminator.len(),
        None => len,
    }
}

/// If a `<` at `idx` opens a region that is NOT markup — an HTML comment, a
/// CDATA section, or the body of a raw-text element — return the index just
/// past that region. Returns `None` when `idx` does not open such a region.
///
/// ~keep A depth-counting close-tag scan MUST consult this before interpreting
/// ~keep a `<`: `<!-- </div> -->` and `<script>{"a":"</div>"}</script>` contain
/// ~keep byte sequences that look like tags but are text. Counting them ends the
/// ~keep scan early (leaking the hidden element's tail into the output) or
/// ~keep inflates the depth (swallowing the following visible sibling).
#[inline]
pub fn skip_opaque_region(bytes: &[u8], idx: usize) -> Option<usize> {
    let len = bytes.len();
    if idx >= len || bytes[idx] != b'<' {
        return None;
    }

    if bytes[idx..].starts_with(COMMENT_OPEN) {
        return Some(find_sequence_end(bytes, idx + COMMENT_OPEN.len(), COMMENT_CLOSE));
    }

    if bytes[idx..].starts_with(CDATA_OPEN) {
        return Some(find_sequence_end(bytes, idx + CDATA_OPEN.len(), CDATA_CLOSE));
    }

    for raw_text_name in RAW_TEXT_ELEMENT_NAMES {
        if matches_tag_start(bytes, idx + 1, raw_text_name) {
            let open_end = find_tag_end(bytes, idx + 1 + raw_text_name.len())?;
            // ~keep First-match close is the spec terminator for raw text — see
            // ~keep the doc comment on `find_closing_tag_bytes`.
            return Some(find_closing_tag_bytes(bytes, open_end, raw_text_name).unwrap_or(len));
        }
    }

    None
}

/// Whether the `<` at `idx` starts a tag rather than a stray `<` in text.
///
/// ~keep Mirrors the `is_valid_tag` test in `preprocess_html`: without it a
/// ~keep literal `<` in text (`a < b`) would make a scan jump to the next `>`,
/// ~keep stepping over the real closing tag in between.
#[inline]
pub(super) fn opens_a_tag(bytes: &[u8], idx: usize) -> bool {
    match bytes.get(idx + 1) {
        Some(b'/' | b'!') => bytes.get(idx + 2).is_some_and(u8::is_ascii_alphabetic),
        Some(byte) => byte.is_ascii_alphabetic(),
        None => false,
    }
}

/// Find the closing tag that matches an opening `<tag>`, counting nested
/// elements of the same name. Returns the position AFTER the closing tag
/// (including the '>'), or `None` if the element is never closed.
///
/// ~keep Required for elements that can nest (`div`, `span`, ...): the
/// ~keep first-match `find_closing_tag_bytes` stops at the INNER `</tag>`, which
/// ~keep leaves the hidden element's tail in the document. Kept separate from
/// ~keep that function because raw-text elements must NOT count depth — see its
/// ~keep doc comment.
///
/// ~keep Only real markup may move the depth counter, so every `<` is first
/// ~keep offered to `skip_opaque_region`, and a tag that is not the target is
/// ~keep jumped via the quote-aware `find_tag_end` rather than byte-by-byte:
/// ~keep otherwise `</tag>` written inside a comment, a quoted attribute value,
/// ~keep or a raw-text body is mistaken for the element's terminator.
#[inline]
pub fn find_closing_tag_bytes_nested(bytes: &[u8], start: usize, tag: &[u8]) -> Option<usize> {
    let len = bytes.len();
    if tag.is_empty() {
        return None;
    }

    let mut idx = start;
    let mut depth = 1usize;

    while idx < len && (idx - start) < MAX_CLOSING_TAG_SCAN {
        if bytes[idx] != b'<' {
            match memchr::memchr(b'<', &bytes[idx..]) {
                Some(pos) => idx += pos,
                None => break,
            }
        }

        if let Some(region_end) = skip_opaque_region(bytes, idx) {
            idx = region_end;
            continue;
        }

        if matches_end_tag_start(bytes, idx + 1, tag) {
            if let Some(close_end) = find_tag_end(bytes, idx + 2 + tag.len()) {
                depth -= 1;
                if depth == 0 {
                    return Some(close_end);
                }
                idx = close_end;
                continue;
            }
        } else if matches_tag_start(bytes, idx + 1, tag) {
            if let Some(open_end) = find_tag_end(bytes, idx + 1 + tag.len()) {
                if !is_self_closing_tag(&bytes[idx..open_end], tag) {
                    depth += 1;
                }
                idx = open_end;
                continue;
            }
        } else if opens_a_tag(bytes, idx) {
            if let Some(unrelated_tag_end) = find_tag_end(bytes, idx + 1) {
                idx = unrelated_tag_end;
                continue;
            }
        }

        idx += 1;
    }

    None
}

/// Compare bytes ignoring ASCII case.
#[inline]
pub fn eq_ascii_insensitive(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| x.eq_ignore_ascii_case(y))
}
