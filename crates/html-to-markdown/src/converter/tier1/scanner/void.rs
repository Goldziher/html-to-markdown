/// ~keep Whether the current break is the first content inside inline buffers at paragraph start.
fn inline_break_starts_paragraph(state: &Tier1State, buffer_len: usize) -> bool {
    let Some((paragraph_index, paragraph_start)) = state
        .stack
        .iter()
        .enumerate()
        .rev()
        .find(|(_, frame)| matches!(frame.spec.kind, TagKind::Paragraph))
        .map(|(index, frame)| (index, frame.content_start))
    else {
        return false;
    };
    let mut expected_start = paragraph_start;
    let mut found_inline_buffer = false;
    for frame in state.stack[paragraph_index + 1..]
        .iter()
        .filter(|frame| frame.own_buffer)
    {
        let marker_width = emitted_inline_marker_width(state, frame);
        if frame.content_start.checked_sub(marker_width) != Some(expected_start) {
            return false;
        }
        expected_start = frame.content_start;
        found_inline_buffer = true;
    }
    found_inline_buffer && expected_start == buffer_len
}

fn emitted_inline_marker_width(state: &Tier1State, frame: &OpenTag) -> usize {
    match frame.spec.kind {
        TagKind::Strong
            if !state.summary_at_top()
                && !frame
                    .prev_escape_ctx
                    .intersects(EscapeCtx::STRONG | EscapeCtx::CODE | EscapeCtx::PRE) =>
        {
            2
        }
        TagKind::Emphasis if !frame.prev_escape_ctx.intersects(EscapeCtx::CODE | EscapeCtx::PRE) => 1,
        TagKind::Strikethrough | TagKind::Inserted
            if !frame.prev_escape_ctx.intersects(EscapeCtx::CODE | EscapeCtx::PRE) =>
        {
            2
        }
        _ => 0,
    }
}

/// Emit a void element (no closing tag).
fn emit_void(
    state: &mut Tier1State,
    spec: &'static TagSpec,
    name_lower: &[u8],
    attrs: &[(&[u8], Option<&[u8]>)],
    html: &str,
    options: &ConversionOptions,
) -> Result<(), BailReason> {
    // ~keep A void element closes the "just closed a custom element" boundary
    // window too (see the field's doc comment on `Tier1State`).
    state.last_closed_custom_element = false;
    let input_type = (name_lower == b"input").then(|| find_attr(attrs, b"type").unwrap_or_default());
    if input_type.is_some_and(|input_type| input_type.eq_ignore_ascii_case(b"checkbox")) {
        // ~keep Tier-2 writes a list item that starts with a checkbox as a task item (issue #632).
        if state
            .stack
            .iter()
            .any(|frame| matches!(frame.spec.kind, TagKind::ListItem))
        {
            return Err(BailReason::ListItemCheckbox);
        }
        return Err(BailReason::InlineMarkerNotReproduced);
    }
    // ~keep Closes the "just emitted an <img>" window too (see
    // `Tier1State::last_emitted_was_img`); the `TagKind::Image` arm below
    // re-sets it to true after this reset runs.
    state.last_emitted_was_img = false;
    // ~keep A `<br>` ends the line of the text before it: a `'\n'` join still at the end would put
    // ~keep the marker on a line of its own, which cleanup turns into a paragraph break (issue
    // ~keep #683). The join goes before the block check below, which must not see a line end
    // ~keep Tier-2 never wrote (`<canvas>First\n</canvas><br>`).
    if let Some(join_end) = state.pending_newline_join.take() {
        let dest = state.cell_or_output_mut();
        if matches!(spec.kind, TagKind::LineBreak) && join_end == dest.len() {
            dest.pop();
        }
    }
    // ~keep In a cell a line break is a break of its own (issue #645).
    if std::mem::take(&mut state.last_closed_block)
        && is_inline_tag(name_lower)
        && !(state.in_table_cell() && matches!(spec.kind, TagKind::LineBreak))
    {
        separate_inline_after_block(state, options.br_in_tables)?;
    }
    state.last_closed_block = is_block_tag(name_lower);

    match spec.kind {
        TagKind::Hr => emit_hr(state)?,

        TagKind::LineBreak => emit_line_break(state, options)?,

        TagKind::Image => emit_image(state, attrs, html, options)?,

        TagKind::Ignored | TagKind::Inline | TagKind::Block => {}

        _ => {}
    }
    Ok(())
}

