//! Handler for link elements (a, anchor).
//!
//! Converts HTML anchor tags to Markdown links with support for:
//! - Standard Markdown link syntax `[label](href "title")`
//! - Autolinks for simple URLs like `<https://example.com>`
//! - Link label escaping for special Markdown characters
//! - Heading-in-link special handling (wraps link around heading)
//! - 512-character label truncation with ellipsis
//! - Visitor callbacks for custom link processing
//! - Metadata collection for links (links, URLs, titles, rel attributes)
//! - Block-level content within links (via inline context)

use crate::converter::utility::content::{is_block_level_element, normalized_tag_name};
use crate::options::ConversionOptions;
use std::collections::BTreeMap;
use tl::{NodeHandle, Parser};

// Type aliases for Context and DomContext to avoid circular imports
// These are imported from converter.rs and should be made accessible
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Maximum length for link label before truncation
const MAX_LINK_LABEL_LEN: usize = 512;

/// Handler for anchor/link elements: `<a>`.
///
/// Processes anchor tags to generate Markdown links:
/// - Detects autolinks (link text matches href)
/// - Extracts and normalizes link labels
/// - Handles nested headings within links
/// - Escapes special characters in labels
/// - Collects metadata when feature is enabled
/// - Supports visitor callbacks for custom processing
///
/// # Link Label Extraction
/// For links with block-level content, extracts text separately.
/// Collapses newlines and normalizes whitespace per Markdown spec.
///
/// # Autolinks
/// When `autolinks` option is enabled, detects links where the text equals
/// the href (e.g., `<a href="https://example.com">https://example.com</a>`)
/// and outputs as `<https://example.com>` instead.
///
/// # Note
/// This function references helper functions from converter.rs
/// which must be accessible (pub(crate)) for this module to work correctly.
pub(crate) fn handle(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    // Import helper functions from parent converter module
    use crate::converter::block::heading::{heading_allows_inline_images, push_heading};
    use crate::converter::utility::content::{normalized_tag_name, truncate_at_char_boundary};
    #[allow(unused_imports)]
    use crate::converter::utility::serialization::serialize_node;
    use crate::converter::{find_single_heading_child, get_text_content, walk_node};

    let Some(node) = node_handle.get(parser) else {
        return;
    };

    let tl::Node::Tag(tag) = node else {
        return;
    };

    // Extract href and title attributes
    let href_attr = tag
        .attributes()
        .get("href")
        .flatten()
        .map(|v| crate::text::decode_html_entities(&v.as_utf8_str()));
    let title = tag
        .attributes()
        .get("title")
        .flatten()
        .map(|v| v.as_utf8_str().to_string());

    if let Some(href) = href_attr {
        let raw_text = crate::text::normalize_whitespace(&get_text_content(node_handle, parser, dom_ctx))
            .trim()
            .to_string();

        // If we're already inside a link, just render the text content, don't create a nested link
        if ctx.in_link {
            let children = tag.children();
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
            }
            return;
        }

        // Check if this should be rendered as an autolink
        let is_autolink = options.autolinks
            && !options.default_title
            && !href.is_empty()
            && (raw_text == href || (href.starts_with("mailto:") && raw_text == href[7..]));

        if is_autolink {
            output.push('<');
            if href.starts_with("mailto:") && raw_text == href[7..] {
                output.push_str(&raw_text);
            } else {
                output.push_str(&href);
            }
            output.push('>');
            return;
        }

        // Check if link contains a single heading child element
        if let Some((heading_level, heading_handle)) = find_single_heading_child(*node_handle, parser) {
            if let Some(heading_node) = heading_handle.get(parser) {
                if let tl::Node::Tag(heading_tag) = heading_node {
                    let heading_name = normalized_tag_name(heading_tag.name().as_utf8_str()).into_owned();
                    let mut heading_text = String::new();
                    let heading_ctx = Context {
                        in_heading: true,
                        convert_as_inline: true,
                        heading_allow_inline_images: heading_allows_inline_images(
                            &heading_name,
                            &ctx.keep_inline_images_in,
                        ),
                        ..ctx.clone()
                    };
                    walk_node(
                        &heading_handle,
                        parser,
                        &mut heading_text,
                        options,
                        &heading_ctx,
                        depth + 1,
                        dom_ctx,
                    );
                    let trimmed_heading = heading_text.trim();
                    if !trimmed_heading.is_empty() {
                        let escaped_label = escape_link_label(trimmed_heading);
                        let mut link_buffer = String::new();
                        append_markdown_link(
                            &mut link_buffer,
                            &escaped_label,
                            href.as_str(),
                            title.as_deref(),
                            raw_text.as_str(),
                            options,
                        );
                        push_heading(output, ctx, options, heading_level, link_buffer.as_str());
                        return;
                    }
                }
            }
        }

        // Collect link label from children
        let children: Vec<_> = tag.children().top().iter().copied().collect();
        let (inline_label, _block_nodes, saw_block) = collect_link_label_text(&children, parser, dom_ctx);
        let mut label = if saw_block {
            let mut content = String::new();
            let link_ctx = Context {
                inline_depth: ctx.inline_depth + 1,
                convert_as_inline: true,
                in_link: true,
                ..ctx.clone()
            };
            for child_handle in &children {
                let mut child_buf = String::new();
                walk_node(
                    child_handle,
                    parser,
                    &mut child_buf,
                    options,
                    &link_ctx,
                    depth + 1,
                    dom_ctx,
                );
                if !child_buf.trim().is_empty()
                    && !content.is_empty()
                    && !content.chars().last().is_none_or(char::is_whitespace)
                    && !child_buf.chars().next().is_none_or(char::is_whitespace)
                {
                    content.push(' ');
                }
                content.push_str(&child_buf);
            }
            if content.trim().is_empty() {
                normalize_link_label(&inline_label)
            } else {
                normalize_link_label(&content)
            }
        } else {
            let mut content = String::new();
            let link_ctx = Context {
                inline_depth: ctx.inline_depth + 1,
                in_link: true,
                ..ctx.clone()
            };
            for child_handle in &children {
                walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    options,
                    &link_ctx,
                    depth + 1,
                    dom_ctx,
                );
            }
            normalize_link_label(&content)
        };

        // Apply fallback label strategies
        if label.is_empty() && saw_block {
            let fallback = crate::text::normalize_whitespace(&get_text_content(node_handle, parser, dom_ctx));
            label = normalize_link_label(&fallback);
        }

        if label.is_empty() && !raw_text.is_empty() {
            label = normalize_link_label(&raw_text);
        }

        if label.is_empty() && !href.is_empty() && !children.is_empty() {
            label = href.clone();
        }

        // Truncate label if it exceeds maximum length
        if label.len() > MAX_LINK_LABEL_LEN {
            truncate_at_char_boundary(&mut label, MAX_LINK_LABEL_LEN);
            label.push('…');
        }

        let escaped_label = escape_link_label(&label);

        // Handle visitor callbacks if feature is enabled
        #[cfg(feature = "visitor")]
        let link_output = if let Some(ref visitor_handle) = ctx.visitor {
            use crate::visitor::{NodeContext, NodeType, VisitResult};

            let attributes: BTreeMap<String, String> = tag
                .attributes()
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                .collect();

            let node_id = node_handle.get_inner();
            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

            let node_ctx = NodeContext {
                node_type: NodeType::Link,
                tag_name: "a".to_string(),
                attributes,
                depth,
                index_in_parent,
                parent_tag,
                is_inline: true,
            };

            let mut visitor = visitor_handle.borrow_mut();
            match visitor.visit_link(&node_ctx, &href, &label, title.as_deref()) {
                VisitResult::Continue => {
                    let mut buf = String::new();
                    append_markdown_link(
                        &mut buf,
                        &escaped_label,
                        href.as_str(),
                        title.as_deref(),
                        label.as_str(),
                        options,
                    );
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
                VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
            }
        } else {
            let mut buf = String::new();
            append_markdown_link(
                &mut buf,
                &escaped_label,
                href.as_str(),
                title.as_deref(),
                label.as_str(),
                options,
            );
            Some(buf)
        };

        #[cfg(not(feature = "visitor"))]
        let link_output = {
            let mut buf = String::new();
            append_markdown_link(
                &mut buf,
                &escaped_label,
                href.as_str(),
                title.as_deref(),
                label.as_str(),
                options,
            );
            Some(buf)
        };

        if let Some(link_text) = link_output {
            output.push_str(&link_text);
        }

        // Collect metadata if feature is enabled
        #[cfg(feature = "metadata")]
        if ctx.metadata_wants_links {
            if let Some(ref collector) = ctx.metadata_collector {
                let rel_attr = tag
                    .attributes()
                    .get("rel")
                    .flatten()
                    .map(|v| v.as_utf8_str().to_string());
                let mut attributes_map = BTreeMap::new();
                for (key, value_opt) in tag.attributes().iter() {
                    let key_str = key.to_string();
                    if key_str == "href" {
                        continue;
                    }

                    let value = value_opt.map(|v| v.to_string()).unwrap_or_default();
                    attributes_map.insert(key_str, value);
                }
                collector
                    .borrow_mut()
                    .add_link(href.clone(), label.clone(), title.clone(), rel_attr, attributes_map);
            }
        }
    } else {
        // No href: just process children as inline content
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
        }
    }
}

