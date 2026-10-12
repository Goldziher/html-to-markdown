// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, NewlineStyle, TierStrategy, convert};

fn markdown(html: &str, tier_strategy: TierStrategy, newline_style: NewlineStyle) -> String {
    convert(
        html,
        Some(ConversionOptions {
            extract_metadata: false,
            tier_strategy,
            newline_style,
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed")
    .content
    .unwrap_or_default()
}

#[test]
fn should_keep_noncollapsible_whitespace_after_a_hard_break_in_every_tier() {
    for (html, expected) in [
        ("<div>a<br>&#12;b</div>", "a  \n\u{c}b\n"),
        ("<div>a<br>\u{c}b</div>", "a  \n\u{c}b\n"),
        ("<div>a<br>&#x3000;b</div>", "a  \n\u{3000}b\n"),
        ("<div>a<br>\u{3000}b</div>", "a  \n\u{3000}b\n"),
    ] {
        for tier_strategy in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
            assert_eq!(
                markdown(html, tier_strategy, NewlineStyle::Spaces),
                expected,
                "{tier_strategy:?}: {html:?}"
            );
        }
    }
}

#[test]
fn should_keep_both_breaks_in_one_list_item_paragraph() {
    let html = "<ul><li>a<br><b><br>b</b></li></ul>";
    let expected = "- a  \n  \\\n  **b**\n";

    for tier_strategy in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
        let output = markdown(html, tier_strategy, NewlineStyle::Spaces);
        assert_eq!(output, expected, "{tier_strategy:?}");
        assert_eq!(
            comrak::markdown_to_html(&output, &comrak::Options::default()),
            "<ul>\n<li>a<br />\n<br />\n<strong>b</strong></li>\n</ul>\n",
            "{tier_strategy:?}"
        );
    }
}

#[test]
fn should_keep_both_backslash_breaks_in_one_list_item_paragraph() {
    let html = "<ul><li>a<br><b><br>b</b></li></ul>";

    assert_eq!(
        markdown(html, TierStrategy::Tier2, NewlineStyle::Backslash),
        "- a\\\n  \\\n  **b**\n"
    );
}

#[test]
fn should_preserve_ideographic_indent_after_hard_breaks() {
    for spelling in ["\u{3000}", "&#12288;", "&#x3000;"] {
        for (style, marker) in [(NewlineStyle::Spaces, "  \n"), (NewlineStyle::Backslash, "\\\n")] {
            let html = format!("<div>a<br>{spelling}b</div>");
            let expected = format!("a{marker}\u{3000}b\n");
            for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
                for extract_metadata in [false, true] {
                    let result = convert(
                        &html,
                        Some(ConversionOptions {
                            extract_metadata,
                            tier_strategy,
                            newline_style: style,
                            ..ConversionOptions::default()
                        }),
                    )
                    .expect("conversion succeeds");
                    assert_eq!(
                        result.content.as_deref(),
                        Some(expected.as_str()),
                        "{html}, {tier_strategy:?}"
                    );
                }
            }
        }
        let pre = format!("<pre>a<br>{spelling}b</pre>");
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            assert_eq!(
                markdown(&pre, tier_strategy, NewlineStyle::Spaces),
                "```\na\n\u{3000}b\n```\n"
            );
        }
    }
}

#[test]
fn should_collapse_only_ascii_runs_in_a_unicode_line_indent() {
    for (html, expected) in [
        ("<div>a<br>  \u{3000}  \u{3000} b</div>", "a  \n\u{3000} \u{3000} b\n"),
        ("<div>a<br>   b</div>", "a  \nb\n"),
        ("<div>a<br>&nbsp;b</div>", "a  \n\u{a0}b\n"),
    ] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            assert_eq!(
                markdown(html, tier_strategy, NewlineStyle::Spaces),
                expected,
                "{html}, {tier_strategy:?}"
            );
        }
    }
}

#[test]
fn should_keep_ideographic_indent_before_an_inline_child() {
    for spelling in ["&#x3000;", "\u{3000}"] {
        let html = format!("<div>a<br>{spelling}<span>b</span></div>");
        assert_eq!(
            html_to_markdown_rs::tier1::run(
                &html,
                &html_to_markdown_rs::prescan::PrescanReport::default(),
                &ConversionOptions {
                    extract_metadata: false,
                    highlight_style: html_to_markdown_rs::HighlightStyle::None,
                    ..ConversionOptions::default()
                },
            )
            .expect("split Unicode-only text remains on the fast scanner"),
            "a  \n\u{3000}b\n"
        );
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            for extract_metadata in [false, true] {
                let result = convert(
                    &html,
                    Some(ConversionOptions {
                        tier_strategy,
                        extract_metadata,
                        ..ConversionOptions::default()
                    }),
                )
                .expect("conversion succeeds");
                assert_eq!(
                    result.content.as_deref(),
                    Some("a  \n\u{3000}b\n"),
                    "{html}, {tier_strategy:?}"
                );
            }
        }
    }
}
