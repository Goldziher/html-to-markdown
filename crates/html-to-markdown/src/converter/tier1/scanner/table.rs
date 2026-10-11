fn close_table(
    state: &mut Tier1State,
    options: &ConversionOptions,
    table_probes: &mut Vec<TableLayoutProbe>,
) -> Result<(), BailReason> {
    // ~keep Pop the table state and (if safe) emit the GFM table to main output.
    let Some(ts) = state.table_stack.pop() else {
        return Ok(());
    };
    // ~keep Popped together with the `TableState` it belongs to; the truncate re-syncs
    // the two stacks if a malformed document ever pushed one without the other.
    let probe = table_probes.pop().unwrap_or_default();
    table_probes.truncate(state.table_stack.len());
    validate_table_layout(&ts, &probe)?;
    render_closed_table(state, ts, table_probes, options);
    Ok(())
}

fn validate_table_layout(
    table: &crate::converter::tier1::state::TableState,
    probe: &TableLayoutProbe,
) -> Result<(), BailReason> {
    // ~keep Safety checks: ensure Tier-2 would also use the GFM path.
    // ~keep
    // ~keep Tier-2 uses the layout (non-GFM) path when ALL of these hold:
    //   (a) no <th> anywhere in the table, AND
    //   (b) no <caption>, AND
    //   (c) looks_like_layout || is_blank || (row_count<=2 && link_count>=3)
    // ~keep
    // ~keep where (block/table/builder.rs)
    //   looks_like_layout = nested_table_count > 1 || (has_span && has_border_zero)
    // ~keep
    // ~keep Both disjuncts are checked below — neither is unreachable here.
    // An earlier revision of this comment claimed nested tables and
    // colspan/rowspan had "already bailed"; both claims were false (Phase HH
    // renders a nested table inline into the parent cell, and open_table_cell
    // expands colspan instead of bailing), and the resulting gap let Tier-1
    // emit a GFM table for input Tier-2 renders as a bullet list.
    // ~keep
    // ~keep If those conditions could apply to this table, we bail rather than
    // emit a GFM table that Tier-2 would have rendered differently.
    // ~keep
    // ~keep When a <caption> is present, Tier-2 always takes the GFM path
    // regardless of <th> presence (has_caption short-circuits the layout check).
    let has_caption = table.caption_text.is_some();
    // One-cell wrappers are unwrapped by Tier-2 to preserve the nested table's
    // structure. Its DOM walker also handles the traversal-depth boundary.
    if !table.has_th
        && !has_caption
        && !probe.has_span
        && probe.nested_table_count > 0
        && table.rows.len() == 1
        && table.first_row_col_count == Some(1)
    {
        return Err(BailReason::TableNestedTable);
    }
    if !table.has_th && !has_caption && table_requires_layout_fallback(table, probe) {
        return Err(BailReason::Classifier);
    }
    // ~keep Issue #484: some row in this table (checked only once the whole table is known,
    // see `TableState::had_single_cell_nested_table_row`) closed with exactly one cell
    // holding a nested table, and the table did not already match the more specific
    // one-cell-*wrapper* bail above. Tier-2 defers that row's nested table to a
    // separate GFM table after this one; this scanner has no equivalent deferred-output
    // buffer, so it bails and lets Tier-2 (authoritative) render it.
    if table.had_single_cell_nested_table_row {
        return Err(BailReason::TableNestedTableInSingleCellRow);
    }
    Ok(())
}

fn table_requires_layout_fallback(
    table: &crate::converter::tier1::state::TableState,
    probe: &TableLayoutProbe,
) -> bool {
    let expanded_columns = |row: &Vec<(String, u16)>| row.iter().map(|(_, span)| usize::from(*span)).sum::<usize>();
    let expected_columns = table.first_row_col_count.unwrap_or(0);
    let inconsistent_columns = table.rows.iter().any(|row| expanded_columns(row) != expected_columns);
    let link_heavy = table.rows.len() <= 2 && table.link_count >= 3;
    let blank = table.rows.is_empty()
        || table
            .rows
            .iter()
            .all(|row| row.iter().all(|(cell, _)| cell.trim().is_empty()));
    let multiple_nested_tables = probe.nested_table_count > 1;
    let spanning_borderless = probe.has_span && probe.border_zero;
    inconsistent_columns || link_heavy || blank || multiple_nested_tables || spanning_borderless
}

