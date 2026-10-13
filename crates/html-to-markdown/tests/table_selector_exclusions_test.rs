#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HiddenContent, convert};

fn markdown(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options)).unwrap().content.unwrap_or_default()
}

fn assert_removed(html: &str, pruned: &str, selector: &str) {
    for compact_tables in [false, true] {
        let options = ConversionOptions {
            compact_tables,
            exclude_selectors: vec![selector.into()],
            ..Default::default()
        };
        let expected = markdown(
            pruned,
            ConversionOptions {
                compact_tables,
                ..Default::default()
            },
        );
        assert_eq!(
            markdown(html, options),
            expected,
            "selector={selector}, compact={compact_tables}"
        );
    }
}

#[test]
fn should_remove_selected_header_and_data_cells() {
    assert_removed(
        r#"<table><tr><th class="hx">H</th><th>Keep</th></tr><tr><td class="hx">C</td><td>Value</td></tr></table>"#,
        "<table><tr><th>Keep</th></tr><tr><td>Value</td></tr></table>",
        ".hx",
    );
}

#[test]
fn should_remove_selected_rows() {
    assert_removed(
        r#"<table><tr><th>Keep</th></tr><tr class="hx"><td>R</td></tr><tr><td>Value</td></tr></table>"#,
        "<table><tr><th>Keep</th></tr><tr><td>Value</td></tr></table>",
        "tr.hx",
    );
}

#[test]
fn should_remove_selected_table_sections() {
    for section in ["thead", "tbody", "tfoot"] {
        let html = format!(
            "<table><{section} class=\"hx\"><tr><th>Removed</th></tr></{section}><tr><td>Keep</td></tr></table>"
        );
        assert_removed(&html, "<table><tr><td>Keep</td></tr></table>", &format!("{section}.hx"));
    }
}

#[test]
fn should_remove_selected_layout_cells_and_rows() {
    assert_removed(
        r#"<table border="0"><tr><td class="hx" colspan="2">Removed cell</td><td>Keep</td></tr><tr class="hx"><td>Removed row</td></tr></table>"#,
        "<table border=\"0\"><tr><td>Keep</td></tr></table>",
        ".hx",
    );
}

#[test]
fn should_remove_selected_nested_table_parts() {
    assert_removed(
        r#"<table><tr><th>Outer</th><th>Other</th></tr><tr><td><table><tbody class="hx"><tr><td>Removed</td></tr></tbody><tr><td>Keep</td></tr></table></td><td>Value</td></tr></table>"#,
        "<table><tr><th>Outer</th><th>Other</th></tr><tr><td><table><tr><td>Keep</td></tr></table></td><td>Value</td></tr></table>",
        "tbody.hx",
    );
}

#[test]
fn should_honor_explicit_cell_exclusions_when_hidden_content_is_retained() {
    for hidden_content in [HiddenContent::Reachable, HiddenContent::All] {
        let options = ConversionOptions {
            hidden_content,
            exclude_selectors: vec!["td.hx".into()],
            ..Default::default()
        };
        let html = r#"<table><tr><th>Keep</th></tr><tr><td class="hx" hidden>Removed</td></tr><tr><td hidden>Value</td></tr></table>"#;
        let expected = markdown(
            "<table><tr><th>Keep</th></tr><tr><td hidden>Value</td></tr></table>",
            ConversionOptions {
                hidden_content,
                ..Default::default()
            },
        );
        assert_eq!(markdown(html, options), expected);
    }
}

#[test]
fn should_preserve_default_and_nonmatching_table_output() {
    let html = r#"<table><tr><th class="hx">H</th><td class="hx">C</td></tr><tr class="hx"><td>R</td></tr></table>"#;
    for (compact_tables, expected) in [
        (false, "| H | C |\n| --- | --- |\n| R |   |\n"),
        (true, "| H | C |\n| --- | --- |\n| R |  |\n"),
    ] {
        let options = ConversionOptions {
            compact_tables,
            ..Default::default()
        };
        assert_eq!(markdown(html, options.clone()), expected);
        assert_eq!(
            markdown(
                html,
                ConversionOptions {
                    exclude_selectors: vec![".absent".into()],
                    ..options
                }
            ),
            expected
        );
    }
}

#[test]
fn should_preserve_existing_table_and_inline_exclusions() {
    assert_removed(
        r#"<p>Keep</p><table class="hx"><tr><td>Removed</td></tr></table>"#,
        "<p>Keep</p>",
        "table.hx",
    );
    assert_removed(
        r#"<table><tr><th>Keep</th></tr><tr><td><span class="hx">Removed</span>Value</td></tr></table>"#,
        "<table><tr><th>Keep</th></tr><tr><td>Value</td></tr></table>",
        "span.hx",
    );
}

#[test]
fn should_remove_spanning_cells_and_rows_without_retaining_grid_slots() {
    assert_removed(
        r#"<table><tr><th>A</th><th>B</th><th>C</th></tr><tr><td class="hx" rowspan="2">Removed</td><td colspan="2">Wide</td></tr><tr><td rowspan="2">Tall</td><td>Keep</td></tr><tr><td>End</td></tr></table>"#,
        r#"<table><tr><th>A</th><th>B</th><th>C</th></tr><tr><td colspan="2">Wide</td></tr><tr><td rowspan="2">Tall</td><td>Keep</td></tr><tr><td>End</td></tr></table>"#,
        "td.hx",
    );
    assert_removed(
        r#"<table><tr><th>A</th><th>B</th></tr><tr class="hx"><td rowspan="2">Removed</td><td>Removed too</td></tr><tr><td colspan="2">Keep</td></tr></table>"#,
        r#"<table><tr><th>A</th><th>B</th></tr><tr><td colspan="2">Keep</td></tr></table>"#,
        "tr.hx",
    );
}

#[test]
fn should_preserve_rowspan_widths_across_an_excluded_section_row() {
    assert_removed(
        r#"<table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td rowspan="2">Tall</td><td>First</td></tr><tr class="hx"><td>Removed</td></tr><tr><td>Longest remaining value</td></tr></tbody></table>"#,
        r#"<table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td rowspan="2">Tall</td><td>First</td></tr><tr><td>Longest remaining value</td></tr></tbody></table>"#,
        "tr.hx",
    );
}

#[test]
fn should_classify_retained_content_without_an_excluded_header() {
    assert_removed(
        r#"<table border="0"><tr><th class="hx">Removed</th><td colspan="2">Keep</td></tr></table>"#,
        r#"<table border="0"><tr><td colspan="2">Keep</td></tr></table>"#,
        "th.hx",
    );
}

#[test]
fn should_exclude_table_parts_from_the_document_grid() {
    let html = r#"<table><thead><tr><th class="hx">Removed header</th><th>Keep</th></tr></thead><tbody class="hx"><tr><td>Removed body</td></tr></tbody><tbody><tr class="hx"><td>Removed row</td></tr><tr><td class="hx">Removed cell</td><td>Value</td></tr></tbody></table>"#;
    let options = ConversionOptions {
        include_document_structure: true,
        exclude_selectors: vec![".hx".into()],
        ..Default::default()
    };
    let result = convert(html, Some(options)).unwrap();
    assert_eq!(result.tables.len(), 1);
    let grid = &result.tables[0].grid;
    assert_eq!((grid.rows, grid.cols), (2, 1));
    assert_eq!(grid.cells.len(), 2);
    assert_eq!(
        grid.cells
            .iter()
            .map(|cell| (cell.content.as_str(), cell.row, cell.col))
            .collect::<Vec<_>>(),
        vec![("Keep", 0, 0), ("Value", 1, 0)]
    );
}
