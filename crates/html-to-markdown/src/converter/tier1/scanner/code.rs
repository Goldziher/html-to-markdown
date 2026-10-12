fn close_code(
    state: &mut Tier1State,
    frame: &OpenTag,
    trim_boundary_whitespace: bool,
    options: &ConversionOptions,
) -> Result<(), BailReason> {
    if state.escape_ctx.contains(EscapeCtx::PRE) || state.escape_ctx.contains(EscapeCtx::CODE) {
        return Ok(());
    }
    // ~keep Phase CC: smart backtick escaping (mirrors inline/code.rs:260).
    // Open emitted nothing; content from `frame.content_start` to buf
    // end is the raw code content.  Choose num_backticks + delimiter
    // spaces from that slice, then truncate and re-emit wrapped.
    let buf = state.cell_or_output_mut();
    let content_start = clamp_to_char_boundary(buf, frame.content_start);
    if content_start >= buf.len() {
        // ~keep issue #481: a body that WAS non-empty but entirely whitespace before
        // `flush_text` dropped it is not an empty one. Tier-2 keeps `<code> </code>` as
        // a code span whose content is that space; erasing it here, as the genuinely
        // empty `<code></code>` case correctly does, would drop both the span and the
        // word separator it stands between.
        if frame.dropped_whitespace_only_text {
            return Err(BailReason::WhitespaceOnlyInlineEmphasis);
        }
        // ~keep No content emitted between open and close — Tier-2 emits
        // nothing for empty <code></code>.
        return Ok(());
    }

    // ~keep issue #483, backtick form: this span's opening backtick is about to be inserted
    // right after a preceding code span's closing one, which reparses as a single span
    // carrying both literal backticks. Tier-2 merges the two spans
    // (`wrapped::emit_code_span`); Tier-1 does not, so it bails.
    if buf[..content_start].ends_with('`') {
        return Err(BailReason::AdjacentInlineEmphasis);
    }

    // ~keep issue #487: `content` may contain internal '\n' bytes, each marking where a
    // `<br>` split this span (`emit_void`'s `TagKind::LineBreak` arm) -- a text
    // node's own line ending never reaches this buffer as a bare '\n' (`flush_text`'s
    // `in_code` branch already folds it to a space). Split on it and render each
    // segment as its own smart-escaped backtick span (`format_inline_code_segment`,
    // computed per segment rather than once over the whole original content), joined
    // by the "  \n" hard-break marker OUTSIDE the backticks -- the only marker shape
    // this scanner ever needs, since the router bails to Tier-2 whenever
    // `newline_style` is not `Spaces`. An empty segment (an adjacent, leading, or
    // trailing `<br>`) is dropped rather than rendered as a dangling empty `` `` ``
    // pair with nothing before or after it. Only the FIRST segment can ever be
    // adjacent to a preceding sibling's closing backtick -- checked once, above,
    // against `buf[..content_start]` before this loop runs -- every later segment is
    // preceded by our own separator instead.
    let (leading, content, trailing) = code_span_parts(&buf[content_start..], trim_boundary_whitespace);
    buf.truncate(content_start);
    // ~keep White space on both sides of the element start is one run: Tier-2's `push_inline_prefix`.
    // ~keep A line end in `leading` is no white space of the source: a block in the code wrote
    // ~keep its separator across the start of the content (`one\n<kbd><div>y</div></kbd>`), or a
    // ~keep `<br>` its split marker. It is written as before the rules of running text.
    if leading.contains('\n') || crate::converter::utility::white_space::space_is_owed(buf) {
        buf.push_str(&leading);
    }

    let mut first = true;
    for segment in content.split('\n').filter(|segment| !segment.is_empty()) {
        if !first {
            buf.push_str(crate::converter::main_helpers::hard_break_marker(options));
        }
        format_inline_code_segment(buf, segment);
        first = false;
    }
    buf.push_str(&trailing);
    Ok(())
}

