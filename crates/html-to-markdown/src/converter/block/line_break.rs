//! Handler for line break elements (br).
//!
//! Converts HTML line break tags to Markdown line breaks using the configured
//! newline style (spaces, backslash, or plain newline).

use crate::converter::block::container::HandlerContext;
use crate::converter::main_helpers::{emit_table_cell_break, hard_break_marker, trim_trailing_whitespace};
use crate::options::ConversionOptions;
#[cfg(feature = "visitor")]
use std::borrow::Cow;
use tl::{NodeHandle, Parser};

type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

fn is_paragraph_block_output(output: &String, ctx: &Context) -> bool {
    ctx.in_paragraph && std::ptr::from_ref::<String>(output) as usize == ctx.block_output_ptr
}

/// Handle line break elements (br).
///
/// Converts to appropriate Markdown line break syntax based on the configured
/// newline style and current context (e.g., in headings).
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle(node_handle: &NodeHandle, parser: &Parser, output: &mut String, handler: HandlerContext<'_>) {
    let HandlerContext {
        options,
        ctx,
        depth: _,
        dom_ctx: _,
    } = handler;
    #[cfg(feature = "visitor")]
    if visit_line_break(node_handle, parser, output, handler) {
        return;
    }

    if let Some(rule_like_text) = &ctx.djot_rule_like_text {
        rule_like_text.advance();
    }

    if write_special_break(output, ctx) {
        return;
    }
    write_flow_break(output, options, ctx);
}

fn write_special_break(output: &mut String, ctx: &Context) -> bool {
    if ctx.in_heading {
        // ~keep A single-line ATX heading cannot carry a hard break at all, so any marker
        // ~keep here is inherently lossy. A single space is the only choice that is
        // ~keep round-trip stable: a renderer's own HTML whitespace collapsing already
        // ~keep reduces a run of literal spaces to one on the next parse, so a
        // ~keep two-space marker here only survives the FIRST conversion before
        // ~keep collapsing on the second, and never reaches a fixpoint. Matches
        // ~keep Tier-1: `tier1/scanner.rs`'s `TagKind::LineBreak` handling emits the same
        // ~keep "  \n" marker regardless of heading context, but `close_heading` then
        // ~keep folds every whitespace run (including that marker) in the finished
        // ~keep heading body down to one space.
        if ctx.inline_code_links.is_some() {
            output.push('\n');
        } else {
            trim_trailing_whitespace(output);
            output.push(' ');
        }
        true
    } else if ctx.in_table_cell && ctx.in_code && !ctx.in_code_block {
        // ~keep Neither a code SPAN nor a table cell can carry a hard break on its own
        // ~keep (see the two ~keep blocks this combines, immediately below and at the
        // ~keep plain `in_table_cell` arm): fold straight to a single space rather than
        // ~keep letting the split path below run, which would break the span in two and
        // ~keep leave the second half on its own physical line -- corrupting the cell's
        // ~keep pipe-row syntax exactly as a raw newline would (issue #455's rule, now
        // ~keep also covering the `<br>`-driven case rather than only a literal source
        // ~keep newline). Not `emit_table_cell_break`: a literal `<br>` in HTML is not
        // ~keep valid content inside a code span regardless of `br_in_tables`, so that
        // ~keep option is never consulted here either.
        if ctx.inline_code_links.is_some() {
            // ~keep Collected link ranges must remain valid until the code span is emitted.
            output.push('\n');
        } else {
            trim_trailing_whitespace(output);
            output.push(' ');
        }
        true
    } else if ctx.in_code_block {
        // ~keep A `<pre>` code BLOCK reproduces its content literally, line structure
        // ~keep included: a `<br>` here is real content, so the byte pushed is a genuine
        // ~keep `\n`, not a marker -- `newline_style` is never consulted (issue #487).
        if let Some(ref offsets) = ctx.pre_cell_break_offsets {
            offsets.borrow_mut().push(output.len());
        }
        output.push('\n');
        true
    } else if ctx.in_code {
        // ~keep A code SPAN's content is otherwise reproduced literally too, but unlike a
        // ~keep block it has no interior line structure of its own to preserve: `<br>` is
        // ~keep a DOM-level split point, not span content. Push a plain '\n' as an
        // ~keep internal-only split marker rather than syntax -- `format_inline_code`
        // ~keep (`handlers/code_block.rs`) and `handle_kbd_samp` (`inline/code.rs`) later
        // ~keep split on it, rendering each half as its own backtick span joined by the
        // ~keep configured `newline_style` marker OUTSIDE the backticks, where it is
        // ~keep syntax (issue #487). This byte can only have come from a real `<br>`: a
        // ~keep source text node's own literal line ending is folded to a space before
        // ~keep it ever reaches this buffer (`text_node.rs`'s `in_code && !in_code_block`
        // ~keep branch), so nothing else can leave a bare '\n' here for this split to
        // ~keep misfire on.
        output.push('\n');
        true
    } else {
        false
    }
}

