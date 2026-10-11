#![allow(missing_docs)]
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

#[test]
fn should_keep_stray_table_cell_text_on_every_conversion_path() {
    for (html, expected) in [
        ("<div><td>One</td>items</div>", "Oneitems\n"),
        ("<td>One</td>items", "Oneitems\n"),
        ("<div><th>One</th>items</div>", "Oneitems\n"),
        ("<p>before <td>One</td> after</p>", "before One after\n"),
        ("<div><span>One</span>items</div>", "Oneitems\n"),
        ("<div><td><em>One</em> &amp; two</td>items</div>", "*One* & twoitems\n"),
    ] {
        for options in [
            ConversionOptions::default(),
            ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                ..ConversionOptions::default()
            },
            ConversionOptions {
                extract_metadata: false,
                highlight_style: HighlightStyle::None,
                ..ConversionOptions::default()
            },
        ] {
            let output = convert(html, Some(options))
                .expect("conversion succeeds")
                .content
                .unwrap_or_default();
            assert_eq!(output, expected, "{html}");
        }
    }
}