fn render_closed_table(
    state: &mut Tier1State,
    table: crate::converter::tier1::state::TableState,
    table_probes: &mut [TableLayoutProbe],
    options: &ConversionOptions,
) {
    // ~keep Nested rows write their cell content into the parent buffer (#760).
    if table.inline_mode {
        if let Some(outer) = state.table_stack.last_mut() {
            outer.had_nested_table = true;
        }
        // ~keep Mirrors Tier-2's `nested_table_count`: the enclosing table counts this
        // one, and stops there — tables nested deeper already counted against
        // their own immediate parent when they closed.
        if let Some(outer_probe) = table_probes.last_mut() {
            outer_probe.nested_table_count += 1;
        }
        // ~keep Markdown tables cannot nest: keep inner cell content and row breaks only (#760).
        let mut nested = table.caption_text.as_deref().unwrap_or_default().to_string();
        for row in &table.rows {
            let mut row_cells = row.iter().filter(|(cell, _)| !cell.is_empty());
            if let Some((first, _)) = row_cells.next() {
                if !nested.is_empty() {
                    crate::converter::main_helpers::emit_table_cell_break(&mut nested, options.br_in_tables);
                }
                nested.push_str(first);
                for (cell, _) in row_cells {
                    nested.push(' ');
                    nested.push_str(cell);
                }
            }
        }
        let write_nested = |dest: &mut String| {
            if !nested.is_empty() && !dest.trim_end().is_empty() {
                crate::converter::main_helpers::emit_table_cell_break(dest, options.br_in_tables);
            }
            dest.push_str(&nested);
        };
        // ~keep Tier-2 separates the cell's child that holds the table from the whole cell
        // ~keep before it, but a quote, heading or code block around the table separates
        // ~keep itself when it closes, so the table breaks only inside that block.
        let in_block = state
            .stack
            .iter()
            .rev()
            .take_while(|frame| !matches!(frame.spec.kind, TagKind::TableCell { .. }))
            .any(|frame| {
                matches!(
                    frame.spec.kind,
                    TagKind::Blockquote | TagKind::Heading(_) | TagKind::Pre
                )
            });
        if in_block {
            with_cell_scratch(state, write_nested);
        } else {
            write_nested(state.cell_or_output_mut());
            if !nested.is_empty()
                && let Some(outer) = state.table_stack.last_mut()
            {
                outer.nested_table_ends.push(outer.current_cell.len());
            }
        }
    } else {
        emit_gfm_table(&mut state.output, table, options.output_format);
    }
}

fn close_table_head(state: &mut Tier1State) {
    if let Some(ts) = state.table_stack.last_mut() {
        ts.in_thead = false;
    }
}

fn close_table_body(state: &mut Tier1State) {
    if let Some(ts) = state.table_stack.last_mut() {
        ts.seen_tbody_close = true;
    }
}

/// Finalise a `<caption>` element.
///
/// Mirrors Tier-2's `builder.rs` caption handling: trim the collected text,
/// replace `-` with `\-` to prevent Markdown table-separator interpretation,
/// and store the result in `ts.caption_text` for emission before the table body.
fn close_table_caption(state: &mut Tier1State) {
    let Some(ts) = state.table_stack.last_mut() else {
        return;
    };
    ts.in_caption = false;
    let raw = std::mem::take(&mut ts.caption_buf);
    let trimmed = raw.trim();
    if !trimmed.is_empty() {
        ts.caption_text = Some(trimmed.replace('-', r"\-"));
    }
}

fn close_table_row(state: &mut Tier1State) {
    let Some(ts) = state.table_stack.last_mut() else {
        return;
    };
    if ts.current_row.is_empty() {
        return;
    }
    // ~keep Issue #484: record (rather than bail immediately) that this row's sole cell
    // held a nested table — `close_table` has seen every row and decides whether
    // the more specific one-cell-*wrapper* bail applies first. See
    // `TableState::had_single_cell_nested_table_row`.
    if ts.pending_single_cell_nested_table && ts.current_row.len() == 1 {
        ts.had_single_cell_nested_table_row = true;
    }
    ts.pending_single_cell_nested_table = false;
    // ~keep Track first-row column count for consistency checking — use the
    // colspan-expanded count so Tier-2's heuristic compares the same numbers.
    let col_count: usize = ts.current_row.iter().map(|(_, c)| usize::from(*c)).sum();
    if ts.first_row_col_count.is_none() {
        ts.first_row_col_count = Some(col_count);
    }
    let row = std::mem::take(&mut ts.current_row);
    ts.rows.push(row);
}

