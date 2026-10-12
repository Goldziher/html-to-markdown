//! Handler for div element.
//!
//! Converts HTML div elements to Markdown by processing children while maintaining
//! appropriate spacing and context awareness for:
//! - Table continuations: Uses table-specific line breaks
//! - List continuations: Uses list indentation
//! - Block context: Adds surrounding newlines for proper block separation

use crate::converter::block::container::HandlerContext;
use crate::converter::main_helpers::{strip_trailing_backslash_breaks, trim_trailing_whitespace};
use crate::options::{ConversionOptions, NewlineStyle};
use tl::{NodeHandle, Parser};

type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handles div elements.
///
/// Divs are generic container elements that need special handling based on context:
/// - When inline context: passes through children without separators
/// - When in table cell: uses table-specific line breaks (<br> or backslash)
/// - When in list item: uses list continuation indentation
/// - When in block context: adds appropriate newlines before/after content
///
/// # Note
/// This function references `walk_node` and helper functions from converter.rs
/// which must be accessible (pub(crate)) for this module to work correctly.
pub fn handle(node_handle: &NodeHandle, parser: &Parser, output: &mut String, handler: HandlerContext<'_>) {
    use crate::converter::walk_node;

    let HandlerContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = handler;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    let is_table_continuation = (ctx.in_table_cell || ctx.in_layout_cell)
        && !output.is_empty()
        && !output.ends_with('|')
        && !output.ends_with("<br>")
        // ~keep A layout cell, unlike a real one, may already hold a newline: its row renders as a
        // ~keep list item, not a pipe row. Separating there opens the next line with a stray space.
        // ~keep Inert for `in_table_cell`, whose buffer never holds a newline by construction.
        && !output.ends_with('\n');

    if ctx.convert_as_inline {
        handle_inline(tag, parser, output, handler, is_table_continuation);
        return;
    }

    let content_start_pos = output.len();
    // ~keep An empty div in a list item leaves the item as it found it (issue #583): its list
    // ~keep separator trims the line end, so the trimmed tail is kept to put back.
    let kept = ctx.in_list_item.then(|| {
        let kept_len = output.trim_end_matches([' ', '\t']).len();
        (kept_len, output[kept_len..].to_string())
    });
    let is_list_continuation = open_block(output, options, ctx);

    // ~keep Measured the same way `block/paragraph.rs` does, so a text node can tell "at the
    // ~keep start of this div's line, in this div's buffer" from an inline wrapper's empty
    // ~keep scratch buffer. Without it a whitespace-only `<span>` opening a `<div>` was pushed
    // ~keep verbatim and `<span>    </span><img>` became an indented code block (issue #501).
    let children_start = output.len();
    let div_ctx = Context {
        block_content_start: output.len(),
        block_output_ptr: std::ptr::from_ref::<String>(output) as usize,
        ..ctx.clone()
    };

    let children = tag.children();
    {
        for child_handle in children.top().iter() {
            walk_node(
                child_handle,
                parser,
                output,
                crate::converter::block::container::HandlerContext::new(options, &div_ctx, depth + 1, dom_ctx),
            );
        }
    }

    if options.newline_style == NewlineStyle::Backslash {
        // ~keep A trailing <br> run with no following sibling has no next dispatch to catch
        // ~keep it in `walk_node`'s pre-block-dispatch strip, since the div is simply
        // ~keep finishing here — so this closes its own trailing run the same way
        // ~keep `paragraph.rs` closes its own (issue #464 follow-up).
        strip_trailing_backslash_breaks(output, content_start_pos);
    }

    if let Some((kept_len, kept_tail)) = kept.filter(|_| output.len() == children_start) {
        output.truncate(kept_len);
        output.push_str(&kept_tail);
    }
    close_block(output, ctx, content_start_pos, is_list_continuation);
}

