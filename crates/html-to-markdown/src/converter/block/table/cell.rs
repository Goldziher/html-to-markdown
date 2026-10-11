//! Table cell conversion utilities.
//!
//! Handles conversion of table cell (td/th) elements to Markdown format,
//! including colspan support and content normalization.

use crate::converter::block::container::HandlerContext;

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
/// The colspan value (minimum 1, maximum `MAX_TABLE_COLS`)
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
/// More efficient than calling `get_colspan` and a separate rowspan lookup.
///
/// # Arguments
/// * `node_handle` - Handle to the cell element
/// * `parser` - HTML parser instance
///
/// # Returns
/// A tuple of (colspan, rowspan), both minimum 1 and maximum `MAX_TABLE_COLS`
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

/// Extract the text content of a table cell for column width calculation.
///
/// Returns the same text that would appear in the rendered cell, without
/// the surrounding pipe delimiters. Used in the first pass to compute
/// maximum column widths before rendering with padding.
///
/// # Arguments
/// * `node_handle` - Handle to the cell element
/// * `parser` - HTML parser instance
/// * `options` - Conversion options
/// * `ctx` - Conversion context
/// * `dom_ctx` - DOM context
/// * `depth` - Current recursion depth (the cell's own depth; children are walked at `depth + 1`)
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn cell_text_content(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    options: &crate::options::ConversionOptions,
    ctx: &super::super::super::Context,
    dom_ctx: &super::super::super::DomContext,
    depth: usize,
) -> String {
    let cell_ctx = super::super::super::Context {
        in_table_cell: true,
        ..ctx.clone()
    };

    // ~keep Width measurement never defers a nested table out to a separate block: the
    // ~keep result is discarded once its length is measured (capped at 200 chars anyway),
    // ~keep and deferring here as well as in the render pass would queue it twice (issue #484).
    render_cell_text(
        node_handle,
        parser,
        HandlerContext::new(options, &cell_ctx, depth, dom_ctx),
        None,
    )
}

/// Initial buffer capacity for a rendered cell's markdown.
const CELL_TEXT_CAPACITY: usize = 128;

/// Render a cell's content to the exact text that appears between its pipe delimiters.
///
/// `cell_ctx` must already carry `in_table_cell = true`; the caller owns that decision so
/// the per-row context can be built once instead of cloned per cell.
///
/// `depth` is the cell's own recursion depth; children are walked at `depth + 1`.
///
/// `deferred_tables`, when `Some`, receives a child that is, or wraps (e.g. a `<div>`), a
/// nested `<table>`, rendered as that child's markdown verbatim (trimmed, unescaped,
/// un-flattened) instead of folding it into this cell's single line. The caller passes `Some`
/// only when the enclosing row holds no other cell — GFM has no way to express a real nested
/// table, but a lone cell's content can be lifted out and rendered as its own separate table
/// after the enclosing one, which keeps the inner table usable instead of flattening it into a
/// line of escaped pipes (issue #484). `None` (the default, and always the case for a cell
/// sharing its row with a sibling, per issue #469) keeps the existing flatten-and-escape
/// behavior.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn render_cell_text(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    handler: HandlerContext<'_>,
    mut deferred_tables: Option<&mut Vec<String>>,
) -> String {
    let mut text = String::with_capacity(CELL_TEXT_CAPACITY);
    render_cell_content(node_handle, parser, &mut text, handler, &mut deferred_tables);
    finalize_cell_text(text, handler.options)
}

fn render_cell_content(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    text: &mut String,
    handler: HandlerContext<'_>,
    deferred_tables: &mut Option<&mut Vec<String>>,
) {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return;
    };
    let children = tag.children();
    let has_tag_child = children
        .top()
        .iter()
        .any(|child_handle| matches!(child_handle.get(parser), Some(tl::Node::Tag(_))));
    if !has_tag_child {
        let raw = handler.dom_ctx.text_content(*node_handle, parser);
        let normalized = if handler.options.whitespace_mode == crate::options::WhitespaceMode::Normalized {
            crate::text::normalize_cell_whitespace_cow(raw.as_str())
        } else {
            crate::text::fold_cell_line_breaks_verbatim_cow(raw.as_str())
        };
        *text = escape_cell_text(normalized.as_ref(), handler.options);
        return;
    }
    let cell_ctx = handler
        .ctx
        .for_cell(*node_handle, parser, handler.dom_ctx, handler.options.br_in_tables);
    let handler = HandlerContext::new(handler.options, &cell_ctx, handler.depth, handler.dom_ctx);
    for child_handle in children.top().iter() {
        render_cell_child(child_handle, parser, text, handler, deferred_tables);
    }
}

/// ~keep A nested table's structural pipes bypass text-node escaping. It must either be deferred
/// ~keep from a single-cell row or flattened with escaped pipes and explicit row separators;
/// ~keep otherwise reparsing widens and eventually truncates the outer row (issues #469/#484/#488).
fn render_cell_child(
    child_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    text: &mut String,
    handler: HandlerContext<'_>,
    deferred_tables: &mut Option<&mut Vec<String>>,
) {
    if !super::utils::is_or_contains_table(child_handle, parser, handler.dom_ctx) {
        super::super::super::walk_node(
            child_handle,
            parser,
            text,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                handler.ctx,
                handler.depth + 1,
                handler.dom_ctx,
            ),
        );
        return;
    }
    let mut nested = String::new();
    super::super::super::walk_node(
        child_handle,
        parser,
        &mut nested,
        crate::converter::block::container::HandlerContext::new(
            handler.options,
            handler.ctx,
            handler.depth + 1,
            handler.dom_ctx,
        ),
    );
    if let Some(buf) = deferred_tables.as_deref_mut() {
        let trimmed = nested.trim();
        if !trimmed.is_empty() {
            buf.push(trimmed.to_string());
        }
        return;
    }
    if nested.contains('|') {
        nested = crate::converter::utility::escaping::escape_bare_pipes_outside_code_spans(&nested);
    }
    let nested = fold_nested_table_rows(&nested, handler.options.br_in_tables);
    if !nested.is_empty() && !text.trim_end().is_empty() {
        crate::converter::emit_table_cell_break(text, handler.options.br_in_tables);
    }
    text.push_str(&nested);
}

