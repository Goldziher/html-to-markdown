//! Handler for emphasis elements (strong, b, em, i).
//!
//! Converts HTML emphasis tags to Markdown formatting with support for:
//! - Bold/strong formatting using configurable symbols (** or __)
//! - Italic/emphasis formatting using configurable symbols (* or _)
//! - Nested emphasis context tracking
//! - Code context handling (suppress formatting in <code>)
//! - Visitor callbacks for custom emphasis processing
//! - Bootstrap caret detection (.caret class)

use crate::converter::inline::HandlerContext;
use crate::options::OutputFormat;
#[cfg(feature = "visitor")]
use std::borrow::Cow;
type Context = crate::converter::Context;

/// Handler for emphasis elements: strong, b (bold) and em, i (italic).
///
/// Processes emphasis content based on context:
/// - Suppresses formatting when already in strong/code context
/// - Applies configurable emphasis symbols (* or _)
/// - Handles nested emphasis with proper context tracking
/// - Supports visitor callbacks for custom behavior
/// - Detects Bootstrap caret elements (.caret class)
///
/// # Note
/// This function references helper functions and `walk_node` from converter.rs
/// which must be accessible (pub(crate)) for this module to work correctly.
pub fn handle(tag_name: &str, context: HandlerContext<'_>) {
    match tag_name {
        "strong" | "b" => handle_strong(context),
        "em" | "i" => handle_emphasis(context),
        _ => {}
    }
}

use crate::converter::inline::wrapped::{
    EMPHASIS_SIBLING_TAGS, InlineDelimiters, InlineSite, STRONG_SIBLING_TAGS, block_runs_are_plain,
    block_runs_are_single_line, emit_first_block_wrapped, emit_wrapped_inline, wrap_block_runs,
};

/// Resolve `<strong>`/`<b>`'s wrapping delimiters for the current context and options, then
/// emit via [`emit_wrapped_inline`].
pub fn emit_strong_wrapped(output: &mut String, content: &str, site: InlineSite<'_>) {
    let options = site.options;
    let ctx = site.ctx;
    let marker = if ctx.in_strong {
        String::new()
    } else if options.output_format == OutputFormat::Djot {
        String::from("*")
    } else {
        [options.strong_em_symbol; 2].iter().collect()
    };
    if emit_first_block_wrapped(output, content, &marker, &marker, ctx, site.parser) {
        return;
    }
    if !marker.is_empty() && content.contains("\n\n") && block_runs_are_plain(content) {
        output.push_str(&wrap_block_runs(content, &marker, &marker));
        return;
    }

    if ctx.in_strong {
        emit_wrapped_inline(
            output,
            content,
            &InlineDelimiters {
                open: "",
                close: "",
                merge_symbol: Some(options.strong_em_symbol),
                sibling_tag_names: &STRONG_SIBLING_TAGS,
            },
            InlineSite {
                node_handle: site.node_handle,
                parser: site.parser,
                dom_ctx: site.dom_ctx,
                ctx,
                options,
            },
        );
    } else if options.output_format == OutputFormat::Djot {
        // ~keep Djot strong always uses `*`, independent of `options.strong_em_symbol`
        // ~keep (pre-existing behaviour, unchanged by this refactor).
        emit_wrapped_inline(
            output,
            content,
            &InlineDelimiters {
                open: "*",
                close: "*",
                merge_symbol: Some('*'),
                sibling_tag_names: &STRONG_SIBLING_TAGS,
            },
            InlineSite {
                node_handle: site.node_handle,
                parser: site.parser,
                dom_ctx: site.dom_ctx,
                ctx,
                options,
            },
        );
    } else {
        emit_wrapped_inline(
            output,
            content,
            &InlineDelimiters {
                open: &marker,
                close: &marker,
                merge_symbol: Some(options.strong_em_symbol),
                sibling_tag_names: &STRONG_SIBLING_TAGS,
            },
            InlineSite {
                node_handle: site.node_handle,
                parser: site.parser,
                dom_ctx: site.dom_ctx,
                ctx,
                options,
            },
        );
    }
}

