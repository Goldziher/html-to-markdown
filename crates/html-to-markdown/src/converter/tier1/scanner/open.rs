/// Emit a `<svg>` element as a Markdown base64 data URI, matching Tier-2's
/// `handle_svg` output byte-for-byte.
///
/// `svg_slice` is the raw HTML source bytes for the entire `<svg…>…</svg>`
/// element.  We re-parse it with `tl::parse` to get the canonical attribute
/// order that `serialize_element` produces (it sorts attributes alphabetically,
/// so raw-source slicing would diverge from Tier-2).
///
/// Mirrors Tier-2's `media/svg.rs::handle_svg`:
/// - Takes the alt text from `graphic_text`, the function Tier-2 uses.
/// - Calls `serialize_element` on the root SVG node.
/// - Base64-encodes (STANDARD engine) the serialized bytes.
/// - Emits `![{title}](data:image/svg+xml;base64,{b64})`.
/// - When `options.skip_images` → emits nothing (matches Tier-2 skip).
fn emit_svg_from_slice(
    svg_slice: &str,
    svg_start_offset: usize,
    state: &mut Tier1State,
    options: &ConversionOptions,
) -> Result<(), BailReason> {
    // ~keep CDATA inside SVG cannot be processed correctly without the prescan's
    // entity-escaping transformation.  Bail to Tier-2 so it sees the
    // prescan-normalized form (where `<![CDATA[` is escaped to `&lt;![CDATA[`).
    if svg_slice.contains("<![CDATA[") {
        return Err(BailReason::Cdata {
            offset: svg_start_offset,
        });
    }

    if options.skip_images {
        return Ok(());
    }

    // ~keep Tier-2 removes a hidden element before it parses, also inside a graphic. The slice
    // ~keep still holds it, and its text would be read as text of the graphic.
    if holds_hidden_element(svg_slice) {
        return Err(BailReason::HiddenElement {
            offset: svg_start_offset,
        });
    }

    use crate::converter::media::svg::serialize_element;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    // ~keep Re-parse just the SVG fragment.  Wrap it in a minimal document so
    // tl has proper context — the same pattern used by head_metadata.rs.
    let wrapped = format!("<html><body>{svg_slice}</body></html>");
    let dom = match tl::parse(&wrapped, tl::ParserOptions::default()) {
        Ok(d) => d,
        Err(_) => {
            // ~keep Parse failure: emit nothing rather than bail — matches
            // Tier-2's silent skip on serialization failure.
            return Ok(());
        }
    };
    let parser = dom.parser();

    let Some(handle) = find_svg_handle(&dom) else {
        return Ok(());
    };
    let Some(tl::Node::Tag(svg)) = handle.get(parser) else {
        return Ok(());
    };
    let title = crate::converter::media::svg::graphic_text(svg, parser);

    let svg_html = serialize_element(&handle, parser);
    let base64_svg = STANDARD.encode(svg_html.as_bytes());

    // ~keep Security fix mirror (media/svg.rs::handle_svg, outside tier1/): an
    // unescaped `<title>` here lets `x](https://evil.example)y` in the SVG
    // source close the image label early and open a second, attacker-
    // controlled Markdown image/link — the input is inert HTML, but the
    // unescaped label turns it into a live injection. `escape_link_label`
    // (utility/content.rs) is the shared helper Tier-2's `<a>` label path
    // already uses; call the same one here rather than a third
    // hand-written escaper.
    let escaped_title = crate::converter::utility::escaping::escape_link_label(&title);

    let dest = state.cell_or_output_mut();
    dest.push_str("![");
    dest.push_str(&escaped_title);
    dest.push_str("](data:image/svg+xml;base64,");
    dest.push_str(&base64_svg);
    dest.push(')');

    Ok(())
}

/// Whether a tag inside the `<svg>` of `svg_slice` is hidden by its attribute or its style.
///
/// ~keep Each search starts where the last tag ended, so the scan reads the slice once.
fn holds_hidden_element(svg_slice: &str) -> bool {
    use crate::converter::utility::preprocessing::{find_tag_end, tag_has_hidden_attribute, tag_has_hidden_style};

    let mut from = 1;
    while let Some(start) = svg_slice
        .get(from..)
        .and_then(|rest| rest.find('<'))
        .map(|at| from + at)
    {
        let end = find_tag_end(svg_slice.as_bytes(), start + 1)
            .unwrap_or(svg_slice.len())
            .max(start + 1);
        let tag = &svg_slice[start..end];
        if tag_has_hidden_attribute(tag) || tag_has_hidden_style(tag) {
            return true;
        }
        from = end;
    }
    false
}

fn find_svg_handle(dom: &tl::VDom<'_>) -> Option<tl::NodeHandle> {
    dom.nodes().iter().enumerate().find_map(|(index, node)| {
        let tl::Node::Tag(tag) = node else { return None };
        tag.name()
            .as_utf8_str()
            .as_ref()
            .eq_ignore_ascii_case("svg")
            .then(|| tl::NodeHandle::new(index as u32))
    })
}

