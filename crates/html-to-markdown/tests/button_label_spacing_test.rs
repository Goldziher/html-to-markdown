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
fn assert_with_button_answer(html: &str, answer: Option<html_to_markdown_rs::VisitResult>, expected: &str) {
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
                    .clone()
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
        Some(VisitResult::Skip),
        "ab\n",
    );
    assert_with_button_answer(
        "<p>a<label><button>x</button><span>y</span></label>b</p>",
        Some(VisitResult::Skip),
        "ayb\n",
    );
    assert_with_button_answer("<p>a<button>x</button>b</p>", Some(VisitResult::Skip), "ab\n");
}

#[cfg(feature = "visitor")]
#[test]
fn should_not_separate_a_label_from_the_text_a_visitor_wrote_for_a_button() {
    use html_to_markdown_rs::VisitResult;
    let q = || Some(VisitResult::Custom("Q".to_string()));
    let empty = || Some(VisitResult::Custom(String::new()));
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
    for answer in [None, Some(VisitResult::Continue)] {
        assert_with_button_answer("<p>a<label><button>x</button></label>b</p>", answer.clone(), "a x b\n");
        assert_with_button_answer(
            "<p>a<label><button>x</button><span>y</span></label>b</p>",
            answer.clone(),
            "a x yb\n",
        );
        assert_with_button_answer("<p>a<button>x</button>b</p>", answer, "a x b\n");
    }
}
