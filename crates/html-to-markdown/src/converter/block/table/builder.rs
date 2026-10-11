//! Core table building and structure calculation.
//!
//! Handles main table element conversion, column calculation, and visitor
//! pattern integration for custom table handling.

use std::borrow::Cow;

use super::cell::{collect_table_cells, get_colspan};
use super::cells::{CellTextCache, RowEnv, RowRender, append_layout_row, collect_row_cell_widths, convert_table_row};
use super::scanner::{TableScan, scan_table};
use super::utils::{is_tag_name, normalized_tag_name};
use crate::converter::block::container::HandlerContext;

/// Return the content cell of a one-cell layout wrapper containing a nested table.
/// Stop at that cell, so nested data headers do not give the wrapper table semantics.
fn nested_table_wrapper_cell(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    scan: &TableScan,
) -> Option<(tl::NodeHandle, usize)> {
    if scan.row_counts != [1] || scan.nested_table_count == 0 || scan.has_span || !scan.has_text {
        return None;
    }
    let mut cell = None;
    let mut pending: Vec<_> = tag.children().top().iter().map(|handle| (*handle, 1)).collect();
    while let Some((handle, depth)) = pending.pop() {
        let Some(tl::Node::Tag(child)) = handle.get(parser) else {
            continue;
        };
        match child.name().as_utf8_str().to_ascii_lowercase().as_str() {
            "td" if cell.is_none() => cell = Some((handle, depth)),
            "thead" | "tbody" | "tfoot" | "tr" => {
                pending.extend(child.children().top().iter().map(|handle| (*handle, depth + 1)));
            }
            _ => return None,
        }
    }
    cell
}

/// Maximum allowed table columns to prevent unbounded memory usage.
const MAX_TABLE_COLS: usize = 1000;

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
/// Maximum column count (minimum 1, maximum `MAX_TABLE_COLS`)
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn table_total_columns(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
) -> usize {
    let mut max_cols = 0usize;
    let mut cells = Vec::new();

    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        for child_handle in tag.children().top().iter() {
            max_cols = max_cols.max(table_child_columns(child_handle, parser, dom_ctx, &mut cells));
        }
    }

    max_cols.clamp(1, MAX_TABLE_COLS)
}

fn table_child_columns(
    child_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
    cells: &mut Vec<tl::NodeHandle>,
) -> usize {
    let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) else {
        return 0;
    };
    let tag_name = dom_ctx
        .tag_name_for(*child_handle, parser)
        .unwrap_or_else(|| normalized_tag_name(child_tag.name().as_utf8_str()));
    match tag_name.as_ref() {
        "thead" | "tbody" | "tfoot" => child_tag
            .children()
            .top()
            .iter()
            .filter(|row_handle| is_tag_name(row_handle, parser, dom_ctx, "tr"))
            .map(|row_handle| row_columns(row_handle, parser, dom_ctx, cells))
            .max()
            .unwrap_or(0),
        "tr" | "row" => row_columns(child_handle, parser, dom_ctx, cells),
        _ => 0,
    }
}

fn row_columns(
    row_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
    cells: &mut Vec<tl::NodeHandle>,
) -> usize {
    collect_table_cells(row_handle, parser, dom_ctx, cells);
    cells.iter().fold(0usize, |count, handle| {
        count.saturating_add(get_colspan(handle, parser))
    })
}

/// Whether each cell's markdown may be rendered once in the width pre-pass and reused
/// verbatim in the render pass.
///
/// ~keep The pre-pass runs with `measure_width_only` (a nested `<table>` degrades to raw text,
/// ~keep issue #406) and `skip_visitor_hooks`, so its rendering equals the render pass only
/// ~keep when the table holds no nested table and no visitor is installed.
///
/// ~keep A collector shared through the context also observes the walk, and two of them feed
/// ~keep back into the emitted bytes or into a caller-visible document model whose entries are
/// ~keep positional: the reference collector numbers `[n]` link labels that appear in the
/// ~keep output, and the structure/inline-image collectors index what they see. Those keep the
/// ~keep two-pass path. The metadata collector does not affect emitted bytes, so reuse stays on
/// ~keep for it — its entries stop being recorded twice per table cell, which is the double
/// ~keep walk's bug, not a behaviour worth preserving.
const fn cell_text_reuse_allowed(ctx: &super::super::super::Context, table_scan: &TableScan) -> bool {
    if table_scan.nested_table_count > 0 {
        return false;
    }
    if ctx.structure_collector.is_some() || ctx.reference_collector.is_some() {
        return false;
    }
    #[cfg(feature = "inline-images")]
    if ctx.inline_collector.is_some() {
        return false;
    }
    #[cfg(feature = "visitor")]
    if ctx.visitor.is_some() {
        return false;
    }
    true
}

