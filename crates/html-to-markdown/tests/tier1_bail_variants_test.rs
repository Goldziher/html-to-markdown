//! One positive trigger per `BailReason` variant — covering the table-specific
//! variants not exercised by `tier1_bail_test.rs`.
//!
//! Coverage:
//!   - `Classifier`               — covered in `tier1_bail_test.rs`
//!   - `DepthMismatch`            — covered in `tier1_bail_test.rs`
//!   - `EofWithOpenBlock`         — covered in `tier1_bail_test.rs`
//!   - `LiteralLt`                — covered in `tier1_bail_test.rs`
//!   - `Cdata`                    — covered in `tier1_bail_test.rs`
//!   - `UnknownCustomElement`     — covered in `tier1_bail_test.rs`
//!   - `TableRowspanColspan`      — Phase F: now handled natively; test checks correct output
//!   - `TableBlockChildInCell`    — Phase J: <p>/<div>/<br>/<ul>/<ol>/<h1>-<h6> handled; <blockquote>/<pre> still bail
//!   - `TableNestedTable`         — NEW (this file)
//!   - `TableCaption`             — Phase F: now handled natively; test checks correct output
//!   - `TableSectionOrder`        — NEW (this file, two orderings)

#![cfg(feature = "testkit")]

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn tier1_run(html: &str) -> Result<String, BailReason> {
    let (cleaned, report) = prescan::run(html);
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    tier1::run(cleaned.as_ref(), &report, &opts)
}

fn tier2(html: &str) -> String {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    convert(html, Some(opts)).unwrap().content.unwrap_or_default()
}

fn force_tier1(html: &str) -> String {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    convert(html, Some(opts)).unwrap().content.unwrap_or_default()
}

// ~keep Phase F: rowspan/colspan are now handled natively (lossy: spanned cell
// ~keep appears as a single Markdown cell, not repeated/expanded).

#[test]
fn should_handle_table_rowspan_greater_than_one() {
    let html = "<table><tr><td rowspan=\"2\">a</td><td>b</td></tr></table>";
    // ~keep Tier-1 must not bail.  The cell content is emitted once (lossy).
    tier1_run(html).expect("Tier-1 should not bail on rowspan");
}

#[test]
fn should_handle_table_colspan_greater_than_one() {
    let html = "<table><tr><th colspan=\"3\">Header</th></tr></table>";
    // ~keep Tier-1 expands colspan by appending (colspan - 1) empty cells to the
    // ~keep row so the column count matches Tier-2's expanded view.  Output must
    // ~keep be byte-identical to Tier-2.
    let t1 = tier1_run(html).expect("Tier-1 should not bail on colspan");
    assert_eq!(t1, tier2(html), "colspan output must match Tier-2");
}

#[test]
fn should_not_bail_when_rowspan_and_colspan_are_one() {
    // ~keep Explicit rowspan="1" and colspan="1" must NOT trigger the bail.
    let html = r#"<table>
<thead><tr><th rowspan="1" colspan="1">A</th><th>B</th></tr></thead>
<tbody><tr><td>1</td><td>2</td></tr></tbody>
</table>"#;
    let result = tier1_run(html);
    assert!(result.is_ok(), "rowspan=1/colspan=1 must not bail; got {result:?}");
}

// ~keep ── TableBlockChildInCell ─────────────────────────────────────────────────────
// ~keep Phase F: <p>, <div>, and <br> are now handled natively in cells.
// ~keep <ul>/<ol>/<blockquote>/<pre>/<h1-6> still bail.

#[test]
fn should_handle_paragraph_in_table_cell() {
    // ~keep <p> is now inlined (no bail); Tier-1 output matches Tier-2.
    let html = "<table><tr><td><p>text</p></td></tr></table>";
    let t1 = tier1_run(html).expect("Tier-1 should not bail on <p> in cell");
    assert_eq!(t1, tier2(html), "<p>-in-cell output must match Tier-2");
}

