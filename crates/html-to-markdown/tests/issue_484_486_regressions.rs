#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #486 (a `<tr>` nested directly inside a `<td>` silently drops
//! its content) and issue #484 (a nested table inside a data table's single-cell row is
//! flattened into a line of escaped, unusable pipes instead of surviving as a usable table).
//!
//! Both are table-nesting regressions against 3.8.3, assigned together because they turned
//! out to touch the same repair machinery — a leading HTML comment used to matter for #486
//! only because of an unrelated bug (any comment-bearing document was misreported as having a
//! custom element, tripping an accidental early html5ever repair pass); that bug is already
//! fixed, so the comment no longer changes anything and both variants are covered here to
//! prove it.

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

// ~keep ── Issue #486: <tr> nested directly inside <td> ─────────────────────────────

const ISSUE_486_MALFORMED: &str = "<table><tr><td><tr><td>Visible text</td></tr></td></tr></table>";
const ISSUE_486_WITH_COMMENT: &str = "<!-- comment --><table><tr><td><tr><td>Visible text</td></tr></td></tr></table>";
const ISSUE_486_WELL_FORMED: &str = "<table><tr><td></td></tr><tr><td>Visible text</td></tr></table>";

#[test]
fn should_preserve_cell_text_when_a_row_is_nested_directly_in_a_cell() {
    let out = content(ISSUE_486_MALFORMED);
    assert!(
        out.contains("Visible text"),
        "the row nested inside a <td> must not silently drop its text: {out:?}"
    );
}

#[test]
fn should_preserve_cell_text_when_a_row_is_nested_directly_in_a_cell_with_leading_comment() {
    let out = content(ISSUE_486_WITH_COMMENT);
    assert!(
        out.contains("Visible text"),
        "a leading comment must not change whether the text is recovered: {out:?}"
    );
}

#[test]
fn should_match_the_well_formed_equivalent_regardless_of_the_leading_comment() {
    let expected = content(ISSUE_486_WELL_FORMED);
    assert!(expected.contains("Visible text"));
    assert_eq!(content(ISSUE_486_MALFORMED), expected);
    assert_eq!(content(ISSUE_486_WITH_COMMENT), expected);
}

#[test]
fn should_render_the_well_formed_two_row_table_unchanged() {
    // ~keep Locks in the exact well-formed rendering so a future change to the repair
    // ~keep path is caught even if it happens to keep passing the two `contains` checks.
    assert_eq!(
        content(ISSUE_486_WELL_FORMED),
        "|              |\n| ------------ |\n| Visible text |\n"
    );
}

#[test]
fn should_bail_with_table_block_child_in_cell_for_a_row_nested_in_a_cell() {
    let options = ConversionOptions::default();
    for html in [ISSUE_486_MALFORMED, ISSUE_486_WITH_COMMENT] {
        let err = tier1_run(html, &options).expect_err("Tier-1 must bail, not silently drop the text");
        assert!(
            matches!(err, BailReason::TableBlockChildInCell),
            "expected TableBlockChildInCell, got {err:?} for input {html:?}"
        );
    }
}

#[test]
fn should_not_bail_on_the_well_formed_two_row_table() {
    let options = ConversionOptions::default();
    tier1_run(ISSUE_486_WELL_FORMED, &options).expect("Tier-1 should not bail on a well-formed table");
}

// ~keep ── Issue #484: nested table inside a data table's single-cell row ────────────

const ISSUE_484_HTML: &str = "<table><tr><th>Summary</th></tr><tr><td>Before<table><tr><th>Item</th>\
<th>Amount</th></tr><tr><td>Alpha</td><td>10</td></tr></table></td></tr></table>";

fn issue_484_options() -> ConversionOptions {
    ConversionOptions {
        br_in_tables: true,
        compact_tables: true,
        ..ConversionOptions::default()
    }
}

#[test]
fn should_render_the_inner_table_separately_instead_of_flattening_it() {
    let out = content_with(ISSUE_484_HTML, issue_484_options());
    assert_eq!(
        out,
        "| Summary |\n| --- |\n| Before |\n\n| Item | Amount |\n| --- | --- |\n| Alpha | 10 |\n"
    );
}

#[test]
fn should_render_the_inner_table_separately_regardless_of_a_leading_comment() {
    let with_comment = format!("<!-- comment -->{ISSUE_484_HTML}");
    assert_eq!(
        content_with(&with_comment, issue_484_options()),
        content_with(ISSUE_484_HTML, issue_484_options())
    );
}

#[test]
fn should_keep_the_header_and_data_rows_of_the_inner_table_usable() {
    let out = content_with(ISSUE_484_HTML, issue_484_options());
    assert!(
        out.contains("Item") && out.contains("Amount"),
        "inner header row lost: {out:?}"
    );
    assert!(
        out.contains("Alpha") && out.contains("10"),
        "inner data row lost: {out:?}"
    );
    assert!(
        !out.contains(r"\|"),
        "no pipe should need escaping once the inner table is separate: {out:?}"
    );
}

#[test]
fn should_bail_with_table_nested_table_in_single_cell_row() {
    let err = tier1_run(ISSUE_484_HTML, &issue_484_options()).expect_err("Tier-1 must bail, not flatten inline");
    assert!(
        matches!(err, BailReason::TableNestedTableInSingleCellRow),
        "expected TableNestedTableInSingleCellRow, got {err:?}"
    );
}

// ~keep ── Control: a sibling cell must keep the pre-existing flattened cell content ──────
// ~keep behavior (issue #469) — the new deferral only ever applies to a row whose nested
// ~keep table's cell has no sibling cell.

#[test]
fn should_not_bail_when_a_nested_table_shares_its_row_with_a_sibling_cell() {
    let html = "<table><tr><td><table><tr><td>inner</td></tr></table></td><td>Other</td></tr></table>";
    tier1_run(html, &ConversionOptions::default())
        .expect("a nested table with a sibling cell keeps flattening natively and must not bail");
}

#[test]
fn should_still_flatten_a_nested_table_that_shares_its_row_with_a_sibling_cell() {
    let html = "<table><tr><td><table><tr><td>inner</td></tr></table></td><td>Other</td></tr></table>";
    let out = content(html);
    assert!(
        out.contains(r"| inner "),
        "a nested table sharing its row with a sibling must still flatten to cell content: {out:?}"
    );
}

// ~keep ── Control: the #478 single-cell layout-wrapper case must still be unwrapped, ──
// ~keep not routed through the new #484 deferral mechanism.

#[test]
fn should_not_regress_the_single_cell_layout_wrapper_case() {
    let html = "<table><tr><td><table><tr><td>x</td></tr></table></td></tr></table>";
    let err = tier1_run(html, &ConversionOptions::default()).expect_err("the wrapper case must still bail");
    assert!(
        matches!(err, BailReason::TableNestedTable),
        "expected the existing TableNestedTable wrapper bail, got {err:?}"
    );
    assert_eq!(content(html), "| x |\n| --- |\n");
}
