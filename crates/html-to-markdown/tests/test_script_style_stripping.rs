//! Tests for script and style tag stripping before parsing.
//!
//! This test suite verifies that script and style tags are completely removed
//! from HTML before parsing, preventing the tl parser from misinterpreting
//! HTML-like content inside scripts as actual tags.

use html_to_markdown_rs::{ConversionOptions, MetadataConfig, convert, convert_with_metadata};

#[test]
fn test_strip_simple_script_tag() {
    // Simple script with plain JavaScript
    let html = r#"<html>
<head>
  <script>var x = 1; var y = 2;</script>
</head>
<body>
  <p>Real content here</p>
</body>
</html>"#;

    let options = ConversionOptions::default();
    let result = convert(html, Some(options)).expect("Failed to convert");

    println!("Output:\n{}", result);

    // Should extract body content
    assert!(result.contains("Real content"), "Should contain body content");

    // Script content should not appear in output
    assert!(!result.contains("var x"), "Script content should be removed");
    assert!(!result.contains("var y"), "Script content should be removed");
}

#[test]
fn test_strip_script_with_html_like_content() {
    // Critical test: script with HTML-like content that confuses parsers
    let html = r#"<html>
<head>
  <script>
    var data = {
      html: '<div class="fake">This looks like HTML</div>',
      tags: '<span>test</span>',
      close: '</body>'
    };
  </script>
</head>
<body>
  <div id="real-content">
    <p>Real article content here</p>
  </div>
</body>
</html>"#;

    let options = ConversionOptions::default();
    let result = convert(html, Some(options)).expect("Failed to convert");

    println!("Output:\n{}", result);

    // Should extract body content
    assert!(
        result.contains("Real article content"),
        "Should contain real body content"
    );

    // Script content should NOT appear
    assert!(!result.contains("fake"), "Fake HTML from script should be removed");
    assert!(!result.contains("var data"), "Script variables should not appear");
}

#[test]
fn test_strip_style_tag() {
    // Style tags with CSS that has HTML-like syntax
    let html = r#"<html>
<head>
  <style>
    div { content: '<fake>test</fake>'; }
    body { display: block; }
  </style>
</head>
<body>
  <p>Paragraph content</p>
</body>
</html>"#;

    let options = ConversionOptions::default();
    let result = convert(html, Some(options)).expect("Failed to convert");

    println!("Output:\n{}", result);

    // Should have body content
    assert!(result.contains("Paragraph content"), "Should contain paragraph");

    // Style content should be removed
    assert!(!result.contains("content:"), "CSS should not appear in output");
    assert!(!result.contains("display:"), "CSS properties should not appear");
}

#[test]
fn test_preserve_json_ld_script() {
    // JSON-LD script tags should be preserved for metadata extraction
    let html = r#"<html>
<head>
  <title>Article Title</title>
  <script type="application/ld+json">
    {
      "@context": "https://schema.org",
      "@type": "Article",
      "headline": "My Article",
      "author": { "@type": "Person", "name": "John Doe" }
    }
  </script>
</head>
<body>
  <p>Article content</p>
</body>
</html>"#;

    let options = ConversionOptions {
        extract_metadata: true,
        ..Default::default()
    };

    let (markdown, metadata) =
        convert_with_metadata(html, Some(options), MetadataConfig::default()).expect("Failed to convert");

    println!("Markdown:\n{}", markdown);
    println!("Metadata: {:?}", metadata.document.title);

    // Should extract both content and metadata
    assert!(markdown.contains("Article content"), "Should contain body content");

    // Should have extracted metadata
    assert_eq!(
        metadata.document.title,
        Some("Article Title".to_string()),
        "Should extract title"
    );

    // JSON-LD should be in structured data
    assert!(!metadata.structured_data.is_empty(), "Should extract JSON-LD");
    if let Some(schema) = metadata.structured_data.first() {
        assert!(
            schema.raw_json.contains("Article"),
            "JSON-LD should contain Article type"
        );
        assert_eq!(
            schema.schema_type,
            Some("Article".to_string()),
            "Should detect schema type"
        );
    }
}

#[test]
fn test_multiple_script_tags() {
    // Multiple script tags with various content
    let html = r#"<html>
<head>
  <script>console.log('script 1');</script>
  <script type="text/javascript">
    if (x < y) {
      document.write('<p>Fake paragraph</p>');
    }
  </script>
  <script type="application/ld+json">
    {"@type": "WebPage", "@context": "https://schema.org"}
  </script>
</head>
<body>
  <h1>Real Title</h1>
  <p>Real paragraph</p>
</body>
</html>"#;

    let options = ConversionOptions {
        extract_metadata: true,
        ..Default::default()
    };

    let (markdown, metadata) =
        convert_with_metadata(html, Some(options), MetadataConfig::default()).expect("Failed to convert");

    println!("Markdown:\n{}", markdown);

    // Should have real content
    assert!(markdown.contains("Real Title"), "Should have h1");
    assert!(markdown.contains("Real paragraph"), "Should have paragraph");

    // Should NOT have fake content from scripts
    assert!(
        !markdown.contains("Fake paragraph"),
        "Should not have fake HTML from script"
    );
    assert!(!markdown.contains("console.log"), "Should not have console.log");
    assert!(!markdown.contains("document.write"), "Should not have document.write");

    // JSON-LD should be extracted in metadata
    assert!(
        !metadata.structured_data.is_empty(),
        "Should extract JSON-LD structured data"
    );
}

