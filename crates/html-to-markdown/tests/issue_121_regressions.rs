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
fn converts_spa_menu_fixture() {
    let html = fs::read_to_string(fixture_path("gh-121-spa-app.html")).expect("read spa html");
    let expected = fs::read_to_string(fixture_path("gh-121-spa-app.md")).expect("read spa markdown");

    let result = convert(&html, Some(default_options())).expect("convert spa html");
    assert_eq!(normalize_newlines(&result), normalize_newlines(&expected));
}

#[test]
fn converts_hacker_news_fixture() {
    let html = fs::read_to_string(fixture_path("gh-121-hacker-news.html")).expect("read hn html");
    let expected = fs::read_to_string(fixture_path("gh-121-hacker-news.md")).expect("read hn markdown");

    let result = convert(&html, Some(default_options())).expect("convert hn html");
    assert_eq!(normalize_newlines(&result), normalize_newlines(&expected));
}
