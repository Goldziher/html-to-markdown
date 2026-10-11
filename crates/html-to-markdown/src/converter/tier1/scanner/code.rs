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
    frame: &OpenTag,
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
    let bracket_search_end = clamp_to_char_boundary(dest, frame.content_start);
    let label_start = if let Some(bracket_pos) = dest[..bracket_search_end].rfind('[') {
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

/// Trim the link label Tier-1 has just written in place, keeping a hard line break that sits
/// at either edge of it.
///
/// A `<br>` against the `</a>` is real content: `<a href="H">A<br></a>B` renders as A, a line
/// break, then B, and `[A  \n](H)B` re-parses to exactly that (issue #497). The blanket
/// trailing-whitespace trim this replaced ate the marker's two spaces and its newline, since
/// flattened they are indistinguishable from incidental whitespace before a `</a>`.
///
/// Mirrors Tier-2's `assemble_label` (`utility/content.rs`) including its two edge rules: a
/// run of breaks at one edge collapses to a single break, because two adjacent markers put a
/// blank line in the label and a blank line ends the paragraph the link lives in; and a label
/// holding nothing but breaks collapses to empty, because a break needs a line on both sides
/// to mean anything.
///
/// ~keep `marker` is selected from the complete conversion options: Markdown Tier-1 uses the
/// spaces form, while Djot always uses the backslash form regardless of `newline_style`.
///
/// With `keeps_breaks` false -- a heading or a pipe-table cell, neither of which can carry a
/// hard break at all -- every marker in the label is folded to a space instead, which is what
/// Tier-2 does at emission time in `line_break.rs`.
fn trim_label_preserving_boundary_hard_breaks(dest: &mut String, trim_start: usize, keeps_breaks: bool, marker: &str) {
    if !keeps_breaks {
        // ~keep Fold the markers away entirely rather than merely declining to keep the ones at
        // the edges. `close_heading` / `close_table_cell` would collapse them to a space
        // anyway, but they run AFTER `close_link` has escaped the label, so a marker left
        // here reaches `escape_link_label` as a real line break and its continuation line
        // gets a #496 block-opener escape that Tier-2 -- which folds at emission time,
        // in `line_break.rs`'s `in_heading` arm -- never applies. That divergence is
        // visible as `# [A \- B](H)` against Tier-2's `# [A - B](H)`.
        let label = &dest[trim_start..];
        if label.contains(marker) {
            let folded = label.replace(marker, " ");
            dest.truncate(trim_start);
            dest.push_str(&folded);
        }
        // ~keep Trim BOTH ends, not just the trailing one: Tier-2 reaches this label through
        // `normalize_link_label`, which trims both, so a folded leading break that left a
        // space behind shows up as `## [ A](H)` against Tier-2's `## [A](H)`.
        let trimmed = dest[trim_start..].trim_matches(|c: char| c.is_whitespace()).to_owned();
        dest.truncate(trim_start);
        dest.push_str(&trimmed);
        return;
    }

    let label = &dest[trim_start..];
    let without_trailing = label.trim_end_matches(marker);
    let has_trailing_break = without_trailing.len() != label.len();
    let body = without_trailing.trim_end_matches(|c: char| c.is_whitespace());
    let without_leading = body.trim_start_matches(marker);
    let has_leading_break = without_leading.len() != body.len();

    let mut rebuilt = String::with_capacity(without_leading.len() + 2 * marker.len());
    if !without_leading.is_empty() {
        if has_leading_break {
            rebuilt.push_str(marker);
        }
        rebuilt.push_str(without_leading);
        if has_trailing_break {
            rebuilt.push_str(marker);
        }
    }

    if rebuilt.len() != label.len() {
        dest.truncate(trim_start);
        dest.push_str(&rebuilt);
    }

    if let std::borrow::Cow::Owned(protected) =
        crate::converter::utility::content::protect_adjacent_hard_breaks(&dest[trim_start..])
    {
        dest.truncate(trim_start);
        dest.push_str(&protected);
    }
}

fn close_link(state: &mut Tier1State, frame: &OpenTag, options: &ConversionOptions) -> Result<(), BailReason> {
    // ~keep Close the link: `](href "title")` or `](href)`
    // If no href, just emit the text as-is (Tier-2 behaviour: no link markup).
    // Link state was pushed to state.link_stack at open; pop it now.
    let (href, title, has_nested_tag, tier2_if_no_text) = state.link_stack.pop().unwrap_or((None, None, false, false));
    // ~keep Mirrors the branch ORDER of Tier-2's `line_break.rs`, where `in_heading` and
    // `in_table_cell` are both tested ahead of the link arm: a single-line ATX heading
    // and a pipe-table cell cannot carry a hard break at all, so a `<br>` in either has
    // already been folded to a space regardless of the link, and a marker kept at the
    // label's edge here would only survive as a stray trailing space.
    let keeps_boundary_hard_breaks = !state.escape_ctx.contains(EscapeCtx::HEADING) && !state.in_table_cell();
    let dest = state.cell_or_output_mut();
    // ~keep Trim the whitespace inside the link label so `[text  ](url)` collapses to
    // `[text](url)` — matches Tier-2's `normalize_link_label` (kimbrain.html and similar
    // source HTML with whitespace before `</a>`), while keeping a `<br>` that sits at
    // either edge of the label (issue #497).
    let trim_start = clamp_to_char_boundary(dest, frame.content_start);
    // ~keep A link with no address is running text in Tier-2, which keeps the white space that
    // ~keep its content ends with. The trim below loses it, so the page goes to Tier-2 (the
    // ~keep scanner does the same where such a link opens, for the white space at its start).
    if href.is_none() && dest[trim_start..].ends_with(char::is_whitespace) {
        return Err(BailReason::Classifier);
    }
    // ~keep Mirror Tier-2's `normalize_whitespace_cow` step inside
    // `normalize_link_label` (utility/content.rs): any Unicode whitespace
    // in the link label (notably NBSP `\u{00a0}`) collapses to a single ASCII
    // space.  Tier-1 otherwise emits `[Designed\u{a0}by](url)` where Tier-2
    // emits `[Designed by](url)`. It runs before the trim, as it does in Tier-2:
    // a no-break space at the edge of the label is white space of the label.
    normalize_link_label_nbsp(dest, trim_start);
    trim_label_preserving_boundary_hard_breaks(
        dest,
        trim_start,
        keeps_boundary_hard_breaks,
        crate::converter::main_helpers::hard_break_marker(options),
    );
    // ~keep Tier-2 labels a link whose content gives no text with the name of the link, and
    // ~keep leaves such a link out when it points into its own page. This scanner has neither
    // ~keep rule, so it leaves the page to Tier-2.
    if tier2_if_no_text && href.is_some() && dest.len() == trim_start {
        return Err(BailReason::Classifier);
    }
    if let Some(href_str) = href.as_deref() {
        if try_emit_autolink(dest, trim_start, frame, href_str, has_nested_tag, options)? {
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
    if let Some(href) = href {
        emit_markdown_link_close(dest, trim_start, &href, title.as_deref(), options);
        // ~keep A link with no text still writes `[](href)`: content, as in Tier-2's `convert_node`.
        state.end_document_start_if_written(trim_start.saturating_sub(1));
    } else {
        let bracket_search_end = clamp_to_char_boundary(dest, frame.content_start);
        if let Some(bracket_pos) = dest[..bracket_search_end].rfind('[') {
            dest.remove(bracket_pos);
        }
    }
    Ok(())
}

fn normalize_link_label_nbsp(dest: &mut String, trim_start: usize) {
    if !dest[trim_start..].contains('\u{00a0}') {
        return;
    }
    let spaced = dest[trim_start..].replace('\u{00a0}', " ");
    // ~keep A no-break space beside a space is one space of the label, and none at its start,
    // ~keep as in Tier-2. A label with a line break keeps the two spaces of its hard break marker.
    let normalized = if spaced.contains('\n') {
        spaced
    } else {
        crate::text::normalize_whitespace_cow(&spaced).trim_start().to_owned()
    };
    dest.truncate(trim_start);
    dest.push_str(&normalized);
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
