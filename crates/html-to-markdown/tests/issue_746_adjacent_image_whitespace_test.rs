#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert, tier1};

const CASES: [(&str, &str); 2] = [
    (r#"<img src="A"/> <img src="B"/>"#, "![](A) ![](B)\n"),
    ("<img src=\"A\"/>\n<img src=\"B\"/>", "![](A)\n![](B)\n"),
];

fn tier1_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    }
}

fn convert_with_tier2(html: &str) -> String {
    convert(
        html,
        Some(ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..tier1_options()
        }),
    )
    .expect("Tier-2 conversion should succeed")
    .content
    .unwrap_or_default()
}

#[test]
fn should_preserve_whitespace_between_adjacent_images_through_public_conversion() {
    for (html, expected) in CASES {
        let actual = convert(html, None)
            .expect("public conversion should succeed")
            .content
            .unwrap_or_default();
        assert_eq!(actual, expected, "unexpected output for {html:?}");
    }
}

#[test]
fn should_preserve_whitespace_between_adjacent_images_in_both_tiers() {
    for (html, expected) in CASES {
        let tier1 = tier1::run(html, &PrescanReport::default(), &tier1_options())
            .unwrap_or_else(|reason| panic!("Tier 1 bailed on {html:?}: {reason:?}"));
        let tier2 = convert_with_tier2(html);

        assert_eq!(tier1, expected, "unexpected Tier-1 output for {html:?}");
        assert_eq!(tier2, expected, "unexpected Tier-2 output for {html:?}");
        assert_eq!(tier1, tier2, "tier divergence for {html:?}");
    }
}

#[test]
fn should_still_drop_whitespace_before_the_first_image() {
    let html = " \n<img src=\"A\"/> <img src=\"B\"/>";
    let expected = "![](A) ![](B)\n";
    let public = convert(html, None)
        .expect("public conversion should succeed")
        .content
        .unwrap_or_default();
    let tier1 = tier1::run(html, &PrescanReport::default(), &tier1_options())
        .unwrap_or_else(|reason| panic!("Tier 1 bailed on {html:?}: {reason:?}"));
    let tier2 = convert_with_tier2(html);

    assert_eq!(public, expected);
    assert_eq!(tier1, expected);
    assert_eq!(tier2, expected);
}

#[test]
fn should_not_treat_a_hidden_image_as_rendered_content() {
    let html = "<img hidden src=\"A\"/> <img src=\"B\"/>";
    let expected = "![](B)\n";
    let public = convert(html, None)
        .expect("public conversion should succeed")
        .content
        .unwrap_or_default();
    let tier2 = convert_with_tier2(html);

    assert_eq!(public, expected);
    assert_eq!(tier2, expected);
}
