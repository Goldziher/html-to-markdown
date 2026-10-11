/// Implicitly close the top-of-stack frame without a matching `</tag>` in the
/// input.  Called by the M4 implicit-close loop when HTML5 optional-tag rules
/// require an open element to be closed before the next tag is pushed.
///
/// Mirrors `emit_close` but skips the stack-pop search (we always close the
/// literal top frame) and skips the tag-name lookup (we use the frame's spec
/// directly).
fn emit_close_for_implicit(
    state: &mut Tier1State,
    options: &ConversionOptions,
    table_probes: &mut Vec<TableLayoutProbe>,
) -> Result<(), BailReason> {
    let frame = state.stack.pop().ok_or_else(|| BailReason::DepthMismatch {
        tag: String::from("(implicit)"),
        expected: 1,
        actual: 0,
    })?;
    let spec = frame.spec;

    state.escape_ctx = frame.prev_escape_ctx;
    state.last_closed_custom_element = std::ptr::eq(spec, &raw const CUSTOM_ELEMENT_INLINE_SPEC);

    match spec.kind {
        TagKind::Paragraph => close_paragraph(state),
        TagKind::Heading(n) => close_heading(state, &frame, n, true, options)?,
        TagKind::Blockquote => close_blockquote(state, &frame, options.br_in_tables),
        TagKind::Pre => close_pre(state, &frame, options),
        // ~keep Strong: suppress close marker when inside summary, or when this
        // frame nested inside another `<strong>` and so never emitted an
        // open marker either (see open-side guard) — `state.escape_ctx` was
        // just restored to `frame.prev_escape_ctx` above.
        TagKind::Strong
            if state.summary_at_top()
                || state.escape_ctx.contains(EscapeCtx::STRONG)
                || state.escape_ctx.contains(EscapeCtx::CODE)
                || state.escape_ctx.contains(EscapeCtx::PRE) => {}
        TagKind::Strong => close_inline_marker(state, &frame, "**")?,
        TagKind::Emphasis
            if state.escape_ctx.contains(EscapeCtx::CODE) || state.escape_ctx.contains(EscapeCtx::PRE) => {}
        TagKind::Emphasis => close_inline_marker(state, &frame, "*")?,
        TagKind::Strikethrough
            if state.escape_ctx.contains(EscapeCtx::CODE) || state.escape_ctx.contains(EscapeCtx::PRE) => {}
        TagKind::Strikethrough => close_inline_marker(state, &frame, "~~")?,
        TagKind::Inserted
            if state.escape_ctx.contains(EscapeCtx::CODE) || state.escape_ctx.contains(EscapeCtx::PRE) => {}
        TagKind::Inserted => close_inline_marker(state, &frame, "==")?,
        TagKind::Code => close_code(state, &frame, false, options)?,
        TagKind::Link => close_link(state, &frame, options)?,
        TagKind::List(ListKind::Definition) => close_dl(state, &frame),
        TagKind::List(kind) => close_list(state, kind),
        TagKind::ListItem => close_list_item(state, &frame)?,
        TagKind::DefinitionTerm => close_dt(state),
        TagKind::DefinitionDescription => close_dd(state),
        TagKind::TableCell { .. } => close_table_cell(state, true, options)?,
        TagKind::TableRow => close_table_row(state),
        // ~keep Summary: pop accumulation buffer, trim, emit `**…**\n\n` (Phase R).
        TagKind::Summary => close_summary(state, &frame),
        // ~keep Figcaption: pop accumulation buffer, trim, emit `*…*\n\n` (Phase FF-2).
        TagKind::Figcaption => close_figcaption(state, &frame),
        // ~keep Button (Phase T): emit `\n\n` on EOF close just like explicit close.
        TagKind::Button => close_button(state, &frame),
        // ~keep An unclosed `<table>` at EOF (html5ever/tl both implicitly close every
        // open element there, per the loop's own doc comment) used to hit the
        // do-nothing arm below, discarding the WHOLE accumulated table --
        // including fully-formed rows -- instead of rendering it. `close_table`
        // already tolerates an incomplete/empty table (its `is_blank` check
        // bails rather than emitting), so it is exactly as safe to call here as
        // it is from `emit_close`'s explicit `</table>` arm.
        TagKind::Table => close_table(state, options, table_probes)?,
        TagKind::Block | TagKind::Inline => {}
        TagKind::LineBreak
        | TagKind::Image
        | TagKind::Hr
        | TagKind::TableHead
        | TagKind::TableBody
        | TagKind::TableFoot
        | TagKind::TableCaption
        | TagKind::RawText(_)
        | TagKind::Ignored => {}
    }

    Ok(())
}

