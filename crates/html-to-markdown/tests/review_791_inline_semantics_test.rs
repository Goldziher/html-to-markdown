#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for the review of pull request #791 and for issue #822.
//!
//! Each input runs on both converters. With these options the router sends every input to
//! Tier-2, so a test also calls the Tier-1 scanner directly: a fallback cannot hide its answer.

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert, prescan, tier1};

const STRATEGIES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier2, TierStrategy::Tier1];

fn options(tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        tier_strategy,
        extract_metadata: false,
        ..ConversionOptions::default()
    }
}

fn convert_with(html: &str, tier_strategy: TierStrategy) -> String {
    convert(html, Some(options(tier_strategy)))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

/// Calls the Tier-1 scanner directly. `Err` holds the reason it hands the page to Tier-2.
fn scan(html: &str) -> Result<String, String> {
    let (cleaned, report) = prescan::run(html);
    tier1::run(cleaned.as_ref(), &report, &options(TierStrategy::Tier1)).map_err(|bail| format!("{bail:?}"))
}

/// Asserts that every strategy and the Tier-1 scanner itself give `expected`.
fn check(html: &str, expected: &str) {
    for tier_strategy in STRATEGIES {
        assert_eq!(
            convert_with(html, tier_strategy),
            expected,
            "{tier_strategy:?}: {html:?}"
        );
    }
    match scan(html) {
        Ok(scanned) => assert_eq!(scanned, expected, "Tier-1 scanner: {html:?}"),
        Err(bail) => panic!("the Tier-1 scanner hands {html:?} over: {bail}"),
    }
}

/// Asserts that the Tier-1 scanner hands the page over and that every strategy gives `expected`.
fn check_handed_over(html: &str, expected: &str) {
    let scanned = scan(html);
    assert!(
        scanned.is_err(),
        "the Tier-1 scanner must hand {html:?} over: {scanned:?}"
    );
    for tier_strategy in STRATEGIES {
        assert_eq!(
            convert_with(html, tier_strategy),
            expected,
            "{tier_strategy:?}: {html:?}"
        );
    }
}

#[test]
fn should_keep_the_word_space_before_a_description_in_a_heading() {
    check("<h2>a <dl><dd>b</dd></dl></h2>", "## a b\n");
}

#[test]
fn should_separate_two_descriptions_in_a_heading() {
    check("<h2>a <dl><dd>b</dd><dd>c</dd></dl></h2>", "## a b c\n");
}

#[test]
fn should_keep_the_word_space_before_a_description_outside_a_list() {
    check("<div>a <dd>b</dd> c</div>", "a b\n\nc\n");
}

#[test]
fn should_keep_a_term_apart_outside_a_list() {
    check("<div>a <dt>t</dt> c</div>", "a t\n\nc\n");
}

#[test]
fn should_keep_a_term_apart_in_a_heading() {
    check("<h2>a<dl><dt>t</dt><dd>b</dd></dl></h2>", "## a t b\n");
}

#[test]
fn should_keep_a_term_apart_in_a_link() {
    check_handed_over(
        "<p><a href=\"/x\">a<dl><dt>t</dt><dd>b</dd></dl></a></p>",
        "[a t b](/x)\n",
    );
}

#[test]
fn should_keep_the_space_between_a_paragraph_and_text_in_a_heading() {
    check("<h2><p>One</p>items</h2>", "## One items\n");
}

#[test]
fn should_write_a_label_in_a_code_block_as_written() {
    check("<pre>a\n<label>  b\n</label>c</pre>", "```\na\n  b\nc\n```\n");
}

#[test]
fn should_write_a_label_in_a_code_block_as_a_span_there() {
    for body in ["a\n<X>  b\n</X>c", "a<X><div>x</div></X>b"] {
        let label = format!("<pre>{}</pre>", body.replace('X', "label"));
        let span = format!("<pre>{}</pre>", body.replace('X', "span"));
        for tier_strategy in STRATEGIES {
            assert_eq!(
                convert_with(&label, tier_strategy),
                convert_with(&span, tier_strategy),
                "{tier_strategy:?}: {label:?}"
            );
        }
        let scanned = scan(&label).unwrap_or_else(|bail| panic!("the Tier-1 scanner hands {label:?} over: {bail}"));
        assert_eq!(Ok(&scanned), scan(&span).as_ref(), "Tier-1 scanner: {label:?}");
        assert_eq!(
            scanned,
            convert_with(&label, TierStrategy::Tier2),
            "Tier-1 scanner: {label:?}"
        );
    }
}

#[test]
fn should_hand_a_media_element_to_tier2_for_its_link() {
    check_handed_over("<p>x</p><VIDEO src=\"/v.mp4\"></VIDEO>", "x\n\n[video](/v.mp4)\n");
    check_handed_over("<video src=\"/v.mp4\" title=\"Clip\"></video>", "[Clip](/v.mp4)\n");
    check_handed_over("<Audio src=\"/a.ogg\"></Audio>", "[audio](/a.ogg)\n");
    check_handed_over(
        "<video><source src=\"/a.mp4\"><source src=\"/b.webm\"></video>",
        "[video](/a.mp4)\n",
    );
    check_handed_over("<iframe src=\"/f\" title=\"Map\"></iframe>", "[Map](/f)\n");
}

#[cfg(feature = "visitor")]
#[test]
fn should_report_the_text_of_a_label_in_a_code_block_one_level_below_the_label() {
    use html_to_markdown_rs::NodeContext;
    use html_to_markdown_rs::visitor::{HtmlVisitor, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct TextDepthVisitor {
        depths: Arc<Mutex<Vec<usize>>>,
    }

    impl HtmlVisitor for TextDepthVisitor {
        fn visit_text(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            if text == "text" {
                self.depths
                    .lock()
                    .expect("depth collector mutex must not be poisoned")
                    .push(ctx.depth);
            }
            VisitResult::Continue
        }
    }

    fn text_depths(html: &str) -> Vec<usize> {
        let depths = Arc::new(Mutex::new(Vec::new()));
        let with_visitor = ConversionOptions {
            visitor: Some(Arc::new(Mutex::new(TextDepthVisitor {
                depths: Arc::clone(&depths),
            }))),
            ..options(TierStrategy::Tier2)
        };
        convert(html, Some(with_visitor)).expect("conversion must succeed");
        let mut collected = depths.lock().expect("depth mutex must not be poisoned");
        std::mem::take(&mut *collected)
    }

    let in_label = text_depths("<pre>a<label>text</label></pre>");
    assert!(!in_label.is_empty(), "the visitor must see the text of the label");
    assert_eq!(in_label, text_depths("<pre>a<span>text</span></pre>"));
}
