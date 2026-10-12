//! The start tag and the end tag of a raw-text element follow the HTML rule on both converters:
//! the name in any case, then tab, line feed, form feed, carriage return, space, `/` or `>`.

#![allow(missing_docs)]
// ~keep The converter is chosen by name, which the crate offers under this feature only.
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

const STRATEGIES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2];

fn content(html: &str, tier_strategy: TierStrategy) -> String {
    let options = ConversionOptions {
        tier_strategy,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
        .trim_end()
        .to_string()
}

const AFTER_THE_NAME: [&str; 7] = [" ", "\t", "\n", "\x0C", "\r", "/", ""];

/// The JSON of a structured data script: no tag, a start tag, and the start tag of a script.
const JSON_VALUES: [&str; 3] = [r#"{"a":"b"}"#, r#"{"a":"<p>"}"#, r#"{"a":"<script>"}"#];

/// Issue 825: a slash or a form feed after the name of a script start tag. The page was empty
/// when the JSON held `<script>`.
#[test]
fn a_script_start_tag_is_read_with_every_byte_after_its_name() {
    for name in ["script", "SCRIPT"] {
        for after_name in ["/", "\x0C", " ", "\t", "\n", "\r"] {
            for value in JSON_VALUES {
                let html = format!(
                    "<html><head><{name}{after_name}type=\"application/ld+json\">{value}</script></head>\
                     <body><h1>Shop</h1><p>visible</p></body></html>"
                );
                for tier_strategy in STRATEGIES {
                    assert_eq!(
                        content(&html, tier_strategy),
                        "# Shop\n\nvisible",
                        "{tier_strategy:?}: {html:?}"
                    );
                }
            }
        }
    }
}

/// Issue 825: a plain script with a slash or a form feed after its name is removed.
#[test]
fn a_plain_script_start_tag_is_read_with_every_byte_after_its_name() {
    for after_name in ["/", "\x0C", " "] {
        let html = format!("<p>before</p>\n<script{after_name}async>var a = \"<p>SECRET\";</script>\n<p>after</p>");
        for tier_strategy in STRATEGIES {
            assert_eq!(
                content(&html, tier_strategy),
                "before\n\nafter",
                "{tier_strategy:?}: {html:?}"
            );
        }
    }
}

/// Issue 826: the end tag of a style or script element in every spelling.
#[test]
fn a_raw_text_end_tag_is_read_with_every_byte_after_its_name() {
    for (name, end_name) in [("style", "style"), ("style", "STYLE"), ("script", "script")] {
        for after_name in AFTER_THE_NAME {
            let html =
                format!("<p>before</p><{name}>p::before{{content:\"<p>SECRET\"}}</{end_name}{after_name}><p>after</p>");
            for tier_strategy in STRATEGIES {
                assert_eq!(
                    content(&html, tier_strategy),
                    "before\n\nafter",
                    "{tier_strategy:?}: {html:?}"
                );
            }
        }
    }
}

/// A longer name does not end the element, on both converters.
#[test]
fn a_longer_name_does_not_end_a_style_element() {
    let html = "<p>before</p><style>a{}</styles>b{}</style><p>after</p>";
    for tier_strategy in STRATEGIES {
        assert_eq!(content(html, tier_strategy), "before\n\nafter", "{tier_strategy:?}");
    }
}

/// Issue 828: the content of a textarea is text. The fast converter hands a textarea over, so
/// the rows run the automatic choice and the full converter.
#[test]
fn the_tags_inside_a_textarea_are_text() {
    for (html, expected) in [
        (
            "<textarea><b>bold</b><i>it</i></textarea><div><p>one</p></div>tail",
            "<b>bold</b><i>it</i>\n\none\n\ntail",
        ),
        (
            "<textarea><b>bold</b><i>it</i></textarea><div><p>one</div>tail",
            "<b>bold</b><i>it</i>\n\none\n\ntail",
        ),
        (
            "<TEXTAREA\x0Cname=a><b>bold</b><i>it</i></textarea\x0C><div><p>one</p></div>tail",
            "<b>bold</b><i>it</i>\n\none\n\ntail",
        ),
    ] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            assert_eq!(content(html, tier_strategy), expected, "{tier_strategy:?}: {html:?}");
        }
    }
}

