use std::fs;
use std::path::PathBuf;

use html_to_markdown_rs::{ConversionOptions, convert};

fn fixture_path(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../test_documents/html/issues", name]
        .iter()
        .collect()
}

fn default_options() -> ConversionOptions {
    let mut options = ConversionOptions::default();
    options.extract_metadata = false;
    options.autolinks = false;
    options
}

fn escape_misc_options() -> ConversionOptions {
    let mut options = ConversionOptions::default();
    options.extract_metadata = false;
    options.autolinks = false;
    options.escape_misc = true;
    options
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
fn converts_nested_tables_fixture() {
    let html = fs::read_to_string(fixture_path("gh-140-nested-tables.html")).unwrap();
    let expected_without_misc = fs::read_to_string(fixture_path("gh-140-nested-tables.md")).unwrap();
    let expected_with_misc = fs::read_to_string(fixture_path("gh-140-nested-tables-with-escape-misc.md")).unwrap();

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
