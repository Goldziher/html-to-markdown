//! Text wrapping functionality for Markdown output.
//!
//! This module provides text wrapping capabilities similar to Python's textwrap.fill(),
//! specifically designed to work with Markdown content while preserving formatting.

use crate::options::ConversionOptions;

/// Wrap text at specified width while preserving Markdown formatting.
///
/// This function wraps paragraphs of text at the specified width, but:
/// - Does not break long words
/// - Does not break on hyphens
/// - Preserves Markdown formatting (links, bold, etc.)
/// - Only wraps paragraph content, not headers, lists, code blocks, etc.
pub fn wrap_markdown(markdown: &str, options: &ConversionOptions) -> String {
    if !options.wrap {
        return markdown.to_string();
    }

    let mut result = String::with_capacity(markdown.len());
    let mut in_code_block = false;
    let mut in_paragraph = false;
    let mut paragraph_buffer = String::new();
    let mut in_blockquote_paragraph = false;
    let mut blockquote_prefix = String::new();
    let mut blockquote_buffer = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        let is_code_fence = trimmed.starts_with("```");
        let is_indented_code = line.starts_with("    ")
            && !is_list_like(trimmed)
            && !is_numbered_list(trimmed)
            && !is_heading(trimmed)
            && !trimmed.starts_with('>')
            && !trimmed.starts_with('|');

        if is_code_fence || is_indented_code {
            if in_paragraph && !paragraph_buffer.is_empty() {
                result.push_str(&wrap_line(&paragraph_buffer, options.wrap_width));
                result.push_str("\n\n");
                paragraph_buffer.clear();
                in_paragraph = false;
            }

            if is_code_fence {
                in_code_block = !in_code_block;
            }
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_code_block {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if let Some((prefix, content)) = parse_blockquote_line(line) {
            if in_paragraph && !paragraph_buffer.is_empty() {
                result.push_str(&wrap_line(&paragraph_buffer, options.wrap_width));
                result.push_str("\n\n");
                paragraph_buffer.clear();
                in_paragraph = false;
            }

            let mut normalized_prefix = prefix;
            if !normalized_prefix.ends_with(' ') {
                normalized_prefix.push(' ');
            }

            if content.is_empty() {
                if in_blockquote_paragraph && !blockquote_buffer.is_empty() {
                    result.push_str(&wrap_blockquote_paragraph(
                        &blockquote_prefix,
                        &blockquote_buffer,
                        options.wrap_width,
                    ));
                    result.push('\n');
                    blockquote_buffer.clear();
                    in_blockquote_paragraph = false;
                }
                result.push_str(normalized_prefix.trim_end());
                result.push('\n');
                continue;
            }

            if in_blockquote_paragraph && normalized_prefix != blockquote_prefix {
                result.push_str(&wrap_blockquote_paragraph(
                    &blockquote_prefix,
                    &blockquote_buffer,
                    options.wrap_width,
                ));
                result.push('\n');
                blockquote_buffer.clear();
                in_blockquote_paragraph = false;
            }

            if !in_blockquote_paragraph {
                blockquote_prefix = normalized_prefix;
                blockquote_buffer.push_str(&content);
                in_blockquote_paragraph = true;
            } else {
                blockquote_buffer.push(' ');
                blockquote_buffer.push_str(&content);
            }
            continue;
        } else if in_blockquote_paragraph && !blockquote_buffer.is_empty() {
            result.push_str(&wrap_blockquote_paragraph(
                &blockquote_prefix,
                &blockquote_buffer,
                options.wrap_width,
            ));
            result.push('\n');
            blockquote_buffer.clear();
            in_blockquote_paragraph = false;
        }

        if let Some((indent, marker, content)) = parse_list_item(line) {
            if in_paragraph && !paragraph_buffer.is_empty() {
                result.push_str(&wrap_line(&paragraph_buffer, options.wrap_width));
                result.push_str("\n\n");
                paragraph_buffer.clear();
                in_paragraph = false;
            }

            result.push_str(&wrap_list_item(&indent, &marker, &content, options.wrap_width));
            continue;
        }

        let is_structural =
            is_heading(trimmed) || trimmed.starts_with('>') || trimmed.starts_with('|') || trimmed.starts_with('=');

        if is_structural {
            if in_paragraph && !paragraph_buffer.is_empty() {
                result.push_str(&wrap_line(&paragraph_buffer, options.wrap_width));
                result.push_str("\n\n");
                paragraph_buffer.clear();
                in_paragraph = false;
            }

            result.push_str(line);
            result.push('\n');
            continue;
        }

        if line.trim().is_empty() {
            if in_paragraph && !paragraph_buffer.is_empty() {
                result.push_str(&wrap_line(&paragraph_buffer, options.wrap_width));
                result.push_str("\n\n");
                paragraph_buffer.clear();
                in_paragraph = false;
            } else if !in_paragraph {
                result.push('\n');
            }
            continue;
        }

        if in_paragraph {
            paragraph_buffer.push(' ');
        }
        paragraph_buffer.push_str(line.trim());
        in_paragraph = true;
    }

    if in_blockquote_paragraph && !blockquote_buffer.is_empty() {
        result.push_str(&wrap_blockquote_paragraph(
            &blockquote_prefix,
            &blockquote_buffer,
            options.wrap_width,
        ));
        result.push('\n');
    }

    if in_paragraph && !paragraph_buffer.is_empty() {
        result.push_str(&wrap_line(&paragraph_buffer, options.wrap_width));
        result.push_str("\n\n");
    }

    result
}

fn parse_blockquote_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('>') {
        return None;
    }

    let indent_len = line.len() - trimmed.len();
    let bytes = line.as_bytes();
    let mut i = indent_len;

    while i < bytes.len() {
        if bytes[i] != b'>' {
            break;
        }
        i += 1;
        if i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        while i + 1 < bytes.len() && bytes[i] == b' ' && bytes[i + 1] == b'>' {
            i += 1;
        }
    }

    let prefix = line[..i].to_string();
    let content = line[i..].trim().to_string();
    Some((prefix, content))
}

