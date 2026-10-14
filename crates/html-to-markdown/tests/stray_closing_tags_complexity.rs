#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression test for issue #715: a run of unmatched closing tags before a `<br>` must scale
//! linearly rather than re-scan the remaining run for every tag.

use std::time::{Duration, Instant};

use html_to_markdown_rs::options::HighlightStyle;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

const BASE_SIZE: usize = 1_000;
const MAX_DOUBLING_RATIO: f64 = 3.0;
const MAX_MEASUREMENT_ATTEMPTS: usize = 3;
const SAMPLES_PER_SIZE: usize = 7;
const MIN_BATCH_DURATION: Duration = Duration::from_millis(20);
const MAX_BATCH_REPEATS: usize = 128;

fn input(size: usize) -> String {
    format!("a\n{}<br>b", "</i>\n".repeat(size))
}

fn timed_batch(html: &str, options: &ConversionOptions, repeats: usize) -> Duration {
    let prepared_options = vec![options.clone(); repeats];
    let mut outputs = Vec::with_capacity(repeats);
    let start = Instant::now();
    for options in prepared_options {
        outputs.push(
            convert(std::hint::black_box(html), Some(options))
                .expect("conversion must succeed")
                .content
                .unwrap_or_default(),
        );
    }
    let elapsed = start.elapsed();
    assert_eq!(outputs.len(), repeats);
    for output in outputs {
        assert_eq!(output, "a  \nb\n");
    }
    elapsed
}

fn batched_sample(html: &str, options: &ConversionOptions) -> f64 {
    let mut repeats = 1;
    loop {
        let elapsed = timed_batch(html, options, repeats);
        if elapsed >= MIN_BATCH_DURATION || repeats == MAX_BATCH_REPEATS {
            let count = u32::try_from(repeats).expect("batch count fits u32");
            return elapsed.as_secs_f64() / f64::from(count);
        }
        repeats *= 2;
    }
}

fn measurement(tier_strategy: TierStrategy) -> (f64, f64, f64) {
    let inputs = [input(BASE_SIZE), input(BASE_SIZE * 2), input(BASE_SIZE * 4)];
    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        tier_strategy,
        ..ConversionOptions::default()
    };
    for html in &inputs {
        let _ = timed_batch(html, &options, 1);
    }
    let mut fastest = [f64::INFINITY; 3];
    // ~keep Coverage and scheduler pauses distort isolated millisecond samples; rotate
    // size order and compare minima from batches with enough conversion work.
    for round in 0..SAMPLES_PER_SIZE {
        for offset in 0..inputs.len() {
            let index = (round + offset) % inputs.len();
            fastest[index] = fastest[index].min(batched_sample(&inputs[index], &options));
        }
    }
    fastest.into()
}

fn scales_linearly(small: f64, medium: f64, large: f64) -> bool {
    medium / small < MAX_DOUBLING_RATIO && large / medium < MAX_DOUBLING_RATIO
}

fn assert_linear(tier_strategy: TierStrategy) {
    let mut failures = Vec::with_capacity(MAX_MEASUREMENT_ATTEMPTS);
    for attempt in 1..=MAX_MEASUREMENT_ATTEMPTS {
        let (small, medium, large) = measurement(tier_strategy);
        let first_ratio = medium / small;
        let second_ratio = large / medium;
        if scales_linearly(small, medium, large) {
            return;
        }
        failures.push(format!(
            "attempt {attempt}: {small:.4}s -> {medium:.4}s -> {large:.4}s ({first_ratio:.2}x, {second_ratio:.2}x)"
        ));
    }
    panic!(
        "{tier_strategy:?} exceeded the doubling ratio across every measurement:\n{}",
        failures.join("\n")
    );
}

#[test]
fn stray_closing_tags_before_a_break_scale_linearly_for_both_tier_strategies() {
    assert_linear(TierStrategy::Tier1);
    assert_linear(TierStrategy::Tier2);
}

#[test]
fn scaling_gate_accepts_linear_work_and_rejects_quadratic_work() {
    assert!(scales_linearly(1.0, 2.0, 4.0));
    assert!(!scales_linearly(1.0, 4.0, 16.0));
    assert!(!scales_linearly(1.0, 3.0, 6.0));
    assert!(!scales_linearly(1.0, 2.0, 6.0));
}
