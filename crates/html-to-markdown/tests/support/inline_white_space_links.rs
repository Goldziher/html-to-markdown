/// A link that has no text still writes its marks: the space after it is kept at the start of
/// a document, on both converters.
#[test]
fn the_space_after_a_link_without_text_is_kept() {
    assert_on_both(&[
        ("<p><a href=\"/y\"></a> one</p>", "[](/y) one\n"),
        ("<div><a href=\"/y\"></a> one</div>", "[](/y) one\n"),
        ("<a href=\"/y\"></a> one", "[](/y) one\n"),
        ("<p> <a href=\"/y\"></a>\none </p>", "[](/y) one\n"),
        ("<p><a id=\"n\"></a> one</p>", "one\n"),
    ]);
}

/// Each row is `(html, the full converter, the fast converter)`, with `None` for the fast
/// converter when it gives the input to the full one.
fn assert_written_as_before(cases: &[(&str, &str, Option<&str>)]) {
    for &(html, full, fast) in cases {
        assert_eq!(tier2(html), full, "full converter: {html:?}");
        assert_eq!(convert_with(html, None), full, "default options: {html:?}");
        let written = tier1::run(html, &PrescanReport::default(), &tier_options());
        assert_eq!(written.as_deref().ok(), fast, "fast converter: {html:?}");
    }
}

/// The white space at an end of a link label is not written: the words on the two sides are
/// joined (issue #800, open). Each row is the output of each converter before the rules of this
/// file, not the output of a browser, which shows a space there.
#[test]
fn white_space_at_an_end_of_a_link_label_is_written_as_before() {
    assert_written_as_before(&[
        (
            "<p>Press <a href=\"/p\">Go </a>now.</p>",
            "Press [Go](/p)now.\n",
            Some("Press [Go](/p)now.\n"),
        ),
        (
            "<p>Press<a href=\"/p\"> Go</a> now.</p>",
            "Press[Go](/p) now.\n",
            Some("Press[Go](/p) now.\n"),
        ),
        (
            "<p>Press<a href=\"/p\"> Go </a>now.</p>",
            "Press[Go](/p)now.\n",
            Some("Press[Go](/p)now.\n"),
        ),
        (
            "<p>Press <a href=\"/p\">Go\n</a>now.</p>",
            "Press [Go](/p)now.\n",
            Some("Press [Go](/p)now.\n"),
        ),
        (
            "<h2>Press <a href=\"/p\">Go </a>now.</h2>",
            "## Press [Go](/p)now.\n",
            Some("## Press [Go](/p)now.\n"),
        ),
        (
            "<ul><li>Press<a href=\"/p\"> Go</a> now.</li></ul>",
            "- Press[Go](/p) now.\n",
            Some("- Press[Go](/p) now.\n"),
        ),
        (
            "<p><a href=\"/a\">one </a><a href=\"/b\">two</a></p>",
            "[one](/a)[two](/b)\n",
            Some("[one](/a)[two](/b)\n"),
        ),
        (
            "<p>Press <a href=\"/p\"><b>Go </b></a>now.</p>",
            "Press [**Go**](/p)now.\n",
            Some("Press [**Go**](/p)now.\n"),
        ),
        (
            "<p>Press <b><a href=\"/p\">Go </a></b>now.</p>",
            "Press **[Go](/p)**now.\n",
            Some("Press **[Go](/p)**now.\n"),
        ),
        (
            "<p>Press <a href=\"/p\">Go&nbsp;</a>now.</p>",
            "Press [Go](/p)now.\n",
            Some("Press [Go](/p)now.\n"),
        ),
        (
            "<p>Press <a href=\"/p\">Go </a>.</p>",
            "Press [Go](/p).\n",
            Some("Press [Go](/p).\n"),
        ),
    ]);
}