/// Close a table cell (`<td>` or `<th>`).
///
/// `is_implicit` skips the pipe-escape bail that only applies when the cell
/// was explicitly closed (implicit closes happen during row/table teardown
/// where we've already committed to the data we have).
fn close_table_cell(state: &mut Tier1State, is_implicit: bool, options: &ConversionOptions) -> Result<(), BailReason> {
    let Some(ts) = state.table_stack.last_mut() else {
        return Ok(());
    };
    ts.in_cell = false;
    // ~keep Trim the accumulated cell text (matches Tier-2 `text.trim()`).
    crate::converter::block::table::cell::separate_nested_table_following_content(
        &mut ts.current_cell,
        &ts.nested_table_ends,
        options.br_in_tables,
    );
    ts.nested_table_ends.clear();
    let cell_text_raw = ts.current_cell.trim().to_owned();
    // ~keep Replace newlines with spaces — mirrors Tier-2's `cell_text_content`
    // which calls `text.replace('\n', " ")` when `br_in_tables` is false.
    // `<br>` itself is handled at emission time (see `TagKind::LineBreak`
    // in `emit_void`), so no sentinel expansion is needed here.
    let cell_text = if cell_text_raw.contains('\n') {
        cell_text_raw.replace('\n', " ")
    } else {
        cell_text_raw
    };
    let cell_text = cell_text.trim().to_owned();
    // ~keep Bail if the cell contains a pipe: Tier-2 escapes `|` → `\|`
    // which changes the cell width computation; Tier-1 does not
    // implement pipe escaping.  Implicit closes skip this check because
    // they are triggered during structural teardown, not fresh cell data.
    // ~keep
    // ~keep Nested cells have already escaped their content pipes; let those bytes pass
    // ~keep rather than treating them as outer-cell delimiters. Reset for the next cell.
    let allow_pipes = ts.had_nested_table;
    ts.had_nested_table = false;
    if !is_implicit && !allow_pipes && cell_text.contains('|') {
        return Err(BailReason::TableBlockChildInCell);
    }
    // ~keep Track whether this row is (so far) a single cell that held a nested table —
    // see `TableState::pending_single_cell_nested_table`. Only the row's FIRST cell
    // can set it; any later cell proves the row has a sibling and clears it,
    // regardless of whether that later cell itself had a nested table.
    ts.pending_single_cell_nested_table = allow_pipes && ts.current_row.is_empty();
    // ~keep Phase L-prep: store (text, colspan) so emit_gfm_table can mirror
    // Tier-2's `for _ in 0..colspan { output.push_str(" |") }` (cell.rs:248)
    // and the layout-heuristic uses the colspan-expanded column count.
    let colspan = ts.current_cell_colspan;
    ts.current_row.push((cell_text, colspan));
    ts.current_cell.clear();
    ts.current_cell_colspan = 1;
    Ok(())
}

/// Flush a raw HTML text segment into the output (or current cell buffer),
/// decoding entities and collapsing whitespace (unless inside `<pre>`).
///
/// `base_offset` is the byte offset of `raw` within the original HTML input;
/// it is forwarded to the entity decoder so that `BailReason::UnknownEntity`
/// carries an accurate position.
///
/// Returns `Err(BailReason::UnknownEntity)` if an unrecognised entity is found.
/// True when `s` ends with an ordered-list marker (`<digit(s)>. ` or `<digit(s)>) `).
///
/// Used by the inter-block whitespace strip to recognise that the scanner just
/// emitted a list-item marker and the next text would be the item content;
/// leading whitespace from the source HTML indentation should be dropped.
fn ends_with_ordered_marker(s: &str) -> bool {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len < 3 || bytes[len - 1] != b' ' {
        return false;
    }
    let punct = bytes[len - 2];
    if punct != b'.' && punct != b')' {
        return false;
    }
    let mut i = len - 2;
    while i > 0 && bytes[i - 1].is_ascii_digit() {
        i -= 1;
    }
    i < len - 2 && (i == 0 || !bytes[i - 1].is_ascii_digit())
}

