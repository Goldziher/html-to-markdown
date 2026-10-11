#[derive(Clone, Copy, Default)]
struct UpcomingTextSibling {
    is_list: bool,
    is_img: bool,
    is_inline: bool,
    /// Comments follow the text, and an inline element opens after them.
    inline_follows_comments: bool,
    /// The content of that inline element starts with a block.
    inline_starts_with_block: bool,
    /// The text ends with a line end and the text after the tag starts with a zero-width space.
    line_end_meets_zero_width_space: bool,
}

#[derive(Clone, Copy)]
struct TextFlush<'a> {
    raw: &'a str,
    base_offset: usize,
    upcoming: UpcomingTextSibling,
    br_in_tables: bool,
    output_format: crate::options::OutputFormat,
}

const UNICODE_WS_ENTITIES: &[&str] = &[
    "&#12;", "&#x0c;", "&#x0C;", "&#xC;", "&nbsp;", "&#160;", "&#xa0;", "&#xA0;", "&ensp;", "&#8194;", "&#x2002;",
    "&emsp;", "&#8195;", "&#x2003;", "&thinsp;", "&#8201;", "&#x2009;", "&hairsp;", "&#8202;", "&#x200a;", "&#x200A;",
    "&#12288;", "&#x3000;",
];

fn handle_structural_text(state: &mut Tier1State, raw: &str, br_in_tables: bool) -> Result<bool, BailReason> {
    if !state.table_stack.is_empty() && !state.in_table_cell() && !state.in_table_caption() {
        return if raw.trim().is_empty() {
            Ok(true)
        } else {
            Err(BailReason::Classifier)
        };
    }
    let has_content = !raw.trim().is_empty();
    if has_content && state.last_closed_block && state.in_table_cell() && in_heading(state) {
        return Err(BailReason::TableBlockChildInCell);
    }
    if has_content
        && std::mem::take(&mut state.last_closed_block)
        && (br_in_tables || !state.in_table_cell() || !raw.as_bytes().first().is_some_and(u8::is_ascii_whitespace))
    {
        separate_inline_after_block(state, br_in_tables)?;
    }
    Ok(false)
}

fn normalize_unicode_whitespace(raw: &str, verbatim: bool) -> std::borrow::Cow<'_, str> {
    if verbatim {
        return std::borrow::Cow::Borrowed(raw);
    }
    let has_entity = UNICODE_WS_ENTITIES.iter().any(|entity| raw.contains(entity));
    let has_literal = raw.chars().any(is_non_ascii_whitespace);
    if !has_entity && !has_literal {
        return std::borrow::Cow::Borrowed(raw);
    }
    let mut without_entities = raw.to_owned();
    for entity in UNICODE_WS_ENTITIES {
        without_entities = without_entities.replace(entity, "");
    }
    if without_entities.chars().all(char::is_whitespace) {
        return std::borrow::Cow::Borrowed(raw);
    }
    let mut normalized = raw.to_owned();
    for entity in UNICODE_WS_ENTITIES {
        normalized = normalized.replace(entity, " ");
    }
    std::borrow::Cow::Owned(
        normalized
            .chars()
            .map(|character| {
                if is_non_ascii_whitespace(character) {
                    ' '
                } else {
                    character
                }
            })
            .collect(),
    )
}

const fn is_non_ascii_whitespace(character: char) -> bool {
    character.is_whitespace() && !matches!(character, ' ' | '\t' | '\n' | '\r')
}

#[derive(Clone, Copy)]
struct TextPosition {
    at_inline_frame_start: bool,
    block_edge: bool,
    whitespace_only: bool,
}

#[derive(Clone, Copy)]
struct WhitespaceHistory {
    after_custom_element: bool,
    after_img: bool,
}

