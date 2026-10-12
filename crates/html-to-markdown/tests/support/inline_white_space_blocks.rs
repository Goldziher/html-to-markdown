#[test]
fn an_empty_inline_element_of_any_kind_keeps_the_space_before_it() {
    assert_on_both(&[
        ("<p>one\n<b></b>two</p>", "one two\n"),
        ("<p>one\n<i></i>two</p>", "one two\n"),
        ("<p>one\n<code></code>two</p>", "one two\n"),
        ("<p>one\n<a></a>two</p>", "one two\n"),
        ("<p>one\n<a id=\"target\"></a>two</p>", "one two\n"),
        ("<p>one\n<i class=\"fa fa-x\"></i>two</p>", "one two\n"),
        ("<p>one <span id=\"a\"></span>two</p>", "one two\n"),
        ("<p>one\t<span id=\"a\"></span>two</p>", "one two\n"),
        ("<p>one<span id=\"a\"></span>\ntwo</p>", "one two\n"),
    ]);
}

/// White space on the two sides of an element that writes nothing is one run of white space.
#[test]
fn white_space_on_both_sides_of_an_empty_inline_element_is_one_space() {
    assert_on_both(&[
        ("<p>one\n<span id=\"a\"></span>\ntwo</p>", "one two\n"),
        ("<p>one <span id=\"a\"></span> two</p>", "one two\n"),
        ("<p>one\t<a id=\"t\"></a>\ttwo</p>", "one two\n"),
        (
            "<p>one\n<span id=\"a\"></span>\n<span id=\"b\"></span>\ntwo</p>",
            "one two\n",
        ),
        ("<p>one <b></b> two</p>", "one two\n"),
        ("<p>one <span> </span> two</p>", "one two\n"),
        ("<p><em>one\n<span id=\"a\"></span>\ntwo</em></p>", "*one two*\n"),
        ("<h2>one\n<span id=\"a\"></span>\ntwo</h2>", "## one two\n"),
        ("<ul><li>one <span id=\"a\"></span> two</li></ul>", "- one two\n"),
        ("<div>one <span id=\"a\"></span> two</div>", "one two\n"),
        ("<blockquote>one <span id=\"a\"></span> two</blockquote>", "> one two\n"),
        (
            "<p><a href=\"/x\">one <span id=\"a\"></span> two</a></p>",
            "[one two](/x)\n",
        ),
        ("<p><span>one </span> two</p>", "one two\n"),
        (
            "<table><tr><th>h</th></tr><tr><td>one <span id=\"a\"></span> two</td></tr></table>",
            "| h       |\n| ------- |\n| one two |\n",
        ),
        (
            "<table><tr><th>h</th></tr><tr><td>one\n<span id=\"a\"></span>\ntwo</td></tr></table>",
            "| h       |\n| ------- |\n| one two |\n",
        ),
    ]);
}

/// A no-break space in a node of its own is kept as it is, also beside a space. In running text
/// the default white space mode writes it as one space, at the edge of the text as in the middle.
#[test]
fn a_no_break_space_does_not_collapse() {
    assert_on_both(&[
        ("<p>one\n<span>&nbsp;</span>two</p>", "one \u{a0}two\n"),
        ("<p>one <b>x</b> &nbsp;<i>two</i></p>", "one **x** \u{a0}*two*\n"),
        ("<p><b>one</b>&nbsp;<i>two</i></p>", "**one**\u{a0}*two*\n"),
        ("<p><b>one</b>&nbsp;&nbsp;<i>two</i></p>", "**one**\u{a0}\u{a0}*two*\n"),
        ("<p>one&nbsp;two</p>", "one two\n"),
        ("<p>one&nbsp;<b>two</b></p>", "one **two**\n"),
        ("<p><b>one</b>&nbsp;two</p>", "**one** two\n"),
        ("<p>one&#160;<b>two</b>&#xa0;three</p>", "one **two** three\n"),
        ("<p>one<span id=\"a\"></span>&nbsp;two</p>", "one two\n"),
        ("<p>one&nbsp;<span id=\"a\"></span>two</p>", "one two\n"),
        ("<p><img src=\"/i.png\" alt=\"i\">&nbsp;text</p>", "![i](/i.png) text\n"),
        ("<p>see&nbsp;<a href=\"/x\">two</a>&nbsp;now</p>", "see [two](/x) now\n"),
        ("<ul><li>one&nbsp;<b>two</b></li></ul>", "- one **two**\n"),
        ("<h2>one&nbsp;<b>two</b></h2>", "## one **two**\n"),
        ("<pre>one&nbsp;two</pre>", "```\none\u{a0}two\n```\n"),
        ("<pre>one &nbsp;two</pre>", "```\none \u{a0}two\n```\n"),
        ("<p>one <span>&nbsp;</span> two</p>", "one \u{a0} two\n"),
        ("<p><b>one\n</b><span>&nbsp;</span>two</p>", "**one** \u{a0}two\n"),
    ]);
}