/// The same for a link around an image, an autolink, a line break at an end of a label and a
/// short quotation (issue #800, open): each row is the output of each converter before the rules
/// of this file.
#[test]
fn white_space_at_an_end_of_an_image_link_an_autolink_or_a_short_quotation_is_written_as_before() {
    assert_written_as_before(&[
        // ~keep A link around an image.
        (
            "<p>Press <a href=\"/p\"><img src=\"/i.png\" alt=\"Go\"> </a>now.</p>",
            "Press [![Go](/i.png)](/p)now.\n",
            Some("Press [![Go](/i.png)](/p)now.\n"),
        ),
        (
            "<p>Press<a href=\"/p\"> <img src=\"/i.png\" alt=\"Go\"></a> now.</p>",
            "Press[![Go](/i.png)](/p) now.\n",
            Some("Press[![Go](/i.png)](/p) now.\n"),
        ),
        // ~keep An autolink.
        (
            "<p>Press <a href=\"https://e.org/\">https://e.org/ </a>now.</p>",
            "Press <https://e.org/>now.\n",
            Some("Press <https://e.org/>now.\n"),
        ),
        (
            "<p>Press<a href=\"https://e.org/\"> https://e.org/</a> now.</p>",
            "Press<https://e.org/> now.\n",
            Some("Press<https://e.org/> now.\n"),
        ),
        // ~keep A line break at an end of a label in a heading and in a table cell.
        (
            "<h2>Press<a href=\"/p\">Go<br></a>now</h2>",
            "## Press[Go](/p)now\n",
            Some("## Press[Go](/p)now\n"),
        ),
        (
            "<h2>Press<a href=\"/p\"><br>Go</a>now</h2>",
            "## Press[Go](/p)now\n",
            Some("## Press[Go](/p)now\n"),
        ),
        (
            "<table><tr><td>Press<a href=\"/p\">Go<br></a>now</td></tr></table>",
            "| Press[Go](/p)now |\n| ---------------- |\n",
            Some("| Press[Go](/p)now |\n| ---------------- |\n"),
        ),
        // ~keep A short quotation. The fast converter does not write its marks.
        ("<p>Press <q>Go </q>now.</p>", "Press \"Go\"now.\n", None),
        ("<p>Press<q> Go</q> now.</p>", "Press\"Go\" now.\n", None),
        ("<p>Press<q> Go </q>now.</p>", "Press\"Go\"now.\n", None),
        ("<p>Press <q>Go<br></q>now.</p>", "Press \"Go\"now.\n", None),
    ]);
}

/// The fast converter keeps the white space that bold text starts with in a link label, in a
/// paragraph and in a sectioning element, and the full converter does not (issue #800, open):
/// the row is the output of each converter before the rules of this file.
#[test]
fn white_space_that_bold_text_starts_with_in_a_link_label_is_written_as_before() {
    let html = "x<footer><a href=\"/x\"><b> Logo</b></a></footer>";
    assert_eq!(tier2(html), "x\n\n[**Logo**](/x)\n");
    let written = tier1::run(html, &PrescanReport::default(), &tier_options());
    assert_eq!(written.as_deref().ok(), Some("x\n\n[ **Logo**](/x)\n"));
}

/// A link with no address is running text: each row is written as the same content in a `span`
/// is. The white space at an end of the content is one space, as a browser shows it (issue
/// #762). The fast converter gives a link with no address and such white space to the full one.
/// In `pre` and in a code span the text is written as it is.
///
/// ~keep The rows of a list item are the output of the full converter, not of a browser: it
/// ~keep writes the indent of the item for a `span` as the fast converter does.
#[test]
fn a_link_with_no_address_is_written_as_a_span_is() {
    let cases = [
        (
            "<p><a> <img src=\"/i.png\" alt=\"Go\"> </a>now</p>",
            "![Go](/i.png) now\n",
        ),
        (
            "<p><a name=\"n\"> <img src=\"/i.png\" alt=\"Go\"> </a>now</p>",
            "![Go](/i.png) now\n",
        ),
        ("<p>Press <a>Go </a>now.</p>", "Press Go now.\n"),
        ("<p>Press<a> Go</a> now.</p>", "Press Go now.\n"),
        ("<p>Press <a> Go </a> now.</p>", "Press Go now.\n"),
        ("<p>Press <a href>Go </a>now.</p>", "Press Go now.\n"),
        ("<p>Press<a> </a>now.</p>", "Press now.\n"),
        ("<h2>Press <a>Go </a>now</h2>", "## Press Go now\n"),
        (
            "<table><tr><td>Press <a>Go </a>now</td></tr></table>",
            "| Press Go now |\n| ------------ |\n",
        ),
    ];
    assert_on_full(&cases);
    assert_on_both(&[("<p>Press <a>Go</a> now.</p>", "Press Go now.\n")]);
    for (html, expected) in cases {
        let span = html
            .replace("<a name=\"n\">", "<span>")
            .replace("<a href>", "<span>")
            .replace("<a>", "<span>")
            .replace("</a>", "</span>");
        assert_eq!(convert_everywhere(&span, false), Ok(expected.to_owned()), "{span:?}");
    }
    assert_on_full(&[
        ("<ul><li>Press <a>Go<br> </a> now</li></ul>", "- Press Go  \n  now\n"),
        ("<ul><li>Press<a>Go<br> </a> now</li></ul>", "- PressGo  \n  now\n"),
        ("<pre>Press <a>Go </a>now</pre>", "```\nPress Go now\n```\n"),
        ("<p><code>Press <a>Go </a>now</code></p>", "`Press Go now`\n"),
    ]);
}

