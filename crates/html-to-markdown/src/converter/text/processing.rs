//! Text processing utilities for HTML to Markdown conversion.
//!
//! This module provides functions for processing text content, including
//! code block dedentation and special character handling.

/// Remove common leading whitespace from all lines in a code block.
///
/// This is useful when HTML authors indent `<pre>` content for readability,
/// so we can strip the shared indentation without touching meaningful spacing.
///
/// # Examples
///
/// ```text
/// "    line1\n    line2" → "line1\nline2"
/// "  indent1\n    indent2" → "indent1\n  indent2" (removes 2 chars, minimum indent)
/// "  \n  code" → "\ncode"
/// ```
pub fn dedent_code_block(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.char_indices()
                .take_while(|(_, c)| c.is_whitespace())
                .map(|(idx, c)| idx + c.len_utf8())
                .last()
                .unwrap_or(0)
        })
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                *line
            } else {
                &line[min_indent.min(line.len())..]
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
