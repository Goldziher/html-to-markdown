// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Issue #752: every walk of a table cell knows whether the cell holds only inputs.
//!
//! A cell that holds only a checkbox writes the state of the checkbox. A cell that holds a
//! checkbox beside text writes only the text.

use html_to_markdown_rs::options::PreprocessingOptions;
use html_to_markdown_rs::{ConversionOptions, ConversionResult, TierStrategy, convert};

const DATA_TABLE: &str = r#"<table><tr><th><input type="checkbox"></th><th>Name</th></tr><tr><td><input type="checkbox" checked></td><td><input type="checkbox"> Bob</td></tr></table>"#;

fn options(tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        tier_strategy,
        preprocessing: PreprocessingOptions {
            remove_forms: false,
            ..PreprocessingOptions::default()
        },
        ..ConversionOptions::default()
    }
}

fn result(html: &str, options: ConversionOptions) -> ConversionResult {
    convert(html, Some(options)).expect("conversion must succeed")
}

fn assert_all_converters(html: &str, expected: &str) {
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Tier1, TierStrategy::Auto] {
        assert_eq!(
            result(html, options(tier_strategy)).content.unwrap_or_default(),
            expected,
            "{tier_strategy:?}: {html}"
        );
    }
}

#[test]
fn should_write_the_checkbox_state_in_a_layout_table_cell_that_holds_only_inputs() {
    for (html, expected) in [
        (
            r#"<table><tr><td><a href="/a">a</a></td><td><input type="checkbox" checked></td><td><a href="/b">b</a></td><td><a href="/c">c</a></td></tr></table>"#,
            "- [a](/a) [x] [b](/b) [c](/c)\n",
        ),
        (
            r#"<table><tr><td><a href="/a">a</a></td><td><input type="checkbox"></td><td><a href="/b">b</a></td><td><a href="/c">c</a></td></tr></table>"#,
            "- [a](/a) [ ] [b](/b) [c](/c)\n",
        ),
        (
            r#"<table><tr><td><a href="/a">a</a></td><td><input type="checkbox" checked> x</td><td><a href="/b">b</a></td><td><a href="/c">c</a></td></tr></table>"#,
            "- [a](/a) x [b](/b) [c](/c)\n",
        ),
    ] {
        assert_all_converters(html, expected);
    }
}

#[test]
fn should_write_the_checkbox_state_in_the_grid_of_a_table() {
    let structured = result(
        DATA_TABLE,
        ConversionOptions {
            include_document_structure: true,
            ..options(TierStrategy::Auto)
        },
    );
    assert_eq!(structured.tables.len(), 1);
    let cells: Vec<&str> = structured.tables[0]
        .grid
        .cells
        .iter()
        .map(|cell| cell.content.as_str())
        .collect();
    assert_eq!(cells, ["[ ]", "Name", "[x]", "Bob"]);
}

/// Asserts that the table with the header `h` and the one body cell `cell` writes `text` in
/// that cell.
fn assert_one_cell(cell: &str, text: &str) {
    let width = text.len().max(1);
    let rule = "-".repeat(width.max(3));
    assert_all_converters(
        &format!("<table><tr><th>h</th></tr><tr>{cell}</tr></table>"),
        &format!("| {:<width$} |\n| {rule} |\n| {text:<width$} |\n", "h"),
    );
}

#[test]
fn should_write_the_checkbox_state_in_a_table_cell_that_holds_only_inputs() {
    for (html, expected) in [
        (
            r#"<table><tr><th>Feature</th><th>Done</th></tr><tr><td>Search</td><td><input type="checkbox" checked disabled></td></tr></table>"#,
            "| Feature | Done |\n| ------- | ---- |\n| Search  | [x]  |\n",
        ),
        (
            r#"<table><tr><th><input type="checkbox"></th><th>Name</th></tr><tr><td><input type="checkbox" checked></td><td>Bob</td></tr></table>"#,
            "| [ ] | Name |\n| --- | ---- |\n| [x] | Bob  |\n",
        ),
        (
            r#"<table><tr><th>A</th><th>B</th></tr><tr><td><input type="checkbox"> one</td><td>two <input type="checkbox" checked></td></tr></table>"#,
            "| A   | B   |\n| --- | --- |\n| one | two |\n",
        ),
    ] {
        assert_all_converters(html, expected);
    }
}

