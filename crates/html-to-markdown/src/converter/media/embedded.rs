//! Embedded media element handling (iframe, video, audio, source).
//!
//! Converts various embedded media elements:
//! - **iframe**: Embedded content frames, outputs src as a link
//! - **video**: Video elements with src or nested source elements
//! - **audio**: Audio elements with src or nested source elements
//! - **source**: Media source elements (handled within parent elements)
//! - **picture**: Picture elements with responsive image sources

use std::borrow::Cow;
use tl::{HTMLTag, NodeHandle, Parser};

use crate::converter::Context;
use crate::converter::inline::link::{MarkdownLink, append_markdown_link};
use crate::converter::main_helpers::tag_name_eq;
use crate::converter::media::MediaContext;
use crate::converter::media::first_address;
use crate::converter::utility::escaping::escape_link_label;
use crate::converter::utility::preprocessing::sanitize_markdown_url;
use crate::converter::walk::{CaptureEnd, Close, Open};
use crate::options::{ConversionOptions, InlineDataMedia};

/// ~keep Append a media element's `src` as a Markdown link where the destination doubles as
/// the label (`[src](src)`), routed through [`append_markdown_link`] so the destination gets
/// the same sanitization/escaping as `<a href>` (audit #24: `src` was previously spliced into
/// both the label and destination positions completely unescaped).
///
/// `raw_text` is passed as `""` rather than `src` to keep `append_markdown_link`'s
/// `default_title` branch (which fires when `raw_text == href`) from ever firing here — these
/// elements never rendered a title before this fix, and `src` is guaranteed non-empty by the
/// `should_output_media_link` guard at the call site, so `"" == src` is never true.
fn append_media_src_link(output: &mut String, src: &str, options: &ConversionOptions, ctx: &Context) {
    let escaped_label = escape_link_label(src);
    append_markdown_link(
        output,
        &MarkdownLink {
            label: &escaped_label,
            href: src,
            title: None,
            raw_text: "",
        },
        options,
        ctx.reference_collector.as_ref(),
    );
}

/// Extract src attribute from media element (audio, video, iframe).
pub fn extract_media_src<'a>(tag: &'a HTMLTag<'a>) -> Cow<'a, str> {
    crate::converter::utility::attributes::decoded_attribute(tag, "src").unwrap_or(Cow::Borrowed(""))
}

/// The address of an `<audio>` or `<video>`: its own `src`, else its first nested `<source>`'s.
///
/// With any choice but [`InlineDataMedia::Keep`], a `data:` address is passed over for the next
/// non-empty one among the element's `src` and every nested `<source>`'s.
fn media_element_src<'a>(tag: &'a HTMLTag<'a>, parser: &'a Parser, choice: InlineDataMedia) -> Cow<'a, str> {
    let own = extract_media_src(tag);
    let children = tag.children();
    let mut sources = children
        .top()
        .iter()
        .filter_map(|child_handle| match child_handle.get(parser) {
            Some(tl::Node::Tag(child_tag)) if is_source_element(child_tag) => Some(child_tag),
            _ => None,
        })
        .map(|source| crate::converter::utility::attributes::decoded_attribute(source, "src"));
    if choice == InlineDataMedia::Keep {
        return if own.is_empty() {
            sources.next().flatten().unwrap_or(Cow::Borrowed(""))
        } else {
            own
        };
    }
    let addresses = std::iter::once(own)
        .chain(sources.flatten())
        .filter(|address| !address.is_empty());
    first_address(choice, addresses).unwrap_or(Cow::Borrowed(""))
}

/// Check if tag is a source element.
pub fn is_source_element(tag: &HTMLTag) -> bool {
    tag_name_eq(tag.name().as_utf8_str(), "source")
}

/// Determine if media should output source link in markdown.
///
/// Returns true if src is non-empty.
pub const fn should_output_media_link(src: &str) -> bool {
    !src.is_empty()
}

/// Open an `audio` or a `video`: writes its address and lists its fallback content in `children`.
///
/// The walk captures the text of that content for [`close_media_fallback`]. Returns `None` for an
/// element that is complete.
pub fn open_media(
    tag_name: &str,
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    context: MediaContext<'_>,
    children: &mut Vec<NodeHandle>,
) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };
    if tag_name == "audio" {
        handle_audio(node_handle, tag, parser, output, context, children);
    } else {
        handle_video(node_handle, tag, parser, output, context, children);
    }
    (!children.is_empty()).then_some(Open {
        scope: None,
        close: Close::Capture(CaptureEnd::MediaFallback),
    })
}

/// List the fallback content of a media element: each child that is no `source`.
fn list_media_fallback(tag: &HTMLTag, parser: &Parser, children: &mut Vec<NodeHandle>) {
    children.extend(tag.children().top().iter().copied().filter(|child_handle| {
        !matches!(child_handle.get(parser), Some(tl::Node::Tag(child_tag)) if is_source_element(child_tag))
    }));
}

