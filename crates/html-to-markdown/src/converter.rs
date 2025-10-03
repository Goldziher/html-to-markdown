//! HTML to Markdown conversion using html5ever.
//!
//! This module provides the core conversion logic for transforming HTML documents into Markdown.
//! It uses the html5ever parser for browser-grade HTML parsing and supports 60+ HTML tags.
//!
//! # Architecture
//!
//! The conversion process follows these steps:
//! 1. Parse HTML into a DOM tree using html5ever
//! 2. Walk the DOM tree recursively
//! 3. Convert each node type to its Markdown equivalent
//! 4. Apply text escaping and whitespace normalization
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
//! use html_to_markdown::{convert, ConversionOptions};
//!
//! let html = "<h1>Title</h1><p>Paragraph with <strong>bold</strong> text.</p>";
//! let markdown = convert(html, None).unwrap();
//! assert_eq!(markdown, "Title\n=====\n\nParagraph with **bold** text.\n\n");
//! ```

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::BTreeMap;

use crate::error::Result;
use crate::options::{ConversionOptions, HeadingStyle};
use crate::text;

/// Chomp whitespace from text, preserving leading/trailing spaces as single spaces.
/// Returns (prefix, suffix, trimmed_text)
fn chomp(text: &str) -> (&str, &str, &str) {
    if text.is_empty() {
        return ("", "", "");
    }

    let prefix = if text.starts_with(&[' ', '\t'][..]) { " " } else { "" };

    let suffix = if text.ends_with(&[' ', '\t'][..]) { " " } else { "" };

    (prefix, suffix, text.trim())
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
}

