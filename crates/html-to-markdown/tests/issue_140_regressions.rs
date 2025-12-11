use std::fs;
use std::path::PathBuf;

use html_to_markdown_rs::{ConversionOptions, convert};

fn fixture_path(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../test_documents/html/issues", name]
        .iter()
        .collect()
}

fn default_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        autolinks: false,
        ..Default::default()
    }
}

fn escape_misc_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        autolinks: false,
        escape_misc: true,
        ..Default::default()
    }
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

#[test]
fn converts_should_not_escape_in_pre_or_code_fixture() {
    let pre_html = r#"<pre>This pipe | should not be escaped.<pre/>"#;

    let pre_markdown_without_misc = convert(pre_html, Some(default_options())).expect("conversion should succeed");
    assert_eq!(pre_markdown_without_misc.trim(), "This pipe | should not be escaped.");

    let pre_markdown_with_misc = convert(pre_html, Some(escape_misc_options())).expect("conversion should succeed");
    assert_eq!(pre_markdown_with_misc.trim(), "This pipe | should not be escaped.");

    let code_html = r#"<code>This pipe | should not be escaped.<code/>"#;

    let code_markdown_without_misc = convert(code_html, None).expect("conversion should succeed");
    assert_eq!(
        code_markdown_without_misc.trim(),
        "`This pipe | should not be escaped.`"
    );

    let code_markdown_with_misc = convert(code_html, Some(escape_misc_options())).expect("conversion should succeed");
    assert_eq!(code_markdown_with_misc.trim(), "`This pipe | should not be escaped.`");
}

#[test]
fn converts_table_cell_pipe_fixture() {
    let html = fs::read_to_string(fixture_path("gh-140-table-cell-pipe.html")).unwrap();
    let expected_without_misc = fs::read_to_string(fixture_path("gh-140-table-cell-pipe.md")).unwrap();
    let expected_with_misc = fs::read_to_string(fixture_path("gh-140-table-cell-pipe-with-escape-misc.md")).unwrap();

    let result_without_misc = convert(&html, Some(default_options())).expect("conversion should succeed");
    assert_eq!(
        normalize_newlines(&result_without_misc),
        normalize_newlines(&expected_without_misc)
    );

    let result_with_misc = convert(&html, Some(escape_misc_options())).expect("conversion should succeed");
    assert_eq!(
        normalize_newlines(&result_with_misc),
        normalize_newlines(&expected_with_misc)
    );
}

#[test]
fn escapes_only_literal_pipes_in_table_cells() {
    let html = r#"
        <table>
            <thead><tr><th>Type</th><th>Span</th><th>Block</th></tr></thead>
            <tbody>
                <tr>
                    <td>text | content</td>
                    <td><code>code | span</code></td>
                    <td><pre>block | content</pre></td>
                </tr>
            </tbody>
        </table>
    "#;

    let markdown = convert(html, Some(default_options())).expect("conversion should succeed");
    assert!(
        markdown.contains("text \\| content"),
        "literal pipe in text cell should be escaped"
    );
    assert!(
        markdown.contains("`code | span`"),
        "pipe inside code span should not be escaped"
    );
    assert!(
        !markdown.contains("`code \\| span`"),
        "code spans must not receive backslash escaping"
    );
    assert!(
        markdown.contains("block | content"),
        "pre/code blocks should retain literal pipe characters"
    );
    assert!(
        !markdown.contains("block \\| content"),
        "pre/code block content should not be escaped"
    );

    let markdown_with_misc = convert(html, Some(escape_misc_options())).expect("conversion should succeed");
    assert!(
        markdown_with_misc.contains("text \\| content"),
        "literal pipe in text cell should be escaped when escape_misc=true"
    );
    assert!(
        markdown_with_misc.contains("`code | span`"),
        "code span pipe should remain unescaped when escape_misc=true"
    );
    assert!(
        !markdown_with_misc.contains("`code \\| span`"),
        "code spans must not be escaped when escape_misc=true"
    );
}

#[test]
fn nested_tables_do_not_double_escape_pipes() {
    let html = r#"
        <table>
            <thead><tr><th>Outer A</th><th>Outer B</th></tr></thead>
            <tbody>
                <tr>
                    <td>
                        <table>
                            <thead><tr><th>Inner A</th><th>Inner B</th></tr></thead>
                            <tbody>
                                <tr><td>Inner | pipe</td><td>B</td></tr>
                            </tbody>
                        </table>
                    </td>
                    <td>Outer | pipe</td>
                </tr>
            </tbody>
        </table>
    "#;

    let markdown = convert(html, Some(default_options())).expect("conversion should succeed");
    assert!(
        markdown.contains("| Inner A | Inner B |"),
        "nested table structure should be preserved"
    );
    assert!(
        markdown.contains("Inner \\| pipe"),
        "literal pipe text inside nested table should be escaped once"
    );
    assert!(
        !markdown.contains("Inner \\\\| pipe"),
        "nested table text should not be double-escaped"
    );
    assert!(
        markdown.contains("Outer \\| pipe"),
        "outer cell literal pipe should be escaped"
    );
    assert!(
        !markdown.contains("Outer \\\\| pipe"),
        "outer cell text should only be escaped once"
    );
}
