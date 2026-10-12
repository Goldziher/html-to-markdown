//! Blockquote element handler for HTML to Markdown conversion.
//!
//! Handles `<blockquote>` elements including:
//! - Basic blockquote markdown output with `> ` prefix
//! - Nested blockquotes
//! - Citation URLs via `cite` attribute
//! - Visitor callback integration

use crate::converter::Context;
use crate::converter::inline::HandlerContext;
use crate::converter::main::walk_node;
use crate::converter::main_helpers::strip_trailing_backslash_breaks_from_fresh_buffer;

#[cfg(feature = "visitor")]
use crate::converter::utility::serialization::serialize_node_to_html;
#[cfg(feature = "visitor")]
use std::borrow::Cow;

/// Handle a `<blockquote>` element and convert to Markdown.
///
/// This handler processes blockquote elements including:
/// - Converting inline blockquotes by processing children as inline
/// - Handling nested blockquotes via `blockquote_depth` tracking
/// - Processing citation URLs from cite attribute
/// - Invoking visitor callbacks when the visitor feature is enabled
/// - Adding proper spacing and blockquote prefix formatting
pub fn handle_blockquote(tag: &tl::HTMLTag, mut handler: HandlerContext<'_>) {
    if handler.context.in_heading && !handler.context.in_table_cell {
        render_heading_quote(tag, &mut handler);
        return;
    }
    if handler.context.convert_as_inline {
        walk_children_to_output(tag, &mut handler);
        return;
    }

    // ~keep Relative citations must remain meaningful after the Markdown leaves its source page.
    let cite = crate::converter::utility::attributes::decoded_attribute(tag, "cite")
        .filter(|_| handler.options.include_blockquote_citations)
        .map(std::borrow::Cow::into_owned)
        .map(|value| {
            handler
                .context
                .resolve_url(&value, handler.node_handle, handler.parser, handler.dom_context)
                .unwrap_or(value)
        });
    let content = collect_quote_content(tag, &handler);
    let trimmed = container_content(tag, handler.parser, &content, handler.options.code_block_style);

    #[cfg(feature = "visitor")]
    if visit_blockquote(tag, trimmed, &mut handler) {
        return;
    }
    if handler.context.in_table_cell {
        render_table_cell_quote(trimmed, cite.as_deref(), &mut handler);
    } else {
        render_quote(trimmed, cite.as_deref(), &mut handler);
    }
}

/// The part of `content`, the Markdown of the children of the container `tag`, without the white
/// space around it.
///
/// ~keep Only a code block keeps its indentation at the start of the container. Text that starts
/// ~keep with spaces (strict white space mode) is not code and loses them.
///
/// ~keep `pub` (not `pub(crate)`, which clippy's `redundant_pub_crate` flags here since
/// ~keep `handlers::blockquote` is itself only `pub` within the crate): the description of a
/// ~keep definition list in `list/definition.rs` trims its content by the same rule.
pub fn container_content<'p, 'a, 'c>(
    tag: &'p tl::HTMLTag<'a>,
    parser: &'p tl::Parser<'a>,
    content: &'c str,
    style: crate::options::CodeBlockStyle,
) -> &'c str {
    let kept = crate::converter::code_scan::trimmed_quotes_content_range(content, style);
    let start = if starts_with_code_block(tag, parser) {
        kept.start
    } else {
        content.len() - content.trim_start().len()
    };
    &content[start..kept.end.max(start)]
}

/// Whether the first content of `tag` is a `pre` element, directly or as the first content of its
/// first element.
fn starts_with_code_block<'p, 'a>(tag: &'p tl::HTMLTag<'a>, parser: &'p tl::Parser<'a>) -> bool {
    let mut current = tag;
    loop {
        let first = current
            .children()
            .top()
            .iter()
            .find_map(|child| match child.get(parser) {
                Some(tl::Node::Tag(element)) => Some(Some(element)),
                Some(tl::Node::Raw(text)) if !text.as_utf8_str().trim().is_empty() => Some(None),
                _ => None,
            });
        match first.flatten() {
            Some(element) if element.name().as_bytes().eq_ignore_ascii_case(b"pre") => return true,
            Some(element) => current = element,
            None => return false,
        }
    }
}

