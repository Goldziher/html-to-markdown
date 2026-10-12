use std::rc::Rc;

use super::{Context, DomContext, HandlerContext, TagCapture};

pub(super) fn is_table_part(node: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    dom_ctx
        .tag_info(node.get_inner(), parser)
        .is_some_and(|info| matches!(info.name.as_str(), "td" | "th" | "tr" | "thead" | "tbody" | "tfoot"))
}

struct TablePartExit<'a> {
    node: tl::NodeHandle,
    depth: usize,
    context: Option<Rc<Context>>,
    walk_context: Option<Rc<Context>>,
    capture: Option<TagCapture<'a>>,
    first_writer_start: Option<usize>,
}

enum TablePartStep<'a> {
    Enter(tl::NodeHandle, usize, Option<Rc<Context>>),
    Exit(TablePartExit<'a>),
}

// ~keep Malformed table parts can form a 20,000-node chain. Keep their enter/exit hooks and
// ~keep authored depth on a heap stack instead of adding a native call frame per ignored tag.
pub(super) fn walk_table_parts<'a>(
    node: &tl::NodeHandle,
    parser: &'a tl::Parser<'a>,
    output: &mut String,
    handler: HandlerContext<'a>,
) {
    let mut pending = vec![TablePartStep::Enter(*node, handler.depth, None)];
    while let Some(step) = pending.pop() {
        match step {
            TablePartStep::Exit(exit) => finish_part(exit, parser, output, handler),
            TablePartStep::Enter(node, depth, context) => {
                let ctx = context.as_deref().unwrap_or(handler.ctx);
                let local = HandlerContext::new(handler.options, ctx, depth, handler.dom_ctx);
                if !is_table_part(&node, parser, handler.dom_ctx) {
                    super::walk_node(&node, parser, output, local);
                    continue;
                }
                let Some(tl::Node::Tag(tag)) = node.get(parser) else {
                    continue;
                };
                ctx.last_list.enter(output);
                let first_writer_start = ctx
                    .first_writer
                    .as_ref()
                    .filter(|writer| writer.is_open())
                    .map(|_| output.len());
                if depth >= super::effective_max_depth(handler.options) {
                    ctx.depth_limit_reached.set(true);
                    pending.push(TablePartStep::Exit(TablePartExit {
                        node,
                        depth,
                        walk_context: context.clone(),
                        context,
                        capture: None,
                        first_writer_start,
                    }));
                    continue;
                }
                if let Some(dom_node) = node.get(parser) {
                    super::separate_from_block(dom_node, &node, parser, output, local);
                }
                let tag_name = super::normalized_node_tag_name(&node, tag, parser, handler.dom_ctx);
                let walk_context = context.clone();
                let context = super::djot_context(&node, parser, tag_name.as_ref(), local)
                    .map(Rc::new)
                    .or(context);
                let ctx = context.as_deref().unwrap_or(handler.ctx);
                let local = HandlerContext::new(handler.options, ctx, depth, handler.dom_ctx);
                let capture = super::begin_tag(&node, tag, parser, output, local, handler.dom_ctx);
                let should_descend = capture.is_some();
                pending.push(TablePartStep::Exit(TablePartExit {
                    node,
                    depth,
                    context: context.clone(),
                    walk_context,
                    capture,
                    first_writer_start,
                }));
                if should_descend {
                    let start = pending.len();
                    for child in tag.children().top().iter() {
                        pending.push(TablePartStep::Enter(*child, depth + 1, context.clone()));
                    }
                    pending[start..].reverse();
                }
            }
        }
    }
}

fn finish_part(exit: TablePartExit<'_>, parser: &tl::Parser, output: &mut String, handler: HandlerContext<'_>) {
    let ctx = exit.context.as_deref().unwrap_or(handler.ctx);
    if let (Some(capture), Some(tl::Node::Tag(tag))) = (exit.capture, exit.node.get(parser)) {
        let tag_name = super::normalized_node_tag_name(&exit.node, tag, parser, handler.dom_ctx);
        super::finish_tag(
            capture,
            tag_name.as_ref(),
            tag,
            output,
            HandlerContext::new(handler.options, ctx, exit.depth, handler.dom_ctx),
            false,
        );
    }
    let ctx = exit.walk_context.as_deref().unwrap_or(handler.ctx);
    if let Some(start) = exit.first_writer_start {
        if let Some(writer) = ctx.first_writer.as_ref() {
            writer.record(exit.node, parser, output.get(start..));
        }
    }
    ctx.last_list.leave(output);
}
