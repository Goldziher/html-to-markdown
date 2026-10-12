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

/// A label with a `<br>` at one edge keeps that side whole. Its other edge still gives up its
/// white space, as a browser shows it (`Press Go`, a line break, `now`), on both converters.
#[test]
fn the_edge_opposite_a_line_break_still_gives_up_its_space() {
    assert_row(
        "<p>Press<a href=\"/p\"><em> Go </em><br></a>now</p>",
        "Press [*Go*  \n](/p)now\n",
    );
    assert_row("<p>Press<a href=\"/p\"><br>Go </a>now</p>", "Press[  \nGo](/p) now\n");
    assert_row("<p>Press<a href=\"/p\"> Go<br></a>now</p>", "Press [Go  \n](/p)now\n");
    assert_row(
        "<p>Press<a href=\"/p\">&nbsp;Go<br></a>now</p>",
        "Press [Go  \n](/p)now\n",
    );
}

/// One definition of white space at a label edge on both converters: a no-break space, a narrow
/// no-break space (U+202F), an em space or a form feed at an edge is the one space of that edge,
/// as it is at the edge of a text. The words beside the link stay apart.
#[test]
fn a_unicode_space_at_a_label_edge_is_one_space_outside_the_link_in_both() {
    assert_row("<p>One<a href=\"/p\">&#8239;Go</a>two</p>", "One [Go](/p)two\n");
    assert_row("<p>One<a href=\"/p\">Go&#8239;</a>two</p>", "One[Go](/p) two\n");
    assert_row("<p>One<a href=\"/p\">&nbsp;Go&nbsp;</a>two</p>", "One [Go](/p) two\n");
    assert_row(
        "<p>One<a href=\"/p\">&emsp;<strong>Go</strong></a>two</p>",
        "One [**Go**](/p)two\n",
    );
    assert_row("<p>One<a href=\"/p\">&#12;Go</a>two</p>", "One [Go](/p)two\n");
    assert_row(
        "<p>One<a href=\"https://e.org/\">&#8239;https://e.org/</a>two</p>",
        "One <https://e.org/>two\n",
    );
    assert_row(
        "<p>One<a href=\"https://e.org/\">https://e.org/&#8239;</a>two</p>",
        "One<https://e.org/> two\n",
    );
    assert_row("<p>One<a href=\"\">&#8239;Go</a>two</p>", "One Gotwo\n");
    // ~keep One run of white space on both sides of the link start is one space.
    assert_row("<p>One <a href=\"/p\">&#8239;Go</a>two</p>", "One [Go](/p)two\n");
}

/// A `<br>` inside a label in a table cell is the cell break of the label alone, on both
/// converters: one space for a `<br>` that the label starts with, nothing for a `<br>` after
/// white space or after another `<br>`. A link with an empty address writes its text with the
/// break, and the block drops a break at its end.
#[test]
fn a_line_break_in_a_label_in_a_table_cell_is_written_alike_in_both() {
    assert_row(
        "<table><tr><td>Press<a href=\"/p\"><br>Go</a>now</td></tr></table>",
        "| Press [Go](/p)now |\n| ----------------- |\n",
    );
    assert_row(
        "<table><tr><td>Press<a href=\"/p\"> <br>Go</a>now</td></tr></table>",
        "| Press[Go](/p)now |\n| ---------------- |\n",
    );
    assert_row(
        "<table><tr><td>(<a href=\"#top\"><br><br>Go</a>).</td></tr></table>",
        "| ([Go](#top)). |\n| ------------- |\n",
    );
    assert_row(
        "<table><tr><td>Press<a href=\"/p\">Go<br>now</a></td></tr></table>",
        "| Press[Go now](/p) |\n| ----------------- |\n",
    );
    assert_row("<p>Press<a href=\"\">Go<br></a>now</p>", "PressGo  \nnow\n");
    assert_row(
        "<blockquote><p><a href=\"\">Go<br>&nbsp;</a></p></blockquote>",
        "> Go\n",
    );
    assert_row("<blockquote><p><a href=\"\">Go<br></a></p></blockquote>", "> Go\n");
}

