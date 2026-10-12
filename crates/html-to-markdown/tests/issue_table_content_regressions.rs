#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, OutputFormat, TierStrategy, convert};

fn content(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion succeeds")
        .content
        .unwrap_or_default()
}

#[test]
fn should_omit_hidden_table_rows_from_plain_text() {
    for wrapper in ["template", "script", "style", "noscript"] {
        let html = format!(
            "<table><tr><th>Name</th></tr><{wrapper}><tr><td>hidden row</td></tr></{wrapper}><tr><td>Ada</td></tr></table>"
        );
        let options = ConversionOptions {
            output_format: OutputFormat::Plain,
            ..ConversionOptions::default()
        };
        assert_eq!(content(&html, options), "Name\nAda\n", "{wrapper}");
    }
}

#[test]
fn should_pad_short_header_and_body_rows_on_both_conversion_paths() {
    for (html, expected) in [
        (
            "<table><tr><th>Name</th></tr><tr><td>Ada</td><td>Lovelace</td></tr></table>",
            "| Name |          |\n| ---- | -------- |\n| Ada  | Lovelace |\n",
        ),
        (
            "<table><tr><th>Name</th><th>Surname</th></tr><tr><td>Ada</td></tr></table>",
            "| Name | Surname |\n| ---- | ------- |\n| Ada  |         |\n",
        ),
        (
            "<table><caption>Names</caption><tr><td>Name</td></tr><tr><td>Ada</td><td>Lovelace</td></tr></table>",
            "*Names*\n\n| Name |          |\n| ---- | -------- |\n| Ada  | Lovelace |\n",
        ),
    ] {
        let fast = ConversionOptions {
            extract_metadata: false,
            highlight_style: HighlightStyle::None,
            ..ConversionOptions::default()
        };
        let slow = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..fast.clone()
        };
        assert_eq!(content(html, fast), expected, "fast: {html}");
        assert_eq!(content(html, slow), expected, "slow: {html}");
    }
}

#[test]
fn should_flatten_nested_tables_without_inventing_table_syntax() {
    let html = "<table><tr><td>outer a<table><tr><td>inner a</td><td>inner b</td></tr><tr><td>inner c</td><td>inner d</td></tr></table></td><td>outer b</td></tr></table>";
    for br in [false, true] {
        for tier in [TierStrategy::Auto, TierStrategy::Tier2] {
            let options = ConversionOptions {
                extract_metadata: false,
                highlight_style: HighlightStyle::None,
                compact_tables: true,
                br_in_tables: br,
                tier_strategy: tier,
                ..ConversionOptions::default()
            };
            let separator = if br { "<br>" } else { " " };
            let expected =
                format!("| outer a{separator}inner a inner b{separator}inner c inner d | outer b |\n| --- | --- |\n");
            assert_eq!(content(html, options), expected, "br={br}, tier={tier:?}");
        }
    }
}

#[test]
fn should_preserve_nested_caption_and_link_content_without_inner_table_syntax() {
    let html = "<table><tr><td>before<table><caption>Names</caption><tr><td><a href=\"/ada\">Ada</a></td><td>Lovelace</td></tr></table></td><td>after</td></tr></table>";
    for tier in [TierStrategy::Auto, TierStrategy::Tier2] {
        let options = ConversionOptions {
            extract_metadata: false,
            highlight_style: HighlightStyle::None,
            compact_tables: true,
            tier_strategy: tier,
            ..ConversionOptions::default()
        };
        assert_eq!(
            content(html, options),
            "| before Names [Ada](/ada) Lovelace | after |\n| --- | --- |\n"
        );
    }
}
