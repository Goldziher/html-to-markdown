use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_basic_row_and_cell_conversion() {
    let html = r#"<table>
    <row><cell>Header 1</cell><cell>Header 2</cell></row>
    <row><cell>Cell 1</cell><cell>Cell 2</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header 1 | Header 2 |"));
    assert!(result.contains("| Cell 1 | Cell 2 |"));
}

#[test]
fn test_cell_role_head_as_table_header() {
    let html = r#"<table>
    <row><cell role="head">Column 1</cell><cell role="head">Column 2</cell></row>
    <row><cell>Data 1</cell><cell>Data 2</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Column 1 | Column 2 |"));
    assert!(result.contains("| Data 1 | Data 2 |"));
    assert!(result.contains("| --- | --- |"));
}

#[test]
fn test_mixed_html_and_xml_elements() {
    let html = r#"<table>
    <row><cell role="head">Name</cell><cell role="head">Age</cell></row>
    <row><td><strong>John</strong></td><td>25</td></row>
    <row><cell><em>Jane</em></cell><cell>30</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Name | Age |"));
    assert!(result.contains("**John**"));
    assert!(result.contains("*Jane*"));
}

#[test]
fn test_tei_cols_and_rows_attributes() {
    let html = r#"<table cols="2" rows="3">
    <row><cell>Header 1</cell><cell>Header 2</cell></row>
    <row><cell>Cell 1</cell><cell>Cell 2</cell></row>
    <row><cell>Cell 3</cell><cell>Cell 4</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header 1 | Header 2 |"));
    assert!(result.contains("| Cell 1 | Cell 2 |"));
    assert!(result.contains("| Cell 3 | Cell 4 |"));
}

#[test]
fn test_graphic_element_with_xlink_href() {
    let html = r#"<table>
    <row>
        <cell role="head">Image</cell>
        <cell role="head">Description</cell>
    </row>
    <row>
        <cell><graphic xlink:href="image.png"/></cell>
        <cell>A test image</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Image | Description |"));
    assert!(result.contains("| A test image |"));
    // graphic element handling may vary based on implementation
}

#[test]
fn test_graphic_in_table_cells() {
    let html = r#"<table>
    <row>
        <cell role="head">Figure</cell>
    </row>
    <row>
        <cell><graphic url="diagram.svg" alt="System Diagram"/></cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Figure |"));
}

#[test]
fn test_empty_cells_xml() {
    let html = r#"<table>
    <row><cell>Data</cell><cell></cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Data |  |"));
}

#[test]
fn test_nested_content_in_cells() {
    let html = r#"<table>
    <row><cell role="head">Text</cell><cell role="head">Formatted</cell></row>
    <row>
        <cell>Plain text</cell>
        <cell><b>Bold</b> and <a href="http://example.com">Link</a></cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Text | Formatted |"));
    assert!(result.contains("| Plain text |"));
    assert!(result.contains("**Bold**"));
    assert!(result.contains("[Link](http://example.com)"));
}

#[test]
fn test_mixed_tr_and_row_in_same_table() {
    let html = r#"<table>
    <tr><th>Col 1</th><th>Col 2</th></tr>
    <row><cell>Data 1</cell><cell>Data 2</cell></row>
    <tr><td>Data 3</td><td>Data 4</td></tr>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Col 1 | Col 2 |"));
    assert!(result.contains("| Data 1 | Data 2 |"));
    assert!(result.contains("| Data 3 | Data 4 |"));
}

#[test]
fn test_cell_without_role_attribute_defaults_to_data() {
    let html = r#"<table>
    <row><cell>Header</cell></row>
    <row><cell>Data Cell</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header |"));
    assert!(result.contains("| Data Cell |"));
}

