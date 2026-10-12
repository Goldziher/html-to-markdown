#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn check(html: &str, expected: &str) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            html,
            ConversionOptions {
                extract_metadata: false,
                tier_strategy,
                ..Default::default()
            },
        )
        .expect("heading blocks convert");
        assert_eq!(result.content.as_deref(), Some(expected), "{tier_strategy:?}: {html}");
    }
}

#[test]
fn should_write_heading_block_labels_without_structural_quote_or_list_markers() {
    for (html, expected) in [
        ("<h2><blockquote>One</blockquote>items</h2>", "## One items\n"),
        (
            "<h2>before<blockquote>One</blockquote>after</h2>",
            "## before One after\n",
        ),
        ("<h2><ul><li>One</li></ul>items</h2>", "## One items\n"),
        (
            "<h2><ol start=10><li>One</li><li>Two</li></ol>items</h2>",
            "## One Two items\n",
        ),
        ("<h2><li>One</li>items</h2>", "## One items\n"),
        (
            "<h2><ul><li><input type=checkbox checked>One</li></ul>items</h2>",
            "## [x] One items\n",
        ),
        (
            "<h2><blockquote><ul><li>One</li><li><blockquote>Two</blockquote></li></ul></blockquote>items</h2>",
            "## One Two items\n",
        ),
        ("<h2><blockquote>&gt; One</blockquote>items</h2>", "## > One items\n"),
        ("<h2><ul><li>- One</li></ul>items</h2>", "## - One items\n"),
    ] {
        check(html, expected);
    }
}

#[test]
fn should_keep_literal_heading_text_and_normal_quote_and_list_blocks() {
    for (html, expected) in [
        ("<h2>&gt; - One items</h2>", "## > - One items\n"),
        ("<h2><p>One</p>items</h2>", "## One items\n"),
        ("<blockquote>One</blockquote>items", "> One\n\nitems\n"),
        ("<ul><li>One</li></ul>items", "- One\n\nitems\n"),
        ("<blockquote><ul><li>One</li></ul></blockquote>", "> - One\n"),
    ] {
        check(html, expected);
    }
}

#[test]
fn should_leave_nested_heading_quotes_and_lists_to_the_full_converter() {
    let options = ConversionOptions::default();
    for html in [
        "<h2><blockquote>One</blockquote>items</h2>",
        "<h2><ul><li>One</li></ul>items</h2>",
        "<h2><ol><li>One</li></ol>items</h2>",
    ] {
        let (page, report) = html_to_markdown_rs::prescan::run(html);
        assert!(matches!(
            html_to_markdown_rs::tier1::run(&page, &report, &options),
            Err(html_to_markdown_rs::tier1::BailReason::Classifier)
        ));
    }
}

#[cfg(feature = "visitor")]
#[derive(Debug)]
struct HeadingVisitor {
    hook: &'static str,
    custom: bool,
    calls: usize,
}

#[cfg(feature = "visitor")]
impl HeadingVisitor {
    fn visit(&mut self, hook: &str) -> html_to_markdown_rs::visitor::VisitResult {
        use html_to_markdown_rs::visitor::VisitResult;
        if self.hook != hook {
            return VisitResult::Continue;
        }
        self.calls += 1;
        if self.custom {
            VisitResult::Custom("CUSTOM".to_owned())
        } else {
            VisitResult::Skip
        }
    }
}

#[cfg(feature = "visitor")]
impl html_to_markdown_rs::visitor::HtmlVisitor for HeadingVisitor {
    fn visit_blockquote(
        &mut self,
        _ctx: &html_to_markdown_rs::visitor::NodeContext,
        _content: &str,
        _depth: usize,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        self.visit("quote")
    }
    fn visit_list_start(
        &mut self,
        _ctx: &html_to_markdown_rs::visitor::NodeContext,
        _ordered: bool,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        self.visit("list")
    }
    fn visit_list_item(
        &mut self,
        _ctx: &html_to_markdown_rs::visitor::NodeContext,
        _ordered: bool,
        _marker: &str,
        _text: &str,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        self.visit("item")
    }
    fn visit_list_end(
        &mut self,
        _ctx: &html_to_markdown_rs::visitor::NodeContext,
        _ordered: bool,
        _output: &str,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        self.visit("end")
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_honor_heading_quote_and_list_visitor_overrides() {
    use std::sync::{Arc, Mutex};
    for (hook, html) in [
        ("quote", "<h2><blockquote>One</blockquote>items</h2>"),
        ("quote", "<h2>before<blockquote>One</blockquote>after</h2>"),
        ("list", "<h2><ul><li>One</li></ul>items</h2>"),
        ("item", "<h2><ul><li>One</li></ul>items</h2>"),
        ("end", "<h2><ul><li>One</li></ul>items</h2>"),
    ] {
        for custom in [false, true] {
            let visitor = Arc::new(Mutex::new(HeadingVisitor { hook, custom, calls: 0 }));
            let result = convert(
                html,
                ConversionOptions {
                    visitor: Some(visitor.clone()),
                    extract_metadata: false,
                    ..Default::default()
                },
            )
            .expect("visitor heading converts");
            let expected = if html.starts_with("<h2>before") {
                if custom {
                    "## before CUSTOM after\n"
                } else {
                    "## before after\n"
                }
            } else {
                match (hook, custom) {
                    ("list", true) => "## CUSTOMOne items\n",
                    ("end", true) => "## One CUSTOM items\n",
                    (_, true) => "## CUSTOM items\n",
                    (_, false) => "## items\n",
                }
            };
            assert_eq!(result.content.as_deref(), Some(expected), "{hook}/{custom}");
            assert_eq!(visitor.lock().expect("visitor state").calls, 1);
        }
    }
}

#[test]
fn should_keep_heading_quotes_and_lists_in_table_cells_on_the_fast_scanner() {
    for cell in [
        "<h2>a<blockquote>b</blockquote></h2>",
        "<h2>a<ul><li>b</li></ul></h2>",
        "<h2>a<ol><li>b</li></ol></h2>",
    ] {
        let html = format!("<table><tr><td>{cell}</td></tr></table>");
        for br_in_tables in [false, true] {
            let options = ConversionOptions {
                extract_metadata: false,
                br_in_tables,
                tier_strategy: TierStrategy::Tier2,
                ..Default::default()
            };
            let expected = convert(&html, options.clone()).expect("table heading converts").content;
            let (page, report) = html_to_markdown_rs::prescan::run(&html);
            let scanned = html_to_markdown_rs::tier1::run(&page, &report, &options)
                .expect("heading block already has an inline table-cell path");
            assert_eq!(Some(scanned), expected, "{cell}: br_in_tables={br_in_tables}");
        }
    }
}
