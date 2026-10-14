//! Handlers for mark/highlight and strikethrough/underline elements.
//!
//! Contains:
//! - Mark (highlight) element with configurable styles
//! - Strikethrough (del, s tags) with ~~ syntax
//! - Inserted/underlined text (ins, u tags) with == syntax

use crate::converter::inline::{
    HandlerContext,
    wrapped::{InlineDelimiters, InlineSite, emit_first_block_wrapped, emit_wrapped_inline},
};
use crate::options::{ConversionOptions, OutputFormat};
#[cfg(feature = "visitor")]
use std::borrow::Cow;
type Context = crate::converter::Context;

/// Handle mark (highlight) element with configurable styles.
///
/// Supports multiple highlight styles:
/// - `DoubleEqual`: `==highlighted==`
/// - Html: `<mark>highlighted</mark>`
/// - Bold: `**highlighted**`
/// - None: just pass through content
pub fn handle_mark(handler: HandlerContext<'_>) {
    // ~keep reason: serialize_node is only used when the visitor feature is active;
    // ~keep walk_node depends on feature-gated code paths.
    #[allow(unused_imports)]
    use crate::converter::{get_text_content, serialize_node, walk_node};

    let HandlerContext {
        node_handle,
        parser,
        output,
        options,
        context: ctx,
        depth,
        dom_context: dom_ctx,
    } = handler;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };
    let site = InlineSite {
        node_handle,
        parser,
        dom_ctx,
        ctx,
        options,
    };

    #[cfg(feature = "visitor")]
    if let Some(outcome) = visit_semantic(tag, SemanticKind::Mark, depth, site) {
        match outcome {
            VisitorOutcome::Continue => {}
            VisitorOutcome::Output(custom_output) => {
                output.push_str(&custom_output);
                return;
            }
            VisitorOutcome::Skip => return,
        }
    }

    if ctx.convert_as_inline || ctx.in_code {
        walk_children(tag, output, ctx, depth, site);
        return;
    }

    use crate::options::HighlightStyle;
    // ~keep Bold highlighting renders its children with `in_strong` set, which suppresses a
    // ~keep nested `<strong>`'s own markers. Every style that writes markers renders them as
    // ~keep marker text (issue #603).
    let child_ctx = Context {
        in_strong: ctx.in_strong || options.highlight_style == HighlightStyle::Bold,
        ..ctx.inline_buffer(output, options.highlight_style != HighlightStyle::None)
    };
    let mut content = String::with_capacity(32);
    walk_children(tag, &mut content, &child_ctx, depth, site);

    let (open, close, merge_symbol) = resolve_mark_delimiters(options);
    // ~keep Markdown delimiters hoist edge whitespace, but raw HTML must preserve the source boundary exactly (#744).
    if options.highlight_style == HighlightStyle::Html {
        if !content.is_empty() {
            output.push_str(&open);
            output.push_str(&content);
            output.push_str(&close);
        }
        return;
    }
    emit_wrapped_inline(
        output,
        &content,
        &InlineDelimiters {
            open: &open,
            close: &close,
            merge_symbol,
            sibling_tag_names: &MARK_SIBLING_TAGS,
        },
        site,
    );
}

fn walk_children(tag: &tl::HTMLTag<'_>, output: &mut String, context: &Context, depth: usize, site: InlineSite<'_>) {
    for child_handle in tag.children().top().iter() {
        crate::converter::walk_node(
            child_handle,
            site.parser,
            output,
            crate::converter::block::container::HandlerContext::new(site.options, context, depth + 1, site.dom_ctx),
        );
    }
}