fn emit_hr(state: &mut Tier1State) -> Result<(), BailReason> {
    if state.in_summary()
        || state.in_table_caption()
        || state.stack.iter().any(|frame| {
            matches!(
                frame.spec.kind,
                TagKind::Strong | TagKind::Emphasis | TagKind::Strikethrough | TagKind::Inserted
            )
        })
    {
        return Err(BailReason::RuleBetweenInlineMarkers);
    }
    // ~keep Tier-2 starts a rule after an item's content at the item's content column
    // ~keep (issue #583); see `BailReason::ListItemUnsupportedBlockChild`.
    if !state.in_table_cell() && state.list_continuation_indent_width() > 0 && !inside_stray_definition(state) {
        if !line_is_bare_list_marker(&state.output) {
            return Err(BailReason::ListItemUnsupportedBlockChild);
        }
        // ~keep A rule at the marker is `___` on the marker line, as in Tier-2: `- ---` is
        // ~keep a rule of its own.
        state.output.push_str("___\n");
        return Ok(());
    }
    let in_cell = state.in_table_cell();
    let write_rule = |dest: &mut String| {
        // ~keep Tier-2 trims the space a cell break left before the rule (issue #628).
        if in_cell {
            crate::converter::main_helpers::trim_trailing_whitespace(dest);
        }
        if !dest.is_empty() && !dest.ends_with("\n\n") {
            if dest.ends_with('\n') {
                dest.push('\n');
            } else {
                dest.push_str("\n\n");
            }
        }
        dest.push_str("---\n");
    };
    if in_cell {
        with_cell_scratch(state, write_rule);
    } else {
        write_rule(state.cell_or_output_mut());
    }
    Ok(())
}

fn emit_line_break(state: &mut Tier1State, options: &ConversionOptions) -> Result<(), BailReason> {
    let in_code = state.escape_ctx.contains(EscapeCtx::CODE);
    let in_pre = state.escape_ctx.contains(EscapeCtx::PRE);
    let inside_link = !in_code && state.stack.iter().any(|frame| matches!(frame.spec.kind, TagKind::Link));
    if inside_link {
        state
            .cell_or_output_mut()
            .push_str(crate::converter::main_helpers::hard_break_marker(options));
    } else if in_code && !in_pre && state.escape_ctx.contains(EscapeCtx::HEADING) {
        let dest = state.cell_or_output_mut();
        crate::converter::main_helpers::trim_trailing_whitespace(dest);
        dest.push(' ');
    } else if in_pre {
        emit_pre_line_break(state, options.br_in_tables);
    } else if state.in_table_cell() {
        emit_cell_line_break(state, options.br_in_tables, in_code);
    } else if in_code {
        state.cell_or_output_mut().push('\n');
    } else {
        emit_regular_line_break(state, options);
    }
    Ok(())
}

fn emit_pre_line_break(state: &mut Tier1State, br_in_tables: bool) {
    if state.in_table_cell() && br_in_tables {
        let offset = state.cell_or_output_mut().len();
        state.pre_cell_break_offsets.push(offset);
    }
    state.cell_or_output_mut().push('\n');
}

fn emit_cell_line_break(state: &mut Tier1State, br_in_tables: bool, in_code: bool) {
    let emit_literal_br = br_in_tables && !in_code;
    let cell_is_empty = state.cell_or_output_mut().is_empty();
    with_cell_scratch(state, |dest| {
        crate::converter::main_helpers::trim_trailing_whitespace(dest);
        if emit_literal_br {
            dest.push_str("<br>");
        } else if !cell_is_empty {
            dest.push(' ');
        }
    });
}

fn emit_regular_line_break(state: &mut Tier1State, options: &ConversionOptions) {
    let paragraph_start = state
        .stack
        .iter()
        .rev()
        .find(|frame| matches!(frame.spec.kind, TagKind::Paragraph))
        .map(|frame| frame.content_start);
    let buffer_len = state.cell_or_output_mut().len();
    let starts_in_inline_buffer = inline_break_starts_paragraph(state, buffer_len);
    let dest = state.cell_or_output_mut();
    if paragraph_start != Some(dest.len()) && !starts_in_inline_buffer {
        crate::converter::main_helpers::trim_trailing_whitespace(dest);
        dest.push_str(crate::converter::main_helpers::hard_break_marker(options));
    }
}