/// A label of white space only has no ends, so no space goes outside the link. The full
/// converter writes the address as the label, the fast converter an empty label.
///
/// ~keep These rows pin the output of the base, not the output of a browser: a browser shows
/// ~keep the white space of such a link as a space between the two words.
#[test]
fn a_link_label_of_white_space_only_puts_no_space_outside_the_link() {
    let cases = [
        (
            "<p>Press<a href=\"/p\"> </a>now</p>",
            "Press[/p](/p)now\n",
            "Press[](/p)now\n",
        ),
        (
            "<p>Press <a href=\"/p\"> </a>.</p>",
            "Press [/p](/p).\n",
            "Press [](/p).\n",
        ),
        (
            "<p>Press<a href=\"/p\">&nbsp;</a>now</p>",
            "Press[/p](/p)now\n",
            "Press[](/p)now\n",
        ),
    ];
    for (html, full, fast) in cases {
        assert_eq!(tier2(html), full, "full converter: {html:?}");
        assert_eq!(convert_with(html, None), full, "default options: {html:?}");
        let written = tier1::run(html, &PrescanReport::default(), &tier_options());
        assert_eq!(written.as_deref().ok(), Some(fast), "fast converter: {html:?}");
    }
}

/// A code span keeps the white space of its text. A link in it is written as its text, and the
/// white space at an end of the label is not moved. The fast converter gives a link in a code
/// span to the full one.
///
/// ~keep These rows pin the output of the base, not the output of a browser, which shows one
/// ~keep space between the words: a code span keeps every space of the source.
#[test]
fn white_space_at_an_end_of_a_link_label_is_not_moved_in_a_code_span() {
    assert_on_full(&[
        (
            "<p><code>Press <a href=\"/p\"> Go </a> now</code></p>",
            "`Press  Go  now`\n",
        ),
        (
            "<p><code>Press <a href=\"/p\">Go </a> now</code></p>",
            "`Press Go  now`\n",
        ),
        (
            "<p><code>Press<a href=\"/p\"> Go </a>now</code></p>",
            "`Press Go now`\n",
        ),
        ("<p><code><a href=\"/p\"> Go </a></code></p>", "` Go `\n"),
        (
            "<p><code><a href=\"/p\"> <img src=\"/i.png\" alt=\"Go\"> </a></code></p>",
            "`  `\n",
        ),
        (
            "<p><kbd>Press <a href=\"/p\"> Go </a> now</kbd></p>",
            "`Press Go now`\n",
        ),
        // ~keep A link with no address is running text of the span, with its white space.
        ("<p><code>Press<a>Go </a>now</code></p>", "`PressGo now`\n"),
    ]);
}

/// In `<pre>`, `<code>`, `<kbd>` and `<samp>` no rule of running text runs. Each row pins the
/// output of the full converter before those rules, not the output of a browser: a link in
/// such an element is written as its text, with the white space of the source. The fast
/// converter gives a link in such an element to the full one; a block in it both write alike.
#[test]
fn the_rules_of_running_text_do_not_run_in_code() {
    assert_on_full(&[
        ("<pre>Press<a href=\"/p\">Go </a>now</pre>", "```\nPressGo now\n```\n"),
        ("<pre>Press<a href=\"/p\"> Go </a>now</pre>", "```\nPress Go now\n```\n"),
        ("<pre>Press<q> Go </q>now</pre>", "```\nPress\"Go\"now\n```\n"),
        (
            "<pre>one\n<a href=\"/y\">t<div>y</div></a></pre>",
            "```\none\nt\ny\n```\n",
        ),
        ("<p><kbd>Press<a name=\"n\">Go </a> now</kbd></p>", "`PressGo now`\n"),
    ]);
    assert_on_both(&[("<pre>x<section> a</section></pre>", "```\nx\n a\n```\n")]);
}