fn code_span_parts(original: &str, trim_boundary_whitespace: bool) -> (String, String, String) {
    let trimmed = original.trim();
    if !trim_boundary_whitespace || trimmed.is_empty() {
        return (String::new(), original.to_owned(), String::new());
    }
    let leading_len = original.len() - original.trim_start().len();
    let trailing_start = original.trim_end().len();
    (
        original[..leading_len].to_owned(),
        trimmed.to_owned(),
        original[trailing_start..].to_owned(),
    )
}

/// Wrap `content` in backtick delimiters, choosing the fence width and delimiter-space
/// padding from `content` alone.
///
/// The per-segment counterpart to what this replaced: `close_code` used to compute these
/// once over the whole span's content, but a `<br>`-split span now needs them computed
/// independently for each segment, since a backtick run in one half must not force a wider
/// fence on a half that has none (issue #487). Mirrors
/// `handlers::code_block::format_inline_code`.
fn format_inline_code_segment(buf: &mut String, content: &str) {
    let contains_backtick = content.contains('`');
    let first_char = content.chars().next();
    let last_char = content.chars().last();
    let starts_with_space = first_char == Some(' ');
    let ends_with_space = last_char == Some(' ');
    let starts_with_backtick = first_char == Some('`');
    let ends_with_backtick = last_char == Some('`');
    // ~keep No all-spaces case: CommonMark strips one space from each end of a code span
    // only when the content is NOT entirely spaces, so padding an all-spaces body
    // changes what it contains. Mirrors `handlers::code_block::format_inline_code`.
    let needs_spaces =
        starts_with_backtick || ends_with_backtick || (starts_with_space && ends_with_space && contains_backtick);
    let num_backticks = if contains_backtick {
        min_safe_code_span_delimiter_length(content)
    } else {
        1
    };

    for _ in 0..num_backticks {
        buf.push('`');
    }
    if needs_spaces {
        buf.push(' ');
    }
    buf.push_str(content);
    if needs_spaces {
        buf.push(' ');
    }
    for _ in 0..num_backticks {
        buf.push('`');
    }
}

/// Smallest backtick-run length (starting at 1) that does not occur as a run inside `content`.
///
/// ~keep Mirrors `converter::handlers::code_block::min_safe_code_span_delimiter_length`
/// byte-for-byte. CommonMark
/// closes an inline code span at the next backtick string of the *same* length as the
/// opener (6.1), so `longest_run + 1` unconditionally over-escapes: content `` `` `` (a
/// single length-2 run, no length-1 run) is valid with a single backtick delimiter.
fn min_safe_code_span_delimiter_length(content: &str) -> usize {
    let mut run_lengths = std::collections::HashSet::new();
    let mut current = 0usize;
    for c in content.chars() {
        if c == '`' {
            current += 1;
        } else {
            if current > 0 {
                run_lengths.insert(current);
            }
            current = 0;
        }
    }
    if current > 0 {
        run_lengths.insert(current);
    }

    let mut candidate = 1usize;
    while run_lengths.contains(&candidate) {
        candidate += 1;
    }
    candidate
}

/// Returns `true` if every byte of `needle` appears in `haystack`, in order, not
/// necessarily contiguously (a byte-level subsequence test). O(haystack.len()), no
/// allocation.
///
/// ~keep Used by `close_link`'s nested-markup autolink guard: greedy left-to-right
/// matching is correct for subsequence testing regardless of UTF-8 char
/// boundaries — if a char sequence is a subsequence of another, each char's
/// byte run appears intact and in order, so the greedy byte scan below always
/// finds it. A byte-level "match" that happens to straddle char boundaries can
/// only make this return `true` MORE often than a char-aware version would,
/// never less — which only makes the caller's bail more conservative, never
/// less safe.
fn is_byte_subsequence(needle: &str, haystack: &str) -> bool {
    let mut needle_bytes = needle.bytes();
    let Some(mut want) = needle_bytes.next() else {
        return true;
    };
    for byte in haystack.bytes() {
        if byte == want {
            match needle_bytes.next() {
                Some(next_want) => want = next_want,
                None => return true,
            }
        }
    }
    false
}

