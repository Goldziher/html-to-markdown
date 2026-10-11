//! The space that a block boundary is in an inline conversion (a link label, a heading).

use super::is_inline_content;
use crate::converter::block::container::HandlerContext;
use crate::converter::dom_context::DomContext;
use crate::converter::main_helpers::is_inline_element;
use crate::converter::utility::content::is_inline_code;
use crate::converter::utility::siblings::previous_content_block_through;

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
