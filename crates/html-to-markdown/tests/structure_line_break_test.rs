#![allow(missing_docs)]

//! A `<br>` must separate the text on either side of it in the document structure.
//!
//! The Markdown gets a hard break for a `<br>`, but the recorded text of a paragraph, heading or
//! list item got nothing, so `Loose words<br>next line` was recorded as `Loose wordsnext line`.

use html_to_markdown_rs::types::NodeContent;
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

#[test]
fn a_line_break_separates_the_text_of_a_paragraph_and_a_list_item() {
    assert_eq!(
        nodes("<p>Loose words<br>next line</p>"),
        [NodeContent::Paragraph {
            text: "Loose words\nnext line".to_string()
        }]
    );
    let list = nodes("<ul><li>One<br>two</li></ul>");
    assert_eq!(
        list[1],
        NodeContent::ListItem {
            text: "One\ntwo".to_string()
        }
    );
}

#[test]
fn a_line_break_in_a_heading_is_a_space() {
    let heading = nodes("<h2>Annual<br>Report</h2>");
    assert!(
        heading.contains(&NodeContent::Heading {
            level: 2,
            text: "Annual Report".to_string()
        }),
        "{heading:?}"
    );
}

#[test]
fn a_line_break_at_the_edge_of_a_paragraph_adds_nothing() {
    assert_eq!(
        nodes("<p><br>Text<br></p>"),
        [NodeContent::Paragraph {
            text: "Text".to_string()
        }]
    );
}

#[test]
fn code_blocks_keep_their_own_line_breaks() {
    assert_eq!(
        nodes("<pre>a<br>b</pre>"),
        [NodeContent::Code {
            text: "a\nb".to_string(),
            language: None
        }]
    );
}
