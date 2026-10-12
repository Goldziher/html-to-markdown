#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, PreprocessingOptions, TierStrategy, convert};

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
fn should_separate_heading_block_children_from_following_text() {
    for block in ["p", "section", "div"] {
        check(
            &format!("<h2><{block}>One</{block}>items</h2>"),
            "## One items\n",
            &ConversionOptions::default(),
        );
        check(
            &format!("<h2>before<{block}>One</{block}>items</h2>"),
            "## before One items\n",
            &ConversionOptions::default(),
        );
    }
}

#[test]
fn should_separate_preserved_form_content_inside_headings() {
    let options = ConversionOptions {
        preprocessing: PreprocessingOptions {
            remove_forms: false,
            ..PreprocessingOptions::default()
        },
        ..ConversionOptions::default()
    };
    check("<h2><form>One</form>items</h2>", "## One items\n", &options);
    check(
        "<h2>before<form>One</form>items</h2>",
        "## before One items\n",
        &options,
    );
    check(
        "<h2><form>One</form>items</h2>",
        "## items\n",
        &ConversionOptions::default(),
    );
}
