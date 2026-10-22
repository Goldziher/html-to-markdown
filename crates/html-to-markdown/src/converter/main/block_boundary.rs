//! What a block boundary writes before the node after it: a space in an inline conversion (a
//! link label, a heading), a line at the content column in a list item.

use super::{ends_with_block_line_end, is_inline_content};
use crate::converter::block::container::HandlerContext;
use crate::converter::dom_context::DomContext;
use crate::converter::main_helpers::{is_inline_element, strip_trailing_backslash_breaks, trim_trailing_whitespace};
use crate::converter::preprocessing_helpers::{is_page_header, should_drop_for_preprocessing};
use crate::converter::utility::content::{is_block_level_element, is_inline_code};
use crate::converter::utility::siblings::previous_content_block_through;
use crate::options::NewlineStyle;

/// Start `node` at the list item's content column when it is a block after other content of the
/// item, or inline content after a block of the item (issue #583).
///
/// ~keep `CommonMark` keeps a block inside a list item only when every line of it starts at the
/// ~keep item's content column. A block after the item's text starts on a new line at that
/// ~keep column; each block handler then writes the blank line it needs before itself. Inline
/// ~keep content after a block starts a paragraph of its own: a blank line, then the column. A
/// ~keep heading is one line that nothing continues, so after it the column alone does, and the
/// ~keep list stays tight (spec example 300: `- ## Bar\n  baz`).
/// ~keep Where the item is not open (a list between markers, issue #615), a quote or a
/// ~keep list written within 3 columns of the innermost real item's content column (the start
/// ~keep of the line when no item is real) is still a real block, and text after it on the next
/// ~keep line continues its last paragraph lazily. That text gets the blank line and the real
/// ~keep item's column. Further in, the block's lines are the paragraph's own text, and a blank
/// ~keep line would split the paragraph between the markers.
/// ~keep A list is left out as the block: it already starts its own line at its own column.
/// ~keep A lone line break is not a block's last line: the block before it wrote nothing.
pub(super) fn separate_in_list_item(
    node: &tl::Node,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let HandlerContext {
        options, ctx, dom_ctx, ..
    } = handler;
    // ~keep A block that preprocessing drops (a `<nav>`) writes nothing, so it gets no line.
    let starts_block = match node {
        tl::Node::Tag(tag) => dom_ctx.tag_info(node_handle.get_inner(), parser).is_some_and(|info| {
            is_block_level_element(&info.name)
                && !matches!(info.name.as_str(), "ul" | "ol" | "li")
                && !should_drop_for_preprocessing(
                    &info.name,
                    tag,
                    options,
                    info.name == "header" && is_page_header(node_handle, parser, dom_ctx),
                )
        }),
        _ => false,
    };
    let indent =
        crate::converter::list::utils::continuation_indent_string(ctx.list_indent_columns, options).unwrap_or_default();
    let (blank_line, block_continues_lazily) = if starts_block {
        // ~keep A hard break right before a block is dropped here too, since the line end
        // ~keep written below would hide it from the dispatch strip in `walk_node`.
        if options.newline_style == NewlineStyle::Backslash {
            strip_trailing_backslash_breaks(output, ctx.block_content_start);
        }
        let line_start = output.rfind('\n').map_or(0, |pos| pos + 1);
        let line = &output[line_start..];
        let after_content = line.is_empty()
            || (!line.trim().is_empty() && !crate::converter::list::utils::line_is_bare_list_marker(output));
        if !after_content {
            return;
        }
        (false, false)
    } else if ends_with_block_line_end(output) && is_inline_content(node, node_handle, parser, dom_ctx) {
        match crate::converter::utility::siblings::previous_content_block(node_handle, parser, dom_ctx) {
            Some(block) => (
                !matches!(block, "h1" | "h2" | "h3" | "h4" | "h5" | "h6"),
                matches!(block, "blockquote" | "ul" | "ol"),
            ),
            None => return,
        }
    } else {
        return;
    };
    let item_is_open = crate::converter::list::utils::item_is_open(output, &indent, ctx);
    let separates =
        item_is_open || (block_continues_lazily && crate::converter::list::utils::block_is_real(ctx, options));
    if !separates {
        return;
    }
    trim_trailing_whitespace(output);
    if !output.ends_with('\n') {
        output.push('\n');
    }
    if blank_line && !output.ends_with("\n\n") {
        output.push('\n');
    }
    if item_is_open {
        output.push_str(&indent);
    } else if let Some(real_item_indent) =
        crate::converter::list::utils::continuation_indent_string(ctx.real_item_columns, options)
    {
        output.push_str(&real_item_indent);
    }
}

