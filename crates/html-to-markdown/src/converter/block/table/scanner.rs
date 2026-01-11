//! Table scanning and analysis utilities.
//!
//! Provides the TableScan struct and scanning functions for analyzing table structure
//! to determine if it should be rendered as a Markdown table or converted to list format.

use std::borrow::Cow;

/// Maximum allowed table columns to prevent unbounded memory usage.
const MAX_TABLE_COLS: usize = 1000;

/// Scan results for a table element.
///
/// Contains metadata about table structure to determine optimal rendering:
/// - Row counts for consistency checking
/// - Presence of headers, captions, and nested tables
/// - Presence of colspan/rowspan (spanning cells)
/// - Link and text content counts
#[derive(Default)]
pub struct TableScan {
    /// Number of cells in each row
    pub row_counts: Vec<usize>,
    /// Whether any cells have colspan or rowspan attributes
    pub has_span: bool,
    /// Whether the table has header cells (th elements or role="head")
    pub has_header: bool,
    /// Whether the table has a caption element
    pub has_caption: bool,
    /// Whether the table contains nested tables
    pub has_nested_table: bool,
    /// Count of anchor elements in the table
    pub link_count: usize,
    /// Whether the table contains text content (not empty)
    pub has_text: bool,
}

/// Scan a table element for structural metadata.
///
/// Analyzes the table to determine characteristics that influence rendering:
/// - Whether to render as a Markdown table or layout table
/// - If spanning cells are present
/// - If the table has semantic meaning (headers, captions)
///
/// # Arguments
/// * `node_handle` - Handle to the table element
/// * `parser` - HTML parser instance
/// * `dom_ctx` - DOM context for tag name resolution
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn scan_table(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
) -> TableScan {
    let mut scan = TableScan::default();
    scan_table_node(node_handle, parser, dom_ctx, true, &mut scan);
    scan
}

/// Recursively scan table structure.
///
/// Internal recursive function that walks the table tree and collects metadata.
///
/// # Arguments
/// * `node_handle` - Current node to scan
/// * `parser` - HTML parser instance
/// * `dom_ctx` - DOM context for tag name resolution
/// * `is_root` - Whether this is the root table element
/// * `scan` - Mutable scan results to accumulate
#[allow(clippy::trivially_copy_pass_by_ref)]
fn scan_table_node(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
    is_root: bool,
    scan: &mut TableScan,
) {
    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Raw(bytes) => {
                if !scan.has_text {
                    let raw = bytes.as_utf8_str();
                    let decoded = crate::text::decode_html_entities_cow(raw.as_ref());
                    if !decoded.trim().is_empty() {
                        scan.has_text = true;
                    }
                }
            }
            tl::Node::Tag(tag) => {
                let tag_name: Cow<'_, str> = dom_ctx.tag_info(node_handle.get_inner(), parser).map_or_else(
                    || normalized_tag_name(tag.name().as_utf8_str()).into_owned().into(),
                    |info| Cow::Borrowed(info.name.as_str()),
                );

                match tag_name.as_ref() {
                    "a" => scan.link_count += 1,
                    "caption" => scan.has_caption = true,
                    "th" => scan.has_header = true,
                    "img" | "graphic" => {
                        // Images with src or alt attributes count as content
                        if tag.attributes().get("src").is_some() || tag.attributes().get("alt").is_some() {
                            scan.has_text = true;
                        }
                    }
                    "cell" => {
                        // Check if cell has role="head" attribute
                        if let Some(role) = tag.attributes().get("role") {
                            if let Some(role_val) = role {
                                let role_str = role_val.as_utf8_str();
                                if role_str == "head" {
                                    scan.has_header = true;
                                }
                            }
                        }
                    }
                    "table" if !is_root => scan.has_nested_table = true,
                    "tr" | "row" => {
                        let mut cell_count = 0;
                        for child in tag.children().top().iter() {
                            if let Some(tl::Node::Tag(cell_tag)) = child.get(parser) {
                                let cell_name: Cow<'_, str> = dom_ctx
                                    .tag_info(child.get_inner(), parser)
                                    .map(|info| Cow::Borrowed(info.name.as_str()))
                                    .unwrap_or_else(|| {
                                        normalized_tag_name(cell_tag.name().as_utf8_str()).into_owned().into()
                                    });
                                if matches!(cell_name.as_ref(), "td" | "th" | "cell") {
                                    cell_count += 1;
                                    let attrs = cell_tag.attributes();
                                    if attrs.get("colspan").is_some() || attrs.get("rowspan").is_some() {
                                        scan.has_span = true;
                                    }
                                }
                            }
                            scan_table_node(child, parser, dom_ctx, false, scan);
                        }
                        scan.row_counts.push(cell_count);
                        return;
                    }
                    _ => {}
                }

                for child in tag.children().top().iter() {
                    scan_table_node(child, parser, dom_ctx, false, scan);
                }
            }
            _ => {}
        }
    }
}

/// Normalize HTML tag names to lowercase.
///
/// Converts tag names to a consistent lowercase form for comparison.
fn normalized_tag_name(raw: Cow<'_, str>) -> Cow<'_, str> {
    let lowercased = raw.to_lowercase();
    if lowercased.as_str() == raw.as_ref() {
        raw
    } else {
        Cow::Owned(lowercased)
    }
}