fn wrap_blockquote_paragraph(prefix: &str, content: &str, width: usize) -> String {
    let prefix_len = prefix.len();
    let inner_width = if width > prefix_len { width - prefix_len } else { 1 };

    let wrapped = wrap_line(content, inner_width);
    let mut out = String::new();
    for (idx, part) in wrapped.split('\n').enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push_str(prefix);
        out.push_str(part);
    }
    out
}

fn is_list_like(trimmed: &str) -> bool {
    matches!(trimmed.chars().next(), Some('-' | '*' | '+'))
}

fn is_numbered_list(trimmed: &str) -> bool {
    let token = trimmed.split_whitespace().next().unwrap_or("");
    if token.is_empty() || !(token.ends_with('.') || token.ends_with(')')) {
        return false;
    }

    let digits = token.trim_end_matches(['.', ')']);
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

fn is_heading(trimmed: &str) -> bool {
    trimmed.starts_with('#')
}

/// Parse a list item into its components: (indent, marker, content)
///
/// Returns Some((indent, marker, content)) if the line is a valid list item,
/// None otherwise.
///
/// Examples:
/// - "- text" -> ("", "- ", "text")
/// - "  - text" -> ("  ", "- ", "text")
/// - "1. text" -> ("", "1. ", "text")
/// - "  42) text" -> ("  ", "42) ", "text")
fn parse_list_item(line: &str) -> Option<(String, String, String)> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];

    if let Some(rest) = trimmed.strip_prefix('-') {
        if rest.starts_with(' ') || rest.is_empty() {
            return Some((indent.to_string(), "- ".to_string(), rest.trim_start().to_string()));
        }
    }
    if let Some(rest) = trimmed.strip_prefix('*') {
        if rest.starts_with(' ') || rest.is_empty() {
            return Some((indent.to_string(), "* ".to_string(), rest.trim_start().to_string()));
        }
    }
    if let Some(rest) = trimmed.strip_prefix('+') {
        if rest.starts_with(' ') || rest.is_empty() {
            return Some((indent.to_string(), "+ ".to_string(), rest.trim_start().to_string()));
        }
    }

    let first_token = trimmed.split_whitespace().next()?;
    if first_token.ends_with('.') || first_token.ends_with(')') {
        let digits = first_token.trim_end_matches(['.', ')']);
        if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
            let marker_len = first_token.len();
            let rest = trimmed[marker_len..].trim_start();
            return Some((
                indent.to_string(),
                trimmed[..marker_len].to_string() + " ",
                rest.to_string(),
            ));
        }
    }

    None
}

