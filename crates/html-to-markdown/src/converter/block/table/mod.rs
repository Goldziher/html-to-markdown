//! Table element handler for HTML to Markdown conversion.
//!
//! This module provides specialized handling for table elements including:
//! - Table structure detection and scanning (`TableScan`)
//! - Row and cell conversion to Markdown table format
//! - Cell content processing with colspan/rowspan support
//! - Layout table detection (tables used for visual layout)
//! - Integration with the visitor pattern for custom table handling
//!
//! Tables are converted to Markdown pipe-delimited format with header separators.
//! Layout tables (tables without proper semantic headers) may be converted to lists
//! instead of tables for better readability.

pub mod builder;
pub mod caption;
pub mod cell;
pub mod cells;
mod flatten;
pub mod layout;
pub mod scanner;
pub(super) mod utils;

use crate::converter::block::container::HandlerContext;

pub use caption::handle_caption;

/// Dispatches table element handling to the main `convert_table` function.
///
/// # Usage in converter.rs
/// ```text
/// if "table" == tag_name {
///     crate::converter::block::table::handle_table(
///         node_handle,
///         parser,
///         output,
///         options,
///         ctx,
///         dom_ctx,
///         depth,
///     );
///     return;
/// }
/// ```
pub fn dispatch_table_handler(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) -> bool {
    match tag_name {
        "table" => {
            builder::handle_table(node_handle, parser, output, handler);
            true
        }
        _ => false,
    }
}

/// Handles table output with context-aware formatting.
///
/// This function wraps `handle_table` with list indentation and whitespace management logic.
/// It handles two distinct contexts:
///
/// 1. **List Context** (`ctx.in_list_item = true`):
///    - Indents table content using `indent_table_for_list` for proper nesting
///    - Preserves special caption formatting (lines starting with `*`)
///    - Adds newlines for proper list item separation
///
/// 2. **Normal Context**:
///    - Ensures proper spacing with double newlines around tables
///    - Handles various output state scenarios (empty, ends with newline, etc.)
///
/// # Arguments
/// * `node_handle` - The table node handle
/// * `parser` - The HTML parser instance
/// * `output` - The output string being built
/// * `options` - Conversion options (includes list indent type)
/// * `ctx` - Conversion context (includes list state)
/// * `dom_ctx` - DOM context for tree structure info
/// * `depth` - Current nesting depth
pub fn handle_table_with_context(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let mut table_output = String::new();
    builder::handle_table(node_handle, parser, &mut table_output, handler);

    if let Some(ref sc) = handler.ctx.structure_collector {
        if let Some(grid) = collect_table_grid(node_handle, parser, handler) {
            sc.borrow_mut().push_table_data(grid, table_output.trim().to_string());
        }
    }

    // ~keep A table that rendered to nothing (a blank layout spacer, an empty `<table>`)
    // ~keep has nothing to separate: the blank line and trailing newline below are for
    // ~keep content. Emitting them anyway split a layout row's list item in two around a
    // ~keep bgcolor bar (`<td><img><table>...</table><br>links</td>`, gh-121 Hacker News).
    if table_output.is_empty() {
        return;
    }

    if handler.ctx.in_list_item {
        let has_caption = table_output.starts_with('*');

        if !has_caption {
            use crate::converter::main_helpers::trim_trailing_whitespace;
            trim_trailing_whitespace(output);
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
        }

        let indented = layout::indent_table_for_list(
            &table_output,
            handler.ctx.list_depth,
            handler.ctx.list_indent_columns,
            handler.options,
        );
        output.push_str(&indented);
    } else {
        if !output.is_empty() && !output.ends_with("\n\n") {
            if output.ends_with('\n') {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }
        output.push_str(&table_output);
    }

    if !output.ends_with('\n') {
        output.push('\n');
    }
}

/// Collect a [`crate::types::TableGrid`] from the DOM for the structure collector.
///
/// Walks the table's rows and cells, extracting text content and span attributes
/// to build a structured grid representation.
fn collect_table_grid(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    handler: HandlerContext<'_>,
) -> Option<crate::types::TableGrid> {
    let tl::Node::Tag(tag) = node_handle.get(parser)? else {
        return None;
    };

    let mut state = GridState::default();

    let children = tag.children();
    for child_handle in children.top().iter() {
        collect_grid_child(child_handle, parser, handler, &mut state);
    }

    if state.row_index == 0 {
        return None;
    }

    Some(crate::types::TableGrid {
        rows: state.row_index,
        cols: state.max_cols,
        cells: state.grid_cells,
    })
}

#[derive(Default)]
struct GridState {
    cell_handles: Vec<tl::NodeHandle>,
    grid_cells: Vec<crate::types::GridCell>,
    row_index: u32,
    max_cols: u32,
}

fn collect_grid_child(
    child_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    handler: HandlerContext<'_>,
    state: &mut GridState,
) {
    use utils::{is_tag_name, normalized_tag_name};

    let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) else {
        return;
    };
    let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
    match tag_name.as_ref() {
        "thead" | "tbody" | "tfoot" => {
            let is_header_section = tag_name.as_ref() == "thead";
            for row_handle in child_tag.children().top().iter() {
                if is_tag_name(row_handle, parser, handler.dom_ctx, "tr") {
                    collect_grid_row(row_handle, parser, handler, state, is_header_section);
                }
            }
        }
        "tr" | "row" => {
            collect_grid_row(child_handle, parser, handler, state, state.row_index == 0);
        }
        _ => {}
    }
}

