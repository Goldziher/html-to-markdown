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

        let is_structural = is_heading(trimmed)
            || is_list_like(trimmed)
            || is_numbered_list(trimmed)
            || trimmed.starts_with('>')
            || trimmed.starts_with('|')
            || trimmed.starts_with('=');

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

    if in_paragraph && !paragraph_buffer.is_empty() {
        result.push_str(&wrap_line(&paragraph_buffer, options.wrap_width));
        result.push_str("\n\n");
    }

    result
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
    fn wrap_markdown_preserves_indented_lists_with_links() {
        let markdown = "- [A](#a)\n  - [B](#b)\n  - [C](#c)\n";
        let options = ConversionOptions {
            wrap: true,
            wrap_width: 20,
            ..Default::default()
        };

        let result = wrap_markdown(markdown, &options);
        let expected = "- [A](#a)\n  - [B](#b)\n  - [C](#c)\n";
        assert_eq!(result, expected);
    }
}
