// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]

//! Regression tests for issue #469: with `br_in_tables: true`, a nested table inside a cell
//! lost its row boundaries entirely.
//!
//! A multi-cell GFM table cannot contain a nested table, so the inner table is flattened into the outer
//! cell and its pipes escaped (`ee77eb2a18`). Until 3.11.2 `br_in_tables: true` merely
//! *skipped* the whole-cell newline fold, so the inner rows leaked out of the cell as raw
//! newlines — malformed, but a GFM parser could still see two rows. `bb67a022b3` made that
//! fold unconditional to fix issues #456/#457, which is correct: a raw newline between two
//! pipes splits the row across physical lines. The rows then collapsed onto one line joined by
//! spaces, and the boundaries were gone.
//!
//! The fix keeps the fold unconditional and instead honours what `br_in_tables` means: under
//! that option the flattened rows are joined with literal `<br>` markers, so the boundaries
//! survive without a raw newline ever reaching the cell.
//!
//! This restores row boundaries, not table structure — a real nested GFM table remains
//! impossible and the inner pipes stay escaped. Single-cell layout wrappers are instead
//! unwrapped to preserve their inner data tables (issue #478); those have separate coverage.

use html_to_markdown_rs::{ConversionOptions, convert};

const NESTED: &str = "<table><tr><td><p>Before</p><table><tr><th>ID</th><th>Status</th></tr><tr><td>123</td><td>Done</td></tr></table></td><td>Other</td></tr></table>";

fn content(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options)).unwrap().content.unwrap_or_default()
}

fn reporter_options(br_in_tables: bool) -> ConversionOptions {
    ConversionOptions {
        compact_tables: true,
        br_in_tables,
        ..ConversionOptions::default()
    }
}

/// The reported reproduction. Every inner row must remain individually visible, separated by
/// the marker `br_in_tables` promises.
#[test]
fn should_separate_flattened_nested_table_rows_with_br() {
    let out = content(NESTED, reporter_options(true));
    assert!(
        out.contains(r"Before<br>ID Status<br>"),
        "the paragraph and the inner header row must be separated by <br>: {out:?}"
    );
    assert!(
        out.contains(r"<br>123 Done"),
        "the inner data row must stay separate from the header row: {out:?}"
    );
}

/// The cell invariant that issues #456/#457 established must survive: no raw newline may reach
/// the rendered row, in either mode.
#[test]
fn should_never_leak_a_raw_newline_into_the_outer_row() {
    for br_in_tables in [true, false] {
        let out = content(NESTED, reporter_options(br_in_tables));
        let row_lines: Vec<&str> = out.lines().filter(|line| line.starts_with('|')).collect();
        for line in &row_lines {
            assert!(
                line.ends_with('|'),
                "a row was split across physical lines (br_in_tables={br_in_tables}): {out:?}"
            );
        }
        assert_eq!(
            row_lines.len(),
            2,
            "expected exactly one row plus its separator (br_in_tables={br_in_tables}): {out:?}"
        );
    }
}

/// Default output is untouched: the change is gated entirely on a non-default option.
#[test]
fn should_leave_the_default_flattening_unchanged() {
    let out = content(NESTED, reporter_options(false));
    assert!(!out.contains("<br>"), "no <br> may appear by default: {out:?}");
    assert!(
        out.contains(r"Before ID Status"),
        "default flattening must still space-join: {out:?}"
    );
}

/// A nested table that is the only content of its cell must not pick up a leading `<br>`.
#[test]
fn should_not_emit_a_leading_break_when_the_nested_table_starts_the_cell() {
    let html = "<table><tr><td><table><tr><td>x</td></tr></table></td></tr></table>";
    let out = content(html, reporter_options(true));
    assert!(
        !out.lines()
            .any(|line| line.starts_with("|<br>") || line.starts_with("| <br>")),
        "a cell opening with a nested table must not start with <br>: {out:?}"
    );
}