/// Rewrites the link the scanner has already opened in `dest` into GFM autolink form
/// (`<href>`) when Tier-2 would do the same, returning `Ok(true)` when it did.
///
/// Mirrors Tier-2's predicate at `handlers/link.rs:91-101` exactly: `options.autolinks &&
/// !options.default_title && href non-empty && has_uri_scheme(href) && (label == href ||
/// (mailto: && label == href[7..]))`. `dest[trim_start..]` is Tier-2's `raw_text` for the
/// common flat-text case -- both sides have already collapsed whitespace/NBSP the same way,
/// and neither has run `escape_link_label` or bracket-wrapping yet.
fn try_emit_autolink(
    dest: &mut String,
    trim_start: usize,
    href_str: &str,
    has_nested_tag: bool,
    options: &ConversionOptions,
) -> Result<bool, BailReason> {
    if !options.autolinks || options.default_title || href_str.is_empty() || !has_uri_scheme(href_str) {
        return Ok(false);
    }
    let mailto_suffix = href_str.strip_prefix("mailto:");
    let label = &dest[trim_start..];
    if has_nested_tag {
        // ~keep A nested tag means the buffer is no longer flat text, so a literal
        // `label == href` compare is untrustworthy: `<b>u</b>` renders as `**u**`,
        // which never equals bare `u`, yet Tier-2 -- which compares against the
        // TAG-STRIPPED text -- would autolink it. Bailing on every nested tag is not
        // the fix either: measured against `tools/benchmark-harness/fixtures`,
        // 85-100% of scheme-href `<a>` elements on every fixture that has any are
        // wrapped in at least a `<span>`, so a blanket bail would drop nearly every
        // real page off the Tier-1 fast path.
        // ~keep
        // ~keep Instead, a cheap PROVABLY CONSERVATIVE precheck. Every transformation
        // Tier-1 applies while rendering a label (emphasis/strong/strikethrough and
        // inserted markers, code-span backtick fences, label-bracket escaping,
        // `![alt](src)` for a nested `<img>`) only INSERTS characters; none delete or
        // substitute the underlying text. Tier-2's `raw_text` is that same underlying
        // text with tags stripped and whitespace collapsed, both of which only REMOVE
        // characters. So `raw_text` is always a subsequence of this buffer, and if
        // `raw_text == target` then `target` is a subsequence of the buffer too.
        // Contrapositive: a `target` that is NOT a subsequence proves Tier-2 cannot
        // autolink either, so falling through to the normal link form is safe. Bail
        // only when the precheck cannot rule it out.
        let could_be_autolink = is_byte_subsequence(href_str, label)
            || mailto_suffix.is_some_and(|suffix| is_byte_subsequence(suffix, label));
        if could_be_autolink {
            return Err(BailReason::LinkAutolinkNestedMarkup);
        }
        // ~keep Ruled out above, so the exact `label == target` match below cannot fire
        // either -- equality would imply the subsequence that was just disproved.
    }
    let rendered = if label == href_str {
        href_str
    } else if mailto_suffix == Some(label) {
        // ~keep `mailto_suffix` is `Some` on this arm by construction.
        mailto_suffix.unwrap_or(href_str)
    } else {
        return Ok(false);
    };
    // ~keep An autolink has no bracket wrapping at all, but the scanner already emitted
    // the opening `[` at link-open time, before it could know the label. Remove it,
    // mirroring the href-less branch in `close_link` that does the same cleanup.
    let label_start = if let Some(bracket_pos) = dest[..trim_start].rfind('[') {
        dest.remove(bracket_pos);
        trim_start - 1
    } else {
        trim_start
    };
    let rendered = rendered.to_owned();
    dest.truncate(label_start);
    dest.push('<');
    dest.push_str(&rendered);
    dest.push('>');
    Ok(true)
}

