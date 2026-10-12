//! Positive-trigger coverage for the Tier-1 bails added when Tier-2's `emit_wrapped_inline`
//! started merging delimiter runs and preserving whitespace-only bodies for every inline
//! family, not just `<strong>`/`<em>`:
//!
//!   - `AdjacentInlineEmphasis` for the `~~`, `==` and backtick delimiters
//!   - `WhitespaceOnlyInlineEmphasis` for `~~`, `==` and code spans
//!   - `InlineMarkerNotReproduced` for `<q>` and `<mark>`
//!
//! Each `tier1_run` call goes straight through `tier1::run`, bypassing `convert()`'s
//! Auto-strategy classifier, so a bail is a hard `Err` here rather than a silent fall-through
//! to Tier-2 that would trivially match itself.

#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn tier1_run(html: &str) -> Result<String, BailReason> {
    let (cleaned, report) = prescan::run(html);
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    tier1::run(cleaned.as_ref(), &report, &options)
}

/// Auto dispatch, proving the bail routes to Tier-2 end-to-end rather than merely
/// returning `Err` from a direct `tier1::run` call.
fn auto(html: &str) -> String {
    convert(
        html,
        Some(ConversionOptions {
            extract_metadata: false,
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed")
    .content
    .unwrap_or_default()
    .trim()
    .to_owned()
}

fn assert_bails_with(html: &str, expected: &BailReason) {
    match tier1_run(html) {
        Err(reason) => assert_eq!(
            std::mem::discriminant(&reason),
            std::mem::discriminant(expected),
            "{html} bailed with {reason} instead of the expected variant"
        ),
        Ok(output) => panic!("{html} did not bail; tier-1 produced {output:?}"),
    }
}

#[test]
fn should_bail_on_adjacent_strikethrough_and_emphasis_delimiter_runs() {
    for html in [
        "<p><del>A</del><del>B</del></p>",
        "<p><s>A</s><s>B</s></p>",
        "<p><strike>A</strike><strike>B</strike></p>",
        "<p><var>A</var><var>B</var></p>",
        "<p><dfn>A</dfn><dfn>B</dfn></p>",
    ] {
        assert_bails_with(html, &BailReason::AdjacentInlineEmphasis);
    }
}

#[test]
fn should_bail_on_adjacent_code_spans() {
    for html in [
        "<p><code>A</code><code>B</code></p>",
        "<p><kbd>A</kbd><kbd>B</kbd></p>",
        "<p><samp>A</samp><samp>B</samp></p>",
    ] {
        assert_bails_with(html, &BailReason::AdjacentInlineEmphasis);
    }
}

#[test]
fn should_bail_on_a_whitespace_only_strikethrough_or_code_body() {
    for html in [
        "<p>A<del> </del>B</p>",
        "<p>A<s> </s>B</p>",
        "<p>A<strike> </strike>B</p>",
        "<p>A<code> </code>B</p>",
        "<p>A<kbd> </kbd>B</p>",
        "<p>A<samp> </samp>B</p>",
    ] {
        assert_bails_with(html, &BailReason::WhitespaceOnlyInlineEmphasis);
    }
}

#[test]
fn should_bail_on_inline_elements_whose_markers_tier1_never_emits() {
    for html in ["<p><q>x</q></p>", "<p><mark>x</mark></p>"] {
        assert_bails_with(html, &BailReason::InlineMarkerNotReproduced);
    }
}

#[test]
fn should_not_bail_on_the_same_shapes_when_a_real_separator_intervenes() {
    // ~keep Control: these are exactly the inputs above with one space added, which removes
    // ~keep the adjacency. A bail here would mean the trigger is far broader than intended.
    for html in [
        "<p><del>A</del> <del>B</del></p>",
        "<p><ins>A</ins> <ins>B</ins></p>",
        "<p><code>A</code> <code>B</code></p>",
        "<p><var>A</var> <var>B</var></p>",
    ] {
        assert!(
            tier1_run(html).is_ok(),
            "{html} bailed even though its delimiters are not adjacent"
        );
    }
}

#[test]
fn should_reach_tier2_output_through_auto_dispatch_for_every_bailed_shape() {
    // ~keep The bail is only useful if Auto actually falls back and produces Tier-2's answer.
    assert_eq!(auto("<p><del>A</del><del>B</del></p>"), "~~AB~~");
    assert_eq!(auto("<p><ins>A</ins><ins>B</ins></p>"), "AB");
    assert_eq!(auto("<p><code>A</code><code>B</code></p>"), "`AB`");
    assert_eq!(auto("<p>A<del> </del>B</p>"), "A B");
    assert_eq!(auto("<p>A<code> </code>B</p>"), "A` `B");
    assert_eq!(auto("<p><q>x</q></p>"), "\"x\"");
    assert_eq!(auto("<p><mark>x</mark></p>"), "==x==");
}
