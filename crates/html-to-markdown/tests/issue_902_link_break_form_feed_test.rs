//! A form feed at a link edge belongs outside the link even beside an authored break (#902). ~keep
#![cfg(feature = "testkit")]
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)]

use html_to_markdown_rs::{ConversionOptions, ConversionResult, NewlineStyle, OutputFormat, TierStrategy, convert};

#[cfg(feature = "visitor")]
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
#[cfg(feature = "visitor")]
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Trace {
    starts: usize,
    ends: usize,
    breaks: usize,
    links: Vec<(String, String, Option<String>)>,
}

#[cfg(feature = "visitor")]
#[derive(Debug, Default)]
struct Observer {
    trace: Trace,
}

#[cfg(feature = "visitor")]
impl HtmlVisitor for Observer {
    fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
        self.trace.starts += usize::from(context.tag_name == "a");
        VisitResult::Continue
    }

    fn visit_element_end(&mut self, context: &NodeContext, _: &str) -> VisitResult {
        self.trace.ends += usize::from(context.tag_name == "a");
        VisitResult::Continue
    }

    fn visit_link(&mut self, _: &NodeContext, href: &str, label: &str, title: Option<&str>) -> VisitResult {
        self.trace
            .links
            .push((href.into(), label.into(), title.map(str::to_owned)));
        VisitResult::Continue
    }

    fn visit_line_break(&mut self, _: &NodeContext) -> VisitResult {
        self.trace.breaks += 1;
        VisitResult::Continue
    }
}

fn configurations() -> Vec<ConversionOptions> {
    let mut routes = Vec::new();
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for newline_style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
            for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
                for extract_metadata in metadata_modes() {
                    routes.push(ConversionOptions {
                        output_format,
                        newline_style,
                        tier_strategy,
                        extract_metadata: *extract_metadata,
                        ..Default::default()
                    });
                }
            }
        }
    }
    assert_eq!(routes.len(), 12 * metadata_modes().len());
    routes
}

const fn metadata_modes() -> &'static [bool] {
    if cfg!(feature = "metadata") {
        &[false, true]
    } else {
        &[false]
    }
}

const fn visitor_modes() -> &'static [bool] {
    if cfg!(feature = "visitor") {
        &[false, true]
    } else {
        &[false]
    }
}

fn convert_case(html: &str, options: ConversionOptions, observed: bool) -> (ConversionResult, Option<Trace>) {
    #[cfg(feature = "visitor")]
    if observed {
        let listener = Arc::new(Mutex::new(Observer::default()));
        let result = convert(
            html,
            ConversionOptions {
                visitor: Some(listener.clone()),
                ..options
            },
        )
        .expect("observed link conversion");
        let trace = listener.lock().expect("observer lock").trace.clone();
        return (result, Some(trace));
    }
    assert!(!observed);
    (convert(html, options).expect("link conversion"), None)
}

fn check_value(actual: &str, expected: &str, name: &str, failures: &mut Vec<String>) {
    if actual != expected {
        failures.push(format!("{name}: expected {expected:?}, got {actual:?}"));
    }
}

fn check_metadata(result: &ConversionResult, enabled: bool, label: &str, failures: &mut Vec<String>) {
    #[cfg(feature = "metadata")]
    if enabled {
        let metadata = &result.metadata;
        if metadata.links.len() != 1 {
            failures.push(format!("metadata link count: expected 1, got {}", metadata.links.len()));
        }
        if let Some(link) = metadata.links.first() {
            check_value(&link.href, "/menu", "metadata href", failures);
            check_value(&link.text, label, "metadata label", failures);
        }
        return;
    }
    assert!(!enabled);
    #[cfg(feature = "metadata")]
    if !result.metadata.links.is_empty() {
        failures.push(format!(
            "metadata link count: expected 0, got {}",
            result.metadata.links.len()
        ));
    }
    let _ = (result, label, failures);
}

fn check_case(name: &str, html: &str, spaces: &str, backslash: &str, labels: (&str, &str), policy: (usize, bool)) {
    let (breaks, fast_control) = policy;
    let mut failures = Vec::new();
    let mut rows = 0;
    for options in configurations() {
        let spaces_marker =
            options.output_format == OutputFormat::Markdown && options.newline_style == NewlineStyle::Spaces;
        let (expected, label) = if spaces_marker {
            (spaces, labels.0)
        } else {
            (backslash, labels.1)
        };
        let fast = fast_control && options.tier_strategy == TierStrategy::Tier1;
        for observed in visitor_modes() {
            let (result, trace) = convert_case(html, options.clone(), *observed);
            check_value(
                result.content.as_deref().expect("link content"),
                expected,
                "content",
                &mut failures,
            );
            check_metadata(&result, options.extract_metadata && !fast, label, &mut failures);
            if let Some(trace) = trace {
                let expected_trace = if fast {
                    Trace::default()
                } else {
                    Trace {
                        starts: 1,
                        ends: 1,
                        breaks,
                        links: vec![("/menu".into(), label.into(), None)],
                    }
                };
                if trace != expected_trace {
                    failures.push(format!("callbacks: expected {expected_trace:?}, got {trace:?}"));
                }
            }
            rows += 1;
        }
    }
    let expected = 12 * metadata_modes().len() * visitor_modes().len();
    eprintln!("ROW_COUNT {name} actual={rows} expected={expected}");
    assert!(expected > 0);
    assert_eq!(rows, expected);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn should_move_a_form_feed_outside_a_link_before_its_leading_break() {
    check_case(
        "reported_default",
        "<p>Read<a href=\"/menu\">\u{c}<br>Menu</a>today</p>",
        "Read\u{c}[  \nMenu](/menu)today\n",
        "Read\u{c}[\\\nMenu](/menu)today\n",
        ("  \nMenu", "\\\nMenu"),
        (1, false),
    );
}

#[test]
fn should_keep_a_form_feed_before_a_link_break_past_a_hidden_span_in_a_quote() {
    check_case(
        "reported_hidden_quote",
        "<blockquote><p>Read<a href=\"/menu\">\u{c}<span hidden>omit</span><br>Menu</a>today</p></blockquote>",
        "> Read\u{c}[  \n> Menu](/menu)today\n",
        "> Read\u{c}[\\\n> Menu](/menu)today\n",
        ("  \nMenu", "\\\nMenu"),
        (1, false),
    );
}

#[test]
fn should_keep_the_existing_form_feed_boundary_when_the_label_has_no_break() {
    check_case(
        "no_break_control",
        "<p>Read<a href=\"/menu\">\u{c}Menu</a>today</p>",
        "Read\u{c}[Menu](/menu)today\n",
        "Read\u{c}[Menu](/menu)today\n",
        ("Menu", "Menu"),
        (0, false),
    );
}

#[test]
fn should_keep_an_ordinary_leading_link_break_without_adding_an_edge() {
    check_case(
        "ordinary_break_control",
        "<p>Read<a href=\"/menu\"><br>Menu</a>today</p>",
        "Read[  \nMenu](/menu)today\n",
        "Read[\\\nMenu](/menu)today\n",
        ("  \nMenu", "\\\nMenu"),
        (1, true),
    );
}

#[test]
fn should_keep_a_form_feed_inside_label_text_beside_an_internal_break() {
    check_case(
        "internal_form_feed_control",
        "<p>Read<a href=\"/menu\">Me\u{c}<br>nu</a>today</p>",
        "Read[Me\u{c}  \nnu](/menu)today\n",
        "Read[Me\u{c}\\\nnu](/menu)today\n",
        ("Me\u{c}  \nnu", "Me\u{c}\\\nnu"),
        (1, true),
    );
}
