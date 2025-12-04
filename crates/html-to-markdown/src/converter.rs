//! HTML to Markdown conversion using the astral-tl parser.
//!
//! This module provides the core conversion logic for transforming HTML documents into Markdown.
//! It uses the astral-tl parser for high-performance HTML parsing and supports 60+ HTML tags.
//!

#![allow(clippy::collapsible_match)]
//! # Architecture
//!
//! The conversion process follows these steps:
//! 1. Parse HTML into a DOM tree using the astral-tl parser
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
use std::collections::{BTreeMap, HashMap};
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
        if text.ends_with("  \n") { "  \n" } else { "\\\n" }
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

/// Remove trailing spaces/tabs from every line while preserving newlines.
fn trim_line_end_whitespace(output: &mut String) {
    if output.is_empty() {
        return;
    }

    let mut cleaned = String::with_capacity(output.len());
    for (idx, line) in output.split('\n').enumerate() {
        if idx > 0 {
            cleaned.push('\n');
        }

        let has_soft_break = line.ends_with("  ");
        let trimmed = line.trim_end_matches([' ', '\t']);

        if has_soft_break {
            cleaned.push_str(trimmed);
            cleaned.push_str("  ");
        } else {
            cleaned.push_str(trimmed);
        }
    }

    cleaned.push('\n');
    *output = cleaned;
}

