//! Issue #145 Regression Tests
//!
//! Tests for ensuring that `strip_newlines=True` doesn't cause excessive whitespace
//! around block elements. The root cause was that newlines were converted to spaces
//! BEFORE whitespace-only node detection, causing the detection to fail.

use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_strip_newlines_preserves_block_spacing() {
    let html = r#"<section>
    <h1>Heading</h1>
    <p>Paragraph one.</p>
    <p>Paragraph two.</p>
</section>"#;

    let options = ConversionOptions {
        strip_newlines: true,
        extract_metadata: false,
        ..Default::default()
    };
    let result = convert(html, Some(options)).unwrap();

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
        "excessive blank lines detected: {} consecutive blanks in:\n{}",
        max_consecutive_blank,
        result
    );

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
    let html = r#"<p>This is a paragraph
with line breaks
in the middle</p>"#;

    let options = ConversionOptions {
        strip_newlines: true,
        extract_metadata: false,
        ..Default::default()
    };
    let result = convert(html, Some(options)).unwrap();

    let text = result.trim();

    let content_lines: Vec<&str> = text.lines().collect();

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

    let options = ConversionOptions {
        strip_newlines: true,
        extract_metadata: false,
        ..Default::default()
    };
    let result = convert(html, Some(options)).unwrap();

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
