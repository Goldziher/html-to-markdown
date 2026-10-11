#![allow(missing_docs)]
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert, prescan, tier1};

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
    // ~keep Issue #792 decides: a no-break space at the start of a text is the one space of that
    // ~keep edge, and it joins a space already written, as after an empty `span`. An image with
    // ~keep no source writes nothing. Before the sync: two spaces ("a  b\n").
    assert_rows(&[
        ("<p>a <img>&nbsp;b</p>", "a b\n"),
        ("<p>a <img>&#160;b</p>", "a b\n"),
        ("<p>a <img>\u{a0}b</p>", "a b\n"),
    ]);
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
