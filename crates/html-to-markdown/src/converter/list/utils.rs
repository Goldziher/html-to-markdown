//! Utility functions for list processing.
//!
//! Contains helper functions for loose list detection, indentation calculation,
//! list spacing, and list child processing.

use super::{ListContext, is_list_item};
use crate::converter::main_helpers::{tag_name_eq, trim_trailing_whitespace};
use crate::converter::utility::content::normalized_tag_name;
use crate::options::{ConversionOptions, ListIndentType, OutputFormat};
use tl;

type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Counter value an `<ol>` starts from when it has no (or an invalid) `start` attribute.
///
/// This mirrors the HTML spec default for ordered list numbering.
pub const DEFAULT_ORDERED_LIST_START: i64 = 1;

/// Parse the `start` attribute of an `<ol>` element into a counter value.
///
/// `start` is untrusted external input: the HTML spec allows any signed integer (browsers
/// count downward from a negative `start`), and a document can supply a magnitude that
/// overflows every fixed-width integer type. Rather than panicking or wrapping, out-of-range
/// magnitudes are clamped to the `i64` bounds the render-time counter uses, and syntactically
/// invalid values (empty, non-numeric) fall back to the spec default of 1.
pub fn parse_ordered_list_start(raw: &str) -> i64 {
    let trimmed = raw.trim();
    if let Ok(value) = trimmed.parse::<i64>() {
        return value;
    }

    let (is_negative, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };

    if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
        let clamped = if is_negative { i64::MIN } else { i64::MAX };
        tracing::warn!(
            target: "html_to_markdown::list",
            raw_value = trimmed,
            clamped_value = clamped,
            "ol start attribute magnitude out of range; clamping to i64 bounds"
        );
        return clamped;
    }

    if !trimmed.is_empty() {
        tracing::warn!(
            target: "html_to_markdown::list",
            raw_value = trimmed,
            default_value = DEFAULT_ORDERED_LIST_START,
            "ol start attribute is not a valid integer; using default start"
        );
    }
    DEFAULT_ORDERED_LIST_START
}

/// The number of tabs that reaches a list item's content column: each tab is four columns
/// wide, and a line indented by four or more columns past the content column is a code block.
pub const fn tabs_for_column(list_indent_columns: usize) -> usize {
    list_indent_columns.div_ceil(4)
}

/// Direct-child tag names that force a list item's own trailing separator (kept in sync with
/// `list/item.rs::has_block_children`'s identical match arms), for every item except the
/// list's last.
///
/// ~keep A list item containing one of these -- even without a `<p>` -- still needs a blank
/// ~keep line before/after it in our rendering to keep item boundaries unambiguous (a bare
/// ~keep `<pre>` or `<blockquote>` sibling can't just run into the next `- ` marker). Once that
/// ~keep blank line exists anywhere BETWEEN two items, every CommonMark-compliant reparse
/// ~keep concludes the *whole* list is loose (blank lines are a per-list, not per-item-pair,
/// ~keep signal) and re-wraps every item's content in `<p>`, including plain-text ones that had
/// ~keep none originally. Treating this same trigger set as "loose" up front -- not just literal
/// ~keep `<p>` -- renders every item with full blank-line separation from the first pass, which
/// ~keep is what the second-generation reparse would force anyway (spec examples 278, 308, 318).
/// ~keep Restricted to "not the last item": one of these tags in the list's OWN last item has
/// ~keep no following sibling to create a boundary blank line with, so it never actually
/// ~keep reparses the list as loose -- unlike literal `<p>`, which is excluded from this gate
/// ~keep below because it is CommonMark's actual, unconditional looseness signal regardless of
/// ~keep position (issue: an ordered list whose only loose-looking item is its last, e.g. a
/// ~keep trailing `<table>`, incorrectly gained a leading blank line without this gate).
const BLOCK_FORCING_CHILD_TAGS: [&str; 6] = ["div", "blockquote", "pre", "table", "hr", "dl"];

