#![allow(missing_docs, clippy::print_stdout)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

#[test]
fn omitted_head_end_should_preserve_body_without_repeating_title() {
    let inputs = [
        "<html><head><meta charset=utf-8><title>t</title><h1>Shown</h1>",
        "<html><head><title>t</title><h1>Shown</h1>",
        "<head><meta charset=utf-8><title>t</title><h1>Shown</h1>",
        "<html><head><meta charset=utf-8><title>t</title><h1>Shown</h1></html>",
        "<html><head><meta charset=utf-8><title>t</title></head><h1>Shown</h1>",
        "<html><head><meta charset=utf-8><title>t</title><body><h1>Shown</h1>",
        "<!doctype html><html><head><meta charset=utf-8><title>t</title><h1>Shown</h1><p>Text</p>",
        "<html><head><meta charset=utf-8><h1>Shown</h1><p>Text</p>",
    ];
    for html in inputs {
        for tier in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let result = convert(
                html,
                ConversionOptions {
                    tier_strategy: tier,
                    ..Default::default()
                },
            )
            .expect("document converts");
            let content = result.content.expect("Markdown exists");
            assert!(content.contains("# Shown\n"), "{tier:?}: {html}: {content:?}");
            assert!(!content.lines().any(|line| line == "t"), "{tier:?}: {content:?}");
            if html.contains("<title>") {
                assert!(content.contains("title: t\n"), "{tier:?}: {content:?}");
            }
            if html.contains("<p>Text") {
                assert!(content.contains("Text\n"), "{tier:?}: {content:?}");
            }
        }
    }
}

#[test]
fn parent_close_before_paragraph_close_should_keep_browser_boundaries() {
    let cases = [
        (
            "<!doctype html><div><p>one</div>two</p>rest</div>tail",
            "one\n\ntwo\n\nresttail\n",
        ),
        (
            "<!doctype html><div><p>one</div><div>two</p></div>tail",
            "one\n\ntwo\n\ntail\n",
        ),
        ("<!doctype html><div><p>one</div>two</p>three", "one\n\ntwo\n\nthree\n"),
    ];
    for whitespace in ["\n", "\t", "\r", "\u{000c}", "/"] {
        let html = format!("<div><p{whitespace}class=x>one</div>two</p>rest</div>tail");
        let result = convert(&html, ConversionOptions::default()).expect("document converts");
        assert_eq!(result.content.as_deref(), Some("one\n\ntwo\n\nresttail\n"), "{html}");
    }
    for (html, expected) in cases {
        for tier in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let result = convert(
                html,
                ConversionOptions {
                    tier_strategy: tier,
                    ..Default::default()
                },
            )
            .expect("document converts");
            assert_eq!(result.content.as_deref(), Some(expected), "{tier:?}: {html}");
        }
    }
}

#[test]
fn omitted_paragraph_end_should_convert_as_the_written_end_does() {
    let pairs = [
        (
            "<ul><li><p>one</li><li>two</li></ul>after",
            "<ul><li><p>one</p></li><li>two</li></ul>after",
        ),
        (
            "<div><p>one</div><p>two</p>after",
            "<div><p>one</p></div><p>two</p>after",
        ),
        (
            "<blockquote><p>one</blockquote>after",
            "<blockquote><p>one</p></blockquote>after",
        ),
    ];
    let mut differences = Vec::new();
    for (omitted, written) in pairs {
        for tier in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let content = |html: &str| {
                let options = ConversionOptions {
                    tier_strategy: tier,
                    ..Default::default()
                };
                convert(html, options).expect("document converts").content
            };
            let (left, right) = (content(omitted), content(written));
            if left != right {
                differences.push(format!("{tier:?}: {omitted}: {left:?} != {right:?}"));
            }
        }
    }
    assert!(differences.is_empty(), "{differences:#?}");
}
