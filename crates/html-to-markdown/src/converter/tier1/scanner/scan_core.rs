/// Entry point for the Tier-1 scanner.
pub fn scan(
    html: &str,
    options: &ConversionOptions,
    effective_base: Option<std::rc::Rc<url::Url>>,
) -> Result<ScanOutput, BailReason> {
    Tier1Scanner::new(html, options, effective_base).run()
}

struct Tier1Scanner<'a> {
    html: &'a str,
    bytes: &'a [u8],
    options: &'a ConversionOptions,
    state: Tier1State,
    table_probes: Vec<TableLayoutProbe>,
    pos: usize,
    text_start: usize,
}

impl<'a> Tier1Scanner<'a> {
    fn new(html: &'a str, options: &'a ConversionOptions, effective_base: Option<std::rc::Rc<url::Url>>) -> Self {
        Self {
            html,
            bytes: html.as_bytes(),
            options,
            state: Tier1State::new(html.len(), effective_base),
            table_probes: Vec::new(),
            pos: 0,
            text_start: 0,
        }
    }

    fn run(mut self) -> Result<ScanOutput, BailReason> {
        while self.pos < self.bytes.len() {
            if self.bytes[self.pos] == b'<' {
                self.scan_markup()?;
            } else {
                self.advance_text();
            }
        }
        finish_scan(
            self.state,
            self.html,
            self.text_start,
            self.pos,
            self.options,
            self.table_probes,
        )
    }

    fn scan_markup(&mut self) -> Result<(), BailReason> {
        self.flush_before_markup()?;
        let next = self.bytes.get(self.pos + 1).copied().unwrap_or(0);
        if next == b'!' {
            return self.scan_bang();
        }
        if next == b'?' {
            return Err(BailReason::Classifier);
        }
        if next == b'/' {
            return self.scan_close_tag();
        }
        if !parse::is_tag_name_start(next) {
            return self.emit_literal_lt();
        }
        self.scan_open_tag()
    }

    fn flush_before_markup(&mut self) -> Result<(), BailReason> {
        if self.text_start >= self.pos {
            return Ok(());
        }
        let next_tag_is_html = upcoming_tag_is_named(self.bytes, self.pos, b"html");
        let ignored_before_head = self.state.head_range.is_none()
            && crate::converter::main_helpers::is_ignorable_before_head(
                &self.html[self.text_start..self.pos],
                next_tag_is_html,
            );
        if ignored_before_head {
            return Ok(());
        }
        self.state.start_body(self.text_start);
        let upcoming = UpcomingTextSibling {
            is_list: upcoming_tag_is_list_open(self.bytes, self.pos),
            is_img: upcoming_tag_is_named(self.bytes, self.pos, b"img"),
            is_span: upcoming_tag_is_named(self.bytes, self.pos, b"span"),
            is_inline: upcoming_tag_is_inline(self.bytes, self.pos),
        };
        flush_text(
            &mut self.state,
            TextFlush {
                raw: &self.html[self.text_start..self.pos],
                base_offset: self.text_start,
                upcoming,
                br_in_tables: self.options.br_in_tables,
                output_format: self.options.output_format,
            },
        )
    }

    fn scan_bang(&mut self) -> Result<(), BailReason> {
        if self.html[self.pos..].starts_with("<![CDATA[") {
            return Err(BailReason::Cdata { offset: self.pos });
        }
        self.pos = skip_bang(self.bytes, self.pos)?;
        self.text_start = self.pos;
        Ok(())
    }