fn walk_children_to_output(tag: &tl::HTMLTag<'_>, handler: &mut HandlerContext<'_>) {
    for child_handle in tag.children().top().iter() {
        walk_node(
            child_handle,
            handler.parser,
            handler.output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                handler.context,
                handler.depth + 1,
                handler.dom_context,
            ),
        );
    }
}

fn render_heading_quote(tag: &tl::HTMLTag<'_>, handler: &mut HandlerContext<'_>) {
    let mut content = String::new();
    walk_children(tag, &mut content, handler.context, handler);
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return;
    }
    if !handler.output.is_empty() && !handler.output.ends_with(char::is_whitespace) {
        handler.output.push(' ');
    }
    handler.output.push_str("> ");
    handler.output.push_str(trimmed);
    handler.output.push(' ');
}

fn collect_quote_content(tag: &tl::HTMLTag<'_>, handler: &HandlerContext<'_>) -> String {
    // ~keep Quote children start in a container of their own; surrounding list columns do not
    // alter nested lists, but an opening inline marker remains active on the first line (#654).
    let follows_markers = handler.output.is_empty() && handler.context.in_marker_text();
    let quote_context = Context {
        blockquote_depth: handler.context.blockquote_depth + 1,
        in_list_item: false,
        in_list: false,
        list_indent_columns: 0,
        real_item_columns: 0,
        inline_buffer_column: None,
        inline_depth: if follows_markers {
            handler.context.inline_depth
        } else {
            0
        },
        quote_starts_after_markers: follows_markers,
        item_lines: crate::converter::list::utils::ItemLineScan::new_item(),
        ..handler.context.clone()
    };
    let mut content = String::with_capacity(256);
    walk_children(tag, &mut content, &quote_context, handler);
    // ~keep No later dispatch can close a trailing `<br>` run at the end of this buffer (#464).
    strip_trailing_backslash_breaks_from_fresh_buffer(&mut content, handler.options.newline_style);
    content
}

fn walk_children(tag: &tl::HTMLTag<'_>, output: &mut String, context: &Context, handler: &HandlerContext<'_>) {
    for child_handle in tag.children().top().iter() {
        walk_node(
            child_handle,
            handler.parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                context,
                handler.depth + 1,
                handler.dom_context,
            ),
        );
    }
}

#[cfg(feature = "visitor")]
fn visit_blockquote(tag: &tl::HTMLTag<'_>, content: &str, handler: &mut HandlerContext<'_>) -> bool {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(visitor) = handler.context.visitor.as_ref() else {
        return false;
    };
    let node_id = handler.node_handle.get_inner();
    let node_context = NodeContext::with_lazy_attributes(
        NodeType::Blockquote,
        Cow::Borrowed("blockquote"),
        tag,
        handler.depth,
        handler.dom_context.get_sibling_index(node_id).unwrap_or(0),
        handler
            .dom_context
            .parent_tag_name(node_id, handler.parser)
            .map(Cow::Borrowed),
        false,
    );
    let result = visitor.lock().expect("visitor mutex poisoned").visit_blockquote(
        &node_context,
        content,
        handler.context.blockquote_depth,
    );
    match result {
        VisitResult::Continue => return false,
        VisitResult::Custom(custom) => handler.output.push_str(&custom),
        VisitResult::Skip => {}
        VisitResult::PreserveHtml => serialize_node_to_html(handler.node_handle, handler.parser, handler.output),
        VisitResult::Error(error) => {
            if handler.context.visitor_error.borrow().is_none() {
                *handler.context.visitor_error.borrow_mut() = Some(error);
            }
        }
    }
    true
}

