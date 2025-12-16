use html_to_markdown_rs::metadata::MetadataConfig;

#[test]
fn extracts_json_ld_from_head_script() {
    let html = r#"
        <html>
          <head>
            <script type="application/ld+json">
              { "@context": "https://schema.org", "@type": "Article", "headline": "Example" }
            </script>
            <title>Title</title>
          </head>
          <body>Hello</body>
        </html>
    "#;

    let (_markdown, metadata) = html_to_markdown_rs::convert_with_metadata(html, None, MetadataConfig::default())
        .expect("convert_with_metadata failed");

    assert_eq!(metadata.structured_data.len(), 1);
    assert!(metadata.structured_data[0].raw_json.contains(r#""@type": "Article""#));
    assert_eq!(metadata.structured_data[0].schema_type.as_deref(), Some("Article"));
}

#[test]
fn extracts_json_ld_from_body_script_and_keeps_content() {
    let html = r#"
        <html>
          <head><title>Title</title></head>
          <body>
            <script type="application/ld+json">
              { "@context": "https://schema.org", "@type": "Article", "headline": "Example" }
            </script>
          </body>
        </html>
    "#;

    let (_markdown, metadata) = html_to_markdown_rs::convert_with_metadata(html, None, MetadataConfig::default())
        .expect("convert_with_metadata failed");

    assert_eq!(metadata.structured_data.len(), 1);
    assert!(!metadata.structured_data[0].raw_json.trim().is_empty());
    assert_eq!(metadata.structured_data[0].schema_type.as_deref(), Some("Article"));
}
