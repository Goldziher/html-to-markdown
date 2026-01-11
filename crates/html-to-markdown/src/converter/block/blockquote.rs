//! Handler for blockquote elements.
//!
//! Converts HTML blockquote tags to Markdown blockquotes with support for:
//! - Nested blockquotes
//! - Optional `cite` attribute attribution
//! - Proper indentation and prefixing with `> `
//! - Spacing management for various contexts
//! - Visitor callbacks for custom blockquote processing

use crate::options::ConversionOptions;
#[allow(unused_imports)]
use std::collections::BTreeMap;
use tl::{NodeHandle, Parser};

// Type aliases for Context and DomContext to avoid circular imports
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handle blockquote elements.
///
/// Processes blockquote content, applies `> ` prefix to each line,
/// handles optional `cite` attribution, and manages spacing.
pub(crate) fn handle(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    use crate::converter::walk_node;

    if ctx.convert_as_inline {
        if let Some(node) = node_handle.get(parser) {
            if let tl::Node::Tag(tag) = node {
                let children = tag.children();
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                }
            }
        }
        return;
    }

    let cite = if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            tag.attributes()
                .get("cite")
                .flatten()
                .map(|v| v.as_utf8_str().to_string())
        } else {
            None
        }
    } else {
        None
    };

    let blockquote_ctx = Context {
        blockquote_depth: ctx.blockquote_depth + 1,
        ..ctx.clone()
    };

    let mut content = String::with_capacity(256);
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let children = tag.children();
            for child_handle in children.top().iter() {
                walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    options,
                    &blockquote_ctx,
                    depth + 1,
                    dom_ctx,
                );
            }
        }
    }

    let trimmed_content = content.trim();

    #[cfg(feature = "visitor")]
    {
        if let Some(ref visitor) = ctx.visitor {
            use crate::visitor::{NodeContext, NodeType, VisitResult};

            if let Some(node) = node_handle.get(parser) {
                if let tl::Node::Tag(tag) = node {
                    let attributes: BTreeMap<String, String> = tag
                        .attributes()
                        .iter()
                        .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                        .collect();

                    let node_id = node_handle.get_inner();
                    let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                    let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                    let node_ctx = NodeContext {
                        node_type: NodeType::Blockquote,
                        tag_name: "blockquote".to_string(),
                        attributes,
                        depth,
                        index_in_parent,
                        parent_tag,
                        is_inline: false,
                    };

                    let mut visitor_ref = visitor.borrow_mut();
                    match visitor_ref.visit_blockquote(&node_ctx, trimmed_content, ctx.blockquote_depth) {
                        VisitResult::Continue => {}
                        VisitResult::Custom(custom) => {
                            output.push_str(&custom);
                            return;
                        }
                        VisitResult::Skip => return,
                        VisitResult::PreserveHtml => {
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
            }
        }
    }

    if !trimmed_content.is_empty() {
        if ctx.blockquote_depth > 0 {
            output.push_str("\n\n\n");
        } else if !output.is_empty() {
            if output.ends_with("\n\n") {
                // Paragraph already added \n\n; blockquote needs just \n
                output.truncate(output.len() - 1);
            } else if !output.ends_with('\n') {
                output.push_str("\n\n");
            } else if !output.ends_with("\n\n") {
                output.push('\n');
            }
        }

        let prefix = "> ";

        for line in trimmed_content.lines() {
            output.push_str(prefix);
            output.push_str(line.trim());
            output.push('\n');
        }

        if let Some(url) = cite {
            output.push('\n');
            output.push_str("— <");
            output.push_str(&url);
            output.push_str(">\n\n");
        }

        // Add trailing newlines only when appropriate for proper spacing
        // (matching paragraph conditional logic for CommonMark compliance)
        if !ctx.convert_as_inline && !ctx.in_table_cell && !ctx.in_list_item {
            while output.ends_with('\n') {
                output.truncate(output.len() - 1);
            }
            output.push_str("\n\n");
        }
    }
}

/// Serialize a node to HTML (used for PreserveHtml visitor result).
#[cfg(feature = "visitor")]
fn serialize_node_to_html(node_handle: &NodeHandle, parser: &Parser, output: &mut String) {
    use crate::converter::serialize_node_to_html as core_serialize;
    core_serialize(node_handle, parser, output);
}
