//! The five kinds of metadata of a page: document, headers, links, images and structured data.
//!
//! `extract_metadata` is the one public option that sets them. It turns all five on or all five
//! off, so these tests assert each kind with its exact values for both settings.

#![cfg(feature = "metadata")]

use html_to_markdown_rs::metadata::HtmlMetadata;
use html_to_markdown_rs::{ConversionOptions, convert};

/// A page that has one or two entries of each kind of metadata.
const PAGE: &str = concat!(
    r#"<html lang="en"><head><title>Page title</title>"#,
    r#"<script type="application/ld+json">{"@type":"Article"}</script></head><body>"#,
    r#"<h1>First</h1><h2>Second</h2><p><a href="https://example.com/a">link</a></p>"#,
    r#"<p><img src="photo.png" alt="a photo"></p></body></html>"#,
);

fn metadata(extract_metadata: bool) -> HtmlMetadata {
    let options = ConversionOptions {
        extract_metadata,
        ..ConversionOptions::default()
    };
    convert(PAGE, Some(options)).expect("conversion must succeed").metadata
}

#[test]
fn should_collect_each_of_the_five_kinds_when_metadata_is_on() {
    let metadata = metadata(true);

    assert_eq!(metadata.document.title.as_deref(), Some("Page title"));
    let headers: Vec<(u8, &str)> = metadata
        .headers
        .iter()
        .map(|header| (header.level, header.text.as_str()))
        .collect();
    assert_eq!(headers, [(1, "First"), (2, "Second")]);
    let links: Vec<(&str, &str)> = metadata
        .links
        .iter()
        .map(|link| (link.href.as_str(), link.text.as_str()))
        .collect();
    assert_eq!(links, [("https://example.com/a", "link")]);
    assert_eq!(metadata.images.len(), 1);
    assert_eq!(metadata.images[0].src, "photo.png");
    assert_eq!(metadata.structured_data.len(), 1);
    assert!(
        metadata.structured_data[0].raw_json.contains("Article"),
        "{:?}",
        metadata.structured_data[0].raw_json
    );
}

#[test]
fn should_collect_none_of_the_five_kinds_when_metadata_is_off() {
    let metadata = metadata(false);

    assert_eq!(metadata.document.title, None);
    assert_eq!(metadata.document.language, None);
    assert_eq!(metadata.headers.len(), 0);
    assert_eq!(metadata.links.len(), 0);
    assert_eq!(metadata.images.len(), 0);
    assert_eq!(metadata.structured_data.len(), 0);
}
