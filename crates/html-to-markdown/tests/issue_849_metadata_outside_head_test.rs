//! Issue #849: a `<meta>` or a `<link rel="canonical">` element before the body of a page is
//! in the head of the document, with a `<head>` tag on the page or not, as in a browser.

#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn run(html: &str, extract_metadata: bool, tier_strategy: TierStrategy) -> String {
    let options = ConversionOptions {
        extract_metadata,
        tier_strategy,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("the conversion succeeds")
        .content
        .unwrap_or_default()
}

/// The text of the default route, after a check that the full and the fast converter give it too.
fn on_every_route(html: &str, extract_metadata: bool) -> String {
    let auto = run(html, extract_metadata, TierStrategy::Auto);
    let full = run(html, extract_metadata, TierStrategy::Tier2);
    let fast = run(html, extract_metadata, TierStrategy::Tier1);
    assert_eq!(full, auto, "full against default for {html:?}");
    assert_eq!(fast, auto, "fast against default for {html:?}");
    auto
}

/// The text of the default route, after a check that the full converter gives it too.
///
/// ~keep A direct call of the fast converter gives no key for these pages: it reads the head
/// ~keep from a byte range of its own. No public route gives that text.
fn on_the_default_and_the_full_route(html: &str, extract_metadata: bool) -> String {
    let auto = run(html, extract_metadata, TierStrategy::Auto);
    let full = run(html, extract_metadata, TierStrategy::Tier2);
    assert_eq!(full, auto, "full against default for {html:?}");
    auto
}

const META: &str = r#"<meta name="description" content="D">"#;
const DESCRIPTION: &str = "---\nmeta-description: D\n---\n\none\n";

#[test]
fn a_metadata_element_before_the_body_of_a_page_with_no_head_tag_gives_its_key() {
    let cases = [
        format!("{META}<p>one</p>"),
        format!("<!doctype html>{META}<p>one</p>"),
        format!("<html>{META}<body><p>one</p></body></html>"),
        format!("<!-- c -->{META}<p>one</p>"),
    ];
    for html in cases {
        assert_eq!(on_the_default_and_the_full_route(&html, true), DESCRIPTION, "{html:?}");
        assert_eq!(on_every_route(&html, false), "one\n", "{html:?}");
    }
    let link = r#"<link rel="canonical" href="https://e.org/"><p>one</p>"#;
    assert_eq!(
        on_the_default_and_the_full_route(link, true),
        "---\ncanonical: https://e.org/\n---\n\none\n"
    );
    assert_eq!(on_every_route(link, false), "one\n");
}

#[test]
fn a_metadata_element_after_the_head_end_tag_gives_its_key() {
    let html = format!("<head></head>{META}<p>one</p>");
    assert_eq!(on_the_default_and_the_full_route(&html, true), DESCRIPTION);
    assert_eq!(on_every_route(&html, false), "one\n");
}

#[test]
fn a_metadata_element_with_no_head_tag_reads_like_the_element_in_a_head() {
    let in_head = format!("<head>{META}</head><p>one</p>");
    assert_eq!(on_every_route(&in_head, true), DESCRIPTION);
    let with_title = format!("{META}<title>T</title><p>one</p>");
    assert_eq!(
        on_every_route(&with_title, true),
        "---\nmeta-description: D\ntitle: T\n---\n\none\n"
    );
}

#[test]
fn an_element_that_gives_no_key_writes_nothing() {
    let cases = [
        r#"<meta charset="utf-8"><p>one</p>"#,
        r#"<meta name="description"><p>one</p>"#,
        r#"<link rel="stylesheet" href="s.css"><p>one</p>"#,
        "<style>p{}</style><p>one</p>",
        "<script>var a;</script><p>one</p>",
    ];
    for html in cases {
        for extract_metadata in [true, false] {
            assert_eq!(on_every_route(html, extract_metadata), "one\n", "{html:?}");
        }
    }
}

#[test]
fn a_metadata_element_after_body_content_gives_no_key() {
    for extract_metadata in [true, false] {
        assert_eq!(on_every_route(&format!("<p>one</p>{META}"), extract_metadata), "one\n");
        assert_eq!(
            on_every_route(&format!("text{META}<p>one</p>"), extract_metadata),
            "text\n\none\n"
        );
    }
}
