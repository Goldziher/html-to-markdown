#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! White space between inline content (issues #751, #762, #778, #795, #803): white space in
//! the source is one space in the output, no white space is no space, and a block boundary inside
//! a link label or a heading separates words. Every input runs on both converters.

use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions, HighlightStyle, TierStrategy, WhitespaceMode, convert, tier1,
};

fn tier_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    }
}

fn convert_with(html: &str, options: Option<ConversionOptions>) -> String {
    convert(html, options)
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn tier2(html: &str) -> String {
    convert_with(
        html,
        Some(ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..tier_options()
        }),
    )
}

/// What the two converters and the default options write for `html`, or the reason they differ.
/// `fast_converter_bails` says whether the fast converter gives this input to the other one.
fn convert_everywhere(html: &str, fast_converter_bails: bool) -> Result<String, String> {
    let full = tier2(html);
    let default = convert_with(html, None);
    if default != full {
        return Err(format!("default options give {default:?}, the full converter {full:?}"));
    }
    match (
        tier1::run(html, &PrescanReport::default(), &tier_options()),
        fast_converter_bails,
    ) {
        (Ok(fast), false) if fast == full => Ok(full),
        (Ok(fast), false) => Err(format!("fast converter gives {fast:?}, the full converter {full:?}")),
        (Ok(fast), true) => Err(format!("fast converter no longer bails: it gives {fast:?}")),
        (Err(_), true) => Ok(full),
        (Err(reason), false) => Err(format!("fast converter bails: {reason}")),
    }
}

fn assert_cases(cases: &[(&str, &str)], fast_converter_bails: bool) {
    let wrong: Vec<String> = cases
        .iter()
        .filter_map(
            |(html, expected)| match convert_everywhere(html, fast_converter_bails) {
                Ok(actual) if actual == *expected => None,
                Ok(actual) => Some(format!("{html:?}\n  expected {expected:?}\n  actual   {actual:?}")),
                Err(reason) => Some(format!("{html:?}\n  {reason}")),
            },
        )
        .collect();
    assert!(wrong.is_empty(), "{} wrong:\n{}", wrong.len(), wrong.join("\n"));
}

/// Cases that both converters convert.
fn assert_on_both(cases: &[(&str, &str)]) {
    assert_cases(cases, false);
}

/// Cases that the fast converter gives to the full one.
fn assert_on_full(cases: &[(&str, &str)]) {
    assert_cases(cases, true);
}

#[test]
fn a_line_break_before_an_inline_element_is_a_space() {
    assert_on_both(&[
        (
            "<p>hard to read\n<span id=\"index-0\"></span>docstrings. Compare</p>",
            "hard to read docstrings. Compare\n",
        ),
        (
            "<p>hard to read\n<span class=\"target\" id=\"index-0\"></span><a href=\"/pep\"><strong>docstrings</strong></a>. Compare</p>",
            "hard to read [**docstrings**](/pep). Compare\n",
        ),
        ("<p>one\n<span>two</span> three</p>", "one two three\n"),
        ("<p>one\n<span>two</span>\nthree</p>", "one two three\n"),
        (
            "<p>one\n<span id=\"a\"></span><span id=\"b\"></span>two</p>",
            "one two\n",
        ),
        (
            "<p>one\n<span id=\"a\"></span>two\n<span id=\"b\"></span>three</p>",
            "one two three\n",
        ),
        (
            "<p>first</p><p>one\n<span id=\"a\"></span>two</p>",
            "first\n\none two\n",
        ),
        ("<p><em>one\n<span id=\"a\"></span>two</em></p>", "*one two*\n"),
        (
            "<p><a href=\"/x\">one\n<span id=\"a\"></span>two</a></p>",
            "[one two](/x)\n",
        ),
        (
            "<p>zero\n<span class=\"ocrx_word\">one</span>\n<span class=\"ocrx_word\">two</span></p>",
            "zero one two\n",
        ),
    ]);
}

/// The container does not change the rule: where a page shows a space, the output has a space,
/// not a line end.
#[test]
fn a_line_break_before_an_inline_element_is_a_space_in_each_container() {
    assert_on_both(&[
        ("<p>one\n<span id=\"a\"></span>two</p>", "one two\n"),
        ("<div>one\n<span id=\"a\"></span>two</div>", "one two\n"),
        ("<ul><li>one\n<span id=\"a\"></span>two</li></ul>", "- one two\n"),
        (
            "<table><tr><th>h</th></tr><tr><td>one\n<span id=\"a\"></span>two</td></tr></table>",
            "| h       |\n| ------- |\n| one two |\n",
        ),
        ("<blockquote>one\n<span id=\"a\"></span>two</blockquote>", "> one two\n"),
        ("<h2>one\n<span id=\"a\"></span>two</h2>", "## one two\n"),
        ("one\n<span id=\"a\"></span>two", "one two\n"),
        (
            "<dl><dt>t</dt><dd>one\n<span id=\"a\"></span>two</dd></dl>",
            "t\none two\n",
        ),
    ]);
}