/// Wrap a list item while preserving its structure.
///
/// The first line of output will be: `<indent><marker><content_start>`
/// Continuation lines will be: `<indent><spaces_matching_marker><content_continued>`
///
/// # Arguments
/// - `indent`: The leading whitespace (for nested lists)
/// - `marker`: The list marker (e.g., "- ", "1. ")
/// - `content`: The text content after the marker
/// - `width`: The maximum line width
fn wrap_list_item(indent: &str, marker: &str, content: &str, width: usize) -> String {
    if content.is_empty() {
        return format!("{}{}\n", indent, marker.trim_end());
    }

    if is_single_inline_link(content) {
        return format!("{}{}{}\n", indent, marker, content.trim());
    }

    let full_marker = format!("{}{}", indent, marker);
    let continuation_indent = format!("{}{}", indent, " ".repeat(marker.len()));

    let first_line_prefix_len = full_marker.len();
    let first_line_width = if width > first_line_prefix_len {
        width - first_line_prefix_len
    } else {
        width
    };

    let cont_line_prefix_len = continuation_indent.len();
    let cont_line_width = if width > cont_line_prefix_len {
        width - cont_line_prefix_len
    } else {
        width
    };

    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return format!("{}\n", full_marker.trim_end());
    }

    let mut result = String::new();
    let mut current_line = String::new();
    let mut current_width = first_line_width;
    let mut is_first_line = true;

    for word in words {
        let word_len = word.len();
        let space_needed = if current_line.is_empty() { 0 } else { 1 };

        if !current_line.is_empty() && current_line.len() + space_needed + word_len > current_width {
            if is_first_line {
                result.push_str(&full_marker);
                is_first_line = false;
            } else {
                result.push_str(&continuation_indent);
            }
            result.push_str(&current_line);
            result.push('\n');
            current_line.clear();
            current_width = cont_line_width;
        }

        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        if is_first_line {
            result.push_str(&full_marker);
        } else {
            result.push_str(&continuation_indent);
        }
        result.push_str(&current_line);
        result.push('\n');
    }

    result
}

fn is_single_inline_link(content: &str) -> bool {
    let trimmed = content.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(')')) {
        return false;
    }

    let Some(mid) = trimmed.find("](") else {
        return false;
    };

    let url_part = &trimmed[mid + 2..trimmed.len() - 1];
    if url_part.chars().any(|c| c.is_whitespace()) {
        return false;
    }

    !trimmed[mid + 2..].contains("](")
}