#[test]
fn should_handle_div_in_table_cell() {
    // ~keep <div> is now inlined (no bail); Tier-1 output matches Tier-2.
    let html = "<table><tr><td><div>block</div></td></tr></table>";
    let t1 = tier1_run(html).expect("Tier-1 should not bail on <div> in cell");
    assert_eq!(t1, tier2(html), "<div>-in-cell output must match Tier-2");
}

#[test]
fn should_handle_br_in_table_cell() {
    // ~keep `<br>` in a cell emits three spaces — Tier-2 walks `<br>` to `"  \n"`
    // ~keep then `cell_text.replace('\n', " ")` produces `"   "` (three spaces).
    // ~keep Tier-1 emits the same three spaces directly to match byte-for-byte.
    let html = "<table><tr><td>line1<br>line2</td></tr></table>";
    let t1 = tier1_run(html).expect("Tier-1 should not bail on <br> in cell");
    assert_eq!(t1, tier2(html), "<br>-in-cell output must match Tier-2");
}

#[test]
fn should_handle_list_in_table_cell() {
    // ~keep Phase J: <ul>/<ol> inside a table cell are now handled natively.
    // ~keep Tier-1 emits the list items without bullet markers (matching Tier-2's
    // ~keep `in_table_cell` path), and the cell accumulator collapses newlines to spaces.
    let html = "<table><tr><td><ul><li>item</li></ul></td></tr></table>";
    let t1 = tier1_run(html).expect("Tier-1 should not bail on <ul> in cell");
    assert_eq!(t1, tier2(html), "<ul>-in-cell output must match Tier-2");
}

// ~keep ── TableNestedTable (Phase HH: now handled natively) ────────────────────────
// ~keep Phase HH flattens a nested `<table>` inside a parent cell to inline GFM
// ~keep markdown.  The parent cell's newline collapse then squashes the inner
// ~keep table's `\n`-separated rows into a single inline run, matching Tier-2.
// ~keep The `TableNestedTable` bail variant is retained for back-compat but is
// ~keep used for single-cell layout wrappers, which Tier-2 unwraps (issue #478).

#[test]
fn should_handle_nested_table_inside_cell() {
    let html = "<table><tr><td><table><tr><td>inner</td></tr></table></td><td>Other</td></tr></table>";
    let t1 = tier1_run(html).expect("Tier-1 should not bail on nested <table> (Phase HH)");
    let t2 = tier2(html);
    // ~keep Inner table syntax is omitted because GFM cannot nest tables (#760).
    assert!(
        t1.starts_with(r"| inner |") && t1.ends_with('\n'),
        "Tier-1 outer row content must include the flattened nested cell content; got: {t1:?}"
    );
    assert!(
        t2.starts_with(r"| inner |") && t2.ends_with('\n'),
        "Tier-2 outer row content must include the flattened nested cell content; got: {t2:?}"
    );
}

#[test]
fn should_fall_back_for_single_cell_nested_table_wrappers() {
    let html = "<table><tr><td><table><tr><td>inner</td></tr></table></td></tr></table>";
    assert!(matches!(tier1_run(html), Err(BailReason::TableNestedTable)));
    let expected = "| inner |\n| ----- |\n";
    assert_eq!(tier2(html), expected);
    assert_eq!(force_tier1(html), expected);
    let options = ConversionOptions {
        extract_metadata: false,
        ..Default::default()
    };
    assert_eq!(convert(html, Some(options)).unwrap().content.unwrap(), expected);
}

// ~keep ── TableCaption ──────────────────────────────────────────────────────────────
// ~keep Caption is now handled natively by Tier-1 (no longer a bail reason).

#[test]
fn should_handle_table_caption_element() {
    let html = "<table><caption>My table</caption><tr><td>a</td></tr></table>";
    // ~keep Tier-1 must succeed (no bail) and produce byte-equal output to Tier-2.
    let t1 = tier1_run(html).expect("Tier-1 should not bail on <caption>");
    let t2 = tier2(html);
    assert_eq!(t1, t2, "Tier-1 caption output must match Tier-2");
}