/// Before an element that the converter does not know as inline (a custom element), a line end
/// is a space in running text: a paragraph, a heading, an inline element. In a `<div>` and a list
/// item it stays a line end, as it was before.
#[test]
fn a_line_break_before_a_custom_element_is_a_space_in_running_text_only() {
    assert_on_both(&[
        ("<p>one\n<x-y>two</x-y></p>", "one two\n"),
        ("<div><b>one\n<x-y>two</x-y></b></div>", "**one two**\n"),
        ("<h2>one\n<x-y>two</x-y></h2>", "## one two\n"),
        ("<div>one\n<x-y>two</x-y></div>", "one\ntwo\n"),
        ("<ul><li>one\n<x-y>two</x-y></li></ul>", "- one\n  two\n"),
    ]);
}

/// A span that holds only white space after a line end is the same run of white space.
#[test]
fn a_line_break_before_white_space_in_an_element_is_one_space_in_each_container() {
    assert_on_both(&[
        ("<p>one\n<span> </span>two</p>", "one two\n"),
        ("<div>one\n<span> </span>two</div>", "one two\n"),
        ("<ul><li>one\n<span> </span>two</li></ul>", "- one two\n"),
        (
            "<table><tr><th>h</th></tr><tr><td>one\n<span> </span>two</td></tr></table>",
            "| h       |\n| ------- |\n| one two |\n",
        ),
        ("<blockquote>one\n<span> </span>two</blockquote>", "> one two\n"),
        ("<h2>one\n<span> </span>two</h2>", "## one two\n"),
        ("one\n<span> </span>two", "one two\n"),
        ("<div>one\n<span> </span> two</div>", "one two\n"),
        ("<ul><li>one\n<span> </span>\ntwo</li></ul>", "- one two\n"),
        ("<ul><li>one\n<span>&nbsp;</span>two</li></ul>", "- one \u{a0}two\n"),
        ("one\n<span>&nbsp;</span>two", "one \u{a0}two\n"),
    ]);
}

/// The line end is the same line end when a character reference writes it.
#[test]
fn a_no_break_space_stays_after_a_line_end_written_as_a_character_reference() {
    assert_on_both(&[
        ("<p>one&#10;<span>&nbsp;</span>two</p>", "one \u{a0}two\n"),
        ("<p>one&#xA;<span>&nbsp;</span>two</p>", "one \u{a0}two\n"),
        ("<p>one&#x0a;<span>&nbsp;</span>two</p>", "one \u{a0}two\n"),
        ("<ul><li>one&#10;<span>&nbsp;</span>two</li></ul>", "- one \u{a0}two\n"),
    ]);
}

/// A comment between the line end and the element is no text.
#[test]
fn a_no_break_space_stays_after_a_line_end_that_a_comment_follows() {
    assert_on_both(&[
        ("<p>one\n<!--c--><span>&nbsp;</span>two</p>", "one \u{a0}two\n"),
        ("<p>one\n<!--c--><!--d--><span>&nbsp;</span>two</p>", "one \u{a0}two\n"),
        ("<p>one&#10;<!--c--><span>&nbsp;</span>two</p>", "one \u{a0}two\n"),
        (
            "<ul><li>one\n<!--c--><span>&nbsp;</span>two</li></ul>",
            "- one \u{a0}two\n",
        ),
    ]);
}

/// White space of the source before an element that holds a no-break space is one space, and
/// the no-break space stays, as a browser shows it: in every spelling of the white space, in
/// every container, on both converters.
///
/// ~keep A carriage return reaches a text only as a reference and is a line end. A comment is
/// ~keep no text, before, after or between the white space. An indent after a line end is the
/// ~keep same run of white space.
#[test]
fn white_space_before_a_no_break_space_is_one_space_in_every_spelling_and_container() {
    const SPELLINGS: [&str; 22] = [
        "\n",
        "\r",
        "\r\n",
        " ",
        "\t",
        "&#10;",
        "&#xA;",
        "&#x0a;",
        "&NewLine;",
        "&#13;",
        "&#13;&#10;",
        "&#32;",
        "&Tab;",
        "\n  ",
        "&#10;&#32;&#32;",
        "<!--c-->\n",
        "\n<!--c-->",
        "<!--c-->&#10;",
        "&#10;<!--c-->",
        "\n<!--c--> ",
        " <!--c-->",
        "<!--c-->&#32;<!--d-->",
    ];
    let containers: [(&str, &str, &str, &str); 4] = [
        ("<p>", "</p>", "", "\n"),
        ("<ul><li>", "</li></ul>", "- ", "\n"),
        ("<h2>", "</h2>", "## ", "\n"),
        (
            "<table><tr><th>h</th></tr><tr><td>",
            "</td></tr></table>",
            "| h        |\n| -------- |\n| ",
            " |\n",
        ),
    ];
    let cases: Vec<(String, String)> = containers
        .iter()
        .flat_map(|(open, close, before, after)| {
            SPELLINGS.iter().map(move |white_space| {
                (
                    format!("{open}one{white_space}<span>&nbsp;</span>two{close}"),
                    format!("{before}one \u{a0}two{after}"),
                )
            })
        })
        .collect();
    assert_eq!(cases.len(), 88);
    let cases: Vec<(&str, &str)> = cases
        .iter()
        .map(|(html, expected)| (html.as_str(), expected.as_str()))
        .collect();
    assert_on_both(&cases);
}