/// A block in marks in a heading is a word of its own: one space after the closing marks. The
/// fast converter gives this input to the full one.
#[test]
fn a_block_in_marks_in_a_heading_is_a_word_of_its_own() {
    assert_on_full(&[
        ("<h2>one\n<b><p>y</p></b>two</h2>", "## one **y** two\n"),
        ("<h2>one <i><div>y</div></i>two</h2>", "## one *y* two\n"),
        ("<h2>one<ins><p>y</p></ins>two</h2>", "## one ==y== two\n"),
        ("<h2>one<del><p>y</p></del>two</h2>", "## one ~~y~~ two\n"),
    ]);
    assert_on_both(&[("<h2>one <b>y</b>two</h2>", "## one **y**two\n")]);
}

/// A sectioning element (`<footer>`, `<section>`, `<article>`, `<aside>`, `<header>`, `<main>`)
/// is a block. The white space that a link label starts with is no space at the start of that
/// block, whatever lies between the element and the link.
#[test]
fn white_space_at_the_start_of_a_link_label_is_no_space_at_the_start_of_a_sectioning_element() {
    assert_on_both(&[
        ("x<footer><a href=\"/x\">\nLogo</a></footer>", "x\n\n[Logo](/x)\n"),
        (
            "x<footer><a href=\"/x\"> Logo</a> now</footer>",
            "x\n\n[Logo](/x) now\n",
        ),
        ("x<footer>\n<a href=\"/x\">\nLogo</a>\n</footer>", "x\n\n[Logo](/x)\n"),
        (
            "x<footer><div><a href=\"/x\">\nLogo</a></div></footer>",
            "x\n\n[Logo](/x)\n",
        ),
        (
            "<p>x</p><footer><p><a href=\"/x\">\nLogo</a></p></footer>",
            "x\n\n[Logo](/x)\n",
        ),
        (
            "x<section><div><a href=\"/x\">\nLogo</a></div></section>",
            "x\n\n[Logo](/x)\n",
        ),
        (
            "x<article><div><a href=\"/x\">\nLogo</a></div></article>",
            "x\n\n[Logo](/x)\n",
        ),
        (
            "x<aside><div><a href=\"/x\">\nLogo</a></div></aside>",
            "x\n\n[Logo](/x)\n",
        ),
        (
            "x<header><div><a href=\"/x\">\nLogo</a></div></header>",
            "x\n\n[Logo](/x)\n",
        ),
        (
            "x<main><article><section><div><a href=\"/x\">\nLogo</a></div></section></article></main>",
            "x\n\n[Logo](/x)\n",
        ),
        (
            "<p>x</p><footer><div>\n<a href=\"/\">\n<img src=\"/l.png\" alt=\"Logo\">\n</a>\n</div></footer>",
            "x\n\n[![Logo](/l.png)](/)\n",
        ),
        (
            "x<footer><div><a href=\"https://e.org/\"> https://e.org/</a></div></footer>",
            "x\n\n<https://e.org/>\n",
        ),
        (
            "x<footer><span><a href=\"/x\">\nLogo</a></span></footer>",
            "x\n\n[Logo](/x)\n",
        ),
        (
            "x<footer><div><b><a href=\"/x\">\nLogo</a></b></div></footer>",
            "x\n\n**[Logo](/x)**\n",
        ),
        (
            "x<footer><a href=\"/x\">\nLogo</a></footer><footer><a href=\"/y\">\nTwo</a></footer>",
            "x\n\n[Logo](/x)\n\n[Two](/y)\n",
        ),
        (
            "<div>x<footer><a href=\"/x\">\nLogo</a></footer></div>",
            "x\n\n[Logo](/x)\n",
        ),
        (
            "<blockquote>x<footer><a href=\"/x\">\nLogo</a></footer></blockquote>",
            "> x\n>\n> [Logo](/x)\n",
        ),
    ]);
    assert_on_full(&[
        (
            "<ul><li>x<footer><a href=\"/x\">\nLogo</a></footer></li></ul>",
            "- x\n\n  [Logo](/x)\n",
        ),
        (
            "<ul><li><footer><a href=\"/x\">\nLogo</a></footer></li></ul>",
            "- [Logo](/x)\n",
        ),
        ("x<footer><div><q> Logo</q></div></footer>", "x\n\n\"Logo\"\n"),
    ]);
}

/// The other content that a sectioning element can start with follows the same rule: text, and
/// an inline element that writes marks.
#[test]
fn a_sectioning_element_does_not_start_with_a_space() {
    assert_on_both(&[
        ("x<footer> Logo</footer>", "x\n\nLogo\n"),
        ("x<footer><b> Logo</b></footer>", "x\n\n**Logo**\n"),
        ("x<footer> <b>Logo</b></footer>", "x\n\n**Logo**\n"),
        ("x<footer><div><i> Logo</i></div></footer>", "x\n\n*Logo*\n"),
    ]);
    // ~keep The fast converter gives a link with no address and white space at an end to the
    // ~keep full one.
    assert_on_full(&[("x<footer><div><a> Logo</a></div></footer>", "x\n\nLogo\n")]);
}

