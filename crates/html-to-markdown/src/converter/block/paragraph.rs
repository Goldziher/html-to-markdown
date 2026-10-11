//! Handler for paragraph elements (p, div).
//!
//! Converts HTML paragraph tags to Markdown paragraphs with proper spacing
//! and support for:
//! - Continuation handling in tables and lists
//! - Proper blank line spacing
//! - Empty element filtering
//! - Visitor callbacks for custom paragraph processing

use crate::converter::block::container::HandlerContext;
use crate::converter::main_helpers::is_ascii_whitespace_only;
use crate::converter::walk::{Close, Open};
use crate::options::{ConversionOptions, NewlineStyle};
use std::rc::Rc;
use tl::{NodeHandle, Parser};

type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// What `close` needs from `open`.
pub struct ParagraphEnd {
    content_start_pos: usize,
    block_content_start: usize,
}

/// Write what a paragraph writes before its children and list the children in `children`.
///
/// Manages spacing, and handles special cases for table cells and list items.
pub fn open(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
    children: &mut Vec<NodeHandle>,
) -> Open {
    let HandlerContext { options, ctx, .. } = handler;

    let content_start_pos = output.len();
    open_paragraph(output, options, ctx);
    let p_ctx = Context {
        in_paragraph: true,
        block_content_start: output.len(),
        block_output_ptr: std::ptr::from_ref::<String>(output) as usize,
        ..ctx.clone()
    };
    list_paragraph_children(node_handle, parser, handler.dom_ctx, children);
    Open {
        close: Close::Paragraph(ParagraphEnd {
            content_start_pos,
            block_content_start: p_ctx.block_content_start,
        }),
        scope: Some(Rc::new(p_ctx)),
    }
}

/// Write what a paragraph writes after its children.
pub fn close(end: &ParagraphEnd, output: &mut String, handler: HandlerContext<'_>) {
    close_paragraph(
        output,
        handler.options,
        handler.ctx,
        end.block_content_start,
        end.content_start_pos,
    );
}

fn open_paragraph(output: &mut String, options: &ConversionOptions, ctx: &Context) {
    let is_table_continuation = (ctx.in_table_cell || ctx.in_layout_cell)
        && !output.is_empty()
        && !output.ends_with('|')
        && !output.ends_with("<br>")
        // ~keep A layout cell, unlike a real one, may already hold a newline: its row renders as a
        // ~keep list item, not a pipe row. Separating there opens the next line with a stray space.
        // ~keep Inert for `in_table_cell`, whose buffer never holds a newline by construction.
        && !output.ends_with('\n');

    let is_list_continuation = ctx.in_list_item && !output.is_empty() && !ends_with_bare_list_marker(output, options);

    let after_code_block = output.ends_with("```\n");
    // ~keep Inside a blockquote, sibling blocks (heading, list, table, pre) already manage
    // ~keep their own trailing spacing and self-terminate without a blank line (matches
    // ~keep CommonMark: an ATX heading ends its line regardless). The one case that still
    // ~keep needs an explicit separator here is a paragraph directly after bare inline text,
    // ~keep which leaves no trailing newline at all — without it the "> " prefixing pass
    // ~keep merges the text and the paragraph into a single line, losing the block break
    // ~keep (issue #13). Requiring "no trailing newline at all" (not just "no blank line")
    // ~keep keeps the existing heading-then-paragraph compact style intact.
    let needs_leading_sep = !ctx.in_table_cell
        && !ctx.in_list_item
        && !ctx.convert_as_inline
        && !output.is_empty()
        && !after_code_block
        && if ctx.blockquote_depth > 0 {
            !output.ends_with('\n')
        } else {
            !output.ends_with("\n\n")
        };

    if is_table_continuation {
        crate::converter::main_helpers::emit_table_cell_break_in_context(output, options.br_in_tables, ctx);
    } else if is_list_continuation {
        crate::converter::list::utils::start_block_in_list_item(output, ctx, options);
    } else if needs_leading_sep {
        crate::converter::trim_trailing_whitespace(output);
        output.push_str("\n\n");
    }
}

