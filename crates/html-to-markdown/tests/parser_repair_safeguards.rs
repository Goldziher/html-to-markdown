#![allow(missing_docs, clippy::print_stdout)]
#![cfg(feature = "testkit")]

use std::fmt::Write;

use html_to_markdown_rs::{ConversionOptions, convert};

fn formatting_tags(count: usize) -> String {
    (0..count).fold(String::new(), |mut bold, id| {
        write!(bold, "<b id='b{id}'>").expect("writing to String succeeds");
        bold
    })
}

#[test]
fn amplified_repair_should_fall_back_without_losing_authored_text() {
    let bold = formatting_tags(250);
    for paragraphs in [1_000, 4_000] {
        let html = format!("<p>{bold}x</p>{}", "<p>y</p>".repeat(paragraphs));
        let result = convert(
            &html,
            ConversionOptions {
                max_depth: Some(512),
                ..Default::default()
            },
        )
        .expect("amplified input converts");
        let content = result.content.expect("Markdown exists");
        assert_eq!(content.matches('x').count(), 1);
        assert_eq!(content.matches('y').count(), paragraphs);
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.kind == html_to_markdown_rs::WarningKind::MalformedHtml)
        );
    }
}

#[test]
fn ordinary_formatting_repair_should_keep_all_text_without_a_budget_warning() {
    for (formatting, paragraphs) in [(5, 200), (250, 2)] {
        let bold = formatting_tags(formatting);
        let html = format!("<p>{bold}x</p>{}", "<p>y</p>".repeat(paragraphs));
        let result = convert(
            &html,
            ConversionOptions {
                max_depth: Some(512),
                ..Default::default()
            },
        )
        .expect("ordinary formatting converts");
        let content = result.content.expect("Markdown exists");
        assert_eq!(content.matches('x').count(), 1);
        assert_eq!(content.matches('y').count(), paragraphs);
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    }
}

/// A page with a `<title>` and no `<head>` tag (issue #816), then one paragraph with open bold
/// elements and `paragraphs` paragraphs after it.
fn page_with_a_title_and_no_head(formatting: usize, paragraphs: usize) -> String {
    format!(
        "<title>T</title><p>{}x</p>{}",
        formatting_tags(formatting),
        "<p>y</p>".repeat(paragraphs)
    )
}

fn convert_with_metadata(html: &str, extract_metadata: bool) -> html_to_markdown_rs::ConversionResult {
    convert(
        html,
        ConversionOptions {
            extract_metadata,
            max_depth: Some(512),
            ..Default::default()
        },
    )
    .expect("a page with a title and no head converts")
}

#[test]
fn should_take_a_title_with_no_head_tag_as_the_document_title_inside_the_node_budget() {
    let html = page_with_a_title_and_no_head(2, 2);

    let with_metadata = convert_with_metadata(&html, true);
    assert_eq!(
        with_metadata.content.as_deref(),
        Some("---\ntitle: T\n---\n\n**x**\n\n**y**\n\n**y**\n")
    );
    assert!(with_metadata.warnings.is_empty(), "{:?}", with_metadata.warnings);

    let without_metadata = convert_with_metadata(&html, false);
    assert_eq!(without_metadata.content.as_deref(), Some("**x**\n\n**y**\n\n**y**\n"));
    assert!(without_metadata.warnings.is_empty(), "{:?}", without_metadata.warnings);
}

#[test]
fn should_keep_a_title_with_no_head_tag_in_the_body_past_the_node_budget() {
    // ~keep The repair that moves the title into the head has the node budget (issue #809).
    // ~keep Past the budget the page keeps its structure as written: the title text is the
    // ~keep first paragraph and there is no front matter.
    for (formatting, paragraphs) in [(60, 12), (250, 1_000), (250, 4_000)] {
        let html = page_with_a_title_and_no_head(formatting, paragraphs);
        let expected = format!("T\n\n**x**\n{}", "\n**y**\n".repeat(paragraphs));
        for extract_metadata in [true, false] {
            let result = convert_with_metadata(&html, extract_metadata);
            assert_eq!(
                result.content.as_deref(),
                Some(expected.as_str()),
                "{formatting} bold elements, {paragraphs} paragraphs, metadata {extract_metadata}"
            );
            assert!(
                result.warnings.iter().any(
                    |warning| warning.kind == html_to_markdown_rs::WarningKind::MalformedHtml
                        && warning.message.contains("node budget")
                ),
                "{:?}",
                result.warnings
            );
        }
    }
}