/// Resolve `<mark>`'s wrapping pair, and the character it can merge a delimiter run with.
///
/// Only the repeated-character styles can form a run with an adjacent sibling's close marker;
/// `<mark>`'s HTML style closes with a distinct end tag and `None` emits no marker at all, so
/// both pass `None` and never merge. ~keep
fn resolve_mark_delimiters(options: &ConversionOptions) -> (String, String, Option<char>) {
    use crate::options::HighlightStyle;
    match options.highlight_style {
        HighlightStyle::DoubleEqual => {
            if options.output_format == OutputFormat::Djot {
                ("{=".to_owned(), "=}".to_owned(), None)
            } else {
                ("==".to_owned(), "==".to_owned(), Some('='))
            }
        }
        HighlightStyle::Html => ("<mark>".to_owned(), "</mark>".to_owned(), None),
        HighlightStyle::Bold => {
            let marker: String = [options.strong_em_symbol; 2].iter().collect();
            (marker.clone(), marker, Some(options.strong_em_symbol))
        }
        HighlightStyle::None => (String::new(), String::new(), None),
    }
}

/// Tag names whose rendered `~~` pair is the same delimiter run, for adjacency merging.
///
/// `<strike>` is a deprecated synonym of `<s>` and renders identically, so a `<del>` closing
/// straight into a `<strike>` forms one run just as two `<del>`s would. ~keep
const STRIKETHROUGH_SIBLING_TAGS: [&str; 3] = ["del", "s", "strike"];

/// Tag names whose rendered `==` pair is the same delimiter run, for adjacency merging.
const INSERTED_SIBLING_TAGS: [&str; 1] = ["ins"];

/// Tag names whose rendered highlight pair is the same delimiter run, for adjacency merging.
const MARK_SIBLING_TAGS: [&str; 1] = ["mark"];

/// Resolve `<del>`/`<s>`/`<strike>`'s wrapping delimiters for the current output format,
/// then emit via [`emit_wrapped_inline`].
fn emit_strikethrough_wrapped(output: &mut String, content: &str, site: InlineSite<'_>) {
    let options = site.options;
    let (open, close, merge_symbol) = if options.output_format == OutputFormat::Djot {
        ("{-", "-}", None)
    } else {
        ("~~", "~~", Some('~'))
    };
    if emit_first_block_wrapped(output, content, open, close, site.ctx, site.parser) {
        return;
    }
    emit_wrapped_inline(
        output,
        content,
        &InlineDelimiters {
            open,
            close,
            merge_symbol,
            sibling_tag_names: &STRIKETHROUGH_SIBLING_TAGS,
        },
        site,
    );
}

/// Resolve `<ins>`'s wrapping delimiters for the current output format, then emit via
/// [`emit_wrapped_inline`].
fn emit_inserted_wrapped(output: &mut String, content: &str, site: InlineSite<'_>) {
    let options = site.options;
    if options.output_format == OutputFormat::Djot {
        emit_wrapped_inline(
            output,
            content,
            &InlineDelimiters {
                open: "{+",
                close: "+}",
                merge_symbol: None,
                sibling_tag_names: &INSERTED_SIBLING_TAGS,
            },
            site,
        );
    } else {
        emit_wrapped_inline(
            output,
            content,
            &InlineDelimiters {
                open: "==",
                close: "==",
                merge_symbol: Some('='),
                sibling_tag_names: &INSERTED_SIBLING_TAGS,
            },
            site,
        );
    }
}

/// Handle strikethrough element (del, s tags).
///
/// Converts to `~~content~~` syntax. Suppresses formatting in code context.
/// Supports visitor callbacks when the visitor feature is enabled.
// ~keep reason: some parameters are only used when the visitor feature is active;
// ~keep using cfg_attr here is equivalent but more verbose given the function signature.
#[allow(unused_variables)]
pub fn handle_strikethrough(tag_name: &str, handler: HandlerContext<'_>) {
    use crate::converter::walk_node;

    let site = handler.inline_site();
    let HandlerContext {
        node_handle,
        parser,
        output,
        options,
        context: ctx,
        depth,
        dom_context: dom_ctx,
    } = handler;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    if ctx.in_code {
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(
                child_handle,
                parser,
                output,
                crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
            );
        }
    } else {
        let mut content = String::with_capacity(32);
        let children = tag.children();
        let marker_ctx = ctx.inline_buffer(output, true);
        for child_handle in children.top().iter() {
            walk_node(
                child_handle,
                parser,
                &mut content,
                crate::converter::block::container::HandlerContext::new(options, &marker_ctx, depth + 1, dom_ctx),
            );
        }

        #[cfg(feature = "visitor")]
        if let Some(outcome) = visit_semantic(tag, SemanticKind::Strikethrough(tag_name), depth, site) {
            match outcome {
                VisitorOutcome::Continue => emit_strikethrough_wrapped(output, &content, site),
                VisitorOutcome::Output(custom_output) => output.push_str(&custom_output),
                VisitorOutcome::Skip => {}
            }
            return;
        }

        emit_strikethrough_wrapped(output, &content, site);
    }
}