fn emit_image(
    state: &mut Tier1State,
    attrs: &[(&[u8], Option<&[u8]>)],
    html: &str,
    options: &ConversionOptions,
) -> Result<(), BailReason> {
    let raw_src = find_attr(attrs, b"src").unwrap_or_default();
    if image_uses_lazy_source(attrs, raw_src) {
        return Err(BailReason::ImageLazyLoadSrc);
    }
    let src = decode_attr(raw_src)?;
    let src = state.resolve_url(&src).unwrap_or(src);
    let alt = decode_attr(find_attr(attrs, b"alt").unwrap_or_default())?;
    let title = find_attr(attrs, b"title")
        .map(decode_attr)
        .transpose()?
        .map(|title| crate::converter::inline::link::escape_markdown_title(&title).into_owned());
    let keep_as_markdown = should_keep_image_as_markdown(html, &state.stack, options);
    let dest = state.cell_or_output_mut();
    if keep_as_markdown {
        emit_markdown_image(dest, &src, &alt, title.as_deref(), options);
    } else {
        dest.push_str(&alt);
    }
    state.last_emitted_was_img = true;
    Ok(())
}

fn image_uses_lazy_source(attrs: &[(&[u8], Option<&[u8]>)], src: &[u8]) -> bool {
    let trimmed = src.trim_ascii();
    let is_placeholder = trimmed.is_empty()
        || trimmed
            .get(..5)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case(b"data:"));
    is_placeholder
        && [
            b"data-src".as_slice(),
            b"data-lazy-src",
            b"data-original",
            b"data-srcset",
            b"srcset",
        ]
        .iter()
        .any(|name| find_attr(attrs, name).is_some())
}

fn emit_markdown_image(dest: &mut String, src: &str, alt: &str, title: Option<&str>, options: &ConversionOptions) {
    let escaped_alt = crate::converter::utility::escaping::escape_image_alt(alt);
    dest.push_str("![");
    dest.push_str(&escaped_alt);
    dest.push_str("](");
    crate::converter::inline::link::append_url_destination(dest, src, options.url_escape_style, title.is_some());
    if let Some(title) = title {
        dest.push_str(" \"");
        dest.push_str(title);
        dest.push('"');
    }
    dest.push(')');
}

/// Decide whether an `<img>` should be emitted as `![alt](src)` markdown.
///
/// Mirrors the Tier-2 logic in `converter.rs` for every feature combination:
/// - outside a heading → always emit a markdown image;
/// - inside a heading → emit markdown only when a matching heading or link
///   ancestor is in `keep_inline_images_in`, otherwise emit alt text only.
///
/// Ancestor matching is ASCII-case-insensitive so callers may supply "H1" or
/// "h1" interchangeably.
#[inline]
#[allow(clippy::missing_const_for_fn)]
fn should_keep_image_as_markdown(html: &str, stack: &[OpenTag], options: &ConversionOptions) -> bool {
    keep_inline_image_for_ancestors(html.as_bytes(), stack, &options.keep_inline_images_in)
}

