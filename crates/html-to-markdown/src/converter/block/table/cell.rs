//! Table cell conversion utilities.
//!
//! Handles conversion of table cell (td/th) elements to Markdown format,
//! including colspan support and content normalization.

use std::borrow::Cow;

/// Maximum allowed table columns to prevent unbounded memory usage.
const MAX_TABLE_COLS: usize = 1000;

/// Get colspan attribute value from an element.
///
/// Reads the colspan attribute from a table cell, with bounds checking
/// to prevent memory exhaustion attacks.
///
/// # Arguments
/// * `node_handle` - Handle to the cell element
/// * `parser` - HTML parser instance
///
/// # Returns
/// The colspan value (minimum 1, maximum MAX_TABLE_COLS)
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn get_colspan(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> usize {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        if let Some(Some(bytes)) = tag.attributes().get("colspan") {
            if let Ok(colspan) = bytes.as_utf8_str().parse::<usize>() {
                return clamp_table_span(colspan);
            }
        }
    }
    1
}

/// Get both colspan and rowspan in a single lookup.
///
/// More efficient than calling get_colspan and a separate rowspan lookup.
///
/// # Arguments
/// * `node_handle` - Handle to the cell element
/// * `parser` - HTML parser instance
///
/// # Returns
/// A tuple of (colspan, rowspan), both minimum 1 and maximum MAX_TABLE_COLS
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn get_colspan_rowspan(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> (usize, usize) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let attrs = tag.attributes();
        let colspan = attrs
            .get("colspan")
            .flatten()
            .and_then(|v| v.as_utf8_str().parse::<usize>().ok())
            .map_or(1, clamp_table_span);
        let rowspan = attrs
            .get("rowspan")
            .flatten()
            .and_then(|v| v.as_utf8_str().parse::<usize>().ok())
            .map_or(1, clamp_table_span);
        (colspan, rowspan)
    } else {
        (1, 1)
    }
}

/// Clamp a table span value to safe bounds.
///
/// Prevents memory exhaustion by clamping colspan/rowspan values.
fn clamp_table_span(value: usize) -> usize {
    if value == 0 { 1 } else { value.min(MAX_TABLE_COLS) }
}

/// Collect table cells (td/th) from a row element.
///
/// Extracts only the direct cell children of a row, filtering by tag name.
///
/// # Arguments
/// * `node_handle` - Handle to the row element
/// * `parser` - HTML parser instance
/// * `dom_ctx` - DOM context for tag name resolution
/// * `cells` - Mutable vector to populate with cell handles
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn collect_table_cells(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
    cells: &mut Vec<tl::NodeHandle>,
) {
    cells.clear();
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        for child_handle in children.top().iter() {
            if let Some(cell_name) = dom_ctx.tag_name_for(*child_handle, parser) {
                if matches!(cell_name.as_ref(), "th" | "td" | "cell") {
                    cells.push(*child_handle);
                }
            }
        }
    }
}

/// Convert a table cell (td or th) to Markdown format.
///
/// Processes cell content and renders it with pipe delimiters for Markdown tables.
/// Handles colspan by adding extra pipes, and escapes pipes in cell content.
///
/// # Arguments
/// * `node_handle` - Handle to the cell element
/// * `parser` - HTML parser instance
/// * `output` - Mutable string to append cell content
/// * `options` - Conversion options (escape settings, br_in_tables)
/// * `ctx` - Conversion context (visitor, etc)
/// * `_tag_name` - Tag name (for consistency, not used)
/// * `dom_ctx` - DOM context for content extraction
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn convert_table_cell(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::super::super::Context,
    _tag_name: &str,
    dom_ctx: &super::super::super::DomContext,
) {
    let mut text = String::with_capacity(128);

    let cell_ctx = super::super::super::Context {
        in_table_cell: true,
        ..ctx.clone()
    };

    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        let has_tag_child = children
            .top()
            .iter()
            .any(|child_handle| matches!(child_handle.get(parser), Some(tl::Node::Tag(_))));

        if has_tag_child {
            for child_handle in children.top().iter() {
                super::super::super::walk_node(child_handle, parser, &mut text, options, &cell_ctx, 0, dom_ctx);
            }
        } else {
            let raw = dom_ctx.text_content(*node_handle, parser);
            let normalized = if options.whitespace_mode == crate::options::WhitespaceMode::Normalized {
                crate::text::normalize_whitespace_cow(raw.as_str())
            } else {
                Cow::Borrowed(raw.as_str())
            };
            let escaped = crate::text::escape(
                normalized.as_ref(),
                options.escape_misc,
                options.escape_asterisks,
                options.escape_underscores,
                options.escape_ascii,
            );
            if options.escape_misc {
                text = escaped;
            } else {
                text = escaped.replace('|', r"\|");
            }
        }
    }

    let text = text.trim();
    let text = if options.br_in_tables {
        let mut joined = String::with_capacity(text.len());
        let mut first = true;
        for segment in text.split('\n').filter(|s| !s.is_empty()) {
            if !first {
                joined.push_str("<br>");
            }
            first = false;
            joined.push_str(segment);
        }
        joined
    } else if text.contains('\n') {
        text.replace('\n', " ")
    } else {
        text.to_string()
    };

    let colspan = get_colspan(node_handle, parser);

    output.push(' ');
    output.push_str(&text);
    output.push_str(&" |".repeat(colspan));
}
