// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Form controls (issues 752, 757 and 776): the time for many controls in one parent grows in
//! proportion to their number. A check that reads all siblings of a control for each control makes
//! the time four times as long for each doubling.

use std::time::{Duration, Instant};

use html_to_markdown_rs::options::PreprocessingOptions;
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

/// The numbers of controls: three doublings.
const SIZES: [usize; 4] = [1_000, 2_000, 4_000, 8_000];
/// A linear conversion doubles its time. A quadratic one takes four times as long.
const MAX_DOUBLING_RATIO: f64 = 3.0;
/// A loaded machine gives a slow sample now and then, so a shape fails only when every attempt
/// is over the ratio.
const MAX_MEASUREMENT_ATTEMPTS: usize = 3;
const REPEATS_PER_SAMPLE: usize = 5;

const CHECKBOX: &str = "<input type=\"checkbox\">";

/// One input for each place where a control asks a question about its siblings: its name, the
/// source for `size` controls, and a text that the output has `count` times.
fn shapes(size: usize) -> [(&'static str, String, &'static str, usize); 7] {
    [
        (
            "checkboxes in labels alone in a table cell",
            format!(
                "<table><tr><th>State</th></tr><tr><td>{}</td></tr></table>",
                format!("<label><span>{CHECKBOX}</span></label>").repeat(size)
            ),
            "[ ]",
            size,
        ),
        (
            "checkboxes alone in a table cell",
            format!(
                "<table><tr><th>State</th></tr><tr><td>{}</td></tr></table>",
                CHECKBOX.repeat(size)
            ),
            "[ ]",
            size,
        ),
        (
            "checkboxes before a word in a table cell",
            format!(
                "<table><tr><th>State</th></tr><tr><td>{}word</td></tr></table>",
                CHECKBOX.repeat(size)
            ),
            "word",
            1,
        ),
        (
            "checkboxes between spaces in a paragraph",
            format!("<p>first {}second</p>", format!("{CHECKBOX} ").repeat(size)),
            "first second",
            1,
        ),
        (
            "checkboxes in a list item",
            format!("<ul><li>{}word</li></ul>", CHECKBOX.repeat(size)),
            "- [ ] word",
            1,
        ),
        (
            "buttons in an inline element",
            format!("<p><span>{}</span></p>", "<button>Go</button>".repeat(size)),
            "Go",
            size,
        ),
        (
            "inputs in a label",
            format!("<p><label>{}word</label></p>", "<input type=\"text\"> ".repeat(size)),
            "word",
            1,
        ),
    ]
}

fn fastest_convert(html: &str, tier_strategy: TierStrategy, needle: &str, count: usize) -> Duration {
    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        tier_strategy,
        preprocessing: PreprocessingOptions {
            remove_forms: false,
            ..PreprocessingOptions::default()
        },
        ..ConversionOptions::default()
    };
    (0..REPEATS_PER_SAMPLE)
        .map(|_| {
            let start = Instant::now();
            let output = convert(std::hint::black_box(html), Some(options.clone()))
                .expect("conversion must succeed")
                .content
                .unwrap_or_default();
            let elapsed = start.elapsed();
            assert_eq!(output.matches(needle).count(), count, "{needle:?} in the output");
            elapsed
        })
        .min()
        .expect("at least one timing sample")
}

/// The time of each size and the largest ratio of a doubling.
fn doubling_ratios(index: usize, tier_strategy: TierStrategy) -> (Vec<f64>, f64) {
    let seconds: Vec<f64> = SIZES
        .iter()
        .map(|size| {
            let (_, html, needle, count) = &shapes(*size)[index];
            fastest_convert(html, tier_strategy, needle, *count)
                .as_secs_f64()
                .max(1e-6)
        })
        .collect();
    let worst = seconds.windows(2).map(|pair| pair[1] / pair[0]).fold(0.0, f64::max);
    (seconds, worst)
}

fn measure(index: usize, tier_strategy: TierStrategy) -> Result<(), String> {
    let mut attempts = Vec::with_capacity(MAX_MEASUREMENT_ATTEMPTS);
    for _ in 0..MAX_MEASUREMENT_ATTEMPTS {
        let (seconds, worst) = doubling_ratios(index, tier_strategy);
        if worst < MAX_DOUBLING_RATIO {
            return Ok(());
        }
        attempts.push(format!("{seconds:.4?} (largest ratio {worst:.2})"));
    }
    Err(format!(
        "{} with {tier_strategy:?}: {}",
        shapes(1)[index].0,
        attempts.join("; ")
    ))
}

#[test]
fn should_convert_many_controls_in_one_parent_in_linear_time() {
    let failures: Vec<String> = (0..shapes(1).len())
        .flat_map(|index| [TierStrategy::Tier2, TierStrategy::Tier1].map(|tier_strategy| (index, tier_strategy)))
        .filter_map(|(index, tier_strategy)| measure(index, tier_strategy).err())
        .collect();
    assert!(
        failures.is_empty(),
        "the time for a doubling of the controls must stay under {MAX_DOUBLING_RATIO} times:\n{}",
        failures.join("\n")
    );
}