    fn scan_close_tag(&mut self) -> Result<(), BailReason> {
        let name_start = self.pos + 2;
        let name_end = parse::scan_tag_name(self.bytes, name_start);
        if name_end == name_start {
            return Err(BailReason::LiteralLt { offset: self.pos });
        }
        let close = parse::find_tag_close(self.bytes, name_end).ok_or(BailReason::LiteralLt { offset: self.pos })?;
        let tag_name = &self.bytes[name_start..name_end];
        let len_before_close = self.state.cell_or_output_mut().len();
        let join_open = self.state.pending_newline_join == Some(len_before_close);
        emit_close(&mut self.state, tag_name, self.options, &mut self.table_probes)?;
        let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
        let name_lower = lowercase_into(tag_name, &mut name_buf);
        let ends_own_line = tier1::lookup(name_lower).is_some_and(|spec| spec.is_block)
            || matches!(
                name_lower,
                b"optgroup" | b"button" | b"progress" | b"meter" | b"output" | b"datalist"
            );
        let dest = self.state.cell_or_output_mut();
        let (dest_len, ends_in_newline) = (dest.len(), dest.ends_with('\n'));
        self.state.pending_newline_join = (join_open && ends_in_newline && !ends_own_line).then_some(dest_len);
        self.pos = close.0 + 1;
        self.text_start = self.pos;
        Ok(())
    }

    fn emit_literal_lt(&mut self) -> Result<(), BailReason> {
        self.state.start_body(self.pos);
        flush_text(
            &mut self.state,
            TextFlush {
                raw: "<",
                base_offset: self.pos,
                upcoming: UpcomingTextSibling::default(),
                br_in_tables: self.options.br_in_tables,
                output_format: self.options.output_format,
            },
        )?;
        self.pos += 1;
        self.text_start = self.pos;
        Ok(())
    }

    fn scan_open_tag(&mut self) -> Result<(), BailReason> {
        let name_start = self.pos + 1;
        let name_end = parse::scan_tag_name(self.bytes, name_start);
        let tag_name_bytes = &self.bytes[name_start..name_end];

        let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
        let name_lower = lowercase_into(tag_name_bytes, &mut name_buf);
        self.prepare_open_tag(name_lower)?;
        if name_lower == b"svg" && self.scan_svg(name_end)? {
            return Ok(());
        }
        if name_lower == b"template" && self.scan_template(name_end) {
            return Ok(());
        }

        let spec = resolve_tag_spec(name_lower, tag_name_bytes, self.pos)?;
        if matches!(spec.kind, TagKind::Ignored) && spec.is_rawtext {
            self.scan_rawtext_ignored(name_start, name_end, name_lower)?;
            return Ok(());
        }
        if matches!(spec.kind, TagKind::Ignored) {
            self.scan_ignored(spec, name_end, name_lower)?;
            return Ok(());
        }

        bail_unsupported(spec, self.pos)?;

        if self.skip_preprocessed_tag(name_lower, name_end)? {
            return Ok(());
        }

        // ~keep Tilde fences still require Tier-2; Tier-1 supports indented/backtick pre blocks.
        if matches!(spec.kind, TagKind::Pre) && self.options.code_block_style == crate::options::CodeBlockStyle::Tildes
        {
            return Err(BailReason::Classifier);
        }

        let close = parse::find_tag_close(self.bytes, name_end).ok_or(BailReason::LiteralLt { offset: self.pos })?;

        let attrs = collect_open_attrs(spec, name_lower, self.bytes, name_end, close);

        self.pos = close.0 + 1;

        self.state.link_stack.iter_mut().for_each(|entry| entry.2 = true);

        if spec.is_void || close.1 {
            if cell_needs_tier2(&mut self.state, spec, name_lower) {
                return Err(BailReason::TableBlockChildInCell);
            }
            emit_void(&mut self.state, spec, name_lower, &attrs, self.html, self.options)?;
            self.text_start = self.pos;
            return Ok(());
        }

        self.close_implicit_tags(spec)?;
        validate_cell_open(&mut self.state, spec, name_lower)?;
        validate_list_open(&mut self.state, spec, name_lower)?;
        let (prev_ctx, ol_start) = self.prepare_open_metadata(spec, name_lower, &attrs)?;
        check_open_depth(&self.state, self.options)?;
        emit_open(
            &mut self.state,
            spec,
            name_lower,
            &attrs,
            &mut self.table_probes,
            self.options,
        )?;
        push_open_frame(
            &mut self.state,
            spec,
            name_lower,
            name_start..name_end,
            prev_ctx,
            ol_start,
        );
        apply_open_escape_ctx(&mut self.state, spec);

        self.text_start = self.pos;
        Ok(())
    }

