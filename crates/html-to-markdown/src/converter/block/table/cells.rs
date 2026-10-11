//! Cell and row handling for Markdown conversion.
//!
//! Provides functionality for processing table cells and rows, including:
//! - Row conversion to Markdown format
//! - Cell layout handling with colspan/rowspan support
//! - Layout table row conversion to list items

use crate::converter::block::container::HandlerContext;
use crate::converter::utility::content::normalized_tag_name;
use std::borrow::Cow;
use std::collections::HashMap;

use super::cell::{cell_text_content, collect_table_cells, convert_table_cell, emit_cell_text, get_colspan_rowspan};

/// Maximum allowed table columns to prevent unbounded memory usage.
const MAX_TABLE_COLS: usize = 1000;

/// Rendered markdown of each cell visited by the column-width pre-pass, keyed by DOM node id.
///
/// ~keep The pre-pass and the render pass walk the same cell subtrees, so the pre-pass result
/// ~keep can be emitted directly instead of rendering twice — but only where the two passes are
/// ~keep provably byte-identical. The pre-pass context sets `measure_width_only` (nested tables
/// ~keep degrade to raw text, issue #406) and `skip_visitor_hooks`, and a second walk is
/// ~keep observable through any collector handle shared via the context. The caller decides via
/// ~keep `enabled` (see `builder::cell_text_reuse_allowed`); when disabled this stores nothing.
pub struct CellTextCache {
    entries: HashMap<u32, String>,
    enabled: bool,
}

impl CellTextCache {
    /// Create a cache; `enabled == false` makes every store a no-op and every lookup a miss.
    pub fn new(enabled: bool) -> Self {
        Self {
            entries: HashMap::new(),
            enabled,
        }
    }

    fn store(&mut self, node_id: u32, text: String) {
        if self.enabled {
            self.entries.insert(node_id, text);
        }
    }

    fn take(&mut self, node_id: u32) -> Option<String> {
        if self.enabled {
            self.entries.remove(&node_id)
        } else {
            None
        }
    }
}

/// Read-only conversion inputs threaded together through row/cell rendering: the parser, user
/// options, conversion `Context`, and DOM lookup context.
///
/// ~keep These four are always passed as a group across this module's row-rendering functions;
/// ~keep bundling them keeps each function's own signature down to its row/cell-specific
/// ~keep parameters instead of repeating all four everywhere (too-many-parameters).
#[derive(Clone, Copy)]
pub struct RowEnv<'p> {
    pub parser: &'p tl::Parser<'p>,
    pub options: &'p crate::options::ConversionOptions,
    pub ctx: &'p super::super::super::Context,
    pub dom_ctx: &'p super::super::super::DomContext,
}

