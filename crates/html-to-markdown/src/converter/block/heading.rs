//! Handler for heading elements (h1-h6).
//!
//! Converts HTML heading tags to Markdown heading syntax with support for:
//! - Multiple heading styles (ATX, underlined, closed ATX)
//! - Inline content processing with proper text normalization
//! - Metadata collection (headers, IDs)
//! - Visitor callbacks for custom heading processing

use crate::options::{ConversionOptions, HeadingStyle};
use std::borrow::Cow;
#[allow(unused_imports)]
use std::collections::BTreeMap;
use tl::{NodeHandle, Parser};

// Type aliases for Context and DomContext to avoid circular imports
// These are imported from converter.rs and should be made accessible
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handle heading elements (h1, h2, h3, h4, h5, h6).
///
/// Extracts the heading level from the tag name, processes inline content,
/// normalizes text, and outputs formatted heading with proper spacing.
///
/// # Note
/// This function references `walk_node` from converter.rs which must be
/// accessible (pub(crate)) for this module to work correctly.
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
    // Import walk_node from parent converter module
    use crate::converter::walk_node;

    let level = tag_name.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1) as usize;

    // Add spacing before heading if needed (similar to paragraph handling)
    let needs_leading_sep = !ctx.in_table_cell
        && !ctx.in_list_item
        && !ctx.convert_as_inline
        && ctx.blockquote_depth == 0
        && !output.is_empty()
        && !output.ends_with("\n\n");

    if needs_leading_sep {
        crate::converter::trim_trailing_whitespace(output);
        output.push_str("\n\n");
    }

    let mut text = String::new();
    let heading_ctx = Context {
        in_heading: true,
        convert_as_inline: true,
        heading_allow_inline_images: heading_allows_inline_images(tag_name, &ctx.keep_inline_images_in),
        ..ctx.clone()
    };

    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let children = tag.children();
            for child_handle in children.top().iter() {
                walk_node(
                    child_handle,
                    parser,
                    &mut text,
                    options,
                    &heading_ctx,
                    depth + 1,
                    dom_ctx,
                );
            }
        }
    }

    let trimmed = text.trim();
    if !trimmed.is_empty() {
        let normalized = normalize_heading_text(trimmed);

        #[cfg(feature = "visitor")]
        let heading_output = visitor_heading_output(
            node_handle,
            parser,
            tag_name,
            level,
            &normalized,
            output,
            options,
            ctx,
            depth,
            dom_ctx,
        );

        #[cfg(not(feature = "visitor"))]
        let heading_output = {
            let mut buf = String::new();
            push_heading(&mut buf, ctx, options, level, normalized.as_ref());
            Some(buf)
        };

        if let Some(heading_text) = heading_output {
            output.push_str(&heading_text);
        }

        #[cfg(feature = "metadata")]
        if ctx.metadata_wants_headers {
            if let Some(ref collector) = ctx.metadata_collector {
                if let Some(node) = node_handle.get(parser) {
                    if let tl::Node::Tag(tag) = node {
                        let id = tag
                            .attributes()
                            .get("id")
                            .flatten()
                            .map(|v| v.as_utf8_str().to_string());
                        collector
                            .borrow_mut()
                            .add_header(level as u8, normalized.to_string(), id, depth, 0);
                    }
                }
            }
        }
    }
}

/// Determine if a heading element should allow inline images.
pub(crate) fn heading_allows_inline_images(
    tag_name: &str,
    keep_inline_images_in: &std::rc::Rc<std::collections::HashSet<String>>,
) -> bool {
    keep_inline_images_in.contains(tag_name)
}

/// Normalize heading text by replacing newlines with spaces.
fn normalize_heading_text(text: &str) -> Cow<'_, str> {
    if !text.contains('\n') && !text.contains('\r') {
        return Cow::Borrowed(text);
    }

    let mut normalized = String::with_capacity(text.len());
    let mut pending_space = false;

    for ch in text.chars() {
        match ch {
            '\n' | '\r' => {
                if !normalized.is_empty() {
                    pending_space = true;
                }
            }
            ' ' | '\t' if pending_space => {}
            _ => {
                if pending_space {
                    if !normalized.ends_with(' ') {
                        normalized.push(' ');
                    }
                    pending_space = false;
                }
                normalized.push(ch);
            }
        }
    }

    Cow::Owned(normalized)
}