    fn prepare_open_tag(&mut self, name_lower: &[u8]) -> Result<(), BailReason> {
        // ~keep Only a br or stripped raw-text tag keeps a pending newline join open.
        if !matches!(name_lower, b"br" | b"script" | b"style") {
            self.state.pending_newline_join = None;
        }
        let tag_end = crate::converter::utility::preprocessing::find_tag_end(self.bytes, self.pos + 1)
            .unwrap_or(self.bytes.len());
        let tag_slice = &self.html[self.pos..tag_end];
        if crate::converter::utility::preprocessing::tag_has_hidden_attribute(tag_slice)
            || crate::converter::utility::preprocessing::tag_has_hidden_style(tag_slice)
        {
            return Err(BailReason::HiddenElement { offset: self.pos });
        }
        if crate::converter::main_helpers::starts_body(name_lower) {
            self.state.start_body(self.pos);
        }
        Ok(())
    }

    fn scan_svg(&mut self, name_end: usize) -> Result<bool, BailReason> {
        let tag_open_start = self.pos;
        let Some((close_pos, is_self_closing)) = parse::find_tag_close(self.bytes, name_end) else {
            self.pos = self.bytes.len();
            self.text_start = self.pos;
            return Ok(true);
        };
        let open_tag_end = close_pos + 1;
        let svg_end = if is_self_closing {
            open_tag_end
        } else {
            find_svg_close(self.bytes, open_tag_end).unwrap_or(self.bytes.len())
        };
        emit_svg_from_slice(
            &self.html[tag_open_start..svg_end],
            tag_open_start,
            &mut self.state,
            self.options,
        )?;
        self.pos = svg_end;
        self.text_start = self.pos;
        Ok(true)
    }

    fn scan_template(&mut self, name_end: usize) -> bool {
        let Some((close_pos, is_self_closing)) = parse::find_tag_close(self.bytes, name_end) else {
            self.pos = self.bytes.len();
            self.text_start = self.pos;
            return true;
        };
        let open_tag_end = close_pos + 1;
        self.pos = if is_self_closing {
            open_tag_end
        } else {
            find_balanced_close(self.bytes, open_tag_end, b"template").unwrap_or(self.bytes.len())
        };
        self.text_start = self.pos;
        true
    }

    fn skip_preprocessed_tag(&mut self, name_lower: &[u8], name_end: usize) -> Result<bool, BailReason> {
        if !is_preprocessing_skip_candidate(name_lower) {
            return Ok(false);
        }
        let close = parse::find_tag_close(self.bytes, name_end).ok_or(BailReason::LiteralLt { offset: self.pos })?;
        let attrs_end = if close.1 { close.0.saturating_sub(1) } else { close.0 };
        let attrs = parse::collect_attrs(self.bytes, name_end, attrs_end);
        let is_page_header = name_lower == b"header" && header_is_page_level(&self.state, self.html);
        if !should_skip_preprocessing(name_lower, &attrs, self.options, is_page_header) {
            return Ok(false);
        }
        if !self.state.in_table_cell() && self.state.list_continuation_indent_width() > 0 {
            return Err(BailReason::ListItemUnsupportedBlockChild);
        }
        self.state.last_closed_block = true;
        let open_end = close.0 + 1;
        self.pos = if close.1 {
            open_end
        } else {
            find_balanced_close(self.bytes, open_end, name_lower).unwrap_or(self.bytes.len())
        };
        self.text_start = self.pos;
        Ok(true)
    }

    fn close_implicit_tags(&mut self, spec: &TagSpec) -> Result<(), BailReason> {
        while let Some(top) = self.state.stack.last() {
            if !spec_rules::should_close_for_new_tag(top.spec, spec) {
                break;
            }
            emit_close_for_implicit(&mut self.state, self.options, &mut self.table_probes)?;
        }
        Ok(())
    }

