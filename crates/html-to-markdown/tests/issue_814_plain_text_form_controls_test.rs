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
fn the_markdown_output_of_the_same_inputs_separates_each_control_too() {
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

/// The plain text of `html` with forms kept and one more change of the options.
fn plain_with(html: &str, change: impl Fn(&mut ConversionOptions)) -> String {
    let mut changed = options(OutputFormat::Plain, TierStrategy::Tier2);
    change(&mut changed);
    converted(html, changed)
}

#[test]
fn a_control_outside_a_form_is_a_word_of_its_own_with_the_default_options() {
    let default_plain = || ConversionOptions {
        output_format: OutputFormat::Plain,
        ..ConversionOptions::default()
    };
    assert_eq!(
        converted("<p>aaa<button>Xone</button>bbb</p>", default_plain()),
        "aaa Xone bbb\n"
    );
    assert_eq!(
        converted(&format!("<p>aaa<select>{OPTIONS}</select>bbb</p>"), default_plain()),
        "aaa Xone Xtwo bbb\n"
    );
    // ~keep The default options remove a `form` element with its content.
    let in_form = converted(
        &format!("<p>ccc</p><form><p>aaa<select>{OPTIONS}</select>bbb</p></form>"),
        default_plain(),
    );
    assert!(in_form.contains("ccc") && !in_form.contains("Xone"), "{in_form:?}");
}

#[test]
fn an_image_with_no_alt_text_after_a_control_adds_no_second_space_in_plain_text() {
    assert_plain(
        "<p>aaa<button>Xone</button><img src=\"i.png\"> bbb</p>",
        "aaa Xone bbb\n",
    );
    assert_plain(
        "<p>aaa<button>Xone</button><img src=\"i.png\">bbb</p>",
        "aaa Xone bbb\n",
    );
    assert_plain(
        &format!("<p>aaa<select>{OPTIONS}</select><img src=\"i.png\"> bbb</p>"),
        "aaa Xone Xtwo bbb\n",
    );
}

#[test]
fn a_tag_that_is_never_text_after_a_control_adds_no_second_space_in_plain_text() {
    assert_plain(
        "<p>aaa<button>Xone</button><math><mi>y</mi></math> bbb</p>",
        "aaa Xone bbb\n",
    );
}

#[test]
fn a_skipped_image_after_a_control_adds_no_second_space_in_plain_text() {
    let html = "<p>aaa<button>Xone</button><img src=\"i.png\" alt=\"pic\"> bbb</p>";
    assert_eq!(plain_with(html, |options| options.skip_images = true), "aaa Xone bbb\n");
}

#[test]
fn an_excluded_element_after_a_control_adds_no_second_space_in_plain_text() {
    let html = "<p>aaa<button>Xone</button><span class=\"x\">zzz</span> bbb</p>";
    assert_eq!(
        plain_with(html, |options| options.exclude_selectors = vec![".x".to_string()]),
        "aaa Xone bbb\n"
    );
}

#[test]
fn the_alt_text_of_an_image_after_a_control_is_a_word_in_plain_text() {
    assert_plain(
        "<p>aaa<button>Xone</button><img src=\"i.png\" alt=\"pic\">.</p>",
        "aaa Xone pic.\n",
    );
    assert_plain(
        "<p>aaa<button>Xone</button><img src=\"i.png\" alt=\"pic\"> bbb</p>",
        "aaa Xone pic bbb\n",
    );
}

#[test]
fn the_text_of_an_inline_graphic_after_a_control_is_a_word_in_plain_text() {
    assert_plain(
        "<p>aaa<button>Xone</button><svg><title>pic</title></svg> bbb</p>",
        "aaa Xone pic bbb\n",
    );
}

/// The plain text of `html` with the default options.
fn default_plain(html: &str) -> String {
    converted(
        html,
        ConversionOptions {
            output_format: OutputFormat::Plain,
            ..ConversionOptions::default()
        },
    )
}

#[test]
fn a_word_after_the_form_of_a_control_is_separated_in_plain_text() {
    // ~keep This writer writes a `form` and a `fieldset` in the line of the text around them.
    let in_heading = format!("<h2>aaa<form><select>{OPTIONS}</select></form>bbb</h2>");
    assert_plain(&in_heading, "aaa Xone Xtwo bbb\n");
    // ~keep The default options remove the `form` element with its content.
    assert_eq!(default_plain(&in_heading), "aaabbb\n");
    assert_plain("<p>aaa<form><button>Xone</button></form>bbb</p>", "aaa Xone bbb\n");
    let in_fieldset = "<p>aaa<fieldset><button>Xone</button></fieldset>bbb</p>";
    assert_plain(in_fieldset, "aaa Xone bbb\n");
    assert_eq!(default_plain(in_fieldset), "aaa Xone bbb\n");
}

#[test]
fn an_element_with_no_text_after_a_control_does_not_join_the_next_word_in_plain_text() {
    for nothing in [
        "<img src=\"i.png\">",
        "<math><mi>y</mi></math>",
        "<span></span>",
        "<select> </select>",
        "<button></button>",
        "<!-- c -->",
    ] {
        for space in ["", " "] {
            let html = format!("<p>aaa<button>Xone</button>{nothing}{space}bbb</p>");
            assert_plain(&html, "aaa Xone bbb\n");
            assert_eq!(default_plain(&html), "aaa Xone bbb\n", "{html}");
        }
    }
    let excluded = "<p>aaa<button>Xone</button><span class=\"x\">zzz</span>bbb</p>";
    assert_eq!(
        plain_with(excluded, |options| options.exclude_selectors = vec![".x".to_string()]),
        "aaa Xone bbb\n"
    );
}

#[test]
fn text_after_a_control_is_separated_by_what_it_starts_with_in_plain_text() {
    // ~keep The decoded text decides: an entity for a letter is a word, an entity for a space is a space.
    assert_plain("<p>aaa<button>Xone</button>&amp;bbb</p>", "aaa Xone &bbb\n");
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Auto] {
        let no_break_space = converted(
            "<p>aaa<button>Xone</button>&nbsp;bbb</p>",
            options(OutputFormat::Plain, tier_strategy),
        );
        assert_eq!(no_break_space.replace('\u{a0}', " "), "aaa Xone bbb\n");
    }
    // ~keep No space before closing punctuation, as in the Markdown output.
    assert_plain("<p>aaa<button>Xone</button>.</p>", "aaa Xone.\n");
    assert_plain("<p>aaa<button>Xone</button>, bbb</p>", "aaa Xone, bbb\n");
    assert_plain("<p>aaa<button>Xone</button>(bbb)</p>", "aaa Xone (bbb)\n");
    assert_markdown(&format!("<p>aaa<select>{OPTIONS}</select>.</p>"), "aaa Xone Xtwo.\n");
    assert_plain("<p>aaa<button>Xone</button><b>bbb</b></p>", "aaa Xone bbb\n");
    assert_plain(
        &format!("<p>aaa<button>Xzero</button><select>{OPTIONS}</select>bbb</p>"),
        "aaa Xzero Xone Xtwo bbb\n",
    );
}

