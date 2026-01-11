//! Handler for emphasis elements (strong, b, em, i).
//!
//! Converts HTML emphasis tags to Markdown formatting with support for:
//! - Bold/strong formatting using configurable symbols (** or __)
//! - Italic/emphasis formatting using configurable symbols (* or _)
//! - Nested emphasis context tracking
//! - Code context handling (suppress formatting in <code>)
//! - Visitor callbacks for custom emphasis processing
//! - Bootstrap caret detection (.caret class)

use crate::options::ConversionOptions;
#[allow(unused_imports)]
use std::collections::BTreeMap;
use tl::{NodeHandle, Parser};

// Type aliases for Context and DomContext to avoid circular imports
// These are imported from converter.rs and should be made accessible
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handler for emphasis elements: strong, b (bold) and em, i (italic).
///
/// Processes emphasis content based on context:
/// - Suppresses formatting when already in strong/code context
/// - Applies configurable emphasis symbols (* or _)
/// - Handles nested emphasis with proper context tracking
/// - Supports visitor callbacks for custom behavior
/// - Detects Bootstrap caret elements (.caret class)
///
/// # Note
/// This function references helper functions and `walk_node` from converter.rs
/// which must be accessible (pub(crate)) for this module to work correctly.
pub(crate) fn handle(
    tag_name: &str,
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    // Import helper functions from parent converter module

    match tag_name {
        "strong" | "b" => {
            handle_strong(node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        "em" | "i" => {
            handle_emphasis(node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        _ => {}
    }
}

/// Handle strong/bold emphasis (strong, b tags).
fn handle_strong(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    #[allow(unused_imports)]
    use crate::converter::{append_inline_suffix, chomp_inline, get_text_content, serialize_node, walk_node};

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    if ctx.in_code {
        // Suppress formatting in code context, just process children
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
        }
    } else {
        let mut content = String::with_capacity(64);
        let children = tag.children();
        {
            let strong_ctx = Context {
                inline_depth: ctx.inline_depth + 1,
                in_strong: true,
                ..ctx.clone()
            };
            for child_handle in children.top().iter() {
                walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    options,
                    &strong_ctx,
                    depth + 1,
                    dom_ctx,
                );
            }
        }

        #[cfg(feature = "visitor")]
        let strong_output = if let Some(ref visitor_handle) = ctx.visitor {
            use crate::visitor::{NodeContext, NodeType, VisitResult};

            let text_content = get_text_content(node_handle, parser, dom_ctx);
            let attributes: BTreeMap<String, String> = tag
                .attributes()
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                .collect();

            let node_id = node_handle.get_inner();
            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

            let node_ctx = NodeContext {
                node_type: NodeType::Strong,
                tag_name: tag.name().as_utf8_str().to_string(),
                attributes,
                depth,
                index_in_parent,
                parent_tag,
                is_inline: true,
            };

            let mut visitor = visitor_handle.borrow_mut();
            match visitor.visit_strong(&node_ctx, &text_content) {
                VisitResult::Continue => None,
                VisitResult::Custom(custom) => Some(custom),
                VisitResult::Skip => Some(String::new()),
                VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                VisitResult::Error(err) => {
                    if ctx.visitor_error.borrow().is_none() {
                        *ctx.visitor_error.borrow_mut() = Some(err);
                    }
                    None
                }
            }
        } else {
            None
        };

        #[cfg(feature = "visitor")]
        if let Some(custom_output) = strong_output {
            output.push_str(&custom_output);
        } else {
            let (prefix, suffix, trimmed) = chomp_inline(&content);
            if !content.trim().is_empty() {
                output.push_str(prefix);
                if ctx.in_strong {
                    output.push_str(trimmed);
                } else {
                    output.push(options.strong_em_symbol);
                    output.push(options.strong_em_symbol);
                    output.push_str(trimmed);
                    output.push(options.strong_em_symbol);
                    output.push(options.strong_em_symbol);
                }
                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
            } else if !content.is_empty() {
                output.push_str(prefix);
                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
            }
        }

        #[cfg(not(feature = "visitor"))]
        {
            let (prefix, suffix, trimmed) = chomp_inline(&content);
            if !content.trim().is_empty() {
                output.push_str(prefix);
                if ctx.in_strong {
                    output.push_str(trimmed);
                } else {
                    output.push(options.strong_em_symbol);
                    output.push(options.strong_em_symbol);
                    output.push_str(trimmed);
                    output.push(options.strong_em_symbol);
                    output.push(options.strong_em_symbol);
                }
                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
            } else if !content.is_empty() {
                output.push_str(prefix);
                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
            }
        }
    }
}

/// Handle emphasis/italic (em, i tags).
fn handle_emphasis(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    #[allow(unused_imports)]
    use crate::converter::{append_inline_suffix, chomp_inline, get_text_content, serialize_node, walk_node};

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    if ctx.in_code {
        // Suppress formatting in code context, just process children
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
        }
    } else {
        let mut content = String::with_capacity(64);
        let children = tag.children();
        {
            let em_ctx = Context {
                inline_depth: ctx.inline_depth + 1,
                ..ctx.clone()
            };
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, &mut content, options, &em_ctx, depth + 1, dom_ctx);
            }
        }

        #[cfg(feature = "visitor")]
        let em_output = if let Some(ref visitor_handle) = ctx.visitor {
            use crate::visitor::{NodeContext, NodeType, VisitResult};

            let text_content = get_text_content(node_handle, parser, dom_ctx);
            let attributes: BTreeMap<String, String> = tag
                .attributes()
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                .collect();

            let node_id = node_handle.get_inner();
            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

            let node_ctx = NodeContext {
                node_type: NodeType::Em,
                tag_name: tag.name().as_utf8_str().to_string(),
                attributes,
                depth,
                index_in_parent,
                parent_tag,
                is_inline: true,
            };

            let mut visitor = visitor_handle.borrow_mut();
            match visitor.visit_emphasis(&node_ctx, &text_content) {
                VisitResult::Continue => None,
                VisitResult::Custom(custom) => Some(custom),
                VisitResult::Skip => Some(String::new()),
                VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                VisitResult::Error(err) => {
                    if ctx.visitor_error.borrow().is_none() {
                        *ctx.visitor_error.borrow_mut() = Some(err);
                    }
                    None
                }
            }
        } else {
            None
        };

        #[cfg(feature = "visitor")]
        if let Some(custom_output) = em_output {
            output.push_str(&custom_output);
        } else {
            let (prefix, suffix, trimmed) = chomp_inline(&content);
            if !content.trim().is_empty() {
                output.push_str(prefix);
                output.push(options.strong_em_symbol);
                output.push_str(trimmed);
                output.push(options.strong_em_symbol);
                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
            } else if !content.is_empty() {
                output.push_str(prefix);
                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
            } else if let Some(class_value) = tag
                .attributes()
                .get("class")
                .and_then(|v| v.as_ref().map(|val| val.as_utf8_str().to_string()))
            {
                if class_value.contains("caret") && !output.ends_with(' ') {
                    output.push_str(" > ");
                }
            }
        }

        #[cfg(not(feature = "visitor"))]
        {
            let (prefix, suffix, trimmed) = chomp_inline(&content);
            if !content.trim().is_empty() {
                output.push_str(prefix);
                output.push(options.strong_em_symbol);
                output.push_str(trimmed);
                output.push(options.strong_em_symbol);
                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
            } else if !content.is_empty() {
                output.push_str(prefix);
                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
            } else if let Some(class_value) = tag
                .attributes()
                .get("class")
                .and_then(|v| v.as_ref().map(|val| val.as_utf8_str().to_string()))
            {
                if class_value.contains("caret") && !output.ends_with(' ') {
                    output.push_str(" > ");
                }
            }
        }
    }
}
