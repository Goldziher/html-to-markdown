//! Coverage for two checks that the `ConversionOptions::hidden_content` change moved without
//! changing them (issue #753).
//!
//! ~keep The first check says whether the caller keeps `<menu>` elements as HTML. The second
//! ~keep reads the text of a link whose content holds a block-level element. Each test fails
//! ~keep when its check returns a constant.

#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

const TIERS: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];

fn convert_with(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

// ~keep The converter writes a `<menu>` in a list item as a `<ul>` before the parse (issue #657),
// ~keep and marks it with an attribute only for a caller that keeps `<menu>`. This caller keeps
// ~keep `<div>`, so the HTML of the `<div>` holds the list with no mark.
#[test]
fn should_not_mark_a_menu_as_kept_when_the_caller_keeps_another_tag() {
    for tier_strategy in TIERS {
        let options = ConversionOptions {
            preserve_tags: vec!["div".to_string()],
            extract_metadata: false,
            tier_strategy,
            ..ConversionOptions::default()
        };
        let actual = convert_with("<div><ul><li>a<menu><li>x</li></menu></li></ul></div>", options);
        assert_eq!(
            actual, "<div><ul><li>a<ul><li>x</li></ul></li></ul></div>\n",
            "{tier_strategy:?}"
        );
    }
}

#[test]
fn should_write_an_autolink_for_a_link_whose_block_content_is_its_address() {
    for tier_strategy in TIERS {
        for (html, expected) in [
            (
                "<a href=\"https://example.com/x\"><div>https://example.com/x</div></a>",
                "<https://example.com/x>\n",
            ),
            (
                "<a href=\"mailto:a@example.com\"><div>a@example.com</div></a>",
                "<a@example.com>\n",
            ),
        ] {
            let options = ConversionOptions {
                extract_metadata: false,
                tier_strategy,
                ..ConversionOptions::default()
            };
            assert_eq!(convert_with(html, options), expected, "{tier_strategy:?} {html}");
        }
    }
}