fn handle_whitespace_text(
    state: &mut Tier1State,
    raw: &str,
    in_pre: bool,
    position: TextPosition,
    history: WhitespaceHistory,
    upcoming: UpcomingTextSibling,
) -> bool {
    handle_adjacent_image_whitespace(state, raw, in_pre, position, history, upcoming)
        || drop_block_edge_whitespace(state, raw, in_pre, position)
        || handle_cell_whitespace(state, raw, in_pre, position, history.after_custom_element)
        || handle_outer_whitespace(state, raw, in_pre, position, history, upcoming)
}

fn handle_adjacent_image_whitespace(
    state: &mut Tier1State,
    raw: &str,
    in_pre: bool,
    position: TextPosition,
    history: WhitespaceHistory,
    upcoming: UpcomingTextSibling,
) -> bool {
    if in_pre || state.in_table_cell() || !position.whitespace_only || !history.after_img || !upcoming.is_img {
        return false;
    }
    state
        .cell_or_output_mut()
        .push(if raw.contains(['\n', '\r']) { '\n' } else { ' ' });
    true
}

fn text_position(state: &mut Tier1State, raw: &str) -> TextPosition {
    let in_summary = state.in_summary();
    let active_len = state.cell_or_output_mut().len();
    let at_inline_frame_start = state.stack.last().is_some_and(|frame| {
        frame.content_start >= active_len
            && (matches!(
                frame.spec.kind,
                TagKind::Link | TagKind::Strong | TagKind::Emphasis | TagKind::Code
            ) || (in_summary
                && matches!(
                    frame.spec.kind,
                    TagKind::Inline | TagKind::Block | TagKind::Paragraph | TagKind::Heading(_)
                )))
    });
    let active = state.cell_or_output_mut();
    let block_edge = active.is_empty()
        || active.ends_with('\n')
        || active.ends_with("- ")
        || active.ends_with("* ")
        || active.ends_with("+ ")
        || ends_with_ordered_marker(active)
        || at_inline_frame_start;
    let whitespace_only = raw.bytes().all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'));
    TextPosition {
        at_inline_frame_start,
        block_edge,
        whitespace_only,
    }
}

fn drop_block_edge_whitespace(state: &mut Tier1State, raw: &str, in_pre: bool, position: TextPosition) -> bool {
    let in_list_item = state
        .stack
        .iter()
        .any(|frame| matches!(frame.spec.kind, TagKind::ListItem));
    let document_start = state.at_document_start
        && !state.in_table_cell()
        && !in_list_item
        && !state.escape_ctx.contains(EscapeCtx::HEADING);
    if in_pre || !position.whitespace_only || (!position.block_edge && !document_start) {
        return false;
    }
    if position.at_inline_frame_start {
        if let Some(frame) = state.stack.last_mut() {
            if matches!(
                frame.spec.kind,
                TagKind::Strong | TagKind::Emphasis | TagKind::Strikethrough | TagKind::Inserted | TagKind::Code
            ) {
                frame.dropped_whitespace_only_text = true;
            }
        }
    }
    let _ = raw;
    true
}

fn handle_cell_whitespace(
    state: &mut Tier1State,
    _raw: &str,
    in_pre: bool,
    position: TextPosition,
    after_custom_element_close: bool,
) -> bool {
    if in_pre || !state.in_table_cell() || !position.whitespace_only || position.block_edge {
        return false;
    }
    if matches!(state.stack.last().map(|frame| frame.spec.kind), Some(TagKind::List(_))) {
        return true;
    }
    let cell_len = state.cell_or_output_mut().len();
    if let Some(frame) = state
        .stack
        .iter_mut()
        .rev()
        .take_while(|frame| !matches!(frame.spec.kind, TagKind::TableCell { .. } | TagKind::Summary))
        .find(|frame| frame.own_buffer)
        .filter(|frame| matches!(frame.spec.kind, TagKind::Blockquote | TagKind::Heading(_)))
        .filter(|frame| frame.content_start == cell_len)
    {
        frame.starts_with_whitespace = true;
    }
    let dest = state.cell_or_output_mut();
    if !dest.is_empty() && !dest.ends_with('\n') && (after_custom_element_close || !dest.ends_with(' ')) {
        dest.push(' ');
    }
    true
}