#[test]
fn no_white_space_in_the_source_is_no_space() {
    assert_on_both(&[
        ("<p>one<span id=\"a\"></span>two</p>", "onetwo\n"),
        ("<p>one<b></b>two</p>", "onetwo\n"),
        ("<p><b>H</b>ello</p>", "**H**ello\n"),
        ("<p>foo<span>bar</span>baz</p>", "foobarbaz\n"),
        ("<p><a href=\"/x\">link</a>.</p>", "[link](/x).\n"),
        ("<p>see <em>this</em>, then</p>", "see *this*, then\n"),
        ("<p>one\n<span id=\"a\"></span>, two</p>", "one , two\n"),
        ("<p><img src=\"/i.png\" alt=\"alt\">text</p>", "![alt](/i.png)text\n"),
        ("<p>one<img src=\"/i.png\" alt=\"i\">two</p>", "one![i](/i.png)two\n"),
        ("<p>see <img src=\"/i.png\" alt=\"i\">.</p>", "see ![i](/i.png).\n"),
    ]);
}

#[test]
fn an_empty_inline_element_is_not_the_start_or_the_end_of_a_line() {
    assert_on_both(&[
        ("<p><span id=\"a\"></span> one two</p>", "one two\n"),
        ("<p>\n<span id=\"a\"></span>\none two</p>", "one two\n"),
        ("<p>one two\n<span id=\"a\"></span></p>", "one two\n"),
        (
            "<p>one two\n<span id=\"a\"></span></p><p>three</p>",
            "one two\n\nthree\n",
        ),
        ("<p>one\n<span id=\"a\"></span><br>two</p>", "one  \ntwo\n"),
        ("<p>one\n<br>two</p>", "one  \ntwo\n"),
    ]);
}

#[test]
fn the_space_after_an_image_is_kept() {
    assert_on_both(&[
        ("<p><img src=\"/i.png\" alt=\"alt\"> text</p>", "![alt](/i.png) text\n"),
        ("<p><img src=\"/i.png\" alt=\"alt\">\ntext</p>", "![alt](/i.png) text\n"),
        (
            "<p><a href=\"/x\"><img src=\"/i.png\" alt=\"alt\"> text</a></p>",
            "[![alt](/i.png) text](/x)\n",
        ),
        (
            "<p><a href=\"/x\">text <img src=\"/i.png\" alt=\"alt\"></a></p>",
            "[text ![alt](/i.png)](/x)\n",
        ),
        (
            "<p><a href=\"/x\"><img src=\"/i.png\" alt=\"i\"></a> text</p>",
            "[![i](/i.png)](/x) text\n",
        ),
        ("<img src=\"/i.png\" alt=\"alt\"> text", "![alt](/i.png) text\n"),
        (
            "<div><img src=\"/i.png\" alt=\"alt\"> text</div>",
            "![alt](/i.png) text\n",
        ),
        (
            "<blockquote><img src=\"/i.png\" alt=\"i\"> text</blockquote>",
            "> ![i](/i.png) text\n",
        ),
        (
            "<p><b><img src=\"/i.png\" alt=\"i\"> text</b></p>",
            "**![i](/i.png) text**\n",
        ),
        (
            "<p><img src=\"/i.png\" alt=\"i\"> <b>bold</b></p>",
            "![i](/i.png) **bold**\n",
        ),
        (
            "<p><img src=\"/a.png\" alt=\"a\"> <img src=\"/b.png\" alt=\"b\"> text</p>",
            "![a](/a.png) ![b](/b.png) text\n",
        ),
        (
            "<p>first</p><p><img src=\"/i.png\" alt=\"alt\"> text</p>",
            "first\n\n![alt](/i.png) text\n",
        ),
        (
            "<p>one <img src=\"/i.png\" alt=\"i\"> two</p>",
            "one ![i](/i.png) two\n",
        ),
        (
            "<p>one\n<img src=\"/i.png\" alt=\"i\">\ntwo</p>",
            "one ![i](/i.png) two\n",
        ),
        ("<p>one <img src=\"/i.png\" alt=\"i\"> </p>", "one ![i](/i.png)\n"),
        ("<p> <img src=\"/i.png\" alt=\"i\"> text</p>", "![i](/i.png) text\n"),
        (
            "<ul><li><img src=\"/i.png\" alt=\"i\"> text</li></ul>",
            "- ![i](/i.png) text\n",
        ),
        ("<h2><img src=\"/i.png\" alt=\"i\"> text</h2>", "## i text\n"),
        (
            "<p><img src=\"/i.png\" alt=\"i\"><br> text</p>",
            "![i](/i.png)  \ntext\n",
        ),
    ]);
}