    fn prepare_open_metadata(
        &mut self,
        spec: &TagSpec,
        name_lower: &[u8],
        attrs: &[(&[u8], Option<&[u8]>)],
    ) -> Result<(EscapeCtx, u16), BailReason> {
        let prev_ctx = self.state.escape_ctx;
        let ol_start = if matches!(spec.kind, TagKind::List(ListKind::Ordered)) {
            extract_ol_start(attrs)
        } else {
            1
        };
        if matches!(spec.kind, TagKind::Link) {
            let (href, title) = extract_link_attrs(attrs)?;
            let href = href.map(|value| {
                self.state
                    .effective_base
                    .as_deref()
                    .and_then(|base| crate::converter::url_resolve::resolve_link_url(base, &value))
                    .unwrap_or(value)
            });
            self.state
                .link_stack
                .push((href, title, false, find_attr(attrs, b"aria-label").is_some()));
        }
        if name_lower == b"abbr" {
            let title = find_attr(attrs, b"title")
                .map(decode_attr)
                .transpose()?
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty());
            self.state.abbr_titles.push(title);
        }
        Ok((prev_ctx, ol_start))
    }

    fn scan_rawtext_ignored(
        &mut self,
        name_start: usize,
        name_end: usize,
        name_lower: &[u8],
    ) -> Result<(), BailReason> {
        let open_end = match parse::find_tag_close(self.bytes, name_end) {
            Some(close) => close.0 + 1,
            None => self.bytes.len(),
        };
        self.pos = find_raw_text_close(self.bytes, open_end, name_lower).unwrap_or(self.bytes.len());

        // ~keep Tier-2's `strip_script_and_style_tags` preprocessing pass
        // (converter/utility/preprocessing.rs, outside tier1/) inserts a
        // boundary space *per removed element* when its source-adjacent
        // self.bytes are non-whitespace.  Two `<script>`/`<style>` tags sitting
        // back-to-back with zero separating whitespace therefore each
        // contribute a boundary space, and — because they collapse to a
        // single whitespace-only DOM text node — Tier-2's downstream
        // whitespace-mode handling of that node produces an idiosyncratic
        // byte pattern (observed: a stray `\n\n  \n` at the nuxt-example
        // fixture's trailing `<script><script></body>`) that is specific
        // to whitespace-only-node handling, not reproducible by mirroring
        // the boundary-space rule alone.  Bail so Tier-2 (authoritative)
        // handles this rare, adjacency-only case; the single-tag word-glue
        // mirror below still covers the common case.
        if is_adjacent_rawtext_ignored_open(self.bytes, self.pos) {
            return Err(BailReason::AdjacentRawTextTags { offset: self.pos });
        }
        // ~keep Tier-2 reads the whitespace on both sides of the removed element as one
        // ~keep text, so `First\n<script>x</script>\n<br>` ends in a blank line, as
        // ~keep `First\n\n<br>` does: the `<br>` must not remove the join (issue #683).
        let after = &self.bytes[self.pos..];
        let after_ws = &after[..after
            .iter()
            .position(|b| !b.is_ascii_whitespace())
            .unwrap_or(after.len())];
        let newline_before = name_start >= 2 && matches!(self.bytes[name_start - 2], b'\n' | b'\r');
        if (newline_before && after_ws.first().is_some_and(|byte| matches!(byte, b'\n' | b'\r')))
            || contains_blank_line(after_ws)
        {
            self.state.pending_newline_join = None;
        }

        self.text_start = self.pos;
        // ~keep Mirror the single-element boundary-space rule: a space is
        // inserted only when the removed tag would otherwise glue two
        // word characters together.  The "before" check uses the emitted
        // output tail (not the raw source byte) so that a space already
        // produced by a preceding sibling is never doubled up; the "after"
        // check peeks the next source byte, matching Tier-2's boundary
        // condition exactly.
        if name_lower == b"script" || name_lower == b"style" {
            let after_is_word = self.pos < self.bytes.len() && !self.bytes[self.pos].is_ascii_whitespace();
            if after_is_word {
                let dest = self.state.cell_or_output_mut();
                let ends_with_word = !dest.is_empty()
                    && !dest.ends_with(' ')
                    && !dest.ends_with('\t')
                    && !dest.ends_with('\n')
                    && !dest.ends_with('<')
                    && !dest.ends_with("<br>");
                if ends_with_word {
                    dest.push(' ');
                }
            }
        }
        Ok(())
    }

    fn scan_ignored(&mut self, spec: &TagSpec, name_end: usize, name_lower: &[u8]) -> Result<(), BailReason> {
        let open_end = match parse::find_tag_close(self.bytes, name_end) {
            Some(close) => close.0 + 1,
            None => self.bytes.len(),
        };
        if spec.is_void {
            self.pos = open_end;
            self.text_start = self.pos;
            return Ok(());
        }
        let (close_start, close_end) = match find_close_tag_range(self.bytes, open_end, name_lower) {
            Some(pair) => pair,
            None => (self.bytes.len(), self.bytes.len()),
        };
        if self.state.head_range.is_none() {
            self.state.head_range = Some(open_end..close_start);
        }
        self.pos = close_end;
        self.text_start = self.pos;
        Ok(())
    }

    fn advance_text(&mut self) {
        match memchr2(b'<', b'&', &self.bytes[self.pos..]) {
            Some(offset) if offset > 0 => self.pos += offset,
            Some(_) => self.pos += 1,
            None => self.pos = self.bytes.len(),
        }
    }
}