/// Return `true` when the `<img>` should be emitted as `![alt](src)` markdown.
///
/// Mirrors the Tier-2 logic in `converter.rs`: images outside headings are kept
/// as markdown, while images inside headings require a matching heading or link
/// ancestor in `keep_inline_images_in`. The nearest heading OR link ancestor decides: if it is a link
/// (`<a>`) whose tag name is in the list, the image is kept regardless of any
/// heading further out (#492, mirroring `ctx.link_allow_inline_images` in
/// `handlers/link.rs`); if it is a link NOT in the list, that link imposes no
/// restriction of its own and the scan continues outward to the next heading
/// or link ancestor; if it is a heading, that heading's own list membership
/// decides and the scan stops there, matching the pre-#492 rule.
///
/// The comparison is ASCII-case-insensitive on both the stack name bytes and the
/// user-supplied strings, so callers may supply "H1" or "h1" interchangeably.
fn keep_inline_image_for_ancestors(input: &[u8], stack: &[OpenTag], keep: &[String]) -> bool {
    // ~keep No `keep.is_empty()` short-circuit. One stood here returning `true`, described as
    // "the Tier-2 default", but Tier-2 strips an image in a heading to its alt text
    // exactly when the list does not name that heading -- and an empty list names
    // nothing. Falling through handles both cases correctly: the heading branch's inner
    // loop matches nothing and returns `false`, and a document with no heading ancestor
    // still reaches the `true` at the end. Unreachable until issue #494 removed the
    // entity bail that had been sending these documents to Tier 2 anyway.
    for frame in stack.iter().rev() {
        if matches!(frame.spec.kind, TagKind::Heading(_)) {
            let name = &input[frame.name_range.clone()];
            for keep_name in keep {
                if eq_ascii_ignore_case(name, keep_name.as_bytes()) {
                    return true;
                }
            }
            return false;
        }
        // ~keep #492: a link ancestor whose tag name is in the keep list keeps the image as
        // markdown outright (mirrors `ctx.link_allow_inline_images` in Tier-2, which is
        // OR'd into `keep_as_markdown` independent of any heading). A non-matching link
        // imposes no restriction of its own -- unlike a heading, it does not stop the
        // scan -- so a heading further out still gets to decide. A link frame OUTSIDE a
        // heading frame is unreachable here: `<a>…<h1>` (heading opening inside a link)
        // already bails to Tier-2 before this scanner runs (scanner.rs:1111-1128), so
        // any `Link` frame this loop sees is nested INSIDE whichever `Heading` frame, if
        // any, sits further down this same stack.
        if matches!(frame.spec.kind, TagKind::Link) {
            let name = &input[frame.name_range.clone()];
            if keep
                .iter()
                .any(|keep_name| eq_ascii_ignore_case(name, keep_name.as_bytes()))
            {
                return true;
            }
        }
    }
    // ~keep No heading or matching-link ancestor at all: no restriction applies — emit
    // markdown image.
    // This matches Tier-2 behaviour: the `keep_inline_images_in` guard only
    // fires when `ctx.in_heading` is true.
    true
}

/// Byte-level ASCII case-insensitive comparison — no allocation.
fn eq_ascii_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Returns `true` when `bytes[pos..]` opens a `<script` or `<style` tag with no
/// separating whitespace — i.e. a second raw-text-ignored element sitting directly
/// adjacent to the one the scanner just finished skipping.  See the bail site in
/// the `TagKind::Ignored`-and-`is_rawtext` branch above for why this forces a
/// Tier-2 fallback rather than being handled inline.
fn is_adjacent_rawtext_ignored_open(bytes: &[u8], pos: usize) -> bool {
    const CANDIDATES: [&[u8]; 2] = [b"script", b"style"];
    if bytes.get(pos) != Some(&b'<') {
        return false;
    }
    let name_start = pos + 1;
    for name in CANDIDATES {
        let name_end = name_start + name.len();
        if bytes.len() < name_end {
            continue;
        }
        if !bytes[name_start..name_end].eq_ignore_ascii_case(name) {
            continue;
        }
        // ~keep Require a valid tag-name terminator so `<scriptx>` doesn't match.
        if let Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/') = bytes.get(name_end) {
            return true;
        }
    }
    false
}

fn emit_close(
    state: &mut Tier1State,
    tag_name_bytes: &[u8],
    options: &ConversionOptions,
    table_probes: &mut Vec<TableLayoutProbe>,
) -> Result<(), BailReason> {
    let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
    let name_lower = lowercase_into(tag_name_bytes, &mut name_buf);

    // ~keep Custom element close tags (e.g. `</x-foo>`) use the same static Inline
    // spec as their corresponding open tag.  All other unknown close tags bail.
    let spec: &'static TagSpec = if name_lower.contains(&b'-') {
        &CUSTOM_ELEMENT_INLINE_SPEC
    } else {
        match tier1::lookup(name_lower) {
            Some(s) => s,
            None => {
                return Err(BailReason::UnknownCustomElement {
                    name: bytes_to_string(tag_name_bytes).into(),
                    offset: 0,
                });
            }
        }
    };

    // ~keep Closing any tag ends the "just emitted an <img>" window too (see
    // `Tier1State::last_emitted_was_img`) — an intervening close means the
    // two images are not both unwrapped direct siblings any more.
    state.last_emitted_was_img = false;

    while let Some(top) = state.stack.last() {
        if kinds_match(&top.spec.kind, &spec.kind) {
            break;
        }
        if top.spec.optional_close.is_some() {
            emit_close_for_implicit(state, options, table_probes)?;
        } else {
            break;
        }
    }

    // ~keep Pop the matching frame from the open-tag stack.
    // Tier-2 is lenient about mismatched tags; for M3c we bail.
    let actual_depth = state.stack.len() as u8;
    let frame = pop_matching_frame(&mut state.stack, spec).ok_or_else(|| BailReason::DepthMismatch {
        tag: bytes_to_string(name_lower),
        expected: 1,
        actual: actual_depth,
    })?;

    state.escape_ctx = frame.prev_escape_ctx;
    state.last_closed_custom_element = std::ptr::eq(spec, &raw const CUSTOM_ELEMENT_INLINE_SPEC);
    if matches!(spec.kind, TagKind::DefinitionTerm | TagKind::DefinitionDescription) {
        trim_start_of_cell_content(state, frame.content_start)?;
    }
    check_whitespace_led_block_in_cell(state, &frame)?;

    dispatch_close(state, spec, &frame, name_lower, options, table_probes)?;
    // ~keep An inline element whose last content is a block ends in that block too (issue #585).
    // ~keep In a cell Tier-2 writes no break after a nested table (see `separate_from_block`).
    let breaks_after = is_block_tag(name_lower) && !(matches!(spec.kind, TagKind::Table) && state.in_table_cell());
    state.last_closed_block = breaks_after || (state.last_closed_block && is_inline_tag(name_lower));

    Ok(())
}

