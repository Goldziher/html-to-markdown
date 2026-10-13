//! Regression coverage for review #871 authored-break and structural-spacing boundaries. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, TierStrategy, convert};
const LITERAL_CONTROLS: [(&str, &str, &str, &str, &str); 14] = [
    (
        "<p><b>a</b><i></i>b</p>",
        "**a**b\n",
        "**a**b\n",
        "**a**b\n",
        "**a**b\n",
    ),
    (
        "<p><b>a</b><i> </i>b</p>",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
    ),
    (
        "<p><b>a</b><i>&#9;</i>b</p>",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
    ),
    (
        "<p><b>a</b><i>&#13;</i>b</p>",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
    ),
    (
        "<p><b>a</b><i>\n</i>b</p>",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
    ),
    (
        "<p><b>a</b><i> \n</i>b</p>",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
    ),
    (
        "<p><b>a</b><i><em></em></i>b</p>",
        "**a**b\n",
        "**a**b\n",
        "**a**b\n",
        "**a**b\n",
    ),
    (
        "<p><b>a</b><i><em>\n</em></i>b</p>",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
    ),
    (
        "<p><b>a</b><i><code>\n</code></i>b</p>",
        "**a**b\n",
        "**a**b\n",
        "**a**b\n",
        "**a**b\n",
    ),
    (
        "<p><b>a</b><i><code>&#13;</code></i>b</p>",
        "**a***`\r`*b\n",
        "**a***`\r`*b\n",
        "**a***`\r`*b\n",
        "**a***`\r`*b\n",
    ),
    (
        "<p><b>a</b><i><pre></pre></i>b</p>",
        "**a**b\n",
        "**a**b\n",
        "**a**b\n",
        "**a**b\n",
    ),
    (
        "<p><b>a</b><i><pre>\n</pre></i>b</p>",
        "**a***```*\n\n*```*b\n",
        "**a***```*\n\n*```*b\n",
        "**a***```*\n\n*```*b\n",
        "**a***```*\n\n*```*b\n",
    ),
    (
        "<p><b>a</b><i>&nbsp;</i>b</p>",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
        "**a** b\n",
    ),
    (
        "<p><b>a</b><i>\\\n</i>b</p>",
        "**a***\\\\* b\n",
        "**a***\\\\* b\n",
        "**a***\\\\* b\n",
        "**a***\\\\* b\n",
    ),
];
#[cfg(feature = "visitor")]
const BREAK_VISITOR_CONTROLS: [(&str, NewlineStyle, usize, &str, &str, usize); 32] = [
    (
        "<p><b>a</b><i> <br></i>b</p>",
        NewlineStyle::Spaces,
        0,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i> <br></i>b</p>",
        NewlineStyle::Spaces,
        1,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i> <br></i>b</p>",
        NewlineStyle::Spaces,
        2,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i> <br></i>b</p>",
        NewlineStyle::Spaces,
        3,
        "**a** *V*b\n",
        "**a** *V*b\n",
        1,
    ),
    (
        "<p><b>a</b><i> <br></i>b</p>",
        NewlineStyle::Backslash,
        0,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i> <br></i>b</p>",
        NewlineStyle::Backslash,
        1,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i> <br></i>b</p>",
        NewlineStyle::Backslash,
        2,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i> <br></i>b</p>",
        NewlineStyle::Backslash,
        3,
        "**a** *V*b\n",
        "**a** *V*b\n",
        1,
    ),
    (
        "<p>x</p><p><b>a</b><em><em><br></em></em>b</p>",
        NewlineStyle::Spaces,
        0,
        "x\n\n**a**b\n",
        "x\n\n**a**  \nb\n",
        0,
    ),
    (
        "<p>x</p><p><b>a</b><em><em><br></em></em>b</p>",
        NewlineStyle::Spaces,
        1,
        "x\n\n**a** b\n",
        "x\n\n**a**  \nb\n",
        0,
    ),
    (
        "<p>x</p><p><b>a</b><em><em><br></em></em>b</p>",
        NewlineStyle::Spaces,
        2,
        "x\n\n**a**  \nb\n",
        "x\n\n**a**  \nb\n",
        0,
    ),
    (
        "<p>x</p><p><b>a</b><em><em><br></em></em>b</p>",
        NewlineStyle::Spaces,
        3,
        "x\n\n**a****V**b\n",
        "x\n\n**a**  \nb\n",
        0,
    ),
    (
        "<p>x</p><p><b>a</b><em><em><br></em></em>b</p>",
        NewlineStyle::Backslash,
        0,
        "x\n\n**a**b\n",
        "x\n\n**a**b\n",
        1,
    ),
    (
        "<p>x</p><p><b>a</b><em><em><br></em></em>b</p>",
        NewlineStyle::Backslash,
        1,
        "x\n\n**a** b\n",
        "x\n\n**a** b\n",
        1,
    ),
    (
        "<p>x</p><p><b>a</b><em><em><br></em></em>b</p>",
        NewlineStyle::Backslash,
        2,
        "x\n\n**a**\\\nb\n",
        "x\n\n**a**\\\nb\n",
        1,
    ),
    (
        "<p>x</p><p><b>a</b><em><em><br></em></em>b</p>",
        NewlineStyle::Backslash,
        3,
        "x\n\n**a****V**b\n",
        "x\n\n**a****V**b\n",
        1,
    ),
    (
        "<p><b>a</b><i><span>\n</span><br></i>b</p>",
        NewlineStyle::Spaces,
        0,
        "**a** b\n",
        "**a**  \nb\n",
        0,
    ),
    (
        "<p><b>a</b><i><span>\n</span><br></i>b</p>",
        NewlineStyle::Spaces,
        1,
        "**a** b\n",
        "**a**  \nb\n",
        0,
    ),
    (
        "<p><b>a</b><i><span>\n</span><br></i>b</p>",
        NewlineStyle::Spaces,
        2,
        "**a** b\n",
        "**a**  \nb\n",
        0,
    ),
    (
        "<p><b>a</b><i><span>\n</span><br></i>b</p>",
        NewlineStyle::Spaces,
        3,
        "**a** *V*b\n",
        "**a**  \nb\n",
        0,
    ),
    (
        "<p><b>a</b><i><span>\n</span><br></i>b</p>",
        NewlineStyle::Backslash,
        0,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i><span>\n</span><br></i>b</p>",
        NewlineStyle::Backslash,
        1,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i><span>\n</span><br></i>b</p>",
        NewlineStyle::Backslash,
        2,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i><span>\n</span><br></i>b</p>",
        NewlineStyle::Backslash,
        3,
        "**a** *V*b\n",
        "**a** *V*b\n",
        1,
    ),
    (
        "<p><b>a</b><i><code>\n</code><br></i>b</p>",
        NewlineStyle::Spaces,
        0,
        "**a**b\n",
        "**a**b\n",
        1,
    ),
    (
        "<p><b>a</b><i><code>\n</code><br></i>b</p>",
        NewlineStyle::Spaces,
        1,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i><code>\n</code><br></i>b</p>",
        NewlineStyle::Spaces,
        2,
        "**a**  \nb\n",
        "**a**  \nb\n",
        1,
    ),
    (
        "<p><b>a</b><i><code>\n</code><br></i>b</p>",
        NewlineStyle::Spaces,
        3,
        "**a***V*b\n",
        "**a***V*b\n",
        1,
    ),
    (
        "<p><b>a</b><i><code>\n</code><br></i>b</p>",
        NewlineStyle::Backslash,
        0,
        "**a**b\n",
        "**a**b\n",
        1,
    ),
    (
        "<p><b>a</b><i><code>\n</code><br></i>b</p>",
        NewlineStyle::Backslash,
        1,
        "**a** b\n",
        "**a** b\n",
        1,
    ),
    (
        "<p><b>a</b><i><code>\n</code><br></i>b</p>",
        NewlineStyle::Backslash,
        2,
        "**a**\\\nb\n",
        "**a**\\\nb\n",
        1,
    ),
    (
        "<p><b>a</b><i><code>\n</code><br></i>b</p>",
        NewlineStyle::Backslash,
        3,
        "**a***V*b\n",
        "**a***V*b\n",
        1,
    ),
];
fn assert_routes(source: &str, style: NewlineStyle, expected: &str) -> usize {
    let mut rows = 0;
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for extract_metadata in [false, true] {
            let result = convert(
                source,
                ConversionOptions {
                    tier_strategy,
                    extract_metadata,
                    newline_style: style,
                    ..Default::default()
                },
            )
            .expect("conversion");
            assert_eq!(
                result.content.as_deref(),
                Some(expected),
                "{source} {style:?} {tier_strategy:?}"
            );
            rows += 1;
        }
    }
    rows
}
#[test]
fn should_retain_authored_breaks_after_strong_through_blank_wrappers() {
    let mut rows = 0;
    for style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
        let marker = if style == NewlineStyle::Spaces { "  \n" } else { "\\\n" };
        for tag in ["i", "b", "s", "mark", "em"] {
            let direct_marker = if style == NewlineStyle::Backslash {
                " \\\n"
            } else {
                marker
            };
            rows += assert_routes(
                &format!("<p><b>a</b><{tag}> <br></{tag}>b</p>"),
                style,
                &format!("**a**{direct_marker}b\n"),
            );
        }
        rows += assert_routes(
            "<p>x</p><p><b>a</b><em><em><br></em></em>b</p>",
            style,
            &format!("x\n\n**a**{marker}b\n"),
        );
    }
    assert_eq!(rows, 72);
}
#[test]
fn should_remove_terminal_definition_description_backslash_break() {
    let mut rows = 0;
    for style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
        rows += assert_routes("<dl><dt>t</dt><dd>a<b><br></b></dd></dl>", style, "t\n\na\n");
    }
    assert_eq!(rows, 12);
}
#[test]
fn should_keep_list_content_spacing_after_empty_wrapped_blocks() {
    let mut rows = 0;
    for style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
        for tag in ["h2", "blockquote", "pre", "section", "div", "table"] {
            rows += assert_routes(
                &format!("<ul><li>a \n<span><{tag}></{tag}></span>&nbsp;b</li></ul>"),
                style,
                "- a\n   b\n",
            );
        }
    }
    assert_eq!(rows, 72);
}