/// Remove common leading whitespace from all lines in a code block.
///
/// This is used for `<pre>` blocks to normalize indentation by removing
/// the minimum common whitespace from all non-empty lines.
fn dedent_code_block(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Find minimum indentation among non-empty lines
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    // Remove that amount of leading whitespace from each line
    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                ""
            } else {
                &line[min_indent.min(line.len())..]
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    if depth > 0 { 2 * depth - 1 } else { 0 }
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
                            if tag_name_eq(child_tag.name().as_utf8_str(), "li") {
                                let li_children = child_tag.children();
                                {
                                    for li_child_handle in li_children.top().iter() {
                                        if let Some(li_child_node) = li_child_handle.get(parser) {
                                            if let tl::Node::Tag(li_child_tag) = li_child_node {
                                                if tag_name_eq(li_child_tag.name().as_utf8_str(), "p") {
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

/// Calculate the indentation string for list continuations based on depth and options.
fn continuation_indent_string(list_depth: usize, options: &ConversionOptions) -> Option<String> {
    let indent_level = calculate_list_continuation_indent(list_depth);
    if indent_level == 0 {
        return None;
    }

    let indent = match options.list_indent_type {
        ListIndentType::Tabs => "\t".repeat(indent_level),
        ListIndentType::Spaces => " ".repeat(options.list_indent_width * indent_level),
    };
    Some(indent)
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
    dom_ctx: &DomContext,
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

                    walk_node(child_handle, parser, output, options, &list_ctx, depth, dom_ctx);

                    if is_ordered {
                        if let Some(child_node) = child_handle.get(parser) {
                            if let tl::Node::Tag(child_tag) = child_node {
                                if tag_name_eq(child_tag.name().as_utf8_str(), "li") {
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
    /// Depth of inline formatting elements (strong/emphasis/span/etc).
    inline_depth: usize,
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
    /// Are we inside a `<strong>` / `<b>` element?
    in_strong: bool,
    #[cfg(feature = "inline-images")]
    /// Shared collector for inline images when enabled.
    inline_collector: Option<InlineCollectorHandle>,
}

struct DomContext {
    parent_map: HashMap<u32, Option<u32>>,
    children_map: HashMap<u32, Vec<tl::NodeHandle>>,
    root_children: Vec<tl::NodeHandle>,
    node_map: HashMap<u32, tl::NodeHandle>,
}

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

fn append_markdown_link(
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
            let escaped_href = href.replace("(", "\\(").replace(")", "\\)");
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

fn find_single_heading_child(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> Option<(usize, tl::NodeHandle)> {
    let node = node_handle.get(parser)?;

    let tl::Node::Tag(tag) = node else {
        return None;
    };

    let children = tag.children();
    let mut heading_data: Option<(usize, tl::NodeHandle)> = None;

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
                let name = normalized_tag_name(child_tag.name().as_utf8_str());
                if let Some(level) = heading_level_from_name(name.as_ref()) {
                    if heading_data.is_some() {
                        return None;
                    }
                    heading_data = Some((level, *child_handle));
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }

    heading_data
}

fn push_heading(output: &mut String, ctx: &Context, options: &ConversionOptions, level: usize, text: &str) {
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
            trim_trailing_whitespace(output);
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

fn normalize_heading_text<'a>(text: &'a str) -> Cow<'a, str> {
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
            ' ' | '\t' if pending_space => continue,
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

fn build_dom_context(dom: &tl::VDom, parser: &tl::Parser) -> DomContext {
    let mut ctx = DomContext {
        parent_map: HashMap::new(),
        children_map: HashMap::new(),
        root_children: dom.children().to_vec(),
        node_map: HashMap::new(),
    };

    for child_handle in dom.children().iter() {
        record_node_hierarchy(child_handle, None, parser, &mut ctx);
    }

    ctx
}

/// Detect block elements that were incorrectly nested under inline ancestors.
fn has_inline_block_misnest(dom_ctx: &DomContext, parser: &tl::Parser) -> bool {
    for handle in dom_ctx.node_map.values() {
        if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
            if is_block_level_element(tag_name.as_ref()) {
                let mut current = dom_ctx.parent_map.get(&handle.get_inner()).and_then(|p| *p);
                while let Some(parent_id) = current {
                    if let Some(parent_handle) = dom_ctx.node_map.get(&parent_id) {
                        if let Some(tl::Node::Tag(parent_tag)) = parent_handle.get(parser) {
                            let parent_name = normalized_tag_name(parent_tag.name().as_utf8_str());
                            if is_inline_element(parent_name.as_ref()) {
                                return true;
                            }
                        }
                    }
                    current = dom_ctx.parent_map.get(&parent_id).and_then(|p| *p);
                }
            }
        }
    }

    false
}

/// Round-trip HTML through html5ever to repair malformed trees.
fn repair_with_html5ever(input: &str) -> Option<String> {
    use html5ever::serialize::{SerializeOpts, serialize};
    use html5ever::tendril::TendrilSink;
    use markup5ever_rcdom::{RcDom, SerializableHandle};

    let dom = html5ever::parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut input.as_bytes())
        .ok()?;

    let mut buf = Vec::with_capacity(input.len());
    let handle = SerializableHandle::from(dom.document.clone());
    serialize(&mut buf, &handle, SerializeOpts::default()).ok()?;
    String::from_utf8(buf).ok()
}

fn record_node_hierarchy(node_handle: &tl::NodeHandle, parent: Option<u32>, parser: &tl::Parser, ctx: &mut DomContext) {
    let id = node_handle.get_inner();
    ctx.parent_map.insert(id, parent);
    ctx.node_map.insert(id, *node_handle);

    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let children: Vec<_> = tag.children().top().iter().copied().collect();
            ctx.children_map.insert(id, children.clone());
            for child in children {
                record_node_hierarchy(&child, Some(id), parser, ctx);
            }
        }
    }
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
                    let tag_name = normalized_tag_name(tag.name().as_utf8_str());

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
                if tag_name_eq(tag.name().as_utf8_str(), "head") {
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
                            let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());

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
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
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

/// Collect inline text for link labels, skipping block-level descendants.
fn collect_link_label_text(children: &[tl::NodeHandle], parser: &tl::Parser) -> (String, Vec<tl::NodeHandle>, bool) {
    let mut text = String::new();
    let mut saw_block = false;
    let mut block_nodes = Vec::new();
    let mut stack: Vec<_> = children.iter().rev().copied().collect();

    while let Some(handle) = stack.pop() {
        if let Some(node) = handle.get(parser) {
            match node {
                tl::Node::Raw(bytes) => {
                    text.push_str(&text::decode_html_entities(&bytes.as_utf8_str()));
                }
                tl::Node::Tag(tag) => {
                    let tag_name = normalized_tag_name(tag.name().as_utf8_str());
                    if is_block_level_element(tag_name.as_ref()) {
                        saw_block = true;
                        block_nodes.push(handle);
                        continue;
                    }

                    let tag_children = tag.children();
                    {
                        let mut child_nodes: Vec<_> = tag_children.top().iter().copied().collect();
                        child_nodes.reverse();
                        for child in child_nodes {
                            stack.push(child);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    (text, block_nodes, saw_block)
}

fn normalize_link_label(label: &str) -> String {
    let collapsed = label
        .chars()
        .map(|ch| if ch == '\n' || ch == '\r' { ' ' } else { ch })
        .collect::<String>();
    text::normalize_whitespace(&collapsed).trim().to_string()
}

/// Serialize an element to HTML string (for SVG and Math elements).
fn serialize_element(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
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

            let has_children = !tag.children().top().is_empty();
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

    use base64::{Engine as _, engine::general_purpose::STANDARD};

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
    // Normalize problematic HTML constructs before parsing
    let mut preprocessed = preprocess_html(html).into_owned();
    let mut preprocessed_len = preprocessed.len();

    let parser_options = tl::ParserOptions::default();
    let mut dom_guard = unsafe {
        tl::parse_owned(preprocessed.clone(), parser_options)
            .map_err(|_| crate::error::ConversionError::ParseError("Failed to parse HTML".to_string()))?
    };
    let mut dom_ref = dom_guard.get_ref();
    let mut parser = dom_ref.parser();
    let mut dom_ctx = build_dom_context(dom_ref, parser);
    let mut output = String::with_capacity(preprocessed_len);

    if has_inline_block_misnest(&dom_ctx, parser) {
        if let Some(repaired_html) = repair_with_html5ever(&preprocessed) {
            preprocessed = preprocess_html(&repaired_html).into_owned();
            preprocessed_len = preprocessed.len();
            dom_guard = unsafe {
                tl::parse_owned(preprocessed.clone(), parser_options)
                    .map_err(|_| crate::error::ConversionError::ParseError("Failed to parse HTML".to_string()))?
            };
            dom_ref = dom_guard.get_ref();
            parser = dom_ref.parser();
            dom_ctx = build_dom_context(dom_ref, parser);
            output = String::with_capacity(preprocessed_len);
        }
    }

    // Check for hOCR document and extract metadata by checking all top-level children
    let mut is_hocr = false;
    for child_handle in dom_ref.children().iter() {
        if is_hocr_document(child_handle, parser) {
            is_hocr = true;
            break;
        }
    }

    if options.extract_metadata && !options.convert_as_inline && !is_hocr {
        for child_handle in dom_ref.children().iter() {
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

        let (elements, metadata) = extract_hocr_document(dom_ref, options.debug);

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
        inline_depth: 0,
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
        in_strong: false,
        #[cfg(feature = "inline-images")]
        inline_collector: inline_collector.clone(),
    };

    // Walk all top-level children
    for child_handle in dom_ref.children().iter() {
        walk_node(child_handle, parser, &mut output, options, &ctx, 0, &dom_ctx);
    }

    // Trim trailing spaces per line, then trim trailing blank lines but preserve final newline
    trim_line_end_whitespace(&mut output);
    let trimmed = output.trim_end_matches('\n');
    if trimmed.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("{}\n", trimmed))
    }
}

fn preprocess_html(input: &str) -> Cow<'_, str> {
    const SELF_CLOSING: [(&[u8], &str); 3] = [(b"<br/>", "<br>"), (b"<hr/>", "<hr>"), (b"<img/>", "<img>")];
    const TAGS: [&[u8]; 2] = [b"script", b"style"];
    const SVG: &[u8] = b"svg";
    const DOCTYPE: &[u8] = b"doctype";
    const EMPTY_COMMENT: &[u8] = b"<!---->";

    let bytes = input.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return Cow::Borrowed(input);
    }

    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;
    let mut svg_depth = 0usize;

    while idx < len {
        if bytes[idx] == b'<' {
            if bytes[idx..].starts_with(EMPTY_COMMENT) {
                let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                out.push_str(&input[last..idx]);
                out.push_str("<!-- -->");
                idx += EMPTY_COMMENT.len();
                last = idx;
                continue;
            }

            let mut replaced = false;
            for (pattern, replacement) in &SELF_CLOSING {
                if bytes[idx..].starts_with(pattern) {
                    let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                    out.push_str(&input[last..idx]);
                    out.push_str(replacement);
                    idx += pattern.len();
                    last = idx;
                    replaced = true;
                    break;
                }
            }
            if replaced {
                continue;
            }

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

                if idx + 2 < len && bytes[idx + 1] == b'!' {
                    let mut cursor = idx + 2;
                    while cursor < len && bytes[cursor].is_ascii_whitespace() {
                        cursor += 1;
                    }

                    if cursor + DOCTYPE.len() <= len
                        && bytes[cursor..cursor + DOCTYPE.len()].eq_ignore_ascii_case(DOCTYPE)
                    {
                        if let Some(end) = find_tag_end(bytes, cursor + DOCTYPE.len()) {
                            let out = output.get_or_insert_with(|| String::with_capacity(input.len()));
                            out.push_str(&input[last..idx]);
                            last = end;
                            idx = end;
                            continue;
                        }
                    }
                }
            }

            let is_valid_tag = if idx + 1 < len {
                match bytes[idx + 1] {
                    b'!' => {
                        idx + 2 < len
                            && (bytes[idx + 2] == b'-'
                                || bytes[idx + 2].is_ascii_alphabetic()
                                || bytes[idx + 2].is_ascii_uppercase())
                    }
                    b'/' => {
                        idx + 2 < len && (bytes[idx + 2].is_ascii_alphabetic() || bytes[idx + 2].is_ascii_uppercase())
                    }
                    b'?' => true,
                    c if c.is_ascii_alphabetic() || c.is_ascii_uppercase() => true,
                    _ => false,
                }
            } else {
                false
            };

            if !is_valid_tag {
                let out = output.get_or_insert_with(|| String::with_capacity(input.len() + 4));
                out.push_str(&input[last..idx]);
                out.push_str("&lt;");
                idx += 1;
                last = idx;
                continue;
            }
        }

        idx += 1;
    }

    if let Some(mut out) = output {
        if last < len {
            out.push_str(&input[last..]);
        }
        Cow::Owned(out)
    } else {
        Cow::Borrowed(input)
    }
}

#[cfg(test)]
fn normalize_self_closing_tags(input: &str) -> Cow<'_, str> {
    const REPLACEMENTS: [(&[u8], &str); 3] = [(b"<br/>", "<br>"), (b"<hr/>", "<hr>"), (b"<img/>", "<img>")];

    if !REPLACEMENTS
        .iter()
        .any(|(pattern, _)| input.as_bytes().windows(pattern.len()).any(|w| w == *pattern))
    {
        return Cow::Borrowed(input);
    }

    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut idx = 0;
    let mut last = 0;

    while idx < bytes.len() {
        let mut matched = false;
        for (pattern, replacement) in &REPLACEMENTS {
            if bytes[idx..].starts_with(*pattern) {
                output.push_str(&input[last..idx]);
                output.push_str(replacement);
                idx += pattern.len();
                last = idx;
                matched = true;
                break;
            }
        }

        if !matched {
            idx += 1;
        }
    }

    if last < input.len() {
        output.push_str(&input[last..]);
    }

    Cow::Owned(output)
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
#[cfg(test)]
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

fn normalized_tag_name<'a>(raw: Cow<'a, str>) -> Cow<'a, str> {
    if raw.as_bytes().iter().any(|b| b.is_ascii_uppercase()) {
        let mut owned = raw.into_owned();
        owned.make_ascii_lowercase();
        Cow::Owned(owned)
    } else {
        raw
    }
}

fn tag_name_eq(name: Cow<'_, str>, needle: &str) -> bool {
    name.eq_ignore_ascii_case(needle)
}

fn should_drop_for_preprocessing(
    node_handle: &tl::NodeHandle,
    tag_name: &str,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
    options: &ConversionOptions,
) -> bool {
    if !options.preprocessing.enabled {
        return false;
    }

    if options.preprocessing.remove_navigation {
        let has_nav_hint = element_has_navigation_hint(tag);

        if tag_name == "nav" {
            return true;
        }

        if tag_name == "header" {
            let inside_semantic_content = has_semantic_content_ancestor(node_handle, parser, dom_ctx);
            if !inside_semantic_content {
                return true;
            }
            if has_nav_hint {
                return true;
            }
        } else if tag_name == "footer" || tag_name == "aside" {
            if has_nav_hint {
                return true;
            }
        } else if has_nav_hint && !matches!(tag_name, "main" | "article" | "html" | "body" | "head") {
            return true;
        }
    }

    if options.preprocessing.remove_forms {
        if tag_name == "form" {
            let preserves_form = options.preserve_tags.iter().any(|t| t == "form");
            if !preserves_form {
                return true;
            }
        } else if matches!(
            tag_name,
            "button" | "select" | "textarea" | "label" | "fieldset" | "legend"
        ) {
            return true;
        }
    }

    false
}

fn has_semantic_content_ancestor(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let mut current_id = node_handle.get_inner();
    while let Some(parent_id) = dom_ctx.parent_map.get(&current_id).copied().flatten() {
        if let Some(parent_handle) = dom_ctx.node_map.get(&parent_id) {
            if let Some(tl::Node::Tag(parent_tag)) = parent_handle.get(parser) {
                let parent_name = normalized_tag_name(parent_tag.name().as_utf8_str());
                if matches!(parent_name.as_ref(), "main" | "article" | "section") {
                    return true;
                }
                if tag_has_main_semantics(parent_tag) {
                    return true;
                }
            }
        }
        current_id = parent_id;
    }
    false
}

fn tag_has_main_semantics(tag: &tl::HTMLTag) -> bool {
    if let Some(role_attr) = tag.attributes().get("role") {
        if let Some(role) = role_attr {
            let lowered = role.as_utf8_str().to_ascii_lowercase();
            if matches!(lowered.as_str(), "main" | "article" | "document" | "region") {
                return true;
            }
        }
    }

    if let Some(class_attr) = tag.attributes().get("class") {
        if let Some(class_bytes) = class_attr {
            let class_value = class_bytes.as_utf8_str().to_ascii_lowercase();
            const MAIN_CLASS_HINTS: &[&str] = &[
                "mw-body",
                "mw-parser-output",
                "content-body",
                "content-container",
                "article-body",
                "article-content",
                "main-content",
                "page-content",
                "entry-content",
                "post-content",
                "document-body",
            ];
            if MAIN_CLASS_HINTS.iter().any(|hint| class_value.contains(hint)) {
                return true;
            }
        }
    }

    false
}

fn element_has_navigation_hint(tag: &tl::HTMLTag) -> bool {
    if attribute_matches_any(tag, "role", &["navigation", "menubar", "tablist", "toolbar"]) {
        return true;
    }

    if attribute_contains_any(
        tag,
        "aria-label",
        &["navigation", "menu", "contents", "table of contents", "toc"],
    ) {
        return true;
    }

    const NAV_KEYWORDS: &[&str] = &[
        "nav",
        "navigation",
        "navbar",
        "breadcrumbs",
        "breadcrumb",
        "toc",
        "sidebar",
        "sidenav",
        "menu",
        "menubar",
        "mainmenu",
        "subnav",
        "tabs",
        "tablist",
        "toolbar",
        "pager",
        "pagination",
        "skipnav",
        "skip-link",
        "skiplinks",
        "site-nav",
        "site-menu",
        "site-header",
        "site-footer",
        "topbar",
        "bottombar",
        "masthead",
        "vector-nav",
        "vector-header",
        "vector-footer",
    ];

    attribute_matches_any(tag, "class", NAV_KEYWORDS) || attribute_matches_any(tag, "id", NAV_KEYWORDS)
}

fn attribute_matches_any(tag: &tl::HTMLTag, attr: &str, keywords: &[&str]) -> bool {
    let Some(attr_value) = tag.attributes().get(attr) else {
        return false;
    };
    let Some(value) = attr_value else {
        return false;
    };
    let raw = value.as_utf8_str();
    raw.split_whitespace()
        .map(|token| {
            token
                .chars()
                .map(|c| match c {
                    '_' | ':' | '.' | '/' => '-',
                    _ => c,
                })
                .collect::<String>()
                .to_ascii_lowercase()
        })
        .filter(|token| !token.is_empty())
        .any(|token| keywords.iter().any(|kw| token == *kw))
}

fn attribute_contains_any(tag: &tl::HTMLTag, attr: &str, keywords: &[&str]) -> bool {
    let Some(attr_value) = tag.attributes().get(attr) else {
        return false;
    };
    let Some(value) = attr_value else {
        return false;
    };
    let lower = value.as_utf8_str().to_ascii_lowercase();
    keywords.iter().any(|kw| lower.contains(*kw))
}

/// Serialize a tag and its children back to HTML.
///
/// This is used for the preserve_tags feature to output original HTML for specific elements.
fn serialize_tag_to_html(handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    let mut html = String::new();
    serialize_node_to_html(handle, parser, &mut html);
    html
}

/// Recursively serialize a node to HTML.
fn serialize_node_to_html(handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String) {
    match handle.get(parser) {
        Some(tl::Node::Tag(tag)) => {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());

            // Opening tag
            output.push('<');
            output.push_str(&tag_name);

            // Attributes
            for (key, value) in tag.attributes().iter() {
                output.push(' ');
                output.push_str(&key);
                if let Some(val) = value {
                    output.push_str("=\"");
                    output.push_str(&val);
                    output.push('"');
                }
            }

            output.push('>');

            // Children
            let children = tag.children();
            for child_handle in children.top().iter() {
                serialize_node_to_html(child_handle, parser, output);
            }

            // Closing tag (skip for self-closing tags)
            if !matches!(
                tag_name.as_ref(),
                "br" | "hr"
                    | "img"
                    | "input"
                    | "meta"
                    | "link"
                    | "area"
                    | "base"
                    | "col"
                    | "embed"
                    | "param"
                    | "source"
                    | "track"
                    | "wbr"
            ) {
                output.push_str("</");
                output.push_str(&tag_name);
                output.push('>');
            }
        }
        Some(tl::Node::Raw(bytes)) => {
            if let Ok(text) = std::str::from_utf8(bytes.as_bytes()) {
                output.push_str(text);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
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

/// Check if an element is block-level (not inline).
fn is_block_level_element(tag_name: &str) -> bool {
    !is_inline_element(tag_name)
        && matches!(
            tag_name,
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "canvas"
                | "dd"
                | "div"
                | "dl"
                | "dt"
                | "fieldset"
                | "figcaption"
                | "figure"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "hr"
                | "li"
                | "main"
                | "nav"
                | "ol"
                | "p"
                | "pre"
                | "section"
                | "table"
                | "tfoot"
                | "ul"
        )
}

fn get_next_sibling_tag(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> Option<String> {
    let id = node_handle.get_inner();
    let parent = dom_ctx.parent_map.get(&id).copied().flatten();

    let siblings = if let Some(parent_id) = parent {
        dom_ctx.children_map.get(&parent_id)?
    } else {
        &dom_ctx.root_children
    };

    let position = siblings.iter().position(|handle| handle.get_inner() == id)?;

    for sibling in siblings.iter().skip(position + 1) {
        if let Some(node) = sibling.get(parser) {
            match node {
                tl::Node::Tag(tag) => return Some(normalized_tag_name(tag.name().as_utf8_str()).into_owned()),
                tl::Node::Raw(raw) => {
                    if !raw.as_utf8_str().trim().is_empty() {
                        return None;
                    }
                }
                _ => {}
            }
        }
    }

    None
}

fn get_previous_sibling_tag(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> Option<String> {
    let id = node_handle.get_inner();
    let parent = dom_ctx.parent_map.get(&id).copied().flatten();

    let siblings = if let Some(parent_id) = parent {
        dom_ctx.children_map.get(&parent_id)?
    } else {
        &dom_ctx.root_children
    };

    let position = siblings.iter().position(|handle| handle.get_inner() == id)?;

    for sibling in siblings.iter().take(position).rev() {
        if let Some(node) = sibling.get(parser) {
            match node {
                tl::Node::Tag(tag) => return Some(normalized_tag_name(tag.name().as_utf8_str()).into_owned()),
                tl::Node::Raw(raw) => {
                    if !raw.as_utf8_str().trim().is_empty() {
                        return None;
                    }
                }
                _ => {}
            }
        }
    }

    None
}

fn previous_sibling_is_inline_tag(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let id = node_handle.get_inner();
    let parent = dom_ctx.parent_map.get(&id).copied().flatten();

    let siblings = if let Some(parent_id) = parent {
        if let Some(children) = dom_ctx.children_map.get(&parent_id) {
            children
        } else {
            return false;
        }
    } else {
        &dom_ctx.root_children
    };

    let Some(position) = siblings.iter().position(|handle| handle.get_inner() == id) else {
        return false;
    };

    for sibling in siblings.iter().take(position).rev() {
        if let Some(node) = sibling.get(parser) {
            match node {
                tl::Node::Tag(tag) => {
                    let name = normalized_tag_name(tag.name().as_utf8_str());
                    return is_inline_element(name.as_ref()) || matches!(name.as_ref(), "script" | "style");
                }
                tl::Node::Raw(raw) => {
                    if raw.as_utf8_str().trim().is_empty() {
                        continue;
                    }
                    return false;
                }
                _ => continue,
            }
        }
    }

    false
}

fn next_sibling_is_whitespace_text(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let id = node_handle.get_inner();
    let parent = dom_ctx.parent_map.get(&id).copied().flatten();

    let siblings = if let Some(parent_id) = parent {
        if let Some(children) = dom_ctx.children_map.get(&parent_id) {
            children
        } else {
            return false;
        }
    } else {
        &dom_ctx.root_children
    };

    let Some(position) = siblings.iter().position(|handle| handle.get_inner() == id) else {
        return false;
    };

    for sibling in siblings.iter().skip(position + 1) {
        if let Some(node) = sibling.get(parser) {
            match node {
                tl::Node::Raw(raw) => return raw.as_utf8_str().trim().is_empty(),
                tl::Node::Tag(_) => return false,
                _ => continue,
            }
        }
    }

    false
}

fn next_sibling_is_inline_tag(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let id = node_handle.get_inner();
    let parent = dom_ctx.parent_map.get(&id).copied().flatten();

    let siblings = if let Some(parent_id) = parent {
        if let Some(children) = dom_ctx.children_map.get(&parent_id) {
            children
        } else {
            return false;
        }
    } else {
        &dom_ctx.root_children
    };

    let Some(position) = siblings.iter().position(|handle| handle.get_inner() == id) else {
        return false;
    };

    for sibling in siblings.iter().skip(position + 1) {
        if let Some(node) = sibling.get(parser) {
            match node {
                tl::Node::Tag(tag) => {
                    let name = normalized_tag_name(tag.name().as_utf8_str());
                    return is_inline_element(name.as_ref()) || matches!(name.as_ref(), "script" | "style");
                }
                tl::Node::Raw(raw) => {
                    if raw.as_utf8_str().trim().is_empty() {
                        continue;
                    }
                    return false;
                }
                _ => continue,
            }
        }
    }

    false
}

fn append_inline_suffix(
    output: &mut String,
    suffix: &str,
    has_core_content: bool,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) {
    if suffix.is_empty() {
        return;
    }

    if suffix == " " && has_core_content && next_sibling_is_whitespace_text(node_handle, parser, dom_ctx) {
        return;
    }

    output.push_str(suffix);
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
    dom_ctx: &DomContext,
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

                if text.contains('\n') {
                    if output.is_empty() {
                        return;
                    }
                    if !output.ends_with("\n\n") {
                        if let Some(next_tag) = get_next_sibling_tag(node_handle, parser, dom_ctx) {
                            if is_inline_element(&next_tag) {
                                return;
                            }
                        }
                    }
                    return;
                }

                if previous_sibling_is_inline_tag(node_handle, parser, dom_ctx)
                    && next_sibling_is_inline_tag(node_handle, parser, dom_ctx)
                {
                    if text.chars().count() > 1 {
                        if !output.ends_with(' ') {
                            output.push(' ');
                        }
                    } else {
                        output.push_str(&text);
                    }
                } else {
                    output.push_str(&text);
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
                    || output.ends_with("] ")
                    || (output.ends_with('\n') && prefix == " ")
                    || (output.ends_with(' ')
                        && prefix == " "
                        && !previous_sibling_is_inline_tag(node_handle, parser, dom_ctx));

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
                        if text.contains("\n\n") || text.contains("\r\n\r\n") {
                            final_text.push('\n');
                        } else if let Some(next_tag) = get_next_sibling_tag(node_handle, parser, dom_ctx) {
                            if options.debug {
                                eprintln!("[DEBUG] Next sibling tag after newline: {}", next_tag);
                            }
                            if matches!(next_tag.as_str(), "span") {
                                // Collapse formatting newlines between inline siblings like span
                            } else if ctx.inline_depth > 0 || ctx.convert_as_inline || ctx.in_paragraph {
                                final_text.push(' ');
                            } else {
                                final_text.push('\n');
                            }
                        } else if ctx.inline_depth > 0 || ctx.convert_as_inline || ctx.in_paragraph {
                            final_text.push(' ');
                        } else {
                            final_text.push('\n');
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
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());

            if should_drop_for_preprocessing(node_handle, tag_name.as_ref(), tag, parser, dom_ctx, options) {
                trim_trailing_whitespace(output);
                if options.debug {
                    eprintln!("[DEBUG] Dropping <{}> subtree due to preprocessing settings", tag_name);
                }
                return;
            }

            if options.strip_tags.iter().any(|t| t.as_str() == tag_name) {
                let children = tag.children();
                {
                    for child_handle in children.top().iter() {
                        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                    }
                }
                return;
            }

            // Preserve tags: output original HTML
            if options.preserve_tags.iter().any(|t| t.as_str() == tag_name) {
                let html = serialize_tag_to_html(node_handle, parser);
                output.push_str(&html);
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
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        let normalized = normalize_heading_text(trimmed);
                        push_heading(output, ctx, options, level, normalized.as_ref());
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
                            walk_node(child_handle, parser, output, options, &p_ctx, depth + 1, dom_ctx);
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
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(64);
                        let children = tag.children();
                        {
                            let strong_ctx = Context {
                                inline_depth: ctx.inline_depth + 1,
                                in_strong: true,
                                ..ctx.clone()
                            };
                            for child_handle in children.top().iter() {
                                walk_node(
                                    child_handle,
                                    parser,
                                    &mut content,
                                    options,
                                    &strong_ctx,
                                    depth + 1,
                                    dom_ctx,
                                );
                            }
                        }
                        let (prefix, suffix, trimmed) = chomp_inline(&content);
                        if !content.trim().is_empty() {
                            output.push_str(prefix);
                            if ctx.in_strong {
                                output.push_str(trimmed);
                            } else {
                                output.push(options.strong_em_symbol);
                                output.push(options.strong_em_symbol);
                                output.push_str(trimmed);
                                output.push(options.strong_em_symbol);
                                output.push(options.strong_em_symbol);
                            }
                            append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                        } else if !content.is_empty() {
                            output.push_str(prefix);
                            append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                        }
                    }
                }

                "em" | "i" => {
                    if ctx.in_code {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(64);
                        let children = tag.children();
                        {
                            let em_ctx = Context {
                                inline_depth: ctx.inline_depth + 1,
                                ..ctx.clone()
                            };
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, &mut content, options, &em_ctx, depth + 1, dom_ctx);
                            }
                        }
                        let (prefix, suffix, trimmed) = chomp_inline(&content);
                        if !content.trim().is_empty() {
                            output.push_str(prefix);
                            output.push(options.strong_em_symbol);
                            output.push_str(trimmed);
                            output.push(options.strong_em_symbol);
                            append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                        } else if !content.is_empty() {
                            output.push_str(prefix);
                            append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                        } else if let Some(class_value) = tag
                            .attributes()
                            .get("class")
                            .and_then(|v| v.as_ref().map(|val| val.as_utf8_str().to_string()))
                        {
                            if class_value.contains("caret") && !output.ends_with(' ') {
                                output.push_str(" > ");
                            }
                        }
                    }
                }

                "a" => {
                    const MAX_LINK_LABEL_LEN: usize = 512;

                    let href_attr = tag
                        .attributes()
                        .get("href")
                        .flatten()
                        .map(|v| text::decode_html_entities(&v.as_utf8_str()));
                    let title = tag
                        .attributes()
                        .get("title")
                        .flatten()
                        .map(|v| v.as_utf8_str().to_string());

                    if let Some(href) = href_attr {
                        let raw_text = text::normalize_whitespace(&get_text_content(node_handle, parser))
                            .trim()
                            .to_string();

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

                        if let Some((heading_level, heading_handle)) = find_single_heading_child(node_handle, parser) {
                            if let Some(heading_node) = heading_handle.get(parser) {
                                if let tl::Node::Tag(heading_tag) = heading_node {
                                    let heading_name =
                                        normalized_tag_name(heading_tag.name().as_utf8_str()).into_owned();
                                    let mut heading_text = String::new();
                                    let heading_ctx = Context {
                                        in_heading: true,
                                        convert_as_inline: true,
                                        heading_tag: Some(heading_name),
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

                        let children: Vec<_> = tag.children().top().iter().copied().collect();
                        let (inline_label, _block_nodes, saw_block) = collect_link_label_text(&children, parser);
                        let mut label = if saw_block {
                            let mut content = String::new();
                            let link_ctx = Context {
                                inline_depth: ctx.inline_depth + 1,
                                convert_as_inline: true,
                                ..ctx.clone()
                            };
                            for child_handle in children.iter() {
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
                                    && !content.chars().last().map(|c| c.is_whitespace()).unwrap_or(true)
                                    && !child_buf.chars().next().map(|c| c.is_whitespace()).unwrap_or(true)
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
                                ..ctx.clone()
                            };
                            for child_handle in children.iter() {
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

                        if label.is_empty() && saw_block {
                            let fallback = text::normalize_whitespace(&get_text_content(node_handle, parser));
                            label = normalize_link_label(&fallback);
                        }

                        if label.is_empty() && !raw_text.is_empty() {
                            label = normalize_link_label(&raw_text);
                        }

                        if label.is_empty() && !href.is_empty() && !children.is_empty() {
                            label = href.clone();
                        }

                        if label.len() > MAX_LINK_LABEL_LEN {
                            label.truncate(MAX_LINK_LABEL_LEN);
                            label.push('…');
                        }

                        let escaped_label = escape_link_label(&label);
                        append_markdown_link(
                            output,
                            &escaped_label,
                            href.as_str(),
                            title.as_deref(),
                            label.as_str(),
                            options,
                        );
                    } else {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                                    .is_none_or(|tag| !options.keep_inline_images_in.iter().any(|t| t == tag))));

                    if should_use_alt_text {
                        output.push_str(&alt);
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
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                                    }
                                }
                                output.push_str("==");
                            }
                            HighlightStyle::Html => {
                                output.push_str("<mark>");
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                                    }
                                }
                                output.push_str("</mark>");
                            }
                            HighlightStyle::Bold => {
                                let symbol = options.strong_em_symbol.to_string().repeat(2);
                                output.push_str(&symbol);
                                let bold_ctx = Context {
                                    in_strong: true,
                                    ..ctx.clone()
                                };
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, &bold_ctx, depth + 1, dom_ctx);
                                    }
                                }
                                output.push_str(&symbol);
                            }
                            HighlightStyle::None => {
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(32);
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                        let (prefix, suffix, trimmed) = chomp_inline(&content);
                        if !content.trim().is_empty() {
                            output.push_str(prefix);
                            output.push_str("~~");
                            output.push_str(trimmed);
                            output.push_str("~~");
                            append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                        } else if !content.is_empty() {
                            output.push_str(prefix);
                            append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                        }
                    }
                }

                "ins" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let (prefix, suffix, trimmed) = chomp_inline(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push_str("==");
                        output.push_str(trimmed);
                        output.push_str("==");
                        append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                    }
                }

                "u" | "small" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &code_ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }
                    let normalized = text::normalize_whitespace(&content);
                    let (prefix, suffix, trimmed) = chomp_inline(&normalized);
                    if !content.trim().is_empty() {
                        output.push_str(prefix);
                        output.push('`');
                        output.push_str(trimmed);
                        output.push('`');
                        append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                    } else if !content.is_empty() {
                        output.push_str(prefix);
                        append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                    }
                }

                "var" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let (prefix, suffix, trimmed) = chomp_inline(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push(options.strong_em_symbol);
                        output.push_str(trimmed);
                        output.push(options.strong_em_symbol);
                        append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                    }
                }

                "dfn" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let (prefix, suffix, trimmed) = chomp_inline(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push(options.strong_em_symbol);
                        output.push_str(trimmed);
                        output.push(options.strong_em_symbol);
                        append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                    }
                }

                "abbr" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                                walk_node(
                                    child_handle,
                                    parser,
                                    &mut content,
                                    options,
                                    &code_ctx,
                                    depth + 1,
                                    dom_ctx,
                                );
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
                                walk_node(child_handle, parser, output, options, &code_ctx, depth + 1, dom_ctx);
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
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &code_ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }

                    if !content.is_empty() {
                        // Trim leading/trailing newlines and dedent
                        let trimmed = content.trim_matches('\n');
                        let dedented = dedent_code_block(trimmed);
                        
                        match options.code_block_style {
                            crate::options::CodeBlockStyle::Indented => {
                                if !ctx.convert_as_inline && !output.is_empty() && !output.ends_with("\n\n") {
                                    if output.ends_with('\n') {
                                        output.push('\n');
                                    } else {
                                        output.push_str("\n\n");
                                    }
                                }

                                let indented = dedented
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
                                output.push_str(&dedented);
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
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &blockquote_ctx,
                                depth + 1,
                                dom_ctx,
                            );
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
                        }

                        while output.ends_with('\n') {
                            output.truncate(output.len() - 1);
                        }
                    }
                }

                "br" => {
                    if ctx.in_heading {
                        trim_trailing_whitespace(output);
                        output.push_str("  ");
                    } else {
                        use crate::options::NewlineStyle;
                        if output.is_empty() || output.ends_with('\n') {
                            output.push('\n');
                        } else {
                            match options.newline_style {
                                NewlineStyle::Spaces => output.push_str("  \n"),
                                NewlineStyle::Backslash => output.push_str("\\\n"),
                            }
                        }
                    }
                }

                "hr" => {
                    // CommonMark: ensure a blank line before the hr so it is not interpreted as a setext heading underline
                    if !output.is_empty() {
                        let prev_tag = get_previous_sibling_tag(node_handle, parser, dom_ctx);
                        let last_line_is_blockquote = output
                            .rsplit('\n')
                            .find(|line| !line.trim().is_empty())
                            .map(|line| line.trim_start().starts_with('>'))
                            .unwrap_or(false);
                        let needs_blank_line = !ctx.in_paragraph
                            && !matches!(prev_tag.as_deref(), Some("blockquote"))
                            && !last_line_is_blockquote;

                        if options.debug {
                            eprintln!(
                                "[DEBUG] <hr> prev_tag={:?} needs_blank_line={} in_paragraph={}",
                                prev_tag, needs_blank_line, ctx.in_paragraph
                            );
                        }

                        if ctx.in_paragraph || !needs_blank_line {
                            if !output.ends_with('\n') {
                                output.push('\n');
                            }
                        } else {
                            trim_trailing_whitespace(output);
                            if output.ends_with('\n') {
                                if !output.ends_with("\n\n") {
                                    output.push('\n');
                                }
                            } else {
                                output.push_str("\n\n");
                            }
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
                        dom_ctx,
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
                        dom_ctx,
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
                                let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                if matches!(
                                    tag_name.as_ref(),
                                    "p" | "div" | "blockquote" | "pre" | "table" | "hr" | "dl"
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
                            if tag_name_eq(node_tag.name().as_utf8_str(), "input") {
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

                        #[allow(clippy::too_many_arguments)]
                        fn render_li_content<'a>(
                            node_handle: &tl::NodeHandle,
                            parser: &'a tl::Parser<'a>,
                            output: &mut String,
                            options: &ConversionOptions,
                            ctx: &Context,
                            depth: usize,
                            checkbox: &Option<tl::NodeHandle>,
                            dom_ctx: &DomContext,
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
                                                dom_ctx,
                                            );
                                        }
                                    }
                                }
                            } else {
                                walk_node(node_handle, parser, output, options, ctx, depth, dom_ctx);
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
                                    dom_ctx,
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
                                walk_node(child_handle, parser, output, options, &li_ctx, depth + 1, dom_ctx);
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
                    let mut table_output = String::new();
                    convert_table(node_handle, parser, &mut table_output, options, ctx, dom_ctx);

                    if ctx.in_list_item {
                        let has_caption = table_output.starts_with('*');

                        if !has_caption {
                            trim_trailing_whitespace(output);
                            if !output.is_empty() && !output.ends_with('\n') {
                                output.push('\n');
                            }
                        }

                        let indented = indent_table_for_list(&table_output, ctx.list_depth, options);
                        output.push_str(&indented);
                    } else {
                        if !output.ends_with("\n\n") {
                            if output.is_empty() || !output.ends_with('\n') {
                                output.push_str("\n\n");
                            } else {
                                output.push('\n');
                            }
                        }
                        output.push_str(&table_output);
                    }

                    if !output.ends_with('\n') {
                        output.push('\n');
                    }
                }

                "thead" | "tbody" | "tfoot" | "tr" | "th" | "td" => {}

                "caption" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
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
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }

                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
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
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
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
                            walk_node(child_handle, parser, &mut figure_content, options, ctx, depth, dom_ctx);
                        }
                    }

                    figure_content = figure_content.replace("\n![", "![");
                    figure_content = figure_content.replace(" ![", "![");

                    let trimmed = figure_content.trim_matches(|c| c == '\n' || c == ' ' || c == '\t');
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                        if !output.ends_with('\n') {
                            output.push('\n');
                        }
                        if !output.ends_with("\n\n") {
                            output.push('\n');
                        }
                    }
                }

                "figcaption" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let text = text.trim();
                    if !text.is_empty() {
                        if !output.is_empty() {
                            if output.ends_with("```\n") {
                                output.push('\n');
                            } else {
                                trim_trailing_whitespace(output);
                                if output.ends_with('\n') && !output.ends_with("\n\n") {
                                    output.push('\n');
                                } else if !output.ends_with('\n') {
                                    output.push_str("\n\n");
                                }
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
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                        }
                    }
                }

                "cite" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
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
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
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
                                let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                (tag_name == "dt", tag_name == "dd")
                            } else {
                                (false, false)
                            };

                            let child_ctx = Context {
                                last_was_dt: in_dt_group && is_dd,
                                ..ctx.clone()
                            };
                            walk_node(child_handle, parser, &mut content, options, &child_ctx, depth, dom_ctx);

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
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
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
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }

                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
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
                    let mut summary_ctx = ctx.clone();
                    if !ctx.convert_as_inline {
                        summary_ctx.in_strong = true;
                    }
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &summary_ctx,
                                depth + 1,
                                dom_ctx,
                            );
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
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }

                    let content_start = output.len();

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
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
                            walk_node(child_handle, parser, output, &menu_options, &list_ctx, depth, dom_ctx);
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
                                        if tag_name_eq(child_tag.name().as_utf8_str(), "source") {
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
                                tag_name_eq(child_tag.name().as_utf8_str(), "source")
                            } else {
                                false
                            };

                            if !is_source {
                                walk_node(child_handle, parser, &mut fallback, options, ctx, depth + 1, dom_ctx);
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
                                        if tag_name_eq(child_tag.name().as_utf8_str(), "source") {
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
                                tag_name_eq(child_tag.name().as_utf8_str(), "source")
                            } else {
                                false
                            };

                            if !is_source {
                                walk_node(child_handle, parser, &mut fallback, options, ctx, depth + 1, dom_ctx);
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
                                if tag_name_eq(child_tag.name().as_utf8_str(), "img") {
                                    walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
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
                                if tag_name_eq(child_tag.name().as_utf8_str(), "title") {
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
                        use base64::{Engine as _, engine::general_purpose::STANDARD};

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
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }

                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
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
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }
                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
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
                    let mut legend_ctx = ctx.clone();
                    if !ctx.convert_as_inline {
                        legend_ctx.in_strong = true;
                    }
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &legend_ctx,
                                depth + 1,
                                dom_ctx,
                            );
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
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                }

                "button" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                                let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
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
                                            let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                            if tag_name == "rt" {
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
                                                    dom_ctx,
                                                );
                                            } else if tag_name != "rp" {
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
                                            let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                            if tag_name == "rt" {
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
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    output.push_str(text.trim());
                }

                "rt" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                        }
                    }
                }

                "div" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
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

                "head" => {
                    // Malformed pages sometimes place <body> or main content inside <head>.
                    // Only walk children if we detect non-head content to avoid rendering metadata.
                    let children = tag.children();
                    let has_body_like = children.top().iter().any(|child_handle| {
                        if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                            let child_name = normalized_tag_name(child_tag.name().as_utf8_str());
                            matches!(
                                child_name.as_ref(),
                                "body" | "main" | "article" | "section" | "div" | "p"
                            )
                        } else {
                            false
                        }
                    });

                    if has_body_like {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                }

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

                    if !ctx.in_code
                        && options.whitespace_mode == crate::options::WhitespaceMode::Normalized
                        && output.ends_with('\n')
                        && !output.ends_with("\n\n")
                    {
                        output.pop();
                    }

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                        }
                    }
                }

                _ => {
                    let len_before = output.len();
                    let had_trailing_space = output.ends_with(' ');

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                        }
                    }

                    let len_after = output.len();
                    if len_after > len_before {
                        // Child processing can pop a trailing byte before appending new content,
                        // so len_before might land inside a multi-byte char; clamp to a safe boundary.
                        let start_idx = if output.is_char_boundary(len_before) {
                            len_before
                        } else {
                            let capped = len_before.min(output.len());
                            output
                                .char_indices()
                                .map(|(idx, _)| idx)
                                .take_while(|idx| *idx <= capped)
                                .last()
                                .unwrap_or(capped)
                        };

                        let added_content = output[start_idx..].to_string();
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
                            output.truncate(start_idx);
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
    dom_ctx: &DomContext,
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
                walk_node(child_handle, parser, &mut text, options, &cell_ctx, 0, dom_ctx);
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
#[allow(clippy::too_many_arguments)]
fn convert_table_row(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    row_index: usize,
    rowspan_tracker: &mut std::collections::HashMap<usize, (String, usize)>,
    dom_ctx: &DomContext,
) {
    let mut row_text = String::with_capacity(256);
    let mut cells = Vec::new();

    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let cell_name = normalized_tag_name(child_tag.name().as_utf8_str());
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
        if let Some((_content, remaining_rows)) = rowspan_tracker.get_mut(&col_index) {
            if *remaining_rows > 0 {
                row_text.push(' ');
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
            let cell_start = row_text.len();
            convert_table_cell(cell_handle, parser, &mut row_text, options, ctx, "", dom_ctx);

            let (colspan, rowspan) = get_colspan_rowspan(cell_handle, parser);

            if rowspan > 1 {
                // Extract the cell content that was just added (without separators)
                let cell_text = &row_text[cell_start..];
                // Strip leading space and trailing " |"
                let cell_content = cell_text
                    .trim_start_matches(' ')
                    .trim_end_matches(" |")
                    .trim()
                    .to_string();
                rowspan_tracker.insert(col_index, (cell_content, rowspan - 1));
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

fn table_has_header(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> bool {
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
            if tag_name.as_ref() == "th" {
                return true;
            }
            let children = tag.children();
            for child in children.top().iter() {
                if table_has_header(child, parser) {
                    return true;
                }
            }
        }
    }
    false
}

fn table_has_caption(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> bool {
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
            if tag_name.as_ref() == "caption" {
                return true;
            }
            let children = tag.children();
            for child in children.top().iter() {
                if table_has_caption(child, parser) {
                    return true;
                }
            }
        }
    }
    false
}

fn table_contains_nested_table(node_handle: &tl::NodeHandle, parser: &tl::Parser, is_root: bool) -> bool {
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
            if !is_root && tag_name.as_ref() == "table" {
                return true;
            }

            for child in tag.children().top().iter() {
                if table_contains_nested_table(child, parser, false) {
                    return true;
                }
            }
        }
    }
    false
}

fn collect_table_row_counts(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    counts: &mut Vec<usize>,
    has_span: &mut bool,
) {
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
            match tag_name.as_ref() {
                "tr" => {
                    let mut cell_count = 0;
                    for child in tag.children().top().iter() {
                        if let Some(tl::Node::Tag(cell_tag)) = child.get(parser) {
                            let cell_name = normalized_tag_name(cell_tag.name().as_utf8_str());
                            if cell_name.as_ref() == "td" || cell_name.as_ref() == "th" {
                                cell_count += 1;
                                let attrs = cell_tag.attributes();
                                if attrs.get("colspan").is_some() || attrs.get("rowspan").is_some() {
                                    *has_span = true;
                                }
                            }
                        }
                    }
                    counts.push(cell_count);
                }
                _ => {
                    for child in tag.children().top().iter() {
                        collect_table_row_counts(child, parser, counts, has_span);
                    }
                }
            }
        }
    }
}