/// Append one layout-table cell's converted text to `row_text`, space-joining after the first
/// non-empty cell.
///
/// Extracted from `append_layout_row`'s per-cell loop body — identical cell-name check,
/// inline-image handling, and whitespace trim, unchanged.
fn append_layout_cell_text(cell_handle: &tl::NodeHandle, row_text: &mut String, env: RowEnv<'_>, depth: usize) {
    let Some(tl::Node::Tag(cell_tag)) = cell_handle.get(env.parser) else {
        return;
    };
    let cell_name: Cow<'_, str> = env.dom_ctx.tag_info(cell_handle.get_inner(), env.parser).map_or_else(
        || normalized_tag_name(cell_tag.name().as_utf8_str()).into_owned().into(),
        |info| Cow::Borrowed(info.name.as_str()),
    );
    if !matches!(cell_name.as_ref(), "td" | "th" | "cell") {
        return;
    }

    let mut cell_text = String::new();
    // ~keep issue #433/#500: a layout row renders as a list item, and list items keep inline
    // ~keep images by default (only headings degrade an image to its alt text), so this cell
    // ~keep always allows them regardless of `keep_inline_images_in`. That option still governs
    // ~keep image handling elsewhere; it is simply not needed to keep images in a layout cell.
    let cell_allow_inline_images = true;
    let cell_ctx = super::super::super::Context {
        convert_as_inline: true,
        in_layout_cell: true,
        cell_allow_inline_images,
        in_cell_of_inputs: super::scanner::cell_holds_only_inputs(
            *cell_handle,
            env.parser,
            env.dom_ctx,
            env.options.br_in_tables,
        ),
        ..env.ctx.clone()
    };
    let cell_children = cell_tag.children();
    for cell_child in cell_children.top().iter() {
        super::super::super::walk_node(
            cell_child,
            env.parser,
            &mut cell_text,
            crate::converter::block::container::HandlerContext::new(env.options, &cell_ctx, depth + 1, env.dom_ctx),
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

/// Append a layout table row as a list item.
///
/// For tables used for visual layout, converts rows to list items
/// instead of table format for better readability.
///
/// # Arguments
/// * `row_handle` - Handle to the row element
/// * `output` - Mutable string to append content
/// * `env` - Parser, options, conversion context, and DOM context
/// * `depth` - Current recursion depth (the row's own depth; cell content is walked at `depth + 1`)
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn append_layout_row(row_handle: &tl::NodeHandle, output: &mut String, env: RowEnv<'_>, depth: usize) {
    let Some(tl::Node::Tag(row_tag)) = row_handle.get(env.parser) else {
        return;
    };
    let mut row_text = String::new();
    let row_children = row_tag.children();
    for cell_handle in row_children.top().iter() {
        append_layout_cell_text(cell_handle, &mut row_text, env, depth);
    }

    let trimmed = row_text.trim();
    if !trimmed.is_empty() {
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        let marker = layout_row_bullet(env.options, env.ctx);
        let formatted = strip_leading_bullet(trimmed, env.options);
        output.push(marker);
        output.push(' ');
        output.push_str(formatted);
        output.push('\n');
    }
}

/// The list marker a layout row renders with.
///
/// A layout table's rows are list items, so they cycle through `options.bullets` by nesting
/// depth exactly as `list::item` does. The row sits one level deeper than its surroundings --
/// `ul_depth` is still the *enclosing* depth here, where `list::item` has already counted its
/// own `<ul>` -- so the index is `ul_depth` rather than `ul_depth - 1`. A layout table nested
/// in a list therefore takes the next marker instead of repeating its parent's (issue #472).
fn layout_row_bullet(options: &crate::options::ConversionOptions, ctx: &super::super::super::Context) -> char {
    let bullets: Vec<char> = options.bullets.chars().collect();
    if bullets.is_empty() {
        return '*';
    }
    bullets[ctx.ul_depth % bullets.len()]
}

/// Drop a leading list marker the cell content already produced, so the row's own marker is not
/// doubled up. Any configured bullet counts, not just `-`: the marker being stripped was written
/// by this same cycling rule, so hardcoding one character missed every other configured set.
fn strip_leading_bullet<'a>(trimmed: &'a str, options: &crate::options::ConversionOptions) -> &'a str {
    for bullet in options.bullets.chars().chain(['-', '*', '+']) {
        let mut marker = String::with_capacity(2);
        marker.push(bullet);
        marker.push(' ');
        if let Some(rest) = trimmed.strip_prefix(marker.as_str()) {
            return rest.trim_start();
        }
    }
    trimmed
}

/// Advance `col` past any columns whose rowspan tracker still has rows remaining, decrementing
/// each as it is skipped and clearing the tracker entry once exhausted.
///
/// Extracted from `collect_row_cell_widths`'s inner loop — identical decrement-and-clear logic,
/// unchanged.
fn skip_rowspan_columns(col: &mut usize, rowspan_tracker: &mut [Option<usize>]) {
    while *col < rowspan_tracker.len() {
        let Some(Some(remaining)) = rowspan_tracker.get_mut(*col) else {
            break;
        };
        if *remaining == 0 {
            break;
        }
        *remaining -= 1;
        if *remaining == 0 {
            rowspan_tracker[*col] = None;
        }
        *col += 1;
    }
}

