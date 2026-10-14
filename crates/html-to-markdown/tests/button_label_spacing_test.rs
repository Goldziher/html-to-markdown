#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

fn convert_as(html: &str, output_format: OutputFormat, tier_strategy: TierStrategy) -> String {
    let options = ConversionOptions {
        output_format,
        tier_strategy,
        ..Default::default()
    };
    convert(html, Some(options)).unwrap().content.unwrap_or_default()
}

/// Markdown output on each route, and Djot and plain output, write `expected`.
fn assert_each_format(html: &str, expected: &str) {
    for (output_format, tier_strategy) in [
        (OutputFormat::Markdown, TierStrategy::Auto),
        (OutputFormat::Markdown, TierStrategy::Tier1),
        (OutputFormat::Markdown, TierStrategy::Tier2),
        (OutputFormat::Djot, TierStrategy::Auto),
        (OutputFormat::Plain, TierStrategy::Auto),
    ] {
        assert_eq!(
            convert_as(html, output_format, tier_strategy),
            expected,
            "{html} {output_format:?} {tier_strategy:?}"
        );
    }
}

#[test]
fn should_separate_buttons_that_stand_side_by_side() {
    assert_each_format(
        "<p><button>All Methods</button><button>Instance Methods</button><button>Concrete Methods</button></p>",
        "All Methods Instance Methods Concrete Methods\n",
    );
}

#[test]
fn should_separate_a_button_from_the_words_beside_it() {
    assert_each_format("<p>Press<button>Save</button>now.</p>", "Press Save now.\n");
}

#[test]
fn should_keep_the_white_space_of_the_source_around_a_button() {
    assert_each_format("<p>Press <button>Save</button> now.</p>", "Press Save now.\n");
}

#[test]
fn should_write_no_space_before_punctuation_after_a_button() {
    assert_each_format("<p>Press <button>Save</button>.</p>", "Press Save.\n");
}
