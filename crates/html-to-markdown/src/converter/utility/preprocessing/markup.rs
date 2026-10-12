use std::borrow::Cow;

use super::raw_text::{PageReads, PassRead, removed_element_name, track_svg_tag};

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
fn normalize_void_tag_case<C: FnMut(PassRead)>(
    input: &str,
    idx: usize,
    last: usize,
    last_gt: usize,
    reads: &mut PageReads<C>,
    output: &mut Option<String>,
) -> Option<usize> {
    let bytes = input.as_bytes();
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
    // ~keep The rest of the tag is read with it: a `<` in an attribute value starts nothing.
    // ~keep No tag ends after the last `>` of the page.
    let whole = if idx < last_gt { reads.tag_end(bytes, idx) } else { None };
    let end = whole.unwrap_or(name_end);
    out.push_str(&input[name_end..end]);
    Some(end)
}

/// Close a raw-text `<script>` or `<style>` start tag at `idx` in the second pass: the element
/// is written as an empty element (`<tag ...></tag>`), not removed outright. A
/// `<script type="application/ld+json">` start tag is left alone (its JSON-LD body must survive
/// for metadata extraction).
///
/// Returns `Some(new_pos)` when a tag was closed, `None` when `idx` does not start such a tag.
///
/// ~keep The first pass removes each element that has an end tag, so the element here has
/// ~keep none as a rule. It is closed at its start tag and the markup after it is written: the
/// ~keep rule of issue 277 for a start tag with no end tag, in every spelling of the start tag.
/// ~keep The first pass does not read a start tag in a CDATA section. This pass does, and
/// ~keep removes the element when it has an end tag. The start tag and the end tag are read
/// ~keep with `PageReads`, the rule of the first pass, and no search is made twice.
fn strip_raw_text_tag_reopen<C: FnMut(PassRead)>(
    input: &str,
    idx: usize,
    last: usize,
    reads: &mut PageReads<C>,
    output: &mut Option<String>,
) -> Option<usize> {
    let bytes = input.as_bytes();
    let (slot, open_end) = reads.raw_text_start_tag_end(bytes, idx)?;
    let name = removed_element_name(slot)?;
    if name == "script" && is_json_ld_script_open_tag(&input[idx..open_end]) {
        return None;
    }
    let remove_end = reads
        .raw_text_end_tag(bytes, open_end, slot)
        .map_or(open_end, |(_, end)| end);
    let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
    out.push_str(&input[last..open_end]);
    out.push_str("</");
    out.push_str(name);
    out.push('>');
    Some(remove_end)
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
    second_pass(input, |_| {})
}