#[test]
fn test_xml_table_with_multiline_content() {
    let html = r#"<table>
    <row><cell role="head">Content</cell></row>
    <row>
        <cell>
            <p>Line 1</p>
            <p>Line 2</p>
        </cell>
    </row>
    </table>"#;

    let options = ConversionOptions {
        br_in_tables: true,
        ..Default::default()
    };
    let result = convert(html, Some(options)).unwrap();

    assert!(result.contains("| Content |"));
    assert!(result.contains("Line 1"));
    assert!(result.contains("Line 2"));
}

#[test]
fn test_cell_with_lists() {
    let html = r#"<table>
    <row><cell role="head">Items</cell></row>
    <row>
        <cell>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
        </cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Items |"));
    assert!(result.contains("Item 1"));
    assert!(result.contains("Item 2"));
}

#[test]
fn test_single_column_xml_table() {
    let html = r#"<table>
    <row><cell role="head">Header</cell></row>
    <row><cell>Data 1</cell></row>
    <row><cell>Data 2</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header |"));
    assert!(result.contains("| --- |"));
    assert!(result.contains("| Data 1 |"));
    assert!(result.contains("| Data 2 |"));
}

#[test]
fn test_cell_with_code_blocks() {
    let html = r#"<table>
    <row><cell role="head">Code</cell></row>
    <row><cell><code>function()</code></cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Code |"));
    assert!(result.contains("`function()`"));
}

#[test]
fn test_xml_table_with_emphasis() {
    let html = r#"<table>
    <row>
        <cell role="head"><em>Emphasized</em></cell>
        <cell role="head"><strong>Strong</strong></cell>
    </row>
    <row>
        <cell><i>Italic</i></cell>
        <cell><b>Bold</b></cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("*Emphasized*"));
    assert!(result.contains("**Strong**"));
    assert!(result.contains("*Italic*"));
    assert!(result.contains("**Bold**"));
}

#[test]
fn test_xml_table_with_multiple_headers() {
    let html = r#"<table>
    <row>
        <cell role="head">First</cell>
        <cell role="head">Second</cell>
        <cell role="head">Third</cell>
    </row>
    <row>
        <cell>A</cell>
        <cell>B</cell>
        <cell>C</cell>
    </row>
    <row>
        <cell>D</cell>
        <cell>E</cell>
        <cell>F</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| First | Second | Third |"));
    assert!(result.contains("| A | B | C |"));
    assert!(result.contains("| D | E | F |"));
    assert!(result.contains("| --- | --- | --- |"));
}

#[test]
fn test_cell_role_variations() {
    let html = r#"<table>
    <row>
        <cell role="head">Header Cell</cell>
        <cell role="data">Data Cell</cell>
    </row>
    <row>
        <cell role="label">Label</cell>
        <cell role="head">Another Header</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header Cell | Data Cell |"));
    assert!(result.contains("| Label | Another Header |"));
}

#[test]
fn test_deeply_nested_xml_content() {
    let html = r#"<table>
    <row><cell role="head">Complex</cell></row>
    <row>
        <cell>
            <div>
                <p>
                    <strong>
                        <em>Nested</em>
                    </strong>
                </p>
            </div>
        </cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Complex |"));
    assert!(result.contains("Nested"));
}

#[test]
fn test_xml_table_with_attributes_preserved() {
    let html = r#"<table id="table1" class="data-table" xmlns:tei="http://www.tei-c.org/ns/1.0">
    <row>
        <cell role="head">Column 1</cell>
        <cell role="head">Column 2</cell>
    </row>
    <row>
        <cell>Value 1</cell>
        <cell>Value 2</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Column 1 | Column 2 |"));
    assert!(result.contains("| Value 1 | Value 2 |"));
}

#[test]
fn test_mixed_cell_types_in_rows() {
    let html = r#"<table>
    <row>
        <cell role="head">Name</cell>
        <th>Age</th>
        <cell role="head">City</cell>
    </row>
    <row>
        <cell>John</cell>
        <td>25</td>
        <cell>NYC</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Name | Age | City |"));
    assert!(result.contains("| John | 25 | NYC |"));
}
