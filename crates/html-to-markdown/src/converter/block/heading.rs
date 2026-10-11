//! Handler for heading elements (h1-h6).
//!
//! Converts HTML heading tags to Markdown heading syntax with support for:
//! - Multiple heading styles (ATX, underlined, closed ATX)
//! - Inline content processing with proper text normalization
//! - Metadata collection (headers, IDs)
//! - Visitor callbacks for custom heading processing

use crate::converter::block::container::HandlerContext;
use crate::options::{ConversionOptions, HeadingStyle, OutputFormat};
use std::borrow::Cow;
use tl::{NodeHandle, Parser};

type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handle heading elements (h1, h2, h3, h4, h5, h6).
///
/// Extracts the heading level from the tag name, processes inline content,
/// normalizes text, and outputs formatted heading with proper spacing.
///
/// # Note
/// This function references `walk_node` from converter.rs which must be
/// accessible (pub(crate)) for this module to work correctly.
pub fn handle(
    tag_name: &str,
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let options = handler.options;
    let ctx = handler.ctx;

    let level = tag_name.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1) as usize;
    separate_heading(output, options, ctx, level);
    let Some(normalized) = heading_text(tag_name, node_handle, parser, handler) else {
        return;
    };

    #[cfg(feature = "visitor")]
    let heading_output = visitor_heading_output(node_handle, parser, tag_name, level, &normalized, handler);

    #[cfg(not(feature = "visitor"))]
    let heading_output = {
        let mut buf = String::new();
        push_heading(&mut buf, ctx, options, level, &normalized);
        Some(buf)
    };

    if let Some(heading_output) = heading_output {
        append_heading(output, &heading_output, options, ctx, level);
    }
    #[cfg(feature = "metadata")]
    record_heading(node_handle, parser, &normalized, level, handler);
}

fn separate_heading(output: &mut String, options: &ConversionOptions, ctx: &Context, level: usize) {
    let needs_leading_sep = !ctx.in_table_cell
        && !ctx.in_list_item
        && !ctx.convert_as_inline
        && (ctx.blockquote_depth == 0 || continues_a_line_in_quote(output, options, level))
        && !output.is_empty()
        && !output.ends_with("\n\n");

    if needs_leading_sep {
        crate::converter::trim_trailing_whitespace(output);
        output.push_str(if ctx.blockquote_depth > 0 && output.ends_with('\n') {
            "\n"
        } else {
            "\n\n"
        });
    }
}

fn heading_text(
    tag_name: &str,
    node_handle: &NodeHandle,
    parser: &Parser,
    handler: HandlerContext<'_>,
) -> Option<String> {
    let mut text = String::new();
    let heading_ctx = Context {
        in_heading: true,
        convert_as_inline: true,
        heading_allow_inline_images: heading_allows_inline_images(tag_name, &handler.ctx.keep_inline_images_in),
        ..handler.ctx.clone()
    };

    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };
    for child_handle in tag.children().top().iter() {
        crate::converter::walk_node(
            child_handle,
            parser,
            &mut text,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                &heading_ctx,
                handler.depth + 1,
                handler.dom_ctx,
            ),
        );
    }
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| normalize_heading_text(trimmed).into_owned())
}

fn append_heading(output: &mut String, heading_text: &str, options: &ConversionOptions, ctx: &Context, level: usize) {
    // ~keep A setext heading's text line after a line of the item would continue that
    // ~keep line's paragraph, so it starts after a blank line (issue #635).
    let line_start = output.rfind('\n').map_or(0, |pos| pos + 1);
    if ctx.in_list_item
        && options.heading_style == HeadingStyle::Underlined
        && level <= 2
        && line_start > 0
        && output[line_start..].trim().is_empty()
        && !output[..line_start - 1]
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        let indent = output.split_off(line_start);
        output.push('\n');
        output.push_str(&indent);
    }
    // ~keep In a cell the heading's text is a block of the cell's one line (issue #645).
    if ctx.in_table_cell && !ctx.convert_as_inline && !ctx.in_code && !heading_text.is_empty() {
        crate::converter::main_helpers::separate_block_in_cell(output, options.br_in_tables);
    }
    output.push_str(heading_text);
}

#[cfg(feature = "metadata")]
fn record_heading(
    node_handle: &NodeHandle,
    parser: &Parser,
    normalized: &str,
    level: usize,
    handler: HandlerContext<'_>,
) {
    let id = node_handle
        .get(parser)
        .and_then(|node| match node {
            tl::Node::Tag(tag) => tag.attributes().get("id").flatten(),
            _ => None,
        })
        .map(|value| value.as_utf8_str().to_string());

    if handler.ctx.metadata_wants.headers {
        if let Some(ref collector) = handler.ctx.metadata_collector {
            collector
                .borrow_mut()
                .add_header(level as u8, normalized.to_string(), id, handler.depth, 0);
        }
    }
}

