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
use crate::converter::walk::{CaptureEnd, Close, Open};
use crate::options::{ConversionOptions, OutputFormat};
#[cfg(feature = "visitor")]
use std::borrow::Cow;
use std::rc::Rc;
type Context = crate::converter::Context;

/// Handle mark (highlight) element with configurable styles.
///
/// Supports multiple highlight styles:
/// - `DoubleEqual`: `==highlighted==`
/// - Html: `<mark>highlighted</mark>`
/// - Bold: `**highlighted**`
/// - None: just pass through content
///
/// Asks the visitor, then lists the children in `children`. Returns `None` when the visitor
/// wrote or skipped the element.
pub fn open_mark(handler: HandlerContext<'_>, children: &mut Vec<tl::NodeHandle>) -> Option<Open> {
    use crate::options::HighlightStyle;
    let site = handler.inline_site();
    let HandlerContext {
        node_handle,
        parser,
        output,
        options,
        context: ctx,
        depth,
        ..
    } = handler;
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };

    #[cfg(feature = "visitor")]
    if let Some(outcome) = visit_semantic(tag, SemanticKind::Mark, depth, site) {
        match outcome {
            VisitorOutcome::Continue => {}
            VisitorOutcome::Output(custom_output) => {
                output.push_str(&custom_output);
                return None;
            }
            VisitorOutcome::Skip => return None,
        }
    }
    #[cfg(not(feature = "visitor"))]
    let _ = (depth, site);

    children.extend(tag.children().top().iter().copied());
    // ~keep Code shows every character as text: no highlight marks. The visitor is asked first.
    if ctx.convert_as_inline || ctx.in_code {
        return Some(Open::children_only());
    }

    // ~keep Bold highlighting renders its children with `in_strong` set, which suppresses a
    // ~keep nested `<strong>`'s own markers. Every style that writes markers renders them as
    // ~keep marker text (issue #603).
    let child_ctx = Context {
        in_strong: ctx.in_strong || options.highlight_style == HighlightStyle::Bold,
        ..match options.highlight_style {
            HighlightStyle::None => unmarked_buffer_context(tag, parser, ctx, output),
            _ => marked_buffer_context(ctx, output),
        }
    };
    Some(Open {
        scope: Some(Rc::new(child_ctx)),
        close: Close::Capture(CaptureEnd::Mark),
    })
}

/// Write the highlight marks around `content`, the text of the children of a mark element.
pub fn close_mark(content: &str, handler: HandlerContext<'_>) {
    use crate::options::HighlightStyle;
    let site = handler.inline_site();
    let HandlerContext { output, options, .. } = handler;

    let (open, close, merge_symbol) = resolve_mark_delimiters(options);
    // ~keep Markdown delimiters hoist edge whitespace, but raw HTML must preserve the source boundary exactly (#744).
    if options.highlight_style == HighlightStyle::Html {
        if !content.is_empty() {
            output.push_str(&open);
            output.push_str(content);
            output.push_str(&close);
        }
        return;
    }
    emit_wrapped_inline(
        output,
        content,
        &InlineDelimiters {
            open: &open,
            close: &close,
            merge_symbol,
            sibling_tag_names: &MARK_SIBLING_TAGS,
        },
        site,
    );
}

fn unmarked_buffer_context(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>, context: &Context, output: &str) -> Context {
    if has_only_blank_text_children(tag, parser) {
        context.clone()
    } else if has_only_raw_text_children(tag, parser) {
        context.inline_text_buffer(output)
    } else {
        context.inline_buffer(output, false)
    }
}