/// ~keep Ragged row widths alone do not imply layout: the regular renderer pads them. Layout
/// ~keep requires stronger evidence such as nested tables, borderless spans, blanks, or dense links.
fn is_layout_table(tag: &tl::HTMLTag<'_>, scan: &TableScan, wrapper_cell: Option<(tl::NodeHandle, usize)>) -> bool {
    if wrapper_cell.is_some() || scan.has_header || scan.has_caption {
        return false;
    }
    let has_border_zero = tag
        .attributes()
        .get("border")
        .is_some_and(|value| value.as_ref().is_some_and(|border| border.as_utf8_str() == "0"));
    let looks_like_layout = scan.nested_table_count > 1 || (scan.has_span && has_border_zero);
    looks_like_layout || !scan.has_text || (scan.row_counts.len() <= 2 && scan.link_count >= 3)
}

fn render_layout_table(tag: &tl::HTMLTag<'_>, parser: &tl::Parser, output: &mut String, handler: HandlerContext<'_>) {
    for child_handle in tag.children().top().iter() {
        render_layout_child(child_handle, parser, output, handler);
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }
}

fn render_layout_child(
    child_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) else {
        return;
    };
    let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
    let env = RowEnv {
        parser,
        options: handler.options,
        ctx: handler.ctx,
        dom_ctx: handler.dom_ctx,
    };
    match tag_name.as_ref() {
        "thead" | "tbody" | "tfoot" => {
            for row_handle in child_tag.children().top().iter() {
                if is_tag_name(row_handle, parser, handler.dom_ctx, "tr") {
                    append_layout_row(row_handle, output, env, handler.depth + 1);
                }
            }
        }
        "tr" | "row" => append_layout_row(child_handle, output, env, handler.depth + 1),
        "colgroup" | "col" => {}
        _ => super::super::super::walk_node(
            child_handle,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                handler.ctx,
                handler.depth + 1,
                handler.dom_ctx,
            ),
        ),
    }
}

fn render_wrapper_cell(
    cell_handle: tl::NodeHandle,
    cell_depth: usize,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let Some(tl::Node::Tag(cell)) = cell_handle.get(parser) else {
        return;
    };
    for child in cell.children().top().iter() {
        super::super::super::walk_node(
            child,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                handler.ctx,
                handler.depth + cell_depth + 1,
                handler.dom_ctx,
            ),
        );
    }
}

struct DataTableState {
    row_index: usize,
    total_cols: usize,
    rowspan_tracker: Vec<Option<usize>>,
    cell_cache: CellTextCache,
    deferred_tables: Vec<String>,
    col_widths: Vec<usize>,
}

#[derive(Clone, Copy)]
struct DataEnv<'a> {
    parser: &'a tl::Parser<'a>,
    handler: HandlerContext<'a>,
    scan: &'a TableScan,
}

fn render_data_table(
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
    scan: &TableScan,
) {
    let total_cols = table_total_columns(node_handle, parser, handler.dom_ctx);
    let reuse_cell_text = !handler.options.compact_tables && cell_text_reuse_allowed(handler.ctx, scan);
    let mut cell_cache = CellTextCache::new(reuse_cell_text);
    let col_widths = column_widths(tag, parser, handler, total_cols, reuse_cell_text, &mut cell_cache);
    let mut state = DataTableState {
        row_index: 0,
        total_cols,
        rowspan_tracker: vec![None; total_cols],
        cell_cache,
        deferred_tables: Vec::new(),
        col_widths,
    };
    let env = DataEnv { parser, handler, scan };
    for child_handle in tag.children().top().iter() {
        render_data_child(child_handle, output, env, &mut state);
    }
    append_deferred_tables(output, &state.deferred_tables);
}

