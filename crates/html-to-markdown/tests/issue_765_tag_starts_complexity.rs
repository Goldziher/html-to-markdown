#![allow(missing_docs)]

//! Issue #765: a page with many tag starts before one `>` took quadratic time, because the pass
//! that removes hidden elements read the same long tag again from every `<` inside it.
//!
//! The bound is on the growth between two sizes, not on a time: four times the input may take
//! at most eight times as long. A linear pass takes about four times, the old pass sixteen.

use std::time::{Duration, Instant};

use html_to_markdown_rs::convert;

const SMALL: usize = 5_000;
const GROWTH: usize = 4;
const MAX_RATIO: f64 = 8.0;
// ~keep One slow sample on a loaded machine is noise; a quadratic pass is slow in every attempt.
const ATTEMPTS: usize = 3;
const REPEATS: usize = 3;

fn fastest(html: &str) -> Duration {
    (0..REPEATS)
        .map(|_| {
            let start = Instant::now();
            let result = convert(html, None);
            let elapsed = start.elapsed();
            assert!(result.is_ok(), "the conversion failed");
            elapsed
        })
        .min()
        .expect("at least one repeat")
}

fn assert_linear(name: &str, page: impl Fn(usize) -> String) {
    let (small, large) = (page(SMALL), page(SMALL * GROWTH));
    let mut seen = Vec::new();
    for _ in 0..ATTEMPTS {
        let (fast, slow) = (fastest(&small), fastest(&large));
        let ratio = slow.as_secs_f64() / fast.as_secs_f64().max(1e-6);
        if ratio <= MAX_RATIO {
            return;
        }
        seen.push(format!("{fast:?} -> {slow:?} ({ratio:.1}x)"));
    }
    panic!(
        "{name}: {GROWTH} times the input took more than {MAX_RATIO} times as long in every attempt: {}",
        seen.join("; ")
    );
}

#[test]
fn should_convert_tag_starts_with_no_end_in_a_graphic_in_linear_time() {
    assert_linear("tag starts with no end", |count| {
        format!("<p>a</p><svg>{}</svg><p>z</p>", "<a ".repeat(count))
    });
}

#[test]
fn should_convert_attribute_values_with_no_closing_quote_in_a_graphic_in_linear_time() {
    assert_linear("attribute values with no closing quote", |count| {
        format!("<p>a</p><svg>{}</svg><p>z</p>", "<g a=\"".repeat(count))
    });
}

#[test]
fn should_convert_tag_starts_with_no_end_outside_a_graphic_in_linear_time() {
    assert_linear("tag starts with no end in a section", |count| {
        format!("<p>a</p><section>{}</section><p>z</p>", "<a ".repeat(count))
    });
}

#[test]
fn should_convert_tag_starts_before_a_quote_with_no_partner_in_linear_time() {
    // ~keep No scan from these starts finds an end: each one stops at the last quote.
    assert_linear("tag starts, then one quote", |count| {
        format!("<p>a</p><svg>{}\"</svg><p>z</p>", "<a ".repeat(count))
    });
    assert_linear("paired quotes, then one quote of the other kind", |count| {
        format!("<p>a</p><svg>{}\"</svg><p>z</p>", "<a ''".repeat(count / 4))
    });
    // ~keep No `>` in the whole page: the pass had a guard of its own for this, which is gone.
    assert_linear("tag starts and no end in the page", |count| "<s ".repeat(count));
}

#[test]
fn should_still_remove_a_hidden_element_after_a_tag_that_holds_tag_starts() {
    for (html, expected) in [
        (
            "<p>a</p><div <span>shown</div><p hidden>gone</p><p>z</p>",
            "a\n\nshown\n\nz\n",
        ),
        (
            r#"<p title="<b hidden>">kept words</p><p style="display:none">gone</p><p>z</p>"#,
            "kept words\n\nz\n",
        ),
    ] {
        let markdown = convert(html, None).expect("conversion").content.unwrap_or_default();
        assert_eq!(markdown, expected, "{html}");
    }
}