fn close_paragraph(state: &mut Tier1State) {
    // ~keep When inside a table cell, `<p>` is transparent — no block separators.
    // Any inter-paragraph separators were already added as `<br>` at open time
    // by `open_paragraph`; `close_paragraph` does nothing in this context.
    if state.in_table_cell() {
        return;
    }
    // ~keep Tier-2 appends "\n\n" after paragraph content (always two newlines).
    // Matching this precisely is required for byte-equal output.
    trim_trailing_inline_whitespace(state);
    state.cell_or_output_mut().push_str("\n\n");
}

/// Close a heading element.
///
/// When `is_implicit` is true the empty-heading guard is skipped: implicitly
/// closed headings have already had their content flushed through the normal
/// path, so we just prepend the prefix unconditionally.
fn close_heading(
    state: &mut Tier1State,
    frame: &OpenTag,
    n: u8,
    is_implicit: bool,
    options: &ConversionOptions,
) -> Result<(), BailReason> {
    // ~keep When inside a table cell, Tier-2 emits the heading text directly into
    // the cell accumulator — no `#` prefix, no block separators.  The
    // `frame.content_start` is a position in the CELL buffer (set by
    // `cell_or_output_mut().len()` at emit_open time), so all position
    // arithmetic must use the cell buffer, not `state.output`.
    if close_heading_in_cell(state, frame, is_implicit, options.br_in_tables) {
        return Ok(());
    }

    trim_trailing_inline_whitespace(state);
    // ~keep All buffer touches below go through `cell_or_output_mut` rather than
    // `state.output` directly. A heading inside `<summary>`/`<figcaption>`
    // is not a table cell (the early return above doesn't catch it) but its
    // `frame.content_start` was captured from the active wrap buffer's
    // length (see the open-tag push site), not `state.output`'s — using
    // `state.output` here would insert the `#` prefix at that same small
    // offset into the real, much longer document output instead, splicing
    // it into the middle of unrelated already-emitted text. See
    // `open_heading`'s matching note.
    let buf = state.cell_or_output_mut();
    let content_start = clamp_to_char_boundary(buf, frame.content_start);

    if !is_implicit && discard_empty_heading(buf, content_start) {
        return Ok(());
    }

    normalize_heading_body(buf, content_start);
    trim_heading_body(buf, content_start);
    escape_heading_closing_sequence(buf, content_start, options);

    let prefix = heading_prefix(n);
    buf.insert_str(content_start, prefix);
    // ~keep Tier-2 leaves a blank line ("\n\n") after a heading. A
    // following paragraph's "\n\n" guard then finds it already and appends
    // nothing, yielding the expected single blank line.
    ensure_blank_line_buf(state.cell_or_output_mut());
    Ok(())
}

fn close_heading_in_cell(state: &mut Tier1State, frame: &OpenTag, is_implicit: bool, br_in_tables: bool) -> bool {
    if !state.in_table_cell() {
        return false;
    }
    let cell_buf = state.cell_or_output_mut();
    while cell_buf.len() > frame.content_start && (cell_buf.ends_with(' ') || cell_buf.ends_with('\t')) {
        cell_buf.pop();
    }
    if !is_implicit {
        let content_start = clamp_to_char_boundary(cell_buf, frame.content_start);
        if cell_buf[content_start..].trim().is_empty() {
            cell_buf.truncate(content_start);
        }
    }
    separate_closed_block_in_cell(state, frame.content_start, br_in_tables);
    true
}

fn discard_empty_heading(buf: &mut String, content_start: usize) -> bool {
    if !buf[content_start..].trim().is_empty() {
        return false;
    }
    buf.truncate(content_start);
    let trimmed_len = buf.trim_end_matches('\n').len();
    if trimmed_len > 0 {
        buf.truncate(trimmed_len);
        buf.push('\n');
    } else {
        buf.clear();
    }
    true
}