const STRUCTURED_DATA: &str = r#"<script type="application/ld+json">{"a":1}</script>"#;
const PLAIN_SCRIPT: &str = "<script>x</script>";

/// Issue 827: a structured data script adds no space of its own, on both converters. The white
/// space on its two sides collapses as if the script were not there, as a browser shows it.
#[test]
fn a_structured_data_script_adds_no_space() {
    for (html, expected) in [
        (format!("<p>foo {STRUCTURED_DATA} bar</p>"), "foo bar"),
        (format!("<p>foo{STRUCTURED_DATA} bar</p>"), "foo bar"),
        (format!("<p>foo {STRUCTURED_DATA}bar</p>"), "foo bar"),
        (format!("<p>foo{STRUCTURED_DATA}bar</p>"), "foobar"),
    ] {
        for tier_strategy in STRATEGIES {
            assert_eq!(content(&html, tier_strategy), expected, "{tier_strategy:?}: {html:?}");
        }
    }
    let html = format!("<pre>foo{STRUCTURED_DATA}bar</pre>");
    for tier_strategy in STRATEGIES {
        let found = content(&html, tier_strategy);
        assert!(found.contains("foobar"), "{tier_strategy:?}: {found:?}");
    }
}

/// A script with `<script` in its text ends at its first end tag, with every start tag form.
#[test]
fn a_script_with_a_start_tag_in_its_text_ends_at_its_first_end_tag() {
    for after_name in ["", " async", "/async", "\x0Casync"] {
        let html = format!("<p>before</p><script{after_name}>var s = \"<script>\";</script><p>after</p>");
        for tier_strategy in STRATEGIES {
            assert_eq!(
                content(&html, tier_strategy),
                "before\n\nafter",
                "{tier_strategy:?}: {html:?}"
            );
        }
    }
}

/// A removed script with white space on its two sides gives the text of the kept script.
#[test]
fn a_plain_script_between_two_spaces_leaves_one_space() {
    let html = format!("<p>foo {PLAIN_SCRIPT} bar</p>");
    for tier_strategy in STRATEGIES {
        assert_eq!(content(&html, tier_strategy), "foo bar", "{tier_strategy:?}");
    }
}

const STYLE: &str = "<style>p{}</style>";

/// Issue 827: in a table cell the white space on the two sides of a script, a style element or
/// a comment gives one space, as in a paragraph, on both converters.
#[test]
fn an_element_that_draws_nothing_between_two_spaces_leaves_one_space_in_a_cell() {
    for element in [STRUCTURED_DATA, PLAIN_SCRIPT, STYLE] {
        let html = format!("<table><tr><td>foo {element} bar</td></tr></table>");
        for tier_strategy in STRATEGIES {
            let found = content(&html, tier_strategy);
            assert!(
                found.starts_with("| foo bar |\n"),
                "{tier_strategy:?}: {html:?} gave {found:?}"
            );
        }
    }
}

/// The content of a cell that ends a part of its text with a space before more text.
const CELLS_WITH_A_SPACE_BEFORE_TEXT: [&str; 6] = [
    "foo <!-- c --> bar",
    "<b>foo </b> bar",
    "<strong>foo&nbsp;</strong> bar",
    "<em>foo </em> bar",
    "<x-widget>foo &nbsp;</x-widget> bar",
    "foo <x-widget></x-widget> bar",
];

/// The fast converter and the full converter write the same cell for each of these contents.
#[test]
fn the_space_that_opens_a_text_in_a_cell_is_the_same_on_both_converters() {
    for cell in CELLS_WITH_A_SPACE_BEFORE_TEXT {
        let html = format!("<table><tr><td>{cell}</td></tr></table>");
        let full = content(&html, TierStrategy::Tier2);
        assert_eq!(content(&html, TierStrategy::Tier1), full, "{html:?}");
        assert_eq!(content(&html, TierStrategy::Auto), full, "{html:?}");
    }
    let html = "<table><tr><td>foo <!-- c --> bar</td></tr></table>";
    for tier_strategy in STRATEGIES {
        let found = content(html, tier_strategy);
        assert!(
            found.starts_with("| foo bar |\n"),
            "{tier_strategy:?}: {html:?} gave {found:?}"
        );
    }
}