fn dispatch_close(
    state: &mut Tier1State,
    spec: &TagSpec,
    frame: &OpenTag,
    name_lower: &[u8],
    options: &ConversionOptions,
    table_probes: &mut Vec<TableLayoutProbe>,
) -> Result<(), BailReason> {
    match spec.kind {
        TagKind::Paragraph => close_paragraph(state),
        TagKind::Heading(n) => close_heading(state, frame, n, false, options)?,
        TagKind::Blockquote => close_blockquote(state, frame, options),
        TagKind::Pre => close_pre(state, frame, options),
        TagKind::Strong if suppress_close_marker(state, EscapeCtx::STRONG, true) => {}
        TagKind::Strong => close_inline_marker(state, frame, "**")?,
        TagKind::Emphasis if suppress_close_marker(state, EscapeCtx::empty(), false) => {}
        TagKind::Emphasis => close_inline_marker(state, frame, "*")?,
        TagKind::Strikethrough if suppress_close_marker(state, EscapeCtx::empty(), false) => {}
        TagKind::Strikethrough => close_inline_marker(state, frame, "~~")?,
        TagKind::Inserted if suppress_close_marker(state, EscapeCtx::empty(), false) => {}
        TagKind::Inserted => close_inline_marker(state, frame, "==")?,
        TagKind::Code => close_code(state, frame, matches!(name_lower, b"kbd" | b"samp"), options)?,
        TagKind::Link => close_link(state, frame, options)?,
        TagKind::List(ListKind::Definition) => close_dl(state, frame, options),
        TagKind::List(kind) => close_list(state, kind),
        TagKind::ListItem => close_list_item(state, frame)?,
        TagKind::DefinitionTerm => close_dt(state),
        TagKind::DefinitionDescription => close_dd(state),
        TagKind::Table => close_table(state, options, table_probes)?,
        TagKind::TableHead => close_table_head(state),
        TagKind::TableBody => close_table_body(state),
        TagKind::TableRow => close_table_row(state),
        TagKind::TableCell { .. } => close_table_cell(state, false)?,
        TagKind::TableCaption => close_table_caption(state),
        TagKind::Block => close_block_container(state, frame, name_lower),
        TagKind::Summary => close_summary(state, frame),
        TagKind::Figcaption => close_figcaption(state, frame),
        TagKind::Button => close_button(state, frame),
        TagKind::Inline if name_lower == b"abbr" => close_abbreviation(state, frame),
        TagKind::Hr
        | TagKind::TableFoot
        | TagKind::Inline
        | TagKind::LineBreak
        | TagKind::Image
        | TagKind::RawText(_)
        | TagKind::Ignored => {}
    }
    Ok(())
}

fn suppress_close_marker(state: &Tier1State, nested_context: EscapeCtx, suppress_in_summary: bool) -> bool {
    (suppress_in_summary && state.summary_at_top())
        || (!nested_context.is_empty() && state.escape_ctx.contains(nested_context))
        || state.escape_ctx.intersects(EscapeCtx::CODE | EscapeCtx::PRE)
}