fn normalize_heading_body(buf: &mut String, content_start: usize) {
    if !buf[content_start..].contains('\n') {
        return;
    }
    let content = buf[content_start..].to_owned();
    let mut normalized = String::with_capacity(content.len());
    let mut prev_was_space = false;
    for ch in content.chars() {
        let is_ws = matches!(ch, ' ' | '\t' | '\n' | '\r');
        if is_ws && !prev_was_space {
            normalized.push(' ');
        } else if !is_ws {
            normalized.push(ch);
        }
        prev_was_space = is_ws;
    }
    buf.truncate(content_start);
    buf.push_str(normalized.trim_end());
}

fn trim_heading_body(buf: &mut String, content_start: usize) {
    let trailing_start = buf[content_start..]
        .char_indices()
        .rev()
        .find(|&(_, character)| !character.is_whitespace())
        .map_or(content_start, |(index, character)| {
            content_start + index + character.len_utf8()
        });
    buf.truncate(trailing_start);
    let leading_len = buf[content_start..]
        .char_indices()
        .find(|&(_, character)| !character.is_whitespace())
        .map_or(buf.len() - content_start, |(index, _)| index);
    buf.replace_range(content_start..content_start + leading_len, "");
}

fn escape_heading_closing_sequence(buf: &mut String, content_start: usize, options: &ConversionOptions) {
    if options.output_format != crate::options::OutputFormat::Markdown {
        return;
    }
    if let Some(offset) = crate::converter::utility::escaping::atx_closing_sequence_offset(&buf[content_start..]) {
        buf.insert(content_start + offset, '\\');
    }
}

fn close_blockquote(state: &mut Tier1State, frame: &OpenTag, br_in_tables: bool) {
    // ~keep Phase GG follow-up: inside a table cell `frame.content_start` indexes
    // into the cell buffer, not `state.output`.  Don't prefix `> ` — Tier-2
    // also sheds the quote marker inside cells (issue #647).
    if close_blockquote_in_cell(state, frame, br_in_tables) {
        return;
    }
    let content_start = clamp_to_char_boundary(&state.output, frame.content_start);
    let content = prepare_blockquote_content(&state.output[content_start..]);
    let prefixed = prefix_blockquote_lines(&content);
    state.output.truncate(content_start);
    separate_blockquote_prefix(&mut state.output, frame.prev_escape_ctx);
    push_list_item_continuation_lines(state, &prefixed);

    // ~keep Tier-2's `handle_blockquote` (blockquote.rs:225-232) unconditionally trims
    // every trailing newline and re-pushes exactly "\n\n" after EVERY close --
    // top-level or nested, and regardless of what follows (a sibling, or nothing
    // at all; a trailing "\n\n" at document end is trimmed back down by the
    // shared end-of-document normalization both tiers already share). Existing
    // tests only covered "nothing follows the blockquote", where a missing
    // trailing separator here is invisible. `<blockquote>a</blockquote>after`
    // (top-level) and `<blockquote><blockquote>a</blockquote>b</blockquote>`
    // (nested) both need it: Tier-2 emits a blank line before `after` / a bare
    // `>` line before `b`. Gated on `!in_list_item` to mirror Tier-2's own
    // `!ctx.in_table_cell && !ctx.in_list_item` guard -- `in_table_cell` already
    // returned early above. In practice every list-item blockquote already bails
    // via `BailReason::ListItemUnsupportedBlockChild` before reaching here; the
    // guard is kept so this stays correct if that bail is ever narrowed.
    let in_list_item = state
        .stack
        .iter()
        .any(|open_frame| matches!(open_frame.spec.kind, TagKind::ListItem));
    if !in_list_item {
        state.ensure_blank_line();
    }
}