#[test]
fn the_space_after_an_inline_graphic_is_kept() {
    const GRAPHIC: &str = "<svg viewBox=\"0 0 1 1\"><title>Logo</title><path d=\"M0 0\"/></svg>";
    for (html, start, end) in [
        (
            format!("<p>{GRAPHIC} text</p>"),
            "![Logo](data:image/svg+xml;base64,",
            ") text\n",
        ),
        (
            format!("<p>{GRAPHIC}text</p>"),
            "![Logo](data:image/svg+xml;base64,",
            ")text\n",
        ),
        (
            format!("<a href=\"/x\">{GRAPHIC} text</a>"),
            "[![Logo](data:image/svg+xml;base64,",
            ") text](/x)\n",
        ),
        (
            format!("<p>one {GRAPHIC} two</p>"),
            "one ![Logo](data:image/svg+xml;base64,",
            ") two\n",
        ),
        (
            format!("<p>one {GRAPHIC}</p>"),
            "one ![Logo](data:image/svg+xml;base64,",
            ")\n",
        ),
    ] {
        let actual = convert_everywhere(&html, false).unwrap_or_else(|reason| panic!("{html:?}: {reason}"));
        assert!(
            actual.starts_with(start) && actual.ends_with(end),
            "{html:?} gives {actual:?}"
        );
    }
}

#[test]
fn the_space_after_other_replaced_content_is_kept() {
    assert_on_full(&[
        ("<p><video src=\"/v.mp4\"></video> text</p>", "[/v.mp4](/v.mp4) text\n"),
        // ~keep A check box outside a list item writes nothing, and no space with it.
        ("<p><input type=\"checkbox\"> text</p>", "text\n"),
    ]);
}

#[test]
fn white_space_of_any_kind_is_one_space() {
    assert_on_both(&[
        ("<p>one\n<b>two</b>\nthree</p>", "one **two** three\n"),
        ("<p>one <b>two</b> three</p>", "one **two** three\n"),
        ("<p>one\t<b>two</b>\tthree</p>", "one **two** three\n"),
        ("<p>one \n\t <b>two</b> \n\t three</p>", "one **two** three\n"),
        ("<p>one\r\n<b>two</b>\r\nthree</p>", "one **two** three\n"),
        ("<p>one<span> </span>two</p>", "one two\n"),
        ("<p>one<span>\n</span>two</p>", "one two\n"),
        ("<p>one<span>\t</span>two</p>", "one two\n"),
        ("<p>one<span><span>\n</span></span>two</p>", "one two\n"),
        ("<p><span> </span>one</p>", "one\n"),
        ("<p>one<span> </span></p>", "one\n"),
        ("<p>one<span>&nbsp;</span>two</p>", "one\u{a0}two\n"),
    ]);
}

#[test]
fn nothing_collapses_in_preformatted_text() {
    assert_on_both(&[(
        "<pre>one\n<span id=\"a\"></span>two  three\n\tfour</pre>",
        "```\none\ntwo  three\n\tfour\n```\n",
    )]);
}

