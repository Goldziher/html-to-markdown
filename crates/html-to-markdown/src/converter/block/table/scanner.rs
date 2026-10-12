//! Table scanning and analysis utilities.
//!
//! Provides the `TableScan` struct and scanning functions for analyzing table structure
//! to determine if it should be rendered as a Markdown table or converted to list format.

use crate::converter::dom_context::TableContentSummary;
use crate::converter::utility::content::normalized_tag_name;
use std::borrow::Cow;

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
    /// Number of nested tables found inside this table
    pub nested_table_count: usize,
    /// Count of anchor elements in the table
    pub link_count: usize,
    /// Whether the table has content to write: text, an image, a rule, or (with
    /// `br_in_tables` on) a line break
    pub has_text: bool,
}

/// Scan a table element for structural metadata.
///
/// Analyzes the table to determine characteristics that influence rendering:
/// - Whether to render as a Markdown table or layout table
/// - If spanning cells are present
/// - If the table has semantic meaning (headers, captions)
///
/// `has_text`/`link_count`/`has_header`/`has_caption` are gathered from the *entire*
/// subtree (including nested tables), since they answer "is there any semantic content
/// here at all"; `has_span`/`nested_table_count`/`row_counts` feed the layout-table
/// heuristic and reflect only *this* table's own direct structure — a straight chain of
/// one-nested-table-per-cell tables is not a layout table (issue #13). The two concerns
/// are computed by separate passes below so neither one re-walks a nested table's subtree
/// that an ancestor table has already accounted for; see [`content_summary`].
///
/// # Arguments
/// * `node_handle` - Handle to the table element
/// * `parser` - HTML parser instance
/// * `dom_ctx` - DOM context for tag name resolution
/// * `br_in_tables` - The conversion option of the same name; a `<br>` counts as content
///   only under this option, since it is what a cell actually writes it as (see
///   [`apply_tag_content`])
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn scan_table(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
    br_in_tables: bool,
) -> TableScan {
    let (row_counts, nested_table_count, has_span) = scan_own_structure(node_handle, parser, dom_ctx);
    let content = content_summary(*node_handle, parser, dom_ctx, br_in_tables);
    TableScan {
        row_counts,
        has_span,
        has_header: content.has_header,
        has_caption: content.has_caption,
        nested_table_count,
        link_count: content.link_count,
        has_text: content.has_text,
    }
}

/// Resolve a node's normalized tag name via the `DomContext` cache, falling back to
/// parsing it directly when the node was not pre-registered (e.g. not yet visited).
fn tag_name_of<'a>(
    handle: &tl::NodeHandle,
    tag: &'a tl::HTMLTag,
    parser: &tl::Parser,
    dom_ctx: &'a super::super::super::DomContext,
) -> Cow<'a, str> {
    dom_ctx.tag_info(handle.get_inner(), parser).map_or_else(
        || normalized_tag_name(tag.name().as_utf8_str()).into_owned().into(),
        |info| Cow::Borrowed(info.name.as_str()),
    )
}

/// Scan a table's own direct row/cell structure: row cell counts, spanning cells, and a
/// count of directly-nested `<table>` elements. Never descends into a nested `<table>`'s
/// subtree — none of these fields count content past that boundary anyway (issue #13), so
/// stopping there bounds each table's own scan to its own layer instead of everything
/// nested below it.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn scan_own_structure(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
) -> (Vec<usize>, usize, bool) {
    let mut row_counts = Vec::new();
    let mut nested_table_count = 0usize;
    let mut has_span = false;

    let Some(tl::Node::Tag(root_tag)) = node_handle.get(parser) else {
        return (row_counts, nested_table_count, has_span);
    };

    let mut work: Vec<tl::NodeHandle> = root_tag.children().top().iter().copied().collect();
    while let Some(handle) = work.pop() {
        let Some(tl::Node::Tag(tag)) = handle.get(parser) else {
            continue;
        };
        let tag_name = tag_name_of(&handle, tag, parser, dom_ctx);

        match tag_name.as_ref() {
            "table" => nested_table_count += 1,
            "tr" | "row" => {
                let (cell_count, row_has_span) = scan_row_cells(tag, parser, dom_ctx);
                row_counts.push(cell_count);
                has_span |= row_has_span;
                // ~keep Still descend into the row's own cells (not their counts, already
                // ~keep captured above) so a `<table>` nested inside a `<td>` is found and
                // ~keep counted; only a nested `<table>` tag itself stops this walk.
                work.extend(tag.children().top().iter().copied());
            }
            _ => work.extend(tag.children().top().iter().copied()),
        }
    }

    (row_counts, nested_table_count, has_span)
}