fn collect_open_attrs<'a>(
    spec: &TagSpec,
    name_lower: &[u8],
    bytes: &'a [u8],
    name_end: usize,
    close: (usize, bool),
) -> Vec<(&'a [u8], Option<&'a [u8]>)> {
    let needs_attrs = matches!(
        spec.kind,
        TagKind::Link
            | TagKind::Image
            | TagKind::List(ListKind::Ordered)
            | TagKind::Table
            | TagKind::TableCell { .. }
            | TagKind::Pre
            | TagKind::Code
            | TagKind::Blockquote
    ) || matches!(name_lower, b"abbr" | b"input");
    if !needs_attrs {
        return Vec::new();
    }
    let attrs_end = if close.1 { close.0.saturating_sub(1) } else { close.0 };
    parse::collect_attrs(bytes, name_end, attrs_end)
}

/// ~keep Custom elements use inline passthrough semantics; unknown standard tags bail.
fn resolve_tag_spec(name_lower: &[u8], tag_name: &[u8], offset: usize) -> Result<&'static TagSpec, BailReason> {
    if name_lower.contains(&b'-') {
        return Ok(&CUSTOM_ELEMENT_INLINE_SPEC);
    }
    tier1::lookup(name_lower).ok_or_else(|| BailReason::UnknownCustomElement {
        name: bytes_to_string(tag_name).into(),
        offset,
    })
}

fn validate_cell_open(state: &mut Tier1State, spec: &TagSpec, name_lower: &[u8]) -> Result<(), BailReason> {
    if cell_needs_tier2(state, spec, name_lower) {
        return Err(BailReason::TableBlockChildInCell);
    }
    let inlineable = matches!(
        spec.kind,
        TagKind::Paragraph
            | TagKind::Block
            | TagKind::Summary
            | TagKind::Figcaption
            | TagKind::Blockquote
            | TagKind::Pre
            | TagKind::List(_)
            | TagKind::ListItem
            | TagKind::Heading(_)
            | TagKind::DefinitionTerm
            | TagKind::DefinitionDescription
            | TagKind::Table
    );
    if state.in_table_cell() && spec.is_block && !inlineable {
        return Err(BailReason::TableBlockChildInCell);
    }
    Ok(())
}

fn validate_list_open(state: &mut Tier1State, spec: &TagSpec, name_lower: &[u8]) -> Result<(), BailReason> {
    if let TagKind::List(kind) = spec.kind {
        let ordered_ancestor = find_parent_list_kind(&state.stack) == Some(ListKind::Ordered);
        if kind != ListKind::Definition && state.list_depth > 0 && (kind == ListKind::Ordered || ordered_ancestor) {
            return Err(BailReason::ListNestedOrdered);
        }
        if let Some(end) = state.last_ordered_list_end.filter(|_| kind == ListKind::Ordered) {
            if state
                .cell_or_output_mut()
                .get(end..)
                .is_some_and(|rest| rest.trim().is_empty())
            {
                return Err(BailReason::OrderedListAfterOrderedList);
            }
        }
    }
    validate_list_item_block(state, spec, name_lower)
}