#[test]
fn should_preserve_literal_and_empty_wrappers_without_authored_breaks() {
    let mut rows = 0;
    for (source, spaces, spaces_fast, backslash, backslash_fast) in LITERAL_CONTROLS {
        for (style, normal, fast) in [
            (NewlineStyle::Spaces, spaces, spaces_fast),
            (NewlineStyle::Backslash, backslash, backslash_fast),
        ] {
            for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
                for extract_metadata in [false, true] {
                    let result = convert(
                        source,
                        ConversionOptions {
                            tier_strategy,
                            extract_metadata,
                            newline_style: style,
                            ..Default::default()
                        },
                    )
                    .expect("literal control");
                    assert_eq!(
                        result.content.as_deref(),
                        Some(if tier_strategy == TierStrategy::Tier1 {
                            fast
                        } else {
                            normal
                        }),
                        "{source} {style:?} {tier_strategy:?}"
                    );
                    rows += 1;
                }
            }
        }
    }
    assert_eq!(rows, 168);
}

#[cfg(feature = "visitor")]
#[test]
fn should_preserve_skipped_and_custom_break_callback_output() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct BreakVisitor {
        mode: usize,
        calls: usize,
    }
    impl HtmlVisitor for BreakVisitor {
        fn visit_line_break(&mut self, _: &NodeContext) -> VisitResult {
            self.calls += 1;
            match self.mode {
                0 => VisitResult::Skip,
                1 => VisitResult::Custom(" ".into()),
                2 => VisitResult::Custom("\n".into()),
                _ => VisitResult::Custom("V".into()),
            }
        }
    }
    let mut rows = 0;
    for (source, style, mode, normal, fast, fast_calls) in BREAK_VISITOR_CONTROLS {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            for extract_metadata in [false, true] {
                let visitor = Arc::new(Mutex::new(BreakVisitor { mode, calls: 0 }));
                let result = convert(
                    source,
                    ConversionOptions {
                        tier_strategy,
                        extract_metadata,
                        newline_style: style,
                        include_document_structure: true,
                        visitor: Some(visitor.clone()),
                        ..Default::default()
                    },
                )
                .expect("observed break");
                assert_eq!(
                    result.content.as_deref(),
                    Some(if tier_strategy == TierStrategy::Tier1 {
                        fast
                    } else {
                        normal
                    })
                );
                assert_eq!(
                    visitor.lock().expect("callback count").calls,
                    if tier_strategy == TierStrategy::Tier1 {
                        fast_calls
                    } else {
                        1
                    }
                );
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 192);
}