/// Normalizes the link label Tier-1 has just written in place, as Tier-2's `normalize_link_label`
/// does with the functions of that rule: each segment between two hard break markers collapses
/// its white space to single spaces (a no-break space, and a line end that a block in the label
/// left, included); the first segment loses the white space at its start and the last one the
/// white space at its end; a run of markers at an edge is one marker, and a label of only markers
/// is empty (issue #497); two markers in a row are kept apart. In a list item every line after a
/// marker then starts with the content column of the item (`indent`) and nothing else, as Tier-2's
/// `indent_hard_break_continuations` writes it after the collapse.
///
/// ~keep A label of ASCII text with single spaces and no marker is already normal and is not read
/// ~keep again.
fn normalize_label_in_place(dest: &mut String, trim_start: usize, marker: &'static str, indent: Option<&str>) {
    use crate::converter::utility::content::{
        assemble_label, collapse_whitespace_into, indent_continuation_lines, protect_adjacent_hard_breaks,
    };
    let label = &dest[trim_start..];
    let is_normal = !label.contains(['\t', '\n', '\r'])
        && !label.contains("  ")
        && !label.chars().any(crate::text::is_unicode_space);
    if is_normal {
        return;
    }
    let mut segments = Vec::new();
    let mut markers = Vec::new();
    let mut rest = label;
    loop {
        let (text, next) = match rest.find(marker) {
            Some(pos) => (&rest[..pos], Some(&rest[pos + marker.len()..])),
            None => (rest, None),
        };
        let mut segment = String::with_capacity(text.len());
        collapse_whitespace_into(&mut segment, text);
        segments.push(segment);
        let Some(next) = next else {
            break;
        };
        markers.push(marker);
        rest = next;
    }
    let normalized = assemble_label(segments, markers);
    let mut normalized = protect_adjacent_hard_breaks(&normalized).into_owned();
    if let Some(indent) = indent {
        indent_continuation_lines(&mut normalized, indent);
    }
    dest.truncate(trim_start);
    dest.push_str(&normalized);
}

/// Gives the link label Tier-1 has just written in place to the edge rule of both converters
/// (`link_label_edges`) and writes what the rule returns: the space of the label start before the
/// `[`, where a space is owed, then the label. Gives back where the label starts now and the
/// white space of the label end, for the caller to write after the link.
///
/// ~keep The label holds its white space as the text scanner wrote it, the white space it starts
/// ~keep with included, and the rule decides the edges (issues #800, #844, #845): this scanner
/// ~keep has no rule of its own for a label edge. A label with no text has no ends.
fn move_label_edges(
    dest: &mut String,
    label_start: usize,
    space_owed_before_link: bool,
    marker: &'static str,
    indent: Option<&str>,
) -> (usize, &'static str) {
    let content = dest.split_off(label_start);
    let (prefix, suffix, label) = crate::converter::utility::content::link_label_edges(&content);
    let has_text = !label.is_empty();
    let mut label_start = label_start;
    if has_text && !prefix.is_empty() && space_owed_before_link && dest.ends_with('[') {
        let bracket_pos = dest.len() - 1;
        dest.insert_str(bracket_pos, prefix);
        label_start += prefix.len();
    }
    dest.push_str(label);
    normalize_label_in_place(dest, label_start, marker, indent);
    // ~keep A label whose text is blank has no text: Tier-2 names such a link by its address or
    // ~keep its title, or leaves it out, and this scanner leaves that to Tier-2 below.
    if dest[label_start..].trim().is_empty() {
        dest.truncate(label_start);
    }
    (label_start, suffix)
}