/// ~keep Legend semantically bolds every child block, including structured list runs (#724).
pub fn emit_strong_wrapped_blocks(output: &mut String, content: &str, site: InlineSite<'_>) {
    let options = site.options;
    let ctx = site.ctx;
    let marker = if ctx.in_strong {
        String::new()
    } else if options.output_format == OutputFormat::Djot {
        String::from("*")
    } else {
        [options.strong_em_symbol; 2].iter().collect()
    };
    if !marker.is_empty() && content.contains("\n\n") && block_runs_are_single_line(content) {
        output.push_str(&wrap_block_runs(content, &marker, &marker));
    } else {
        emit_strong_wrapped(output, content, site);
    }
}

/// Resolve `<em>`/`<i>`'s wrapping delimiters for the current context and options, then emit
/// via [`emit_wrapped_inline`].
fn emit_emphasis_wrapped(output: &mut String, content: &str, site: InlineSite<'_>) {
    let options = site.options;
    let marker = if options.output_format == OutputFormat::Djot {
        String::from("_")
    } else {
        options.strong_em_symbol.to_string()
    };
    if emit_first_block_wrapped(output, content, &marker, &marker, site.ctx, site.parser) {
        return;
    }
    if content.contains("\n\n") && block_runs_are_plain(content) {
        output.push_str(&wrap_block_runs(content, &marker, &marker));
        return;
    }

    if options.output_format == OutputFormat::Djot {
        // ~keep Djot emphasis always uses `_`, independent of `options.strong_em_symbol`
        // ~keep (pre-existing behaviour, unchanged by this refactor).
        emit_wrapped_inline(
            output,
            content,
            &InlineDelimiters {
                open: "_",
                close: "_",
                merge_symbol: Some('_'),
                sibling_tag_names: &EMPHASIS_SIBLING_TAGS,
            },
            InlineSite {
                node_handle: site.node_handle,
                parser: site.parser,
                dom_ctx: site.dom_ctx,
                ctx: site.ctx,
                options,
            },
        );
    } else {
        emit_wrapped_inline(
            output,
            content,
            &InlineDelimiters {
                open: &marker,
                close: &marker,
                merge_symbol: Some(options.strong_em_symbol),
                sibling_tag_names: &EMPHASIS_SIBLING_TAGS,
            },
            InlineSite {
                node_handle: site.node_handle,
                parser: site.parser,
                dom_ctx: site.dom_ctx,
                ctx: site.ctx,
                options,
            },
        );
    }
}

/// Handle strong/bold emphasis (strong, b tags).
fn handle_strong(handler: HandlerContext<'_>) {
    handle_emphasis_element(handler, EmphasisKind::Strong);
}

/// Handle emphasis/italic (em, i tags).
fn handle_emphasis(handler: HandlerContext<'_>) {
    handle_emphasis_element(handler, EmphasisKind::Emphasis);
}

#[derive(Clone, Copy)]
enum EmphasisKind {
    Strong,
    Emphasis,
}

fn handle_emphasis_element(mut handler: HandlerContext<'_>, kind: EmphasisKind) {
    let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
        return;
    };
    if handler.context.in_code {
        walk_children_to_output(tag, &mut handler);
        return;
    }

    let after_hard_break = if handler.output.is_empty() {
        handler.context.inline_buffer_after_hard_break
    } else {
        crate::converter::utility::escaping::ends_with_hard_break(handler.output)
    };
    let child_context = Context {
        inline_depth: handler.context.inline_depth + 1,
        in_strong: handler.context.in_strong || matches!(kind, EmphasisKind::Strong),
        inline_buffer_after_hard_break: after_hard_break,
        ..handler.context.clone()
    };
    let mut content = String::with_capacity(64);
    collect_children(tag, &mut content, &child_context, &handler);

    #[cfg(feature = "visitor")]
    if let Some(custom_output) = visit_emphasis(tag, kind, &handler) {
        handler.output.push_str(&custom_output);
        return;
    }

    let site = handler.inline_site();
    match kind {
        EmphasisKind::Strong => emit_strong_wrapped(handler.output, &content, site),
        EmphasisKind::Emphasis => {
            emit_emphasis_wrapped(handler.output, &content, site);
            maybe_emit_caret(handler.output, &content, tag);
        }
    }
}