#[test]
fn should_write_the_checkbox_state_in_a_table_cell_that_holds_only_wrapped_inputs() {
    for (cell, expected) in [
        (r#"<td><label><input type="checkbox" checked></label></td>"#, "[x]"),
        (r#"<td><span><input type="checkbox"></span></td>"#, "[ ]"),
        (
            r#"<td><label><input type="checkbox"></label><span><input type="checkbox" checked></span></td>"#,
            "[ ][x]",
        ),
        (
            r#"<td><label><input type="hidden" name="a"><input type="checkbox" checked></label></td>"#,
            "[x]",
        ),
        (
            r#"<td><div><label><span><input type="checkbox" checked></span></label></div></td>"#,
            "[x]",
        ),
        (r#"<td><b></b> <label> <input type="checkbox"> </label></td>"#, "[ ]"),
        // ~keep A character reference for white space is white space.
        (r#"<td><input type="checkbox" checked>&#32;</td>"#, "[x]"),
        (
            r#"<td><span>&#x20;<input type="checkbox" checked>&nbsp;</span></td>"#,
            "[x]",
        ),
    ] {
        assert_one_cell(cell, expected);
    }
    for (html, expected) in [
        (
            r#"<table><tr><th>h</th><th>k</th></tr><tr><td><label><input type="checkbox" checked></label></td><td>v</td></tr></table>"#,
            "| h   | k |\n| --- | --- |\n| [x] | v |\n",
        ),
        (
            r#"<table><tr><td><span><input type="checkbox"></span></td></tr></table>"#,
            "| [ ] |\n| --- |\n",
        ),
        (
            r#"<table><tr><th><label><input type="checkbox"></label></th><th>n</th></tr><tr><td>x</td><td>y</td></tr></table>"#,
            "| [ ] | n |\n| --- | --- |\n| x   | y |\n",
        ),
    ] {
        assert_all_converters(html, expected);
    }
}

#[test]
fn should_write_only_the_text_of_a_table_cell_that_holds_a_wrapped_checkbox_beside_text() {
    for (cell, expected) in [
        (r#"<td><label><input type="checkbox" checked> Bob</label></td>"#, "Bob"),
        (r#"<td><label><input type="checkbox"></label> Bob</td>"#, "Bob"),
        (r#"<td><span><input type="checkbox"></span><b>Bob</b></td>"#, "**Bob**"),
        (
            r#"<td><span><input type="checkbox"></span><img src="i.png" alt="pic"></td>"#,
            "![pic](i.png)",
        ),
        // ~keep A radio button writes nothing, with a wrapper or without one.
        (r#"<td><label><input type="radio" checked></label></td>"#, ""),
    ] {
        assert_one_cell(cell, expected);
    }
}

#[test]
fn should_write_only_the_content_of_a_table_cell_that_holds_a_checkbox_beside_content() {
    for (cell, expected) in [
        // ~keep A rule and a link are content of a table without text of their own.
        (r"<td><input type=checkbox checked><hr></td>", "---"),
        (r#"<td><input type=checkbox checked><a href="/a"></a></td>"#, "[](/a)"),
        (r#"<td><label><input type="checkbox" checked></label><hr></td>"#, "---"),
        (r#"<td><input type="checkbox"><span><hr></span></td>"#, "---"),
        (
            r#"<td><label><input type="checkbox" checked></label><label><a href="/a"></a></label></td>"#,
            "[](/a)",
        ),
        (
            r#"<td><input type="checkbox" checked><a href="/a">t</a></td>"#,
            "[t](/a)",
        ),
        (
            r#"<td><input type="checkbox" checked><select><option>A</option></select></td>"#,
            "A",
        ),
        (r#"<td><input type="checkbox" checked><button>B</button></td>"#, "B"),
        (r#"<td><input type="checkbox" checked><textarea>T</textarea></td>"#, "T"),
        // ~keep A character reference for a letter is text.
        (r#"<td><input type="checkbox" checked>&#x41;</td>"#, "A"),
    ] {
        assert_one_cell(cell, expected);
    }
    let nested = r#"<table><tr><th>h</th></tr><tr><td><input type="checkbox" checked><table><tr><td>in</td></tr></table></td></tr></table>"#;
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Auto] {
        let content = result(nested, options(tier_strategy)).content.unwrap_or_default();
        assert!(content.contains("in") && !content.contains("[x]"), "{content}");
    }
}

#[test]
fn should_write_the_checkbox_state_in_a_table_cell_that_holds_a_checkbox_beside_no_content() {
    for (cell, expected) in [
        (r#"<td><input type="checkbox" checked><!-- c --></td>"#, "[x]"),
        (r#"<td><!-- <hr> --><label><input type="checkbox"></label></td>"#, "[ ]"),
        (r#"<td><input type="checkbox" checked>&nbsp;</td>"#, "[x]"),
        (r#"<td><input type="checkbox" checked>&#160;</td>"#, "[x]"),
        (r#"<td><input type="checkbox" checked>&#xA0;</td>"#, "[x]"),
        (r#"<td><input type="checkbox" checked><button></button></td>"#, "[x]"),
        // ~keep A line break is content of a table only under `br_in_tables`.
        (r#"<td><input type="checkbox" checked><br></td>"#, "[x]"),
    ] {
        assert_one_cell(cell, expected);
    }
}

#[test]
fn should_write_only_the_line_break_of_a_table_cell_under_br_in_tables() {
    let table = |cell: &str| format!("<table><tr><th>h</th></tr><tr><td>{cell}</td></tr></table>");
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Auto] {
        let convert_cell = |cell: &str| {
            let options = ConversionOptions {
                br_in_tables: true,
                ..options(tier_strategy)
            };
            result(&table(cell), options).content.unwrap_or_default()
        };
        let with_checkbox = convert_cell(r#"<input type="checkbox" checked><br>"#);
        assert!(with_checkbox.contains("<br>"), "{with_checkbox}");
        assert_eq!(with_checkbox, convert_cell("<br>"), "{tier_strategy:?}");
    }
}

#[test]
fn should_write_the_checkbox_state_in_a_cell_element_that_holds_only_inputs() {
    for (html, expected) in [
        (
            r#"<table><tr><th>Done</th><th>Name</th></tr><row><cell><input type="checkbox" checked></cell><cell>Bob</cell></row></table>"#,
            "| Done | Name |\n| ---- | ---- |\n| [x]  | Bob  |\n",
        ),
        (
            r#"<table><tr><th>Done</th><th>Name</th></tr><row><cell><input type="checkbox" checked> x</cell><cell>Bob</cell></row></table>"#,
            "| Done | Name |\n| ---- | ---- |\n| x    | Bob  |\n",
        ),
    ] {
        assert_all_converters(html, expected);
    }
}

#[cfg(feature = "visitor")]
mod visitor {
    use super::{DATA_TABLE, options, result};
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
    use html_to_markdown_rs::{ConversionOptions, TierStrategy};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct Rows(Vec<Vec<String>>);

    impl HtmlVisitor for Rows {
        fn visit_table_row(&mut self, _ctx: &NodeContext<'_>, cells: &[String], _is_header: bool) -> VisitResult {
            self.0.push(cells.to_vec());
            VisitResult::Continue
        }
    }

    #[test]
    fn should_give_a_visitor_the_checkbox_state_of_a_cell_that_holds_only_inputs() {
        let rows = Arc::new(Mutex::new(Rows::default()));
        let visitor: VisitorHandle = rows.clone();
        let converted = result(
            DATA_TABLE,
            ConversionOptions {
                visitor: Some(visitor),
                ..options(TierStrategy::Auto)
            },
        );
        assert_eq!(
            converted.content.as_deref(),
            Some("| [ ] | Name |\n| --- | ---- |\n| [x] | Bob  |\n")
        );
        assert_eq!(
            rows.lock().expect("the visitor lock").0,
            [["[ ]", "Name"], ["[x]", "Bob"]]
        );
    }
}