// ~keep Blank raw text returns before Markdown text-content processing, without reading inline line state.
// ~keep Tags and comments retain column-aware child context; parent hooks ran first.
fn has_only_blank_text_children(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> bool {
    tag.children().top().iter().all(|handle| {
        matches!(handle.get(parser), Some(tl::Node::Raw(raw))
            if crate::converter::utility::white_space::is_blank(raw.as_utf8_str().as_ref()))
    })
}

// ~keep Encoded tags and text-hook replacements stay text; only real child tags can consume list columns.
fn has_only_raw_text_children(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> bool {
    tag.children()
        .top()
        .iter()
        .all(|handle| matches!(handle.get(parser), Some(tl::Node::Raw(_))))
}

// ~keep A guaranteed opening marker makes the last-line column irrelevant to list rendering.
// ~keep Hard-break state remains live when children start in their detached buffer.
fn marked_buffer_context(context: &Context, output: &str) -> Context {
    Context {
        in_marker_span: true,
        inline_buffer_after_hard_break: if output.is_empty() {
            context.inline_buffer_after_hard_break
        } else {
            crate::converter::utility::escaping::ends_with_hard_break(output)
        },
        ..context.clone()
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
        output.push_str(content);
    }
}

/// Handle strikethrough element (del, s tags).
///
/// Converts to `~~content~~` syntax. Suppresses formatting in code context.
/// Supports visitor callbacks when the visitor feature is enabled.
///
/// Lists the children in `children`. In code they write to the buffer of the element and no mark
/// is written. Otherwise the walk captures their text for [`close_strikethrough`].
pub fn open_strikethrough(
    tag_name: &str,
    handler: &HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    let tag_name = match tag_name {
        "del" => "del",
        "s" => "s",
        _ => "strike",
    };
    open_marked(handler, children, CaptureEnd::Strikethrough(tag_name))
}

/// Handle inserted/underlined text (ins tag).
///
/// Converts to `==content==` syntax. Supports visitor callbacks when enabled.
///
/// Lists the children in `children`, as [`open_strikethrough`] does.
// ~keep In code `<ins>` writes no `==`, as `<del>` writes no `~~`: there the mark is text.
pub fn open_inserted(handler: HandlerContext<'_>, children: &mut Vec<tl::NodeHandle>) -> Option<Open> {
    if handler.options.output_format != OutputFormat::Djot && !handler.context.in_code {
        let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
            return None;
        };
        #[cfg(feature = "visitor")]
        if let Some(outcome) = visit_semantic(
            tag,
            SemanticKind::Underline("ins"),
            handler.depth,
            handler.inline_site(),
        ) {
            match outcome {
                VisitorOutcome::Continue => {}
                VisitorOutcome::Output(custom) => {
                    handler.output.push_str(&custom);
                    return None;
                }
                VisitorOutcome::Skip => return None,
            }
        }
        children.extend(tag.children().top().iter().copied());
        return Some(Open::children_only());
    }
    open_marked(&handler, children, CaptureEnd::Inserted)
}

fn open_marked(handler: &HandlerContext<'_>, children: &mut Vec<tl::NodeHandle>, end: CaptureEnd) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    if handler.context.in_code {
        return Some(Open::children_only());
    }
    Some(Open {
        scope: Some(Rc::new(marked_buffer_context(handler.context, handler.output))),
        close: Close::Capture(end),
    })
}

/// Write the strikethrough marks around `content`, the text of the children.
pub fn close_strikethrough(tag_name: &str, content: &str, handler: HandlerContext<'_>) {
    let site = handler.inline_site();
    #[cfg(not(feature = "visitor"))]
    let _ = tag_name;
    #[cfg(feature = "visitor")]
    {
        let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
            return;
        };
        let kind = SemanticKind::Strikethrough(tag_name);
        if let Some(outcome) = visit_semantic(tag, kind, handler.depth, site) {
            match outcome {
                VisitorOutcome::Continue => emit_strikethrough_wrapped(handler.output, content, site),
                VisitorOutcome::Output(custom_output) => handler.output.push_str(&custom_output),
                VisitorOutcome::Skip => {}
            }
            return;
        }
    }

    emit_strikethrough_wrapped(handler.output, content, site);
}

/// Write the marks of inserted text around `content`, the text of the children.
pub fn close_inserted(content: &str, handler: HandlerContext<'_>) {
    let site = handler.inline_site();
    #[cfg(feature = "visitor")]
    {
        let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
            return;
        };
        if let Some(outcome) = visit_semantic(tag, SemanticKind::Underline("ins"), handler.depth, site) {
            match outcome {
                VisitorOutcome::Continue => emit_inserted_wrapped(handler.output, content, site),
                VisitorOutcome::Output(custom_output) => handler.output.push_str(&custom_output),
                VisitorOutcome::Skip => {}
            }
            return;
        }
    }

    emit_inserted_wrapped(handler.output, content, site);
}

