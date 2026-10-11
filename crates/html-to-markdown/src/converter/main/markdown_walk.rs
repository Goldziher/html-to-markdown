//! The Markdown walk: the driver of `converter/walk.rs` over the DOM, with the work that
//! surrounds each handler.

use std::borrow::Cow;
use std::rc::Rc;

use super::table_parts::{is_table_part, walk_table_parts};
use super::{
    Begun, Context, DomContext, HandlerContext, TagCapture, begin_tag, djot_context, effective_max_depth, finish_tag,
    normalized_node_tag_name, separate_from_block,
};
use crate::converter::main_dispatch::{Dispatched, close_tag, dispatch_tag};
use crate::converter::walk::{Close, Open, Walker, drive};

/// Walk each node of `nodes` with its subtree, in order, and write the Markdown to `output`.
pub(super) fn walk_nodes(
    nodes: &[tl::NodeHandle],
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let mut walk = MarkdownWalk {
        parser,
        output,
        root: handler,
    };
    drive(&mut walk, nodes);
}

/// The Markdown walk of one buffer: `root` is the state of the nodes the walk starts from.
struct MarkdownWalk<'a, 'output> {
    parser: &'a tl::Parser<'a>,
    output: &'output mut String,
    root: HandlerContext<'a>,
}

/// The conversion state that the nodes of one subtree read.
#[derive(Clone)]
enum Scope<'a> {
    Root(&'a Context),
    Shared(Rc<Context>),
}

impl std::ops::Deref for Scope<'_> {
    type Target = Context;

    fn deref(&self) -> &Context {
        match self {
            Self::Root(ctx) => ctx,
            Self::Shared(ctx) => ctx,
        }
    }
}

/// An element whose children the walk visits before it ends the element.
struct Frame<'a> {
    node: tl::NodeHandle,
    /// The state of the element.
    scope: Scope<'a>,
    /// The state of the children, when the handler made one for them.
    child_scope: Option<Rc<Context>>,
    depth: usize,
    output_start: usize,
    first_writer_start: Option<usize>,
    /// The end of the element. `None` for a tag that is dropped: nothing ends it.
    end: Option<TagEnd<'a>>,
}

/// What ends an element after its children: the second half of its handler, then `finish_tag`.
struct TagEnd<'a> {
    tag: &'a tl::HTMLTag<'a>,
    tag_name: Cow<'a, str>,
    capture: TagCapture<'a>,
    close: Close,
}

/// What the entry of a node leaves to do.
enum Entered<'a> {
    Complete,
    Children {
        scope: Option<Rc<Context>>,
        child_scope: Option<Rc<Context>>,
        output_start: usize,
        end: Option<TagEnd<'a>>,
    },
}

impl<'a> Walker for MarkdownWalk<'a, '_> {
    type Node = tl::NodeHandle;
    type Frame = Frame<'a>;

    fn enter(
        &mut self,
        node_handle: tl::NodeHandle,
        parent: Option<&Frame<'a>>,
        children: &mut Vec<tl::NodeHandle>,
    ) -> Option<Frame<'a>> {
        let (scope, depth) = match parent {
            Some(frame) => (
                frame
                    .child_scope
                    .as_ref()
                    .map_or_else(|| frame.scope.clone(), |ctx| Scope::Shared(Rc::clone(ctx))),
                frame.depth + 1,
            ),
            None => (Scope::Root(self.root.ctx), self.root.depth),
        };
        let handler = HandlerContext::new(self.root.options, &scope, depth, self.root.dom_ctx);
        if is_table_part(&node_handle, self.parser, handler.dom_ctx) {
            walk_table_parts(&node_handle, self.parser, self.output, handler);
            return None;
        }
        scope.last_list.enter(self.output);
        // ~keep In a task item, the render of each node before the first content reports whether
        // ~keep it wrote, so the item knows which element wrote first (issue #650).
        let first_writer_start = scope
            .first_writer
            .as_ref()
            .filter(|first_writer| first_writer.is_open())
            .map(|_| self.output.len());
        match enter_node(
            &node_handle,
            self.parser,
            self.output,
            handler,
            self.root.dom_ctx,
            children,
        ) {
            Entered::Complete => {
                leave_node(node_handle, self.parser, self.output, &scope, first_writer_start);
                None
            }
            Entered::Children {
                scope: djot_scope,
                child_scope,
                output_start,
                end,
            } => Some(Frame {
                node: node_handle,
                scope: djot_scope.map_or(scope, Scope::Shared),
                child_scope,
                depth,
                output_start,
                first_writer_start,
                end,
            }),
        }
    }

    fn leave(&mut self, frame: Frame<'a>) {
        if let Some(end) = frame.end {
            let handler = HandlerContext::new(self.root.options, &frame.scope, frame.depth, self.root.dom_ctx);
            close_tag(end.close, self.parser, self.output, handler);
            finish_tag(end.capture, end.tag_name.as_ref(), end.tag, self.output, handler, false);
        }
        end_fresh_block_start(&frame.scope, self.output, frame.output_start);
        leave_node(
            frame.node,
            self.parser,
            self.output,
            &frame.scope,
            frame.first_writer_start,
        );
    }
}

