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
fn should_keep_word_boundaries_when_a_data_list_is_excluded() {
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let result = convert(
                "k\n<datalist><option>o2</option></datalist>d",
                ConversionOptions {
                    tier_strategy,
                    output_format,
                    exclude_selectors: vec!["datalist".to_string()],
                    ..ConversionOptions::default()
                },
            )
            .expect("excluded data list converts");
            assert_eq!(
                result.content.as_deref(),
                Some("k\nd\n"),
                "{tier_strategy:?}/{output_format:?}"
            );
        }
    }
}

#[test]
fn should_keep_blocks_inside_a_data_list_in_the_same_line() {
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
        assert_output(
            "<p>k<datalist><div>blk</div></datalist>d</p>",
            "k blk d\n",
            output_format,
        );
    }
}

#[test]
fn should_keep_source_line_feeds_around_an_empty_data_list() {
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        assert_output("k&nbsp;\n<datalist></datalist>&nbsp;\nd", "k\nd\n", output_format);
    }
    assert_output(
        "k&nbsp;\n<datalist></datalist>&nbsp;\nd",
        "k\n\nd\n",
        OutputFormat::Plain,
    );
}

#[test]
fn should_keep_line_feeds_when_data_list_children_write_no_text() {
    for (body, exclude) in [
        ("<!--gap-->", ""),
        ("<span></span>", ""),
        ("<option><span></span></option>", ""),
        ("<template>x</template>", ""),
        ("<option>x</option>", "option"),
    ] {
        let html = format!("k&nbsp;\n<datalist>{body}</datalist>&nbsp;\nd");
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let result = convert(
                &html,
                ConversionOptions {
                    tier_strategy,
                    exclude_selectors: if exclude.is_empty() {
                        vec![]
                    } else {
                        vec![exclude.to_string()]
                    },
                    ..ConversionOptions::default()
                },
            )
            .expect("data list children convert");
            assert_eq!(result.content.as_deref(), Some("k\nd\n"), "{tier_strategy:?}/{body}");
        }
    }
}

#[test]
fn should_not_add_a_space_after_an_option_group_inside_an_option_list() {
    for tag in ["select", "datalist"] {
        let html = format!("<p>k<{tag}><optgroup><option>one</option>\n</optgroup>)</{tag}>d</p>");
        assert_output(&html, "k one) d\n", OutputFormat::Markdown);
    }
}

#[test]
fn should_keep_media_only_options_as_inline_controls() {
    assert_output(
        "k\n<datalist><option><img src='/a.png' alt='a'></option></datalist>d",
        "k ![a](/a.png) d\n",
        OutputFormat::Markdown,
    );
}

#[test]
fn should_consider_only_preserved_content_inside_the_adjacent_data_list() {
    for (html, preserve, expected) in [
        ("k&nbsp;\n<datalist></datalist>&nbsp;\nd", "p", "k\nd\n"),
        ("k\n<datalist><span></span></datalist>d", "span", "k <span></span> d\n"),
    ] {
        let result = convert(
            html,
            ConversionOptions {
                preserve_tags: vec![preserve.to_string()],
                ..ConversionOptions::default()
            },
        )
        .expect("preserved content converts");
        assert_eq!(result.content.as_deref(), Some(expected));
    }
}

#[test]
fn should_use_the_conversion_depth_limit_for_data_list_whitespace() {
    let result = convert(
        "k\n<datalist><span>x</span></datalist>d",
        ConversionOptions {
            max_depth: Some(2),
            ..ConversionOptions::default()
        },
    )
    .expect("depth-limited data list converts");
    assert_eq!(result.content.as_deref(), Some("k\nd\n"));
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.kind == html_to_markdown_rs::WarningKind::DepthLimitExceeded)
    );
}

#[cfg(feature = "visitor")]
#[test]
fn should_keep_a_visitor_custom_data_list_in_the_surrounding_line() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct DataListVisitor;
    impl HtmlVisitor for DataListVisitor {
        fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
            if ctx.tag_name == "datalist" {
                VisitResult::Custom("custom ".to_string())
            } else {
                VisitResult::Continue
            }
        }
    }

    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for excluded in [false, true] {
            let result = convert(
                "k\n<datalist></datalist>d",
                ConversionOptions {
                    tier_strategy,
                    visitor: Some(Arc::new(Mutex::new(DataListVisitor))),
                    exclude_selectors: if excluded { vec!["datalist".to_string()] } else { vec![] },
                    ..ConversionOptions::default()
                },
            )
            .expect("visitor custom data list converts");
            assert_eq!(
                result.content.as_deref(),
                Some(if excluded { "k\ncustom d\n" } else { "k custom d\n" })
            );
        }
    }
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

#[test]
fn should_keep_a_line_feed_before_an_input_only_data_list() {
    assert_output(
        "k\n<datalist><input value='v'></datalist>d",
        "k\nd\n",
        OutputFormat::Markdown,
    );
}

#[test]
fn should_keep_a_line_feed_before_blank_or_hidden_data_list_text() {
    for body in [
        " ",
        "&nbsp;",
        "<span hidden>x</span>",
        "<span style='display:none'>x</span>",
    ] {
        let html = format!("k\n<datalist>{body}</datalist>d");
        assert_output(&html, "k\nd\n", OutputFormat::Markdown);
    }
}

#[test]
fn should_join_data_list_block_children_to_the_preceding_line() {
    assert_output(
        "k\n<datalist><div>blk</div></datalist>d",
        "k blk d\n",
        OutputFormat::Markdown,
    );
}

#[test]
fn should_keep_the_boundary_before_preprocessing_skips_all_data_list_content() {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            "k\n<datalist><nav>x</nav></datalist>d",
            ConversionOptions {
                tier_strategy,
                preprocessing: PreprocessingOptions {
                    enabled: true,
                    remove_navigation: true,
                    remove_forms: false,
                    ..PreprocessingOptions::default()
                },
                ..ConversionOptions::default()
            },
        )
        .expect("removed navigation converts");
        assert_eq!(result.content.as_deref(), Some("k\nd\n"), "{tier_strategy:?}");
    }
}
