#![allow(missing_docs)]
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, OutputFormat, TierStrategy, convert, prescan, tier1};

fn convert_with(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion succeeds")
        .content
        .unwrap_or_default()
}

/// The default strategy, the fast converter's options, the fast converter forced (the full
/// converter answers only when the fast converter refuses the page) and the full converter alone.
fn paths() -> [ConversionOptions; 4] {
    let fast = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    [
        ConversionOptions::default(),
        fast.clone(),
        ConversionOptions {
            tier_strategy: TierStrategy::Tier1,
            ..fast.clone()
        },
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..fast
        },
    ]
}

fn assert_rows(rows: &[(&str, &str)]) {
    for (html, expected) in rows {
        for options in paths() {
            assert_eq!(convert_with(html, options), *expected, "{html}");
        }
    }
}

#[test]
fn should_keep_every_space_at_an_image_with_no_source_in_preformatted_text() {
    // A browser shows both spaces inside `pre`.
    assert_rows(&[
        ("<pre>a <img> b</pre>", "```\na  b\n```\n"),
        ("<pre>a <img>&#32;b</pre>", "```\na  b\n```\n"),
        ("<pre>a <img><!-- c --> b</pre>", "```\na  b\n```\n"),
    ]);
}

#[test]
fn should_write_one_space_at_an_image_with_no_source_before_a_space() {
    assert_rows(&[("<p>a <img> b</p>", "a b\n"), ("<p>a <img>b</p>", "a b\n")]);
}

#[test]
fn should_write_one_space_at_an_image_with_no_source_before_a_character_reference() {
    assert_rows(&[
        ("<p>a <img>&#32;b</p>", "a b\n"),
        ("<p>a <img>&#x20;b</p>", "a b\n"),
        ("<p>a <img>&Tab;b</p>", "a b\n"),
    ]);
}

#[test]
fn should_write_one_space_at_an_image_with_no_source_before_a_comment() {
    assert_rows(&[
        ("<p>a <img><!-- c --> b</p>", "a b\n"),
        ("<p>a <img><!-- c --><!-- d -->&#32;b</p>", "a b\n"),
    ]);
}

#[test]
fn should_keep_the_space_at_an_image_with_no_source_before_a_no_break_space() {
    // A browser shows the space and the no-break space: only ASCII white space folds. The
    // converter writes a no-break space in running text as a space, so two spaces stay.
    assert_rows(&[
        ("<p>a <img>&nbsp;b</p>", "a  b\n"),
        ("<p>a <img>&#160;b</p>", "a  b\n"),
        ("<p>a <img>\u{a0}b</p>", "a  b\n"),
    ]);
}

