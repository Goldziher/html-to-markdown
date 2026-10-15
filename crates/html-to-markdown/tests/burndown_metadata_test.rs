#![allow(missing_docs)]

#[cfg(feature = "testkit")]
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn routes() -> Vec<TierStrategy> {
    vec![
        TierStrategy::Auto,
        TierStrategy::Tier2,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
    ]
}

fn content(html: &str, tier_strategy: TierStrategy, extract_metadata: bool) -> String {
    convert(
        html,
        ConversionOptions {
            tier_strategy,
            extract_metadata,
            ..Default::default()
        },
    )
    .expect("conversion succeeds")
    .content
    .expect("content is requested")
}

#[test]
fn should_extract_metadata_without_an_explicit_head_on_every_route() {
    for (html, field) in [
        (
            r#"<meta name="description" content="D"><p>one</p>"#,
            "meta-description: D",
        ),
        (
            r#"<link rel="canonical" href="https://e.org/"><p>one</p>"#,
            "canonical: https://e.org/",
        ),
        (
            r#"<head></head><meta name="description" content="D"><p>one</p>"#,
            "meta-description: D",
        ),
        (
            r#"<!doctype html><meta name="description" content="D"><p>one</p>"#,
            "meta-description: D",
        ),
        (
            r#"<html><meta name="description" content="D"><body><p>one</p></body></html>"#,
            "meta-description: D",
        ),
        (
            r#"<!-- c --><meta name="description" content="D"><p>one</p>"#,
            "meta-description: D",
        ),
    ] {
        for tier in routes() {
            assert_eq!(
                content(html, tier, true),
                format!("---\n{field}\n---\n\none\n"),
                "{tier:?}: {html}"
            );
            assert_eq!(content(html, tier, false), "one\n", "{tier:?}: {html}");
        }
    }
}

#[cfg(feature = "testkit")]
#[test]
fn should_extract_implicit_head_metadata_in_the_direct_fast_converter() {
    for html in [
        r#"<meta name="description" content="D"><p>one</p>"#,
        r#"<head></head><meta name="description" content="D"><p>one</p>"#,
        r#"<!-- <head> --><meta name="description" content="D"><p>one</p>"#,
    ] {
        let options = ConversionOptions::default();
        let result = html_to_markdown_rs::tier1::run(html, &PrescanReport::default(), &options)
            .expect("the fast converter supports metadata and paragraphs");
        assert_eq!(result, "---\nmeta-description: D\n---\n\none\n", "{html}");
    }
}

#[test]
fn should_ignore_commented_head_and_body_tags_when_classifying_titles() {
    for html in [
        "<!-- <head> --><title>T</title><p>one</p>",
        "<head></head><!-- <body> --><title>T</title><p>one</p>",
        "<!-- <header> --><title>T</title><p>one</p>",
    ] {
        for tier in routes() {
            assert_eq!(
                content(html, tier, true),
                "---\ntitle: T\n---\n\none\n",
                "{tier:?}: {html}"
            );
            assert_eq!(content(html, tier, false), "one\n", "{tier:?}: {html}");
        }
    }
}

#[test]
fn should_not_collect_metadata_after_inert_elements_that_start_the_body() {
    for html in [
        r#"<template><p>inert</p></template><meta name="description" content="D"><p>body</p>"#,
        r#"<head></head><noscript></noscript><meta name="description" content="D"><p>body</p>"#,
    ] {
        for tier in routes() {
            let actual = content(html, tier, true);
            assert_eq!(actual, "body\n", "{tier:?}: {html}");
        }
        #[cfg(feature = "testkit")]
        match html_to_markdown_rs::tier1::run(html, &PrescanReport::default(), &ConversionOptions::default()) {
            Ok(output) => assert_eq!(output, "body\n", "{html}"),
            Err(html_to_markdown_rs::tier1::BailReason::Classifier) => {}
            Err(reason) => panic!("unexpected fast-route failure for {html}: {reason:?}"),
        }
    }
}

