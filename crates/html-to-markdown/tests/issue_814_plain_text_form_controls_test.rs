// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Form controls in plain text output (issue 814): a control is a word of its own, and so is each
//! option of a select list. The rule is the rule of the Markdown output. The full converter
//! answers plain text output: the router hands every plain text conversion over to it.

use html_to_markdown_rs::options::PreprocessingOptions;
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::tier1::router::{RouterDecision, classify};
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, OutputFormat, TierStrategy, convert};

const OPTIONS: &str = "<option>Xone</option><option>Xtwo</option>";

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

/// Asserts the plain text of `html` on the full converter and on the default route.
fn assert_plain(html: &str, expected: &str) {
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Auto] {
        assert_eq!(
            converted(html, options(OutputFormat::Plain, tier_strategy)),
            expected,
            "plain {tier_strategy:?}: {html}"
        );
    }
}

/// Asserts the Markdown of `html` on the full converter, the fast converter with its fallback
/// and the default route.
fn assert_markdown(html: &str, expected: &str) {
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Tier1, TierStrategy::Auto] {
        assert_eq!(
            converted(html, options(OutputFormat::Markdown, tier_strategy)),
            expected,
            "markdown {tier_strategy:?}: {html}"
        );
    }
}

#[test]
fn the_router_hands_plain_text_output_to_the_full_converter() {
    let plain = options(OutputFormat::Plain, TierStrategy::Auto);
    assert_eq!(classify(&PrescanReport::default(), &plain), RouterDecision::Tier2);
}

#[test]
fn the_options_of_a_select_list_are_separate_words_in_plain_text() {
    assert_plain(
        &format!("<p>aaa<select>{OPTIONS}</select>bbb</p>"),
        "aaa Xone Xtwo bbb\n",
    );
    assert_plain(
        &format!("<p>aaa <select>{OPTIONS}</select> bbb</p>"),
        "aaa Xone Xtwo bbb\n",
    );
}

#[test]
fn the_options_of_an_option_group_are_separate_words_in_plain_text() {
    // ~keep The label of the group is an attribute. It is not text of the page.
    assert_plain(
        &format!("<p>aaa<select><optgroup label=\"Xgrp\">{OPTIONS}</optgroup></select>bbb</p>"),
        "aaa Xone Xtwo bbb\n",
    );
}

#[test]
fn a_button_is_a_word_of_its_own_in_plain_text() {
    assert_plain(
        "<h1>aaa<button>Xone</button><button>Xtwo</button>bbb</h1>",
        "aaa Xone Xtwo bbb\n",
    );
}

#[test]
fn a_select_list_adds_no_space_after_an_opening_bracket_in_plain_text() {
    assert_plain(
        &format!("<p>Pick (<select>{OPTIONS}</select>) now</p>"),
        "Pick (Xone Xtwo) now\n",
    );
}

#[test]
fn a_selected_option_and_an_empty_option_change_nothing_in_plain_text() {
    assert_plain(
        "<p>aaa<select><option>Xone</option><option selected>Xtwo</option></select>bbb</p>",
        "aaa Xone Xtwo bbb\n",
    );
    assert_plain(
        "<p>aaa<select><option></option><option>Xone</option><option value=\"\">  </option><option>Xtwo</option></select>bbb</p>",
        "aaa Xone Xtwo bbb\n",
    );
    assert_plain("<p>aaa<select><option></option></select>bbb</p>", "aaabbb\n");
}

#[test]
fn the_options_of_a_data_list_are_separate_words_in_plain_text() {
    assert_plain(
        &format!("<p>aaa<datalist>{OPTIONS}</datalist>bbb</p>"),
        "aaa Xone Xtwo bbb\n",
    );
}

#[test]
fn white_space_and_inline_markup_in_an_option_do_not_change_its_word_in_plain_text() {
    assert_plain(
        "<p>aaa<select>\n  <option>  Xone \n</option>\n  <option>\tXtwo </option>\n</select>bbb</p>",
        "aaa Xone Xtwo bbb\n",
    );
    // ~keep A line end of the source in a list must not end the row of a table.
    assert_plain(
        "<table><tr><td>aaa<select>\n<option>Xone</option>\n<option>Xtwo</option>\n</select>bbb</td><td>ccc</td></tr></table>",
        "aaa Xone Xtwo bbb\tccc\n",
    );
    assert_plain(
        "<p>aaa<select><option><b>X</b>one</option><option>X<i>two</i></option></select>bbb</p>",
        "aaa Xone Xtwo bbb\n",
    );
}

#[test]
fn a_select_list_stays_in_its_table_cell_in_plain_text() {
    assert_plain(
        &format!("<table><tr><td>aaa<select>{OPTIONS}</select>bbb</td><td>ccc</td></tr></table>"),
        "aaa Xone Xtwo bbb\tccc\n",
    );
}

#[test]
fn a_select_list_in_a_link_and_in_a_list_item_is_separate_words_in_plain_text() {
    assert_plain(
        &format!("<p>aaa<a href=\"/x\"><select>{OPTIONS}</select></a>bbb</p>"),
        "aaa Xone Xtwo bbb\n",
    );
    assert_plain(
        &format!("<ul><li><select>{OPTIONS}</select>bbb</li></ul>"),
        "- Xone Xtwo bbb\n",
    );
}

#[test]
fn a_pre_element_keeps_a_button_as_the_source_has_it_in_plain_text() {
    assert_plain(
        "<pre>aaa<button>Xone</button><button>Xtwo</button>bbb</pre>",
        "aaaXoneXtwobbb\n",
    );
    // ~keep The options of a select list stay separate words in code, as in the Markdown output.
    assert_plain(
        &format!("<pre>aaa<select>{OPTIONS}</select>bbb</pre>"),
        "aaa Xone Xtwo bbb\n",
    );
}

#[test]
fn the_markdown_output_of_the_same_inputs_does_not_change() {
    assert_markdown(
        &format!("<p>aaa<select>{OPTIONS}</select>bbb</p>"),
        "aaa Xone Xtwo bbb\n",
    );
    assert_markdown(
        &format!("<p>aaa<select><optgroup label=\"Xgrp\">{OPTIONS}</optgroup></select>bbb</p>"),
        "aaa Xone Xtwo bbb\n",
    );
    assert_markdown(
        "<h1>aaa<button>Xone</button><button>Xtwo</button>bbb</h1>",
        "# aaa Xone Xtwo bbb\n",
    );
}