/// Check if a document is an hOCR (HTML-based OCR) document.
///
/// hOCR documents should have metadata extraction disabled to avoid
/// including OCR metadata (system info, capabilities, etc.) in output.
///
/// Detection criteria:
/// - meta tag with name="ocr-system" or name="ocr-capabilities"
/// - Elements with classes: ocr_page, ocrx_word, ocr_carea, ocr_par, ocr_line
fn is_hocr_document(handle: &Handle) -> bool {
    fn check_node(handle: &Handle) -> bool {
        match &handle.data {
            NodeData::Element { name, attrs, .. } => {
                let tag_name = name.local.as_ref();
                let attrs = attrs.borrow();

                // Check for hOCR meta tags
                if tag_name == "meta" {
                    for attr in attrs.iter() {
                        if attr.name.local.as_ref() == "name" {
                            let value = attr.value.as_ref();
                            if value == "ocr-system" || value == "ocr-capabilities" {
                                return true;
                            }
                        }
                    }
                }

                // Check for hOCR class names
                for attr in attrs.iter() {
                    if attr.name.local.as_ref() == "class" {
                        let class_value = attr.value.as_ref();
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

                // Recursively check children
                for child in handle.children.borrow().iter() {
                    if check_node(child) {
                        return true;
                    }
                }
                false
            }
            NodeData::Document => {
                // Check children for document node
                for child in handle.children.borrow().iter() {
                    if check_node(child) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    check_node(handle)
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
fn extract_metadata(handle: &Handle) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();

    // Recursively search for head element
    fn find_head(handle: &Handle) -> Option<Handle> {
        if let NodeData::Element { name, .. } = &handle.data {
            if name.local.as_ref() == "head" {
                return Some(handle.clone());
            }
        }
        for child in handle.children.borrow().iter() {
            if let Some(head) = find_head(child) {
                return Some(head);
            }
        }
        None
    }

    let head = match find_head(handle) {
        Some(h) => h,
        None => return metadata,
    };

    // Extract from head children
    for child in head.children.borrow().iter() {
        if let NodeData::Element { name, attrs, .. } = &child.data {
            let tag_name = name.local.as_ref();

            match tag_name {
                "title" => {
                    // Extract title text
                    if let Some(text_node) = child.children.borrow().first() {
                        if let NodeData::Text { contents } = &text_node.data {
                            // Normalize whitespace (collapse multiple spaces) and trim
                            let title = text::normalize_whitespace(&contents.borrow()).trim().to_string();
                            if !title.is_empty() {
                                metadata.insert("title".to_string(), title);
                            }
                        }
                    }
                }
                "base" => {
                    // Extract base href
                    for attr in attrs.borrow().iter() {
                        if attr.name.local.as_ref() == "href" {
                            let href = attr.value.to_string();
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

                    for attr in attrs.borrow().iter() {
                        match attr.name.local.as_ref() {
                            "name" => name_attr = Some(attr.value.to_string()),
                            "property" => property_attr = Some(attr.value.to_string()),
                            "http-equiv" => http_equiv_attr = Some(attr.value.to_string()),
                            "content" => content_attr = Some(attr.value.to_string()),
                            _ => {}
                        }
                    }

                    if let Some(content) = content_attr {
                        if let Some(name) = name_attr {
                            // Standard meta name tags
                            let key = format!("meta-{}", name.to_lowercase());
                            metadata.insert(key, content);
                        } else if let Some(property) = property_attr {
                            // Open Graph, Twitter Card, etc. (property attributes)
                            let key = format!("meta-{}", property.to_lowercase().replace(':', "-"));
                            metadata.insert(key, content);
                        } else if let Some(http_equiv) = http_equiv_attr {
                            // HTTP-equiv meta tags
                            let key = format!("meta-{}", http_equiv.to_lowercase());
                            metadata.insert(key, content);
                        }
                    }
                }
                "link" => {
                    let mut rel_attr = None;
                    let mut href_attr = None;

                    for attr in attrs.borrow().iter() {
                        match attr.name.local.as_ref() {
                            "rel" => rel_attr = Some(attr.value.to_string()),
                            "href" => href_attr = Some(attr.value.to_string()),
                            _ => {}
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

    metadata
}

/// Format metadata as HTML comment.
fn format_metadata_comment(metadata: &BTreeMap<String, String>) -> String {
    if metadata.is_empty() {
        return String::new();
    }

    let mut lines = vec!["<!--".to_string()];
    for (key, value) in metadata {
        // Escape --> in values
        let escaped_value = value.replace("-->", "--&gt;");
        lines.push(format!("{}: {}", key, escaped_value));
    }
    lines.push("-->".to_string());

    lines.join("\n") + "\n\n"
}

/// Check if a handle is an empty inline element (abbr, var, ins, dfn, etc. with no text content).
fn is_empty_inline_element(handle: &Handle) -> bool {
    // List of inline elements that should be considered when empty
    const EMPTY_WHEN_NO_CONTENT_TAGS: &[&str] = &[
        "abbr", "var", "ins", "dfn", "time", "data", "cite", "q", "mark", "small", "u",
    ];

    if let NodeData::Element { name, .. } = &handle.data {
        let tag_name = name.local.as_ref();
        if EMPTY_WHEN_NO_CONTENT_TAGS.contains(&tag_name) {
            // Check if the element has no text content
            return get_text_content(handle).trim().is_empty();
        }
    }
    false
}

/// Get the text content of a node and its children.
fn get_text_content(handle: &Handle) -> String {
    let mut text = String::new();
    for child in handle.children.borrow().iter() {
        match &child.data {
            NodeData::Text { contents } => {
                text.push_str(&contents.borrow());
            }
            NodeData::Element { .. } => {
                text.push_str(&get_text_content(child));
            }
            _ => {}
        }
    }
    text
}

/// Convert HTML to Markdown using html5ever DOM parser.
pub fn convert_html(html: &str, options: &ConversionOptions) -> Result<String> {
    // Parse HTML into DOM
    let dom = parse_document(RcDom::default(), Default::default()).one(html);

    // Walk the DOM and build Markdown
    let mut output = String::new();

    // Extract and prepend metadata if enabled (but not in inline mode or hOCR documents)
    // hOCR documents should not include metadata to avoid OCR system info in output
    if options.extract_metadata && !options.convert_as_inline && !is_hocr_document(&dom.document) {
        let metadata = extract_metadata(&dom.document);
        let metadata_comment = format_metadata_comment(&metadata);
        output.push_str(&metadata_comment);
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
        in_list: false,
        loose_list: false,
        prev_item_had_blocks: false,
        in_heading: false,
        heading_tag: None,
        in_paragraph: false,
        in_ruby: false,
    };
    walk_node(&dom.document, &mut output, options, &ctx, 0);

    Ok(output)
}

/// Recursively walk DOM nodes and convert to Markdown.
#[allow(clippy::only_used_in_recursion)]
fn walk_node(handle: &Handle, output: &mut String, options: &ConversionOptions, ctx: &Context, depth: usize) {
    match &handle.data {
        NodeData::Document => {
            // Process children
            for child in handle.children.borrow().iter() {
                walk_node(child, output, options, ctx, depth);
            }
        }

        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();

            // Skip empty text
            if text.is_empty() {
                return;
            }

            // For whitespace-only text, preserve one space (unless at block boundaries)
            if text.trim().is_empty() {
                // In code blocks, preserve all whitespace exactly
                if ctx.in_code {
                    output.push_str(&text);
                    return;
                }

                // In strict mode, preserve whitespace intelligently
                if options.whitespace_mode == crate::options::WhitespaceMode::Strict {
                    // Always preserve in inline mode, table cells, or list items
                    if ctx.convert_as_inline || ctx.in_table_cell || ctx.in_list_item {
                        output.push_str(&text);
                        return;
                    }
                    // At block level with multiple consecutive newlines:
                    // Output only a single newline IF we don't already have double newlines
                    if text.contains("\n\n") || text.contains("\r\n\r\n") {
                        // Only add newline if output doesn't already end with \n\n
                        if !output.ends_with("\n\n") {
                            output.push('\n');
                        }
                        return;
                    }
                    // Single newline or spaces - preserve as-is
                    output.push_str(&text);
                    return;
                }

                // If whitespace contains newlines, ignore it (per Python behavior)
                if text.contains('\n') {
                    return;
                }

                // Don't preserve whitespace after:
                // - Block boundaries (\n\n)
                // - List item bullets (*, -, 1., etc followed by space)
                // - At the start of output
                let skip_whitespace = output.is_empty()
                    || output.ends_with("\n\n")
                    || output.ends_with("* ")
                    || output.ends_with("- ")
                    || output.ends_with(". ")
                    || output.ends_with("] "); // Task list checkboxes

                // Preserve whitespace in inline mode, in table cells, or after inline content
                let should_preserve =
                    (ctx.convert_as_inline || ctx.in_table_cell || !output.is_empty()) && !skip_whitespace;

                if should_preserve {
                    output.push(' ');
                }
                return;
            }

            // Apply text escaping (unless we're in code or table cells)
            let processed_text = if ctx.in_code || ctx.in_table_cell || ctx.in_ruby {
                // In code blocks, table cells, or ruby elements, preserve text as-is
                // Normalize whitespace if in normalized mode (but only if not in code/ruby)
                if ctx.in_code || ctx.in_ruby {
                    text
                } else if options.whitespace_mode == crate::options::WhitespaceMode::Normalized {
                    text::normalize_whitespace(&text)
                } else {
                    text
                }
            } else if options.whitespace_mode == crate::options::WhitespaceMode::Strict {
                // In strict mode, preserve text exactly as-is (including all whitespace/newlines)
                text::escape(
                    &text,
                    options.escape_misc,
                    options.escape_asterisks,
                    options.escape_underscores,
                )
            } else {
                // Normalized mode: collapse whitespace and handle boundary spaces
                let normalized_text = text::normalize_whitespace(&text);

                // Use chomp to extract boundary whitespace
                let (prefix, suffix, core) = text::chomp(&normalized_text);

                // Don't add leading space if output ends with a separator
                let skip_prefix = output.ends_with("\n\n")
                    || output.ends_with("* ")
                    || output.ends_with("- ")
                    || output.ends_with(". ")
                    || output.ends_with("] ");

                // Build the final text with appropriate boundary spaces
                let mut final_text = String::new();
                if !skip_prefix && !prefix.is_empty() {
                    final_text.push_str(prefix);
                }

                // Apply escaping to core text
                let escaped_core = text::escape(
                    core,
                    options.escape_misc,
                    options.escape_asterisks,
                    options.escape_underscores,
                );
                final_text.push_str(&escaped_core);

                if !suffix.is_empty() {
                    final_text.push_str(suffix);
                }

                final_text
            };

            // Handle double newlines in list items - indent continuation paragraphs
            if ctx.in_list_item && processed_text.contains("\n\n") {
                let parts: Vec<&str> = processed_text.split("\n\n").collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        // Add separator and indentation for continuation paragraphs
                        output.push_str("\n\n");
                        output.push_str(&" ".repeat(4 * ctx.list_depth));
                    }
                    // Trim leading/trailing spaces from each part to avoid double-spacing
                    output.push_str(part.trim());
                }
            } else {
                output.push_str(&processed_text);
            }
        }

        NodeData::Element { name, attrs, .. } => {
            let tag_name = name.local.as_ref();

            match tag_name {
                // Headings
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = tag_name.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1) as usize;

                    let mut text = String::new();
                    let heading_ctx = Context {
                        in_heading: true,
                        heading_tag: Some(tag_name.to_string()),
                        ..ctx.clone()
                    };
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, &heading_ctx, depth + 1);
                    }
                    let text = text.trim();

                    if !text.is_empty() {
                        // In inline mode or table cell, just output the text without formatting
                        if ctx.convert_as_inline {
                            output.push_str(text);
                            return;
                        }

                        // In table cells, check if we need <br> separator
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

                        match options.heading_style {
                            HeadingStyle::Underlined => {
                                if level == 1 {
                                    output.push_str(text);
                                    output.push('\n');
                                    output.push_str(&"=".repeat(text.len()));
                                    output.push_str("\n\n");
                                } else if level == 2 {
                                    output.push_str(text);
                                    output.push('\n');
                                    output.push_str(&"-".repeat(text.len()));
                                    output.push_str("\n\n");
                                } else {
                                    output.push_str(&"#".repeat(level));
                                    output.push(' ');
                                    output.push_str(text);
                                    output.push_str("\n\n");
                                }
                            }
                            HeadingStyle::Atx => {
                                output.push_str(&"#".repeat(level));
                                output.push(' ');
                                output.push_str(text);
                                output.push_str("\n\n");
                            }
                            HeadingStyle::AtxClosed => {
                                output.push_str(&"#".repeat(level));
                                output.push(' ');
                                output.push_str(text);
                                output.push(' ');
                                output.push_str(&"#".repeat(level));
                                output.push_str("\n\n");
                            }
                        }
                    }
                }

                // Paragraph
                "p" => {
                    // Track where content for this paragraph starts
                    let content_start_pos = output.len();

                    // Check if this is a continuation in a table cell
                    // In table cells, if there's any content (not just starting with |), add <br>
                    let is_table_continuation =
                        ctx.in_table_cell && !output.is_empty() && !output.ends_with('|') && !output.ends_with("<br>");

                    // Check if this is a continuation paragraph in a list item
                    let is_list_continuation = ctx.in_list_item
                        && !output.is_empty()
                        && !output.ends_with("* ")
                        && !output.ends_with("- ")
                        && !output.ends_with(". ");

                    // Check if we need leading block separation
                    // Don't add spacing after code blocks (ending with ```\n)
                    let after_code_block = output.ends_with("```\n");
                    let needs_leading_sep = !ctx.in_table_cell
                        && !ctx.in_list_item
                        && !ctx.convert_as_inline
                        && !output.is_empty()
                        && !output.ends_with("\n\n")
                        && !after_code_block;

                    if is_table_continuation {
                        // In table cells, separate multiple block elements with <br>
                        // Trim trailing whitespace first
                        while output.ends_with(' ') || output.ends_with('\t') {
                            output.pop();
                        }
                        output.push_str("<br>");
                    } else if is_list_continuation {
                        // Trim trailing spaces/tabs before adding newline
                        while output.ends_with(' ') || output.ends_with('\t') {
                            output.pop();
                        }
                        // Paragraphs in list items should always have blank line separation (double newline)
                        // This ensures proper spacing between different block elements
                        if !output.ends_with("\n\n") {
                            if output.ends_with('\n') {
                                output.push('\n');
                            } else {
                                output.push_str("\n\n");
                            }
                        }
                        // Continuation indentation: base indentation + content indentation
                        // Formula: (list_depth - 1) * 4 spaces for nesting + list_depth * 4 for continuation
                        // Simplified: (2 * list_depth - 1) * 4 spaces
                        let indent_level = if ctx.list_depth > 0 { 2 * ctx.list_depth - 1 } else { 0 };
                        output.push_str(&"    ".repeat(indent_level));
                    } else if needs_leading_sep {
                        // Trim trailing whitespace before adding block separation
                        while output.ends_with(' ') || output.ends_with('\t') {
                            output.pop();
                        }
                        // Add leading block separation for paragraphs in normal context
                        output.push_str("\n\n");
                    }

                    // Create paragraph context for children
                    let p_ctx = Context {
                        in_paragraph: true,
                        ..ctx.clone()
                    };

                    // Process children, skipping whitespace-only text nodes between empty inline elements
                    let children: Vec<_> = handle.children.borrow().iter().cloned().collect();
                    for (i, child) in children.iter().enumerate() {
                        // Check if this is a whitespace-only text node between empty inline elements
                        if let NodeData::Text { contents } = &child.data {
                            let text = contents.borrow();
                            if text.trim().is_empty() && i > 0 && i < children.len() - 1 {
                                let prev = &children[i - 1];
                                let next = &children[i + 1];
                                if is_empty_inline_element(prev) && is_empty_inline_element(next) {
                                    // Skip this whitespace text node
                                    continue;
                                }
                            }
                        }
                        walk_node(child, output, options, &p_ctx, depth + 1);
                    }

                    // Only add trailing separator if paragraph had content
                    let has_content = output.len() > content_start_pos;

                    if has_content && !ctx.convert_as_inline && !ctx.in_table_cell {
                        // Add trailing separator in normal and list contexts
                        output.push_str("\n\n");
                    }
                }

                // Strong/Bold
                "strong" | "b" => {
                    let symbol = options.strong_em_symbol.to_string().repeat(2);
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        output.push_str(&symbol);
                        output.push_str(trimmed);
                        output.push_str(&symbol);
                    }
                }

                // Emphasis/Italic
                "em" | "i" => {
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        output.push(options.strong_em_symbol);
                        output.push_str(trimmed);
                        output.push(options.strong_em_symbol);
                    }
                }

                // Links
                "a" => {
                    let href = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "href")
                        .map(|attr| attr.value.to_string())
                        .unwrap_or_default();

                    output.push('[');
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }
                    output.push_str("](");
                    output.push_str(&href);
                    output.push(')');
                }

                // Images
                "img" => {
                    let src = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "src")
                        .map(|attr| attr.value.to_string())
                        .unwrap_or_default();

                    let alt = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "alt")
                        .map(|attr| attr.value.to_string())
                        .unwrap_or_default();

                    let title = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "title")
                        .map(|attr| attr.value.to_string());

                    // Check for width/height attributes
                    let width = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "width")
                        .map(|attr| attr.value.to_string());

                    let height = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "height")
                        .map(|attr| attr.value.to_string());

                    // Determine if image should be rendered as alt text
                    let should_use_alt_text = ctx.convert_as_inline
                        || (ctx.in_heading
                            && ctx
                                .heading_tag
                                .as_ref()
                                .is_none_or(|tag| !options.keep_inline_images_in.contains(tag)));

                    if should_use_alt_text {
                        // In inline mode or headings (unless in keep_inline_images_in), just output alt text
                        output.push_str(&alt);
                    } else if width.is_some() || height.is_some() {
                        // If width or height specified, output as HTML tag
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
                        // In normal mode, output ![alt](src) or ![alt](src "title")
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

                // Inline formatting tags
                "mark" => {
                    use crate::options::HighlightStyle;
                    match options.highlight_style {
                        HighlightStyle::DoubleEqual => {
                            output.push_str("==");
                            for child in handle.children.borrow().iter() {
                                walk_node(child, output, options, ctx, depth + 1);
                            }
                            output.push_str("==");
                        }
                        HighlightStyle::Html => {
                            output.push_str("<mark>");
                            for child in handle.children.borrow().iter() {
                                walk_node(child, output, options, ctx, depth + 1);
                            }
                            output.push_str("</mark>");
                        }
                        HighlightStyle::Bold => {
                            let symbol = options.strong_em_symbol.to_string().repeat(2);
                            output.push_str(&symbol);
                            for child in handle.children.borrow().iter() {
                                walk_node(child, output, options, ctx, depth + 1);
                            }
                            output.push_str(&symbol);
                        }
                        HighlightStyle::None => {
                            // Plain text, no formatting
                            for child in handle.children.borrow().iter() {
                                walk_node(child, output, options, ctx, depth + 1);
                            }
                        }
                    }
                }

                "del" | "s" => {
                    output.push_str("~~");
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }
                    output.push_str("~~");
                }

                "ins" => {
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let (prefix, suffix, trimmed) = chomp(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push_str("==");
                        output.push_str(trimmed);
                        output.push_str("==");
                        output.push_str(suffix);
                    }
                }

                "u" | "small" => {
                    // No special formatting, just render text
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }
                }

                "sub" => {
                    if !options.sub_symbol.is_empty() {
                        output.push_str(&options.sub_symbol);
                    }
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }
                    if !options.sub_symbol.is_empty() {
                        output.push_str(&options.sub_symbol);
                    }
                }

                "sup" => {
                    if !options.sup_symbol.is_empty() {
                        output.push_str(&options.sup_symbol);
                    }
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }
                    if !options.sup_symbol.is_empty() {
                        output.push_str(&options.sup_symbol);
                    }
                }

                "kbd" | "samp" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };
                    output.push('`');
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, &code_ctx, depth + 1);
                    }
                    output.push('`');
                }

                "var" => {
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let (prefix, suffix, trimmed) = chomp(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push(options.strong_em_symbol);
                        output.push_str(trimmed);
                        output.push(options.strong_em_symbol);
                        output.push_str(suffix);
                    }
                }

                "dfn" => {
                    // Definition - same as var, outputs with * delimiters
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let (prefix, suffix, trimmed) = chomp(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push(options.strong_em_symbol);
                        output.push_str(trimmed);
                        output.push(options.strong_em_symbol);
                        output.push_str(suffix);
                    }
                }

                "abbr" => {
                    // Collect content
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();

                    // Only output if there's content (no prefix/suffix preservation for abbr)
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);

                        // Optionally add title as footnote (trimmed)
                        if let Some(title) = attrs
                            .borrow()
                            .iter()
                            .find(|attr| attr.name.local.as_ref() == "title")
                            .map(|attr| attr.value.to_string())
                        {
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
                    // Just render text content
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }
                }

                "wbr" => {
                    // Word break opportunity - render as empty
                }

                // Code (inline) - set in_code=true
                "code" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };
                    // If already in code block (inside <pre>), don't add backticks
                    if !ctx.in_code {
                        output.push('`');
                    }
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, &code_ctx, depth + 1);
                    }
                    if !ctx.in_code {
                        output.push('`');
                    }
                }

                // Preformatted/Code blocks - set in_code=true
                "pre" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };

                    // Collect content first to check if it's empty
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, &code_ctx, depth + 1);
                    }

                    // Only output code block if there's content (don't trim - preserve whitespace)
                    if !content.is_empty() {
                        output.push_str("```");
                        if !options.code_language.is_empty() {
                            output.push_str(&options.code_language);
                        }
                        output.push('\n');
                        output.push_str(&content);
                        output.push_str("\n```\n");
                    }
                }

                // Blockquote
                "blockquote" => {
                    // In inline mode, just process children without blockquote formatting
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth + 1);
                        }
                        return;
                    }

                    // Extract cite attribute
                    let cite = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "cite")
                        .map(|attr| attr.value.to_string());

                    // Process children with increased blockquote depth
                    let blockquote_ctx = Context {
                        blockquote_depth: ctx.blockquote_depth + 1,
                        ..ctx.clone()
                    };
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, &blockquote_ctx, depth + 1);
                    }

                    // Trim trailing newlines to avoid extra empty blockquote lines
                    let trimmed_content = content.trim_end_matches('\n');

                    // Only output if there's content (skip empty blockquotes entirely)
                    if !trimmed_content.is_empty() {
                        // Add leading block separation
                        if ctx.blockquote_depth > 0 {
                            // Nested blockquotes need triple newline for spacing
                            output.push_str("\n\n\n");
                        } else if output.is_empty() {
                            // First element in document: single newline
                            output.push('\n');
                        } else if !output.ends_with("\n\n") {
                            // Not first element and doesn't have block separation: add \n\n
                            output.push_str("\n\n");
                        }
                        // else: output already ends with \n\n, don't add more

                        // Each blockquote adds exactly one level of > prefix
                        let prefix = "> ";

                        // Output content with prefix, trimming trailing spaces from each line
                        for line in trimmed_content.lines() {
                            output.push_str(prefix);
                            output.push_str(line.trim_end());
                            output.push('\n');
                        }

                        // Add cite attribution if present
                        // Cite is written without prefix - outer blockquote will add it if needed
                        if let Some(url) = cite {
                            output.push('\n');
                            output.push_str("— <");
                            output.push_str(&url);
                            output.push_str(">\n\n");
                        } else {
                            output.push('\n');
                        }
                    }
                }

                // Line break
                "br" => {
                    use crate::options::NewlineStyle;
                    match options.newline_style {
                        NewlineStyle::Spaces => output.push_str("  \n"),
                        NewlineStyle::Backslash => output.push_str("\\\n"),
                    }
                }

                // Horizontal rule
                "hr" => {
                    output.push_str("---\n\n");
                }

                // Lists
                "ul" => {
                    // In table cells, check if we need <br> separator
                    if ctx.in_table_cell {
                        let is_table_continuation = !output.is_empty()
                            && !output.ends_with('|')
                            && !output.ends_with(' ')
                            && !output.ends_with("<br>");
                        if is_table_continuation {
                            output.push_str("<br>");
                        }
                    } else if !output.is_empty() && !ctx.in_list {
                        // For lists outside of other lists, add block separation if needed
                        // (e.g., list after text in a dd element)
                        let needs_newline = !output.ends_with("\n\n")
                            && !output.ends_with("* ")
                            && !output.ends_with("- ")
                            && !output.ends_with(". ");
                        if needs_newline {
                            output.push_str("\n\n");
                        }
                    }

                    // If in a list item with content before the nested list, add newline
                    if ctx.in_list_item && !output.is_empty() {
                        let needs_newline = !output.ends_with('\n')
                            && !output.ends_with("* ")
                            && !output.ends_with("- ")
                            && !output.ends_with(". ");
                        if needs_newline {
                            // Trim trailing spaces/tabs before adding newlines
                            while output.ends_with(' ') || output.ends_with('\t') {
                                output.pop();
                            }
                            output.push_str("\n\n");
                        }
                    }

                    // If in a list but NOT in a list item, this is incorrectly nested - increment depth
                    // If in a list item, the depth was already incremented by the <li>
                    let nested_depth = if ctx.in_list && !ctx.in_list_item {
                        ctx.list_depth + 1
                    } else {
                        ctx.list_depth
                    };

                    // Check if this is a "loose" list (any li contains paragraphs)
                    // Lists with paragraphs should have blank lines between ALL items
                    // Note: divs are handled via has_block_children per-item, not loose list status
                    let mut is_loose = false;
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name, .. } = &child.data {
                            if name.local.as_ref() == "li" {
                                // Check if this li contains paragraph elements
                                for li_child in child.children.borrow().iter() {
                                    if let NodeData::Element { name, .. } = &li_child.data {
                                        if name.local.as_ref() == "p" {
                                            is_loose = true;
                                            break;
                                        }
                                    }
                                }
                                if is_loose {
                                    break;
                                }
                            }
                        }
                    }

                    let mut prev_had_blocks = false;
                    for child in handle.children.borrow().iter() {
                        // Skip whitespace-only text nodes between list items
                        if let NodeData::Text { contents } = &child.data {
                            if contents.borrow().trim().is_empty() {
                                continue;
                            }
                        }

                        let list_ctx = Context {
                            in_ordered_list: false,
                            list_counter: 0,
                            in_list: true,
                            list_depth: nested_depth,
                            loose_list: is_loose,
                            prev_item_had_blocks: prev_had_blocks,
                            ..ctx.clone()
                        };
                        let before_len = output.len();
                        walk_node(child, output, options, &list_ctx, depth);

                        // Check if this li had block children by detecting if output has continuation patterns
                        let li_output = &output[before_len..];
                        let had_blocks = li_output.contains("\n\n    ") || li_output.contains("\n    ");
                        prev_had_blocks = had_blocks;
                    }
                    // Nested lists (in list items) should add trailing newline to separate from following content
                    // This creates blank line before next continuation element (if there is one)
                    if ctx.in_list_item {
                        // Only add extra newline if not already ending with double newline
                        // (which would be the case if it's the last child of the list item)
                        if !output.ends_with("\n\n") {
                            if !output.ends_with('\n') {
                                output.push('\n');
                            }
                            // Add another newline to create blank line
                            output.push('\n');
                        }
                    }
                }

                "ol" => {
                    // For lists outside of other lists, add block separation if needed
                    if !output.is_empty() && !ctx.in_list && !ctx.in_table_cell {
                        let needs_newline = !output.ends_with("\n\n")
                            && !output.ends_with("* ")
                            && !output.ends_with("- ")
                            && !output.ends_with(". ");
                        if needs_newline {
                            output.push_str("\n\n");
                        }
                    }

                    // If in a list item with content before the nested list, add newline
                    if ctx.in_list_item && !output.is_empty() {
                        let needs_newline = !output.ends_with('\n')
                            && !output.ends_with("* ")
                            && !output.ends_with("- ")
                            && !output.ends_with(". ");
                        if needs_newline {
                            // Trim trailing spaces/tabs before adding newlines
                            while output.ends_with(' ') || output.ends_with('\t') {
                                output.pop();
                            }
                            output.push_str("\n\n");
                        }
                    }

                    // If in a list but NOT in a list item, this is incorrectly nested - increment depth
                    // If in a list item, the depth was already incremented by the <li>
                    let nested_depth = if ctx.in_list && !ctx.in_list_item {
                        ctx.list_depth + 1
                    } else {
                        ctx.list_depth
                    };

                    // Check if this is a "loose" list (any li contains paragraphs)
                    // Lists with paragraphs should have blank lines between ALL items
                    // Note: divs are handled via has_block_children per-item, not loose list status
                    let mut is_loose = false;
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name, .. } = &child.data {
                            if name.local.as_ref() == "li" {
                                // Check if this li contains paragraph elements
                                for li_child in child.children.borrow().iter() {
                                    if let NodeData::Element { name, .. } = &li_child.data {
                                        if name.local.as_ref() == "p" {
                                            is_loose = true;
                                            break;
                                        }
                                    }
                                }
                                if is_loose {
                                    break;
                                }
                            }
                        }
                    }

                    // Create base context for this list
                    let base_list_ctx = Context {
                        in_list: true,
                        list_depth: nested_depth,
                        loose_list: is_loose,
                        ..ctx.clone()
                    };

                    let mut counter = 1;
                    let mut prev_had_blocks = false;
                    for child in handle.children.borrow().iter() {
                        // Check if this is an li element
                        if let NodeData::Element { name, .. } = &child.data {
                            if name.local.as_ref() == "li" {
                                let list_ctx = Context {
                                    in_ordered_list: true,
                                    list_counter: counter,
                                    prev_item_had_blocks: prev_had_blocks,
                                    ..base_list_ctx.clone()
                                };
                                let before_len = output.len();
                                walk_node(child, output, options, &list_ctx, depth);
                                counter += 1;

                                // Check if this li had block children by detecting if output has continuation patterns
                                let li_output = &output[before_len..];
                                let had_blocks = li_output.contains("\n\n    ") || li_output.contains("\n    ");
                                prev_had_blocks = had_blocks;

                                continue;
                            }
                        }
                        // Skip whitespace-only text nodes between list items
                        if let NodeData::Text { contents } = &child.data {
                            if contents.borrow().trim().is_empty() {
                                continue;
                            }
                        }
                        // For non-li children, use base context with in_list set
                        walk_node(child, output, options, &base_list_ctx, depth);
                    }
                    // Nested lists (in list items) should add trailing newline to separate from following content
                    // This creates blank line before next continuation element (if there is one)
                    if ctx.in_list_item {
                        // Only add extra newline if not already ending with double newline
                        // (which would be the case if it's the last child of the list item)
                        if !output.ends_with("\n\n") {
                            if !output.ends_with('\n') {
                                output.push('\n');
                            }
                            // Add another newline to create blank line
                            output.push('\n');
                        }
                    }
                }

                "li" => {
                    // Indentation for nested lists only (list_depth > 0)
                    if ctx.list_depth > 0 {
                        output.push_str(&" ".repeat(ctx.list_depth * options.list_indent_width));
                    }

                    // Check if this li contains block-level elements (p, div, ul, ol, etc.)
                    let mut has_block_children = false;
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name, .. } = &child.data {
                            let tag_name = name.local.as_ref();
                            if matches!(
                                tag_name,
                                "p" | "div"
                                    | "ul"
                                    | "ol"
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

                    // Check if this is a task list item (contains checkbox as first child)
                    let mut is_task_list = false;
                    let mut task_checked = false;
                    let mut skip_first_input = false;

                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name, attrs, .. } = &child.data {
                            if name.local.as_ref() == "input" {
                                let input_type = attrs
                                    .borrow()
                                    .iter()
                                    .find(|attr| attr.name.local.as_ref() == "type")
                                    .map(|attr| attr.value.to_string());

                                if input_type.as_deref() == Some("checkbox") {
                                    is_task_list = true;
                                    task_checked =
                                        attrs.borrow().iter().any(|attr| attr.name.local.as_ref() == "checked");
                                    skip_first_input = true;
                                    break;
                                }
                            }
                        }
                        // Stop checking after first non-whitespace element
                        if let NodeData::Text { contents } = &child.data {
                            if !contents.borrow().trim().is_empty() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    // Create context for list item children
                    let li_ctx = Context {
                        in_list_item: true,
                        list_depth: ctx.list_depth + 1,
                        ..ctx.clone()
                    };

                    if is_task_list {
                        // Task list item
                        output.push('-');
                        output.push(' ');
                        output.push_str(if task_checked { "[x]" } else { "[ ]" });

                        // Process children, skipping the checkbox input
                        let mut first_input_seen = false;
                        let mut task_text = String::new();
                        for child in handle.children.borrow().iter() {
                            if !first_input_seen && skip_first_input {
                                if let NodeData::Element { name, attrs, .. } = &child.data {
                                    if name.local.as_ref() == "input" {
                                        let input_type = attrs
                                            .borrow()
                                            .iter()
                                            .find(|attr| attr.name.local.as_ref() == "type")
                                            .map(|attr| attr.value.to_string());

                                        if input_type.as_deref() == Some("checkbox") {
                                            first_input_seen = true;
                                            continue;
                                        }
                                    }
                                }
                            }
                            walk_node(child, &mut task_text, options, &li_ctx, depth + 1);
                        }
                        // Always add space after checkbox, then add trimmed text
                        output.push(' ');
                        let trimmed_task = task_text.trim();
                        if !trimmed_task.is_empty() {
                            output.push_str(trimmed_task);
                        }
                    } else {
                        // Regular list item
                        // In table cells, don't add bullets
                        if !ctx.in_table_cell {
                            if ctx.in_ordered_list {
                                // Ordered list - use number
                                output.push_str(&format!("{}. ", ctx.list_counter));
                            } else {
                                // Unordered list - use bullet from options.bullets based on depth
                                let bullets: Vec<char> = options.bullets.chars().collect();
                                let bullet = bullets.get(ctx.list_depth % bullets.len()).copied().unwrap_or('*');
                                output.push(bullet);
                                output.push(' ');
                            }
                        }

                        // Process children
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, &li_ctx, depth + 1);
                        }

                        // Trim trailing spaces/tabs but preserve newlines
                        while output.ends_with(' ') || output.ends_with('\t') {
                            output.pop();
                        }
                    }

                    // Ensure list items end with proper separator (but not in table cells)
                    if !ctx.in_table_cell {
                        // Use \n\n if:
                        // - Item has block children, OR
                        // - List is loose (contains paragraphs), OR
                        // - Previous item had block children
                        if has_block_children || ctx.loose_list || ctx.prev_item_had_blocks {
                            // List items with block children or in loose lists should end with \n\n
                            if !output.ends_with("\n\n") {
                                if output.ends_with('\n') {
                                    output.push('\n');
                                } else {
                                    output.push_str("\n\n");
                                }
                            }
                        } else {
                            // Simple list items in tight lists should end with single \n
                            if !output.ends_with('\n') {
                                output.push('\n');
                            }
                        }
                    }
                }

                // Tables - process at table level to track row indices
                "table" => {
                    // Add leading block separation (tables always start with \n\n)
                    if !output.ends_with("\n\n") {
                        if output.is_empty() || !output.ends_with('\n') {
                            output.push_str("\n\n");
                        } else {
                            output.push('\n');
                        }
                    }
                    convert_table(handle, output, options, ctx);
                    output.push('\n');
                }

                "thead" | "tbody" | "tfoot" | "tr" | "th" | "td" => {
                    // These are handled by convert_table()
                    // Don't process them directly to avoid double-processing
                }

                "caption" => {
                    let mut text = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, ctx, depth + 1);
                    }
                    let text = text.trim();
                    if !text.is_empty() {
                        output.push('*');
                        output.push_str(text);
                        output.push_str("*\n\n");
                    }
                }

                "colgroup" | "col" => {
                    // Skip colgroup and col elements
                }

                // Semantic block elements
                "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" => {
                    // In inline mode, just process children
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }

                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth);
                    }
                    // Check if content has non-whitespace (don't trim - preserve whitespace)
                    if content.trim().is_empty() {
                        return;
                    }

                    if !output.is_empty() && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }
                    output.push_str(&content);
                    // Ensure output ends with \n\n
                    if content.ends_with('\n') && !content.ends_with("\n\n") {
                        output.push('\n');
                    } else if !content.ends_with('\n') {
                        output.push_str("\n\n");
                    }
                }

                "figure" => {
                    // In inline mode, just process children
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }

                    // Process children into a temporary buffer
                    let mut figure_content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut figure_content, options, ctx, depth);
                    }

                    // Strip leading newlines and output
                    let trimmed = figure_content.trim_start_matches('\n');
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                        // Add trailing block separation if needed
                        if !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                    }
                }

                "figcaption" => {
                    let mut text = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, ctx, depth + 1);
                    }
                    let text = text.trim();
                    if !text.is_empty() {
                        // Add block separation before caption if there's prior content
                        if !output.is_empty() {
                            if output.ends_with("```\n") {
                                // After code block, add single newline
                                output.push('\n');
                            } else if !output.ends_with("\n\n") {
                                // Otherwise add double newline
                                output.push_str("\n\n");
                            }
                        }
                        output.push('*');
                        output.push_str(text);
                        output.push_str("*\n\n");
                    }
                }

                "hgroup" => {
                    // Process children (h1-h6) normally
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }
                }

                // Citation and quote tags
                "cite" => {
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    // Trim whitespace from content
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
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            output.push('"');
                            // Escape inner quotes
                            let escaped = trimmed.replace('"', r#"\""#);
                            output.push_str(&escaped);
                            output.push('"');
                        }
                    }
                }

                // Definition lists
                "dl" => {
                    // In inline mode, just process children
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }

                    // Process children and track whether the previous element was a dt or dd after dt
                    let mut content = String::new();
                    let mut in_dt_group = false;
                    for child in handle.children.borrow().iter() {
                        // Check element type
                        let (is_dt, is_dd) = if let NodeData::Element { name, .. } = &child.data {
                            (name.local.as_ref() == "dt", name.local.as_ref() == "dd")
                        } else {
                            (false, false)
                        };

                        // dd elements in a dt group should have the prefix
                        let child_ctx = Context {
                            last_was_dt: in_dt_group && is_dd,
                            ..ctx.clone()
                        };
                        walk_node(child, &mut content, options, &child_ctx, depth);

                        // Update tracking
                        if is_dt {
                            in_dt_group = true; // Start a dt group
                        } else if !is_dd {
                            in_dt_group = false; // Non-dd/dt element resets the group
                        }
                        // Note: dd elements don't change the flag, they stay in the group
                    }

                    // Format as block element: strip and add \n\n if not empty
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        // Add leading newlines if output already has content
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "dt" => {
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            // In inline mode, just output text
                            output.push_str(trimmed);
                        } else {
                            // In normal mode, output text + newline
                            output.push_str(trimmed);
                            output.push('\n');
                        }
                    }
                }

                "dd" => {
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }

                    let trimmed = content.trim();

                    if ctx.convert_as_inline {
                        // In inline mode, just output text
                        if !trimmed.is_empty() {
                            output.push_str(trimmed);
                        }
                    } else {
                        // In normal mode, use :   prefix if previous sibling was dt
                        if ctx.last_was_dt {
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
                }

                // Interactive elements
                "details" => {
                    // In inline mode, just process children
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }

                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth);
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
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
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
                    // In inline mode, just process children
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }

                    // Track content start
                    let content_start = output.len();

                    // Process children
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }

                    // Trim trailing whitespace (but not newlines) from dialog content
                    while output.len() > content_start && (output.ends_with(' ') || output.ends_with('\t')) {
                        output.pop();
                    }

                    // Add trailing block separation if dialog had content
                    if output.len() > content_start && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }
                }

                "menu" => {
                    // Track content start for inline mode
                    let content_start = output.len();

                    // Create custom options with '-' as the only bullet
                    let menu_options = ConversionOptions {
                        bullets: "-".to_string(),
                        ..options.clone()
                    };

                    // Treat like ul with forced '-' bullet
                    let list_ctx = Context {
                        in_ordered_list: false,
                        list_counter: 0,
                        in_list: true,
                        list_depth: ctx.list_depth,
                        ..ctx.clone()
                    };

                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, &menu_options, &list_ctx, depth);
                    }

                    // Add trailing block separation if menu had content and not in inline mode
                    if !ctx.convert_as_inline && output.len() > content_start {
                        if !output.ends_with("\n\n") {
                            if output.ends_with('\n') {
                                output.push('\n');
                            } else {
                                output.push_str("\n\n");
                            }
                        }
                    } else if ctx.convert_as_inline {
                        // In inline mode, remove trailing newline if present
                        while output.ends_with('\n') {
                            output.pop();
                        }
                    }
                }

                // Media elements
                "audio" => {
                    let src = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "src")
                        .map(|attr| attr.value.to_string())
                        .or_else(|| {
                            // Check for source child
                            for child in handle.children.borrow().iter() {
                                if let NodeData::Element {
                                    name,
                                    attrs: child_attrs,
                                    ..
                                } = &child.data
                                {
                                    if name.local.as_ref() == "source" {
                                        return child_attrs
                                            .borrow()
                                            .iter()
                                            .find(|attr| attr.name.local.as_ref() == "src")
                                            .map(|attr| attr.value.to_string());
                                    }
                                }
                            }
                            None
                        })
                        .unwrap_or_default();

                    if !src.is_empty() {
                        // Output [src](src) format
                        output.push('[');
                        output.push_str(&src);
                        output.push_str("](");
                        output.push_str(&src);
                        output.push(')');
                        // Add block spacing only when not inside paragraph/inline contexts
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }

                    // Process fallback content (non-source children)
                    let mut fallback = String::new();
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name, .. } = &child.data {
                            if name.local.as_ref() != "source" {
                                walk_node(child, &mut fallback, options, ctx, depth + 1);
                            }
                        } else {
                            // Process text nodes
                            walk_node(child, &mut fallback, options, ctx, depth + 1);
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
                    let src = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "src")
                        .map(|attr| attr.value.to_string())
                        .or_else(|| {
                            // Check for source child
                            for child in handle.children.borrow().iter() {
                                if let NodeData::Element {
                                    name,
                                    attrs: child_attrs,
                                    ..
                                } = &child.data
                                {
                                    if name.local.as_ref() == "source" {
                                        return child_attrs
                                            .borrow()
                                            .iter()
                                            .find(|attr| attr.name.local.as_ref() == "src")
                                            .map(|attr| attr.value.to_string());
                                    }
                                }
                            }
                            None
                        })
                        .unwrap_or_default();

                    if !src.is_empty() {
                        // Output [src](src) format
                        output.push('[');
                        output.push_str(&src);
                        output.push_str("](");
                        output.push_str(&src);
                        output.push(')');
                        // Add block spacing only when not inside paragraph/inline contexts
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }

                    // Process fallback content (non-source children)
                    let mut fallback = String::new();
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name, .. } = &child.data {
                            if name.local.as_ref() != "source" {
                                walk_node(child, &mut fallback, options, ctx, depth + 1);
                            }
                        } else {
                            // Process text nodes
                            walk_node(child, &mut fallback, options, ctx, depth + 1);
                        }
                    }
                    if !fallback.is_empty() {
                        output.push_str(fallback.trim());
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "source" => {
                    // Handled by parent audio/video element
                }

                "picture" => {
                    // Extract the img element or first source
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name, .. } = &child.data {
                            if name.local.as_ref() == "img" {
                                walk_node(child, output, options, ctx, depth);
                                break;
                            }
                        }
                    }
                }

                "iframe" => {
                    let src = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "src")
                        .map(|attr| attr.value.to_string())
                        .unwrap_or_default();

                    if !src.is_empty() {
                        // Output [url](url) format
                        output.push('[');
                        output.push_str(&src);
                        output.push_str("](");
                        output.push_str(&src);
                        output.push(')');
                        // Add block spacing only when not inside paragraph/inline contexts
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "svg" | "math" => {
                    // For now, extract text content
                    // TODO: Could make this configurable (inline vs extract)
                    let mut text = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, ctx, depth + 1);
                    }
                    let text = text.trim();
                    if !text.is_empty() {
                        output.push_str(text);
                    }
                }

                // Form elements
                "form" => {
                    // In inline mode, just process children
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }

                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth);
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
                    // In inline mode, just process children
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth);
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
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            // In inline mode, just output text
                            output.push_str(trimmed);
                        } else {
                            // In normal mode, output **text**
                            let symbol = options.strong_em_symbol.to_string().repeat(2);
                            output.push_str(&symbol);
                            output.push_str(trimmed);
                            output.push_str(&symbol);
                            output.push_str("\n\n");
                        }
                    }
                }

                "label" => {
                    // Collect label content to a buffer so we can trim it
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                        if !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "input" => {
                    // Input elements output nothing by themselves
                    // Checkboxes in task lists are handled in the list item logic
                }

                "textarea" => {
                    // Output text content
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    // Add trailing newlines if content was added (but not in inline mode)
                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "select" => {
                    // Process options
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    // Add trailing newline if content was added (but not in inline mode)
                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push('\n');
                    }
                }

                "option" => {
                    // Check if option is selected
                    let selected = attrs.borrow().iter().any(|attr| attr.name.local.as_ref() == "selected");

                    // Output option text followed by newline (unless in inline mode)
                    let mut text = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, ctx, depth + 1);
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
                    let label = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "label")
                        .map(|attr| attr.value.to_string())
                        .unwrap_or_default();

                    if !label.is_empty() {
                        let symbol = options.strong_em_symbol.to_string().repeat(2);
                        output.push_str(&symbol);
                        output.push_str(&label);
                        output.push_str(&symbol);
                        output.push('\n');
                    }

                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }
                }

                "button" => {
                    // Output button text content
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    // Add trailing newlines if content was added (but not in inline mode)
                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "progress" => {
                    // Output children text content
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    // Add trailing newlines if content was added (but not in inline mode)
                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "meter" => {
                    // Output children text content
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    // Add trailing newlines if content was added (but not in inline mode)
                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "output" => {
                    // Output children text content
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    // Add trailing newlines if content was added (but not in inline mode)
                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "datalist" => {
                    // Process options (same as select)
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    // Add trailing newline if content was added (but not in inline mode)
                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push('\n');
                    }
                }

                // Ruby annotations
                "ruby" => {
                    // Process ruby content: handle rb, rt, and rtc elements
                    let ruby_ctx = ctx.clone();

                    // Analyze structure: collect tag names to check pattern
                    let tag_sequence: Vec<String> = handle
                        .children
                        .borrow()
                        .iter()
                        .filter_map(|child| {
                            if let NodeData::Element { name, .. } = &child.data {
                                let tag = name.local.as_ref();
                                if tag == "rb" || tag == "rt" || tag == "rtc" {
                                    Some(tag.to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Check if there's an rtc element
                    let has_rtc = tag_sequence.iter().any(|tag| tag == "rtc");

                    // Check if rb/rt are interleaved (rb rt rb rt) vs grouped (rb rb rt rt)
                    let is_interleaved = tag_sequence.windows(2).any(|w| w[0] == "rb" && w[1] == "rt");

                    if is_interleaved && !has_rtc {
                        // Interleaved pattern: output rb(rt) pairs as we go
                        let mut current_base = String::new();
                        for child in handle.children.borrow().iter() {
                            match &child.data {
                                NodeData::Element { name, .. } => {
                                    let tag_name = name.local.as_ref();
                                    if tag_name == "rt" {
                                        let mut annotation = String::new();
                                        walk_node(child, &mut annotation, options, &ruby_ctx, depth);
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
                                        walk_node(child, &mut current_base, options, &ruby_ctx, depth);
                                    } else if tag_name != "rp" {
                                        walk_node(child, &mut current_base, options, &ruby_ctx, depth);
                                    }
                                }
                                NodeData::Text { .. } => {
                                    walk_node(child, &mut current_base, options, &ruby_ctx, depth);
                                }
                                _ => {}
                            }
                        }
                        if !current_base.is_empty() {
                            output.push_str(current_base.trim());
                        }
                    } else {
                        // Grouped pattern: collect all, then output base + (annotations) + rtc
                        let mut base_text = String::new();
                        let mut rt_annotations = Vec::new();
                        let mut rtc_content = String::new();

                        for child in handle.children.borrow().iter() {
                            match &child.data {
                                NodeData::Element { name, .. } => {
                                    let tag_name = name.local.as_ref();
                                    if tag_name == "rt" {
                                        let mut annotation = String::new();
                                        walk_node(child, &mut annotation, options, &ruby_ctx, depth);
                                        rt_annotations.push(annotation);
                                    } else if tag_name == "rtc" {
                                        walk_node(child, &mut rtc_content, options, &ruby_ctx, depth);
                                    } else if tag_name != "rp" {
                                        walk_node(child, &mut base_text, options, &ruby_ctx, depth);
                                    }
                                }
                                NodeData::Text { .. } => {
                                    walk_node(child, &mut base_text, options, &ruby_ctx, depth);
                                }
                                _ => {}
                            }
                        }

                        // Check if base text has trailing whitespace before trimming
                        let trimmed_base = base_text.trim();

                        // Output base text - trimmed
                        output.push_str(trimmed_base);

                        if !rt_annotations.is_empty() {
                            let rt_text = rt_annotations.iter().map(|s| s.trim()).collect::<Vec<_>>().join("");
                            // No space before annotation - annotations already include parentheses
                            // Ruby base text and annotations should be adjacent: 漢字(kanji)
                            if !rt_text.is_empty() {
                                // Wrap rt annotations in extra parentheses only if multiple rt + rtc
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
                    // Ruby base text - collect into temp buffer and trim
                    let mut text = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, ctx, depth + 1);
                    }
                    output.push_str(text.trim());
                }

                "rt" => {
                    // Ruby annotation text - check for rp siblings to decide on parentheses
                    let mut text = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, ctx, depth + 1);
                    }
                    let trimmed = text.trim();

                    // Heuristic: if output ends with '(' from a preceding <rp>, don't add parentheses
                    // This handles: <rp>(</rp><rt>text</rt><rp>)</rp> -> (text)
                    if output.ends_with('(') {
                        // Previous rp added opening paren, just output content
                        output.push_str(trimmed);
                    } else {
                        // No rp before, wrap in parentheses
                        output.push('(');
                        output.push_str(trimmed);
                        output.push(')');
                    }
                }

                "rp" => {
                    // Ruby parentheses - output the text content (stripped)
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                    }
                }

                "rtc" => {
                    // Ruby text container - process children
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }
                }

                // Div - handle list item indentation and table cell <br>
                "div" => {
                    // Track where the content for this div starts
                    let content_start_pos = output.len();

                    // Check if this is a continuation in a table cell
                    // In table cells, if there's any content (not just starting with |), add <br>
                    let is_table_continuation =
                        ctx.in_table_cell && !output.is_empty() && !output.ends_with('|') && !output.ends_with("<br>");

                    // Check if this is a continuation div in a list item
                    let is_list_continuation = ctx.in_list_item
                        && !output.is_empty()
                        && !output.ends_with("* ")
                        && !output.ends_with("- ")
                        && !output.ends_with(". ");

                    // Check if we need leading block separation (not in table/list, has preceding content)
                    let needs_leading_sep = !ctx.in_table_cell
                        && !ctx.in_list_item
                        && !ctx.convert_as_inline
                        && !output.is_empty()
                        && !output.ends_with("\n\n");

                    if is_table_continuation {
                        // In table cells, separate multiple block elements with <br>
                        // Trim trailing whitespace first
                        while output.ends_with(' ') || output.ends_with('\t') {
                            output.pop();
                        }
                        output.push_str("<br>");
                    } else if is_list_continuation {
                        // Trim trailing spaces/tabs before adding newline
                        while output.ends_with(' ') || output.ends_with('\t') {
                            output.pop();
                        }
                        // Add newline before indentation if not already present
                        // For consecutive divs, we want single newline between them
                        if !output.ends_with('\n') {
                            output.push('\n');
                        }
                        // Continuation indentation: base indentation + content indentation
                        // Formula: (list_depth - 1) * 4 spaces for nesting + list_depth * 4 for continuation
                        // Simplified: (2 * list_depth - 1) * 4 spaces
                        let indent_level = if ctx.list_depth > 0 { 2 * ctx.list_depth - 1 } else { 0 };
                        output.push_str(&"    ".repeat(indent_level));
                    } else if needs_leading_sep {
                        // Trim trailing whitespace before adding block separation
                        while output.ends_with(' ') || output.ends_with('\t') {
                            output.pop();
                        }
                        // Add leading block separation for divs in normal context
                        output.push_str("\n\n");
                    }

                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }

                    // Only add trailing separator if div had content
                    let has_content = output.len() > content_start_pos;

                    if has_content {
                        // Trim trailing whitespace from div content (but not newlines)
                        while output.ends_with(' ') || output.ends_with('\t') {
                            output.pop();
                        }

                        // Add trailing newlines based on context
                        if ctx.in_table_cell {
                            // No trailing separator in table cells
                        } else if ctx.in_list_item {
                            // In list items:
                            // - First div (non-continuation) adds \n\n for blank line before next continuation
                            // - Continuation div adds \n to separate from next element
                            if is_list_continuation {
                                // Continuation div: add single newline
                                if !output.ends_with('\n') {
                                    output.push('\n');
                                }
                            } else {
                                // First div (inline with bullet): add double newline
                                if !output.ends_with("\n\n") {
                                    if output.ends_with('\n') {
                                        output.push('\n');
                                    } else {
                                        output.push_str("\n\n");
                                    }
                                }
                            }
                        } else if !ctx.in_list_item && !ctx.convert_as_inline {
                            // In normal context, ensure double newline block separation
                            if output.ends_with("\n\n") {
                                // Already has proper separation
                            } else if output.ends_with('\n') {
                                // Has single newline, add one more
                                output.push('\n');
                            } else {
                                // No newline, add double
                                output.push_str("\n\n");
                            }
                        }
                    }
                }

                // Head - always skip rendering (metadata extracted separately if enabled)
                "head" => {
                    // Skip - head content should never be rendered as markdown
                }

                // Span - check for hOCR word elements
                "span" => {
                    // Check if this is an hOCR word element (class="ocrx_word")
                    let is_hocr_word = attrs
                        .borrow()
                        .iter()
                        .any(|attr| attr.name.local.as_ref() == "class" && attr.value.as_ref().contains("ocrx_word"));

                    if is_hocr_word {
                        // Add space before hOCR word if output doesn't already end with whitespace
                        if !output.is_empty()
                            && !output.ends_with(' ')
                            && !output.ends_with('\t')
                            && !output.ends_with('\n')
                        {
                            output.push(' ');
                        }
                    }

                    // Process children normally
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }
                }

                // Other containers - just process children
                _ => {
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }
                }
            }
        }

        NodeData::Comment { .. } => {
            // Skip comments
        }

        NodeData::Doctype { .. } => {
            // Skip doctype
        }

        NodeData::ProcessingInstruction { .. } => {
            // Skip processing instructions
        }
    }
}

