//! Regression coverage for a Tier-1/Tier-2 divergence on LEADING whitespace at
//! the start of an inline `<strong>`/`<em>` body.
//!
//! Tier-2's `chomp_inline` (utility/content.rs) moves a leading whitespace run
//! (including `&nbsp;`, other Unicode whitespace, and multi-character runs —
//! all folded to a single ASCII space by `normalize_whitespace` before
//! `chomp_inline` ever sees them) OUTSIDE the emphasis markers instead of
//! discarding it. Tier-1 used to discard it outright, because
//! `flush_text`'s `at_inline_frame_start` leading-whitespace strip (correct
//! for `<a>`, where `normalize_link_label` really does trim) applied the same
//! deletion to `<strong>`/`<em>` bodies. See `close_inline_marker` in
//! `src/converter/tier1/scanner.rs` for the migration mirror of the
//! already-existing trailing-whitespace migration (commit d438031143).

#![cfg(feature = "testkit")]

use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::tier1;
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

// ~keep Mirrors `tier1_friendly_options()` in `tests/tier_parity_corpus.rs`:
// ~keep clears every `router::classify` gate so a direct `tier1::run` call is
// ~keep exercising the same options a real `Auto`-dispatched call would pick
// ~keep Tier-1 for.
fn tier1_friendly_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    }
}

/// Runs the Tier-1 scanner directly (never through `convert()`'s Auto/Tier1
/// fallback dispatch) so a bail is a hard test failure, not a silent
/// fall-through to Tier-2 that would trivially "match" itself.
fn run_tier1(html: &str) -> String {
    let report = PrescanReport::default();
    match tier1::run(html, &report, &tier1_friendly_options()) {
        Ok(markdown) => markdown,
        Err(reason) => panic!("tier1 bailed on {html:?}: {reason:?}"),
    }
}

fn run_tier2(html: &str) -> String {
    let mut options = tier1_friendly_options();
    options.tier_strategy = TierStrategy::Tier2;
    convert(html, Some(options)).unwrap().content.unwrap_or_default()
}

fn assert_tier1_matches_tier2(html: &str) {
    let t1 = run_tier1(html);
    let t2 = run_tier2(html);
    assert_eq!(
        t1, t2,
        "tier1 diverged from tier2\ninput: {html:?}\ntier1: {t1:?}\ntier2: {t2:?}"
    );
}

#[test]
fn should_migrate_nbsp_entity_leading_whitespace_outside_em() {
    assert_tier1_matches_tier2("<p>a<em>&nbsp;x</em></p>");
}

#[test]
fn should_migrate_literal_nbsp_leading_whitespace_outside_em() {
    assert_tier1_matches_tier2("<p>a<em>\u{a0}x</em></p>");
}

#[test]
fn should_migrate_em_space_leading_whitespace_outside_em() {
    assert_tier1_matches_tier2("<p>a<em>\u{2003}x</em></p>");
}

#[test]
fn should_migrate_ideographic_space_leading_whitespace_outside_em() {
    assert_tier1_matches_tier2("<p>a<em>\u{3000}x</em></p>");
}

#[test]
fn should_migrate_leading_whitespace_at_start_of_em_body_with_no_preceding_text() {
    assert_tier1_matches_tier2("<p><em>\u{2003}x</em></p>");
}

#[test]
fn should_migrate_em_space_leading_whitespace_outside_strong_with_trailing_text() {
    assert_tier1_matches_tier2("<p>a<strong>\u{2003}x</strong>b</p>");
}

#[test]
fn should_collapse_multi_character_ascii_leading_run_to_single_space() {
    assert_tier1_matches_tier2("<p>a<em>   x</em></p>");
}

#[test]
fn should_collapse_mixed_newline_and_space_leading_run_to_single_space() {
    assert_tier1_matches_tier2("<p>a<em>\n  x</em></p>");
}

#[test]
fn should_collapse_mixed_unicode_whitespace_leading_run_to_single_space() {
    assert_tier1_matches_tier2("<p>a<em>&nbsp;\u{2003}x</em></p>");
}

#[test]
fn should_migrate_leading_whitespace_outside_nested_strong_and_em() {
    assert_tier1_matches_tier2("<p><strong><em>&nbsp;x</em></strong></p>");
}

