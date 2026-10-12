#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, WhitespaceMode, convert, prescan, tier1};

fn check(html: &str, expected: &str, options: &ConversionOptions) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            html,
            ConversionOptions {
                tier_strategy,
                ..options.clone()
            },
        )
        .expect("block boundaries convert");
        assert_eq!(result.content.as_deref(), Some(expected), "{tier_strategy:?}: {html}");
    }
}

fn check_scanner(html: &str, expected: &str, options: &ConversionOptions) {
    let (page, report) = prescan::run(html);
    assert_eq!(
        tier1::run(&page, &report, options).expect("scanner converts"),
        expected,
        "{html}"
    );
    assert_eq!(
        tier1::run(html, &PrescanReport::default(), options).expect("raw scanner converts"),
        expected,
        "raw: {html}"
    );
}

#[test]
fn should_keep_div_boundaries_in_strict_headings() {
    let options = ConversionOptions {
        extract_metadata: false,
        whitespace_mode: WhitespaceMode::Strict,
        ..Default::default()
    };
    for (html, expected) in [
        ("<h2>a<div>b</div>c</h2>", "## a b c\n"),
        ("<h2><div>a</div><div>b</div></h2>", "## a b\n"),
    ] {
        check(html, expected, &options);
        check_scanner(html, expected, &options);
    }
}

#[test]
fn should_keep_div_boundaries_inside_heading_code() {
    let options = ConversionOptions {
        extract_metadata: false,
        ..Default::default()
    };
    for (html, expected) in [
        (
            "<h1>one <code><div>y</div><div>z</div></code> two</h1>",
            "# one `y z ` two\n",
        ),
        ("<h1>one<code>x<div>y</div>z</code>two</h1>", "# one`x y z`two\n"),
    ] {
        check(html, expected, &options);
        let (page, report) = prescan::run(html);
        assert!(matches!(
            tier1::run(&page, &report, &options),
            Err(tier1::BailReason::Classifier)
        ));
        assert!(matches!(
            tier1::run(html, &PrescanReport::default(), &options),
            Err(tier1::BailReason::Classifier)
        ));
    }
}

#[test]
fn should_hand_link_descriptions_to_the_full_converter() {
    let options = ConversionOptions {
        extract_metadata: false,
        ..Default::default()
    };
    let html = "<a href='/x'>a<dd>b</dd></a>";
    check(html, "[a b](/x)\n", &options);
    let (page, report) = prescan::run(html);
    assert!(matches!(
        tier1::run(&page, &report, &options),
        Err(tier1::BailReason::Classifier)
    ));
    assert!(matches!(
        tier1::run(html, &PrescanReport::default(), &options),
        Err(tier1::BailReason::Classifier)
    ));
}

#[test]
fn should_separate_a_stray_description_after_a_list_item() {
    let options = ConversionOptions {
        extract_metadata: false,
        ..Default::default()
    };
    let html = "<ul><li>a</li><dd>b</dd></ul>";
    check(html, "- a\n\nb\n", &options);
    check_scanner(html, "- a\n\nb\n", &options);
}

#[test]
fn should_keep_source_whitespace_at_strict_div_edges() {
    let options = ConversionOptions {
        extract_metadata: false,
        whitespace_mode: WhitespaceMode::Strict,
        ..Default::default()
    };
    for (html, expected) in [
        ("<h2>a<div> b</div>c</h2>", "## a b c\n"),
        ("<h2>a<div>  b  </div>c</h2>", "## a  b  c\n"),
        ("<h2>a <div>b</div> c</h2>", "## a b  c\n"),
    ] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            let result = convert(
                html,
                ConversionOptions {
                    tier_strategy,
                    ..options.clone()
                },
            )
            .expect("strict heading converts");
            assert_eq!(result.content.as_deref(), Some(expected), "{tier_strategy:?}: {html}");
        }
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_report_restored_div_separators_to_the_inline_code_visitor() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct CodeVisitor {
        calls: Vec<String>,
    }

    impl HtmlVisitor for CodeVisitor {
        fn visit_code_inline(&mut self, _ctx: &html_to_markdown_rs::NodeContext, code: &str) -> VisitResult {
            self.calls.push(code.to_owned());
            VisitResult::Continue
        }
    }

    for whitespace_mode in [WhitespaceMode::Normalized, WhitespaceMode::Strict] {
        let visitor = Arc::new(Mutex::new(CodeVisitor::default()));
        let result = convert(
            "<h1>one <code><div>y</div><div>z</div></code> two</h1>",
            ConversionOptions {
                extract_metadata: false,
                whitespace_mode,
                visitor: Some(visitor.clone()),
                ..Default::default()
            },
        )
        .expect("visitor heading converts");
        assert_eq!(result.content.as_deref(), Some("# one `y z ` two\n"));
        assert_eq!(visitor.lock().expect("visitor state").calls, ["y z "]);
    }
}