fn close_abbreviation(state: &mut Tier1State, frame: &OpenTag) {
    let Some(Some(title)) = state.abbr_titles.pop() else {
        return;
    };
    let dest = state.cell_or_output_mut();
    let content_start = clamp_to_char_boundary(dest, frame.content_start);
    let trailing_start = content_start + dest[content_start..].trim_end().len();
    let trailing = dest[trailing_start..].to_owned();
    dest.truncate(trailing_start);
    dest.push_str(" (");
    dest.push_str(&title);
    dest.push(')');
    dest.push_str(&trailing);
}

/// Append a paragraph-break separator after a generic block container close
/// (`<div>`, `<section>`, etc.) when it produced visible content.
///
/// Without this Tier-1 emits adjacent block content with no separator
/// (e.g. `[image-link](href)EN` instead of `[image-link](href)\n\nEN`),
/// diverging from Tier-2 which always emits `\n\n` after a block-with-content
/// close (see Tier-2 `block/div.rs`).  Skipped inside table cells and inline
/// contexts where the surrounding code already handles spacing.
fn close_block_container(state: &mut Tier1State, frame: &OpenTag, name_lower: &[u8]) {
    if block_container_is_passthrough(name_lower) {
        // ~keep Mirrors the open-side skip in `emit_open`'s `TagKind::Block` arm:
        // Tier-2's catch-all handler for these names never emits a trailing
        // separator either.
        return;
    }
    if state.in_table_cell() || start_line_in_pre(state, name_lower) {
        return;
    }
    let buf = state.cell_or_output_mut();
    if buf.len() <= frame.content_start {
        return;
    }
    // ~keep Drop trailing horizontal whitespace (left over from inter-tag whitespace
    // preservation) before emitting the block separator.  Same rationale as
    // `ensure_blank_line` (Phase U-2).
    while buf.ends_with(' ') || buf.ends_with('\t') {
        buf.pop();
    }
    if buf.ends_with("\n\n") {
        return;
    }
    if buf.ends_with('\n') {
        buf.push('\n');
    } else {
        buf.push_str("\n\n");
    }
}

// ~keep ── Summary strong-wrap (Phase R) ────────────────────────────────────────────

/// Open a `<summary>` element.
///
/// Push a fresh accumulation buffer so all child text collects here instead
/// of in the outer destination (main output, table cell, or caption).
/// The summary buffer has the highest priority in `cell_or_output_mut`, so
/// even when inside a table cell the children write to this buffer rather
/// than the cell buffer.  This matches Tier-2's `handle_summary` which
/// always processes children into a local `content` buffer then wraps with
/// `**…**\n\n` before writing to the outer output.
///
/// No leading separator is emitted on open; deferred to `close_summary`
/// once we know whether the content is non-empty.
fn open_summary(state: &mut Tier1State) {
    state.push_summary_buf(crate::converter::tier1::state::WrapKind::Summary);
}

/// Close a `<summary>` element.
///
/// Pops the accumulation buffer (if any), trims it, and emits
/// `**{trimmed}**\n\n` into the parent destination (main output, an outer
/// summary buffer, a table cell, or a caption).
///
/// Mirrors Tier-2's `handle_summary` (semantic/summary.rs:138–249):
/// - collect children with `in_strong: true` (block children render inline)
/// - trim
/// - emit `**…**\n\n`
fn close_summary(state: &mut Tier1State, _frame: &OpenTag) {
    // ~keep Pop the buffer we pushed in open_summary.
    let buf = match state.pop_summary_buf() {
        Some(b) => b,
        None => return,
    };
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return;
    }
    // ~keep Acquire the parent destination.  Because we already popped the buffer
    // above, cell_or_output_mut now returns the next-outer target — which may
    // be the table cell buffer (when the summary was inside a <td>), an outer
    // summary buffer, or the main output.
    // ~keep
    // ~keep Check whether we're emitting into a table cell BEFORE borrowing `dest`,
    // so we can decide whether to add a leading separator without conflicting
    // with the mutable borrow.
    let writing_to_cell = state.in_table_cell();
    let dest = state.cell_or_output_mut();
    // ~keep Ensure a blank-line separator before the summary block when there is
    // preceding content and we're NOT writing to a table cell (cells are
    // rendered to a single line; block separators would be collapsed anyway).
    if !writing_to_cell && !dest.is_empty() && !dest.ends_with("\n\n") {
        if dest.ends_with('\n') {
            dest.push('\n');
        } else {
            dest.push_str("\n\n");
        }
    }
    if trimmed.contains("\n\n") && crate::converter::inline::wrapped::block_runs_are_plain(trimmed) {
        dest.push_str(&crate::converter::inline::wrapped::wrap_block_runs(trimmed, "**", "**"));
    } else {
        dest.push_str("**");
        dest.push_str(trimmed);
        dest.push_str("**");
    }
    dest.push_str("\n\n");
}