/// List the children of a paragraph that the walk visits, in document order.
fn list_paragraph_children(
    node_handle: &NodeHandle,
    parser: &Parser,
    dom_ctx: &DomContext,
    children: &mut Vec<NodeHandle>,
) {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return;
    };
    let child_handles: std::borrow::Cow<'_, [NodeHandle]> = match dom_ctx.children_of(node_handle.get_inner()) {
        Some(children) => std::borrow::Cow::Borrowed(children.as_slice()),
        None => std::borrow::Cow::Owned(tag.children().top().iter().copied().collect()),
    };

    for (index, child_handle) in child_handles.iter().enumerate() {
        if should_skip_interstitial_whitespace(child_handle, index, &child_handles, parser) {
            continue;
        }
        children.push(*child_handle);
    }
}

fn should_skip_interstitial_whitespace(
    child_handle: &NodeHandle,
    index: usize,
    child_handles: &[NodeHandle],
    parser: &Parser,
) -> bool {
    let Some(tl::Node::Raw(bytes)) = child_handle.get(parser) else {
        return false;
    };
    let text = bytes.as_utf8_str();
    if !is_ascii_whitespace_only(&text) || index == 0 || index == child_handles.len() - 1 {
        return false;
    }
    if is_image_element(&child_handles[index - 1], parser) && is_image_element(&child_handles[index + 1], parser) {
        return false;
    }
    is_empty_inline_element(&child_handles[index - 1], parser)
        && is_empty_inline_element(&child_handles[index + 1], parser)
}

fn is_image_element(node_handle: &NodeHandle, parser: &Parser) -> bool {
    matches!(
        node_handle.get(parser),
        Some(tl::Node::Tag(tag)) if tag.name().as_utf8_str().eq_ignore_ascii_case("img")
    )
}

fn close_paragraph(
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    block_content_start: usize,
    content_start_pos: usize,
) {
    if options.newline_style == NewlineStyle::Backslash {
        // ~keep A trailing run of <br> has no next line to break to, so the backslash
        // ~keep markers it emitted would otherwise leave literal, visible "\" characters at
        // ~keep the end of the block (issue #464). The two-space style is left alone here:
        // ~keep its leftover marker is invisible trailing whitespace, not a visible artifact.
        crate::converter::strip_trailing_backslash_breaks(output, block_content_start);
    }

    let has_content = output.len() > content_start_pos;

    if has_content && !ctx.convert_as_inline && !ctx.in_table_cell {
        output.push_str("\n\n");
    }
}

#[cfg(test)]
fn structure_text(output: &str, content_start_pos: usize) -> &str {
    let safe_start = crate::converter::utility::content::floor_char_boundary(output, content_start_pos);
    output[safe_start..].trim()
}

/// Whether `output` ends with a bare list marker and nothing else: an ordered marker's
/// "N. " (matched generically via the trailing ". ", regardless of digit count) or one of the
/// user-configured bullet characters in `options.bullets` followed by its trailing space.
///
/// ~keep Hardcoding only '*' and '-' here missed any other configured bullet -- the default
/// ~keep `bullets` cycle is "-*+", so a paragraph as the first content of a THIRD-level nested
/// ~keep list item (marker "+ ") fell through this check, was wrongly treated as a
/// ~keep mid-paragraph continuation, and got a second, redundant continuation indent stacked
/// ~keep onto the very first line after its own marker (spec example 307).
fn ends_with_bare_list_marker(output: &str, options: &ConversionOptions) -> bool {
    if output.ends_with(". ") {
        return true;
    }
    let mut chars = output.chars().rev();
    if chars.next() != Some(' ') {
        return false;
    }
    chars
        .next()
        .is_some_and(|marker_char| options.bullets.contains(marker_char))
}

/// Check if an element is empty (has no text content).
fn is_empty_inline_element(node_handle: &NodeHandle, parser: &Parser) -> bool {
    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Tag(tag) => {
                let tag_name = tag.name().as_utf8_str();
                matches!(tag_name.as_ref(), "br" | "hr" | "img" | "input" | "meta" | "link")
            }
            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::structure_text;

    #[test]
    fn should_clamp_structure_slice_when_output_shrinks_below_a_multibyte_start() {
        assert_eq!(structure_text("中", 4), "");
    }

    #[test]
    fn should_floor_structure_slice_when_start_lands_inside_a_multibyte_character() {
        assert_eq!(structure_text("a中", 2), "中");
    }
}
