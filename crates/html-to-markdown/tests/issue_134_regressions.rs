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

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

#[test]
fn converts_pre_code_fixture() {
    let html = fs::read_to_string(fixture_path("gh-134-pre-code.html")).unwrap();
    let expected = fs::read_to_string(fixture_path("gh-134-pre-code.md")).unwrap();

    let result = convert(&html, Some(default_options())).unwrap();
    assert_eq!(normalize_newlines(&result), normalize_newlines(&expected));
}