/// Collect the rendered text content of every cell in a row for width calculation.
///
/// `rowspan_tracker` mirrors the tracker in `convert_table_row` so that spanned
/// columns are skipped in the width pre-pass just as they are skipped in rendering.
/// Pass a shared tracker across all row calls to correctly handle multi-row spans.
///
/// `depth` is the row's own recursion depth; cell content is measured at `depth + 1`.
///
/// # Arguments
/// * `node_handle` - Handle to the row element
/// * `env` - Parser, options, conversion context, and DOM context
/// * `col_widths` - Per-column max content widths accumulated so far
/// * `rowspan_tracker` - Mutable array tracking rowspan remainder for each column
/// * `cell_cache` - Rendered markdown to store for later reuse by the render pass
/// * `depth` - Row's own recursion depth
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn collect_row_cell_widths(
    node_handle: &tl::NodeHandle,
    env: RowEnv<'_>,
    col_widths: &mut Vec<usize>,
    rowspan_tracker: &mut Vec<Option<usize>>,
    cell_cache: &mut CellTextCache,
    depth: usize,
) {
    let mut cells = Vec::new();
    collect_table_cells(node_handle, env.parser, env.dom_ctx, &mut cells);

    let mut col = 0usize;
    let mut cell_iter = cells.iter();

    loop {
        skip_rowspan_columns(&mut col, rowspan_tracker);

        let Some(cell_handle) = cell_iter.next() else {
            break;
        };

        let text = cell_text_content(cell_handle, env.parser, env.options, env.ctx, env.dom_ctx, depth + 1);
        const MAX_CELL_WIDTH: usize = 200;
        let width = text.chars().count().min(MAX_CELL_WIDTH);
        cell_cache.store(cell_handle.get_inner(), text);

        if col >= col_widths.len() {
            col_widths.resize(col + 1, 0);
        }
        if width > col_widths[col] {
            col_widths[col] = width;
        }

        let (colspan, rowspan) = get_colspan_rowspan(cell_handle, env.parser);

        if rowspan > 1 {
            if col >= rowspan_tracker.len() {
                rowspan_tracker.resize(col + 1, None);
            }
            rowspan_tracker[col] = Some(rowspan - 1);
        }

        col = col.saturating_add(colspan);
    }
}

/// Emit one cell of a rendered row, reusing the pre-pass rendering when it is cached.
///
/// `env.ctx` is used as the cell's own context (already carrying `in_table_cell = true`).
/// `depth` is the cell's own recursion depth. `deferred_tables` is forwarded to
/// [`convert_table_cell`]; see [`super::cell::render_cell_text`] for what it does.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn emit_row_cell(
    cell_handle: &tl::NodeHandle,
    row_text: &mut String,
    env: RowEnv<'_>,
    emission: CellEmission,
    cell_cache: &mut CellTextCache,
    deferred_tables: Option<&mut Vec<String>>,
) {
    if let Some(text) = cell_cache.take(cell_handle.get_inner()) {
        emit_cell_text(cell_handle, env.parser, row_text, &text, emission.col_width);
    } else {
        convert_table_cell(
            cell_handle,
            env.parser,
            row_text,
            HandlerContext::new(env.options, env.ctx, emission.depth, env.dom_ctx),
            emission.col_width,
            deferred_tables,
        );
    }
}

#[derive(Clone, Copy)]
struct CellEmission {
    col_width: Option<usize>,
    depth: usize,
}

/// Minimum separator dash count per column (matches `---`).
const MIN_SEPARATOR_DASHES: usize = 3;

