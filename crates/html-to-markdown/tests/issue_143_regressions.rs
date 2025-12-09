use std::fs;
use std::path::PathBuf;

use html_to_markdown_rs::{ConversionOptions, convert};

fn fixture_path(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../test_documents/html/issues", name]
        .iter()
        .collect()
}

fn options_with_wrap() -> ConversionOptions {
    ConversionOptions {
        wrap: true,
        wrap_width: 80,
        extract_metadata: false,
        autolinks: false,
        ..Default::default()
    }
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

#[test]
fn wrap_preserves_link_only_list_items() {
    let html = fs::read_to_string(fixture_path("gh-143-links-wordwrap.html")).unwrap();
    let expected = fs::read_to_string(fixture_path("gh-143-links-wordwrap.md")).unwrap();

    let result = convert(&html, Some(options_with_wrap())).expect("conversion should succeed");

    assert_eq!(
        normalize_newlines(&result).trim(),
        normalize_newlines(&expected).trim(),
        "word wrapping should not merge nested link-only list items"
    );
}
