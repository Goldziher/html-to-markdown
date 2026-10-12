//! List item handling (li element).
//!
//! Processes list items with support for:
//! - Task list detection and rendering (checkboxes)
//! - Block-level children detection
//! - Proper bullet/number formatting
//! - Indentation and spacing

use crate::converter::list::ListContext;
use crate::converter::list::utils::add_list_leading_separator;
use crate::converter::main_helpers::effective_max_depth;
use crate::converter::main_helpers::strip_trailing_backslash_breaks;
use crate::converter::main_helpers::trim_trailing_whitespace;
use crate::converter::utility::content::normalized_tag_name;
use crate::converter::walk_node;
use crate::options::{ConversionOptions, NewlineStyle, OutputFormat};
#[cfg(feature = "visitor")]
use std::borrow::Cow;
use tl;

type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handle list item element (<li>).
///
/// Processes list item content with support for task lists (checkboxes),
/// proper indentation, and block-level element detection.
pub fn handle_li(
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    output: &mut String,
    context: ListContext<'_>,
) {
    let mut line_after_text = None;
    write_li(node_handle, tag, parser, output, context, &mut line_after_text);
    // ~keep Whether the item has content is known once it is written: a marker line without
    // ~keep content cannot interrupt the text before it either (issue #667).
    if let Some(line_start) = line_after_text {
        if line_start < output.len()
            && crate::converter::list::utils::marker_line_after_text_needs_blank_line(output, line_start)
        {
            output.insert(line_start, '\n');
        }
    }
}

#[derive(Clone, Copy)]
struct TaskInfo {
    is_task: bool,
    checked: bool,
    checkbox: Option<tl::NodeHandle>,
}

/// What the search for the checkbox of a task item met first in a node.
enum FirstContent {
    /// A checkbox, with its checked state.
    Checkbox(bool, tl::NodeHandle),
    /// Content that is not a checkbox: the item is not a task item.
    Other,
    /// Nothing a reader sees.
    Nothing,
}

/// Finds the checkbox a list item starts with.
///
/// ~keep A task item starts with its checkbox. A checkbox after other content of the item is a
/// ~keep control in the text, not the item's marker.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn find_checkbox<'a>(
    node_handle: &tl::NodeHandle,
    parser: &'a tl::Parser<'a>,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
) -> FirstContent {
    // ~keep This helper recurses over the li subtree independently of `walk_node`
    // ~keep (it runs before any handler dispatch), so it needs its own depth guard
    // ~keep instead of relying on the main walker's check.
    if depth >= effective_max_depth(options) {
        ctx.depth_limit_reached.set(true);
        return FirstContent::Other;
    }
    match node_handle.get(parser) {
        Some(tl::Node::Raw(raw)) if !raw.as_utf8_str().trim().is_empty() => FirstContent::Other,
        Some(tl::Node::Tag(node_tag)) => {
            match normalized_tag_name(node_tag.name().as_utf8_str()).as_ref() {
                "input" => {
                    return crate::converter::form::elements::checkbox_state(node_tag)
                        .map_or(FirstContent::Nothing, |checked| {
                            FirstContent::Checkbox(checked, *node_handle)
                        });
                }
                // ~keep A nested list's items own the checkboxes inside it (issue #604).
                "ul" | "ol" | "img" | "hr" => return FirstContent::Other,
                "script" | "style" | "template" | "noscript" => return FirstContent::Nothing,
                _ => {}
            }
            for child_handle in node_tag.children().top().iter() {
                let found = find_checkbox(child_handle, parser, options, ctx, depth + 1);
                if !matches!(found, FirstContent::Nothing) {
                    return found;
                }
            }
            FirstContent::Nothing
        }
        _ => FirstContent::Nothing,
    }
}

impl TaskInfo {
    fn new(node_handle: &tl::NodeHandle, parser: &tl::Parser, context: ListContext<'_>) -> Self {
        match find_checkbox(node_handle, parser, context.options, context.ctx, context.depth) {
            FirstContent::Checkbox(checked, checkbox) => Self {
                is_task: true,
                checked,
                checkbox: Some(checkbox),
            },
            FirstContent::Other | FirstContent::Nothing => Self {
                is_task: false,
                checked: false,
                checkbox: None,
            },
        }
    }
}

