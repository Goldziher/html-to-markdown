//! Text processing utilities for Markdown conversion.

use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;

/// Regex for escaping miscellaneous characters
static ESCAPE_MISC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\\&<`\[\]>~#=+|\-])").unwrap());

/// Regex for escaping numbered lists
static ESCAPE_NUMBERED_LIST_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([0-9])([.)])").unwrap());

/// Regex for escaping ASCII punctuation (CommonMark spec example 12)
/// Matches: ! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~
static ESCAPE_ASCII_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([!\x22#$%&\x27()*+,\-./:;<=>?@\[\\\]^_`{|}~])").unwrap());

/// Escape Markdown special characters in text.
///
/// # Arguments
///
/// * `text` - Text to escape
/// * `escape_misc` - Escape miscellaneous characters (\ & < ` [ > ~ # = + | -)
/// * `escape_asterisks` - Escape asterisks (*)
/// * `escape_underscores` - Escape underscores (_)
/// * `escape_ascii` - Escape all ASCII punctuation (for CommonMark spec compliance)
///
/// # Returns
///
/// Escaped text
pub fn escape(
    text: &str,
    escape_misc: bool,
    escape_asterisks: bool,
    escape_underscores: bool,
    escape_ascii: bool,
) -> String {
    if text.is_empty() {
        return String::new();
    }

    if !escape_misc && !escape_asterisks && !escape_underscores && !escape_ascii {
        return text.to_string();
    }

    if escape_ascii
        && !text.as_bytes().iter().any(|b| {
            matches!(
                b,
                b'!' | b'"'
                    | b'#'
                    | b'$'
                    | b'%'
                    | b'&'
                    | b'\''
                    | b'('
                    | b')'
                    | b'*'
                    | b'+'
                    | b','
                    | b'-'
                    | b'.'
                    | b'/'
                    | b':'
                    | b';'
                    | b'<'
                    | b'='
                    | b'>'
                    | b'?'
                    | b'@'
                    | b'['
                    | b'\\'
                    | b']'
                    | b'^'
                    | b'_'
                    | b'`'
                    | b'{'
                    | b'|'
                    | b'}'
                    | b'~'
            )
        })
    {
        return text.to_string();
    }

    if !escape_ascii && escape_misc && !escape_asterisks && !escape_underscores {
        let needs_misc = text.as_bytes().iter().any(|b| {
            matches!(
                b,
                b'\\' | b'&' | b'<' | b'`' | b'[' | b']' | b'>' | b'~' | b'#' | b'=' | b'+' | b'|' | b'-'
            )
        });
        let needs_numbered = text.as_bytes().iter().any(|b| matches!(b, b'.' | b')'));
        if !needs_misc && !needs_numbered {
            return text.to_string();
        }
    }

    let mut result = text.to_string();

    if escape_ascii {
        result = ESCAPE_ASCII_RE.replace_all(&result, r"\$1").to_string();
        return result;
    }

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
/// Prefix/suffix are " " if original text had leading/trailing whitespace.
/// However, suffix is "" if the trailing whitespace is only newlines (not spaces/tabs).
/// This prevents trailing newlines from becoming trailing spaces in the output.
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

    let suffix = if text.ends_with("\n\n") || text.ends_with("\r\n\r\n") {
        "\n\n"
    } else if text.ends_with([' ', '\t']) {
        " "
    } else {
        ""
    };

    let trimmed = if suffix == "\n\n" {
        text.trim_end_matches("\n\n").trim_end_matches("\r\n\r\n").trim()
    } else {
        text.trim()
    };

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

pub fn normalize_whitespace_cow(text: &str) -> Cow<'_, str> {
    let mut prev_was_space = false;

    for ch in text.chars() {
        let is_space = ch == ' ' || ch == '\t' || is_unicode_space(ch);
        if is_space {
            if prev_was_space || ch != ' ' {
                return Cow::Owned(normalize_whitespace(text));
            }
            prev_was_space = true;
        } else {
            prev_was_space = false;
        }
    }

    Cow::Borrowed(text)
}

/// Decode common HTML entities.
///
/// Decodes the most common HTML entities to their character equivalents:
/// - `&quot;` → `"`
/// - `&apos;` → `'`
/// - `&lt;` → `<`
/// - `&gt;` → `>`
/// - `&amp;` → `&` (must be last to avoid double-decoding)
///
/// # Arguments
///
/// * `text` - Text containing HTML entities
///
/// # Returns
///
/// Text with entities decoded
pub fn decode_html_entities(text: &str) -> String {
    html_escape::decode_html_entities(text).into_owned()
}

pub fn decode_html_entities_cow(text: &str) -> Cow<'_, str> {
    if !text.contains('&') {
        return Cow::Borrowed(text);
    }

    html_escape::decode_html_entities(text)
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
        assert_eq!(escape("foo & bar", true, false, false, false), r"foo \& bar");
        assert_eq!(escape("foo [bar]", true, false, false, false), r"foo \[bar\]");
        assert_eq!(escape("1. Item", true, false, false, false), r"1\. Item");
        assert_eq!(escape("1) Item", true, false, false, false), r"1\) Item");
    }

    #[test]
    fn test_escape_asterisks() {
        assert_eq!(escape("foo * bar", false, true, false, false), r"foo \* bar");
        assert_eq!(escape("**bold**", false, true, false, false), r"\*\*bold\*\*");
    }

    #[test]
    fn test_escape_underscores() {
        assert_eq!(escape("foo_bar", false, false, true, false), r"foo\_bar");
        assert_eq!(escape("__bold__", false, false, true, false), r"\_\_bold\_\_");
    }

    #[test]
    fn test_escape_ascii() {
        assert_eq!(escape(r##"!"#$%&"##, false, false, false, true), r##"\!\"\#\$\%\&"##);
        assert_eq!(escape("*+,-./", false, false, false, true), r"\*\+\,\-\.\/");
        assert_eq!(escape("<=>?@", false, false, false, true), r"\<\=\>\?\@");
        assert_eq!(escape(r"[\]^_`", false, false, false, true), r"\[\\\]\^\_\`");
        assert_eq!(escape("{|}~", false, false, false, true), r"\{\|\}\~");
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