#[test]
fn should_keep_the_shallow_text_and_warn_when_the_page_is_deeper_than_the_walker_limit() {
    for prefix in ["<table><td>visible", "<b><p>visible</p></b>", "<x-custom>visible"] {
        let html = format!("{prefix}{}tail", "<div>".repeat(20_000));
        let result = convert(&html, ConversionOptions::default()).expect("deep input converts");
        let content = result.content.expect("Markdown exists");
        assert_eq!(content.matches("visible").count(), 1, "{content:?}");
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.kind == html_to_markdown_rs::WarningKind::DepthLimitExceeded)
        );
    }
}

#[test]
fn should_keep_every_word_when_the_repair_stops_at_its_depth_limit() {
    let open = "<div>".repeat(600);
    let close = "</div>".repeat(600);
    for prefix in ["<table><td>visible", "<b><p>visible</p></b>", "<x-custom>visible"] {
        for html in [format!("{prefix}{open}deep{close}tail"), format!("{prefix}{open}tail")] {
            let result = convert(
                &html,
                ConversionOptions {
                    max_depth: Some(1_000),
                    ..Default::default()
                },
            )
            .expect("deep input converts");
            let content = result.content.expect("Markdown exists");
            let shape = &html[..prefix.len() + 5];
            assert!(content.contains("visible"), "{shape}: {content:?}");
            assert_eq!(content.matches("tail").count(), 1, "{shape}: {content:?}");
            assert_eq!(
                content.matches("deep").count(),
                html.matches("deep").count(),
                "{shape}: {content:?}"
            );
            assert!(
                result.warnings.iter().any(
                    |warning| warning.kind == html_to_markdown_rs::WarningKind::MalformedHtml
                        && warning.message.contains("depth limit of 512")
                ),
                "{shape}: {:?}",
                result.warnings
            );
        }
    }
}

fn fastest_deep_repair(prefix: &str, count: usize) -> std::time::Duration {
    let html = format!("{prefix}{}tail", "<div>".repeat(count));
    (0..3)
        .map(|_| {
            let start = std::time::Instant::now();
            let result = convert(&html, ConversionOptions::default()).expect("deep input converts");
            assert_eq!(
                result.content.as_deref().unwrap_or_default().matches("visible").count(),
                1
            );
            assert!(
                result
                    .warnings
                    .iter()
                    .any(|warning| warning.kind == html_to_markdown_rs::WarningKind::DepthLimitExceeded)
            );
            start.elapsed()
        })
        .min()
        .expect("three measurements")
}

#[test]
fn deep_repair_should_scale_linearly_on_every_fallback_route() {
    for prefix in ["<table><td>visible", "<b><p>visible</p></b>", "<x-custom>visible"] {
        let mut measurements = Vec::new();
        for _ in 0..3 {
            let small = fastest_deep_repair(prefix, 5_000).as_secs_f64();
            let medium = fastest_deep_repair(prefix, 10_000).as_secs_f64();
            let large = fastest_deep_repair(prefix, 20_000).as_secs_f64();
            measurements.push((small, medium, large));
            if medium < small * 3.0 && large < medium * 3.0 {
                break;
            }
        }
        println!("{prefix}: {measurements:?}");
        assert!(
            measurements
                .iter()
                .any(|(small, medium, large)| *medium < small * 3.0 && *large < medium * 3.0),
            "{measurements:?}"
        );
    }
}

#[test]
fn should_keep_style_text_hidden_when_the_repair_stops_at_a_style_tag() {
    for depth in 500..=520 {
        let html = format!(
            "<b><p>visible</p></b>{}<style>hidden-style</style>tail",
            "<div>".repeat(depth)
        );
        let result = convert(
            &html,
            ConversionOptions {
                max_depth: Some(1_000),
                ..Default::default()
            },
        )
        .expect("deep input converts");
        let content = result.content.expect("Markdown exists");
        assert!(!content.contains("hidden-style"), "depth {depth}: {content:?}");
        assert_eq!(content.matches("tail").count(), 1, "depth {depth}: {content:?}");
    }
}

#[test]
fn should_convert_three_hundred_open_formatting_elements_inside_the_repair_limit() {
    let mut html = String::from("<p>");
    for id in 0..300 {
        write!(html, "<b id='b{id}'>").expect("write to a string");
    }
    html.push('x');
    let result = convert(
        &html,
        ConversionOptions {
            max_depth: Some(1_000),
            ..Default::default()
        },
    )
    .expect("open formatting converts");
    assert_eq!(result.content.as_deref(), Some("**x**\n"));
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
}
