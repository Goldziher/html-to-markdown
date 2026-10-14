#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn markdown(html: &str, tier_strategy: TierStrategy) -> String {
    convert(
        html,
        Some(ConversionOptions {
            extract_metadata: false,
            tier_strategy,
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed")
    .content
    .unwrap_or_default()
}

#[test]
fn should_keep_media_links_in_tables_without_text_headers() {
    for (element, link) in [
        ("<video src=\"/v.mp4\"></video>", "[video](/v.mp4)"),
        ("<audio src=\"/a.mp3\"></audio>", "[audio](/a.mp3)"),
        ("<iframe src=\"/f.html\"></iframe>", "[iframe](/f.html)"),
        ("<video><source src=\"/v.mp4\"></video>", "[video](/v.mp4)"),
        ("<audio><source src=\"/a.mp3\"></audio>", "[audio](/a.mp3)"),
    ] {
        let html = format!("<table><tr><td>{element}</td></tr></table>");
        let expected = format!("| {link} |\n| {} |\n", "-".repeat(link.len()));
        for tier in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
            assert_eq!(markdown(&html, tier), expected, "{tier:?}: {element}");
        }
    }
}

#[test]
fn should_keep_inline_svg_in_a_table_without_text_headers() {
    let element = "<svg width=\"10\" height=\"10\"><circle r=\"3\"/></svg>";
    let html = format!("<table><tr><td>{element}</td></tr></table>");
    for tier in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
        let image = markdown(element, tier).trim_end().to_owned();
        assert!(image.starts_with("![](data:image/svg+xml;base64,"));
        let expected = format!("| {image} |\n| {} |\n", "-".repeat(image.len()));
        assert_eq!(markdown(&html, tier), expected, "{tier:?}");
    }
}

#[test]
fn should_still_drop_tables_with_empty_media_elements() {
    for element in ["<video></video>", "<audio></audio>", "<iframe></iframe>"] {
        let html = format!("<table><tr><td>{element}</td></tr></table>");
        for tier in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
            assert_eq!(markdown(&html, tier), "", "{tier:?}: {element}");
        }
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_drop_a_table_when_its_only_media_is_skipped_by_a_visitor() {
    use html_to_markdown_rs::NodeContext;
    use html_to_markdown_rs::visitor::{HtmlVisitor, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct SkipMedia;
    impl HtmlVisitor for SkipMedia {
        fn visit_video(&mut self, _ctx: &NodeContext<'_>, _src: Option<&str>) -> VisitResult {
            VisitResult::Skip
        }
    }
    let html = "<table><tr><td><video src=\"/v.mp4\"></video></td></tr></table>";
    let result = convert(
        html,
        Some(ConversionOptions {
            extract_metadata: false,
            visitor: Some(Arc::new(Mutex::new(SkipMedia))),
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed");
    assert_eq!(result.content.unwrap_or_default(), "");
}

#[test]
fn should_drop_a_table_when_its_only_media_is_excluded() {
    let html = "<table><tr><td><video src=\"/v.mp4\"></video></td></tr></table>";
    let result = convert(
        html,
        Some(ConversionOptions {
            extract_metadata: false,
            exclude_selectors: vec!["video".to_string()],
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed");
    assert_eq!(result.content.unwrap_or_default(), "");
}

#[test]
fn should_keep_literal_pipe_and_rule_content_when_checking_rendered_rows() {
    for (cell, expected) in [("|", "\\|"), ("---", "---")] {
        let html = format!("<table><tr><td>{cell}</td></tr></table>");
        let result = convert(
            &html,
            Some(ConversionOptions {
                extract_metadata: false,
                compact_tables: true,
                tier_strategy: TierStrategy::Tier2,
                ..ConversionOptions::default()
            }),
        )
        .expect("conversion should succeed");
        assert_eq!(result.content.unwrap_or_default(), format!("| {expected} |\n| --- |\n"));
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_keep_custom_row_output_when_source_media_has_no_rendered_content() {
    use html_to_markdown_rs::NodeContext;
    use html_to_markdown_rs::visitor::{HtmlVisitor, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct CustomRow;
    impl HtmlVisitor for CustomRow {
        fn visit_table_row(&mut self, _ctx: &NodeContext<'_>, _cells: &[String], _is_header: bool) -> VisitResult {
            VisitResult::Custom("custom row".to_string())
        }
    }
    let html = "<table><tr><td><video src=\"/v.mp4\"></video></td></tr></table>";
    let result = convert(
        html,
        Some(ConversionOptions {
            extract_metadata: false,
            visitor: Some(Arc::new(Mutex::new(CustomRow))),
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed");
    assert_eq!(result.content.unwrap_or_default(), "custom row\n");
}

#[test]
fn should_keep_a_header_when_all_media_body_cells_are_excluded() {
    let html = "<table><tr><th>h</th></tr><tr><td><video src=\"/v.mp4\"></video></td></tr></table>";
    let result = convert(
        html,
        Some(ConversionOptions {
            extract_metadata: false,
            compact_tables: true,
            exclude_selectors: vec!["video".to_string()],
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed");
    assert_eq!(result.content.unwrap_or_default(), "| h |\n| --- |\n|  |\n");
}
