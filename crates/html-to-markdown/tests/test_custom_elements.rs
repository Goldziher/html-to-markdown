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

#[test]
fn test_custom_elements() {
    let html = fs::read_to_string(fixture_path("test-with-custom-elements.html")).expect("read html");

    eprintln!("HTML: {}", html);

    let result = convert(&html, Some(default_options())).expect("convert html");
    eprintln!("Result: {}", result);

    assert!(result.contains("Team Appraisal"));
    assert!(result.contains("Team"));
    assert!(result.contains("Pending"));
}