/// Get colspan attribute value from element
fn get_colspan(handle: &Handle) -> usize {
    if let NodeData::Element { attrs, .. } = &handle.data {
        for attr in attrs.borrow().iter() {
            if attr.name.local.as_ref() == "colspan" {
                if let Ok(colspan) = attr.value.to_string().parse::<usize>() {
                    return colspan;
                }
            }
        }
    }
    1
}

fn get_rowspan(handle: &Handle) -> usize {
    if let NodeData::Element { attrs, .. } = &handle.data {
        for attr in attrs.borrow().iter() {
            if attr.name.local.as_ref() == "rowspan" {
                if let Ok(rowspan) = attr.value.to_string().parse::<usize>() {
                    return rowspan;
                }
            }
        }
    }
    1
}

/// Convert table cell (td or th)
fn convert_table_cell(
    handle: &Handle,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    _tag_name: &str,
) {
    let mut text = String::new();

    // Process children with in_table_cell context
    let cell_ctx = Context {
        in_table_cell: true,
        ..ctx.clone()
    };

    if let NodeData::Element { .. } = &handle.data {
        for child in handle.children.borrow().iter() {
            walk_node(child, &mut text, options, &cell_ctx, 0);
        }
    }

    // Replace multiple newlines with <br> if enabled, otherwise collapse to space
    let text = text.trim();
    let text = if options.br_in_tables {
        // Replace one or more newlines with a single <br>
        text.split('\n')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("<br>")
    } else {
        text.replace('\n', " ")
    };

    let colspan = get_colspan(handle);

    output.push(' ');
    output.push_str(&text);
    output.push_str(&" |".repeat(colspan));
}

