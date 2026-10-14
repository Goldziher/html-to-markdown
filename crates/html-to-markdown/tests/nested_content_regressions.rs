//! Regression coverage for nested layout tables and malformed anchors.

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

#[test]
fn nested_data_table_preserves_rows_and_columns() {
    let html = "<table><tr><td><p>Summary</p><table><tr><th>Item</th><th>Amount</th></tr><tr><td>Alpha</td><td>10</td></tr></table></td></tr></table>";
    let options = ConversionOptions {
        br_in_tables: true,
        compact_tables: true,
        ..Default::default()
    };
    assert_eq!(
        convert(html, Some(options)).unwrap().content.unwrap(),
        "Summary\n\n| Item | Amount |\n| --- | --- |\n| Alpha | 10 |\n"
    );
}

#[test]
fn nested_anchors_preserve_both_destinations() {
    let html = r#"<a href="https://example.com/outer">Outer <a href="https://example.com/inner">Inner</a></a>"#;
    assert_eq!(
        convert(html, None).unwrap().content.unwrap().trim(),
        "[Outer](https://example.com/outer)[Inner](https://example.com/inner)"
    );
}

#[test]
fn nested_anchors_inside_formatting_preserve_links_in_tier_two() {
    let html = r#"<a href="/outer">Outer <span><a href="/inner">Inner</a></span></a> Tail"#;
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        ..Default::default()
    };
    assert_eq!(
        convert(html, Some(options)).unwrap().content.unwrap().trim(),
        "[Outer](/outer)[Inner](/inner) Tail"
    );
}

#[test]
fn ordinary_sibling_anchors_keep_separation() {
    let html = r#"<a href="/first">First</a> <a href="/second">Second</a>"#;
    assert_eq!(
        convert(html, None).unwrap().content.unwrap().trim(),
        "[First](/first) [Second](/second)"
    );
}

#[test]
fn wrapper_preserves_content_after_nested_table() {
    let html = "<table><tbody><tr><td>Before<table><tr><td>A</td><td>B</td></tr></table><p>After</p></td></tr></tbody></table>";
    let options = ConversionOptions {
        compact_tables: true,
        ..Default::default()
    };
    assert_eq!(
        convert(html, Some(options)).unwrap().content.unwrap(),
        "Before\n\n| A | B |\n| --- | --- |\n\nAfter\n"
    );
}

#[test]
fn preformatted_nested_anchors_keep_literal_content() {
    let html = r#"<pre><a href="/outer">Outer <a href="/inner">Inner</a></a></pre>"#;
    assert_eq!(convert(html, None).unwrap().content.unwrap(), "```\nOuter Inner\n```\n");
}

#[cfg(feature = "visitor")]
#[test]
fn nested_table_wrapper_preserves_balanced_visitor_callbacks() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct Tables {
        events: Vec<&'static str>,
    }
    impl HtmlVisitor for Tables {
        fn visit_table_start(&mut self, _: &NodeContext) -> VisitResult {
            self.events.push("start");
            VisitResult::Continue
        }
        fn visit_table_end(&mut self, _: &NodeContext, _: &str) -> VisitResult {
            self.events.push("end");
            VisitResult::Continue
        }
    }
    let visitor = Arc::new(Mutex::new(Tables::default()));
    let options = ConversionOptions {
        visitor: Some(visitor.clone()),
        compact_tables: true,
        ..Default::default()
    };
    let result = convert(
        "<table><tr><td><table><tr><td>A</td><td>B</td></tr></table></td></tr></table>",
        Some(options),
    )
    .unwrap();
    assert_eq!(result.content.unwrap(), "| A | B |\n| --- | --- |\n");
    assert_eq!(visitor.lock().unwrap().events, ["start", "start", "end", "end"]);
}

#[test]
fn deeply_nested_wrappers_preserve_surrounding_content_and_depth_warning() {
    const WRAPPER_COUNT: usize = 20;
    const MAX_DEPTH: usize = 8;
    const MAX_OUTPUT_BYTES: usize = 256;
    let mut html = String::from("<p>Before</p>");
    html.push_str(&"<table><tr><td>".repeat(WRAPPER_COUNT));
    html.push_str("Leaf");
    html.push_str(&"</td></tr></table>".repeat(WRAPPER_COUNT));
    html.push_str("<p>After</p>");
    let options = ConversionOptions {
        max_depth: Some(MAX_DEPTH),
        compact_tables: true,
        include_document_structure: true,
        ..Default::default()
    };
    let result = convert(&html, Some(options)).unwrap();
    let content = result.content.unwrap();
    assert!(content.starts_with("Before\n"));
    assert!(content.ends_with("After\n"));
    assert!(
        content.contains("| --- |"),
        "the truncated table must still render its structure: {content}"
    );
    assert!(content.len() < MAX_OUTPUT_BYTES);
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(
        result.warnings[0].kind,
        html_to_markdown_rs::WarningKind::DepthLimitExceeded
    );
    assert!(!result.tables.is_empty());
}
