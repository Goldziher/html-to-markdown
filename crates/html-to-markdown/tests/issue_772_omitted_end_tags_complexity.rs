#![allow(missing_docs)]

//! Issue #772: the second parse for an element with no end tag must stay linear in the size of
//! the page.
//!
//! The HTML tree builder searches its stack of open elements for most start tags, so a page that
//! never ends its blocks cost the square of its depth. The repair gives up past the depth a
//! browser builds, and the converter keeps the first parse. The library converts untrusted HTML,
//! so a quadratic page is a denial-of-service vector.

use std::fmt::Write;
use std::time::{Duration, Instant};

use html_to_markdown_rs::convert;

/// ~keep Same instrument as `list_item_complexity.rs`: linear code doubles its time when the
/// ~keep input doubles, quadratic code quadruples it, and 3.0 separates the two with room for
/// ~keep noise.
const MAX_DOUBLING_RATIO: f64 = 3.0;

/// ~keep A failure must reproduce on every independent attempt; one that does not is noise.
const MAX_MEASUREMENT_ATTEMPTS: usize = 3;

fn fastest_convert(html: &str, repeats: usize) -> Duration {
    (0..repeats)
        .map(|_| {
            let start = Instant::now();
            let result = convert(html, None);
            let elapsed = start.elapsed();
            assert!(result.is_ok(), "conversion must succeed");
            elapsed
        })
        .min()
        .expect("at least one run")
}

/// Convert `page(size)` at `base_size`, twice and four times that size, and fail when both
/// doublings look quadratic on every attempt.
fn assert_linear(name: &str, base_size: usize, repeats: usize, page: impl Fn(usize) -> String) {
    let pages = [page(base_size), page(base_size * 2), page(base_size * 4)];
    let mut failures = Vec::with_capacity(MAX_MEASUREMENT_ATTEMPTS);
    for attempt in 1..=MAX_MEASUREMENT_ATTEMPTS {
        let seconds: Vec<f64> = pages
            .iter()
            .map(|html| fastest_convert(html, repeats).as_secs_f64().max(1e-6))
            .collect();
        let first = seconds[1] / seconds[0];
        let second = seconds[2] / seconds[1];
        if first < MAX_DOUBLING_RATIO && second < MAX_DOUBLING_RATIO {
            return;
        }
        failures.push(format!(
            "attempt {attempt}: doublings took {first:.1}x and {second:.1}x as long ({:.4}s, {:.4}s, {:.4}s)",
            seconds[0], seconds[1], seconds[2]
        ));
    }
    panic!(
        "{name}: scaled super-linearly on every attempt, expected under {MAX_DOUBLING_RATIO}x for each doubling:\n{}",
        failures.join("\n")
    );
}

/// The page of the issue: `blocks` notes, each a paragraph with no end tag, and a footer.
fn note_page(blocks: usize) -> String {
    let mut notes = String::new();
    for number in 1..=blocks {
        write!(notes, "<div class='note'><p>Note {number} with some words in it.</div>").expect("write to a string");
    }
    format!("<!doctype html><html><body><main>{notes}</main><footer><p>Built with care.</p></footer></body></html>")
}

#[test]
fn should_convert_blocks_with_no_end_tag_that_nest_in_linear_time() {
    assert_linear("each block in the one before", 2_500, 3, |count| {
        format!("{}x", "<div>".repeat(count))
    });
    assert_linear("a paragraph in each block", 3_125, 3, |count| {
        "<div><p>x ".repeat(count)
    });
}

#[test]
fn should_convert_a_page_that_holds_five_hundred_blocks_open_in_linear_time() {
    // ~keep 500 open blocks stay under the limit of the repair, so the tree builder searches
    // ~keep all of them for each paragraph: the largest cost for one start tag that the repair
    // ~keep accepts.
    let held = "<div>".repeat(500);
    assert_linear("paragraphs in 500 open blocks", 6_250, 3, |count| {
        format!("{held}{}", "<p>x ".repeat(count))
    });
    // ~keep The blocks after the 500 go past the limit, so the open blocks before them count.
    assert_linear("a paragraph in each block in 500 open blocks", 3_125, 3, |count| {
        format!("{held}{}", "<div><p>x ".repeat(count))
    });
}

#[test]
fn should_convert_one_hundred_thousand_paragraphs_with_no_end_tag_in_linear_time() {
    assert_linear("paragraphs with no end tag", 25_000, 1, |count| "<p>x ".repeat(count));

    let result = convert(&"<p>x ".repeat(100_000), None).expect("conversion must succeed");
    let content = result.content.unwrap_or_default();
    assert_eq!(content.split_whitespace().count(), 100_000);
    assert!(result.warnings.is_empty(), "warnings: {:?}", result.warnings);
}

#[test]
fn should_convert_a_five_megabyte_page_of_paragraphs_with_no_end_tag_in_linear_time() {
    const BLOCKS: usize = 84_000;
    let page = note_page(BLOCKS);
    assert!(page.len() > 5_000_000, "{} bytes", page.len());

    assert_linear("the page of the issue", BLOCKS / 4, 1, note_page);

    let result = convert(&page, None).expect("conversion must succeed");
    let content = result.content.unwrap_or_default();
    assert!(content.starts_with("Note 1 with some words in it.\n\nNote 2 with"));
    assert!(content.ends_with(&format!("Note {BLOCKS} with some words in it.\n\nBuilt with care.\n")));
    assert_eq!(content.matches("\n\n").count(), BLOCKS);
    assert!(result.warnings.is_empty(), "warnings: {:?}", result.warnings);
}