/// A comment and a script side by side between two spaces leave one space, on both converters.
#[test]
fn a_comment_beside_a_script_between_two_spaces_leaves_one_space() {
    for script in [STRUCTURED_DATA, PLAIN_SCRIPT] {
        for (before, after) in [
            ("<p>foo ", " bar</p>"),
            ("<ul><li>foo ", " bar</li></ul>"),
            ("<table><tr><td>foo ", " bar</td></tr></table>"),
        ] {
            let html = format!("{before}<!-- c -->{script}{after}");
            for tier_strategy in STRATEGIES {
                let without = content(&format!("{before}{after}"), tier_strategy);
                assert!(without.contains("foo bar"), "{tier_strategy:?}: {without:?}");
                assert_eq!(content(&html, tier_strategy), without, "{tier_strategy:?}: {html:?}");
            }
        }
    }
}

/// A list item and a heading use the rule of a paragraph for the same three elements.
#[test]
fn an_element_that_draws_nothing_between_two_spaces_leaves_one_space_in_an_item_and_a_heading() {
    for element in [STRUCTURED_DATA, PLAIN_SCRIPT, STYLE] {
        for (before, after) in [("<ul><li>foo ", " bar</li></ul>"), ("<h2>foo ", " bar</h2>")] {
            let html = format!("{before}{element}{after}");
            for tier_strategy in STRATEGIES {
                let without = content(&format!("{before}{after}"), tier_strategy);
                assert!(without.ends_with("foo bar"), "{tier_strategy:?}: {without:?}");
                assert_eq!(content(&html, tier_strategy), without, "{tier_strategy:?}: {html:?}");
            }
        }
    }
}

/// Issue 827: a structured data script gives the text of the same page without the script, in
/// every context and on both converters.
#[test]
fn a_structured_data_script_gives_the_text_of_the_page_without_it() {
    for (before, after) in [
        ("<pre>foo", "bar</pre>"),
        ("<pre>foo ", " bar</pre>"),
        ("<p><code>foo", "bar</code></p>"),
        ("<p><b>foo</b>", "<i>bar</i></p>"),
        ("<p><b>foo</b> ", " <i>bar</i></p>"),
        ("<p>foo</p>", "<p>bar</p>"),
        ("<p>foo</p>", "bar"),
        ("<ul><li>foo", "</li><li>bar</li></ul>"),
        ("<table><tr><td>foo", "bar</td></tr></table>"),
        ("<h1>foo ", " bar</h1>"),
    ] {
        for tier_strategy in STRATEGIES {
            let without = content(&format!("{before}{after}"), tier_strategy);
            let structured_data = content(&format!("{before}{STRUCTURED_DATA}{after}"), tier_strategy);
            assert_eq!(structured_data, without, "{tier_strategy:?}: {before:?} {after:?}");
            assert!(
                without.contains("foo") && without.contains("bar"),
                "{tier_strategy:?}: {without:?}"
            );
        }
    }
}

/// Issue 827: a line end in an inline element, then a structured data script, then text that
/// starts with a space. The full converter asks what follows the inline element, and the
/// script is not what follows: the text is. The fast converter does not ask this question.
#[test]
fn a_structured_data_script_is_not_what_follows_an_inline_element() {
    let (before, after) = ("<p>Alpha<span>\n</span>", " 13</p>");
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
        let without = content(&format!("{before}{after}"), tier_strategy);
        let structured_data = content(&format!("{before}{STRUCTURED_DATA}{after}"), tier_strategy);
        assert_eq!(structured_data, without, "{tier_strategy:?}");
        assert!(
            without.contains("Alpha") && without.contains("13"),
            "{tier_strategy:?}: {without:?}"
        );
    }
}

#[cfg(feature = "metadata")]
mod structured_data {
    use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

    /// The JSON of each script, by the automatic choice and by the full converter.
    fn raw_json(html: &str) -> Vec<Vec<String>> {
        [TierStrategy::Auto, TierStrategy::Tier2]
            .into_iter()
            .map(|tier_strategy| {
                let options = ConversionOptions {
                    tier_strategy,
                    ..ConversionOptions::default()
                };
                convert(html, Some(options))
                    .expect("conversion should succeed")
                    .metadata
                    .structured_data
                    .into_iter()
                    .map(|data| data.raw_json)
                    .collect()
            })
            .collect()
    }

