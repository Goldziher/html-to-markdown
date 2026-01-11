#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_basic_table() {
    let html = r#"<table>
    <tr><th>Header 1</th><th>Header 2</th></tr>
    <tr><td>Cell 1</td><td>Cell 2</td></tr>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header 1 | Header 2 |"));
    assert!(result.contains("| Cell 1 | Cell 2 |"));
    assert!(result.contains("| --- | --- |"));
}

#[test]
fn test_table_with_sections() {
    let html = r#"<table>
        <thead>
            <tr><th>Name</th><th>Age</th></tr>
        </thead>
        <tbody>
            <tr><td>John</td><td>25</td></tr>
            <tr><td>Jane</td><td>30</td></tr>
        </tbody>
        <tfoot>
            <tr><td>Total</td><td>2</td></tr>
        </tfoot>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Name | Age |"));
    assert!(result.contains("| John | 25 |"));
    assert!(result.contains("| Jane | 30 |"));
    assert!(result.contains("| Total | 2 |"));
}

#[test]
fn test_table_caption() {
    let html = r#"<table><caption>Table Caption</caption><tr><td>Data</td></tr></table>"#;
    let result = convert(html, None).unwrap();
    assert!(result.contains("*Table Caption*"));
    assert!(result.contains("| Data |"));
}

#[test]
fn test_table_rowspan() {
    let html = r#"<table>
<tr><th>Header 1</th><th>Header 2</th></tr>
<tr><td rowspan="2">Spanning cell</td><td>
    <div>First row content</div>
    <div>Second line</div>
</td></tr>
<tr><td>
    <div>Next row</div>
    <div>More content</div>
</td></tr>
</table>"#;

    let options = ConversionOptions {
        br_in_tables: true,
        ..Default::default()
    };
    let result = convert(html, Some(options)).unwrap();

    assert!(result.contains("| Spanning cell | First row content<br>Second line |"));
    assert!(result.contains("|  | Next row<br>More content |"));
}

#[test]
fn test_table_colspan() {
    let html = r#"<table>
<tr><th colspan="2">Wide Header</th></tr>
<tr><td>Cell 1</td><td>Cell 2</td></tr>
</table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Wide Header |"));
    assert!(result.contains("| Cell 1 | Cell 2 |"));
}

#[test]
fn test_table_cell_multiline_content() {
    let html = r#"<table>
<tr><th>Header 1</th><th>Header 2</th></tr>
<tr><td>Cell 3</td><td>
    <div>Cell 4-1</div>
    <div>Cell 4-2</div>
</td></tr>
</table>"#;

    let options = ConversionOptions {
        br_in_tables: true,
        ..Default::default()
    };
    let result = convert(html, Some(options)).unwrap();

    assert!(result.contains("| Header 1 | Header 2 |"));
    assert!(result.contains("| Cell 3 | Cell 4-1<br>Cell 4-2 |"));
}

#[test]
fn test_table_first_row_in_tbody_without_header() {
    let html = r#"<table>
    <tbody>
        <tr><td>Cell 1</td><td>Cell 2</td></tr>
    </tbody>
    </table>"#;

    let result = convert(html, None).unwrap();
    let expected = "\n\n| Cell 1 | Cell 2 |\n| --- | --- |\n";
    assert_eq!(result, expected);
}

#[test]
fn test_tbody_only() {
    let html = "<table><tbody><tr><td>Data</td></tr></tbody></table>";
    let result = convert(html, None).unwrap();
    assert!(result.contains("| Data |"));
}

#[test]
fn test_tfoot_basic() {
    let html = "<table><tfoot><tr><td>Footer</td></tr></tfoot><tbody><tr><td>Data</td></tr></tbody></table>";
    let result = convert(html, None).unwrap();
    assert!(result.contains("| Footer |"));
    assert!(result.contains("| Data |"));
}

#[test]
fn test_caption_with_formatting() {
    let html = r#"<table><caption>Sales <strong>Report</strong> 2023</caption><tr><td>Data</td></tr></table>"#;
    let result = convert(html, None).unwrap();
    assert!(result.contains("*Sales **Report** 2023*"));
}

#[test]
fn test_table_with_links() {
    let html = r#"<table>
<tr><th>Name</th><th>Website</th></tr>
<tr><td>Example</td><td><a href="https://example.com">Link</a></td></tr>
</table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Name | Website |"));
    assert!(result.contains("[Link](https://example.com)"));
}

#[test]
fn test_table_with_code() {
    let html = r#"<table>
<tr><th>Command</th></tr>
<tr><td><code>ls -la</code></td></tr>
</table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Command |"));
    assert!(result.contains("`ls -la`"));
}

#[test]
fn test_table_empty_cells() {
    let html = r#"<table>
<tr><td>Data</td><td></td></tr>
</table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Data |  |"));
}

#[test]
fn test_table_single_column() {
    let html = r#"<table>
<tr><th>Header</th></tr>
<tr><td>Cell 1</td></tr>
<tr><td>Cell 2</td></tr>
</table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header |"));
    assert!(result.contains("| --- |"));
    assert!(result.contains("| Cell 1 |"));
    assert!(result.contains("| Cell 2 |"));
}

#[test]
fn test_blogger_table_with_image() {
    // Regression test for Issue #175: Image tags inside Blogger table wrappers not being processed
    let html = r#"
<table class="tr-caption-container">
  <a href="https://example.com/full-image.jpg">
    <img border="0" height="480"
         src="https://blogger.googleusercontent.com/img/test/IMG_0427.JPG"
         width="640" alt="Test Image" />
  </a>
</table>
"#;

    let result = convert(html, None).unwrap();

    // The image should be converted to markdown (wrapped in a link)
    assert!(
        result.contains("!["),
        "Result should contain markdown image syntax: {}",
        result
    );
    assert!(
        result.contains("blogger.googleusercontent.com"),
        "Result should contain image URL: {}",
        result
    );
    assert!(
        result.contains("example.com/full-image.jpg"),
        "Result should contain link URL: {}",
        result
    );
}

#[test]
fn test_table_with_image_no_rows() {
    // Test that images in tables without proper rows are still processed
    let html = r#"<table><img src="https://example.com/image.jpg" alt="test image"></table>"#;
    let result = convert(html, None).unwrap();

    assert!(
        result.contains("![test image](https://example.com/image.jpg)"),
        "Image should be converted to markdown: {}",
        result
    );
}

#[test]
fn test_table_with_link_and_image_no_rows() {
    // Test that link-wrapped images in tables without proper rows are processed
    let html =
        r#"<table><a href="https://example.com"><img src="https://example.com/image.jpg" alt="test"></a></table>"#;
    let result = convert(html, None).unwrap();

    assert!(
        result.contains("[![test](https://example.com/image.jpg)](https://example.com)"),
        "Link-wrapped image should be converted to markdown: {}",
        result
    );
}
