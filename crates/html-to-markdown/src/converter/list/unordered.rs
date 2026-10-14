//! Unordered list handling (ul, li elements).
//!
//! Processes unordered lists with support for:
//! - Bullet cycling based on nesting depth
//! - Nested list handling
//! - Loose/tight list detection
//! - Proper indentation

use super::ListContext;
use super::utils::{
    ListChildrenContext, add_list_leading_separator, add_nested_list_trailing_separator, calculate_list_nesting_depth,
    is_loose_list, preceding_same_type_list_separator_comment, process_list_children,
};
use tl;

/// Handle unordered list element (<ul>).
///
/// Detects loose/tight list format, handles nested bullets,
/// and processes list items with proper indentation.
#[allow(clippy::too_many_arguments)]
pub fn handle_ul(node_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String, context: ListContext<'_>) {
    if super::render_itemless_list_as_div(node_handle, parser, output, context) {
        return;
    }
    let ListContext {
        options, ctx, dom_ctx, ..
    } = context;
    let separator_comment = preceding_same_type_list_separator_comment(*node_handle, parser, dom_ctx, "ul");
    add_list_leading_separator(output, ctx, options);
    if let Some(comment) = separator_comment {
        output.push_str(&comment);
        output.push_str("\n\n");
    }

    let nested_depth = calculate_list_nesting_depth(ctx);
    let is_loose = is_loose_list(*node_handle, parser, dom_ctx);

    // ~keep reason: `tag` is only used when the visitor feature is active (for attribute collection).
    #[allow(unused_variables)]
    let tag = match node_handle.get(parser) {
        Some(tl::Node::Tag(t)) => t,
        _ => return,
    };

    #[cfg(feature = "visitor")]
    let list_output_start = output.len();

    #[cfg(feature = "visitor")]
    let list_start_custom = match super::visit_list_start(node_handle, tag, parser, output, context, false) {
        super::ListStartResult::Continue(custom) => custom,
        super::ListStartResult::Stop => return,
    };

    if !ctx.in_table_cell {
        if let Some(ref sc) = ctx.structure_collector {
            sc.borrow_mut().push_list_start(false);
        }
    }

    process_list_children(
        *node_handle,
        parser,
        output,
        ListChildrenContext {
            list: context,
            ordered: false,
            loose: is_loose,
            nested_depth,
            start_counter: 1,
            delimiter: None,
        },
    );

    if !ctx.in_table_cell {
        if let Some(ref sc) = ctx.structure_collector {
            sc.borrow_mut().push_list_end();
        }
    }

    add_nested_list_trailing_separator(output, ctx);

    #[cfg(feature = "visitor")]
    super::visit_list_end(
        node_handle,
        tag,
        parser,
        output,
        context,
        super::ListVisitorOutput {
            start: list_output_start,
            custom_start: list_start_custom,
            ordered: false,
        },
    );
}

pub fn open(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: ListContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<crate::converter::walk::Open> {
    if !super::has_list_item_child(*node_handle, parser, context.dom_ctx) {
        return crate::converter::block::div::open(
            node_handle,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                context.options,
                context.ctx,
                context.depth,
                context.dom_ctx,
            ),
            children,
        );
    }
    let ListContext {
        options, ctx, dom_ctx, ..
    } = context;
    let separator_comment = preceding_same_type_list_separator_comment(*node_handle, parser, dom_ctx, "ul");
    add_list_leading_separator(output, ctx, options);
    if let Some(comment) = separator_comment {
        output.push_str(&comment);
        output.push_str("\n\n");
    }

    let nested_depth = calculate_list_nesting_depth(ctx);
    let is_loose = is_loose_list(*node_handle, parser, dom_ctx);

    // ~keep reason: `tag` is only used when the visitor feature is active (for attribute collection).
    #[allow(unused_variables)]
    let tag = match node_handle.get(parser) {
        Some(tl::Node::Tag(t)) => t,
        _ => return None,
    };

    #[cfg(feature = "visitor")]
    let list_output_start = output.len();

    #[cfg(feature = "visitor")]
    let list_start_custom = match super::visit_list_start(node_handle, tag, parser, output, context, false) {
        super::ListStartResult::Continue(custom) => custom,
        super::ListStartResult::Stop => return None,
    };

    if !ctx.in_table_cell {
        if let Some(ref sc) = ctx.structure_collector {
            sc.borrow_mut().push_list_start(false);
        }
    }

    Some(super::utils::open_children_walk(
        *node_handle,
        parser,
        ListChildrenContext {
            list: context,
            ordered: false,
            loose: is_loose,
            nested_depth,
            start_counter: 1,
            delimiter: None,
        },
        super::utils::ListWalkEnd {
            ordered: false,
            delimiter: None,
            #[cfg(feature = "visitor")]
            visitor: super::ListVisitorOutput {
                start: list_output_start,
                custom_start: list_start_custom,
                ordered: false,
            },
        },
        children,
    ))
}

/// Public alias for `handle_ul` to match the expected module interface.
pub use handle_ul as handle;
