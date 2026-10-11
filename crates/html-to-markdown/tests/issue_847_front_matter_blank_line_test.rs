//! Issue #847: a blank line follows the front matter on both converters, also when the body
//! starts with text that is in no element or with an inline element.
//!
//! Each input runs on the default route, on the full converter and on the forced fast
//! converter, and the three texts are compared with each other.

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

#[test]
fn text_after_the_front_matter_starts_after_a_blank_line_on_both_converters() {
    let cases = [
        ("<head><title>T</title></head>tail", "tail\n"),
        ("<html><head><title>T</title></head><body>tail</body></html>", "tail\n"),
        ("<title>T</title>tail", "tail\n"),
        ("<head><title>T</title></head>\n  tail", "tail\n"),
        ("<head><title>T</title></head><em>x</em> tail", "*x* tail\n"),
        (r#"<head><title>T</title></head><a href="/x">L</a>"#, "[L](/x)\n"),
        (
            r#"<head><title>T</title></head><img src="p.gif" alt="A1">"#,
            "![A1](p.gif)\n",
        ),
    ];
    for (html, body) in cases {
        assert_eq!(
            on_every_route(html, true),
            format!("---\ntitle: T\n---\n\n{body}"),
            "{html:?}"
        );
        assert_eq!(on_every_route(html, false), body, "{html:?}");
    }
}

#[test]
fn a_block_after_the_front_matter_starts_after_one_blank_line() {
    let cases = [
        ("<head><title>T</title></head><p>one</p>", "one\n"),
        ("<head><title>T</title></head><h1>H</h1>tail", "# H\n\ntail\n"),
        ("<head><title>T</title></head><ul><li>a</li></ul>", "- a\n"),
    ];
    for (html, body) in cases {
        assert_eq!(
            on_every_route(html, true),
            format!("---\ntitle: T\n---\n\n{body}"),
            "{html:?}"
        );
    }
}

#[test]
fn front_matter_with_no_body_ends_with_its_last_line() {
    assert_eq!(
        on_every_route("<head><title>T</title></head>", true),
        "---\ntitle: T\n---\n"
    );
}