/// ~keep Width measurement is an internal walk. When rendered text cannot be reused, collectors
/// ~keep are detached so the pre-pass cannot duplicate caller-visible metadata or structure.
fn column_widths(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    handler: HandlerContext<'_>,
    total_cols: usize,
    reuse_cell_text: bool,
    cell_cache: &mut CellTextCache,
) -> Vec<usize> {
    if handler.options.compact_tables {
        return Vec::new();
    }
    let mut prepass_ctx = super::super::super::Context {
        skip_visitor_hooks: true,
        measure_width_only: true,
        ..handler.ctx.clone()
    };
    if !reuse_cell_text {
        #[cfg(feature = "metadata")]
        {
            prepass_ctx.metadata_collector = None;
        }
        prepass_ctx.structure_collector = None;
        #[cfg(feature = "inline-images")]
        {
            prepass_ctx.inline_collector = None;
        }
    }
    let env = RowEnv {
        parser,
        options: handler.options,
        ctx: &prepass_ctx,
        dom_ctx: handler.dom_ctx,
    };
    let mut state = WidthState {
        widths: Vec::new(),
        rowspan: vec![None; total_cols],
        cell_cache,
    };
    for child_handle in tag.children().top().iter() {
        collect_width_child(child_handle, env, handler.depth + 1, &mut state);
    }
    state.widths
}

struct WidthState<'a> {
    widths: Vec<usize>,
    rowspan: Vec<Option<usize>>,
    cell_cache: &'a mut CellTextCache,
}

fn collect_width_child(child_handle: &tl::NodeHandle, env: RowEnv<'_>, depth: usize, state: &mut WidthState<'_>) {
    let Some(tl::Node::Tag(child_tag)) = child_handle.get(env.parser) else {
        return;
    };
    let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
    match tag_name.as_ref() {
        "thead" | "tbody" | "tfoot" => {
            for row_handle in child_tag.children().top().iter() {
                if is_tag_name(row_handle, env.parser, env.dom_ctx, "tr") {
                    collect_width_row(row_handle, env, depth, state);
                }
            }
        }
        "tr" | "row" => collect_width_row(child_handle, env, depth, state),
        _ => {}
    }
}

fn collect_width_row(row_handle: &tl::NodeHandle, env: RowEnv<'_>, depth: usize, state: &mut WidthState<'_>) {
    collect_row_cell_widths(
        row_handle,
        env,
        &mut state.widths,
        &mut state.rowspan,
        state.cell_cache,
        depth,
    );
}

fn render_data_child(child_handle: &tl::NodeHandle, output: &mut String, env: DataEnv<'_>, state: &mut DataTableState) {
    let Some(tl::Node::Tag(child_tag)) = child_handle.get(env.parser) else {
        return;
    };
    let tag_name = env
        .handler
        .dom_ctx
        .tag_info(child_handle.get_inner(), env.parser)
        .map_or_else(
            || normalized_tag_name(child_tag.name().as_utf8_str()).into_owned().into(),
            |info| Cow::Borrowed(info.name.as_str()),
        );
    match tag_name.as_ref() {
        "caption" => render_caption(child_tag, output, env),
        "thead" | "tbody" | "tfoot" => {
            let is_header = tag_name.as_ref() == "thead";
            for row_handle in child_tag.children().top().iter() {
                if is_tag_name(row_handle, env.parser, env.handler.dom_ctx, "tr") {
                    render_data_row(row_handle, output, env, state, is_header);
                }
            }
        }
        "tr" | "row" => render_data_row(child_handle, output, env, state, state.row_index == 0),
        "colgroup" | "col" => {}
        _ => super::super::super::walk_node(
            child_handle,
            env.parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                env.handler.options,
                env.handler.ctx,
                env.handler.depth + 1,
                env.handler.dom_ctx,
            ),
        ),
    }
}