fn render_table_cell_quote(content: &str, cite: Option<&str>, handler: &mut HandlerContext<'_>) {
    if content.is_empty() {
        return;
    }
    // ~keep Table cells shed the quote marker; code preserves physical breaks for later folding (#647).
    if handler.context.in_code {
        if !handler.output.is_empty() && !handler.output.ends_with('\n') {
            handler.output.push('\n');
        }
    } else {
        crate::converter::main_helpers::separate_block_in_cell(handler.output, handler.options.br_in_tables);
    }
    handler.output.push_str(content);
    if handler.context.in_code {
        handler.output.push('\n');
    }
    if let Some(url) = cite {
        crate::converter::main_helpers::separate_block_in_cell(handler.output, handler.options.br_in_tables);
        handler.output.push_str("— <");
        handler.output.push_str(url);
        handler.output.push('>');
    }
}

fn render_quote(content: &str, cite: Option<&str>, handler: &mut HandlerContext<'_>) {
    if content.is_empty() {
        return;
    }
    let list_indent = handler.context.in_list_item.then(|| {
        crate::converter::list::utils::continuation_indent_string(
            crate::converter::list::utils::block_columns(handler.context, handler.options),
            handler.options,
        )
    });
    let list_indent = list_indent.flatten();
    let continuation = list_indent.is_some()
        && !handler.output.is_empty()
        && !crate::converter::list::utils::line_is_bare_list_marker(handler.output);
    separate_before_quote(handler.output, handler.context);
    emit_quote_lines(
        content,
        list_indent.as_deref(),
        continuation,
        handler.options.code_block_style,
        handler.output,
    );
    if let Some(url) = cite {
        handler.output.push('\n');
        if let Some(indent) = list_indent.as_deref() {
            handler.output.push_str(indent);
        }
        handler.output.push_str("— <");
        handler.output.push_str(url);
        handler.output.push_str(">\n\n");
    }
    if !handler.context.in_list_item {
        while handler.output.ends_with('\n') {
            handler.output.truncate(handler.output.len() - 1);
        }
        handler.output.push_str("\n\n");
    }
}

fn separate_before_quote(output: &mut String, context: &Context) {
    // ~keep A quote that is the item's first content starts on the marker line (#617).
    if context.in_list_item && crate::converter::list::utils::trim_whitespace_after_bare_marker(output) {
        return;
    }
    if context.blockquote_depth > 0 && !context.in_list_item {
        if !output.is_empty() {
            while output.ends_with('\n') {
                output.truncate(output.len() - 1);
            }
            output.push_str("\n\n");
        }
        return;
    }
    if output.is_empty() {
        return;
    }
    if context.in_list_item {
        crate::converter::trim_trailing_whitespace(output);
    }
    if output.ends_with("\n\n") {
        output.truncate(output.len() - 1);
    } else if context.in_list_item {
        // ~keep CommonMark lets a quote interrupt list-item text without a blank line (examples 320–321).
        if !output.ends_with('\n') {
            output.push('\n');
        }
    } else if !output.ends_with('\n') {
        output.push_str("\n\n");
    } else if !output.ends_with("\n\n") {
        output.push('\n');
    }
}

fn emit_quote_lines(
    content: &str,
    indent: Option<&str>,
    continuation: bool,
    style: crate::options::CodeBlockStyle,
    output: &mut String,
) {
    let mut code = crate::converter::code_scan::CodeScan::of_quote_content(style, content);
    // ~keep Every physical quote line needs the list continuation indent to remain in the item (#13).
    for (index, line) in content.lines().enumerate() {
        if (index > 0 || continuation) && indent.is_some() {
            output.push_str(indent.unwrap_or_default());
        }
        // ~keep A blank line of the quote is the marker alone: a line of code keeps its line end,
        // ~keep so no later pass removes a space written here. A line of code that is only
        // ~keep white space keeps that white space. Only such a line asks whether it is code.
        let is_space = !line.is_empty() && line.trim().is_empty();
        let is_code = if is_space {
            code.is_code(line)
        } else {
            code.pass(line);
            false
        };
        if line.is_empty() || (is_space && !is_code) {
            output.push('>');
        } else {
            output.push_str("> ");
            output.push_str(line);
        }
        output.push('\n');
    }
}