/// Skip the body of a raw-text element (script/style/textarea/iframe/…).
///
/// `open_end` is the byte index immediately after the tag's `>`.  `tag_name`
/// is the lowercased open-tag name.  Returns the byte index after the
/// matching `</tag>` close, or `None` if no matching close tag exists in the
/// remainder of the input.
///
/// Mirrors the prescan's STRIP_CONTENT_TAGS handling: content is discarded,
/// only the position advances.  Matches Tier-2's behaviour byte-for-byte
/// because Tier-2 sees this content already stripped by the prescan.
fn find_raw_text_close(bytes: &[u8], open_end: usize, tag_name: &[u8]) -> Option<usize> {
    let len = bytes.len();
    let mut idx = open_end;
    while idx < len {
        match memchr3(b'<', b'<', b'<', &bytes[idx..]) {
            Some(off) => idx += off,
            None => return None,
        }
        if idx + 2 < len && bytes[idx + 1] == b'/' {
            let after_slash = idx + 2;
            if after_slash + tag_name.len() <= len
                && bytes[after_slash..after_slash + tag_name.len()].eq_ignore_ascii_case(tag_name)
            {
                let post_name = after_slash + tag_name.len();
                if matches!(bytes.get(post_name), Some(b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r')) {
                    return find_tag_end(bytes, post_name);
                }
            }
        }
        idx += 1;
    }
    None
}

fn find_tag_end(bytes: &[u8], start: usize) -> Option<usize> {
    memchr::memchr(b'>', &bytes[start..]).map(|offset| start + offset + 1)
}

#[inline]
const fn bail_unsupported(spec: &TagSpec, _offset: usize) -> Result<(), BailReason> {
    match spec.kind {
        // ~keep This arm IS load-bearing — do not delete it as unreachable.  The inline
        // raw-text handling above is gated on `TagKind::Ignored && is_rawtext`, which
        // is only `<script>`/`<style>`.  The seven genuine `TagKind::RawText` kinds
        // (title / xmp / textarea / iframe / noscript / noembed / noframes) fall
        // through to here, and this is the bail that keeps Tier-1 from emitting their
        // text content incorrectly.  See the sibling note above `find_raw_text_close`.
        TagKind::RawText(_) => Err(BailReason::Classifier),

        // ~keep `Ignored` tags (head/meta/link/script/style) are now handled inline
        // by the main scan loop (see the dispatch above `bail_unsupported`).
        // The match arm is kept for exhaustiveness — it cannot fire in
        // practice.
        TagKind::Ignored => Err(BailReason::Classifier),

        _ => Ok(()),
    }
}

// ~keep `TagKind::Block` is Tier-1's catch-all for HTML5 "generic block container"
// elements, but Tier-2's `main.rs` dispatch match does NOT give every one of
// them a dedicated separator-emitting handler. `<div>` (block/div.rs), the
// semantic/media/form-dispatched names (section, article, header, footer,
// aside, main, nav, details, dialog, figure, menu, audio, video, fieldset,
// legend, form), and -- as of the fix below -- `address`/`search`/`hgroup`/
// `center` (routed straight to `block::div::handle`, since they are
// content-bearing containers with no formatting beyond block separation) all
// render through a handler that pushes a leading and/or trailing `"\n\n"`.
// The names below are the ones that still fall through to Tier-2's `_ =>` arm
// (`block::unknown::handle`) or, for `html`/`body`,
// `block::container::handle_structural_container` -- both of which just walk
// children with NO separator of their own, and deliberately so:
// `colgroup`/`col` are table-internal metadata (never render visible content
// of their own), `base` is void metadata, and `html`/`body` are document
// wrappers whose children ARE the document -- giving them a separator would
// change the shape of every converted document, not fix a content-merging bug.
// `nav`/`form`/`header`/`footer`/`aside` are excluded from this list even
// though they can also hit a preprocessing-strip shortcut elsewhere
// (`is_preprocessing_skip_candidate`): when that shortcut does NOT fire, they
// get semantic-handler separator behaviour like `<div>`.
fn block_container_is_passthrough(name_lower: &[u8]) -> bool {
    matches!(name_lower, b"colgroup" | b"col" | b"base" | b"html" | b"body")
}

fn emit_open(
    state: &mut Tier1State,
    spec: &'static TagSpec,
    name_lower: &[u8],
    attrs: &[(&[u8], Option<&[u8]>)],
    table_probes: &mut Vec<TableLayoutProbe>,
    options: &ConversionOptions,
) -> Result<(), BailReason> {
    prepare_open_state(state, spec, name_lower, options)?;
    emit_open_kind(state, spec, name_lower, attrs, table_probes, options)
}

fn prepare_open_state(
    state: &mut Tier1State,
    spec: &TagSpec,
    name_lower: &[u8],
    options: &ConversionOptions,
) -> Result<(), BailReason> {
    state.last_closed_custom_element = false;
    state.last_emitted_was_img = false;
    if std::mem::take(&mut state.last_closed_block) && is_inline_tag(name_lower) {
        if state.in_table_cell() {
            return Err(BailReason::TableBlockChildInCell);
        }
        separate_inline_after_block(state, options.br_in_tables)?;
    }
    if name_lower == b"q" || (name_lower == b"mark" && options.highlight_style != crate::options::HighlightStyle::None)
    {
        return Err(BailReason::InlineMarkerNotReproduced);
    }
    if matches!(name_lower, b"select" | b"option" | b"optgroup" | b"datalist") {
        return Err(BailReason::FormControl);
    }
    let is_link_block = matches!(
        spec.kind,
        TagKind::Block
            | TagKind::Paragraph
            | TagKind::Heading(_)
            | TagKind::Blockquote
            | TagKind::Pre
            | TagKind::List(_)
            | TagKind::Table
            | TagKind::Summary
    );
    if is_link_block && state.stack.iter().any(|frame| matches!(frame.spec.kind, TagKind::Link)) {
        return Err(BailReason::Classifier);
    }
    // ~keep In code (`<pre>`, `<code>`, `<kbd>`, `<samp>`) Tier-2 writes a link label as it did
    // ~keep before the rules of running text. This scanner trims the label, so it leaves the
    // ~keep page to Tier-2. A block in code writes the same lines in both converters.
    let in_code = state.escape_ctx.intersects(EscapeCtx::CODE | EscapeCtx::PRE);
    if in_code && matches!(spec.kind, TagKind::Link) {
        return Err(BailReason::Classifier);
    }
    // ~keep A block inside marks inside a heading (`<h2>one<b><p>y</p></b>two</h2>`): Tier-2
    // ~keep writes the block boundary as one space after the closing marks (issue #751). This
    // ~keep scanner closes the marks before it knows what follows, so it leaves the page to Tier-2.
    let in_marks = |frame: &OpenTag| {
        matches!(
            frame.spec.kind,
            TagKind::Strong | TagKind::Emphasis | TagKind::Strikethrough | TagKind::Inserted
        )
    };
    if is_link_block && state.escape_ctx.contains(EscapeCtx::HEADING) && state.stack.iter().any(in_marks) {
        return Err(BailReason::Classifier);
    }
    Ok(())
}

fn emit_open_kind(
    state: &mut Tier1State,
    spec: &TagSpec,
    name_lower: &[u8],
    attrs: &[(&[u8], Option<&[u8]>)],
    table_probes: &mut Vec<TableLayoutProbe>,
    options: &ConversionOptions,
) -> Result<(), BailReason> {
    match spec.kind {
        TagKind::Paragraph => open_paragraph(state, options.br_in_tables),
        TagKind::Heading(_) => open_heading(state),
        TagKind::Blockquote => {
            // ~keep A citation is rendered after the quoted content and resolved against
            // `base_url`; Tier-1 does neither. See `BailReason::BlockquoteCite`.
            if find_attr(attrs, b"cite").is_some_and(|value| !value.is_empty()) {
                return Err(BailReason::BlockquoteCite);
            }
            open_blockquote(state);
        }
        TagKind::Pre => open_pre(state, attrs),
        TagKind::List(ListKind::Definition) => open_dl(state),
        TagKind::List(kind) => open_list(state, kind, options),
        TagKind::ListItem => open_list_item(state, options),
        TagKind::DefinitionTerm => open_dt(state),
        TagKind::DefinitionDescription => open_dd(state),
        TagKind::Strong => open_strong(state)?,
        TagKind::Emphasis => open_emphasis(state)?,
        TagKind::Strikethrough => open_marker(state, "~~")?,
        TagKind::Inserted => open_marker(state, "==")?,
        TagKind::Code if !state.escape_ctx.contains(EscapeCtx::PRE) && !state.escape_ctx.contains(EscapeCtx::CODE) => {}
        TagKind::Code if state.pre_lang.is_none() && state.escape_ctx.contains(EscapeCtx::PRE) => {
            if let Some(lang) = extract_language_from_class(attrs) {
                state.pre_lang = Some(lang);
            }
        }
        TagKind::Link => open_link(state),
        TagKind::Table => open_table(state, attrs, table_probes),
        TagKind::TableCaption => open_table_caption(state),
        TagKind::TableHead => open_table_head(state)?,
        TagKind::TableBody => open_table_body(state)?,
        TagKind::TableFoot => open_table_foot(state),
        TagKind::TableRow => open_table_row(state),
        TagKind::TableCell { is_header } => open_table_cell(state, attrs, is_header, table_probes)?,
        // ~keep Block containers: emit a leading blank-line separator when there's
        // already preceding content.  Mirrors Tier-2's div/sectioning handlers
        // (`block/div.rs`'s `needs_leading_sep` branch and the separator push in
        // `semantic/sectioning.rs`) which prefix block content with `\n\n` to
        // separate it from siblings.
        // ~keep
        // ~keep Inside a table cell, Tier-2's `is_table_continuation` in `div.rs` treats
        // a sibling-div as a "table continuation" and emits `"  \n"` when
        // the cell already has non-`|`/non-`<br>` content.  After
        // `close_table_cell`'s `replace('\n', ' ')` step, this becomes a 3-space
        // run between sibling divs — matching Tier-2's lists_timeline cell
        // layout `[link]   [other-link]`.  Without this, Tier-1 emits 1 space.
        TagKind::Block => open_block_container(state, name_lower, options.br_in_tables),
        TagKind::Summary => open_summary_container(state, options.br_in_tables)?,
        TagKind::Figcaption => open_figcaption(state),
        TagKind::Button => open_button(state)?,
        TagKind::Inline => {}
        _ => {}
    }

    Ok(())
}

/// ~keep Button: nothing on open. `close_button` writes the space before its text and
/// ~keep the `\n\n` after it, as Tier-2 `handle_button` does. In a line that goes on
/// ~keep after the control, Tier-2 reads the text that follows it instead. In code Tier-2
/// ~keep writes neither.
fn open_button(state: &Tier1State) -> Result<(), BailReason> {
    if state.escape_ctx.intersects(EscapeCtx::CODE | EscapeCtx::PRE) {
        return Err(BailReason::FormControl);
    }
    let in_inline_container = state.stack.iter().any(|frame| {
        matches!(
            frame.spec.kind,
            TagKind::Heading(_) | TagKind::Summary | TagKind::Figcaption | TagKind::Link | TagKind::TableCaption
        )
    });
    let in_inline_element = state
        .stack
        .last()
        .is_some_and(|parent| crate::converter::form::spacing::line_goes_on_in(Some(parent.spec)));
    if in_inline_container || in_inline_element {
        return Err(BailReason::FormControl);
    }
    Ok(())
}

/// ~keep Nested strong/code contexts suppress markers; adjacent delimiters require Tier-2 merging.
fn open_strong(state: &mut Tier1State) -> Result<(), BailReason> {
    let suppressed = state.summary_at_top()
        || state.escape_ctx.contains(EscapeCtx::STRONG)
        || state.escape_ctx.contains(EscapeCtx::CODE)
        || state.escape_ctx.contains(EscapeCtx::PRE);
    if suppressed {
        return Ok(());
    }
    if state.cell_or_output_mut().ends_with("**") {
        return Err(BailReason::AdjacentInlineEmphasis);
    }
    state.cell_or_output_mut().push_str("**");
    Ok(())
}

fn open_emphasis(state: &mut Tier1State) -> Result<(), BailReason> {
    if state.escape_ctx.intersects(EscapeCtx::CODE | EscapeCtx::PRE) {
        return Ok(());
    }
    let output = state.cell_or_output_mut();
    if output.ends_with('*') && !output.ends_with("**") {
        return Err(BailReason::AdjacentInlineEmphasis);
    }
    output.push('*');
    Ok(())
}

fn open_marker(state: &mut Tier1State, marker: &str) -> Result<(), BailReason> {
    if state.escape_ctx.intersects(EscapeCtx::CODE | EscapeCtx::PRE) {
        return Ok(());
    }
    if state.cell_or_output_mut().ends_with(marker) {
        return Err(BailReason::AdjacentInlineEmphasis);
    }
    state.cell_or_output_mut().push_str(marker);
    Ok(())
}

fn open_block_container(state: &mut Tier1State, name_lower: &[u8], br_in_tables: bool) {
    if block_container_is_passthrough(name_lower) || start_line_in_pre(state, name_lower) {
        return;
    }
    if state.in_table_cell() {
        break_cell_before_block(state, br_in_tables);
        return;
    }
    let output = state.cell_or_output_mut();
    if !output.is_empty() && !output.ends_with("\n\n") {
        crate::converter::tier1::state::trim_trailing_horizontal(output);
        output.push_str("\n\n");
    }
}

fn open_summary_container(state: &mut Tier1State, br_in_tables: bool) -> Result<(), BailReason> {
    if state.in_summary() {
        return Err(BailReason::Classifier);
    }
    if state.in_table_cell() {
        break_cell_before_block(state, br_in_tables);
    }
    open_summary(state);
    Ok(())
}

/// Writes the cell break Tier-2's `div::handle` writes before a block in a cell that already has
/// content.
fn break_cell_before_block(state: &mut Tier1State, br_in_tables: bool) {
    if state.escape_ctx.contains(EscapeCtx::PRE) {
        let should_break = {
            let cell_buf = state.cell_or_output_mut();
            !cell_buf.is_empty() && !cell_buf.ends_with('|') && !cell_buf.ends_with("<br>") && !cell_buf.ends_with('\n')
        };
        if should_break {
            emit_tier1_table_cell_break(state, br_in_tables);
        }
        return;
    }
    with_cell_scratch(state, |cell_buf| {
        if !cell_buf.is_empty() && !cell_buf.ends_with('|') && !cell_buf.ends_with("<br>") && !cell_buf.ends_with('\n')
        {
            // ~keep Tier-2's helper, so the break follows `br_in_tables` as Tier-2's does.
            crate::converter::main_helpers::emit_table_cell_break(cell_buf, br_in_tables);
        }
    });
}

fn emit_tier1_table_cell_break(state: &mut Tier1State, br_in_tables: bool) {
    if br_in_tables && state.escape_ctx.contains(EscapeCtx::PRE) {
        let offset = {
            let output = state.cell_or_output_mut();
            crate::converter::main_helpers::trim_trailing_whitespace(output);
            output.len()
        };
        state.pre_cell_break_offsets.push(offset);
        state.cell_or_output_mut().push('\n');
    } else {
        crate::converter::main_helpers::emit_table_cell_break(state.cell_or_output_mut(), br_in_tables);
    }
}

fn open_paragraph(state: &mut Tier1State, br_in_tables: bool) {
    // ~keep When inside a table cell, treat `<p>` as a transparent container.
    // Tier-2's paragraph.rs writes the cell break (`<br>` under `br_in_tables`,
    // a space otherwise) when `in_table_cell` and there is already cell content
    // (issue #647); we mirror that behaviour so the cell buffer stays on one
    // logical line (no `\n` in cell output to collapse later).
    if state.in_table_cell() {
        if state.escape_ctx.contains(EscapeCtx::PRE) {
            break_cell_before_block(state, br_in_tables);
            return;
        }
        with_cell_scratch(state, |cell_buf| {
            if !cell_buf.is_empty() && !cell_buf.ends_with("<br>") && !cell_buf.ends_with('\n') {
                crate::converter::main_helpers::emit_table_cell_break(cell_buf, br_in_tables);
            }
        });
        return;
    }
    // ~keep Mirrors Tier-2: when output is non-empty and doesn't already end
    // with "\n\n", push "\n\n" (may produce three newlines total when
    // output ends with a single "\n", e.g. right after a table row or
    // an `<hr>`).
    // Phase EE: when the paragraph is the first child of a list-item
    // (output ends with a freshly-emitted bullet like `- ` or `1. `),
    // the paragraph content joins the bullet inline.  Tier-2's
    // paragraph.rs (`is_list_continuation`) only applies this special case
    // when `ctx.in_list_item` is true -- gate on the same condition here.
    // Without it, ordinary top-level text that happens to end in "- "/"* "/
    // "+ "/"N. " (a real bullet-looking suffix, OR the `<strong>`/`<em>`
    // ambiguity `line_is_bare_list_marker` exists to rule out) wrongly
    // skipped the "\n\n" separator before a following `<p>` and glued the
    // two blocks onto a single line, even with no list anywhere in sight.
    // Check BEFORE `trim_trailing_horizontal`, which would strip the
    // trailing space from the bullet.
    let in_list_item = state
        .stack
        .iter()
        .any(|frame| matches!(frame.spec.kind, TagKind::ListItem));
    if in_list_item {
        let dest = state.cell_or_output_mut();
        if dest.ends_with("- ") || dest.ends_with("* ") || dest.ends_with("+ ") || ends_with_ordered_marker(dest) {
            return;
        }
    }
    // ~keep Drop trailing horizontal whitespace from inter-tag preservation
    // (Phase U-2) before the block separator.
    let dest = state.cell_or_output_mut();
    crate::converter::tier1::state::trim_trailing_horizontal(dest);
    if !dest.is_empty() && !dest.ends_with("\n\n") {
        dest.push_str("\n\n");
    }
}

fn open_heading(state: &mut Tier1State) {
    // ~keep When inside a table cell, Tier-2 does NOT add a leading separator before
    // the heading (`needs_leading_sep = false` when `in_table_cell`).  The
    // heading text is emitted directly into the cell accumulator with no `#`
    // prefix and no surrounding newlines.
    if state.in_table_cell() {
        return;
    }
    // ~keep A heading inside `<summary>`/`<figcaption>` is not in a table cell but
    // also must not touch `state.output`: Tier-2's `handle_summary` walks
    // children with a fresh LOCAL `content` buffer as `output`, so
    // `heading.rs`'s leading-separator step runs against that buffer, not
    // the real document output. `cell_or_output_mut` already resolves to the
    // active summary/figcaption buffer here (it takes priority over table
    // cells), so routing through it — instead of the hardcoded
    // `state.ensure_blank_line()` — keeps the separator (and, in
    // `close_heading`, the `#` prefix) inside the same buffer the heading's
    // own text lands in. Without this, `content_start` (captured right after
    // this call, from that same buffer's length) gets treated as an offset
    // into `state.output` instead — an unrelated, much larger buffer — and
    // `close_heading` splices its `#` prefix into the middle of whatever
    // text happens to sit at that byte offset in the real output.
    ensure_blank_line_buf(state.cell_or_output_mut());
}

/// Buffer-generic equivalent of [`Tier1State::ensure_blank_line`].
///
/// Operates on whichever accumulator buffer the caller passes in — `state.output`,
/// a table cell, or a `<summary>`/`<figcaption>` wrap buffer — rather than assuming
/// `state.output`. See `Tier1State::cell_or_output_mut`'s buffer-selection priority.
fn ensure_blank_line_buf(buf: &mut String) {
    if buf.is_empty() {
        return;
    }
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

fn open_blockquote(state: &mut Tier1State) {
    // ~keep Tier-2's `handle_blockquote` (blockquote.rs) branches on nesting: a
    // NESTED blockquote (already inside an outer one, per `EscapeCtx::BLOCKQUOTE`)
    // unconditionally gets a blank-line separator on open. A TOP-LEVEL
    // blockquote instead computes its separator from whatever the
    // immediately preceding sibling already left in `output` — see
    // `close_blockquote`, which needs that untouched pre-open tail, so
    // nothing is done here for the top-level case.
    if state.escape_ctx.contains(EscapeCtx::BLOCKQUOTE) {
        state.ensure_blank_line();
    }
}

fn open_pre(state: &mut Tier1State, attrs: &[(&[u8], Option<&[u8]>)]) {
    state.ensure_blank_line();
    if state.in_table_cell() {
        state.pre_cell_break_offsets.clear();
    }
    if let Some(lang) = extract_language_from_class(attrs) {
        state.pre_lang = Some(lang);
    }
}

/// Extract the language tag from a `class` attribute matching `language-X`
/// or `lang-X`.  Mirrors Tier-2's `extract_language_from_pre`.
fn extract_language_from_class(attrs: &[(&[u8], Option<&[u8]>)]) -> Option<String> {
    let class_bytes = find_attr(attrs, b"class")?;
    // ~keep Decoded before the `language-` prefix scan, matching Tier-2's `code_block.rs`
    // (issue #494). Decoding first is also what makes the split correct: a character
    // reference can encode the whitespace this splits on.
    let class = decode_attr(class_bytes).ok()?;
    for cls in class.split_ascii_whitespace() {
        if let Some(rest) = cls.strip_prefix("language-") {
            return Some(rest.to_owned());
        }
        if let Some(rest) = cls.strip_prefix("lang-") {
            return Some(rest.to_owned());
        }
    }
    None
}

/// Strip one bare list marker -- a single bullet char (`-`, `*`, `+`) followed by a space,
/// or one-or-more ASCII digits followed by `". "` -- from the front of `text`, returning
/// what remains after it. Returns `None` when `text` does not start with a marker. Mirrors
/// Tier-2's `strip_leading_bare_marker` (`list/utils.rs`).
fn strip_leading_bare_marker(text: &str) -> Option<&str> {
    let digit_count = text.bytes().take_while(u8::is_ascii_digit).count();
    if digit_count > 0 {
        if let Some(rest) = text[digit_count..].strip_prefix(". ") {
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
/// indentation and no other content. Mirrors Tier-2's `line_is_bare_list_marker`
/// (`list/utils.rs`).
///
/// ~keep A plain suffix check like `output.ends_with("* ")` also matches the closing
/// `"**"` of `<strong>` (or the closing `"*"` of `<em>`) immediately followed by a
/// migrated trailing space, e.g. `"**b** "`: its last two bytes are literally `'*'`
/// and `' '`, indistinguishable by suffix alone from a real bare `"* "` bullet. That
/// false positive suppressed the newline before a nested list, flattening it onto
/// the parent line and destroying it on reparse. Requiring the WHOLE line (after
/// stripping only leading indentation) to decompose into nothing but marker tokens
/// rules that out, and also handles several single-child lists nested directly
/// inside each other, whose bare markers stack on one physical line with nothing
/// else between them.
/// Whether the scanner is inside a `<dt>` or `<dd>` that a list item holds without a `<dl>`.
/// Tier-2 writes such a term below an empty item, so the rule after it and the text after that
/// stay on the fast path, where `- t` keeps the term in the item.
fn inside_stray_definition(state: &Tier1State) -> bool {
    for frame in state.stack.iter().rev() {
        match frame.spec.kind {
            TagKind::DefinitionTerm | TagKind::DefinitionDescription => return true,
            TagKind::ListItem | TagKind::List(_) => return false,
            _ => {}
        }
    }
    false
}

fn line_is_bare_list_marker(output: &str) -> bool {
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

fn open_list(state: &mut Tier1State, kind: ListKind, options: &ConversionOptions) {
    // ~keep When inside a table cell, mirror Tier-2's `add_list_leading_separator`:
    // emit a line-break separator if there is already cell content (but not if it
    // already ends with `|`, ` `, or `<br>`) -- a literal `<br>` under
    // `br_in_tables`, otherwise a single space, exactly like
    // `main_helpers::emit_table_cell_break`.  Do not touch `state.output`.
    if state.in_table_cell() {
        with_cell_scratch(state, |cell_buf| {
            if !cell_buf.is_empty()
                && !cell_buf.ends_with('|')
                && !cell_buf.ends_with(' ')
                && !cell_buf.ends_with("<br>")
            {
                if options.br_in_tables {
                    cell_buf.push_str("<br>");
                } else {
                    cell_buf.push(' ');
                }
            }
        });
        state.list_depth = state.list_depth.saturating_add(1);
        if matches!(kind, ListKind::Unordered) {
            state.ul_depth = state.ul_depth.saturating_add(1);
        }
        return;
    }
    let current_list_depth = state.list_depth;
    {
        let dest = state.cell_or_output_mut();
        if !dest.is_empty() {
            if current_list_depth == 0 {
                // ~keep Mirror Tier-2's top-level `add_list_leading_separator` branch
                // (`!ctx.in_list`, list/utils.rs): append "\n\n" unless the tail is
                // already a blank line or the current line is nothing but a bare
                // list marker (see `line_is_bare_list_marker`'s doc comment for why
                // a plain suffix check on "* "/"- "/". " is not enough here).
                let needs_newline = !dest.ends_with("\n\n") && !line_is_bare_list_marker(dest);
                if needs_newline {
                    dest.push_str("\n\n");
                }
            } else {
                // ~keep Mirror Tier-2's `ctx.in_list_item` branch the same way: the same
                // whole-line bare-marker check (against a bare newline instead of a
                // blank line), trimming ONLY once it actually decides to insert the
                // separator — not eagerly beforehand.
                let needs_newline = !dest.ends_with('\n') && !line_is_bare_list_marker(dest);
                if needs_newline {
                    crate::converter::tier1::state::trim_trailing_horizontal(dest);
                    dest.push('\n');
                }
            }
        }
    }
    state.list_depth = state.list_depth.saturating_add(1);
    if matches!(kind, ListKind::Unordered) {
        state.ul_depth = state.ul_depth.saturating_add(1);
    }
}

/// Cycle through the canonical default `options.bullets` value (`"-*+"`) by
/// `<ul>` nesting depth.  The router (`router.rs::classify`) gates Tier-1 to
/// the literal default, so this hardcoded cycle reproduces Tier-2 byte-for-byte.
const TIER1_BULLETS: [u8; 3] = [b'-', b'*', b'+'];

fn open_list_item(state: &mut Tier1State, options: &ConversionOptions) {
    // ~keep When inside a table cell, Tier-2 does NOT emit bullet/number prefixes
    // for list items (see list/item.rs: `if !ctx.in_table_cell { ... bullet ... }`).
    // Sibling <li>s still need a boundary though — mirror Tier-2's reuse of
    // `add_list_leading_separator` per item (list/item.rs's `else if
    // ctx.in_table_cell` arm) with the same continuation condition already used by
    // `open_list` above: a literal `<br>` under `br_in_tables`, otherwise a single
    // space (mirroring `main_helpers::emit_table_cell_break`), so
    // `<li>a</li><li>b</li>` in a cell becomes `a<br>b` (or `a b` with
    // `br_in_tables: false`) instead of the two items' raw text running together.
    if state.in_table_cell() {
        if find_parent_list_kind(&state.stack) == Some(ListKind::Ordered) {
            increment_ol_counter(&mut state.stack);
        }
        with_cell_scratch(state, |cell_buf| {
            if !cell_buf.is_empty()
                && !cell_buf.ends_with('|')
                && !cell_buf.ends_with(' ')
                && !cell_buf.ends_with("<br>")
            {
                if options.br_in_tables {
                    cell_buf.push_str("<br>");
                } else {
                    cell_buf.push(' ');
                }
            }
        });
        return;
    }
    let parent_kind = find_parent_list_kind(&state.stack);
    let indent_depth = state.list_depth.saturating_sub(1);
    // ~keep Mirror Tier-2's fresh-line-only indent (list/item.rs): a nested list that is
    // the sole/first content of its enclosing <li> renders directly after that
    // parent's own bare marker on the SAME physical line -- the parent marker's own
    // printed width already reaches this item's target column, so indenting here
    // too would double-count it. The indent is only needed when this item genuinely
    // starts a fresh physical line.
    let ordered_index = (parent_kind == Some(ListKind::Ordered)).then(|| {
        let counter = increment_ol_counter(&mut state.stack);
        find_ol_start(&state.stack).saturating_sub(1) + counter
    });
    // ~keep Mirror Tier-2 (list/item.rs, issue #625): text inside the list before this item
    // ~keep ends its line, with a blank line when the marker line cannot interrupt the text. A
    // ~keep bullet with content always can.
    let line_start = state.output.rfind('\n').map_or(0, |pos| pos + 1);
    let after_text = !state.output[line_start..].trim().is_empty() && !line_is_bare_list_marker(&state.output);
    state.list_items_after_text.push(after_text);
    if after_text {
        state.output.push('\n');
        if ordered_index
            .is_some_and(|index| !crate::converter::utility::escaping::line_opens_block(&format!("{index}. x")))
        {
            state.output.push('\n');
        }
    }
    if indent_depth > 0 && (state.output.is_empty() || state.output.ends_with('\n')) {
        push_list_item_indent(&mut state.output, indent_depth);
    }
    if let Some(index) = ordered_index {
        let marker = format!("{index}. ");
        state.list_item_marker_widths.push(marker.len());
        state.output.push_str(&marker);
    } else {
        let bullet_idx = state.ul_depth.saturating_sub(1) as usize % TIER1_BULLETS.len();
        state.list_item_marker_widths.push(2);
        state.output.push(TIER1_BULLETS[bullet_idx] as char);
        state.output.push(' ');
    }
}

fn open_link(state: &mut Tier1State) {
    // ~keep Track link count inside tables for layout-table detection.
    if let Some(ts) = state.table_stack.last_mut() {
        ts.link_count += 1;
    }
    state.cell_or_output_mut().push('[');
}

/// Per-table inputs to Tier-2's layout-table heuristic that `TableState` does not
/// already carry.
///
/// Mirrors the two `TableScan` fields (`block/table/scanner.rs`) that Tier-2's
/// `looks_like_layout` reads in `block/table/builder.rs`:
///
/// ```text
/// looks_like_layout = nested_table_count > 1 || (has_span && has_border_zero)
/// ```
///
/// Both are collected here; `TableState` already tracks everything else `close_table`
/// needs, including `inconsistent_cols` (ragged row lengths), which is no longer part of
/// Tier-2's formula (issue #500) but stays a Tier-1-only bail: a stricter-than-Tier-2 bail
/// is parity-safe since it only ever routes more input to the fallback, and Tier-1 has no
/// need to learn Tier-2's ragged-row padding (issue #13) just to stay in step with it. One
/// entry is pushed by [`open_table`] and popped by [`close_table`], in lockstep with
/// `Tier1State::table_stack`, so `last_mut()` is always the innermost open table.
#[derive(Debug, Clone, Copy, Default)]
struct TableLayoutProbe {
    /// Number of directly-nested `<table>` elements closed inside this table.
    ///
    /// Counts one level only: a table nested inside a nested table increments its
    /// immediate parent, never this frame — matching Tier-2's `scan_own_structure`,
    /// which stops its walk at each nested `<table>` boundary.
    nested_table_count: usize,
    /// True once any cell in this table carried a `colspan`/`rowspan` attribute.
    has_span: bool,
    /// True when the `<table>` tag carried `border="0"` exactly.
    border_zero: bool,
}

fn open_table(state: &mut Tier1State, attrs: &[(&[u8], Option<&[u8]>)], table_probes: &mut Vec<TableLayoutProbe>) {
    // ~keep Phase HH: nested tables are accumulated natively; an inner table inherits
    // `inline_mode = true` so its final GFM rendering writes into the parent
    // cell buffer rather than `state.output`.  The parent cell's newline
    // collapse then flattens the inner table to a single inline run.
    let inline_mode = !state.table_stack.is_empty();
    state.table_stack.push(crate::converter::tier1::state::TableState {
        inline_mode,
        ..Default::default()
    });
    // ~keep Tier-2 compares the raw attribute value against the literal string "0"
    // (`builder.rs`: `b.as_utf8_str() == "0"`), with no trimming and no numeric
    // parse: `border="00"`, `border=" 0"` and a valueless `border` are all NOT
    // border-zero there, so they must not be here either.
    table_probes.push(TableLayoutProbe {
        border_zero: find_attr(attrs, b"border").is_some_and(|value| value == b"0".as_slice()),
        ..TableLayoutProbe::default()
    });
}

fn open_table_caption(state: &mut Tier1State) {
    if let Some(ts) = state.table_stack.last_mut() {
        ts.caption_buf.clear();
        ts.in_caption = true;
    }
}

fn open_table_head(state: &mut Tier1State) -> Result<(), BailReason> {
    if let Some(ts) = state.table_stack.last_mut() {
        if ts.seen_tbody_close || ts.seen_tfoot {
            return Err(BailReason::TableSectionOrder);
        }
        ts.in_thead = true;
    }
    Ok(())
}

fn open_table_body(state: &mut Tier1State) -> Result<(), BailReason> {
    if let Some(ts) = state.table_stack.last_mut() {
        if ts.seen_tfoot {
            return Err(BailReason::TableSectionOrder);
        }
    }
    Ok(())
}

fn open_table_foot(state: &mut Tier1State) {
    if let Some(ts) = state.table_stack.last_mut() {
        ts.seen_tfoot = true;
    }
}

fn open_table_row(state: &mut Tier1State) {
    if let Some(ts) = state.table_stack.last_mut() {
        ts.current_row.clear();
    }
}

fn open_table_cell(
    state: &mut Tier1State,
    attrs: &[(&[u8], Option<&[u8]>)],
    is_header: bool,
    table_probes: &mut [TableLayoutProbe],
) -> Result<(), BailReason> {
    // ~keep Tier-2's `has_span` (block/table/scanner.rs::scan_row_cells) is set by the
    // mere *presence* of a `colspan`/`rowspan` attribute — `attrs.get(k).is_some()`
    // — so `colspan="1"` and a valueless `colspan` both count.  Deliberately NOT
    // `value > 1`: this feeds `looks_like_layout` in close_table and a tighter
    // predicate would leave the byte-equality divergence in place for exactly the
    // tables it excluded.
    let spanning = has_attr(attrs, b"colspan") || has_attr(attrs, b"rowspan");
    if let Some(probe) = table_probes.last_mut() {
        probe.has_span |= spanning;
    }
    // ~keep rowspan: accepted but not expanded (lossy — a spanned cell renders once,
    // matching mdream).  colspan: expanded by `close_table_cell` adding
    // `(colspan - 1)` empty cells so Tier-2's column-count expectations are
    // met (without this, infobox-style `<th colspan="2">` rows trigger Tier-2's
    // layout-table fallback in close_table on what should be a normal GFM table).
    let colspan = find_attr(attrs, b"colspan")
        .and_then(|b| std::str::from_utf8(b).ok())
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(1)
        .max(1);
    if let Some(ts) = state.table_stack.last_mut() {
        ts.current_cell.clear();
        ts.in_cell = true;
        ts.definition_in_cell = false;
        ts.current_cell_colspan = colspan;
        if is_header {
            ts.has_th = true;
        }
    }
    Ok(())
}
