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
            &format!("{prefix}<i><hr></i><div><em><hr><img alt=\"  \"><br>"),
            &format!("{prefix}*---*\n\n*--- ![  ](<>)*  \n"),
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