/// Inside `pre` no rule for running text applies: every spelling of a line end is one line end
/// and the no-break space stays. A carriage return reference is written as before this file's
/// rules (no browser output is recorded for it).
#[test]
fn a_line_end_before_a_no_break_space_in_pre_is_written_as_it_is() {
    let cases: Vec<(String, &str)> = ["\n", "&#10;", "&#xA;", "&#x0a;", "&NewLine;", "<!--c-->&#10;"]
        .iter()
        .map(|line_end| {
            (
                format!("<pre>one{line_end}<span>&nbsp;</span>two</pre>"),
                "```\none\n\u{a0}two\n```\n",
            )
        })
        .collect();
    let cases: Vec<(&str, &str)> = cases
        .iter()
        .map(|(html, expected)| (html.as_str(), *expected))
        .collect();
    assert_on_both(&cases);
    assert_written_as_before(&[(
        "<pre>one&#13;<span>&nbsp;</span>two</pre>",
        "```\none\r\u{a0}two\n```\n",
        Some("```\none\r\u{a0}two\n```\n"),
    )]);
}

/// A text of one reference to a line end is white space between two elements, as the line end.
#[test]
fn a_line_end_written_as_a_reference_between_two_elements_is_one_space() {
    assert_on_both(&[
        ("<p><code>a</code>&#10;<code>b</code></p>", "`a` `b`\n"),
        ("<p><code>a</code>\n<code>b</code></p>", "`a` `b`\n"),
        ("<p><b>a</b>&#13;<b>b</b></p>", "**a** **b**\n"),
        ("<p><b>a</b><!--c-->&#xA;<b>b</b></p>", "**a** **b**\n"),
    ]);
}

/// The element that follows the line end holds text: the same space in each container.
#[test]
fn a_line_break_before_a_filled_inline_element_is_a_space_in_each_container() {
    assert_on_both(&[
        ("<p>one\n<b>two</b></p>", "one **two**\n"),
        ("<div>one\n<b>two</b></div>", "one **two**\n"),
        ("<ul><li>one\n<b>two</b></li></ul>", "- one **two**\n"),
        (
            "<table><tr><th>h</th></tr><tr><td>one\n<b>two</b></td></tr></table>",
            "| h           |\n| ----------- |\n| one **two** |\n",
        ),
        ("<blockquote>one\n<b>two</b></blockquote>", "> one **two**\n"),
        ("<h2>one\n<b>two</b></h2>", "## one **two**\n"),
        ("one\n<b>two</b>", "one **two**\n"),
        ("<div>one\n<span>two</span> three</div>", "one two three\n"),
        (
            "<div>see\n<a href=\"/x\">two</a>\n<code>three</code></div>",
            "see [two](/x) `three`\n",
        ),
        ("<div>one\n<br>two</div>", "one  \ntwo\n"),
        // ~keep A comment before the element changes nothing.
        ("<div>one\n<!-- c --><b>two</b></div>", "one **two**\n"),
        ("<div>one\n<!-- c --> <!-- d --><b>two</b></div>", "one **two**\n"),
        ("<div>one\n<!-- c --><br>two</div>", "one  \ntwo\n"),
    ]);
}