#[test]
fn blocks_inside_a_link_are_separate_words_at_every_depth() {
    assert_on_full(&[
        (
            "<a href=\"/x\"><div>Next</div><div>Cross-references</div></a>",
            "[Next Cross-references](/x)\n",
        ),
        (
            "<a href=\"/x\"><div><div>Next</div><div>Cross-references</div></div></a>",
            "[Next Cross-references](/x)\n",
        ),
        (
            "<a href=\"/x\"><div><div><div>Next</div><div>Cross-references</div></div></div></a>",
            "[Next Cross-references](/x)\n",
        ),
        (
            "<a href=\"/x\"><div><div><span>Next</span></div><div>Cross-references</div></div></a>",
            "[Next Cross-references](/x)\n",
        ),
        (
            "<a href=\"/x\">\n  <div>\n    <div>Next</div>\n    <div>Cross-references</div>\n  </div>\n</a>",
            "[Next Cross-references](/x)\n",
        ),
        (
            "<a href=\"/x\"><div><p>Next</p><p>Cross-references</p></div></a>",
            "[Next Cross-references](/x)\n",
        ),
        (
            "<a href=\"/x\"><div>Next<div>Cross-references</div></div></a>",
            "[Next Cross-references](/x)\n",
        ),
        (
            "<a href=\"/x\"><div><div>Next</div>Cross-references</div></a>",
            "[Next Cross-references](/x)\n",
        ),
        (
            "<a href=\"/x\"><div><span>Next</span><div>Cross-references</div></div></a>",
            "[Next Cross-references](/x)\n",
        ),
        (
            "<a href=\"/x\"><div><div>a</div><div>b</div><div>c</div></div></a>",
            "[a b c](/x)\n",
        ),
        (
            "<a href=\"/x\">Next<b><div>Cross</div></b></a>",
            "[Next **Cross**](/x)\n",
        ),
        (
            "<a href=\"/x\"><b><div>Next</div></b>Cross</a>",
            "[**Next** Cross](/x)\n",
        ),
        (
            "<a href=\"/x\"><div><div></div><div>Next</div><div></div><div>Cross</div></div></a>",
            "[Next Cross](/x)\n",
        ),
        (
            "<p>See <a href=\"/x\"><div><div>Next</div><div>Cross-references</div></div></a> now</p>",
            "See [Next Cross-references](/x) now\n",
        ),
        (
            "<ul><li><a href=\"/x\"><div><div>Next</div><div>Cross</div></div></a></li></ul>",
            "- [Next Cross](/x)\n",
        ),
        (
            "<a class=\"right-next\" href=\"cross.html\" title=\"next page\">\n<div class=\"prev-next-info\">\n<p class=\"prev-next-subtitle\">next</p>\n<p class=\"prev-next-title\">Cross-references</p>\n</div>\n<i class=\"fa-solid fa-angle-right\"></i>\n</a>",
            "[next Cross-references](cross.html \"next page\")\n",
        ),
    ]);
}

#[test]
fn inline_content_beside_a_block_in_a_link_gets_no_space_of_its_own() {
    assert_on_full(&[
        ("<a href=\"/x\"><b>H</b>ello<div>x</div></a>", "[**H**ello x](/x)\n"),
        (
            "<a href=\"/x\"><span>a</span><span>b</span><div>c</div></a>",
            "[ab c](/x)\n",
        ),
        (
            "<a href=\"/x\"><span>a</span> <span>b</span><div>c</div></a>",
            "[a b c](/x)\n",
        ),
        (
            "<a href=\"/x\">Next<span>Cross<div>x</div></span></a>",
            "[NextCross x](/x)\n",
        ),
        (
            "<a href=\"/x\">Next<span><!-- c -->Cross<div>x</div></span></a>",
            "[NextCross x](/x)\n",
        ),
        (
            "<a href=\"/x\">Next<span>\n<div>Cross</div></span></a>",
            "[Next Cross](/x)\n",
        ),
        ("<a href=\"/x\"><div>x</div>text</a>", "[x text](/x)\n"),
        ("<a href=\"/x\">text<div>x</div></a>", "[text x](/x)\n"),
    ]);
}

