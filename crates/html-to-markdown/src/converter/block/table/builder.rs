//! Table building and formatting utilities.
//!
//! Handles table structure building, row/cell rendering, and conversion
//! of layout tables to list format. Includes visitor pattern integration
//! for custom table handling.

use crate::options::ListIndentType;
use std::borrow::Cow;

/// Maximum allowed table columns to prevent unbounded memory usage.
const MAX_TABLE_COLS: usize = 1000;

use super::cell::{collect_table_cells, convert_table_cell, get_colspan, get_colspan_rowspan};
use super::scanner::scan_table;

/// Calculate total columns in a table.
///
/// Scans all rows and cells to determine the maximum column count,
/// accounting for colspan values.
///
/// # Arguments
/// * `node_handle` - Handle to the table element
/// * `parser` - HTML parser instance
/// * `dom_ctx` - DOM context for tag name resolution
///
/// # Returns
/// Maximum column count (minimum 1, maximum MAX_TABLE_COLS)
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn table_total_columns(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
) -> usize {
    let mut max_cols = 0usize;
    let mut cells = Vec::new();

    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        for child_handle in children.top().iter() {
            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                let tag_name = dom_ctx
                    .tag_name_for(*child_handle, parser)
                    .unwrap_or_else(|| normalized_tag_name(child_tag.name().as_utf8_str()));
                match tag_name.as_ref() {
                    "thead" | "tbody" | "tfoot" => {
                        for row_handle in child_tag.children().top().iter() {
                            if is_tag_name(row_handle, parser, dom_ctx, "tr") {
                                collect_table_cells(row_handle, parser, dom_ctx, &mut cells);
                                let col_count = cells
                                    .iter()
                                    .fold(0usize, |acc, h| acc.saturating_add(get_colspan(h, parser)));
                                max_cols = max_cols.max(col_count);
                            }
                        }
                    }
                    "tr" | "row" => {
                        collect_table_cells(child_handle, parser, dom_ctx, &mut cells);
                        let col_count = cells
                            .iter()
                            .fold(0usize, |acc, h| acc.saturating_add(get_colspan(h, parser)));
                        max_cols = max_cols.max(col_count);
                    }
                    _ => {}
                }
            }
        }
    }

    max_cols.clamp(1, MAX_TABLE_COLS)
}

/// Check if a node has a specific tag name.
///
/// Handles both direct tag matching and DOM context-based tag resolution.
///
/// # Arguments
/// * `node_handle` - Handle to the node
/// * `parser` - HTML parser instance
/// * `dom_ctx` - DOM context for tag name resolution
/// * `name` - Expected tag name
///
/// # Returns
/// True if node has the specified tag name
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_tag_name(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
    name: &str,
) -> bool {
    if let Some(info) = dom_ctx.tag_info(node_handle.get_inner(), parser) {
        return info.name == name;
    }
    matches!(
        node_handle.get(parser),
        Some(tl::Node::Tag(tag)) if tag_name_eq(tag.name().as_utf8_str(), name)
    )
}