/// Write `fallback`, the text of the fallback content of a media element.
pub fn close_media_fallback(fallback: &str, output: &mut String, ctx: &Context) {
    if !fallback.is_empty() {
        output.push_str(fallback.trim());
        if !ctx.in_paragraph && !ctx.convert_as_inline {
            output.push_str("\n\n");
        }
    }
}

/// Handle audio element conversion to Markdown.
///
/// Extracts src from audio tag or nested source elements, outputs as a link,
/// and lists the fallback content (e.g., browser compatibility text) in `children`.
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
fn handle_audio(
    node_handle: &NodeHandle,
    tag: &HTMLTag,
    parser: &Parser,
    output: &mut String,
    context: MediaContext<'_>,
    children: &mut Vec<NodeHandle>,
) {
    let MediaContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = context;
    let raw_src = media_element_src(tag, parser, options.inline_data_media);
    let base_resolved_src = ctx.resolve_url(&raw_src, node_handle, parser, dom_ctx);
    let src = sanitize_markdown_url(base_resolved_src.as_deref().unwrap_or(&raw_src)).into_owned();
    let src_opt: Option<&str> = if src.is_empty() { None } else { Some(src.as_str()) };

    #[cfg(feature = "visitor")]
    if let Some(ref visitor_handle) = ctx.visitor {
        use crate::visitor::{NodeContext, NodeType, VisitResult};

        let node_id = node_handle.get_inner();
        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);
        let node_ctx = NodeContext::with_lazy_attributes(
            NodeType::Element,
            Cow::Borrowed("audio"),
            tag,
            depth,
            index_in_parent,
            parent_tag.map(Cow::Borrowed),
            false,
        );
        let visit_result = {
            let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
            visitor.visit_audio(&node_ctx, src_opt)
        };
        match visit_result {
            VisitResult::Continue => {}
            VisitResult::Skip => return,
            VisitResult::Custom(custom) => {
                output.push_str(&custom);
                if !ctx.in_paragraph && !ctx.convert_as_inline && !custom.ends_with('\n') {
                    output.push('\n');
                }
                return;
            }
            VisitResult::PreserveHtml => {
                use crate::converter::utility::serialization::serialize_node;
                output.push_str(&serialize_node(node_handle, parser));
                return;
            }
            VisitResult::Error(err) => {
                if ctx.visitor_error.borrow().is_none() {
                    *ctx.visitor_error.borrow_mut() = Some(err);
                }
                return;
            }
        }
    }

    let inline_data = ctx.inline_data_treatment(options.inline_data_media, &src);
    if inline_data == InlineDataMedia::DropElement {
        return;
    }
    if inline_data == InlineDataMedia::Keep && should_output_media_link(&src) {
        append_media_src_link(output, &src, options, ctx);
        if !ctx.in_paragraph && !ctx.convert_as_inline {
            output.push_str("\n\n");
        }
    }

    list_media_fallback(tag, parser, children);
}

/// Handle video element conversion to Markdown.
///
/// Extracts src from video tag or nested source elements, outputs as a link,
/// and lists the fallback content (e.g., browser compatibility text) in `children`.
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
fn handle_video(
    node_handle: &NodeHandle,
    tag: &HTMLTag,
    parser: &Parser,
    output: &mut String,
    context: MediaContext<'_>,
    children: &mut Vec<NodeHandle>,
) {
    let MediaContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = context;
    let raw_src = media_element_src(tag, parser, options.inline_data_media);
    let base_resolved_src = ctx.resolve_url(&raw_src, node_handle, parser, dom_ctx);
    let src = sanitize_markdown_url(base_resolved_src.as_deref().unwrap_or(&raw_src)).into_owned();
    let src_opt: Option<&str> = if src.is_empty() { None } else { Some(src.as_str()) };

    #[cfg(feature = "visitor")]
    if let Some(ref visitor_handle) = ctx.visitor {
        use crate::visitor::{NodeContext, NodeType, VisitResult};

        let node_id = node_handle.get_inner();
        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);
        let node_ctx = NodeContext::with_lazy_attributes(
            NodeType::Element,
            Cow::Borrowed("video"),
            tag,
            depth,
            index_in_parent,
            parent_tag.map(Cow::Borrowed),
            false,
        );
        let visit_result = {
            let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
            visitor.visit_video(&node_ctx, src_opt)
        };
        match visit_result {
            VisitResult::Continue => {}
            VisitResult::Skip => return,
            VisitResult::Custom(custom) => {
                output.push_str(&custom);
                if !ctx.in_paragraph && !ctx.convert_as_inline && !custom.ends_with('\n') {
                    output.push('\n');
                }
                return;
            }
            VisitResult::PreserveHtml => {
                use crate::converter::utility::serialization::serialize_node;
                output.push_str(&serialize_node(node_handle, parser));
                return;
            }
            VisitResult::Error(err) => {
                if ctx.visitor_error.borrow().is_none() {
                    *ctx.visitor_error.borrow_mut() = Some(err);
                }
                return;
            }
        }
    }

    let inline_data = ctx.inline_data_treatment(options.inline_data_media, &src);
    if inline_data == InlineDataMedia::DropElement {
        return;
    }
    if inline_data == InlineDataMedia::Keep && should_output_media_link(&src) {
        append_media_src_link(output, &src, options, ctx);
        if !ctx.in_paragraph && !ctx.convert_as_inline {
            output.push_str("\n\n");
        }
    }

    list_media_fallback(tag, parser, children);
}