/// Write the space that a block boundary is in an inline conversion (a link label, a heading):
/// before a block, and before inline content that follows a block (issue #751).
///
/// ~keep A reader sees two blocks on two lines, so their texts are two words at every depth of
/// ~keep nesting. Inline content beside inline content gets nothing here: its own white space
/// ~keep decides, so `<b>H</b>ello` stays one word.
pub(super) fn separate_words_at_block_boundary(
    node: &tl::Node,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let HandlerContext { ctx, dom_ctx, .. } = handler;
    if output.ends_with(char::is_whitespace) {
        return;
    }
    // ~keep A kept HTML block is text here, as it is in a cell: it is no boundary on either side.
    let kept_as_html = dom_ctx
        .tag_info(node_handle.get_inner(), parser)
        .is_some_and(|info| ctx.preserve_tags.contains(info.name.as_str()));
    if kept_as_html {
        return;
    }
    // ~keep A link label wrote one space beside a child with a block before this rule, also
    // ~keep beside code. A heading wrote none beside code. Each keeps what it wrote.
    let looks_into: fn(&str) -> bool = if ctx.in_link {
        is_inline_element
    } else {
        wraps_running_text
    };
    let at_boundary = starts_with_block(node_handle, parser, dom_ctx, looks_into)
        || (is_inline_content(node, node_handle, parser, dom_ctx)
            && previous_content_block_through(node_handle, parser, dom_ctx, looks_into)
                .is_some_and(|block| !ctx.preserve_tags.contains(block)));
    if at_boundary {
        output.push(' ');
        crate::converter::structure_capture::append_text(ctx, " ");
    }
}

/// Whether an inline element holds running text: every inline element but code.
///
/// ~keep A block in code (`<code><hr></code>`) is code text, written as it was before this
/// ~keep rule. It is no block boundary of the text around the code.
fn wraps_running_text(tag_name: &str) -> bool {
    is_inline_element(tag_name) && !is_inline_code(tag_name)
}

/// Whether `node_handle` is a block, or an element whose content starts with a block
/// (`<b><div>...</div></b>`). White space text and comments before the block are skipped.
/// The walk looks only into the inline elements that `looks_into` accepts.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn starts_with_block(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
    looks_into: fn(&str) -> bool,
) -> bool {
    let mut handle = *node_handle;
    // ~keep A loop rather than recursion: nested inline wrappers are attacker-controlled depth.
    loop {
        if crate::converter::utility::content::node_is_block_level(&handle, parser, dom_ctx) {
            return true;
        }
        let is_closed = dom_ctx
            .tag_info(handle.get_inner(), parser)
            .is_some_and(|info| is_inline_element(&info.name) && !looks_into(&info.name));
        if is_closed {
            return false;
        }
        let first_content = dom_ctx.children_of(handle.get_inner()).and_then(|children| {
            children.iter().find(|child| match child.get(parser) {
                Some(tl::Node::Raw(raw)) => !raw.as_utf8_str().trim().is_empty(),
                Some(tl::Node::Tag(_)) => true,
                _ => false,
            })
        });
        // ~keep Text as the first content ends the walk at the next turn: it is no block and holds nothing.
        let Some(first) = first_content else {
            return false;
        };
        handle = *first;
    }
}