fn validate_list_item_block(state: &Tier1State, spec: &TagSpec, name_lower: &[u8]) -> Result<(), BailReason> {
    if state.in_table_cell() || state.list_continuation_indent_width() == 0 {
        return Ok(());
    }
    let bare_marker = line_is_bare_list_marker(&state.output);
    let unsupported = match spec.kind {
        TagKind::Blockquote | TagKind::Table | TagKind::Heading(_) => true,
        TagKind::Block | TagKind::List(ListKind::Definition) => true,
        TagKind::Paragraph => !bare_marker,
        TagKind::Pre => bare_marker,
        TagKind::List(_) | TagKind::ListItem | TagKind::DefinitionTerm | TagKind::DefinitionDescription => false,
        _ => is_block_tag(name_lower) && !bare_marker,
    };
    if unsupported {
        Err(BailReason::ListItemUnsupportedBlockChild)
    } else {
        Ok(())
    }
}

fn check_open_depth(state: &Tier1State, options: &ConversionOptions) -> Result<(), BailReason> {
    let max_depth = crate::converter::main_helpers::effective_max_depth(options);
    if state.stack.len() >= max_depth {
        return Err(BailReason::DepthLimitExceeded {
            depth: state.stack.len(),
            max_depth,
        });
    }
    Ok(())
}

fn push_open_frame(
    state: &mut Tier1State,
    spec: &'static TagSpec,
    name_lower: &[u8],
    name_range: std::ops::Range<usize>,
    prev_ctx: EscapeCtx,
    ol_start: u16,
) {
    let content_start = state.cell_or_output_mut().len();
    state.stack.push(OpenTag {
        spec,
        content_start,
        prev_escape_ctx: prev_ctx,
        list_index: 0,
        ol_start,
        name_range,
        dropped_whitespace_only_text: false,
        own_buffer: renders_into_own_buffer(spec.kind, name_lower, prev_ctx),
        starts_with_whitespace: false,
        children_in_own_buffer: matches!(name_lower, b"mark" | b"sub" | b"sup" | b"abbr")
            || matches!(spec.kind, TagKind::DefinitionTerm | TagKind::DefinitionDescription),
    });
}