fn close_blockquote_in_cell(state: &mut Tier1State, frame: &OpenTag, br_in_tables: bool) -> bool {
    if !state.in_table_cell() {
        return false;
    }
    if !state.escape_ctx.contains(EscapeCtx::CODE) {
        separate_closed_block_in_cell(state, frame.content_start, br_in_tables);
        return true;
    }
    let cell_buf = state.cell_or_output_mut();
    let content_start = clamp_to_char_boundary(cell_buf, frame.content_start);
    let content = cell_buf.split_off(content_start);
    if !content.trim().is_empty() {
        if !cell_buf.is_empty() && !cell_buf.ends_with('\n') {
            cell_buf.push('\n');
        }
        cell_buf.push_str(content.trim());
        cell_buf.push('\n');
    }
    true
}

fn prepare_blockquote_content(content: &str) -> String {
    let mut content = content.to_owned();
    crate::converter::main_helpers::trim_trailing_whitespace(&mut content);
    let leading_len = content
        .char_indices()
        .find(|&(_, character)| !character.is_whitespace())
        .map_or(content.len(), |(index, _)| index);
    content.replace_range(0..leading_len, "");
    content
}

fn separate_blockquote_prefix(output: &mut String, previous_context: EscapeCtx) {
    if previous_context.contains(EscapeCtx::BLOCKQUOTE) {
        if output.ends_with("\n\n") {
            output.pop();
        }
        return;
    }
    if output.is_empty() {
        return;
    }
    if output.ends_with("\n\n") {
        output.pop();
    } else if output.ends_with('\n') {
        output.push('\n');
    } else {
        output.push_str("\n\n");
    }
}

/// Whether Tier-2 renders the content of the element `name_lower` of kind `kind`, opened in the
/// escape context `ctx`, into a buffer of its own rather than into its parent's.
///
/// A cell break checks and trims only that buffer in Tier-2, so Tier-1 must not look past the
/// start of such an element in the flat cell buffer (see [`cell_scratch_start`]).
fn renders_into_own_buffer(kind: TagKind, name_lower: &[u8], ctx: EscapeCtx) -> bool {
    match kind {
        // ~keep In a cell Tier-2 writes a figure, fieldset, dl, details, dialog, menu and form
        // ~keep straight into the cell. It gives a legend and a figure caption a buffer of their
        // ~keep own, but this scanner writes neither the way Tier-2 does there, so they are not
        // ~keep marked here.
        TagKind::Blockquote
        | TagKind::Heading(_)
        | TagKind::Pre
        | TagKind::DefinitionTerm
        | TagKind::DefinitionDescription => true,
        // ~keep Inside code Tier-2 writes these straight into the code span.
        TagKind::Code | TagKind::Strong | TagKind::Emphasis | TagKind::Strikethrough | TagKind::Inserted => {
            !ctx.contains(EscapeCtx::CODE)
        }
        _ => matches!(name_lower, b"sub" | b"sup" | b"abbr"),
    }
}

/// The start, in the current cell buffer, of the buffer Tier-2 writes the current content into:
/// the content start of the innermost element opened in this cell that has a buffer of its own,
/// or 0 for the cell itself.
fn cell_scratch_start(state: &Tier1State) -> usize {
    innermost_own_buffer(state).map_or(0, |frame| frame.content_start)
}

/// The innermost element opened in the current cell that has a buffer of its own.
fn innermost_own_buffer(state: &Tier1State) -> Option<&OpenTag> {
    state
        .stack
        .iter()
        .rev()
        .take_while(|frame| !matches!(frame.spec.kind, TagKind::TableCell { .. } | TagKind::Summary))
        .find(|frame| frame.own_buffer)
}