/// Handle inserted/underlined text (ins tag).
///
/// Converts to `==content==` syntax. Supports visitor callbacks when enabled.
pub fn handle_inserted(handler: HandlerContext<'_>) {
    use crate::converter::walk_node;

    let site = handler.inline_site();
    let HandlerContext {
        node_handle,
        parser,
        output,
        options,
        context: ctx,
        depth,
        dom_context: dom_ctx,
    } = handler;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    if ctx.in_code {
        // ~keep `handle_strikethrough` already suppresses its `~~` under `in_code`; `<ins>`
        // ~keep did not, so it emitted literal `==` INTO code content, where it is text.
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(
                child_handle,
                parser,
                output,
                crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
            );
        }
        return;
    }

    let mut content = String::with_capacity(32);
    let children = tag.children();
    let marker_ctx = ctx.inline_buffer(output, true);
    for child_handle in children.top().iter() {
        walk_node(
            child_handle,
            parser,
            &mut content,
            crate::converter::block::container::HandlerContext::new(options, &marker_ctx, depth + 1, dom_ctx),
        );
    }

    #[cfg(feature = "visitor")]
    if let Some(outcome) = visit_semantic(tag, SemanticKind::Underline("ins"), depth, site) {
        match outcome {
            VisitorOutcome::Continue => emit_inserted_wrapped(output, &content, site),
            VisitorOutcome::Output(custom_output) => output.push_str(&custom_output),
            VisitorOutcome::Skip => {}
        }
        return;
    }

    emit_inserted_wrapped(output, &content, site);
}

/// Handle underline element (u tag).
///
/// Just passes through content (HTML doesn't have native underline in Markdown).
/// Supports visitor callbacks when enabled, which can provide custom formatting.
pub fn handle_underline(handler: HandlerContext<'_>) {
    use crate::converter::walk_node;

    let HandlerContext {
        node_handle,
        parser,
        output,
        options,
        context: ctx,
        depth,
        dom_context: dom_ctx,
    } = handler;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    #[cfg(feature = "visitor")]
    if let Some(outcome) = visit_semantic(
        tag,
        SemanticKind::Underline("u"),
        depth,
        InlineSite {
            node_handle,
            parser,
            dom_ctx,
            ctx,
            options,
        },
    ) {
        match outcome {
            VisitorOutcome::Continue => {}
            VisitorOutcome::Output(custom_output) => {
                output.push_str(&custom_output);
                return;
            }
            VisitorOutcome::Skip => return,
        }
    }

    for child_handle in tag.children().top().iter() {
        walk_node(
            child_handle,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
        );
    }
}