struct ItemMarker<'a> {
    task: TaskInfo,
    numbered: bool,
    ctx: &'a Context,
    options: &'a ConversionOptions,
}

impl<'a> ItemMarker<'a> {
    fn new(task: TaskInfo, ctx: &'a Context, options: &'a ConversionOptions) -> Self {
        // ~keep A task item in an ordered list keeps its number (issue #659). Djot has task
        // ~keep items only in bullet lists, so there it keeps the bullet.
        let numbered = ctx.in_ordered_list && !(task.is_task && options.output_format == OutputFormat::Djot);
        Self {
            task,
            numbered,
            ctx,
            options,
        }
    }

    fn list_marker(&self) -> String {
        // ~keep An ordered list right after an ordered list writes `)` (issue #666).
        if self.numbered {
            format!(
                "{}{} ",
                self.ctx.list_counter,
                self.ctx.ordered_delimiter.unwrap_or('.')
            )
        } else if self.task.is_task {
            String::from("- ")
        } else {
            format!("{} ", unordered_bullet(self.ctx, self.options))
        }
    }

    fn full_marker(&self) -> String {
        if self.task.is_task {
            format!(
                "{}{} ",
                self.list_marker(),
                if self.task.checked { "[x]" } else { "[ ]" }
            )
        } else {
            self.list_marker()
        }
    }

    fn width(&self) -> usize {
        if self.numbered {
            self.options
                .list_indent_width
                .max(format!("{}. ", self.ctx.list_counter).chars().count())
        } else {
            self.options.list_indent_width.max(2)
        }
    }
}

fn separate_marker_from_text(output: &mut String, marker: &ItemMarker<'_>, line_after_text: &mut Option<usize>) {
    let line_start = output.rfind('\n').map_or(0, |position| position + 1);
    if marker.ctx.in_table_cell
        || output[line_start..].trim().is_empty()
        || crate::converter::list::utils::line_is_bare_list_marker(output)
    {
        return;
    }
    output.push('\n');
    if crate::converter::utility::escaping::line_opens_block(&format!("{}x", marker.full_marker())) {
        if !marker.ctx.in_marker_text() {
            *line_after_text = Some(output.len());
        }
    } else {
        output.push('\n');
    }
}

fn marker_column(output: &mut String, ctx: &Context, options: &ConversionOptions) -> (Option<usize>, usize) {
    let marker_line_start = (!output.is_empty() && output.ends_with('\n')).then_some(output.len());
    let marker_follows_markers = output.is_empty() && ctx.in_marker_text();
    let buffer_column = ctx
        .inline_buffer_column
        .filter(|_| output.is_empty() && !marker_follows_markers);
    let column = if let Some(column) = buffer_column {
        column
    } else if ctx.list_depth > 0 && (output.is_empty() || output.ends_with('\n')) {
        let indent = crate::converter::list::utils::continuation_indent_string(
            crate::converter::list::utils::block_columns(ctx, options),
            options,
        )
        .unwrap_or_default();
        output.push_str(&indent);
        if marker_follows_markers {
            ctx.list_indent_columns
        } else {
            crate::converter::utility::escaping::leading_indent(&indent).1
        }
    } else {
        ctx.list_indent_columns
    };
    (marker_line_start, column)
}

fn has_block_children(tag: &tl::HTMLTag, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    tag.children().top().iter().any(|child_handle| {
        if let Some(info) = dom_ctx.tag_info(child_handle.get_inner(), parser) {
            matches!(
                info.name.as_str(),
                "p" | "div" | "blockquote" | "pre" | "table" | "hr" | "dl"
            )
        } else if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
            matches!(
                normalized_tag_name(child_tag.name().as_utf8_str()).as_ref(),
                "p" | "div" | "blockquote" | "pre" | "table" | "hr" | "dl"
            )
        } else {
            false
        }
    })
}

