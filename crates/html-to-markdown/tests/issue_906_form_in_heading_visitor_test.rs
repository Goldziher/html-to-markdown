#![allow(missing_docs)]

//! A form inside a heading reaches the visitor, as a form in any other place does (#906).

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

#[cfg(feature = "testkit")]
const ROUTES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2];
#[cfg(not(feature = "testkit"))]
const ROUTES: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];

const BREAD: &str = r#"Bread <form action="/order">Order now</form> Cheese"#;

fn heading(level: usize) -> String {
    format!("<h{level}>{BREAD}</h{level}>")
}

fn marks(level: usize) -> String {
    "#".repeat(level)
}

/// Converts `html` in Markdown and Djot, on each route, with metadata on and off. Gives one line
/// for each conversion whose output is not `expected`.
fn mismatches(html: &str, expected: &str, options: &ConversionOptions) -> Vec<String> {
    let mut failures = Vec::new();
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for tier_strategy in ROUTES {
            for extract_metadata in [false, true] {
                let options = ConversionOptions {
                    output_format,
                    tier_strategy,
                    extract_metadata,
                    ..options.clone()
                };
                let content = convert(html, options).expect("form in a heading converts").content;
                if content.as_deref() != Some(expected) {
                    failures.push(format!(
                        "{output_format:?}/{tier_strategy:?}/metadata={extract_metadata}: {html}: {content:?}, expected {expected:?}"
                    ));
                }
            }
        }
    }
    failures
}

#[test]
fn should_not_change_a_form_in_a_heading_with_no_visitor() {
    let mut kept = ConversionOptions::default();
    kept.preprocessing.remove_forms = false;
    let mut failures = Vec::new();
    for level in 1..=6 {
        let marks = marks(level);
        failures.extend(mismatches(
            &heading(level),
            &format!("{marks} Bread Cheese\n"),
            &ConversionOptions::default(),
        ));
        failures.extend(mismatches(
            &heading(level),
            &format!("{marks} Bread Order now Cheese\n"),
            &kept,
        ));
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[cfg(feature = "visitor")]
mod with_a_visitor {
    use super::{ConversionOptions, heading, marks, mismatches};
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct FormVisitor {
        result: VisitResult,
        actions: Vec<String>,
    }

    impl HtmlVisitor for FormVisitor {
        fn visit_form(&mut self, _ctx: &NodeContext, action: Option<&str>, _method: Option<&str>) -> VisitResult {
            self.actions.push(action.unwrap_or_default().to_owned());
            self.result.clone()
        }
    }

    /// Converts `html` with a visitor that gives `result` for each form. Checks the output and
    /// that the visitor saw the one form of the page in each conversion.
    fn assert_visited(html: &str, result: &VisitResult, expected: &str) {
        let recorder = Arc::new(Mutex::new(FormVisitor {
            result: result.clone(),
            actions: Vec::new(),
        }));
        let visitor: VisitorHandle = recorder.clone();
        let options = ConversionOptions {
            visitor: Some(visitor),
            ..Default::default()
        };
        let failures = mismatches(html, expected, &options);
        assert!(failures.is_empty(), "{result:?}: {failures:#?}");
        let actions = recorder.lock().expect("visitor lock").actions.clone();
        assert!(!actions.is_empty(), "{result:?}: {html}: the visitor saw no form");
        assert!(
            actions.iter().all(|action| action == "/order"),
            "{result:?}: {html}: {actions:?}"
        );
    }

    #[test]
    fn should_write_the_custom_text_of_the_visitor_for_a_form_in_a_heading() {
        let custom = VisitResult::Custom("[FORM REPLACED]".to_owned());
        for level in 1..=6 {
            let marks = marks(level);
            assert_visited(
                &heading(level),
                &custom,
                &format!("{marks} Bread [FORM REPLACED] Cheese\n"),
            );
        }
    }

    #[test]
    fn should_write_nothing_for_a_form_in_a_heading_that_the_visitor_skips() {
        for level in 1..=6 {
            let marks = marks(level);
            assert_visited(&heading(level), &VisitResult::Skip, &format!("{marks} Bread Cheese\n"));
        }
    }

    #[test]
    fn should_write_the_form_text_in_a_heading_when_the_visitor_continues() {
        for level in 1..=6 {
            let marks = marks(level);
            assert_visited(
                &heading(level),
                &VisitResult::Continue,
                &format!("{marks} Bread Order now Cheese\n"),
            );
        }
    }

    /// Outside a heading the custom text of a form is a block, so in a list item it starts on a
    /// line of its own below the marker.
    #[test]
    fn should_keep_the_custom_text_of_a_form_in_a_list_item() {
        let custom = VisitResult::Custom("[FORM REPLACED]".to_owned());
        assert_visited(
            r#"<ul><li><form action="/order">Order now</form></li></ul>"#,
            &custom,
            "-\n\n[FORM REPLACED]\n",
        );
    }
}
