//! Issue #850: a page that the converter rebuilds with the HTML tree builder keeps the blank
//! first line of a `<pre>`. The parser drops one line feed after the start tag, and the page
//! that is written back has that line feed again.

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
fn a_rebuilt_page_keeps_the_blank_first_line_of_a_pre() {
    // ~keep A title with no head tag makes the converter rebuild the page.
    let rebuilt = "<title>T</title><pre>\n\nx\n</pre>";
    let with_head = "<head><title>T</title></head><pre>\n\nx\n</pre>";
    assert_eq!(on_every_route(rebuilt, true), "---\ntitle: T\n---\n\n```\n\nx\n```\n");
    assert_eq!(on_every_route(rebuilt, false), "```\n\nx\n```\n");
    for extract_metadata in [true, false] {
        assert_eq!(
            on_every_route(rebuilt, extract_metadata),
            on_every_route(with_head, extract_metadata)
        );
    }
}

#[test]
fn a_rebuilt_page_reads_like_the_page_with_a_head_for_each_count_of_line_feeds() {
    for line_feeds in ["", "\n", "\n\n", "\n\n\n", "\r\n\r\n"] {
        for (open, close) in [("<pre>", "</pre>"), ("<pre><code>", "</code></pre>")] {
            let rebuilt = format!("<title>T</title>{open}{line_feeds}x\n{close}");
            let with_head = format!("<head><title>T</title></head>{open}{line_feeds}x\n{close}");
            assert_eq!(
                on_every_route(&rebuilt, false),
                on_every_route(&with_head, false),
                "{rebuilt:?}"
            );
        }
    }
}

#[test]
fn a_page_rebuilt_for_an_omitted_end_tag_keeps_the_blank_first_line_of_a_pre() {
    let html = "<div><p>one</div><pre>\n\nx\n</pre>";
    assert_eq!(on_every_route(html, false), "one\n\n```\n\nx\n```\n");
}
