#![allow(missing_docs)]

//! A `<br>` must separate the text on either side of it in the document structure.
//!
//! The Markdown gets a hard break for a `<br>`, but the recorded text of a paragraph, heading or
//! list item got nothing, so `Loose words<br>next line` was recorded as `Loose wordsnext line`.

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

#[test]
fn inline_code_breaks_separate_the_enclosing_paragraph_text() {
    for tag in ["code", "kbd", "samp"] {
        let html = format!("<p>a<{tag}>b<br>c</{tag}>d</p>");
        assert_eq!(
            nodes(&html),
            [NodeContent::Paragraph {
                text: "ab\ncd".to_string()
            }],
            "inline tag: {tag}"
        );
    }
}

#[test]
fn heading_breaks_collapse_adjacent_whitespace() {
    for html in ["<h2>A<br><br>B</h2>", "<h2>A <br>B</h2>", "<h2>A<br> B</h2>"] {
        let heading = nodes(html)
            .into_iter()
            .find(|content| matches!(content, NodeContent::Heading { .. }))
            .expect("a heading should be recorded");
        assert_eq!(
            heading,
            NodeContent::Heading {
                level: 2,
                text: "A B".to_string()
            },
            "input: {html}"
        );
    }
}

#[test]
fn heading_breaks_keep_annotation_byte_offsets_after_collapsing_whitespace() {
    for (html, text, start, end) in [
        ("<h2>A<br><b>B</b></h2>", "A B", 2, 3),
        ("<h2>A <br><br><b>B</b></h2>", "A B", 2, 3),
        ("<h2>é<br><br><b>β</b></h2>", "é β", 3, 5),
        ("<h2>A<br><b> B</b></h2>", "A B", 2, 3),
        ("<h2>é<br><b> β</b></h2>", "é β", 3, 5),
    ] {
        let document = convert(
            html,
            Some(ConversionOptions {
                include_document_structure: true,
                ..ConversionOptions::default()
            }),
        )
        .expect("conversion should succeed")
        .document
        .expect("document structure should be populated");
        let heading = document
            .nodes
            .iter()
            .find(|node| matches!(node.content, NodeContent::Heading { .. }))
            .expect("a heading should be recorded");
        assert_eq!(
            heading.content,
            NodeContent::Heading {
                level: 2,
                text: text.to_string()
            },
            "input: {html}"
        );
        assert_eq!(
            heading.annotations,
            [TextAnnotation {
                start,
                end,
                kind: AnnotationKind::Bold
            }],
            "input: {html}"
        );
    }
}