#[test]
fn test_reuters_like_structure() {
    // Mimics Reuters structure with divs containing data-testid attributes
    let html = r#"<!DOCTYPE html>
<html>
<head>
  <title>Reuters Article</title>
  <meta property="og:title" content="Breaking News">
  <meta property="og:description" content="Important story">
  <script>
    window.data = {
      paragraphs: [
        '<div data-testid="paragraph-0">Fake content from script</div>',
        '<div data-testid="paragraph-1">Another fake</div>'
      ]
    };
  </script>
</head>
<body>
<article>
  <div data-testid="ArticleBody" class="article-body-module__wrapper">
    <div data-testid="paragraph-0" class="article-body-module__paragraph">
      SAN FRANCISCO, Dec 27 (Reuters) - A widespread power outage in San Francisco.
    </div>
    <div data-testid="paragraph-1" class="article-body-module__paragraph">
      The outage affected thousands of residents and businesses across the city.
    </div>
  </div>
</article>
</body>
</html>"#;

    let options = ConversionOptions {
        extract_metadata: true,
        ..Default::default()
    };

    let (markdown, metadata) =
        convert_with_metadata(html, Some(options), MetadataConfig::default()).expect("Failed to convert");

    println!("Markdown output:\n{}", markdown);
    println!("Metadata title: {:?}", metadata.document.title);

    // Should have extracted metadata
    assert_eq!(
        metadata.document.title,
        Some("Reuters Article".to_string()),
        "Should extract title"
    );
    assert!(
        metadata.document.open_graph.contains_key("title"),
        "Should extract OG title"
    );

    // Should have article content
    assert!(markdown.contains("SAN FRANCISCO"), "Should contain first paragraph");
    assert!(
        markdown.contains("widespread power outage"),
        "Should contain article text"
    );
    assert!(
        markdown.contains("thousands of residents"),
        "Should contain second paragraph"
    );

    // Should NOT have fake content from script
    assert!(!markdown.contains("window.data"), "Should not have window.data");
    assert!(
        !markdown.contains("'<div data-testid"),
        "Should not have fake HTML strings from script"
    );
}

#[test]
fn test_complex_nested_script_content() {
    // Script with complex nested structures that might confuse parsers
    let html = r#"<html>
<head>
  <script>
    var config = {
      template: `
        <html>
          <body>
            <div class="container">
              <p>Nested HTML in template string</p>
            </div>
          </body>
        </html>
      `,
      patterns: [
        { regex: '/<body>.*?<\/body>/gs' },
        { html: '<script>alert("xss")</script>' }
      ]
    };
  </script>
</head>
<body>
  <section>
    <h2>Main Content</h2>
    <p>This is the actual article.</p>
  </section>
</body>
</html>"#;

    let options = ConversionOptions::default();
    let result = convert(html, Some(options)).expect("Failed to convert");

    println!("Output:\n{}", result);

    // Should have actual content
    assert!(result.contains("Main Content"), "Should have h2");
    assert!(result.contains("actual article"), "Should have paragraph");

    // Should NOT have nested fake HTML from script
    assert!(
        !result.contains("Nested HTML in template"),
        "Should not have template content"
    );
    assert!(!result.contains("Container"), "Should not have nested div");
}

#[test]
fn test_case_insensitive_script_style_tags() {
    // Test that script/style tags are matched case-insensitively
    let html = r#"<html>
<head>
  <SCRIPT>console.log('uppercase script');</SCRIPT>
  <Style>body { margin: 0; }</Style>
  <ScRiPt>var x = 1;</ScRiPt>
</head>
<body>
  <p>Content</p>
</body>
</html>"#;

    let options = ConversionOptions::default();
    let result = convert(html, Some(options)).expect("Failed to convert");

    println!("Output:\n{}", result);

    // Should have content
    assert!(result.contains("Content"), "Should have content");

    // Should not have script content regardless of case
    assert!(!result.contains("console.log"), "Should remove SCRIPT tag content");
    assert!(!result.contains("margin:"), "Should remove STYLE tag content");
    assert!(!result.contains("var x"), "Should remove ScRiPt tag content");
}

#[test]
fn test_performance_large_script() {
    // Performance test: large script tag shouldn't cause issues
    let mut html = String::from(r#"<html><head><script>"#);

    // Add 1MB of fake content
    for _ in 0..10000 {
        html.push_str("var data = '<div>fake content</div>'; ");
    }

    html.push_str(r#"</script></head><body><p>Real content</p></body></html>"#);

    println!("Testing with {} byte HTML", html.len());

    let options = ConversionOptions::default();
    let result = convert(&html, Some(options)).expect("Failed to convert");

    // Should work and extract real content
    assert!(
        result.contains("Real content"),
        "Should extract body despite large script"
    );
}

#[test]
fn test_inline_script_attributes_not_affected() {
    // Inline event handlers should not be affected by script stripping
    let html = r#"<html>
<head>
  <script>console.log('bad');</script>
</head>
<body>
  <button onclick="console.log('click');">Click me</button>
  <p>Content</p>
</body>
</html>"#;

    let options = ConversionOptions::default();
    let result = convert(html, Some(options)).expect("Failed to convert");

    println!("Output:\n{}", result);

    // Should have content and button
    assert!(result.contains("Click me"), "Should have button text");
    assert!(result.contains("Content"), "Should have paragraph");

    // The onclick script inside the tag attribute might appear in output (that's okay)
    // but the inline script tag content should be gone
    assert!(
        !result.contains("console.log('bad')"),
        "Should remove script tag content"
    );
}
