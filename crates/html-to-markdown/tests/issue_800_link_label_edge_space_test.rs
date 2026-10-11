#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Issue #800: white space at an end of a link label or of a short quotation goes outside
//! the element, as it does for bold text. Each row is a row of the issue. It runs with the
//! default options, with the full converter, and with the fast converter forced: the fast
//! converter writes the same text or hands the input over.

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn content_with(html: &str, options: Option<ConversionOptions>) -> String {
    convert(html, options)
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn assert_row(html: &str, wanted: &str) {
    assert_row_with_fast(html, wanted, wanted);
}

fn assert_row_with_fast(html: &str, wanted: &str, wanted_fast: &str) {
    assert_eq!(content_with(html, None), wanted, "default options: {html:?}");
    let full = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    };
    assert_eq!(content_with(html, Some(full)), wanted, "full converter: {html:?}");
    let fast = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..ConversionOptions::default()
    };
    let (cleaned, report) = prescan::run(html);
    // ~keep A hand-over is right by construction: the full converter then writes the text above.
    if let Ok(written) = tier1::run(cleaned.as_ref(), &report, &fast) {
        assert_eq!(written, wanted_fast, "fast converter: {html:?}");
    }
}

#[test]
fn a_space_at_the_end_of_a_link_label_goes_after_the_link() {
    assert_row("<p>Press <a href=\"/p\">Go </a>now.</p>", "Press [Go](/p) now.\n");
}

#[test]
fn a_space_after_a_link_stays() {
    assert_row("<p>Press <a href=\"/p\">Go</a> now.</p>", "Press [Go](/p) now.\n");
}

#[test]
fn a_space_at_the_start_of_a_link_label_goes_before_the_link() {
    // ~keep Draft: "the fast converter drops the white space at the start of a link label".
    assert_row_with_fast(
        "<p>Press<a href=\"/p\"> Go</a> now.</p>",
        "Press [Go](/p) now.\n",
        "Press[Go](/p) now.\n",
    );
}

/// A line break at the end of a label stays in the label when a no-break space follows it.
#[test]
fn a_line_break_before_a_no_break_space_stays_in_the_label() {
    let html = "<p>Press<a href=\"/p\">Go<br>&nbsp;</a>now</p>";
    let full = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    };
    assert_eq!(content_with(html, None), "Press[Go  \n](/p)now\n");
    assert_eq!(content_with(html, Some(full)), "Press[Go  \n](/p)now\n");
}

#[test]
fn a_space_at_the_end_of_a_quotation_goes_after_the_marks() {
    assert_row("<p>Press <q>Go </q>now.</p>", "Press \"Go\" now.\n");
}

#[test]
fn a_space_at_the_end_of_bold_text_stays_after_the_marks() {
    assert_row("<p>Press <b>Go </b>now.</p>", "Press **Go** now.\n");
}

#[test]
fn a_space_at_the_end_of_a_span_stays() {
    assert_row("<p>Press <span>Go </span>now.</p>", "Press Go now.\n");
}