fn render_caption(tag: &tl::HTMLTag<'_>, output: &mut String, env: DataEnv<'_>) {
    let caption_ctx = super::super::super::Context {
        text_in_markers: true,
        escapes_hyphens: true,
        ..env.handler.ctx.clone()
    };
    let mut text = String::new();
    for child_handle in tag.children().top().iter() {
        super::super::super::walk_node(
            child_handle,
            env.parser,
            &mut text,
            crate::converter::block::container::HandlerContext::new(
                env.handler.options,
                &caption_ctx,
                env.handler.depth + 1,
                env.handler.dom_ctx,
            ),
        );
    }
    let text = text.trim();
    if !text.is_empty() {
        output.push('*');
        output.push_str(&text.replace('-', r"\-"));
        output.push_str("*\n\n");
    }
}

fn render_data_row(
    row_handle: &tl::NodeHandle,
    output: &mut String,
    env: DataEnv<'_>,
    state: &mut DataTableState,
    is_header: bool,
) {
    let emitted = convert_table_row(
        row_handle,
        output,
        RowEnv {
            parser: env.parser,
            options: env.handler.options,
            ctx: env.handler.ctx,
            dom_ctx: env.handler.dom_ctx,
        },
        &mut RowRender {
            row_index: state.row_index,
            has_span: env.scan.has_span,
            rowspan_tracker: &mut state.rowspan_tracker,
            total_cols: state.total_cols,
            header_cols: state.total_cols,
            depth: env.handler.depth + 1,
            is_header,
            col_widths: &state.col_widths,
            cell_cache: &mut state.cell_cache,
            deferred_tables: &mut state.deferred_tables,
        },
    );
    if emitted {
        state.row_index += 1;
    }
}

/// ~keep GFM cannot nest tables, so a nested table from a single-cell row is emitted as a
/// ~keep separate table after its enclosing table instead of being flattened (issues #469/#484).
fn append_deferred_tables(output: &mut String, deferred_tables: &[String]) {
    for nested in deferred_tables {
        if !output.ends_with('\n') {
            output.push('\n');
        }
        if !output.ends_with("\n\n") {
            output.push('\n');
        }
        output.push_str(nested);
        output.push('\n');
    }
}

#[cfg(feature = "visitor")]
struct TableVisitorState {
    output_start: usize,
    custom_start: Option<String>,
}

#[cfg(feature = "visitor")]
fn begin_table_visit(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    tag: &tl::HTMLTag<'_>,
    handler: HandlerContext<'_>,
) -> Option<TableVisitorState> {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let output_start = output.len();
    let Some(ref visitor_handle) = handler.ctx.visitor else {
        return Some(TableVisitorState {
            output_start,
            custom_start: None,
        });
    };
    let node_id = node_handle.get_inner();
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::Table,
        Cow::Borrowed("table"),
        tag,
        handler.depth,
        handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0),
        handler.dom_ctx.parent_tag_name(node_id, parser).map(Cow::Borrowed),
        false,
    );
    let result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_table_start(&node_ctx);
    match result {
        VisitResult::Continue => Some(TableVisitorState {
            output_start,
            custom_start: None,
        }),
        VisitResult::Custom(custom) => Some(TableVisitorState {
            output_start,
            custom_start: Some(custom),
        }),
        VisitResult::Skip => None,
        VisitResult::Error(err) => {
            if handler.ctx.visitor_error.borrow().is_none() {
                *handler.ctx.visitor_error.borrow_mut() = Some(err);
            }
            None
        }
        VisitResult::PreserveHtml => {
            output.push_str(&super::super::super::serialize_node(node_handle, parser));
            None
        }
    }
}

#[cfg(feature = "visitor")]
fn finish_table_visit(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    tag: &tl::HTMLTag<'_>,
    handler: HandlerContext<'_>,
    state: TableVisitorState,
) {
    use crate::visitor::{NodeContext, NodeType};

    let Some(ref visitor_handle) = handler.ctx.visitor else {
        return;
    };
    let node_id = node_handle.get_inner();
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::Table,
        Cow::Borrowed("table"),
        tag,
        handler.depth,
        handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0),
        handler.dom_ctx.parent_tag_name(node_id, parser).map(Cow::Borrowed),
        false,
    );
    let result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_table_end(&node_ctx, &output[state.output_start..]);
    apply_table_visit_result(result, node_handle, parser, output, handler, state);
}

