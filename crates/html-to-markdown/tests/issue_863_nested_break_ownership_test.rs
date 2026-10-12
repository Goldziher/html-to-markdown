//! Public regressions for owning flow buffers and detached inline breaks. ~keep
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, NewlineStyle, TierStrategy, convert};

fn assert_routes(html: &str, expected: &str, newline_style: NewlineStyle) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for extract_metadata in [false, true] {
            assert_conversion(html, expected, newline_style, tier_strategy, extract_metadata);
        }
    }
}

fn assert_conversion(
    html: &str,
    expected: &str,
    newline_style: NewlineStyle,
    tier_strategy: TierStrategy,
    extract_metadata: bool,
) {
    let result = convert(
        html,
        Some(ConversionOptions {
            newline_style,
            tier_strategy,
            extract_metadata,
            highlight_style: HighlightStyle::None,
            ..ConversionOptions::default()
        }),
    )
    .expect("inline break converts");
    assert_eq!(
        result.content.as_deref(),
        Some(expected),
        "{html:?}, {tier_strategy:?}, metadata={extract_metadata}, {newline_style:?}"
    );
}

fn nested(tag: &str, depth: usize, body: &str) -> String {
    format!(
        "{}{body}{}",
        format!("<{tag}>").repeat(depth),
        format!("</{tag}>").repeat(depth)
    )
}

fn owning_blocks(body: &str) -> [(String, &'static str); 4] {
    [
        (format!("<ul><li>a{body}b</li></ul>"), "- a  \n  b\n"),
        (format!("<blockquote>a{body}b</blockquote>"), "> a  \n> b\n"),
        (format!("<dd>a{body}b</dd>"), "a  \nb\n"),
        (
            format!("<figure><figcaption>a{body}b</figcaption></figure>"),
            "*a  \nb*\n",
        ),
    ]
}

#[test]
fn should_keep_nested_breaks_after_text_in_a_later_paragraph() {
    let mut rows = 0;
    for tag in ["b", "strong", "em", "i"] {
        for depth in [2, 3, 6] {
            for (style, expected) in [
                (NewlineStyle::Spaces, "x\n\na  \nb\n"),
                (NewlineStyle::Backslash, "x\n\na\\\nb\n"),
            ] {
                let html = format!("<p>x</p><p>a{}b</p>", nested(tag, depth, "<br>"));
                assert_routes(&html, expected, style);
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 24);
}

#[test]
fn should_keep_a_nested_blank_break_in_every_owning_flow_block() {
    let mut rows = 0;
    for tag in ["b", "strong", "em", "i"] {
        for depth in [2, 3, 6] {
            for (html, expected) in owning_blocks(&nested(tag, depth, "&nbsp;<br>")) {
                assert_routes(&html, expected, NewlineStyle::Spaces);
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 48);
}

#[test]
fn should_emit_a_configured_break_inside_a_nonempty_inline_scratch_buffer() {
    let mut rows = 0;
    for (tag, marker) in [("b", "**"), ("strong", "**"), ("em", "*"), ("i", "*")] {
        for (style, line_break) in [(NewlineStyle::Spaces, "  \n"), (NewlineStyle::Backslash, "\\\n")] {
            let html = format!("<ul><li><{tag}>x<{tag}><br></{tag}>y</{tag}></li></ul>");
            let expected = format!("- {marker}x{line_break}  y{marker}\n");
            assert_routes(&html, &expected, style);
            rows += 1;
        }
    }
    assert_eq!(rows, 8);
}

#[test]
fn should_drop_leading_wrapped_breaks_in_full_flow_buffers() {
    let mut rows = 0;
    // ~keep The forced fast scanner's leading unmarked-break difference predates this PR.
    for tag in ["b", "strong", "em", "i", "mark"] {
        for depth in [1, 2, 6] {
            let body = nested(tag, depth, "<br>");
            for (html, expected) in [
                (format!("<ul><li>{body}w</li></ul>"), "- w\n"),
                (format!("<ul><li><input type=checkbox>{body}w</li></ul>"), "- [ ] w\n"),
                (format!("<blockquote>{body}w</blockquote>"), "> w\n"),
                (format!("<dd>{body}w</dd>"), "w\n"),
                (format!("<dt>{body}w</dt>"), "w\n"),
                (format!("<figure><figcaption>{body}w</figcaption></figure>"), "*w*\n"),
            ] {
                for style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
                    for metadata in [false, true] {
                        assert_conversion(&html, expected, style, TierStrategy::Tier2, metadata);
                        rows += 1;
                    }
                }
            }
        }
    }
    assert_eq!(rows, 360);
}

#[test]
fn should_collapse_source_line_feeds_without_inventing_a_hard_break() {
    for tag in ["b", "strong", "em", "i"] {
        for metadata in [false, true] {
            assert_conversion(
                &format!("<p>x</p><p>a{}b</p>", nested(tag, 3, "\n")),
                "x\n\na b\n",
                NewlineStyle::Spaces,
                TierStrategy::Tier2,
                metadata,
            );
        }
    }
}

#[test]
fn should_preserve_blank_wrapper_separators_after_real_strong_siblings() {
    let mut rows = 0;
    for body in [
        "<b><br></b>",
        "<b><span><br></span></b>",
        "<strong><em><br></em></strong>",
        "<mark><em><br></em></mark>",
        "<b><b><br></b></b>",
        "<em><em><br></em></em>",
    ] {
        for paragraph in [false, true] {
            for newline_style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
                for (strong_em_symbol, expected) in [('*', "**Alpha** Beta\n"), ('_', "__Alpha__ Beta\n")] {
                    for highlight_style in [HighlightStyle::None, HighlightStyle::DoubleEqual] {
                        for extract_metadata in [false, true] {
                            let html = format!("<strong>Alpha</strong>{body}Beta");
                            let html = if paragraph { format!("<p>{html}</p>") } else { html };
                            let result = convert(
                                &html,
                                Some(ConversionOptions {
                                    tier_strategy: TierStrategy::Tier2,
                                    newline_style,
                                    strong_em_symbol,
                                    highlight_style,
                                    extract_metadata,
                                    ..ConversionOptions::default()
                                }),
                            )
                            .expect("blank formatted separator converts");
                            assert_eq!(result.content.as_deref(), Some(expected), "{html}, {newline_style:?}");
                            rows += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(rows, 192);
}

#[test]
fn should_keep_a_hard_break_after_literal_marker_looking_prose() {
    for tag in ["b", "strong", "em", "i"] {
        for style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
            let expected = if style == NewlineStyle::Spaces {
                "Alpha **  \nBeta\n"
            } else {
                "Alpha **\\\nBeta\n"
            };
            for metadata in [false, true] {
                assert_conversion(
                    &format!("<p>Alpha **{}Beta</p>", nested(tag, 3, "<br>")),
                    expected,
                    style,
                    TierStrategy::Tier2,
                    metadata,
                );
            }
        }
    }
}
