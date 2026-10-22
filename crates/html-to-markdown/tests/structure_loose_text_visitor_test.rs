#![allow(missing_docs)]
#![cfg(feature = "visitor")]

use std::sync::{Arc, Mutex};

use html_to_markdown_rs::types::{AnnotationKind, NodeContent, TextAnnotation};
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{ConversionOptions, ConversionResult, convert};

#[derive(Debug)]
enum Replacement {
    StartDiv,
    EndSpan,
    SkipLink,
}

impl HtmlVisitor for Replacement {
    fn visit_element_start(&mut self, ctx: &NodeContext<'_>) -> VisitResult {
        if matches!(self, Self::StartDiv) && ctx.tag_name == "div" {
            VisitResult::Custom("X".to_string())
        } else {
            VisitResult::Continue
        }
    }

    fn visit_element_end(&mut self, ctx: &NodeContext<'_>, _: &str) -> VisitResult {
        if matches!(self, Self::EndSpan) && ctx.tag_name == "span" {
            VisitResult::Custom("X".to_string())
        } else {
            VisitResult::Continue
        }
    }

    fn visit_link(&mut self, _: &NodeContext<'_>, _: &str, _: &str, _: Option<&str>) -> VisitResult {
        if matches!(self, Self::SkipLink) {
            VisitResult::Skip
        } else {
            VisitResult::Continue
        }
    }
}

fn converted(html: &str, visitor: Option<Replacement>) -> ConversionResult {
    let mut options = ConversionOptions {
        include_document_structure: true,
        ..ConversionOptions::default()
    };
    options.visitor = visitor.map(|visitor| Arc::new(Mutex::new(visitor)) as _);
    convert(html, Some(options)).expect("conversion should succeed")
}

fn paragraphs(result: &ConversionResult) -> Vec<(&str, &[TextAnnotation])> {
    result
        .document
        .as_ref()
        .expect("document structure should be populated")
        .nodes
        .iter()
        .filter_map(|node| match &node.content {
            NodeContent::Paragraph { text } => Some((text.as_str(), node.annotations.as_slice())),
            _ => None,
        })
        .collect()
}

#[test]
fn should_capture_custom_generic_element_end_without_losing_replacement_text() {
    let result = converted("<div><span>one</span>two</div>", Some(Replacement::EndSpan));
    assert_eq!(result.content.as_deref().map(str::trim), Some("Xtwo"));
    assert_eq!(paragraphs(&result), [("Xtwo", [].as_slice())]);
}

#[test]
fn should_capture_custom_generic_element_start_as_a_loose_paragraph() {
    let result = converted("<div>one</div>", Some(Replacement::StartDiv));
    assert_eq!(result.content.as_deref().map(str::trim), Some("X"));
    assert_eq!(paragraphs(&result), [("X", [].as_slice())]);
}

#[test]
fn should_restore_text_before_a_skipped_link_that_contains_a_block() {
    let result = converted(
        "<div>before<a href=\"/x\"><div>inside</div></a>after</div>",
        Some(Replacement::SkipLink),
    );
    assert_eq!(result.content.as_deref().map(str::trim), Some("beforeafter"));
    assert_eq!(paragraphs(&result), [("beforeafter", [].as_slice())]);
}

#[test]
fn should_capture_flattened_block_link_text_with_its_annotation() {
    let result = converted("<a href=\"/x\"><div>before</div><div>inside</div></a>", None);
    assert_eq!(result.content.as_deref(), Some("[before inside](/x)\n"));
    assert_eq!(
        paragraphs(&result),
        [(
            "before inside",
            [TextAnnotation {
                start: 0,
                end: 13,
                kind: AnnotationKind::Link {
                    url: "/x".to_string(),
                    title: None,
                },
            }]
            .as_slice()
        )]
    );
}

#[test]
fn should_restore_completed_paragraph_before_replacing_a_block_wrapper() {
    let result = converted(
        "<div>before<span><div>inside</div></span>after</div>",
        Some(Replacement::EndSpan),
    );
    assert_eq!(paragraphs(&result), [("beforeXafter", [].as_slice())]);
}

#[test]
fn should_continue_outer_annotations_across_loose_paragraph_boundaries() {
    let result = converted("<b>before<div>inside</div>after</b>", None);
    assert_eq!(
        paragraphs(&result),
        [
            (
                "before",
                [TextAnnotation {
                    start: 0,
                    end: 6,
                    kind: AnnotationKind::Bold
                }]
                .as_slice()
            ),
            (
                "inside",
                [TextAnnotation {
                    start: 0,
                    end: 6,
                    kind: AnnotationKind::Bold
                }]
                .as_slice()
            ),
            (
                "after",
                [TextAnnotation {
                    start: 0,
                    end: 5,
                    kind: AnnotationKind::Bold
                }]
                .as_slice()
            ),
        ]
    );
}