/// Format heading output with appropriate markdown syntax.
pub(crate) fn push_heading(output: &mut String, ctx: &Context, options: &ConversionOptions, level: usize, text: &str) {
    if text.is_empty() {
        return;
    }

    if ctx.convert_as_inline {
        output.push_str(text);
        return;
    }

    if ctx.in_table_cell {
        let is_table_continuation =
            !output.is_empty() && !output.ends_with('|') && !output.ends_with(' ') && !output.ends_with("<br>");
        if is_table_continuation {
            output.push_str("<br>");
        }
        output.push_str(text);
        return;
    }

    if ctx.in_list_item {
        if output.ends_with('\n') {
            if let Some(indent) = continuation_indent_string(ctx.list_depth, options) {
                output.push_str(&indent);
            }
        } else if !output.ends_with(' ') && !output.is_empty() {
            output.push(' ');
        }
    } else if !output.is_empty() && !output.ends_with("\n\n") {
        if output.ends_with('\n') {
            output.push('\n');
        } else {
            crate::converter::trim_trailing_whitespace(output);
            output.push_str("\n\n");
        }
    }

    let heading_suffix = if ctx.in_list_item || ctx.blockquote_depth > 0 {
        "\n"
    } else {
        "\n\n"
    };

    match options.heading_style {
        HeadingStyle::Underlined => {
            if level == 1 {
                output.push_str(text);
                output.push('\n');
                output.push_str(&"=".repeat(text.len()));
            } else if level == 2 {
                output.push_str(text);
                output.push('\n');
                output.push_str(&"-".repeat(text.len()));
            } else {
                output.push_str(&"#".repeat(level));
                output.push(' ');
                output.push_str(text);
            }
        }
        HeadingStyle::Atx => {
            output.push_str(&"#".repeat(level));
            output.push(' ');
            output.push_str(text);
        }
        HeadingStyle::AtxClosed => {
            output.push_str(&"#".repeat(level));
            output.push(' ');
            output.push_str(text);
            output.push(' ');
            output.push_str(&"#".repeat(level));
        }
    }
    output.push_str(heading_suffix);
}

/// Get continuation indent string for list items.
fn continuation_indent_string(list_depth: usize, _options: &ConversionOptions) -> Option<String> {
    if list_depth == 0 {
        return None;
    }
    Some(" ".repeat(4 * list_depth))
}

/// Process heading with visitor callback if available.
#[cfg(feature = "visitor")]
fn visitor_heading_output(
    node_handle: &NodeHandle,
    parser: &Parser,
    tag_name: &str,
    level: usize,
    normalized: &str,
    _output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) -> Option<String> {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    if let Some(ref visitor_handle) = ctx.visitor {
        if let Some(node) = node_handle.get(parser) {
            if let tl::Node::Tag(tag) = node {
                let id_attr = tag
                    .attributes()
                    .get("id")
                    .flatten()
                    .map(|v| v.as_utf8_str().to_string());

                let attributes: BTreeMap<String, String> = tag
                    .attributes()
                    .iter()
                    .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                    .collect();

                let node_id = node_handle.get_inner();
                let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                let node_ctx = NodeContext {
                    node_type: NodeType::Heading,
                    tag_name: tag_name.to_string(),
                    attributes,
                    depth,
                    index_in_parent,
                    parent_tag,
                    is_inline: false,
                };

                let mut visitor = visitor_handle.borrow_mut();
                match visitor.visit_heading(&node_ctx, level as u32, normalized, id_attr.as_deref()) {
                    VisitResult::Continue => {
                        let mut buf = String::new();
                        push_heading(&mut buf, ctx, options, level, normalized);
                        Some(buf)
                    }
                    VisitResult::Custom(custom) => Some(custom),
                    VisitResult::Skip => None,
                    VisitResult::Error(err) => {
                        if ctx.visitor_error.borrow().is_none() {
                            *ctx.visitor_error.borrow_mut() = Some(err);
                        }
                        None
                    }
                    VisitResult::PreserveHtml => {
                        let mut buf = String::new();
                        push_heading(&mut buf, ctx, options, level, normalized);
                        Some(buf)
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        let mut buf = String::new();
        push_heading(&mut buf, ctx, options, level, normalized);
        Some(buf)
    }
}

/// Find a single heading element within a node, filtering out non-heading content.
///
/// Returns the heading level and node handle if the node contains exactly one
/// heading with no other non-whitespace content. Returns None if:
/// - The node is not a tag
/// - Multiple headings are found
/// - Non-whitespace non-heading content exists
/// - Non-text comments exist
pub(crate) fn find_single_heading_child(node_handle: NodeHandle, parser: &Parser) -> Option<(usize, NodeHandle)> {
    let node = node_handle.get(parser)?;

    let tl::Node::Tag(tag) = node else {
        return None;
    };

    let children = tag.children();
    let mut heading_data: Option<(usize, NodeHandle)> = None;

    for child_handle in children.top().iter() {
        let Some(child_node) = child_handle.get(parser) else {
            continue;
        };

        match child_node {
            tl::Node::Raw(bytes) => {
                if !bytes.as_utf8_str().trim().is_empty() {
                    return None;
                }
            }
            tl::Node::Tag(child_tag) => {
                let name = crate::converter::utility::content::normalized_tag_name(child_tag.name().as_utf8_str());
                if let Some(level) = heading_level_from_name(name.as_ref()) {
                    if heading_data.is_some() {
                        return None;
                    }
                    heading_data = Some((level, *child_handle));
                } else {
                    return None;
                }
            }
            tl::Node::Comment(_) => return None,
        }
    }

    heading_data
}

/// Extract heading level from tag name (h1-h6).
///
/// Returns Some(level) for valid heading tags, None otherwise.
fn heading_level_from_name(name: &str) -> Option<usize> {
    match name {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}
