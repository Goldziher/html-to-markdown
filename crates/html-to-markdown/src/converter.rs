//! HTML to Markdown conversion using tl parser.
//!
//! This module provides the core conversion logic for transforming HTML documents into Markdown.
//! It uses the tl parser for high-performance HTML parsing and supports 60+ HTML tags.
//!

#![allow(clippy::collapsible_match)]
//! # Architecture
//!
//! The conversion process follows these steps:
//! 1. Parse HTML into a DOM tree using tl parser
//! 2. Walk the DOM tree recursively
//! 3. Convert each node type to its Markdown equivalent
//! 4. Apply text escaping and whitespace normalization
//!
//! # Whitespace Handling
//!
//! This library preserves whitespace exactly as it appears in the HTML source.
//! Text nodes retain their original spacing, including multiple spaces and newlines.
//!
//! - **Raw text preservation**: All whitespace in text nodes is preserved
//! - **No HTML5 normalization**: Whitespace is not collapsed according to HTML5 rules
//! - **Full control**: Applications can handle whitespace as needed
//!
//! # Supported Features
//!
//! - **Block elements**: headings, paragraphs, lists, tables, blockquotes
//! - **Inline formatting**: bold, italic, code, links, images, strikethrough
//! - **Semantic HTML5**: article, section, nav, aside, header, footer
//! - **Forms**: inputs, select, button, textarea, fieldset
//! - **Media**: audio, video, picture, iframe, svg
//! - **Advanced**: task lists, ruby annotations, definition lists
//!
//! # Examples
//!
//! ```rust
//! use html_to_markdown_rs::{convert, ConversionOptions};
//!
//! let html = "<h1>Title</h1><p>Paragraph with <strong>bold</strong> text.</p>";
//! let markdown = convert(html, None).unwrap();
//! assert_eq!(markdown, "# Title\n\nParagraph with **bold** text.\n");
//! ```

#[cfg(feature = "inline-images")]
use std::cell::RefCell;
use std::collections::BTreeMap;
#[cfg(feature = "inline-images")]
use std::rc::Rc;

use std::borrow::Cow;
use std::str;

use crate::error::Result;
#[cfg(feature = "inline-images")]
use crate::inline_images::{InlineImageCollector, InlineImageFormat, InlineImageSource};
use crate::options::{ConversionOptions, HeadingStyle, ListIndentType};
use crate::text;

#[cfg(feature = "inline-images")]
type InlineCollectorHandle = Rc<RefCell<InlineImageCollector>>;
#[cfg(not(feature = "inline-images"))]
type InlineCollectorHandle = ();

/// Chomp whitespace from inline element content, preserving line breaks.
///
/// Similar to text::chomp but handles line breaks from <br> tags specially.
/// Line breaks are extracted as suffix to be placed outside formatting.
/// Returns (prefix, suffix, trimmed_text).
fn chomp_inline(text: &str) -> (&str, &str, &str) {
    if text.is_empty() {
        return ("", "", "");
    }

    let prefix = if text.starts_with(&[' ', '\t'][..]) { " " } else { "" };

    let has_trailing_linebreak = text.ends_with("  \n") || text.ends_with("\\\n");

    let suffix = if has_trailing_linebreak {
        if text.ends_with("  \n") {
            "  \n"
        } else {
            "\\\n"
        }
    } else if text.ends_with(&[' ', '\t'][..]) {
        " "
    } else {
        ""
    };

    let trimmed = if has_trailing_linebreak {
        if let Some(stripped) = text.strip_suffix("  \n") {
            stripped.trim()
        } else if let Some(stripped) = text.strip_suffix("\\\n") {
            stripped.trim()
        } else {
            text.trim()
        }
    } else {
        text.trim()
    };

    (prefix, suffix, trimmed)
}

/// Remove trailing spaces and tabs from output string.
///
/// This is used before adding block separators or newlines to ensure
/// clean Markdown output without spurious whitespace.
fn trim_trailing_whitespace(output: &mut String) {
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
}

/// Calculate indentation level for list item continuations.
///
/// Returns the number of 4-space indent groups needed for list continuations.
///
/// List continuations (block elements inside list items) need special indentation:
/// - Base indentation: (depth - 1) groups (for the nesting level)
/// - Content indentation: depth groups (for the list item content)
/// - Combined formula: (2 * depth - 1) groups of 4 spaces each
///
/// # Examples
///
/// ```text
/// * Item 1           (depth=0, no continuation)
/// * Item 2           (depth=0)
///     Continuation   (depth=0: 0 groups = 0 spaces)
///
/// * Level 1          (depth=0)
///     + Level 2      (depth=1)
///             Cont   (depth=1: (2*1-1) = 1 group = 4 spaces, total 12 with bullet indent)
/// ```
fn calculate_list_continuation_indent(depth: usize) -> usize {
    if depth > 0 {
        2 * depth - 1
    } else {
        0
    }
}

