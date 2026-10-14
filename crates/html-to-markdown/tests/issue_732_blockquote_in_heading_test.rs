#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn convert_with(html: &str, tier_strategy: TierStrategy) -> String {
    let options = ConversionOptions {
        extract_metadata: false,
        tier_strategy,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

#[test]
fn should_keep_a_blockquote_inside_a_heading_apart_from_surrounding_text() {
    let html = "<h2>a<blockquote>x</blockquote>c</h2>";
    let expected = "## a x c\n";

    for tier_strategy in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
        assert_eq!(convert_with(html, tier_strategy), expected, "{tier_strategy:?}");
    }
}

#[test]
fn should_keep_a_blockquote_inside_a_list_heading_apart_from_surrounding_text() {
    let html = "<ol><li><h3>a<blockquote>x</blockquote>c</h3></li></ol>";
    let expected = "1. ### a x c\n";

    for tier_strategy in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
        assert_eq!(convert_with(html, tier_strategy), expected, "{tier_strategy:?}");
    }
}

#[test]
fn should_keep_a_blockquote_inside_a_table_heading_inline() {
    let html = "<table><tr><td><h2>a<blockquote>x</blockquote></h2></td></tr></table>";
    let expected = "| ax |\n| --- |\n";

    for tier_strategy in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
        assert_eq!(convert_with(html, tier_strategy), expected, "{tier_strategy:?}");
    }
}