/// Convert table row (tr)
fn convert_table_row(
    handle: &Handle,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    row_index: usize,
    rowspan_tracker: &mut std::collections::HashMap<usize, usize>,
) {
    let mut row_text = String::new();
    let mut cells = Vec::new();

    // Collect cells
    if let NodeData::Element { .. } = &handle.data {
        for child in handle.children.borrow().iter() {
            if let NodeData::Element { name, .. } = &child.data {
                let cell_name = name.local.as_ref();
                if cell_name == "th" || cell_name == "td" {
                    cells.push(child.clone());
                }
            }
        }
    }

    // Process cells and track column positions with rowspan handling
    let mut col_index = 0;
    let mut cell_iter = cells.iter();

    loop {
        // Check if there's an active rowspan cell for this column
        if let Some(remaining_rows) = rowspan_tracker.get_mut(&col_index) {
            if *remaining_rows > 0 {
                // Insert empty placeholder for this column
                row_text.push_str(" |");
                *remaining_rows -= 1;
                if *remaining_rows == 0 {
                    rowspan_tracker.remove(&col_index);
                }
                col_index += 1;
                continue;
            }
        }

        // Process next actual cell
        if let Some(cell_handle) = cell_iter.next() {
            convert_table_cell(cell_handle, &mut row_text, options, ctx, "");

            let colspan = get_colspan(cell_handle);
            let rowspan = get_rowspan(cell_handle);

            // If this cell has rowspan > 1, track it for future rows
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

    // Add header separator after first row
    let is_first_row = row_index == 0;
    if is_first_row {
        let total_cols = cells.iter().map(get_colspan).sum::<usize>().max(1);
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
fn convert_table(handle: &Handle, output: &mut String, options: &ConversionOptions, ctx: &Context) {
    if let NodeData::Element { .. } = &handle.data {
        let mut row_index = 0;
        let mut rowspan_tracker = std::collections::HashMap::new();

        // Process all table children in order
        for child in handle.children.borrow().iter() {
            if let NodeData::Element { name, .. } = &child.data {
                let tag_name = name.local.as_ref();

                match tag_name {
                    "caption" => {
                        // Caption is handled separately
                        let mut text = String::new();
                        for grandchild in child.children.borrow().iter() {
                            walk_node(grandchild, &mut text, options, ctx, 0);
                        }
                        let text = text.trim();
                        if !text.is_empty() {
                            output.push('*');
                            output.push_str(text);
                            output.push_str("*\n\n");
                        }
                    }

                    "thead" | "tbody" | "tfoot" => {
                        // Process rows within these sections
                        for row_child in child.children.borrow().iter() {
                            if let NodeData::Element { name: row_name, .. } = &row_child.data {
                                if row_name.local.as_ref() == "tr" {
                                    convert_table_row(row_child, output, options, ctx, row_index, &mut rowspan_tracker);
                                    row_index += 1;
                                }
                            }
                        }
                    }

                    "tr" => {
                        // Direct tr children (no thead/tbody)
                        convert_table_row(child, output, options, ctx, row_index, &mut rowspan_tracker);
                        row_index += 1;
                    }

                    "colgroup" | "col" => {
                        // Skip colgroup and col elements
                    }

                    _ => {
                        // Skip other elements
                    }
                }
            }
        }
    }
}