/// An inline element whose content starts with a block starts a line of its own in a browser:
/// the line end before it stays a line end, and a heading in a list item stays a heading.
#[test]
fn a_line_break_before_an_inline_element_that_starts_with_a_block_stays_a_line_break() {
    assert_on_both(&[
        ("<div>one\n<b><p>y</p></b></div>", "one\n**y**\n"),
        ("one\n<b><p>y</p></b>", "one\n**y**\n"),
        ("<blockquote>one\n<b><p>y</p></b></blockquote>", "> one\n> **y**\n"),
        ("<div>one\n<b> <!-- c --> <p>y</p></b></div>", "one\n**y**\n"),
        ("<div>one\n<!-- c --><b><p>y</p></b></div>", "one\n**y**\n"),
        ("<div>one\n<b><i><p>y</p></i></b></div>", "one\n***y***\n"),
        (
            "<blockquote>one\n<span><h3>y</h3></span></blockquote>",
            "> one\n>\n> ### y\n",
        ),
        (
            "<details><summary>one\n<span><div>y</div></span></summary>body</details>",
            "**one**\n\n**y**\n\nbody\n",
        ),
        // ~keep A `<span>` writes no marks: the block in it breaks the line as it does alone.
        (
            "<blockquote>one\n<span><div>y</div></span></blockquote>",
            "> one\n>\n> y\n",
        ),
        (
            "<blockquote>one\n<span><ul><li>y</li></ul></span>two</blockquote>",
            "> one\n>\n> - y\n>\n> two\n",
        ),
        ("<div>one\n<span><b><p>y</p></b></span></div>", "one\n**y**\n"),
        // ~keep Text before the block, or no block: the element is inline content.
        ("<div>one\n<b>x<p>y</p></b></div>", "one **x**\n\n**y**\n"),
        ("<div>one\n<b><i>y</i></b></div>", "one ***y***\n"),
        ("<div>one\n<b>y</b></div>", "one **y**\n"),
        // ~keep A self-closing element has no content: the block after it is a block of its own.
        ("<blockquote>one\n<b/><div>y</div></blockquote>", "> one\n>\n> y\n"),
    ]);
    assert_on_full(&[
        ("<div>one\n<a href=\"/y\"><div>y</div></a></div>", "one\n[y](/y)\n"),
        (
            "<ul><li>one\n<a href=\"/y\"><div>y</div></a></li></ul>",
            "- one\n[y](/y)\n",
        ),
        (
            "<ul><li>one\n<a href=\"/y\"><h3>y</h3></a></li></ul>",
            "- one\n    ### [y](/y)\n",
        ),
        (
            "<ol><li>one\n<a href=\"/y\"><h2>Title</h2></a></li></ol>",
            "1. one\n    ## [Title](/y)\n",
        ),
        ("<ul><li>one\n<b><p>y</p></b></li></ul>", "- one\n**y**\n"),
    ]);
}

/// White space before an element and white space at its start are one run: one space, and it
/// goes outside the marks of the element.
#[test]
fn white_space_on_both_sides_of_an_element_start_is_one_space() {
    assert_on_both(&[
        ("<p>one <b> y</b>two</p>", "one **y**two\n"),
        ("<p>one <b> y</b> two</p>", "one **y** two\n"),
        ("<p>one <b> y </b>two</p>", "one **y** two\n"),
        ("<p>one <b> y </b> two</p>", "one **y** two\n"),
        ("<p>one <b>\ny</b>two</p>", "one **y**two\n"),
        ("<p>one\n<b> y</b>two</p>", "one **y**two\n"),
        ("<p>one\t<b>\ty</b>two</p>", "one **y**two\n"),
        ("<p><b>one </b><i> two</i></p>", "**one** *two*\n"),
        ("<p>one <em> y</em>two</p>", "one *y*two\n"),
        ("<p>one <del> y</del>two</p>", "one ~~y~~two\n"),
        ("<p>one <b> y</b></p>", "one **y**\n"),
        ("<ul><li><b> y</b>one</li></ul>", "- **y**one\n"),
        ("<ul><li>one <b> y</b>two</li></ul>", "- one **y**two\n"),
        ("<div>one <b> y</b>two</div>", "one **y**two\n"),
        ("<blockquote>one <b> y</b>two</blockquote>", "> one **y**two\n"),
        ("<h2>one <b> y</b>two</h2>", "## one **y**two\n"),
        ("one <b> y</b>two", "one **y**two\n"),
        (
            "<table><tr><th>h</th></tr><tr><td>one <b> y</b>two</td></tr></table>",
            "| h            |\n| ------------ |\n| one **y**two |\n",
        ),
    ]);
}

/// The white space at the start of an element is the only white space there: it is kept.
#[test]
fn white_space_at_an_element_start_alone_is_one_space() {
    assert_on_both(&[
        ("<p>one<b> y</b>two</p>", "one **y**two\n"),
        ("<p>one<b> y </b>two</p>", "one **y** two\n"),
        ("<p>one<em>\ny</em>two</p>", "one *y*two\n"),
        ("<p>one <b>y</b>two</p>", "one **y**two\n"),
        ("<p>one<b>y</b>two</p>", "one**y**two\n"),
    ]);
}

/// The other inline elements that write their own marks follow the same rule.
#[test]
fn white_space_on_both_sides_of_the_start_of_any_marked_element_is_one_space() {
    assert_on_both(&[
        ("<p>one <kbd> y</kbd>two</p>", "one `y`two\n"),
        ("<p>one <abbr> y</abbr>two</p>", "one ytwo\n"),
        ("<p>one <dfn> y</dfn>two</p>", "one *y*two\n"),
        ("<p>one <ins> y</ins>two</p>", "one ==y==two\n"),
        ("<p>one <b><i> y</i></b>two</p>", "one ***y***two\n"),
    ]);
}

