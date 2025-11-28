use std::fs;
use std::path::PathBuf;

use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions, HeadingStyle, HighlightStyle, ListIndentType, PreprocessingOptions,
    PreprocessingPreset, WhitespaceMode, convert,
};

fn fixture_path(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../test_documents/html/issues", name]
        .iter()
        .collect()
}

fn issue_127_options() -> ConversionOptions {
    let mut options = ConversionOptions::default();
    options.heading_style = HeadingStyle::Atx;
    options.bullets = "-".to_string();
    options.list_indent_type = ListIndentType::Spaces;
    options.list_indent_width = 2;
    options.whitespace_mode = WhitespaceMode::Normalized;
    options.highlight_style = HighlightStyle::DoubleEqual;
    options.wrap = false;
    options.br_in_tables = true;
    options.code_block_style = CodeBlockStyle::Backticks;
    options.strip_newlines = true;
    options.extract_metadata = false;
    options.hocr_spatial_tables = true;
    options.preprocessing = PreprocessingOptions {
        enabled: true,
        preset: PreprocessingPreset::Minimal,
        remove_navigation: true,
        remove_forms: true,
    };
    options
}

#[test]
fn converts_multilingual_fixture_without_utf8_boundary_panic() {
    let html = fs::read_to_string(fixture_path("gh-127-issue.html")).expect("read issue fixture");

    let markdown = convert(&html, Some(issue_127_options())).expect("convert should not panic on utf-8 boundaries");

    assert!(!markdown.is_empty(), "converted output should contain content");
    assert!(
        markdown.contains("MW841") && markdown.contains("كريب"),
        "converted output should preserve multilingual product content"
    );
}
