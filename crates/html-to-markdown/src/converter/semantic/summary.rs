//! Handlers for HTML5 interactive elements.
//!
//! Processes interactive disclosure semantic elements:
//! - `<details>` - Expandable/collapsible disclosure widget
//! - `<summary>` - Summary or caption for a details element
//!
//! These elements are treated as block-level content containers
//! with special formatting for the summary element.

#[cfg(feature = "visitor")]
use std::borrow::Cow;

/// Handles the `<details>` element.
///
/// A details element represents a disclosure widget that can be toggled
/// to show/hide additional content. In Markdown, it's rendered as a block
/// with all content visible.
///
/// # Behavior
///
/// - **Inline mode**: Children are processed inline without block spacing
/// - **Block mode**: Content is collected and wrapped with proper blank-line spacing
/// - **Empty content**: Skipped entirely
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_details(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        #[cfg(feature = "visitor")]
        if let Some(ref visitor_handle) = handler.ctx.visitor {
            use crate::visitor::{NodeContext, NodeType, VisitResult};

            let node_id = node_handle.get_inner();
            let parent_tag = handler.dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0);
            let open = tag.attributes().get("open").is_some();
            let node_ctx = NodeContext::with_lazy_attributes(
                NodeType::Details,
                Cow::Borrowed("details"),
                tag,
                handler.depth,
                index_in_parent,
                parent_tag.map(Cow::Borrowed),
                false,
            );
            let visit_result = {
                let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
                visitor.visit_details(&node_ctx, open)
            };
            match visit_result {
                VisitResult::Continue => {}
                VisitResult::Skip => return,
                VisitResult::Custom(custom) => {
                    if !output.is_empty() && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }
                    output.push_str(&custom);
                    output.push_str("\n\n");
                    return;
                }
                VisitResult::PreserveHtml => {
                    use crate::converter::utility::serialization::serialize_node;
                    output.push_str(&serialize_node(node_handle, parser));
                    return;
                }
                VisitResult::Error(err) => {
                    if handler.ctx.visitor_error.borrow().is_none() {
                        *handler.ctx.visitor_error.borrow_mut() = Some(err);
                    }
                    return;
                }
            }
        }

        crate::converter::block::div::handle(node_handle, parser, output, handler);
    }
}

pub fn open_details(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<crate::converter::walk::Open> {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        #[cfg(not(feature = "visitor"))]
        let _ = tag;
        #[cfg(feature = "visitor")]
        if let Some(ref visitor_handle) = handler.ctx.visitor {
            use crate::visitor::{NodeContext, NodeType, VisitResult};

            let node_id = node_handle.get_inner();
            let parent_tag = handler.dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0);
            let open = tag.attributes().get("open").is_some();
            let node_ctx = NodeContext::with_lazy_attributes(
                NodeType::Details,
                Cow::Borrowed("details"),
                tag,
                handler.depth,
                index_in_parent,
                parent_tag.map(Cow::Borrowed),
                false,
            );
            let visit_result = {
                let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
                visitor.visit_details(&node_ctx, open)
            };
            match visit_result {
                VisitResult::Continue => {}
                VisitResult::Skip => return None,
                VisitResult::Custom(custom) => {
                    if !output.is_empty() && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }
                    output.push_str(&custom);
                    output.push_str("\n\n");
                    return None;
                }
                VisitResult::PreserveHtml => {
                    use crate::converter::utility::serialization::serialize_node;
                    output.push_str(&serialize_node(node_handle, parser));
                    return None;
                }
                VisitResult::Error(err) => {
                    if handler.ctx.visitor_error.borrow().is_none() {
                        *handler.ctx.visitor_error.borrow_mut() = Some(err);
                    }
                    return None;
                }
            }
        }

        return crate::converter::block::div::open(node_handle, parser, output, handler, children);
    }
    None
}