/// A zero-width space is a place where a line can break, not a space. A browser drops the line
/// end beside it, so the two parts stay one word.
#[test]
fn a_line_end_before_a_zero_width_space_is_no_space() {
    assert_on_both(&[
        ("<p>one\n<span></span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p>one\n<span id=\"a\"></span>&#8203;two</p>", "one\u{200b}two\n"),
        ("<p>one\n<span>\u{200b}</span>two</p>", "one\u{200b}two\n"),
        ("<p>one\n<span>\u{200b}</span> two</p>", "one\u{200b} two\n"),
        ("<p>one\n<b>\u{200b}two</b></p>", "one**\u{200b}two**\n"),
        (
            "<p>one\n<span><i></i></span><!-- c -->\u{200b}two</p>",
            "one\u{200b}two\n",
        ),
        ("<p>one \n <span></span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p>one\n<wbr>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<ul><li>one\n<span></span>\u{200b}two</li></ul>", "- one\u{200b}two\n"),
        ("<h2>one\n<span></span>\u{200b}two</h2>", "## one\u{200b}two\n"),
        (
            "<p><a href=\"/x\">one\n<span></span>\u{200b}two</a></p>",
            "[one\u{200b}two](/x)\n",
        ),
        ("<div>one\n<span></span>\u{200b}two</div>", "one\u{200b}two\n"),
        (
            "<blockquote>one\n<span></span>\u{200b}two</blockquote>",
            "> one\u{200b}two\n",
        ),
        ("one\n<span></span>\u{200b}two", "one\u{200b}two\n"),
        // ~keep The line end is the last thing in an element and the zero-width space follows
        // ~keep that element.
        ("<p><span>one\n</span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p><b>one\n</b>\u{200b}two</p>", "**one**\u{200b}two\n"),
        ("<p><b><i>one\n</i></b>\u{200b}two</p>", "***one***\u{200b}two\n"),
        ("<p><b>one\n</b><i>\u{200b}two</i></p>", "**one***\u{200b}two*\n"),
        ("<ul><li><span>one\n</span>\u{200b}two</li></ul>", "- one\u{200b}two\n"),
        ("<p><b>one\n</b><span></span>\u{200b}two</p>", "**one**\u{200b}two\n"),
        (
            "<p><b>x</b> <a href=\"/x\">one\n</a>\u{200b}two</p>",
            "**x** [one](/x)\u{200b}two\n",
        ),
        (
            "<table><tr><td>one\n<span></span>\u{200b}two</td></tr></table>",
            "| one\u{200b}two |\n| ------- |\n",
        ),
        (
            "<table><tr><td>one\n<!-- c -->\u{200b}two</td></tr></table>",
            "| one\u{200b}two |\n| ------- |\n",
        ),
    ]);
}

/// A run of line ends is one line end for a browser, so two or more of them before a
/// zero-width space are no space and no new paragraph: Chrome shows `one`, the zero-width space
/// and `two` for each input.
#[test]
fn a_run_of_line_ends_before_a_zero_width_space_is_no_space() {
    assert_on_both(&[
        ("<p>one\n\n<span></span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p>one\n \n<span></span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p>one\r\n\r\n<span></span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p>one\n\n\n<span></span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p>one\n\n<!-- c -->\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p><span>one\n\n</span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<div>one\n\n<span></span>\u{200b}two</div>", "one\u{200b}two\n"),
        ("one\n\n<span></span>\u{200b}two", "one\u{200b}two\n"),
        (
            "<ul><li>one\n\n<span></span>\u{200b}two</li></ul>",
            "- one\u{200b}two\n",
        ),
        (
            "<blockquote>one\n\n<span></span>\u{200b}two</blockquote>",
            "> one\u{200b}two\n",
        ),
        ("<h2>one\n\n<span></span>\u{200b}two</h2>", "## one\u{200b}two\n"),
        (
            "<table><tr><td>one\n\n<span></span>\u{200b}two</td></tr></table>",
            "| one\u{200b}two |\n| ------- |\n",
        ),
    ]);
}

/// The rule has two sides: a line end after a zero-width space is no space either. Chrome shows
/// `one`, the zero-width space and `y` for each input outside `pre`.
#[test]
fn a_line_end_after_a_zero_width_space_is_no_space() {
    assert_on_both(&[
        ("<div>one\u{200b}\n<b>y</b></div>", "one\u{200b}**y**\n"),
        ("<div>one&#8203;\n<b>y</b></div>", "one\u{200b}**y**\n"),
        ("<div>one&#x200B;\n<b>y</b></div>", "one\u{200b}**y**\n"),
        ("<div>one\u{200b} \n <b>y</b></div>", "one\u{200b}**y**\n"),
        ("<ul><li>one\u{200b}\n<b>y</b></li></ul>", "- one\u{200b}**y**\n"),
        // ~keep A space that is no line end stays, and `pre` keeps its line end.
        ("<div>one\u{200b} <b>y</b></div>", "one\u{200b} **y**\n"),
        ("<pre>one\u{200b}\n<span>y</span></pre>", "```\none\u{200b}\ny\n```\n"),
    ]);
}