/// Returns `true` when the output tail is an explicit inline-element close
/// marker emitted by Tier-1.  These markers signal that the next whitespace
/// text node is between two inline siblings and should collapse to a single
/// space — even when the whitespace run contains a newline (Phase U-2).
///
/// Recognised markers:
/// - `**` — `</strong>` / `</b>` close
/// - `*` — `</em>` / `</i>` close (only a lone `*`, not part of `**`)
/// - `` ` `` — `</code>` close
/// - `)` — `</a>` (link) close, e.g. `](href)`
///
/// Block edges (`\n`, empty output, trailing space) are explicitly excluded.
fn output_ends_with_inline_close_marker(output: &str) -> bool {
    if output.is_empty() || output.ends_with('\n') || output.ends_with(' ') || output.ends_with('\t') {
        return false;
    }
    if output.ends_with("**") || output.ends_with('`') || output.ends_with(')') {
        return true;
    }
    output.ends_with('*') && !output.ends_with("**")
}

/// Returns `true` when the output tail is a non-marker text character —
/// e.g. ending in a letter, digit, or punctuation other than the inline-
/// close markers.  Text-tail preservation only fires for *horizontal*
/// whitespace runs (no `\n`/`\r`) because we cannot tell at flush time
/// whether the next tag is inline or block; preserving a space across a
/// newline-bearing run risks `text \n\n<list>` regressions.
fn output_ends_with_inline_text(output: &str) -> bool {
    if output.is_empty() || output.ends_with('\n') || output.ends_with(' ') || output.ends_with('\t') {
        return false;
    }
    !output_ends_with_inline_close_marker(output)
}

/// Start a new paragraph for inline content that directly follows a block whose output ends with
/// a single line break (a list, a table, `<hr>`), so it does not continue the block's last line
/// (issues #570, #571). In a table cell the cell break separates it instead (issue #645).
/// Mirrors Tier-2's `separate_from_block` in `walk_node`.
fn separate_inline_after_block(state: &mut Tier1State, br_in_tables: bool) -> Result<(), BailReason> {
    // ~keep `<pre>` sets the CODE bit too, so one test covers code spans and code blocks.
    if state.escape_ctx.contains(EscapeCtx::CODE) {
        return Ok(());
    }
    // ~keep A block in a cell ends with no line end: the cell break separates (issue #645).
    if state.in_table_cell() {
        let at = state.cell_or_output_mut().len();
        separate_block_in_cell_at(state, at, br_in_tables);
        return Ok(());
    }
    // ~keep Inside a list item Tier-2 starts it at the item's content column after a blank
    // ~keep line (issue #583); see `BailReason::ListItemUnsupportedBlockChild`.
    if state.list_continuation_indent_width() > 0 && !inside_stray_definition(state) {
        return Err(BailReason::ListItemUnsupportedBlockChild);
    }
    if state.list_depth > 0 {
        return Ok(());
    }
    let dest = state.cell_or_output_mut();
    if dest.len() > 1 && dest.ends_with('\n') && !dest.ends_with("\n\n") {
        dest.push('\n');
    }
    Ok(())
}

/// Whether a heading is open inside the current table cell.
fn in_heading(state: &Tier1State) -> bool {
    state
        .stack
        .iter()
        .rev()
        .take_while(|frame| !matches!(frame.spec.kind, TagKind::TableCell { .. }))
        .any(|frame| matches!(frame.spec.kind, TagKind::Heading(_)))
}

/// Tier-2's inline-element test, which decides what counts as inline content after a block.
fn is_inline_tag(name_lower: &[u8]) -> bool {
    std::str::from_utf8(name_lower).is_ok_and(crate::converter::main_helpers::is_inline_element)
}

/// Tier-2's block-level test, which decides what counts as the block before inline content.
fn is_block_tag(name_lower: &[u8]) -> bool {
    std::str::from_utf8(name_lower).is_ok_and(crate::converter::utility::content::is_block_level_element)
}