/// Handles the `<summary>` element.
///
/// A summary element contains a caption for a details element.
/// It is rendered as strong (bold) text to distinguish it from regular content.
///
/// # Behavior
///
/// - **Inline mode**: Content is rendered inline without emphasis
/// - **Block mode**: Content is wrapped in strong markers (e.g., `**text**`)
/// - Uses the configured strong/emphasis symbol from `ConversionOptions`
///
/// # Implementation Details
///
/// The handler:
/// 1. Creates a context with `in_strong: true` for nested formatting
/// 2. Collects content from all children
/// 3. Wraps non-empty content in strong markers (repeated twice per Markdown spec)
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_summary(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut content = String::with_capacity(64);

        // ~keep Set strong context for nested content. Skip the clone entirely
        // ~keep when the resulting Context would be byte-identical to ctx —
        // ~keep either convert_as_inline (block path inactive) or already
        // ~keep in_strong upstream.
        let want_strong = !handler.ctx.convert_as_inline;
        let summary_ctx_owned;
        let summary_ctx = if want_strong && !handler.ctx.in_strong {
            summary_ctx_owned = super::Context {
                in_strong: true,
                text_in_markers: true,
                ..handler.ctx.clone()
            };
            &summary_ctx_owned
        } else {
            handler.ctx
        };

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    crate::converter::block::container::HandlerContext::new(
                        handler.options,
                        summary_ctx,
                        handler.depth + 1,
                        handler.dom_ctx,
                    ),
                );
            }
        }

        // ~keep A trailing <br> run with no following sibling has no next dispatch to catch
        // ~keep it in `walk_node`'s pre-block-dispatch strip, since the summary content is
        // ~keep simply finished here — so this closes its own trailing run the same way
        // ~keep `paragraph.rs` closes its own (issue #464 follow-up).
        crate::converter::main_helpers::strip_trailing_backslash_breaks_from_fresh_buffer(
            &mut content,
            handler.options.newline_style,
        );

        let trimmed = content.trim();
        if trimmed.is_empty() {
            return;
        }

        #[cfg(feature = "visitor")]
        if visit_summary(tag, node_handle, parser, output, trimmed, handler) {
            return;
        }

        if handler.ctx.convert_as_inline {
            output.push_str(trimmed);
        } else {
            render_summary_block(output, trimmed, node_handle, parser, handler);
        }
    }
}

fn render_summary_block(
    output: &mut String,
    content: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    handler: super::HandlerContext<'_>,
) {
    let mut bold = String::with_capacity(content.len() + 4);
    crate::converter::inline::emphasis::emit_strong_wrapped(
        &mut bold,
        content,
        crate::converter::inline::wrapped::InlineSite {
            node_handle,
            parser,
            dom_ctx: handler.dom_ctx,
            ctx: handler.ctx,
            options: handler.options,
        },
    );
    crate::converter::block::div::push_block(output, handler.options, handler.ctx, &bold);
}

#[cfg(feature = "visitor")]
fn visit_summary(
    tag: &tl::HTMLTag<'_>,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    content: &str,
    handler: super::HandlerContext<'_>,
) -> bool {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(visitor_handle) = handler.ctx.visitor.as_ref() else {
        return false;
    };
    let node_id = node_handle.get_inner();
    let parent_tag = handler.dom_ctx.parent_tag_name(node_id, parser);
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::Summary,
        Cow::Borrowed("summary"),
        tag,
        handler.depth,
        handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0),
        parent_tag.map(Cow::Borrowed),
        false,
    );
    let result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_summary(&node_ctx, content);
    match result {
        VisitResult::Continue => false,
        VisitResult::Skip => true,
        VisitResult::Custom(custom) => {
            output.push_str(&custom);
            if !handler.ctx.convert_as_inline && !custom.ends_with('\n') {
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
        VisitResult::Error(error) => {
            if handler.ctx.visitor_error.borrow().is_none() {
                *handler.ctx.visitor_error.borrow_mut() = Some(error);
            }
            true
        }
    }
}

/// Dispatcher for interactive elements.
///
/// Routes `<details>` and `<summary>` elements to their respective handlers.
pub fn handle(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    match tag_name {
        "details" => handle_details(tag_name, node_handle, parser, output, handler),
        "summary" => handle_summary(tag_name, node_handle, parser, output, handler),
        _ => {}
    }
}

pub fn open_summary(
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    handler: super::HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<crate::converter::walk::Open> {
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    let scope = (!handler.ctx.convert_as_inline && !handler.ctx.in_strong).then(|| {
        std::rc::Rc::new(super::Context {
            in_strong: true,
            text_in_markers: true,
            ..handler.ctx.clone()
        })
    });
    Some(crate::converter::walk::Open {
        scope,
        close: crate::converter::walk::Close::Capture(crate::converter::walk::CaptureEnd::Summary),
    })
}

pub fn close_summary(
    captured: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return;
    };
    #[cfg(not(feature = "visitor"))]
    let _ = tag;
    let mut content = captured.to_owned();
    // ~keep A trailing <br> run with no following sibling has no next dispatch to catch
    // ~keep it in `walk_node`'s pre-block-dispatch strip, since the summary content is
    // ~keep simply finished here — so this closes its own trailing run the same way
    // ~keep `paragraph.rs` closes its own (issue #464 follow-up).
    crate::converter::main_helpers::strip_trailing_backslash_breaks_from_fresh_buffer(
        &mut content,
        handler.options.newline_style,
    );

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return;
    }

    #[cfg(feature = "visitor")]
    if visit_summary(tag, node_handle, parser, output, trimmed, handler) {
        return;
    }

    if handler.ctx.convert_as_inline {
        output.push_str(trimmed);
    } else {
        render_summary_block(output, trimmed, node_handle, parser, handler);
    }
}
