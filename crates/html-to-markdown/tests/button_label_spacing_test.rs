#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

fn convert_as(html: &str, output_format: OutputFormat, tier_strategy: TierStrategy) -> String {
    let options = ConversionOptions {
        output_format,
        tier_strategy,
        ..Default::default()
    };
    convert(html, Some(options)).unwrap().content.unwrap_or_default()
}

/// Markdown output on each route, and Djot and plain output, write `expected`.
fn assert_each_format(html: &str, expected: &str) {
    for (output_format, tier_strategy) in [
        (OutputFormat::Markdown, TierStrategy::Auto),
        (OutputFormat::Markdown, TierStrategy::Tier1),
        (OutputFormat::Markdown, TierStrategy::Tier2),
        (OutputFormat::Djot, TierStrategy::Auto),
        (OutputFormat::Plain, TierStrategy::Auto),
    ] {
        assert_eq!(
            convert_as(html, output_format, tier_strategy),
            expected,
            "{html} {output_format:?} {tier_strategy:?}"
        );
    }
}

#[test]
fn should_separate_buttons_that_stand_side_by_side() {
    assert_each_format(
        "<p><button>All Methods</button><button>Instance Methods</button><button>Concrete Methods</button></p>",
        "All Methods Instance Methods Concrete Methods\n",
    );
}

#[test]
fn should_separate_a_button_from_the_words_beside_it() {
    assert_each_format("<p>Press<button>Save</button>now.</p>", "Press Save now.\n");
}

#[test]
fn should_keep_the_white_space_of_the_source_around_a_button() {
    assert_each_format("<p>Press <button>Save</button> now.</p>", "Press Save now.\n");
}

#[test]
fn should_write_no_space_before_punctuation_after_a_button() {
    assert_each_format("<p>Press <button>Save</button>.</p>", "Press Save.\n");
}