#[test]
fn should_keep_a_no_break_space_after_a_space_only_across_an_element_with_no_text() {
    // ~keep Each carrier stands between the space and the no-break space. `fast` is what the fast
    // ~keep converter does when it is called directly: its text, or the reason it hands the page
    // ~keep over with. An element with no text keeps the two apart. Inside one text, and with
    // ~keep only a comment between, the two are one run of white space, as on `main`.
    let carriers: [(&str, &str, Result<&str, &str>); 4] = [
        ("<img>", "a  b\n", Err("ImageMissingSource")),
        ("<span></span>", "a  b\n", Err("UnicodeSpaceAfterSpace")),
        ("<!-- c -->", "a b\n", Err("UnicodeSpaceAfterSpace")),
        ("", "a b\n", Ok("a b\n")),
    ];
    let [_, fast, ..] = paths();
    let mut wrong = Vec::new();
    for (carrier, expected, expected_fast) in carriers {
        for form in ["&nbsp;", "&#160;", "&#xA0;", "\u{a0}"] {
            let html = format!("<p>a {carrier}{form}b</p>");
            for options in paths() {
                let written = convert_with(&html, options);
                if written != expected {
                    wrong.push(format!("{html:?}: {written:?}, expected {expected:?}"));
                }
            }
            let direct = fast_direct(&html, &fast);
            if direct.as_deref().map_err(String::as_str) != expected_fast {
                wrong.push(format!("{html:?}: fast {direct:?}, expected {expected_fast:?}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{} wrong:\n{}", wrong.len(), wrong.join("\n"));
}

#[test]
fn should_join_a_no_break_space_to_a_space_at_a_line_break_and_at_an_element_start() {
    // ~keep A `<br>` in a heading is a space, and the no-break space after it would start the
    // ~keep line. The no-break space that starts an element joins the space before the element.
    // ~keep The fast converter writes the heading rows itself: its `<br>` there is a line end. It
    // ~keep writes the row of a text that starts where the opening tag ends. A comment hides
    // ~keep that place, and it hands the page over. The space that ends an element with text is the same run as
    // ~keep the no-break space after the element, as on `main`: the fast converter writes that
    // ~keep row or hands the page over (`None`). A form feed is no Unicode space: the fast
    // ~keep converter answers.
    let [_, fast, ..] = paths();
    let rows: [(&str, &str, Option<bool>); 8] = [
        ("<h2>a<br>&nbsp;b</h2>", "## a b\n", Some(true)),
        ("<h2>a<br>\u{a0}b</h2>", "## a b\n", Some(true)),
        ("<p>a <span>&nbsp;x</span></p>", "a x\n", Some(true)),
        ("<p>a <span>&#xA0;x</span></p>", "a x\n", Some(true)),
        ("<ul><li>&nbsp;b</li></ul>", "- b\n", Some(true)),
        ("<p>a <span><!-- c -->&#160;x</span></p>", "a x\n", Some(false)),
        ("<p><b>a </b>&nbsp;b</p>", "**a** b\n", None),
        ("<p>a <span></span>\u{c}b</p>", "a b\n", Some(true)),
    ];
    for (html, expected, fast_answers) in rows {
        for options in paths() {
            assert_eq!(convert_with(html, options), expected, "{html}");
        }
        let direct = fast_direct(html, &fast);
        let direct = direct.as_deref().map_err(String::as_str);
        let hands_over = direct == Err("UnicodeSpaceAfterSpace");
        match fast_answers {
            Some(true) => assert_eq!(direct, Ok(expected), "fast: {html}"),
            Some(false) => assert!(hands_over, "fast: {html}: {direct:?}"),
            None => assert!(hands_over || direct == Ok(expected), "fast: {html}: {direct:?}"),
        }
    }
}

#[test]
fn should_keep_a_no_break_space_after_a_space_in_plain_text() {
    for html in ["<p>a <img>&nbsp;b</p>", "<p>a <span></span>&nbsp;b</p>"] {
        for options in paths() {
            let plain = ConversionOptions {
                output_format: OutputFormat::Plain,
                ..options
            };
            assert_eq!(convert_with(html, plain).trim_end_matches('\n'), "a  b", "{html}");
        }
    }
}

#[test]
fn should_keep_a_no_break_space_after_a_space_in_a_table_cell() {
    // ~keep The fast converter hands both rows over, so each route writes the full converter's cell.
    let cell = "| h    |\n| ---- |\n| a  b |\n";
    assert_rows(&[
        (
            "<table><tr><th>h</th></tr><tr><td>a <img>&nbsp;b</td></tr></table>",
            cell,
        ),
        (
            "<table><tr><th>h</th></tr><tr><td>a <span></span>&nbsp;b</td></tr></table>",
            cell,
        ),
    ]);
}

#[test]
fn should_write_a_no_break_space_after_a_space_in_code_text_on_the_fast_converter() {
    // ~keep In `pre` and in a code span no white space rule applies: the no-break space stays the
    // ~keep character, and the fast converter answers.
    assert_literal_on_both(
        &[
            ("<pre>a <span></span>&nbsp;b</pre>", "```\na \u{a0}b\n```\n"),
            ("<p><code>a <span></span>&nbsp;b</code></p>", "`a \u{a0}b`\n"),
        ],
        &no_metadata(),
    );
    assert_rows(&[("<pre>a <img>&nbsp;b</pre>", "```\na \u{a0}b\n```\n")]);
}

#[test]
fn should_keep_a_no_break_space_reference_with_no_semicolon_after_a_space() {
    // ~keep A browser reads `&nbsp` with no semicolon as the no-break space. The fast converter
    // ~keep hands both rows over: the image has no source, and it does not read that reference.
    let [_, fast, ..] = paths();
    let rows = [
        ("<p>a <img>&nbspb</p>", "ImageMissingSource"),
        ("<p>a <span></span>&nbsp b</p>", "UnknownEntity"),
    ];
    for (html, reason) in rows {
        for options in paths() {
            assert_eq!(convert_with(html, options), "a  b\n", "{html}");
        }
        let direct = fast_direct(html, &fast);
        assert!(
            direct.as_ref().is_err_and(|found| found.starts_with(reason)),
            "fast: {html}: {direct:?}"
        );
    }
}

#[test]
fn should_keep_a_no_break_space_after_a_line_break_in_a_table_cell() {
    // ~keep A `<br>` in a table cell is a space between two words, and the no-break space after
    // ~keep it is a character, as on `main`. The fast converter does not know the place of a
    // ~keep text after a `<br>`: it hands each row over. Text after the `<br>` gets one space.
    let [_, fast, ..] = paths();
    let kept = "| h    |\n| ---- |\n| a  b |\n";
    for cell in [
        "a<br>&nbsp;b",
        "a<br>&#160;b",
        "a <br>&nbsp;b",
        "a<br> &nbsp;b",
        "a<br><!-- c -->&nbsp;b",
    ] {
        let html = format!("<table><tr><th>h</th></tr><tr><td>{cell}</td></tr></table>");
        for options in paths() {
            assert_eq!(convert_with(&html, options), kept, "{html}");
        }
        assert_eq!(
            fast_direct(&html, &fast),
            Err("UnicodeSpaceAfterSpace".to_string()),
            "fast: {html}"
        );
    }
    assert_rows(&[(
        "<table><tr><th>h</th></tr><tr><td>a<br>b</td></tr></table>",
        "| h   |\n| --- |\n| a b |\n",
    )]);
}

#[test]
fn should_keep_a_no_break_space_after_a_line_break_in_a_heading_in_a_table_cell() {
    // ~keep A heading in a table cell is text of the cell: its `<br>` is a space between two
    // ~keep words there too, as on `main`. The fast converter hands each row over.
    let [_, fast, ..] = paths();
    let kept = "| h    |\n| ---- |\n| a  b |\n";
    let rows = [
        ("", "<h2>a<br>&nbsp;b</h2>", kept.to_string()),
        ("", "<h2>a <br>&nbsp;b</h2>", kept.to_string()),
        (
            "",
            "<h2><b>a<br>&nbsp;b</b></h2>",
            "| h        |\n| -------- |\n| **a  b** |\n".to_string(),
        ),
        ("<h2>t</h2>", "<h3>a<br>&nbsp;b</h3>", format!("## t\n\n{kept}")),
    ];
    for (before, cell, expected) in rows {
        let html = format!("{before}<table><tr><th>h</th></tr><tr><td>{cell}</td></tr></table>");
        for options in paths() {
            assert_eq!(convert_with(&html, options), expected, "{html}");
        }
        assert_eq!(
            fast_direct(&html, &fast),
            Err("UnicodeSpaceAfterSpace".to_string()),
            "fast: {html}"
        );
    }
}

#[test]
fn should_keep_a_no_break_space_after_an_element_of_white_space_only() {
    // ~keep An element that holds only white space, comments and empty elements holds no text,
    // ~keep as on `main`: its white space is the run of the space before it, and the no-break
    // ~keep space after it is a character. The fast converter hands each row over.
    let [_, fast, ..] = paths();
    let rows = [
        ("<p>a <span> </span>&nbsp;b</p>", "a  b\n"),
        ("<p>a <span>  </span>&nbsp;b</p>", "a  b\n"),
        ("<p>a <span>\n</span>&nbsp;b</p>", "a  b\n"),
        ("<p>a <span>&#9;</span>&#160;b</p>", "a  b\n"),
        ("<p>a <span><span> </span></span>&nbsp;b</p>", "a  b\n"),
        ("<p>a <span> <!-- c --> </span>&nbsp;b</p>", "a  b\n"),
        ("<p>a <span></span><span> </span>&nbsp;b</p>", "a  b\n"),
        ("<p>a <span> </span><!-- c -->&nbsp;b</p>", "a  b\n"),
        ("<p>a <span> </span> &nbsp;b</p>", "a  b\n"),
        ("<p>a<span> </span>&nbsp;b</p>", "a  b\n"),
        ("<h2>a <span> </span>&nbsp;b</h2>", "## a  b\n"),
        ("<ul><li>a <span> </span>&nbsp;b</li></ul>", "- a  b\n"),
        (
            "<table><tr><th>h</th></tr><tr><td>a <span> </span>&nbsp;b</td></tr></table>",
            "| h    |\n| ---- |\n| a  b |\n",
        ),
    ];
    for (html, expected) in rows {
        for options in paths() {
            assert_eq!(convert_with(html, options), expected, "{html}");
        }
        assert_eq!(
            fast_direct(html, &fast),
            Err("UnicodeSpaceAfterSpace".to_string()),
            "fast: {html}"
        );
    }
    // ~keep White space before the text of an element does not make the element empty.
    assert_rows(&[("<p>a <span> <b>x </b></span>&nbsp;b</p>", "a **x** b\n")]);
}

#[test]
fn should_keep_a_no_break_space_after_a_line_break_and_an_empty_element_in_a_heading() {
    // ~keep A heading has one line: its `<br>` is a space, and the no-break space after an
    // ~keep element with no text is a character, as on `main`. The fast converter writes a hard
    // ~keep break for that `<br>` and does not know the place of the text: it hands each row over.
    let [_, fast, ..] = paths();
    let rows = [
        ("<h2>a<br><span></span>&nbsp;b</h2>", "## a  b\n"),
        ("<h2>a<br><i></i><span></span>&nbsp;b</h2>", "## a  b\n"),
        ("<h2>a<br><span> </span>&nbsp;b</h2>", "## a  b\n"),
        ("<h2>a<span><br></span>&nbsp;b</h2>", "## a  b\n"),
        ("<h2><b>a<br><span></span>&nbsp;b</b></h2>", "## **a  b**\n"),
    ];
    for (html, expected) in rows {
        for options in paths() {
            assert_eq!(convert_with(html, options.clone()), expected, "{html}");
            let backslash = ConversionOptions {
                newline_style: html_to_markdown_rs::NewlineStyle::Backslash,
                ..options
            };
            assert_eq!(convert_with(html, backslash), expected, "backslash: {html}");
        }
        assert_eq!(
            fast_direct(html, &fast),
            Err("UnicodeSpaceAfterSpace".to_string()),
            "fast: {html}"
        );
    }
}

/// The fast converter called directly: its text, or the reason when it hands the page over.
fn fast_direct(html: &str, options: &ConversionOptions) -> Result<String, String> {
    let (cleaned, report) = prescan::run(html);
    tier1::run(cleaned.as_ref(), &report, options).map_err(|reason| format!("{reason:?}"))
}

fn full_only(html: &str, options: &ConversionOptions) -> String {
    convert_with(
        html,
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..options.clone()
        },
    )
}

/// Each row on both converters: the fast converter must accept the page and write `expected`.
fn assert_literal_on_both(rows: &[(&str, &str)], options: &ConversionOptions) {
    for (html, expected) in rows {
        let fast = fast_direct(html, options);
        assert_eq!(
            fast.as_deref().map(str::trim_end),
            Ok(expected.trim_end()),
            "fast: {html}"
        );
        assert_eq!(full_only(html, options), *expected, "full: {html}");
    }
}

fn no_metadata() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        ..ConversionOptions::default()
    }
}

#[test]
fn should_write_the_alt_text_of_an_image_in_a_code_span_as_code_text() {
    assert_literal_on_both(
        &[
            ("<p><code>a<img src=\"/i.png\" alt=\"pic\">b</code></p>", "`apicb`\n"),
            ("<p><code>a<img src=\"/i.png\" alt=\"*p*\">b</code></p>", "`a*p*b`\n"),
        ],
        &no_metadata(),
    );
}

#[test]
fn should_write_nothing_for_an_image_with_no_alt_text_in_a_code_span() {
    assert_literal_on_both(
        &[
            ("<p><code>a<img src=\"/i.png\">b</code></p>", "`ab`\n"),
            ("<p><code>a<img src=\"/i.png\" alt=\"\">b</code></p>", "`ab`\n"),
        ],
        &no_metadata(),
    );
}

#[test]
fn should_write_the_alt_text_of_an_image_in_keyboard_and_sample_text_as_code_text() {
    assert_literal_on_both(
        &[
            ("<p><kbd>a<img src=\"/i.png\" alt=\"pic\">b</kbd></p>", "`apicb`\n"),
            ("<p><samp>a<img src=\"/i.png\" alt=\"pic\">b</samp></p>", "`apicb`\n"),
        ],
        &no_metadata(),
    );
}

#[test]
fn should_write_the_alt_text_of_an_image_in_preformatted_text_as_code_text() {
    assert_literal_on_both(
        &[("<pre>a<img src=\"/i.png\" alt=\"pic\">b</pre>", "```\napicb\n```\n")],
        &no_metadata(),
    );
}

#[test]
fn should_write_the_alt_text_of_each_of_two_images_in_a_code_span() {
    assert_literal_on_both(
        &[(
            "<p><code><img src=\"/i.png\" alt=\"one\">+<img src=\"/j.png\" alt=\"two\"></code></p>",
            "`one+two`\n",
        )],
        &no_metadata(),
    );
}

/// Each row: the fast converter hands the page over, and the full converter writes `expected`.
fn assert_handed_to_full(rows: &[(&str, &str)], options: &ConversionOptions) {
    for (html, expected) in rows {
        assert_eq!(fast_direct(html, options), Err("Classifier".to_owned()), "fast: {html}");
        assert_eq!(full_only(html, options), *expected, "full: {html}");
    }
}

#[test]
fn should_hand_white_space_between_two_images_in_a_code_span_to_the_full_converter() {
    assert_handed_to_full(
        &[
            (
                "<p><code><img src=\"/i.png\" alt=\"one\"> <img src=\"/j.png\" alt=\"two\"></code></p>",
                "`one two`\n",
            ),
            (
                "<p><code><img src=\"/i.png\" alt=\"one\">  <img src=\"/j.png\" alt=\"two\"></code></p>",
                "`one  two`\n",
            ),
            (
                "<p><code><img src=\"/i.png\"> <img src=\"/j.png\" alt=\"two\"></code></p>",
                "` two`\n",
            ),
        ],
        &no_metadata(),
    );
}

#[test]
fn should_hand_white_space_before_an_image_at_the_start_of_a_code_span_to_the_full_converter() {
    assert_handed_to_full(
        &[(
            "<p>text <code> <img src=\"/i.png\" alt=\"pic\"></code></p>",
            "text ` pic`\n",
        )],
        &no_metadata(),
    );
}

#[test]
fn should_hand_white_space_after_an_image_at_the_end_of_a_code_span_to_the_full_converter() {
    let html = "<p><code><img src=\"/i.png\" alt=\"pic\"> </code></p>";
    assert_eq!(fast_direct(html, &no_metadata()), Err("Classifier".to_owned()));
    let full = full_only(html, &no_metadata());
    assert!(full.contains("pic") && !full.contains("!["), "{full:?}");
}

#[test]
fn should_hand_white_space_beside_an_image_in_a_code_span_in_a_table_cell_to_the_full_converter() {
    assert_handed_to_full(
        &[(
            "<table><tr><th>h</th></tr><tr><td><code><img src=\"/i.png\"> <img alt=\"two\" src=\"/j.png\"></code></td></tr></table>",
            "| h      |\n| ------ |\n| ` two` |\n",
        )],
        &no_metadata(),
    );
}

#[test]
fn should_write_the_text_after_a_code_span_that_holds_only_an_image_as_the_full_converter_does() {
    let options = no_metadata();
    let html = "<p><code><img src=\"/i.png\" alt=\"pic\"></code> y</p>";
    let full = full_only(html, &options);
    let fast = fast_direct(html, &options);
    assert_eq!(fast.as_deref().map(str::trim_end), Ok(full.trim_end()));
}

#[test]
fn should_write_nothing_for_an_image_in_a_code_span_when_images_are_skipped() {
    let options = ConversionOptions {
        skip_images: true,
        ..no_metadata()
    };
    assert_literal_on_both(
        &[("<p><code>a<img src=\"/i.png\" alt=\"pic\">b</code></p>", "`ab`\n")],
        &options,
    );
}

#[test]
fn should_hand_an_image_in_a_link_in_a_code_span_to_the_full_converter() {
    // The fast converter hands every link in code text to the full converter, which splits the
    // code span at the link. The image in that link is its alt text there too.
    let html = "<p><code>a<a href=\"/y\"><img src=\"/i.png\" alt=\"pic\"></a>b</code></p>";
    let fast = fast_direct(html, &no_metadata());
    assert_eq!(fast, Err("Classifier".to_owned()));
    let full = full_only(html, &no_metadata());
    assert!(full.contains("pic") && !full.contains("!["), "{full:?}");
}

#[test]
fn should_hand_an_image_with_a_line_end_in_its_alt_text_in_a_code_span_to_the_full_converter() {
    let html = "<p><code>a<img src=\"/i.png\" alt=\"p&#10;c\">b</code></p>";
    let fast = fast_direct(html, &no_metadata());
    assert_eq!(fast, Err("Classifier".to_owned()));
    assert!(!full_only(html, &no_metadata()).contains("!["));
}

#[test]
fn should_keep_the_text_of_a_cell_inside_a_cell() {
    for options in paths() {
        let html = "<table><tr><td>a<td>One</td>items</td><td>z</td></tr></table>";
        let output = convert_with(html, options);
        assert_eq!(output.lines().next(), Some("| aOneitems | z |"), "{output:?}");
    }
}

#[test]
fn should_write_the_empty_label_for_a_link_that_holds_an_empty_division() {
    assert_rows(&[
        ("<p>a <a href=\"/page\"><span></span></a> b</p>", "a [](/page) b\n"),
        ("<p>a <a href=\"/page\"><div></div></a> b</p>", "a [](/page) b\n"),
        ("<p>a <a href=\"/page\"><DIV> </DIV></a> b</p>", "a [](/page) b\n"),
    ]);
}

#[test]
fn should_keep_the_address_as_the_label_for_a_link_that_holds_a_division_with_a_class() {
    // The vote link of the #121 fixture: a style sheet draws the arrow, so the link is visible.
    assert_rows(&[(
        "<p>a <a href=\"/vote\"><div class=\"votearrow\" title=\"upvote\"></div></a> b</p>",
        "a [/vote](/vote) b\n",
    )]);
}

#[test]
fn should_write_the_empty_label_for_a_link_that_holds_an_empty_italic_with_no_class() {
    assert_rows(&[
        ("<p>a <a href=\"/page\"><i></i></a> b</p>", "a [](/page) b\n"),
        ("<p>a <a href=\"/page\"><I>&#32;</I></a> b</p>", "a [](/page) b\n"),
        // The icon font form keeps the address as its label (#774).
        (
            "<p>a <a href=\"/cart\"><i class=\"icon-cart\"></i></a> b</p>",
            "a [/cart](/cart) b\n",
        ),
    ]);
}

#[test]
fn should_make_the_header_rule_as_wide_as_a_cell_with_a_nested_table() {
    assert_rows(&[(
        "<table><tr><td>o<table><tr><td></td><td>b</td></tr><tr><td>c</td><td></td></tr></table></td><td>x</td></tr></table>",
        "| o b c | x |\n| ----- | --- |\n",
    )]);
}

#[test]
fn should_keep_a_backslash_in_a_code_span_that_holds_a_link() {
    // A browser shows `a\bl` in code, and `l` is the link.
    assert_rows(&[("<p><code>a\\b<a href=\"/x\">l</a></code></p>", "`a\\b`[`l`](/x)\n")]);
}

#[test]
fn should_write_a_blank_line_between_a_text_after_a_quote_and_a_rule() {
    // With one line end before the rule, Markdown reads `text` as a heading.
    assert_rows(&[("<blockquote>q</blockquote>text<hr>", "> q\n\ntext\n\n---\n")]);
}
