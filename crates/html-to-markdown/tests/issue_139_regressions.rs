use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn long_multibyte_link_label_does_not_panic() {
    let mut html = String::from("<a href=\"https://example.com/article\">");
    html.push_str(&"a".repeat(511));
    html.push('👍');
    html.push_str("</a>");

    let markdown = convert(&html, Some(ConversionOptions::default())).unwrap();
    let expected_label = format!("{}…", "a".repeat(511));

    assert!(
        markdown.contains(&format!("[{}]", expected_label)),
        "expected truncated label to appear in markdown output; got: {markdown}"
    );
}