fn item_context(
    output: &String,
    marker_line_start: Option<usize>,
    marker_column: usize,
    marker: &ItemMarker<'_>,
) -> Context {
    let list_item_open = !marker.ctx.in_marker_text();
    let item_is_real = list_item_open || {
        let full_marker = marker.full_marker();
        !(marker.ctx.escapes_hyphens && full_marker.starts_with('-'))
            && crate::converter::list::utils::marker_starts_item(
                output,
                marker_line_start,
                &full_marker,
                marker.ctx.real_item_columns,
                (
                    &marker.ctx.previous_marker,
                    std::ptr::from_ref::<String>(output) as usize,
                ),
                marker.options,
            )
    };
    let marker_width = marker.width();
    Context {
        in_list_item: true,
        list_item_open,
        list_depth: marker.ctx.list_depth + 1,
        list_indent_columns: marker_column + marker_width,
        real_item_columns: if item_is_real {
            marker_column + marker_width
        } else {
            marker.ctx.real_item_columns
        },
        first_writer: marker.task.is_task.then(FirstWriter::default),
        task_item_scope: marker
            .task
            .is_task
            .then_some((marker.ctx.list_depth + 1, marker.ctx.blockquote_depth))
            .or(marker.ctx.task_item_scope),
        ..marker.ctx.clone()
    }
}

fn is_checkbox_node(node_handle: &tl::NodeHandle, checkbox: Option<tl::NodeHandle>) -> bool {
    checkbox.is_some_and(|checkbox| node_handle == &checkbox)
}

fn contains_checkbox<'a>(
    node_handle: &tl::NodeHandle,
    parser: &'a tl::Parser<'a>,
    checkbox: Option<tl::NodeHandle>,
    context: ListContext<'_>,
) -> bool {
    if context.depth >= effective_max_depth(context.options) {
        context.ctx.depth_limit_reached.set(true);
        return false;
    }
    if is_checkbox_node(node_handle, checkbox) {
        return true;
    }
    let Some(tl::Node::Tag(node_tag)) = node_handle.get(parser) else {
        return false;
    };
    node_tag.children().top().iter().any(|child_handle| {
        contains_checkbox(
            child_handle,
            parser,
            checkbox,
            ListContext {
                depth: context.depth + 1,
                ..context
            },
        )
    })
}

fn render_li_content<'a>(
    node_handle: &tl::NodeHandle,
    parser: &'a tl::Parser<'a>,
    output: &mut String,
    checkbox: Option<tl::NodeHandle>,
    context: ListContext<'_>,
) {
    // ~keep Independent recursion from `walk_node` while probing for the nested checkbox,
    // ~keep so it needs its own guard rather than relying on the guard inside `walk_node`.
    if context.depth >= effective_max_depth(context.options) {
        context.ctx.depth_limit_reached.set(true);
        return;
    }
    if is_checkbox_node(node_handle, checkbox) {
        return;
    }
    if contains_checkbox(node_handle, parser, checkbox, context) {
        if let Some(tl::Node::Tag(node_tag)) = node_handle.get(parser) {
            for child_handle in node_tag.children().top().iter() {
                render_li_content(
                    child_handle,
                    parser,
                    output,
                    checkbox,
                    ListContext {
                        depth: context.depth + 1,
                        ..context
                    },
                );
            }
        }
    } else {
        walk_node(
            node_handle,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                context.options,
                context.ctx,
                context.depth,
                context.dom_ctx,
            ),
        );
    }
}

fn collect_task_text(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    marker: &ItemMarker<'_>,
    item_ctx: &Context,
    context: ListContext<'_>,
) -> String {
    let mut task_text = String::new();
    for child_handle in tag.children().top().iter() {
        render_li_content(
            child_handle,
            parser,
            &mut task_text,
            marker.task.checkbox,
            ListContext {
                ctx: item_ctx,
                depth: context.depth + 1,
                ..context
            },
        );
    }
    task_text
}

fn first_task_block<'a>(task_text: &'a str, parser: &tl::Parser, item_ctx: &Context) -> Option<&'a str> {
    match item_ctx
        .first_writer
        .as_ref()
        .and_then(|first_writer| first_writer.content(parser, item_ctx))
    {
        Some(TaskFirstContent::Block) => Some(task_text.trim()),
        Some(TaskFirstContent::CodeBlock) => {
            // ~keep An indented code block's first line keeps its indent (issue #634).
            let code_content = task_text.trim_end();
            let first = code_content.len() - code_content.trim_start().len();
            Some(&code_content[code_content[..first].rfind('\n').map_or(0, |position| position + 1)..])
        }
        _ => None,
    }
}

