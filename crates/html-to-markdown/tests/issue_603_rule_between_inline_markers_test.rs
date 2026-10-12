// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #603: a rule written between inline markers (a summary, a caption,
//! `<b>`, `<em>` and the other marker wrappers) started after a blank line, which ended the
//! paragraph between the markers, so the markers rendered as literal text.

use html_to_markdown_rs::options::NewlineStyle;
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert, tier1};

fn tier2_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    }
}

fn convert_with(html: &str, options: &ConversionOptions) -> String {
    convert(html, Some(options.clone()))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn tier2(html: &str) -> String {
    convert_with(html, &tier2_options())
}

fn render(markdown: &str) -> String {
    comrak::markdown_to_html(markdown, &comrak::Options::default())
}

/// A definition list that starts its definition with a rule, inside each emphasis wrapper.
const DEFINITION_IN_MARKERS: [(&str, &str); 5] = [
    (
        "<figure><figcaption><dl><dt>t</dt><dd><hr></dd></dl></figcaption></figure>",
        "<p><em>t ---</em></p>\n",
    ),
    (
        "<details><summary><dl><dt>t</dt><dd><hr></dd></dl></summary></details>",
        "<p><strong>t ---</strong></p>\n",
    ),
    (
        "<div><b><dl><dt>t</dt><dd><hr></dd></dl></b></div>",
        "<p><strong>t ---</strong></p>\n",
    ),
    (
        "<div><em><dl><dt>t</dt><dd><hr></dd></dl></em></div>",
        "<p><em>t ---</em></p>\n",
    ),
    (
        "<details><summary><dl><dt>t</dt><dd><hr>c</dd></dl></summary></details>",
        "<p><strong>t --- c</strong></p>\n",
    ),
];

/// A bare rule between text inside each wrapper whose markers `CommonMark` renders.
const RULE_IN_MARKERS: [(&str, &str); 8] = [
    (
        "<details><summary>t<hr></summary></details>",
        "<p><strong>t ---</strong></p>\n",
    ),
    (
        "<details><summary>a<hr>c</summary></details>",
        "<p><strong>a --- c</strong></p>\n",
    ),
    (
        "<details><summary><hr>c</summary></details>",
        "<p><strong>--- c</strong></p>\n",
    ),
    (
        "<figure><figcaption>a<hr>c</figcaption></figure>",
        "<p><em>a --- c</em></p>\n",
    ),
    ("<div><b>a<hr>c</b></div>", "<p><strong>a --- c</strong></p>\n"),
    (
        "<div><strong>a<hr>c</strong></div>",
        "<p><strong>a --- c</strong></p>\n",
    ),
    ("<div><em>a<hr>c</em></div>", "<p><em>a --- c</em></p>\n"),
    ("<div><i>a<hr>c</i></div>", "<p><em>a --- c</em></p>\n"),
];

#[test]
fn should_keep_the_emphasis_of_a_definition_that_starts_with_a_rule() {
    let failures: Vec<String> = DEFINITION_IN_MARKERS
        .iter()
        .filter_map(|(html, expected)| {
            let out = tier2(html);
            let rendered = render(&out);
            (rendered != *expected).then(|| format!("{html:?}: {out:?} renders {rendered:?}"))
        })
        .collect();
    assert!(
        failures.is_empty(),
        "the markers must stay one run:\n{}",
        failures.join("\n")
    );
}

#[test]
fn should_write_a_rule_between_markers_as_text_in_the_line() {
    let failures: Vec<String> = RULE_IN_MARKERS
        .iter()
        .filter_map(|(html, expected)| {
            let out = tier2(html);
            let rendered = render(&out);
            (rendered != *expected).then(|| format!("{html:?}: {out:?} renders {rendered:?}"))
        })
        .collect();
    assert!(
        failures.is_empty(),
        "the markers must stay one run:\n{}",
        failures.join("\n")
    );
}

#[test]
fn should_write_the_issue_example_as_one_emphasis_run() {
    assert_eq!(
        tier2("<figure><figcaption><dl><dt>t</dt><dd><hr></dd></dl></figcaption></figure>").trim_end(),
        "*t ---*"
    );
    assert_eq!(
        tier2("<details><summary>t<hr></summary></details>").trim_end(),
        "**t ---**"
    );
}

#[test]
fn should_write_a_rule_inside_the_other_marker_wrappers_as_text_in_the_line() {
    let cases = [
        ("<div><del>a<hr>c</del></div>", "~~a --- c~~"),
        ("<div><s>a<hr>c</s></div>", "~~a --- c~~"),
        ("<div><strike>a<hr>c</strike></div>", "~~a --- c~~"),
        ("<div><mark>a<hr>c</mark></div>", "==a --- c=="),
        ("<div><var>a<hr>c</var></div>", "*a --- c*"),
        ("<div><dfn>a<hr>c</dfn></div>", "*a --- c*"),
        ("<div><q>a<hr>c</q></div>", "\"a --- c\""),
    ];
    for (html, expected) in cases {
        assert_eq!(tier2(html).trim_end(), expected, "{html:?}");
    }
    let with_script_symbols = ConversionOptions {
        sub_symbol: "~".to_owned(),
        sup_symbol: "^".to_owned(),
        ..tier2_options()
    };
    assert_eq!(
        convert_with("<div><sub>a<hr>c</sub></div>", &with_script_symbols).trim_end(),
        "~a --- c~"
    );
    assert_eq!(
        convert_with("<div><sup>a<hr>c</sup></div>", &with_script_symbols).trim_end(),
        "^a --- c^"
    );
}

#[test]
fn should_write_a_rule_in_a_table_caption_as_text_in_the_line() {
    let out = tier2("<table><caption>a<hr>c</caption><tr><td>x</td></tr></table>");
    assert!(
        render(&out).starts_with("<p><em>a --- c</em></p>\n"),
        "{out:?} renders {:?}",
        render(&out)
    );
}

#[test]
fn should_drop_a_line_break_right_before_a_rule_between_markers() {
    for newline_style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
        let options = ConversionOptions {
            newline_style,
            ..tier2_options()
        };
        let out = convert_with("<details><summary>a<br><hr>c</summary></details>", &options);
        assert_eq!(out.trim_end(), "**a --- c**", "{newline_style:?}");
    }
}