/// Escape special Markdown characters in link labels.
///
/// Escapes unmatched closing brackets `]` to prevent accidental link termination.
/// Tracks bracket nesting to avoid escaping matched closing brackets.
///
/// # Examples
/// ```text
/// Input:  "Click [here] for more"
/// Output: "Click [here\\] for more"  (closing bracket is escaped because it's unmatched)
///
/// Input:  "Normal text"
/// Output: "Normal text"  (no escaping needed)
/// ```
fn escape_link_label(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(text.len());
    let mut backslash_count = 0usize;
    let mut bracket_depth = 0usize;

    for ch in text.chars() {
        if ch == '\\' {
            result.push('\\');
            backslash_count += 1;
            continue;
        }

        let is_escaped = backslash_count % 2 == 1;
        backslash_count = 0;

        match ch {
            '[' if !is_escaped => {
                bracket_depth = bracket_depth.saturating_add(1);
                result.push('[');
            }
            ']' if !is_escaped => {
                if bracket_depth == 0 {
                    result.push('\\');
                } else {
                    bracket_depth -= 1;
                }
                result.push(']');
            }
            _ => result.push(ch),
        }
    }

    result
}

/// Format and append a Markdown link to the output string.
///
/// Generates the link syntax: `[label](href "title")`
/// Handles special cases:
/// - Empty href renders as `[label]()`
/// - Hrefs with spaces/newlines get wrapped in angle brackets: `[label](<URL with spaces>)`
/// - Unbalanced parentheses in href get escaped: `[label](url\(example\))`
/// - Titles are wrapped in quotes and quotes inside are escaped
/// - When `default_title` option is true and raw_text equals href, adds href as title
///
/// # Arguments
/// * `output` - Output buffer to append the link to
/// * `label` - The link text (already escaped)
/// * `href` - The URL/destination
/// * `title` - Optional link title attribute
/// * `raw_text` - Original unprocessed text (for default_title option)
/// * `options` - Conversion options
pub(crate) fn append_markdown_link(
    output: &mut String,
    label: &str,
    href: &str,
    title: Option<&str>,
    raw_text: &str,
    options: &ConversionOptions,
) {
    output.push('[');
    output.push_str(label);
    output.push_str("](");

    if href.is_empty() {
        output.push_str("<>");
    } else if href.contains(' ') || href.contains('\n') {
        output.push('<');
        output.push_str(href);
        output.push('>');
    } else {
        let open_count = href.chars().filter(|&c| c == '(').count();
        let close_count = href.chars().filter(|&c| c == ')').count();

        if open_count == close_count {
            output.push_str(href);
        } else {
            let escaped_href = href.replace('(', "\\(").replace(')', "\\)");
            output.push_str(&escaped_href);
        }
    }

    if let Some(title_text) = title {
        output.push_str(" \"");
        if title_text.contains('"') {
            let escaped_title = title_text.replace('"', "\\\"");
            output.push_str(&escaped_title);
        } else {
            output.push_str(title_text);
        }
        output.push('"');
    } else if options.default_title && raw_text == href {
        output.push_str(" \"");
        if href.contains('"') {
            let escaped_href = href.replace('"', "\\\"");
            output.push_str(&escaped_href);
        } else {
            output.push_str(href);
        }
        output.push('"');
    }

    output.push(')');
}