fn write_task_content(
    output: &mut String,
    task_text: &str,
    first_block: Option<&str>,
    item_ctx: &Context,
    context: ListContext<'_>,
) {
    // ~keep GFM reads a checkbox only in a paragraph with content after the marker, so
    // ~keep cmark-gfm prints a bare `[ ]` line as text. A space written as a character
    // ~keep reference is that content and renders as a space; a trailing space is not.
    const CHECKBOX_CONTENT: &str = " &#32;";
    let item_starts = item_ctx.list_item_open && !context.ctx.in_table_cell && !context.ctx.convert_as_inline;
    let writes_checkbox_content =
        item_starts && context.options.output_format == OutputFormat::Markdown && !context.ctx.in_code;
    let indent =
        crate::converter::list::utils::continuation_indent_string(item_ctx.list_indent_columns, context.options);
    match (indent, first_block) {
        (Some(indent), Some(block)) if item_starts => {
            // ~keep A line that cannot interrupt the checkbox paragraph needs a blank line
            // ~keep before it, and a `---` line under it would make it a heading (issue #634).
            let first_line = block.lines().next().unwrap_or_default();
            if writes_checkbox_content {
                output.push_str(CHECKBOX_CONTENT);
            }
            let separator = if crate::converter::utility::escaping::is_heading_underline(first_line)
                || !crate::converter::utility::escaping::line_opens_block(first_line)
            {
                "\n\n"
            } else {
                "\n"
            };
            output.push_str(separator);
            output.push_str(&indent);
            output.push_str(block);
        }
        _ if writes_checkbox_content && task_text.trim().is_empty() => output.push_str(CHECKBOX_CONTENT),
        _ => {
            output.push(' ');
            output.push_str(task_text.trim());
        }
    }
}

fn render_task_item(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    output: &mut String,
    marker: &ItemMarker<'_>,
    item_ctx: &Context,
    context: ListContext<'_>,
) {
    output.push_str(&marker.list_marker());
    output.push_str(if marker.task.checked { "[x]" } else { "[ ]" });
    let task_text = collect_task_text(tag, parser, marker, item_ctx, context);
    let first_block = first_task_block(&task_text, parser, item_ctx);
    write_task_content(output, &task_text, first_block, item_ctx, context);
}

struct ItemRenderContext<'render, 'context> {
    marker: &'render ItemMarker<'context>,
    item_ctx: &'render Context,
    list: ListContext<'context>,
}

fn write_regular_children(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    output: &mut String,
    render: &ItemRenderContext<'_, '_>,
) -> (usize, usize) {
    let item_start = output.len();
    let mut text_end = item_start;
    for child_handle in tag.children().top().iter() {
        let is_nested_list = child_handle.get(parser).is_some_and(|node| {
            let tl::Node::Tag(child_tag) = node else {
                return false;
            };
            matches!(
                normalized_tag_name(child_tag.name().as_utf8_str()).as_ref(),
                "ul" | "ol"
            )
        });
        walk_node(
            child_handle,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                render.list.options,
                render.item_ctx,
                render.list.depth + 1,
                render.list.dom_ctx,
            ),
        );
        if !is_nested_list {
            text_end = output.len();
        }
    }
    (item_start, text_end)
}

