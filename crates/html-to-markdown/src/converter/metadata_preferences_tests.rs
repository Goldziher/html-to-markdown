//! Each of the five metadata choices of the conversion context turns on its own kind only.
//!
//! The public options turn all five on or off together. These tests give the converter a
//! collector that asks for one kind, so a choice that reads the flag of another kind fails.

use std::cell::RefCell;
use std::rc::Rc;

use super::main::ConversionParameters;
use crate::ConversionOptions;
use crate::metadata::{DEFAULT_MAX_STRUCTURED_DATA_SIZE, HtmlMetadata, MetadataCollector, MetadataConfig};

/// A page that has one or two entries of each kind of metadata.
const PAGE: &str = concat!(
    r#"<html lang="en"><head><title>Page title</title>"#,
    r#"<script type="application/ld+json">{"@type":"Article"}</script></head><body>"#,
    r#"<h1>First</h1><h2>Second</h2><p><a href="https://example.com/a">link</a></p>"#,
    r#"<p><img src="photo.png" alt="a photo"></p></body></html>"#,
);

const NOTHING: MetadataConfig = MetadataConfig {
    extract_document: false,
    extract_headers: false,
    extract_links: false,
    extract_images: false,
    extract_structured_data: false,
    max_structured_data_size: DEFAULT_MAX_STRUCTURED_DATA_SIZE,
};

/// The counts of a result: document title, headers, links, images, structured data.
fn collect(config: MetadataConfig) -> [usize; 5] {
    let collector = Rc::new(RefCell::new(MetadataCollector::new(config)));
    let parameters = ConversionParameters {
        inline_collector: None,
        metadata_collector: Some(Rc::clone(&collector)),
        #[cfg(feature = "visitor")]
        visitor: None,
        structure_collector: None,
        base_url: None,
        document_base_href: None,
    };
    super::convert_html_impl(PAGE, &ConversionOptions::default(), parameters).expect("conversion must succeed");
    let Ok(collector) = Rc::try_unwrap(collector) else {
        panic!("the conversion must release the collector");
    };
    let metadata: HtmlMetadata = collector.into_inner().finish();
    [
        usize::from(metadata.document.title.as_deref() == Some("Page title")),
        metadata.headers.len(),
        metadata.links.len(),
        metadata.images.len(),
        metadata.structured_data.len(),
    ]
}

#[test]
fn should_collect_only_the_one_kind_that_is_on() {
    let document = MetadataConfig {
        extract_document: true,
        ..NOTHING
    };
    assert_eq!(collect(document), [1, 0, 0, 0, 0], "document");
    let headers = MetadataConfig {
        extract_headers: true,
        ..NOTHING
    };
    assert_eq!(collect(headers), [0, 2, 0, 0, 0], "headers");
    let links = MetadataConfig {
        extract_links: true,
        ..NOTHING
    };
    assert_eq!(collect(links), [0, 0, 1, 0, 0], "links");
    let images = MetadataConfig {
        extract_images: true,
        ..NOTHING
    };
    assert_eq!(collect(images), [0, 0, 0, 1, 0], "images");
    let structured_data = MetadataConfig {
        extract_structured_data: true,
        ..NOTHING
    };
    assert_eq!(collect(structured_data), [0, 0, 0, 0, 1], "structured data");
}

#[test]
fn should_collect_all_five_kinds_or_none() {
    assert_eq!(collect(MetadataConfig::default()), [1, 2, 1, 1, 1], "all on");
    assert_eq!(collect(NOTHING), [0, 0, 0, 0, 0], "all off");
}