/// Check if a list (ul or ol) is "loose".
///
/// A loose list is one where any list item contains block-level elements
/// like paragraphs (<p>). In loose lists, all items should have blank line
/// separation (ending with \n\n) regardless of their own content.
///
/// # Examples
///
/// ```html
/// <!-- Loose list (has <p> in an item) -->
/// <ul>
///   <li><p>Item 1</p></li>
///   <li>Item 2</li>  <!-- Also gets \n\n ending -->
/// </ul>
///
/// <!-- Tight list (no block elements) -->
/// <ul>
///   <li>Item 1</li>
///   <li>Item 2</li>
/// </ul>
/// ```
fn is_loose_list(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> bool {
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let children = tag.children();
            {
                for child_handle in children.top().iter() {
                    if let Some(child_node) = child_handle.get(parser) {
                        if let tl::Node::Tag(child_tag) = child_node {
                            if child_tag.name().as_utf8_str() == "li" {
                                let li_children = child_tag.children();
                                {
                                    for li_child_handle in li_children.top().iter() {
                                        if let Some(li_child_node) = li_child_handle.get(parser) {
                                            if let tl::Node::Tag(li_child_tag) = li_child_node {
                                                if li_child_tag.name().as_utf8_str() == "p" {
                                                    return true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Add list continuation indentation to output.
///
/// Used when block elements (like <p> or <div>) appear inside list items.
/// Adds appropriate line separation and indentation to continue the list item.
///
/// # Arguments
///
/// * `output` - The output string to append to
/// * `list_depth` - Current list nesting depth
/// * `blank_line` - If true, adds blank line separation (\n\n); if false, single newline (\n)
///
/// # Examples
///
/// ```text
/// Paragraph continuation (blank_line = true):
///   * First para
///
///       Second para  (blank line + indentation)
///
/// Div continuation (blank_line = false):
///   * First div
///       Second div   (single newline + indentation)
/// ```
fn add_list_continuation_indent(output: &mut String, list_depth: usize, blank_line: bool, options: &ConversionOptions) {
    trim_trailing_whitespace(output);

    if blank_line {
        if !output.ends_with("\n\n") {
            if output.ends_with('\n') {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }
    } else if !output.ends_with('\n') {
        output.push('\n');
    }

    let indent_level = calculate_list_continuation_indent(list_depth);
    let indent_char = match options.list_indent_type {
        ListIndentType::Tabs => "\t",
        ListIndentType::Spaces => &" ".repeat(options.list_indent_width),
    };
    output.push_str(&indent_char.repeat(indent_level));
}

/// Add appropriate leading separator before a list.
///
/// Lists need different separators depending on context:
/// - In table cells: <br> tag if there's already content
/// - Outside lists: blank line (\n\n) if needed
/// - Inside list items: blank line before nested list
fn add_list_leading_separator(output: &mut String, ctx: &Context) {
    if ctx.in_table_cell {
        let is_table_continuation =
            !output.is_empty() && !output.ends_with('|') && !output.ends_with(' ') && !output.ends_with("<br>");
        if is_table_continuation {
            output.push_str("<br>");
        }
        return;
    }

    if !output.is_empty() && !ctx.in_list {
        let needs_newline =
            !output.ends_with("\n\n") && !output.ends_with("* ") && !output.ends_with("- ") && !output.ends_with(". ");
        if needs_newline {
            output.push_str("\n\n");
        }
        return;
    }

    if ctx.in_list_item && !output.is_empty() {
        let needs_newline =
            !output.ends_with('\n') && !output.ends_with("* ") && !output.ends_with("- ") && !output.ends_with(". ");
        if needs_newline {
            trim_trailing_whitespace(output);
            output.push('\n');
        }
    }
}

/// Add appropriate trailing separator after a nested list.
///
/// Nested lists inside list items need trailing newlines to separate
/// from following content. In loose lists, use blank line (\n\n). In tight lists, single newline (\n).
fn add_nested_list_trailing_separator(output: &mut String, ctx: &Context) {
    if !ctx.in_list_item {
        return;
    }

    if ctx.loose_list {
        if !output.ends_with("\n\n") {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.push('\n');
        }
    } else if !output.ends_with('\n') {
        output.push('\n');
    }
}

/// Calculate the nesting depth for a list.
///
/// If we're in a list but NOT in a list item, this is incorrectly nested HTML
/// and we need to increment the depth. If in a list item, the depth was already
/// incremented by the <li> element.
fn calculate_list_nesting_depth(ctx: &Context) -> usize {
    if ctx.in_list && !ctx.in_list_item {
        ctx.list_depth + 1
    } else {
        ctx.list_depth
    }
}

/// Process a list's children, tracking which items had block elements.
///
/// This is used to determine proper spacing between list items.
/// Returns true if the last processed item had block children.
#[allow(clippy::too_many_arguments)]
fn process_list_children(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    is_ordered: bool,
    is_loose: bool,
    nested_depth: usize,
    start_counter: usize,
) {
    let mut counter = start_counter;

    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let children = tag.children();
            {
                for child_handle in children.top().iter() {
                    if let Some(child_node) = child_handle.get(parser) {
                        if let tl::Node::Raw(bytes) = child_node {
                            if bytes.as_utf8_str().trim().is_empty() {
                                continue;
                            }
                        }
                    }

                    let list_ctx = Context {
                        in_ordered_list: is_ordered,
                        list_counter: if is_ordered { counter } else { 0 },
                        in_list: true,
                        list_depth: nested_depth,
                        ul_depth: if is_ordered { ctx.ul_depth } else { ctx.ul_depth + 1 },
                        loose_list: is_loose,
                        prev_item_had_blocks: false,
                        ..ctx.clone()
                    };

                    walk_node(child_handle, parser, output, options, &list_ctx, depth);

                    if is_ordered {
                        if let Some(child_node) = child_handle.get(parser) {
                            if let tl::Node::Tag(child_tag) = child_node {
                                if child_tag.name().as_utf8_str() == "li" {
                                    counter += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Conversion context to track state during traversal
#[derive(Debug, Clone)]
struct Context {
    /// Are we inside a code-like element (pre, code, kbd, samp)?
    in_code: bool,
    /// Current list item counter for ordered lists
    list_counter: usize,
    /// Are we in an ordered list (vs unordered)?
    in_ordered_list: bool,
    /// Track if previous sibling in dl was a dt
    last_was_dt: bool,
    /// Blockquote nesting depth
    blockquote_depth: usize,
    /// Are we inside a table cell (td/th)?
    in_table_cell: bool,
    /// Should we convert block elements as inline?
    convert_as_inline: bool,
    /// Are we inside a list item?
    in_list_item: bool,
    /// List nesting depth (for indentation)
    list_depth: usize,
    /// Unordered list nesting depth (for bullet cycling)
    ul_depth: usize,
    /// Are we inside any list (ul or ol)?
    in_list: bool,
    /// Is this a "loose" list where all items should have blank lines?
    loose_list: bool,
    /// Did a previous list item have block children?
    prev_item_had_blocks: bool,
    /// Are we inside a heading element (h1-h6)?
    in_heading: bool,
    /// Current heading tag (h1, h2, etc.) if in_heading is true
    heading_tag: Option<String>,
    /// Are we inside a paragraph element?
    in_paragraph: bool,
    /// Are we inside a ruby element?
    in_ruby: bool,
    #[cfg(feature = "inline-images")]
    /// Shared collector for inline images when enabled.
    inline_collector: Option<InlineCollectorHandle>,
}

/// Check if a document is an hOCR (HTML-based OCR) document.
///
/// hOCR documents should have metadata extraction disabled to avoid
/// including OCR metadata (system info, capabilities, etc.) in output.
///
/// Detection criteria:
/// - meta tag with name="ocr-system" or name="ocr-capabilities"
/// - Elements with classes: ocr_page, ocrx_word, ocr_carea, ocr_par, ocr_line
fn is_hocr_document(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> bool {
    fn check_node(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> bool {
        if let Some(node) = node_handle.get(parser) {
            match node {
                tl::Node::Tag(tag) => {
                    let tag_name = tag.name().as_utf8_str();

                    if tag_name == "meta" {
                        if let Some(name_attr) = tag.attributes().get("name") {
                            if let Some(name_bytes) = name_attr {
                                let name_value = name_bytes.as_utf8_str();
                                if name_value == "ocr-system" || name_value == "ocr-capabilities" {
                                    return true;
                                }
                            }
                        }
                    }

                    if let Some(class_attr) = tag.attributes().get("class") {
                        if let Some(class_bytes) = class_attr {
                            let class_value = class_bytes.as_utf8_str();
                            if class_value.contains("ocr_page")
                                || class_value.contains("ocrx_word")
                                || class_value.contains("ocr_carea")
                                || class_value.contains("ocr_par")
                                || class_value.contains("ocr_line")
                            {
                                return true;
                            }
                        }
                    }

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            if check_node(child_handle, parser) {
                                return true;
                            }
                        }
                    }
                    false
                }
                _ => false,
            }
        } else {
            false
        }
    }

    check_node(node_handle, parser)
}

/// Extract metadata from HTML document head.
///
/// Extracts comprehensive document metadata including:
/// - title: Document title from <title> tag
/// - meta tags: description, keywords, author, etc.
/// - Open Graph tags: og:title, og:description, og:image, etc.
/// - Twitter Card tags: twitter:card, twitter:title, etc.
/// - base-href: Base URL from <base> tag
/// - canonical: Canonical URL from <link rel="canonical">
/// - link relations: author, license, alternate links
fn extract_metadata(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();

    fn find_head(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> Option<tl::NodeHandle> {
        if let Some(node) = node_handle.get(parser) {
            if let tl::Node::Tag(tag) = node {
                if tag.name().as_utf8_str() == "head" {
                    return Some(*node_handle);
                }
                let children = tag.children();
                {
                    for child_handle in children.top().iter() {
                        if let Some(result) = find_head(child_handle, parser) {
                            return Some(result);
                        }
                    }
                }
            }
        }
        None
    }

    let head_handle = match find_head(node_handle, parser) {
        Some(h) => h,
        None => return metadata,
    };

    if let Some(head_node) = head_handle.get(parser) {
        if let tl::Node::Tag(head_tag) = head_node {
            let children = head_tag.children();
            {
                for child_handle in children.top().iter() {
                    if let Some(child_node) = child_handle.get(parser) {
                        if let tl::Node::Tag(child_tag) = child_node {
                            let tag_name = child_tag.name().as_utf8_str();

                            match tag_name.as_ref() {
                                "title" => {
                                    let title_children = child_tag.children();
                                    {
                                        if let Some(first_child) = title_children.top().iter().next() {
                                            if let Some(text_node) = first_child.get(parser) {
                                                if let tl::Node::Raw(bytes) = text_node {
                                                    let title = text::normalize_whitespace(&bytes.as_utf8_str())
                                                        .trim()
                                                        .to_string();
                                                    if !title.is_empty() {
                                                        metadata.insert("title".to_string(), title);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                "base" => {
                                    if let Some(href_attr) = child_tag.attributes().get("href") {
                                        if let Some(href_bytes) = href_attr {
                                            let href = href_bytes.as_utf8_str().to_string();
                                            if !href.is_empty() {
                                                metadata.insert("base-href".to_string(), href);
                                            }
                                        }
                                    }
                                }
                                "meta" => {
                                    let mut name_attr = None;
                                    let mut property_attr = None;
                                    let mut http_equiv_attr = None;
                                    let mut content_attr = None;

                                    if let Some(attr) = child_tag.attributes().get("name") {
                                        if let Some(bytes) = attr {
                                            name_attr = Some(bytes.as_utf8_str().to_string());
                                        }
                                    }
                                    if let Some(attr) = child_tag.attributes().get("property") {
                                        if let Some(bytes) = attr {
                                            property_attr = Some(bytes.as_utf8_str().to_string());
                                        }
                                    }
                                    if let Some(attr) = child_tag.attributes().get("http-equiv") {
                                        if let Some(bytes) = attr {
                                            http_equiv_attr = Some(bytes.as_utf8_str().to_string());
                                        }
                                    }
                                    if let Some(attr) = child_tag.attributes().get("content") {
                                        if let Some(bytes) = attr {
                                            content_attr = Some(bytes.as_utf8_str().to_string());
                                        }
                                    }

                                    if let Some(content) = content_attr {
                                        if let Some(name) = name_attr {
                                            let key = format!("meta-{}", name.to_lowercase());
                                            metadata.insert(key, content);
                                        } else if let Some(property) = property_attr {
                                            let key = format!("meta-{}", property.to_lowercase().replace(':', "-"));
                                            metadata.insert(key, content);
                                        } else if let Some(http_equiv) = http_equiv_attr {
                                            let key = format!("meta-{}", http_equiv.to_lowercase());
                                            metadata.insert(key, content);
                                        }
                                    }
                                }
                                "link" => {
                                    let mut rel_attr = None;
                                    let mut href_attr = None;

                                    if let Some(attr) = child_tag.attributes().get("rel") {
                                        if let Some(bytes) = attr {
                                            rel_attr = Some(bytes.as_utf8_str().to_string());
                                        }
                                    }
                                    if let Some(attr) = child_tag.attributes().get("href") {
                                        if let Some(bytes) = attr {
                                            href_attr = Some(bytes.as_utf8_str().to_string());
                                        }
                                    }

                                    if let (Some(rel), Some(href)) = (rel_attr, href_attr) {
                                        let rel_lower = rel.to_lowercase();
                                        match rel_lower.as_str() {
                                            "canonical" => {
                                                metadata.insert("canonical".to_string(), href);
                                            }
                                            "author" | "license" | "alternate" => {
                                                metadata.insert(format!("link-{}", rel_lower), href);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    metadata
}

/// Format metadata as YAML frontmatter.
fn format_metadata_frontmatter(metadata: &BTreeMap<String, String>) -> String {
    if metadata.is_empty() {
        return String::new();
    }

    let mut lines = vec!["---".to_string()];
    for (key, value) in metadata {
        // Escape YAML special characters and quote if needed
        let needs_quotes = value.contains(':') || value.contains('#') || value.contains('[') || value.contains(']');
        if needs_quotes {
            let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
            lines.push(format!("{}: \"{}\"", key, escaped));
        } else {
            lines.push(format!("{}: {}", key, value));
        }
    }
    lines.push("---".to_string());

    lines.join("\n") + "\n\n"
}

/// Check if a handle is an empty inline element (abbr, var, ins, dfn, etc. with no text content).
fn is_empty_inline_element(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> bool {
    const EMPTY_WHEN_NO_CONTENT_TAGS: &[&str] = &[
        "abbr", "var", "ins", "dfn", "time", "data", "cite", "q", "mark", "small", "u",
    ];

    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let tag_name = tag.name().as_utf8_str();
            if EMPTY_WHEN_NO_CONTENT_TAGS.contains(&tag_name.as_ref()) {
                return get_text_content(node_handle, parser).trim().is_empty();
            }
        }
    }
    false
}

/// Get the text content of a node and its children.
fn get_text_content(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    let mut text = String::with_capacity(64);
    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Raw(bytes) => {
                text.push_str(&text::decode_html_entities(&bytes.as_utf8_str()));
            }
            tl::Node::Tag(tag) => {
                let children = tag.children();
                {
                    for child_handle in children.top().iter() {
                        text.push_str(&get_text_content(child_handle, parser));
                    }
                }
            }
            _ => {}
        }
    }
    text
}

/// Serialize an element to HTML string (for SVG and Math elements).
fn serialize_element(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let tag_name = tag.name().as_utf8_str();
            let mut html = String::with_capacity(256);
            html.push('<');
            html.push_str(&tag_name);

            // Serialize attributes
            for (key, value_opt) in tag.attributes().iter() {
                html.push(' ');
                html.push_str(&key);
                if let Some(value) = value_opt {
                    html.push_str("=\"");
                    html.push_str(&value);
                    html.push('"');
                }
            }

            let has_children = tag.children().top().len() > 0;
            if !has_children {
                html.push_str(" />");
            } else {
                html.push('>');
                let children = tag.children();
                {
                    for child_handle in children.top().iter() {
                        html.push_str(&serialize_node(child_handle, parser));
                    }
                }
                html.push_str("</");
                html.push_str(&tag_name);
                html.push('>');
            }
            return html;
        }
    }
    String::new()
}

#[cfg(feature = "inline-images")]
fn non_empty_trimmed(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(feature = "inline-images")]
fn handle_inline_data_image(
    collector_ref: &InlineCollectorHandle,
    src: &str,
    alt: &str,
    title: Option<&str>,
    attributes: BTreeMap<String, String>,
) {
    let trimmed_src = src.trim();
    if !trimmed_src.starts_with("data:") {
        return;
    }

    let mut collector = collector_ref.borrow_mut();
    let index = collector.next_index();

    let Some((meta, payload)) = trimmed_src.split_once(',') else {
        collector.warn_skip(index, "missing data URI separator");
        return;
    };

    if payload.trim().is_empty() {
        collector.warn_skip(index, "empty data URI payload");
        return;
    }

    if !meta.starts_with("data:") {
        collector.warn_skip(index, "invalid data URI scheme");
        return;
    }

    let header = &meta["data:".len()..];
    if header.is_empty() {
        collector.warn_skip(index, "missing MIME type");
        return;
    }

    let mut segments = header.split(';');
    let mime = segments.next().unwrap_or("");
    let Some((top_level, subtype_raw)) = mime.split_once('/') else {
        collector.warn_skip(index, "missing MIME subtype");
        return;
    };

    if !top_level.eq_ignore_ascii_case("image") {
        collector.warn_skip(index, format!("unsupported MIME type {mime}"));
        return;
    }

    let subtype_raw = subtype_raw.trim();
    if subtype_raw.is_empty() {
        collector.warn_skip(index, "missing MIME subtype");
        return;
    }

    let subtype_lower = subtype_raw.to_ascii_lowercase();

    let mut is_base64 = false;
    let mut inline_name: Option<String> = None;
    for segment in segments {
        if segment.eq_ignore_ascii_case("base64") {
            is_base64 = true;
        } else if let Some(value) = segment.strip_prefix("name=") {
            inline_name = non_empty_trimmed(value.trim_matches('"'));
        } else if let Some(value) = segment.strip_prefix("filename=") {
            inline_name = non_empty_trimmed(value.trim_matches('"'));
        }
    }

    if !is_base64 {
        collector.warn_skip(index, "missing base64 encoding marker");
        return;
    }

    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let payload_clean = payload.trim();
    let decoded = match STANDARD.decode(payload_clean) {
        Ok(bytes) => bytes,
        Err(_) => {
            collector.warn_skip(index, "invalid base64 payload");
            return;
        }
    };

    if decoded.is_empty() {
        collector.warn_skip(index, "empty base64 payload");
        return;
    }

    let max_size = collector.max_decoded_size();
    if decoded.len() as u64 > max_size {
        collector.warn_skip(
            index,
            format!(
                "decoded payload ({} bytes) exceeds configured max ({})",
                decoded.len(),
                max_size
            ),
        );
        return;
    }

    let format = match subtype_lower.as_str() {
        "png" => InlineImageFormat::Png,
        "jpeg" | "jpg" => InlineImageFormat::Jpeg,
        "gif" => InlineImageFormat::Gif,
        "bmp" => InlineImageFormat::Bmp,
        "webp" => InlineImageFormat::Webp,
        "svg+xml" => InlineImageFormat::Svg,
        other => InlineImageFormat::Other(other.to_string()),
    };

    let description = non_empty_trimmed(alt).or_else(|| title.and_then(non_empty_trimmed));

    let filename_candidate = attributes
        .get("data-filename")
        .cloned()
        .or_else(|| attributes.get("filename").cloned())
        .or_else(|| attributes.get("data-name").cloned())
        .or(inline_name);

    let dimensions = collector.infer_dimensions(index, &decoded, &format);

    let image = collector.build_image(
        decoded,
        format,
        filename_candidate,
        description,
        dimensions,
        InlineImageSource::ImgDataUri,
        attributes,
    );

    collector.push_image(index, image);
}

#[cfg(feature = "inline-images")]
fn handle_inline_svg(
    collector_ref: &InlineCollectorHandle,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    title_opt: Option<String>,
    attributes: BTreeMap<String, String>,
) {
    {
        let borrow = collector_ref.borrow();
        if !borrow.capture_svg() {
            return;
        }
    }

    let mut collector = collector_ref.borrow_mut();
    let index = collector.next_index();

    let serialized = serialize_element(node_handle, parser);
    if serialized.is_empty() {
        collector.warn_skip(index, "unable to serialize SVG element");
        return;
    }

    let data = serialized.into_bytes();
    let max_size = collector.max_decoded_size();
    if data.len() as u64 > max_size {
        collector.warn_skip(
            index,
            format!(
                "serialized SVG payload ({} bytes) exceeds configured max ({})",
                data.len(),
                max_size
            ),
        );
        return;
    }

    let description = attributes
        .get("aria-label")
        .and_then(|value| non_empty_trimmed(value))
        .or_else(|| title_opt.clone().and_then(|t| non_empty_trimmed(&t)));

    let filename_candidate = attributes
        .get("data-filename")
        .cloned()
        .or_else(|| attributes.get("filename").cloned())
        .or_else(|| attributes.get("data-name").cloned());

    let image = collector.build_image(
        data,
        InlineImageFormat::Svg,
        filename_candidate,
        description,
        None,
        InlineImageSource::SvgElement,
        attributes,
    );

    collector.push_image(index, image);
}

/// Serialize a node to HTML string.
fn serialize_node(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Raw(bytes) => bytes.as_utf8_str().to_string(),
            tl::Node::Tag(_) => serialize_element(node_handle, parser),
            _ => String::new(),
        }
    } else {
        String::new()
    }
}

/// Convert HTML to Markdown using tl DOM parser.
pub fn convert_html(html: &str, options: &ConversionOptions) -> Result<String> {
    convert_html_impl(html, options, None)
}

#[cfg(feature = "inline-images")]
pub(crate) fn convert_html_with_inline_collector(
    html: &str,
    options: &ConversionOptions,
    collector: InlineCollectorHandle,
) -> Result<String> {
    convert_html_impl(html, options, Some(collector))
}

#[cfg_attr(not(feature = "inline-images"), allow(unused_variables))]
fn convert_html_impl(
    html: &str,
    options: &ConversionOptions,
    inline_collector: Option<InlineCollectorHandle>,
) -> Result<String> {
    // Fix: tl parser has issues with self-closing tags like <br/>
    // Temporarily convert them to non-self-closing format
    let html = html
        .replace("<br/>", "<br>")
        .replace("<hr/>", "<hr>")
        .replace("<img/>", "<img>");

    // Escape malformed angle brackets in text content to prevent parser failures
    let html = escape_malformed_angle_brackets(&html);

    let html = strip_script_and_style_sections(&html);

    let dom = tl::parse(html.as_ref(), tl::ParserOptions::default())
        .map_err(|_| crate::error::ConversionError::ParseError("Failed to parse HTML".to_string()))?;

    let parser = dom.parser();
    let mut output = String::with_capacity(html.len());

    // Check for hOCR document and extract metadata by checking all top-level children
    let mut is_hocr = false;
    for child_handle in dom.children().iter() {
        if is_hocr_document(child_handle, parser) {
            is_hocr = true;
            break;
        }
    }

    if options.extract_metadata && !options.convert_as_inline && !is_hocr {
        for child_handle in dom.children().iter() {
            let metadata = extract_metadata(child_handle, parser);
            if !metadata.is_empty() {
                let metadata_frontmatter = format_metadata_frontmatter(&metadata);
                output.push_str(&metadata_frontmatter);
                break;
            }
        }
    }

    if is_hocr {
        use crate::hocr::{convert_to_markdown_with_options as convert_hocr_to_markdown, extract_hocr_document};

        let (elements, metadata) = extract_hocr_document(&dom, options.debug);

        // Extract hOCR metadata as YAML frontmatter
        if options.extract_metadata && !options.convert_as_inline {
            let mut metadata_map = BTreeMap::new();
            if let Some(system) = metadata.ocr_system {
                metadata_map.insert("ocr-system".to_string(), system);
            }
            if !metadata.ocr_capabilities.is_empty() {
                metadata_map.insert("ocr-capabilities".to_string(), metadata.ocr_capabilities.join(", "));
            }
            if let Some(pages) = metadata.ocr_number_of_pages {
                metadata_map.insert("ocr-number-of-pages".to_string(), pages.to_string());
            }
            if !metadata.ocr_langs.is_empty() {
                metadata_map.insert("ocr-langs".to_string(), metadata.ocr_langs.join(", "));
            }
            if !metadata.ocr_scripts.is_empty() {
                metadata_map.insert("ocr-scripts".to_string(), metadata.ocr_scripts.join(", "));
            }

            if !metadata_map.is_empty() {
                output.push_str(&format_metadata_frontmatter(&metadata_map));
            }
        }

        let mut markdown = convert_hocr_to_markdown(&elements, true, options.hocr_spatial_tables);

        if markdown.trim().is_empty() {
            return Ok(output);
        }

        markdown.truncate(markdown.trim_end().len());
        output.push_str(&markdown);
        output.push('\n');

        return Ok(output);
    }

    let ctx = Context {
        in_code: false,
        list_counter: 0,
        in_ordered_list: false,
        last_was_dt: false,
        blockquote_depth: 0,
        in_table_cell: false,
        convert_as_inline: options.convert_as_inline,
        in_list_item: false,
        list_depth: 0,
        ul_depth: 0,
        in_list: false,
        loose_list: false,
        prev_item_had_blocks: false,
        in_heading: false,
        heading_tag: None,
        in_paragraph: false,
        in_ruby: false,
        #[cfg(feature = "inline-images")]
        inline_collector: inline_collector.clone(),
    };

    // Walk all top-level children
    for child_handle in dom.children().iter() {
        walk_node(child_handle, parser, &mut output, options, &ctx, 0);
    }

    // Trim trailing blank lines but preserve final newline
    let trimmed = output.trim_end_matches('\n');
    if trimmed.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("{}\n", trimmed))
    }
}

/// Escape malformed angle brackets in HTML that are not part of valid tags.
///
/// This function ensures robust parsing by escaping bare `<` and `>` characters
/// that appear in text content and are not part of HTML tags. This prevents
/// parser failures on malformed HTML like "1<2" or comparisons in text.
///
/// # Examples
///
/// - `1<2` becomes `1&lt;2`
/// - `<div>1<2</div>` becomes `<div>1&lt;2</div>`
/// - `<script>1 < 2</script>` remains unchanged (handled by script stripping)
fn escape_malformed_angle_brackets(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;

    while idx < len {
        if bytes[idx] == b'<' {
            // Check if this is a valid tag start
            if idx + 1 < len {
                let next = bytes[idx + 1];

                // Valid tag patterns: <tagname, </tagname, <!doctype, <!--
                let is_valid_tag = match next {
                    b'!' => {
                        // DOCTYPE or comment
                        idx + 2 < len
                            && (bytes[idx + 2] == b'-'
                                || bytes[idx + 2].is_ascii_alphabetic()
                                || bytes[idx + 2].is_ascii_uppercase())
                    }
                    b'/' => {
                        // Closing tag
                        idx + 2 < len && (bytes[idx + 2].is_ascii_alphabetic() || bytes[idx + 2].is_ascii_uppercase())
                    }
                    b'?' => {
                        // XML declaration
                        true
                    }
                    c if c.is_ascii_alphabetic() || c.is_ascii_uppercase() => {
                        // Opening tag
                        true
                    }
                    _ => false,
                };

                if !is_valid_tag {
                    // This is a bare `<` that should be escaped
                    let out = output.get_or_insert_with(|| String::with_capacity(input.len() + 4));
                    out.push_str(&input[last..idx]);
                    out.push_str("&lt;");
                    last = idx + 1;
                }
            } else {
                // `<` at end of string - escape it
                let out = output.get_or_insert_with(|| String::with_capacity(input.len() + 4));
                out.push_str(&input[last..idx]);
                out.push_str("&lt;");
                last = idx + 1;
            }
        }
        idx += 1;
    }

    if let Some(mut out) = output {
        if last < input.len() {
            out.push_str(&input[last..]);
        }
        Cow::Owned(out)
    } else {
        Cow::Borrowed(input)
    }
}

fn strip_script_and_style_sections(input: &str) -> Cow<'_, str> {
    const TAGS: [&[u8]; 2] = [b"script", b"style"];
    const SVG: &[u8] = b"svg";

    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;
    let mut svg_depth = 0usize;

    while idx < len {
        if bytes[idx] == b'<' {
            if matches_tag_start(bytes, idx + 1, SVG) {
                if let Some(open_end) = find_tag_end(bytes, idx + 1 + SVG.len()) {
                    svg_depth += 1;
                    idx = open_end;
                    continue;
                }
            } else if matches_end_tag_start(bytes, idx + 1, SVG) {
                if let Some(close_end) = find_tag_end(bytes, idx + 2 + SVG.len()) {
                    if svg_depth > 0 {
                        svg_depth = svg_depth.saturating_sub(1);
                    }
                    idx = close_end;
                    continue;
                }
            }

            if svg_depth == 0 {
                let mut handled = false;
                for tag in TAGS {
                    if matches_tag_start(bytes, idx + 1, tag) {
                        if let Some(open_end) = find_tag_end(bytes, idx + 1 + tag.len()) {
                            let remove_end = find_closing_tag(bytes, open_end, tag).unwrap_or(len);
                            let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                            out.push_str(&input[last..idx]);
                            out.push_str(&input[idx..open_end]);
                            out.push_str("</");
                            out.push_str(str::from_utf8(tag).unwrap());
                            out.push('>');

                            last = remove_end;
                            idx = remove_end;
                            handled = true;
                        }
                    }

                    if handled {
                        break;
                    }
                }

                if handled {
                    continue;
                }
            }
        }

        idx += 1;
    }

    if let Some(mut out) = output {
        if last < input.len() {
            out.push_str(&input[last..]);
        }
        Cow::Owned(out)
    } else {
        Cow::Borrowed(input)
    }
}

fn matches_tag_start(bytes: &[u8], mut start: usize, tag: &[u8]) -> bool {
    if start >= bytes.len() {
        return false;
    }

    if start + tag.len() > bytes.len() {
        return false;
    }

    if !bytes[start..start + tag.len()].eq_ignore_ascii_case(tag) {
        return false;
    }

    start += tag.len();

    match bytes.get(start) {
        Some(b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r') => true,
        Some(_) => false,
        None => true,
    }
}

fn find_tag_end(bytes: &[u8], mut idx: usize) -> Option<usize> {
    let len = bytes.len();
    let mut in_quote: Option<u8> = None;

    while idx < len {
        match bytes[idx] {
            b'"' | b'\'' => {
                if let Some(current) = in_quote {
                    if current == bytes[idx] {
                        in_quote = None;
                    }
                } else {
                    in_quote = Some(bytes[idx]);
                }
            }
            b'>' if in_quote.is_none() => return Some(idx + 1),
            _ => {}
        }
        idx += 1;
    }

    None
}

fn find_closing_tag(bytes: &[u8], mut idx: usize, tag: &[u8]) -> Option<usize> {
    let len = bytes.len();
    let mut depth = 1usize;

    while idx < len {
        if bytes[idx] == b'<' {
            if matches_tag_start(bytes, idx + 1, tag) {
                if let Some(next) = find_tag_end(bytes, idx + 1 + tag.len()) {
                    depth += 1;
                    idx = next;
                    continue;
                }
            } else if matches_end_tag_start(bytes, idx + 1, tag) {
                if let Some(close) = find_tag_end(bytes, idx + 2 + tag.len()) {
                    depth -= 1;
                    if depth == 0 {
                        return Some(close);
                    }
                    idx = close;
                    continue;
                }
            }
        }

        idx += 1;
    }

    None
}

fn matches_end_tag_start(bytes: &[u8], start: usize, tag: &[u8]) -> bool {
    if start >= bytes.len() || bytes[start] != b'/' {
        return false;
    }
    matches_tag_start(bytes, start + 1, tag)
}

/// Check if an element is inline (not block-level).
fn is_inline_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "a" | "abbr"
            | "b"
            | "bdi"
            | "bdo"
            | "br"
            | "cite"
            | "code"
            | "data"
            | "dfn"
            | "em"
            | "i"
            | "kbd"
            | "mark"
            | "q"
            | "rp"
            | "rt"
            | "ruby"
            | "s"
            | "samp"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "sup"
            | "time"
            | "u"
            | "var"
            | "wbr"
            | "del"
            | "ins"
            | "img"
            | "map"
            | "area"
            | "audio"
            | "video"
            | "picture"
            | "source"
            | "track"
            | "embed"
            | "object"
            | "param"
            | "input"
            | "label"
            | "button"
            | "select"
            | "textarea"
            | "output"
            | "progress"
            | "meter"
    )
}

/// Get the next sibling element tag name if it exists and is an element.
///
/// TODO: This function needs architectural changes for tl API.
/// tl doesn't provide parent references like html5ever did.
/// Options:
/// 1. Pass sibling context from parent when iterating
/// 2. Build a sibling lookup map during parsing
/// 3. Accept loss of this optimization
///
/// For now, returning None to allow compilation.
fn get_next_sibling_tag(_node_handle: &tl::NodeHandle, _parser: &tl::Parser) -> Option<String> {
    // TODO: Implement sibling lookup with tl API
    // This affects whitespace handling between inline elements
    None
}

/// Recursively walk DOM nodes and convert to Markdown.
#[allow(clippy::only_used_in_recursion)]
fn walk_node(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
) {
    let Some(node) = node_handle.get(parser) else { return };

    match node {
        tl::Node::Raw(bytes) => {
            let mut text = text::decode_html_entities(&bytes.as_utf8_str());

            if text.is_empty() {
                return;
            }

            if options.strip_newlines {
                text = text.replace(['\r', '\n'], " ");
            }

            if text.trim().is_empty() {
                if ctx.in_code {
                    output.push_str(&text);
                    return;
                }

                if options.whitespace_mode == crate::options::WhitespaceMode::Strict {
                    if ctx.convert_as_inline || ctx.in_table_cell || ctx.in_list_item {
                        output.push_str(&text);
                        return;
                    }
                    if text.contains("\n\n") || text.contains("\r\n\r\n") {
                        if !output.ends_with("\n\n") {
                            output.push('\n');
                        }
                        return;
                    }
                    output.push_str(&text);
                    return;
                }

                if text.trim().is_empty() && text.contains('\n') {
                    if !output.ends_with("\n\n") {
                        if let Some(next_tag) = get_next_sibling_tag(node_handle, parser) {
                            if is_inline_element(&next_tag) {
                                output.push(' ');
                                return;
                            }
                        }
                    }
                    return;
                }

                let skip_whitespace = output.ends_with("\n\n")
                    || output.ends_with("* ")
                    || output.ends_with("- ")
                    || output.ends_with(". ")
                    || output.ends_with("] ");

                let should_preserve = ctx.convert_as_inline || ctx.in_table_cell || !skip_whitespace;

                if should_preserve {
                    output.push(' ');
                }
                return;
            }

            let processed_text = if ctx.in_code || ctx.in_table_cell || ctx.in_ruby {
                if ctx.in_code || ctx.in_ruby {
                    text
                } else if options.whitespace_mode == crate::options::WhitespaceMode::Normalized {
                    text::normalize_whitespace(&text)
                } else {
                    text
                }
            } else if options.whitespace_mode == crate::options::WhitespaceMode::Strict {
                text::escape(
                    &text,
                    options.escape_misc,
                    options.escape_asterisks,
                    options.escape_underscores,
                    options.escape_ascii,
                )
            } else {
                let has_trailing_single_newline =
                    text.ends_with('\n') && !text.ends_with("\n\n") && !text.ends_with("\r\n\r\n");

                let normalized_text = text::normalize_whitespace(&text);

                let (prefix, suffix, core) = text::chomp(&normalized_text);

                let skip_prefix = output.ends_with("\n\n")
                    || output.ends_with("* ")
                    || output.ends_with("- ")
                    || output.ends_with(". ")
                    || output.ends_with("] ");

                let mut final_text = String::new();
                if !skip_prefix && !prefix.is_empty() {
                    final_text.push_str(prefix);
                }

                let escaped_core = text::escape(
                    core,
                    options.escape_misc,
                    options.escape_asterisks,
                    options.escape_underscores,
                    options.escape_ascii,
                );
                final_text.push_str(&escaped_core);

                if !suffix.is_empty() {
                    final_text.push_str(suffix);
                } else if has_trailing_single_newline {
                    let at_paragraph_break = output.ends_with("\n\n");
                    if options.debug {
                        eprintln!(
                            "[DEBUG] Text had trailing single newline that was chomped, at_paragraph_break={}",
                            at_paragraph_break
                        );
                    }
                    if !at_paragraph_break {
                        if let Some(next_tag) = get_next_sibling_tag(node_handle, parser) {
                            if options.debug {
                                eprintln!(
                                    "[DEBUG] Next sibling tag: {}, is_inline: {}",
                                    next_tag,
                                    is_inline_element(&next_tag)
                                );
                            }
                            if is_inline_element(&next_tag) {
                                final_text.push(' ');
                            }
                        }
                    }
                }

                final_text
            };

            if ctx.in_list_item && processed_text.contains("\n\n") {
                let parts: Vec<&str> = processed_text.split("\n\n").collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        output.push_str("\n\n");
                        output.push_str(&" ".repeat(4 * ctx.list_depth));
                    }
                    output.push_str(part.trim());
                }
            } else {
                output.push_str(&processed_text);
            }
        }

        tl::Node::Tag(tag) => {
            let tag_name = tag.name().as_utf8_str();

            if options.strip_tags.iter().any(|t| t.as_str() == tag_name) {
                let children = tag.children();
                {
                    for child_handle in children.top().iter() {
                        walk_node(child_handle, parser, output, options, ctx, depth + 1);
                    }
                }
                return;
            }

            match tag_name.as_ref() {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = tag_name.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1) as usize;

                    let mut text = String::new();
                    let heading_ctx = Context {
                        in_heading: true,
                        convert_as_inline: true,
                        heading_tag: Some(tag_name.to_string()),
                        ..ctx.clone()
                    };
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, &heading_ctx, depth + 1);
                        }
                    }
                    let text = text.trim();

                    if !text.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(text);
                            return;
                        }

                        if ctx.in_table_cell {
                            let is_table_continuation = !output.is_empty()
                                && !output.ends_with('|')
                                && !output.ends_with(' ')
                                && !output.ends_with("<br>");
                            if is_table_continuation {
                                output.push_str("<br>");
                            }
                            output.push_str(text);
                            return;
                        }

                        if !output.is_empty() && !output.ends_with("\n\n") {
                            if output.ends_with('\n') {
                                output.push('\n');
                            } else {
                                trim_trailing_whitespace(output);
                                output.push_str("\n\n");
                            }
                        }

                        let heading_suffix = if ctx.blockquote_depth > 0 { "\n" } else { "\n\n" };

                        match options.heading_style {
                            HeadingStyle::Underlined => {
                                if level == 1 {
                                    output.push_str(text);
                                    output.push('\n');
                                    output.push_str(&"=".repeat(text.len()));
                                    output.push_str(heading_suffix);
                                } else if level == 2 {
                                    output.push_str(text);
                                    output.push('\n');
                                    output.push_str(&"-".repeat(text.len()));
                                    output.push_str(heading_suffix);
                                } else {
                                    output.push_str(&"#".repeat(level));
                                    output.push(' ');
                                    output.push_str(text);
                                    output.push_str(heading_suffix);
                                }
                            }
                            HeadingStyle::Atx => {
                                output.push_str(&"#".repeat(level));
                                output.push(' ');
                                output.push_str(text);
                                output.push_str(heading_suffix);
                            }
                            HeadingStyle::AtxClosed => {
                                output.push_str(&"#".repeat(level));
                                output.push(' ');
                                output.push_str(text);
                                output.push(' ');
                                output.push_str(&"#".repeat(level));
                                output.push_str(heading_suffix);
                            }
                        }
                    }
                }

                "p" => {
                    let content_start_pos = output.len();

                    let is_table_continuation =
                        ctx.in_table_cell && !output.is_empty() && !output.ends_with('|') && !output.ends_with("<br>");

                    let is_list_continuation = ctx.in_list_item
                        && !output.is_empty()
                        && !output.ends_with("* ")
                        && !output.ends_with("- ")
                        && !output.ends_with(". ");

                    let after_code_block = output.ends_with("```\n");
                    let needs_leading_sep = !ctx.in_table_cell
                        && !ctx.in_list_item
                        && !ctx.convert_as_inline
                        && ctx.blockquote_depth == 0
                        && !output.is_empty()
                        && !output.ends_with("\n\n")
                        && !after_code_block;

                    if is_table_continuation {
                        trim_trailing_whitespace(output);
                        output.push_str("<br>");
                    } else if is_list_continuation {
                        add_list_continuation_indent(output, ctx.list_depth, true, options);
                    } else if needs_leading_sep {
                        trim_trailing_whitespace(output);
                        output.push_str("\n\n");
                    }

                    let p_ctx = Context {
                        in_paragraph: true,
                        ..ctx.clone()
                    };

                    let children = tag.children();
                    {
                        let child_handles: Vec<_> = children.top().iter().collect();
                        for (i, child_handle) in child_handles.iter().enumerate() {
                            // Skip whitespace-only text nodes between empty inline elements
                            if let Some(node) = child_handle.get(parser) {
                                if let tl::Node::Raw(bytes) = node {
                                    let text = bytes.as_utf8_str();
                                    if text.trim().is_empty() && i > 0 && i < child_handles.len() - 1 {
                                        let prev = &child_handles[i - 1];
                                        let next = &child_handles[i + 1];
                                        if is_empty_inline_element(prev, parser)
                                            && is_empty_inline_element(next, parser)
                                        {
                                            continue;
                                        }
                                    }
                                }
                            }
                            walk_node(child_handle, parser, output, options, &p_ctx, depth + 1);
                        }
                    }

                    let has_content = output.len() > content_start_pos;

                    if has_content && !ctx.convert_as_inline && !ctx.in_table_cell {
                        output.push_str("\n\n");
                    }
                }

                "strong" | "b" => {
                    if ctx.in_code {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(64);
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                            }
                        }
                        let (prefix, suffix, trimmed) = chomp_inline(&content);
                        if !content.trim().is_empty() {
                            output.push_str(prefix);
                            output.push(options.strong_em_symbol);
                            output.push(options.strong_em_symbol);
                            output.push_str(trimmed);
                            output.push(options.strong_em_symbol);
                            output.push(options.strong_em_symbol);
                            output.push_str(suffix);
                        } else if !content.is_empty() {
                            output.push_str(prefix);
                            output.push_str(suffix);
                        }
                    }
                }

                "em" | "i" => {
                    if ctx.in_code {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(64);
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                            }
                        }
                        let (prefix, suffix, trimmed) = chomp_inline(&content);
                        if !content.trim().is_empty() {
                            output.push_str(prefix);
                            output.push(options.strong_em_symbol);
                            output.push_str(trimmed);
                            output.push(options.strong_em_symbol);
                            output.push_str(suffix);
                        } else if !content.is_empty() {
                            output.push_str(prefix);
                            output.push_str(suffix);
                        }
                    }
                }

                "a" => {
                    let href_attr = tag
                        .attributes()
                        .get("href")
                        .flatten()
                        .map(|v| text::decode_html_entities(&v.as_utf8_str()));
                    let title = tag.attributes().get("title").flatten().map(|v| v.as_utf8_str());

                    if let Some(href) = href_attr {
                        let raw_text = get_text_content(node_handle, parser).trim().to_string();

                        let is_autolink = options.autolinks
                            && !options.default_title
                            && !href.is_empty()
                            && (raw_text == href.as_ref() || (href.starts_with("mailto:") && raw_text == href[7..]));

                        if is_autolink {
                            output.push('<');
                            if href.starts_with("mailto:") && raw_text == href[7..] {
                                output.push_str(&raw_text);
                            } else {
                                output.push_str(&href);
                            }
                            output.push('>');
                        } else {
                            output.push('[');
                            let children = tag.children();
                            {
                                for child_handle in children.top().iter() {
                                    walk_node(child_handle, parser, output, options, ctx, depth + 1);
                                }
                            }
                            output.push_str("](");

                            if href.is_empty() {
                                output.push_str("<>");
                            } else if href.contains(' ') || href.contains('\n') {
                                output.push('<');
                                output.push_str(&href);
                                output.push('>');
                            } else {
                                let open_count = href.chars().filter(|&c| c == '(').count();
                                let close_count = href.chars().filter(|&c| c == ')').count();

                                if open_count == close_count {
                                    output.push_str(&href);
                                } else {
                                    // Only allocate when escaping is needed
                                    let escaped_href = href.replace("(", "\\(").replace(")", "\\)");
                                    output.push_str(&escaped_href);
                                }
                            }

                            if let Some(title_text) = title {
                                output.push_str(" \"");
                                // Only allocate if quotes are present
                                if title_text.contains('"') {
                                    let escaped_title = title_text.replace('"', "\\\"");
                                    output.push_str(&escaped_title);
                                } else {
                                    output.push_str(&title_text);
                                }
                                output.push('"');
                            } else if options.default_title && raw_text == href.as_ref() {
                                output.push_str(" \"");
                                // Only allocate if quotes are present
                                if href.contains('"') {
                                    let escaped_href = href.replace('"', "\\\"");
                                    output.push_str(&escaped_href);
                                } else {
                                    output.push_str(&href);
                                }
                                output.push('"');
                            }

                            output.push(')');
                        }
                    } else {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1);
                            }
                        }
                    }
                }

                "img" => {
                    use std::borrow::Cow;

                    let src = tag
                        .attributes()
                        .get("src")
                        .flatten()
                        .map(|v| v.as_utf8_str())
                        .unwrap_or(Cow::Borrowed(""));

                    let alt = tag
                        .attributes()
                        .get("alt")
                        .flatten()
                        .map(|v| v.as_utf8_str())
                        .unwrap_or(Cow::Borrowed(""));

                    let title = tag.attributes().get("title").flatten().map(|v| v.as_utf8_str());

                    let width = tag.attributes().get("width").flatten().map(|v| v.as_utf8_str());

                    let height = tag.attributes().get("height").flatten().map(|v| v.as_utf8_str());

                    #[cfg(feature = "inline-images")]
                    if let Some(ref collector_ref) = ctx.inline_collector {
                        let mut attributes_map = BTreeMap::new();
                        for (key, value_opt) in tag.attributes().iter() {
                            let key_str = key.to_string();
                            let keep = key_str == "width"
                                || key_str == "height"
                                || key_str == "filename"
                                || key_str == "aria-label"
                                || key_str.starts_with("data-");
                            if keep {
                                let value = value_opt.map(|value| value.to_string()).unwrap_or_default();
                                attributes_map.insert(key_str, value);
                            }
                        }
                        handle_inline_data_image(
                            collector_ref,
                            src.as_ref(),
                            alt.as_ref(),
                            title.as_deref(),
                            attributes_map,
                        );
                    }

                    let keep_as_markdown = ctx.in_heading
                        && ctx
                            .heading_tag
                            .as_ref()
                            .is_some_and(|tag| options.keep_inline_images_in.iter().any(|t| t == tag));

                    let should_use_alt_text = !keep_as_markdown
                        && (ctx.convert_as_inline
                            || (ctx.in_heading
                                && ctx
                                    .heading_tag
                                    .as_ref()
                                    .map_or(true, |tag| !options.keep_inline_images_in.iter().any(|t| t == tag))));

                    if should_use_alt_text {
                        output.push_str(&alt);
                    } else if width.is_some() || height.is_some() {
                        output.push_str("<img src='");
                        output.push_str(&src);
                        output.push_str("' alt='");
                        output.push_str(&alt);
                        output.push_str("' title='");
                        if let Some(title_text) = &title {
                            output.push_str(title_text);
                        }
                        output.push('\'');
                        if let Some(w) = &width {
                            output.push_str(" width='");
                            output.push_str(w);
                            output.push('\'');
                        }
                        if let Some(h) = &height {
                            output.push_str(" height='");
                            output.push_str(h);
                            output.push('\'');
                        }
                        output.push_str(" />");
                    } else {
                        output.push_str("![");
                        output.push_str(&alt);
                        output.push_str("](");
                        output.push_str(&src);
                        if let Some(title_text) = title {
                            output.push_str(" \"");
                            output.push_str(&title_text);
                            output.push('"');
                        }
                        output.push(')');
                    }
                }

                "mark" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1);
                            }
                        }
                    } else {
                        use crate::options::HighlightStyle;
                        match options.highlight_style {
                            HighlightStyle::DoubleEqual => {
                                output.push_str("==");
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1);
                                    }
                                }
                                output.push_str("==");
                            }
                            HighlightStyle::Html => {
                                output.push_str("<mark>");
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1);
                                    }
                                }
                                output.push_str("</mark>");
                            }
                            HighlightStyle::Bold => {
                                let symbol = options.strong_em_symbol.to_string().repeat(2);
                                output.push_str(&symbol);
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1);
                                    }
                                }
                                output.push_str(&symbol);
                            }
                            HighlightStyle::None => {
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1);
                                    }
                                }
                            }
                        }
                    }
                }

                "del" | "s" => {
                    if ctx.in_code {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(32);
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                            }
                        }
                        let (prefix, suffix, trimmed) = chomp_inline(&content);
                        if !content.trim().is_empty() {
                            output.push_str(prefix);
                            output.push_str("~~");
                            output.push_str(trimmed);
                            output.push_str("~~");
                            output.push_str(suffix);
                        } else if !content.is_empty() {
                            output.push_str(prefix);
                            output.push_str(suffix);
                        }
                    }
                }

                "ins" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let (prefix, suffix, trimmed) = chomp_inline(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push_str("==");
                        output.push_str(trimmed);
                        output.push_str("==");
                        output.push_str(suffix);
                    }
                }

                "u" | "small" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }
                }

                "sub" => {
                    if !ctx.in_code && !options.sub_symbol.is_empty() {
                        output.push_str(&options.sub_symbol);
                    }
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }
                    if !ctx.in_code && !options.sub_symbol.is_empty() {
                        if options.sub_symbol.starts_with('<') && !options.sub_symbol.starts_with("</") {
                            output.push_str(&options.sub_symbol.replace('<', "</"));
                        } else {
                            output.push_str(&options.sub_symbol);
                        }
                    }
                }

                "sup" => {
                    if !ctx.in_code && !options.sup_symbol.is_empty() {
                        output.push_str(&options.sup_symbol);
                    }
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }
                    if !ctx.in_code && !options.sup_symbol.is_empty() {
                        if options.sup_symbol.starts_with('<') && !options.sup_symbol.starts_with("</") {
                            output.push_str(&options.sup_symbol.replace('<', "</"));
                        } else {
                            output.push_str(&options.sup_symbol);
                        }
                    }
                }

                "kbd" | "samp" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, &code_ctx, depth + 1);
                        }
                    }
                    let normalized = text::normalize_whitespace(&content);
                    let (prefix, suffix, trimmed) = chomp_inline(&normalized);
                    if !content.trim().is_empty() {
                        output.push_str(prefix);
                        output.push('`');
                        output.push_str(trimmed);
                        output.push('`');
                        output.push_str(suffix);
                    } else if !content.is_empty() {
                        output.push_str(prefix);
                        output.push_str(suffix);
                    }
                }

                "var" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let (prefix, suffix, trimmed) = chomp_inline(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push(options.strong_em_symbol);
                        output.push_str(trimmed);
                        output.push(options.strong_em_symbol);
                        output.push_str(suffix);
                    }
                }

                "dfn" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let (prefix, suffix, trimmed) = chomp_inline(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push(options.strong_em_symbol);
                        output.push_str(trimmed);
                        output.push(options.strong_em_symbol);
                        output.push_str(suffix);
                    }
                }

                "abbr" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = content.trim();

                    if !trimmed.is_empty() {
                        output.push_str(trimmed);

                        if let Some(title) = tag.attributes().get("title").flatten().map(|v| v.as_utf8_str()) {
                            let trimmed_title = title.trim();
                            if !trimmed_title.is_empty() {
                                output.push_str(" (");
                                output.push_str(trimmed_title);
                                output.push(')');
                            }
                        }
                    }
                }

                "time" | "data" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }
                }

                "wbr" => {}

                "code" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };

                    if !ctx.in_code {
                        let mut content = String::with_capacity(32);
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, &mut content, options, &code_ctx, depth + 1);
                            }
                        }

                        let trimmed = &content;

                        if !content.trim().is_empty() {
                            let contains_backtick = trimmed.contains('`');

                            let needs_delimiter_spaces = {
                                let first_char = trimmed.chars().next();
                                let last_char = trimmed.chars().last();
                                let starts_with_space = first_char == Some(' ');
                                let ends_with_space = last_char == Some(' ');
                                let starts_with_backtick = first_char == Some('`');
                                let ends_with_backtick = last_char == Some('`');
                                let all_spaces = trimmed.chars().all(|c| c == ' ');

                                all_spaces
                                    || starts_with_backtick
                                    || ends_with_backtick
                                    || (starts_with_space && ends_with_space && contains_backtick)
                            };

                            let (num_backticks, needs_spaces) = if contains_backtick {
                                let max_consecutive = trimmed
                                    .chars()
                                    .fold((0, 0), |(max, current), c| {
                                        if c == '`' {
                                            let new_current = current + 1;
                                            (max.max(new_current), new_current)
                                        } else {
                                            (max, 0)
                                        }
                                    })
                                    .0;
                                let num = if max_consecutive == 1 { 2 } else { 1 };
                                (num, needs_delimiter_spaces)
                            } else {
                                (1, needs_delimiter_spaces)
                            };

                            for _ in 0..num_backticks {
                                output.push('`');
                            }
                            if needs_spaces {
                                output.push(' ');
                            }
                            output.push_str(trimmed);
                            if needs_spaces {
                                output.push(' ');
                            }
                            for _ in 0..num_backticks {
                                output.push('`');
                            }
                        }
                    } else {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, &code_ctx, depth + 1);
                            }
                        }
                    }
                }

                "pre" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };

                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, &code_ctx, depth + 1);
                        }
                    }

                    if !content.is_empty() {
                        match options.code_block_style {
                            crate::options::CodeBlockStyle::Indented => {
                                if !ctx.convert_as_inline && !output.is_empty() && !output.ends_with("\n\n") {
                                    if output.ends_with('\n') {
                                        output.push('\n');
                                    } else {
                                        output.push_str("\n\n");
                                    }
                                }

                                let indented = content
                                    .lines()
                                    .map(|line| {
                                        if line.is_empty() {
                                            String::new()
                                        } else {
                                            format!("    {}", line)
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                output.push_str(&indented);

                                output.push_str("\n\n");
                            }
                            crate::options::CodeBlockStyle::Backticks | crate::options::CodeBlockStyle::Tildes => {
                                if !ctx.convert_as_inline && !output.is_empty() && !output.ends_with("\n\n") {
                                    if output.ends_with('\n') {
                                        output.push('\n');
                                    } else {
                                        output.push_str("\n\n");
                                    }
                                }

                                let fence = if options.code_block_style == crate::options::CodeBlockStyle::Backticks {
                                    "```"
                                } else {
                                    "~~~"
                                };

                                output.push_str(fence);
                                if !options.code_language.is_empty() {
                                    output.push_str(&options.code_language);
                                }
                                output.push('\n');
                                output.push_str(&content);
                                output.push('\n');
                                output.push_str(fence);
                                output.push('\n');
                            }
                        }
                    }
                }

                "blockquote" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1);
                            }
                        }
                        return;
                    }

                    let cite = tag
                        .attributes()
                        .get("cite")
                        .flatten()
                        .map(|v| v.as_utf8_str().to_string());

                    let blockquote_ctx = Context {
                        blockquote_depth: ctx.blockquote_depth + 1,
                        ..ctx.clone()
                    };
                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, &blockquote_ctx, depth + 1);
                        }
                    }

                    let trimmed_content = content.trim();

                    if !trimmed_content.is_empty() {
                        // Handle spacing before blockquote
                        if ctx.blockquote_depth > 0 {
                            // Nested blockquote
                            output.push_str("\n\n\n");
                        } else if !output.is_empty() {
                            // CommonMark: blockquote needs only single newline before it
                            if !output.ends_with('\n') {
                                output.push('\n');
                            } else if output.ends_with("\n\n") {
                                // Remove one trailing newline (paragraph already added \n\n)
                                output.truncate(output.len() - 1);
                            }
                        }
                        // If output.is_empty(), add nothing (no leading newline)

                        let prefix = "> ";

                        for line in trimmed_content.lines() {
                            output.push_str(prefix);
                            output.push_str(line.trim());
                            output.push('\n');
                        }

                        // Add spacing after blockquote
                        if let Some(url) = cite {
                            output.push('\n');
                            output.push_str("— <");
                            output.push_str(&url);
                            output.push_str(">\n\n");
                        } else {
                            // Add single newline after blockquote (CommonMark spacing)
                            output.push('\n');
                        }
                    }
                }

                "br" => {
                    if ctx.in_heading {
                        trim_trailing_whitespace(output);
                        output.push_str("  ");
                    } else {
                        use crate::options::NewlineStyle;
                        match options.newline_style {
                            NewlineStyle::Spaces => output.push_str("  \n"),
                            NewlineStyle::Backslash => output.push_str("\\\n"),
                        }
                    }
                }

                "hr" => {
                    // CommonMark: hr needs to be on its own line but doesn't need blank line before
                    if !output.is_empty() {
                        if !output.ends_with('\n') {
                            output.push('\n');
                        } else if output.ends_with("\n\n") {
                            // Remove extra newline (e.g., after blockquote)
                            output.truncate(output.len() - 1);
                        }
                    }
                    output.push_str("---\n");
                }

                "ul" => {
                    add_list_leading_separator(output, ctx);

                    let nested_depth = calculate_list_nesting_depth(ctx);
                    let is_loose = is_loose_list(node_handle, parser);

                    process_list_children(
                        node_handle,
                        parser,
                        output,
                        options,
                        ctx,
                        depth,
                        false,
                        is_loose,
                        nested_depth,
                        1,
                    );

                    add_nested_list_trailing_separator(output, ctx);
                }

                "ol" => {
                    add_list_leading_separator(output, ctx);

                    let nested_depth = calculate_list_nesting_depth(ctx);
                    let is_loose = is_loose_list(node_handle, parser);

                    let start = tag
                        .attributes()
                        .get("start")
                        .flatten()
                        .and_then(|v| v.as_utf8_str().parse::<usize>().ok())
                        .unwrap_or(1);

                    process_list_children(
                        node_handle,
                        parser,
                        output,
                        options,
                        ctx,
                        depth,
                        true,
                        is_loose,
                        nested_depth,
                        start,
                    );

                    add_nested_list_trailing_separator(output, ctx);
                }

                "li" => {
                    if ctx.list_depth > 0 {
                        let indent = match options.list_indent_type {
                            ListIndentType::Tabs => "\t".repeat(ctx.list_depth),
                            ListIndentType::Spaces => " ".repeat(ctx.list_depth * options.list_indent_width),
                        };
                        output.push_str(&indent);
                    }

                    let mut has_block_children = false;
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                let tag_name = child_tag.name().as_utf8_str();
                                if matches!(
                                    tag_name.as_ref(),
                                    "p" | "div"
                                        | "blockquote"
                                        | "pre"
                                        | "table"
                                        | "h1"
                                        | "h2"
                                        | "h3"
                                        | "h4"
                                        | "h5"
                                        | "h6"
                                        | "hr"
                                        | "dl"
                                ) {
                                    has_block_children = true;
                                    break;
                                }
                            }
                        }
                    }

                    fn find_checkbox<'a>(
                        node_handle: &tl::NodeHandle,
                        parser: &'a tl::Parser<'a>,
                    ) -> Option<(bool, tl::NodeHandle)> {
                        if let Some(tl::Node::Tag(node_tag)) = node_handle.get(parser) {
                            if node_tag.name().as_utf8_str() == "input" {
                                let input_type = node_tag.attributes().get("type").flatten().map(|v| v.as_utf8_str());

                                if input_type.as_deref() == Some("checkbox") {
                                    let checked = node_tag.attributes().get("checked").is_some();
                                    return Some((checked, *node_handle));
                                }
                            }

                            let children = node_tag.children();
                            {
                                for child_handle in children.top().iter() {
                                    if let Some(result) = find_checkbox(child_handle, parser) {
                                        return Some(result);
                                    }
                                }
                            }
                        }
                        None
                    }

                    let (is_task_list, task_checked, checkbox_node) =
                        if let Some((checked, node)) = find_checkbox(node_handle, parser) {
                            (true, checked, Some(node))
                        } else {
                            (false, false, None)
                        };

                    let li_ctx = Context {
                        in_list_item: true,
                        list_depth: ctx.list_depth + 1,
                        ..ctx.clone()
                    };

                    if is_task_list {
                        output.push('-');
                        output.push(' ');
                        output.push_str(if task_checked { "[x]" } else { "[ ]" });

                        fn is_checkbox_node(node_handle: &tl::NodeHandle, checkbox: &Option<tl::NodeHandle>) -> bool {
                            if let Some(cb) = checkbox {
                                node_handle == cb
                            } else {
                                false
                            }
                        }

                        fn contains_checkbox<'a>(
                            node_handle: &tl::NodeHandle,
                            parser: &'a tl::Parser<'a>,
                            checkbox: &Option<tl::NodeHandle>,
                        ) -> bool {
                            if is_checkbox_node(node_handle, checkbox) {
                                return true;
                            }
                            if let Some(tl::Node::Tag(node_tag)) = node_handle.get(parser) {
                                let children = node_tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        if contains_checkbox(child_handle, parser, checkbox) {
                                            return true;
                                        }
                                    }
                                }
                            }
                            false
                        }

                        fn render_li_content<'a>(
                            node_handle: &tl::NodeHandle,
                            parser: &'a tl::Parser<'a>,
                            output: &mut String,
                            options: &ConversionOptions,
                            ctx: &Context,
                            depth: usize,
                            checkbox: &Option<tl::NodeHandle>,
                        ) {
                            if is_checkbox_node(node_handle, checkbox) {
                                return;
                            }

                            if contains_checkbox(node_handle, parser, checkbox) {
                                if let Some(tl::Node::Tag(node_tag)) = node_handle.get(parser) {
                                    let children = node_tag.children();
                                    {
                                        for child_handle in children.top().iter() {
                                            render_li_content(
                                                child_handle,
                                                parser,
                                                output,
                                                options,
                                                ctx,
                                                depth,
                                                checkbox,
                                            );
                                        }
                                    }
                                }
                            } else {
                                walk_node(node_handle, parser, output, options, ctx, depth);
                            }
                        }

                        let mut task_text = String::new();
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                render_li_content(
                                    child_handle,
                                    parser,
                                    &mut task_text,
                                    options,
                                    &li_ctx,
                                    depth + 1,
                                    &checkbox_node,
                                );
                            }
                        }
                        output.push(' ');
                        let trimmed_task = task_text.trim();
                        if !trimmed_task.is_empty() {
                            output.push_str(trimmed_task);
                        }
                    } else {
                        if !ctx.in_table_cell {
                            if ctx.in_ordered_list {
                                output.push_str(&format!("{}. ", ctx.list_counter));
                            } else {
                                let bullets: Vec<char> = options.bullets.chars().collect();
                                let bullet_index = if ctx.ul_depth > 0 { ctx.ul_depth - 1 } else { 0 };
                                let bullet = bullets.get(bullet_index % bullets.len()).copied().unwrap_or('*');
                                output.push(bullet);
                                output.push(' ');
                            }
                        }

                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, &li_ctx, depth + 1);
                            }
                        }

                        trim_trailing_whitespace(output);
                    }

                    if !ctx.in_table_cell {
                        if has_block_children || ctx.loose_list || ctx.prev_item_had_blocks {
                            if !output.ends_with("\n\n") {
                                if output.ends_with('\n') {
                                    output.push('\n');
                                } else {
                                    output.push_str("\n\n");
                                }
                            }
                        } else if !output.ends_with('\n') {
                            output.push('\n');
                        }
                    }
                }

                "table" => {
                    if !output.ends_with("\n\n") {
                        if output.is_empty() || !output.ends_with('\n') {
                            output.push_str("\n\n");
                        } else {
                            output.push('\n');
                        }
                    }
                    convert_table(node_handle, parser, output, options, ctx);
                    output.push('\n');
                }

                "thead" | "tbody" | "tfoot" | "tr" | "th" | "td" => {}

                "caption" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1);
                        }
                    }
                    let text = text.trim();
                    if !text.is_empty() {
                        // Escape dashes in captions to avoid confusion with table separators
                        let escaped_text = text.replace('-', r"\-");
                        output.push('*');
                        output.push_str(&escaped_text);
                        output.push_str("*\n\n");
                    }
                }

                "colgroup" | "col" => {}

                "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth);
                            }
                        }
                        return;
                    }

                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth);
                        }
                    }
                    if content.trim().is_empty() {
                        return;
                    }

                    if !output.is_empty() && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }
                    output.push_str(&content);
                    if content.ends_with('\n') && !content.ends_with("\n\n") {
                        output.push('\n');
                    } else if !content.ends_with('\n') {
                        output.push_str("\n\n");
                    }
                }

                "figure" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth);
                            }
                        }
                        return;
                    }

                    if !output.is_empty() && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }

                    let mut figure_content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut figure_content, options, ctx, depth);
                        }
                    }

                    let trimmed = figure_content.trim_start_matches('\n');
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                        if !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                    }
                }

                "figcaption" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1);
                        }
                    }
                    let text = text.trim();
                    if !text.is_empty() {
                        if !output.is_empty() {
                            if output.ends_with("```\n") {
                                output.push('\n');
                            } else if !output.ends_with("\n\n") {
                                output.push_str("\n\n");
                            }
                        }
                        output.push('*');
                        output.push_str(text);
                        output.push_str("*\n\n");
                    }
                }

                "hgroup" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth);
                        }
                    }
                }

                "cite" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            output.push('*');
                            output.push_str(trimmed);
                            output.push('*');
                        }
                    }
                }

                "q" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            output.push('"');
                            // Escape backslashes first, then quotes
                            let escaped = trimmed.replace('\\', r"\\").replace('"', r#"\""#);
                            output.push_str(&escaped);
                            output.push('"');
                        }
                    }
                }

                "dl" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth);
                            }
                        }
                        return;
                    }

                    let mut content = String::new();
                    let mut in_dt_group = false;
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            let (is_dt, is_dd) = if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                let tag_name = child_tag.name().as_utf8_str();
                                (tag_name == "dt", tag_name == "dd")
                            } else {
                                (false, false)
                            };

                            let child_ctx = Context {
                                last_was_dt: in_dt_group && is_dd,
                                ..ctx.clone()
                            };
                            walk_node(child_handle, parser, &mut content, options, &child_ctx, depth);

                            if is_dt {
                                in_dt_group = true;
                            } else if !is_dd {
                                in_dt_group = false;
                            }
                        }
                    }

                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "dt" => {
                    let mut content = String::with_capacity(64);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            output.push_str(trimmed);
                            output.push('\n');
                        }
                    }
                }

                "dd" => {
                    let mut content = String::with_capacity(128);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }

                    let trimmed = content.trim();

                    if ctx.convert_as_inline {
                        if !trimmed.is_empty() {
                            output.push_str(trimmed);
                        }
                    } else if ctx.last_was_dt {
                        if !trimmed.is_empty() {
                            output.push_str(":   ");
                            output.push_str(trimmed);
                            output.push_str("\n\n");
                        } else {
                            output.push_str(":   \n\n");
                        }
                    } else if !trimmed.is_empty() {
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "details" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth);
                            }
                        }
                        return;
                    }

                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "summary" => {
                    let mut content = String::with_capacity(64);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            let symbol = options.strong_em_symbol.to_string().repeat(2);
                            output.push_str(&symbol);
                            output.push_str(trimmed);
                            output.push_str(&symbol);
                            output.push_str("\n\n");
                        }
                    }
                }

                "dialog" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth);
                            }
                        }
                        return;
                    }

                    let content_start = output.len();

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth);
                        }
                    }

                    while output.len() > content_start && (output.ends_with(' ') || output.ends_with('\t')) {
                        output.pop();
                    }

                    if output.len() > content_start && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }
                }

                "menu" => {
                    let content_start = output.len();

                    let menu_options = ConversionOptions {
                        bullets: "-".to_string(),
                        ..options.clone()
                    };

                    let list_ctx = Context {
                        in_ordered_list: false,
                        list_counter: 0,
                        in_list: true,
                        list_depth: ctx.list_depth,
                        ..ctx.clone()
                    };

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, &menu_options, &list_ctx, depth);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > content_start {
                        if !output.ends_with("\n\n") {
                            if output.ends_with('\n') {
                                output.push('\n');
                            } else {
                                output.push_str("\n\n");
                            }
                        }
                    } else if ctx.convert_as_inline {
                        while output.ends_with('\n') {
                            output.pop();
                        }
                    }
                }

                "audio" => {
                    use std::borrow::Cow;

                    let src = tag
                        .attributes()
                        .get("src")
                        .flatten()
                        .map(|v| v.as_utf8_str())
                        .or_else(|| {
                            let children = tag.children();
                            {
                                for child_handle in children.top().iter() {
                                    if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                        if child_tag.name().as_utf8_str() == "source" {
                                            return child_tag
                                                .attributes()
                                                .get("src")
                                                .flatten()
                                                .map(|v| v.as_utf8_str());
                                        }
                                    }
                                }
                            }
                            None
                        })
                        .unwrap_or(Cow::Borrowed(""));

                    if !src.is_empty() {
                        output.push('[');
                        output.push_str(&src);
                        output.push_str("](");
                        output.push_str(&src);
                        output.push(')');
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }

                    let mut fallback = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            let is_source = if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                child_tag.name().as_utf8_str() == "source"
                            } else {
                                false
                            };

                            if !is_source {
                                walk_node(child_handle, parser, &mut fallback, options, ctx, depth + 1);
                            }
                        }
                    }
                    if !fallback.is_empty() {
                        output.push_str(fallback.trim());
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "video" => {
                    use std::borrow::Cow;

                    let src = tag
                        .attributes()
                        .get("src")
                        .flatten()
                        .map(|v| v.as_utf8_str())
                        .or_else(|| {
                            let children = tag.children();
                            {
                                for child_handle in children.top().iter() {
                                    if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                        if child_tag.name().as_utf8_str() == "source" {
                                            return child_tag
                                                .attributes()
                                                .get("src")
                                                .flatten()
                                                .map(|v| v.as_utf8_str());
                                        }
                                    }
                                }
                            }
                            None
                        })
                        .unwrap_or(Cow::Borrowed(""));

                    if !src.is_empty() {
                        output.push('[');
                        output.push_str(&src);
                        output.push_str("](");
                        output.push_str(&src);
                        output.push(')');
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }

                    let mut fallback = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            let is_source = if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                child_tag.name().as_utf8_str() == "source"
                            } else {
                                false
                            };

                            if !is_source {
                                walk_node(child_handle, parser, &mut fallback, options, ctx, depth + 1);
                            }
                        }
                    }
                    if !fallback.is_empty() {
                        output.push_str(fallback.trim());
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "source" => {}

                "picture" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                if child_tag.name().as_utf8_str() == "img" {
                                    walk_node(child_handle, parser, output, options, ctx, depth);
                                    break;
                                }
                            }
                        }
                    }
                }

                "iframe" => {
                    use std::borrow::Cow;

                    let src = tag
                        .attributes()
                        .get("src")
                        .flatten()
                        .map(|v| v.as_utf8_str())
                        .unwrap_or(Cow::Borrowed(""));

                    if !src.is_empty() {
                        output.push('[');
                        output.push_str(&src);
                        output.push_str("](");
                        output.push_str(&src);
                        output.push(')');
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "svg" => {
                    let mut title = String::from("SVG Image");
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                if child_tag.name().as_utf8_str() == "title" {
                                    title = get_text_content(child_handle, parser).trim().to_string();
                                    break;
                                }
                            }
                        }
                    }

                    #[cfg(feature = "inline-images")]
                    if let Some(ref collector_ref) = ctx.inline_collector {
                        let title_opt = if title == "SVG Image" {
                            None
                        } else {
                            Some(title.clone())
                        };
                        let mut attributes_map = BTreeMap::new();
                        for (key, value_opt) in tag.attributes().iter() {
                            let key_str = key.to_string();
                            let keep = key_str == "width"
                                || key_str == "height"
                                || key_str == "filename"
                                || key_str == "aria-label"
                                || key_str.starts_with("data-");
                            if keep {
                                let value = value_opt.map(|value| value.to_string()).unwrap_or_default();
                                attributes_map.insert(key_str, value);
                            }
                        }
                        handle_inline_svg(collector_ref, node_handle, parser, title_opt, attributes_map);
                    }

                    if ctx.convert_as_inline {
                        output.push_str(&title);
                    } else {
                        use base64::{engine::general_purpose::STANDARD, Engine as _};

                        let svg_html = serialize_element(node_handle, parser);

                        let base64_svg = STANDARD.encode(svg_html.as_bytes());

                        output.push_str("![");
                        output.push_str(&title);
                        output.push_str("](data:image/svg+xml;base64,");
                        output.push_str(&base64_svg);
                        output.push(')');
                    }
                }

                "math" => {
                    let text_content = get_text_content(node_handle, parser).trim().to_string();

                    if text_content.is_empty() {
                        return;
                    }

                    let math_html = serialize_element(node_handle, parser);

                    let escaped_text = text::escape(
                        &text_content,
                        options.escape_misc,
                        options.escape_asterisks,
                        options.escape_underscores,
                        options.escape_ascii,
                    );

                    let is_display_block = tag
                        .attributes()
                        .get("display")
                        .flatten()
                        .map(|v| v.as_utf8_str() == "block")
                        .unwrap_or(false);

                    if is_display_block && !ctx.in_paragraph && !ctx.convert_as_inline {
                        output.push_str("\n\n");
                    }

                    output.push_str("<!-- MathML: ");
                    output.push_str(&math_html);
                    output.push_str(" --> ");
                    output.push_str(&escaped_text);

                    if is_display_block && !ctx.in_paragraph && !ctx.convert_as_inline {
                        output.push_str("\n\n");
                    }
                }

                "form" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth);
                            }
                        }
                        return;
                    }

                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "fieldset" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth);
                            }
                        }
                        return;
                    }
                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "legend" => {
                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            let symbol = options.strong_em_symbol.to_string().repeat(2);
                            output.push_str(&symbol);
                            output.push_str(trimmed);
                            output.push_str(&symbol);
                            output.push_str("\n\n");
                        }
                    }
                }

                "label" => {
                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                        if !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "input" => {}

                "textarea" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "select" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push('\n');
                    }
                }

                "option" => {
                    let selected = tag.attributes().iter().any(|(name, _)| name.as_ref() == "selected");

                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if selected && !ctx.convert_as_inline {
                            output.push_str("* ");
                        }
                        output.push_str(trimmed);
                        if !ctx.convert_as_inline {
                            output.push('\n');
                        }
                    }
                }

                "optgroup" => {
                    use std::borrow::Cow;

                    let label = tag
                        .attributes()
                        .get("label")
                        .flatten()
                        .map(|v| v.as_utf8_str())
                        .unwrap_or(Cow::Borrowed(""));

                    if !label.is_empty() {
                        let symbol = options.strong_em_symbol.to_string().repeat(2);
                        output.push_str(&symbol);
                        output.push_str(&label);
                        output.push_str(&symbol);
                        output.push('\n');
                    }

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }
                }

                "button" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "progress" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "meter" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "output" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "datalist" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push('\n');
                    }
                }

                "ruby" => {
                    let ruby_ctx = ctx.clone();

                    let tag_sequence: Vec<String> = tag
                        .children()
                        .top()
                        .iter()
                        .filter_map(|child_handle| {
                            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                let tag_name = child_tag.name().as_utf8_str();
                                if tag_name == "rb" || tag_name == "rt" || tag_name == "rtc" {
                                    Some(tag_name.to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    let has_rtc = tag_sequence.iter().any(|tag| tag == "rtc");

                    let is_interleaved = tag_sequence.windows(2).any(|w| w[0] == "rb" && w[1] == "rt");

                    if is_interleaved && !has_rtc {
                        let mut current_base = String::new();
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                if let Some(node) = child_handle.get(parser) {
                                    match node {
                                        tl::Node::Tag(child_tag) => {
                                            let tag_name = child_tag.name().as_utf8_str();
                                            if tag_name == "rt" {
                                                let mut annotation = String::new();
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut annotation,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                );
                                                if !current_base.is_empty() {
                                                    output.push_str(current_base.trim());
                                                    current_base.clear();
                                                }
                                                output.push_str(annotation.trim());
                                            } else if tag_name == "rb" {
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
                                                );
                                            } else if tag_name != "rp" {
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut current_base,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                );
                                            }
                                        }
                                        tl::Node::Raw(_) => {
                                            walk_node(
                                                child_handle,
                                                parser,
                                                &mut current_base,
                                                options,
                                                &ruby_ctx,
                                                depth,
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        if !current_base.is_empty() {
                            output.push_str(current_base.trim());
                        }
                    } else {
                        let mut base_text = String::new();
                        let mut rt_annotations = Vec::new();
                        let mut rtc_content = String::new();

                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                if let Some(node) = child_handle.get(parser) {
                                    match node {
                                        tl::Node::Tag(child_tag) => {
                                            let tag_name = child_tag.name().as_utf8_str();
                                            if tag_name == "rt" {
                                                let mut annotation = String::new();
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut annotation,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                );
                                                rt_annotations.push(annotation);
                                            } else if tag_name == "rtc" {
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut rtc_content,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                );
                                            } else if tag_name != "rp" {
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut base_text,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                );
                                            }
                                        }
                                        tl::Node::Raw(_) => {
                                            walk_node(child_handle, parser, &mut base_text, options, &ruby_ctx, depth);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        let trimmed_base = base_text.trim();

                        output.push_str(trimmed_base);

                        if !rt_annotations.is_empty() {
                            let rt_text = rt_annotations.iter().map(|s| s.trim()).collect::<Vec<_>>().join("");
                            if !rt_text.is_empty() {
                                if has_rtc && !rtc_content.trim().is_empty() && rt_annotations.len() > 1 {
                                    output.push('(');
                                    output.push_str(&rt_text);
                                    output.push(')');
                                } else {
                                    output.push_str(&rt_text);
                                }
                            }
                        }

                        if !rtc_content.trim().is_empty() {
                            output.push_str(rtc_content.trim());
                        }
                    }
                }

                "rb" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1);
                        }
                    }
                    output.push_str(text.trim());
                }

                "rt" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = text.trim();

                    if output.ends_with('(') {
                        output.push_str(trimmed);
                    } else {
                        output.push('(');
                        output.push_str(trimmed);
                        output.push(')');
                    }
                }

                "rp" => {
                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                    }
                }

                "rtc" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth);
                        }
                    }
                }

                "div" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1);
                            }
                        }
                        return;
                    }

                    let content_start_pos = output.len();

                    let is_table_continuation =
                        ctx.in_table_cell && !output.is_empty() && !output.ends_with('|') && !output.ends_with("<br>");

                    let is_list_continuation = ctx.in_list_item
                        && !output.is_empty()
                        && !output.ends_with("* ")
                        && !output.ends_with("- ")
                        && !output.ends_with(". ");

                    let needs_leading_sep = !ctx.in_table_cell
                        && !ctx.in_list_item
                        && !ctx.convert_as_inline
                        && !output.is_empty()
                        && !output.ends_with("\n\n");

                    if is_table_continuation {
                        trim_trailing_whitespace(output);
                        output.push_str("<br>");
                    } else if is_list_continuation {
                        add_list_continuation_indent(output, ctx.list_depth, false, options);
                    } else if needs_leading_sep {
                        trim_trailing_whitespace(output);
                        output.push_str("\n\n");
                    }

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth);
                        }
                    }

                    let has_content = output.len() > content_start_pos;

                    if has_content {
                        if content_start_pos == 0 && output.starts_with('\n') && !output.starts_with("\n\n") {
                            output.remove(0);
                        }
                        trim_trailing_whitespace(output);

                        if ctx.in_table_cell {
                        } else if ctx.in_list_item {
                            if is_list_continuation {
                                if !output.ends_with('\n') {
                                    output.push('\n');
                                }
                            } else if !output.ends_with("\n\n") {
                                if output.ends_with('\n') {
                                    output.push('\n');
                                } else {
                                    output.push_str("\n\n");
                                }
                            }
                        } else if !ctx.in_list_item && !ctx.convert_as_inline {
                            if output.ends_with("\n\n") {
                            } else if output.ends_with('\n') {
                                output.push('\n');
                            } else {
                                output.push_str("\n\n");
                            }
                        }
                    }
                }

                "head" => {}

                "script" | "style" => {}

                "span" => {
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
                            walk_node(child_handle, parser, output, options, ctx, depth);
                        }
                    }
                }

                _ => {
                    let len_before = output.len();
                    let had_trailing_space = output.ends_with(' ');

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth);
                        }
                    }

                    let len_after = output.len();
                    if len_after > len_before {
                        let added_content = output[len_before..].to_string();
                        if options.debug {
                            eprintln!(
                                "[DEBUG] <{}> added {:?}, trim={:?}, had_trailing_space={}",
                                tag_name,
                                added_content,
                                added_content.trim(),
                                had_trailing_space
                            );
                        }

                        // Don't truncate code blocks (indented or fenced)
                        let is_code_block = added_content.starts_with("    ")
                            || added_content.starts_with("```")
                            || added_content.starts_with("~~~");

                        if options.debug && added_content.trim().is_empty() {
                            eprintln!(
                                "[DEBUG] Whitespace-only content, is_code_block={}, will_truncate={}",
                                is_code_block, !is_code_block
                            );
                        }

                        if added_content.trim().is_empty() && !is_code_block {
                            output.truncate(len_before);
                            if !had_trailing_space && added_content.contains(' ') {
                                output.push(' ');
                            }
                            if options.debug {
                                eprintln!(
                                    "[DEBUG] Truncated, output now ends with space: {}",
                                    output.ends_with(' ')
                                );
                            }
                        }
                    }
                }
            }
        }

        tl::Node::Comment(_) => {
            // Comments are ignored
        }
    }
}