/// A block in code writes its blank line on the fast converter, after a line end of the source
/// too: the rule for the space at an element start does not take the separator of the block.
///
/// ~keep This row pins the output of the base on the fast converter, where the two converters
/// ~keep differ: the full converter writes `y` with no code marks. Chrome shows `one`, a blank
/// ~keep line and `y`.
#[test]
fn a_block_in_code_after_a_line_end_keeps_its_blank_line_on_the_fast_converter() {
    for html in [
        "<p>one\n<kbd><div>y</div></kbd></p>",
        "<p>one\n  <kbd><div>y</div></kbd></p>",
    ] {
        let written = tier1::run(html, &PrescanReport::default(), &tier_options());
        assert_eq!(written.as_deref().ok(), Some("one\n\n`y`\n"), "{html}");
    }
}

/// A text of a form feed and a line end is white space for the fast converter, so the comment
/// scan does not run for it and the line end stays a line end.
///
/// ~keep This row pins the output of the base on the fast converter, where the two converters
/// ~keep differ: the full converter writes `one*y*`, the form feed and `**two**`. Chrome shows
/// ~keep `oney`, the form feed, a space and `two`; Markdown shows the line end as that space.
#[test]
fn a_text_of_a_form_feed_and_a_line_end_keeps_its_line_end_before_a_comment() {
    let html = "<div>one<i>y</i>\u{c}\n<!-- c --><b>two</b></div>";
    let written = tier1::run(html, &PrescanReport::default(), &tier_options());
    assert_eq!(written.as_deref().ok(), Some("one*y*\u{c}\n**two**\n"));
}

/// A space that is not a line end stays beside a zero-width space, and so does a line end
/// that content of its own separates from that character.
#[test]
fn a_space_beside_a_zero_width_space_is_kept() {
    assert_on_both(&[
        ("<p>one <span></span>\u{200b}two</p>", "one \u{200b}two\n"),
        ("<p>one\n<span></span>two</p>", "one two\n"),
        ("<p>one\n<wbr>two</p>", "one two\n"),
        (
            "<p>one\n<img src=\"/i.png\" alt=\"i\">\u{200b}two</p>",
            "one ![i](/i.png)\u{200b}two\n",
        ),
        ("<p>one\n<span>x</span>\u{200b}two</p>", "one x\u{200b}two\n"),
        ("<p><span>one </span>\u{200b}two</p>", "one \u{200b}two\n"),
        // ~keep The line end is the last thing in an element with a style, or in a block.
        (
            "<p><span style=\"display:inline-block\">one\n</span>\u{200b}two</p>",
            "one \u{200b}two\n",
        ),
        ("<div><span>one\n</span></div>\u{200b}two", "one\n\n\u{200b}two\n"),
        // ~keep An element whose style sets `display` or `white-space` can be a box of its
        // ~keep own or keep its line ends, and a browser keeps the line end beside it.
        (
            "<p>one\n<span style=\"display:inline-block\"></span>\u{200b}two</p>",
            "one \u{200b}two\n",
        ),
        (
            "<p>one\n<span style=\"display:inline-block\">\u{200b}two</span></p>",
            "one \u{200b}two\n",
        ),
        (
            "<table><tr><td>one \n <span style=\"display:inline-block\"></span>\u{200b}two</td></tr></table>",
            "| one \u{200b}two |\n| -------- |\n",
        ),
        (
            "<p>one\n<span style=\"white-space:pre\">\u{200b}two</span></p>",
            "one \u{200b}two\n",
        ),
        (
            "<pre>one\n<span></span>\u{200b}two</pre>",
            "```\none\n\u{200b}two\n```\n",
        ),
    ]);
}

/// A `style` attribute that sets neither `display` nor `white-space` changes nothing: the
/// element is no box of its own, and the line end before the zero-width space is no space.
#[test]
fn a_style_that_sets_no_display_and_no_white_space_keeps_no_space_before_a_zero_width_space() {
    assert_on_both(&[
        (
            "<p>one\n<span style=\"color:red\"></span>\u{200b}two</p>",
            "one\u{200b}two\n",
        ),
        (
            "<p>one\n<span style=\"color:red\">\u{200b}two</span></p>",
            "one\u{200b}two\n",
        ),
        (
            "<p><span style=\"color:red\">one\n</span>\u{200b}two</p>",
            "one\u{200b}two\n",
        ),
        ("<p>one\n<span style=\"\"></span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p>one\n<span style=\"\">\u{200b}two</span></p>", "one\u{200b}two\n"),
        ("<p>one\n<span style></span>\u{200b}two</p>", "one\u{200b}two\n"),
        ("<p>one\n<span style>\u{200b}two</span></p>", "one\u{200b}two\n"),
        (
            "<table><tr><td>one\n<span style=\"color:red\"></span>\u{200b}two</td></tr></table>",
            "| one\u{200b}two |\n| ------- |\n",
        ),
        (
            "<pre>one\n<span style=\"color:red\"></span>\u{200b}two</pre>",
            "```\none\n\u{200b}two\n```\n",
        ),
    ]);
}

