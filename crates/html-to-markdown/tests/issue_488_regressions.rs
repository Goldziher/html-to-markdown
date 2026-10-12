#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #488: a `<div>` wrapping a nested `<table>` inside a data
//! table's single-cell row bypassed the issue #484 deferral, the issue #469 pipe escape,
//! and `fold_nested_table_rows` entirely, because `render_cell_text` tested each direct
//! child of the cell with a single-node tag-name equality check instead of searching the
//! child's subtree for a nested table. The `<div>` took the `else` branch and rendered the
//! inner table's row/separator syntax straight into the cell buffer as raw, unescaped
//! pipes — real content loss, since the inner table's `|` delimiters read as the outer
//! row's cell boundaries on reparse.

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn content(html: &str) -> String {
    convert(html, Some(ConversionOptions::default()))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn content_with(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn tier1_run(html: &str, options: &ConversionOptions) -> Result<String, BailReason> {
    let (cleaned, report) = prescan::run(html);
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..options.clone()
    };
    tier1::run(cleaned.as_ref(), &report, &opts)
}

fn compact_options() -> ConversionOptions {
    ConversionOptions {
        br_in_tables: true,
        compact_tables: true,
        ..ConversionOptions::default()
    }
}

// ~keep The unwrapped issue #484 fixture, reused here as the control the div-wrapped shape
// ~keep must match once the div is transparent to nested-table detection.
const ISSUE_484_HTML: &str = "<table><tr><th>Summary</th></tr><tr><td>Before<table><tr><th>Item</th>\
<th>Amount</th></tr><tr><td>Alpha</td><td>10</td></tr></table></td></tr></table>";

const ISSUE_488_DIV_HTML: &str = "<table><tr><th>Summary</th></tr><tr><td>Before<div><table><tr><th>Item</th>\
<th>Amount</th></tr><tr><td>Alpha</td><td>10</td></tr></table></div></td></tr></table>";

const ISSUE_488_TWO_LEVELS_HTML: &str = "<table><tr><th>Summary</th></tr><tr><td>Before<div><section><table><tr>\
<th>Item</th><th>Amount</th></tr><tr><td>Alpha</td><td>10</td></tr></table></section></div></td></tr></table>";

const ISSUE_488_WRAPPER_WITH_CONTENT_HTML: &str = "<table><tr><th>Summary</th></tr><tr><td>Before<div>text<table>\
<tr><th>Item</th><th>Amount</th></tr><tr><td>Alpha</td><td>10</td></tr></table>more</div></td></tr></table>";

const ISSUE_469_SIBLING_HTML: &str =
    "<table><tr><td><table><tr><td>inner</td></tr></table></td><td>Other</td></tr></table>";

const ISSUE_488_DIV_SIBLING_HTML: &str =
    "<table><tr><td><div><table><tr><td>inner</td></tr></table></div></td><td>Other</td></tr></table>";

const ISSUE_488_DIV_WRAPPER_HTML: &str =
    "<table><tr><td><div><table><tr><td>x</td></tr></table></div></td></tr></table>";

const ISSUE_488_NO_TABLE_WRAPPER_HTML: &str = "<table><tr><td>Before<div>text</div></td></tr></table>";

#[test]
fn should_render_a_div_wrapped_nested_table_separately() {
    // ~keep Structure only, per the padding caveat below -- byte-equality with the unwrapped
    // ~keep control is only guaranteed under compact_tables (should_match_the_unwrapped_-
    // ~keep nested_table_byte_for_byte), since default padding measures the wrapper's own
    // ~keep rendered width via the issue #406 measure_width_only short-circuit.
    let out = content(ISSUE_488_DIV_HTML);
    let table_count = out.matches("| ---").count() + out.matches("| ----").count();
    assert!(
        out.contains("Item") && out.contains("Amount") && out.contains("Alpha") && out.contains("10"),
        "div-wrapped inner table content lost: {out:?}"
    );
    assert!(
        !out.contains(r"\|"),
        "no pipe should need escaping once the div-wrapped inner table is separate: {out:?}"
    );
    assert_eq!(
        out.matches("\n\n").count(),
        1,
        "expected exactly one blank line separating the outer table from the deferred inner table: {out:?}"
    );
    assert!(
        table_count >= 1,
        "expected at least one separator row past the outer table: {out:?}"
    );
}

#[test]
fn should_match_the_unwrapped_nested_table_byte_for_byte() {
    let expected = content_with(ISSUE_484_HTML, compact_options());
    let actual = content_with(ISSUE_488_DIV_HTML, compact_options());
    assert_eq!(
        actual, expected,
        "a div wrapper must be transparent to nested-table detection under compact_tables:\nexpected: {expected:?}\nactual:   {actual:?}"
    );
}

#[test]
fn should_render_a_nested_table_wrapped_two_levels_deep() {
    let expected = content_with(ISSUE_484_HTML, compact_options());
    let actual = content_with(ISSUE_488_TWO_LEVELS_HTML, compact_options());
    assert_eq!(
        actual, expected,
        "a div>section>table wrapper chain must also be transparent:\nexpected: {expected:?}\nactual:   {actual:?}"
    );
}

#[test]
fn should_keep_the_wrappers_own_content_with_the_deferred_table() {
    let out = content_with(ISSUE_488_WRAPPER_WITH_CONTENT_HTML, compact_options());
    assert_eq!(
        out,
        "| Summary |\n| --- |\n| Before |\n\ntext\n\n| Item | Amount |\n| --- | --- |\n| Alpha | 10 |\nmore\n"
    );
}

#[test]
fn should_escape_and_fold_a_div_wrapped_nested_table_that_shares_its_row_with_a_sibling_cell() {
    let out = content(ISSUE_488_DIV_SIBLING_HTML);
    assert!(
        out.contains(r"| inner "),
        "a div-wrapped nested table sharing its row with a sibling must still flatten to cell content: {out:?}"
    );
}

#[test]
fn should_match_the_unwrapped_sibling_variant_byte_for_byte() {
    let expected = content(ISSUE_469_SIBLING_HTML);
    let actual = content(ISSUE_488_DIV_SIBLING_HTML);
    assert_eq!(
        actual, expected,
        "a div wrapper around a sibling-row nested table must match the unwrapped flattened cell content output:\nexpected: {expected:?}\nactual:   {actual:?}"
    );
}

#[test]
fn should_bail_with_table_nested_table_in_single_cell_row_for_the_div_wrapper() {
    let err = tier1_run(ISSUE_488_DIV_HTML, &compact_options()).expect_err("Tier-1 must bail, not flatten inline");
    assert!(
        matches!(err, BailReason::TableNestedTableInSingleCellRow),
        "expected TableNestedTableInSingleCellRow, got {err:?}"
    );
}

#[test]
fn should_not_bail_when_a_div_wrapped_nested_table_shares_its_row_with_a_sibling_cell() {
    tier1_run(ISSUE_488_DIV_SIBLING_HTML, &ConversionOptions::default())
        .expect("a div-wrapped nested table with a sibling cell keeps flattening natively and must not bail");
}

// ~keep ── Controls: pre-existing behavior this fix must not disturb ─────────────────

#[test]
fn should_not_change_the_unwrapped_issue_484_output() {
    assert_eq!(
        content_with(ISSUE_484_HTML, compact_options()),
        "| Summary |\n| --- |\n| Before |\n\n| Item | Amount |\n| --- | --- |\n| Alpha | 10 |\n"
    );
}

#[test]
fn should_not_change_the_issue_469_sibling_flattening() {
    let out = content(ISSUE_469_SIBLING_HTML);
    assert!(
        out.contains(r"| inner "),
        "unwrapped sibling flattened cell content must be unchanged: {out:?}"
    );
}

#[test]
fn should_not_regress_the_single_cell_layout_wrapper_case_with_a_div_wrapper() {
    let out = content(ISSUE_488_DIV_WRAPPER_HTML);
    assert_eq!(out, "| x |\n| --- |\n");
}

#[test]
fn should_leave_a_cell_with_a_wrapper_but_no_nested_table_unchanged() {
    let out = content(ISSUE_488_NO_TABLE_WRAPPER_HTML);
    assert_eq!(out, "| Before text |\n| ----------- |\n");
}