/// Get colspan attribute value from element
fn get_colspan(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> usize {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        if let Some(Some(bytes)) = tag.attributes().get("colspan") {
            if let Ok(colspan) = bytes.as_utf8_str().parse::<usize>() {
                return colspan;
            }
        }
    }
    1
}

/// Get both colspan and rowspan in a single lookup
fn get_colspan_rowspan(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> (usize, usize) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let attrs = tag.attributes();
        let colspan = attrs
            .get("colspan")
            .flatten()
            .and_then(|v| v.as_utf8_str().parse::<usize>().ok())
            .unwrap_or(1);
        let rowspan = attrs
            .get("rowspan")
            .flatten()
            .and_then(|v| v.as_utf8_str().parse::<usize>().ok())
            .unwrap_or(1);
        (colspan, rowspan)
    } else {
        (1, 1)
    }
}

/// Convert table cell (td or th)
fn convert_table_cell(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    _tag_name: &str,
) {
    let mut text = String::with_capacity(128);

    let cell_ctx = Context {
        in_table_cell: true,
        ..ctx.clone()
    };

    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, &mut text, options, &cell_ctx, 0);
            }
        }
    }

    let text = text.trim();
    let text = if options.br_in_tables {
        text.split('\n')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("<br>")
    } else {
        text.replace('\n', " ")
    };

    let colspan = get_colspan(node_handle, parser);

    output.push(' ');
    output.push_str(&text);
    output.push_str(&" |".repeat(colspan));
}

