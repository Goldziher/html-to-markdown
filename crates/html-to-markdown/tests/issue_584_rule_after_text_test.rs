// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #584: a horizontal rule right after a line of text was written on the
//! next line with no blank line. `CommonMark` reads `---` under text as a setext heading underline, so
//! the text became a heading and the rule disappeared.

use html_to_markdown_rs::options::{NewlineStyle, WhitespaceMode};
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert, tier1};

/// A rule right after text, in every container that wrote it on the next line.
const RULE_AFTER_TEXT: [&str; 21] = [
    "<p>t<hr>B</p>",
    "<p>t<hr></p>",
    "<p>t<br><hr>B</p>",
    "<p>t<wbr><hr></p>",
    "<p><em>t</em><hr></p>",
    "<p>t <hr></p>",
    "<p>t\n<hr></p>",
    "<p>t<hr><hr>B</p>",
    "<div><p>t<hr>B</p></div>",
    "<blockquote><p>t<hr>B</p></blockquote>",
    "<dl><dt>t</dt><dd><hr></dd></dl>",
    "<dl><dt>t</dt><dd><hr>x</dd></dl>",
    "<dl><dt>t</dt><dd>\n<hr>\n</dd></dl>",
    "<dl><dt>t</dt><dd><div><hr></div></dd></dl>",
    "<dl><dt>t</dt><dd><p><hr></p></dd></dl>",
    "<dl><dt>t</dt><dd><span><hr></span></dd></dl>",
    "<dl><dt>t</dt><dt><hr></dt></dl>",
    "<dl>t<dd><hr></dd></dl>",
    "<blockquote><dl><dt>t</dt><dd><hr></dd></dl></blockquote>",
    "<ul><li>t<dl><dt>t</dt><dd><hr></dd></dl></li></ul>",
    "<ul><li><p>A<hr>B</p></li></ul>",
];

fn tier2_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
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

/// The rendered HTML must hold a rule and no heading.
fn rule_not_heading(html: &str, markdown: &str) -> Option<String> {
    let rendered = render(markdown);
    if rendered.contains("<hr />") && !rendered.contains("<h1>") && !rendered.contains("<h2>") {
        None
    } else {
        Some(format!("{html:?}: {markdown:?} renders {rendered:?}"))
    }
}

#[test]
fn should_write_a_blank_line_before_a_rule_after_text_in_a_definition() {
    let out = tier2("<dl><dt>t</dt><dd><hr></dd></dl>");
    assert_eq!(out, "t\n\n---\n", "the rule must start after a blank line");
    assert_eq!(render(&out), "<p>t</p>\n<hr />\n", "t must stay a paragraph");
}

#[test]
fn should_write_a_blank_line_before_a_rule_after_text_in_a_paragraph() {
    let out = tier2("<p>t<hr>B</p>");
    assert_eq!(out, "t\n\n---\n\nB\n", "the rule must start after a blank line");
    assert_eq!(render(&out), "<p>t</p>\n<hr />\n<p>B</p>\n", "t must stay a paragraph");
}