/// Resolve a node's normalized tag name via the `DomContext` cache, falling back to the raw
/// `tl` tag when no cached `TagInfo` exists for it.
fn resolve_tag_name(node_handle: tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> Option<String> {
    if let Some(info) = dom_ctx.tag_info(node_handle.get_inner(), parser) {
        return Some(info.name.clone());
    }
    match node_handle.get(parser) {
        Some(tl::Node::Tag(tag)) => Some(normalized_tag_name(tag.name().as_utf8_str()).into_owned()),
        _ => None,
    }
}

/// Check if a list (ul or ol) is "loose".
///
/// A loose list is one where any list item contains block-level elements like paragraphs
/// (`<p>`), or any other element that forces our own rendering to add a blank-line separator
/// (see `BLOCK_FORCING_CHILD_TAGS`), or a nested sublist that is itself loose (a loose nested
/// list's own trailing blank line becomes the boundary before the next item of THIS list when
/// it is that item's last content). In loose lists, all items should have blank line
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
pub fn is_loose_list(node_handle: tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return false;
    };

    let children = tag.children();
    let items: Vec<tl::NodeHandle> = children
        .top()
        .iter()
        .copied()
        .filter(|child_handle| {
            dom_ctx.tag_info(child_handle.get_inner(), parser).map_or_else(
                || {
                    matches!(
                        child_handle.get(parser),
                        Some(tl::Node::Tag(child_tag))
                            if tag_name_eq(child_tag.name().as_utf8_str(), "li")
                    )
                },
                |info| info.name == "li",
            )
        })
        .collect();
    let Some(last_index) = items.len().checked_sub(1) else {
        return false;
    };

    for (index, item_handle) in items.iter().enumerate() {
        let Some(tl::Node::Tag(item_tag)) = item_handle.get(parser) else {
            continue;
        };
        let is_last = index == last_index;
        let li_children = item_tag.children();
        for li_child_handle in li_children.top().iter() {
            let Some(name) = resolve_tag_name(*li_child_handle, parser, dom_ctx) else {
                continue;
            };
            if name == "p" {
                return true;
            }
            if is_last {
                continue;
            }
            if BLOCK_FORCING_CHILD_TAGS.contains(&name.as_str()) {
                return true;
            }
            if matches!(name.as_str(), "ul" | "ol") && is_loose_list(*li_child_handle, parser, dom_ctx) {
                return true;
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
/// * `list_indent_columns` - The item's content column
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
pub fn add_list_continuation_indent(
    output: &mut String,
    list_indent_columns: usize,
    blank_line: bool,
    options: &ConversionOptions,
) {
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

    match options.list_indent_type {
        ListIndentType::Tabs => {
            for _ in 0..tabs_for_column(list_indent_columns) {
                output.push('\t');
            }
        }
        // ~keep `list_indent_columns` is the item's content column (see
        // ~keep Context::list_indent_columns), not a uniform per-depth value.
        ListIndentType::Spaces => {
            for _ in 0..list_indent_columns {
                output.push(' ');
            }
        }
    }
}

/// Calculate the indentation string for list continuations based on depth and options.
pub fn continuation_indent_string(list_indent_columns: usize, options: &ConversionOptions) -> Option<String> {
    match options.list_indent_type {
        ListIndentType::Tabs => {
            let tabs = tabs_for_column(list_indent_columns);
            if tabs == 0 {
                return None;
            }
            Some("\t".repeat(tabs))
        }
        // ~keep `list_indent_columns` is the item's content column (see
        // ~keep Context::list_indent_columns), not a uniform per-depth value.
        ListIndentType::Spaces => {
            if list_indent_columns == 0 {
                return None;
            }
            Some(" ".repeat(list_indent_columns))
        }
    }
}

/// Write the list item's continuation indent when `output` is at the start of a line inside the
/// item, so the text or hard break written next stays in the item. Not in verbatim content or in a
/// detached buffer, where `output` is not the item's own text.
pub fn indent_list_item_line_start(output: &mut String, ctx: &Context, options: &ConversionOptions) {
    if ctx.in_list_item
        && !ctx.in_code
        && !ctx.in_ruby
        && !ctx.in_table_cell
        && !ctx.convert_as_inline
        && output.ends_with('\n')
        && !output.ends_with("\n\n")
    {
        if let Some(indent) = continuation_indent_string(ctx.list_indent_columns, options) {
            output.push_str(&indent);
        }
    }
}

/// The column that the continuation indent for `list_indent_columns` reaches.
pub fn indent_column(list_indent_columns: usize, options: &ConversionOptions) -> usize {
    continuation_indent_string(list_indent_columns, options).map_or(0, |indent| {
        crate::converter::utility::escaping::leading_indent(&indent).1
    })
}

/// Whether a line at the list item's content column can start a block: it is within 3 columns
/// of the content column of the innermost item whose marker starts a list item. Further in, the
/// line is the text of that item's paragraph.
pub fn block_is_real(ctx: &Context, options: &ConversionOptions) -> bool {
    indent_column(ctx.list_indent_columns, options).saturating_sub(indent_column(ctx.real_item_columns, options)) < 4
}

/// The column a block that starts its own line in the list item is written at: the item's content
/// column, or the column of the item whose marker starts a list item where the content column
/// starts no block in a quote whose first line is outside it.
///
/// ~keep That quote holds no paragraph for a line at the content column to continue, so the
/// ~keep line would be an indented code block.
pub fn block_columns(ctx: &Context, options: &ConversionOptions) -> usize {
    if ctx.quote_starts_after_markers && !block_is_real(ctx, options) {
        ctx.real_item_columns
    } else {
        ctx.list_indent_columns
    }
}

/// Whether `marker`, written by a list item between markers on the line starting at
/// `marker_line_start`, starts a real list item inside the item at `enclosing_columns`.
///
/// ~keep The marker must start its own line: the buffer's first line follows the opening
/// ~keep marker. Measured from the enclosing item's content column, the marker line with the
/// ~keep item's content after it must open a block that can interrupt a paragraph (the check
/// ~keep that escapes a link label's continuation lines). Where no paragraph is open, any
/// ~keep marker within 3 columns starts an item.
pub fn marker_starts_item(
    output: &str,
    marker_line_start: Option<usize>,
    marker: &str,
    enclosing_columns: usize,
    previous: (&PreviousMarker, usize),
    options: &ConversionOptions,
) -> bool {
    let Some(line_start) = marker_line_start else {
        return false;
    };
    let enclosing_column = indent_column(enclosing_columns, options);
    let marker_column = crate::converter::utility::escaping::leading_indent(&output[line_start..]).1;
    let column = marker_column.saturating_sub(enclosing_column);
    if !paragraph_is_open_before(output, line_start, enclosing_column, previous) {
        return column < 4;
    }
    let line = format!("{}{marker}x", " ".repeat(column));
    crate::converter::utility::escaping::line_opens_block(&line)
}

/// Whether the first item of a list, whose marker `line` starts at `line_start` in `output` inside
/// the item at `enclosing_columns`, needs a blank line before it: the line cannot interrupt a
/// paragraph, and a paragraph is open there. `line` has no indent.
///
/// ~keep In `CommonMark` a marker line with content interrupts a paragraph unless its marker is
/// ~keep an ordered one other than `1.` (issue #662). A marker line without content (an empty
/// ~keep item) cannot, and a lone `-` is a heading underline (issue #667). In Djot no list can
/// ~keep (issue #670). The items after the first follow a list item.
pub fn list_needs_blank_line(
    (output, line_start): (&str, usize),
    line: &str,
    enclosing_columns: usize,
    previous: (&PreviousMarker, usize),
    options: &ConversionOptions,
) -> bool {
    !marker_line_interrupts(line, options)
        && paragraph_is_open_before(output, line_start, indent_column(enclosing_columns, options), previous)
}

/// Whether the marker `line` of a list's first item, without its indent, can interrupt a paragraph.
fn marker_line_interrupts(line: &str, options: &ConversionOptions) -> bool {
    options.output_format != OutputFormat::Djot && marker_line_can_interrupt(line)
}

/// Whether the marker `line`, without its indent, can interrupt a `CommonMark` paragraph.
fn marker_line_can_interrupt(line: &str) -> bool {
    use crate::converter::utility::escaping::{is_heading_underline, line_opens_block};
    line_opens_block(line) && !is_heading_underline(line.trim_start_matches([' ', '\t']))
}

/// Whether the item that wrote its marker line at `line_start` in `output`, on the line after text
/// inside its list, needs a blank line before that line: the line cannot interrupt the text.
///
/// ~keep The text before the item was checked with an item that has content (issue #625). A marker
/// ~keep line without content cannot interrupt, and a lone `-` is a heading underline (issue #667).
pub fn marker_line_after_text_needs_blank_line(output: &str, line_start: usize) -> bool {
    !marker_line_can_interrupt(written_marker_line(output, line_start))
}

/// The marker line that the first item of a list wrote at `line_start` in `output`, without its
/// indent.
///
/// ~keep The stand-in line has no indent either: the marker sits where the list's items start.
fn written_marker_line(output: &str, line_start: usize) -> &str {
    let written = &output[line_start..];
    written[..written.find('\n').unwrap_or(written.len())].trim_start_matches([' ', '\t'])
}

/// The indent that puts a marker on the next line at the column where it starts at `end` on the
/// current line of `output`: the line up to `end`, with every byte but a tab written as a space.
fn column_of_written_marker(output: &str, end: usize) -> String {
    let start = output[..end].rfind('\n').map_or(0, |pos| pos + 1);
    output[start..end]
        .chars()
        .map(|ch| if ch == '\t' { '\t' } else { ' ' })
        .collect()
}

/// Whether the line of `output` that holds `end`, a line of bare list markers up to `end`, reads as
/// a thematic break from its start or from one of its markers on.
fn bare_marker_line_is_rule(output: &str, end: usize) -> bool {
    let start = output[..end].rfind('\n').map_or(0, |pos| pos + 1);
    let line_end = output[end..].find('\n').map_or(output.len(), |pos| end + pos);
    let mut rest = output[start..line_end].trim_start_matches([' ', '\t']);
    loop {
        if crate::converter::utility::escaping::is_rule(rest) {
            return true;
        }
        match strip_leading_bare_marker(rest) {
            Some(next) => rest = next,
            None => return false,
        }
    }
}

pub use super::{ItemLineScan, LastList, PreviousMarker, switched_delimiter};

/// Whether a paragraph is open before the line at `line_start`, stored for the next marker line.
fn paragraph_is_open_before(
    output: &str,
    line_start: usize,
    enclosing_column: usize,
    (previous, buffer): (&PreviousMarker, usize),
) -> bool {
    let open = paragraph_is_open(
        &output[..line_start],
        enclosing_column,
        previous.get(buffer, output, enclosing_column),
    );
    previous.set(buffer, output, line_start, enclosing_column, open);
    open
}

/// Whether a paragraph is open at the end of `output` in the item whose content starts at
/// `enclosing_column`: only then must a marker line interrupt it to start a list item.
///
/// ~keep `CommonMark` checks the interrupt rule only when the deepest open block a line reaches
/// ~keep is a paragraph (issue #633). The lines within 3 columns of the item's content column
/// ~keep decide it; a deeper line is inside the block above it. A blank line, or a line that
/// ~keep opens a block, leaves no paragraph open: after a list item's line the open block is
/// ~keep the list.
/// ~keep A marker line that cannot interrupt a paragraph is an item only when none was open
/// ~keep before it, so the walk looks past it. Any other line is paragraph text, and so is the
/// ~keep buffer's first line, which follows the opening marker.
fn paragraph_is_open(output: &str, enclosing_column: usize, previous: Option<(usize, bool)>) -> bool {
    use crate::converter::utility::escaping::{leading_indent, opens_block};
    let text = output.strip_suffix('\n').unwrap_or(output);
    let mut end = text.len();
    loop {
        let start = text[..end].rfind('\n').map_or(0, |pos| pos + 1);
        if start == 0 {
            return true;
        }
        let line = &text[start..end];
        end = start - 1;
        if line.trim().is_empty() {
            return false;
        }
        let (indent, column) = leading_indent(line);
        if column < enclosing_column {
            return true;
        }
        if column - enclosing_column >= 4 {
            continue;
        }
        let rest = &line[indent..];
        if opens_block(rest) {
            return false;
        }
        if strip_leading_bare_marker(rest).is_none() {
            return true;
        }
        // ~keep The previous marker line of this item's lists: its own check already walked
        // ~keep back from here, so the walk stays linear in the number of items.
        if let Some((_, open)) = previous.filter(|&(line_start, _)| line_start == start) {
            return open;
        }
    }
}

/// The answer of the paragraph check at the previous marker line of an item's lists: the address
/// of the buffer, the line start, the line before it, the enclosing content column and whether a
/// paragraph was open before the line.
///
/// ~keep The lists share it through the context. The buffer address, the enclosing content column
/// ~keep and the line before the marker line must match: in another buffer, in a list at another
/// ~keep depth, or after a write that changed the end of the buffer, the line start means nothing.
/// If this list is immediately preceded by an HTML comment whose own immediately preceding
/// sibling is a list of this same tag (`ul`/`ol`), return an empty separator comment.
///
/// ~keep `CommonMark` merges two adjacent lists of the same type into one list unless
/// ~keep something else -- and per the spec, only a raw HTML comment qualifies -- sits between
/// ~keep them. This converter otherwise drops every HTML comment unconditionally (a real
/// ~keep content-preservation policy for stray markup elsewhere), but dropping THIS one
/// ~keep specific comment discards the only thing keeping the two lists apart, so it un-merges
/// ~keep them on every reparse and the next conversion pass never recovers a matching
/// ~keep separator (spec example 308). A comment anywhere else (inline text, the sole content
/// ~keep of a block) is unrelated to this ambiguity and keeps the existing strip behavior --
/// ~keep this check only fires for the exact position where CommonMark assigns the comment
/// ~keep separator meaning.
pub fn preceding_same_type_list_separator_comment(
    node_handle: tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
    tag_name: &str,
) -> Option<String> {
    let id = node_handle.get_inner();
    let siblings = match dom_ctx.parent_of(id) {
        Some(parent_id) => dom_ctx.children_of(parent_id)?,
        None => &dom_ctx.root_children,
    };
    let position = dom_ctx
        .sibling_index(id)
        .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))?;

    // ~keep The source text between two block siblings (e.g. the "\n" between `</ul>` and
    // ~keep `<!-- -->`) parses as its own whitespace-only Raw sibling node -- skip those to
    // ~keep find the nearest MEANINGFUL sibling on each side, exactly like
    // ~keep `get_previous_sibling_tag` does for the tag-name-only lookup.
    let mut cursor = position;
    loop {
        cursor = cursor.checked_sub(1)?;
        match siblings.get(cursor)?.get(parser) {
            Some(tl::Node::Comment(_)) => break,
            Some(tl::Node::Raw(raw)) if raw.as_utf8_str().trim().is_empty() => {}
            _ => return None,
        }
    }

    let previous_list_name = loop {
        cursor = cursor.checked_sub(1)?;
        let sibling = *siblings.get(cursor)?;
        if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
            if raw.as_utf8_str().trim().is_empty() {
                continue;
            }
        }
        break resolve_tag_name(sibling, parser, dom_ctx)?;
    };

    if previous_list_name == tag_name {
        Some("<!-- -->".to_owned())
    } else {
        None
    }
}

/// Strip one bare list marker -- a single bullet char (`-`, `*`, `+`) followed by a space,
/// or one-or-more ASCII digits followed by `". "` or `") "` -- from the front of `text`,
/// returning what remains after it. Returns `None` when `text` does not start with a marker.
pub(super) fn strip_leading_bare_marker(text: &str) -> Option<&str> {
    let digit_count = text.bytes().take_while(u8::is_ascii_digit).count();
    if digit_count > 0 {
        let rest = &text[digit_count..];
        if let Some(rest) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
            return Some(rest);
        }
    }
    let mut chars = text.chars();
    let first = chars.next()?;
    if matches!(first, '-' | '*' | '+') {
        return chars.as_str().strip_prefix(' ');
    }
    None
}