#[test]
fn should_bail_when_thead_appears_after_tbody_close() {
    let html = "<table><tbody><tr><td>a</td></tr></tbody><thead><tr><th>h</th></tr></thead></table>";
    let err = tier1_run(html).unwrap_err();
    assert!(
        matches!(err, BailReason::TableSectionOrder),
        "expected TableSectionOrder, got {err:?}"
    );
    assert_eq!(force_tier1(html), tier2(html), "fallback output must match Tier-2");
}

#[test]
fn should_bail_when_tbody_appears_after_tfoot() {
    let html = "<table><tfoot><tr><td>f</td></tr></tfoot><tbody><tr><td>b</td></tr></tbody></table>";
    let err = tier1_run(html).unwrap_err();
    assert!(
        matches!(err, BailReason::TableSectionOrder),
        "expected TableSectionOrder for tbody-after-tfoot, got {err:?}"
    );
    assert_eq!(force_tier1(html), tier2(html), "fallback output must match Tier-2");
}

// ~keep ── BailReason::TableRowspanColspan display ───────────────────────────────────

#[test]
fn table_bail_reason_display_strings_are_non_empty() {
    let reasons = [
        BailReason::TableRowspanColspan,
        BailReason::TableBlockChildInCell,
        BailReason::TableNestedTable,
        BailReason::TableCaption,
        BailReason::TableSectionOrder,
    ];
    for reason in &reasons {
        let s = reason.to_string();
        assert!(!s.is_empty(), "Display for {reason:?} produced empty string");
    }
}

#[test]
fn sibling_list_items_in_a_table_cell_are_separated() {
    // ~keep `should_handle_list_in_table_cell` above uses a single <li>, which is why
    // ~keep this regression went unnoticed: with the marker stripped for cell context,
    // ~keep consecutive items were appended with no separator at all, so <li>a</li><li>b</li>
    // ~keep rendered as the fabricated word `ab`. Both tiers now emit the same
    // ~keep `br_in_tables`-gated boundary already used between sibling <p>/<div> in a
    // ~keep cell: a literal `<br>` when true, a single space when false (see
    // ~keep `main_helpers::emit_table_cell_break`) -- `br_in_tables: false` used to be
    // ~keep ignored here and a literal `<br>` was emitted regardless of the option.
    for (html, expected_cell_br_true, expected_cell_br_false) in [
        (
            "<table><tr><td><ul><li>a</li><li>b</li></ul></td></tr></table>",
            "a<br>b",
            "a b",
        ),
        (
            "<table><tr><td><ol><li>one</li><li>two</li></ol></td></tr></table>",
            "one<br>two",
            "one two",
        ),
    ] {
        for (br_in_tables, expected_cell) in [(true, expected_cell_br_true), (false, expected_cell_br_false)] {
            let opts = ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                extract_metadata: false,
                br_in_tables,
                ..ConversionOptions::default()
            };
            let t2 = convert(html, Some(opts)).unwrap().content.unwrap_or_default();
            assert!(
                t2.contains(expected_cell),
                "Tier-2 cell (br_in_tables: {br_in_tables}) should contain `{expected_cell}`, got:\n{t2}"
            );
            assert!(!t2.contains("ab |"), "list items must not be concatenated, got:\n{t2}");

            let (cleaned, report) = prescan::run(html);
            let opts1 = ConversionOptions {
                tier_strategy: TierStrategy::Tier1,
                extract_metadata: false,
                br_in_tables,
                ..ConversionOptions::default()
            };
            let t1 = tier1::run(cleaned.as_ref(), &report, &opts1).expect("Tier-1 should not bail on a list in a cell");
            assert_eq!(
                t1, t2,
                "Tier-1 and Tier-2 must agree byte-for-byte (br_in_tables: {br_in_tables})"
            );
        }
    }
}