#[test]
fn only_the_text_right_after_a_control_is_separated_in_plain_text() {
    assert_plain("<p>aaa<button>Xone</button> ccc<b>ddd</b></p>", "aaa Xone cccddd\n");
    assert_plain("<p>aaa<button>Xone</button>ccc<b>ddd</b></p>", "aaa Xone cccddd\n");
}

#[test]
fn a_control_that_ends_with_white_space_gets_no_second_space_in_plain_text() {
    assert_plain("<p>aaa<button>Xone </button>bbb</p>", "aaa Xone bbb\n");
    // ~keep In code the line end of the source after the options is the end of the list.
    assert_plain(
        "<pre>aaa<select>\n<option>Xone</option>\n</select>bbb</pre>",
        "aaa\nXone\nbbb\n",
    );
}

#[test]
fn a_line_end_after_a_control_gets_no_space_in_plain_text() {
    assert_plain("<p>aaa<button>Xone</button><br>bbb</p>", "aaa Xone\nbbb\n");
    assert_plain("<p>aaa<button>Xone</button></p><p>bbb</p>", "aaa Xone\n\nbbb\n");
    assert_plain("<div>aaa<button>Xone</button><div>bbb</div></div>", "aaa Xone\n\nbbb\n");
    assert_plain("<p>aaa<button>Xone</button></p>", "aaa Xone\n");
}

