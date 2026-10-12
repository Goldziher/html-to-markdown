//! Regression coverage for a systemic Tier-1/Tier-2 divergence: a double space at the
//! boundary between adjacent bare inline elements (`<span>`, `<u>`, `<mark>`, ...) when a
//! whitespace-only sibling text node separates them -- the exact shape produced by
//! pretty-printed multi-`<span>` exports from Google Docs and `WordPress`.
//!
//! Root cause: Tier-2's generic per-text-node `skip_prefix` (`src/converter/text_node.rs`)
//! drops a text node's leading whitespace run to nothing when `output.ends_with(' ') &&
//! prefix == " "`, whatever the previous sibling is. Tier-1's `at_inline_frame_start` (in
//! `src/converter/tier1/scanner.rs`) already mirrors this for `<a>`/`<strong>`/`<em>`/
//! `<code>` -- kinds with their own always-on trim wrapper in Tier-2 (link-label
//! normalization, `chomp_inline`'s marker migration, code's verbatim path) -- but a bare
//! `TagKind::Inline` element (`<span>`, `<u>`, ...) has none of those; its children flow
//! through the same generic per-text-node path as top-level prose, so it needs Tier-2's
//! actual CONDITIONAL rule, not an unconditional strip. See
//! `bare_inline_frame_start_after_space` in `scanner.rs` for the fix. `<del>`/`<ins>`
//! (Strikethrough/Inserted) are deliberately excluded: Tier-2's own dedicated strike/ins
//! wrapper leaves a genuine double space in the equivalent shape (verified against
//! Tier-2 directly), so mirroring that divergence away would itself be wrong.
//!
//! Found via `test_documents/html/office-gdocs/gdocs-web-page-export.html`, whose
//! multi-`<span>` paragraph (lines 30-34) reproduces this exact shape.

#![cfg(feature = "testkit")]

use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::tier1;
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

fn tier1_friendly_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    }
}

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
fn adjacent_spans_separated_by_pretty_printed_whitespace_collapse_to_one_space() {
    let html = "<p><span>with </span>\n  <span>bold</span>\n  <span> emphasis</span></p>";
    assert_eq!(
        run_tier2(html),
        "with bold emphasis\n",
        "tier2 ground truth changed; update this test"
    );
    assert_eq!(run_tier1(html), "with bold emphasis\n");
}

#[test]
fn adjacent_u_elements_also_collapse_to_one_space() {
    let html = "<p><u>with </u>\n  <u>bold</u>\n  <u> emphasis</u></p>";
    assert_tier1_matches_tier2(html);
}

#[test]
fn adjacent_spans_with_no_separating_whitespace_keep_the_real_leading_space() {
    // ~keep No whitespace-only sibling between the tags means `output` does NOT already end
    // ~keep in a space when the third span opens, so its own leading space must survive.
    let html = "<p><span>a</span><span>bold</span><span> baz</span></p>";
    assert_eq!(
        run_tier2(html),
        "abold baz\n",
        "tier2 ground truth changed; update this test"
    );
    assert_eq!(run_tier1(html), "abold baz\n");
}

#[test]
fn adjacent_links_separated_by_pretty_printed_whitespace_collapse_to_one_space() {
    let html = "<p><a href=\"x\">with </a>\n  <a href=\"y\">bold</a>\n  <a href=\"z\"> emphasis</a></p>";
    assert_tier1_matches_tier2(html);
}

#[test]
fn strikethrough_and_inserted_collapse_to_one_space_too() {
    // ~keep White space on both sides of the start of an element is one run, so one space.
    let del_html = "<p><del>with </del>\n  <del>bold</del>\n  <del> emphasis</del></p>";
    assert_eq!(
        run_tier2(del_html),
        "~~with~~ ~~bold~~ ~~emphasis~~\n",
        "tier2 ground truth changed; update this test"
    );
    assert_tier1_matches_tier2(del_html);

    let ins_html = "<p><ins>with </ins>\n  <ins>bold</ins>\n  <ins> emphasis</ins></p>";
    assert_eq!(
        run_tier2(ins_html),
        "==with== ==bold== ==emphasis==\n",
        "tier2 ground truth changed; update this test"
    );
    assert_tier1_matches_tier2(ins_html);
}

#[test]
fn lone_leading_space_span_is_unaffected() {
    assert_tier1_matches_tier2("<p><span> lead</span></p>");
}

#[test]
fn plain_text_before_span_still_matches() {
    assert_tier1_matches_tier2("<p>a <span> b</span></p>");
    assert_tier1_matches_tier2("<p><span>a</span> <span> b</span></p>");
}