/// Handle underline element (u tag).
///
/// Just passes through content (HTML doesn't have native underline in Markdown).
/// Supports visitor callbacks when enabled, which can provide custom formatting.
///
/// Returns whether the visitor wrote or skipped the element, so that the walk does not open it.
#[cfg(feature = "visitor")]
pub fn visit_underline(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: crate::converter::block::container::HandlerContext<'_>,
) -> bool {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return false;
    };
    let site = InlineSite {
        node_handle,
        parser,
        dom_ctx: handler.dom_ctx,
        ctx: handler.ctx,
        options: handler.options,
    };
    match visit_semantic(tag, SemanticKind::Underline("u"), handler.depth, site) {
        None | Some(VisitorOutcome::Continue) => false,
        Some(VisitorOutcome::Output(custom_output)) => {
            output.push_str(&custom_output);
            true
        }
        Some(VisitorOutcome::Skip) => true,
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
    if site.options.whitespace_mode == crate::options::WhitespaceMode::Strict
        && (matches!(&result, VisitResult::Skip)
            || matches!(&result, VisitResult::Custom(custom) if custom.trim_matches([' ', '\t']).is_empty()))
    {
        site.dom_ctx.record_absent_breaks(node_id, true);
    }
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

#[cfg(test)]
mod line_state_cost_tests {
    use crate::converter::context::INLINE_BUFFER_COLUMN_CHAR_READS;
    use crate::{ConversionOptions, HighlightStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

    #[test]
    fn marked_wrappers_should_not_count_the_growing_output_column() {
        for (tag, style) in [
            ("del", HighlightStyle::DoubleEqual),
            ("s", HighlightStyle::DoubleEqual),
            ("strike", HighlightStyle::DoubleEqual),
            ("ins", HighlightStyle::DoubleEqual),
            ("mark", HighlightStyle::DoubleEqual),
            ("mark", HighlightStyle::Bold),
            ("mark", HighlightStyle::Html),
        ] {
            for n in [100, 400] {
                assert_eq!(column_reads(tag, style, n), 0, "{tag}, {style:?}, {n}");
            }
        }
    }

    #[test]
    fn unmarked_wrappers_after_text_should_not_count_an_unused_output_column() {
        for tag in ["mark", "abbr", "label", "select", "sub", "sup", "rb", "u", "span"] {
            let reads = [100, 400].map(|n| column_reads(tag, HighlightStyle::None, n));
            assert_eq!(reads, [0, 0], "{tag}: n=100 and n=400");
        }
    }

    #[test]
    fn marker_looking_text_tails_should_not_count_an_unused_output_column() {
        for text in ["x. ", "x) ", "x- ", "x* ", "x+ "] {
            for output_format in [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot] {
                for n in [100, 400] {
                    INLINE_BUFFER_COLUMN_CHAR_READS.with(|reads| reads.set(0));
                    let html = format!("<div>{}</div>", format!("<mark>{text}</mark>").repeat(n));
                    let options = ConversionOptions {
                        output_format,
                        highlight_style: HighlightStyle::None,
                        tier_strategy: TierStrategy::Tier2,
                        extract_metadata: false,
                        ..Default::default()
                    };
                    let content = convert(&html, options)
                        .expect("convert marker-looking tails")
                        .content
                        .expect("content");
                    assert_eq!(content, format!("{}\n", text.repeat(n).trim_end()));
                    assert_eq!(
                        INLINE_BUFFER_COLUMN_CHAR_READS.with(std::cell::Cell::get),
                        0,
                        "{text:?}, {output_format:?}, {n}"
                    );
                }
            }
        }
    }

    #[test]
    fn numeric_text_tails_should_bound_suffix_and_column_reads() {
        for text in ["x1. ", "x1) ", "雪1. ", "雪12) ", "x1234567890123456. ", "x1.\u{a0}"] {
            for output_format in [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot] {
                for n in [100, 400] {
                    INLINE_BUFFER_COLUMN_CHAR_READS.with(|reads| reads.set(0));
                    crate::converter::context::INLINE_BUFFER_TAIL_CHAR_READS.with(|reads| reads.set(0));
                    let options = ConversionOptions {
                        output_format,
                        highlight_style: HighlightStyle::None,
                        tier_strategy: TierStrategy::Tier2,
                        extract_metadata: false,
                        ..Default::default()
                    };
                    let content = convert(
                        &format!("<div>{}</div>", format!("<mark>{text}</mark>").repeat(n)),
                        options,
                    )
                    .expect("convert numeric tails")
                    .content
                    .expect("content");
                    assert_eq!(
                        content,
                        format!("{}\n", text.replace('\u{a0}', " ").repeat(n).trim_end())
                    );
                    let columns = INLINE_BUFFER_COLUMN_CHAR_READS.with(std::cell::Cell::get);
                    let suffix = crate::converter::context::INLINE_BUFFER_TAIL_CHAR_READS.with(std::cell::Cell::get);
                    assert_eq!(columns, 0, "{text:?}, {output_format:?}, {n}");
                    assert!(
                        suffix <= n * text.chars().count(),
                        "{text:?}, {output_format:?}, {n}: {suffix}"
                    );
                    assert_eq!(suffix, text.trim_end().chars().count() * (n - 1));
                }
            }
        }
    }

    #[test]
    fn empty_marks_should_not_scan_growing_blank_output() {
        for space in ['\u{2000}', '\u{a0}', ' ', '\t'] {
            for (output_format, whitespace_mode) in [
                (OutputFormat::Markdown, WhitespaceMode::Strict),
                (OutputFormat::Markdown, WhitespaceMode::Normalized),
                (OutputFormat::Plain, WhitespaceMode::Strict),
                (OutputFormat::Plain, WhitespaceMode::Normalized),
                (OutputFormat::Djot, WhitespaceMode::Strict),
                (OutputFormat::Djot, WhitespaceMode::Normalized),
            ] {
                for n in [100, 400] {
                    INLINE_BUFFER_COLUMN_CHAR_READS.with(|reads| reads.set(0));
                    crate::converter::context::INLINE_BUFFER_CONTEXT_CALLS.with(|calls| calls.set(0));
                    let options = ConversionOptions {
                        output_format,
                        whitespace_mode,
                        highlight_style: HighlightStyle::None,
                        tier_strategy: TierStrategy::Tier2,
                        extract_metadata: false,
                        ..Default::default()
                    };
                    let html = format!("<div>{}z</div>", format!("{space}<mark></mark>").repeat(n));
                    let content = convert(&html, options)
                        .expect("convert empty marks")
                        .content
                        .expect("content");
                    let expected_space = if output_format == OutputFormat::Plain {
                        ' '
                    } else {
                        space
                    };
                    let expected = if output_format == OutputFormat::Plain || whitespace_mode == WhitespaceMode::Strict
                    {
                        format!("{}z\n", expected_space.to_string().repeat(n))
                    } else {
                        "z\n".to_string()
                    };
                    assert_eq!(
                        content, expected,
                        "{space:?}, {output_format:?}, {whitespace_mode:?}, {n}"
                    );
                    assert_eq!(INLINE_BUFFER_COLUMN_CHAR_READS.with(std::cell::Cell::get), 0);
                    assert_eq!(
                        crate::converter::context::INLINE_BUFFER_CONTEXT_CALLS.with(std::cell::Cell::get),
                        0
                    );
                }
            }
        }
    }

    #[test]
    fn nonempty_marks_should_keep_live_columns_for_nested_children() {
        for n in [100, 400] {
            INLINE_BUFFER_COLUMN_CHAR_READS.with(|reads| reads.set(0));
            let html = format!("<div>{}z</div>", "\u{2000}<mark><span></span></mark>".repeat(n));
            let options = ConversionOptions {
                whitespace_mode: WhitespaceMode::Strict,
                highlight_style: HighlightStyle::None,
                tier_strategy: TierStrategy::Tier2,
                extract_metadata: false,
                ..Default::default()
            };
            assert_eq!(
                convert(&html, options)
                    .expect("convert nested empty span")
                    .content
                    .expect("content"),
                format!("{}z\n", "\u{2000}".repeat(n))
            );
            assert_eq!(
                INLINE_BUFFER_COLUMN_CHAR_READS.with(std::cell::Cell::get),
                n * (n + 1) / 2
            );
        }
    }

    #[test]
    fn empty_marks_should_preserve_styles_tags_wrapping_and_structure() {
        for output_format in [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot] {
            for highlight_style in [
                HighlightStyle::None,
                HighlightStyle::DoubleEqual,
                HighlightStyle::Bold,
                HighlightStyle::Html,
            ] {
                for preserved in [false, true] {
                    let options = ConversionOptions {
                        output_format,
                        highlight_style,
                        whitespace_mode: WhitespaceMode::Strict,
                        wrap: true,
                        wrap_width: 40,
                        include_document_structure: true,
                        extract_metadata: true,
                        preserve_tags: if preserved { vec!["mark".to_string()] } else { vec![] },
                        tier_strategy: TierStrategy::Tier2,
                        ..Default::default()
                    };
                    let html = "<html><head><title>T</title></head><body><div>\u{2000}<mark></mark>\u{2000}<mark></mark>z</div></body></html>";
                    let result = convert(html, options).expect("convert empty mark options");
                    let gap = if output_format == OutputFormat::Plain {
                        " "
                    } else {
                        "\u{2000}"
                    };
                    let markup = if preserved { "<mark></mark>" } else { "" };
                    let expected = if output_format == OutputFormat::Plain {
                        "  z\n\n".to_string()
                    } else {
                        let head = "---\ntitle: T\n---\n\n";
                        format!("{head}{gap}{markup}{gap}{markup}z\n\n")
                    };
                    assert_eq!(result.content.as_deref(), Some(expected.as_str()));
                    assert_eq!(
                        result.document,
                        Some(crate::DocumentStructure {
                            nodes: vec![],
                            source_format: Some("html".to_string())
                        })
                    );
                    #[cfg(feature = "metadata")]
                    {
                        let expected_metadata = crate::metadata::HtmlMetadata {
                            document: crate::metadata::DocumentMetadata {
                                title: Some("T".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        assert_eq!(
                            serde_json::to_value(&result.metadata).expect("metadata value"),
                            serde_json::to_value(expected_metadata).expect("expected metadata value")
                        );
                    }
                }
            }
        }
    }

    #[cfg(feature = "visitor")]
    #[test]
    fn empty_marks_should_keep_visitor_callbacks_before_context_selection() {
        use crate::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
        use std::sync::{Arc, Mutex};
        #[derive(Debug)]
        struct Recorder {
            answer: VisitResult,
            calls: usize,
        }
        impl HtmlVisitor for Recorder {
            fn visit_mark(&mut self, _ctx: &NodeContext<'_>, text: &str) -> VisitResult {
                assert_eq!(text, "");
                self.calls += 1;
                self.answer.clone()
            }
        }
        for (answer, markup) in [
            (VisitResult::Continue, ""),
            (VisitResult::Skip, ""),
            (VisitResult::Custom("C".to_string()), "C"),
            (VisitResult::PreserveHtml, "<mark />"),
        ] {
            for output_format in [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot] {
                let recorder = Arc::new(Mutex::new(Recorder {
                    answer: answer.clone(),
                    calls: 0,
                }));
                let handle: VisitorHandle = recorder.clone();
                let options = ConversionOptions {
                    output_format,
                    whitespace_mode: WhitespaceMode::Strict,
                    highlight_style: HighlightStyle::None,
                    tier_strategy: TierStrategy::Tier2,
                    extract_metadata: false,
                    visitor: Some(handle),
                    ..Default::default()
                };
                let result = convert("<div>\u{2000}<mark></mark>\u{2000}<mark></mark>z</div>", options)
                    .expect("convert visited empty marks");
                let expected = if output_format == OutputFormat::Plain {
                    "  z\n".to_string()
                } else {
                    format!("\u{2000}{markup}\u{2000}{markup}z\n")
                };
                assert_eq!(result.content.as_deref(), Some(expected.as_str()));
                assert_eq!(recorder.lock().expect("visitor recorder").calls, 2);
            }
        }
    }

    fn column_reads(tag: &str, highlight_style: HighlightStyle, n: usize) -> usize {
        INLINE_BUFFER_COLUMN_CHAR_READS.with(|reads| reads.set(0));
        let html = format!("<div>{}</div>", format!("<{tag}>x</{tag}>").repeat(n));
        let options = ConversionOptions {
            output_format: if tag == "ins" {
                OutputFormat::Djot
            } else {
                OutputFormat::Markdown
            },
            highlight_style,
            tier_strategy: TierStrategy::Tier2,
            extract_metadata: false,
            ..Default::default()
        };
        let content = convert(&html, options)
            .expect("convert marked wrappers")
            .content
            .expect("content");
        let expected = match (tag, highlight_style) {
            ("ins", _) => format!("{}\n", "{+x+}".repeat(n)),
            (_, HighlightStyle::Html) => format!("{}\n", "<mark>x</mark>".repeat(n)),
            ("select", _) => format!("{}\n", vec!["x"; n].join(" ")),
            (_, HighlightStyle::None) => format!("{}\n", "x".repeat(n)),
            _ => {
                let marker = if tag == "del" || tag == "s" || tag == "strike" {
                    "~~"
                } else if highlight_style == HighlightStyle::Bold {
                    "**"
                } else {
                    "=="
                };
                format!("{marker}{}{marker}\n", "x".repeat(n))
            }
        };
        assert_eq!(content, expected);
        INLINE_BUFFER_COLUMN_CHAR_READS.with(std::cell::Cell::get)
    }
}

#[cfg(test)]
#[path = "blank_text_cost_tests.rs"]
mod blank_text_cost_tests;

#[cfg(test)]
#[path = "raw_text_column_cost_tests.rs"]
mod raw_text_column_cost_tests;
