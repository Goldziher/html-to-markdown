//! Unordered list handling (ul, li elements).
//!
//! Processes unordered lists with support for:
//! - Bullet cycling based on nesting depth
//! - Nested list handling
//! - Loose/tight list detection
//! - Proper indentation

use super::utils::{
    add_list_leading_separator, add_nested_list_trailing_separator, calculate_list_nesting_depth, is_loose_list,
    process_list_children,
};
use crate::options::ConversionOptions;
use tl;

// Type aliases for Context and DomContext to avoid circular imports
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handle unordered list element (<ul>).
///
/// Detects loose/tight list format, handles nested bullets,
/// and processes list items with proper indentation.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_ul(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    add_list_leading_separator(output, ctx);

    let nested_depth = calculate_list_nesting_depth(ctx);
    let is_loose = is_loose_list(*node_handle, parser, dom_ctx);

    #[allow(unused_variables)]
    let tag = match node_handle.get(parser) {
        Some(tl::Node::Tag(t)) => t,
        _ => return,
    };

    #[cfg(feature = "visitor")]
    let list_output_start = output.len();

    #[cfg(feature = "visitor")]
    let mut list_start_custom: Option<String> = None;

    #[cfg(feature = "visitor")]
    if let Some(ref visitor_handle) = ctx.visitor {
        use crate::visitor::{NodeContext, NodeType, VisitResult};
        use std::collections::BTreeMap;

        let attributes: BTreeMap<String, String> = tag
            .attributes()
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
            .collect();

        let parent_tag = dom_ctx
            .parent_of(node_handle.get_inner())
            .and_then(|pid| dom_ctx.tag_name_for(dom_ctx.node_handle(pid).copied()?, parser))
            .map(|s| s.to_string());

        let index = dom_ctx.sibling_index(node_handle.get_inner()).unwrap_or(0);

        let node_ctx = NodeContext {
            node_type: NodeType::List,
            tag_name: "ul".to_string(),
            attributes,
            depth,
            index_in_parent: index,
            parent_tag,
            is_inline: false,
        };

        let mut visitor = visitor_handle.borrow_mut();
        match visitor.visit_list_start(&node_ctx, false) {
            VisitResult::Continue => {}
            VisitResult::Custom(custom) => {
                list_start_custom = Some(custom);
            }
            VisitResult::Skip => {
                return;
            }
            VisitResult::PreserveHtml => {
                use crate::converter::serialize_node_to_html;
                serialize_node_to_html(node_handle, parser, output);
                return;
            }
            VisitResult::Error(err) => {
                if ctx.visitor_error.borrow().is_none() {
                    *ctx.visitor_error.borrow_mut() = Some(err);
                }
                return;
            }
        }
    }

    process_list_children(
        *node_handle,
        parser,
        output,
        options,
        ctx,
        depth,
        false,
        is_loose,
        nested_depth,
        1,
        dom_ctx,
    );

    add_nested_list_trailing_separator(output, ctx);

    #[cfg(feature = "visitor")]
    if let Some(ref visitor_handle) = ctx.visitor {
        use crate::visitor::{NodeContext, NodeType, VisitResult};
        use std::collections::BTreeMap;

        let attributes: BTreeMap<String, String> = tag
            .attributes()
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
            .collect();

        let parent_tag = dom_ctx
            .parent_of(node_handle.get_inner())
            .and_then(|pid| dom_ctx.tag_name_for(dom_ctx.node_handle(pid).copied()?, parser))
            .map(|s| s.to_string());

        let index = dom_ctx.sibling_index(node_handle.get_inner()).unwrap_or(0);

        let node_ctx = NodeContext {
            node_type: NodeType::List,
            tag_name: "ul".to_string(),
            attributes,
            depth,
            index_in_parent: index,
            parent_tag,
            is_inline: false,
        };

        let list_content = &output[list_output_start..];

        let mut visitor = visitor_handle.borrow_mut();
        match visitor.visit_list_end(&node_ctx, false, list_content) {
            VisitResult::Continue => {
                if let Some(custom_start) = list_start_custom {
                    output.insert_str(list_output_start, &custom_start);
                }
            }
            VisitResult::Custom(custom) => {
                let children_output = output[list_output_start..].to_string();
                output.truncate(list_output_start);
                if let Some(custom_start) = list_start_custom {
                    output.push_str(&custom_start);
                }
                output.push_str(&children_output);
                output.push_str(&custom);
            }
            VisitResult::Skip => {
                output.truncate(list_output_start);
            }
            VisitResult::PreserveHtml => {
                output.truncate(list_output_start);
                use crate::converter::serialize_node_to_html;
                serialize_node_to_html(node_handle, parser, output);
            }
            VisitResult::Error(err) => {
                if ctx.visitor_error.borrow().is_none() {
                    *ctx.visitor_error.borrow_mut() = Some(err);
                }
                output.truncate(list_output_start);
            }
        }
    }
}

/// Public alias for `handle_ul` to match the expected module interface.
pub(crate) use handle_ul as handle;