/// If `col_index` still has pending rowspan rows, emit its blank continuation cell (padded to
/// `col_widths`) into `row_text` and advance `col_index` past it.
///
/// Returns `true` when a continuation cell was emitted (caller should retry from the top of the
/// loop) and `false` when `col_index` is not a pending rowspan continuation.
///
/// Extracted from `convert_table_row`'s has-span loop — identical padding and rowspan-tracker
/// bookkeeping, unchanged.
fn emit_rowspan_continuation(
    col_index: &mut usize,
    total_cols: usize,
    rowspan_tracker: &mut [Option<usize>],
    col_widths: &[usize],
    row_text: &mut String,
) -> bool {
    if *col_index >= total_cols {
        return false;
    }
    let Some(Some(remaining_rows)) = rowspan_tracker.get_mut(*col_index) else {
        return false;
    };
    if *remaining_rows == 0 {
        return false;
    }

    let width = col_widths.get(*col_index).copied();
    row_text.push(' ');
    if let Some(w) = width {
        for _ in 0..w {
            row_text.push(' ');
        }
    }
    row_text.push_str(" |");
    *remaining_rows -= 1;
    if *remaining_rows == 0 {
        rowspan_tracker[*col_index] = None;
    }
    *col_index += 1;
    true
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
/// * `col_widths` - Per-column max content widths for padding (empty = no padding)
/// * `cell_cache` - Markdown already rendered for these cells by the width pre-pass
/// * `deferred_tables` - Collects a nested table's markdown when this row's sole cell holds
///   one; see [`super::cell::render_cell_text`]. The caller renders these separately, after the
///   enclosing table, once every row has been processed (issue #484).
///
/// # Returns
/// `false` when the row collected zero cells (nothing was emitted); `true` otherwise. Callers
/// must only advance their own row counter when this returns `true` (issue #489).
/// Run the row-level visitor hook, if one is registered.
///
/// Returns `Some(bool)` when the visitor decided the row is fully handled and
/// `convert_table_row` must return that value immediately; `None` when rendering should
/// continue as normal. Split out of `convert_table_row` to keep that function under the
/// cyclomatic-complexity gate.
#[cfg(feature = "visitor")]
fn run_row_visitor_hook(node_handle: &tl::NodeHandle, output: &mut String, visit: RowVisit<'_>) -> Option<bool> {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let visitor_handle = visit.env.ctx.visitor.as_ref()?;
    let cell_contents = visitor_cell_contents(visit);
    let tl::Node::Tag(tag) = node_handle.get(visit.env.parser)? else {
        return None;
    };

    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::TableRow,
        Cow::Borrowed("tr"),
        tag,
        visit.depth,
        visit.row_index,
        Some(Cow::Borrowed("table")),
        false,
    );

    let visit_result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        visitor.visit_table_row(&node_ctx, &cell_contents, visit.is_header)
    };

    match visit_result {
        VisitResult::Continue => None,
        // ~keep Pre-existing visitor early returns, unrelated to issue #489: `true`
        // ~keep preserves prior behavior of always advancing `row_index` here.
        VisitResult::Skip => Some(true),
        VisitResult::Custom(custom) => {
            output.push_str(&custom);
            Some(true)
        }
        VisitResult::Error(err) => {
            if visit.env.ctx.visitor_error.borrow().is_none() {
                *visit.env.ctx.visitor_error.borrow_mut() = Some(err);
            }
            Some(true)
        }
        VisitResult::PreserveHtml => {
            output.push_str(&super::super::super::serialize_node(node_handle, visit.env.parser));
            Some(true)
        }
    }
}

#[cfg(feature = "visitor")]
#[derive(Clone, Copy)]
struct RowVisit<'a> {
    env: RowEnv<'a>,
    cells: &'a [tl::NodeHandle],
    row_index: usize,
    is_header: bool,
    depth: usize,
}

