//! Comment-end-bang closes at its nearest terminator and preserves following content (#905). ~keep
#![cfg(feature = "testkit")]
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)]

use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::tier1;
use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

fn configurations() -> Vec<ConversionOptions> {
    let mut configs = Vec::new();
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            for extract_metadata in [false, true] {
                configs.push(ConversionOptions {
                    output_format,
                    tier_strategy,
                    extract_metadata,
                    ..Default::default()
                });
            }
        }
    }
    assert_eq!(configs.len(), 18);
    configs
}

fn finish(label: &str, rows: usize, expected: usize, failures: &[String]) {
    eprintln!("ROW_COUNT {label} actual={rows} expected={expected}");
    assert_eq!(rows, expected);
    assert!(expected > 0);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn exact_rows(source: &str, expected: &str, failures: &mut Vec<String>) -> usize {
    for config in configurations() {
        let actual = convert(source, config.clone()).expect("comment conversion").content;
        if actual.as_deref() != Some(expected) {
            failures.push(format!("{config:?}: {source}: expected {expected:?}, got {actual:?}"));
        }
    }
    18
}

fn equivalent_rows(source: &str, control: &str, failures: &mut Vec<String>) -> usize {
    for mut config in configurations() {
        config.preprocessing.remove_forms = false;
        let expected = convert(control, config.clone()).expect("control conversion");
        assert!(expected.content.as_deref().is_some_and(|text| !text.is_empty()));
        let actual = convert(source, config.clone()).expect("comment conversion");
        if actual.content != expected.content {
            failures.push(format!(
                "{config:?}: {source}: expected {:?}, got {:?}",
                expected.content, actual.content
            ));
        }
        #[cfg(feature = "metadata")]
        if actual.metadata.document.title != expected.metadata.document.title {
            failures.push(format!("{config:?}: title differs for {source}"));
        }
    }
    36
}

#[test]
fn should_preserve_content_after_the_nearest_comment_end_bang() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for source in [
        "<p>a</p><!-- old note --!><p>b</p><p>c</p>",
        "<p>a</p><!-- old note --!><p>b</p><!-- end --><p>c</p>",
        "<p>a</p><!-- old note ---!><p>b</p><p>c</p>",
        "<p>a</p><!-- first --><p>b</p><!-- second --!><p>c</p>",
    ] {
        rows += exact_rows(source, "a\n\nb\n\nc\n", &mut failures);
    }
    finish("nearest-end-bang", rows, 72, &failures);
}

#[test]
fn should_not_close_a_comment_when_bang_is_followed_by_another_character() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for false_end in ["--!x>", "--!->", "--! >", "-!>"] {
        let source = format!("<p>a</p><!-- note {false_end}<p>hidden</p>--><p>c</p>");
        rows += exact_rows(&source, "a\n\nc\n", &mut failures);
    }
    finish("false-bang-ends", rows, 72, &failures);
}

#[test]
fn should_keep_existing_normal_and_abrupt_empty_comment_closures() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for comment in ["<!-- note -->", "<!-- note --->", "<!-- note ---->", "<!-->", "<!--->"] {
        rows += exact_rows(&format!("<p>a</p>{comment}<p>c</p>"), "a\n\nc\n", &mut failures);
    }
    finish("ordinary-comment-controls", rows, 90, &failures);
}

#[test]
fn should_preserve_comment_lookalikes_in_attributes_and_raw_text() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for (source, control) in [
        (
            "<p><a href=/u title=\"<!-- old --!>\">x</a></p>",
            "<p><a href=/u title=\"&lt;!-- old --!&gt;\">x</a></p>",
        ),
        (
            "<p>a<textarea><!-- old --!></textarea>b</p>",
            "<p>a<textarea>&lt;!-- old --!&gt;</textarea>b</p>",
        ),
        (
            "<p>a</p><style><!-- old --!>p { color: red; }</style><p>c</p>",
            "<p>a</p><style>p { color: red; }</style><p>c</p>",
        ),
    ] {
        rows += equivalent_rows(source, control, &mut failures);
    }
    finish("literal-comment-controls", rows, 108, &failures);
}

#[test]
fn should_preserve_head_metadata_after_comment_end_bang() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for source in [
        "<!-- note --!><html><head><title>Bread</title></head><body><p>Keep.</p></body></html>",
        "<html><head><!-- note --!><title>Bread</title></head><body><p>Keep.</p></body></html>",
    ] {
        let control = source.replace("--!>", "-->");
        rows += equivalent_rows(source, &control, &mut failures);
    }
    finish("comment-head-metadata", rows, 72, &failures);
}

#[test]
fn should_close_comment_end_bang_in_the_direct_fast_converter() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for source in [
        "<p>a</p><!-- note --!><p>b</p><p>c</p>",
        "<p>a</p><!-- note --!><p>b</p><!-- later --><p>c</p>",
    ] {
        for config in configurations()
            .into_iter()
            .filter(|config| config.tier_strategy == TierStrategy::Tier1)
        {
            match tier1::run(source, &PrescanReport::default(), &config) {
                Ok(actual) if actual == "a\n\nb\n\nc\n" => {}
                actual => failures.push(format!("{config:?}: direct scanner {source}: {actual:?}")),
            }
            rows += 1;
        }
    }
    finish("direct-fast-comments", rows, 12, &failures);
}
