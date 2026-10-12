#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

#[test]
fn emphasis_should_keep_adjacent_markers_on_a_long_unicode_line() {
    for (tag, marker) in [("i", "*"), ("strong", "**")] {
        for n in [100, 400] {
            let html = format!("<div>{}</div>", format!("<{tag}>界</{tag}>").repeat(n));
            let options = ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                extract_metadata: false,
                ..Default::default()
            };
            let content = convert(&html, options)
                .expect("convert emphasis")
                .content
                .expect("content");
            assert_eq!(content, format!("{marker}{}{marker}\n", "界".repeat(n)));
        }
    }
}