#[cfg(feature = "visitor")]
/// ~keep This visitor-only walk feeds the callback before the render pass walks each cell again.
/// ~keep Detaching shared collectors prevents duplicate metadata and structure entries.
fn visitor_cell_contents(visit: RowVisit<'_>) -> Vec<String> {
    let mut collect_ctx = super::super::super::Context {
        in_table_cell: true,
        ..visit.env.ctx.clone()
    };
    #[cfg(feature = "metadata")]
    {
        collect_ctx.metadata_collector = None;
    }
    collect_ctx.structure_collector = None;
    #[cfg(feature = "inline-images")]
    {
        collect_ctx.inline_collector = None;
    }
    visit
        .cells
        .iter()
        .map(|cell_handle| visitor_cell_text(cell_handle, visit.env, &collect_ctx, visit.depth))
        .collect()
}

#[cfg(feature = "visitor")]
fn visitor_cell_text(
    cell_handle: &tl::NodeHandle,
    env: RowEnv<'_>,
    collect_ctx: &super::super::super::Context,
    depth: usize,
) -> String {
    let mut text = String::new();
    if let Some(tl::Node::Tag(tag)) = cell_handle.get(env.parser) {
        let cell_ctx = collect_ctx.for_cell(*cell_handle, env.parser, env.dom_ctx, env.options.br_in_tables);
        for child_handle in tag.children().top().iter() {
            super::super::super::walk_node(
                child_handle,
                env.parser,
                &mut text,
                crate::converter::block::container::HandlerContext::new(env.options, &cell_ctx, depth + 1, env.dom_ctx),
            );
        }
    }
    crate::text::normalize_whitespace_cow(&text).trim().to_string()
}

#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn convert_table_row(
    node_handle: &tl::NodeHandle,
    output: &mut String,
    env: RowEnv<'_>,
    render: &mut RowRender<'_>,
) -> bool {
    let mut row_text = String::with_capacity(256);
    let mut cells = Vec::new();

    collect_table_cells(node_handle, env.parser, env.dom_ctx, &mut cells);
    // ~keep A nested table may only be deferred out of a cell that shares its row with no
    // ~keep other cell -- pulling it out of a row with a sibling would leave that sibling's
    // ~keep column position undefined (issue #469 locks the sibling-cell shape to the
    // ~keep existing flatten-and-escape behavior; issue #484 is the single-cell-row shape).
    let is_single_cell_row = cells.len() == 1;

    // ~keep A row whose only children were non-cell elements (e.g. `tl`'s `read_end`
    // ~keep dropped an unmatched `</table>`, stranding a `<p>` inside this `<tr>`)
    // ~keep collects zero cells here. Bailing before any output lets the *next* real
    // ~keep row become row 0 -- and thus the header -- matching Tier 1's behavior
    // ~keep (issue #489). The caller only advances `row_index` when this returns
    // ~keep `true`, so a skipped row does not consume a row-index slot.
    if cells.is_empty() {
        return false;
    }

    #[cfg(feature = "visitor")]
    if let Some(early_return) = run_row_visitor_hook(
        node_handle,
        output,
        RowVisit {
            env,
            cells: &cells,
            row_index: render.row_index,
            is_header: render.is_header,
            depth: render.depth,
        },
    ) {
        return early_return;
    }

    // ~keep Build the per-cell context once for the entire row.  Tier-2 hot-spot
    // ~keep pass III: avoids cloning `Context` (which holds several Rc<HashSet> and
    // ~keep optional collector handles) on every cell in wikipedia-class tables.
    let cell_ctx = super::super::super::Context {
        in_table_cell: true,
        ..env.ctx.clone()
    };

    let row_env = RowEnv { ctx: &cell_ctx, ..env };
    let mut filled_cols = render_row_cells(&cells, &mut row_text, row_env, render, is_single_cell_row);
    pad_row(&mut row_text, &mut filled_cols, render.total_cols, render.col_widths);
    output.push('|');
    output.push_str(&row_text);
    output.push('\n');
    if render.row_index == 0 {
        emit_header_separator(output, env.options, render.header_cols, render.col_widths);
    }
    true
}