fn handle_outer_whitespace(
    state: &mut Tier1State,
    raw: &str,
    in_pre: bool,
    position: TextPosition,
    history: WhitespaceHistory,
    upcoming: UpcomingTextSibling,
) -> bool {
    if in_pre || state.in_table_cell() || !position.whitespace_only {
        return false;
    }
    let inside_inline = state.in_summary()
        || state.stack.iter().any(|frame| {
            matches!(
                frame.spec.kind,
                TagKind::Link | TagKind::Strong | TagKind::Emphasis | TagKind::Code
            )
        });
    if inside_inline {
        let active = state.cell_or_output_mut();
        if !active.is_empty() && !active.ends_with(' ') && !active.ends_with('\n') {
            active.push(' ');
        }
        return true;
    }
    let direct_paragraph = matches!(
        state.stack.last().map(|frame| frame.spec.kind),
        Some(TagKind::Paragraph)
    );
    if direct_paragraph && history.after_img && upcoming.is_img {
        return true;
    }
    let active = state.cell_or_output_mut();
    let has_inline_tail = output_ends_with_inline_close_marker(active)
        || output_ends_with_inline_text(active)
        || (history.after_custom_element && !active.is_empty() && !active.ends_with('\n'));
    if has_inline_tail {
        if upcoming.is_list && !raw.contains('\n') {
            active.push_str(raw);
        } else if !upcoming.is_list {
            active.push(' ');
        }
    }
    true
}

fn trim_leading_text_run<'a>(
    state: &mut Tier1State,
    raw: &'a str,
    in_pre: bool,
    in_code: bool,
    at_inline_frame_start: bool,
) -> std::borrow::Cow<'a, str> {
    let was_at_document_start = std::mem::replace(&mut state.at_document_start, false);
    let document_start_strip = should_strip_document_start(state, was_at_document_start);
    let block_separator_after = follows_block_separator(state.cell_or_output_mut());
    let leading_ws_migrates = at_inline_frame_start
        && !document_start_strip
        && matches!(
            state.stack.last().map(|frame| frame.spec.kind),
            Some(TagKind::Strong | TagKind::Emphasis)
        );
    let in_link_frame = matches!(state.stack.last().map(|frame| frame.spec.kind), Some(TagKind::Link));
    let after_line_end = follows_line_end(state);
    // ~keep White space after a space is the same run of white space, whatever lies between
    // ~keep the two: an element that wrote nothing is no word. Mirrors Tier-2's `skip_prefix`.
    let after_space = state.cell_or_output_mut().ends_with(' ');
    let should_trim = !in_pre
        && !in_code
        && (after_space
            || ((!state.in_table_cell() || in_link_frame)
                && (at_inline_frame_start || block_separator_after || document_start_strip || after_line_end)));
    if !should_trim {
        return std::borrow::Cow::Borrowed(raw);
    }
    let trimmed = trim_leading_text_whitespace(raw);
    if leading_ws_migrates && trimmed.len() < raw.len() {
        state.cell_or_output_mut().push(' ');
    }
    std::borrow::Cow::Borrowed(trimmed)
}

fn should_strip_document_start(state: &Tier1State, was_at_document_start: bool) -> bool {
    let in_list_item = state
        .stack
        .iter()
        .any(|frame| matches!(frame.spec.kind, TagKind::ListItem));
    was_at_document_start && !state.in_table_cell() && !in_list_item && !state.escape_ctx.contains(EscapeCtx::HEADING)
}

fn follows_block_separator(active: &str) -> bool {
    active.ends_with("\n\n")
        || active.ends_with("- ")
        || active.ends_with("* ")
        || active.ends_with("+ ")
        || ends_with_ordered_marker(active)
}

fn follows_line_end(state: &mut Tier1State) -> bool {
    state.cell_or_output_mut().ends_with('\n')
}