fn count_links(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> usize {
    let mut total = 0;
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
            if tag_name.as_ref() == "a" {
                total += 1;
            }

            for child in tag.children().top().iter() {
                total += count_links(child, parser);
            }
        }
    }
    total
}

fn append_layout_row(
    row_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    dom_ctx: &DomContext,
) {
    if let Some(tl::Node::Tag(row_tag)) = row_handle.get(parser) {
        let mut row_text = String::new();
        let row_children = row_tag.children();
        for cell_handle in row_children.top().iter() {
            if let Some(tl::Node::Tag(cell_tag)) = cell_handle.get(parser) {
                let cell_name = normalized_tag_name(cell_tag.name().as_utf8_str());
                if cell_name.as_ref() == "td" || cell_name.as_ref() == "th" {
                    let mut cell_text = String::new();
                    let cell_ctx = Context {
                        convert_as_inline: true,
                        ..ctx.clone()
                    };
                    let cell_children = cell_tag.children();
                    for cell_child in cell_children.top().iter() {
                        walk_node(cell_child, parser, &mut cell_text, options, &cell_ctx, 0, dom_ctx);
                    }
                    let cell_content = text::normalize_whitespace(&cell_text);
                    if !cell_content.trim().is_empty() {
                        if !row_text.is_empty() {
                            row_text.push(' ');
                        }
                        row_text.push_str(cell_content.trim());
                    }
                }
            }
        }

        let trimmed = row_text.trim();
        if !trimmed.is_empty() {
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            let formatted = trimmed.strip_prefix("- ").unwrap_or(trimmed).trim_start();
            output.push_str("- ");
            output.push_str(formatted);
            output.push('\n');
        }
    }
}

