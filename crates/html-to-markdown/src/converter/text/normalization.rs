//! Text normalization utilities for HTML to Markdown conversion.
//!
//! This module provides functions for normalizing text content extracted from HTML,
//! including chomping whitespace, normalizing heading text, and handling trailing whitespace.

use std::borrow::Cow;

/// Chomp whitespace from inline element content, preserving line breaks.
///
/// Returns (prefix, suffix, trimmed_text) where:
/// - prefix: leading whitespace (space or tab)
/// - suffix: trailing whitespace (including soft breaks like "  \n" or "\\\n")
/// - trimmed_text: the trimmed content
///
/// # Examples
///
/// ```text
/// " text  \n" → (" ", "  \n", "text")
/// "  text  " → (" ", " ", "text")
/// "text" → ("", "", "text")
/// ```
pub fn chomp_inline(text: &str) -> (&str, &str, &str) {
    if text.is_empty() {
        return ("", "", "");
    }

    let prefix = if text.starts_with(&[' ', '\t'][..]) { " " } else { "" };

    let has_trailing_linebreak = text.ends_with("  \n") || text.ends_with("\\\n");

    let suffix = if has_trailing_linebreak {
        if text.ends_with("  \n") { "  \n" } else { "\\\n" }
    } else if text.ends_with(&[' ', '\t'][..]) {
        " "
    } else {
        ""
    };

    let trimmed = if has_trailing_linebreak {
        text.strip_suffix("  \n").map_or_else(
            || text.strip_suffix("\\\n").map_or_else(|| text.trim(), |s| s.trim()),
            |s| s.trim(),
        )
    } else {
        text.trim()
    };

    (prefix, suffix, trimmed)
}

/// Remove trailing spaces and tabs from output string.
///
/// This is used before adding block separators or newlines to ensure
/// clean Markdown output without spurious whitespace.
pub fn trim_trailing_whitespace(output: &mut String) {
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
}

/// Remove trailing spaces/tabs from every line while preserving newlines.
pub fn trim_line_end_whitespace(output: &mut String) {
    if output.is_empty() {
        return;
    }

    let mut cleaned = String::with_capacity(output.len());
    for (idx, line) in output.split('\n').enumerate() {
        if idx > 0 {
            cleaned.push('\n');
        }

        let has_soft_break = line.ends_with("  ");
        let trimmed = line.trim_end_matches([' ', '\t']);

        cleaned.push_str(trimmed);
        if has_soft_break {
            cleaned.push_str("  ");
        }
    }

    cleaned.push('\n');
    *output = cleaned;
}

/// Truncate a string at a valid UTF-8 boundary.
pub fn truncate_at_char_boundary(value: &mut String, max_len: usize) {
    if value.len() <= max_len {
        return;
    }

    let mut new_len = max_len.min(value.len());
    while new_len > 0 && !value.is_char_boundary(new_len) {
        new_len -= 1;
    }
    value.truncate(new_len);
}

/// Normalize heading text by replacing newlines and extra whitespace.
///
/// Heading text should be on a single line in Markdown. This function collapses
/// any newlines and multiple spaces into single spaces.
///
/// # Examples
///
/// ```text
/// "Hello\nWorld" → "Hello World"
/// "Text  with   spaces" → "Text  with   spaces" (unchanged if no newlines)
/// ```
pub fn normalize_heading_text(text: &str) -> Cow<'_, str> {
    if !text.contains('\n') && !text.contains('\r') {
        return Cow::Borrowed(text);
    }

    let mut normalized = String::with_capacity(text.len());
    let mut pending_space = false;

    for ch in text.chars() {
        match ch {
            '\n' | '\r' => {
                if !normalized.is_empty() {
                    pending_space = true;
                }
            }
            ' ' | '\t' if pending_space => {}
            _ => {
                if pending_space {
                    if !normalized.ends_with(' ') {
                        normalized.push(' ');
                    }
                    pending_space = false;
                }
                normalized.push(ch);
            }
        }
    }

    Cow::Owned(normalized)
}
