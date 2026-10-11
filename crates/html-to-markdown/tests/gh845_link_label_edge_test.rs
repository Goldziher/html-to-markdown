#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Issues #844 and #845: every writer of a link moves the white space at an edge of the label
//! outside the link. Each row runs with the default options, with the full converter, and with
//! the fast converter called directly, which writes the same text or hands the input over.

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, WhitespaceMode, convert};

fn content_with(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn with_tier(tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        tier_strategy,
        ..ConversionOptions::default()
    }
}

/// What the fast converter writes when it is called directly: `None` when it hands over.
fn fast(html: &str) -> Option<String> {
    let (cleaned, report) = prescan::run(html);
    tier1::run(cleaned.as_ref(), &report, &with_tier(TierStrategy::Tier1)).ok()
}

/// A row that both converters write.
fn assert_row(html: &str, wanted: &str) {
    assert_row_of_full_converter(html, wanted);
    assert_eq!(fast(html).as_deref(), Some(wanted), "fast converter: {html:?}");
}

/// A row that the full converter writes, and that the fast converter writes or hands over.
fn assert_row_of_full_converter(html: &str, wanted: &str) {
    assert_eq!(
        content_with(html, ConversionOptions::default()),
        wanted,
        "default: {html:?}"
    );
    assert_eq!(
        content_with(html, with_tier(TierStrategy::Tier2)),
        wanted,
        "full: {html:?}"
    );
    if let Some(written) = fast(html) {
        assert_eq!(written, wanted, "fast converter: {html:?}");
    }
}

#[test]
fn an_autolink_writes_its_label_edge_space_outside() {
    assert_row(
        "<p>Press <a href=\"https://e.org/\">https://e.org/ </a>now.</p>",
        "Press <https://e.org/> now.\n",
    );
    assert_row(
        "<p>Press<a href=\"https://e.org/\"> https://e.org/</a>now.</p>",
        "Press <https://e.org/>now.\n",
    );
    assert_row(
        "<p>Press <a href=\"mailto:a@e.org\">a@e.org </a>now.</p>",
        "Press <a@e.org> now.\n",
    );
    assert_row(
        "<h2>Press <a href=\"https://e.org/\">https://e.org/ </a>now</h2>",
        "## Press <https://e.org/> now\n",
    );
    assert_row(
        "<table><tr><td>Press <a href=\"https://e.org/\">https://e.org/ </a>now</td></tr></table>",
        "| Press <https://e.org/> now |\n| -------------------------- |\n",
    );
    assert_row_of_full_converter(
        "<p>Press <a href=\"https://e.org/\"><span>https://e.org/</span> </a>now.</p>",
        "Press <https://e.org/> now.\n",
    );
}

/// An autolink has no label to keep a line break in: the break goes after the marks.
#[test]
fn a_line_break_at_the_end_of_an_autolink_label_goes_after_the_autolink() {
    assert_row_of_full_converter(
        "<p>Press<a href=\"https://e.org/\">https://e.org/<br></a>now</p>",
        "Press<https://e.org/>  \nnow\n",
    );
    assert_row_of_full_converter(
        "<h2>Press<a href=\"https://e.org/\">https://e.org/<br></a>now</h2>",
        "## Press<https://e.org/> now\n",
    );
}

#[test]
fn the_fast_converter_writes_the_start_space_of_a_label() {
    assert_row("<p>Press<a href=\"/p\"> Go</a> now.</p>", "Press [Go](/p) now.\n");
    assert_row(
        "<ul><li>Press<a href=\"/p\"> Go</a> now.</li></ul>",
        "- Press [Go](/p) now.\n",
    );
    assert_row(
        "<p>Press<a href=\"/p\"> <img src=\"/i.png\" alt=\"Go\"></a> now.</p>",
        "Press [![Go](/i.png)](/p) now.\n",
    );
    assert_row("<p>Press <a href=\"/p\"> Go</a> now.</p>", "Press [Go](/p) now.\n");
}

#[test]
fn a_line_break_at_a_label_edge_in_a_heading_or_a_cell_is_a_space_in_both() {
    assert_row("<h2>Press<a href=\"/p\">Go<br></a>now</h2>", "## Press[Go](/p) now\n");
    assert_row("<h2>Press<a href=\"/p\"><br>Go</a>now</h2>", "## Press [Go](/p)now\n");
    assert_row(
        "<table><tr><td>Press<a href=\"/p\">Go<br></a>now</td></tr></table>",
        "| Press[Go](/p) now |\n| ----------------- |\n",
    );
}

#[test]
fn a_line_break_before_a_no_break_space_stays_in_the_label_in_both() {
    assert_row(
        "<p>Press<a href=\"/p\">Go<br>&nbsp;</a>now</p>",
        "Press[Go  \n](/p)now\n",
    );
}

#[test]
fn code_and_strict_mode_keep_their_text() {
    assert_row_of_full_converter(
        "<p>See <code>Press<a href=\"/p\"> Go </a>now</code> end</p>",
        "See `Press`[` Go `](/p)`now` end\n",
    );
    assert_row_of_full_converter("<pre>Press<a href=\"/p\"> Go </a>now</pre>", "```\nPress Go now\n```\n");
    let strict = ConversionOptions {
        whitespace_mode: WhitespaceMode::Strict,
        ..ConversionOptions::default()
    };
    assert_eq!(
        content_with("<p>Press<a href=\"/p\"> Go </a>now</p>", strict.clone()),
        "Press[Go](/p)now\n"
    );
    assert_eq!(
        content_with("<p>Press<a href=\"https://e.org/\">https://e.org/ </a>now</p>", strict),
        "Press<https://e.org/>now\n"
    );
}