/// Whether the current line of `output` (from the last `\n`, or the very start of the
/// buffer) is nothing but one or more bare list markers -- concatenated bullets
/// (`"- "`, `"* "`, `"+ "`) and/or ordered markers (`"N. "`) -- with optional leading
/// indentation and no other content.
///
/// ~keep A plain suffix check like `output.ends_with("* ")` also matches the closing
/// ~keep `"**"` of `<strong>` (or the closing `"*"` of `<em>`) immediately followed by a
/// ~keep migrated trailing space, e.g. `"**b** "`: its last two bytes are literally `'*'`
/// ~keep and `' '`, indistinguishable by suffix alone from a real bare `"* "` bullet. That
/// ~keep false positive suppressed the newline before a nested list, flattening it onto
/// ~keep the parent line and destroying it on reparse. Requiring the WHOLE line (after
/// ~keep stripping only leading indentation) to decompose into nothing but marker tokens
/// ~keep rules that out: real inline content preceding a marker-looking tail is not itself
/// ~keep a marker, so the decomposition fails and the check correctly returns `false`. This
/// ~keep also naturally handles several single-child lists nested directly inside each
/// ~keep other, whose bare markers stack on one physical line with nothing else between
/// ~keep them (CommonMark spec example 299: `"1. - 2. foo"`).
///
/// ~keep `pub` (not `pub(crate)`, which clippy's `redundant_pub_crate` flags here since
/// ~keep `list::utils` is itself only `pub` within the crate): the same ambiguity affects
/// ~keep the "is this block the item's first content, or a continuation?" decision in
/// ~keep `handlers/blockquote.rs`, `handlers/code_block.rs`, and `block/div.rs`, which now
/// ~keep reuse this instead of repeating a hardcoded, `+`-missing, two-byte suffix check.
pub fn line_is_bare_list_marker(output: &str) -> bool {
    let line_start = output.rfind('\n').map_or(0, |pos| pos + 1);
    let mut rest = output[line_start..].trim_start_matches([' ', '\t']);
    if rest.is_empty() {
        return false;
    }
    while let Some(next) = strip_leading_bare_marker(rest) {
        if next.is_empty() {
            return true;
        }
        rest = next;
    }
    false
}