/// Inside an element whose `style` attribute sets `white-space`, the line end before a
/// zero-width space stays: the element can keep its line ends, and its content inherits that.
#[test]
fn a_line_end_before_a_zero_width_space_stays_inside_a_style_that_sets_white_space() {
    assert_on_both(&[
        (
            "<div style=\"white-space:pre\">one\n<!-- c -->\u{200b}two</div>",
            "one\n\u{200b}two\n",
        ),
        // ~keep Chrome shows a line end in the next two; Markdown shows the line end as this space.
        (
            "<p style=\"white-space: pre-wrap\">one\n<!-- c -->\u{200b}two</p>",
            "one \u{200b}two\n",
        ),
        (
            "<div style=\"white-space:pre\">one\n<b>\u{200b}two</b></div>",
            "one **\u{200b}two**\n",
        ),
    ]);
}

/// The name of the `style` attribute has no letter case: both converters read `STYLE` and
/// `Style` as `style`.
#[test]
fn the_style_attribute_name_is_read_in_any_letter_case() {
    assert_on_both(&[
        (
            "<p>one\n<span STYLE=\"white-space:pre\">&#8203;two</span></p>",
            "one \u{200b}two\n",
        ),
        (
            "<p>one\n<span Style=\"display:inline-block\"></span>&#8203;two</p>",
            "one \u{200b}two\n",
        ),
        (
            "<p><span STYLE=\"display:inline-block\">one\n</span>\u{200b}two</p>",
            "one \u{200b}two\n",
        ),
        (
            "<table><tr><td>one\n<span STYLE=\"display:inline-block\">&#8203;two</span></td></tr></table>",
            "| one \u{200b}two |\n| -------- |\n",
        ),
        (
            "<p>one\n<span STYLE=\"color:red\">&#8203;two</span></p>",
            "one\u{200b}two\n",
        ),
        (
            "<table><tr><td>one\n<span STYLE=\"color:red\">&#8203;two</span></td></tr></table>",
            "| one\u{200b}two |\n| ------- |\n",
        ),
    ]);
}

/// A `style` value is read after its character references are decoded, as a browser reads it:
/// `white-space&#58;pre` is `white-space:pre`.
#[test]
fn a_style_value_with_character_references_sets_display_or_white_space() {
    const KEPT: &str = "one \u{200b}two\n";
    assert_on_both(&[
        (
            "<p>one\n<span style=\"white-space&#58;pre\"></span>&#8203;two</p>",
            KEPT,
        ),
        (
            "<p>one\n<span style=\"dis&#112;lay:inline-block\"></span>&#8203;two</p>",
            KEPT,
        ),
        (
            "<p>one\n<span style=\"display&#x3a;inline-block\"></span>&#8203;two</p>",
            KEPT,
        ),
        (
            "<p>one\n<span style=\"display&#X3A;inline-block\"></span>&#8203;two</p>",
            KEPT,
        ),
        (
            "<p>one\n<span style=\"display&colon;inline-block\"></span>&#8203;two</p>",
            KEPT,
        ),
        (
            "<p>one\n<span STYLE=\"DIS&#80;LAY:INLINE-BLOCK\"></span>&#8203;two</p>",
            KEPT,
        ),
        (
            "<p>one\n<span style=\"DISPLAY:INLINE-BLOCK\"></span>&#8203;two</p>",
            KEPT,
        ),
        (
            "<p>one\n<span style=\"color:red&#59;display:inline-block\"></span>&#8203;two</p>",
            KEPT,
        ),
        (
            "<p>one\n<span style=\"white-space&#58;pre\">&#8203;two</span></p>",
            KEPT,
        ),
        (
            "<p><span style=\"display&#58;inline-block\">one\n</span>&#8203;two</p>",
            KEPT,
        ),
        (
            "<div style=\"white-space&#58;pre\">one\n<!-- c -->\u{200b}two</div>",
            "one\n\u{200b}two\n",
        ),
        (
            "<table><tr><td>one\n<span style=\"display&#58;inline-block\">&#8203;two</span></td></tr></table>",
            "| one \u{200b}two |\n| -------- |\n",
        ),
    ]);
}