#[test]
fn a_control_in_a_table_cell_stays_in_its_cell_in_plain_text() {
    assert_plain(
        "<table><tr><td>aaa<button>Xone</button><img src=\"i.png\">bbb</td><td><button>Xb</button></td><td>ccc</td></tr></table>",
        "aaa Xone bbb\tXb\tccc\n",
    );
}

#[test]
fn an_entity_for_a_closing_bracket_after_a_control_gets_no_space_in_plain_text() {
    assert_plain("<p>(<button>X</button>&#41;</p>", "(X)\n");
    assert_plain("<p>(<select><option>X</option></select>&#41;</p>", "(X)\n");
}

#[test]
fn an_entity_for_a_space_after_a_control_gets_no_second_space_in_plain_text() {
    assert_plain("<p>aaa<button>X</button>&#32;bbb</p>", "aaa X bbb\n");
    assert_plain("<p>aaa<select><option>X</option></select>&#32;bbb</p>", "aaa X bbb\n");
}

#[test]
fn a_control_that_ends_with_the_spaces_of_an_alt_text_separates_no_later_text_in_plain_text() {
    // ~keep The end of the first block removes the three spaces. The text after them is as
    // ~keep long as the spaces when the next word arrives.
    assert_plain(
        "<p>aaa<button><img alt=\"X   \"></button></p><p>b<i>cc</i></p>",
        "aaa X\n\nbcc\n",
    );
}

#[test]
fn a_control_with_a_block_end_after_spaces_is_separated_from_the_next_word_in_plain_text() {
    // ~keep The rule in the button ends the line and removes the three spaces before the
    // ~keep button. The button still ends at the end of its text.
    assert_plain("<div><img alt=\"b   \"><button><hr>e</button>ccc</div>", "b\n\ne ccc\n");
    assert_plain("<div><img alt=\"b\"><button><hr>e</button>ccc</div>", "b\n\ne ccc\n");
}

#[test]
fn a_control_with_a_block_end_after_spaces_and_a_character_of_two_bytes_is_separated_in_plain_text() {
    // ~keep The length of the text when the button started is inside the character now.
    assert_plain(
        "<div><img alt=\"b   \"><button><hr>\u{e9}</button>ccc</div>",
        "b\n\n\u{e9} ccc\n",
    );
}

#[test]
fn a_control_that_starts_with_a_block_after_spaces_ends_with_that_block_in_plain_text() {
    assert_plain(
        "<div><img alt=\"b   \"><button><div>e</div></button>ccc</div>",
        "b\n\ne\n\nccc\n",
    );
}

#[test]
fn a_control_with_a_block_end_and_no_text_after_spaces_adds_no_space_in_plain_text() {
    assert_plain("<div><img alt=\"b   \"><button><hr></button>ccc</div>", "b\n\nccc\n");
}

#[test]
fn a_block_in_an_option_of_a_list_item_is_a_word_of_its_own_in_plain_text() {
    // ~keep The text before each block is as long as the marker of its list item.
    assert_plain(
        "<ul><li><select><option>ab<div>x</div>y</option></select></li></ul>",
        "- ab x y\n",
    );
    assert_plain("<ul><li><select>ab<div>x</div></select></li></ul>", "- ab x\n");
    assert_plain(
        "<table><tr><td><ol><li><select><option>abc<p>x</p></option></select></li></ol></td></tr></table>",
        "1. abc x\n",
    );
    // ~keep An option with no list around it writes to a buffer of its own too. Its text keeps
    // ~keep the lines of the block, as before the rule for form controls.
    assert_plain("<ul><li><option>ab<div>x</div>y</option></li></ul>", "- ab\n\nx\n\ny\n");
}

#[test]
fn twenty_thousand_controls_in_one_paragraph_are_each_a_word_of_their_own_in_plain_text() {
    // ~keep Each control gets a space written before its text, in the middle of the output.
    const CONTROLS: usize = 20_000;
    let html = format!("<p>{}</p>", "z<button>X</button>".repeat(CONTROLS));
    let plain = converted(&html, options(OutputFormat::Plain, TierStrategy::Tier2));
    assert_eq!(plain.len(), 4 * CONTROLS);
    assert_eq!(&plain[..30], "z X z X z X z X z X z X z X z ");
    assert_eq!(&plain[plain.len() - 30..], "X z X z X z X z X z X z X z X\n");
}

