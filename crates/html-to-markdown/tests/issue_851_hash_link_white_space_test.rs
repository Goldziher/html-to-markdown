#![allow(missing_docs)]

//! ~keep Hash-only links keep empty labels (#851), with source whitespace outside the link (#929).

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

/// The converters that a caller can force: the full one, and the fast one with `testkit`.
#[cfg(feature = "testkit")]
const FORCED: [TierStrategy; 2] = [TierStrategy::Tier2, TierStrategy::Tier1];
#[cfg(not(feature = "testkit"))]
const FORCED: [TierStrategy; 1] = [TierStrategy::Tier2];

/// The options that let the router choose the fast converter.
fn metadata_off() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    }
}

fn converters() -> Vec<ConversionOptions> {
    let forced = FORCED.into_iter().map(|tier_strategy| ConversionOptions {
        tier_strategy,
        ..metadata_off()
    });
    [ConversionOptions::default(), metadata_off()]
        .into_iter()
        .chain(forced)
        .collect()
}

#[test]
fn should_keep_hash_link_labels_empty_and_preserve_source_whitespace() {
    for options in converters() {
        for (content, expected) in [
            ("", "a[](#)b\n"),
            (" ", "a [](#)b\n"),
            ("<!--x-->", "a[](#)b\n"),
            ("<span></span>", "a[](#)b\n"),
            ("<i> </i>", "a [](#)b\n"),
        ] {
            let html = format!("<p>a<a href=\"#\">{content}</a>b</p>");
            let output = convert(&html, Some(options.clone()))
                .expect("conversion succeeds")
                .content
                .unwrap_or_default();
            assert_eq!(output, expected, "{:?}: {html}", options.tier_strategy);
        }
    }
}