/// Convert one DOM node up to its children.
fn enter_node<'a>(
    node_handle: &tl::NodeHandle,
    parser: &'a tl::Parser<'a>,
    output: &mut String,
    handler: HandlerContext<'_>,
    dom_ctx: &'a DomContext,
    children: &mut Vec<tl::NodeHandle>,
) -> Entered<'a> {
    let options = handler.options;
    let ctx = handler.ctx;
    let depth = handler.depth;
    let Some(node) = node_handle.get(parser) else {
        return Entered::Complete;
    };

    if depth >= effective_max_depth(options) {
        ctx.depth_limit_reached.set(true);
        return Entered::Complete;
    }

    separate_from_block(node, node_handle, parser, output, handler);

    let output_start = output.len();
    match node {
        tl::Node::Raw(bytes) => {
            let raw = bytes.as_utf8_str();
            crate::converter::text_node::process_text_node(
                raw.as_ref(),
                node_handle,
                parser,
                output,
                crate::converter::block::container::HandlerContext::new(options, ctx, depth, dom_ctx),
            );
        }

        tl::Node::Tag(_) => {
            if let Some(opened) = enter_tag(node_handle, parser, output, handler, dom_ctx, children) {
                return Entered::Children {
                    scope: opened.scope,
                    child_scope: opened.child_scope,
                    output_start,
                    end: opened.end,
                };
            }
        }

        tl::Node::Comment(_) => {}
    }
    end_fresh_block_start(ctx, output, output_start);
    Entered::Complete
}

/// ~keep Whatever wrote content ends the start of the document: text, and also an image, a
/// ~keep graphic or a form control. Text after it keeps its leading white space (issue #762).
/// ~keep An element that wrote nothing, or white space only, is not content.
fn end_fresh_block_start(ctx: &Context, output: &str, node_output_start: usize) {
    if ctx.at_fresh_block_start.get()
        && output
            .get(node_output_start..)
            .is_some_and(|written| !written.trim().is_empty())
    {
        ctx.at_fresh_block_start.set(false);
    }
}

/// End the walk of one node: the report to the task item and to the list state.
fn leave_node(
    node_handle: tl::NodeHandle,
    parser: &tl::Parser,
    output: &str,
    ctx: &Context,
    first_writer_start: Option<usize>,
) {
    if let (Some(first_writer), Some(start)) = (ctx.first_writer.as_ref(), first_writer_start) {
        first_writer.record(node_handle, parser, output.get(start..));
    }
    ctx.last_list.leave(output);
}

/// An element that `enter_tag` opened: its own state when the element made one, the state of its
/// children when the handler made one, and what ends it.
struct OpenedTag<'a> {
    scope: Option<Rc<Context>>,
    child_scope: Option<Rc<Context>>,
    end: Option<TagEnd<'a>>,
}

/// Start the element of `node_handle` and run its handler, or the first half of it. Returns
/// `None` when the element is complete.
fn enter_tag<'a>(
    node_handle: &tl::NodeHandle,
    parser: &'a tl::Parser<'a>,
    output: &mut String,
    handler: HandlerContext<'_>,
    dom_ctx: &'a DomContext,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<OpenedTag<'a>> {
    let tl::Node::Tag(tag) = node_handle.get(parser)? else {
        return None;
    };
    let tag_name = normalized_node_tag_name(node_handle, tag, parser, dom_ctx);
    let djot_scope = djot_context(node_handle, parser, tag_name.as_ref(), handler).map(Rc::new);
    let ctx = djot_scope.as_deref().unwrap_or(handler.ctx);
    let handler = HandlerContext::new(handler.options, ctx, handler.depth, handler.dom_ctx);
    match begin_tag(node_handle, tag, parser, output, handler, dom_ctx) {
        Begun::Complete => None,
        Begun::Children => {
            children.extend(tag.children().top().iter().copied());
            Some(OpenedTag {
                scope: djot_scope,
                child_scope: None,
                end: None,
            })
        }
        Begun::Open(capture) => {
            let dispatched = dispatch_tag(tag_name.as_ref(), node_handle, parser, output, handler, children);
            match dispatched {
                Dispatched::Complete { stop } => {
                    finish_tag(capture, tag_name.as_ref(), tag, output, handler, stop);
                    None
                }
                Dispatched::Opened(Open { scope, close }) => Some(OpenedTag {
                    scope: djot_scope,
                    child_scope: scope,
                    end: Some(TagEnd {
                        tag,
                        tag_name,
                        capture,
                        close,
                    }),
                }),
            }
        }
    }
}

/// Write an element that `open` starts to its end, in a walk of its own.
///
/// ~keep This is the bridge for a handler that the walk does not drive yet: such a handler calls
/// ~keep the handler of another element and needs the whole of its output on return.
pub fn walk_opened(
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
    open: impl FnOnce(&mut String, &mut Vec<tl::NodeHandle>) -> Option<Open>,
) {
    let mut children = Vec::new();
    let Some(Open { scope, close }) = open(output, &mut children) else {
        return;
    };
    let ctx = scope.as_deref().unwrap_or(handler.ctx);
    walk_nodes(
        &children,
        parser,
        output,
        HandlerContext::new(handler.options, ctx, handler.depth + 1, handler.dom_ctx),
    );
    close_tag(close, parser, output, handler);
}
