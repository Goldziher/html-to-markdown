//! Regression coverage for whitespace and break boundaries in inline elements. ~keep

#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, NewlineStyle, TierStrategy, convert};

fn assert_routes(html: &str, expected: &str) {
    assert_routes_with_newlines(html, expected, NewlineStyle::Spaces);
}

fn assert_routes_with_newlines(html: &str, expected: &str, newline_style: NewlineStyle) {
    for tier in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for extract_metadata in [false, true] {
            let options = ConversionOptions {
                tier_strategy: tier,
                extract_metadata,
                newline_style,
                highlight_style: HighlightStyle::None,
                ..ConversionOptions::default()
            };
            let result = convert(html, Some(options)).expect("conversion succeeds");
            assert_eq!(
                result.content.as_deref(),
                Some(expected),
                "{html:?}, {tier:?}, metadata={extract_metadata}"
            );
        }
    }
}

#[test]
fn should_preserve_nbsp_only_wrapped_bodies_after_spaces() {
    for tag in ["em", "b", "strong", "i", "span"] {
        assert_routes(&format!("<p>a <{tag}>&nbsp;</{tag}> b</p>"), "a \u{a0} b\n");
        assert_routes(
            &format!("<pre>a <{tag}>&nbsp;</{tag}> b</pre>"),
            "```\na \u{a0} b\n```\n",
        );
    }
}

#[test]
fn should_preserve_hard_break_regardless_of_prefix_length() {
    for length in 1..=24 {
        let prefix = "a".repeat(length);
        assert_routes(
            &format!("{prefix}<i><hr></i><div><em><hr><img src=/i alt=\"  \"><br>"),
            &format!("{prefix}*---*\n\n*--- ![  ](/i)*  \n"),
        );
    }
}

#[test]
fn should_keep_existing_leading_break_and_blank_wrapper_rules() {
    assert_routes("<p><br>word</p>", "word\n");
    assert_routes("<p>a <i> </i> b</p>", "a b\n");
    assert_routes("<p>a<em><br>word</em></p>", "a  \n*word*\n");
    assert_routes("<p>a<em>word<br></em>b</p>", "a*word*  \nb\n");
}

#[test]
fn should_use_full_converter_tree_rules_for_code_like_blocks_and_leading_breaks() {
    for tag in ["kbd", "samp", "code"] {
        for (before, after) in [("one", ""), ("one\n", ""), ("one ", " z")] {
            let html = format!("<p>{before}<{tag}><div>y</div></{tag}>{after}</p>");
            let expected = if tag == "code" {
                if before == "one" {
                    "one`y`\n"
                } else if after.is_empty() {
                    "one `y`\n"
                } else {
                    "one `y` z\n"
                }
            } else if after.is_empty() {
                "one\n\ny\n"
            } else {
                "one\n\ny\n\nz\n"
            };
            assert_routes(&html, expected);
        }
        assert_routes(&format!("<p>one <{tag}><br>y</{tag}></p>"), "one `y`\n");
    }
}

#[test]
fn should_bail_direct_scanner_only_for_unsupported_code_boundaries() {
    use html_to_markdown_rs::{prescan::PrescanReport, tier1};
    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    assert_eq!(
        tier1::run(
            "<h1>one<code><p>y</p></code>two</h1>",
            &PrescanReport::default(),
            &options
        )
        .expect("heading code boundaries remain fast"),
        "# one`y`two\n"
    );
    assert_routes("<h1>one<code><p>y</p></code>two</h1>", "# one`y`two\n");
    assert_eq!(
        tier1::run(
            "<table><tr><td><code>b c<code><br></code></code></td></tr></table>",
            &PrescanReport::default(),
            &options
        )
        .expect("cell code breaks remain fast"),
        "| `b c ` |\n| ------ |\n"
    );
    for tag in ["kbd", "samp", "code"] {
        for body in ["<div>y</div>", "<br>y"] {
            let html = format!("<p>one <{tag}>{body}</{tag}></p>");
            assert!(
                tier1::run(&html, &PrescanReport::default(), &options).is_err(),
                "{html}"
            );
        }
        let html = format!("<p>one <{tag}>x<br>y</{tag}></p>");
        assert_eq!(
            tier1::run(&html, &PrescanReport::default(), &options).expect("ordinary code remains fast"),
            "one `x`  \n`y`\n"
        );
    }
}

#[test]
fn should_preserve_nested_noncollapsible_blank_characters() {
    assert_routes("<p>one&#10;<i><span>&nbsp;</span></i>two</p>", "one \u{a0}two\n");
    for character in ["\u{a0}", "\u{2003}", "\u{c}"] {
        assert_routes(
            &format!("<p>a <em>{character}</em> b</p>"),
            &format!("a {character} b\n"),
        );
    }
    assert_routes("<p>a <em> word </em> b</p>", "a *word* b\n");
    for (style, separator) in [(NewlineStyle::Spaces, "  \n"), (NewlineStyle::Backslash, "\\\n")] {
        assert_routes_with_newlines(
            "<p>a <em>&nbsp;<br></em>b</p>",
            &format!("a \u{a0}{separator}b\n"),
            style,
        );
    }
}

#[test]
fn should_keep_a_break_only_wrapper_after_content_in_a_later_paragraph() {
    assert_routes("<p>a</p><p>q<em><br></em></p>", "a\n\nq  \n");
}

#[test]
fn should_keep_nbsp_normalization_outside_the_after_space_trigger() {
    for (body, expected) in [
        ("a<b>&nbsp;</b>b", "a b"),
        ("a<b> &nbsp;</b>", "a"),
        ("a <em><br>&nbsp;</em> b", "a b"),
        ("a <em> &nbsp;</em>b", "a b"),
    ] {
        assert_routes(&format!("<p>{body}</p>"), &format!("{expected}\n"));
        assert_routes(&format!("<h2>{body}</h2>"), &format!("## {expected}\n"));
        assert_routes(
            &format!("<table><tr><td>{body}</td></tr></table>"),
            &format!("| {expected} |\n| --- |\n"),
        );
    }
}

#[test]
fn should_keep_a_break_only_wrapper_after_content_in_the_first_paragraph() {
    assert_routes("<p>a<em><br></em>b</p>", "a  \nb\n");
    assert_routes("<p><em><br></em>word</p>", "word\n");
    assert_routes("<p><em><br>word</em></p>", "*word*\n");
    assert_routes("<br>word", "\nword\n");
}

#[test]
fn should_preserve_nbsp_after_a_space_in_default_marked_highlights() {
    for highlight_style in [HighlightStyle::DoubleEqual, HighlightStyle::Bold] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let options = ConversionOptions {
                tier_strategy,
                highlight_style,
                extract_metadata: false,
                ..Default::default()
            };
            let output = convert("<p>a <mark>&nbsp;</mark> b</p>", options)
                .expect("marked highlight")
                .content
                .unwrap();
            assert_eq!(output, "a \u{a0} b\n", "{highlight_style:?}, {tier_strategy:?}");
        }
    }
}

#[test]
fn should_keep_configured_break_only_wrapper_markers_outside_empty_emphasis() {
    assert_routes_with_newlines("<p>a<em><br></em>b</p>", "a\\\nb\n", NewlineStyle::Backslash);
    assert_routes_with_newlines("<p>a</p><p>q<em><br></em></p>", "a\n\nq\n", NewlineStyle::Backslash);
}