/// Whether the list item that `output` ends inside is still open: the item is open where this
/// buffer starts (`ctx.list_item_open`), and every non-blank line since the item's marker line
/// starts at the item's content column (`indent`). The buffer's first line counts as at the
/// column: the container that owns the buffer puts it there.
///
/// ~keep A line at a shallower column is a block that already left the item, and nothing
/// ~keep reopens it. Writing the content column after it opens an indented code block once the
/// ~keep column is 4 or more (issue #583).
/// ~keep Each block of an item asks this, so the answer for the lines before the last one is
/// ~keep kept in the item's context and the next check reads only the lines written since
/// ~keep (issue #649).
pub fn item_is_open(output: &str, indent: &str, ctx: &Context) -> bool {
    if !ctx.list_item_open {
        return false;
    }
    let content = output.trim_end();
    if content.is_empty() {
        return true;
    }
    let last_start = content.rfind('\n').map_or(0, |pos| pos + 1);
    let before = lines_are_open(&output[..last_start], indent, &ctx.item_lines);
    let last = output[last_start..].split('\n').next().unwrap_or_default();
    match before {
        _ if strip_leading_bare_marker(last.trim_start_matches([' ', '\t'])).is_some() => true,
        None => true,
        Some(_) if indent.is_empty() || !last.starts_with(indent) => false,
        Some(open) => open,
    }
}