fn emit_verbatim_text(
    state: &mut Tier1State,
    raw: &str,
    in_pre: bool,
    in_code: bool,
    base_offset: usize,
) -> Result<bool, BailReason> {
    if !in_pre && !in_code {
        return Ok(false);
    }
    let has_entities = raw.contains('&');
    if in_pre {
        if has_entities {
            decode_entities_into(state.cell_or_output_mut(), raw, base_offset, ReferenceContext::Text)?;
        } else {
            state.cell_or_output_mut().push_str(raw);
        }
        return Ok(true);
    }
    let decoded = if has_entities {
        let mut buffer = String::with_capacity(raw.len());
        decode_entities_into(&mut buffer, raw, base_offset, ReferenceContext::Text)?;
        std::borrow::Cow::Owned(buffer)
    } else {
        std::borrow::Cow::Borrowed(raw)
    };
    let folded = crate::text::fold_cell_line_breaks_verbatim_cow(decoded.as_ref());
    state.cell_or_output_mut().push_str(folded.as_ref());
    Ok(true)
}

struct PreparedText<'a> {
    text: std::borrow::Cow<'a, str>,
    has_entities: bool,
    ends_in_newline_join: bool,
}

fn prepare_normal_text<'a>(
    state: &Tier1State,
    raw: &'a str,
    inside_inline: bool,
    base_offset: usize,
    next: NextElement,
) -> Result<PreparedText<'a>, BailReason> {
    let in_cell = state.in_table_cell();
    let (decoded, predecoded) = if !inside_inline && !in_cell && raw.contains('&') {
        let mut output = String::with_capacity(raw.len());
        decode_entities_into(&mut output, raw, base_offset, ReferenceContext::Text)?;
        (std::borrow::Cow::Owned(output), true)
    } else {
        (std::borrow::Cow::Borrowed(raw), false)
    };
    let transformed = if !inside_inline && !in_cell {
        chomp_normal_text(state, decoded.as_ref(), next)
    } else {
        None
    };
    let (text, ends_in_newline_join) = match transformed {
        Some((text, ends_in_newline_join)) => (std::borrow::Cow::Owned(text), ends_in_newline_join),
        None => (decoded, false),
    };
    let has_entities = !predecoded && text.contains('&');
    Ok(PreparedText {
        text,
        has_entities,
        ends_in_newline_join,
    })
}

fn chomp_normal_text(state: &Tier1State, raw: &str, next: NextElement) -> Option<(String, bool)> {
    let trim_chars: &[char] = &['\n', '\r', ' ', '\t'];
    let after_leading = raw.trim_start_matches(trim_chars);
    let leading_len = raw.len() - after_leading.len();
    let trimmed_len = raw.trim_end_matches(trim_chars).len();
    let leading_has_newline = raw.as_bytes()[..leading_len]
        .iter()
        .any(|&byte| matches!(byte, b'\n' | b'\r'));
    let trailing = &raw[trimmed_len..];
    let trailing_has_newline = trailing.bytes().any(|byte| matches!(byte, b'\n' | b'\r'));
    if (!leading_has_newline && !trailing_has_newline) || leading_len >= trimmed_len {
        return None;
    }
    let prefix = if leading_len > 0 { " " } else { "" };
    let (suffix, ends_in_newline_join) = trailing_text_suffix(state, trailing, next);
    Some((
        format!("{prefix}{}{suffix}", &raw[leading_len..trimmed_len]),
        ends_in_newline_join,
    ))
}

fn trailing_text_suffix<'a>(state: &Tier1State, trailing: &'a str, next: NextElement) -> (&'a str, bool) {
    if contains_blank_line(trailing.as_bytes()) {
        return ("\n\n", false);
    }
    if trailing.bytes().any(|byte| matches!(byte, b' ' | b'\t')) {
        return (" ", false);
    }
    if trailing.bytes().any(|byte| matches!(byte, b'\n' | b'\r')) {
        let join = trailing_single_newline_join(state, next);
        return (join, join == "\n");
    }
    (trailing, false)
}

struct TextEmission<'a> {
    raw: &'a str,
    has_entities: bool,
    inside_inline: bool,
    base_offset: usize,
    output_format: crate::options::OutputFormat,
    next_tag_is_inline: bool,
    ends_in_newline_join: bool,
}