/// Where the text before the `[` of the link that is closing ends, when the marks of the inline
/// elements that opened right before the link are read past: `bracket_pos` is the `[`.
///
/// ~keep Tier-2 builds the body of an inline element in a buffer of its own and asks
/// ~keep `space_is_owed_in` at each level: an empty block buffer owes no space. This scanner
/// ~keep writes the marks of bold and italic text in the one buffer when they open, so a space
/// ~keep before the `[` of `<p><b><a href="/p">&nbsp;Go</a></b></p>` is owed by the text behind
/// ~keep the marks, which is the block start and owes none.
fn text_end_before_link(state: &Tier1State, bracket_pos: usize) -> usize {
    let mut pos = bracket_pos;
    for frame in state.stack.iter().rev() {
        let width = emitted_inline_marker_width(state, frame);
        if width == 0 || frame.content_start != pos {
            break;
        }
        pos = pos.saturating_sub(width);
    }
    pos
}

fn close_link(state: &mut Tier1State, frame: &OpenTag, options: &ConversionOptions) -> Result<(), BailReason> {
    // ~keep Close the link: `](href "title")` or `](href)`
    // If no href, just emit the text as-is (Tier-2 behaviour: no link markup).
    // Link state was pushed to state.link_stack at open; pop it now.
    let (href, title, has_nested_tag, tier2_if_no_text) = state.link_stack.pop().unwrap_or((None, None, false, false));
    let marker = crate::converter::main_helpers::hard_break_marker(options);
    let dest = state.cell_or_output_mut();
    let trim_start = clamp_to_char_boundary(dest, frame.content_start);
    // ~keep A link with no address is running text in Tier-2, which keeps the white space that
    // ~keep its content ends with. The edge rule below moves it, so the page goes to Tier-2 (the
    // ~keep scanner does the same where such a link opens, for the white space at its start).
    if href.is_none() && dest[trim_start..].ends_with(char::is_whitespace) {
        return Err(BailReason::Classifier);
    }
    // ~keep One run of white space on both sides of the link start is one space, before the `[`
    // ~keep (rule 4 of the white space module); the start of a block owes none.
    let text_end = text_end_before_link(state, trim_start.saturating_sub(1));
    // ~keep The content column of the list item that holds the link, for the lines of the label
    // ~keep after a hard break (Tier-2's `indent_hard_break_continuations`).
    let in_list_item = state.stack.iter().any(|frame| matches!(frame.spec.kind, TagKind::ListItem));
    let indent = in_list_item
        .then(|| {
            crate::converter::list::utils::continuation_indent_string(state.list_continuation_indent_width(), options)
        })
        .flatten();
    let dest = state.cell_or_output_mut();
    let space_owed_before_link =
        text_end > 0 && crate::converter::utility::white_space::space_is_owed(&dest[..text_end]);
    let (trim_start, suffix) = move_label_edges(dest, trim_start, space_owed_before_link, marker, indent.as_deref());
    // ~keep Tier-2 decides what a link with no text writes: the name of the link as its label,
    // ~keep nothing when the link points into its own page, and the label of a link that holds
    // ~keep only empty elements, or only a title. This scanner has none of these rules, so it
    // ~keep leaves the page to Tier-2.
    let named_or_own_page = tier2_if_no_text && href.is_some();
    if dest.len() == trim_start && (named_or_own_page || has_nested_tag || title.is_some()) {
        return Err(BailReason::LinkEmptyLabel);
    }
    if let Some(href_str) = href.as_deref() {
        if try_emit_autolink(dest, trim_start, href_str, has_nested_tag, options)? {
            dest.push_str(suffix);
            return Ok(());
        }
    }
    // ~keep Wikipedia back-reference normalisation (Tier-2 `handlers/link.rs:205`):
    // a label of exactly `^` paired with an `#anchor` href is rewritten to
    // `↑` so it does not look like Markdown's footnote syntax.
    if let Some(href_str) = href.as_deref() {
        if href_str.starts_with('#') && dest.len() == trim_start + 1 && dest.as_bytes()[trim_start] == b'^' {
            dest.truncate(trim_start);
            dest.push('↑');
        }
    }
    if let Some(href) = href.filter(|value| !value.is_empty()) {
        emit_markdown_link_close(dest, trim_start, &href, title.as_deref(), options);
        // ~keep A link with no text still writes `[](href)`: content, as in Tier-2's `convert_node`.
        state.end_document_start_if_written(trim_start.saturating_sub(1));
    } else if let Some(bracket_pos) = dest[..trim_start].rfind('[') {
        dest.remove(bracket_pos);
    }
    // ~keep The white space of the label end goes after the link, and after the text of a link
    // ~keep with an empty address.
    state.cell_or_output_mut().push_str(suffix);
    Ok(())
}