/// Whether the item is open after the complete lines `output` (it ends at a line start), or
/// `None` when they are all blank. Stores the answer in `scan` for the next check.
pub(super) fn lines_are_open(output: &str, indent: &str, scan: &ItemLineScan) -> Option<bool> {
    let buffer = output.as_ptr() as usize;
    let (start, below) = scan
        .read(buffer, indent, output)
        .unwrap_or_else(|| (last_marker_line_start(output), None));
    let mut open = below;
    for line in output[start..].split('\n').filter(|line| !line.trim().is_empty()) {
        open = Some(match open {
            _ if strip_leading_bare_marker(line.trim_start_matches([' ', '\t'])).is_some() => true,
            None => true,
            Some(_) if indent.is_empty() || !line.starts_with(indent) => false,
            Some(open) => open,
        });
    }
    scan.write(buffer, indent, output, open);
    open
}

/// The start of the last line of `output` that is a bare list marker line, or 0 when there is none.
///
/// ~keep A marker line opens the item whatever the lines before it say, so a check with no stored
/// ~keep answer reads forward from there: it reads the lines of the innermost item, not the buffer.
fn last_marker_line_start(output: &str) -> usize {
    let mut end = output.len();
    while end > 0 {
        let start = output[..end - 1].rfind('\n').map_or(0, |pos| pos + 1);
        if strip_leading_bare_marker(output[start..end].trim_start_matches([' ', '\t'])).is_some() {
            return start;
        }
        end = start;
    }
    0
}

