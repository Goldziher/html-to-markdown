//! Handlers for typography and text semantic elements.
//!
//! Contains:
//! - Small text (pass through)
//! - Subscript and superscript with configurable symbols
//! - Variable (var) and definition (dfn) text with italic formatting
//! - Abbreviation (abbr) with optional title
//! - Span element with special OCR handling

use crate::converter::inline::{
    HandlerContext,
    wrapped::{EMPHASIS_SIBLING_TAGS, InlineDelimiters, emit_wrapped_inline},
};
use crate::options::{ConversionOptions, OutputFormat};
#[cfg(feature = "visitor")]
use std::borrow::Cow;
type Context = crate::converter::Context;

/// Handle small element.
///
/// Small text has no direct Markdown equivalent, so just pass through content.
pub fn handle_small(handler: HandlerContext<'_>) {
    use crate::converter::walk_node;

    let Some(node) = handler.node_handle.get(handler.parser) else {
        return;
    };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    let children = tag.children();
    for child_handle in children.top().iter() {
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

/// Resolve a `<sub>`/`<sup>` wrapping pair for the current output format and configured symbol.
///
/// An HTML-ish symbol such as `<sub>` closes with its matching end tag rather than a repeat of
/// itself; every other symbol, including the empty default, closes with the symbol as given. ~keep
fn resolve_script_delimiters(options: &ConversionOptions, symbol: &str, djot_marker: char) -> (String, String) {
    if options.output_format == OutputFormat::Djot {
        let marker = djot_marker.to_string();
        return (marker.clone(), marker);
    }
    if symbol.starts_with('<') && !symbol.starts_with("</") {
        return (symbol.to_owned(), symbol.replace('<', "</"));
    }
    (symbol.to_owned(), symbol.to_owned())
}

/// Handle subscript element (sub tag).
///
/// Wraps content with configurable subscript symbol from options.
pub fn handle_subscript(handler: HandlerContext<'_>) {
    handle_script(handler, ScriptKind::Subscript);
}

/// Handle superscript element (sup tag).
///
/// Wraps content with configurable superscript symbol from options.
pub fn handle_superscript(handler: HandlerContext<'_>) {
    handle_script(handler, ScriptKind::Superscript);
}

#[derive(Clone, Copy)]
enum ScriptKind {
    Subscript,
    Superscript,
}

fn handle_script(handler: HandlerContext<'_>, kind: ScriptKind) {
    let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
        return;
    };
    let symbol = match kind {
        ScriptKind::Subscript => &handler.options.sub_symbol,
        ScriptKind::Superscript => &handler.options.sup_symbol,
    };
    let djot_marker = match kind {
        ScriptKind::Subscript => '~',
        ScriptKind::Superscript => '^',
    };
    let (open, close) = resolve_script_delimiters(handler.options, symbol, djot_marker);
    let marker_context = handler.context.inline_buffer(handler.output, !open.is_empty());
    let mut content = String::with_capacity(32);
    collect_children(tag, &mut content, &marker_context, &handler);

    if handler.context.in_code {
        handler.output.push_str(&content);
        return;
    }

    #[cfg(feature = "visitor")]
    if let Some(custom_output) = visit_script(tag, kind, &handler) {
        handler.output.push_str(&custom_output);
        return;
    }

    let site = handler.inline_site();
    emit_wrapped_inline(
        handler.output,
        &content,
        &InlineDelimiters {
            open: &open,
            close: &close,
            merge_symbol: None,
            sibling_tag_names: &[],
        },
        site,
    );
}

#[cfg(feature = "visitor")]
fn visit_script(tag: &tl::HTMLTag<'_>, kind: ScriptKind, handler: &HandlerContext<'_>) -> Option<String> {
    use crate::converter::{get_text_content, serialize_node};
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let visitor_handle = handler.context.visitor.as_ref()?;
    let text_content = get_text_content(handler.node_handle, handler.parser, handler.dom_context);
    let node_id = handler.node_handle.get_inner();
    let node_context = NodeContext::with_lazy_attributes(
        match kind {
            ScriptKind::Subscript => NodeType::Subscript,
            ScriptKind::Superscript => NodeType::Superscript,
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
            ScriptKind::Subscript => visitor.visit_subscript(&node_context, &text_content),
            ScriptKind::Superscript => visitor.visit_superscript(&node_context, &text_content),
        }
    };
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

/// Handle variable element (var tag).
///
/// Wraps content with italic symbol (`strong_em_symbol` from options).
pub fn handle_variable(handler: HandlerContext<'_>) {
    handle_italic_semantic(handler);
}

/// Handle definition element (dfn tag).
///
/// Wraps content with italic symbol (`strong_em_symbol` from options).
pub fn handle_definition(handler: HandlerContext<'_>) {
    handle_italic_semantic(handler);
}

fn handle_italic_semantic(mut handler: HandlerContext<'_>) {
    let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
        return;
    };

    if handler.context.in_code {
        walk_children_to_output(tag, &mut handler);
        return;
    }

    let marker_context = handler.context.inline_buffer(handler.output, true);
    let mut content = String::with_capacity(32);
    collect_children(tag, &mut content, &marker_context, &handler);
    let marker = handler.options.strong_em_symbol.to_string();
    let site = handler.inline_site();
    emit_wrapped_inline(
        handler.output,
        &content,
        &InlineDelimiters {
            open: &marker,
            close: &marker,
            merge_symbol: Some(handler.options.strong_em_symbol),
            sibling_tag_names: &EMPHASIS_SIBLING_TAGS,
        },
        site,
    );
}

/// Handle abbreviation element (abbr tag).
///
/// Passes through content and optionally appends title attribute in parentheses.
pub fn handle_abbreviation(handler: HandlerContext<'_>) {
    use crate::converter::{append_inline_suffix, chomp_inline, walk_node};

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

    let mut content = String::with_capacity(32);
    let children = tag.children();
    let abbr_ctx = ctx.inline_buffer(output, false);
    for child_handle in children.top().iter() {
        walk_node(
            child_handle,
            parser,
            &mut content,
            crate::converter::block::container::HandlerContext::new(options, &abbr_ctx, depth + 1, dom_ctx),
        );
    }

    let (prefix, suffix, trimmed) = chomp_inline(&content);

    if trimmed.is_empty() {
        // ~keep issue #481: `<abbr>` renders no delimiters of its own, so a whitespace-only
        // ~keep body left nothing behind at all and joined the words either side of it.
        if !content.is_empty() && !output.ends_with(' ') {
            output.push_str(prefix);
        }
        return;
    }

    crate::converter::utility::content::push_inline_prefix(output, prefix, options, ctx);
    output.push_str(trimmed);

    if let Some(title) = crate::converter::utility::attributes::decoded_attribute(tag, "title") {
        let trimmed_title = title.trim();
        if !trimmed_title.is_empty() {
            output.push_str(" (");
            output.push_str(trimmed_title);
            output.push(')');
        }
    }
    append_inline_suffix(output, suffix, true, node_handle, parser, dom_ctx);
}

/// Handle span element.
///
/// Processes span elements with special handling for:
/// - OCR words (elements with class "`ocrx_word")`: adds space before if needed
/// - Otherwise passes through content normally
pub fn handle_span(handler: HandlerContext<'_>) {
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

    let is_hocr_word = tag.attributes().iter().any(|(name, value)| {
        name.as_ref() == "class" && value.as_ref().is_some_and(|v| v.as_ref().contains("ocrx_word"))
    });

    if is_hocr_word
        && !output.is_empty()
        && !output.ends_with(' ')
        && !output.ends_with('\t')
        && !output.ends_with('\n')
    {
        output.push(' ');
    }

    let children = tag.children();
    {
        for child_handle in children.top().iter() {
            walk_node(
                child_handle,
                parser,
                output,
                crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
            );
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

#[cfg(all(test, feature = "visitor"))]
mod tests {
    use crate::convert;
    use crate::options::ConversionOptions;
    use crate::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct SubSkipVisitor;

    impl HtmlVisitor for SubSkipVisitor {
        fn visit_subscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
            VisitResult::Skip
        }
    }

    #[derive(Debug)]
    struct SubCustomVisitor;

    impl HtmlVisitor for SubCustomVisitor {
        fn visit_subscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
            VisitResult::Custom("REPLACED".to_string())
        }
    }

    #[derive(Debug)]
    struct SubPreserveVisitor;

    impl HtmlVisitor for SubPreserveVisitor {
        fn visit_subscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
            VisitResult::PreserveHtml
        }
    }

    #[derive(Debug)]
    struct SupSkipVisitor;

    impl HtmlVisitor for SupSkipVisitor {
        fn visit_superscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
            VisitResult::Skip
        }
    }

    #[derive(Debug)]
    struct SupCustomVisitor;

    impl HtmlVisitor for SupCustomVisitor {
        fn visit_superscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
            VisitResult::Custom("REPLACED".to_string())
        }
    }

    #[derive(Debug)]
    struct SupPreserveVisitor;

    impl HtmlVisitor for SupPreserveVisitor {
        fn visit_superscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
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
    fn test_visitor_subscript_skip() {
        let html = "<p>H<sub>2</sub>O</p>";
        let result = convert(html, Some(make_visitor(SubSkipVisitor))).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(!content.contains('2'), "sub content should be absent: {content}");
        assert!(content.contains('H'), "surrounding text should be present: {content}");
    }

    #[test]
    fn test_visitor_subscript_custom() {
        let html = "<p>H<sub>2</sub>O</p>";
        let result = convert(html, Some(make_visitor(SubCustomVisitor))).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(
            content.contains("REPLACED"),
            "custom output should be present: {content}"
        );
    }

    #[test]
    fn test_visitor_subscript_preserve_html() {
        let html = "<p>H<sub>2</sub>O</p>";
        let result = convert(html, Some(make_visitor(SubPreserveVisitor))).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(
            content.contains("<sub>2</sub>"),
            "original html should be preserved: {content}"
        );
    }

    #[test]
    fn test_visitor_superscript_skip() {
        let html = "<p>E=mc<sup>2</sup></p>";
        let result = convert(html, Some(make_visitor(SupSkipVisitor))).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(!content.contains('2'), "sup content should be absent: {content}");
    }

    #[test]
    fn test_visitor_superscript_custom() {
        let html = "<p>E=mc<sup>2</sup></p>";
        let result = convert(html, Some(make_visitor(SupCustomVisitor))).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(
            content.contains("REPLACED"),
            "custom output should be present: {content}"
        );
    }

    #[test]
    fn test_visitor_superscript_preserve_html() {
        let html = "<p>E=mc<sup>2</sup></p>";
        let result = convert(html, Some(make_visitor(SupPreserveVisitor))).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(
            content.contains("<sup>2</sup>"),
            "original html should be preserved: {content}"
        );
    }
}
