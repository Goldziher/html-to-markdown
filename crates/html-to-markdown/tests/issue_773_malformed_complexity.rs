#![allow(missing_docs, clippy::print_stdout)]

//! ~keep Regression timing probes for repeated malformed markup and inline list fragments (#773).

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};
use std::time::{Duration, Instant};

fn input(shape: &str, count: usize) -> String {
    if shape == "list" {
        format!("<ul><li>{}</li></ul>", "<span>- t</span> ".repeat(count))
    } else if shape == "bogus" {
        format!("<p>a</p><?php x ?>{}", "<g a=\"".repeat(count))
    } else {
        format!("<p>a</p>{}", shape.repeat(count))
    }
}

fn fastest(html: &str) -> Duration {
    (0..2)
        .map(|_| {
            let start = Instant::now();
            let result = convert(
                html,
                Some(ConversionOptions {
                    extract_metadata: false,
                    tier_strategy: TierStrategy::Tier2,
                    ..Default::default()
                }),
            )
            .expect("malformed input must convert");
            let content = result.content.expect("text output exists");
            if html.starts_with("<ul>") {
                assert_eq!(content.matches("- t").count(), html.matches("<span>").count());
            } else {
                assert_eq!(content, "a\n");
            }
            start.elapsed()
        })
        .min()
        .expect("two samples")
}

#[test]
fn should_scale_linearly_for_repeated_unfinished_markup_and_inline_list_text() {
    let mut failures = Vec::new();
    for shape in ["<script ", "<style ", "<span hidden>", "<svg x=\"", "bogus", "list"] {
        let small = input(shape, if shape == "list" { 40_000 } else { 1_000 });
        let large = input(shape, if shape == "list" { 160_000 } else { 4_000 });
        let mut timings = Vec::new();
        for _ in 0..3 {
            let baseline = fastest(&small).max(Duration::from_millis(1));
            let elapsed = fastest(&large);
            timings.push((baseline, elapsed));
            if elapsed.as_secs_f64() <= baseline.as_secs_f64() * if shape == "list" { 6.0 } else { 8.0 } {
                break;
            }
        }
        println!("{shape:?}: {timings:?}");
        if !timings
            .iter()
            .any(|(small, large)| large.as_secs_f64() <= small.as_secs_f64() * if shape == "list" { 6.0 } else { 8.0 })
        {
            failures.push(format!("{shape:?}: {timings:?}"));
        }
    }
    assert!(failures.is_empty(), "repeated scans: {failures:?}");
}

#[test]
fn should_keep_visible_text_and_remove_closed_hidden_descendants_of_unclosed_parents() {
    let html = "<span hidden>a<span hidden>b</span>c";
    assert_eq!(
        convert(html, None).expect("conversion succeeds").content.as_deref(),
        Some("ac\n")
    );
}

#[test]
fn should_keep_visible_tail_after_hidden_self_closing_and_void_elements() {
    for html in ["<div hidden/>visible</div>", "<input hidden>visible</input>"] {
        assert_eq!(
            convert(html, None).expect("conversion succeeds").content.as_deref(),
            Some("visible\n")
        );
    }
}