// ~keep ── Figcaption italic-wrap (Phase FF-2) ──────────────────────────────────────

/// Open a `<figcaption>` element.
///
/// Reuses the summary accumulation buffer stack — children write into it,
/// `close_figcaption` pops + wraps with `*…*\n\n` (vs Summary's `**…**`).
fn open_figcaption(state: &mut Tier1State) {
    state.push_summary_buf(crate::converter::tier1::state::WrapKind::Figcaption);
}

/// Close a `<figcaption>` element.
///
/// Mirrors Tier-2's `semantic/figure.rs::handle_figcaption`:
/// - collect children into a local buffer
/// - trim
/// - prepend single-space-or-blank-line separator
/// - emit `*{trimmed}*\n\n`
///
/// An empty/whitespace-only caption emits nothing (Tier-2 returns early).
fn close_figcaption(state: &mut Tier1State, _frame: &OpenTag) {
    let buf = match state.pop_summary_buf() {
        Some(b) => b,
        None => return,
    };
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return;
    }
    let writing_to_cell = state.in_table_cell();
    let dest = state.cell_or_output_mut();
    // ~keep Phase FF-2: trim trailing horizontal whitespace introduced by
    // Phase U-2's inter-tag-whitespace preservation, so the block
    // separator (\n\n) doesn't sit after a stray space.  Tier-2 does
    // not emit that space when the figcaption follows inline content.
    while dest.ends_with(' ') || dest.ends_with('\t') {
        dest.pop();
    }
    if !writing_to_cell && !dest.is_empty() && !dest.ends_with("\n\n") {
        if dest.ends_with('\n') {
            dest.push('\n');
        } else {
            dest.push_str("\n\n");
        }
    }
    dest.push('*');
    dest.push_str(trimmed);
    dest.push_str("*\n\n");
}

/// Close a `<button>` (Phase T).  When the button produced visible content, write the space
/// that keeps it a word of its own before it and `\n\n` after it.  The `\n\n` is skipped in
/// table cells (cells stay one logical line).
///
/// Mirrors Tier-2 `form/elements.rs`'s `write_line_end_control`, with the same `ControlStart`.
fn close_button(state: &mut Tier1State, frame: &OpenTag) {
    let dest = state.cell_or_output_mut();
    let content_start = clamp_to_char_boundary(dest, frame.content_start);
    crate::converter::form::spacing::ControlStart::at(dest, content_start).finish(dest);
    if state.in_table_cell() {
        return;
    }
    let dest = state.cell_or_output_mut();
    if dest.len() <= frame.content_start {
        return;
    }
    // ~keep Drop trailing horizontal whitespace from the inter-tag fix before the
    // block separator (Phase U-2).
    while dest.ends_with(' ') || dest.ends_with('\t') {
        dest.pop();
    }
    if dest.ends_with("\n\n") {
        return;
    }
    if dest.ends_with('\n') {
        dest.push('\n');
    } else {
        dest.push_str("\n\n");
    }
}

