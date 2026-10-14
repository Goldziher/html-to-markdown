//! The Markdown walk: the driver of `converter/walk.rs` over the DOM, with the work that
//! surrounds each handler.

use std::borrow::Cow;
use std::rc::Rc;

use super::super::context::MediaBoundaryState;
use super::table_parts::{is_table_part, walk_table_parts};
use super::{
    Begun, Context, DomContext, HandlerContext, TagCapture, begin_tag, djot_context, effective_max_depth, finish_tag,
    normalized_node_tag_name, separate_from_block,
};
use crate::converter::main_dispatch::{Dispatched, close_capture, close_tag, dispatch_tag};
use crate::converter::walk::{Child, Close, HookSite, Open, Scratch, Walker, drive};

/// Walk each node of `nodes` with its subtree, in order, and write the Markdown to `output`.
pub(super) fn walk_nodes(
    nodes: &[tl::NodeHandle],
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    // ~keep Text and comments open no element, so a walk of them needs no vector.
    let opens = nodes
        .iter()
        .any(|node| matches!(node.get(parser), Some(tl::Node::Tag(_))));
    let spares = &handler.ctx.walk_spares;
    let mut scratch = if opens { spares.take() } else { Scratch::default() };
    let mut walk = MarkdownWalk {
        parser,
        output,
        root: handler,
        captures: Vec::new(),
        spare_captures: Vec::new(),
    };
    drive(&mut walk, nodes, &mut scratch);
    if opens {
        spares.give(scratch);
    }
}

/// The vectors of the walks that ended, for the walks that start.
///
/// ~keep A handler that the walk does not drive yet starts a walk for each child. Without the
/// ~keep spare vectors each of those walks allocates its own.
#[derive(Default)]
pub struct WalkSpares(std::cell::RefCell<Vec<Scratch<tl::NodeHandle, Frame<'static>>>>);

impl WalkSpares {
    fn take<'a>(&self) -> Scratch<tl::NodeHandle, Frame<'a>> {
        self.0.borrow_mut().pop().unwrap_or_default().with_frames()
    }

    fn give(&self, scratch: Scratch<tl::NodeHandle, Frame<'_>>) {
        self.0.borrow_mut().push(scratch.with_frames());
    }
}

/// The Markdown walk of one buffer: `root` is the state of the nodes the walk starts from.
struct MarkdownWalk<'a, 'output> {
    parser: &'a tl::Parser<'a>,
    output: &'output mut String,
    root: HandlerContext<'a>,
    /// The buffers of the open elements that capture their children, the innermost last.
    captures: Vec<CaptureBuffer>,
    spare_captures: Vec<CaptureBuffer>,
}

/// The buffer that the children of a capturing element write to.
///
/// ~keep The text is boxed: a block in the buffer keeps the address of the buffer
/// ~keep (`Context::block_output_ptr`), and the list of buffers moves its items when it grows.
#[derive(Default)]
struct CaptureBuffer(Box<CaptureText>);

/// The text of a capture, at an address that does not change while the capture is open.
#[derive(Default)]
struct CaptureText(String);

fn end_media_boundary(scope: &Context, media_state: Option<MediaBoundaryState>, output: &mut String, start: usize) {
    if let (Some(boundary), Some(incoming)) = (scope.media_boundary.as_ref(), media_state) {
        super::finish_media_boundary(output, start, boundary, incoming);
    }
}

/// Where the output of the node starts, when the node is among those that a task item watches
/// for its first writer.
fn first_writer_start(scope: &Context, node_handle: &tl::NodeHandle, output: &str) -> Option<usize> {
    scope
        .first_writer
        .as_ref()
        .filter(|first_writer| {
            first_writer.is_open() && !scope.task_checkbox_ancestors.contains(&node_handle.get_inner())
        })
        .map(|_| output.len())
}

/// The state that an element reads when its handler made none: the state of its parent's children.
fn inherited_scope<'a>(parent: Option<&Frame<'a>>, root_ctx: &'a Context) -> Scope<'a> {
    match parent {
        Some(frame) => frame
            .child_scope
            .as_ref()
            .map_or_else(|| frame.scope.clone(), |ctx| Scope::Shared(Rc::clone(ctx))),
        None => Scope::Root(root_ctx),
    }
}

/// Move the options that a close carries to the frame, for the children to read.
fn adopt_close_options(frame: &mut Frame<'_>) {
    if let Some(end) = frame.end.as_mut() {
        if matches!(end.close, Close::WithOptions(_, _)) {
            if let Close::WithOptions(options, close) = std::mem::replace(&mut end.close, Close::Nothing) {
                frame.options = Some(options);
                end.close = *close;
            }
        }
    }
}

/// Whether the buffer of the capture owns the line breaks of the inline content in it.
fn owns_inline_break(close: &Close) -> bool {
    use crate::converter::walk::CaptureEnd;
    matches!(
        close,
        Close::Capture(
            CaptureEnd::Blockquote
                | CaptureEnd::Figcaption
                | CaptureEnd::TaskItem(_)
                | CaptureEnd::Definition("dt" | "dd")
        )
    )
}

/// The buffer that the walk writes to now: the innermost capture, or the output of the walk.
fn current_output<'s>(captures: &'s mut [CaptureBuffer], output: &'s mut String) -> &'s mut String {
    match captures.last_mut() {
        Some(buffer) => &mut buffer.0.0,
        None => output,
    }
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
    options: Option<Rc<crate::options::ConversionOptions>>,
    output_start: usize,
    first_writer_start: Option<usize>,
    /// The end of the element. `None` for a tag that is dropped: nothing ends it.
    end: Option<Box<TagEnd<'a>>>,
    /// Whether the child that the walk visits now writes to a buffer that the hook gave for it.
    child_has_buffer: bool,
    child_depth: Option<usize>,
    media_start: usize,
    media_state: Option<super::super::context::MediaBoundaryState>,
}

impl Frame<'_> {
    /// Whether the children of the element write to a buffer of their own.
    fn captures(&self) -> bool {
        self.end.as_ref().is_some_and(|end| end.close.captures())
    }
}