/// Convert table row (tr)
fn convert_table_row(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    row_index: usize,
    rowspan_tracker: &mut std::collections::HashMap<usize, usize>,
) {
    let mut row_text = String::with_capacity(256);
    let mut cells = Vec::new();

    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let cell_name = child_tag.name().as_utf8_str();
                    if cell_name == "th" || cell_name == "td" {
                        cells.push(*child_handle);
                    }
                }
            }
        }
    }

    let mut col_index = 0;
    let mut cell_iter = cells.iter();

    loop {
        if let Some(remaining_rows) = rowspan_tracker.get_mut(&col_index) {
            if *remaining_rows > 0 {
                row_text.push_str(" |");
                *remaining_rows -= 1;
                if *remaining_rows == 0 {
                    rowspan_tracker.remove(&col_index);
                }
                col_index += 1;
                continue;
            }
        }

        if let Some(cell_handle) = cell_iter.next() {
            convert_table_cell(cell_handle, parser, &mut row_text, options, ctx, "");

            let (colspan, rowspan) = get_colspan_rowspan(cell_handle, parser);

            if rowspan > 1 {
                rowspan_tracker.insert(col_index, rowspan - 1);
            }

            col_index += colspan;
        } else {
            break;
        }
    }

    output.push('|');
    output.push_str(&row_text);
    output.push('\n');

    let is_first_row = row_index == 0;
    if is_first_row {
        let total_cols = cells.iter().map(|h| get_colspan(h, parser)).sum::<usize>().max(1);
        output.push_str("| ");
        for i in 0..total_cols {
            if i > 0 {
                output.push_str(" | ");
            }
            output.push_str("---");
        }
        output.push_str(" |\n");
    }
}

