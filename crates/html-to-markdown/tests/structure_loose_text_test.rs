#![allow(missing_docs)]

//! Text outside any paragraph, heading or list item must reach the document structure.
//!
//! The structure records the text of `<p>`, `<h1>`–`<h6>` and `<li>`. Text written straight
//! into a `<div>`, `<body>`, `<blockquote>` or after a list or table was left out, although the
//! markdown keeps it, so a consumer that reads the structure lost that text. It is now a
//! paragraph of its own, ended by the next block element.

use html_to_markdown_rs::types::{AnnotationKind, NodeContent, TextAnnotation};
use html_to_markdown_rs::{ConversionOptions, convert};

fn nodes(html: &str) -> Vec<NodeContent> {
    let options = ConversionOptions {
        include_document_structure: true,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .document
        .expect("document structure should be populated")
        .nodes
        .into_iter()
        .map(|node| node.content)
        .collect()
}

fn paragraph(text: &str) -> NodeContent {
    NodeContent::Paragraph { text: text.to_string() }
}

#[test]
fn text_outside_a_paragraph_is_a_paragraph_of_its_own() {
    assert_eq!(
        nodes("<div><p>First</p>Loose words</div><div>Second div text</div>"),
        [
            paragraph("First"),
            paragraph("Loose words"),
            paragraph("Second div text")
        ]
    );
    assert_eq!(nodes("<body>Only bare text</body>"), [paragraph("Only bare text")]);
    assert_eq!(
        nodes("<blockquote>Quoted text</blockquote>"),
        [paragraph("Quoted text")]
    );
}

#[test]
fn text_after_a_list_or_a_table_is_a_paragraph() {
    let after_list = nodes("<ul><li>Item</li></ul>After the list");
    assert!(matches!(after_list[0], NodeContent::List { .. }), "{after_list:?}");
    assert_eq!(
        after_list[1..],
        [
            NodeContent::ListItem {
                text: "Item".to_string()
            },
            paragraph("After the list")
        ]
    );

    let after_table = nodes("<table><tr><td>Cell</td></tr></table>After the table<pre>code text</pre>");
    assert!(matches!(after_table[0], NodeContent::Table { .. }), "{after_table:?}");
    assert_eq!(after_table[1], paragraph("After the table"));
    assert!(matches!(after_table[2], NodeContent::Code { .. }), "{after_table:?}");
    assert_eq!(
        after_table.len(),
        3,
        "a table cell or a code block must not repeat as a paragraph"
    );
}

#[test]
fn loose_text_keeps_its_inline_annotations() {
    let options = ConversionOptions {
        include_document_structure: true,
        ..ConversionOptions::default()
    };
    let document = convert(
        "<div><h2>Title</h2>Intro text, <b>bold</b> more.<p>Para</p>Trailing.</div>",
        Some(options),
    )
    .expect("conversion should succeed")
    .document
    .expect("document structure should be populated");
    let paragraphs: Vec<(&str, &[TextAnnotation])> = document
        .nodes
        .iter()
        .filter_map(|node| match &node.content {
            NodeContent::Paragraph { text } => Some((text.as_str(), node.annotations.as_slice())),
            _ => None,
        })
        .collect();

    assert_eq!(
        paragraphs,
        [
            (
                "Intro text, bold more.",
                [TextAnnotation {
                    start: 12,
                    end: 16,
                    kind: AnnotationKind::Bold,
                }]
                .as_slice()
            ),
            ("Para", [].as_slice()),
            ("Trailing.", [].as_slice()),
        ]
    );
}

#[test]
fn white_space_between_blocks_makes_no_paragraph() {
    assert_eq!(
        nodes("<div>\n  <p>A</p>\n  \n  <p>B</p>\n</div>"),
        [paragraph("A"), paragraph("B")]
    );
}
