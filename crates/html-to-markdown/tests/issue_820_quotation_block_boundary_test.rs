#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn assert_all_tiers(html: &str, expected: &str) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let options = ConversionOptions {
            tier_strategy,
            ..ConversionOptions::default()
        };
        let output = convert(html, Some(options))
            .expect("conversion should succeed")
            .content
            .unwrap_or_default();
        assert_eq!(output, expected, "unexpected output for {tier_strategy:?}: {html}");
    }
}

#[test]
fn should_separate_a_quoted_paragraph_from_preceding_text() {
    for html in ["<div>one <q><p>y</p></q></div>", "<div>one\n<q><p>y</p></q></div>"] {
        assert_all_tiers(html, "one\n\n\"y\"\n");
    }
}

#[test]
fn should_separate_a_quoted_paragraph_from_following_text() {
    assert_all_tiers("<div>one <q><p>y</p></q>two</div>", "one\n\n\"y\"\n\ntwo\n");
}

#[test]
fn should_keep_an_ordinary_quotation_inline() {
    assert_all_tiers("<div>one <q>y</q> two</div>", "one \"y\" two\n");
}

#[test]
fn should_keep_a_quotation_with_inline_formatting_inline() {
    assert_all_tiers("<div>one <q><em>y</em></q> two</div>", "one \"*y*\" two\n");
}

#[test]
fn should_match_a_paragraph_inside_a_span() {
    assert_all_tiers("<div>one <span><p>y</p></span></div>", "one\n\ny\n");
}

#[test]
fn should_keep_an_empty_quoted_paragraph_empty() {
    assert_all_tiers("<div>one <q><p></p></q>two</div>", "one two\n");
}

#[test]
fn should_render_a_standalone_quoted_paragraph_without_leading_space() {
    assert_all_tiers("<q><p>y</p></q>", "\"y\"\n");
}

#[test]
fn should_separate_a_quoted_paragraph_with_a_following_inline_tail() {
    assert_all_tiers("<div>one <q><p>y</p>z</q></div>", "one\n\n\"y\n\nz\"\n");
}

#[test]
fn should_keep_a_quotation_in_a_heading_inline() {
    assert_all_tiers("<h1>one <q>y</q> two</h1>", "# one \"y\" two\n");
}