    /// Issue 825: a script start tag with any byte of the HTML rule after its name gives its
    /// JSON as written.
    #[test]
    fn a_script_start_tag_gives_its_json_with_every_byte_after_its_name() {
        for name in ["script", "SCRIPT"] {
            for after_name in ["/", "\x0C", " ", "\t", "\n", "\r"] {
                for value in super::JSON_VALUES {
                    let html = format!(
                        "<html><head><{name}{after_name}type=\"application/ld+json\">{value}</script></head>\
                         <body><h1>Shop</h1><p>visible</p></body></html>"
                    );
                    assert_eq!(raw_json(&html), vec![vec![value.to_string()]; 2], "{html:?}");
                }
            }
        }
    }

    /// Issue 824 holds beside the rule for `preserve_tags`: with no `preserve_tags` the metadata
    /// gives the JSON as written, on a page as written and on a repaired page.
    #[test]
    fn a_json_with_an_ampersand_is_given_as_written() {
        for json in super::JSON_WITH_AN_AMPERSAND {
            let page = format!("<p>foo <script type=\"application/ld+json\">{json}</script> bar</p><p>after</p>");
            for html in [page.clone(), format!("{}{page}", super::REPAIRED_PAGE_START)] {
                assert_eq!(raw_json(&html), vec![vec![json.to_string()]; 2], "{html:?}");
            }
        }
    }
}

/// A textarea with no tag in it gives the same text as before.
#[test]
fn a_textarea_without_a_tag_is_unchanged() {
    let html = "<p>before</p><textarea>a &amp; b</textarea><p>after</p>";
    let auto = content(html, TierStrategy::Auto);
    assert_eq!(auto, content(html, TierStrategy::Tier2));
    assert!(auto.contains("a & b"), "{auto:?}");
}

/// The name of a raw-text element in a comment starts no element: the page after it is read.
#[test]
fn a_raw_text_start_tag_in_a_comment_starts_nothing() {
    for (name, element, shown) in [
        ("textarea", "<textarea>t</textarea>", "\n\nt"),
        ("script", "<script>x</script>", ""),
        ("style", "<style>x</style>", ""),
    ] {
        let html = format!("<p>before</p><!-- <{name}> --><p>mid <b>bold</b></p>{element}<p>after</p>");
        for tier_strategy in STRATEGIES {
            assert_eq!(
                content(&html, tier_strategy),
                format!("before\n\nmid **bold**{shown}\n\nafter"),
                "{tier_strategy:?}: {html:?}"
            );
        }
    }
}

/// A `textarea` start tag and its end tag in two comments hold nothing: the markup between the
/// two comments is read as markup.
#[test]
fn a_textarea_tag_in_each_of_two_comments_leaves_the_markup_between_them() {
    let html = "<!--<textarea>--><b>Shown</b><!--</textarea>--><p>after</p>";
    for tier_strategy in STRATEGIES {
        assert_eq!(content(html, tier_strategy), "**Shown**\n\nafter", "{tier_strategy:?}");
    }
}

/// A `script` or `style` start tag and its end tag in two comments hold nothing either.
#[test]
fn a_script_tag_in_each_of_two_comments_leaves_the_markup_between_them() {
    for name in ["script", "style"] {
        let html = format!("<!--<{name}>--><b>Shown</b><!--</{name}>--><p>after</p>");
        for tier_strategy in STRATEGIES {
            assert_eq!(
                content(&html, tier_strategy),
                "**Shown**\n\nafter",
                "{tier_strategy:?}: {html:?}"
            );
        }
    }
}

/// The name of a raw-text element in an attribute value starts no element: the title is the
/// value as written and the page after the link is read.
#[test]
fn a_raw_text_start_tag_in_an_attribute_value_starts_nothing() {
    for (name, element, shown) in [
        ("textarea", "<textarea>t</textarea>", "\n\nt"),
        ("script", "<script>x</script>", ""),
        ("style", "<style>x</style>", ""),
    ] {
        for quote in ["\"", "'"] {
            let html = format!(
                "<p>before</p><a href=\"/x\" title={quote}<{name}>{quote}>link</a><p>mid <b>bold</b></p>\
                 {element}<p>after</p>"
            );
            for tier_strategy in STRATEGIES {
                assert_eq!(
                    content(&html, tier_strategy),
                    format!("before\n\n[link](/x \"<{name}>\")\n\nmid **bold**{shown}\n\nafter"),
                    "{tier_strategy:?}: {html:?}"
                );
            }
        }
    }
}

