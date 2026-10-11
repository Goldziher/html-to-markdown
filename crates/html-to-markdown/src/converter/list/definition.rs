//! Definition list handling (dl, dt, dd elements).
//!
//! Processes definition lists with:
//! - Definition terms (dt)
//! - Definition descriptions (dd)
//! - Plain block formatting (no Pandoc colon syntax)

use super::ListContext;
#[cfg(feature = "visitor")]
use std::borrow::Cow;
use tl;

/// Handle definition list element (<dl>).
///
/// Groups dt/dd pairs and formats them with proper Markdown separation.
pub fn handle_dl(node_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String, context: ListContext<'_>) {
    let ListContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = context;
    let tag = match node_handle.get(parser) {
        Some(tl::Node::Tag(t)) => t,
        _ => return,
    };

    if ctx.convert_as_inline {
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                use crate::converter::walk_node;
                walk_node(
                    child_handle,
                    parser,
                    output,
                    crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
                );
            }
        }
        return;
    }

    let mut rendered = String::new();
    let children = tag.children();
    {
        let dl_ctx = crate::converter::list::utils::nested_block_context(output, ctx, options);
        for child_handle in children.top().iter() {
            crate::converter::walk_node(
                child_handle,
                parser,
                &mut rendered,
                crate::converter::block::container::HandlerContext::new(options, &dl_ctx, depth + 1, dom_ctx),
            );
        }
    }

    let trimmed =
        crate::converter::handlers::blockquote::container_content(tag, parser, &rendered, options.code_block_style);
    if !trimmed.is_empty() {
        // ~keep Inside a list item the list starts at the item's content column (issue #583).
        if ctx.in_list_item && !ctx.in_table_cell && !output.is_empty() {
            crate::converter::list::utils::start_block_in_list_item(output, ctx, options);
        } else if !output.is_empty() && !output.ends_with("\n\n") {
            output.push_str("\n\n");
        }
        crate::converter::block::horizontal_rule::separate_leading_rule(output, trimmed, ctx);
        output.push_str(trimmed);
        output.push_str("\n\n");
    }
}

/// Handle definition term element (<dt>).
///
/// Outputs the term text followed by a newline.
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_dt(node_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String, context: ListContext<'_>) {
    let ListContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = context;
    let tag = match node_handle.get(parser) {
        Some(tl::Node::Tag(t)) => t,
        _ => return,
    };

    let mut rendered = String::with_capacity(64);
    let child_context = crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx);
    let children = tag.children();
    {
        for child_handle in children.top().iter() {
            crate::converter::walk_node(child_handle, parser, &mut rendered, child_context);
        }
    }
    let trimmed = rendered.trim().to_owned();
    if trimmed.is_empty() {
        return;
    }

    #[cfg(feature = "visitor")]
    if let Some(ref visitor_handle) = ctx.visitor {
        use crate::visitor::{NodeContext, NodeType, VisitResult};

        let node_id = node_handle.get_inner();
        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);
        let node_ctx = NodeContext::with_lazy_attributes(
            NodeType::DefinitionTerm,
            Cow::Borrowed("dt"),
            tag,
            depth,
            index_in_parent,
            parent_tag.map(Cow::Borrowed),
            false,
        );
        let visit_result = {
            let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
            visitor.visit_definition_term(&node_ctx, &trimmed)
        };
        match visit_result {
            VisitResult::Continue => {}
            VisitResult::Skip => return,
            VisitResult::Custom(custom) => {
                output.push_str(&custom);
                if !ctx.convert_as_inline && !custom.ends_with('\n') {
                    output.push('\n');
                }
                return;
            }
            VisitResult::PreserveHtml => {
                use crate::converter::utility::serialization::serialize_node;
                output.push_str(&serialize_node(node_handle, parser));
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

    if ctx.convert_as_inline {
        output.push_str(&trimmed);
    } else {
        crate::converter::block::horizontal_rule::separate_leading_rule(output, &trimmed, ctx);
        output.push_str(&trimmed);
        output.push('\n');
    }
}

#[cfg(feature = "visitor")]
fn visit_definition_description(
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    output: &mut String,
    rendered: &str,
    context: ListContext<'_>,
) -> bool {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(ref visitor_handle) = context.ctx.visitor else {
        return false;
    };
    let node_id = node_handle.get_inner();
    let parent_tag = context.dom_ctx.parent_tag_name(node_id, parser);
    let index_in_parent = context.dom_ctx.get_sibling_index(node_id).unwrap_or(0);
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::DefinitionDescription,
        Cow::Borrowed("dd"),
        tag,
        context.depth,
        index_in_parent,
        parent_tag.map(Cow::Borrowed),
        false,
    );
    let visit_result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        visitor.visit_definition_description(&node_ctx, rendered)
    };
    match visit_result {
        VisitResult::Continue => false,
        VisitResult::Skip => true,
        VisitResult::Custom(custom) => {
            output.push_str(&custom);
            if !context.ctx.convert_as_inline && !custom.ends_with('\n') {
                output.push_str("\n\n");
            }
            true
        }
        VisitResult::PreserveHtml => {
            output.push_str(&crate::converter::utility::serialization::serialize_node(
                node_handle,
                parser,
            ));
            true
        }
        VisitResult::Error(err) => {
            if context.ctx.visitor_error.borrow().is_none() {
                *context.ctx.visitor_error.borrow_mut() = Some(err);
            }
            true
        }
    }
}

/// Handle definition description element (<dd>).
///
/// Outputs the description as a plain block.
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_dd(node_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String, context: ListContext<'_>) {
    let ListContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = context;
    let tag = match node_handle.get(parser) {
        Some(tl::Node::Tag(t)) => t,
        _ => return,
    };

    let mut rendered = String::with_capacity(128);
    let children = tag.children();
    {
        for child_handle in children.top().iter() {
            crate::converter::walk_node(
                child_handle,
                parser,
                &mut rendered,
                crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
            );
        }
    }

    let trimmed =
        crate::converter::handlers::blockquote::container_content(tag, parser, &rendered, options.code_block_style)
            .to_owned();
    if trimmed.is_empty() {
        return;
    }

    #[cfg(feature = "visitor")]
    if visit_definition_description(node_handle, tag, parser, output, &trimmed, context) {
        return;
    }

    if ctx.convert_as_inline {
        output.push_str(&trimmed);
    } else {
        crate::converter::block::horizontal_rule::separate_leading_rule(output, &trimmed, ctx);
        // ~keep Indented code cannot interrupt the paragraph of the term, so a blank line goes
        // ~keep before a description that starts with it.
        if trimmed.starts_with([' ', '\t']) && !output.is_empty() && !output.ends_with("\n\n") {
            output.push_str(if output.ends_with('\n') { "\n" } else { "\n\n" });
        }
        output.push_str(&trimmed);
        output.push_str("\n\n");
    }
}
