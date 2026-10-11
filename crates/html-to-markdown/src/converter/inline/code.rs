//! Handler for the code-like inline elements `<kbd>` and `<samp>`.
//!
//! Converts them to Markdown inline code formatting with support for:
//! - Keyboard input (<kbd>) rendered as code
//! - Sample output (<samp>) rendered as code
//! - Nested code context tracking (suppress formatting inside <code>/<pre>)
//! - Whitespace normalization for kbd/samp elements
//!
//! `<code>` itself is handled by `converter::handlers::code_block::handle_code`, which is
//! what `converter::main`'s walk dispatches to; this module's own `<code>` arm was a second,
//! silently divergent implementation that nothing ever reached. ~keep

use crate::converter::inline::{HandlerContext, wrapped::InlineSite, wrapped::emit_code_span};
use crate::text;

type Context = crate::converter::Context;

/// Handler for code-related inline elements: code, kbd (keyboard), and samp (sample output).
///
/// Processes code content based on context:
/// - For <code> within <code>: passes content through without wrapping backticks (nested code detection)
/// - For <kbd> and <samp>: normalizes whitespace and wraps with backticks
/// - For standalone <code>: applies smart backtick escaping and delimiter spacing
/// - Handles visitor callbacks for custom behavior when feature is enabled
/// - Properly escapes backticks in content that contains them
///
/// # Note
/// This function references helper functions and `walk_node` from converter.rs
/// which must be accessible (pub(crate)) for this module to work correctly.
pub fn handle(tag_name: &str, context: HandlerContext<'_>) {
    match tag_name {
        "kbd" | "samp" => handle_kbd_samp(context),
        _ => {}
    }
}

/// Handle keyboard and sample output elements (<kbd> and <samp> tags).
///
/// These elements are rendered as inline code with:
/// - Whitespace normalization (via `text::normalize_whitespace`)
/// - Chomp inline handling for prefix/suffix spacing
/// - The same delimiter selection and padding as `<code>` ~keep
fn handle_kbd_samp(mut handler: HandlerContext<'_>) {
    use crate::converter::{append_inline_suffix, chomp_inline};

    let site = handler.inline_site();
    let Some(node) = handler.node_handle.get(handler.parser) else {
        return;
    };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    let children = tag.children();
    if handler.context.code_element_is_transparent() {
        // ~keep A nested `<code>` renders transparently inside an outer code span
        // ~keep (`handlers::code_block::handle_code`); `<kbd>`/`<samp>` wrapped their own
        // ~keep backticks anyway, so the outer span grew a second, nested pair.
        for child_handle in children.top().iter() {
            walk_child(child_handle, handler.output, handler.context, handler.depth, site);
        }
        return;
    }

    let mut content = String::with_capacity(32);
    let code_ctx = Context {
        in_code: true,
        ..handler.context.clone()
    };
    for child_handle in children.top().iter() {
        walk_child(child_handle, &mut content, &code_ctx, handler.depth, site);
    }

    let normalized = text::normalize_whitespace(&content);
    let (prefix, suffix, trimmed) = chomp_inline(&normalized);

    if content.is_empty() {
        return;
    }

    // ~keep issue #481: an all-whitespace body is not an empty one -- `<kbd> </kbd>` is a key
    // ~keep whose label is a space. Render the span over the normalized body rather than over
    // ~keep `trimmed`, which is empty exactly when the whole body was whitespace.
    let body = if trimmed.is_empty() {
        normalized.as_str()
    } else {
        trimmed
    };
    let emit_prefix = if trimmed.is_empty() { "" } else { prefix };
    let emit_suffix = if trimmed.is_empty() { "" } else { suffix };

    crate::converter::utility::content::push_inline_prefix(
        handler.output,
        emit_prefix,
        handler.options,
        handler.context,
    );
    emit_kbd_samp_segments(body, emit_prefix.is_empty(), &mut handler);
    append_inline_suffix(
        handler.output,
        emit_suffix,
        !body.is_empty(),
        handler.node_handle,
        handler.parser,
        handler.dom_context,
    );
}

fn walk_child(
    child_handle: &tl::NodeHandle,
    output: &mut String,
    context: &Context,
    depth: usize,
    site: InlineSite<'_>,
) {
    crate::converter::walk_node(
        child_handle,
        site.parser,
        output,
        crate::converter::block::container::HandlerContext::new(site.options, context, depth + 1, site.dom_ctx),
    );
}

/// Render `body` as one or more backtick spans, split on the `'\n'` internal split marker
/// `line_break.rs`'s code-SPAN branch pushes for each `<br>` the element contained (see
/// `handlers::code_block::emit_inline_code`'s doc comment for the full reasoning, shared
/// verbatim by `<kbd>`/`<samp>` here). `may_merge_first` is false whenever `emit_prefix` was
/// non-empty (the caller already pushed literal prefix text, so the first span can no longer
/// be adjacent to a preceding sibling's closing backtick). ~keep
fn emit_kbd_samp_segments(body: &str, may_merge_first: bool, handler: &mut HandlerContext<'_>) {
    let separator = crate::converter::main_helpers::hard_break_marker(handler.options);
    let mut first = true;
    for segment in body.split('\n').filter(|segment| !segment.is_empty()) {
        if first {
            let mut span = String::with_capacity(segment.len() + 2);
            crate::converter::handlers::code_block::format_inline_code(segment, &mut span);
            if may_merge_first {
                emit_code_span(
                    &span,
                    segment,
                    handler.output,
                    handler.node_handle,
                    handler.parser,
                    handler.dom_context,
                );
            } else {
                handler.output.push_str(&span);
            }
        } else {
            // ~keep Only the first segment may merge into a preceding sibling span
            // ~keep (issue #483): every later segment is preceded by our own
            // ~keep separator, never a bare closing backtick.
            handler.output.push_str(separator);
            crate::converter::handlers::code_block::format_inline_code(segment, handler.output);
        }
        first = false;
    }
}