/// What ends an element after its children: the second half of its handler, then `finish_tag`.
struct TagEnd<'a> {
    tag: &'a tl::HTMLTag<'a>,
    tag_name: Cow<'a, str>,
    capture: TagCapture<'a>,
    close: Close,
    passthrough_start: Option<usize>,
}

/// What the entry of a node leaves to do.
enum Entered<'a> {
    Complete,
    Children {
        scope: Option<Rc<Context>>,
        child_scope: Option<Rc<Context>>,
        output_start: usize,
        end: Option<Box<TagEnd<'a>>>,
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
        // ~keep The node reads the state of its parent in place. Only an element that opens takes
        // ~keep a share of it, so text costs no count of shares.
        let (scope, depth): (&Context, usize) = match parent {
            Some(frame) => (
                frame.child_scope.as_deref().unwrap_or(&frame.scope),
                frame.child_depth.unwrap_or(frame.depth + 1),
            ),
            None => (self.root.ctx, self.root.depth),
        };
        let options = parent
            .and_then(|frame| frame.options.as_deref())
            .unwrap_or(self.root.options);
        let handler = HandlerContext::new(options, scope, depth, self.root.dom_ctx);
        let output = current_output(&mut self.captures, self.output);
        let media_start = output.len();
        let media_state = scope
            .media_boundary
            .as_ref()
            .map(|state| state.replace(Default::default()));
        if is_table_part(&node_handle, self.parser, handler.dom_ctx) {
            walk_table_parts(&node_handle, self.parser, output, handler);
            end_media_boundary(scope, media_state, output, media_start);
            return None;
        }
        scope.last_list.enter(output);
        // ~keep In a task item, the render of each node before the first content reports whether
        // ~keep it wrote, so the item knows which element wrote first (issue #650).
        let first_writer_start = first_writer_start(scope, &node_handle, output);
        let mut frame = match enter_node(&node_handle, self.parser, output, handler, self.root.dom_ctx, children) {
            Entered::Complete => {
                leave_node(node_handle, self.parser, output, scope, first_writer_start);
                end_media_boundary(scope, media_state, output, media_start);
                return None;
            }
            Entered::Children {
                scope: djot_scope,
                child_scope,
                output_start,
                end,
            } => Frame {
                node: node_handle,
                scope: djot_scope.map_or_else(|| inherited_scope(parent, self.root.ctx), Scope::Shared),
                child_scope,
                depth,
                options: parent.and_then(|frame| frame.options.clone()),
                output_start,
                first_writer_start,
                end,
                child_has_buffer: false,
                child_depth: None,
                media_start,
                media_state,
            },
        };
        adopt_close_options(&mut frame);
        if frame.captures() {
            let buffer = self.spare_captures.pop().unwrap_or_default();
            if frame.end.as_ref().is_some_and(|end| owns_inline_break(&end.close)) {
                let context = frame
                    .child_scope
                    .as_deref()
                    .unwrap_or(&frame.scope)
                    .with_inline_break_owner(&buffer.0.0);
                frame.child_scope = Some(Rc::new(context));
            }
            self.captures.push(buffer);
        }
        Some(frame)
    }

    fn leave(&mut self, frame: Frame<'a>) {
        let captured = if frame.captures() { self.captures.pop() } else { None };
        let output = current_output(&mut self.captures, self.output);
        if let Some(end) = frame.end {
            let handler = HandlerContext::new(
                frame.options.as_deref().unwrap_or(self.root.options),
                &frame.scope,
                frame.depth,
                self.root.dom_ctx,
            );
            let normal_unknown = matches!(end.close, Close::Unknown(_));
            match (end.close, captured.as_ref()) {
                (Close::Capture(close), Some(buffer)) => {
                    close_capture(close, &frame.node, &buffer.0.0, self.parser, output, handler);
                }
                (Close::Hooked(hook), _) => {
                    hook.close(&mut hook_site(&frame.node, self.parser, output, handler));
                }
                (close, _) => close_tag(close, self.parser, output, handler),
            }
            finish_tag(end.capture, end.tag_name.as_ref(), end.tag, output, handler, false);
            if let Some(written) = end
                .passthrough_start
                .filter(|_| normal_unknown || end.tag_name == "font")
                .and_then(|start| output.get(start..))
            {
                handler
                    .dom_ctx
                    .record_blank_passthrough_render(frame.node.get_inner(), written);
            }
        }
        end_fresh_block_start(&frame.scope, output, frame.output_start);
        leave_node(frame.node, self.parser, output, &frame.scope, frame.first_writer_start);
        if let (Some(boundary), Some(incoming)) = (frame.scope.media_boundary.as_ref(), frame.media_state) {
            super::finish_media_boundary(output, frame.media_start, boundary, incoming);
        }
        if let Some(mut buffer) = captured {
            buffer.0.0.clear();
            self.spare_captures.push(buffer);
        }
    }

    fn hooks_children(&self, frame: &Frame<'a>) -> bool {
        frame.end.as_ref().is_some_and(|end| end.close.hooks_children())
    }

    fn before_child(&mut self, frame: &mut Frame<'a>, index: usize) -> bool {
        let Some(Close::Hooked(hook)) = frame.end.as_mut().map(|end| &mut end.close) else {
            return true;
        };
        let handler = HandlerContext::new(
            frame.options.as_deref().unwrap_or(self.root.options),
            &frame.scope,
            frame.depth,
            self.root.dom_ctx,
        );
        let output = current_output(&mut self.captures, self.output);
        let child = hook.before_child(index, &mut hook_site(&frame.node, self.parser, output, handler));
        let Child::Walk { buffer, scope, depth } = child else {
            return false;
        };
        frame.child_scope = scope;
        frame.child_depth = depth;
        frame.child_has_buffer = buffer.is_some();
        if let Some(text) = buffer {
            let mut capture = self.spare_captures.pop().unwrap_or_default();
            capture.0.0 = text;
            self.captures.push(capture);
        }
        true
    }

    fn after_child(&mut self, frame: &mut Frame<'a>, index: usize) {
        let Some(Close::Hooked(hook)) = frame.end.as_mut().map(|end| &mut end.close) else {
            return;
        };
        let own = if frame.child_has_buffer {
            self.captures.pop().map(|mut capture| {
                let text = std::mem::take(&mut capture.0.0);
                self.spare_captures.push(capture);
                text
            })
        } else {
            None
        };
        let handler = HandlerContext::new(
            frame.options.as_deref().unwrap_or(self.root.options),
            &frame.scope,
            frame.depth,
            self.root.dom_ctx,
        );
        let output = current_output(&mut self.captures, self.output);
        hook.after_child(index, own, &mut hook_site(&frame.node, self.parser, output, handler));
    }
}