/// The context for the children of a container that renders them into a buffer of its own:
/// they see whether the list item is still open where that buffer will be written.
pub fn nested_block_context(output: &str, ctx: &Context, options: &ConversionOptions) -> Context {
    let indent = continuation_indent_string(ctx.list_indent_columns, options).unwrap_or_default();
    Context {
        list_item_open: item_is_open(output, &indent, ctx),
        item_lines: ItemLineScan::new_item(),
        ..ctx.clone()
    }
}

/// Trim whitespace that follows a bare list marker at the end of `output` back to the marker's
/// own space, and say whether `output` ends in a bare marker afterwards.
///
/// ~keep Whitespace-only text after the marker (kept in strict whitespace mode) is not content:
/// ~keep counting it made the first block of the item start after a blank line, which ends the
/// ~keep item in `CommonMark` (issue #583).
pub fn trim_whitespace_after_bare_marker(output: &mut String) -> bool {
    let content_end = output.trim_end().len();
    if content_end == output.len() {
        return line_is_bare_list_marker(output);
    }
    let tail = output[content_end..].to_string();
    output.truncate(content_end);
    output.push(' ');
    if line_is_bare_list_marker(output) {
        return true;
    }
    output.truncate(content_end);
    output.push_str(&tail);
    false
}

/// Start a block inside a list item: on the marker line when the item has no content yet, after
/// a blank line at the content column while the item is open, and after a blank line at the
/// start of the line once the item has ended (issue #583).
pub fn start_block_in_list_item(output: &mut String, ctx: &Context, options: &ConversionOptions) {
    if trim_whitespace_after_bare_marker(output) {
        return;
    }
    let indent = continuation_indent_string(ctx.list_indent_columns, options).unwrap_or_default();
    if item_is_open(output, &indent, ctx) {
        add_list_continuation_indent(output, ctx.list_indent_columns, true, options);
    } else {
        trim_trailing_whitespace(output);
        if !output.ends_with("\n\n") {
            output.push_str(if output.ends_with('\n') { "\n" } else { "\n\n" });
        }
    }
}

