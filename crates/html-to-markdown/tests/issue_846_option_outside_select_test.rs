// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! An option with no select list around it (issue 846): the outermost `select`, `datalist` or
//! `option` element writes an option list, so an option is a word of its own in every parent.
//! The Markdown output and the plain text output share the rule. Only a select list and a data
//! list are one line of plain text: an option outside them keeps its blocks, and an option
//! group outside them keeps its line ends.

use html_to_markdown_rs::options::PreprocessingOptions;
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, OutputFormat, TierStrategy, convert};

/// Each parent of an option: its name, the text before the option and the text after it.
const PARENTS: [(&str, &str, &str); 10] = [
    ("no parent", "", ""),
    ("p", "<p>", "</p>"),
    ("div", "<div>", "</div>"),
    ("span", "<p><span>", "</span></p>"),
    ("b", "<p><b>", "</b></p>"),
    ("li", "<ul><li>", "</li></ul>"),
    ("h2", "<h2>", "</h2>"),
    ("td", "<table><tr><th>h</th></tr><tr><td>", "</td></tr></table>"),
    ("pre", "<pre>", "</pre>"),
    ("code", "<p><code>", "</code></p>"),
];

/// A select list, and the option lists that must write the same text.
const LISTS: [(&str, [&str; 6]); 2] = [
    (
        "<select><option>Xone</option></select>",
        [
            "<option>Xone</option>",
            "<option>Xone",
            "<optgroup label=\"Xgrp\"><option>Xone</option></optgroup>",
            "<optgroup label=\"Xgrp\"><option>Xone</optgroup>",
            "<datalist><option>Xone</option></datalist>",
            "<datalist><option>Xone</datalist>",
        ],
    ),
    (
        "<select><option>Xone</option><option>Xtwo</option></select>",
        [
            "<option>Xone</option><option>Xtwo</option>",
            "<option>Xone<option>Xtwo</option>",
            "<optgroup label=\"Xgrp\"><option>Xone</option><option>Xtwo</option></optgroup>",
            "<optgroup label=\"Xgrp\"><option>Xone<option>Xtwo</optgroup>",
            "<datalist><option>Xone</option><option>Xtwo</option></datalist>",
            "<datalist><option>Xone<option>Xtwo</datalist>",
        ],
    ),
];

/// Options that keep forms, for the given output format and converter.
fn options(output_format: OutputFormat, tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        output_format,
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        tier_strategy,
        preprocessing: PreprocessingOptions {
            remove_forms: false,
            ..PreprocessingOptions::default()
        },
        ..ConversionOptions::default()
    }
}