/// Markdown and Djot output, with forms removed and with forms kept, write `expected` when the
/// visitor of the conversion answers `visit_button` with `answer`, or with no visitor.
#[cfg(feature = "visitor")]
fn assert_with_button_answer(html: &str, answer: Option<&html_to_markdown_rs::VisitResult>, expected: &str) {
    use html_to_markdown_rs::visitor::HtmlVisitor;
    use html_to_markdown_rs::{NodeContext, PreprocessingOptions, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct ButtonAnswer(VisitResult);
    impl HtmlVisitor for ButtonAnswer {
        fn visit_button(&mut self, _ctx: &NodeContext<'_>, _text: &str) -> VisitResult {
            self.0.clone()
        }
    }

    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for remove_forms in [true, false] {
            let options = ConversionOptions {
                output_format,
                preprocessing: PreprocessingOptions {
                    remove_forms,
                    ..Default::default()
                },
                visitor: answer
                    .cloned()
                    .map(|answer| -> html_to_markdown_rs::VisitorHandle { Arc::new(Mutex::new(ButtonAnswer(answer))) }),
                ..Default::default()
            };
            assert_eq!(
                convert(html, Some(options)).unwrap().content.unwrap_or_default(),
                expected,
                "{html} {answer:?} {output_format:?} remove_forms={remove_forms}"
            );
        }
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_not_separate_a_label_from_a_button_that_wrote_nothing() {
    assert_with_button_answer(
        "<p>a<label><!--c--><button></button><span>y</span></label>b</p>",
        None,
        "ayb\n",
    );
    assert_with_button_answer("<p>a<label><button></button><span>y</span></label>b</p>", None, "ayb\n");
    assert_with_button_answer(
        "<p>a<label><button> </button><span>y</span></label>b</p>",
        None,
        "ayb\n",
    );
    assert_with_button_answer("<p>a<label><button></button></label>b</p>", None, "ab\n");
    assert_with_button_answer("<p>a<button></button>b</p>", None, "ab\n");
    assert_with_button_answer("<p>a<button> </button>b</p>", None, "a b\n");
}

#[cfg(feature = "visitor")]
#[test]
fn should_not_separate_a_label_from_a_button_that_a_visitor_skipped() {
    use html_to_markdown_rs::VisitResult;
    assert_with_button_answer(
        "<p>a<label><button>x</button></label>b</p>",
        Some(&VisitResult::Skip),
        "ab\n",
    );
    assert_with_button_answer(
        "<p>a<label><button>x</button><span>y</span></label>b</p>",
        Some(&VisitResult::Skip),
        "ayb\n",
    );
    assert_with_button_answer("<p>a<button>x</button>b</p>", Some(&VisitResult::Skip), "ab\n");
}

#[cfg(feature = "visitor")]
#[test]
fn should_not_separate_a_label_from_the_text_a_visitor_wrote_for_a_button() {
    use html_to_markdown_rs::VisitResult;
    let q_answer = VisitResult::Custom("Q".to_string());
    let empty_answer = VisitResult::Custom(String::new());
    let q = || Some(&q_answer);
    let empty = || Some(&empty_answer);
    assert_with_button_answer("<p>a<label><button>x</button></label>b</p>", q(), "aQb\n");
    assert_with_button_answer(
        "<p>a<label><button>x</button><span>y</span></label>b</p>",
        q(),
        "aQyb\n",
    );
    assert_with_button_answer("<p>a<button>x</button>b</p>", q(), "aQb\n");
    assert_with_button_answer("<p>a<label><button>x</button></label>b</p>", empty(), "ab\n");
    assert_with_button_answer(
        "<p>a<label><button>x</button><span>y</span></label>b</p>",
        empty(),
        "ayb\n",
    );
    assert_with_button_answer("<p>a<button>x</button>b</p>", empty(), "ab\n");
}

#[cfg(feature = "visitor")]
#[test]
fn should_separate_a_label_from_a_button_that_wrote_its_own_text() {
    use html_to_markdown_rs::VisitResult;
    for answer in [None, Some(&VisitResult::Continue)] {
        assert_with_button_answer("<p>a<label><button>x</button></label>b</p>", answer, "a x b\n");
        assert_with_button_answer(
            "<p>a<label><button>x</button><span>y</span></label>b</p>",
            answer,
            "a x yb\n",
        );
        assert_with_button_answer("<p>a<button>x</button>b</p>", answer, "a x b\n");
    }
}

/// Markdown and Djot output, on the default route and on the full converter, with forms kept
/// and strict white space, write `expected` for `REWRITE_HTML` when the generic visitor hooks
/// answer `start` and `end` for the button. `calls` is the count of start, end and
/// `visit_button` calls for the button in each conversion.
#[cfg(feature = "visitor")]
fn assert_with_element_hooks(
    start: &html_to_markdown_rs::VisitResult,
    end: &html_to_markdown_rs::VisitResult,
    expected: &str,
    calls: [usize; 3],
) {
    use html_to_markdown_rs::visitor::HtmlVisitor;
    use html_to_markdown_rs::{NodeContext, PreprocessingOptions, VisitResult, WhitespaceMode};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct ElementHooks {
        start: VisitResult,
        end: VisitResult,
        calls: [usize; 3],
    }
    fn is_rewrite(ctx: &NodeContext<'_>) -> bool {
        ctx.attributes().get("id").map(String::as_str) == Some("rewrite")
    }
    impl HtmlVisitor for ElementHooks {
        fn visit_element_start(&mut self, ctx: &NodeContext<'_>) -> VisitResult {
            if !is_rewrite(ctx) {
                return VisitResult::Continue;
            }
            self.calls[0] += 1;
            self.start.clone()
        }
        fn visit_element_end(&mut self, ctx: &NodeContext<'_>, _output: &str) -> VisitResult {
            if !is_rewrite(ctx) {
                return VisitResult::Continue;
            }
            self.calls[1] += 1;
            self.end.clone()
        }
        fn visit_button(&mut self, ctx: &NodeContext<'_>, _text: &str) -> VisitResult {
            if is_rewrite(ctx) {
                self.calls[2] += 1;
            }
            VisitResult::Continue
        }
    }

    const REWRITE_HTML: &str = r#"<p>a<label><button id="rewrite">x</button>y</label>b</p>"#;
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            let hooks = Arc::new(Mutex::new(ElementHooks {
                start: start.clone(),
                end: end.clone(),
                calls: [0; 3],
            }));
            let options = ConversionOptions {
                output_format,
                tier_strategy,
                whitespace_mode: WhitespaceMode::Strict,
                preprocessing: PreprocessingOptions {
                    remove_forms: false,
                    ..Default::default()
                },
                visitor: Some(hooks.clone()),
                ..Default::default()
            };
            let label = format!("{start:?} {end:?} {output_format:?} {tier_strategy:?}");
            let written = convert(REWRITE_HTML, Some(options)).unwrap().content;
            assert_eq!(written.unwrap_or_default(), expected, "{label}");
            assert_eq!(hooks.lock().unwrap().calls, calls, "{label}");
        }
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_not_separate_a_label_from_a_button_that_an_end_hook_replaced() {
    use html_to_markdown_rs::VisitResult;
    let custom = |text: &str| VisitResult::Custom(text.to_string());
    assert_with_element_hooks(&VisitResult::Continue, &VisitResult::Skip, "ayb\n", [1, 1, 1]);
    assert_with_element_hooks(&VisitResult::Continue, &custom("Q"), "aQyb\n", [1, 1, 1]);
    assert_with_element_hooks(&VisitResult::Continue, &custom(""), "ayb\n", [1, 1, 1]);
    assert_with_element_hooks(&VisitResult::Continue, &custom(" "), "ayb\n", [1, 1, 1]);
}

#[cfg(feature = "visitor")]
#[test]
fn should_not_separate_a_label_from_a_button_that_a_start_hook_replaced() {
    use html_to_markdown_rs::VisitResult;
    assert_with_element_hooks(&VisitResult::Skip, &VisitResult::Continue, "ayb\n", [1, 0, 0]);
    assert_with_element_hooks(
        &VisitResult::Custom("Q".to_string()),
        &VisitResult::Continue,
        "aQyb\n",
        [1, 1, 0],
    );
}

#[cfg(feature = "visitor")]
#[test]
fn should_separate_a_label_from_a_button_that_the_element_hooks_kept() {
    use html_to_markdown_rs::VisitResult;
    use html_to_markdown_rs::{PreprocessingOptions, WhitespaceMode};
    assert_with_element_hooks(&VisitResult::Continue, &VisitResult::Continue, "a x yb\n", [1, 1, 1]);
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            let options = ConversionOptions {
                output_format,
                tier_strategy,
                whitespace_mode: WhitespaceMode::Strict,
                preprocessing: PreprocessingOptions {
                    remove_forms: false,
                    ..Default::default()
                },
                ..Default::default()
            };
            let html = r#"<p>a<label><button id="rewrite">x</button>y</label>b</p>"#;
            let written = convert(html, Some(options)).unwrap().content;
            assert_eq!(
                written.unwrap_or_default(),
                "a x yb\n",
                "{output_format:?} {tier_strategy:?}"
            );
        }
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_separate_a_label_from_a_button_inside_transparent_or_empty_spans() {
    use html_to_markdown_rs::VisitResult;
    for html in [
        "<p>a<label><span><button>x</button></span></label>b</p>",
        "<p>a<label><span></span><button>x</button></label>b</p>",
        "<p>a<label><span><span><button>x</button></span></span></label>b</p>",
    ] {
        for answer in [None, Some(&VisitResult::Continue)] {
            assert_with_button_answer(html, answer, "a x b\n");
        }
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_preserve_wrapped_button_visitor_replacements_and_ordinary_label_text() {
    use html_to_markdown_rs::VisitResult;
    for html in [
        "<p>a<label><span><button>x</button></span></label>b</p>",
        "<p>a<label><span></span><button>x</button></label>b</p>",
    ] {
        assert_with_button_answer(html, Some(&VisitResult::Skip), "ab\n");
        assert_with_button_answer(html, Some(&VisitResult::Custom("Q".to_string())), "aQb\n");
    }
    assert_with_button_answer(
        "<p>a<label><span>y<button>x</button></span></label>b</p>",
        None,
        "ay x b\n",
    );
    assert_with_button_answer(
        "<p>a<label><span>y</span><button>x</button></label>b</p>",
        None,
        "ay x b\n",
    );
}

#[cfg(feature = "visitor")]
#[test]
fn should_base_a_wrapped_label_boundary_on_the_first_button_output() {
    use html_to_markdown_rs::visitor::HtmlVisitor;
    use html_to_markdown_rs::{NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Rewrite;
    impl HtmlVisitor for Rewrite {
        fn visit_element_end(&mut self, ctx: &NodeContext<'_>, _text: &str) -> VisitResult {
            if ctx.attributes().get("id").map(String::as_str) == Some("rewrite") {
                VisitResult::Custom("Q".to_string())
            } else {
                VisitResult::Continue
            }
        }
    }

    for (html, expected) in [
        (
            r#"<p>a<label><span><button id="rewrite">x</button><button>y</button></span></label>b</p>"#,
            "aQ y b\n",
        ),
        (
            r#"<p>a<label><span><button>x</button><button id="rewrite">y</button></span></label>b</p>"#,
            "a x Qb\n",
        ),
    ] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
            let options = ConversionOptions {
                output_format,
                visitor: Some(Arc::new(Mutex::new(Rewrite))),
                ..Default::default()
            };
            assert_eq!(
                convert(html, Some(options)).unwrap().content.unwrap(),
                expected,
                "{html} {output_format:?}"
            );
        }
    }
}

#[test]
fn should_separate_a_button_through_deeply_nested_transparent_spans() {
    let html = format!(
        "<p>a<label>{}<button>x</button>{}</label>b</p>",
        "<span>".repeat(32),
        "</span>".repeat(32)
    );
    assert_each_format(&html, "a x b\n");
}