#[cfg(feature = "visitor")]
fn visit_emphasis(tag: &tl::HTMLTag<'_>, kind: EmphasisKind, handler: &HandlerContext<'_>) -> Option<String> {
    use crate::converter::{get_text_content, serialize_node};
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let visitor_handle = handler.context.visitor.as_ref()?;
    let text_content = get_text_content(handler.node_handle, handler.parser, handler.dom_context);
    let node_id = handler.node_handle.get_inner();
    let node_context = NodeContext::with_lazy_attributes(
        match kind {
            EmphasisKind::Strong => NodeType::Strong,
            EmphasisKind::Emphasis => NodeType::Em,
        },
        tag.name().as_utf8_str(),
        tag,
        handler.depth,
        handler.dom_context.get_sibling_index(node_id).unwrap_or(0),
        handler
            .dom_context
            .parent_tag_name(node_id, handler.parser)
            .map(Cow::Borrowed),
        true,
    );
    let result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        match kind {
            EmphasisKind::Strong => visitor.visit_strong(&node_context, &text_content),
            EmphasisKind::Emphasis => visitor.visit_emphasis(&node_context, &text_content),
        }
    };
    if handler.options.whitespace_mode == crate::options::WhitespaceMode::Strict
        && (matches!(&result, VisitResult::Skip)
            || matches!(&result, VisitResult::Custom(custom) if custom.trim_matches([' ', '\t']).is_empty()))
    {
        handler.dom_context.record_absent_breaks(node_id, true);
    }
    match result {
        VisitResult::Continue => None,
        VisitResult::Custom(custom) => {
            crate::converter::structure_capture::replace_element(handler.context, Some(&custom));
            Some(custom)
        }
        VisitResult::Skip => {
            crate::converter::structure_capture::replace_element(handler.context, None);
            Some(String::new())
        }
        VisitResult::PreserveHtml => {
            let html = serialize_node(handler.node_handle, handler.parser);
            crate::converter::structure_capture::replace_element(handler.context, Some(&html));
            Some(html)
        }
        VisitResult::Error(error) => {
            crate::converter::structure_capture::replace_element(handler.context, None);
            if handler.context.visitor_error.borrow().is_none() {
                *handler.context.visitor_error.borrow_mut() = Some(error);
            }
            None
        }
    }
}

fn walk_children_to_output(tag: &tl::HTMLTag<'_>, handler: &mut HandlerContext<'_>) {
    for child_handle in tag.children().top().iter() {
        crate::converter::walk_node(
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

fn collect_children(tag: &tl::HTMLTag<'_>, output: &mut String, context: &Context, handler: &HandlerContext<'_>) {
    for child_handle in tag.children().top().iter() {
        crate::converter::walk_node(
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

/// Detect a Bootstrap `.caret` marker (`<i class="caret"></i>` and similar) on a genuinely
/// empty (not merely whitespace-only) `<em>`/`<i>` body and render it as `" > "`.
///
/// Only reachable when `content` is empty: [`emit_wrapped_inline`] already handles the
/// non-empty and whitespace-only-but-non-empty cases and leaves `output` untouched otherwise.
fn maybe_emit_caret(output: &mut String, content: &str, tag: &tl::HTMLTag) {
    if !content.is_empty() {
        return;
    }
    if let Some(class_value) = tag
        .attributes()
        .get("class")
        .and_then(|v| v.as_ref().map(|val| val.as_utf8_str()))
    {
        if class_value.contains("caret") && !output.ends_with(' ') {
            output.push_str(" > ");
        }
    }
}
