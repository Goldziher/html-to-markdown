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

use crate::converter::walk::{CaptureEnd, Child, ChildHook, Close, HookSite, Open};
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
/// Lists the children that write text in `children`. The walk visits each of them between two
/// calls of the hook that this function gives, and the hook writes the ruby.
pub fn open(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &str,
    context: &Context,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };
    let tag_sequence = ruby_tag_sequence(tag, parser);
    let has_rtc = tag_sequence.iter().any(|tag| tag == "rtc");
    let interleaved = !has_rtc && tag_sequence.windows(2).any(|tags| tags[0] == "rb" && tags[1] == "rt");
    let mut parts = Vec::new();
    for child_handle in tag.children().top().iter() {
        let part = match child_handle.get(parser) {
            Some(tl::Node::Tag(child_tag)) => {
                match crate::converter::normalized_tag_name(child_tag.name().as_utf8_str()).as_ref() {
                    "rt" => RubyPart::Annotation,
                    "rp" => continue,
                    "rb" if interleaved => RubyPart::NextBase,
                    "rtc" if !interleaved => RubyPart::Container,
                    _ => RubyPart::Base,
                }
            }
            Some(tl::Node::Raw(_)) => RubyPart::Base,
            _ => continue,
        };
        children.push(*child_handle);
        parts.push(part);
    }
    Some(Open {
        scope: None,
        close: Close::Hooked(Box::new(RubyHook {
            scope: Rc::new(context.inline_buffer(output, false)),
            parts,
            interleaved,
            has_rtc,
            grouped: GroupedRuby::default(),
        })),
    })
}

/// What a child of a ruby writes.
#[derive(Clone, Copy)]
enum RubyPart {
    /// Text of the base.
    Base,
    /// An `rb` of an interleaved ruby: the base before it is complete.
    NextBase,
    /// An `rt`.
    Annotation,
    /// An `rtc` of a grouped ruby.
    Container,
}

/// The work between the children of a ruby.
///
/// ~keep Each child writes to a buffer of the hook: the text of the base, one annotation, or the
/// ~keep text of the `rtc` elements. A child reads the end of the buffer that it writes to, so
/// ~keep the three texts cannot share one buffer.
struct RubyHook {
    /// The conversion state of the children.
    scope: Rc<Context>,
    /// What each listed child writes.
    parts: Vec<RubyPart>,
    interleaved: bool,
    has_rtc: bool,
    grouped: GroupedRuby,
}

impl ChildHook for RubyHook {
    fn before_child(&mut self, index: usize, element: &mut HookSite<'_>) -> Child {
        let buffer = match self.parts[index] {
            RubyPart::Annotation => String::new(),
            RubyPart::NextBase => {
                flush_base(&mut self.grouped.base_text, element.output);
                std::mem::take(&mut self.grouped.base_text)
            }
            RubyPart::Base => std::mem::take(&mut self.grouped.base_text),
            RubyPart::Container => std::mem::take(&mut self.grouped.rtc_content),
        };
        Child::Walk {
            depth: None,
            buffer: Some(buffer),
            scope: Some(Rc::clone(&self.scope)),
        }
    }

    fn after_child(&mut self, index: usize, own: Option<String>, element: &mut HookSite<'_>) {
        let text = own.unwrap_or_default();
        match self.parts[index] {
            RubyPart::Annotation if self.interleaved => {
                flush_base(&mut self.grouped.base_text, element.output);
                element.output.push_str(text.trim());
            }
            RubyPart::Annotation => self.grouped.annotations.push(text),
            RubyPart::NextBase | RubyPart::Base => self.grouped.base_text = text,
            RubyPart::Container => self.grouped.rtc_content = text,
        }
    }

    fn close(mut self: Box<Self>, element: &mut HookSite<'_>) {
        if self.interleaved {
            flush_base(&mut self.grouped.base_text, element.output);
        } else {
            emit_grouped(&self.grouped, self.has_rtc, element.output);
        }
    }
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

fn emit_grouped(grouped: &GroupedRuby, has_rtc: bool, output: &mut String) {
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