/// Handle picture element conversion to Markdown.
///
/// Finds and processes the first child img element, skipping source elements.
///
/// Lists that child in `children` for the walk.
pub fn open_picture(tag: &HTMLTag, parser: &Parser, children: &mut Vec<NodeHandle>) -> crate::converter::walk::Open {
    for child_handle in tag.children().top().iter() {
        if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
            if tag_name_eq(child_tag.name().as_utf8_str(), "img") {
                children.push(*child_handle);
                break;
            }
        }
    }
    crate::converter::walk::Open::children_only()
}

/// Handle iframe element conversion to Markdown.
///
/// Extracts src attribute from iframe and outputs as a markdown link.
/// iframes cannot be embedded in markdown, so we just provide a link to the source.
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_iframe(
    node_handle: &NodeHandle,
    tag: &HTMLTag,
    parser: &Parser,
    output: &mut String,
    context: MediaContext<'_>,
) {
    let MediaContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = context;
    let raw_src = crate::converter::utility::attributes::decoded_attribute(tag, "src").unwrap_or(Cow::Borrowed(""));
    let base_resolved_src = ctx.resolve_url(&raw_src, node_handle, parser, dom_ctx);
    let src = sanitize_markdown_url(base_resolved_src.as_deref().unwrap_or(&raw_src)).into_owned();
    let src_opt: Option<&str> = if src.is_empty() { None } else { Some(src.as_str()) };

    #[cfg(feature = "visitor")]
    if let Some(ref visitor_handle) = ctx.visitor {
        use crate::visitor::{NodeContext, NodeType, VisitResult};

        let node_id = node_handle.get_inner();
        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);
        let node_ctx = NodeContext::with_lazy_attributes(
            NodeType::Element,
            Cow::Borrowed("iframe"),
            tag,
            depth,
            index_in_parent,
            parent_tag.map(Cow::Borrowed),
            false,
        );
        let visit_result = {
            let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
            visitor.visit_iframe(&node_ctx, src_opt)
        };
        match visit_result {
            VisitResult::Continue => {}
            VisitResult::Skip => return,
            VisitResult::Custom(custom) => {
                output.push_str(&custom);
                if !ctx.in_paragraph && !ctx.convert_as_inline && !custom.ends_with('\n') {
                    output.push('\n');
                }
                return;
            }
            VisitResult::PreserveHtml => {
                use crate::converter::utility::serialization::serialize_node;
                output.push_str(&serialize_node(node_handle, parser));
                return;
            }
            VisitResult::Error(err) => {
                if ctx.visitor_error.borrow().is_none() {
                    *ctx.visitor_error.borrow_mut() = Some(err);
                }
                return;
            }
        }
    }

    if ctx.inline_data_treatment(options.inline_data_media, &src) == InlineDataMedia::Keep && !src.is_empty() {
        append_media_src_link(output, &src, options, ctx);
        if !ctx.in_paragraph && !ctx.convert_as_inline {
            output.push_str("\n\n");
        }
    }
}

#[cfg(test)]
mod tests {
    // ~keep audit #24 finding 2: `src` was spliced into both the label and destination positions of
    // `[src](src)` completely unescaped, so a single inert `src` value containing `](` could
    // close the manufactured link early and open a second, attacker-controlled one.

    #[test]
    fn should_escape_a_src_that_would_close_the_iframe_link_and_open_a_new_one() {
        let html = "<iframe src='a](https://evil.example/payload)b'></iframe>";
        let result = crate::convert(html, None).unwrap();
        let content = result.content.unwrap_or_default();
        assert_eq!(
            content,
            "[a\\](https://evil.example/payload)b](a](https://evil.example/payload)b)\n"
        );
    }

    #[test]
    fn should_escape_a_src_that_would_close_the_audio_link_and_open_a_new_one() {
        let html = "<audio src='a](https://evil.example/payload)b'></audio>";
        let result = crate::convert(html, None).unwrap();
        let content = result.content.unwrap_or_default();
        assert_eq!(
            content,
            "[a\\](https://evil.example/payload)b](a](https://evil.example/payload)b)\n"
        );
    }

    #[test]
    fn should_escape_a_src_that_would_close_the_video_link_and_open_a_new_one() {
        let html = "<video src='a](https://evil.example/payload)b'></video>";
        let result = crate::convert(html, None).unwrap();
        let content = result.content.unwrap_or_default();
        assert_eq!(
            content,
            "[a\\](https://evil.example/payload)b](a](https://evil.example/payload)b)\n"
        );
    }
}
