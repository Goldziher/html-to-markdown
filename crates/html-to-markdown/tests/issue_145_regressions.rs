//! Issue #145 Regression Tests
//!
//! Tests for ensuring that `strip_newlines=True` doesn't cause excessive whitespace
//! around block elements. The root cause was that newlines were converted to spaces
//! BEFORE whitespace-only node detection, causing the detection to fail.

use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_strip_newlines_preserves_block_spacing() {
    // Issue #145: strip_newlines=True causes excessive whitespace around block elements
    // When strip_newlines is enabled, newlines in formatting should not create
    // excessive blank lines between block elements.
    let html = r#"<section>
    <h1>Heading</h1>
    <p>Paragraph one.</p>
    <p>Paragraph two.</p>
</section>"#;

    let mut options = ConversionOptions::default();
    options.strip_newlines = true;
    options.extract_metadata = false;
    let result = convert(html, Some(options)).unwrap();

    // The result should have proper spacing: heading, then paragraphs
    // Should NOT have excessive blank lines between blocks
    let lines: Vec<&str> = result.lines().collect();

    // Count consecutive blank lines - should not have more than 1
    let mut max_consecutive_blank = 0;
    let mut current_blank_count = 0;
    for line in &lines {
        if line.trim().is_empty() {
            current_blank_count += 1;
            max_consecutive_blank = max_consecutive_blank.max(current_blank_count);
        } else {
            current_blank_count = 0;
        }
    }

    assert!(
        max_consecutive_blank <= 1,
        "excessive blank lines detected: {} consecutive blanks in:\n{}",
        max_consecutive_blank,
        result
    );

    // Verify content is present
    assert!(result.contains("Heading"), "heading missing from: {}", result);
    assert!(
        result.contains("Paragraph one"),
        "paragraph one missing from: {}",
        result
    );
    assert!(
        result.contains("Paragraph two"),
        "paragraph two missing from: {}",
        result
    );
}

#[test]
fn test_strip_newlines_removes_inline_newlines() {
    // When strip_newlines is enabled, newlines within inline content
    // (like within paragraphs) should be converted to spaces
    let html = r#"<p>This is a paragraph
with line breaks
in the middle</p>"#;

    let mut options = ConversionOptions::default();
    options.strip_newlines = true;
    options.extract_metadata = false;
    let result = convert(html, Some(options)).unwrap();

    // The paragraph text should be on a single line or properly spaced
    let text = result.trim();

    // Should not have raw newlines within the paragraph content
    // (newlines are allowed at the end of a paragraph block)
    let content_lines: Vec<&str> = text.lines().collect();

    // Find the paragraph line (should contain the text)
    let has_paragraph_line = content_lines.iter().any(|line| {
        let trimmed = line.trim();
        trimmed.contains("This is a paragraph")
            && trimmed.contains("with line breaks")
            && trimmed.contains("in the middle")
    });

    assert!(
        has_paragraph_line,
        "paragraph should have inline newlines converted to spaces in: {}",
        result
    );
}

#[test]
fn test_strip_newlines_handles_nested_blocks() {
    // When strip_newlines is enabled, nested block elements should
    // maintain proper spacing without creating excessive whitespace
    let html = r#"<section>
    <div>
        <h2>Nested Heading</h2>
        <p>Content inside nested div.</p>
    </div>
    <div>
        <h2>Another Section</h2>
        <p>More content here.</p>
    </div>
</section>"#;

    let mut options = ConversionOptions::default();
    options.strip_newlines = true;
    options.extract_metadata = false;
    let result = convert(html, Some(options)).unwrap();

    // Verify all content is present
    assert!(
        result.contains("Nested Heading"),
        "nested heading missing from: {}",
        result
    );
    assert!(
        result.contains("Content inside nested div"),
        "nested content missing from: {}",
        result
    );
    assert!(
        result.contains("Another Section"),
        "another section heading missing from: {}",
        result
    );
    assert!(
        result.contains("More content here"),
        "more content missing from: {}",
        result
    );

    // Check for excessive whitespace
    let lines: Vec<&str> = result.lines().collect();
    let mut max_consecutive_blank = 0;
    let mut current_blank_count = 0;
    for line in &lines {
        if line.trim().is_empty() {
            current_blank_count += 1;
            max_consecutive_blank = max_consecutive_blank.max(current_blank_count);
        } else {
            current_blank_count = 0;
        }
    }

    assert!(
        max_consecutive_blank <= 1,
        "excessive blank lines in nested blocks: {} consecutive blanks in:\n{}",
        max_consecutive_blank,
        result
    );
}
