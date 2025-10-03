//! Text processing utilities for Markdown conversion.

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for escaping miscellaneous characters
static ESCAPE_MISC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\\&<`\[\]>~#=+|\-])").unwrap());

/// Regex for escaping numbered lists
static ESCAPE_NUMBERED_LIST_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([0-9])([.)])").unwrap());

/// Escape Markdown special characters in text.
///
/// # Arguments
///
/// * `text` - Text to escape
/// * `escape_misc` - Escape miscellaneous characters (\ & < ` [ > ~ # = + | -)
/// * `escape_asterisks` - Escape asterisks (*)
/// * `escape_underscores` - Escape underscores (_)
///
/// # Returns
///
/// Escaped text
pub fn escape(text: &str, escape_misc: bool, escape_asterisks: bool, escape_underscores: bool) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut result = text.to_string();

    if escape_misc {
        result = ESCAPE_MISC_RE.replace_all(&result, r"\$1").to_string();

        result = ESCAPE_NUMBERED_LIST_RE.replace_all(&result, r"$1\$2").to_string();
    }

    if escape_asterisks {
        result = result.replace('*', r"\*");
    }

    if escape_underscores {
        result = result.replace('_', r"\_");
    }

    result
}

/// Extract boundary whitespace from text (chomp).
///
/// Returns (prefix, suffix, trimmed_text) tuple.
/// Prefix/suffix are " " if original text had leading/trailing whitespace (including newlines).
/// The trimmed text has all leading/trailing whitespace removed.
pub fn chomp(text: &str) -> (&str, &str, &str) {
    if text.is_empty() {
        return ("", "", "");
    }

    let prefix = if text.starts_with(|c: char| c.is_whitespace()) {
        " "
    } else {
        ""
    };

    let suffix = if text.ends_with(|c: char| c.is_whitespace()) {
        " "
    } else {
        ""
    };

    let trimmed = text.trim();

    (prefix, suffix, trimmed)
}

/// Normalize whitespace by collapsing consecutive spaces and tabs.
///
/// Multiple spaces and tabs are replaced with a single space.
/// Newlines are preserved.
/// Unicode spaces are normalized to ASCII spaces.
///
/// # Arguments
///
/// * `text` - The text to normalize
///
/// # Returns
///
/// Normalized text with collapsed spaces/tabs but preserved newlines
pub fn normalize_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_was_space = false;

    for ch in text.chars() {
        let is_space = ch == ' ' || ch == '\t' || is_unicode_space(ch);

        if is_space {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }

    result
}

/// Check if a character is a unicode space character.
///
/// Includes: non-breaking space, various width spaces, etc.
fn is_unicode_space(ch: char) -> bool {
    matches!(
        ch,
        '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            | '\u{2001}'
            | '\u{2002}'
            | '\u{2003}'
            | '\u{2004}'
            | '\u{2005}'
            | '\u{2006}'
            | '\u{2007}'
            | '\u{2008}'
            | '\u{2009}'
            | '\u{200A}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

/// Underline text with a character.
pub fn underline(text: &str, pad_char: char) -> String {
    let text = text.trim_end();
    if text.is_empty() {
        return String::new();
    }
    format!("{}\n{}\n\n", text, pad_char.to_string().repeat(text.len()))
}

/// Indent text with a string prefix.
pub fn indent(text: &str, level: usize, indent_str: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let prefix = indent_str.repeat(level);
    text.lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{}{}", prefix, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_misc() {
        assert_eq!(escape("foo & bar", true, false, false), r"foo \& bar");
        assert_eq!(escape("foo [bar]", true, false, false), r"foo \[bar\]");
        assert_eq!(escape("1. Item", true, false, false), r"1\. Item");
        assert_eq!(escape("1) Item", true, false, false), r"1\) Item");
    }

    #[test]
    fn test_escape_asterisks() {
        assert_eq!(escape("foo * bar", false, true, false), r"foo \* bar");
        assert_eq!(escape("**bold**", false, true, false), r"\*\*bold\*\*");
    }

    #[test]
    fn test_escape_underscores() {
        assert_eq!(escape("foo_bar", false, false, true), r"foo\_bar");
        assert_eq!(escape("__bold__", false, false, true), r"\_\_bold\_\_");
    }

    #[test]
    fn test_chomp() {
        assert_eq!(chomp("  text  "), (" ", " ", "text"));
        assert_eq!(chomp("text"), ("", "", "text"));
        assert_eq!(chomp(" text"), (" ", "", "text"));
        assert_eq!(chomp("text "), ("", " ", "text"));
        assert_eq!(chomp(""), ("", "", ""));
    }

    #[test]
    fn test_underline() {
        assert_eq!(underline("Title", '='), "Title\n=====\n\n");
        assert_eq!(underline("Subtitle", '-'), "Subtitle\n--------\n\n");
        assert_eq!(underline("", '='), "");
    }

    #[test]
    fn test_indent() {
        assert_eq!(indent("line1\nline2", 1, "\t"), "\tline1\n\tline2");
        assert_eq!(indent("text", 2, "  "), "    text");
        assert_eq!(indent("", 1, "\t"), "");
    }
}
