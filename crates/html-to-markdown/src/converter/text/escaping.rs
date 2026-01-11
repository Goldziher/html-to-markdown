//! Escaping utilities for Markdown special characters.
//!
//! This module provides functions for escaping characters that have special meaning
//! in Markdown, including brackets in link labels and angle brackets.

use std::borrow::Cow;

/// Escape special characters in link labels.
///
/// Markdown link labels can contain brackets, which need careful escaping to avoid
/// being interpreted as nested links. This function escapes unescaped closing brackets
/// that would break the link syntax.
///
/// # Examples
///
/// ```text
/// "Simple text" → "Simple text"
/// "Text [with brackets]" → "Text [with brackets\]"
/// "Text \\[escaped\\]" → "Text \\[escaped\\]"
/// ```
pub fn escape_link_label(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(text.len());
    let mut backslash_count = 0usize;
    let mut bracket_depth = 0usize;

    for ch in text.chars() {
        if ch == '\\' {
            result.push('\\');
            backslash_count += 1;
            continue;
        }

        let is_escaped = backslash_count % 2 == 1;
        backslash_count = 0;

        match ch {
            '[' if !is_escaped => {
                bracket_depth = bracket_depth.saturating_add(1);
                result.push('[');
            }
            ']' if !is_escaped => {
                if bracket_depth == 0 {
                    result.push('\\');
                } else {
                    bracket_depth -= 1;
                }
                result.push(']');
            }
            _ => result.push(ch),
        }
    }

    result
}

/// Escape malformed angle brackets in markdown output.
///
/// Markdown uses `<...>` for automatic links. Angle brackets that don't form valid
/// link syntax should be escaped as `&lt;` to prevent parser confusion.
///
/// A valid tag must have:
/// - `<!` followed by `-` or alphabetic character (for comments/declarations)
/// - `</` followed by alphabetic character (for closing tags)
/// - `<?` (for processing instructions)
/// - `<` followed by alphabetic character (for opening tags)
///
/// # Examples
///
/// ```text
/// "<valid@example.com>" → "<valid@example.com>" (unchanged, valid link)
/// "< invalid" → "&lt; invalid" (escaped, invalid)
/// "Text <2 more" → "Text &lt;2 more" (escaped, invalid)
/// ```
pub fn escape_malformed_angle_brackets(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;

    while idx < len {
        if bytes[idx] == b'<' {
            if idx + 1 < len {
                let next = bytes[idx + 1];

                let is_valid_tag = match next {
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
                };

                if !is_valid_tag {
                    let out = output.get_or_insert_with(|| String::with_capacity(input.len() + 4));
                    out.push_str(&input[last..idx]);
                    out.push_str("&lt;");
                    last = idx + 1;
                }
            } else {
                let out = output.get_or_insert_with(|| String::with_capacity(input.len() + 4));
                out.push_str(&input[last..idx]);
                out.push_str("&lt;");
                last = idx + 1;
            }
        }
        idx += 1;
    }

    if let Some(mut out) = output {
        if last < input.len() {
            out.push_str(&input[last..]);
        }
        Cow::Owned(out)
    } else {
        Cow::Borrowed(input)
    }
}