/// A label that holds a block is written without Markdown images: an image is its alternative
/// text there, and a reader sees the image and the text beside it as two things.
#[test]
fn the_text_of_an_image_beside_a_block_in_a_link_is_a_word_of_its_own() {
    assert_on_full(&[
        (
            "<a href=\"/x\"><img src=\"/i.png\" alt=\"Logo\"><span>Docs</span><p>desc</p></a>",
            "[Logo Docs desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><span>Docs</span><img src=\"/i.png\" alt=\"Logo\"><p>desc</p></a>",
            "[Docs Logo desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><img src=\"/i.png\" alt=\"Logo\">Docs<div>desc</div></a>",
            "[Logo Docs desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><img src=\"/a.png\" alt=\"A\"><img src=\"/b.png\" alt=\"B\"><div>desc</div></a>",
            "[A B desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><img src=\"/i.png\" alt=\"Logo\"><b>Docs</b><p>desc</p></a>",
            "[Logo **Docs** desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><img src=\"/i.png\" alt=\"Logo\"> <span>Docs</span><p>desc</p></a>",
            "[Logo Docs desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><img src=\"/i.png\" alt=\"\"><span>Docs</span><p>desc</p></a>",
            "[Docs desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><span>Docs</span><img src=\"/i.png\" alt=\"\"><span>New</span><p>desc</p></a>",
            "[DocsNew desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><img src=\"/i.png\" alt=\"Logo\"><span id=\"a\"></span><span>Docs</span><p>desc</p></a>",
            "[Logo Docs desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><svg viewBox=\"0 0 1 1\"><title>Logo</title><path d=\"M0 0\"/></svg><span>Docs</span><p>desc</p></a>",
            "[Logo Docs desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><picture><source srcset=\"/a.webp\"><img src=\"/i.png\" alt=\"Logo\"></picture><span>Docs</span><p>desc</p></a>",
            "[Logo Docs desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><img src=\"/i.png\" alt=\"Logo\">.<p>desc</p></a>",
            "[Logo. desc](/x)\n",
        ),
        (
            "<a href=\"/x\">(<img src=\"/i.png\" alt=\"Logo\">)<p>desc</p></a>",
            "[(Logo) desc](/x)\n",
        ),
        (
            "<a href=\"/x\"><span>Docs</span><span>New</span><p>desc</p></a>",
            "[DocsNew desc](/x)\n",
        ),
    ]);
}

#[test]
fn a_block_in_code_in_a_heading_is_no_block_boundary() {
    assert_on_both(&[
        ("<h1>one<code><hr></code>two</h1>", "# one`---`two\n"),
        ("<h1>one\n<code><hr></code>two</h1>", "# one `---`two\n"),
        ("<h1>one<code><hr></code> two</h1>", "# one`---` two\n"),
        (
            "<blockquote><h2>one<code><hr></code>two</h2></blockquote>",
            "> ## one`---`two\n",
        ),
        ("<h1>one<code><p>y</p></code>two</h1>", "# one`y`two\n"),
    ]);
    assert_eq!(tier2("<h1>one<kbd><hr></kbd>two</h1>"), "# one`---`two\n");
    // ~keep In a link label the space beside such code is older than the rule: it stays.
    assert_eq!(
        tier2("<h2><a href=\"/h\">one<code><hr></code>two</a></h2>"),
        "## [one `--- ` two](/h)\n"
    );
}

/// A thematic break in a link in a heading is a block of the label: a label that starts with it
/// is a word of its own (issue #751). The fast converter gives these inputs to the full one. In
/// a table cell both converters write the label as before; in `pre` the full one writes the
/// link as its text and the break as a line of its own.
#[test]
fn a_thematic_break_in_a_link_in_a_heading_is_a_word_of_its_own() {
    assert_on_full(&[
        ("<h2>one<a href=\"/y\"><hr>y</a></h2>", "## one [--- y](/y)\n"),
        ("<h2>one<a href=\"/y\"><hr>y</a>two</h2>", "## one [--- y](/y)two\n"),
        ("<h2>one <a href=\"/y\"><hr>y</a>two</h2>", "## one [--- y](/y)two\n"),
        ("<h2>one<a href=\"/y\">y<hr></a>two</h2>", "## one[y ---](/y) two\n"),
        ("<h2>one<a><hr>y</a>two</h2>", "## one --- ytwo\n"),
        (
            "<pre>one<a href=\"/y\"><hr>y</a>two</pre>",
            "```\none\n\n---\nytwo\n```\n",
        ),
    ]);
    assert_on_both(&[(
        "<table><tr><td>one<a href=\"/y\"><hr>y</a>two</td></tr></table>",
        "| one[--- y](/y)two |\n| ----------------- |\n",
    )]);
}

#[test]
fn blocks_inside_a_heading_or_a_cell_are_separate_words() {
    assert_on_both(&[
        ("<h2><p>a</p><p>b</p></h2>", "## a b\n"),
        ("<h2><div><div>a</div><div>b</div></div></h2>", "## a b\n"),
        (
            "<table><tr><th>h</th></tr><tr><td><div><div>a</div><div>b</div></div></td></tr></table>",
            "| h   |\n| --- |\n| a b |\n",
        ),
    ]);
}