/// Count cells (respecting `colspan`) in a single `<tr>`/`row` element and report whether
/// any cell carries a `colspan`/`rowspan` attribute.
fn scan_row_cells(
    row_tag: &tl::HTMLTag,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
) -> (usize, bool) {
    let mut cell_count = 0;
    let mut has_span = false;
    for child in row_tag.children().top().iter() {
        let Some(tl::Node::Tag(cell_tag)) = child.get(parser) else {
            continue;
        };
        let cell_name = tag_name_of(child, cell_tag, parser, dom_ctx);
        if !matches!(cell_name.as_ref(), "td" | "th" | "cell") {
            continue;
        }
        cell_count += super::cell::get_colspan(child, parser);
        let attrs = cell_tag.attributes();
        if attrs.get("colspan").is_some() || attrs.get("rowspan").is_some() {
            has_span = true;
        }
    }
    (cell_count, has_span)
}

/// A pending frame in the [`content_summary`] iterative traversal.
enum ContentFrame {
    /// Visit a node: accumulate its own content into the innermost open table accumulator,
    /// opening a new one first if the node is itself a `<table>`.
    Enter(tl::NodeHandle),
    /// All descendants of the `<table>` node with this id have been visited; finalize and
    /// cache its accumulator, merging it into the now-current parent accumulator (if any).
    ExitTable(u32),
}

/// Compute (and memoize) the full-subtree content summary — text/links/headers/caption,
/// including nested tables — for the `<table>` at `table_handle`.
///
/// A naive per-table full-subtree walk is O(n²) on a chain of n nested tables, since each
/// outer table's walk re-visits every table nested below it. This performs a single
/// iterative post-order traversal that opens a fresh accumulator on every `<table>` boundary
/// it crosses (including the root) and, on leaving each one, caches its result and folds it
/// into the enclosing accumulator. The first call for the outermost table in a chain pays
/// the full O(n) walk once and populates the cache for every table below it; subsequent
/// calls for those nested tables (made later, as the walk renders into them) are O(1) cache
/// hits. Explicit `Vec`-backed frames avoid native recursion, so this does not reintroduce
/// the stack-depth risk `deeply_nested_tables_do_not_overflow_stack` guards against.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn content_summary(
    table_handle: tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
    br_in_tables: bool,
) -> TableContentSummary {
    let table_id = table_handle.get_inner();
    if let Some(cached) = dom_ctx.cached_table_content_summary(table_id) {
        return cached;
    }

    let mut acc_stack: Vec<(u32, TableContentSummary)> = Vec::new();
    let mut work = vec![ContentFrame::Enter(table_handle)];
    while let Some(frame) = work.pop() {
        match frame {
            ContentFrame::Enter(handle) => {
                visit_content_node(handle, parser, dom_ctx, &mut acc_stack, &mut work, br_in_tables);
            }
            ContentFrame::ExitTable(id) => finish_table_accumulator(id, dom_ctx, &mut acc_stack),
        }
    }

    dom_ctx.cached_table_content_summary(table_id).unwrap_or_default()
}