/// Wrap a single line of text at the specified width.
///
/// This function wraps text without breaking long words or on hyphens,
/// similar to Python's textwrap.fill() with break_long_words=False and break_on_hyphens=False.
fn wrap_line(text: &str, width: usize) -> String {
    if text.len() <= width {
        return text.to_string();
    }

    let mut result = String::new();
    let mut current_line = String::new();
    let words: Vec<&str> = text.split_whitespace().collect();

    for word in words {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&current_line);
            current_line.clear();
            current_line.push_str(word);
        }
    }

    if !current_line.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&current_line);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::ConversionOptions;

    #[test]
    fn test_wrap_line_short() {
        let text = "Short text";
        let wrapped = wrap_line(text, 80);
        assert_eq!(wrapped, "Short text");
    }

    #[test]
    fn test_wrap_line_long() {
        let text = "123456789 123456789";
        let wrapped = wrap_line(text, 10);
        assert_eq!(wrapped, "123456789\n123456789");
    }

    #[test]
    fn test_wrap_line_no_break_long_words() {
        let text = "12345678901 12345";
        let wrapped = wrap_line(text, 10);
        assert_eq!(wrapped, "12345678901\n12345");
    }

    #[test]
    fn test_wrap_markdown_disabled() {
        let markdown = "This is a very long line that would normally be wrapped at 40 characters";
        let options = ConversionOptions {
            wrap: false,
            ..Default::default()
        };
        let result = wrap_markdown(markdown, &options);
        assert_eq!(result, markdown);
    }

    #[test]
    fn test_wrap_markdown_paragraph() {
        let markdown = "This is a very long line that would normally be wrapped at 40 characters\n\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 40,
            ..Default::default()
        };
        let result = wrap_markdown(markdown, &options);
        assert!(result.lines().all(|line| line.len() <= 40 || line.trim().is_empty()));
    }

    #[test]
    fn test_wrap_markdown_blockquote_paragraph() {
        let markdown = "> This is a very long blockquote line that should wrap at 30 characters\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 30,
            ..Default::default()
        };
        let result = wrap_markdown(markdown, &options);
        assert!(
            result.lines().all(|line| line.len() <= 30 || line.trim().is_empty()),
            "Some lines exceed wrap width. Got: {}",
            result
        );
        assert!(
            result.contains("> This is a very"),
            "Missing expected wrapped content. Got: {}",
            result
        );
        assert!(
            result.lines().filter(|l| l.starts_with("> ")).count() >= 2,
            "Expected multiple wrapped blockquote lines. Got: {}",
            result
        );
    }

    #[test]
    fn test_wrap_markdown_preserves_code() {
        let markdown = "```\nThis is a very long line in a code block that should not be wrapped\n```\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 40,
            ..Default::default()
        };
        let result = wrap_markdown(markdown, &options);
        assert!(result.contains("This is a very long line in a code block that should not be wrapped"));
    }

    #[test]
    fn test_wrap_markdown_preserves_headings() {
        let markdown = "# This is a very long heading that should not be wrapped even if it exceeds the width\n\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 40,
            ..Default::default()
        };
        let result = wrap_markdown(markdown, &options);
        assert!(
            result.contains("# This is a very long heading that should not be wrapped even if it exceeds the width")
        );
    }

    #[test]
    fn wrap_markdown_wraps_long_list_items() {
        let markdown = "- This is a very long list item that should definitely be wrapped when it exceeds the specified wrap width\n- Short item\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 60,
            ..Default::default()
        };

        let result = wrap_markdown(markdown, &options);

        assert!(
            result.contains("- This is a very long list item that should definitely be\n  wrapped"),
            "First list item not properly wrapped. Got: {}",
            result
        );
        assert!(
            result.contains("- Short item"),
            "Short list item incorrectly modified. Got: {}",
            result
        );
    }

    #[test]
    fn wrap_markdown_wraps_ordered_lists() {
        let markdown = "1. This is a numbered list item with a very long text that should be wrapped at the specified width\n2. Short\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 60,
            ..Default::default()
        };

        let result = wrap_markdown(markdown, &options);

        assert!(
            result.lines().all(|line| line.len() <= 60 || line.trim().is_empty()),
            "Some lines exceed wrap width. Got: {}",
            result
        );
        assert!(result.contains("1."), "Lost ordered list marker. Got: {}", result);
        assert!(
            result.contains("2."),
            "Lost second ordered list marker. Got: {}",
            result
        );
    }

    #[test]
    fn wrap_markdown_preserves_nested_list_structure() {
        let markdown = "- Item one with some additional text that will need to be wrapped across multiple lines\n  - Nested item with long text that also needs wrapping at the specified width\n  - Short nested\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 50,
            ..Default::default()
        };

        let result = wrap_markdown(markdown, &options);

        assert!(result.contains("- Item"), "Lost top-level list marker. Got: {}", result);
        assert!(
            result.contains("  - Nested"),
            "Lost nested list structure. Got: {}",
            result
        );
        assert!(
            result.lines().all(|line| line.len() <= 50 || line.trim().is_empty()),
            "Some lines exceed wrap width. Got: {}",
            result
        );
    }

    #[test]
    fn wrap_markdown_handles_list_with_links() {
        let markdown = "- [A](#a) with additional text that is long enough to require wrapping at the configured width\n  - [B](#b) also has more content that needs wrapping\n  - [C](#c)\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 50,
            ..Default::default()
        };

        let result = wrap_markdown(markdown, &options);

        assert!(result.contains("[A](#a)"), "Lost link in list. Got: {}", result);
        assert!(result.contains("[B](#b)"), "Lost nested link. Got: {}", result);
        assert!(result.contains("[C](#c)"), "Lost short nested link. Got: {}", result);
        assert!(
            result.contains("- [A](#a)"),
            "Lost list structure with link. Got: {}",
            result
        );
        assert!(
            result.contains("  - [B](#b)"),
            "Lost nested list structure. Got: {}",
            result
        );
    }

    #[test]
    fn wrap_markdown_handles_empty_list_items() {
        let markdown = "- \n- Item with text\n- \n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 40,
            ..Default::default()
        };

        let result = wrap_markdown(markdown, &options);

        assert!(result.contains("- "), "Lost list markers. Got: {}", result);
        assert!(result.contains("Item with text"), "Lost item text. Got: {}", result);
    }

    #[test]
    fn wrap_markdown_preserves_indented_lists_with_wrapping() {
        let markdown = "- [A](#a) with some additional text that makes this line very long and should be wrapped\n  - [B](#b)\n  - [C](#c) with more text that is also quite long and needs wrapping\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 50,
            ..Default::default()
        };

        let result = wrap_markdown(markdown, &options);

        assert!(result.contains("- [A](#a)"), "Lost top-level link. Got: {}", result);
        assert!(result.contains("  - [B](#b)"), "Lost nested link B. Got: {}", result);
        assert!(result.contains("  - [C](#c)"), "Lost nested link C. Got: {}", result);
        assert!(
            result.lines().all(|line| line.len() <= 50),
            "Some lines exceed wrap width:\n{}",
            result
        );
    }

    #[test]
    fn wrap_markdown_does_not_wrap_link_only_items() {
        let markdown = "- [A very long link label that would exceed wrap width](#a-very-long-link-label)\n  - [Nested very long link label that would also exceed](#nested)\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 30,
            ..Default::default()
        };

        let result = wrap_markdown(markdown, &options);

        assert!(result.contains("- [A very long link label that would exceed wrap width](#a-very-long-link-label)"));
        assert!(result.contains("  - [Nested very long link label that would also exceed](#nested)"));
    }
}
