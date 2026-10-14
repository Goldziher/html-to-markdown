//! A bold element after bold text that holds two or more `<br>`, or a blank element before its
//! `<br>`, keeps the hard break at every position (issue #897). ~keep
//!
//! Only a wrapper that holds one `<br>` and nothing else is the separator of issue #502.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, NewlineStyle, TierStrategy, convert};

fn assert_routes(html: &str, expected: &str, newline_style: NewlineStyle) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            html,
            Some(ConversionOptions {
                newline_style,
                tier_strategy,
                extract_metadata: false,
                ..ConversionOptions::default()
            }),
        )
        .expect("the page converts");
        assert_eq!(
            result.content.as_deref(),
            Some(expected),
            "{html:?}, {tier_strategy:?}, {newline_style:?}"
        );
    }
}

/// Each row: the page, the text with two-space breaks, the text with backslash breaks.
fn assert_rows(rows: &[(&str, &str, &str)]) {
    for (html, spaces, backslash) in rows {
        assert_routes(html, spaces, NewlineStyle::Spaces);
        assert_routes(html, backslash, NewlineStyle::Backslash);
    }
}

#[test]
fn should_keep_the_break_of_two_breaks_in_a_later_block() {
    assert_rows(&[
        (
            "<h1>Notes</h1><p><b>Name</b><b><br><br></b>Ada</p>",
            "# Notes\n\n**Name**  \nAda\n",
            "# Notes\n\n**Name**\\\nAda\n",
        ),
        (
            "<p>xy</p><p><b>a</b><b><br><br></b><b>c</b></p>",
            "xy\n\n**a**  \n**c**\n",
            "xy\n\n**a**\\\n**c**\n",
        ),
        (
            "<p>xy</p><p><b>a</b><b><br><br><br></b><b>c</b></p>",
            "xy\n\n**a**  \n**c**\n",
            "xy\n\n**a**\\\n**c**\n",
        ),
    ]);
}

#[test]
fn should_keep_the_break_of_two_breaks_in_the_block_that_starts_the_output() {
    assert_rows(&[
        (
            "<p><b>a</b><b><br><br></b><b>c</b></p>",
            "**a**  \n**c**\n",
            "**a**\\\n**c**\n",
        ),
        (
            "<p><b>a</b><mark><br><br></mark><b>c</b></p>",
            "**a**  \n**c**\n",
            "**a**\\\n**c**\n",
        ),
    ]);
}

#[test]
fn should_keep_the_break_of_two_breaks_in_a_list_item_a_block_quote_and_a_link_label() {
    assert_rows(&[
        (
            "<ul><li><b>a</b><b><br><br></b><b>c</b></li></ul>",
            "- **a**  \n**c**\n",
            "- **a**\\\n**c**\n",
        ),
        (
            "<blockquote><b>a</b><b><br><br></b><b>c</b></blockquote>",
            "> **a**  \n> **c**\n",
            "> **a**\\\n> **c**\n",
        ),
        (
            "<p><a href=\"u\"><b>a</b><b><br><br></b><b>c</b></a></p>",
            "[**a**  \n**c**](u)\n",
            "[**a**\\\n**c**](u)\n",
        ),
    ]);
}

#[test]
fn should_keep_the_break_after_a_blank_span() {
    assert_rows(&[
        (
            "<p><b>a</b><b><span> </span><br></b><b>c</b></p>",
            "**a**  \n**c**\n",
            "**a** \\\n**c**\n",
        ),
        (
            "<b>a</b><b><span> </span><br></b><b>c</b>",
            "**a**  \n**c**\n",
            "**a** \\\n**c**\n",
        ),
        (
            "<ul><li><b>a</b><b><span> </span><br></b><b>c</b></li></ul>",
            "- **a**  \n**c**\n",
            "- **a** \\\n**c**\n",
        ),
        (
            "<blockquote><b>a</b><b><span> </span><br></b><b>c</b></blockquote>",
            "> **a**  \n> **c**\n",
            "> **a** \\\n> **c**\n",
        ),
    ]);
}

#[test]
fn should_keep_the_separator_where_no_break_ends_the_wrapper() {
    for style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
        assert_routes(
            "<p><b>a</b><b><span></span><br></b><b>c</b></p>",
            "**a** **c**\n",
            style,
        );
        assert_routes("<p><b>a</b><b><br><br> </b><b>c</b></p>", "**a** **c**\n", style);
        assert_routes(
            "<p><b>a</b><b><span> </span><br> </b><b>c</b></p>",
            "**a** **c**\n",
            style,
        );
        assert_routes("<h2><b>a</b><b><br><br></b><b>c</b></h2>", "## **a** **c**\n", style);
    }
}