#[cfg(feature = "visitor")]
fn visit_regular_item(
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    output: &mut String,
    render: &ItemRenderContext<'_, '_>,
) -> bool {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(visitor_handle) = render.list.ctx.visitor.as_ref() else {
        return false;
    };
    let parent_tag = render
        .list
        .dom_ctx
        .parent_of(node_handle.get_inner())
        .and_then(|parent_id| {
            render
                .list
                .dom_ctx
                .tag_name_for(render.list.dom_ctx.node_handle(parent_id).copied()?, parser)
        })
        .map(std::borrow::Cow::into_owned);
    let index = render.list.dom_ctx.sibling_index(node_handle.get_inner()).unwrap_or(0);
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::ListItem,
        Cow::Borrowed("li"),
        tag,
        render.list.depth,
        index,
        parent_tag.map(Cow::Owned),
        false,
    );
    let line_start = output.rfind('\n').map_or(0, |position| position + 1);
    let last_line = &output[line_start..];
    let marker_text = render.marker.list_marker().trim_end().to_string();
    let text_start = last_line
        .find(&marker_text)
        .map_or(0, |position| position + marker_text.len());
    let visit_result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        visitor.visit_list_item(
            &node_ctx,
            render.list.ctx.in_ordered_list,
            &marker_text,
            last_line[text_start..].trim(),
        )
    };
    match visit_result {
        VisitResult::Continue => false,
        VisitResult::Custom(custom) => {
            crate::converter::structure_capture::replace_element(render.list.ctx, Some(&custom));
            replace_regular_item_output(output, line_start, &custom, render.list.ctx.in_table_cell);
            true
        }
        VisitResult::Skip => {
            crate::converter::structure_capture::replace_element(render.list.ctx, None);
            output.truncate(line_start);
            true
        }
        VisitResult::PreserveHtml => {
            let html = crate::converter::serialize_node(node_handle, parser);
            crate::converter::structure_capture::replace_element(render.list.ctx, Some(&html));
            replace_regular_item_output(output, line_start, &html, render.list.ctx.in_table_cell);
            true
        }
        VisitResult::Error(error) => {
            crate::converter::structure_capture::replace_element(render.list.ctx, None);
            if render.list.ctx.visitor_error.borrow().is_none() {
                *render.list.ctx.visitor_error.borrow_mut() = Some(error);
            }
            true
        }
    }
}

#[cfg(feature = "visitor")]
fn replace_regular_item_output(output: &mut String, line_start: usize, replacement: &str, in_table_cell: bool) {
    output.truncate(line_start);
    output.push_str(replacement);
    if !in_table_cell && !output.ends_with('\n') {
        output.push('\n');
    }
}

fn render_regular_item(
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    output: &mut String,
    render: &ItemRenderContext<'_, '_>,
) -> bool {
    #[cfg(not(feature = "visitor"))]
    let _ = node_handle;
    if render.list.ctx.in_table_cell {
        add_list_leading_separator(output, render.list.ctx, render.list.options);
    } else {
        output.push_str(&render.marker.list_marker());
    }
    let (item_start, _) = write_regular_children(tag, parser, output, render);
    trim_trailing_whitespace(output);
    if render.list.options.newline_style == NewlineStyle::Backslash {
        strip_trailing_backslash_breaks(output, item_start);
    }
    #[cfg(feature = "visitor")]
    if visit_regular_item(node_handle, tag, parser, output, render) {
        return true;
    }
    false
}