/// The element of a hook, as the hook sees it: `output` is the buffer of the element.
const fn hook_site<'s>(
    node_handle: &'s tl::NodeHandle,
    parser: &'s tl::Parser<'s>,
    output: &'s mut String,
    handler: HandlerContext<'s>,
) -> HookSite<'s> {
    HookSite::new((
        node_handle,
        parser,
        output,
        handler.options,
        handler.ctx,
        handler.depth,
        handler.dom_ctx,
    ))
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

    if ctx.task_checkbox == Some(*node_handle) {
        return Entered::Complete;
    }
    if ctx.task_checkbox_ancestors.contains(&node_handle.get_inner()) {
        if let tl::Node::Tag(tag) = node {
            children.extend(tag.children().top().iter().copied());
            return Entered::Children {
                scope: None,
                child_scope: None,
                output_start: output.len(),
                end: None,
            };
        }
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
    end: Option<Box<TagEnd<'a>>>,
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
    let passthrough_start = dom_ctx.begin_passthrough_render(node_handle.get_inner(), output.len(), handler);
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
                    end: Some(Box::new(TagEnd {
                        tag,
                        tag_name,
                        capture,
                        close,
                        passthrough_start,
                    })),
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
    debug_assert!(
        !close.captures() && !close.hooks_children(),
        "only the dispatcher opens an element that captures or hooks its children"
    );
    let ctx = scope.as_deref().unwrap_or(handler.ctx);
    walk_nodes(
        &children,
        parser,
        output,
        HandlerContext::new(handler.options, ctx, handler.depth + 1, handler.dom_ctx),
    );
    close_tag(close, parser, output, handler);
}