fn emit_markdown_link_close(
    dest: &mut String,
    trim_start: usize,
    href: &str,
    title: Option<&str>,
    options: &ConversionOptions,
) {
    if let std::borrow::Cow::Owned(escaped_label) =
        crate::converter::utility::escaping::escape_link_label(&dest[trim_start..])
    {
        dest.truncate(trim_start);
        dest.push_str(&escaped_label);
    }
    dest.push_str("](");
    crate::converter::inline::link::append_url_destination(dest, href, options.url_escape_style, title.is_some());
    if let Some(title) = title {
        let escaped_title = crate::converter::inline::link::escape_markdown_title(title);
        dest.push_str(" \"");
        dest.push_str(&escaped_title);
        dest.push('"');
    }
    dest.push(')');
}

fn close_list(state: &mut Tier1State, kind: ListKind) {
    state.list_depth = state.list_depth.saturating_sub(1);
    if matches!(kind, ListKind::Unordered) {
        state.ul_depth = state.ul_depth.saturating_sub(1);
    }
    // ~keep When inside a table cell, Tier-2 does NOT add a trailing newline after
    // the list — the cell accumulator handles any separators via the
    // `\n → space` replacement at cell-close time.
    if state.in_table_cell() {
        return;
    }
    let dest = state.cell_or_output_mut();
    if !dest.ends_with('\n') {
        dest.push('\n');
    }
    if kind == ListKind::Ordered {
        let end = dest.trim_end().len();
        state.last_ordered_list_end = Some(end);
    }
}

fn close_list_item(state: &mut Tier1State, frame: &OpenTag) -> Result<(), BailReason> {
    // ~keep When inside a table cell, Tier-2 does NOT add a trailing newline after
    // each list item (see list/item.rs: `if !ctx.in_table_cell { ... \n ... }`).
    // Items are concatenated directly in the cell accumulator.
    if state.in_table_cell() {
        let cell_buf = state.cell_or_output_mut();
        while cell_buf.ends_with(' ') || cell_buf.ends_with('\t') {
            cell_buf.pop();
        }
        return Ok(());
    }
    state.list_item_marker_widths.pop();
    let after_text = state.list_items_after_text.pop().unwrap_or_default();
    trim_trailing_inline_whitespace(state);
    // ~keep A nested item, or an item after text inside its list, with nothing on its marker line
    // ~keep cannot interrupt the text before it (issue #667). Tier 2 decides whether that text is
    // ~keep an open paragraph.
    let follows_text = state.list_depth > 1 || after_text;
    let dest = state.cell_or_output_mut();
    let marker_line = dest
        .get(clamp_to_char_boundary(dest, frame.content_start)..)
        .unwrap_or_default();
    if follows_text && marker_line.split('\n').next().unwrap_or_default().trim().is_empty() {
        return Err(BailReason::EmptyNestedListItem);
    }
    // ~keep Phase EE: loose-list separator.  When this item had block-level
    // children (its content range contains a `\n\n` block separator),
    // mirror Tier-2's `handle_li` ensure_trailing_blank_line behaviour
    // so the next sibling `<li>` starts after a blank line.  Plain text
    // items still get the tight `\n` terminator.
    let had_block_children = {
        let start = clamp_to_char_boundary(dest, frame.content_start);
        dest[start..].contains("\n\n")
    };
    if had_block_children {
        if !dest.ends_with("\n\n") {
            if dest.ends_with('\n') {
                dest.push('\n');
            } else {
                dest.push_str("\n\n");
            }
        }
    } else if !dest.is_empty() && !dest.ends_with('\n') {
        dest.push('\n');
    }
    Ok(())
}

