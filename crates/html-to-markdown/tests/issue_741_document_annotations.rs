#![allow(missing_docs)]

use html_to_markdown_rs::types::{AnnotationKind, DocumentNode, NodeContent, TextAnnotation, build_document_structure};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};
#[cfg(feature = "visitor")]
use html_to_markdown_rs::{NodeContext, visitor::HtmlVisitor, visitor::VisitResult};
#[cfg(feature = "visitor")]
use std::sync::{Arc, Mutex};

fn node_for(
    document: &html_to_markdown_rs::types::DocumentStructure,
    predicate: impl Fn(&NodeContent) -> bool,
) -> &DocumentNode {
    document
        .nodes
        .iter()
        .find(|node| predicate(&node.content))
        .expect("expected document node")
}

#[test]
fn should_collect_all_supported_inline_annotations_during_conversion() {
    let inline = concat!(
        "<strong>B</strong><em>I</em><u>U</u><s>S</s>",
        "<code>C</code><sub>b</sub><sup>p</sup><mark>H</mark>",
        r#"<a href="child?q=1#part" title="Child">L</a>"#,
    );
    let html = format!("<h2>{inline}</h2><p>{inline}</p><ul><li>{inline}</li></ul>");
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        include_document_structure: true,
        ..ConversionOptions::default()
    };
    let converted = convert(&html, Some(options))
        .expect("conversion should succeed")
        .document
        .expect("document structure should be populated");
    let dom = tl::parse(&html, tl::ParserOptions::default()).expect("HTML should parse");
    let rebuilt = build_document_structure(&dom);

    let expected = vec![
        TextAnnotation {
            start: 0,
            end: 1,
            kind: AnnotationKind::Bold,
        },
        TextAnnotation {
            start: 1,
            end: 2,
            kind: AnnotationKind::Italic,
        },
        TextAnnotation {
            start: 2,
            end: 3,
            kind: AnnotationKind::Underline,
        },
        TextAnnotation {
            start: 3,
            end: 4,
            kind: AnnotationKind::Strikethrough,
        },
        TextAnnotation {
            start: 4,
            end: 5,
            kind: AnnotationKind::Code,
        },
        TextAnnotation {
            start: 5,
            end: 6,
            kind: AnnotationKind::Subscript,
        },
        TextAnnotation {
            start: 6,
            end: 7,
            kind: AnnotationKind::Superscript,
        },
        TextAnnotation {
            start: 7,
            end: 8,
            kind: AnnotationKind::Highlight,
        },
        TextAnnotation {
            start: 8,
            end: 9,
            kind: AnnotationKind::Link {
                url: "child?q=1#part".to_string(),
                title: Some("Child".to_string()),
            },
        },
    ];

    let cases: [fn(&NodeContent) -> bool; 3] = [
        |content| matches!(content, NodeContent::Heading { .. }),
        |content| matches!(content, NodeContent::Paragraph { .. }),
        |content| matches!(content, NodeContent::ListItem { .. }),
    ];
    for predicate in cases {
        let rebuilt_node = node_for(&rebuilt, predicate);
        let converted_node = node_for(&converted, predicate);
        assert_eq!(rebuilt_node.annotations, expected);
        assert_eq!(converted_node.annotations, expected);
        assert_eq!(converted_node.content, rebuilt_node.content);
    }
}

#[test]
fn should_adjust_annotation_byte_offsets_after_trimming_block_whitespace() {
    let html = "<p>\n  <strong>é</strong> <a href=\"child\">λ</a>\n</p>";
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        include_document_structure: true,
        ..ConversionOptions::default()
    };
    let converted = convert(html, Some(options))
        .expect("conversion should succeed")
        .document
        .expect("document structure should be populated");
    let dom = tl::parse(html, tl::ParserOptions::default()).expect("HTML should parse");
    let rebuilt = build_document_structure(&dom);
    let predicate = |content: &NodeContent| matches!(content, NodeContent::Paragraph { .. });
    let converted_node = node_for(&converted, predicate);
    let rebuilt_node = node_for(&rebuilt, predicate);

    assert_eq!(
        converted_node.content,
        NodeContent::Paragraph {
            text: "é λ".to_string()
        }
    );
    assert_eq!(converted_node.content, rebuilt_node.content);
    assert_eq!(
        converted_node.annotations,
        [
            TextAnnotation {
                start: 0,
                end: 2,
                kind: AnnotationKind::Bold,
            },
            TextAnnotation {
                start: 3,
                end: 5,
                kind: AnnotationKind::Link {
                    url: "child".to_string(),
                    title: None,
                },
            },
        ]
    );
    assert_eq!(converted_node.annotations, rebuilt_node.annotations);
}

