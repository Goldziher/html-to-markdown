use super::{
    Context, ItemMarker, ListContext, NewlineStyle, TaskInfo, add_list_leading_separator, finish_list_item,
    first_task_block, has_block_children, item_context, marker_column, separate_marker_from_text,
    strip_trailing_backslash_breaks, trim_trailing_whitespace, write_task_content,
};
#[cfg(feature = "visitor")]
use super::{ItemRenderContext, visit_regular_item};
pub fn open_regular(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: ListContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<crate::converter::walk::Open> {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };
    let task = TaskInfo::new(node_handle, parser, context);
    if task.is_task {
        return open_task(node_handle, parser, output, context, task, children);
    }
    let marker = ItemMarker::new(task, context.ctx, context.options);
    let mut line_after_text = None;
    separate_marker_from_text(output, &marker, &mut line_after_text);
    let (marker_line_start, column) = marker_column(output, context.ctx, context.options);
    let item_ctx = item_context(output, marker_line_start, column, &marker);
    let has_blocks = has_block_children(tag, parser, context.dom_ctx);
    if context.ctx.in_table_cell {
        add_list_leading_separator(output, context.ctx, context.options);
    } else if !context.ctx.in_heading {
        output.push_str(&marker.list_marker());
    }
    let item_start = output.len();
    let item_ctx = std::rc::Rc::new(item_ctx.with_inline_break_owner(output));
    children.extend(tag.children().top().iter().copied());
    Some(crate::converter::walk::Open {
        scope: Some(std::rc::Rc::clone(&item_ctx)),
        close: crate::converter::walk::Close::Hooked(Box::new(RegularItemHook {
            task,
            item_ctx,
            has_blocks,
            item_start,
            line_after_text,
        })),
    })
}

struct RegularItemHook {
    task: TaskInfo,
    item_ctx: std::rc::Rc<Context>,
    has_blocks: bool,
    item_start: usize,
    line_after_text: Option<usize>,
}

impl crate::converter::walk::ChildHook for RegularItemHook {
    fn before_child(
        &mut self,
        _index: usize,
        _site: &mut crate::converter::walk::HookSite<'_>,
    ) -> crate::converter::walk::Child {
        crate::converter::walk::Child::Walk {
            depth: None,
            buffer: None,
            scope: Some(std::rc::Rc::clone(&self.item_ctx)),
        }
    }
    fn after_child(&mut self, _index: usize, _own: Option<String>, _site: &mut crate::converter::walk::HookSite<'_>) {}
    fn close(self: Box<Self>, site: &mut crate::converter::walk::HookSite<'_>) {
        let Some(tl::Node::Tag(tag)) = site.node_handle.get(site.parser) else {
            return;
        };
        let marker = ItemMarker::new(self.task, site.context, site.options);
        trim_trailing_whitespace(site.output);
        if site.options.newline_style == NewlineStyle::Backslash {
            strip_trailing_backslash_breaks(site.output, self.item_start);
        }
        #[cfg(feature = "visitor")]
        if visit_regular_item(
            site.node_handle,
            tag,
            site.parser,
            site.output,
            &ItemRenderContext {
                marker: &marker,
                item_ctx: &self.item_ctx,
                list: ListContext {
                    options: site.options,
                    ctx: site.context,
                    depth: site.depth,
                    dom_ctx: site.dom_context,
                },
            },
        ) {
            return;
        }
        #[cfg(not(feature = "visitor"))]
        let _ = (tag, marker);
        finish_list_item(site.output, site.context, self.has_blocks);
        if let Some(line_start) = self.line_after_text {
            if line_start < site.output.len()
                && crate::converter::list::utils::marker_line_after_text_needs_blank_line(site.output, line_start)
            {
                site.output.insert(line_start, '\n');
            }
        }
    }
}

pub struct TaskItemEnd {
    item_ctx: std::rc::Rc<Context>,
    has_blocks: bool,
    line_after_text: Option<usize>,
}

fn open_task(
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: ListContext<'_>,
    task: TaskInfo,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<crate::converter::walk::Open> {
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return None;
    };
    let marker = ItemMarker::new(task, context.ctx, context.options);
    let mut line_after_text = None;
    separate_marker_from_text(output, &marker, &mut line_after_text);
    let (marker_line_start, column) = marker_column(output, context.ctx, context.options);
    let mut item_ctx = item_context(output, marker_line_start, column, &marker);
    let mut path = std::collections::HashSet::new();
    let mut current = task
        .checkbox
        .and_then(|checkbox| context.dom_ctx.parent_of(checkbox.get_inner()));
    while let Some(id) = current.filter(|id| *id != node.get_inner()) {
        path.insert(id);
        current = context.dom_ctx.parent_of(id);
    }
    item_ctx.task_checkbox = task.checkbox;
    item_ctx.task_checkbox_ancestors = std::rc::Rc::new(path);
    let item_ctx = std::rc::Rc::new(item_ctx);
    let has_blocks = has_block_children(tag, parser, context.dom_ctx);
    if !context.ctx.in_heading {
        output.push_str(&marker.list_marker());
    }
    output.push_str(if task.checked { "[x]" } else { "[ ]" });
    children.extend(tag.children().top().iter().copied());
    Some(crate::converter::walk::Open {
        scope: Some(std::rc::Rc::clone(&item_ctx)),
        close: crate::converter::walk::Close::Capture(crate::converter::walk::CaptureEnd::TaskItem(TaskItemEnd {
            item_ctx,
            has_blocks,
            line_after_text,
        })),
    })
}

pub fn close_task(end: &TaskItemEnd, content: &str, handler: crate::converter::inline::HandlerContext<'_>) {
    let list = ListContext {
        options: handler.options,
        ctx: handler.context,
        depth: handler.depth,
        dom_ctx: handler.dom_context,
    };
    let first_block = first_task_block(content, handler.parser, &end.item_ctx);
    write_task_content(handler.output, content, first_block, &end.item_ctx, list);
    finish_list_item(handler.output, handler.context, end.has_blocks);
    if let Some(line_start) = end.line_after_text {
        if line_start < handler.output.len()
            && crate::converter::list::utils::marker_line_after_text_needs_blank_line(handler.output, line_start)
        {
            handler.output.insert(line_start, '\n');
        }
    }
}