/// Collect text content from direct inline children of a link element.
///
/// Performs a shallow scan to find text content, distinguishing between:
/// - Inline text (normal flow, accumulated)
/// - Block-level elements (stop at them, mark `saw_block`)
/// - Comments (stop processing)
///
/// Returns:
/// - `(text, block_nodes, saw_block)` where:
///   - `text` is concatenated inline text
///   - `block_nodes` is list of block-level children found
///   - `saw_block` indicates if any block elements were encountered
///
/// # Algorithm
/// Uses a stack-based approach to traverse the DOM tree, accumulating text
/// from inline elements while identifying block-level boundaries.
fn collect_link_label_text(
    children: &[NodeHandle],
    parser: &Parser,
    dom_ctx: &DomContext,
) -> (String, Vec<NodeHandle>, bool) {
    let mut text = String::new();
    let mut saw_block = false;
    let mut block_nodes = Vec::new();
    let mut stack: Vec<_> = children.iter().rev().copied().collect();

    while let Some(handle) = stack.pop() {
        if let Some(node) = handle.get(parser) {
            match node {
                tl::Node::Raw(bytes) => {
                    let raw = bytes.as_utf8_str();
                    let decoded = crate::text::decode_html_entities_cow(raw.as_ref());
                    text.push_str(decoded.as_ref());
                }
                tl::Node::Tag(tag) => {
                    let is_block = dom_ctx.tag_info(handle.get_inner(), parser).map_or_else(
                        || {
                            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
                            is_block_level_element(tag_name.as_ref())
                        },
                        |info| info.is_block,
                    );
                    if is_block {
                        saw_block = true;
                        block_nodes.push(handle);
                        continue;
                    }

                    if let Some(children) = dom_ctx.children_of(handle.get_inner()) {
                        for child in children.iter().rev() {
                            stack.push(*child);
                        }
                    } else {
                        let tag_children = tag.children();
                        let mut child_nodes: Vec<_> = tag_children.top().iter().copied().collect();
                        child_nodes.reverse();
                        stack.extend(child_nodes);
                    }
                }
                _ => {}
            }
        }
    }

    (text, block_nodes, saw_block)
}