/// A `style` value that sets neither property after it is decoded changes nothing: a longer
/// property name, and a reference that the source escapes (`&amp;#58;` is the text `&#58;`).
#[test]
fn a_style_value_that_only_looks_like_display_or_white_space_keeps_no_space() {
    const JOINED: &str = "one\u{200b}two\n";
    assert_on_both(&[
        ("<p>one\n<span style=\"display-x:1\"></span>&#8203;two</p>", JOINED),
        ("<p>one\n<span style=\"display-x&#58;1\"></span>&#8203;two</p>", JOINED),
        (
            "<p>one\n<span style=\"white-space&amp;#58;pre\"></span>&#8203;two</p>",
            JOINED,
        ),
        (
            "<p>one\n<span style=\"display&amp;colon;inline-block\"></span>&#8203;two</p>",
            JOINED,
        ),
        ("<p>one\n<span style=\"color&#58;red\"></span>&#8203;two</p>", JOINED),
        ("<p>one\n<span style=\"color&#58;red\">&#8203;two</span></p>", JOINED),
        (
            "<p>one\n<span title=\"display&#58;block\"></span>&#8203;two</p>",
            JOINED,
        ),
        (
            "<p>one\n<span style=\"color:red\"></span><!-- style=\"display&#58;block\" -->&#8203;two</p>",
            JOINED,
        ),
        (
            "<pre>one\n<span style=\"white-space&#58;pre\"></span>&#8203;two</pre>",
            "```\none\n\u{200b}two\n```\n",
        ),
    ]);
}

/// A character reference to a line feed, a space or a tab is white space as the literal
/// character is: the text is read after its references are decoded, so `one&#10;` ends with a
/// line end before the zero-width space as `one\n` does.
#[test]
fn a_line_end_as_a_character_reference_before_a_zero_width_space_is_no_space() {
    const JOINED: &str = "one\u{200b}two\n";
    assert_on_both(&[
        ("<p>one&#10;<span></span>&#8203;two</p>", JOINED),
        ("<p>one&#xA;<span></span>&#8203;two</p>", JOINED),
        ("<p>one&#Xa;<span></span>&#8203;two</p>", JOINED),
        ("<p>one&NewLine;<span></span>&#8203;two</p>", JOINED),
        ("<p>one&#13;&#10;<span></span>&#8203;two</p>", JOINED),
        ("<p>one&#10;&#10;<span></span>&#8203;two</p>", JOINED),
        ("<p>one\n&#32;<span></span>&#8203;two</p>", JOINED),
        ("<p>one&#32;\n<span></span>&#8203;two</p>", JOINED),
        ("<p>one&#9;&#10;<span></span>&#8203;two</p>", JOINED),
        ("<p>one&Tab;\n&#x20;<span></span>&#8203;two</p>", JOINED),
        ("<p>one&#10;<!-- c -->&#8203;two</p>", JOINED),
        ("<div>one&#10;<span></span>&#8203;two</div>", JOINED),
        (
            "<ul><li>one&#10;<span></span>&#8203;two</li></ul>",
            "- one\u{200b}two\n",
        ),
        (
            "<table><tr><td>one&#10;<span></span>&#8203;two</td></tr></table>",
            "| one\u{200b}two |\n| ------- |\n",
        ),
    ]);
}

/// An escaped ampersand writes the text of a reference, not the character: `&amp;#10;` shows
/// `&#10;`. No name that only looks like the reference to a line feed is a line end.
#[test]
fn the_text_of_a_line_feed_reference_before_a_zero_width_space_is_kept() {
    assert_on_both(&[
        ("<p>one&amp;#10;<span></span>&#8203;two</p>", "one&#10;\u{200b}two\n"),
        (
            "<p>one&amp;NewLine;<span></span>&#8203;two</p>",
            "one&NewLine;\u{200b}two\n",
        ),
        ("<p>one&amp;#10;\n<span></span>&#8203;two</p>", "one&#10;\u{200b}two\n"),
        (
            "<p>one&NewLines;<span></span>&#8203;two</p>",
            "one&NewLines;\u{200b}two\n",
        ),
        (
            "<p>one&NEWLINE;<span></span>&#8203;two</p>",
            "one&NEWLINE;\u{200b}two\n",
        ),
        ("<p>one<!-- &#10; --><span></span>&#8203;two</p>", "one\u{200b}two\n"),
        ("<p>one&#32;<span></span>&#8203;two</p>", "one \u{200b}two\n"),
        ("<p>one&nbsp;\n<span></span>&#8203;two</p>", "one \u{200b}two\n"),
        (
            "<pre>one&#10;<span></span>&#8203;two</pre>",
            "```\none\n\u{200b}two\n```\n",
        ),
        (
            "<p><code>one&#10;<span></span>&#8203;two</code></p>",
            "`one \u{200b}two`\n",
        ),
    ]);
}

include!("support/inline_white_space_links.rs");
include!("support/inline_white_space_blocks.rs");
