//! Handler for ruby annotation inline elements (ruby, rb, rt, rp, rtc).
//!
//! Converts HTML ruby annotation elements to Markdown format with support for:
//! - Ruby base text elements (<ruby>, <rb>)
//! - Ruby text annotations (<rt>) for phonetic guidance (common in CJK)
//! - Ruby parentheses (<rp>) for fallback presentation in browsers without ruby support
//! - Ruby text container (<rtc>) for secondary annotations or separate ruby text grouping
//! - Interleaved rendering mode: rb/rt pairs rendered inline (rb1(rt1)rb2(rt2))
//! - Grouped rendering mode: all rb text followed by rt annotations in parentheses
//! - Proper handling of CJK (Chinese/Japanese/Korean) text with multiple annotations
//! - Visitor callbacks for custom ruby processing
//! - Whitespace normalization and trimming

use crate::converter::inline::HandlerContext;
use crate::converter::walk::{CaptureEnd, Close, Open};
use std::rc::Rc;

type Context = crate::converter::Context;

/// Handles ruby annotation elements: ruby, rb, rt, rp, rtc.
///
/// Ruby annotations are used in East Asian typography to show pronunciation guides
/// or provide alternate text. The handler supports two rendering modes:
///
/// # Rendering Modes
///
/// **Interleaved mode** (when rb and rt elements are alternated without rtc):
/// - Renders ruby text inline with base text: `base(annotation)base(annotation)`
/// - Example: `<ruby><rb>漢</rb><rt>かん</rt></ruby>` → `漢(かん)`
///
/// **Grouped mode** (when rtc is present or rb/rt are not interleaved):
/// - Renders all base text first, then all annotations in parentheses: `base(annotation1annotation2)`
/// - Handles multiple rt elements and rtc (ruby text container) grouping
/// - Example: `<ruby><rb>東</rb><rb>京</rb><rt>とう</rt><rt>きょう</rt></ruby>` → `東京(とうきょう)`
///
/// # Element Handling
///
/// - `<ruby>`: Main container, detects layout and delegates to appropriate rendering mode
/// - `<rb>`: Base text; content is extracted and used in output
/// - `<rt>`: Annotation text; wrapped in parentheses in standalone contexts
/// - `<rp>`: Ruby parentheses (fallback for browsers without ruby support); skipped in most contexts
/// - `<rtc>`: Ruby text container for grouped annotations; content extracted after rt annotations
///
/// # Note
/// This function references `walk_node` and `normalized_tag_name` from converter.rs,
/// which must be accessible (pub(crate)) for this module to work correctly.
pub fn handle(mut handler: HandlerContext<'_>) {
    let Some(tl::Node::Tag(tag)) = handler.node_handle.get(handler.parser) else {
        return;
    };
    handle_ruby(tag, &mut handler);
}

/// Open a part of a ruby: `rb`, `rt`, `rp` or `rtc`. Lists the children in `children`.
///
/// The children of an `rtc` write to the buffer of the element. The walk captures the text of
/// the children of the other parts for [`close_text`] or [`close_annotation`].
pub fn open_part(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &str,
    context: &Context,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    let (opens_with_marker, end) = match tag_name {
        "rb" | "rp" => (false, CaptureEnd::RubyText),
        "rt" => (true, CaptureEnd::RubyAnnotation),
        _ => return Some(Open::children_only()),
    };
    Some(Open {
        scope: Some(Rc::new(context.inline_buffer(output, opens_with_marker))),
        close: Close::Capture(end),
    })
}

/// Write `text`, the text of the children of an `rb` or an `rp`, with no space around it.
pub fn close_text(text: &str, output: &mut String) {
    output.push_str(text.trim());
}

/// Write `text`, the text of the children of an `rt`, in parentheses.
pub fn close_annotation(text: &str, output: &mut String) {
    let trimmed = text.trim();
    if output.ends_with('(') {
        output.push_str(trimmed);
    } else {
        output.push('(');
        output.push_str(trimmed);
        output.push(')');
    }
}

fn handle_ruby(tag: &tl::HTMLTag<'_>, handler: &mut HandlerContext<'_>) {
    let ruby_context = handler.context.inline_buffer(handler.output, false);
    let tag_sequence = ruby_tag_sequence(tag, handler.parser);
    let has_rtc = tag_sequence.iter().any(|tag| tag == "rtc");
    let is_interleaved = tag_sequence.windows(2).any(|tags| tags[0] == "rb" && tags[1] == "rt");

    if is_interleaved && !has_rtc {
        render_interleaved(tag, &ruby_context, handler);
    } else {
        render_grouped(tag, &ruby_context, has_rtc, handler);
    }
}