/// Whether a heading in a quote's buffer would join the line before it: text on the same line,
/// or, for an underlined heading, a line of text above it (issue #640).
///
/// ~keep In a quote, a heading after a block keeps the compact style of the quote (no blank
/// ~keep line); only a line the heading would continue gets one. The underline of a heading
/// ~keep before it ends that heading, so it is no text.
fn continues_a_line_in_quote(output: &str, options: &ConversionOptions, level: usize) -> bool {
    let Some(before_line_end) = output.strip_suffix('\n') else {
        return true;
    };
    let line = before_line_end.rsplit('\n').next().unwrap_or_default().trim();
    options.heading_style == HeadingStyle::Underlined
        && level <= 2
        && !line.is_empty()
        && !crate::converter::utility::escaping::is_heading_underline(line)
}

/// Determine if a heading element should allow inline images.
pub fn heading_allows_inline_images(
    tag_name: &str,
    keep_inline_images_in: &std::rc::Rc<std::collections::HashSet<String>>,
) -> bool {
    keep_inline_images_in.contains(tag_name)
}

/// Normalize heading text by replacing newlines with spaces.
fn normalize_heading_text(text: &str) -> Cow<'_, str> {
    if !text.contains('\n') && !text.contains('\r') {
        return Cow::Borrowed(text);
    }

    let mut normalized = String::with_capacity(text.len());
    let mut pending_space = false;

    for ch in text.chars() {
        match ch {
            '\n' | '\r' => {
                if !normalized.is_empty() {
                    pending_space = true;
                }
            }
            ' ' | '\t' if pending_space => {}
            _ => {
                if pending_space {
                    if !normalized.ends_with(' ') {
                        normalized.push(' ');
                    }
                    pending_space = false;
                }
                normalized.push(ch);
            }
        }
    }

    Cow::Owned(normalized)
}

/// Format heading output with appropriate markdown syntax.
pub fn push_heading(output: &mut String, ctx: &Context, options: &ConversionOptions, level: usize, text: &str) {
    if text.is_empty() {
        return;
    }
    if write_inline_heading(output, ctx, text) {
        return;
    }
    prepare_block_heading(output, ctx, options);
    render_heading_style(output, ctx, options, level, text);
    output.push_str(if ctx.in_list_item || ctx.blockquote_depth > 0 {
        "\n"
    } else {
        "\n\n"
    });
}

fn write_inline_heading(output: &mut String, ctx: &Context, text: &str) -> bool {
    if ctx.convert_as_inline {
        output.push_str(text);
        return true;
    }
    if !ctx.in_table_cell {
        return false;
    }
    let is_table_continuation =
        !output.is_empty() && !output.ends_with('|') && !output.ends_with(' ') && !output.ends_with("<br>");
    if is_table_continuation {
        output.push_str("<br>");
    }
    output.push_str(text);
    true
}

fn prepare_block_heading(output: &mut String, ctx: &Context, options: &ConversionOptions) {
    if ctx.in_list_item {
        if output.ends_with('\n') {
            if let Some(indent) = continuation_indent_string(ctx.list_depth, options) {
                output.push_str(&indent);
            }
        } else if !output.ends_with(' ') && !output.is_empty() {
            output.push(' ');
        }
    } else if !output.is_empty() && !output.ends_with("\n\n") {
        if output.ends_with('\n') {
            output.push('\n');
        } else {
            crate::converter::trim_trailing_whitespace(output);
            output.push_str("\n\n");
        }
    }
}

fn render_heading_style(output: &mut String, ctx: &Context, options: &ConversionOptions, level: usize, text: &str) {
    match options.heading_style {
        HeadingStyle::Underlined => render_underlined_heading(output, ctx, options, level, text),
        HeadingStyle::Atx => {
            output.extend(std::iter::repeat_n('#', level));
            output.push(' ');
            output.push_str(&atx_heading_text(text, options));
        }
        HeadingStyle::AtxClosed => {
            output.extend(std::iter::repeat_n('#', level));
            output.push(' ');
            output.push_str(text);
            output.push(' ');
            output.extend(std::iter::repeat_n('#', level));
        }
    }
}

fn render_underlined_heading(
    output: &mut String,
    ctx: &Context,
    options: &ConversionOptions,
    level: usize,
    text: &str,
) {
    // ~keep The underline is a line of the item like every quote line, so it gets the
    // ~keep item's continuation indent; at column 0 a `-` underline is a new list item
    // ~keep (issue #635).
    let underline_indent = if ctx.in_list_item {
        crate::converter::list::utils::continuation_indent_string(ctx.list_indent_columns, options)
    } else {
        None
    };
    // ~keep The text is a paragraph line, so a list marker or other block opener at its
    // ~keep start is escaped (issue #653).
    if level == 1 {
        output.push_str(&crate::converter::utility::escaping::escape_paragraph_start(text, b'='));
        output.push('\n');
        output.push_str(underline_indent.as_deref().unwrap_or_default());
        output.extend(std::iter::repeat_n('=', text.len()));
    } else if level == 2 {
        output.push_str(&crate::converter::utility::escaping::escape_paragraph_start(text, b'-'));
        output.push('\n');
        output.push_str(underline_indent.as_deref().unwrap_or_default());
        // ~keep In a list item a lone `-` line reads as an empty item marker, both to
        // ~keep CommonMark after a blank line and to the item's own marker checks, so the
        // ~keep underline there has at least two dashes (issue #635).
        let width = if ctx.in_list_item {
            text.len().max(2)
        } else {
            text.len()
        };
        output.extend(std::iter::repeat_n('-', width));
    } else {
        output.extend(std::iter::repeat_n('#', level));
        output.push(' ');
        output.push_str(&atx_heading_text(text, options));
    }
}