#[test]
fn should_migrate_leading_whitespace_outside_nested_strong_and_em_at_document_start() {
    assert_tier1_matches_tier2("<strong><em>&nbsp;x</em></strong>");
}

#[test]
fn should_migrate_leading_whitespace_inside_list_item() {
    assert_tier1_matches_tier2("<ul><li>a<em>&nbsp;x</em></li></ul>");
}

#[test]
fn should_migrate_leading_whitespace_inside_ordered_list_item() {
    assert_tier1_matches_tier2("<ol><li>a<strong>&nbsp;x</strong></li></ol>");
}

#[test]
fn should_leave_leading_whitespace_untouched_inside_table_cell() {
    // ~keep Table cells are excluded from the leading-strip branch entirely
    // ~keep (`!state.in_table_cell()` guard in flush_text) and already matched
    // ~keep Tier-2 before this fix — kept here as a regression guard against
    // ~keep ever widening the migration into cells.
    assert_tier1_matches_tier2("<table><tr><td>a<em>&nbsp;x</em></td></tr></table>");
}

#[test]
fn should_migrate_leading_whitespace_inside_heading() {
    assert_tier1_matches_tier2("<h2>a<em>&nbsp;x</em></h2>");
}

#[test]
fn should_migrate_leading_whitespace_inside_blockquote() {
    assert_tier1_matches_tier2("<blockquote>a<em>&nbsp;x</em></blockquote>");
}

#[test]
fn should_migrate_leading_whitespace_at_document_start() {
    assert_tier1_matches_tier2("<p><em>&nbsp;x</em></p>");
}

#[test]
fn should_migrate_leading_whitespace_at_paragraph_end() {
    assert_tier1_matches_tier2("<p>a<em>&nbsp;x</em></p>");
}

#[test]
fn should_still_trim_link_label_leading_whitespace_across_child_element() {
    // ~keep Regression guard: the fix must not touch `<a>`'s leading-whitespace
    // ~keep deletion, which Tier-2's `normalize_link_label` genuinely performs
    // ~keep (no migration for links).
    assert_tier1_matches_tier2("<a href=\"x\">\n <span>EN</span>\n</a>");
}

#[test]
fn should_still_trim_link_label_leading_whitespace_direct_text() {
    assert_tier1_matches_tier2("<p><a href=\"https://x.example/\">   EN</a></p>");
}

#[test]
fn should_still_delete_leading_whitespace_between_strong_close_and_summary_child() {
    // ~keep Regression guard: the summary-accumulation arm of
    // ~keep `at_inline_frame_start` (Inline/Block/Paragraph/Heading kinds inside
    // ~keep `<summary>`) is untouched by this fix and still deletes rather than
    // ~keep migrates.
    assert_tier1_matches_tier2("<details><summary>a <span>&nbsp;x</span></summary></details>");
}

#[test]
fn should_keep_whitespace_only_text_between_leading_images_at_document_start() {
    let html = "<p><img src=\"/a.png\" alt=\"a\"> <img src=\"/b.png\" alt=\"b\"> dolor</p>";
    assert_tier1_matches_tier2(html);
    assert_eq!(run_tier1(html), "![a](/a.png) ![b](/b.png) dolor\n");
}

#[test]
fn should_keep_space_after_leading_image_inside_heading() {
    // ~keep Regression guard for the SPACE, not for the image: a heading's `<img>` is
    // ~keep flattened to bare alt text rather than going through the generic
    // ~keep `at_fresh_block_start` text-node pipeline, so it must still keep a real
    // ~keep separating space after that alt text. Both `document_start_strip` and
    // ~keep `document_start_drops_ws` must exclude heading context or they drop it.
    //
    // ~keep The expectation used to be `![a](/x.png) quick`, describing the flattening as a
    // ~keep Tier-1/Tier-2 divergence that was allow-listed and out of scope. Tier-1 now
    // ~keep flattens too (issue #494 removed the bail that hid the divergence), so both tiers
    // ~keep agree and the subject of this test -- the surviving space -- is unchanged.
    let html = "<h5><img src=\"/x.png\" alt=\"a\"> quick</h5>";
    let t1 = run_tier1(html);
    assert_eq!(t1, "##### a quick\n", "unexpected tier1 output for {html:?}");
}
