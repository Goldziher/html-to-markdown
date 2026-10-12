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

#[test]
fn marked_wrappers_should_preserve_hard_break_list_and_rule_context() {
    use html_to_markdown_rs::HighlightStyle;
    for (tag, style, marker) in [
        ("del", HighlightStyle::DoubleEqual, "~~"),
        ("s", HighlightStyle::DoubleEqual, "~~"),
        ("strike", HighlightStyle::DoubleEqual, "~~"),
        ("mark", HighlightStyle::DoubleEqual, "=="),
        ("mark", HighlightStyle::Bold, "**"),
    ] {
        for (html, expected) in [
            (
                format!("<div>a<br><{tag}> 界</{tag}></div>"),
                format!("a  \n{marker}界{marker}\n"),
            ),
            (
                format!("<div>a<br><i><{tag}> 界</{tag}></i></div>"),
                format!("a  \n*{marker}界{marker}*\n"),
            ),
            (
                format!("<div><{tag}><ul><li>x</li></ul></{tag}></div>"),
                format!("{marker}- x{marker}\n"),
            ),
            (
                format!("<div><{tag}><hr></{tag}></div>"),
                format!("{marker}---{marker}\n"),
            ),
        ] {
            assert_eq!(marked_content(&html, style), expected, "{tag}, {style:?}");
        }
    }
}

#[test]
fn html_highlighting_should_preserve_source_boundaries() {
    use html_to_markdown_rs::HighlightStyle;
    for (html, expected) in [
        ("<div>a<br><mark> 界</mark></div>", "a  \n<mark> 界</mark>\n"),
        ("<div>a<br><i><mark> 界</mark></i></div>", "a  \n*<mark> 界</mark>*\n"),
        ("<div><mark><ul><li>x</li></ul></mark></div>", "<mark>- x\n</mark>\n"),
        ("<div><mark><hr></mark></div>", "<mark>--- </mark>\n"),
    ] {
        assert_eq!(marked_content(html, HighlightStyle::Html), expected);
    }
}

#[test]
fn unmarked_highlighting_should_preserve_the_live_list_column() {
    use html_to_markdown_rs::HighlightStyle;
    for (html, expected) in [
        ("<ul><li><mark><ul><li>x</li></ul></mark></li></ul>", "- * x\n"),
        ("<div><mark><ul><li>x</li></ul></mark></div>", "- x\n"),
        ("<div>a<br><mark> 界</mark></div>", "a  \n界\n"),
    ] {
        assert_eq!(marked_content(html, HighlightStyle::None), expected);
    }
}

fn marked_content(html: &str, highlight_style: html_to_markdown_rs::HighlightStyle) -> String {
    convert(
        html,
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            extract_metadata: false,
            highlight_style,
            ..Default::default()
        },
    )
    .expect("convert marked wrapper")
    .content
    .expect("content")
}

#[test]
fn marked_wrappers_should_keep_adjacent_markers_on_a_long_unicode_line() {
    use html_to_markdown_rs::HighlightStyle;
    for (tag, style, marker) in [
        ("del", HighlightStyle::DoubleEqual, "~~"),
        ("s", HighlightStyle::DoubleEqual, "~~"),
        ("strike", HighlightStyle::DoubleEqual, "~~"),
        ("mark", HighlightStyle::DoubleEqual, "=="),
        ("mark", HighlightStyle::Bold, "**"),
        ("mark", HighlightStyle::Html, "<mark>"),
        ("mark", HighlightStyle::None, ""),
    ] {
        for n in [100, 400] {
            let html = format!("<div>{}</div>", format!("<{tag}>界</{tag}>").repeat(n));
            let expected = if style == HighlightStyle::Html {
                format!("{}\n", "<mark>界</mark>".repeat(n))
            } else {
                format!("{marker}{}{marker}\n", "界".repeat(n))
            };
            assert_eq!(marked_content(&html, style), expected, "{tag}, {style:?}, {n}");
        }
    }
}

#[test]
fn djot_inserted_text_should_preserve_marker_and_boundary_context() {
    for (html, expected) in [
        ("<div>a<br><ins> 界</ins></div>", "a\\\n{+界+}\n"),
        ("<div>a<br><i><ins> 界</ins></i></div>", "a\\\n_{+界+}_\n"),
        ("<div><ins><ul><li>x</li></ul></ins></div>", "{+- x+}\n"),
        ("<div><ins><hr></ins></div>", "{+---+}\n"),
    ] {
        assert_eq!(inserted_content(html, OutputFormat::Djot), expected);
    }
    for n in [100, 400] {
        let html = format!("<div>{}</div>", "<ins>界</ins>".repeat(n));
        assert_eq!(
            inserted_content(&html, OutputFormat::Djot),
            format!("{}\n", "{+界+}".repeat(n))
        );
    }
}

#[test]
fn markdown_and_plain_inserted_text_should_keep_the_incoming_plain_text_contract() {
    for output_format in [OutputFormat::Markdown, OutputFormat::Plain] {
        assert_eq!(inserted_content("<p>a<ins>b</ins>c</p>", output_format), "abc\n");
        for n in [100, 400] {
            let html = format!("<div>{}</div>", "<ins>界</ins>".repeat(n));
            assert_eq!(inserted_content(&html, output_format), format!("{}\n", "界".repeat(n)));
        }
    }
}

fn inserted_content(html: &str, output_format: OutputFormat) -> String {
    convert(
        html,
        ConversionOptions {
            output_format,
            tier_strategy: TierStrategy::Tier2,
            extract_metadata: false,
            ..Default::default()
        },
    )
    .expect("convert inserted text")
    .content
    .expect("content")
}
