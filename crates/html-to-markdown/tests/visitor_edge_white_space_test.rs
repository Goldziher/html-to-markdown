// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "visitor")]

//! Issue #800: the white space at an end of a link label or of a short quotation goes outside
//! the element only when the converter writes the element itself. A visitor that skips the
//! element, replaces it, or keeps a link as HTML gets the output it got before that change: no
//! space appears beside its answer.
//!
//! Each row runs with the default options and with the full converter. The router gives every
//! input with a visitor to the full converter, so the two runs must agree.

use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};
use std::sync::{Arc, Mutex};

const LINK: &str = r#"<p>one<a href="/x"> X </a>two</p>"#;
const QUOTATION: &str = "<p>one<q> X </q>two</p>";

/// The callback that answers for the element under test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hook {
    Link,
    ElementStart,
    ElementEnd,
}

#[derive(Debug, Clone, Copy)]
enum Answer {
    Continue,
    Skip,
    PreserveHtml,
    Custom,
}

/// Gives `answer` from `hook` for the element named `tag`, and `Continue` everywhere else.
#[derive(Debug)]
struct Answers {
    tag: &'static str,
    hook: Hook,
    answer: Answer,
}

impl Answers {
    fn answer_at(&self, hook: Hook, tag: &str) -> VisitResult {
        if hook != self.hook || tag != self.tag {
            return VisitResult::Continue;
        }
        match self.answer {
            Answer::Continue => VisitResult::Continue,
            Answer::Skip => VisitResult::Skip,
            Answer::PreserveHtml => VisitResult::PreserveHtml,
            Answer::Custom => VisitResult::Custom("NEW".to_string()),
        }
    }
}

impl HtmlVisitor for Answers {
    fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
        self.answer_at(Hook::ElementStart, &ctx.tag_name)
    }

    fn visit_element_end(&mut self, ctx: &NodeContext<'_>, _output: &str) -> VisitResult {
        self.answer_at(Hook::ElementEnd, &ctx.tag_name)
    }

    fn visit_link(&mut self, ctx: &NodeContext<'_>, _href: &str, _text: &str, _title: Option<&str>) -> VisitResult {
        self.answer_at(Hook::Link, &ctx.tag_name)
    }
}

fn written(html: &str, tier_strategy: TierStrategy, visitor: Option<Answers>) -> String {
    let mut options = ConversionOptions {
        tier_strategy,
        ..ConversionOptions::default()
    };
    if let Some(visitor) = visitor {
        options.visitor = Some(Arc::new(Mutex::new(visitor)));
    }
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn assert_written(html: &str, tag: &'static str, hook: Hook, answer: Answer, wanted: &str) {
    for (route, tier_strategy) in [
        ("default options", TierStrategy::Auto),
        ("full converter", TierStrategy::Tier2),
    ] {
        let visitor = Answers { tag, hook, answer };
        assert_eq!(
            written(html, tier_strategy, Some(visitor)),
            wanted,
            "{route}: {hook:?} answers {answer:?} for {html:?}"
        );
    }
}

#[test]
fn the_converter_writes_the_edge_white_space_of_a_link_and_of_a_quotation_itself() {
    assert_eq!(written(LINK, TierStrategy::Tier2, None), "one [X](/x) two\n");
    assert_eq!(written(QUOTATION, TierStrategy::Tier2, None), "one \"X\" two\n");
}

#[test]
fn a_skipped_link_writes_no_space_where_the_link_was() {
    assert_written(LINK, "a", Hook::Link, Answer::Skip, "onetwo\n");
}

#[test]
fn a_link_kept_as_html_stays_beside_the_words_as_in_the_source() {
    assert_written(
        LINK,
        "a",
        Hook::Link,
        Answer::PreserveHtml,
        "one<a href=\"/x\"> X </a>two\n",
    );
}

#[test]
fn a_replaced_link_writes_no_space_beside_the_replacement() {
    assert_written(LINK, "a", Hook::Link, Answer::Custom, "oneNEWtwo\n");
}

#[test]
fn a_visitor_that_continues_gets_the_link_of_the_converter() {
    assert_written(LINK, "a", Hook::Link, Answer::Continue, "one [X](/x) two\n");
}

#[test]
fn a_link_skipped_at_its_element_start_writes_no_space() {
    assert_written(LINK, "a", Hook::ElementStart, Answer::Skip, "onetwo\n");
}

#[test]
fn a_link_replaced_at_its_element_start_writes_no_space_beside_the_replacement() {
    assert_written(LINK, "a", Hook::ElementStart, Answer::Custom, "oneNEWtwo\n");
}

#[test]
fn a_link_skipped_at_its_element_end_writes_no_space() {
    assert_written(LINK, "a", Hook::ElementEnd, Answer::Skip, "onetwo\n");
}

#[test]
fn a_link_replaced_at_its_element_end_writes_no_space_beside_the_replacement() {
    assert_written(LINK, "a", Hook::ElementEnd, Answer::Custom, "oneNEWtwo\n");
}

#[test]
fn a_quotation_skipped_at_its_element_start_writes_no_space() {
    assert_written(QUOTATION, "q", Hook::ElementStart, Answer::Skip, "onetwo\n");
}

#[test]
fn a_quotation_replaced_at_its_element_start_writes_no_space_beside_the_replacement() {
    assert_written(QUOTATION, "q", Hook::ElementStart, Answer::Custom, "oneNEWtwo\n");
}

#[test]
fn a_quotation_skipped_at_its_element_end_writes_no_space() {
    assert_written(QUOTATION, "q", Hook::ElementEnd, Answer::Skip, "onetwo\n");
}

#[test]
fn a_quotation_replaced_at_its_element_end_writes_no_space_beside_the_replacement() {
    assert_written(QUOTATION, "q", Hook::ElementEnd, Answer::Custom, "oneNEWtwo\n");
}

/// The element callbacks do not keep an element as HTML: the converter writes the element, with
/// its edge white space.
#[test]
fn an_element_callback_that_asks_for_html_gets_the_element_of_the_converter() {
    for hook in [Hook::ElementStart, Hook::ElementEnd] {
        assert_written(LINK, "a", hook, Answer::PreserveHtml, "one [X](/x) two\n");
        assert_written(QUOTATION, "q", hook, Answer::PreserveHtml, "one \"X\" two\n");
    }
}