/// A quotation drops a hard break at the end of its content, with a link and without one, on
/// both converters: the text of a link with an empty address that ends with a `<br>` is such
/// content.
#[test]
fn a_quotation_drops_a_hard_break_at_the_end_of_its_content_in_both() {
    assert_row("<blockquote><p>Go<br></p></blockquote>", "> Go\n");
    assert_row("<blockquote><p>Press<br>Go<br></p></blockquote>", "> Press  \n> Go\n");
}

/// The white space after a leading `<br>` and before a trailing one is content of the label, as
/// in the full converter: the indent of a list item, the space that a no-break space after the
/// break stands for. A no-break space after a `<br>` in a heading or a table cell is white space
/// of the label end, on both converters.
#[test]
fn the_white_space_beside_an_edge_line_break_is_written_alike_in_both() {
    assert_row(
        "<ul><li>Press<a href=\"/p\"><br>Go</a>now</li></ul>",
        "- Press[  \n  Go](/p)now\n",
    );
    assert_row(
        "<p>Press<a href=\"/p\"><br>&nbsp;<img src=\"/i.png\" alt=\"Go\"><br></a>now</p>",
        "Press[  \n ![Go](/i.png)  \n](/p)now\n",
    );
    assert_row(
        "<h2>Press <a href=\"/p\">Go<br>&nbsp;</a>now</h2>",
        "## Press [Go](/p) now\n",
    );
    assert_row(
        "<table><tr><td>Press <a href=\"/p\">Go<br>&nbsp;</a>now</td></tr></table>",
        "| Press [Go](/p) now |\n| ------------------ |\n",
    );
    assert_row(
        "<p><a href=\"/x\">one<span>&nbsp;</span> two</a> <a href=\"/y\">a <span></span>&nbsp;b</a></p>",
        "[one two](/x) [a b](/y)\n",
    );
}

/// A block in a label leaves line ends in it that are no hard break: each is one space of the
/// label, and none at an edge, on both converters.
#[test]
fn a_line_end_that_a_block_left_in_a_label_is_a_space_in_both() {
    assert_row("<p>one<a href=\"/y\"><hr>y</a>two</p>", "one[--- y](/y)two\n");
    assert_row(
        "<table><tr><td>one<a href=\"/y\"><hr>y</a>two</td></tr></table>",
        "| one[--- y](/y)two |\n| ----------------- |\n",
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
    // ~keep The strict mode keeps the white space of the source inside the link: the full
    // ~keep converter writes every such page, on the default route too.
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::default()] {
        let strict = ConversionOptions {
            whitespace_mode: WhitespaceMode::Strict,
            ..with_tier(tier_strategy)
        };
        assert_eq!(
            content_with("<p>Press<a href=\"/p\"> Go </a>now</p>", strict.clone()),
            "Press[Go](/p)now\n",
            "{tier_strategy:?}"
        );
        assert_eq!(
            content_with("<p>Press<a href=\"https://e.org/\">https://e.org/ </a>now</p>", strict),
            "Press<https://e.org/>now\n",
            "{tier_strategy:?}"
        );
    }
}

/// Two `<br>` in a row in a label are kept apart, so that the label does not end its paragraph,
/// on both converters.
#[test]
fn two_line_breaks_in_a_row_in_a_label_are_kept_apart_in_both() {
    assert_row(
        "<p>Press<a href=\"/p\">Go<br><br>now</a>two</p>",
        "Press[Go  \n\\\nnow](/p)two\n",
    );
}