/// Convert a table row (tr) to Markdown format.
///
/// Processes all cells in a row, handling colspan and rowspan for proper
/// column alignment. Renders header separator row after the first row.
/// Integrates with visitor pattern for custom row handling.
///
/// # Arguments
/// * `node_handle` - Handle to the row element
/// * `parser` - HTML parser instance
/// * `output` - Mutable string to append row content
/// * `options` - Conversion options
/// * `ctx` - Conversion context (visitor, etc)
/// * `row_index` - Index of this row in the table
/// * `has_span` - Whether table has colspan/rowspan
/// * `rowspan_tracker` - Mutable array tracking rowspan remainder for each column
/// * `total_cols` - Total columns in the table
/// * `header_cols` - Columns to render in separator row
/// * `dom_ctx` - DOM context
/// * `depth` - Nesting depth
/// * `is_header` - Whether this is a header row
#[allow(clippy::too_many_arguments)]
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn convert_table_row(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::super::super::Context,
    row_index: usize,
    has_span: bool,
    rowspan_tracker: &mut [Option<usize>],
    total_cols: usize,
    header_cols: usize,
    dom_ctx: &super::super::super::DomContext,
    depth: usize,
    is_header: bool,
) {
    let mut row_text = String::with_capacity(256);
    let mut cells = Vec::new();

    collect_table_cells(node_handle, parser, dom_ctx, &mut cells);

    #[cfg(feature = "visitor")]
    let cell_contents: Vec<String> = if ctx.visitor.is_some() {
        cells
            .iter()
            .map(|cell_handle| {
                let mut text = String::new();
                let cell_ctx = super::super::super::Context {
                    in_table_cell: true,
                    ..ctx.clone()
                };
                if let Some(tl::Node::Tag(tag)) = cell_handle.get(parser) {
                    for child_handle in tag.children().top().iter() {
                        super::super::super::walk_node(child_handle, parser, &mut text, options, &cell_ctx, 0, dom_ctx);
                    }
                }
                crate::text::normalize_whitespace_cow(&text).trim().to_string()
            })
            .collect()
    } else {
        Vec::new()
    };

    #[cfg(feature = "visitor")]
    if let Some(ref visitor_handle) = ctx.visitor {
        use crate::visitor::{NodeContext, NodeType, VisitResult};
        use std::collections::BTreeMap;

        if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
            let attributes: BTreeMap<String, String> = tag
                .attributes()
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                .collect();

            let node_ctx = NodeContext {
                node_type: NodeType::TableRow,
                tag_name: "tr".to_string(),
                attributes,
                depth,
                index_in_parent: row_index,
                parent_tag: Some("table".to_string()),
                is_inline: false,
            };

            let mut visitor = visitor_handle.borrow_mut();
            match visitor.visit_table_row(&node_ctx, &cell_contents, is_header) {
                VisitResult::Continue => {}
                VisitResult::Skip => return,
                VisitResult::Custom(custom) => {
                    output.push_str(&custom);
                    return;
                }
                VisitResult::Error(err) => {
                    if ctx.visitor_error.borrow().is_none() {
                        *ctx.visitor_error.borrow_mut() = Some(err);
                    }
                    return;
                }
                VisitResult::PreserveHtml => {
                    output.push_str(&super::super::super::serialize_node(node_handle, parser));
                    return;
                }
            }
        }
    }

    if has_span {
        let mut col_index = 0;
        let mut cell_iter = cells.iter();

        loop {
            if col_index < total_cols {
                if let Some(Some(remaining_rows)) = rowspan_tracker.get_mut(col_index) {
                    if *remaining_rows > 0 {
                        row_text.push(' ');
                        row_text.push_str(" |");
                        *remaining_rows -= 1;
                        if *remaining_rows == 0 {
                            rowspan_tracker[col_index] = None;
                        }
                        col_index += 1;
                        continue;
                    }
                }
            }

            if let Some(cell_handle) = cell_iter.next() {
                convert_table_cell(cell_handle, parser, &mut row_text, options, ctx, "", dom_ctx);

                let (colspan, rowspan) = get_colspan_rowspan(cell_handle, parser);

                if rowspan > 1 && col_index < total_cols {
                    rowspan_tracker[col_index] = Some(rowspan - 1);
                }

                col_index = col_index.saturating_add(colspan);
            } else {
                break;
            }
        }
    } else {
        for cell_handle in &cells {
            convert_table_cell(cell_handle, parser, &mut row_text, options, ctx, "", dom_ctx);
        }
    }

    output.push('|');
    output.push_str(&row_text);
    output.push('\n');

    let is_first_row = row_index == 0;
    if is_first_row {
        let total_cols = header_cols.clamp(1, MAX_TABLE_COLS);
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

/// Append a layout table row as a list item.
///
/// For tables used for visual layout, converts rows to list items
/// instead of table format for better readability.
///
/// # Arguments
/// * `row_handle` - Handle to the row element
/// * `parser` - HTML parser instance
/// * `output` - Mutable string to append content
/// * `options` - Conversion options
/// * `ctx` - Conversion context
/// * `dom_ctx` - DOM context
#[allow(clippy::trivially_copy_pass_by_ref)]
fn append_layout_row(
    row_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::super::super::Context,
    dom_ctx: &super::super::super::DomContext,
) {
    if let Some(tl::Node::Tag(row_tag)) = row_handle.get(parser) {
        let mut row_text = String::new();
        let row_children = row_tag.children();
        for cell_handle in row_children.top().iter() {
            if let Some(tl::Node::Tag(cell_tag)) = cell_handle.get(parser) {
                let cell_name: Cow<'_, str> = dom_ctx.tag_info(cell_handle.get_inner(), parser).map_or_else(
                    || normalized_tag_name(cell_tag.name().as_utf8_str()).into_owned().into(),
                    |info| Cow::Borrowed(info.name.as_str()),
                );
                if matches!(cell_name.as_ref(), "td" | "th" | "cell") {
                    let mut cell_text = String::new();
                    let cell_ctx = super::super::super::Context {
                        convert_as_inline: true,
                        ..ctx.clone()
                    };
                    let cell_children = cell_tag.children();
                    for cell_child in cell_children.top().iter() {
                        super::super::super::walk_node(
                            cell_child,
                            parser,
                            &mut cell_text,
                            options,
                            &cell_ctx,
                            0,
                            dom_ctx,
                        );
                    }
                    let cell_content = crate::text::normalize_whitespace_cow(&cell_text);
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

/// Indent table lines for list context.
///
/// When a table appears inside a list item, this function indents the table
/// content so it maintains proper list nesting.
///
/// # Arguments
/// * `table_content` - The Markdown table content to indent
/// * `list_depth` - The nesting depth in the list hierarchy
/// * `options` - Conversion options (for indent type)
///
/// # Returns
/// Indented table content
pub(crate) fn indent_table_for_list(
    table_content: &str,
    list_depth: usize,
    options: &crate::options::ConversionOptions,
) -> String {
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

/// Get continuation indent string for list nesting.
fn continuation_indent_string(list_depth: usize, options: &crate::options::ConversionOptions) -> Option<String> {
    use crate::converter::continuation_indent_string;
    continuation_indent_string(list_depth, options)
}

/// Convert an entire table element to Markdown.
///
/// Main entry point for table conversion. Analyzes table structure to determine
/// if it should be rendered as a Markdown table or converted to list format.
/// Handles layout tables, blank tables, and tables with semantic meaning.
/// Integrates with visitor pattern for custom table handling.
///
/// # Arguments
/// * `node_handle` - Handle to the table element
/// * `parser` - HTML parser instance
/// * `output` - Mutable string to append table content
/// * `options` - Conversion options
/// * `ctx` - Conversion context (visitor, etc)
/// * `dom_ctx` - DOM context
/// * `depth` - Nesting depth
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn handle_table(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::super::super::Context,
    dom_ctx: &super::super::super::DomContext,
    depth: usize,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        #[cfg(feature = "visitor")]
        let table_output_start = output.len();

        #[cfg(feature = "visitor")]
        let mut table_start_custom: Option<String> = None;

        #[cfg(feature = "visitor")]
        if let Some(ref visitor_handle) = ctx.visitor {
            use crate::visitor::{NodeContext, NodeType, VisitResult};
            use std::collections::BTreeMap;

            let attributes: BTreeMap<String, String> = tag
                .attributes()
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                .collect();

            let node_id = node_handle.get_inner();
            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

            let node_ctx = NodeContext {
                node_type: NodeType::Table,
                tag_name: "table".to_string(),
                attributes,
                depth,
                index_in_parent,
                parent_tag,
                is_inline: false,
            };

            let mut visitor = visitor_handle.borrow_mut();
            match visitor.visit_table_start(&node_ctx) {
                VisitResult::Continue => {}
                VisitResult::Skip => return,
                VisitResult::Custom(custom) => {
                    table_start_custom = Some(custom);
                }
                VisitResult::Error(err) => {
                    if ctx.visitor_error.borrow().is_none() {
                        *ctx.visitor_error.borrow_mut() = Some(err);
                    }
                    return;
                }
                VisitResult::PreserveHtml => {
                    output.push_str(&super::super::super::serialize_node(node_handle, parser));
                    return;
                }
            }
        }

        let table_scan = scan_table(node_handle, parser, dom_ctx);
        let row_count = table_scan.row_counts.len();
        let mut distinct_counts: Vec<_> = table_scan.row_counts.iter().copied().filter(|c| *c > 0).collect();
        distinct_counts.sort_unstable();
        distinct_counts.dedup();

        let looks_like_layout = table_scan.has_nested_table || table_scan.has_span || distinct_counts.len() > 1;
        let link_count = table_scan.link_count;
        let is_blank_table = !table_scan.has_text;

        if !table_scan.has_header
            && !table_scan.has_caption
            && (looks_like_layout || is_blank_table || (row_count <= 2 && link_count >= 3))
        {
            // Skip truly blank tables (no text, no links, no images)
            if is_blank_table && link_count == 0 {
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
                                    let row_tag_name = normalized_tag_name(row_tag.name().as_utf8_str());
                                    if matches!(row_tag_name.as_ref(), "tr" | "row") {
                                        append_layout_row(row_handle, parser, output, options, ctx, dom_ctx);
                                    }
                                }
                            }
                        }
                        "tr" | "row" => append_layout_row(child_handle, parser, output, options, ctx, dom_ctx),
                        "colgroup" | "col" => {}
                        _ => {
                            // Handle non-table-structure elements (like <a>, <img>, etc.) that may be
                            // direct children of layout tables (e.g., Blogger table wrappers)
                            super::super::super::walk_node(
                                child_handle,
                                parser,
                                output,
                                options,
                                ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }
                }
            }
            if !output.ends_with('\n') {
                output.push('\n');
            }
            return;
        }

        let mut row_index = 0;
        let total_cols = table_total_columns(node_handle, parser, dom_ctx);
        let mut first_row_cols: Option<usize> = None;
        let mut rowspan_tracker = vec![None; total_cols];
        let mut row_cells = Vec::new();

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let tag_name: Cow<'_, str> = dom_ctx.tag_info(child_handle.get_inner(), parser).map_or_else(
                        || normalized_tag_name(child_tag.name().as_utf8_str()).into_owned().into(),
                        |info| Cow::Borrowed(info.name.as_str()),
                    );

                    match tag_name.as_ref() {
                        "caption" => {
                            let mut text = String::new();
                            let grandchildren = child_tag.children();
                            {
                                for grandchild_handle in grandchildren.top().iter() {
                                    super::super::super::walk_node(
                                        grandchild_handle,
                                        parser,
                                        &mut text,
                                        options,
                                        ctx,
                                        0,
                                        dom_ctx,
                                    );
                                }
                            }
                            let text = text.trim();
                            if !text.is_empty() {
                                let escaped_text = text.replace('-', r"\-");
                                output.push('*');
                                output.push_str(&escaped_text);
                                output.push_str("*\n\n");
                            }
                        }

                        "thead" | "tbody" | "tfoot" => {
                            let is_header_section = tag_name.as_ref() == "thead";
                            let section_children = child_tag.children();
                            {
                                for row_handle in section_children.top().iter() {
                                    if let Some(tl::Node::Tag(row_tag)) = row_handle.get(parser) {
                                        let row_tag_name = dom_ctx
                                            .tag_name_for(*row_handle, parser)
                                            .unwrap_or_else(|| normalized_tag_name(row_tag.name().as_utf8_str()));
                                        if matches!(row_tag_name.as_ref(), "tr" | "row") {
                                            if first_row_cols.is_none() {
                                                collect_table_cells(row_handle, parser, dom_ctx, &mut row_cells);
                                                let cols = row_cells
                                                    .iter()
                                                    .fold(0usize, |acc, h| acc.saturating_add(get_colspan(h, parser)));
                                                first_row_cols = Some(cols.clamp(1, MAX_TABLE_COLS));
                                            }
                                            convert_table_row(
                                                row_handle,
                                                parser,
                                                output,
                                                options,
                                                ctx,
                                                row_index,
                                                table_scan.has_span,
                                                &mut rowspan_tracker,
                                                total_cols,
                                                first_row_cols.unwrap_or(total_cols),
                                                dom_ctx,
                                                depth + 1,
                                                is_header_section,
                                            );
                                            row_index += 1;
                                        }
                                    }
                                }
                            }
                        }

                        "tr" | "row" => {
                            if first_row_cols.is_none() {
                                collect_table_cells(child_handle, parser, dom_ctx, &mut row_cells);
                                let cols = row_cells
                                    .iter()
                                    .fold(0usize, |acc, h| acc.saturating_add(get_colspan(h, parser)));
                                first_row_cols = Some(cols.clamp(1, MAX_TABLE_COLS));
                            }
                            convert_table_row(
                                child_handle,
                                parser,
                                output,
                                options,
                                ctx,
                                row_index,
                                table_scan.has_span,
                                &mut rowspan_tracker,
                                total_cols,
                                first_row_cols.unwrap_or(total_cols),
                                dom_ctx,
                                depth + 1,
                                row_index == 0,
                            );
                            row_index += 1;
                        }

                        "colgroup" | "col" => {}

                        _ => {
                            // Handle non-table-structure elements (like <a>, <img>, etc.) that may be
                            // direct children of tables without proper structure (e.g., Blogger table wrappers)
                            super::super::super::walk_node(
                                child_handle,
                                parser,
                                output,
                                options,
                                ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }
                }
            }
        }

        #[cfg(feature = "visitor")]
        if let Some(ref visitor_handle) = ctx.visitor {
            use crate::visitor::{NodeContext, NodeType, VisitResult};
            use std::collections::BTreeMap;

            let attributes: BTreeMap<String, String> = tag
                .attributes()
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                .collect();

            let node_id = node_handle.get_inner();
            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

            let node_ctx = NodeContext {
                node_type: NodeType::Table,
                tag_name: "table".to_string(),
                attributes,
                depth,
                index_in_parent,
                parent_tag,
                is_inline: false,
            };

            let table_content = &output[table_output_start..];

            let mut visitor = visitor_handle.borrow_mut();
            match visitor.visit_table_end(&node_ctx, table_content) {
                VisitResult::Continue => {
                    if let Some(custom_start) = table_start_custom {
                        output.insert_str(table_output_start, &custom_start);
                    }
                }
                VisitResult::Custom(custom) => {
                    let rows_output = output[table_output_start..].to_string();
                    output.truncate(table_output_start);
                    if let Some(custom_start) = table_start_custom {
                        output.push_str(&custom_start);
                    }
                    output.push_str(&rows_output);
                    output.push_str(&custom);
                }
                VisitResult::Skip => {
                    output.truncate(table_output_start);
                }
                VisitResult::Error(err) => {
                    if ctx.visitor_error.borrow().is_none() {
                        *ctx.visitor_error.borrow_mut() = Some(err);
                    }
                }
                VisitResult::PreserveHtml => {
                    output.truncate(table_output_start);
                    output.push_str(&super::super::super::serialize_node(node_handle, parser));
                }
            }
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

/// Check tag name equality with case-insensitive comparison.
fn tag_name_eq(name: Cow<'_, str>, needle: &str) -> bool {
    name.eq_ignore_ascii_case(needle)
}
