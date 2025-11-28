use html_to_markdown_rs::convert;

#[test]
fn images_with_dimensions_render_as_markdown_links() {
    let html = r#"<img src="data:image/png;base64,xyz==" alt="Pixel" width="100" height="100"/>"#;

    let markdown = convert(html, None).expect("image conversion should succeed");

    assert_eq!(markdown.trim(), "![Pixel](data:image/png;base64,xyz==)");
}
