#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

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

#[test]
fn semantic_italics_should_keep_adjacent_markers_on_a_long_unicode_line() {
    for tag in ["var", "dfn"] {
        for n in [100, 400] {
            for output_format in [OutputFormat::Markdown, OutputFormat::Plain] {
                let html = format!("<div>{}</div>", format!("<{tag}>界</{tag}>").repeat(n));
                let options = ConversionOptions {
                    output_format,
                    tier_strategy: TierStrategy::Tier2,
                    extract_metadata: false,
                    ..Default::default()
                };
                let content = convert(&html, options)
                    .expect("convert semantic italics")
                    .content
                    .expect("content");
                let expected = if output_format == OutputFormat::Markdown {
                    format!("*{}*\n", "界".repeat(n))
                } else {
                    format!("{}\n", "界".repeat(n))
                };
                assert_eq!(content, expected, "{tag}, {output_format:?}, {n}");
            }
        }
    }
}

#[test]
fn semantic_italics_should_preserve_a_direct_or_inherited_hard_break() {
    for tag in ["var", "dfn"] {
        for (html, expected) in [
            (format!("<div>a<br><{tag}> 界</{tag}></div>"), "a  \n*界*\n"),
            (format!("<div>a<br><i><{tag}> 界</{tag}></i></div>"), "a  \n**界**\n"),
        ] {
            assert_eq!(semantic_italics_content(&html), expected);
        }
    }
}

#[test]
fn semantic_italics_should_keep_list_rule_and_quote_marker_context() {
    for tag in ["var", "dfn"] {
        for (html, expected) in [
            (format!("<div><{tag}><ul><li>x</li></ul></{tag}></div>"), "*- x*\n"),
            (format!("<div><{tag}><hr></{tag}></div>"), "*---*\n"),
            (
                format!("<ul><li><{tag}><blockquote><ul><li>x</li></ul></blockquote></{tag}></li></ul>"),
                "- *> * x*\n",
            ),
        ] {
            assert_eq!(semantic_italics_content(&html), expected);
        }
    }
}

fn semantic_italics_content(html: &str) -> String {
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: false,
        ..Default::default()
    };
    convert(html, options)
        .expect("convert semantic italics")
        .content
        .expect("content")
}