/// Process one node in the [`content_summary`] traversal: fold raw text and recognized tags
/// into the innermost open accumulator, opening a new one when `handle` is itself a
/// `<table>`, and queue its children for a later `Enter`.
fn visit_content_node(
    handle: tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &super::super::super::DomContext,
    acc_stack: &mut Vec<(u32, TableContentSummary)>,
    work: &mut Vec<ContentFrame>,
    br_in_tables: bool,
) {
    match handle.get(parser) {
        Some(tl::Node::Raw(bytes)) => {
            let raw = bytes.as_utf8_str();
            let decoded = crate::text::decode_html_entities_cow(raw.as_ref());
            if !decoded.trim().is_empty() {
                if let Some((_, acc)) = acc_stack.last_mut() {
                    acc.has_text = true;
                }
            }
        }
        Some(tl::Node::Tag(tag)) => {
            let tag_name = tag_name_of(&handle, tag, parser, dom_ctx);
            if tag_name.as_ref() == "table" {
                let id = handle.get_inner();
                acc_stack.push((id, TableContentSummary::default()));
                work.push(ContentFrame::ExitTable(id));
            } else {
                apply_tag_content(
                    &tag_name,
                    tag,
                    parser,
                    acc_stack.last_mut().map(|(_, acc)| acc),
                    br_in_tables,
                );
            }
            work.extend(tag.children().top().iter().copied().map(ContentFrame::Enter));
        }
        _ => {}
    }
}

/// Fold a single non-table tag's contribution (link/header/caption/image/rule/line break)
/// into the current accumulator, if one is open.
fn apply_tag_content(
    tag_name: &str,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    acc: Option<&mut TableContentSummary>,
    br_in_tables: bool,
) {
    let Some(acc) = acc else { return };
    match tag_name {
        "a" => acc.link_count += 1,
        "caption" => acc.has_caption = true,
        "th" => acc.has_header = true,
        "img" | "graphic" if tag.attributes().get("src").is_some() || tag.attributes().get("alt").is_some() => {
            acc.has_text = true;
        }
        "svg" => acc.has_text = true,
        "iframe" if !crate::converter::media::embedded::extract_media_src(tag).is_empty() => acc.has_text = true,
        "audio" | "video"
            if !crate::converter::media::embedded::media_element_src(
                tag,
                parser,
                crate::options::InlineDataMedia::Keep,
            )
            .is_empty() =>
        {
            acc.has_text = true;
        }
        "input"
            if tag
                .attributes()
                .get("type")
                .flatten()
                .is_some_and(|value| value.as_utf8_str().eq_ignore_ascii_case("checkbox")) =>
        {
            acc.has_text = true;
        }
        // ~keep A rule is content without text: a table whose cells hold only rules is not a blank
        // ~keep spacer, and dropping it lost the whole table (issue #628).
        "hr" => acc.has_text = true,
        // ~keep A cell writes a literal `<br>` only when `br_in_tables` is on (see
        // ~keep `emit_table_cell_break`); with it off the break collapses to a space a cell
        // ~keep trims away, so only the `br_in_tables` case is content (issue #646).
        "br" if br_in_tables => acc.has_text = true,
        "cell" => {
            if let Some(Some(role)) = tag.attributes().get("role") {
                if role.as_utf8_str() == "head" {
                    acc.has_header = true;
                }
            }
        }
        _ => {}
    }
}

/// Pop the accumulator for `id`, cache it, and fold it into the new top-of-stack
/// accumulator (the enclosing table), if any.
fn finish_table_accumulator(
    id: u32,
    dom_ctx: &super::super::super::DomContext,
    acc_stack: &mut Vec<(u32, TableContentSummary)>,
) {
    let Some((_finished_id, finished)) = acc_stack.pop() else {
        return;
    };
    dom_ctx.cache_table_content_summary(id, finished);
    if let Some((_, parent)) = acc_stack.last_mut() {
        parent.merge(finished);
    }
}
