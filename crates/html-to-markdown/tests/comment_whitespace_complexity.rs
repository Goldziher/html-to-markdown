#![allow(missing_docs, clippy::print_stdout)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};
use std::time::{Duration, Instant};

fn fastest(count: usize, tier_strategy: TierStrategy) -> Duration {
    let html = format!("{}<b>x</b>", " <!-- c -->".repeat(count));
    (0..3)
        .map(|_| {
            let start = Instant::now();
            let content = convert(
                &html,
                ConversionOptions {
                    tier_strategy,
                    extract_metadata: false,
                    ..Default::default()
                },
            )
            .expect("comments convert")
            .content;
            assert_eq!(content.as_deref(), Some("**x**\n"));
            start.elapsed()
        })
        .min()
        .expect("three measurements")
}

#[test]
fn whitespace_before_comments_should_scale_linearly() {
    for tier in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let mut measurements = Vec::new();
        for _ in 0..3 {
            let small = fastest(1_000, tier).as_secs_f64();
            let medium = fastest(2_000, tier).as_secs_f64();
            let large = fastest(4_000, tier).as_secs_f64();
            measurements.push((small, medium, large));
            if medium < small * 3.0 && large < medium * 3.0 {
                break;
            }
        }
        println!("{tier:?}: {measurements:?}");
        assert!(
            measurements
                .iter()
                .any(|(small, medium, large)| *medium < small * 3.0 && *large < medium * 3.0),
            "{tier:?} rescanned comments: {measurements:?}"
        );
    }
}