/// ~keep EOF closes open frames, then applies Tier-2's trim/collapse/trailing-newline order.
fn finish_scan(
    mut state: Tier1State,
    html: &str,
    text_start: usize,
    pos: usize,
    options: &ConversionOptions,
    mut table_probes: Vec<TableLayoutProbe>,
) -> Result<ScanOutput, BailReason> {
    if text_start < pos {
        flush_text(
            &mut state,
            TextFlush {
                raw: &html[text_start..pos],
                base_offset: text_start,
                upcoming: UpcomingTextSibling::default(),
                br_in_tables: options.br_in_tables,
                output_format: options.output_format,
            },
        )?;
    }

    while !state.stack.is_empty() {
        let buf = &mut state.output;
        while matches!(buf.as_bytes().last(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            buf.pop();
        }
        emit_close_for_implicit(&mut state, options, &mut table_probes)?;
    }

    crate::converter::main_helpers::trim_line_end_whitespace(&mut state.output);
    if state.output.contains("\n\n\n") {
        collapse_excess_blank_lines(&mut state.output);
    }

    if !state.output.is_empty() {
        let trimmed_end = state.output.trim_end_matches('\n');
        if trimmed_end.is_empty() {
            state.output.clear();
        } else {
            let trimmed_len = trimmed_end.len();
            state.output.truncate(trimmed_len);
            state.output.push('\n');
        }
    }

    Ok(ScanOutput {
        body: state.output,
        head_range: state.head_range,
    })
}

// ~keep ── Bail guard ────────────────────────────────────────────────────────────────

/// Return `Err(BailReason::Classifier)` for tag kinds not supported in M9.
///
/// Table-related tags are now handled by the scanner (M9); they are no longer
/// bailed here.  Table-specific bail reasons are emitted by the table-handling
/// code in `emit_open` and `emit_close`.
/// Locate the matching close tag for `tag_name` starting at `open_end`.
///
/// Returns `Some((close_start, close_end))` where `close_start` is the byte
/// index of the `<` opening the `</tag>` close and `close_end` is the byte
/// index immediately after its `>`.  `None` when no matching close exists.
///
/// Used by `<head>` silent-skip to record the content slice
/// (`open_end..close_start`) for metadata extraction while advancing past the
/// entire `<head>…</head>` block.
fn find_close_tag_range(bytes: &[u8], open_end: usize, tag_name: &[u8]) -> Option<(usize, usize)> {
    let mut idx = open_end;
    while idx < bytes.len() {
        match memchr3(b'<', b'<', b'<', &bytes[idx..]) {
            Some(off) => idx += off,
            None => return None,
        }
        let is_close = bytes.get(idx + 1) == Some(&b'/');
        let name_start = idx + usize::from(is_close) + 1;
        if is_close && let Some(after) = matching_tag_after(bytes, name_start, tag_name) {
            return find_closing_bracket(bytes, after).map(|end| (idx, end));
        }
        idx += 1;
    }
    None
}

// ~keep ── SVG helpers ───────────────────────────────────────────────────────────────

/// Find the byte offset immediately after the matching `</svg>` close tag,
/// starting from `open_end` (the byte after the `>` of the opening `<svg ...>`).
///
/// Tracks nesting depth so nested `<svg>` elements (valid in SVG 1.1) are
/// handled correctly.  Returns `None` when no matching close is found.
fn find_svg_close(bytes: &[u8], open_end: usize) -> Option<usize> {
    find_balanced_close(bytes, open_end, b"svg")
}

/// Find the byte offset immediately after the matching close tag for
/// `tag_name`, starting from `open_end` (the byte after the `>` of the
/// opening tag).  Tracks nesting depth so nested same-name elements are
/// handled correctly.  Returns `None` when no matching close is found.
fn find_balanced_close(bytes: &[u8], open_end: usize, tag_name: &[u8]) -> Option<usize> {
    let mut idx = open_end;
    let mut depth = 1usize;
    while idx < bytes.len() {
        match memchr::memchr(b'<', &bytes[idx..]) {
            Some(off) => idx += off,
            None => return None,
        }
        let is_close = bytes.get(idx + 1) == Some(&b'/');
        let name_start = idx + usize::from(is_close) + 1;
        let Some(after) = matching_tag_after(bytes, name_start, tag_name) else {
            idx += 1;
            continue;
        };
        if is_close {
            depth -= 1;
            if depth == 0 {
                return find_closing_bracket(bytes, after);
            }
        } else if !tag_is_self_closing(bytes, after) {
            depth += 1;
        }
        idx += 1;
    }
    None
}

fn matching_tag_after(bytes: &[u8], name_start: usize, tag_name: &[u8]) -> Option<usize> {
    let after = name_start.checked_add(tag_name.len())?;
    if after > bytes.len() || !bytes[name_start..after].eq_ignore_ascii_case(tag_name) {
        return None;
    }
    matches!(
        bytes.get(after),
        Some(b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r') | None
    )
    .then_some(after)
}

fn find_closing_bracket(bytes: &[u8], after_name: usize) -> Option<usize> {
    memchr::memchr(b'>', bytes.get(after_name..)?).map(|offset| after_name + offset + 1)
}

fn tag_is_self_closing(bytes: &[u8], after_name: usize) -> bool {
    let mut quote = None;
    for (offset, byte) in bytes[after_name..].iter().copied().enumerate() {
        match byte {
            b'"' | b'\'' if quote == Some(byte) => quote = None,
            b'"' | b'\'' if quote.is_none() => quote = Some(byte),
            b'>' if quote.is_none() => return after_name + offset > 0 && bytes[after_name + offset - 1] == b'/',
            _ => {}
        }
    }
    false
}