/// The text of an ATX heading line that ends without a closing sequence of its own, with a `#` run
/// at its end escaped when a parser would read it as that sequence (issue #661).
///
/// ~keep A Djot heading has no closing sequence, so its text stays as it is.
fn atx_heading_text<'a>(text: &'a str, options: &ConversionOptions) -> Cow<'a, str> {
    match crate::converter::utility::escaping::atx_closing_sequence_offset(text) {
        Some(at) if options.output_format == OutputFormat::Markdown => {
            Cow::Owned(format!("{}\\{}", &text[..at], &text[at..]))
        }
        _ => Cow::Borrowed(text),
    }
}

/// Get continuation indent string for list items.
fn continuation_indent_string(list_depth: usize, _options: &ConversionOptions) -> Option<String> {
    if list_depth == 0 {
        return None;
    }
    let mut indent = String::new();
    for _ in 0..(4 * list_depth) {
        indent.push(' ');
    }
    Some(indent)
}

/// Process heading with visitor callback if available.
#[cfg(feature = "visitor")]
fn visitor_heading_output(
    node_handle: &NodeHandle,
    parser: &Parser,
    tag_name: &str,
    level: usize,
    normalized: &str,
    handler: HandlerContext<'_>,
) -> Option<String> {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let ctx = handler.ctx;
    let Some(ref visitor_handle) = ctx.visitor else {
        let mut buf = String::new();
        push_heading(&mut buf, ctx, handler.options, level, normalized);
        return Some(buf);
    };
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };
    let id_attr = tag
        .attributes()
        .get("id")
        .flatten()
        .map(|value| value.as_utf8_str().to_string());
    let node_id = node_handle.get_inner();
    let parent_tag = handler.dom_ctx.parent_tag_name(node_id, parser);
    let index_in_parent = handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0);
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::Heading,
        Cow::Borrowed(tag_name),
        tag,
        handler.depth,
        index_in_parent,
        parent_tag.map(Cow::Borrowed),
        false,
    );
    let visit_result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        visitor.visit_heading(&node_ctx, level as u32, normalized, id_attr.as_deref())
    };
    match visit_result {
        VisitResult::Continue | VisitResult::PreserveHtml => {
            let mut buf = String::new();
            push_heading(&mut buf, ctx, handler.options, level, normalized);
            Some(buf)
        }
        VisitResult::Custom(custom) => {
            if let Some(collector) = ctx.structure_collector.as_ref() {
                collector.borrow_mut().replace_current_element(Some(&custom));
            }
            Some(custom)
        }
        VisitResult::Skip => {
            if let Some(collector) = ctx.structure_collector.as_ref() {
                collector.borrow_mut().replace_current_element(None);
            }
            None
        }
        VisitResult::Error(err) => {
            if let Some(collector) = ctx.structure_collector.as_ref() {
                collector.borrow_mut().replace_current_element(None);
            }
            if ctx.visitor_error.borrow().is_none() {
                *ctx.visitor_error.borrow_mut() = Some(err);
            }
            None
        }
    }
}

/// Find a single heading element within a node, filtering out non-heading content.
///
/// Returns the heading level and node handle if the node contains exactly one
/// heading with no other non-whitespace content. Returns None if:
/// - The node is not a tag
/// - Multiple headings are found
/// - Non-whitespace non-heading content exists
/// - Non-text comments exist
pub fn find_single_heading_child(node_handle: NodeHandle, parser: &Parser) -> Option<(usize, NodeHandle)> {
    let node = node_handle.get(parser)?;

    let tl::Node::Tag(tag) = node else {
        return None;
    };

    let children = tag.children();
    let mut heading_data: Option<(usize, NodeHandle)> = None;

    for child_handle in children.top().iter() {
        let Some(child_node) = child_handle.get(parser) else {
            continue;
        };

        match child_node {
            tl::Node::Raw(bytes) => {
                if !bytes.as_utf8_str().trim().is_empty() {
                    return None;
                }
            }
            tl::Node::Tag(child_tag) => {
                let name = crate::converter::utility::content::normalized_tag_name(child_tag.name().as_utf8_str());
                {
                    let level = heading_level_from_name(name.as_ref())?;
                    if heading_data.is_some() {
                        return None;
                    }
                    heading_data = Some((level, *child_handle));
                }
            }
            tl::Node::Comment(_) => return None,
        }
    }

    heading_data
}

/// Extract heading level from tag name (h1-h6).
///
/// Returns Some(level) for valid heading tags, None otherwise.
fn heading_level_from_name(name: &str) -> Option<usize> {
    match name {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}
