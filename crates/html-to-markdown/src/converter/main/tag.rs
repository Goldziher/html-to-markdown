use super::{HandlerContext, begin_tag, djot_context, finish_tag, normalized_node_tag_name};

pub(super) fn convert_tag(
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let tag_name = normalized_node_tag_name(node_handle, tag, parser, handler.dom_ctx);
    let djot_scope = djot_context(node_handle, parser, tag_name.as_ref(), handler);
    let ctx = djot_scope.as_ref().unwrap_or(handler.ctx);
    let handler = HandlerContext::new(handler.options, ctx, handler.depth, handler.dom_ctx);
    let font_start = handler
        .dom_ctx
        .begin_font_render(node_handle.get_inner(), parser, output.len(), handler);
    let Some(capture) = begin_tag(node_handle, tag, parser, output, handler, handler.dom_ctx) else {
        return;
    };
    let stop = super::super::main_dispatch::dispatch_tag(tag_name.as_ref(), node_handle, tag, parser, output, handler);
    finish_tag(capture, tag_name.as_ref(), tag, output, handler, stop);
    if let Some(written) = font_start.and_then(|start| output.get(start..)) {
        handler
            .dom_ctx
            .record_blank_font_render(node_handle.get_inner(), written);
    }
}