/// Whether Tier-2 lays out `name_lower` in the current table cell in a way this scanner does not
/// reproduce, so the cell goes to Tier-2 (issue #645): a block inside an inline element with a
/// buffer of its own, a line break at the start of such an element or of a definition term or
/// definition, a block at the start of a definition term or definition after other cell content, a
/// legend, a
/// figure caption or a label, and a sectioning element after other cell content, which Tier-2
/// separates with a blank line that the cell folds into two spaces. An inline element right
/// after a block in a cell is left to Tier-2 too (see `emit_open`), and so are a definition after
/// another definition in the cell and a quote or heading that starts with whitespace (see
/// `check_whitespace_led_block_in_cell`).
fn cell_needs_tier2(state: &mut Tier1State, spec: &TagSpec, name_lower: &[u8]) -> bool {
    if !state.in_table_cell() {
        return false;
    }
    if matches!(name_lower, b"legend" | b"figcaption" | b"label") {
        return true;
    }
    let definition = matches!(spec.kind, TagKind::DefinitionTerm | TagKind::DefinitionDescription);
    if definition && state.table_stack.last().is_some_and(|ts| ts.definition_in_cell) {
        return true;
    }
    let sectioning = matches!(
        name_lower,
        b"article" | b"section" | b"nav" | b"aside" | b"header" | b"footer" | b"main"
    );
    let in_inline_buffer = state
        .stack
        .iter()
        .rev()
        .take_while(|frame| !matches!(frame.spec.kind, TagKind::TableCell { .. } | TagKind::Summary))
        .any(|frame| frame.own_buffer && !frame.spec.is_block);
    let inline_buffer = innermost_own_buffer(state)
        .filter(|frame| {
            !frame.spec.is_block
                || matches!(
                    frame.spec.kind,
                    TagKind::DefinitionTerm | TagKind::DefinitionDescription
                )
        })
        .map(|frame| frame.content_start);
    let cell_buf = state.cell_or_output_mut();
    let buffer_so_far = inline_buffer.map(|start| &cell_buf[clamp_to_char_boundary(cell_buf, start)..]);
    let at_buffer_start = buffer_so_far.is_some_and(|text| text.trim().is_empty());
    let break_at_buffer_start = matches!(spec.kind, TagKind::LineBreak) && at_buffer_start;
    let block_at_buffer_start = spec.is_block && at_buffer_start && !cell_buf.trim().is_empty();
    (sectioning && !cell_buf.is_empty())
        || (spec.is_block && in_inline_buffer)
        || break_at_buffer_start
        || block_at_buffer_start
}

/// Drop the whitespace at the start of the content a definition term or definition wrote into a
/// table cell from `content_start`, as Tier-2 trims the buffer of each. Tier-2 separates a rule at
/// the start of a definition from the cell content before it, and this scanner cannot tell a rule
/// from `---` text there, so content that starts with whitespace and `---` goes to Tier-2.
fn trim_start_of_cell_content(state: &mut Tier1State, content_start: usize) -> Result<(), BailReason> {
    if !state.in_table_cell() {
        return Ok(());
    }
    let cell_buf = state.cell_or_output_mut();
    let start = clamp_to_char_boundary(cell_buf, content_start);
    let content = cell_buf[start..].trim_start();
    let leading = cell_buf.len() - start - content.len();
    if leading > 0 && content.starts_with("---") {
        return Err(BailReason::TableBlockChildInCell);
    }
    cell_buf.replace_range(start..start + leading, "");
    Ok(())
}

/// Leave a table cell to Tier-2 when a quote or heading in it closes with content after
/// whitespace at its start: Tier-2 keeps that whitespace in the element's own buffer and breaks
/// the cell twice before the content. A quote or heading with no other content writes nothing.
fn check_whitespace_led_block_in_cell(state: &mut Tier1State, frame: &OpenTag) -> Result<(), BailReason> {
    if !frame.starts_with_whitespace || !state.in_table_cell() {
        return Ok(());
    }
    let cell_buf = state.cell_or_output_mut();
    let start = clamp_to_char_boundary(cell_buf, frame.content_start);
    if cell_buf[start..].trim().is_empty() {
        return Ok(());
    }
    Err(BailReason::TableBlockChildInCell)
}

/// Run `write` on the part of the cell buffer Tier-2 writes the current content into, so a cell
/// break checks and trims only that content, as Tier-2's does (issue #645).
fn with_cell_scratch<R>(state: &mut Tier1State, write: impl FnOnce(&mut String) -> R) -> R {
    let start = cell_scratch_start(state);
    let cell_buf = state.cell_or_output_mut();
    let start = clamp_to_char_boundary(cell_buf, start);
    if start == 0 {
        return write(cell_buf);
    }
    let mut scratch = cell_buf.split_off(start);
    let result = write(&mut scratch);
    cell_buf.push_str(&scratch);
    result
}

