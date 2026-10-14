//! A bold element that holds only a `<br>`, after a bold text, keeps its hard break in every
//! block that does not start the output, and in a link label. ~keep
//!
//! `x<p><b>x</b><b><br></b>x</p>` wrote `**x**`, a hard break and `x` until the break-only
//! wrapper after bold text became one space at every position. Only the block that starts the
//! output had that space before (issue #502), and it keeps it.
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
fn should_keep_the_break_of_the_smallest_page() {
    assert_rows(&[("x<p><b>x</b><b><br></b>x</p>", "x\n\n**x**  \nx\n", "x\n\n**x**\\\nx\n")]);
}

#[test]
fn should_keep_the_break_in_a_paragraph_after_other_content() {
    assert_rows(&[
        (
            "<p>x</p><p><b>a</b><b><br></b><b>c</b></p>",
            "x\n\n**a**  \n**c**\n",
            "x\n\n**a**\\\n**c**\n",
        ),
        (
            "<h1>x</h1><p><b>a</b><b><br></b><b>c</b></p>",
            "# x\n\n**a**  \n**c**\n",
            "# x\n\n**a**\\\n**c**\n",
        ),
        (
            "<ul><li>x</li></ul><p><b>a</b><b><br></b><b>c</b></p>",
            "- x\n\n**a**  \n**c**\n",
            "- x\n\n**a**\\\n**c**\n",
        ),
        (
            "<div><p>x</p><p><b>a</b><b><br></b><b>c</b></p></div>",
            "x\n\n**a**  \n**c**\n",
            "x\n\n**a**\\\n**c**\n",
        ),
        (
            "<p>x</p><p><strong>a</strong><strong><br></strong><strong>c</strong></p>",
            "x\n\n**a**  \n**c**\n",
            "x\n\n**a**\\\n**c**\n",
        ),
        (
            "<p>x</p><p><b>a</b><b><span><br></span></b><b>c</b></p>",
            "x\n\n**a**  \n**c**\n",
            "x\n\n**a**\\\n**c**\n",
        ),
    ]);
}

#[test]
fn should_keep_each_break_of_the_page_form() {
    assert_rows(&[
        (
            "<p>x</p><p><b>a</b><b><br /></b><b>c</b><b><br /></b><b>d</b><br />e</p>",
            "x\n\n**a**  \n**c**  \n**d**  \ne\n",
            "x\n\n**a**\\\n**c**\\\n**d**\\\ne\n",
        ),
        (
            "<p>x</p><p><b>a&nbsp;</b><span><span><b>m</b></span></span><b>&nbsp;. n</b><b><br /></b><b>c</b></p>",
            "x\n\n**a** **m** **. n**  \n**c**\n",
            "x\n\n**a** **m** **. n**\\\n**c**\n",
        ),
    ]);
}

#[test]
fn should_keep_the_break_in_a_later_block_that_is_no_paragraph() {
    assert_rows(&[
        (
            "<p>x</p><div><b>a</b><b><br></b><b>c</b></div>",
            "x\n\n**a**  \n**c**\n",
            "x\n\n**a**\\\n**c**\n",
        ),
        (
            "<p>x</p><div><ul><li><b>a</b><b><br></b><b>c</b></li></ul></div>",
            "x\n\n- **a**  \n**c**\n",
            "x\n\n- **a**\\\n**c**\n",
        ),
    ]);
}

#[test]
fn should_keep_the_break_in_a_link_label() {
    assert_rows(&[(
        "<p><a href=\"u\"><b>a</b><b><br></b><b>c</b></a></p>",
        "[**a**  \n**c**](u)\n",
        "[**a**\\\n**c**](u)\n",
    )]);
}

#[test]
fn should_keep_the_separator_in_the_block_that_starts_the_output() {
    assert_rows(&[
        ("<p><b>a</b><b><br></b><b>c</b></p>", "**a** **c**\n", "**a** **c**\n"),
        ("<b>a</b><b><br></b><b>c</b>", "**a** **c**\n", "**a** **c**\n"),
        (
            "<div><p><b>a</b><b><br></b><b>c</b></p></div>",
            "**a** **c**\n",
            "**a** **c**\n",
        ),
    ]);
}

#[test]
fn should_keep_the_break_of_a_wrapper_with_two_breaks() {
    assert_rows(&[(
        "<p>x</p><p><b>a</b><b><br><br></b><b>c</b></p>",
        "x\n\n**a**  \n**c**\n",
        "x\n\n**a**\\\n**c**\n",
    )]);
}

#[test]
fn should_keep_the_separator_of_a_wrapper_with_white_space_after_the_break() {
    for style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
        assert_routes(
            "<p>x</p><p><b>a</b><b><br> </b><b>c</b></p>",
            "x\n\n**a** **c**\n",
            style,
        );
    }
}
