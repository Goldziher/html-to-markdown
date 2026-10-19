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
use crate::converter::walk::{CaptureEnd, Close, Open};
use crate::options::{ConversionOptions, OutputFormat};
#[cfg(feature = "visitor")]
use std::borrow::Cow;
use std::rc::Rc;

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

#[derive(Clone, Copy)]
enum ScriptKind {
    Subscript,
    Superscript,
}

/// What `close_script` needs from `open_script`: the kind of the element and its marks.
pub struct ScriptEnd {
    kind: ScriptKind,
    open: String,
    close: String,
}

/// Open a subscript (`sub`) or a superscript (`sup`): the marks come from the options, and the
/// walk captures the text of the children for [`close_script`].
// ~keep In code the dispatcher gives the children to the walk and this handler does not run.
pub fn open_script(tag_name: &str, handler: &HandlerContext<'_>, children: &mut Vec<tl::NodeHandle>) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
        return None;
    };
    let (kind, symbol, djot_marker) = if tag_name == "sub" {
        (ScriptKind::Subscript, &handler.options.sub_symbol, '~')
    } else {
        (ScriptKind::Superscript, &handler.options.sup_symbol, '^')
    };
    let (open, close) = resolve_script_delimiters(handler.options, symbol, djot_marker);
    let marker_context = handler.context.inline_buffer(handler.output, !open.is_empty());
    children.extend(tag.children().top().iter().copied());
    Some(Open {
        scope: Some(Rc::new(marker_context)),
        close: Close::Capture(CaptureEnd::Script(ScriptEnd { kind, open, close })),
    })
}

/// Write the marks of a subscript or a superscript around `content`, the text of the children.
pub fn close_script(end: &ScriptEnd, content: &str, handler: HandlerContext<'_>) {
    #[cfg(feature = "visitor")]
    {
        let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
            return;
        };
        if let Some(custom_output) = visit_script(tag, end.kind, &handler) {
            handler.output.push_str(&custom_output);
            return;
        }
    }
    #[cfg(not(feature = "visitor"))]
    let _ = end.kind;

    let site = handler.inline_site();
    emit_wrapped_inline(
        handler.output,
        content,
        &InlineDelimiters {
            open: &end.open,
            close: &end.close,
            merge_symbol: None,
            sibling_tag_names: &[],
        },
        site,
    );
    // ~keep A captured block ends with a blank line. Inline chomping must not erase its
    // ~keep boundary and join the next sibling's word to the block's last word (#921).
    if content.ends_with("\n\n") && !handler.context.convert_as_inline && !handler.context.in_table_cell {
        handler.output.push_str("\n\n");
    }
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

/// Open a variable (`var`) or a definition (`dfn`), which write the italic symbol
/// (`strong_em_symbol` from the options) around their content.
///
/// In code the children write to the buffer of the element and no mark is written. Otherwise
/// the walk captures their text for [`close_italic_semantic`].
pub fn open_italic_semantic(handler: &HandlerContext<'_>, children: &mut Vec<tl::NodeHandle>) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    if handler.context.in_code {
        return Some(Open::children_only());
    }
    Some(Open {
        scope: Some(Rc::new(handler.context.inline_buffer(handler.output, true))),
        close: Close::Capture(CaptureEnd::ItalicSemantic),
    })
}

/// Write the italic symbol around `content`, the text of the children of a `var` or a `dfn`.
pub fn close_italic_semantic(content: &str, handler: HandlerContext<'_>) {
    let marker = handler.options.strong_em_symbol.to_string();
    let site = handler.inline_site();
    emit_wrapped_inline(
        handler.output,
        content,
        &InlineDelimiters {
            open: &marker,
            close: &marker,
            merge_symbol: Some(handler.options.strong_em_symbol),
            sibling_tag_names: &EMPHASIS_SIBLING_TAGS,
        },
        site,
    );
}

/// Open an abbreviation (`abbr`): the walk captures the text of the children for
/// [`close_abbreviation`].
pub fn open_abbreviation(handler: &HandlerContext<'_>, children: &mut Vec<tl::NodeHandle>) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    Some(Open {
        scope: Some(Rc::new(handler.context.inline_buffer(handler.output, false))),
        close: Close::Capture(CaptureEnd::Abbreviation),
    })
}

/// Write `content`, the text of the children of an abbreviation, and its title in parentheses.
pub fn close_abbreviation(content: &str, handler: HandlerContext<'_>) {
    use crate::converter::{append_inline_suffix, chomp_inline};

    let HandlerContext {
        node_handle,
        parser,
        output,
        options,
        context: ctx,
        dom_context: dom_ctx,
        ..
    } = handler;

    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return;
    };

    let (prefix, suffix, trimmed) = chomp_inline(content);

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

    if options.expand_abbreviations
        && let Some(title) = crate::converter::utility::attributes::decoded_attribute(tag, "title")
    {
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
///
/// Writes what a span writes before its children and lists the children in `children`.
pub fn open_span(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<crate::converter::walk::Open> {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
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

    children.extend(tag.children().top().iter().copied());
    Some(crate::converter::walk::Open::children_only())
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