/// Add appropriate leading separator before a list.
///
/// Lists need different separators depending on context:
/// - In table cells: a line-break separator if there's already content -- a literal `<br>`
///   under `br_in_tables`, otherwise a single space (see `emit_table_cell_break`)
/// - Outside lists: blank line (\n\n) if needed
/// - Inside list items: blank line before nested list
///
/// ~keep A GFM pipe cell cannot hold a literal newline, so the two adjacent `<li>`s (or the
/// ~keep prior cell content and this list) need an in-cell substitute for the line break a
/// ~keep block boundary would normally get. `br_in_tables` governs exactly this substitution
/// ~keep everywhere else it applies (`main_helpers::emit_table_cell_break`, consulted by
/// ~keep `<br>`/`<div>`/`<p>` continuations in a cell) — reusing it here keeps list-in-cell
/// ~keep breaks consistent with every other break-in-cell site instead of hardcoding `<br>`
/// ~keep regardless of the option (issue: `br_in_tables: false` was ignored for list items
/// ~keep sharing a table cell).
pub fn add_list_leading_separator(output: &mut String, ctx: &Context, options: &ConversionOptions) {
    if ctx.in_table_cell {
        let is_table_continuation =
            !output.is_empty() && !output.ends_with('|') && !output.ends_with(' ') && !output.ends_with("<br>");
        if is_table_continuation {
            crate::converter::main_helpers::emit_table_cell_break_in_context(output, options.br_in_tables, ctx);
        }
        return;
    }

    if !output.is_empty() && !ctx.in_list {
        let needs_newline = !output.ends_with("\n\n") && !line_is_bare_list_marker(output);
        if needs_newline {
            output.push_str("\n\n");
        }
        return;
    }

    if ctx.in_list_item && !output.is_empty() {
        if line_is_bare_list_marker(output) {
            return;
        }

        // ~keep A loose list wraps every item's leading text in a real <p> on any
        // ~keep CommonMark-compliant reparse (looseness is a per-list, not per-item,
        // ~keep property), and `block/paragraph.rs` always follows a <p> with a blank line
        // ~keep even inside a list item. So a nested list that is this item's next sibling
        // ~keep needs that same blank line here when the CONTAINING list is loose, even
        // ~keep though the leading text itself arrived as bare inline text with no <p> --
        // ~keep otherwise this pass's tighter join reparses with the blank line the loose
        // ~keep list demands, moving the nested list's `<p>`-wrapped leading item further
        // ~keep from a fixpoint instead of closer (spec example 319).
        if ctx.loose_list {
            trim_trailing_whitespace(output);
            if !output.ends_with("\n\n") {
                if output.ends_with('\n') {
                    output.push('\n');
                } else {
                    output.push_str("\n\n");
                }
            }
        } else if !output.ends_with('\n') {
            trim_trailing_whitespace(output);
            output.push('\n');
        }
    }
}

/// Add appropriate trailing separator after a nested list.
///
/// Nested lists inside list items need trailing newlines to separate
/// from following content. In loose lists, use blank line (\n\n). In tight lists, single newline (\n).
pub fn add_nested_list_trailing_separator(output: &mut String, ctx: &Context) {
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
pub const fn calculate_list_nesting_depth(ctx: &Context) -> usize {
    if ctx.in_list && !ctx.in_list_item {
        ctx.list_depth + 1
    } else {
        ctx.list_depth
    }
}

#[derive(Clone, Copy)]
pub(super) struct ListChildrenContext<'a> {
    pub list: ListContext<'a>,
    pub ordered: bool,
    pub loose: bool,
    pub nested_depth: usize,
    pub start_counter: i64,
    pub delimiter: Option<char>,
}

#[derive(Default)]
struct MarkerPosition {
    line_start: Option<usize>,
    bare_end: Option<usize>,
}

struct ListChildProcessor<'a> {
    config: ListChildrenContext<'a>,
    item_ctx: Context,
    counter: i64,
    counter_saturated: bool,
    first_item: bool,
}

