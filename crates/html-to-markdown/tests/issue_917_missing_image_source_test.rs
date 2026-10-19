#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

fn assert_paths(html: &str, expected: &str) {
    for tier_strategy in [
        TierStrategy::Auto,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
        TierStrategy::Tier2,
    ] {
        let output = convert(
            html,
            Some(ConversionOptions {
                extract_metadata: false,
                highlight_style: HighlightStyle::None,
                tier_strategy,
                ..ConversionOptions::default()
            }),
        )
        .expect("image conversion should succeed")
        .content
        .unwrap_or_default();
        assert_eq!(output, expected, "{html}");
    }
}

#[test]
fn should_preserve_image_markup_when_a_source_is_missing_or_empty() {
    for source in ["", " src=\"\"", " src=\"   \""] {
        let html = format!(r#"<img{source} alt="A \times B">"#);
        assert_paths(&html, "![A \\times B](<>)\n");
    }
}

#[test]
fn should_preserve_a_missing_source_image_title_and_escape_its_alt_text() {
    assert_paths(
        r#"<p>before <img alt="[A" title="Diagram"> after</p>"#,
        "before ![\\[A](<> \"Diagram\") after\n",
    );
}

#[test]
fn should_keep_empty_images_silent_and_code_and_heading_images_as_alt_text() {
    assert_paths("<p>a <img> b</p>", "a b\n");
    assert_paths(r#"<code><img alt="A \times B"></code>"#, "`A \\times B`\n");
    assert_paths(r#"<h2><img alt="A \times B"></h2>"#, "## A \\times B\n");
}
