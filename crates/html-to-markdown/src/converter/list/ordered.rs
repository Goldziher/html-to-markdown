//! Ordered list handling (ol, li elements).
//!
//! Processes ordered lists with support for:
//! - Custom start counters
//! - Nested list handling
//! - Loose/tight list detection
//! - Proper indentation and numbering

use super::ListContext;
use super::utils::{
    DEFAULT_ORDERED_LIST_START, ListChildrenContext, add_list_leading_separator, add_nested_list_trailing_separator,
    calculate_list_nesting_depth, is_loose_list, parse_ordered_list_start, preceding_same_type_list_separator_comment,
    process_list_children, switched_delimiter,
};
use tl;

fn ordered_start(tag: &tl::HTMLTag) -> i64 {
    tag.attributes()
        .get("start")
        .flatten()
        .map_or(DEFAULT_ORDERED_LIST_START, |value| {
            parse_ordered_list_start(&value.as_utf8_str())
        })
}

fn leading_separator_delimiter(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: ListContext<'_>,
) -> Option<char> {
    let ListContext {
        options, ctx, dom_ctx, ..
    } = context;
    let separator_comment = preceding_same_type_list_separator_comment(*node_handle, parser, dom_ctx, "ol");
    add_list_leading_separator(output, ctx, options);
    if let Some(comment) = separator_comment {
        output.push_str(&comment);
        output.push_str("\n\n");
        None
    } else {
        switched_delimiter(output, ctx)
    }
}

/// Handle ordered list element (<ol>).
///
/// Extracts the `start` attribute to set initial counter value,
/// detects loose/tight list format, and processes list items.
#[allow(clippy::too_many_arguments)]
pub fn handle_ol(node_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String, context: ListContext<'_>) {
    if super::render_itemless_list_as_div(node_handle, parser, output, context) {
        return;
    }
    let ListContext { ctx, dom_ctx, .. } = context;
    let delimiter = leading_separator_delimiter(node_handle, parser, output, context);

    let nested_depth = calculate_list_nesting_depth(ctx);
    let is_loose = is_loose_list(*node_handle, parser, dom_ctx);

    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return;
    };

    let start = ordered_start(tag);

    #[cfg(feature = "visitor")]
    let list_output_start = output.len();

    #[cfg(feature = "visitor")]
    let list_start_custom = match super::visit_list_start(node_handle, tag, parser, output, context, true) {
        super::ListStartResult::Continue(custom) => custom,
        super::ListStartResult::Stop => return,
    };

    if !ctx.in_table_cell {
        if let Some(ref sc) = ctx.structure_collector {
            sc.borrow_mut().push_list_start(true);
        }
    }

    process_list_children(
        *node_handle,
        parser,
        output,
        ListChildrenContext {
            list: context,
            ordered: true,
            loose: is_loose,
            nested_depth,
            start_counter: start,
            delimiter,
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
            ordered: true,
        },
    );

    ctx.last_list.set(output, ctx, delimiter.unwrap_or('.'));
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
    let ListContext { ctx, dom_ctx, .. } = context;
    let delimiter = leading_separator_delimiter(node_handle, parser, output, context);

    let nested_depth = calculate_list_nesting_depth(ctx);
    let is_loose = is_loose_list(*node_handle, parser, dom_ctx);

    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };

    let start = ordered_start(tag);

    #[cfg(feature = "visitor")]
    let list_output_start = output.len();

    #[cfg(feature = "visitor")]
    let list_start_custom = match super::visit_list_start(node_handle, tag, parser, output, context, true) {
        super::ListStartResult::Continue(custom) => custom,
        super::ListStartResult::Stop => return None,
    };

    if !ctx.in_table_cell {
        if let Some(ref sc) = ctx.structure_collector {
            sc.borrow_mut().push_list_start(true);
        }
    }

    Some(super::utils::open_children_walk(
        *node_handle,
        parser,
        ListChildrenContext {
            list: context,
            ordered: true,
            loose: is_loose,
            nested_depth,
            start_counter: start,
            delimiter,
        },
        super::utils::ListWalkEnd {
            ordered: true,
            delimiter,
            #[cfg(feature = "visitor")]
            visitor: super::ListVisitorOutput {
                start: list_output_start,
                custom_start: list_start_custom,
                ordered: true,
            },
        },
        children,
    ))
}

/// Public alias for `handle_ol` to match the expected module interface.
pub use handle_ol as handle;