/// Process a single table row for grid collection.
///
/// `depth` is the row's own recursion depth; cell content is walked at `depth + 1`.
fn collect_grid_row(
    row_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    handler: HandlerContext<'_>,
    state: &mut GridState,
    is_header_section: bool,
) {
    use cell::{collect_table_cells, get_colspan_rowspan};

    collect_table_cells(row_handle, parser, handler.dom_ctx, &mut state.cell_handles);

    let mut col_index: u32 = 0;
    for cell_handle in &state.cell_handles {
        let is_header = is_header_section
            || handler
                .dom_ctx
                .tag_name_for(*cell_handle, parser)
                .is_some_and(|name| name.as_ref() == "th");

        let mut text = String::new();
        // ~keep This walk runs after `builder::handle_table` has already rendered — and recorded —
        // ~keep the same cells; it exists only to fill the `TableGrid` returned below, which is
        // ~keep built from `text` and not from any collector. The collectors are `Rc`s that
        // ~keep `..ctx.clone()` shares, so without detaching them here `include_document_structure`
        // ~keep alone would add a third recording of every link, image and nested table in a cell —
        // ~keep including into the very `DocumentStructure` this walk exists to build.
        let mut cell_ctx = super::super::Context {
            in_table_cell: true,
            ..handler.ctx.clone()
        };
        #[cfg(feature = "metadata")]
        {
            cell_ctx.metadata_collector = None;
        }
        cell_ctx.structure_collector = None;
        #[cfg(feature = "inline-images")]
        {
            cell_ctx.inline_collector = None;
        }
        if let Some(tl::Node::Tag(cell_tag)) = cell_handle.get(parser) {
            for child_handle in cell_tag.children().top().iter() {
                super::super::walk_node(
                    child_handle,
                    parser,
                    &mut text,
                    crate::converter::block::container::HandlerContext::new(
                        handler.options,
                        &cell_ctx,
                        handler.depth + 2,
                        handler.dom_ctx,
                    ),
                );
            }
        }
        let content = crate::text::normalize_whitespace_cow(&text).trim().to_string();

        let (colspan, rowspan) = get_colspan_rowspan(cell_handle, parser);

        state.grid_cells.push(crate::types::GridCell {
            content,
            row: state.row_index,
            col: col_index,
            row_span: rowspan as u32,
            col_span: colspan as u32,
            is_header,
        });

        col_index += colspan as u32;
    }
    state.max_cols = state.max_cols.max(col_index);
    state.row_index += 1;
}
