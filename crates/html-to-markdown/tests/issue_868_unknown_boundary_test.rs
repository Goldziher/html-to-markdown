//! Unknown passthrough boundaries follow the full main whitespace policy. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn assert_routes(source: &str, expected: &str) -> usize {
    assert_routes_with_fast(source, expected, None)
}
fn assert_routes_with_fast(source: &str, expected: &str, fast: Option<&str>) -> usize {
    assert_eq!(
        convert(source, None).expect("default conversion").content.as_deref(),
        Some(expected),
        "{source}"
    );
    let mut rows = 1;
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for extract_metadata in [false, true] {
            let result = convert(
                source,
                ConversionOptions {
                    tier_strategy,
                    extract_metadata,
                    ..Default::default()
                },
            )
            .expect("conversion");
            let expected = if tier_strategy == TierStrategy::Tier1 {
                fast.unwrap_or(expected)
            } else {
                expected
            };
            assert_eq!(
                result.content.as_deref(),
                Some(expected),
                "{source} {tier_strategy:?} metadata={extract_metadata}"
            );
            rows += 1;
        }
    }
    rows
}

#[test]
fn should_merge_unknown_collapsible_boundaries_with_nbsp() {
    let mut rows = 0;
    for container in ["p", "h2"] {
        for tag in ["x-y", "mystery"] {
            for content in [
                "",
                "<!-- x -->",
                "<span></span>",
                " ",
                "  ",
                "\n",
                "&#9;",
                "&#13;",
                "\r",
            ] {
                for prefix in ["", " ", "\n", " \n"] {
                    for tail in ["&nbsp;", "\u{a0}"] {
                        let source = format!("<{container}>a{prefix}<{tag}>{content}</{tag}>{tail}b</{container}>");
                        rows += assert_routes(&source, if container == "h2" { "## a b\n" } else { "a b\n" });
                    }
                }
            }
        }
    }
    assert_eq!(rows, 2016);
}

#[cfg(feature = "visitor")]
#[derive(Debug)]
struct UnknownBoundaryVisitor {
    mode: usize,
    starts: usize,
    ends: usize,
}
#[cfg(feature = "visitor")]
impl html_to_markdown_rs::visitor::HtmlVisitor for UnknownBoundaryVisitor {
    fn visit_element_start(
        &mut self,
        ctx: &html_to_markdown_rs::visitor::NodeContext,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        use html_to_markdown_rs::visitor::VisitResult;
        if ctx.tag_name == "x-y" {
            self.starts += 1;
            if self.mode == 1 {
                return VisitResult::Custom("V ".into());
            }
            if self.mode == 3 {
                return VisitResult::Skip;
            }
        }
        if self.mode == 4 && ctx.tag_name == "span" {
            return VisitResult::Custom("V ".into());
        }
        VisitResult::Continue
    }
    fn visit_element_end(
        &mut self,
        ctx: &html_to_markdown_rs::visitor::NodeContext,
        _out: &str,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        use html_to_markdown_rs::visitor::VisitResult;
        if ctx.tag_name == "x-y" {
            self.ends += 1;
            if self.mode == 2 {
                return VisitResult::Custom("V ".into());
            }
        }
        VisitResult::Continue
    }
    fn visit_custom_element(
        &mut self,
        _ctx: &html_to_markdown_rs::visitor::NodeContext,
        _tag: &str,
        _raw: &str,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        use html_to_markdown_rs::visitor::VisitResult;
        match self.mode {
            5 => VisitResult::Custom("Q ".into()),
            6 => VisitResult::Skip,
            7 => VisitResult::PreserveHtml,
            _ => VisitResult::Continue,
        }
    }
}
#[cfg(feature = "visitor")]
fn assert_unknown_observer(source: &str, mode: usize, expected: &str, starts: usize, ends: usize) -> usize {
    use std::sync::{Arc, Mutex};
    let mut rows = 0;
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for extract_metadata in [false, true] {
            let visitor = Arc::new(Mutex::new(UnknownBoundaryVisitor {
                mode,
                starts: 0,
                ends: 0,
            }));
            let result = convert(
                source,
                ConversionOptions {
                    tier_strategy,
                    extract_metadata,
                    include_document_structure: true,
                    visitor: Some(visitor.clone()),
                    ..Default::default()
                },
            )
            .expect("observed unknown output");
            let expected = if tier_strategy == TierStrategy::Tier1 && source.starts_with("<pre>") {
                "```\na  \u{a0}b\n```\n"
            } else {
                expected
            };
            assert_eq!(
                result.content.as_deref(),
                Some(expected),
                "{source} observer {mode} {tier_strategy:?}"
            );
            if let Some(document) = result.document {
                assert_eq!(document.source_format.as_deref(), Some("html"));
                assert!(!document.nodes.is_empty());
            }
            let observed = {
                let observer = visitor.lock().expect("observer calls");
                (observer.starts, observer.ends)
            };
            assert_eq!(
                observed,
                if tier_strategy == TierStrategy::Tier1 && source.starts_with("<pre>") {
                    (0, 0)
                } else {
                    (starts, ends)
                }
            );
            rows += 1;
        }
    }
    rows
}
#[cfg(feature = "visitor")]
#[test]
fn should_preserve_unknown_observer_replacements_and_table_measurements() {
    let mut rows = 0;
    for (source, expected) in [
        (
            "<p>a <x-y></x-y>&nbsp;b</p>",
            [
                ("a b\n", 1, 1),
                ("a V  b\n", 1, 1),
                ("a V  b\n", 1, 1),
                ("a  b\n", 1, 0),
                ("a b\n", 1, 1),
            ],
        ),
        (
            "<p>a<x-y> </x-y>&nbsp;b</p>",
            [
                ("a b\n", 1, 1),
                ("aV  b\n", 1, 1),
                ("aV  b\n", 1, 1),
                ("a b\n", 1, 0),
                ("a b\n", 1, 1),
            ],
        ),
        (
            "<p>a <x-y><span></span></x-y>&nbsp;b</p>",
            [
                ("a b\n", 1, 1),
                ("a V  b\n", 1, 1),
                ("a V  b\n", 1, 1),
                ("a  b\n", 1, 0),
                ("a V  b\n", 1, 1),
            ],
        ),
        (
            "<h2>a <x-y></x-y>&nbsp;b</h2>",
            [
                ("## a b\n", 1, 1),
                ("## a V  b\n", 1, 1),
                ("## a V  b\n", 1, 1),
                ("## a  b\n", 1, 0),
                ("## a b\n", 1, 1),
            ],
        ),
        (
            "<table><tr><td>a <x-y></x-y>&nbsp;b</td></tr></table>",
            [
                ("| a b  |\n| ---- |\n", 3, 3),
                ("| a V  b |\n| ---- |\n", 3, 3),
                ("| a V  b |\n| ---- |\n", 3, 3),
                ("| a  b |\n| ---- |\n", 3, 0),
                ("| a b  |\n| ---- |\n", 3, 3),
            ],
        ),
        (
            "<pre>a <x-y> </x-y>&nbsp;b</pre>",
            [
                ("```\na  b\n```\n", 1, 1),
                ("```\na V  b\n```\n", 1, 1),
                ("```\na V  b\n```\n", 1, 1),
                ("```\na  b\n```\n", 1, 0),
                ("```\na  b\n```\n", 1, 1),
            ],
        ),
    ] {
        for (mode, (content, starts, ends)) in expected.into_iter().enumerate() {
            rows += assert_unknown_observer(source, mode, content, starts, ends);
        }
    }
    assert_eq!(rows, 180);
}