/// Normalize link label text.
///
/// Collapses line breaks and normalizes whitespace:
/// - Replaces `\n` and `\r` with spaces
/// - Collapses multiple consecutive spaces to single space
/// - Trims leading/trailing whitespace
///
/// This is required by the Markdown spec for link labels to function properly.
///
/// # Examples
/// ```text
/// Input:  "Line 1\nLine 2"
/// Output: "Line 1 Line 2"
///
/// Input:  "Text  with   spaces"
/// Output: "Text with spaces"
/// ```
#[allow(clippy::trivially_copy_pass_by_ref)]
fn normalize_link_label(label: &str) -> String {
    let mut needs_collapse = false;
    for ch in label.chars() {
        if ch == '\n' || ch == '\r' {
            needs_collapse = true;
            break;
        }
    }

    let collapsed = if needs_collapse {
        let mut collapsed = String::with_capacity(label.len());
        for ch in label.chars() {
            if ch == '\n' || ch == '\r' {
                collapsed.push(' ');
            } else {
                collapsed.push(ch);
            }
        }
        std::borrow::Cow::Owned(collapsed)
    } else {
        std::borrow::Cow::Borrowed(label)
    };

    let normalized = crate::text::normalize_whitespace_cow(collapsed.as_ref());
    normalized.as_ref().trim().to_string()
}