/// A whole element of raw text in an attribute value is a part of the value, also on a void
/// element in upper case, and a start tag in a value with no quotes ends at its `>`.
#[test]
fn a_raw_text_element_in_an_attribute_value_is_a_part_of_the_value() {
    for name in ["script", "style", "textarea"] {
        let in_a_title = format!("<a href=\"/x\" title=\"<{name}></{name}>\">x</a><p>after <b>bold</b></p>");
        let in_a_void = format!("<p>before</p><IMG src=\"/i.png\" alt=\"<{name}>\"><p>after <b>bold</b></p>");
        let no_quotes = format!("<p title=<{name}>y</p><p>after <b>bold</b></p>");
        for tier_strategy in STRATEGIES {
            assert_eq!(
                content(&in_a_title, tier_strategy),
                format!("[x](/x \"<{name}></{name}>\")\n\nafter **bold**"),
                "{tier_strategy:?}: {in_a_title:?}"
            );
            let markdown = content(&in_a_void, tier_strategy);
            assert!(
                markdown.starts_with("before\n\n![") && markdown.ends_with("\n\nafter **bold**"),
                "{tier_strategy:?}: {in_a_void:?}: {markdown:?}"
            );
            assert!(
                !markdown.contains("</"),
                "{tier_strategy:?}: {in_a_void:?}: {markdown:?}"
            );
            assert_eq!(
                content(&no_quotes, tier_strategy),
                "y\n\nafter **bold**",
                "{tier_strategy:?}: {no_quotes:?}"
            );
        }
    }
}

/// An apostrophe in an attribute value with no quotes opens no quote in the first pass: the
/// `textarea` after the tag is an element and the tags inside it are text.
///
/// ~keep The full converter only. The fast converter reads the tag of the link with a reader
/// ~keep of its own, which opens a quote at the apostrophe. It did so before this rule; see
/// ~keep the issue "an apostrophe in an attribute value with no quotes opens a quote on the
/// ~keep fast converter".
#[test]
fn an_apostrophe_in_an_unquoted_attribute_value_opens_no_quote() {
    let html = "<p>before</p><a href=/x title=it's>link</a><textarea><b>x</b></textarea><p>it's after</p>";
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
        let markdown = content(html, tier_strategy);
        assert!(
            markdown.contains("<b>x</b>") && markdown.ends_with("it's after"),
            "{tier_strategy:?}: {html:?}: {markdown:?}"
        );
    }
}

/// Issue 277: the full converter closes a `script` or `style` start tag with no end tag at
/// once and writes the markup after it. Each spelling of the start tag follows that rule.
///
/// ~keep The fast converter writes nothing after such a start tag. It did so before this rule;
/// ~keep see the issue "a script or style start tag with no end tag ends the page on the fast
/// ~keep converter only".
#[test]
fn a_raw_text_start_tag_with_no_end_tag_follows_one_rule_in_every_spelling() {
    let plain = content("<p>a</p><script><p>b</p>", TierStrategy::Tier2);
    assert_eq!(plain, "a\n\nb");
    for start_tag in [
        "<script>",
        "<script data=it's>",
        "<script\x0C>",
        "<style>",
        "<style media=it's>",
        "<style\x0C>",
    ] {
        let html = format!("<p>a</p>{start_tag}<p>b</p>");
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            assert_eq!(content(&html, tier_strategy), plain, "{tier_strategy:?}: {html:?}");
        }
        assert_eq!(content(&html, TierStrategy::Tier1), "a", "the fast converter: {html:?}");
    }
}

/// A `textarea` start tag with no end tag gives one text in every spelling of the start tag.
#[test]
fn a_textarea_start_tag_with_no_end_tag_follows_one_rule_in_every_spelling() {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
        let plain = content("<p>a</p><textarea><p>b</p>", tier_strategy);
        for start_tag in ["<textarea data=it's>", "<textarea\x0C>"] {
            let html = format!("<p>a</p>{start_tag}<p>b</p>");
            assert_eq!(content(&html, tier_strategy), plain, "{tier_strategy:?}: {html:?}");
        }
    }
}

