// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #679: the fast converter dropped a line break that no element
//! encloses (`a<br>b` with no `<p>`, `<div>` or `<body>` around it), so the two lines joined.

use html_to_markdown_rs::options::{HighlightStyle, NewlineStyle};
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert, tier1};

fn options(tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        tier_strategy,
        ..ConversionOptions::default()
    }
}

fn tier2(html: &str) -> String {
    convert(html, Some(options(TierStrategy::Tier2)))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

/// The fast converter's own output: an `Err` is a bail, never a fallback to the full converter.
fn tier1(html: &str) -> String {
    tier1::run(html, &PrescanReport::default(), &options(TierStrategy::Tier1))
        .unwrap_or_else(|reason| panic!("the fast converter must not bail on {html:?}: {reason:?}"))
}

/// A `<br>` that no element encloses, with what comes before and after it varied.
const TOP_LEVEL_BREAKS: &[&str] = &[
    "a<br>b",
    "a<br>1) t",
    "a<br>- t",
    "a<br>2. t",
    "a<br/>b",
    "a<BR>b",
    "x<br>y<br>z",
    "a<br><br>b",
    "a<br><br><br>b",
    "a<br>",
    "a<br><br>",
    "<br>a",
    "<br><br>a",
    "<br>",
    "&amp;<br>b",
    "a&nbsp;<br>b",
    "a <br>b",
    "a<br> b",
    "a <br> b",
    "a<br>  1) t",
    "<b>a</b><br>b",
    "a<br><b>b</b>",
    "<a href=\"u\">a</a><br>b",
    "<code>a</code><br>b",
    "<img src=\"x.png\" alt=\"i\"><br>b",
    "<span>a</span><br>b",
    "a<!-- c --><br>b",
    "<!-- c --><br>a",
    "<p>a</p>b<br>c",
    "<p>a</p><br>b",
    "a<br><p>b</p>",
    "a<br><ul><li>x</li></ul>",
    "<ul><li>x</li></ul><br>a",
];

#[test]
fn should_keep_a_top_level_hard_break_in_the_fast_converter() {
    assert_eq!(tier1("a<br>1) t"), "a  \n1\\) t\n");
}

#[test]
fn should_write_every_top_level_break_the_same_in_both_converters() {
    for html in TOP_LEVEL_BREAKS {
        assert_eq!(tier1(html), tier2(html), "input: {html:?}");
    }
}

#[test]
fn should_write_a_top_level_break_as_it_is_written_inside_body() {
    for html in TOP_LEVEL_BREAKS {
        assert_eq!(tier1(html), tier1(&format!("<body>{html}</body>")), "input: {html:?}");
    }
}

#[test]
fn should_drop_the_leading_space_of_the_line_after_a_hard_break() {
    for html in [
        "<div>a<br> b</div>",
        "<p>a <br> b</p>",
        "<body>a<br>  1) t</body>",
        "<div>a<br>\t- t</div>",
        "<p><b>a</b><br> b</p>",
        // ~keep Inside an element with a buffer of its own that already holds text, the
        // ~keep break is in that buffer, so the space is dropped there too.
        "<dl><dd>a<br> b</dd></dl>",
        "<dl><dt>a<br> b</dt></dl>",
        "<div>a<br><sup>y<br> x</sup></div>",
    ] {
        assert_eq!(tier1(html), tier2(html), "input: {html:?}");
    }
    let no_highlight = ConversionOptions {
        highlight_style: HighlightStyle::None,
        ..options(TierStrategy::Tier2)
    };
    let html = "<p>a<br><mark>y<br> x</mark></p>";
    let scanned = tier1::run(html, &PrescanReport::default(), &no_highlight).expect("the fast converter must not bail");
    let converted = convert(html, Some(no_highlight))
        .expect("conversion must succeed")
        .content;
    assert_eq!(Some(scanned), converted);
}

#[test]
fn should_write_no_space_at_the_start_of_the_line_after_a_break() {
    // ~keep The full converter writes these elements' children into a buffer of their own. The
    // ~keep white space at their start is still white space at the start of a line: none.
    for (html, expected) in [
        ("<div>a<br><sup> x</sup></div>", "a  \nx\n"),
        ("<blockquote>a<br><sub> x</sub></blockquote>", "> a  \n> x\n"),
        ("<p>a<br><abbr> x</abbr></p>", "a  \nx\n"),
        ("<div>a<br><sup><span> x</span></sup></div>", "a  \nx\n"),
    ] {
        assert_eq!(tier2(html), expected, "input: {html:?}");
        assert_eq!(tier1(html), expected, "input: {html:?}");
    }
    // ~keep The white space between a definition term and the next term or description is a
    // ~keep line of one space in the full converter, as before this change; the fast converter
    // ~keep writes the start of the line with no space (step 3).
    for html in ["<dl><dt>a</dt> <dd> b</dd></dl>", "<dl><dt>a</dt> <dt> b</dt></dl>"] {
        assert_eq!(tier2(html), "a\n b\n", "input: {html:?}");
        assert_eq!(tier1(html), "a\nb\n", "input: {html:?}");
    }
    let no_highlight = ConversionOptions {
        highlight_style: HighlightStyle::None,
        ..options(TierStrategy::Tier2)
    };
    let html = "<div>a<br><mark> x</mark></div>";
    let scanned = tier1::run(html, &PrescanReport::default(), &no_highlight).expect("the fast converter must not bail");
    let converted = convert(html, Some(no_highlight))
        .expect("conversion must succeed")
        .content;
    assert_eq!(Some(scanned), converted);
}

#[test]
fn should_keep_a_top_level_hard_break_when_auto_picks_the_fast_converter() {
    // ~keep With metadata off and no highlight marker the router picks the fast converter,
    // ~keep and the scanner does not bail on this input, so Auto's output is the scanner's.
    let auto = ConversionOptions {
        highlight_style: HighlightStyle::None,
        ..options(TierStrategy::Auto)
    };
    let html = "a<br>b";
    let scanned = tier1::run(html, &PrescanReport::default(), &auto).expect("the fast converter must not bail");
    let converted = convert(html, Some(auto)).expect("conversion must succeed").content;
    assert_eq!(scanned, "a  \nb\n");
    assert_eq!(converted.as_deref(), Some("a  \nb\n"));
}

#[test]
fn should_drop_a_whitespace_character_reference_after_a_hard_break() {
    // ~keep A newline written as `&#10;` after a break made a blank line in the fast converter.
    for (html, expected) in [
        ("<div>a<br>&#10;b</div>", "a  \nb\n"),
        ("<p>a<br>&#xA;&#10;b</p>", "a  \nb\n"),
        ("<p>a<br>&#32;b</p>", "a  \nb\n"),
        ("<p>a<br>&#9;b</p>", "a  \nb\n"),
        ("<p>a<br>&#13;b</p>", "a  \nb\n"),
        ("<ul><li>a<br>&#10;b</li></ul>", "- a  \n  b\n"),
    ] {
        assert_eq!(tier1(html), expected, "input: {html:?}");
        assert_eq!(tier2(html), expected, "input: {html:?}");
    }
}

#[test]
fn should_write_one_space_for_a_hard_break_in_a_heading() {
    for html in [
        "<h2>a<br> b</h2>",
        "<h2>a <br>  b</h2>",
        "<h2>a<br>\n  b</h2>",
        "<h2>a<br>&nbsp;b</h2>",
    ] {
        assert_eq!(tier2(html), "## a b\n", "input: {html:?}");
        assert_eq!(tier1(html), "## a b\n", "input: {html:?}");
    }
    // ~keep A space the text before wrote is not a break: both converters keep both spaces.
    for html in ["<h2><span>a </span> b</h2>", "<h2><b>a </b> b</h2>"] {
        assert_eq!(tier2(html), tier1(html), "input: {html:?}");
    }
}

#[test]
fn should_indent_a_line_that_holds_only_a_hard_break_in_a_list_item() {
    // ~keep Issue #681: the second backslash line had no indent, and with wrap on the lines
    // ~keep after it left the item.
    let backslash = |wrap: bool| ConversionOptions {
        newline_style: NewlineStyle::Backslash,
        wrap,
        wrap_width: 20,
        ..options(TierStrategy::Tier2)
    };
    let convert_with = |html: &str, options: ConversionOptions| {
        convert(html, Some(options))
            .expect("conversion must succeed")
            .content
            .unwrap_or_default()
    };
    for (html, expected) in [
        ("<ul><li>a<br><br>b</li></ul>", "- a\\\n  \\\n  b\n"),
        ("<ol><li>a<br><br>b</li></ol>", "1. a\\\n   \\\n   b\n"),
        (
            "<ul><li>x<ul><li>a<br><br>b</li></ul></li></ul>",
            "- x\n  * a\\\n    \\\n    b\n",
        ),
        (
            "<blockquote><ul><li>a<br><br>b</li></ul></blockquote>",
            "> - a\\\n>   \\\n>   b\n",
        ),
    ] {
        assert_eq!(convert_with(html, backslash(false)), expected, "input: {html:?}");
        assert_eq!(convert_with(html, backslash(true)), expected, "wrapped input: {html:?}");
    }
    // ~keep A break-only line at the end of the item has no line to break, so no `\` is left.
    for (html, expected) in [
        ("<ul><li>a<br><br></li></ul>", "- a\n"),
        ("<ul><li>a<br><br><br></li><li>b</li></ul>", "- a\n\n- b\n"),
        ("<ol><li>a<br><br></li></ol>", "1. a\n"),
    ] {
        assert_eq!(convert_with(html, backslash(false)), expected, "input: {html:?}");
        let auto = ConversionOptions {
            newline_style: NewlineStyle::Backslash,
            ..options(TierStrategy::Auto)
        };
        assert_eq!(convert_with(html, auto), expected, "auto input: {html:?}");
    }
    // ~keep With the spaces style the line stays blank.
    for (html, expected) in [
        ("<ul><li>a<br><br>b</li></ul>", "- a  \n\n  b\n"),
        (
            "<blockquote><ul><li>a<br><br>b</li></ul></blockquote>",
            "> - a  \n>\n>   b\n",
        ),
    ] {
        assert_eq!(tier2(html), expected, "input: {html:?}");
    }
}

#[test]
fn should_keep_a_hard_break_before_an_escaped_line_when_wrapping() {
    // ~keep Issue #680: a `===` line left of the item's text column cannot underline the item's
    // ~keep paragraph, so it is paragraph text; the wrapper cut the paragraph there, read the
    // ~keep `1990.` line as an empty list item and dropped its hard break.
    let html = "<ul><li>q<br>word<br> &nbsp; *** - - *\n ===<br>2) ---<br>1990.<br> &gt;</li></ul>";
    let wrapped = ConversionOptions {
        wrap: true,
        wrap_width: 80,
        ..options(TierStrategy::Tier2)
    };
    let output = convert(html, Some(wrapped))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default();
    assert!(output.contains("1990.  \n  \\>"), "output: {output:?}");
}