#[test]
fn should_keep_office_head_metadata_only_before_real_body_content() {
    let html = "<html><head><o:p></o:p><meta name='Generator' content='Word'><title>Document</title></head><body><p>Visible</p></body></html>";
    let after_body = "<head><o:p></o:p><p>Visible</p><meta name='Generator' content='Word'></head>";
    for tier in routes() {
        assert_eq!(
            content(html, tier, true),
            "---\nmeta-Generator: Word\ntitle: Document\n---\n\nVisible\n",
            "{tier:?}"
        );
        let body_metadata = content(after_body, tier, true);
        assert!(!body_metadata.starts_with("---\n"), "{tier:?}: {body_metadata:?}");
        assert!(!body_metadata.contains("meta-Generator"), "{tier:?}: {body_metadata:?}");
    }
}

#[test]
fn should_keep_an_image_that_implicitly_ends_the_head() {
    for html in [
        r#"<head><img src="p.gif" alt="A1"></head><p>End</p>"#,
        r#"<html><head><img src="p.gif" alt="A1"></head><body><p>End</p></body></html>"#,
    ] {
        for tier in routes() {
            for metadata in [false, true] {
                assert_eq!(
                    content(html, tier, metadata),
                    "![A1](p.gif)\n\nEnd\n",
                    "{tier:?}: {html}"
                );
            }
        }
    }
}

#[test]
fn should_separate_front_matter_from_bare_body_text_on_every_route() {
    for html in [
        "<head><title>T</title></head>tail",
        "<title>T</title>tail",
        "<html><head><title>T</title></head><body>tail</body></html>",
    ] {
        for tier in routes() {
            assert_eq!(
                content(html, tier, true),
                "---\ntitle: T\n---\n\ntail\n",
                "{tier:?}: {html}"
            );
            assert_eq!(content(html, tier, false), "tail\n", "{tier:?}: {html}");
        }
    }
}

#[test]
fn should_preserve_table_section_measurement_depth() {
    let sectioned = "<table><thead><tr><th>HEADER</th></tr></thead><tbody><tr><td>BODY</td></tr></tbody></table>";
    let direct = "<table><tr><th>HEADER</th></tr><tr><td>BODY</td></tr></table>";
    for max_depth in 3..=6 {
        let options = ConversionOptions {
            max_depth: Some(max_depth),
            tier_strategy: TierStrategy::Tier2,
            ..Default::default()
        };
        let actual = convert(sectioned, options.clone())
            .expect("conversion succeeds")
            .content;
        let expected = convert(direct, options).expect("conversion succeeds").content;
        assert_eq!(actual, expected, "max_depth={max_depth}");
        if max_depth == 6 {
            assert!(actual.as_deref().is_some_and(|text| text.contains("BODY")));
        }
    }
}