impl<'a> ListChildProcessor<'a> {
    fn new(config: ListChildrenContext<'a>) -> Self {
        let context = config.list.ctx;
        let counter = config.start_counter;
        let item_ctx = Context {
            in_ordered_list: config.ordered,
            list_counter: if config.ordered { counter } else { 0 },
            in_list: true,
            list_depth: config.nested_depth,
            ul_depth: if config.ordered {
                context.ul_depth
            } else {
                context.ul_depth + 1
            },
            loose_list: config.loose,
            prev_item_had_blocks: false,
            ordered_delimiter: config.delimiter,
            ..context.clone()
        };
        Self {
            config,
            item_ctx,
            counter,
            counter_saturated: false,
            first_item: true,
        }
    }

    fn marker_before(
        &mut self,
        child_handle: tl::NodeHandle,
        parser: &tl::Parser,
        output: &mut String,
    ) -> MarkerPosition {
        let context = self.config.list.ctx;
        if !self.first_item || !is_list_item(child_handle, parser, self.config.list.dom_ctx) {
            return MarkerPosition::default();
        }
        self.first_item = false;
        let item_block = context.in_list_item && context.inline_depth == 0 && !context.text_in_markers;
        if !item_block {
            return MarkerPosition::default();
        }
        if line_is_bare_list_marker(output) {
            return MarkerPosition {
                bare_end: Some(output.len()),
                ..MarkerPosition::default()
            };
        }
        if !output.ends_with('\n') {
            return MarkerPosition::default();
        }
        let marker = if self.config.ordered {
            format!("{}. x", self.counter)
        } else {
            String::from("- x")
        };
        if list_needs_blank_line(
            (output, output.len()),
            &marker,
            context.real_item_columns,
            (&context.previous_marker, std::ptr::from_ref::<String>(output) as usize),
            self.config.list.options,
        ) {
            output.push('\n');
            MarkerPosition::default()
        } else {
            MarkerPosition {
                line_start: (!context.in_marker_span && marker_line_interrupts(&marker, self.config.list.options))
                    .then_some(output.len()),
                bare_end: None,
            }
        }
    }

    fn marker_after(&self, output: &mut String, position: MarkerPosition) {
        let context = self.config.list.ctx;
        if let Some(line_start) = position.line_start {
            let line = written_marker_line(output, line_start);
            if list_needs_blank_line(
                (output, line_start),
                line,
                context.real_item_columns,
                (&context.previous_marker, std::ptr::from_ref::<String>(output) as usize),
                self.config.list.options,
            ) {
                output.insert(line_start, '\n');
            }
        }
        // ~keep An empty item that ends a line of bare markers can complete a thematic
        // ~keep break (`- - -`), so its marker starts the next line instead (issue #667).
        if let Some(end) = position.bare_end.filter(|&end| bare_marker_line_is_rule(output, end)) {
            let indent = column_of_written_marker(output, end);
            output.replace_range(end - 1..end, &format!("\n{indent}"));
        }
    }

    fn advance_counter(&mut self, child_handle: tl::NodeHandle, parser: &tl::Parser) {
        if !self.config.ordered || !is_list_item(child_handle, parser, self.config.list.dom_ctx) {
            return;
        }
        if self.counter < i64::MAX {
            self.counter += 1;
        } else if !self.counter_saturated {
            tracing::warn!(
                target: "html_to_markdown::list",
                counter = self.counter,
                "ordered list counter reached i64::MAX; subsequent items repeat this value"
            );
            self.counter_saturated = true;
        }
    }

    fn render(&mut self, child_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String) {
        if self.config.ordered {
            self.item_ctx.list_counter = self.counter;
        }
        let marker = self.marker_before(*child_handle, parser, output);
        crate::converter::walk_node(
            child_handle,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                self.config.list.options,
                &self.item_ctx,
                self.config.list.depth + 1,
                self.config.list.dom_ctx,
            ),
        );
        self.marker_after(output, marker);
        self.advance_counter(*child_handle, parser);
    }
}

fn is_whitespace_node(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> bool {
    matches!(node_handle.get(parser), Some(tl::Node::Raw(bytes)) if bytes.as_utf8_str().trim().is_empty())
}

/// Process a list's children while maintaining marker and counter state.
pub(super) fn process_list_children(
    node_handle: tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: ListChildrenContext<'_>,
) {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return;
    };
    let mut processor = ListChildProcessor::new(context);
    for child_handle in tag.children().top().iter() {
        if !is_whitespace_node(child_handle, parser) {
            processor.render(child_handle, parser, output);
        }
    }
}
