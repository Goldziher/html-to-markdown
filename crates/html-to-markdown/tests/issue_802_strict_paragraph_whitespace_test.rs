#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, WhitespaceMode, convert};

fn check(html: &str, expected: &str, options: &ConversionOptions) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            html,
            Some(ConversionOptions {
                tier_strategy,
                ..options.clone()
            }),
        )
        .expect("conversion must succeed");
        let markdown = result.content.unwrap_or_default();
        assert_eq!(markdown, expected, "{tier_strategy:?}: {html:?}");
    }
}

#[test]
fn should_strip_strict_paragraph_leading_whitespace_in_every_container() {
    let options = ConversionOptions {
        whitespace_mode: WhitespaceMode::Strict,
        ..Default::default()
    };
    for spaces in ["  ", "      ", "\t"] {
        check(&format!("<p>{spaces}text</p>"), "text\n", &options);
        check(&format!("<ul><li><p>{spaces}text</p></li></ul>"), "- text\n", &options);
        check(
            &format!("<blockquote><p>{spaces}text</p></blockquote>"),
            "> text\n",
            &options,
        );
    }
}

#[test]
fn should_strip_a_strict_whitespace_only_paragraph_prefix_before_inline_content() {
    let options = ConversionOptions {
        whitespace_mode: WhitespaceMode::Strict,
        ..Default::default()
    };
    check("<p>      <b>text</b></p>", "**text**\n", &options);
    check("<ul><li><p>      <b>text</b></p></li></ul>", "- **text**\n", &options);
}
