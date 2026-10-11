//! Issue #816: a `<title>` before the body content of a page with no `<head>` tag is in the
//! head of the document, as in a browser. It is metadata, not body text.
//!
//! Each input runs through the public entry with the default route, with the full converter
//! and with the fast converter forced. A forced fast run that hands the page over returns the
//! text of the full converter, so the three texts must be equal.

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

/// The text of the default route, after a check that the full and the fast route give it too.
fn on_every_route(html: &str, extract_metadata: bool) -> String {
    let auto = run(html, extract_metadata, TierStrategy::Auto);
    let full = run(html, extract_metadata, TierStrategy::Tier2);
    let fast = run(html, extract_metadata, TierStrategy::Tier1);
    assert_eq!(full, auto, "full against default for {html:?}");
    assert_eq!(fast, auto, "fast against default for {html:?}");
    auto
}

const WITHOUT_HEAD: &str = "<title>T</title><div><p>one</p></div>tail";
const WITH_HEAD: &str = "<head><title>T</title></head><div><p>one</p></div>tail";

#[test]
fn a_title_with_no_head_tag_is_metadata_when_metadata_is_on() {
    let expected = "---\ntitle: T\n---\n\none\n\ntail\n";
    assert_eq!(on_every_route(WITHOUT_HEAD, true), expected);
    assert_eq!(on_every_route(WITH_HEAD, true), expected);
}

#[test]
fn a_title_with_no_head_tag_is_not_body_text_when_metadata_is_off() {
    let expected = "one\n\ntail\n";
    assert_eq!(on_every_route(WITHOUT_HEAD, false), expected);
    assert_eq!(on_every_route(WITH_HEAD, false), expected);
}

#[test]
fn a_title_with_no_head_tag_reads_like_the_page_with_one() {
    let pairs = [
        (
            "<!DOCTYPE html><title>T</title><p>one</p>",
            "<!DOCTYPE html><head><title>T</title></head><p>one</p>",
        ),
        (
            "<html><title>T</title><body><p>one</p></body></html>",
            "<html><head><title>T</title></head><body><p>one</p></body></html>",
        ),
        ("<TITLE>T</TITLE><P>one</P>", "<HEAD><TITLE>T</TITLE></HEAD><P>one</P>"),
        (
            "<title>A</title><title>B</title><p>one</p>",
            "<head><title>A</title><title>B</title></head><p>one</p>",
        ),
        ("<title></title><p>one</p>", "<head><title></title></head><p>one</p>"),
        (
            "<title>T &amp; U</title><p>tail</p>",
            "<head><title>T &amp; U</title></head><p>tail</p>",
        ),
    ];
    for (without_head, with_head) in pairs {
        for extract_metadata in [true, false] {
            assert_eq!(
                on_every_route(without_head, extract_metadata),
                on_every_route(with_head, extract_metadata),
                "{without_head:?} with extract_metadata = {extract_metadata}"
            );
        }
    }
}

#[test]
fn the_first_of_two_titles_is_the_document_title() {
    let markdown = on_every_route("<title>A</title><title>B</title><p>one</p>", true);
    assert_eq!(markdown, "---\ntitle: A\n---\n\none\n");
}

#[test]
fn an_empty_title_writes_no_front_matter_block() {
    assert_eq!(on_every_route("<title></title><p>one</p>", true), "one\n");
    assert_eq!(on_every_route("<head><title></title></head><p>one</p>", true), "one\n");
}

#[test]
fn a_title_after_body_content_is_not_moved_to_the_head() {
    // ~keep A browser keeps this `title` element in the body, after the paragraph.
    for extract_metadata in [true, false] {
        assert_eq!(
            on_every_route("<p>one</p><title>T</title>", extract_metadata),
            "one\n\nT\n"
        );
    }
}

#[test]
fn a_title_before_bare_text_reads_like_the_page_with_a_head_on_each_route() {
    // ~keep The two converters differ on the blank line between the front matter and bare
    // ~keep text, with a head tag too. Each route is compared with itself.
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2, TierStrategy::Tier1] {
        for extract_metadata in [true, false] {
            let without_head = run("<title>T</title>tail", extract_metadata, tier_strategy);
            let with_head = run("<head><title>T</title></head>tail", extract_metadata, tier_strategy);
            assert_eq!(without_head, with_head, "{tier_strategy:?}, {extract_metadata}");
            assert!(without_head.ends_with("tail\n"), "{without_head:?}");
            assert_eq!(without_head.starts_with("---\ntitle: T\n---\n"), extract_metadata);
        }
    }
}

#[test]
fn a_title_in_a_drawing_is_not_the_document_title() {
    // ~keep The title of a drawing is the alternative text of its image.
    let html = r#"<svg width="4" height="4"><title>T</title><circle r="1"/></svg><p>one</p>"#;
    let image = "![T](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjQiIHdpZHRoPSI0Ij48dGl0bGU+VDwvdGl0bGU+PGNpcmNsZSByPSIxIiAvPjwvc3ZnPg==)";
    for extract_metadata in [true, false] {
        assert_eq!(on_every_route(html, extract_metadata), format!("{image}\n\none\n"));
    }
}