/// Convert an entire table element
fn convert_table(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut row_index = 0;
        let mut rowspan_tracker = std::collections::HashMap::new();

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let tag_name = child_tag.name().as_utf8_str();

                    match tag_name.as_ref() {
                        "caption" => {
                            let mut text = String::new();
                            let grandchildren = child_tag.children();
                            {
                                for grandchild_handle in grandchildren.top().iter() {
                                    walk_node(grandchild_handle, parser, &mut text, options, ctx, 0);
                                }
                            }
                            let text = text.trim();
                            if !text.is_empty() {
                                // Escape dashes in captions to avoid confusion with table separators
                                let escaped_text = text.replace('-', r"\-");
                                output.push('*');
                                output.push_str(&escaped_text);
                                output.push_str("*\n\n");
                            }
                        }

                        "thead" | "tbody" | "tfoot" => {
                            let section_children = child_tag.children();
                            {
                                for row_handle in section_children.top().iter() {
                                    if let Some(tl::Node::Tag(row_tag)) = row_handle.get(parser) {
                                        if row_tag.name().as_utf8_str() == "tr" {
                                            convert_table_row(
                                                row_handle,
                                                parser,
                                                output,
                                                options,
                                                ctx,
                                                row_index,
                                                &mut rowspan_tracker,
                                            );
                                            row_index += 1;
                                        }
                                    }
                                }
                            }
                        }

                        "tr" => {
                            convert_table_row(
                                child_handle,
                                parser,
                                output,
                                options,
                                ctx,
                                row_index,
                                &mut rowspan_tracker,
                            );
                            row_index += 1;
                        }

                        "colgroup" | "col" => {}

                        _ => {}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_trailing_whitespace() {
        let mut s = String::from("hello   ");
        trim_trailing_whitespace(&mut s);
        assert_eq!(s, "hello");

        let mut s = String::from("hello\t\t");
        trim_trailing_whitespace(&mut s);
        assert_eq!(s, "hello");

        let mut s = String::from("hello \t \t");
        trim_trailing_whitespace(&mut s);
        assert_eq!(s, "hello");

        let mut s = String::from("hello");
        trim_trailing_whitespace(&mut s);
        assert_eq!(s, "hello");

        let mut s = String::from("");
        trim_trailing_whitespace(&mut s);
        assert_eq!(s, "");

        let mut s = String::from("hello\n");
        trim_trailing_whitespace(&mut s);
        assert_eq!(s, "hello\n");
    }

    #[test]
    fn test_chomp_preserves_boundary_spaces() {
        assert_eq!(chomp_inline("  text  "), (" ", " ", "text"));
        assert_eq!(chomp_inline("text"), ("", "", "text"));
        assert_eq!(chomp_inline("  text"), (" ", "", "text"));
        assert_eq!(chomp_inline("text  "), ("", " ", "text"));
        assert_eq!(chomp_inline("   "), (" ", " ", ""));
        assert_eq!(chomp_inline(""), ("", "", ""));
    }

    #[test]
    fn test_calculate_list_continuation_indent() {
        assert_eq!(calculate_list_continuation_indent(0), 0);

        assert_eq!(calculate_list_continuation_indent(1), 1);

        assert_eq!(calculate_list_continuation_indent(2), 3);

        assert_eq!(calculate_list_continuation_indent(3), 5);

        assert_eq!(calculate_list_continuation_indent(4), 7);
    }

    #[test]
    fn strips_script_sections_without_removing_following_content() {
        let input = "<div>before</div><script>1 < 2</script><p>after</p>";
        let stripped = strip_script_and_style_sections(input);
        assert_eq!(stripped, "<div>before</div><script></script><p>after</p>");
    }

    #[test]
    fn strips_multiline_script_sections() {
        let input = "<html>\n<script>1 < 2</script>\nContent\n</html>";
        let stripped = strip_script_and_style_sections(input);
        assert!(stripped.contains("Content"));
        assert!(stripped.contains("<script"));
        assert!(!stripped.contains("1 < 2"));
    }

    #[test]
    fn test_add_list_continuation_indent_blank_line() {
        let opts = ConversionOptions::default();
        let mut output = String::from("* First para");
        add_list_continuation_indent(&mut output, 1, true, &opts);
        assert_eq!(output, "* First para\n\n  ");

        let mut output = String::from("* First para\n");
        add_list_continuation_indent(&mut output, 1, true, &opts);
        assert_eq!(output, "* First para\n\n  ");

        let mut output = String::from("* First para\n\n");
        add_list_continuation_indent(&mut output, 1, true, &opts);
        assert_eq!(output, "* First para\n\n  ");

        let mut output = String::from("* First para");
        add_list_continuation_indent(&mut output, 2, true, &opts);
        assert_eq!(output, "* First para\n\n      ");
    }

    #[test]
    fn test_add_list_continuation_indent_single_line() {
        let opts = ConversionOptions::default();
        let mut output = String::from("* First div");
        add_list_continuation_indent(&mut output, 1, false, &opts);
        assert_eq!(output, "* First div\n  ");

        let mut output = String::from("* First div\n");
        add_list_continuation_indent(&mut output, 1, false, &opts);
        assert_eq!(output, "* First div\n  ");

        let mut output = String::from("* First div\n");
        add_list_continuation_indent(&mut output, 1, false, &opts);
        assert_eq!(output, "* First div\n  ");
    }

    #[test]
    fn test_trim_trailing_whitespace_in_continuation() {
        let opts = ConversionOptions::default();
        let mut output = String::from("* First   ");
        add_list_continuation_indent(&mut output, 1, true, &opts);
        assert_eq!(output, "* First\n\n  ");

        let mut output = String::from("* First\t\t");
        add_list_continuation_indent(&mut output, 1, false, &opts);
        assert_eq!(output, "* First\n  ");
    }

    #[test]
    fn test_escape_malformed_angle_brackets_bare() {
        let input = "1<2";
        let escaped = escape_malformed_angle_brackets(input);
        assert_eq!(escaped, "1&lt;2");
    }

    #[test]
    fn test_escape_malformed_angle_brackets_in_text() {
        let input = "<html>1<2 Content</html>";
        let escaped = escape_malformed_angle_brackets(input);
        assert_eq!(escaped, "<html>1&lt;2 Content</html>");
    }

    #[test]
    fn test_escape_malformed_angle_brackets_multiple() {
        let input = "1 < 2 < 3";
        let escaped = escape_malformed_angle_brackets(input);
        assert_eq!(escaped, "1 &lt; 2 &lt; 3");
    }

    #[test]
    fn test_escape_malformed_angle_brackets_preserves_valid_tags() {
        let input = "<div>content</div>";
        let escaped = escape_malformed_angle_brackets(input);
        assert_eq!(escaped, "<div>content</div>");
    }

    #[test]
    fn test_escape_malformed_angle_brackets_mixed() {
        let input = "<div>1<2</div><p>3<4</p>";
        let escaped = escape_malformed_angle_brackets(input);
        assert_eq!(escaped, "<div>1&lt;2</div><p>3&lt;4</p>");
    }

    #[test]
    fn test_escape_malformed_angle_brackets_at_end() {
        let input = "test<";
        let escaped = escape_malformed_angle_brackets(input);
        assert_eq!(escaped, "test&lt;");
    }

    #[test]
    fn test_escape_malformed_angle_brackets_preserves_comments() {
        let input = "<!-- comment -->1<2";
        let escaped = escape_malformed_angle_brackets(input);
        assert_eq!(escaped, "<!-- comment -->1&lt;2");
    }

    #[test]
    fn test_escape_malformed_angle_brackets_preserves_doctype() {
        let input = "<!DOCTYPE html>1<2";
        let escaped = escape_malformed_angle_brackets(input);
        assert_eq!(escaped, "<!DOCTYPE html>1&lt;2");
    }

    #[test]
    fn test_convert_with_malformed_angle_brackets() {
        // Test the full conversion pipeline (issue #94)
        let html = "<html>1<2\nContent</html>";
        let result = convert_html(html, &ConversionOptions::default()).unwrap();
        assert!(
            result.contains("Content"),
            "Result should contain 'Content': {:?}",
            result
        );
        assert!(
            result.contains("1<2") || result.contains("1&lt;2"),
            "Result should contain escaped or unescaped comparison"
        );
    }

    #[test]
    fn test_convert_with_malformed_angle_brackets_in_div() {
        let html = "<html><div>1<2</div><div>Content</div></html>";
        let result = convert_html(html, &ConversionOptions::default()).unwrap();
        assert!(
            result.contains("Content"),
            "Result should contain 'Content': {:?}",
            result
        );
    }

    #[test]
    fn test_convert_with_multiple_malformed_angle_brackets() {
        let html = "<html>1 < 2 < 3<p>Content</p></html>";
        let result = convert_html(html, &ConversionOptions::default()).unwrap();
        assert!(
            result.contains("Content"),
            "Result should contain 'Content': {:?}",
            result
        );
    }
}