/// Write the cell break at `at` in the cell buffer, between a block in a table cell and the cell
/// content before it, as Tier-2 does (issue #645).
///
/// Only the content written since the innermost element with a buffer of its own opened counts
/// (see [`cell_scratch_start`]). Inside a heading or code Tier-2 writes the content inline with no
/// break. Tier-2 trims the start of a block's content, so the content after `at` is trimmed too.
fn separate_block_in_cell_at(state: &mut Tier1State, at: usize, br_in_tables: bool) {
    if state.escape_ctx.intersects(EscapeCtx::HEADING | EscapeCtx::CODE) {
        return;
    }
    let scratch_start = cell_scratch_start(state);
    let cell_buf = state.cell_or_output_mut();
    let at = clamp_to_char_boundary(cell_buf, at);
    let scratch_start = clamp_to_char_boundary(cell_buf, scratch_start.min(at));
    if cell_buf[scratch_start..at].trim().is_empty() {
        return;
    }
    let content = cell_buf.split_off(at);
    crate::converter::main_helpers::separate_block_in_cell(cell_buf, br_in_tables);
    cell_buf.push_str(content.trim_start());
}

/// Separate the content of a block that closes in a table cell from the cell content before it.
fn separate_closed_block_in_cell(state: &mut Tier1State, content_start: usize, br_in_tables: bool) {
    let cell_buf = state.cell_or_output_mut();
    let content_start = clamp_to_char_boundary(cell_buf, content_start);
    if cell_buf[content_start..].trim().is_empty() {
        return;
    }
    separate_block_in_cell_at(state, content_start, br_in_tables);
}

fn close_pre(state: &mut Tier1State, frame: &OpenTag, options: &ConversionOptions) {
    use crate::options::CodeBlockStyle;
    // ~keep A pipe table cannot contain a fenced block, so render the preformatted
    // ~keep content as one or more code spans and keep real `<br>` nodes between spans.
    if close_pre_in_cell(state, frame, options) {
        return;
    }
    let content_start = clamp_to_char_boundary(&state.output, frame.content_start);
    let raw = state.output[content_start..].to_owned();
    state.output.truncate(content_start);
    // ~keep Render into a scratch buffer first, then (when inside a list item)
    // indent every physical line to the item's continuation column
    // before appending to `state.output` — see `push_list_item_continuation_lines`.
    let rendered = match options.code_block_style {
        CodeBlockStyle::Indented | CodeBlockStyle::Tildes => indent_pre_lines(&raw),
        CodeBlockStyle::Backticks => render_backtick_pre(&raw, state.pre_lang.take(), &options.code_language),
    };
    push_list_item_continuation_lines(state, &rendered);
    state.pre_lang = None;
}

fn close_pre_in_cell(state: &mut Tier1State, frame: &OpenTag, options: &ConversionOptions) -> bool {
    if !state.in_table_cell() {
        return false;
    }
    let break_offsets = std::mem::take(&mut state.pre_cell_break_offsets);
    let cell_buf = state.cell_or_output_mut();
    let content_start = clamp_to_char_boundary(cell_buf, frame.content_start);
    let content = cell_buf.split_off(content_start);
    let relative_offsets = break_offsets
        .into_iter()
        .filter_map(|offset| offset.checked_sub(content_start))
        .filter(|&offset| offset < content.len())
        .collect::<Vec<_>>();
    if !content.trim_matches('\n').is_empty() {
        crate::converter::main_helpers::separate_block_in_cell(cell_buf, options.br_in_tables);
        crate::converter::handlers::code_block::format_preformatted_cell_content(
            &content,
            &relative_offsets,
            cell_buf,
            options.br_in_tables,
            options.whitespace_mode == crate::options::WhitespaceMode::Strict,
        );
    }
    true
}

fn render_backtick_pre(raw: &str, language: Option<String>, default_language: &str) -> String {
    let fence_length = (longest_consecutive_backtick_run(raw) + 1).max(MIN_FENCE_LENGTH);
    let fence = std::iter::repeat_n('`', fence_length).collect::<String>();
    let language = language.as_deref().unwrap_or(default_language);
    let content = raw.strip_prefix('\n').unwrap_or(raw).trim_end_matches('\n');
    format!("{fence}{language}\n{content}\n{fence}\n\n")
}