/// Clamp a stored byte offset (e.g. `OpenTag::content_start`, captured as
/// `buf.len()` when the tag opened) to a valid, in-bounds char boundary of
/// `buf` as it stands *now*.
///
/// `content_start` is read back at close time, sometimes against a different
/// buffer than the one it was captured against (`state.output` vs. the
/// current table-cell accumulator — see the `state.in_table_cell()` branches
/// throughout this file) or after other frames' close handlers have mutated
/// the buffer. On the correct path `content_start` is already valid, so this
/// is a no-op there (500k-case fuzzing under `TierStrategy::Auto` never hit a
/// clamp); it exists so a stale offset degrades to a clamped position instead
/// of an out-of-bounds or not-a-char-boundary panic in `&buf[start..]`,
/// `buf.truncate(start)`, or `buf.insert_str(start, …)`.
fn clamp_to_char_boundary(buf: &str, at: usize) -> usize {
    let mut at = at.min(buf.len());
    while at > 0 && !buf.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// Close an inline emphasis-style element (`<strong>`, `<em>`, `<b>`, `<i>`).
///
/// When the element produced no visible content (the source had `<strong></strong>`
/// or `<i>   </i>`), erase the open marker too instead of emitting an empty
/// `**` / `*` pair.  Tier-2's DOM walker reaches the same result by emitting
/// nothing for an empty inline node; the byte-equality oracle requires us to
/// match that.
fn close_inline_marker(state: &mut Tier1State, frame: &OpenTag, marker: &str) -> Result<(), BailReason> {
    let indent_width = state.list_continuation_indent_width();
    let buf = state.cell_or_output_mut();
    let mut content_start = clamp_to_char_boundary(buf, frame.content_start);
    normalize_marker_leading_break(buf, &mut content_start, marker, indent_width);
    if handle_empty_inline_body(buf, content_start, frame, marker)? {
        return Ok(());
    }
    if wrap_inline_block_runs(buf, content_start, marker) {
        return Ok(());
    }
    content_start = migrate_leading_inline_whitespace(buf, content_start, marker);
    if migrate_trailing_inline_whitespace(buf, content_start, marker) {
        return Ok(());
    }
    buf.push_str(marker);
    Ok(())
}

fn normalize_marker_leading_break(buf: &mut String, content_start: &mut usize, marker: &str, indent_width: usize) {
    if !buf[*content_start..].starts_with("  \n") {
        return;
    }
    let marker_start = clamp_to_char_boundary(buf, content_start.saturating_sub(marker.len()));
    let indent = std::iter::repeat_n(' ', indent_width).collect::<String>();
    let mut replacement = String::with_capacity(marker.len() + indent.len() * 2 + 3);
    if buf[..marker_start].ends_with('\n') {
        replacement.push_str(&indent);
        replacement.push_str("\\\n");
    } else {
        replacement.push_str("  \n");
    }
    replacement.push_str(&indent);
    replacement.push_str(marker);
    let mut body_start = *content_start + 3;
    while matches!(buf.as_bytes().get(body_start), Some(b' ' | b'\t')) {
        body_start += 1;
    }
    buf.replace_range(marker_start..body_start, &replacement);
    *content_start = marker_start + replacement.len();
}

fn handle_empty_inline_body(
    buf: &mut String,
    content_start: usize,
    frame: &OpenTag,
    marker: &str,
) -> Result<bool, BailReason> {
    let content_absent = buf.len() <= content_start;
    let whitespace_only = !content_absent
        && buf[content_start..]
            .bytes()
            .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'));
    if !content_absent && !whitespace_only {
        return Ok(false);
    }
    let was_whitespace_only = whitespace_only || (content_absent && frame.dropped_whitespace_only_text);
    if was_whitespace_only && matches!(marker, "**" | "*" | "~~" | "==") {
        return Err(BailReason::WhitespaceOnlyInlineEmphasis);
    }
    let marker_start = clamp_to_char_boundary(buf, content_start.saturating_sub(marker.len()));
    buf.truncate(marker_start);
    Ok(true)
}

fn wrap_inline_block_runs(buf: &mut String, content_start: usize, marker: &str) -> bool {
    let content = &buf[content_start..];
    if !content.contains("\n\n") || !crate::converter::inline::wrapped::block_runs_are_plain(content) {
        return false;
    }
    let content = content.to_owned();
    let marker_start = clamp_to_char_boundary(buf, content_start.saturating_sub(marker.len()));
    buf.truncate(marker_start);
    buf.push_str(&crate::converter::inline::wrapped::wrap_block_runs(
        &content, marker, marker,
    ));
    true
}

fn migrate_leading_inline_whitespace(buf: &mut String, content_start: usize, marker: &str) -> usize {
    let content = &buf[content_start..];
    let leading_len = content.len() - content.trim_start().len();
    if leading_len == 0 {
        return content_start;
    }
    let leading = content[..leading_len].to_owned();
    buf.replace_range(content_start..content_start + leading_len, "");
    let marker_start = clamp_to_char_boundary(buf, content_start.saturating_sub(marker.len()));
    buf.insert_str(marker_start, &leading);
    content_start + leading_len
}

fn migrate_trailing_inline_whitespace(buf: &mut String, content_start: usize, marker: &str) -> bool {
    let content = &buf[content_start..];
    let trailing_len = content.len() - content.trim_end().len();
    if trailing_len == 0 {
        return false;
    }
    let trailing_start = buf.len() - trailing_len;
    let trailing = buf[trailing_start..].to_owned();
    buf.truncate(trailing_start);
    buf.push_str(marker);
    buf.push_str(&trailing);
    true
}