/// The space of a label start is owed by the text before the link, read behind the marks of bold
/// or italic text that opened right before it: none at the start of a paragraph, on both
/// converters.
#[test]
fn the_space_of_a_label_start_is_not_owed_behind_marks_at_a_block_start() {
    assert_row(
        "<p><b><a href=\"/p\">&nbsp;<em> Go </em></a></b></p>",
        "**[*Go*](/p)**\n",
    );
    assert_row("<p>x</p><p><b><a href=\"/p\"> Go</a></b></p>", "x\n\n**[Go](/p)**\n");
    assert_row(
        "<p>Press<b><a href=\"/p\"> Go</a></b>now</p>",
        "Press **[Go](/p)**now\n",
    );
    assert_row(
        "<p>Press <b><a href=\"/p\"> Go</a></b>now</p>",
        "Press **[Go](/p)**now\n",
    );
}

/// A continuation line of a label in a list item starts with the indent of the item and nothing
/// else, on both converters: after text, after an element that follows the break, and on the
/// line that keeps two breaks apart.
#[test]
fn a_continuation_line_of_a_label_in_a_list_item_starts_with_its_indent_in_both() {
    assert_row(
        "<ul><li>Press<a href=\"/p\"><br>&nbsp;<em> Go </em>&nbsp;<br></a>now</li></ul>",
        "- Press[  \n  *Go*  \n](/p)now\n",
    );
    assert_row(
        "<ul><li>Press&nbsp;<a href=\"/p\"> <br><em> Go </em></a>&nbsp;now</li></ul>",
        "- Press [  \n  *Go*](/p) now\n",
    );
    assert_row(
        "<ul><li>Press<a href=\"/p\"><br><img src=\"/i.png\" alt=\"Go\"></a>now</li></ul>",
        "- Press[  \n  ![Go](/i.png)](/p)now\n",
    );
    assert_row(
        "<ul><li>Press<a href=\"/p\">Go<br><br>now</a></li></ul>",
        "- Press[Go  \n  \\\n  now](/p)\n",
    );
}

/// A label whose text is blank between two line breaks has no text: the fast converter leaves
/// such a link to the full one, as before.
#[test]
fn a_label_of_breaks_around_blank_text_goes_to_the_full_converter() {
    for html in [
        "<p>Press<a href=\"/p\"><br>&nbsp;&nbsp;<br></a>now</p>",
        "<div><a href=\"/p\"><br>&nbsp;&nbsp;<br></a></div>",
    ] {
        assert_eq!(fast(html), None, "fast converter: {html:?}");
    }
}

/// A no-break space after a link whose label gave up the space at its end joins that space, on
/// both converters: the fast converter knows what stands before a text after the close tag of an
/// element that holds text and is no `span`.
#[test]
fn a_no_break_space_after_a_link_joins_the_space_of_the_label_end_in_both() {
    assert_row(
        "<p>Press&nbsp;<a href=\"/p\">Go <b>Up</b> &nbsp;</a>&nbsp;now</p>",
        "Press [Go **Up**](/p) now\n",
    );
    assert_row("<p>Press <a href=\"/p\">Go </a>&nbsp;now</p>", "Press [Go](/p) now\n");
    assert_row("<p>Press <b>Go </b>&nbsp;now</p>", "Press **Go** now\n");
    // ~keep After an element with no text (an image is none, white space is none) and after a
    // ~keep `span` whose text ends with a space the no-break space is a second space: the fast
    // ~keep converter leaves that page to the full one, as before.
    for (html, wanted) in [
        (
            "<p>Press <a href=\"/p\">Go </a><span></span>&nbsp;now</p>",
            "Press [Go](/p)  now\n",
        ),
        (
            "<p>Press <a href=\"/p\"><img src=\"/i.png\" alt=\"Go\"> </a>&nbsp;now</p>",
            "Press [![Go](/i.png)](/p)  now\n",
        ),
        (
            "<p>Press <a href=\"/p\">Go </a><small> </small>&nbsp;now</p>",
            "Press [Go](/p)  now\n",
        ),
        ("<p>a <span>x </span>&nbsp;b</p>", "a x  b\n"),
    ] {
        assert_row_of_full_converter(html, wanted);
        assert_eq!(fast(html), None, "fast converter: {html:?}");
    }
}