fn emit_normal_text(state: &mut Tier1State, emission: TextEmission<'_>) -> Result<(), BailReason> {
    let in_cell = state.in_table_cell();
    let folds_lines = in_cell
        || state
            .stack
            .iter()
            .any(|frame| matches!(frame.spec.kind, TagKind::Heading(_)));
    indent_fresh_list_item_text_line(state);
    if emit_inline_backslash_text(state, &emission, in_cell, folds_lines)? {
        return Ok(());
    }
    let in_list_item = state
        .stack
        .iter()
        .any(|frame| matches!(frame.spec.kind, TagKind::ListItem));
    let list_indent_width = if in_list_item {
        state.list_continuation_indent_width()
    } else {
        0
    };
    let dest = state.cell_or_output_mut();
    let emitted_from = dest.len();
    append_collapsed_text(
        dest,
        emission.raw,
        emission.has_entities,
        emission.inside_inline,
        emission.base_offset,
    )?;
    indent_list_item_text_continuation_lines(dest, emitted_from, list_indent_width);
    escape_backslash_run(dest, emitted_from, in_cell);
    escape_emitted_text(
        dest,
        EmittedTextContext {
            emitted_from,
            output_format: emission.output_format,
            inside_inline: emission.inside_inline,
            in_cell,
            folds_lines,
            in_list_item,
            next_tag_is_inline: emission.next_tag_is_inline,
        },
    );
    if emission.ends_in_newline_join {
        state.pending_newline_join = Some(dest.len());
    }
    Ok(())
}

fn emit_inline_backslash_text(
    state: &mut Tier1State,
    emission: &TextEmission<'_>,
    in_cell: bool,
    folds_lines: bool,
) -> Result<bool, BailReason> {
    if !emission.inside_inline || in_cell || !emission.raw.contains('\\') {
        return Ok(false);
    }
    let mut staged = String::with_capacity(emission.raw.len() + 8);
    if emission.has_entities {
        decode_entities_into(&mut staged, emission.raw, emission.base_offset, ReferenceContext::Text)?;
    } else {
        staged.push_str(emission.raw);
    }
    escape_backslash_run(&mut staged, 0, false);
    let dest = state.cell_or_output_mut();
    let emitted_from = dest.len();
    decode_and_collapse_into_inline(dest, &staged, false, emission.base_offset)?;
    if !folds_lines {
        crate::converter::utility::escaping::escape_continuation_line_start(dest, emitted_from, false);
    }
    Ok(true)
}

fn append_collapsed_text(
    dest: &mut String,
    raw: &str,
    has_entities: bool,
    inside_inline: bool,
    base_offset: usize,
) -> Result<(), BailReason> {
    if has_entities {
        if inside_inline {
            return decode_and_collapse_into_inline(dest, raw, true, base_offset);
        }
        return decode_and_collapse_into(dest, raw, true, base_offset);
    }
    let needs_collapse = if inside_inline {
        memchr3(b' ', b'\t', b'\n', raw.as_bytes()).is_some()
    } else {
        memchr::memchr2(b' ', b'\t', raw.as_bytes()).is_some()
    };
    if !needs_collapse {
        dest.push_str(raw);
        return Ok(());
    }
    if inside_inline {
        decode_and_collapse_into_inline(dest, raw, false, base_offset)
    } else {
        decode_and_collapse_into(dest, raw, false, base_offset)
    }
}

struct EmittedTextContext {
    emitted_from: usize,
    output_format: crate::options::OutputFormat,
    inside_inline: bool,
    in_cell: bool,
    folds_lines: bool,
    in_list_item: bool,
    next_tag_is_inline: bool,
}

