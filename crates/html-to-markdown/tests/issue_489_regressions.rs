#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #489: two compounding defects around a table whose final row
//! is never closed.
//!
//! 1. `tl`'s `read_end` (astral-tl `parser/base.rs:240-256`) silently discards a close tag
//!    that does not match the top of its element stack instead of erroring. With the stack
//!    `[table, tr]`, a stray `</table>` is dropped and a following sibling element (e.g. a
//!    footer `<p>`) attaches as a child of the still-open `<tr>` instead of becoming the
//!    table's sibling. `collect_table_cells` only walks `td`/`th`/`cell` children, so the
//!    misattached element -- and everything after it -- is silently dropped. The fix widens
//!    `has_inline_block_misnest`'s repair gate (`src/converter/preprocessing_helpers.rs`) to
//!    detect a `<tr>`/`<row>`/`<thead>`/`<tbody>`/`<tfoot>` holding a misplaced element
//!    child, routing the document through the existing `repair_with_html5ever` escape hatch.
//!
//! 2. Once repaired, `html5ever` still leaves the empty `<tr>` in the tree (it closed
//!    `<table>` correctly but the row itself has no cells), and the ragged-row padding in
//!    `convert_table_row` printed a phantom `|   |` for it. The fix makes
//!    `convert_table_row` return `false` for a row that collects zero cells, before any
//!    output is written, and has its two callers in `builder.rs` only advance their row
//!    counter when it returns `true` -- so a cell-less row is skipped entirely rather than
//!    consuming the header slot.

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn content(html: &str) -> String {
    convert(html, Some(ConversionOptions::default()))
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

/// The issue's minimal repro: a table whose final `<tr>` is never closed, followed by a
/// visible sibling paragraph that `tl` misattaches as that `<tr>`'s child.
const ISSUE_HTML: &str = "<table><tr><td>Before</td></tr><tr></table><p>Visible footer</p>";

#[test]
fn should_keep_content_after_a_table_whose_final_row_is_never_closed() {
    assert_eq!(content(ISSUE_HTML), "| Before |\n| ------ |\n\nVisible footer\n");
}

#[test]
fn should_match_tier1_byte_for_byte() {
    let options = ConversionOptions::default();
    assert_eq!(
        tier1_run(ISSUE_HTML, &options).expect("Tier-1 does not bail on this input"),
        content(ISSUE_HTML)
    );
}

#[test]
fn should_not_emit_a_phantom_row_for_a_row_with_no_cells() {
    // ~keep A literal empty `<tr></tr>` in an otherwise well-formed table -- no misnesting,
    // ~keep no repair route involved -- must still be dropped rather than rendered as a
    // ~keep padded blank row. This isolates Edit B (the cells.is_empty() skip) from
    // ~keep Edit A (the repair-gate widening) exercised by the other tests here.
    let html = "<table><tr><td>x</td></tr><tr></tr></table>";
    assert_eq!(content(html), "| x |\n| --- |\n");
}

#[test]
fn should_recover_the_row_after_an_unclosed_tr() {
    let html = "<table><tr><td>a</td><tr><td>b</td></table>";
    assert_eq!(content(html), "| a |\n| --- |\n| b |\n");
}

#[test]
fn should_render_a_tbody_holding_cells_without_a_row() {
    let html = "<table><tbody><td>a</td></tbody></table>";
    assert_eq!(content(html), "| a |\n| --- |\n");
}

// ~keep ── Controls: must stay exactly as they are today ────────────────────────────────

#[test]
fn should_leave_a_well_formed_table_and_trailing_paragraph_unchanged() {
    let html = "<table><tr><td>a</td></tr><tr><td>b</td></tr></table><p>After</p>";
    assert_eq!(content(html), "| a |\n| --- |\n| b |\n\nAfter\n");
}

#[test]
fn should_still_pad_a_ragged_row_that_has_at_least_one_cell() {
    let html = "<table><tr><th>A</th><th>B</th></tr><tr><td>1</td></tr></table>";
    assert_eq!(content(html), "| A | B |\n| --- | --- |\n| 1 |   |\n");
}

#[test]
fn should_keep_a_row_holding_only_an_entity_space_out_of_the_repair_path() {
    // ~keep Text-only content inside <tr> deliberately does not trip the new gate (see the
    // ~keep module doc): `&nbsp;` decodes to non-whitespace, so a naive text check would
    // ~keep route every Outlook/newsletter `<tr>&nbsp;</tr>` through a full html5ever
    // ~keep re-parse. The row still collects zero <td>/<th> cells, so Edit B's cell-less-row
    // ~keep skip still applies -- orthogonal to, and independent of, the repair-path gate
    // ~keep this test pins. Losing the entity-space text is the pre-existing, documented,
    // ~keep out-of-scope defect this control exists to freeze.
    assert_eq!(content("<table><tr>&nbsp;</tr></table>"), "");
}

#[test]
fn should_end_a_cell_with_no_end_tag_at_the_end_of_its_table() {
    // ~keep The end tag of the table ends the open cell and its row, as the HTML tree builder
    // ~keep says, so the paragraph follows the table (issue #772). Before, it was text of the cell.
    let html = "<table><tr><td>Before</td></tr><tr><td></table><p>Visible footer</p>";
    assert_eq!(content(html), "| Before |\n| ------ |\n|        |\n\nVisible footer\n");
}

#[test]
fn should_report_two_rows_via_structure_even_though_markdown_shows_one() {
    // ~keep Known residual, out of scope for #489: `collect_table_grid`
    // ~keep (`block/table/mod.rs`) builds `result.tables`/`document.nodes` independently of
    // ~keep `convert_table_row`, so `--json --include-structure`-style structure extraction
    // ~keep still counts the phantom row that the rendered markdown no longer shows.
    let options = ConversionOptions {
        include_document_structure: true,
        ..ConversionOptions::default()
    };
    let result = convert(ISSUE_HTML, Some(options)).expect("conversion should succeed");
    assert_eq!(
        result.tables.len(),
        1,
        "expected exactly one table: {:?}",
        result.tables
    );
    assert_eq!(
        result.tables[0].grid.rows, 2,
        "structure extraction still reports the phantom row (known residual, tracked separately): {:?}",
        result.tables[0].grid
    );
}
