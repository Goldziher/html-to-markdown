#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::options::PreprocessingOptions;
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, OutputFormat, TierStrategy, convert};

fn failures(html: &str, expected: &str, output_format: OutputFormat) -> Vec<String> {
    let mut failures = Vec::new();
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let options = ConversionOptions {
            output_format,
            extract_metadata: false,
            highlight_style: HighlightStyle::None,
            tier_strategy,
            preprocessing: PreprocessingOptions {
                remove_forms: false,
                ..PreprocessingOptions::default()
            },
            ..ConversionOptions::default()
        };
        let actual = convert(html, Some(options)).expect("conversion succeeds").content;
        if actual.as_deref() != Some(expected) {
            failures.push(format!(
                "{output_format:?} {tier_strategy:?}: {html:?}: {actual:?} != {expected:?}"
            ));
        }
    }
    let options = ConversionOptions {
        output_format,
        ..ConversionOptions::default()
    };
    let actual = convert(html, Some(options))
        .expect("default conversion succeeds")
        .content;
    if actual.as_deref() != Some(expected) {
        failures.push(format!(
            "{output_format:?} default options: {html:?}: {actual:?} != {expected:?}"
        ));
    }
    failures
}

fn assert_output(html: &str, expected: &str, output_format: OutputFormat) {
    let failures = failures(html, expected, output_format);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn assert_rows(rows: &[(&str, &str)], output_format: OutputFormat) {
    let failures: Vec<_> = rows
        .iter()
        .flat_map(|(html, expected)| failures(html, expected, output_format))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn should_separate_a_word_after_a_bare_option_group_ending_in_a_line_feed() {
    assert_rows(
        &[
            (
                "<p>k<optgroup>\n<option>one</option>\n<option>two</option>\n</optgroup>d</p>",
                "k one two d\n",
            ),
            ("<p>k<optgroup>\n<option>one</option>\n</optgroup>d</p>", "k one d\n"),
            ("<p>k<optgroup><option>one</option>\n</optgroup>d</p>", "k one d\n"),
        ],
        OutputFormat::Markdown,
    );
}

#[test]
fn should_keep_a_line_feed_before_a_data_list_as_a_space_after_a_bracket() {
    assert_rows(
        &[
            (
                "<p>(<option></option>\n<datalist><option>o2</option></datalist>w3</p>",
                "( o2 w3\n",
            ),
            (
                "(<option></option>\n<datalist><option>o2</option></datalist>w3",
                "( o2 w3\n",
            ),
            (
                "(<option></option>\r\n<datalist><option>o2</option></datalist>w3",
                "( o2 w3\n",
            ),
        ],
        OutputFormat::Markdown,
    );
}

#[test]
fn should_preserve_a_no_break_space_in_blank_text_before_a_data_list() {
    assert_output(
        "k<b>x</b>\n\u{00a0}<datalist><option>o2</option></datalist>w3",
        "k**x**\u{00a0}o2 w3\n",
        OutputFormat::Markdown,
    );
}

#[test]
fn should_collapse_a_word_line_feed_before_a_data_list() {
    assert_output(
        "k\n<datalist><option>o2</option></datalist>w3",
        "k o2 w3\n",
        OutputFormat::Markdown,
    );
}

#[test]
fn should_keep_existing_form_and_span_whitespace_controls() {
    for (html, expected) in [
        ("<p>k<optgroup><option>one</option> </optgroup>d</p>", "k one d\n"),
        ("<p>k<span>\n<b>one</b>\n</span>d</p>", "k **one** d\n"),
        (
            "<p>k<select><optgroup>\n<option>one</option>\n<option>two</option>\n</optgroup></select>d</p>",
            "k one two d\n",
        ),
        (
            "<p>(<option></option>\n<select><option>o2</option></select>w3</p>",
            "( o2 w3\n",
        ),
        ("<p>(<option></option>\n<option>o2</option>w3</p>", "( o2 w3\n"),
        (
            "k<b>x</b>\n\u{00a0}<select><option>o2</option></select>w3",
            "k**x**\u{00a0}o2 w3\n",
        ),
        ("k\n<select><option>o2</option></select>w3", "k o2 w3\n"),
        (
            "<p>k<b>x</b> <datalist><option>o2</option></datalist>w3</p>",
            "k**x** o2 w3\n",
        ),
    ] {
        assert_output(html, expected, OutputFormat::Markdown);
    }
}

#[test]
fn should_keep_source_line_feeds_in_plain_output_for_bare_groups_and_data_lists() {
    assert_output(
        "<p>k<optgroup>\n<option>one</option>\n<option>two</option>\n</optgroup>d</p>",
        "k\none\ntwo\nd\n",
        OutputFormat::Plain,
    );
    assert_output(
        "<p>(<option></option>\n<datalist><option>o2</option></datalist>w3</p>",
        "(\no2 w3\n",
        OutputFormat::Plain,
    );
}
