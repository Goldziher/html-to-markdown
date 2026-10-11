#![allow(missing_docs)]

//! Issue #851: a link with `href="#"` whose content is white space or a comment is written as
//! the same link with no content is.

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
fn should_write_a_hash_link_with_white_space_or_a_comment_as_the_link_with_no_content() {
    for options in converters() {
        for content in ["", " ", "<!--x-->", "<span></span>"] {
            let html = format!("<p>a<a href=\"#\">{content}</a>b</p>");
            let output = convert(&html, Some(options.clone()))
                .expect("conversion succeeds")
                .content
                .unwrap_or_default();
            assert_eq!(output, "a[](#)b\n", "{:?}: {html}", options.tier_strategy);
        }
    }
}