fn ruby_tag_sequence(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> Vec<String> {
    tag.children()
        .top()
        .iter()
        .filter_map(|child_handle| {
            let tl::Node::Tag(child_tag) = child_handle.get(parser)? else {
                return None;
            };
            let tag_name = crate::converter::normalized_tag_name(child_tag.name().as_utf8_str());
            matches!(tag_name.as_ref(), "rb" | "rt" | "rtc").then(|| tag_name.into_owned())
        })
        .collect()
}

fn render_interleaved(tag: &tl::HTMLTag<'_>, ruby_context: &Context, handler: &mut HandlerContext<'_>) {
    let mut current_base = String::new();
    for child_handle in tag.children().top().iter() {
        render_interleaved_child(child_handle, &mut current_base, ruby_context, handler);
    }
    flush_base(&mut current_base, handler.output);
}

fn render_interleaved_child(
    child_handle: &tl::NodeHandle,
    current_base: &mut String,
    ruby_context: &Context,
    handler: &mut HandlerContext<'_>,
) {
    match child_handle.get(handler.parser) {
        Some(tl::Node::Tag(child_tag)) => {
            let tag_name = crate::converter::normalized_tag_name(child_tag.name().as_utf8_str());
            match tag_name.as_ref() {
                "rt" => {
                    let mut annotation = String::new();
                    walk_child(child_handle, &mut annotation, ruby_context, handler);
                    flush_base(current_base, handler.output);
                    handler.output.push_str(annotation.trim());
                }
                "rb" => {
                    flush_base(current_base, handler.output);
                    walk_child(child_handle, current_base, ruby_context, handler);
                }
                "rp" => {}
                _ => walk_child(child_handle, current_base, ruby_context, handler),
            }
        }
        Some(tl::Node::Raw(_)) => walk_child(child_handle, current_base, ruby_context, handler),
        _ => {}
    }
}

fn flush_base(current_base: &mut String, output: &mut String) {
    if !current_base.is_empty() {
        output.push_str(current_base.trim());
        current_base.clear();
    }
}

#[derive(Default)]
struct GroupedRuby {
    base_text: String,
    annotations: Vec<String>,
    rtc_content: String,
}

fn render_grouped(tag: &tl::HTMLTag<'_>, ruby_context: &Context, has_rtc: bool, handler: &mut HandlerContext<'_>) {
    let mut grouped = GroupedRuby::default();
    for child_handle in tag.children().top().iter() {
        collect_grouped_child(child_handle, ruby_context, &mut grouped, handler);
    }
    emit_grouped(grouped, has_rtc, handler.output);
}

fn collect_grouped_child(
    child_handle: &tl::NodeHandle,
    ruby_context: &Context,
    grouped: &mut GroupedRuby,
    handler: &HandlerContext<'_>,
) {
    match child_handle.get(handler.parser) {
        Some(tl::Node::Tag(child_tag)) => {
            let tag_name = crate::converter::normalized_tag_name(child_tag.name().as_utf8_str());
            match tag_name.as_ref() {
                "rt" => {
                    let mut annotation = String::new();
                    walk_child(child_handle, &mut annotation, ruby_context, handler);
                    grouped.annotations.push(annotation);
                }
                "rtc" => walk_child(child_handle, &mut grouped.rtc_content, ruby_context, handler),
                "rp" => {}
                _ => walk_child(child_handle, &mut grouped.base_text, ruby_context, handler),
            }
        }
        Some(tl::Node::Raw(_)) => walk_child(child_handle, &mut grouped.base_text, ruby_context, handler),
        _ => {}
    }
}

fn emit_grouped(grouped: GroupedRuby, has_rtc: bool, output: &mut String) {
    output.push_str(grouped.base_text.trim());
    let annotation = grouped.annotations.iter().map(|value| value.trim()).collect::<String>();
    if !annotation.is_empty() {
        if has_rtc && !grouped.rtc_content.trim().is_empty() && grouped.annotations.len() > 1 {
            output.push('(');
            output.push_str(&annotation);
            output.push(')');
        } else {
            output.push_str(&annotation);
        }
    }
    if !grouped.rtc_content.trim().is_empty() {
        output.push_str(grouped.rtc_content.trim());
    }
}

fn walk_child(child_handle: &tl::NodeHandle, output: &mut String, context: &Context, handler: &HandlerContext<'_>) {
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
