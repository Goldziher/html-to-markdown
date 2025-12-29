//! Issue #146 Regression Tests
//!
//! Tests for ensuring that `strip_tags` and `preserve_tags` properly prevent
//! `<meta>` and `<title>` tags from appearing in YAML frontmatter when metadata
//! extraction is enabled.

use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_strip_tags_prevents_metadata_extraction() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Document</title>
    <meta name="author" content="John Doe">
    <meta name="description" content="A test document">
    <meta property="og:title" content="Test OG Title">
</head>
<body>
    <p>Main content here</p>
</body>
</html>"#;

    let options = ConversionOptions {
        extract_metadata: true,
        strip_tags: vec!["meta".to_string()],
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    assert!(
        result.contains("Main content here"),
        "Body content should be preserved: {}",
        result
    );

    assert!(
        result.contains("title: Test Document"),
        "Title should still be extracted in frontmatter: {}",
        result
    );

    assert!(
        !result.contains("meta-author"),
        "meta-author should NOT be in frontmatter when strip_tags=['meta']: {}",
        result
    );
    assert!(
        !result.contains("meta-description"),
        "meta-description should NOT be in frontmatter when strip_tags=['meta']: {}",
        result
    );
    assert!(
        !result.contains("meta-og-title"),
        "meta-og-title should NOT be in frontmatter when strip_tags=['meta']: {}",
        result
    );
}

#[test]
fn test_strip_tags_title_prevents_extraction() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Should Be Stripped</title>
    <meta name="author" content="Jane Smith">
    <meta name="keywords" content="test, demo">
</head>
<body>
    <h1>Document Heading</h1>
    <p>Some content</p>
</body>
</html>"#;

    let options = ConversionOptions {
        extract_metadata: true,
        strip_tags: vec!["title".to_string()],
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    assert!(
        result.contains("Document Heading") && result.contains("Some content"),
        "Body content should be preserved: {}",
        result
    );

    assert!(
        result.contains("meta-author"),
        "meta-author should still be extracted when only title is stripped: {}",
        result
    );
    assert!(
        result.contains("meta-keywords"),
        "meta-keywords should still be extracted when only title is stripped: {}",
        result
    );

    assert!(
        !result.contains("title: Should Be Stripped"),
        "title should NOT be in frontmatter when strip_tags=['title']: {}",
        result
    );
}

#[test]
fn test_preserve_tags_prevents_metadata_extraction() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Preserved Title</title>
    <meta name="viewport" content="width=device-width">
    <meta name="author" content="Test Author">
</head>
<body>
    <div>
        <p>Body content</p>
    </div>
</body>
</html>"#;

    let options = ConversionOptions {
        extract_metadata: true,
        preserve_tags: vec!["meta".to_string()],
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    assert!(
        result.contains("Body content"),
        "Body content should be preserved: {}",
        result
    );

    assert!(
        result.contains("title: Preserved Title"),
        "title should still be extracted in frontmatter: {}",
        result
    );

    assert!(
        !result.contains("meta-viewport"),
        "meta-viewport should NOT be in YAML frontmatter when preserve_tags=['meta']: {}",
        result
    );
    assert!(
        !result.contains("meta-author"),
        "meta-author should NOT be in YAML frontmatter when preserve_tags=['meta']: {}",
        result
    );
}