fn finalize_cell_text(mut text: String, options: &crate::options::ConversionOptions) -> String {
    trim_in_place(&mut text);
    // ~keep Final invariant: a rendered cell never contains a newline, in any mode. This used
    // ~keep to be gated on `!br_in_tables`, which is what let issues #456 and #457 reach the
    // ~keep output — `br_in_tables` selects how a *line break* is represented (`<br>` vs a
    // ~keep space), it does not make a raw newline legal between two pipes. Handlers that know
    // ~keep they are in a cell still emit `<br>` themselves; this only catches what they miss.
    if text.contains('\n') {
        text = text.replace('\n', " ");
    }
    // ~keep A `|` a handler wrote into the cell (a code span, a link destination or title, an
    // ~keep image description, a nested table) splits the row like one in plain text does.
    // ~keep GFM splits on a pipe in a code span too and reads `\|` there as `|`; Djot does not
    // ~keep split inside a verbatim span, and a backslash there stays literal.
    if text.contains('|') {
        match options.output_format {
            crate::options::OutputFormat::Markdown => {
                text = crate::converter::utility::escaping::escape_cell_pipes(&text).into_owned();
            }
            crate::options::OutputFormat::Djot => {
                text = crate::converter::utility::escaping::escape_bare_pipes_outside_code_spans(&text);
            }
            crate::options::OutputFormat::Plain => {}
        }
    }
    text
}

/// Flatten a nested table's rendered rows onto the single line a Markdown cell allows.
///
/// Each row arrives on its own line. `br_in_tables` selects how the boundary between them is
/// spelled — a literal `<br>` when set, a single space otherwise — matching what every other
/// in-cell handler does with a line break (`emit_table_cell_break`, issues #453/#454). Blank
/// lines the inner table emits around itself carry no content and are dropped.
pub fn fold_nested_table_rows(nested: &str, br_in_tables: bool) -> String {
    let separator = if br_in_tables { "<br>" } else { " " };
    nested
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(separator)
}

/// Trim leading and trailing whitespace without reallocating.
fn trim_in_place(text: &mut String) {
    let end = text.trim_end().len();
    text.truncate(end);
    let start = text.len() - text.trim_start().len();
    text.drain(..start);
}

/// Escape text for use inside a table cell.
///
/// Applies every `escape_*` option exactly as outside a table (issue #638), and
/// escapes `|` (pipe) when neither `escape_misc` nor `escape_ascii` already does. Raw
/// backticks are always escaped for Djot because they otherwise consume the table syntax.
fn escape_cell_text(text: &str, options: &crate::options::ConversionOptions) -> String {
    let escaped = crate::text::escape(
        text,
        options.escape_misc,
        options.escape_asterisks,
        options.escape_underscores,
        options.escape_ascii,
    );
    let escaped = if options.escape_misc || options.escape_ascii {
        escaped.into_owned()
    } else {
        escaped.replace('|', r"\|")
    };
    crate::converter::utility::escaping::escape_djot_table_cell_literal(&escaped, options.output_format, true)
        .into_owned()
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
/// * `options` - Conversion options (escape settings, `br_in_tables`)
/// * `ctx` - Conversion context (visitor, etc)
/// * `_tag_name` - Tag name (for consistency, not used)
/// * `dom_ctx` - DOM context for content extraction
/// * `col_width` - Optional target width for padding (None = no padding)
/// * `depth` - Current recursion depth (the cell's own depth; children are walked at `depth + 1`)
/// * `deferred_tables` - See [`render_cell_text`]; forwarded unchanged.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn convert_table_cell(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
    col_width: Option<usize>,
    deferred_tables: Option<&mut Vec<String>>,
) {
    let text = render_cell_text(node_handle, parser, handler, deferred_tables);
    emit_cell_text(node_handle, parser, output, &text, col_width);
}

/// Emit already-rendered cell text with its padding and colspan pipe delimiters.
///
/// Split out from [`convert_table_cell`] so a cell rendered during the column-width
/// pre-pass can be emitted without walking its subtree a second time.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn emit_cell_text(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    text: &str,
    col_width: Option<usize>,
) {
    let colspan = get_colspan(node_handle, parser);

    output.push(' ');
    output.push_str(text);
    if let Some(width) = col_width {
        let text_len = text.chars().count();
        if text_len < width {
            for _ in 0..(width - text_len) {
                output.push(' ');
            }
        }
    }
    for _ in 0..colspan {
        output.push_str(" |");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rich_formatting_preserved_in_cells() {
        let html = "<table><tr><th>H</th></tr><tr><td><strong>Bold</strong> and <em>italic</em></td></tr></table>";
        let result = crate::convert(html, None).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(
            content.contains("**Bold**") || content.contains("__Bold__"),
            "bold should be preserved: {content}"
        );
        assert!(
            content.contains("*italic*") || content.contains("_italic_"),
            "italic should be preserved: {content}"
        );
    }
}