#[test]
fn should_capture_semantic_text_before_markdown_escaping() {
    let html = "<p>* &amp; é <strong>_</strong></p>";
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        include_document_structure: true,
        ..ConversionOptions::default()
    };
    let converted = convert(html, Some(options))
        .expect("conversion should succeed")
        .document
        .expect("document structure should be populated");
    let dom = tl::parse(html, tl::ParserOptions::default()).expect("HTML should parse");
    let rebuilt = build_document_structure(&dom);
    let predicate = |content: &NodeContent| matches!(content, NodeContent::Paragraph { .. });
    let converted_node = node_for(&converted, predicate);
    let rebuilt_node = node_for(&rebuilt, predicate);

    assert_eq!(
        converted_node.content,
        NodeContent::Paragraph {
            text: "* & é _".to_string()
        }
    );
    assert_eq!(converted_node.content, rebuilt_node.content);
    assert_eq!(converted_node.annotations, rebuilt_node.annotations);
}

fn converted_paragraph(html: &str, options: ConversionOptions) -> DocumentNode {
    let document = convert(html, Some(options))
        .expect("conversion should succeed")
        .document
        .expect("document structure should be populated");
    node_for(&document, |content| matches!(content, NodeContent::Paragraph { .. })).clone()
}

#[cfg(feature = "visitor")]
#[derive(Debug, Clone, Copy)]
enum StrongVisitorMode {
    Custom,
    Skip,
    PreserveHtml,
}

#[cfg(feature = "visitor")]
#[derive(Debug)]
struct StrongVisitor(StrongVisitorMode);

#[cfg(feature = "visitor")]
impl HtmlVisitor for StrongVisitor {
    fn visit_strong(&mut self, _ctx: &NodeContext<'_>, _text: &str) -> VisitResult {
        match self.0 {
            StrongVisitorMode::Custom => VisitResult::Custom("replacement".to_string()),
            StrongVisitorMode::Skip => VisitResult::Skip,
            StrongVisitorMode::PreserveHtml => VisitResult::PreserveHtml,
        }
    }
}

#[cfg(feature = "visitor")]
fn paragraph_with_strong_visitor(mode: StrongVisitorMode) -> DocumentNode {
    converted_paragraph(
        "<p>before <strong>hidden</strong> after</p>",
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            include_document_structure: true,
            visitor: Some(Arc::new(Mutex::new(StrongVisitor(mode)))),
            ..ConversionOptions::default()
        },
    )
}

#[test]
fn should_not_collect_text_or_annotations_from_excluded_descendants() {
    let paragraph = converted_paragraph(
        r#"<p>kept <a class="drop" href="secret"><strong>hidden</strong></a> tail</p>"#,
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            include_document_structure: true,
            exclude_selectors: vec![".drop".to_string()],
            ..ConversionOptions::default()
        },
    );

    assert_eq!(
        paragraph.content,
        NodeContent::Paragraph {
            text: "kept  tail".to_string()
        }
    );
    assert!(paragraph.annotations.is_empty());
}

#[test]
fn should_not_collect_text_or_annotations_beyond_max_depth() {
    let paragraph = converted_paragraph(
        r#"<p>kept <span><span><a href="secret"><strong>hidden</strong></a></span></span></p>"#,
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            include_document_structure: true,
            max_depth: Some(3),
            ..ConversionOptions::default()
        },
    );

    assert_eq!(
        paragraph.content,
        NodeContent::Paragraph {
            text: "kept".to_string()
        }
    );
    assert!(paragraph.annotations.is_empty());
}

#[cfg(feature = "visitor")]
#[test]
fn should_replace_collected_descendants_when_visitor_replaces_element() {
    #[derive(Debug)]
    struct ReplaceSpan;

    impl HtmlVisitor for ReplaceSpan {
        fn visit_element_end(&mut self, ctx: &NodeContext<'_>, _output: &str) -> VisitResult {
            if ctx.tag_name == "span" {
                VisitResult::Custom("replacement".to_string())
            } else {
                VisitResult::Continue
            }
        }
    }

    let paragraph = converted_paragraph(
        r#"<p>kept <span><a href="secret"><strong>hidden</strong></a></span> tail</p>"#,
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            include_document_structure: true,
            visitor: Some(Arc::new(Mutex::new(ReplaceSpan))),
            ..ConversionOptions::default()
        },
    );

    assert_eq!(
        paragraph.content,
        NodeContent::Paragraph {
            text: "kept replacement tail".to_string()
        }
    );
    assert!(paragraph.annotations.is_empty());
}

#[cfg(feature = "visitor")]
#[test]
fn should_rollback_strong_annotation_when_specialized_visitor_skips_element() {
    let paragraph = paragraph_with_strong_visitor(StrongVisitorMode::Skip);

    assert_eq!(
        paragraph.content,
        NodeContent::Paragraph {
            text: "before after".to_string()
        }
    );
    assert!(paragraph.annotations.is_empty());
}

