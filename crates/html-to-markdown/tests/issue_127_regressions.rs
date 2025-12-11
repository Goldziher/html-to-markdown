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
    ConversionOptions {
        heading_style: HeadingStyle::Atx,
        bullets: "-".to_string(),
        list_indent_type: ListIndentType::Spaces,
        list_indent_width: 2,
        whitespace_mode: WhitespaceMode::Normalized,
        highlight_style: HighlightStyle::DoubleEqual,
        wrap: false,
        br_in_tables: true,
        code_block_style: CodeBlockStyle::Backticks,
        strip_newlines: true,
        extract_metadata: false,
        hocr_spatial_tables: true,
        preprocessing: PreprocessingOptions {
            enabled: true,
            preset: PreprocessingPreset::Minimal,
            remove_navigation: true,
            remove_forms: true,
        },
        ..Default::default()
    }
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