#[cfg(feature = "visitor")]
fn apply_table_visit_result(
    result: crate::visitor::VisitResult,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
    state: TableVisitorState,
) {
    use crate::visitor::VisitResult;

    match result {
        VisitResult::Continue => {
            if let Some(custom_start) = state.custom_start {
                output.insert_str(state.output_start, &custom_start);
            }
        }
        VisitResult::Custom(custom) => {
            let rows_output = output[state.output_start..].to_string();
            output.truncate(state.output_start);
            if let Some(custom_start) = state.custom_start {
                output.push_str(&custom_start);
            }
            output.push_str(&rows_output);
            output.push_str(&custom);
        }
        VisitResult::Skip => output.truncate(state.output_start),
        VisitResult::Error(err) => {
            if handler.ctx.visitor_error.borrow().is_none() {
                *handler.ctx.visitor_error.borrow_mut() = Some(err);
            }
        }
        VisitResult::PreserveHtml => {
            output.truncate(state.output_start);
            output.push_str(&super::super::super::serialize_node(node_handle, parser));
        }
    }
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
    handler: HandlerContext<'_>,
) {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return;
    };
    #[cfg(feature = "visitor")]
    let Some(visitor_state) = begin_table_visit(node_handle, parser, output, tag, handler) else {
        return;
    };
    if handler.ctx.in_table_cell && !handler.ctx.in_layout_cell && !handler.ctx.allow_nested_table_markup {
        super::flatten::render(tag, parser, output, handler);
        #[cfg(feature = "visitor")]
        finish_table_visit(node_handle, parser, output, tag, handler, visitor_state);
        return;
    }
    let table_scan = scan_table(node_handle, parser, handler.dom_ctx, handler.options.br_in_tables);
    let wrapper_cell = nested_table_wrapper_cell(tag, parser, &table_scan).filter(|(_, cell_depth)| {
        handler.depth + cell_depth + 1 < crate::converter::main_helpers::effective_max_depth(handler.options)
    });
    if is_layout_table(tag, &table_scan, wrapper_cell) {
        if table_scan.has_text || table_scan.link_count > 0 {
            render_layout_table(tag, parser, output, handler);
        }
        return;
    }
    if let Some((cell_handle, cell_depth)) = wrapper_cell {
        render_wrapper_cell(cell_handle, cell_depth, parser, output, handler);
    } else {
        render_data_table(node_handle, tag, parser, output, handler, &table_scan);
    }
    #[cfg(feature = "visitor")]
    finish_table_visit(node_handle, parser, output, tag, handler, visitor_state);
}

#[cfg(test)]
mod tests {
    #[test]
    fn single_nested_table_stays_as_table() {
        let html = r"<table><tr><td>Label</td><td><table><tr><td>A</td><td>B</td></tr></table></td></tr></table>";
        let result = crate::convert(html, None).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(content.contains('|'), "should produce pipe table, not list");
    }

    /// Regression test for issue #406: deeply-nested layout tables caused
    /// exponential output growth because `cell_text_content` recursed into
    /// nested `<table>` elements (including their separator rows) when
    /// computing outer column widths.  With 5 levels of nesting the output
    /// must stay well under 10 KB.
    #[test]
    fn deeply_nested_layout_tables_do_not_produce_runaway_output() {
        // ~keep Build a 5-level-deep nested layout table.
        // ~keep Each cell contains another complete table, mirroring the structure
        // ~keep seen in email digests that triggered the regression.
        let inner = "<table><tr><td>leaf</td></tr></table>";
        let level1 = format!("<table><tr><td>{inner}</td></tr></table>");
        let level2 = format!("<table><tr><td>{level1}</td></tr></table>");
        let level3 = format!("<table><tr><td>{level2}</td></tr></table>");
        let level4 = format!("<table><tr><td>{level3}</td></tr></table>");
        let level5 = format!("<table><tr><td>{level4}</td></tr></table>");

        let result = crate::convert(&level5, None).unwrap();
        let content = result.content.unwrap_or_default();

        const MAX_BYTES: usize = 10_000;
        assert!(
            content.len() < MAX_BYTES,
            "nested layout table output should be < {MAX_BYTES} bytes, got {} bytes (issue #406 regression)",
            content.len()
        );
    }
}
