#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #497: a `<br>` at the very start or end of an `<a>` label was
//! silently dropped.
//!
//! `<a href="H">A<br></a>B` renders as A, a line break, then B. The markdown that re-parses
//! to exactly that HTML is `[A  \n](H)B`, whose label ends in a hard break -- verified with
//! comrak, which renders it back to `<p><a href="H">A<br /></a>B</p>`. The converter emitted
//! `[A](H)B` instead, losing the break.
//!
//! Both tiers dropped it, for the same reason expressed twice: the whole label was
//! whitespace-trimmed, and once flattened a `"  \n"` marker is indistinguishable from the
//! incidental whitespace that really does belong before a `</a>`. Tier-2 additionally never
//! emitted a *leading* break at all, because `line_break.rs`'s "nothing on this line yet"
//! test compares a fresh label buffer's length against the enclosing block's start offset.
//!
//! The reporter's second example asks for `B[A  \n](H)`, which moves the break to the far
//! side of the label text. The break is preserved where the `<br>` actually was instead:
//! `B[  \nA](H)`, which comrak renders back to `B<a href="H"><br />A</a>` -- the input DOM.

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn content_with(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn tier1_run(html: &str, options: &ConversionOptions) -> Result<String, BailReason> {
    let (cleaned, report) = prescan::run(html);
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..options.clone()
    };
    tier1::run(cleaned.as_ref(), &report, &options)
}

fn assert_tiers_agree(html: &str, expected: &str) {
    assert_tiers_agree_with(html, &ConversionOptions::default(), expected);
}

fn assert_tiers_agree_with(html: &str, options: &ConversionOptions, expected: &str) {
    assert_eq!(
        content_with(html, options.clone()),
        expected,
        "tier-2 output for {html:?}"
    );
    match tier1_run(html, options) {
        Ok(tier1) => assert_eq!(tier1, expected, "tier-1 output for {html:?} must match tier-2"),
        Err(reason) => panic!("tier-1 must not bail on {html:?}: {reason:?}"),
    }
}

// ~keep ── The reporter's two cases ───────────────────────────────────────────────────

