//! HTML preprocessing and normalization.
//!
//! Functions for preprocessing HTML before conversion, including script/style stripping,
//! tag repair, and malformed HTML handling.

use std::borrow::Cow;
use std::str;

/// Strip script and style tags and their content from HTML.
pub(crate) fn strip_script_and_style_tags(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();

    if len == 0 {
        return Cow::Borrowed(input);
    }

    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;
    let mut svg_depth = 0usize;

    // Fast-path: check if there are any < characters at all
    if !bytes.contains(&b'<') {
        return Cow::Borrowed(input);
    }

    while idx < len {
        if bytes[idx] == b'<' && idx + 1 < len {
            if matches_tag_start(bytes, idx + 1, b"svg") {
                if let Some(open_end) = find_tag_end(bytes, idx + 1 + b"svg".len()) {
                    svg_depth += 1;
                    idx = open_end;
                    continue;
                }
            } else if matches_end_tag_start(bytes, idx + 1, b"svg") {
                if let Some(close_end) = find_tag_end(bytes, idx + 2 + b"svg".len()) {
                    if svg_depth > 0 {
                        svg_depth = svg_depth.saturating_sub(1);
                    }
                    idx = close_end;
                    continue;
                }
            }

            if svg_depth > 0 {
                idx += 1;
                continue;
            }

            // Check for </script or </style (closing tags first for safety)
            if bytes[idx + 1] == b'/' && idx + 2 < len {
                // Match </script>
                if idx + 9 <= len && eq_ascii_insensitive(&bytes[idx..idx + 9], b"</script>") {
                    idx += 9;
                    continue;
                }

                // Match </style>
                if idx + 8 <= len && eq_ascii_insensitive(&bytes[idx..idx + 8], b"</style>") {
                    idx += 8;
                    continue;
                }
            }

            // Check for <script or <style (opening tags)
            // Match <script (case insensitive)
            if idx + 7 < len && eq_ascii_insensitive(&bytes[idx..idx + 7], b"<script") {
                // Check if this is actually "<script" followed by whitespace, >, or attribute
                let after_tag = bytes[idx + 7];
                if after_tag == b'>'
                    || after_tag == b' '
                    || after_tag == b'\t'
                    || after_tag == b'\n'
                    || after_tag == b'\r'
                {
                    // Find the opening tag end
                    let mut tag_end = idx + 7;
                    while tag_end < len && bytes[tag_end] != b'>' {
                        tag_end += 1;
                    }

                    if tag_end < len {
                        tag_end += 1; // Include the '>'

                        // Check if this is a JSON-LD script tag
                        let tag_content = &input[idx..tag_end];
                        if !is_json_ld_script_open_tag(tag_content) {
                            // Find the closing </script> tag
                            let close_tag = find_closing_tag_bytes(bytes, tag_end, b"script");
                            if let Some(close_idx) = close_tag {
                                let out = output.get_or_insert_with(|| String::with_capacity(len));
                                out.push_str(&input[last..idx]);
                                if idx > 0
                                    && close_idx < len
                                    && !bytes[idx - 1].is_ascii_whitespace()
                                    && !bytes[close_idx].is_ascii_whitespace()
                                {
                                    out.push(' ');
                                }
                                last = close_idx;
                                idx = close_idx;
                                continue;
                            }
                        }
                    }
                }
            }
            // Match <style (case insensitive)
            else if idx + 6 < len && eq_ascii_insensitive(&bytes[idx..idx + 6], b"<style") {
                // Check if this is actually "<style" followed by whitespace, >, or attribute
                let after_tag = bytes[idx + 6];
                if after_tag == b'>'
                    || after_tag == b' '
                    || after_tag == b'\t'
                    || after_tag == b'\n'
                    || after_tag == b'\r'
                {
                    // Find the opening tag end
                    let mut tag_end = idx + 6;
                    while tag_end < len && bytes[tag_end] != b'>' {
                        tag_end += 1;
                    }

                    if tag_end < len {
                        tag_end += 1; // Include the '>'

                        // Find the closing </style> tag
                        let close_tag = find_closing_tag_bytes(bytes, tag_end, b"style");
                        if let Some(close_idx) = close_tag {
                            let out = output.get_or_insert_with(|| String::with_capacity(len));
                            out.push_str(&input[last..idx]);
                            if idx > 0
                                && close_idx < len
                                && !bytes[idx - 1].is_ascii_whitespace()
                                && !bytes[close_idx].is_ascii_whitespace()
                            {
                                out.push(' ');
                            }
                            last = close_idx;
                            idx = close_idx;
                            continue;
                        }
                    }
                }
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

/// Find the position of a closing tag in bytes.
/// Returns the position AFTER the closing tag (including the '>').
/// This is highly optimized for performance and uses a fast-path scan.
#[inline]
pub(crate) fn find_closing_tag_bytes(bytes: &[u8], start: usize, tag: &[u8]) -> Option<usize> {
    let len = bytes.len();
    let tag_len = tag.len();

    // Fast path: look for the closing tag pattern byte-by-byte
    // We use a simple byte scan to find '</' then validate the tag name
    let mut idx = start;

    // Limit search to prevent stack overflow on large files
    // Look for closing tag within reasonable bounds
    const MAX_SCAN: usize = 100_000_000; // 100MB limit per tag - prevents pathological cases

    while idx < len && (idx - start) < MAX_SCAN {
        // Optimization: skip forward to next '<' quickly
        if bytes[idx] != b'<' {
            idx += 1;
            continue;
        }

        // Check for </ pattern
        if idx + 2 < len && bytes[idx + 1] == b'/' {
            // Check if tag name matches
            if idx + 2 + tag_len <= len && eq_ascii_insensitive(&bytes[idx + 2..idx + 2 + tag_len], tag) {
                // Ensure it's followed by > or whitespace
                let after_tag = idx + 2 + tag_len;
                if after_tag < len && (bytes[after_tag] == b'>' || bytes[after_tag].is_ascii_whitespace()) {
                    // Find the >
                    let mut close_idx = after_tag;
                    while close_idx < len && bytes[close_idx] != b'>' {
                        close_idx += 1;
                    }
                    if close_idx < len {
                        return Some(close_idx + 1); // Include the '>'
                    }
                }
            }
        }

        idx += 1;
    }

    None
}

/// Compare bytes ignoring ASCII case.
#[inline]
pub(crate) fn eq_ascii_insensitive(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| x.eq_ignore_ascii_case(y))
}

/// Preprocess HTML to normalize tags and fix common issues.
pub(crate) fn preprocess_html(input: &str) -> Cow<'_, str> {
    const SELF_CLOSING: [(&[u8], &str); 3] = [(b"<br/>", "<br>"), (b"<hr/>", "<hr>"), (b"<img/>", "<img>")];
    const TAGS: [&[u8]; 2] = [b"script", b"style"];
    const SVG: &[u8] = b"svg";
    const DOCTYPE: &[u8] = b"doctype";
    const EMPTY_COMMENT: &[u8] = b"<!---->";

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
        if bytes[idx] == b'<' {
            if bytes[idx..].starts_with(EMPTY_COMMENT) {
                let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                out.push_str(&input[last..idx]);
                out.push_str("<!-- -->");
                idx += EMPTY_COMMENT.len();
                last = idx;
                continue;
            }

            let mut replaced = false;
            for (pattern, replacement) in &SELF_CLOSING {
                if bytes[idx..].starts_with(pattern) {
                    let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                    out.push_str(&input[last..idx]);
                    out.push_str(replacement);
                    idx += pattern.len();
                    last = idx;
                    replaced = true;
                    break;
                }
            }
            if replaced {
                continue;
            }

            if matches_tag_start(bytes, idx + 1, SVG) {
                if let Some(open_end) = find_tag_end(bytes, idx + 1 + SVG.len()) {
                    svg_depth += 1;
                    idx = open_end;
                    continue;
                }
            } else if matches_end_tag_start(bytes, idx + 1, SVG) {
                if let Some(close_end) = find_tag_end(bytes, idx + 2 + SVG.len()) {
                    if svg_depth > 0 {
                        svg_depth = svg_depth.saturating_sub(1);
                    }
                    idx = close_end;
                    continue;
                }
            }

            if svg_depth == 0 {
                let mut handled = false;
                for tag in TAGS {
                    if matches_tag_start(bytes, idx + 1, tag) {
                        if let Some(open_end) = find_tag_end(bytes, idx + 1 + tag.len()) {
                            if tag == b"script" && is_json_ld_script_open_tag(&input[idx..open_end]) {
                                continue;
                            }
                            let remove_end = find_closing_tag(bytes, open_end, tag).unwrap_or(len);
                            let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                            out.push_str(&input[last..idx]);
                            out.push_str(&input[idx..open_end]);
                            out.push_str("</");
                            out.push_str(str::from_utf8(tag).unwrap());
                            out.push('>');

                            last = remove_end;
                            idx = remove_end;
                            handled = true;
                        }
                    }

                    if handled {
                        break;
                    }
                }

                if handled {
                    continue;
                }

                if idx + 2 < len && bytes[idx + 1] == b'!' {
                    let mut cursor = idx + 2;
                    while cursor < len && bytes[cursor].is_ascii_whitespace() {
                        cursor += 1;
                    }

                    if cursor + DOCTYPE.len() <= len
                        && bytes[cursor..cursor + DOCTYPE.len()].eq_ignore_ascii_case(DOCTYPE)
                    {
                        if let Some(end) = find_tag_end(bytes, cursor + DOCTYPE.len()) {
                            let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                            out.push_str(&input[last..idx]);
                            last = end;
                            idx = end;
                            continue;
                        }
                    }
                }
            }

            let is_valid_tag = if idx + 1 < len {
                match bytes[idx + 1] {
                    b'!' => {
                        idx + 2 < len
                            && (bytes[idx + 2] == b'-'
                                || bytes[idx + 2].is_ascii_alphabetic()
                                || bytes[idx + 2].is_ascii_uppercase())
                    }
                    b'/' => {
                        idx + 2 < len && (bytes[idx + 2].is_ascii_alphabetic() || bytes[idx + 2].is_ascii_uppercase())
                    }
                    b'?' => true,
                    c if c.is_ascii_alphabetic() || c.is_ascii_uppercase() => true,
                    _ => false,
                }
            } else {
                false
            };

            if !is_valid_tag {
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

/// Check if a script tag is a JSON-LD script.
pub(crate) fn is_json_ld_script_open_tag(tag: &str) -> bool {
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
                    let mut end = start;
                    while end < bytes.len() && !bytes[end].is_ascii_whitespace() && bytes[end] != b'>' {
                        end += 1;
                    }
                    (start, end)
                }
            };

            let value = &tag[value_start..value_end];
            let media_type = value.split(';').next().unwrap_or(value).trim();
            return eq_ascii_case_insensitive(media_type.as_bytes(), b"application/ld+json");
        }
        idx += 1;
    }
    false
}

/// Case-insensitive byte comparison for ASCII.
#[inline]
pub(crate) fn eq_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    haystack
        .iter()
        .zip(needle.iter())
        .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

/// Check if bytes match a tag start pattern.
pub(crate) fn matches_tag_start(bytes: &[u8], mut start: usize, tag: &[u8]) -> bool {
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
pub(crate) fn find_tag_end(bytes: &[u8], mut idx: usize) -> Option<usize> {
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

/// Find the closing tag for a given tag name.
pub(crate) fn find_closing_tag(bytes: &[u8], mut idx: usize, tag: &[u8]) -> Option<usize> {
    let len = bytes.len();
    let mut depth = 1usize;

    while idx < len {
        if bytes[idx] == b'<' {
            if matches_tag_start(bytes, idx + 1, tag) {
                if let Some(next) = find_tag_end(bytes, idx + 1 + tag.len()) {
                    depth += 1;
                    idx = next;
                    continue;
                }
            } else if matches_end_tag_start(bytes, idx + 1, tag) {
                if let Some(close) = find_tag_end(bytes, idx + 2 + tag.len()) {
                    depth -= 1;
                    if depth == 0 {
                        return Some(close);
                    }
                    idx = close;
                    continue;
                }
            }
        }

        idx += 1;
    }

    None
}

/// Check if bytes match an end tag pattern.
pub(crate) fn matches_end_tag_start(bytes: &[u8], start: usize, tag: &[u8]) -> bool {
    if start >= bytes.len() || bytes[start] != b'/' {
        return false;
    }
    matches_tag_start(bytes, start + 1, tag)
}