/// Only the one space of collapsed white space is removed at the start of a sectioning
/// element: the indent of a code block stays, and so does white space in the strict mode.
#[test]
fn a_sectioning_element_keeps_an_indent_and_strict_white_space() {
    let indented = convert_with(
        "x<footer><pre><code>a</code></pre></footer>",
        Some(ConversionOptions {
            code_block_style: CodeBlockStyle::Indented,
            tier_strategy: TierStrategy::Tier2,
            ..tier_options()
        }),
    );
    assert_eq!(indented, "x\n\n    a\n");
    let strict = convert_with(
        "x<footer> Logo</footer>",
        Some(ConversionOptions {
            whitespace_mode: WhitespaceMode::Strict,
            tier_strategy: TierStrategy::Tier2,
            ..tier_options()
        }),
    );
    assert_eq!(strict, "x\n\n Logo\n");
}

/// The strict white space mode keeps the white space of the source. No rule of running text
/// applies there: each row is the output that the strict mode had before those rules. The strict
/// mode always runs on the full converter.
#[test]
fn the_strict_mode_writes_what_it_wrote_before_the_rules_of_running_text() {
    let cases = [
        // ~keep The white space at an end of a link label writes no space outside the link.
        ("<p>Press <a href=\"/p\">Go </a> now</p>", "Press [Go](/p) now\n"),
        ("<p><a href=\"/p\"> Go </a>now</p>", "[Go](/p)now\n"),
        // ~keep A marked element writes the white space it starts with, also after a space.
        ("<p>one <b> y</b>two</p>", "one  **y**two\n"),
        // ~keep A short quotation writes nothing outside its marks.
        ("<p>Press <q> Go </q> now</p>", "Press \"Go\" now\n"),
        ("<p>Press<q>Go<br></q>now</p>", "Press\"Go\"now\n"),
        // ~keep A block boundary in a heading writes no space of its own.
        ("<h2>x<footer> Logo</footer></h2>", "## x Logo\n"),
        // ~keep Two children of a link label that touch get one space when a block is among them.
        ("<a href=\"/x\"><b>H</b>ello<div>x</div></a>", "[**H** ello x](/x)\n"),
    ];
    let wrong: Vec<String> = cases
        .iter()
        .flat_map(|(html, expected)| {
            [TierStrategy::Tier2, TierStrategy::default()].map(|tier_strategy| {
                let actual = convert_with(
                    html,
                    Some(ConversionOptions {
                        whitespace_mode: WhitespaceMode::Strict,
                        tier_strategy,
                        ..tier_options()
                    }),
                );
                (actual != *expected).then(|| format!("{html:?}\n  expected {expected:?}\n  actual   {actual:?}"))
            })
        })
        .flatten()
        .collect();
    assert!(wrong.is_empty(), "{} wrong:\n{}", wrong.len(), wrong.join("\n"));
}

/// In a code span the white space at the ends of a short quotation is not moved out of the marks:
/// the quotation is written as before that rule, as a link in a code span is. A browser shows no
/// space outside the marks that the source does not have: `Press" Go "now` and `Press " Go " now`.
#[test]
fn white_space_at_an_end_of_a_short_quotation_is_not_moved_in_a_code_span() {
    assert_on_full(&[
        ("<p><code><q> Go </q></code></p>", "`\"Go\"`\n"),
        ("<p><code>Press<q> Go </q>now</code></p>", "`Press\"Go\"now`\n"),
        ("<p><code>Press <q> Go </q> now</code></p>", "`Press \"Go\" now`\n"),
        ("<p><code>Press <q> Go </q>.</code></p>", "`Press \"Go\".`\n"),
    ]);
}

/// A no-break space beside a space in a link label is one space, on both converters.
#[test]
fn a_no_break_space_beside_a_space_in_a_link_label_is_one_space() {
    assert_on_both(&[
        (
            "<p><a href=\"/x\">one<span>&nbsp;</span> two</a></p>",
            "[one two](/x)\n",
        ),
        ("<p><a href=\"/x\"><span>&nbsp;</span> one</a></p>", "[one](/x)\n"),
        ("<p><a href=\"/x\">one&nbsp;two</a></p>", "[one two](/x)\n"),
    ]);
}