fn escape_emitted_text(dest: &mut String, context: EmittedTextContext) {
    if context.in_cell && context.output_format == crate::options::OutputFormat::Djot {
        let escaped = crate::converter::utility::escaping::escape_djot_table_cell_literal(
            &dest[context.emitted_from..],
            context.output_format,
            true,
        );
        if let std::borrow::Cow::Owned(escaped) = escaped {
            dest.replace_range(context.emitted_from.., &escaped);
        }
    }
    if context.folds_lines {
        return;
    }
    match context.output_format {
        crate::options::OutputFormat::Markdown => {
            if !context.inside_inline {
                crate::converter::utility::escaping::escape_block_start(
                    dest,
                    context.emitted_from,
                    context.in_list_item,
                    context.next_tag_is_inline,
                );
            }
            crate::converter::utility::escaping::escape_continuation_line_start(dest, context.emitted_from, false);
        }
        crate::options::OutputFormat::Djot => {
            if !context.inside_inline {
                crate::converter::utility::escaping::escape_djot_list_item_start(
                    dest,
                    context.emitted_from,
                    context.in_list_item,
                );
            }
            crate::converter::utility::escaping::escape_djot_continuation_line_start(dest, context.emitted_from, false);
        }
        crate::options::OutputFormat::Plain => {}
    }
}

/// The source text `raw` without the line end at its end, when `raw` has other content too.
///
/// ~keep A line end beside a zero-width space is no white space: the two parts are one word.
/// ~keep Mirrors Tier-2's `without_line_end_before_zero_width_space`, which reads decoded text:
/// ~keep a character reference to white space counts here as that white space.
fn without_line_end_at_zero_width_space(raw: &str) -> &str {
    crate::converter::utility::content::without_trailing_line_end_in_source(raw).unwrap_or(raw)
}

fn flush_text(state: &mut Tier1State, request: TextFlush<'_>) -> Result<(), BailReason> {
    let TextFlush {
        raw,
        base_offset,
        upcoming,
        br_in_tables,
        output_format,
    } = request;
    if raw.is_empty() {
        return Ok(());
    }

    let history = WhitespaceHistory {
        after_custom_element: state.last_closed_custom_element,
        after_img: state.last_emitted_was_img,
    };
    state.last_closed_custom_element = false;
    state.last_emitted_was_img = false;

    if handle_structural_text(state, raw, br_in_tables)? {
        return Ok(());
    }

    let in_pre = state.escape_ctx.contains(EscapeCtx::PRE);
    let in_code = state.escape_ctx.contains(EscapeCtx::CODE);

    let raw = if upcoming.line_end_meets_zero_width_space && !in_pre && !in_code {
        without_line_end_at_zero_width_space(raw)
    } else {
        raw
    };

    let normalized_whitespace = normalize_unicode_whitespace(raw, in_pre || in_code);
    let raw = normalized_whitespace.as_ref();

    let position = text_position(state, raw);
    if handle_whitespace_text(state, raw, in_pre, position, history, upcoming) {
        return Ok(());
    }
    let trimmed_leading = trim_leading_text_run(state, raw, in_pre, in_code, position.at_inline_frame_start);
    let raw = trimmed_leading.as_ref();
    if raw.is_empty() {
        return Ok(());
    }

    if emit_verbatim_text(state, raw, in_pre, in_code, base_offset)? {
        return Ok(());
    }

    let inside_inline = state.in_table_cell()
        || state.in_summary()
        || state.stack.iter().any(|frame| matches!(frame.spec.kind, TagKind::Link));
    let next = NextElement::new(
        upcoming.is_inline || upcoming.inline_follows_comments,
        upcoming.inline_starts_with_block,
    );
    let prepared = prepare_normal_text(state, raw, inside_inline, base_offset, next)?;
    if prepared.text.is_empty() {
        return Ok(());
    }
    let raw = prepared.text.as_ref();
    let has_entities = prepared.has_entities;
    let ends_in_newline_join = prepared.ends_in_newline_join;

    emit_normal_text(
        state,
        TextEmission {
            raw,
            has_entities,
            inside_inline,
            base_offset,
            output_format,
            next_tag_is_inline: upcoming.is_inline,
            ends_in_newline_join,
        },
    )?;
    Ok(())
}
