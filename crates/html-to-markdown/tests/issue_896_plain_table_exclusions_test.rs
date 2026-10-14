#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, convert};

const TABLE: &str = r#"<table><tr class="r0"><th id="h1">H1</th><th>H2</th></tr><tr class="r1"><td class="a" data-x="1">A</td><td>B</td></tr><tr class="r2"><td>C</td><td>D</td></tr></table>"#;

fn plain(html: &str, selectors: &[&str]) -> String {
    let options = ConversionOptions {
        output_format: OutputFormat::Plain,
        exclude_selectors: selectors.iter().map(|selector| (*selector).to_string()).collect(),
        ..Default::default()
    };
    convert(html, Some(options)).unwrap().content.unwrap_or_default()
}

/// The page with the selector writes `expected`, and so does `pruned`, the page without the
/// elements that the selector names.
fn assert_removed(html: &str, selector: &str, pruned: &str, expected: &str) {
    assert_eq!(plain(html, &[selector]), expected, "selector={selector}");
    assert_eq!(plain(pruned, &[]), expected, "the pruned page, selector={selector}");
}

#[test]
fn should_not_write_the_excluded_cell_of_the_issue() {
    assert_removed(
        r#"<p>ZkpreZ</p><table><caption>ZkcapZ</caption><tr><th>Zks0r0c0Z</th><th>Zks0r0c1Z</th></tr><tr><td class="hx">Zks0r1c0Z</td><td>Zks0r1c1Z</td></tr></table><p>ZkpostZ</p>"#,
        ".hx",
        "<p>ZkpreZ</p><table><caption>ZkcapZ</caption><tr><th>Zks0r0c0Z</th><th>Zks0r0c1Z</th></tr><tr><td>Zks0r1c1Z</td></tr></table><p>ZkpostZ</p>",
        "ZkpreZ\n\nZks0r0c0Z\tZks0r0c1Z\nZks0r1c1Z\n\nZkpostZ\n",
    );
}

#[test]
fn should_remove_a_cell_by_class_id_tag_and_attribute() {
    let no_a = "<table><tr><th>H1</th><th>H2</th></tr><tr><td>B</td></tr><tr><td>C</td><td>D</td></tr></table>";
    assert_removed(TABLE, ".a", no_a, "H1\tH2\nB\nC\tD\n");
    assert_removed(TABLE, "[data-x]", no_a, "H1\tH2\nB\nC\tD\n");
    assert_removed(
        TABLE,
        "#h1",
        "<table><tr><th>H2</th></tr><tr><td>A</td><td>B</td></tr><tr><td>C</td><td>D</td></tr></table>",
        "H2\nA\tB\nC\tD\n",
    );
    assert_removed(
        "<table><tr><th>H</th><td>A</td></tr><tr><th>I</th><td>B</td></tr></table>",
        "th",
        "<table><tr><td>A</td></tr><tr><td>B</td></tr></table>",
        "A\nB\n",
    );
}

#[test]
fn should_remove_a_row() {
    assert_removed(
        TABLE,
        ".r1",
        "<table><tr><th>H1</th><th>H2</th></tr><tr><td>C</td><td>D</td></tr></table>",
        "H1\tH2\nC\tD\n",
    );
}

#[test]
fn should_remove_the_first_row() {
    assert_removed(
        TABLE,
        "tr.r0",
        "<table><tr><td>A</td><td>B</td></tr><tr><td>C</td><td>D</td></tr></table>",
        "A\tB\nC\tD\n",
    );
}

#[test]
fn should_write_no_row_when_every_row_is_excluded() {
    let html = format!("<p>Pre</p>{TABLE}<p>Post</p>");
    assert_removed(&html, "tr", "<p>Pre</p><table></table><p>Post</p>", "Pre\n\nPost\n");
}

#[test]
fn should_remove_a_table_section() {
    let html = r#"<table><thead class="hx"><tr><th>H</th></tr></thead><tbody id="b"><tr><td>B</td></tr></tbody><tfoot data-x="1"><tr><td>F</td></tr></tfoot></table>"#;
    assert_removed(
        html,
        "thead.hx",
        "<table><tbody><tr><td>B</td></tr></tbody><tfoot><tr><td>F</td></tr></tfoot></table>",
        "B\nF\n",
    );
    assert_removed(
        html,
        "#b",
        "<table><thead><tr><th>H</th></tr></thead><tfoot><tr><td>F</td></tr></tfoot></table>",
        "H\nF\n",
    );
    assert_removed(
        html,
        "[data-x]",
        "<table><thead><tr><th>H</th></tr></thead><tbody><tr><td>B</td></tr></tbody></table>",
        "H\nB\n",
    );
}

#[test]
fn should_remove_parts_of_a_nested_table() {
    assert_removed(
        r#"<table><tr><td>Outer</td><td><table><tr><td class="hx">Gone cell</td><td>Keep</td></tr><tr class="hx"><td>Gone row</td></tr><tbody class="hx"><tr><td>Gone section</td></tr></tbody></table></td></tr></table>"#,
        ".hx",
        "<table><tr><td>Outer</td><td><table><tr><td>Keep</td></tr></table></td></tr></table>",
        "Outer\tKeep\n",
    );
}

#[test]
fn should_remove_cells_and_rows_that_span() {
    assert_removed(
        r#"<table><tr><th>A</th><th>B</th><th>C</th></tr><tr><td class="hx" rowspan="2">Gone</td><td colspan="2">Wide</td></tr><tr><td>Keep</td></tr><tr class="hx"><td colspan="3">Gone row</td></tr></table>"#,
        ".hx",
        r#"<table><tr><th>A</th><th>B</th><th>C</th></tr><tr><td colspan="2">Wide</td></tr><tr><td>Keep</td></tr></table>"#,
        "A\tB\tC\nWide\nKeep\n",
    );
}

#[test]
fn should_keep_the_rows_when_the_caption_is_excluded() {
    assert_removed(
        r#"<table><caption class="hx">Cap</caption><tr><td>A</td><td>B</td></tr></table>"#,
        "caption.hx",
        "<table><tr><td>A</td><td>B</td></tr></table>",
        "A\tB\n",
    );
}

#[test]
fn should_remove_a_whole_table() {
    assert_removed(
        r#"<p>Pre</p><table class="hx"><tr><td>A</td></tr></table><p>Post</p>"#,
        "table.hx",
        "<p>Pre</p><p>Post</p>",
        "Pre\n\nPost\n",
    );
}

#[test]
fn should_write_every_cell_with_no_selector_and_with_a_selector_that_matches_nothing() {
    assert_eq!(plain(TABLE, &[]), "H1\tH2\nA\tB\nC\tD\n");
    assert_eq!(plain(TABLE, &[".absent"]), "H1\tH2\nA\tB\nC\tD\n");
}