#[test]
fn should_keep_a_break_between_the_label_text_and_the_closing_anchor() {
    assert_tiers_agree(r#"<a href="H">A<br></a>B"#, "[A  \n](H)B\n");
}

#[test]
fn should_keep_a_break_between_the_opening_anchor_and_the_label_text() {
    assert_tiers_agree(r#"B<a href="H"><br>A</a>"#, "B[  \nA](H)\n");
}

#[test]
fn should_keep_a_boundary_break_when_the_link_is_the_whole_paragraph() {
    assert_tiers_agree(r#"<p><a href="H">A<br></a></p>"#, "[A  \n](H)\n");
    assert_tiers_agree(r#"<p><a href="H"><br>A</a></p>"#, "[  \nA](H)\n");
}

#[test]
fn should_keep_a_boundary_break_past_incidental_whitespace_at_the_label_edge() {
    assert_tiers_agree(r#"<a href="H">A<br> </a>"#, "[A  \n](H)\n");
    assert_tiers_agree(r#"<a href="H"> <br>A</a>"#, "[  \nA](H)\n");
}

// ~keep A run of breaks at one edge collapses to a single break: two adjacent markers put a
// ~keep BLANK line in the label, and a blank line ends the paragraph the link lives in --
// ~keep which would destroy the link outright rather than preserve a second, invisible break.
#[test]
fn should_collapse_a_run_of_boundary_breaks_to_one() {
    assert_tiers_agree(r#"<a href="H"><br><br>A</a>"#, "[  \nA](H)\n");
    assert_tiers_agree(r#"<a href="H">A<br><br></a>"#, "[A  \n](H)\n");
}

#[test]
fn should_keep_a_break_at_both_edges_of_the_same_label() {
    assert_tiers_agree(r#"x<a href="H"><br>A<br></a>y"#, "x[  \nA  \n](H)y\n");
}

#[test]
fn should_keep_a_boundary_break_beside_an_inline_image_in_the_label() {
    assert_tiers_agree(
        r#"<a href="H"><img src="i.png" alt="a"><br></a>"#,
        "[![a](i.png)  \n](H)\n",
    );
}

// ~keep ── CONTROL cases ───────────────────────────────────────────────────────────────────

// ~keep A break-only label still collapses to empty; Tier-1 now delegates its fallback
// ~keep to Tier-2, preserving boundary normalization while eliminating label divergence (#774).
#[test]
fn should_still_drop_a_label_that_is_nothing_but_breaks() {
    assert_eq!(
        content_with(r#"<a href="H"><br></a>"#, ConversionOptions::default()),
        "[H](H)\n"
    );
    assert!(matches!(
        tier1_run(r#"<a href="H"><br></a>"#, &ConversionOptions::default()),
        Err(BailReason::LinkEmptyLabel)
    ));
}

#[test]
fn should_leave_a_mid_label_break_exactly_as_it_was() {
    assert_tiers_agree(r#"<a href="H">A<br>B</a>"#, "[A  \nB](H)\n");
    assert_tiers_agree(r#"<a href="H">A<br> B</a>"#, "[A  \nB](H)\n");
}

#[test]
fn should_leave_a_label_without_any_break_exactly_as_it_was() {
    assert_tiers_agree(r#"<a href="H"> A </a>"#, "[A](H)\n");
    assert_tiers_agree(r#"<a href="H">A</a>B"#, "[A](H)B\n");
}

// ~keep A single-line ATX heading and a pipe-table cell cannot carry a hard break at all, so
// ~keep a `<br>` anywhere inside a link in either still folds to a space. Without this the
// ~keep preserved marker survives as a stray trailing space inside the brackets.
#[test]
fn should_still_fold_a_boundary_break_inside_a_heading() {
    assert_tiers_agree(r#"<h1><a href="H">A<br></a></h1>"#, "# [A](H)\n");
    assert_tiers_agree(r#"<h2><a href="H"><br>A</a></h2>"#, "## [A](H)\n");
}

#[test]
fn should_still_fold_a_boundary_break_inside_a_table_cell() {
    assert_tiers_agree(
        r#"<table><tr><td><a href="H">A<br></a></td></tr></table>"#,
        "| [A](H) |\n| ------ |\n",
    );
}

// ~keep `br_in_tables` inside a LINK inside a cell is a pre-existing tier divergence, not
// ~keep one this change introduced: Tier-2 tests `in_table_cell` ahead of the link arm in
// ~keep `line_break.rs` and so emits the literal `<br>`, while Tier-1's `TagKind::LineBreak`
// ~keep tests the link frame first and emits its marker, which the cell fold later turns into
// ~keep a space. Both are unchanged by #497 -- asserted per tier rather than through
// ~keep `assert_tiers_agree` so this file does not claim a parity that never held.
#[test]
fn should_keep_the_pre_existing_br_in_tables_behaviour_for_a_link_inside_a_cell() {
    let html = r#"<table><tr><td><a href="H">A<br>B</a></td></tr></table>"#;
    let options = ConversionOptions {
        br_in_tables: true,
        ..ConversionOptions::default()
    };
    assert_eq!(
        content_with(html, options.clone()),
        "| [A<br>B](H) |\n| ----------- |\n"
    );
    assert_eq!(
        tier1_run(html, &options).as_deref().ok(),
        Some("| [A B](H) |\n| -------- |\n")
    );
}

#[test]
fn should_drop_a_paragraph_leading_break_outside_a_link() {
    assert_tiers_agree("<p>x<br></p>", "x  \n");
    assert_tiers_agree("<p><br>x</p>", "x\n");
}

// ~keep A code SPAN inside a link splits into one backtick span per segment with the marker
// ~keep OUTSIDE the backticks (issue #487); that path must keep working unchanged.
#[test]
fn should_leave_a_break_inside_a_code_span_in_a_link_exactly_as_it_was() {
    assert_tiers_agree(r#"<a href="H"><code>A<br>B</code></a>"#, "[`A`  \n`B`](H)\n");
}
