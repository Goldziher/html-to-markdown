// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Form controls (issues 752, 757 and 776) in code and in strict white space mode: the boundary of
//! a control is not white space of the source, so a control is a word of its own there too. Each
//! expected string is the text that a browser shows when the control is a word of its own, inside
//! the code marks. The last tests hold the rows for a checkbox next to a word, a table cell with
//! a checkbox and another element, and an image after a control.

use html_to_markdown_rs::options::PreprocessingOptions;
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::tier1;
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, WhitespaceMode, convert};

const SELECT: &str = "<select><option>One</option><option>Two</option></select>";
const OPTION_GROUP: &str =
    "<select><optgroup label=\"Group\"><option>One</option><option>Two</option></optgroup></select>";

/// Options that let the fast converter run, with forms kept.
fn options(tier_strategy: TierStrategy, whitespace_mode: WhitespaceMode) -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        tier_strategy,
        whitespace_mode,
        preprocessing: PreprocessingOptions {
            remove_forms: false,
            ..PreprocessingOptions::default()
        },
        ..ConversionOptions::default()
    }
}

fn markdown(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

/// Asserts `expected` for `html` in the given white space mode on the full converter, the fast
/// converter with its fallback and the router. The fast converter alone must write the same text
/// or hand the input over.
fn assert_in_mode(html: &str, expected: &str, whitespace_mode: WhitespaceMode) {
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Tier1, TierStrategy::Auto] {
        assert_eq!(
            markdown(html, options(tier_strategy, whitespace_mode)),
            expected,
            "{tier_strategy:?} {whitespace_mode:?}: {html}"
        );
    }
    let fast_options = options(TierStrategy::Tier1, whitespace_mode);
    if let Ok(fast) = tier1::run(html, &PrescanReport::default(), &fast_options) {
        assert_eq!(fast, expected, "fast converter alone {whitespace_mode:?}: {html}");
    }
}

/// Asserts `expected` for `html` in the default white space mode and in strict mode.
fn assert_in_both_modes(html: &str, expected: &str) {
    assert_in_mode(html, expected, WhitespaceMode::Normalized);
    assert_in_mode(html, expected, WhitespaceMode::Strict);
}

#[test]
fn should_add_no_space_for_a_control_without_text_in_code() {
    assert_in_both_modes(
        "<pre>before<input type=\"text\" value=\"typed\">after</pre>",
        "```\nbeforeafter\n```\n",
    );
    assert_in_both_modes(
        "<p>See <code>before<input>after</code> and <kbd>a<input type=\"radio\">b</kbd> and <samp>a<input>b</samp></p>",
        "See `beforeafter` and `ab` and `ab`\n",
    );
}

#[test]
fn should_write_the_text_of_a_button_in_code_as_the_source_has_it() {
    assert_in_both_modes(
        "<pre>before<span><button>Go</button></span>after</pre>",
        "```\nbeforeGoafter\n```\n",
    );
    assert_in_both_modes("<pre>before<button>Go</button>after</pre>", "```\nbeforeGoafter\n```\n");
    assert_in_both_modes(
        "<pre>before<label><button>Go</button></label>after</pre>",
        "```\nbeforeGoafter\n```\n",
    );
    assert_in_both_modes("<pre>a <output>42</output> b</pre>", "```\na 42 b\n```\n");
    assert_in_both_modes(
        "<p>See <code>before<button>Go</button>after</code> here</p>",
        "See `beforeGoafter` here\n",
    );
}

#[test]
fn should_separate_the_options_of_a_select_list_in_code() {
    assert_in_both_modes(
        &format!("<pre>before{SELECT}after</pre>"),
        "```\nbefore One Two after\n```\n",
    );
    assert_in_both_modes(
        &format!("<pre>before\n{SELECT}\nafter</pre>"),
        "```\nbefore\nOne Two\nafter\n```\n",
    );
    assert_in_both_modes(
        "<pre>if (x) {\n  a<select>\n<option>One</option>\n<option>Two</option>\n</select>b;\n}</pre>",
        "```\nif (x) {\n  a\nOne\nTwo\nb;\n}\n```\n",
    );
    assert_in_both_modes(
        &format!("<p>See <code>before{SELECT}after</code> here</p>"),
        "See `before One Two after` here\n",
    );
}

#[test]
fn should_write_nothing_for_a_checkbox_in_code() {
    assert_in_both_modes(
        "<pre>before<input type=\"checkbox\">after</pre>",
        "```\nbeforeafter\n```\n",
    );
    assert_in_both_modes(
        "<pre>before<input type=\"checkbox\" checked>after</pre>",
        "```\nbeforeafter\n```\n",
    );
    assert_in_both_modes(
        "<p>See <code>before<input type=\"checkbox\">after</code> here</p>",
        "See `beforeafter` here\n",
    );
    // ~keep The white space of the source on both sides of the checkbox stays in code.
    assert_in_both_modes("<pre>a <input type=\"checkbox\"> b</pre>", "```\na  b\n```\n");
}

#[test]
fn should_keep_the_space_between_a_word_and_a_checkbox_before_a_word() {
    assert_in_both_modes("<p>a <input type=\"checkbox\">b</p>", "a b\n");
}

#[test]
fn should_keep_the_space_after_a_hidden_input_at_the_start_of_a_label() {
    assert_in_both_modes("<p>a<label><input type=\"hidden\"> x</label></p>", "a x\n");
}

#[test]
fn should_write_no_checkbox_state_in_a_cell_that_holds_another_element() {
    assert_in_both_modes(
        "<table><tr><th>h</th></tr><tr><td><input type=\"checkbox\"><span>x</span></td></tr></table>",
        "| h |\n| --- |\n| x |\n",
    );
}

#[test]
fn should_separate_a_control_from_an_image_after_it() {
    assert_in_both_modes(
        "<p><span><button>One</button><img src=\"a.png\" alt=\"x\"></span></p>",
        "One ![x](a.png)\n",
    );
}

#[test]
fn should_not_write_the_label_of_an_option_group_in_code() {
    assert_in_both_modes(
        &format!("<pre>before{OPTION_GROUP}after</pre>"),
        "```\nbefore One Two after\n```\n",
    );
    assert_in_both_modes(
        &format!("<p>See <code>before{OPTION_GROUP}after</code> here</p>"),
        "See `before One Two after` here\n",
    );
}

#[test]
fn should_write_a_control_in_strict_white_space_mode_as_in_the_default_mode() {
    for (html, expected) in [
        (
            "<p>before<input type=\"text\" value=\"typed\">after</p>",
            "beforeafter\n",
        ),
        (&*format!("<p>before{SELECT}after</p>"), "before One Two after\n"),
        ("<p>before<input type=\"checkbox\">after</p>", "beforeafter\n"),
        ("<p>before <input type=\"checkbox\"> after</p>", "before after\n"),
        (&*format!("<p>before{OPTION_GROUP}after</p>"), "before One Two after\n"),
    ] {
        assert_in_mode(html, expected, WhitespaceMode::Strict);
    }
}