#[test]
fn inline_code_breaks_keep_unicode_annotation_byte_offsets() {
    let document = convert(
        "<p>é<code>β<br>γ</code>δ</p>",
        Some(ConversionOptions {
            include_document_structure: true,
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed")
    .document
    .expect("document structure should be populated");
    assert_eq!(
        document.nodes[0].content,
        NodeContent::Paragraph {
            text: "éβ\nγδ".to_string()
        }
    );
    assert_eq!(
        document.nodes[0].annotations,
        [TextAnnotation {
            start: 2,
            end: 7,
            kind: AnnotationKind::Code
        }]
    );
}

#[cfg(feature = "visitor")]
#[test]
fn skipped_heading_wrappers_restore_pending_break_whitespace() {
    use html_to_markdown_rs::{NodeContext, visitor::HtmlVisitor, visitor::VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct SkipSpan(bool);
    impl HtmlVisitor for SkipSpan {
        fn visit_element_end(&mut self, ctx: &NodeContext<'_>, _output: &str) -> VisitResult {
            if ctx.tag_name == "span" && self.0 {
                VisitResult::Custom(String::new())
            } else if ctx.tag_name == "span" {
                VisitResult::Skip
            } else {
                VisitResult::Continue
            }
        }
    }

    for (html, start, custom) in [
        ("<h2>A<br><span>discard</span><b> B</b></h2>", 2, false),
        ("<h2>A<span><br></span><b> B</b></h2>", 1, false),
        ("<h2>A<br><span>discard</span><b> B</b></h2>", 2, true),
        ("<h2>A<span><br></span><b> B</b></h2>", 1, true),
    ] {
        let document = convert(
            html,
            Some(ConversionOptions {
                include_document_structure: true,
                visitor: Some(Arc::new(Mutex::new(SkipSpan(custom)))),
                ..ConversionOptions::default()
            }),
        )
        .expect("conversion should succeed")
        .document
        .expect("document structure should be populated");
        let heading = document
            .nodes
            .iter()
            .find(|node| matches!(node.content, NodeContent::Heading { .. }))
            .expect("a heading should be recorded");
        assert_eq!(
            heading.content,
            NodeContent::Heading {
                level: 2,
                text: "A B".to_string()
            },
            "input: {html}"
        );
        assert_eq!(
            heading.annotations,
            [TextAnnotation {
                start,
                end: 3,
                kind: AnnotationKind::Bold
            }],
            "input: {html}"
        );
    }
}

#[test]
fn heading_breaks_preserve_spaces_inside_inline_code() {
    for (html, start) in [
        ("<h2>A<br><code> B</code></h2>", 2),
        ("<h2><code>A<br> B</code></h2>", 0),
        ("<h2><code>A <br>B</code></h2>", 0),
        ("<h2><code>A<br><br>B</code></h2>", 0),
    ] {
        let document = convert(
            html,
            Some(ConversionOptions {
                include_document_structure: true,
                ..ConversionOptions::default()
            }),
        )
        .expect("conversion should succeed")
        .document
        .expect("document structure should be populated");
        let heading = document
            .nodes
            .iter()
            .find(|node| matches!(node.content, NodeContent::Heading { .. }))
            .expect("a heading should be recorded");
        assert_eq!(
            heading.content,
            NodeContent::Heading {
                level: 2,
                text: "A  B".to_string()
            },
            "input: {html}"
        );
        assert_eq!(
            heading.annotations,
            [TextAnnotation {
                start,
                end: 4,
                kind: AnnotationKind::Code
            }],
            "input: {html}"
        );
    }
}

#[test]
fn strict_mark_newlines_and_breaks_match_paragraph_and_loose_text_captures() {
    use html_to_markdown_rs::{HighlightStyle, TierStrategy, WhitespaceMode};

    let mut checked = 0;
    for (mark_text, expected) in [
        ("\r\n ", [("x\n z", 1, 3), ("x\n\n z", 2, 4), ("x\n\n\n z", 3, 5)]),
        (" ", [("x z", 1, 2), ("x\n z", 2, 3), ("x\n\n z", 3, 4)]),
        ("\r\n1. ", [("x\n1. z", 1, 5), ("x\n1. z", 2, 5), ("x\n\n\n1. z", 3, 7)]),
        ("1. ", [("x1. z", 1, 4), ("x\n1. z", 2, 5), ("x\n\n1. z", 3, 6)]),
    ] {
        for tag in ["p", "div"] {
            for (breaks, (text, start, end)) in expected.into_iter().enumerate() {
                let html = format!("<{tag}>x{}<mark>{mark_text}</mark>z</{tag}>", "<br>".repeat(breaks));
                let document = convert(
                    &html,
                    Some(ConversionOptions {
                        include_document_structure: true,
                        whitespace_mode: WhitespaceMode::Strict,
                        highlight_style: HighlightStyle::None,
                        tier_strategy: TierStrategy::Tier2,
                        ..ConversionOptions::default()
                    }),
                )
                .expect("conversion should succeed")
                .document
                .expect("document structure should be populated");
                assert_eq!(document.nodes.len(), 1, "input: {html:?}");
                assert_eq!(
                    document.nodes[0].content,
                    NodeContent::Paragraph { text: text.to_string() },
                    "input: {html:?}"
                );
                assert_eq!(
                    document.nodes[0].annotations,
                    [TextAnnotation {
                        start,
                        end,
                        kind: AnnotationKind::Highlight
                    }],
                    "input: {html:?}"
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 24);
}