fn converted(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

/// The text of `html` in `output_format`. The full converter, the fast converter with its
/// fallback, the default route and the default options must all write it.
fn text(html: &str, output_format: OutputFormat) -> String {
    let full = converted(html, options(output_format, TierStrategy::Tier2));
    for tier_strategy in [TierStrategy::Tier1, TierStrategy::Auto] {
        assert_eq!(
            converted(html, options(output_format, tier_strategy)),
            full,
            "{output_format:?} {tier_strategy:?}: {html}"
        );
    }
    let defaults = ConversionOptions {
        output_format,
        ..ConversionOptions::default()
    };
    assert_eq!(
        converted(html, defaults),
        full,
        "{output_format:?} default options: {html}"
    );
    full
}

fn assert_markdown(html: &str, expected: &str) {
    assert_eq!(text(html, OutputFormat::Markdown), expected, "markdown: {html}");
}

fn assert_plain(html: &str, expected: &str) {
    assert_eq!(text(html, OutputFormat::Plain), expected, "plain: {html}");
}

/// Asserts that each option list writes the text of its select list in every parent, with a
/// word on both sides and with white space on both sides.
fn assert_every_list_writes_the_text_of_the_select_list(output_format: OutputFormat) {
    for (parent, open, close) in PARENTS {
        for (before, after) in [("aaa", "bbb"), ("aaa ", " bbb")] {
            for (select, lists) in LISTS {
                for list in lists {
                    // ~keep An option with no end tag holds the text after it, so its parent
                    // ~keep ends it here and no text follows.
                    let after = if list.ends_with("Xone") { "" } else { after };
                    let html = format!("{open}{before}{list}{after}{close}");
                    let wanted = text(&format!("{open}{before}{select}{after}{close}"), output_format);
                    assert_eq!(
                        text(&html, output_format),
                        wanted,
                        "{output_format:?} in {parent}: {html}"
                    );
                }
            }
        }
    }
}

#[test]
fn an_option_outside_a_select_list_is_a_word_of_its_own_in_markdown() {
    assert_markdown("<p>aaa<option>Xone</option>bbb</p>", "aaa Xone bbb\n");
    assert_markdown("<p>aaa <option>Xone</option> bbb</p>", "aaa Xone bbb\n");
    assert_markdown("<div>aaa<option>Xone</option>bbb</div>", "aaa Xone bbb\n");
    assert_markdown(
        "<table><tr><th>h</th></tr><tr><td>aaa<option>Xone</option>bbb</td></tr></table>",
        "| h            |\n| ------------ |\n| aaa Xone bbb |\n",
    );
    assert_markdown("<pre>aaa<option>Xone</option>bbb</pre>", "```\naaa Xone bbb\n```\n");
}

#[test]
fn an_option_outside_a_select_list_is_a_word_of_its_own_in_plain_text() {
    assert_plain("<p>aaa<option>Xone</option>bbb</p>", "aaa Xone bbb\n");
    assert_plain("<p>aaa <option>Xone</option> bbb</p>", "aaa Xone bbb\n");
    assert_plain("<div>aaa<option>Xone</option>bbb</div>", "aaa Xone bbb\n");
    assert_plain(
        "<table><tr><th>h</th></tr><tr><td>aaa<option>Xone</option>bbb</td></tr></table>",
        "h\naaa Xone bbb\n",
    );
    assert_plain("<pre>aaa<option>Xone</option>bbb</pre>", "aaa Xone bbb\n");
}

#[test]
fn the_options_of_an_option_group_outside_a_select_list_are_separate_words() {
    let html = "<p>aaa<optgroup label=\"Xgrp\"><option>Xone</option><option>Xtwo</option></optgroup>bbb</p>";
    assert_markdown(html, "aaa Xone Xtwo bbb\n");
    assert_plain(html, "aaa Xone Xtwo bbb\n");
}

#[test]
fn an_option_group_outside_a_select_list_keeps_its_line_ends_in_plain_text() {
    // ~keep No word joins an option here, so the group writes its children as they are.
    let html = "<p>k<optgroup>\n<option>one</option>\n<option>two</option>\n</optgroup>d</p>";
    assert_plain(html, "k\none\ntwo\nd\n");
    // ~keep In a select list the same group is a part of the one line.
    assert_plain(
        "<p>k<select><optgroup>\n<option>one</option>\n<option>two</option>\n</optgroup></select>d</p>",
        "k one two d\n",
    );
}

#[test]
fn an_empty_option_outside_a_select_list_keeps_the_line_end_of_the_source_beside_it() {
    // ~keep The line end before an option outside a select list is one space in Markdown,
    // ~keep also after an opening bracket, where the option writes no space of its own.
    let html = "<ul><li>(<option></option>\n<option>o2</option>w3</li></ul>";
    assert_markdown(html, "- ( o2 w3\n");
    assert_plain(html, "- (\no2 w3\n");
    assert_markdown("<p>(<option></option>\n<option>o2</option>w3</p>", "( o2 w3\n");
    assert_markdown("<p>k<option>o2</option>\n<option></option>w3</p>", "k o2 w3\n");
    // ~keep The line end before an option group outside a select list is one space too.
    assert_markdown(
        "<p>(<option></option>\n<optgroup label=\"L\"><option>o2</option></optgroup>w3</p>",
        "( o2 w3\n",
    );
    // ~keep With no white space in the source nothing separates the bracket and the option.
    assert_markdown("<p>(<option>o2</option>w3</p>", "(o2 w3\n");
}

#[test]
fn a_line_end_of_the_source_before_an_option_in_a_select_list_writes_no_space() {
    // ~keep In a select list or a data list the line end before an option writes nothing.
    assert_markdown("<select>(<option></option>\n<option></option>w3</select>", "(w3\n");
}

#[test]
fn a_line_end_of_the_source_before_an_option_in_a_data_list_writes_no_space() {
    assert_markdown("<datalist>(<option></option>\n<option></option>w3</datalist>", "(w3\n");
}

#[test]
fn options_with_no_end_tag_outside_a_select_list_are_separate_words() {
    for (html, expected) in [
        ("<p>aaa<option>Xone<option>Xtwo</p><p>bbb</p>", "aaa Xone Xtwo\n\nbbb\n"),
        ("<div>aaa<option>Xone<option>Xtwo</div>bbb", "aaa Xone Xtwo\n\nbbb\n"),
        (
            "<p>aaa<option>Xone<option>Xtwo</option></option>bbb</p>",
            "aaa Xone Xtwo bbb\n",
        ),
    ] {
        assert_markdown(html, expected);
        assert_plain(html, expected);
    }
}

#[test]
fn an_option_outside_a_select_list_adds_no_space_beside_punctuation() {
    for (html, expected) in [
        ("<p>Pick (<option>Xone</option>) now</p>", "Pick (Xone) now\n"),
        ("<p>aaa<option>Xone</option>, bbb</p>", "aaa Xone, bbb\n"),
        ("<p>aaa<option></option>bbb</p>", "aaabbb\n"),
    ] {
        assert_markdown(html, expected);
        assert_plain(html, expected);
    }
}

#[test]
fn a_block_in_an_option_outside_a_select_list_keeps_its_lines() {
    // ~keep Outside a select list an option is no list control: a browser shows a block in it
    // ~keep as a block. Its lines stay, and its text does not join the word after it.
    let html = "<div>k<option>ab<div>x</div>y</option>d</div>";
    assert_markdown(html, "k ab\n\nx\n\ny d\n");
    assert_plain(html, "k ab\n\nx\n\ny d\n");
    assert_plain("<p>k<option>ab<br>cd</option>d</p>", "k ab\ncd d\n");
    // ~keep In a select list the same option is one line of plain text.
    assert_plain(
        "<div>k<select><option>ab<div>x</div>y</option></select>d</div>",
        "k ab x y d\n",
    );
    // ~keep A rule after text is a rule in Markdown too. In a select list it is inline text.
    assert_markdown("<div>k<option>ab<hr>y</option>d</div>", "k ab\n\n---\n\ny d\n");
    assert_markdown(
        "<div>k<optgroup><option>ab<hr>y</option><option>two</option></optgroup>d</div>",
        "k ab\n\n---\n\ny two d\n",
    );
    assert_markdown(
        "<div>k<select><option>ab<hr>y</option></select>d</div>",
        "k ab --- y d\n",
    );
}

#[test]
fn every_option_list_writes_the_text_of_the_select_list_in_markdown() {
    assert_every_list_writes_the_text_of_the_select_list(OutputFormat::Markdown);
}

#[test]
fn every_option_list_writes_the_text_of_the_select_list_in_plain_text() {
    assert_every_list_writes_the_text_of_the_select_list(OutputFormat::Plain);
}

#[test]
fn the_fast_converter_hands_an_option_outside_a_select_list_to_the_full_converter() {
    let result = tier1::run(
        "<p>aaa<option>Xone</option>bbb</p>",
        &PrescanReport::default(),
        &options(OutputFormat::Markdown, TierStrategy::Tier1),
    );
    assert!(matches!(result, Err(BailReason::FormControl)), "{result:?}");
}
