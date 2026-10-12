//! HTML inert elements ignore self-closing syntax. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

#[test]
fn should_hide_literal_inert_body_after_a_self_closing_html_opener() {
    for tag in ["noscript", "iframe", "template"] {
        for opener_suffix in ["/>", " />"] {
            let html = format!("<p>before</p><{tag}{opener_suffix}hidden</{tag}><p>after</p>");
            for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
                let result = convert(
                    &html,
                    ConversionOptions {
                        tier_strategy,
                        ..ConversionOptions::default()
                    },
                )
                .expect("convert self-closing inert HTML");
                assert_eq!(
                    result.content.as_deref(),
                    Some("before\n\nafter\n"),
                    "{tier_strategy:?}: {html}"
                );
            }
        }
    }
}