fn finish_list_item(output: &mut String, ctx: &Context, has_block_children: bool) {
    if ctx.in_table_cell {
        return;
    }
    if has_block_children || ctx.loose_list || ctx.prev_item_had_blocks {
        if !output.ends_with("\n\n") {
            if output.ends_with('\n') {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }
    } else if !output.ends_with('\n') {
        output.push('\n');
    }
}

/// Write the list item, and set `line_after_text` to the start of its marker line when that line
/// follows text inside the list and the check with an item that has content wrote no blank line.
fn write_li(
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    output: &mut String,
    context: ListContext<'_>,
    line_after_text: &mut Option<usize>,
) {
    let ListContext {
        options,
        ctx,
        depth: _,
        dom_ctx,
    } = context;
    let task = TaskInfo::new(node_handle, parser, context);
    let marker = ItemMarker::new(task, ctx, options);
    separate_marker_from_text(output, &marker, line_after_text);
    let (marker_line_start, marker_column) = marker_column(output, ctx, options);
    let li_ctx = item_context(output, marker_line_start, marker_column, &marker);
    let has_block_children = has_block_children(tag, parser, dom_ctx);

    if marker.task.is_task {
        render_task_item(tag, parser, output, &marker, &li_ctx, context);
    } else {
        let render = ItemRenderContext {
            marker: &marker,
            item_ctx: &li_ctx,
            list: context,
        };
        if render_regular_item(node_handle, tag, parser, output, &render) {
            return;
        }
    }
    finish_list_item(output, ctx, has_block_children);
}

/// What a task item renders first after its checkbox.
enum TaskFirstContent {
    /// Paragraph text on the checkbox line.
    Text,
    /// A block that starts its own line: a quote, a list, a heading, a rule or a table.
    Block,
    /// A code block, whose first line can be indented code.
    CodeBlock,
}

/// The element of a task item whose render writes the item's first content.
///
/// ~keep Every node the item renders reports what it wrote, until one wrote content. An element
/// ~keep without a block opener that starts with the first line of the child that wrote writes
/// ~keep nothing before that child, so the writer is the outermost element that wrote and is not
/// ~keep such a container. A node that writes nothing (an empty element, a line break, a dropped
/// ~keep element, anything past `max_depth`) is never the writer. A node that drops the output of
/// ~keep its children drops their writer too.
#[derive(Clone, Default)]
pub struct FirstWriter(std::rc::Rc<std::cell::RefCell<FirstWriterState>>);

#[derive(Default)]
struct FirstWriterState {
    node: Option<tl::NodeHandle>,
    /// The first line that `node` wrote, without its indentation.
    first_line: String,
}

impl FirstWriter {
    /// Whether no node has written yet, so the next node's render must report.
    pub fn is_open(&self) -> bool {
        self.0.borrow().node.is_none()
    }

    /// Whether the first node that wrote is a block quote.
    pub(crate) fn starts_with_blockquote(&self, parser: &tl::Parser) -> bool {
        let state = self.0.borrow();
        state.node.is_some_and(|node| {
            let Some(tl::Node::Tag(tag)) = node.get(parser) else {
                return false;
            };
            normalized_tag_name(tag.name().as_utf8_str()) == "blockquote"
        })
    }

    /// Record the render of `node`, which started while no node had written and wrote `written`.
    pub fn record(&self, node: tl::NodeHandle, parser: &tl::Parser, written: Option<&str>) {
        let child_starts_blockquote = self.starts_with_blockquote(parser);
        let mut state = self.0.borrow_mut();
        let Some(text) = written.map(str::trim_start).filter(|text| !text.is_empty()) else {
            *state = FirstWriterState::default();
            return;
        };
        let first_line = text.split('\n').next().unwrap_or_default();
        // ~keep An inline container may reposition delimiters around a quote's opener, but
        // ~keep the quote still decides whether the task starts with a block (#643).
        let container = state.node.is_some()
            && is_container(node, parser)
            && (first_line.starts_with(state.first_line.as_str()) || child_starts_blockquote);
        if !container {
            state.node = Some(node);
            state.first_line = first_line.to_string();
        }
    }

    /// What the first writer wrote, or `None` when no node wrote.
    ///
    /// ~keep Text is paragraph text on the checkbox line, also when it reads like an opener. An
    /// ~keep element that `preserve_tags` writes as HTML starts a block when its HTML does.
    fn content(&self, parser: &tl::Parser, ctx: &Context) -> Option<TaskFirstContent> {
        let state = self.0.borrow();
        let Some(tl::Node::Tag(tag)) = state.node?.get(parser) else {
            return Some(TaskFirstContent::Text);
        };
        let name = normalized_tag_name(tag.name().as_utf8_str());
        Some(match block_content(&name) {
            Some(block) => block,
            None if ctx.preserve_tags.contains(name.as_ref())
                && crate::converter::utility::escaping::opens_block(&state.first_line) =>
            {
                TaskFirstContent::Block
            }
            None => TaskFirstContent::Text,
        })
    }
}

/// Whether `node` is an element without a block opener of its own, like a `<div>` or a `<span>`.
fn is_container(node: tl::NodeHandle, parser: &tl::Parser) -> bool {
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return false;
    };
    block_content(&normalized_tag_name(tag.name().as_utf8_str())).is_none()
}

/// The first content that an element named `name` writes when it is a block with an opener.
fn block_content(name: &str) -> Option<TaskFirstContent> {
    match name {
        "pre" => Some(TaskFirstContent::CodeBlock),
        "blockquote" | "ul" | "ol" | "hr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "table" => {
            Some(TaskFirstContent::Block)
        }
        _ => None,
    }
}

/// The bullet of an unordered list item at `ctx.ul_depth`: the list's bullets cycle by depth.
fn unordered_bullet(ctx: &Context, options: &ConversionOptions) -> char {
    let bullets: Vec<char> = options.bullets.chars().collect();
    let bullet_index = if ctx.ul_depth > 0 { ctx.ul_depth - 1 } else { 0 };
    if bullets.is_empty() {
        '*'
    } else {
        bullets[bullet_index % bullets.len()]
    }
}