#[test]
fn should_reuse_existing_fast_bail_for_unknown_unicode_space_boundary() {
    use html_to_markdown_rs::{prescan, tier1};
    let source = "<p>a <x-y></x-y>&nbsp;b</p>";
    let (cleaned, report) = prescan::run(source);
    assert!(matches!(
        tier1::run(
            cleaned.as_ref(),
            &report,
            &ConversionOptions {
                extract_metadata: false,
                ..Default::default()
            }
        ),
        Err(tier1::BailReason::UnicodeSpaceAfterSpace)
    ));
    let source = "<h2>a\n<x-y></x-y>&nbsp;b</h2>";
    let (cleaned, report) = prescan::run(source);
    assert_eq!(
        tier1::run(
            cleaned.as_ref(),
            &report,
            &ConversionOptions {
                extract_metadata: false,
                ..Default::default()
            }
        )
        .expect("supported heading"),
        "## a b\n"
    );
}

#[test]
fn should_preserve_visible_and_owned_unknown_output_boundaries() {
    let mut rows = 0;
    for (source, expected) in [
        ("<p>a <x-y>x </x-y>&nbsp;b</p>", "a x b\n"),
        ("<p>a\n<x-y><br></x-y>&nbsp;b</p>", "a  b\n"),
        ("<p>a\n<x-y><img src=\"x\" alt=\"y\"></x-y>&nbsp;b</p>", "a ![y](x) b\n"),
        ("<p>a\n<x-y><code>\n</code></x-y>&nbsp;b</p>", "a  b\n"),
        ("<p><code>a<x-y> </x-y>&nbsp;b</code></p>", "`a \u{a0}b`\n"),
        ("<p>a <font> </font>&nbsp;b</p>", "a b\n"),
        ("<p>a\n<span>\n</span>&nbsp;b</p>", "a  b\n"),
    ] {
        rows += assert_routes(source, expected);
    }
    rows += assert_routes_with_fast("<p>a<x-y>&nbsp;</x-y>&nbsp;b</p>", "a b\n", Some("a\u{a0} b\n"));
    assert_eq!(rows, 56);
}
#[cfg(feature = "visitor")]
#[test]
fn should_preserve_custom_unknown_callback_results() {
    let mut rows = 0;
    for (source, values) in [
        (
            "<p>a <x-y></x-y>&nbsp;b</p>",
            ["a Q  b\n", "a  b\n", "a <x-y></x-y> b\n"],
        ),
        (
            "<h2>a <x-y></x-y>&nbsp;b</h2>",
            ["## a Q  b\n", "## a  b\n", "## a <x-y></x-y> b\n"],
        ),
    ] {
        for (index, expected) in values.into_iter().enumerate() {
            rows += assert_unknown_observer(source, index + 5, expected, 1, 1);
        }
    }
    assert_eq!(rows, 36);
}
