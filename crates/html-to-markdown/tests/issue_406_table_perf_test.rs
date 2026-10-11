// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]

//! Regression test for issue #406 — severe table perf regression since 3.4.1.
//!
//! Root cause: the column-width pre-pass called `walk_node` per cell, which
//! acquired an `Arc<Mutex<>>` for every tag even when no visitor was attached.
//! For a 1000×10 table this produced ~50 k OS-level lock acquisitions.
//!
//! Fix: the pre-pass context holds no visitor, so the pre-pass takes no visitor
//! Mutex.  The tests here verify:
//!
//! 1. A large table (500×8) converts without error (sanity / no-panic).
//! 2. The output has the correct pipe-table structure.
//! 3. With a visitor attached, the visitor hooks still fire during the main
//!    rendering pass (not suppressed globally).
//! 4. Visitor hooks do NOT fire during the pre-pass (the whole point of the fix).

fn build_large_table(rows: usize, cols: usize) -> String {
    use std::fmt::Write as _;

    let mut html = String::from("<table><thead><tr>");
    for c in 0..cols {
        write!(html, "<th>H{c}</th>").unwrap();
    }
    html.push_str("</tr></thead><tbody>");
    for r in 0..rows {
        html.push_str("<tr>");
        for c in 0..cols {
            write!(html, "<td>R{r}C{c}</td>").unwrap();
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    html
}

/// Large table converts without error and produces valid pipe-table output.
#[test]
fn large_table_converts_correctly() {
    let html = build_large_table(500, 8);
    let result = html_to_markdown_rs::convert(&html, None)
        .expect("conversion must not fail")
        .content
        .unwrap_or_default();

    assert!(result.contains("| H0"), "header row missing");
    assert!(result.contains("| ---"), "separator row missing");
    assert!(result.contains("| R0C0"), "first data row missing");
    assert!(result.contains("| R499C7"), "last data row missing");
}

/// Single-row table is unaffected by the fix (sanity check).
#[test]
fn single_row_table_still_works() {
    let html = "<table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>";
    let result = html_to_markdown_rs::convert(html, None)
        .expect("conversion must not fail")
        .content
        .unwrap_or_default();
    assert!(result.contains("| A |"), "header A missing");
    assert!(result.contains("| B |"), "header B missing");
    assert!(result.contains("| 1 |"), "cell 1 missing");
    assert!(result.contains("| 2 |"), "cell 2 missing");
}

/// With a visitor, the visitor hooks still fire during the main rendering pass.
/// (Verifies that only the pre-pass context is without the visitor.)
#[cfg(feature = "visitor")]
#[test]
fn visitor_hooks_fire_during_main_pass_not_suppressed() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct CountingVisitor {
        table_start_count: usize,
        table_end_count: usize,
    }

    impl HtmlVisitor for CountingVisitor {
        fn visit_table_start(&mut self, _ctx: &NodeContext) -> VisitResult {
            self.table_start_count += 1;
            VisitResult::Continue
        }

        fn visit_table_end(&mut self, _ctx: &NodeContext, _content: &str) -> VisitResult {
            self.table_end_count += 1;
            VisitResult::Continue
        }
    }

    let visitor = Arc::new(Mutex::new(CountingVisitor::default()));
    let html = "<table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody><tr><td>foo</td><td>42</td></tr></tbody></table>";

    let opts = html_to_markdown_rs::ConversionOptions {
        visitor: Some(visitor.clone()),
        ..Default::default()
    };

    html_to_markdown_rs::convert(html, Some(opts)).expect("conversion must not fail");

    let (table_start_count, table_end_count) = {
        let v = visitor.lock().unwrap();
        (v.table_start_count, v.table_end_count)
    };
    assert_eq!(table_start_count, 1, "visit_table_start must fire exactly once");
    assert_eq!(table_end_count, 1, "visit_table_end must fire exactly once");
}

/// With a visitor and a large table, the visitor fires the correct number of times.
/// If hooks were being called in the pre-pass, we'd see double the count.
#[cfg(feature = "visitor")]
#[test]
fn visitor_hook_count_matches_table_row_count() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct RowCounter {
        row_count: usize,
    }

    impl HtmlVisitor for RowCounter {
        fn visit_table_row(&mut self, _ctx: &NodeContext, _cell_contents: &[String], _is_header: bool) -> VisitResult {
            self.row_count += 1;
            VisitResult::Continue
        }
    }

    const ROW_COUNT: usize = 50;
    let html = build_large_table(ROW_COUNT, 4);

    let visitor = Arc::new(Mutex::new(RowCounter::default()));
    let opts = html_to_markdown_rs::ConversionOptions {
        visitor: Some(visitor.clone()),
        ..Default::default()
    };

    html_to_markdown_rs::convert(&html, Some(opts)).expect("conversion must not fail");

    let row_count = visitor.lock().unwrap().row_count;
    assert_eq!(
        row_count,
        ROW_COUNT + 1,
        "visit_table_row must fire exactly once per row (header + body), not doubled by pre-pass"
    );
}