#[cfg(feature = "visitor")]
mod with_a_visitor {
    use std::sync::{Arc, Mutex};

    use html_to_markdown_rs::NodeContext;
    use html_to_markdown_rs::visitor::{HtmlVisitor, VisitResult};

    use super::plain_with;

    /// Answers `at_start` at the start and `at_end` at the end of every `span` element.
    #[derive(Debug)]
    struct SpanVisitor {
        at_start: VisitResult,
        at_end: VisitResult,
    }

    impl HtmlVisitor for SpanVisitor {
        fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
            if ctx.tag_name == "span" {
                self.at_start.clone()
            } else {
                VisitResult::Continue
            }
        }

        fn visit_element_end(&mut self, ctx: &NodeContext, _output: &str) -> VisitResult {
            if ctx.tag_name == "span" {
                self.at_end.clone()
            } else {
                VisitResult::Continue
            }
        }
    }

    fn plain_with_spans(html: &str, at_start: &VisitResult, at_end: &VisitResult) -> String {
        plain_with(html, |options| {
            options.visitor = Some(Arc::new(Mutex::new(SpanVisitor {
                at_start: at_start.clone(),
                at_end: at_end.clone(),
            })));
        })
    }

    #[test]
    fn an_element_that_a_visitor_skips_after_a_control_does_not_join_the_next_word_in_plain_text() {
        for (at_start, at_end) in [
            (VisitResult::Skip, VisitResult::Continue),
            (VisitResult::Continue, VisitResult::Skip),
            (VisitResult::Custom("QQ".to_string()), VisitResult::Skip),
        ] {
            for space in ["", " "] {
                let html = format!("<p>aaa<button>Xone</button><span>zzz</span>{space}bbb</p>");
                assert_eq!(
                    plain_with_spans(&html, &at_start, &at_end),
                    "aaa Xone bbb\n",
                    "{at_start:?} {at_end:?}: {html}"
                );
            }
        }
    }

    #[test]
    fn the_text_of_a_visitor_after_a_control_is_a_word_in_plain_text() {
        let html = "<p>aaa<button>Xone</button><span>zzz</span>bbb</p>";
        let custom = VisitResult::Custom("QQ".to_string());
        assert_eq!(
            plain_with_spans(html, &VisitResult::Continue, &custom),
            "aaa Xone QQbbb\n"
        );
        assert_eq!(
            plain_with_spans(html, &custom, &VisitResult::Continue),
            "aaa Xone QQbbb\n"
        );
    }

    #[test]
    fn a_text_that_a_visitor_replaces_at_an_element_start_after_a_control_is_a_word_in_plain_text() {
        // ~keep The visitor answers a text at the start of the element and another for that text.
        let html = "<p>aaa<button>Xone</button><span>zzz</span>bbb</p>";
        let first = VisitResult::Custom("QQ".to_string());
        let second = VisitResult::Custom("RR".to_string());
        assert_eq!(plain_with_spans(html, &first, &second), "aaa Xone RRbbb\n");
    }

    /// Answers `QQ` for the text node `bbb`.
    #[derive(Debug)]
    struct TextVisitor;

    impl HtmlVisitor for TextVisitor {
        fn visit_text(&mut self, _ctx: &NodeContext, text: &str) -> VisitResult {
            if text == "bbb" {
                VisitResult::Custom("QQ".to_string())
            } else {
                VisitResult::Continue
            }
        }
    }

    #[test]
    fn the_text_of_a_visitor_for_a_text_node_after_a_control_is_a_word_in_plain_text() {
        let html = "<p>aaa<button>Xone</button>bbb</p>";
        let with_visitor = plain_with(html, |options| {
            options.visitor = Some(Arc::new(Mutex::new(TextVisitor)));
        });
        assert_eq!(with_visitor, "aaa Xone QQ\n");
    }

    #[test]
    fn a_control_that_a_visitor_replaces_with_its_element_separates_no_text_in_plain_text() {
        // ~keep The text of the visitor is as long as the text it replaces up to the end of the control.
        let html = "<p>aaa<span><button>X</button>yy</span>bbb</p>";
        let custom = VisitResult::Custom("QQ".to_string());
        assert_eq!(plain_with_spans(html, &VisitResult::Continue, &custom), "aaaQQbbb\n");
    }

    #[test]
    fn a_removed_element_with_a_control_keeps_the_space_after_the_control_before_it_in_plain_text() {
        let html = "<p>aaa<button>X</button><span><button>Y</button></span>bbb</p>";
        for (at_start, at_end) in [
            (VisitResult::Continue, VisitResult::Skip),
            (VisitResult::Skip, VisitResult::Continue),
            (VisitResult::Custom("QQ".to_string()), VisitResult::Skip),
        ] {
            assert_eq!(
                plain_with_spans(html, &at_start, &at_end),
                "aaa X bbb\n",
                "{at_start:?} {at_end:?}"
            );
        }
    }

    #[test]
    fn the_text_of_a_visitor_for_an_element_with_a_control_is_a_word_after_the_control_before_it_in_plain_text() {
        let html = "<p>aaa<button>X</button><span><button>Y</button></span>bbb</p>";
        let custom = VisitResult::Custom("Q".to_string());
        assert_eq!(plain_with_spans(html, &VisitResult::Continue, &custom), "aaa X Qbbb\n");
    }

    #[test]
    fn a_removed_element_with_a_control_and_no_control_before_it_adds_no_space_in_plain_text() {
        let html = "<p>aaa<span><button>Y</button></span>bbb</p>";
        assert_eq!(
            plain_with_spans(html, &VisitResult::Continue, &VisitResult::Skip),
            "aaabbb\n"
        );
    }

    #[test]
    fn a_control_after_a_removed_element_is_a_word_of_its_own_in_plain_text() {
        for (html, expected) in [
            ("<p>aaa<span>zzz</span><button>X</button>bbb</p>", "aaa X bbb\n"),
            (
                "<p>aaa<span><button>Y</button></span><button>X</button>bbb</p>",
                "aaa X bbb\n",
            ),
            (
                "<p>aaa<button>W</button><span><button>Y</button></span><button>X</button>bbb</p>",
                "aaa W X bbb\n",
            ),
        ] {
            assert_eq!(
                plain_with_spans(html, &VisitResult::Continue, &VisitResult::Skip),
                expected,
                "{html}"
            );
        }
    }

    #[test]
    fn a_removed_element_in_a_removed_element_keeps_the_space_after_the_control_before_them_in_plain_text() {
        let html = "<p>aaa<button>X</button><span><span><button>Y</button></span></span>bbb</p>";
        assert_eq!(
            plain_with_spans(html, &VisitResult::Continue, &VisitResult::Skip),
            "aaa X bbb\n"
        );
    }

    /// Answers `X` with three spaces after it for the text node `X`.
    #[derive(Debug)]
    struct PaddedTextVisitor;

    impl HtmlVisitor for PaddedTextVisitor {
        fn visit_text(&mut self, _ctx: &NodeContext, text: &str) -> VisitResult {
            if text == "X" {
                VisitResult::Custom("X   ".to_string())
            } else {
                VisitResult::Continue
            }
        }
    }

    fn plain_with_padded_text(html: &str) -> String {
        plain_with(html, |options| {
            options.visitor = Some(Arc::new(Mutex::new(PaddedTextVisitor)));
        })
    }

    #[test]
    fn a_control_whose_last_spaces_a_block_end_removes_separates_no_later_text_in_plain_text() {
        // ~keep The text after the removed spaces is as long as the spaces: the output has the
        // ~keep length it had at the end of the control when the next word arrives.
        let html = "<p><button>X</button></p><div>Z<b>w</b></div>";
        assert_eq!(plain_with_padded_text(html), "X\n\nZw\n");
    }

    #[test]
    fn a_character_where_a_block_end_removed_spaces_stays_whole_at_an_element_end_in_plain_text() {
        // ~keep The inner block starts after the three spaces. It removes them, and a character
        // ~keep of two bytes then stands across the place where the block started.
        let html = "<div>X<div>\u{e9}</div></div>";
        assert_eq!(plain_with_padded_text(html), "X\n\n\u{e9}\n");
    }

    /// Answers `at_end` at the end of every element named `tag`. With `pad`, answers `X` with
    /// three spaces after it for the text node `X`.
    #[derive(Debug)]
    struct TagVisitor {
        tag: &'static str,
        at_end: VisitResult,
        pad: bool,
    }

    impl HtmlVisitor for TagVisitor {
        fn visit_element_end(&mut self, ctx: &NodeContext, _output: &str) -> VisitResult {
            if ctx.tag_name == self.tag {
                self.at_end.clone()
            } else {
                VisitResult::Continue
            }
        }

        fn visit_text(&mut self, _ctx: &NodeContext, text: &str) -> VisitResult {
            if self.pad && text == "X" {
                VisitResult::Custom("X   ".to_string())
            } else {
                VisitResult::Continue
            }
        }
    }

    fn plain_with_tag(html: &str, tag: &'static str, at_end: &VisitResult, pad: bool) -> String {
        plain_with(html, |options| {
            options.visitor = Some(Arc::new(Mutex::new(TagVisitor {
                tag,
                at_end: at_end.clone(),
                pad,
            })));
        })
    }

    #[test]
    fn a_removed_block_after_spaces_leaves_no_text_of_its_own_in_plain_text() {
        // ~keep The block starts after three spaces and removes them before it writes.
        let html = "<pre>aaa   <div>bcd</div></pre>";
        assert_eq!(plain_with_tag(html, "div", &VisitResult::Skip, false), "aaa\n");
        assert_eq!(plain_with(html, |_| {}), "aaa\n\nbcd\n");
    }

    #[test]
    fn the_text_of_a_visitor_for_a_block_after_spaces_stands_after_the_spaces_in_plain_text() {
        let html = "<pre>aaa   <div>bcd</div></pre>";
        let custom = VisitResult::Custom("Q".to_string());
        assert_eq!(plain_with_tag(html, "div", &custom, false), "aaa   Q\n");
    }

    #[test]
    fn a_removed_block_after_a_space_keeps_the_space_between_the_words_beside_it_in_plain_text() {
        let html = "<p>aaa <div>bcd</div>ccc</p>";
        assert_eq!(plain_with_tag(html, "div", &VisitResult::Skip, false), "aaa ccc\n");
    }

    #[test]
    fn a_block_that_a_visitor_keeps_after_a_space_is_written_as_with_no_visitor_in_plain_text() {
        let html = "<p>aaa <div>bcd</div>ccc</p>";
        let expected = "aaa\n\nbcd\n\nccc\n";
        assert_eq!(plain_with(html, |_| {}), expected);
        assert_eq!(plain_with_tag(html, "div", &VisitResult::Continue, false), expected);
    }

    #[test]
    fn a_removed_inline_element_that_holds_a_block_after_a_space_keeps_the_space_in_plain_text() {
        // ~keep The block in the `span` removes the space before the `span`. The removal of
        // ~keep the `span` writes it again.
        let html = "<div>aaa <span><div>bcd</div></span>ccc</div>";
        assert_eq!(plain_with_tag(html, "span", &VisitResult::Skip, false), "aaa ccc\n");
    }

    #[test]
    fn a_removed_element_with_an_option_list_keeps_the_space_after_the_control_before_it_in_plain_text() {
        let html =
            "<p>aaa<button>X</button><span><select><option>one</option><option>two</option></select></span>bbb</p>";
        assert_eq!(plain_with_tag(html, "span", &VisitResult::Skip, false), "aaa X bbb\n");
        let custom = VisitResult::Custom("Q".to_string());
        assert_eq!(plain_with_tag(html, "span", &custom, false), "aaa X Qbbb\n");
    }

    #[test]
    fn a_removed_block_with_a_control_after_a_control_and_a_space_keeps_one_space_in_plain_text() {
        let html = "<p>aaa<button>X</button> <div><button>Y</button></div>bbb</p>";
        assert_eq!(plain_with_tag(html, "div", &VisitResult::Skip, false), "aaa X bbb\n");
    }

    #[test]
    fn a_removed_block_after_a_control_that_ends_with_spaces_keeps_the_spaces_in_plain_text() {
        let html = "<p>aaa<button>X</button><section>b</section>ccc</p>";
        assert_eq!(
            plain_with_tag(html, "section", &VisitResult::Skip, true),
            "aaa X   ccc\n"
        );
    }
}