fn handle_inline(
    tag: &tl::HTMLTag<'_>,
    parser: &Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
    is_table_continuation: bool,
) {
    if is_table_continuation {
        crate::converter::main_helpers::emit_table_cell_break_in_context(
            output,
            handler.options.br_in_tables,
            handler.ctx,
        );
    }
    // ~keep Heading block separators are running-text syntax, not code-span content.
    // ~keep Code retains its source spaces without synthetic boundary padding.
    let in_heading_line = handler.ctx.in_heading
        && !handler.ctx.in_code
        && !is_table_continuation
        && handler.options.whitespace_mode != crate::options::WhitespaceMode::Strict;
    if in_heading_line && !output.is_empty() && !output.ends_with(char::is_whitespace) {
        output.push(' ');
    }
    for child_handle in tag.children().top().iter() {
        crate::converter::walk_node(
            child_handle,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                handler.ctx,
                handler.depth + 1,
                handler.dom_ctx,
            ),
        );
    }
    if in_heading_line && !output.ends_with(char::is_whitespace) {
        output.push(' ');
    }
}

/// Writes `text` as its own block, with the separators a div writes around its content.
pub fn push_block(output: &mut String, options: &ConversionOptions, ctx: &Context, text: &str) {
    let content_start_pos = output.len();
    let is_list_continuation = open_block(output, options, ctx);
    output.push_str(text);
    close_block(output, ctx, content_start_pos, is_list_continuation);
}

/// Writes the break a div writes before its content and returns whether that content continues
/// a list item.
fn open_block(output: &mut String, options: &ConversionOptions, ctx: &Context) -> bool {
    let is_table_continuation = (ctx.in_table_cell || ctx.in_layout_cell)
        && !output.is_empty()
        && !output.ends_with('|')
        && !output.ends_with("<br>")
        && !output.ends_with('\n');

    // ~keep A plain suffix check like `output.ends_with("* ")` also matches the closing
    // ~keep "**"/"*" of `<strong>`/`<em>` immediately followed by a migrated trailing
    // ~keep space, indistinguishable from a real bare bullet by suffix alone -- and,
    // ~keep being hardcoded to `-`/`*`, never matched the third bullet `+` at all. The
    // ~keep false positive misclassified this div as sitting right after the marker,
    // ~keep which skips BOTH branches below (neither `is_list_continuation` nor
    // ~keep `needs_leading_sep` fires), so the div's content got glued directly onto the
    // ~keep preceding inline text with no separator at all. See
    // ~keep `list::utils::line_is_bare_list_marker`'s doc comment for the full rationale.
    let is_list_continuation =
        ctx.in_list_item && !output.is_empty() && !crate::converter::list::utils::line_is_bare_list_marker(output);

    let needs_leading_sep = !ctx.in_table_cell
        && !ctx.in_list_item
        && !ctx.convert_as_inline
        && !output.is_empty()
        && !output.ends_with("\n\n");

    if is_table_continuation {
        crate::converter::main_helpers::emit_table_cell_break_in_context(output, options.br_in_tables, ctx);
    } else if is_list_continuation {
        crate::converter::list::utils::start_block_in_list_item(output, ctx, options);
    } else if needs_leading_sep {
        trim_trailing_whitespace(output);
        output.push_str("\n\n");
    }
    is_list_continuation
}

/// Writes the break a div writes after its content, when anything was written from
/// `content_start_pos` on.
fn close_block(output: &mut String, ctx: &Context, content_start_pos: usize, is_list_continuation: bool) {
    let has_content = output.len() > content_start_pos;

    if has_content {
        if content_start_pos == 0 && output.starts_with('\n') && !output.starts_with("\n\n") {
            output.remove(0);
        }
        trim_trailing_whitespace(output);

        if ctx.in_table_cell {
            // ~keep No trailing separator in table cells
        } else if ctx.in_list_item {
            if is_list_continuation {
                if !output.ends_with('\n') {
                    output.push('\n');
                }
            } else if !output.ends_with("\n\n") {
                if output.ends_with('\n') {
                    output.push('\n');
                } else {
                    output.push_str("\n\n");
                }
            }
        } else if !ctx.in_list_item && !ctx.convert_as_inline {
            if output.ends_with("\n\n") {
            } else if output.ends_with('\n') {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }
    }
}
