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

use crate::options::ConversionOptions;
use tl::{NodeHandle, Parser};

// Type aliases for Context and DomContext to avoid circular imports
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

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
pub(crate) fn handle(
    tag_name: &str,
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    // Import helper functions from parent converter module
    use crate::converter::{normalized_tag_name, walk_node};

    match tag_name {
        "ruby" => {
            // Clone context for ruby children processing
            let ruby_ctx = ctx.clone();

            // Scan child elements to determine rendering mode
            let tag_sequence: Vec<String> = node_handle
                .children()
                .top()
                .iter()
                .filter_map(|child_handle| {
                    if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                        let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                        // Only track rb, rt, rtc tags to determine structure
                        if matches!(tag_name.as_ref(), "rb" | "rt" | "rtc") {
                            Some(tag_name.into_owned())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // Detect presence of ruby text container
            let has_rtc = tag_sequence.iter().any(|tag| tag == "rtc");

            // Detect interleaved mode: rb followed immediately by rt
            let is_interleaved = tag_sequence.windows(2).any(|w| w[0] == "rb" && w[1] == "rt");

            if is_interleaved && !has_rtc {
                // Interleaved rendering: process rb/rt pairs inline
                let mut current_base = String::new();
                let children = node_handle.children();
                {
                    for child_handle in children.top().iter() {
                        if let Some(node) = child_handle.get(parser) {
                            match node {
                                tl::Node::Tag(child_tag) => {
                                    let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                    if tag_name == "rt" {
                                        // Process rt (ruby text/annotation)
                                        let mut annotation = String::new();
                                        walk_node(
                                            child_handle,
                                            parser,
                                            &mut annotation,
                                            options,
                                            &ruby_ctx,
                                            depth,
                                            dom_ctx,
                                        );
                                        // Output any pending base text
                                        if !current_base.is_empty() {
                                            output.push_str(current_base.trim());
                                            current_base.clear();
                                        }
                                        // Output annotation text
                                        output.push_str(annotation.trim());
                                    } else if tag_name == "rb" {
                                        // Process rb (ruby base)
                                        if !current_base.is_empty() {
                                            output.push_str(current_base.trim());
                                            current_base.clear();
                                        }
                                        walk_node(
                                            child_handle,
                                            parser,
                                            &mut current_base,
                                            options,
                                            &ruby_ctx,
                                            depth,
                                            dom_ctx,
                                        );
                                    } else if tag_name != "rp" {
                                        // Skip rp, process other elements into current_base
                                        walk_node(
                                            child_handle,
                                            parser,
                                            &mut current_base,
                                            options,
                                            &ruby_ctx,
                                            depth,
                                            dom_ctx,
                                        );
                                    }
                                }
                                tl::Node::Raw(_) => {
                                    // Process raw text nodes
                                    walk_node(
                                        child_handle,
                                        parser,
                                        &mut current_base,
                                        options,
                                        &ruby_ctx,
                                        depth,
                                        dom_ctx,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // Flush remaining base text
                if !current_base.is_empty() {
                    output.push_str(current_base.trim());
                }
            } else {
                // Grouped rendering: collect all bases, then annotations
                let mut base_text = String::new();
                let mut rt_annotations = Vec::new();
                let mut rtc_content = String::new();

                let children = node_handle.children();
                {
                    for child_handle in children.top().iter() {
                        if let Some(node) = child_handle.get(parser) {
                            match node {
                                tl::Node::Tag(child_tag) => {
                                    let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                    if tag_name == "rt" {
                                        // Collect rt annotations
                                        let mut annotation = String::new();
                                        walk_node(
                                            child_handle,
                                            parser,
                                            &mut annotation,
                                            options,
                                            &ruby_ctx,
                                            depth,
                                            dom_ctx,
                                        );
                                        rt_annotations.push(annotation);
                                    } else if tag_name == "rtc" {
                                        // Collect rtc (ruby text container) content
                                        walk_node(
                                            child_handle,
                                            parser,
                                            &mut rtc_content,
                                            options,
                                            &ruby_ctx,
                                            depth,
                                            dom_ctx,
                                        );
                                    } else if tag_name != "rp" {
                                        // Collect base text (skip rp elements)
                                        walk_node(
                                            child_handle,
                                            parser,
                                            &mut base_text,
                                            options,
                                            &ruby_ctx,
                                            depth,
                                            dom_ctx,
                                        );
                                    }
                                }
                                tl::Node::Raw(_) => {
                                    // Collect raw text into base
                                    walk_node(child_handle, parser, &mut base_text, options, &ruby_ctx, depth, dom_ctx);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Output base text
                let trimmed_base = base_text.trim();
                output.push_str(trimmed_base);

                // Output rt annotations in parentheses if present
                if !rt_annotations.is_empty() {
                    let rt_text = rt_annotations.iter().map(|s| s.trim()).collect::<Vec<_>>().join("");
                    if !rt_text.is_empty() {
                        // Wrap in parentheses only if we have rtc content and multiple annotations
                        if has_rtc && !rtc_content.trim().is_empty() && rt_annotations.len() > 1 {
                            output.push('(');
                            output.push_str(&rt_text);
                            output.push(')');
                        } else {
                            output.push_str(&rt_text);
                        }
                    }
                }

                // Output rtc content after rt annotations
                if !rtc_content.trim().is_empty() {
                    output.push_str(rtc_content.trim());
                }
            }
        }

        "rb" => {
            // Ruby base text element (typically used within ruby)
            // When standalone, just extract and output the text
            let mut text = String::new();
            let children = node_handle.children();
            {
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
                }
            }
            output.push_str(text.trim());
        }

        "rt" => {
            // Ruby text/annotation element
            // When standalone (outside ruby context), wrap annotation in parentheses
            let mut text = String::new();
            let children = node_handle.children();
            {
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
                }
            }
            let trimmed = text.trim();

            // Check if output already ends with opening paren (interleaved mode)
            if output.ends_with('(') {
                output.push_str(trimmed);
            } else {
                // Otherwise wrap annotation in parentheses
                output.push('(');
                output.push_str(trimmed);
                output.push(')');
            }
        }

        "rp" => {
            // Ruby parenthesis element (fallback for non-ruby-supporting browsers)
            // In Markdown output, generally skip these as annotations are in parentheses
            let mut content = String::new();
            let children = node_handle.children();
            {
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                }
            }
            let trimmed = content.trim();
            // Only output non-empty rp content
            if !trimmed.is_empty() {
                output.push_str(trimmed);
            }
        }

        "rtc" => {
            // Ruby text container element
            // When standalone, just process children normally
            let children = node_handle.children();
            {
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                }
            }
        }

        _ => {
            // Fallback for unknown ruby-related tags: process children normally
            let children = node_handle.children();
            {
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                }
            }
        }
    }
}