#[cfg(feature = "visitor")]
#[derive(Clone, Copy)]
enum SemanticKind<'a> {
    Mark,
    Strikethrough(&'a str),
    Underline(&'a str),
}

#[cfg(feature = "visitor")]
enum VisitorOutcome {
    Continue,
    Output(String),
    Skip,
}

#[cfg(feature = "visitor")]
fn visit_semantic(
    tag: &tl::HTMLTag<'_>,
    kind: SemanticKind<'_>,
    depth: usize,
    site: InlineSite<'_>,
) -> Option<VisitorOutcome> {
    use crate::converter::{get_text_content, serialize_node};
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let visitor_handle = site.ctx.visitor.as_ref()?;
    let text_content = get_text_content(site.node_handle, site.parser, site.dom_ctx);
    let node_id = site.node_handle.get_inner();
    let (node_type, tag_name) = match kind {
        SemanticKind::Mark => (NodeType::Mark, tag.name().as_utf8_str()),
        SemanticKind::Strikethrough(tag_name) => (NodeType::Strikethrough, Cow::Borrowed(tag_name)),
        SemanticKind::Underline(tag_name) => (NodeType::Underline, Cow::Borrowed(tag_name)),
    };
    let node_context = NodeContext::with_lazy_attributes(
        node_type,
        tag_name,
        tag,
        depth,
        site.dom_ctx.get_sibling_index(node_id).unwrap_or(0),
        site.dom_ctx.parent_tag_name(node_id, site.parser).map(Cow::Borrowed),
        true,
    );
    let result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        match kind {
            SemanticKind::Mark => visitor.visit_mark(&node_context, &text_content),
            SemanticKind::Strikethrough(_) => visitor.visit_strikethrough(&node_context, &text_content),
            SemanticKind::Underline(_) => visitor.visit_underline(&node_context, &text_content),
        }
    };
    Some(match result {
        VisitResult::Continue => VisitorOutcome::Continue,
        VisitResult::Custom(custom) => {
            crate::converter::structure_capture::replace_element(site.ctx, Some(&custom));
            VisitorOutcome::Output(custom)
        }
        VisitResult::Skip => {
            crate::converter::structure_capture::replace_element(site.ctx, None);
            VisitorOutcome::Skip
        }
        VisitResult::PreserveHtml => {
            let html = serialize_node(site.node_handle, site.parser);
            crate::converter::structure_capture::replace_element(site.ctx, Some(&html));
            VisitorOutcome::Output(html)
        }
        VisitResult::Error(error) => {
            crate::converter::structure_capture::replace_element(site.ctx, None);
            if site.ctx.visitor_error.borrow().is_none() {
                *site.ctx.visitor_error.borrow_mut() = Some(error);
            }
            VisitorOutcome::Continue
        }
    })
}

#[cfg(all(test, feature = "visitor"))]
mod tests {
    use crate::convert;
    use crate::options::ConversionOptions;
    use crate::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct MarkSkipVisitor;

    impl HtmlVisitor for MarkSkipVisitor {
        fn visit_mark(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
            VisitResult::Skip
        }
    }

    #[derive(Debug)]
    struct MarkCustomVisitor;

    impl HtmlVisitor for MarkCustomVisitor {
        fn visit_mark(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
            VisitResult::Custom("REPLACED".to_string())
        }
    }

    #[derive(Debug)]
    struct MarkPreserveVisitor;

    impl HtmlVisitor for MarkPreserveVisitor {
        fn visit_mark(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
            VisitResult::PreserveHtml
        }
    }

    fn make_visitor<V: HtmlVisitor + 'static>(v: V) -> ConversionOptions {
        ConversionOptions {
            visitor: Some(Arc::new(Mutex::new(v))),
            ..ConversionOptions::default()
        }
    }

    #[test]
    fn test_visitor_mark_skip() {
        let html = "<p>before <mark>highlighted</mark> after</p>";
        let result = convert(html, Some(make_visitor(MarkSkipVisitor))).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(
            !content.contains("highlighted"),
            "mark content should be absent: {content}"
        );
        assert!(
            content.contains("before"),
            "surrounding text should be present: {content}"
        );
    }

    #[test]
    fn test_visitor_mark_custom() {
        let html = "<p>before <mark>highlighted</mark> after</p>";
        let result = convert(html, Some(make_visitor(MarkCustomVisitor))).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(
            content.contains("REPLACED"),
            "custom output should be present: {content}"
        );
    }

    #[test]
    fn test_visitor_mark_preserve_html() {
        let html = "<p>before <mark>highlighted</mark> after</p>";
        let result = convert(html, Some(make_visitor(MarkPreserveVisitor))).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(
            content.contains("<mark>highlighted</mark>"),
            "original html should be preserved: {content}"
        );
    }
}