/// The second pass over one page, with the reads that a test counts.
pub(super) fn second_pass<C: FnMut(PassRead)>(input: &str, count: C) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return Cow::Borrowed(input);
    }

    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;
    let mut svg_depth = 0usize;
    let mut reads = PageReads::new(count);
    let last_gt = memchr::memrchr(b'>', bytes).unwrap_or(0);

    while idx < len {
        if bytes[idx] != b'<' {
            idx += 1;
            continue;
        }
        // ~keep No tag can terminate beyond the final `>`; do not rescan unfinished suffixes (#773).
        if last_gt <= idx && looks_like_tag_start(bytes, idx, len) {
            break;
        }
        let rewritten = collapse_empty_comment(input, bytes, idx, last, &mut output)
            .or_else(|| normalize_self_closing_void(input, bytes, idx, last, &mut output));
        if let Some(new_pos) = rewritten {
            last = new_pos;
            idx = new_pos;
            continue;
        }

        if let Some(new_idx) = track_svg_tag(bytes, idx, &mut svg_depth) {
            idx = new_idx;
            continue;
        }

        let rewritten = if svg_depth == 0 {
            normalize_void_tag_case(input, idx, last, last_gt, &mut reads, &mut output)
                .or_else(|| strip_raw_text_tag_reopen(input, idx, last, &mut reads, &mut output))
                .or_else(|| strip_doctype(input, bytes, idx, len, last, &mut output))
        } else {
            None
        };
        if let Some(new_pos) = rewritten {
            last = new_pos;
            idx = new_pos;
            continue;
        }

        if !looks_like_tag_start(bytes, idx, len) {
            let out = output.get_or_insert_with(|| String::with_capacity(input.len() + 4));
            out.push_str(&input[last..idx]);
            out.push_str("&lt;");
            idx += 1;
            last = idx;
            continue;
        }

        // ~keep A comment and a tag are read whole, as the first pass reads them: a `<` in a
        // ~keep comment or in an attribute value starts nothing. No tag ends after the last `>`
        // ~keep of the page, and no search for the end of a tag starts there.
        let whole = if svg_depth == 0 && idx < last_gt {
            reads.comment_end(bytes, idx).or_else(|| reads.tag_end(bytes, idx))
        } else {
            None
        };
        idx = whole.unwrap_or(idx + 1);
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

/// Check if bytes match an end tag pattern.
pub fn matches_end_tag_start(bytes: &[u8], start: usize, tag: &[u8]) -> bool {
    if start >= bytes.len() || bytes[start] != b'/' {
        return false;
    }
    matches_tag_start(bytes, start + 1, tag)
}

/// ~keep CDATA outside SVG/MathML is an HTML bogus comment, while foreign content owns its body.
pub fn strip_html_cdata(input: &str) -> Cow<'_, str> {
    if !input.contains("<![CDATA[") {
        return Cow::Borrowed(input);
    }
    let bytes = input.as_bytes();
    let mut cursor = 0;
    let mut last = 0;
    let mut foreign_depth = 0usize;
    let mut output = None;
    while let Some(offset) = memchr::memchr(b'<', &bytes[cursor..]) {
        cursor += offset;
        if bytes[cursor..].starts_with(b"<![CDATA[") {
            let end = cdata_end(bytes, cursor, foreign_depth == 0);
            if foreign_depth == 0 {
                let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                out.push_str(&input[last..cursor]);
                last = end;
            }
            cursor = end;
            continue;
        }
        if let Some(end) =
            super::raw_text::skip_opaque_region(bytes, cursor).or_else(|| skip_cdata_text_element(bytes, cursor))
        {
            cursor = end;
            continue;
        }
        let name_start = cursor + 1 + usize::from(bytes.get(cursor + 1) == Some(&b'/'));
        if !bytes.get(name_start).is_some_and(u8::is_ascii_alphabetic) {
            cursor += 1;
            continue;
        }
        let Some(end) = find_tag_end(bytes, cursor + 1) else {
            break;
        };
        update_cdata_foreign_depth(bytes, cursor, end, &mut foreign_depth);
        cursor = end;
    }
    match output {
        Some(mut out) => {
            out.push_str(&input[last..]);
            Cow::Owned(out)
        }
        None => Cow::Borrowed(input),
    }
}

fn skip_cdata_text_element(bytes: &[u8], cursor: usize) -> Option<usize> {
    for tag in [
        b"textarea".as_slice(),
        b"title",
        b"xmp",
        b"iframe",
        b"noembed",
        b"noframes",
        b"noscript",
    ] {
        if matches_tag_start(bytes, cursor + 1, tag) {
            let open_end = find_tag_end(bytes, cursor + 1 + tag.len())?;
            return Some(super::raw_text::find_closing_tag_bytes(bytes, open_end, tag).unwrap_or(bytes.len()));
        }
    }
    matches_tag_start(bytes, cursor + 1, b"plaintext").then_some(bytes.len())
}

fn update_cdata_foreign_depth(bytes: &[u8], cursor: usize, end: usize, depth: &mut usize) {
    for tag in [b"svg".as_slice(), b"math".as_slice()] {
        if matches_tag_start(bytes, cursor + 1, tag) && !bytes[cursor..end].ends_with(b"/>") {
            *depth += 1;
        } else if matches_end_tag_start(bytes, cursor + 1, tag) {
            *depth = depth.saturating_sub(1);
        }
    }
}

fn cdata_end(bytes: &[u8], cursor: usize, in_html: bool) -> usize {
    if in_html {
        // ~keep HTML bogus comments end at the first `>`; only foreign CDATA uses `]]>`.
        memchr::memchr(b'>', &bytes[cursor + 2..]).map_or(bytes.len(), |offset| cursor + 2 + offset + 1)
    } else {
        super::raw_text::skip_opaque_region(bytes, cursor).unwrap_or(bytes.len())
    }
}
