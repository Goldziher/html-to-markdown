use crate::converter::block::container::HandlerContext;

// ~keep A nested GFM table cannot fit inside a cell; retain row content without generating
// ~keep structural pipes or delimiter dashes (#760). Single-cell deferred tables bypass this.
pub(super) fn render(
    table: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let mut pending: Vec<_> = table
        .children()
        .top()
        .iter()
        .map(|handle| (*handle, handler.depth + 1))
        .collect();
    pending.reverse();
    while let Some((child, depth)) = pending.pop() {
        let Some(tl::Node::Tag(tag)) = child.get(parser) else {
            continue;
        };
        if depth >= crate::converter::main_helpers::effective_max_depth(handler.options) {
            handler.ctx.depth_limit_reached.set(true);
            continue;
        }
        match tag.name().as_utf8_str().to_ascii_lowercase().as_str() {
            "thead" | "tbody" | "tfoot" => {
                let mut children: Vec<_> = tag.children().top().iter().map(|child| (*child, depth + 1)).collect();
                children.reverse();
                pending.extend(children);
            }
            "tr" | "row" => render_row(&child, parser, output, HandlerContext { depth, ..handler }),
            "caption" => append_content(
                output,
                &super::cell::cell_text_content(&child, parser, handler.options, handler.ctx, handler.dom_ctx, depth),
                handler,
            ),
            _ => {}
        }
    }
}

fn render_row(row: &tl::NodeHandle, parser: &tl::Parser<'_>, output: &mut String, handler: HandlerContext<'_>) {
    let mut cells = Vec::new();
    super::cell::collect_table_cells(row, parser, handler.dom_ctx, &mut cells);
    let row = cells
        .iter()
        .map(|cell| {
            super::cell::cell_text_content(
                cell,
                parser,
                handler.options,
                handler.ctx,
                handler.dom_ctx,
                handler.depth + 1,
            )
        })
        .filter(|cell| !cell.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    append_content(output, &row, handler);
}

fn append_content(output: &mut String, content: &str, handler: HandlerContext<'_>) {
    if content.is_empty() {
        return;
    }
    if !output.is_empty() {
        crate::converter::emit_table_cell_break(output, handler.options.br_in_tables);
    }
    output.push_str(content);
}