#[test]
fn should_keep_a_rule_outside_markers_on_its_own_line() {
    let cases = [
        ("<div><span>a<hr>c</span></div>", "a\n\n---\n\nc"),
        ("<div><u>a<hr>c</u></div>", "a\n\n---\n\nc"),
        ("<div><ins>a<hr>c</ins></div>", "a\n\n---\n\nc"),
        ("<div><sub>a<hr>c</sub></div>", "a\n\n---\n\nc"),
        ("<p>a<hr>c</p>", "a\n\n---\n\nc"),
        ("<dl><dt>t</dt><dd><hr></dd></dl>", "t\n\n---"),
    ];
    for (html, expected) in cases {
        assert_eq!(tier2(html).trim_end(), expected, "{html:?}");
    }
}

#[test]
fn should_keep_a_rule_in_a_link_label_unchanged() {
    assert_eq!(tier2("<div><a href=\"u\">a<hr>c</a></div>").trim_end(), "[a --- c](u)");
}

#[test]
fn should_hand_a_rule_between_markers_to_the_full_converter() {
    let options = ConversionOptions {
        extract_metadata: false,
        tier_strategy: TierStrategy::Auto,
        ..ConversionOptions::default()
    };
    for html in [
        "<div><b>a<hr>c</b></div>",
        "<div><em>a<hr>c</em></div>",
        "<div><del>a<hr>c</del></div>",
        "<details><summary>a<hr>c</summary></details>",
        "<figure><figcaption>a<hr>c</figcaption></figure>",
    ] {
        let result = tier1::run(html, &PrescanReport::default(), &options);
        assert!(
            matches!(result, Err(tier1::BailReason::RuleBetweenInlineMarkers)),
            "{html:?}: {result:?}"
        );
    }
}

#[test]
fn should_agree_across_tiers_on_a_rule_between_markers() {
    let options = ConversionOptions {
        extract_metadata: false,
        tier_strategy: TierStrategy::Auto,
        ..ConversionOptions::default()
    };
    let mut failures = Vec::new();
    for (html, _) in DEFINITION_IN_MARKERS.iter().chain(RULE_IN_MARKERS.iter()) {
        let tier2_out = tier2(html);
        if let Ok(tier1_out) = tier1::run(html, &PrescanReport::default(), &options)
            && tier1_out != tier2_out
        {
            failures.push(format!("{html:?}: tier1 {tier1_out:?} vs tier2 {tier2_out:?}"));
        }
        let auto_out = convert_with(html, &options);
        if auto_out != tier2_out {
            failures.push(format!("{html:?}: auto {auto_out:?} vs tier2 {tier2_out:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn should_join_a_rule_that_starts_a_term_or_a_section_between_markers() {
    for (html, expected) in [
        ("<div><b><dl><dd>d</dd><dt><hr>t</dt></dl></b></div>", "**d --- t**"),
        ("<div><b>a<section><hr>x</section></b></div>", "**a --- x**"),
    ] {
        assert_eq!(tier2(html).trim_end(), expected, "{html:?}");
    }
}

/// ~keep A paragraph splits the emphasis into valid per-block runs; the rule after it still follows
/// the surrounding marker's judgement and continues the second run.
#[test]
fn should_write_a_rule_after_a_paragraph_between_markers_as_text() {
    assert_eq!(
        tier2("<div><b>a<p>x</p><hr>c</b></div>").trim_end(),
        "**a**\n\n**x --- c**"
    );
}

#[test]
fn should_write_text_after_a_block_in_a_list_in_a_marker_only_wrapper_out_of_the_block() {
    let with_script_symbols = ConversionOptions {
        sub_symbol: "~".to_owned(),
        sup_symbol: "^".to_owned(),
        ..tier2_options()
    };
    let mut failures = Vec::new();
    for (tag, open, close) in [
        ("del", "~~", "~~"),
        ("s", "~~", "~~"),
        ("mark", "==", "=="),
        ("var", "*", "*"),
        ("dfn", "*", "*"),
        ("sub", "~", "~"),
        ("sup", "^", "^"),
    ] {
        for (body, expected) in [
            (
                "<ul><li>x<blockquote>q</blockquote>t</li></ul>",
                format!("{open}- x\n  > q\n\nt{close}"),
            ),
            ("<ul><li>x<p>p</p>t</li></ul>", format!("{open}- x\n\np\n\nt{close}")),
        ] {
            let html = format!("<div><{tag}>{body}</{tag}></div>");
            let out = convert_with(&html, &with_script_symbols);
            if out.trim_end() != expected {
                failures.push(format!("{html:?}: {out:?}, expected {expected:?}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "the item's marker is text, so the text after a block starts at column 0 and leaves the block:\n{}",
        failures.join("\n")
    );
}