/// An `=` and an apostrophe in an attribute value with no quotes are bytes of the value: the
/// value ends at white space or at `>`. The `textarea` after the link is an element and the
/// tags inside it are text, as they are with the same value in quotes.
///
/// ~keep The full converter only, for the reason given at
/// ~keep `an_apostrophe_in_an_unquoted_attribute_value_opens_no_quote`.
#[test]
fn an_equals_sign_in_an_unquoted_attribute_value_starts_no_value() {
    let no_quotes = "<a href=/search?q='tis>link</a><textarea><b>x</b></textarea><p>it's after</p>";
    let quotes = "<a href=\"/search?q='tis\">link</a><textarea><b>x</b></textarea><p>it's after</p>";
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
        for html in [no_quotes, quotes] {
            assert_eq!(
                content(html, tier_strategy),
                "[link](/search?q='tis) <b>x</b>\n\nit's after",
                "{tier_strategy:?}: {html:?}"
            );
        }
    }
}

/// The forms of a bogus comment: a tokenizer reads each one to its first `>`.
const BOGUS_COMMENTS: [(&str, &str); 3] = [("<?x ", "?>"), ("<! ", ">"), ("</ ", ">")];

/// The start tag and the end tag of a raw-text element in two bogus comments hold nothing: the
/// markup between the two comments is read as markup.
#[test]
fn a_raw_text_tag_in_each_of_two_bogus_comments_leaves_the_markup_between_them() {
    for name in ["textarea", "script", "style"] {
        for (open, close) in BOGUS_COMMENTS {
            let html = format!("<p>before</p>{open}<{name}>{close}<b>Shown</b>{open}</{name}>{close}<p>after</p>");
            for tier_strategy in STRATEGIES {
                let markdown = content(&html, tier_strategy);
                assert!(
                    markdown.starts_with("before\n\n") && markdown.contains("**Shown**") && markdown.ends_with("after"),
                    "{tier_strategy:?}: {html:?}: {markdown:?}"
                );
            }
        }
    }
}

/// A `textarea` start tag and its end tag in two CDATA sections hold nothing either.
#[test]
fn a_textarea_tag_in_each_of_two_cdata_sections_leaves_the_markup_between_them() {
    let html = "<p>before</p><![CDATA[<textarea>]]><b>Shown</b><![CDATA[</textarea>]]><p>after</p>";
    for tier_strategy in STRATEGIES {
        let markdown = content(html, tier_strategy);
        assert!(
            markdown.contains("**Shown**") && markdown.ends_with("after"),
            "{tier_strategy:?}: {markdown:?}"
        );
    }
}

/// A real `textarea` after a bogus comment that holds a `textarea` start tag is an element:
/// the tags inside it are text.
#[test]
fn a_textarea_after_a_bogus_comment_is_an_element() {
    let html = "<p>before</p></ <textarea>><b>Shown</b><textarea>x <i>y</i></textarea><p>after</p>";
    for tier_strategy in STRATEGIES {
        let markdown = content(html, tier_strategy);
        assert!(
            markdown.contains("**Shown**") && markdown.contains("x <i>y</i>") && markdown.ends_with("after"),
            "{tier_strategy:?}: {markdown:?}"
        );
    }
}

/// In a table cell a no-break space after a space is kept when an empty inline element is
/// between the two, also after `. ` and after a link. A script between the two is no element:
/// the space and the no-break space are one space.
#[test]
fn a_no_break_space_in_a_table_cell_is_kept_after_an_element_and_not_after_a_script() {
    for (cell, expected) in [
        ("a. <span></span>&nbsp;b", "| a.  b |\n| ----- |"),
        (
            "<a href=\"/y\">x</a> <span></span>&nbsp;b",
            "| [x](/y)  b |\n| ---------- |",
        ),
        ("a. <script>x()</script>&nbsp;b", "| a. b |\n| ---- |"),
        ("a <style>p{}</style>&nbsp;b", "| a b |\n| --- |"),
    ] {
        let html = format!("<table><tr><td>{cell}</td></tr></table>");
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            assert_eq!(content(&html, tier_strategy), expected, "{tier_strategy:?}: {html:?}");
        }
    }
}

/// The JSON of a structured data script with a `&`: in a URL, in a reference, in the reference
/// of a `<`.
const JSON_WITH_AN_AMPERSAND: [&str; 3] = [
    r#"{"a":"https://e.org/?a=1&b=2"}"#,
    r#"{"a":"a &amp; b"}"#,
    r#"{"a":"a &lt; b"}"#,
];

