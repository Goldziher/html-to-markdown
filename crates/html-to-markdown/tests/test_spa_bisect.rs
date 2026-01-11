#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_spa_first_half() {
    let html = std::fs::read_to_string("/tmp/minimal-failing.html").unwrap();

    let opts = ConversionOptions {
        extract_metadata: false,
        autolinks: false,
        ..Default::default()
    };

    let result = convert(&html, Some(opts)).unwrap();
    eprintln!("Result length: {}", result.len());
    assert!(!result.is_empty());
}
