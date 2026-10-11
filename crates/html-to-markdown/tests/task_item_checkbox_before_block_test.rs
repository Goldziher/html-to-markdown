// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for the checkbox of a task item whose first content is a block. GFM reads a
//! checkbox only in a paragraph with content after the marker, so the checkbox line holds a space
//! written as a character reference and the block starts on the next line.

use html_to_markdown_rs::options::CodeBlockStyle;
use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

fn options_with(strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        tier_strategy: strategy,
        ..ConversionOptions::default()
    }
}

fn convert_with(html: &str, options: &ConversionOptions) -> String {
    convert(html, Some(options.clone()))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn render(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.tasklist = true;
    options.extension.table = true;
    options.render.r#unsafe = true;
    comrak::markdown_to_html(markdown, &options)
}

/// Whether `rendered` holds a checkbox and `block_tag` opens inside a list item.
fn keeps_checkbox_and_block(rendered: &str, block_tag: &str) -> bool {
    let first_item = rendered.find("<li>").map_or(rendered.len(), |pos| pos + "<li>".len());
    let Some(block) = rendered[first_item..].find(block_tag).map(|pos| pos + first_item) else {
        return false;
    };
    let before = &rendered[..block];
    rendered.contains(r#"<input type="checkbox""#) && before.matches("<li").count() > before.matches("</li>").count()
}

/// The block elements a task item can start with: the HTML, the markdown lines of the block
/// (`{i}` is the item's content indent, `{b}` the bullet of a list nested in it), the line break
/// after the checkbox line, and the tag that proves the block in the rendering.
const BLOCKS: &[(&str, &[&str], &str, &str)] = &[
    ("<blockquote>q</blockquote>", &["{i}> q"], "\n", "<blockquote>"),
    ("<h2>h</h2>", &["{i}## h"], "\n", "<h2>"),
    // ~keep The converter writes a table's delimiter row four columns in, past the content
    // ~keep column of either marker.
    (
        "<table><tr><td>c</td></tr></table>",
        &["{i}| c |", "    | --- |"],
        "\n\n",
        "<table>",
    ),
    ("<ul><li>x</li></ul>", &["{i}{b} x"], "\n", "<ul>"),
    (
        r#"<ol start="3"><li>x</li></ol>"#,
        &["{i}3. x"],
        "\n\n",
        r#"<ol start="3">"#,
    ),
    ("<pre>c</pre>", &["{i}```", "{i}c", "{i}```"], "\n", "<pre>"),
    ("<hr>", &["{i}---"], "\n\n", "<hr />"),
];

#[test]
fn should_keep_the_checkbox_of_a_task_item_that_starts_with_a_block() {
    for strategy in [TierStrategy::Tier2, TierStrategy::Auto, TierStrategy::Tier1] {
        let options = options_with(strategy);
        for (list, marker, indent, bullet) in [("ul", "-", "  ", "*"), ("ol", "1.", "   ", "-")] {
            for (checked, box_text) in [("", "[ ]"), (" checked", "[x]")] {
                for (block_html, lines, separator, block_tag) in BLOCKS {
                    let html = format!(r#"<{list}><li><input type="checkbox"{checked}>{block_html}</li></{list}>"#);
                    let body: String = lines
                        .iter()
                        .map(|line| line.replace("{i}", indent).replace("{b}", bullet) + "\n")
                        .collect();
                    let expected = format!("{marker} {box_text} &#32;{separator}{body}");
                    let markdown = convert_with(&html, &options);
                    assert_eq!(markdown, expected, "{strategy:?} {html}");
                    let rendered = render(&markdown);
                    assert!(
                        keeps_checkbox_and_block(&rendered, block_tag),
                        "{strategy:?} {html}: {rendered:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn should_keep_the_checkbox_of_a_task_item_that_starts_with_indented_code() {
    let options = ConversionOptions {
        code_block_style: CodeBlockStyle::Indented,
        ..options_with(TierStrategy::Tier2)
    };
    let markdown = convert_with(r#"<ul><li><input type="checkbox"><pre>c</pre></li></ul>"#, &options);
    assert_eq!(markdown, "- [ ] &#32;\n\n      c\n");
    let rendered = render(&markdown);
    assert!(keeps_checkbox_and_block(&rendered, "<pre>"), "{rendered:?}");
}

#[test]
fn should_keep_the_checkbox_of_a_task_item_that_starts_with_a_preserved_block() {
    let options = ConversionOptions {
        preserve_tags: vec!["div".to_string()],
        ..options_with(TierStrategy::Tier2)
    };
    let markdown = convert_with(r#"<ul><li><input type="checkbox"><div>d</div></li></ul>"#, &options);
    assert_eq!(markdown, "- [ ] &#32;\n  <div>d</div>\n");
    let rendered = render(&markdown);
    assert!(keeps_checkbox_and_block(&rendered, "<div>d</div>"), "{rendered:?}");
}

#[test]
fn should_write_the_same_task_item_after_a_render_of_its_markdown() {
    let options = options_with(TierStrategy::Tier2);
    for html in [
        r#"<ul><li><input type="checkbox"><blockquote>q</blockquote></li></ul>"#,
        r#"<ol><li><input type="checkbox" checked><h2>h</h2></li></ol>"#,
    ] {
        let markdown = convert_with(html, &options);
        assert_eq!(convert_with(&render(&markdown), &options), markdown, "{html}");
    }
}

#[test]
fn should_keep_the_checkbox_of_an_empty_task_item() {
    for strategy in [TierStrategy::Tier2, TierStrategy::Auto, TierStrategy::Tier1] {
        let options = options_with(strategy);
        for (html, expected) in [
            (r#"<ul><li><input type="checkbox"></li></ul>"#, "- [ ] &#32;\n"),
            (r#"<ol><li><input type="checkbox" checked></li></ol>"#, "1. [x] &#32;\n"),
            (
                r#"<ul><li><input type="checkbox"></li><li><input type="checkbox">t</li></ul>"#,
                "- [ ] &#32;\n- [ ] t\n",
            ),
        ] {
            let markdown = convert_with(html, &options);
            assert_eq!(markdown, expected, "{strategy:?} {html}");
            let rendered = render(&markdown);
            assert!(
                rendered.starts_with("<ul>\n<li><input type=\"checkbox\"")
                    || rendered.starts_with("<ol>\n<li><input type=\"checkbox\""),
                "{strategy:?} {html}: {rendered:?}"
            );
        }
    }
}

#[test]
fn should_keep_the_output_where_a_checkbox_is_text() {
    // Code, an inline element around the list, a table cell and inline mode write no reference,
    // and an empty item there keeps the space after its checkbox, in both output formats and both
    // tiers. Each case gives the Markdown output, then the Djot output.
    let empty = r#"<ul><li><input type="checkbox"></li></ul>"#;
    let quote = r#"<ul><li><input type="checkbox"><blockquote>q</blockquote></li></ul>"#;
    let same = |inline: bool, html: String, expected: &'static str| (inline, html, expected, expected);
    let cases = [
        same(false, format!("<code>{empty}</code>"), "`- [ ] `\n"),
        same(false, format!("<code><b>{empty}</b></code>"), "`- [ ] `\n"),
        (
            false,
            format!("<code>{quote}</code>"),
            "`- [ ]`  \n`  > q`\n",
            "`- [ ]`\\\n`  > q`\n",
        ),
        // ~keep A line of a code block keeps its line end, so the space after the marker stays.
        same(false, format!("<pre>{empty}</pre>"), "```\n- [ ] \n```\n"),
        same(false, format!("<pre>{quote}</pre>"), "```\n- [ ]\n  > q\n```\n"),
        (false, format!("<b>{empty}</b>"), "**- [ ]**\n", "*- [ ]*\n"),
        (false, format!("<b>{quote}</b>"), "**- [ ] > q**\n", "*- [ ] > q*\n"),
        (false, format!("<em>{empty}</em>"), "*- [ ]*\n", "_- [ ]_\n"),
        (false, format!("<em>{quote}</em>"), "*- [ ] > q*\n", "_- [ ] > q_\n"),
        same(false, format!(r#"<a href="u">{empty}</a>"#), "[- [ ]](u)\n"),
        same(false, format!(r#"<a href="u">{quote}</a>"#), "[- [ ] q](u)\n"),
        (
            false,
            format!("<table><tr><td>{quote}</td></tr></table>"),
            "| - [ ] q |\n| ------- |\n",
            "| - [ ] q |\n|-------|\n",
        ),
        same(true, empty.to_string(), "- [ ]\n"),
        same(true, quote.to_string(), "- [ ] q\n"),
        same(true, format!("<code>{empty}</code>"), "`- [ ] `\n"),
    ];
    for format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
            for (inline, html, markdown, djot) in &cases {
                let options = ConversionOptions {
                    output_format: format,
                    convert_as_inline: *inline,
                    ..options_with(strategy)
                };
                let expected = if format == OutputFormat::Djot { *djot } else { *markdown };
                assert_eq!(
                    convert_with(html, &options),
                    expected,
                    "{html} {format:?} {strategy:?} inline={inline}"
                );
            }
        }
    }
}

#[test]
fn should_keep_a_task_item_as_before_in_djot() {
    let options = ConversionOptions {
        output_format: OutputFormat::Djot,
        ..options_with(TierStrategy::Tier2)
    };
    for (html, expected) in [
        (
            r#"<ul><li><input type="checkbox"><blockquote>q</blockquote></li></ul>"#,
            "- [ ]\n  > q\n",
        ),
        (r#"<ul><li><input type="checkbox"></li></ul>"#, "- [ ]\n"),
    ] {
        assert_eq!(convert_with(html, &options), expected, "{html}");
    }
}