/// Indent table lines so they stay within their parent list item.
fn indent_table_for_list(table_content: &str, list_depth: usize, options: &ConversionOptions) -> String {
    if list_depth == 0 {
        return table_content.to_string();
    }

    let Some(mut indent) = continuation_indent_string(list_depth, options) else {
        return table_content.to_string();
    };

    if matches!(options.list_indent_type, ListIndentType::Spaces) {
        let space_count = indent.chars().filter(|c| *c == ' ').count();
        if space_count < 4 {
            indent.push_str(&" ".repeat(4 - space_count));
        }
    }

    let mut result = String::with_capacity(table_content.len() + indent.len() * 4);
    for segment in table_content.split_inclusive('\n') {
        if segment.starts_with('|') {
            result.push_str(&indent);
            result.push_str(segment);
        } else {
            result.push_str(segment);
        }
    }
    result
}

/// Convert an entire table element
fn convert_table(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    dom_ctx: &DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut row_counts = Vec::new();
        let mut has_span = false;
        collect_table_row_counts(node_handle, parser, &mut row_counts, &mut has_span);

        let row_count = row_counts.len();
        let mut distinct_counts: Vec<_> = row_counts.into_iter().filter(|c| *c > 0).collect();
        distinct_counts.sort_unstable();
        distinct_counts.dedup();

        let looks_like_layout =
            table_contains_nested_table(node_handle, parser, true) || has_span || distinct_counts.len() > 1;
        let link_count = count_links(node_handle, parser);
        let table_text = text::normalize_whitespace(&get_text_content(node_handle, parser));
        let is_blank_table = table_text.trim().is_empty();

        if !table_has_header(node_handle, parser)
            && !table_has_caption(node_handle, parser)
            && (looks_like_layout || is_blank_table || (row_count <= 2 && link_count >= 3))
        {
            if is_blank_table {
                return;
            }

            let table_children = tag.children();
            for child_handle in table_children.top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                    match tag_name.as_ref() {
                        "thead" | "tbody" | "tfoot" => {
                            for row_handle in child_tag.children().top().iter() {
                                if let Some(tl::Node::Tag(row_tag)) = row_handle.get(parser) {
                                    if tag_name_eq(row_tag.name().as_utf8_str(), "tr") {
                                        append_layout_row(row_handle, parser, output, options, ctx, dom_ctx);
                                    }
                                }
                            }
                        }
                        "tr" => append_layout_row(child_handle, parser, output, options, ctx, dom_ctx),
                        _ => {}
                    }
                }
            }
            if !output.ends_with('\n') {
                output.push('\n');
            }
            return;
        }

        let mut row_index = 0;
        let mut rowspan_tracker = std::collections::HashMap::new();

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());

                    match tag_name.as_ref() {
                        "caption" => {
                            let mut text = String::new();
                            let grandchildren = child_tag.children();
                            {
                                for grandchild_handle in grandchildren.top().iter() {
                                    walk_node(grandchild_handle, parser, &mut text, options, ctx, 0, dom_ctx);
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
                                        if tag_name_eq(row_tag.name().as_utf8_str(), "tr") {
                                            convert_table_row(
                                                row_handle,
                                                parser,
                                                output,
                                                options,
                                                ctx,
                                                row_index,
                                                &mut rowspan_tracker,
                                                dom_ctx,
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
                                dom_ctx,
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
    use crate::options::HighlightStyle;

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
    fn nested_strong_markup_is_normalized() {
        let html = "<strong><strong>Bold</strong></strong>";
        let result = convert_html(html, &ConversionOptions::default()).unwrap();
        assert_eq!(result.trim(), "**Bold**");
    }

    #[test]
    fn nested_strong_with_additional_text_is_normalized() {
        let html = "<strong>Hello <strong>World</strong></strong>";
        let result = convert_html(html, &ConversionOptions::default()).unwrap();
        assert_eq!(result.trim(), "**Hello World**");
    }

    #[test]
    fn nested_strong_partial_segments_are_normalized() {
        let html = "<b>bo<b>ld</b>er</b>";
        let result = convert_html(html, &ConversionOptions::default()).unwrap();
        assert_eq!(result.trim(), "**bolder**");
    }

    #[test]
    fn summary_with_inner_strong_is_not_double_wrapped() {
        let html = "<details><summary><strong>Title</strong></summary></details>";
        let mut options = ConversionOptions::default();
        options.preprocessing.remove_forms = false;
        let result = convert_html(html, &options).unwrap();
        assert_eq!(result.trim(), "**Title**");
    }

    #[test]
    fn legend_with_inner_strong_is_not_double_wrapped() {
        let html = "<fieldset><legend><strong>Section</strong></legend></fieldset>";
        let mut options = ConversionOptions::default();
        options.preprocessing.remove_forms = false; // keep form controls for this regression check
        let result = convert_html(html, &options).unwrap();
        assert_eq!(result.trim(), "**Section**");
    }

    #[test]
    fn preprocessing_keeps_article_header_inside_main() {
        let html = r#"
        <body>
            <header class="global-header">
                <div>Global Navigation</div>
            </header>
            <main>
                <header class="article-header">
                    <h1>Primary Title</h1>
                </header>
                <p>Body content stays.</p>
            </main>
        </body>
        "#;
        let mut options = ConversionOptions::default();
        options.preprocessing.enabled = true;
        let result = convert_html(html, &options).unwrap();
        assert!(
            result.contains("Primary Title"),
            "article header was removed: {}",
            result
        );
        assert!(
            result.contains("Body content stays"),
            "main body content missing: {}",
            result
        );
        assert!(
            !result.contains("Global Navigation"),
            "site chrome unexpectedly rendered: {}",
            result
        );
    }

    #[test]
    fn preprocessing_drops_nav_but_keeps_body() {
        let html = r##"
        <main>
            <nav aria-label="Primary navigation">
                <a href="#a">NavOnly</a>
            </nav>
            <article>
                <p>Important narrative</p>
            </article>
        </main>
        "##;
        let mut options = ConversionOptions::default();
        options.preprocessing.enabled = true;
        let result = convert_html(html, &options).unwrap();
        assert!(
            !result.contains("NavOnly"),
            "navigation text should not appear: {}",
            result
        );
        assert!(
            result.contains("Important narrative"),
            "article text should remain: {}",
            result
        );
    }

    #[test]
    fn preprocessing_retains_section_headers_inside_articles() {
        let html = r#"
        <article>
            <header>
                <h2>Section Heading</h2>
            </header>
            <section>
                <p>Section body</p>
            </section>
        </article>
        "#;
        let result = convert_html(html, &ConversionOptions::default()).unwrap();
        assert!(
            result.contains("Section Heading"),
            "section heading was stripped: {}",
            result
        );
        assert!(result.contains("Section body"), "section body missing: {}", result);
    }

    #[test]
    fn bold_highlight_suppresses_nested_strong() {
        let mut options = ConversionOptions::default();
        options.highlight_style = HighlightStyle::Bold;
        let html = "<p><mark><strong>Hot</strong></mark></p>";
        let result = convert_html(html, &options).unwrap();
        assert_eq!(result.trim(), "**Hot**");
    }

    #[test]
    fn atx_heading_swallows_layout_line_breaks() {
        let html = r#"<h2>
  Heading
  Text
  with
  Line
  Breaks
</h2>"#;
        let result = convert_html(html, &ConversionOptions::default()).unwrap();
        assert_eq!(result.trim(), "## Heading Text with Line Breaks");
    }

    #[test]
    fn doctype_is_removed() {
        let html = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.0 Transitional//EN" "http://www.w3.org/TR/REC-html40/loose.dtd">
            <html>
                <head><title>Example</title></head>
                <body><p>Hello World</p></body>
            </html>"#;
        let mut options = ConversionOptions::default();
        options.extract_metadata = false;
        let result = convert_html(html, &options).unwrap();
        assert_eq!(result.trim(), "Hello World");
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
    fn hr_inside_paragraph_matches_inline_expectation() {
        let mut options = ConversionOptions::default();
        options.extract_metadata = false;
        let markdown = convert_html("<p>Hello<hr>World</p>", &options).unwrap();
        assert_eq!(markdown, "Hello\n---\nWorld\n");
    }

    #[test]
    fn hr_inside_paragraph_matches_inline_expectation_via_public_api() {
        let mut options = ConversionOptions::default();
        options.extract_metadata = false;
        let markdown = crate::convert("<p>Hello<hr>World</p>", Some(options)).unwrap();
        assert_eq!(markdown, "Hello\n---\nWorld\n");
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

    #[test]
    fn test_preserve_tags_simple_table() {
        let html = r#"<div><table><tr><td>Cell 1</td><td>Cell 2</td></tr></table><p>Text</p></div>"#;
        let options = ConversionOptions {
            preserve_tags: vec!["table".to_string()],
            ..Default::default()
        };
        let result = convert_html(html, &options).unwrap();

        assert!(result.contains("<table>"), "Should preserve table tag");
        assert!(result.contains("</table>"), "Should have closing table tag");
        assert!(result.contains("<tr>"), "Should preserve tr tag");
        assert!(result.contains("<td>"), "Should preserve td tag");
        assert!(result.contains("Text"), "Should convert other elements");
    }

    #[test]
    fn test_preserve_tags_with_attributes() {
        let html = r#"<table class="data" id="mytable"><tr><td>Data</td></tr></table>"#;
        let options = ConversionOptions {
            preserve_tags: vec!["table".to_string()],
            ..Default::default()
        };
        let result = convert_html(html, &options).unwrap();

        assert!(result.contains("<table"), "Should preserve table tag");
        assert!(result.contains("class="), "Should preserve class attribute");
        assert!(result.contains("id="), "Should preserve id attribute");
        assert!(result.contains("</table>"), "Should have closing tag");
    }

    #[test]
    fn test_preserve_tags_multiple_tags() {
        let html = r#"<div><table><tr><td>Table</td></tr></table><form><input type="text"/></form><p>Text</p></div>"#;
        let options = ConversionOptions {
            preserve_tags: vec!["table".to_string(), "form".to_string()],
            ..Default::default()
        };
        let result = convert_html(html, &options).unwrap();

        assert!(result.contains("<table>"), "Should preserve table");
        assert!(result.contains("<form>"), "Should preserve form");
        assert!(result.contains("Text"), "Should convert paragraph");
    }

    #[test]
    fn test_preserve_tags_nested_content() {
        let html = r#"<table><thead><tr><th>Header</th></tr></thead><tbody><tr><td>Data</td></tr></tbody></table>"#;
        let options = ConversionOptions {
            preserve_tags: vec!["table".to_string()],
            ..Default::default()
        };
        let result = convert_html(html, &options).unwrap();

        assert!(result.contains("<thead>"), "Should preserve nested thead");
        assert!(result.contains("<tbody>"), "Should preserve nested tbody");
        assert!(result.contains("<th>"), "Should preserve th tag");
        assert!(result.contains("Header"), "Should preserve text content");
    }

    #[test]
    fn test_preserve_tags_empty_list() {
        let html = r#"<table><tr><td>Cell</td></tr></table>"#;
        let options = ConversionOptions::default(); // No preserve_tags
        let result = convert_html(html, &options).unwrap();

        // Should convert to markdown table (or at least not preserve HTML)
        assert!(
            !result.contains("<table>"),
            "Should not preserve table without preserve_tags"
        );
    }

    #[test]
    fn test_preserve_tags_vs_strip_tags() {
        let html = r#"<table><tr><td>Table</td></tr></table><div><span>Text</span></div>"#;
        let options = ConversionOptions {
            preserve_tags: vec!["table".to_string()],
            strip_tags: vec!["span".to_string()],
            ..Default::default()
        };
        let result = convert_html(html, &options).unwrap();

        assert!(result.contains("<table>"), "Should preserve table");
        assert!(!result.contains("<span>"), "Should strip span tag");
        assert!(result.contains("Text"), "Should keep span text content");
    }

    #[test]
    fn example_com_remains_visible() {
        let html = "<!doctype html><html lang=\"en\"><head><title>Example Domain</title><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>body{background:#eee;width:60vw;margin:15vh auto;font-family:system-ui,sans-serif}h1{font-size:1.5em}div{opacity:0.8}a:link,a:visited{color:#348}</style><body><div><h1>Example Domain</h1><p>This domain is for use in documentation examples without needing permission. Avoid use in operations.<p><a href=\"https://iana.org/domains/example\">Learn more</a></div></body></html>";

        let mut options = ConversionOptions::default();
        options.extract_metadata = false; // matches CLI default
        let result = convert_html(html, &options).unwrap();

        assert!(
            result.contains("Example Domain"),
            "content unexpectedly missing: {}",
            result
        );
    }
}
#[test]
fn normalize_self_closing_tags_noop_when_absent() {
    let html = "<div><p>text</p></div>";
    let normalized = normalize_self_closing_tags(html);
    assert!(matches!(normalized, Cow::Borrowed(_)));
    assert_eq!(normalized.as_ref(), html);
}

#[test]
fn normalize_self_closing_tags_replaces_targets() {
    let html = "<br/><hr/><img/>";
    let normalized = normalize_self_closing_tags(html);
    assert_eq!(normalized.as_ref(), "<br><hr><img>");
}