/// Indent a text node that starts a fresh, still-unindented physical line
/// inside a list item (e.g. sibling text right after a heading, which only
/// emits a single trailing newline rather than a blank line) to the item's
/// continuation column — the same indent every block handler
/// (`push_list_item_continuation_lines`) adds before its own first line.
/// Without it the line lands flush left and the item (and the rest of the
/// list) falls out of the list on reparse (CommonMark spec example 300).
///
/// Excluded from verbatim contexts (checked by the caller: `in_pre`/`in_code`
/// text never reaches this point) and from contexts that accumulate into a
/// detached scratch buffer rather than the real document (`in_table_cell`,
/// `in_summary`, `in_table_caption`), where `cell_or_output_mut()` is not the
/// list item's own accumulating text and indenting it would corrupt literal
/// or already-wrapped content instead. Mirrors Tier-2's `text_node.rs`
/// (`ctx.in_list_item && output.ends_with('\n') && !output.ends_with("\n\n")`).
fn indent_fresh_list_item_text_line(state: &mut Tier1State) {
    if state.in_table_cell() || state.in_summary() || state.in_table_caption() {
        return;
    }
    let in_list_item = state
        .stack
        .iter()
        .any(|frame| matches!(frame.spec.kind, TagKind::ListItem));
    if !in_list_item {
        return;
    }
    let indent_width = state.list_continuation_indent_width();
    if indent_width == 0 {
        return;
    }
    let dest = state.cell_or_output_mut();
    if dest.ends_with('\n') && !dest.ends_with("\n\n") {
        let indent: String = std::iter::repeat_n(' ', indent_width).collect();
        dest.push_str(&indent);
    }
}

fn indent_list_item_text_continuation_lines(buffer: &mut String, from: usize, indent_width: usize) {
    let emitted = &buffer[from..];
    if indent_width == 0 || !emitted.contains('\n') || emitted.contains("\n\n") {
        return;
    }

    // ~keep Tier-2 gives every source-line continuation inside one text node the
    // ~keep list item's content indent (#637). Whitespace collapse has removed the
    // ~keep source indentation by this point, so restore the structural indent here.
    let indent = " ".repeat(indent_width);
    let mut indented = String::with_capacity(emitted.len() + indent.len());
    let mut lines = emitted.split_inclusive('\n').peekable();
    while let Some(line) = lines.next() {
        indented.push_str(line);
        if lines.peek().is_some() {
            indented.push_str(&indent);
        }
    }
    buffer.truncate(from);
    buffer.push_str(&indented);
}

/// Append `rendered` (a fully-formatted block's text, possibly spanning
/// several physical lines) to `state.output`, indenting every line to the
/// innermost open list item's continuation column when inside one.
///
/// CommonMark matches list containment per physical line (spec examples
/// 263, 273, 274, 318, 324): a non-blank line that isn't indented to the
/// item's continuation width falls out of the item — and the list — on
/// reparse. Mirrors Tier-2's `format_code_block_in_list_item`
/// (handlers/code_block.rs): the very first line skips the indent when it
/// is NOT a continuation (i.e. it sits directly after the item's own
/// marker, like `- ` + the block's first line, and already starts at the
/// right column); every other non-blank line always gets indented. Blank
/// lines are left bare — an indented blank line would just be trailing
/// whitespace.
fn push_list_item_continuation_lines(state: &mut Tier1State, rendered: &str) {
    let indent_width = state.list_continuation_indent_width();
    if indent_width == 0 {
        state.output.push_str(rendered);
        return;
    }
    // ~keep A plain suffix check like `ends_with("* ")` also matches the closing
    // "**"/"*" of `<strong>`/`<em>` immediately followed by a migrated trailing
    // space, indistinguishable from a real bare bullet by suffix alone. Reuse
    // `line_is_bare_list_marker` (this file, mirrors Tier-2's
    // `list::utils::line_is_bare_list_marker`) instead of repeating that ambiguity.
    let is_continuation = !state.output.is_empty() && !line_is_bare_list_marker(&state.output);
    let indent: String = std::iter::repeat_n(' ', indent_width).collect();
    for (index, segment) in rendered.split_inclusive('\n').enumerate() {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        if line.is_empty() || (index == 0 && !is_continuation) {
            state.output.push_str(segment);
        } else {
            state.output.push_str(&indent);
            state.output.push_str(segment);
        }
    }
}