#[test]
fn should_render_a_rule_after_text_as_a_rule_in_every_mode() {
    let modes = [
        ("default", tier2_options()),
        (
            "strict whitespace",
            ConversionOptions {
                whitespace_mode: WhitespaceMode::Strict,
                ..tier2_options()
            },
        ),
        (
            "backslash breaks",
            ConversionOptions {
                newline_style: NewlineStyle::Backslash,
                ..tier2_options()
            },
        ),
        (
            "automatic tier",
            ConversionOptions {
                tier_strategy: TierStrategy::Auto,
                ..tier2_options()
            },
        ),
    ];
    let mut failures = Vec::new();
    for (mode, options) in &modes {
        for html in RULE_AFTER_TEXT {
            if let Some(failure) = rule_not_heading(html, &convert_with(html, options)) {
                failures.push(format!("{mode}: {failure}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "the text before the rule became a heading:\n{}",
        failures.join("\n")
    );
}

#[test]
fn should_agree_across_tiers_on_a_rule_after_text() {
    // ~keep Tier 1 bails on some of these (a definition list inside a list item, for one); the
    // ~keep count assertion keeps this from passing when every case bails.
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        ..tier2_options()
    };
    let mut compared = 0;
    let mut failures = Vec::new();
    for html in RULE_AFTER_TEXT {
        if let Ok(tier1_out) = tier1::run(html, &PrescanReport::default(), &options) {
            let tier2_out = tier2(html);
            if tier1_out != tier2_out {
                failures.push(format!("{html:?}: tier1 {tier1_out:?} vs tier2 {tier2_out:?}"));
            }
            compared += 1;
        }
    }
    assert!(
        failures.is_empty(),
        "Tier 1 and Tier 2 must agree:\n{}",
        failures.join("\n")
    );
    assert!(compared >= 15, "Tier 1 converted only {compared} cases");
}

#[test]
fn should_keep_a_rule_after_text_off_the_text_line_in_inline_mode() {
    let options = ConversionOptions {
        convert_as_inline: true,
        ..tier2_options()
    };
    let html = "<p>t<hr>B</p>";
    let out = convert_with(html, &options);
    assert_eq!(
        out,
        convert_with("<div>t<hr>B</div>", &options),
        "a rule in a paragraph must be spaced like a rule in a div"
    );
    assert_eq!(rule_not_heading(html, &out), None, "t must stay a paragraph");
}

#[test]
fn should_leave_rules_that_already_had_a_blank_line_or_follow_a_quote_unchanged() {
    for (html, expected) in [
        ("<p>A</p><hr><p>B</p>", "A\n\n---\n\nB\n"),
        ("<div>t<hr>B</div>", "t\n\n---\n\nB\n"),
        ("<p><hr>t</p>", "---\n\nt\n"),
        ("<blockquote>q</blockquote><hr>", "> q\n---\n"),
        ("<dl><dt>t</dt><dd>d<hr></dd></dl>", "t\n\nd\n\n---\n"),
        ("<dl><dt>t</dt><dd>d</dd><dd><hr></dd></dl>", "t\n\nd\n\n---\n"),
        (
            "<table><tr><th>h</th></tr><tr><td>A<hr>B</td></tr></table>",
            "| h        |\n| -------- |\n| A  --- B |\n",
        ),
    ] {
        assert_eq!(
            tier2(html),
            expected,
            "Tier 2 changed a rule that was already separated: {html:?}"
        );
    }
}

#[test]
fn should_keep_a_definition_separate_from_its_term() {
    for (html, expected) in [
        ("<dl><dt>T</dt><dd>D</dd></dl>", "T\n\nD\n"),
        ("<dl><dt>t</dt><dd><ul><li>a</li></ul></dd></dl>", "t\n\n- a\n"),
        ("<dl><dt>t</dt><dd><h2>h</h2></dd></dl>", "t\n\n## h\n"),
        ("<dl><dt>t</dt><dt>u</dt></dl>", "t\n\nu\n"),
    ] {
        assert_eq!(tier2(html), expected, "Tier 2 changed a definition: {html:?}");
    }
}

#[test]
fn should_not_add_a_second_blank_line_before_a_rule_that_already_has_one() {
    // ~keep Inside a quote every line keeps its `>`, so a second blank line survives as an extra
    // ~keep `>` line instead of being collapsed at the end of the conversion.
    let html = "<blockquote><dl><dt>t</dt><dd>d</dd><dd><hr></dd></dl></blockquote>";
    assert_eq!(
        tier2(html),
        "> t\n>\n> d\n>\n> ---\n",
        "a rule after a blank line must not get another one: {html:?}"
    );
}

#[test]
fn should_not_start_the_output_with_blank_lines_before_a_leading_rule() {
    let html = "<dd><hr></dd>";
    assert_eq!(
        tier2(html),
        "---\n",
        "a rule with no text before it needs no blank line: {html:?}"
    );
}
