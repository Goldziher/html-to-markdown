#![allow(missing_docs)]

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
fn test_nested_simple() {
    let html = fs::read_to_string(fixture_path("test-nested-simple.html")).expect("read html");
    let expected = fs::read_to_string(fixture_path("test-nested-simple.md")).expect("read markdown");

    eprintln!("HTML: {}", html);
    eprintln!("Expected: {}", expected);

    let result = convert(&html, Some(default_options())).expect("convert html");
    eprintln!("Result: {}", result);

    assert_eq!(normalize_newlines(&result), normalize_newlines(&expected));
}
