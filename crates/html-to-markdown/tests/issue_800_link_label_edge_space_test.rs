#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Issue #800: white space at an end of a link label or of a short quotation goes outside
//! the element, as it does for bold text. Each row is a row of the issue. It runs with the
//! default options, with the full converter, and with the fast converter called directly. Each
//! row says what the fast converter does: it writes a named text, or it hands the input over.

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

/// What the fast converter does with a row when it is called directly.
enum Fast<'a> {
    /// It writes this text.
    Writes(&'a str),
    /// It gives the input to the full converter, which then writes the text of the row.
    HandsOver,
}

fn content_with(html: &str, options: Option<ConversionOptions>) -> String {
    convert(html, options)
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

/// A row that the fast converter writes as the full converter does.
fn assert_row(html: &str, wanted: &str) {
    assert_row_with_fast(html, wanted, &Fast::Writes(wanted));
}

fn assert_row_with_fast(html: &str, wanted: &str, fast: &Fast<'_>) {
    assert_eq!(content_with(html, None), wanted, "default options: {html:?}");
    let full = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    };
    assert_eq!(content_with(html, Some(full)), wanted, "full converter: {html:?}");
    let fast_options = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..ConversionOptions::default()
    };
    let (cleaned, report) = prescan::run(html);
    match (tier1::run(cleaned.as_ref(), &report, &fast_options), fast) {
        (Ok(written), Fast::Writes(wanted_fast)) => assert_eq!(written, *wanted_fast, "fast converter: {html:?}"),
        (Ok(written), Fast::HandsOver) => panic!("the fast converter no longer hands over {html:?}: {written:?}"),
        (Err(reason), Fast::Writes(_)) => panic!("the fast converter hands over {html:?}: {reason}"),
        (Err(_), Fast::HandsOver) => {}
    }
}

#[test]
fn a_space_at_the_end_of_a_link_label_goes_after_the_link() {
    assert_row("<p>Press <a href=\"/p\">Go </a>now.</p>", "Press [Go](/p) now.\n");
}

/// A link with an empty address writes its text and no marks. The space at the end of that
/// text stays, in both converters.
#[test]
fn a_space_at_the_end_of_a_link_with_an_empty_address_stays_after_its_text() {
    assert_row("<p>Press <a href=\"\">Go </a>now.</p>", "Press Go now.\n");
}

/// A link to a place in its own page with a label of white space is left out. The space of the
/// text before it stays: one space between the two words.
#[test]
fn a_space_before_a_link_into_its_own_page_with_a_blank_label_stays() {
    assert_row_with_fast("<p>aaa <a href=\"#top\"> </a>bbb</p>", "aaa bbb\n", &Fast::HandsOver);
}

#[test]
fn a_space_after_a_link_stays() {
    assert_row("<p>Press <a href=\"/p\">Go</a> now.</p>", "Press [Go](/p) now.\n");
}

#[test]
fn a_space_at_the_start_of_a_link_label_goes_before_the_link() {
    assert_row("<p>Press<a href=\"/p\"> Go</a> now.</p>", "Press [Go](/p) now.\n");
}

/// A line break at the end of a label stays in the label when a no-break space follows it.
#[test]
fn a_line_break_before_a_no_break_space_stays_in_the_label() {
    assert_row(
        "<p>Press<a href=\"/p\">Go<br>&nbsp;</a>now</p>",
        "Press[Go  \n](/p)now\n",
    );
}

/// A line break at the end of a label stays in the label when a space follows it, and that
/// space does not go after the link.
#[test]
fn a_space_after_a_line_break_at_the_end_of_a_label_writes_no_space_after_the_link() {
    assert_row("<p>Press<a href=\"/p\">Go<br> </a>now</p>", "Press[Go  \n](/p)now\n");
}

#[test]
fn a_space_at_the_end_of_a_quotation_goes_after_the_marks() {
    // ~keep The fast converter does not write the marks of a quotation.
    assert_row_with_fast("<p>Press <q>Go </q>now.</p>", "Press \"Go\" now.\n", &Fast::HandsOver);
}

#[test]
fn a_space_at_the_end_of_bold_text_stays_after_the_marks() {
    assert_row("<p>Press <b>Go </b>now.</p>", "Press **Go** now.\n");
}

#[test]
fn a_space_at_the_end_of_a_span_stays() {
    assert_row("<p>Press <span>Go </span>now.</p>", "Press Go now.\n");
}
