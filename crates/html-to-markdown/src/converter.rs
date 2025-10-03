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

/// Indentation width for list items (4 spaces per level).
#[allow(dead_code)]
const LIST_INDENT_WIDTH: usize = 4;

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
fn is_loose_list(handle: &Handle) -> bool {
    for child in handle.children.borrow().iter() {
        if let NodeData::Element { name, .. } = &child.data {
            if name.local.as_ref() == "li" {
                for li_child in child.children.borrow().iter() {
                    if let NodeData::Element { name, .. } = &li_child.data {
                        if name.local.as_ref() == "p" {
                            return true;
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
fn add_list_continuation_indent(output: &mut String, list_depth: usize, blank_line: bool) {
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
    output.push_str(&"    ".repeat(indent_level));
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
            output.push_str("\n\n");
        }
    }
}

/// Add appropriate trailing separator after a nested list.
///
/// Nested lists inside list items need a trailing blank line to separate
/// from following content (if any).
fn add_nested_list_trailing_separator(output: &mut String, ctx: &Context) {
    if ctx.in_list_item && !output.ends_with("\n\n") {
        if !output.ends_with('\n') {
            output.push('\n');
        }
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
    handle: &Handle,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    is_ordered: bool,
    is_loose: bool,
    nested_depth: usize,
) {
    let mut counter = 1;
    let mut prev_had_blocks = false;

    for child in handle.children.borrow().iter() {
        if let NodeData::Text { contents } = &child.data {
            if contents.borrow().trim().is_empty() {
                continue;
            }
        }

        let list_ctx = Context {
            in_ordered_list: is_ordered,
            list_counter: if is_ordered { counter } else { 0 },
            in_list: true,
            list_depth: nested_depth,
            loose_list: is_loose,
            prev_item_had_blocks: prev_had_blocks,
            ..ctx.clone()
        };

        let before_len = output.len();
        walk_node(child, output, options, &list_ctx, depth);

        let li_output = &output[before_len..];
        let had_blocks = li_output.contains("\n\n    ") || li_output.contains("\n    ");
        prev_had_blocks = had_blocks;

        if is_ordered {
            if let NodeData::Element { name, .. } = &child.data {
                if name.local.as_ref() == "li" {
                    counter += 1;
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

                for child in handle.children.borrow().iter() {
                    if check_node(child) {
                        return true;
                    }
                }
                false
            }
            NodeData::Document => {
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

    for child in head.children.borrow().iter() {
        if let NodeData::Element { name, attrs, .. } = &child.data {
            let tag_name = name.local.as_ref();

            match tag_name {
                "title" => {
                    if let Some(text_node) = child.children.borrow().first() {
                        if let NodeData::Text { contents } = &text_node.data {
                            let title = text::normalize_whitespace(&contents.borrow()).trim().to_string();
                            if !title.is_empty() {
                                metadata.insert("title".to_string(), title);
                            }
                        }
                    }
                }
                "base" => {
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
        let escaped_value = value.replace("-->", "--&gt;");
        lines.push(format!("{}: {}", key, escaped_value));
    }
    lines.push("-->".to_string());

    lines.join("\n") + "\n\n"
}

/// Check if a handle is an empty inline element (abbr, var, ins, dfn, etc. with no text content).
fn is_empty_inline_element(handle: &Handle) -> bool {
    const EMPTY_WHEN_NO_CONTENT_TAGS: &[&str] = &[
        "abbr", "var", "ins", "dfn", "time", "data", "cite", "q", "mark", "small", "u",
    ];

    if let NodeData::Element { name, .. } = &handle.data {
        let tag_name = name.local.as_ref();
        if EMPTY_WHEN_NO_CONTENT_TAGS.contains(&tag_name) {
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
    let dom = parse_document(RcDom::default(), Default::default()).one(html);

    let mut output = String::new();

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

    if options.hocr_extract_tables && is_hocr_document(&dom.document) {
        use crate::hocr::{extract_hocr_words, reconstruct_table, table_to_markdown};

        let words = extract_hocr_words(&dom.document, 0.0);

        if !words.is_empty() {
            let table = reconstruct_table(
                &words,
                options.hocr_table_column_threshold,
                options.hocr_table_row_threshold_ratio,
            );

            if !table.is_empty() {
                let table_markdown = table_to_markdown(&table);
                if !table_markdown.is_empty() {
                    output.push_str(&table_markdown);
                    output.push_str("\n\n");
                }
            }
        }
    }

    walk_node(&dom.document, &mut output, options, &ctx, 0);

    Ok(output)
}

/// Recursively walk DOM nodes and convert to Markdown.
#[allow(clippy::only_used_in_recursion)]
fn walk_node(handle: &Handle, output: &mut String, options: &ConversionOptions, ctx: &Context, depth: usize) {
    match &handle.data {
        NodeData::Document => {
            for child in handle.children.borrow().iter() {
                walk_node(child, output, options, ctx, depth);
            }
        }

        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();

            if text.is_empty() {
                return;
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

                if text.contains('\n') {
                    return;
                }

                let skip_whitespace = output.is_empty()
                    || output.ends_with("\n\n")
                    || output.ends_with("* ")
                    || output.ends_with("- ")
                    || output.ends_with(". ")
                    || output.ends_with("] ");

                let should_preserve =
                    (ctx.convert_as_inline || ctx.in_table_cell || !output.is_empty()) && !skip_whitespace;

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
                )
            } else {
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
                );
                final_text.push_str(&escaped_core);

                if !suffix.is_empty() {
                    final_text.push_str(suffix);
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

        NodeData::Element { name, attrs, .. } => {
            let tag_name = name.local.as_ref();

            match tag_name {
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
                        && !output.is_empty()
                        && !output.ends_with("\n\n")
                        && !after_code_block;

                    if is_table_continuation {
                        trim_trailing_whitespace(output);
                        output.push_str("<br>");
                    } else if is_list_continuation {
                        add_list_continuation_indent(output, ctx.list_depth, true);
                    } else if needs_leading_sep {
                        trim_trailing_whitespace(output);
                        output.push_str("\n\n");
                    }

                    let p_ctx = Context {
                        in_paragraph: true,
                        ..ctx.clone()
                    };

                    let children: Vec<_> = handle.children.borrow().iter().cloned().collect();
                    for (i, child) in children.iter().enumerate() {
                        if let NodeData::Text { contents } = &child.data {
                            let text = contents.borrow();
                            if text.trim().is_empty() && i > 0 && i < children.len() - 1 {
                                let prev = &children[i - 1];
                                let next = &children[i + 1];
                                if is_empty_inline_element(prev) && is_empty_inline_element(next) {
                                    continue;
                                }
                            }
                        }
                        walk_node(child, output, options, &p_ctx, depth + 1);
                    }

                    let has_content = output.len() > content_start_pos;

                    if has_content && !ctx.convert_as_inline && !ctx.in_table_cell {
                        output.push_str("\n\n");
                    }
                }

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

                    let should_use_alt_text = ctx.convert_as_inline
                        || (ctx.in_heading
                            && ctx
                                .heading_tag
                                .as_ref()
                                .map_or(true, |tag| !options.keep_inline_images_in.contains(tag)));

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
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();

                    if !trimmed.is_empty() {
                        output.push_str(trimmed);

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
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }
                }

                "wbr" => {}

                "code" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };
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

                "pre" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };

                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, &code_ctx, depth + 1);
                    }

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

                "blockquote" => {
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth + 1);
                        }
                        return;
                    }

                    let cite = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "cite")
                        .map(|attr| attr.value.to_string());

                    let blockquote_ctx = Context {
                        blockquote_depth: ctx.blockquote_depth + 1,
                        ..ctx.clone()
                    };
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, &blockquote_ctx, depth + 1);
                    }

                    let trimmed_content = content.trim_end_matches('\n');

                    if !trimmed_content.is_empty() {
                        if ctx.blockquote_depth > 0 {
                            output.push_str("\n\n\n");
                        } else if output.is_empty() {
                            output.push('\n');
                        } else if !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }

                        let prefix = "> ";

                        for line in trimmed_content.lines() {
                            output.push_str(prefix);
                            output.push_str(line.trim_end());
                            output.push('\n');
                        }

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

                "br" => {
                    use crate::options::NewlineStyle;
                    match options.newline_style {
                        NewlineStyle::Spaces => output.push_str("  \n"),
                        NewlineStyle::Backslash => output.push_str("\\\n"),
                    }
                }

                "hr" => {
                    output.push_str("---\n\n");
                }

                "ul" => {
                    add_list_leading_separator(output, ctx);

                    let nested_depth = calculate_list_nesting_depth(ctx);
                    let is_loose = is_loose_list(handle);

                    process_list_children(handle, output, options, ctx, depth, false, is_loose, nested_depth);

                    add_nested_list_trailing_separator(output, ctx);
                }

                "ol" => {
                    add_list_leading_separator(output, ctx);

                    let nested_depth = calculate_list_nesting_depth(ctx);
                    let is_loose = is_loose_list(handle);

                    process_list_children(handle, output, options, ctx, depth, true, is_loose, nested_depth);

                    add_nested_list_trailing_separator(output, ctx);
                }

                "li" => {
                    if ctx.list_depth > 0 {
                        output.push_str(&" ".repeat(ctx.list_depth * options.list_indent_width));
                    }

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
                        if let NodeData::Text { contents } = &child.data {
                            if !contents.borrow().trim().is_empty() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    let li_ctx = Context {
                        in_list_item: true,
                        list_depth: ctx.list_depth + 1,
                        ..ctx.clone()
                    };

                    if is_task_list {
                        output.push('-');
                        output.push(' ');
                        output.push_str(if task_checked { "[x]" } else { "[ ]" });

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
                                let bullet = bullets.get(ctx.list_depth % bullets.len()).copied().unwrap_or('*');
                                output.push(bullet);
                                output.push(' ');
                            }
                        }

                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, &li_ctx, depth + 1);
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
                    convert_table(handle, output, options, ctx);
                    output.push('\n');
                }

                "thead" | "tbody" | "tfoot" | "tr" | "th" | "td" => {}

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

                "colgroup" | "col" => {}

                "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" => {
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
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }

                    let mut figure_content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut figure_content, options, ctx, depth);
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
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, ctx, depth + 1);
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
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }
                }

                "cite" => {
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
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
                            let escaped = trimmed.replace('"', r#"\""#);
                            output.push_str(&escaped);
                            output.push('"');
                        }
                    }
                }

                "dl" => {
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }

                    let mut content = String::new();
                    let mut in_dt_group = false;
                    for child in handle.children.borrow().iter() {
                        let (is_dt, is_dd) = if let NodeData::Element { name, .. } = &child.data {
                            (name.local.as_ref() == "dt", name.local.as_ref() == "dd")
                        } else {
                            (false, false)
                        };

                        let child_ctx = Context {
                            last_was_dt: in_dt_group && is_dd,
                            ..ctx.clone()
                        };
                        walk_node(child, &mut content, options, &child_ctx, depth);

                        if is_dt {
                            in_dt_group = true;
                        } else if !is_dd {
                            in_dt_group = false;
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
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
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
                    let mut content = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
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
                    if ctx.convert_as_inline {
                        for child in handle.children.borrow().iter() {
                            walk_node(child, output, options, ctx, depth);
                        }
                        return;
                    }

                    let content_start = output.len();

                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
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

                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, &menu_options, &list_ctx, depth);
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
                    let src = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "src")
                        .map(|attr| attr.value.to_string())
                        .or_else(|| {
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
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name, .. } = &child.data {
                            if name.local.as_ref() != "source" {
                                walk_node(child, &mut fallback, options, ctx, depth + 1);
                            }
                        } else {
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
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name, .. } = &child.data {
                            if name.local.as_ref() != "source" {
                                walk_node(child, &mut fallback, options, ctx, depth + 1);
                            }
                        } else {
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

                "source" => {}

                "picture" => {
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

                "svg" | "math" => {
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

                "form" => {
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

                "input" => {}

                "textarea" => {
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "select" => {
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push('\n');
                    }
                }

                "option" => {
                    let selected = attrs.borrow().iter().any(|attr| attr.name.local.as_ref() == "selected");

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
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "progress" => {
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "meter" => {
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "output" => {
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "datalist" => {
                    let start_len = output.len();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth + 1);
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push('\n');
                    }
                }

                "ruby" => {
                    let ruby_ctx = ctx.clone();

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

                    let has_rtc = tag_sequence.iter().any(|tag| tag == "rtc");

                    let is_interleaved = tag_sequence.windows(2).any(|w| w[0] == "rb" && w[1] == "rt");

                    if is_interleaved && !has_rtc {
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
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, ctx, depth + 1);
                    }
                    output.push_str(text.trim());
                }

                "rt" => {
                    let mut text = String::new();
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut text, options, ctx, depth + 1);
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
                    for child in handle.children.borrow().iter() {
                        walk_node(child, &mut content, options, ctx, depth + 1);
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                    }
                }

                "rtc" => {
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }
                }

                "div" => {
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
                        add_list_continuation_indent(output, ctx.list_depth, false);
                    } else if needs_leading_sep {
                        trim_trailing_whitespace(output);
                        output.push_str("\n\n");
                    }

                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }

                    let has_content = output.len() > content_start_pos;

                    if has_content {
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

                "span" => {
                    let is_hocr_word = attrs
                        .borrow()
                        .iter()
                        .any(|attr| attr.name.local.as_ref() == "class" && attr.value.as_ref().contains("ocrx_word"));

                    if is_hocr_word
                        && !output.is_empty()
                        && !output.ends_with(' ')
                        && !output.ends_with('\t')
                        && !output.ends_with('\n')
                    {
                        output.push(' ');
                    }

                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }
                }

                _ => {
                    for child in handle.children.borrow().iter() {
                        walk_node(child, output, options, ctx, depth);
                    }
                }
            }
        }

        NodeData::Comment { .. } => {}

        NodeData::Doctype { .. } => {}

        NodeData::ProcessingInstruction { .. } => {}
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

    let cell_ctx = Context {
        in_table_cell: true,
        ..ctx.clone()
    };

    if let NodeData::Element { .. } = &handle.data {
        for child in handle.children.borrow().iter() {
            walk_node(child, &mut text, options, &cell_ctx, 0);
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
            convert_table_cell(cell_handle, &mut row_text, options, ctx, "");

            let colspan = get_colspan(cell_handle);
            let rowspan = get_rowspan(cell_handle);

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

        for child in handle.children.borrow().iter() {
            if let NodeData::Element { name, .. } = &child.data {
                let tag_name = name.local.as_ref();

                match tag_name {
                    "caption" => {
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
                        convert_table_row(child, output, options, ctx, row_index, &mut rowspan_tracker);
                        row_index += 1;
                    }

                    "colgroup" | "col" => {}

                    _ => {}
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
        assert_eq!(chomp("  text  "), (" ", " ", "text"));
        assert_eq!(chomp("text"), ("", "", "text"));
        assert_eq!(chomp("  text"), (" ", "", "text"));
        assert_eq!(chomp("text  "), ("", " ", "text"));
        assert_eq!(chomp("   "), (" ", " ", ""));
        assert_eq!(chomp(""), ("", "", ""));
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
    fn test_add_list_continuation_indent_blank_line() {
        let mut output = String::from("* First para");
        add_list_continuation_indent(&mut output, 1, true);
        assert_eq!(output, "* First para\n\n    ");

        let mut output = String::from("* First para\n");
        add_list_continuation_indent(&mut output, 1, true);
        assert_eq!(output, "* First para\n\n    ");

        let mut output = String::from("* First para\n\n");
        add_list_continuation_indent(&mut output, 1, true);
        assert_eq!(output, "* First para\n\n    ");

        let mut output = String::from("* First para");
        add_list_continuation_indent(&mut output, 2, true);
        assert_eq!(output, "* First para\n\n            ");
    }

    #[test]
    fn test_add_list_continuation_indent_single_line() {
        let mut output = String::from("* First div");
        add_list_continuation_indent(&mut output, 1, false);
        assert_eq!(output, "* First div\n    ");

        let mut output = String::from("* First div\n");
        add_list_continuation_indent(&mut output, 1, false);
        assert_eq!(output, "* First div\n    ");

        let mut output = String::from("* First div\n");
        add_list_continuation_indent(&mut output, 1, false);
        assert_eq!(output, "* First div\n    ");
    }

    #[test]
    fn test_trim_trailing_whitespace_in_continuation() {
        let mut output = String::from("* First   ");
        add_list_continuation_indent(&mut output, 1, true);
        assert_eq!(output, "* First\n\n    ");

        let mut output = String::from("* First\t\t");
        add_list_continuation_indent(&mut output, 1, false);
        assert_eq!(output, "* First\n    ");
    }
}