// ~keep ── Definition-list helpers ───────────────────────────────────────────────────
// ~keep
// ~keep Tier-2 reference: crates/html-to-markdown/src/converter/list/definition.rs.
// Tier-2 builds the full <dl> content in a buffer, trims it, then emits with
// "\n\n" boundaries. <dt> emits trimmed term + "\n"; <dd> emits trimmed
// description + "\n\n". Tier-1 streams the same shape by:
//   - open_dl: ensure blank line; record content_start on the frame
//   - close_dt: trim trailing whitespace, push "\n"
//   - close_dd: trim trailing whitespace, push "\n\n"
//   - close_dl: trim leading/trailing whitespace inside the dl range, then
//               normalise the trailing separator to "\n\n"
// ~keep
// ~keep Bails on dl/dt/dd are removed (see bail_unsupported). Implicit close of an
// open dt/dd when a sibling dt/dd opens is wired via OptionalCloseRule::
// CloseSiblingDtDd in spec_rules.rs and runs the same close_dt/close_dd path
// through emit_close_for_implicit.

fn open_dl(state: &mut Tier1State) {
    if state.in_table_cell() {
        return;
    }
    state.ensure_blank_line();
}

fn open_dt(state: &mut Tier1State) {
    mark_definition_in_cell(state);
}

fn open_dd(state: &mut Tier1State) {
    mark_definition_in_cell(state);
}

fn mark_definition_in_cell(state: &mut Tier1State) {
    if let Some(ts) = state.table_stack.last_mut().filter(|ts| ts.in_cell) {
        ts.definition_in_cell = true;
    }
}

fn close_dt(state: &mut Tier1State) {
    if state.in_table_cell() {
        return;
    }
    trim_trailing_inline_whitespace(state);
    let buf = state.cell_or_output_mut();
    if buf.is_empty() || buf.ends_with('\n') {
        return;
    }
    buf.push('\n');
}

fn close_dd(state: &mut Tier1State) {
    if state.in_table_cell() {
        return;
    }
    trim_trailing_inline_whitespace(state);
    let buf = state.cell_or_output_mut();
    if buf.is_empty() {
        return;
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

fn close_dl(state: &mut Tier1State, frame: &OpenTag, options: &ConversionOptions) {
    if state.in_table_cell() {
        return;
    }
    let buf = state.cell_or_output_mut();
    // ~keep Empty dl: emit nothing (matches Tier-2 which skips when trimmed content
    // is empty).
    if buf.len() <= frame.content_start {
        return;
    }
    // ~keep A last line of indented code keeps its line end, as in Tier-2: the spaces are code.
    let content_start = clamp_to_char_boundary(buf, frame.content_start);
    let kept_end = content_start
        + crate::converter::code_scan::quote_content_range(&buf[content_start..], options.code_block_style).end;
    // ~keep Tier-2 trims the dl's accumulated content, so any trailing whitespace
    // from the last dt/dd close should collapse to a single "\n\n" separator.
    while buf.len() > frame.content_start.max(kept_end) {
        let last = buf.as_bytes()[buf.len() - 1];
        if matches!(last, b' ' | b'\t' | b'\n' | b'\r') {
            buf.pop();
        } else {
            break;
        }
    }
    if buf.len() == frame.content_start {
        return;
    }
    buf.push_str("\n\n");
}