#[cfg(feature = "visitor")]
mod visitor {
    use super::*;
    use html_to_markdown_rs::{
        HiddenContent,
        visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle},
    };
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct SkipInert {
        skipped: &'static str,
        seen: Vec<String>,
    }

    impl HtmlVisitor for SkipInert {
        fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
            self.seen.push(ctx.tag_name.to_string());
            if ctx.tag_name == self.skipped {
                VisitResult::Skip
            } else {
                VisitResult::Continue
            }
        }
    }

    #[test]
    fn should_preserve_kept_inert_content_with_a_continuing_visitor() {
        let pages = [
            "<x-note>z</x-note><template><p>Rye.</p></template><p>End.</p>",
            "<div><p>z</div><template><p>Rye.</p></template><p>End.</p>",
            "<p>z</p><template><ul><li>Rye.</li></ul></template><p>End.</p>",
            "<table><tr><th>Grain</th></tr><template><tr><td>Rye.</td></tr></template></table>",
            "<table><tr><td><template><p>Rye.</p></template></td></tr></table>",
        ];
        for html in pages {
            let recorder = Arc::new(Mutex::new(SkipInert {
                skipped: "",
                seen: Vec::new(),
            }));
            let visitor: VisitorHandle = recorder.clone();
            let options = ConversionOptions {
                hidden_content: HiddenContent::All,
                ..Default::default()
            };
            let expected = convert(html, options.clone()).expect("conversion succeeds").content;
            assert!(
                expected.as_deref().is_some_and(|text| text.contains("Rye.")),
                "{html}: {expected:?}"
            );
            let actual = convert(
                html,
                ConversionOptions {
                    visitor: Some(visitor),
                    ..options
                },
            )
            .expect("visitor conversion succeeds")
            .content;
            assert_eq!(actual, expected, "{html}");
            assert_eq!(
                recorder
                    .lock()
                    .expect("visitor lock")
                    .seen
                    .iter()
                    .filter(|tag| *tag == "template")
                    .count(),
                if html.contains("<td><template>") { 2 } else { 1 },
                "{html}"
            );
        }
    }

    #[test]
    fn should_keep_all_hidden_content_in_plain_output_with_a_visitor() {
        let visitor: VisitorHandle = Arc::new(Mutex::new(SkipInert {
            skipped: "",
            seen: Vec::new(),
        }));
        let options = ConversionOptions {
            output_format: html_to_markdown_rs::OutputFormat::Plain,
            hidden_content: HiddenContent::All,
            visitor: Some(visitor),
            ..Default::default()
        };
        let result = convert("<p>A</p><template><p>B</p></template><p>C</p>", options)
            .expect("conversion succeeds")
            .content;
        assert_eq!(result.as_deref(), Some("A\n\nB\n\nC\n"));
    }

    #[derive(Debug)]
    struct InertDecision {
        decision: VisitResult,
        end_decision: VisitResult,
        starts: usize,
        ends: usize,
    }

    impl HtmlVisitor for InertDecision {
        fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
            if ctx.tag_name == "template" {
                self.starts += 1;
                self.decision.clone()
            } else {
                VisitResult::Continue
            }
        }

        fn visit_element_end(&mut self, ctx: &NodeContext, _: &str) -> VisitResult {
            if ctx.tag_name == "template" {
                self.ends += 1;
                self.end_decision.clone()
            } else {
                VisitResult::Continue
            }
        }
    }

    #[test]
    fn should_apply_visitor_decisions_to_a_template_around_table_rows() {
        let html = "<table><thead><tr><th>Grain</th></tr></thead><tbody><template><tr><td>Rye.</td></tr></template></tbody></table>";
        for (decision, expected, ends) in [
            (VisitResult::Continue, "| Grain |\n| ----- |\n| Rye.  |\n", 1),
            (VisitResult::Skip, "| Grain |\n| ----- |\n", 0),
            (
                VisitResult::Custom("CUSTOM\n".to_owned()),
                "| Grain |\n| ----- |\nCUSTOM\n",
                1,
            ),
        ] {
            let recorder = Arc::new(Mutex::new(InertDecision {
                decision,
                end_decision: VisitResult::Continue,
                starts: 0,
                ends: 0,
            }));
            let visitor: VisitorHandle = recorder.clone();
            let options = ConversionOptions {
                visitor: Some(visitor),
                hidden_content: HiddenContent::All,
                ..Default::default()
            };
            let actual = convert(html, options).expect("conversion succeeds").content;
            assert_eq!(actual.as_deref(), Some(expected));
            let observed = recorder.lock().expect("visitor lock");
            assert_eq!((observed.starts, observed.ends), (1, ends));
        }
    }

    #[test]
    fn should_keep_custom_output_when_all_table_rows_are_in_a_template() {
        let visitor: VisitorHandle = Arc::new(Mutex::new(InertDecision {
            decision: VisitResult::Custom("CUSTOM\n".to_owned()),
            end_decision: VisitResult::Continue,
            starts: 0,
            ends: 0,
        }));
        let options = ConversionOptions {
            visitor: Some(visitor),
            hidden_content: HiddenContent::All,
            ..Default::default()
        };
        let result = convert("<table><template><tr><td>Rye.</td></tr></template></table>", options)
            .expect("conversion succeeds")
            .content;
        assert_eq!(result.as_deref(), Some("CUSTOM\n"));
    }

    #[test]
    fn should_apply_end_decisions_without_stale_table_padding_or_rowspans() {
        let html = "<table><tr><th>Grain</th><th>Price</th></tr><template><tr><td rowspan=2>Rye.</td><td>1</td></tr></template><tr><td>Wheat</td><td>2</td></tr></table>";
        let control = "<table><tr><th>Grain</th><th>Price</th></tr><tr><td>Wheat</td><td>2</td></tr></table>";
        for output_format in [
            html_to_markdown_rs::OutputFormat::Markdown,
            html_to_markdown_rs::OutputFormat::Djot,
        ] {
            for compact_tables in [false, true] {
                for end_decision in [VisitResult::Skip, VisitResult::Custom("X\n".to_owned())] {
                    let visitor: VisitorHandle = Arc::new(Mutex::new(InertDecision {
                        decision: VisitResult::Continue,
                        end_decision: end_decision.clone(),
                        starts: 0,
                        ends: 0,
                    }));
                    let options = ConversionOptions {
                        output_format,
                        compact_tables,
                        hidden_content: HiddenContent::All,
                        ..Default::default()
                    };
                    let mut expected = convert(control, options.clone())
                        .expect("control conversion succeeds")
                        .content
                        .expect("content");
                    let actual = convert(
                        html,
                        ConversionOptions {
                            visitor: Some(visitor),
                            ..options
                        },
                    )
                    .expect("conversion succeeds")
                    .content
                    .expect("content");
                    if matches!(end_decision, VisitResult::Custom(_)) {
                        let row_start = expected.rfind("\n|").expect("control has a data row") + 1;
                        expected.insert_str(row_start, "X\n");
                    }
                    assert_eq!(
                        actual, expected,
                        "{output_format:?}, compact={compact_tables}, {end_decision:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn should_respect_the_original_metadata_boundary_with_kept_inert_visitors() {
        for (html, expected) in [
            (
                r#"<template><p>inert</p></template><meta name="description" content="D"><p>body</p>"#,
                "inert\n\nbody\n",
            ),
            (
                r#"<head></head><noscript></noscript><meta name="description" content="D"><p>body</p>"#,
                "body\n",
            ),
        ] {
            let visitor: VisitorHandle = Arc::new(Mutex::new(SkipInert {
                skipped: "",
                seen: Vec::new(),
            }));
            let options = ConversionOptions {
                visitor: Some(visitor),
                hidden_content: HiddenContent::All,
                ..Default::default()
            };
            assert_eq!(
                convert(html, options).expect("conversion succeeds").content.as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn should_preserve_selector_exclusions_with_a_kept_inert_visitor() {
        let html = "<p>A</p><template class=drop><p>Hidden.</p></template><noscript><p class=drop>Hidden.</p><p>B</p></noscript>";
        let visitor: VisitorHandle = Arc::new(Mutex::new(SkipInert {
            skipped: "",
            seen: Vec::new(),
        }));
        let options = ConversionOptions {
            hidden_content: HiddenContent::All,
            visitor: Some(visitor),
            exclude_selectors: vec![".drop".to_owned()],
            ..Default::default()
        };
        assert_eq!(
            convert(html, options).expect("conversion succeeds").content.as_deref(),
            Some("A\n\nB\n")
        );
    }

    #[test]
    fn should_let_the_visitor_skip_kept_noscript_and_template_elements() {
        let html = "<p>Bread.</p><noscript><p>Scripts.</p></noscript><template><p>Rye.</p></template><p>Cheese.</p>";
        for (skipped, expected) in [
            ("noscript", "Bread.\n\nRye.\n\nCheese.\n"),
            ("template", "Bread.\n\nScripts.\n\nCheese.\n"),
        ] {
            for tier in routes() {
                let recorder = Arc::new(Mutex::new(SkipInert {
                    skipped,
                    seen: Vec::new(),
                }));
                let visitor: VisitorHandle = recorder.clone();
                let options = ConversionOptions {
                    tier_strategy: tier,
                    hidden_content: HiddenContent::All,
                    visitor: Some(visitor),
                    ..Default::default()
                };
                let actual = convert(html, options).expect("conversion succeeds").content;
                assert_eq!(actual.as_deref(), Some(expected), "{tier:?}: skip {skipped}");
                let seen = recorder.lock().expect("visitor lock");
                assert_eq!(seen.seen.iter().filter(|tag| *tag == skipped).count(), 1);
            }
        }
    }
    #[derive(Debug, Default)]
    struct InertDepths {
        wrappers: Vec<usize>,
        rows: Vec<usize>,
        spans: Vec<usize>,
    }

    impl HtmlVisitor for InertDepths {
        fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
            match ctx.tag_name.as_ref() {
                "template" => self.wrappers.push(ctx.depth),
                "span" => self.spans.push(ctx.depth),
                _ => {}
            }
            VisitResult::Continue
        }

        fn visit_table_row(&mut self, ctx: &NodeContext, cells: &[String], _: bool) -> VisitResult {
            if cells.iter().any(|cell| cell == "Rye.") {
                self.rows.push(ctx.depth);
            }
            VisitResult::Continue
        }
    }

    #[test]
    fn should_report_row_depth_between_its_inert_wrapper_and_cell_content() {
        for html in [
            "<table><tr><th>Grain</th></tr><template><tr><td><span>Rye.</span></td></tr></template></table>",
            "<table><thead><tr><th>Grain</th></tr></thead><tbody><template><tr><td><span>Rye.</span></td></tr></template></tbody></table>",
        ] {
            let recorder = Arc::new(Mutex::new(InertDepths::default()));
            let visitor: VisitorHandle = recorder.clone();
            let options = ConversionOptions {
                visitor: Some(visitor),
                hidden_content: HiddenContent::All,
                ..Default::default()
            };
            let actual = convert(html, options)
                .expect("conversion succeeds")
                .content
                .expect("content");
            assert!(actual.contains("Rye."), "{actual}");
            let depths = recorder.lock().expect("visitor lock");
            assert_eq!(
                (depths.wrappers.len(), depths.rows.len(), depths.spans.len()),
                (1, 1, 2),
                "{html}"
            );
            assert!(depths.wrappers[0] < depths.rows[0], "{html}: {depths:?}");
            assert!(
                depths.spans.iter().all(|depth| depths.rows[0] < *depth),
                "{html}: {depths:?}"
            );
        }
    }

    #[test]
    fn should_keep_inert_lookalikes_opaque_to_a_continuing_visitor() {
        for name in ["plaintext", "noembed", "noframes"] {
            for inert in ["template", "noscript"] {
                for body in [
                    format!("<{inert}><p>x</p></{inert}>"),
                    format!("<table><tr><th>A</th></tr><{inert}><tr><td>x</td></tr></{inert}></table>"),
                ] {
                    let html = format!("<p>see</p><{name}>{body}</{name}><p>after</p>");
                    let visitor: VisitorHandle = Arc::new(Mutex::new(SkipInert {
                        skipped: "",
                        seen: Vec::new(),
                    }));
                    let options = ConversionOptions {
                        visitor: Some(visitor),
                        hidden_content: HiddenContent::All,
                        ..Default::default()
                    };
                    let actual = convert(&html, options).expect("conversion succeeds").content;
                    let expected = super::content(&html, TierStrategy::Tier2, false);
                    assert_eq!(actual.as_deref(), Some(expected.as_str()), "{html}");
                    assert!(
                        !actual.as_deref().unwrap_or_default().contains('x'),
                        "{html}: {actual:?}"
                    );
                }
            }
        }
    }
}