fn write_flow_break(output: &mut String, options: &ConversionOptions, ctx: &Context) {
    if ctx.in_table_cell || ctx.in_layout_cell {
        // ~keep Shared with div/p continuations inside a cell (issue #453, #454): a cell
        // ~keep cannot contain a hard line break, so newline_style is never consulted and
        // ~keep source whitespace before the <br> is trimmed rather than leaked. A layout
        // ~keep cell is one list item's line and already sends its div/p continuations
        // ~keep through this rule (issue #470); a hard-break marker there ended the item.
        // ~keep An empty buffer here is an inline wrapper's scratch buffer, not the cell's
        // ~keep own: `<td>A<em><br></em>B</td>` saw the continuation rule suppress its space
        // ~keep against `<em>`'s fresh `String`, and the words joined (issue #504). The one
        // ~keep space the `<br>` stands for is pushed there; a cell's own buffer is trimmed
        // ~keep by `render_cell_text`/`append_layout_cell_text`, so a genuinely leading
        // ~keep `<br>` still contributes nothing. A literal `<br>` under `br_in_tables` is
        // ~keep unchanged.
        if output.is_empty() && !options.br_in_tables {
            output.push(' ');
        } else {
            emit_table_cell_break(output, options.br_in_tables);
        }
    } else if ctx.in_link {
        // ~keep #497: inside a link label a `<br>` with nothing before it is still real
        // ~keep content -- `B<a href="H"><br>A</a>` renders as B, a line break, then A, and
        // ~keep `B[  \n A](H)` (without the space) re-parses to exactly that. The
        // ~keep `block_content_start` test below cannot see this case: a label is built into a
        // ~keep fresh local `String` whose length starts at 0 while `block_content_start` still
        // ~keep refers to the ENCLOSING block's buffer, so "nothing on this line yet" comes out
        // ~keep true or false by coincidence. Whether a break at either edge of the label
        // ~keep survives is `normalize_link_label`'s decision, not this one's.
        output.push_str(hard_break_marker(options));
    } else if output.len() == ctx.block_content_start {
        // ~keep A paragraph-leading break has no preceding line and therefore emits nothing
        // ~keep (#572), but only the paragraph's own buffer can prove that position. A fresh
        // ~keep inline-wrapper scratch buffer can have the same length after real paragraph
        // ~keep content; its bare newline is an internal sentinel that the wrapper turns into
        // ~keep either a real break before following content or a separator when break-only.
        // ~keep A bare top-level break also remains an intentional leading line (#112).
        if !is_paragraph_block_output(output, ctx) {
            output.push('\n');
        }
    } else {
        // ~keep A break on a line of its own (`<li>a<br><br>b</li>`) is written at the item's
        // ~keep content column like text there, or a backslash line leaves the item (issue #681).
        crate::converter::list::utils::indent_list_item_line_start(output, ctx, options);
        output.push_str(hard_break_marker(options));
    }
}

#[cfg(feature = "visitor")]
fn visit_line_break(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) -> bool {
    use crate::visitor::EMPTY_ATTRS;
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(ref visitor_handle) = handler.ctx.visitor else {
        return false;
    };
    let node_id = node_handle.get_inner();
    let parent_tag = handler.dom_ctx.parent_tag_name(node_id, parser);
    let index_in_parent = handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0);
    let node_ctx = if let Some(tl::Node::Tag(t)) = node_handle.get(parser) {
        NodeContext::with_lazy_attributes(
            NodeType::Br,
            Cow::Borrowed("br"),
            t,
            handler.depth,
            index_in_parent,
            parent_tag.map(Cow::Borrowed),
            true,
        )
    } else {
        NodeContext::with_borrowed_attributes(
            NodeType::Br,
            Cow::Borrowed("br"),
            &EMPTY_ATTRS,
            handler.depth,
            index_in_parent,
            parent_tag.map(Cow::Borrowed),
            true,
        )
    };
    let visit_result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        visitor.visit_line_break(&node_ctx)
    };
    match visit_result {
        VisitResult::Continue => false,
        VisitResult::Skip => true,
        VisitResult::Custom(custom) => {
            output.push_str(&custom);
            true
        }
        VisitResult::PreserveHtml => {
            use crate::converter::utility::serialization::serialize_node;
            output.push_str(&serialize_node(node_handle, parser));
            true
        }
        VisitResult::Error(err) => {
            if handler.ctx.visitor_error.borrow().is_none() {
                *handler.ctx.visitor_error.borrow_mut() = Some(err);
            }
            true
        }
    }
}
