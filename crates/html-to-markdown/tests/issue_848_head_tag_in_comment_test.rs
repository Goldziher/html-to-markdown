//! Issue #848: a `<head>` or a `<body>` tag inside a comment is no tag. A title before the
//! body of such a page is the document title, as on the page without the comment.

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
fn a_head_tag_in_a_comment_does_not_keep_a_title_in_the_body() {
    let cases = [
        "<!-- <head> --><title>T</title><p>one</p>",
        "<head></head><!-- <body> --><title>T</title><p>one</p>",
        "<!-- <head> --><!-- <body> --><title>T</title><p>one</p>",
        "<!-- </head> --><head><title>T</title></head><p>one</p>",
        "<!-- <HEAD> --><TITLE>T</TITLE><P>one</P>",
    ];
    for html in cases {
        assert_eq!(on_every_route(html, true), "---\ntitle: T\n---\n\none\n", "{html:?}");
        assert_eq!(on_every_route(html, false), "one\n", "{html:?}");
    }
}

#[test]
fn a_title_in_a_comment_is_no_title() {
    for extract_metadata in [true, false] {
        assert_eq!(
            on_every_route("<!-- <title>X</title> --><p>one</p>", extract_metadata),
            "one\n"
        );
    }
}

#[test]
fn a_tag_with_a_longer_name_is_not_a_head_tag() {
    // ~keep A `header` starts the body, so the title after it stays where it is.
    for extract_metadata in [true, false] {
        assert_eq!(
            on_every_route("<header>h</header><title>T</title><p>one</p>", extract_metadata),
            "h\n\nT\n\none\n"
        );
    }
}