#[cfg(feature = "visitor")]
#[test]
fn should_replace_strong_annotation_when_specialized_visitor_returns_custom_text() {
    let paragraph = paragraph_with_strong_visitor(StrongVisitorMode::Custom);

    assert_eq!(
        paragraph.content,
        NodeContent::Paragraph {
            text: "before replacement after".to_string()
        }
    );
    assert!(paragraph.annotations.is_empty());
}

#[cfg(feature = "visitor")]
#[test]
fn should_replace_strong_annotation_when_specialized_visitor_preserves_html() {
    let paragraph = paragraph_with_strong_visitor(StrongVisitorMode::PreserveHtml);

    assert_eq!(
        paragraph.content,
        NodeContent::Paragraph {
            text: "before <strong>hidden</strong> after".to_string()
        }
    );
    assert!(paragraph.annotations.is_empty());
}

#[cfg(feature = "visitor")]
#[test]
fn should_create_block_nodes_when_start_visitor_returns_custom_text() {
    #[derive(Debug)]
    struct ReplaceBlocksAtStart;

    impl HtmlVisitor for ReplaceBlocksAtStart {
        fn visit_element_start(&mut self, ctx: &NodeContext<'_>) -> VisitResult {
            match ctx.tag_name.as_ref() {
                "p" => VisitResult::Custom("paragraph replacement".to_string()),
                "h2" => VisitResult::Custom("heading replacement".to_string()),
                "li" => VisitResult::Custom("item replacement".to_string()),
                _ => VisitResult::Continue,
            }
        }
    }

    let result = convert(
        concat!(
            "<p><strong>hidden paragraph</strong></p>",
            "<h2><em>hidden heading</em></h2>",
            "<ul><li><a href=\"secret\">hidden item</a></li></ul>",
        ),
        Some(ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            include_document_structure: true,
            visitor: Some(Arc::new(Mutex::new(ReplaceBlocksAtStart))),
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed");
    let document = result.document.expect("document structure should be populated");

    let paragraph = node_for(&document, |content| matches!(content, NodeContent::Paragraph { .. }));
    assert_eq!(
        paragraph.content,
        NodeContent::Paragraph {
            text: "paragraph replacement".to_string()
        }
    );
    assert!(paragraph.annotations.is_empty());

    let heading = node_for(&document, |content| matches!(content, NodeContent::Heading { .. }));
    assert_eq!(
        heading.content,
        NodeContent::Heading {
            level: 2,
            text: "heading replacement".to_string()
        }
    );
    assert!(heading.annotations.is_empty());

    let item = node_for(&document, |content| matches!(content, NodeContent::ListItem { .. }));
    assert_eq!(
        item.content,
        NodeContent::ListItem {
            text: "item replacement".to_string()
        }
    );
    assert!(item.annotations.is_empty());
}

#[cfg(feature = "visitor")]
#[derive(Debug, Clone, Copy)]
enum WrapperVisitorMode {
    Custom,
    Skip,
}

#[cfg(feature = "visitor")]
#[derive(Debug)]
struct WrapperVisitor(WrapperVisitorMode);

#[cfg(feature = "visitor")]
impl HtmlVisitor for WrapperVisitor {
    fn visit_element_end(&mut self, ctx: &NodeContext<'_>, _output: &str) -> VisitResult {
        if ctx.tag_name != "div" {
            return VisitResult::Continue;
        }
        match self.0 {
            WrapperVisitorMode::Custom => VisitResult::Custom("replacement".to_string()),
            WrapperVisitorMode::Skip => VisitResult::Skip,
        }
    }
}

#[cfg(feature = "visitor")]
fn assert_wrapper_rollback(mode: WrapperVisitorMode) {
    let result = convert(
        "<div><h2>hidden heading</h2><p>hidden paragraph</p><table><tr><td>hidden cell</td></tr></table></div><p>after</p>",
        Some(ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            include_document_structure: true,
            visitor: Some(Arc::new(Mutex::new(WrapperVisitor(mode)))),
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed");
    let document = result.document.expect("document structure should be populated");

    assert!(result.tables.is_empty());
    assert_eq!(document.nodes.len(), 1);
    assert_eq!(
        document.nodes[0].content,
        NodeContent::Paragraph {
            text: "after".to_string()
        }
    );
    assert_eq!(document.nodes[0].parent, None);
    assert!(document.nodes[0].children.is_empty());
}

#[cfg(feature = "visitor")]
#[test]
fn should_rollback_nested_structure_when_wrapper_end_visitor_returns_custom_text() {
    assert_wrapper_rollback(WrapperVisitorMode::Custom);
}

#[cfg(feature = "visitor")]
#[test]
fn should_rollback_nested_structure_when_wrapper_end_visitor_skips_element() {
    assert_wrapper_rollback(WrapperVisitorMode::Skip);
}