/// A page that the converter repairs before it reads it: a `div` in a `p` and a custom element.
const REPAIRED_PAGE_START: &str = "<p>one<div>two</div><my-part>q</my-part>";

fn content_with_preserved(html: &str, tag: &str, tier_strategy: TierStrategy) -> String {
    let options = ConversionOptions {
        tier_strategy,
        preserve_tags: vec![tag.to_string()],
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
        .trim_end()
        .to_string()
}

/// A structured data script that `preserve_tags` keeps, by its own name or by the name of a
/// parent, is written with the `&` of the page: `?a=1&b=2` gave `?a=1&amp;b=2`, and `&amp;` gave
/// `&amp;amp;`.
#[test]
fn a_preserved_structured_data_script_keeps_the_ampersands_of_the_page() {
    for json in JSON_WITH_AN_AMPERSAND {
        let script = format!(r#"<script type="application/ld+json">{json}</script>"#);
        for (tag, html, expected) in [
            (
                "script",
                format!("<p>foo {script} bar</p><p>after</p>"),
                format!("foo {script} bar\n\nafter"),
            ),
            (
                "div",
                format!("<div>foo {script} bar</div><p>after</p>"),
                format!("<div>foo {script} bar</div>\n\nafter"),
            ),
        ] {
            for tier_strategy in STRATEGIES {
                assert_eq!(
                    content_with_preserved(&html, tag, tier_strategy),
                    expected,
                    "{tier_strategy:?}, {tag}: {html:?}"
                );
            }
        }
    }
}

/// The same on a page that the converter repairs, with the script in the body and in the head.
#[test]
fn a_preserved_structured_data_script_keeps_the_ampersands_of_a_repaired_page() {
    for json in JSON_WITH_AN_AMPERSAND {
        let script = format!(r#"<script type="application/ld+json">{json}</script>"#);
        for (tag, page, expected) in [
            (
                "script",
                format!("<p>foo {script} bar</p><p>after</p>"),
                format!("one\n\ntwo\n\nq\n\nfoo {script} bar\n\nafter"),
            ),
            (
                "div",
                format!("<div>foo {script} bar</div><p>after</p>"),
                format!("one\n\n<div>two</div>\n\nq<div>foo {script} bar</div>\n\nafter"),
            ),
            (
                "script",
                format!("<html><head>{script}</head><body><p>foo bar</p></body></html>"),
                format!("one\n\ntwo\n\nq{script}\n\nfoo bar"),
            ),
        ] {
            let html = format!("{REPAIRED_PAGE_START}{page}");
            for tier_strategy in STRATEGIES {
                assert_eq!(
                    content_with_preserved(&html, tag, tier_strategy),
                    expected,
                    "{tier_strategy:?}, {tag}: {html:?}"
                );
            }
        }
    }
}

/// A structured data script in the head of a page that needs no repair is not written, with
/// `preserve_tags` or without.
#[test]
fn a_structured_data_script_in_a_head_is_not_written() {
    for json in JSON_WITH_AN_AMPERSAND {
        let html = format!(
            "<html><head><script type=\"application/ld+json\">{json}</script></head>\
             <body><p>foo bar</p></body></html>"
        );
        for tier_strategy in STRATEGIES {
            assert_eq!(content(&html, tier_strategy), "foo bar", "{tier_strategy:?}: {html:?}");
            assert_eq!(
                content_with_preserved(&html, "script", tier_strategy),
                "foo bar",
                "{tier_strategy:?}: {html:?}"
            );
        }
    }
}

/// With no `preserve_tags` the script is not written, on a page as written and on a repaired
/// page.
#[test]
fn a_structured_data_script_with_an_ampersand_is_not_written() {
    for json in JSON_WITH_AN_AMPERSAND {
        let page = format!("<p>foo <script type=\"application/ld+json\">{json}</script> bar</p><p>after</p>");
        for tier_strategy in STRATEGIES {
            assert_eq!(
                content(&page, tier_strategy),
                "foo bar\n\nafter",
                "{tier_strategy:?}: {page:?}"
            );
            let repaired = format!("{REPAIRED_PAGE_START}{page}");
            assert_eq!(
                content(&repaired, tier_strategy),
                "one\n\ntwo\n\nq\n\nfoo bar\n\nafter",
                "{tier_strategy:?}: {repaired:?}"
            );
        }
    }
}
