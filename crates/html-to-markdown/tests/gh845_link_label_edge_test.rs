#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Issues #844 and #845: every writer of a link moves the white space at an edge of the label
//! outside the link. Each row runs with the default options, with the full converter, and with
//! the fast converter called directly, which writes the same text or hands the input over.

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1;
use html_to_markdown_rs::{ConversionOptions, LinkStyle, TierStrategy, WhitespaceMode, convert};

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

/// What the full converter writes.
fn full(html: &str) -> String {
    content_with(html, with_tier(TierStrategy::Tier2))
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
    assert_row(
        "<p><a href=\"https://e.org/\">\thttps://e.org/\t</a></p>",
        "<https://e.org/>\n",
    );
    assert_row("<div><a href=\"mailto:a@e.org\"> a@e.org</a></div>", "<a@e.org>\n");
    assert_row(
        "<p><b><a href=\"mailto:a@e.org\">\ta@e.org</a></b></p>",
        "**<a@e.org>**\n",
    );
    assert_row(
        "<p>Press<a href=\"https://e.org/\">&nbsp;https://e.org/&nbsp;</a>now</p>",
        "Press <https://e.org/> now\n",
    );
    assert_row_of_full_converter(
        "<p>Press <a href=\"https://e.org/\"><span>https://e.org/</span> </a>now.</p>",
        "Press <https://e.org/> now.\n",
    );
}

/// A `<br>` in an autolink is no text of the link: it writes nothing, as before the rule. A
/// `<br>` after the autolink ends its line once, and the paragraph stays one paragraph.
#[test]
fn a_line_break_in_an_autolink_writes_nothing() {
    assert_row_of_full_converter(
        "<p>Press<a href=\"https://e.org/\">https://e.org/<br></a>now</p>",
        "Press<https://e.org/>now\n",
    );
    assert_row_of_full_converter(
        "<p>Press<a href=\"https://e.org/\"><br>https://e.org/</a>now</p>",
        "Press<https://e.org/>now\n",
    );
    assert_row_of_full_converter(
        "<p>Press<br><a href=\"https://e.org/\">https://e.org/<br></a><br>now</p>",
        "Press  \n<https://e.org/>  \nnow\n",
    );
    assert_row_of_full_converter(
        "<ul><li>Press<br><a href=\"https://e.org/\">https://e.org/<br></a><br>now</li></ul>",
        "- Press  \n<https://e.org/>  \n  now\n",
    );
    // White space beside a `<br>` at an end of the link is no white space of that end.
    assert_row_of_full_converter(
        "<p>Press<a href=\"https://e.org/\">https://e.org/<br>&nbsp;</a>now</p>",
        "Press<https://e.org/>now\n",
    );
    assert_row_of_full_converter(
        "<p>Press<a href=\"https://e.org/\"><br>&nbsp;https://e.org/</a>now</p>",
        "Press<https://e.org/>now\n",
    );
}

/// The content of an autolink is not walked for the white space at its ends: an image in it
/// gets no reference definition, because the text does not use one.
#[test]
fn an_element_in_an_autolink_is_not_collected() {
    let html = "<p><a href=\"https://e.org/\">https://e.org/<img src=\"/i.png\" alt=\"\"> </a>x</p>";
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::default()] {
        let options = ConversionOptions {
            link_style: LinkStyle::Reference,
            ..with_tier(tier_strategy)
        };
        assert_eq!(content_with(html, options), "<https://e.org/> x\n", "{tier_strategy:?}");
    }
}

/// A label with a `<br>` at one edge is written whole. The fast converter keeps the white space
/// of the other edge as it wrote it before the rule, and a browser shows that space. The full
/// converter drops it.
#[test]
fn the_fast_converter_keeps_the_space_at_the_edge_opposite_a_line_break() {
    let start_space = "<p>Press<a href=\"/p\"><em> Go </em><br></a>now</p>";
    assert_eq!(fast(start_space).as_deref(), Some("Press[ *Go*  \n](/p)now\n"));
    assert_eq!(full(start_space), "Press[*Go*  \n](/p)now\n");
    let end_space = "<p>Press<a href=\"/p\"><br>Go </a>now</p>";
    assert_eq!(fast(end_space).as_deref(), Some("Press[  \nGo](/p) now\n"));
    assert_eq!(full(end_space), "Press[  \nGo](/p)now\n");
    // ~keep The white space of the link itself at the label start is dropped in both.
    assert_row("<p>Press<a href=\"/p\"> Go<br></a>now</p>", "Press[Go  \n](/p)now\n");
}

/// The fast converter hands over a label that it cannot write as the full converter does: a
/// label in a table cell that starts with a `<br>` after white space or another `<br>`, and
/// the text of a link with an empty address that ends with a `<br>`.
#[test]
fn the_fast_converter_hands_over_a_label_that_only_the_full_converter_writes() {
    for (html, wanted) in [
        (
            "<table><tr><td>Press<a href=\"/p\"> <br>Go</a>now</td></tr></table>",
            "| Press[Go](/p)now |\n| ---------------- |\n",
        ),
        (
            "<table><tr><td>(<a href=\"#top\"><br><br>Go</a>).</td></tr></table>",
            "| ([Go](#top)). |\n| ------------- |\n",
        ),
        (
            "<blockquote><p><a href=\"\">Go<br>&nbsp;</a></p></blockquote>",
            "> Go\n",
        ),
    ] {
        assert_eq!(fast(html), None, "fast converter: {html:?}");
        assert_row_of_full_converter(html, wanted);
    }
    assert_row(
        "<table><tr><td>Press<a href=\"/p\"><br>Go</a>now</td></tr></table>",
        "| Press [Go](/p)now |\n| ----------------- |\n",
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