pub struct RowRender<'a> {
    pub row_index: usize,
    pub has_span: bool,
    pub rowspan_tracker: &'a mut [Option<usize>],
    pub total_cols: usize,
    pub header_cols: usize,
    pub depth: usize,
    pub is_header: bool,
    pub col_widths: &'a [usize],
    pub cell_cache: &'a mut CellTextCache,
    pub deferred_tables: &'a mut Vec<String>,
}

fn render_row_cells(
    cells: &[tl::NodeHandle],
    row_text: &mut String,
    env: RowEnv<'_>,
    render: &mut RowRender<'_>,
    is_single_cell_row: bool,
) -> usize {
    if !render.has_span {
        for (cell_index, cell_handle) in cells.iter().enumerate() {
            let deferred = is_single_cell_row.then_some(&mut *render.deferred_tables);
            emit_row_cell(
                cell_handle,
                row_text,
                env,
                CellEmission {
                    col_width: render.col_widths.get(cell_index).copied(),
                    depth: render.depth + 1,
                },
                render.cell_cache,
                deferred,
            );
        }
        return cells.len();
    }
    render_spanned_row(cells, row_text, env, render, is_single_cell_row)
}

fn render_spanned_row(
    cells: &[tl::NodeHandle],
    row_text: &mut String,
    env: RowEnv<'_>,
    render: &mut RowRender<'_>,
    is_single_cell_row: bool,
) -> usize {
    let mut col_index = 0;
    let mut cell_iter = cells.iter();
    loop {
        if emit_rowspan_continuation(
            &mut col_index,
            render.total_cols,
            render.rowspan_tracker,
            render.col_widths,
            row_text,
        ) {
            continue;
        }
        let Some(cell_handle) = cell_iter.next() else {
            break;
        };
        let deferred = is_single_cell_row.then_some(&mut *render.deferred_tables);
        emit_row_cell(
            cell_handle,
            row_text,
            env,
            CellEmission {
                col_width: render.col_widths.get(col_index).copied(),
                depth: render.depth + 1,
            },
            render.cell_cache,
            deferred,
        );
        let (colspan, rowspan) = get_colspan_rowspan(cell_handle, env.parser);
        if rowspan > 1 && col_index < render.total_cols {
            render.rowspan_tracker[col_index] = Some(rowspan - 1);
        }
        col_index = col_index.saturating_add(colspan);
    }
    col_index
}

/// ~keep Ragged rows must still declare the widest row's column count or GFM rejects the table
/// ~keep and renderers can discard cells beyond the delimiter row (issue #13).
fn pad_row(row_text: &mut String, filled_cols: &mut usize, total_cols: usize, col_widths: &[usize]) {
    while *filled_cols < total_cols {
        let width = col_widths.get(*filled_cols).copied();
        row_text.push(' ');
        if let Some(width) = width {
            row_text.extend(std::iter::repeat_n(' ', width));
        }
        row_text.push_str(" |");
        *filled_cols += 1;
    }
}

fn emit_header_separator(
    output: &mut String,
    options: &crate::options::ConversionOptions,
    header_cols: usize,
    col_widths: &[usize],
) {
    let total_cols = header_cols.clamp(1, MAX_TABLE_COLS);
    let is_djot = options.output_format == crate::options::OutputFormat::Djot;
    output.push('|');
    if !is_djot {
        output.push(' ');
    }
    for index in 0..total_cols {
        if index > 0 {
            output.push_str(if is_djot { "|" } else { " | " });
        }
        let dash_count = col_widths.get(index).copied().unwrap_or(0).max(MIN_SEPARATOR_DASHES);
        output.extend(std::iter::repeat_n('-', dash_count));
    }
    if !is_djot {
        output.push(' ');
    }
    output.push_str("|\n");
}
