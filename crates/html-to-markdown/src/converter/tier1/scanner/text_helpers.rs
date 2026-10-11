/// Apply `text::backslash_needs_escape` to the run `buffer[from..]`.
///
/// `run_ends_at_last_byte` selects where the run ends, which genuinely differs between
/// the two Tier-2 call sites this mirrors:
///
/// - Prose (`text_node.rs`'s normalized branch) escapes `chomp()`'s *core*, the text
///   node with its boundary whitespace stripped — so a trailing `\` counts as
///   end-of-run even when spaces or a newline follow it in the emitted bytes. Pass
///   `false`.
/// - A table cell (`text_node.rs`'s `in_table_cell` branch) escapes the whole
///   normalized text node with no chomp, so the run ends at the last byte whatever it
///   is. Pass `true`.
///
/// Getting this backwards flips `<p>a\ </p>` or `<td>a\ </td>` by one byte against
/// Tier-2. ~keep
fn escape_backslash_run(buffer: &mut String, from: usize, run_ends_at_last_byte: bool) {
    if memchr::memchr(b'\\', &buffer.as_bytes()[from..]).is_none() {
        return;
    }
    let run = buffer[from..].to_owned();
    let run_end = if run_ends_at_last_byte {
        run.len()
    } else {
        run.trim_end().len()
    };
    let bytes = &run.as_bytes()[..run_end];

    let mut rewritten = String::with_capacity(run.len() + 4);
    let mut copied_to = 0usize;
    for i in memchr::memchr_iter(b'\\', bytes) {
        if crate::text::backslash_needs_escape(bytes, i) {
            rewritten.push_str(&run[copied_to..i]);
            rewritten.push_str(r"\\");
            copied_to = i + 1;
        }
    }
    rewritten.push_str(&run[copied_to..]);

    buffer.truncate(from);
    buffer.push_str(&rewritten);
}

/// Decode HTML entities directly into `out` (no intermediate allocation).
///
/// `base_offset` is the byte offset of `s` within the original HTML input and
/// is used to report the position of any unrecognised entity in the bail reason.
///
/// Uses memchr to quickly find the next `&` and bulk-copies non-entity runs.
///
/// Returns `Err(BailReason::UnknownEntity)` when an entity cannot be decoded.
fn decode_entities_into(
    out: &mut String,
    s: &str,
    base_offset: usize,
    context: ReferenceContext,
) -> Result<(), BailReason> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(pos) = memchr::memchr(b'&', &bytes[i..]) {
            let amp_pos = i + pos;
            if amp_pos > i {
                out.push_str(&s[i..amp_pos]);
            }
            i = decode_entity_at(bytes, s, amp_pos, out, base_offset, context)?;
        } else {
            if i < bytes.len() {
                out.push_str(&s[i..]);
            }
            break;
        }
    }
    Ok(())
}

/// Decode entities AND collapse spaces/tabs in one pass, directly into `out`.
///
/// `base_offset` is the byte offset of `s` within the original HTML input and
/// is used to report the position of any unrecognised entity in the bail reason.
///
/// Uses memchr3 to quickly find the next special byte (space/tab/&), then
/// bulk-copies the run in one `push_str` to avoid per-byte overhead.
///
/// Returns `Err(BailReason::UnknownEntity)` when an entity cannot be decoded.
fn decode_and_collapse_into(
    out: &mut String,
    s: &str,
    has_entities: bool,
    base_offset: usize,
) -> Result<(), BailReason> {
    decode_and_collapse_into_inner(out, s, has_entities, base_offset, false)
}

/// Collapse like `decode_and_collapse_into` but treat `\n`/`\r` as collapsible
/// whitespace too.  Used for text inside `<a>`/`<strong>`/`<em>` frames where
/// Tier-2's `normalize_link_label` first replaces newlines with spaces, then
/// runs whitespace normalization.
fn decode_and_collapse_into_inline(
    out: &mut String,
    s: &str,
    has_entities: bool,
    base_offset: usize,
) -> Result<(), BailReason> {
    decode_and_collapse_into_inner(out, s, has_entities, base_offset, true)
}

fn decode_and_collapse_into_inner(
    out: &mut String,
    s: &str,
    has_entities: bool,
    base_offset: usize,
    collapse_newlines: bool,
) -> Result<(), BailReason> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut prev_was_space = false;
    // ~keep Mirrors Tier-2's `normalize_block_whitespace_cow` (text.rs): when a literal
    // `\n` survives into the output (only possible here when `collapse_newlines`
    // is false -- the `true` variant folds `\n` straight into a space below and
    // never leaves one in `out`), a run of spaces/tabs immediately after it is a
    // Markdown continuation line's leading indentation. A compliant parser drops
    // that entirely on reparse regardless of width (CommonMark spec 4.9), so
    // collapsing it to one space here was not a fixed point -- see the CommonMark
    // spec fixpoint oracle, example 182. Zero is. `s` has already had its own
    // leading/trailing whitespace resolved into a synthetic prefix/suffix by the
    // caller's Phase Y step, so every `\n` this sees with more content after it is
    // a genuine mid-text line break, never the text node's own edge.
    let mut at_line_start = false;
    while i < bytes.len() {
        let next_special = match (has_entities, collapse_newlines) {
            (true, true) => {
                let s_pos = memchr3(b' ', b'\t', b'\n', &bytes[i..]).map(|pos| i + pos);
                let e_pos = memchr::memchr(b'&', &bytes[i..]).map(|pos| i + pos);
                match (s_pos, e_pos) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (Some(a), None) | (None, Some(a)) => Some(a),
                    (None, None) => None,
                }
            }
            (true, false) => memchr3(b' ', b'\t', b'&', &bytes[i..]).map(|pos| i + pos),
            (false, true) => memchr3(b' ', b'\t', b'\n', &bytes[i..]).map(|pos| i + pos),
            (false, false) => memchr::memchr2(b' ', b'\t', &bytes[i..]).map(|pos| i + pos),
        };

        if let Some(pos) = next_special {
            if pos > i {
                out.push_str(&s[i..pos]);
                prev_was_space = false;
                at_line_start = !collapse_newlines && bytes[pos - 1] == b'\n';
            }
            match bytes[pos] {
                b' ' | b'\t' if at_line_start => {
                    i = pos + 1;
                }
                b' ' | b'\t' => {
                    if !prev_was_space {
                        out.push(' ');
                    }
                    prev_was_space = true;
                    i = pos + 1;
                }
                b'\n' if collapse_newlines => {
                    if !prev_was_space {
                        out.push(' ');
                    }
                    prev_was_space = true;
                    i = pos + 1;
                }
                b'&' => {
                    prev_was_space = false;
                    at_line_start = false;
                    i = decode_entity_at(bytes, s, pos, out, base_offset, ReferenceContext::Text)?;
                }
                _ => unreachable!(),
            }
        } else {
            if i < bytes.len() {
                out.push_str(&s[i..]);
            }
            break;
        }
    }
    Ok(())
}

/// `raw` without its leading whitespace, where a character reference that decodes to whitespace
/// (`&#10;`, `&#32;`, `&Tab;`) counts as whitespace too.
///
/// ~keep Tier-2 decodes a text node before it trims the node's leading whitespace, so a newline
/// ~keep written as `&#10;` after a hard break is dropped there; trimmed before decoding, it
/// ~keep survived here as a second line end and made a blank line.
fn trim_leading_text_whitespace(raw: &str) -> &str {
    const WHITESPACE: [char; 4] = [' ', '\t', '\n', '\r'];
    let mut rest = raw.trim_start_matches(WHITESPACE);
    while rest.starts_with('&') {
        match crate::text::decode_character_reference(rest, 0, ReferenceContext::Text) {
            Some((end, first, None)) if rest.as_bytes()[end - 1] == b';' && WHITESPACE.contains(&first) => {
                rest = rest[end..].trim_start_matches(WHITESPACE);
            }
            _ => break,
        }
    }
    rest
}

/// Scan and decode a single HTML entity starting at `amp_pos` (the `&` byte).
///
/// Tries the hot subset in `decode_entity_into` for an alphanumeric name closed by `;`; every
/// other reference goes through Tier-2's decoder, `text::decode_character_reference`, so both
/// tiers read one table. When nothing decodes, writes the `&` alone and resumes at the next byte,
/// as Tier-2 does, so a reference after an unknown name still decodes (#554).
///
/// Returns the position immediately after the reference, or after the `&`.
///
/// Emits `Err(BailReason::UnknownEntity)` when Tier-2's decoder would decode a
/// reference this function does not: one without its `;`.
fn decode_entity_at(
    bytes: &[u8],
    s: &str,
    amp_pos: usize,
    out: &mut String,
    base_offset: usize,
    context: ReferenceContext,
) -> Result<usize, BailReason> {
    let amp = amp_pos;
    let name_len = bytes[amp + 1..]
        .iter()
        .take_while(|b| b.is_ascii_alphanumeric())
        .count();
    let name_end = amp + 1 + name_len;
    if bytes.get(name_end) == Some(&b';') && decode_entity_into(out, &s[amp + 1..name_end]) {
        return Ok(name_end + 1);
    }
    match crate::text::decode_character_reference(s, amp, context) {
        Some((reference_end, first, second)) if bytes[reference_end - 1] == b';' => {
            out.push(first);
            if let Some(second) = second {
                out.push(second);
            }
            Ok(reference_end)
        }
        // ~keep A legacy named reference (#545) or a numeric one (#553) without its `;` is
        // ~keep rare; Tier-2 owns the longest-name rule and the attribute exception.
        Some((reference_end, ..)) => Err(BailReason::UnknownEntity {
            name: s[amp + 1..reference_end].into(),
            offset: base_offset + amp,
        }),
        None => {
            out.push('&');
            Ok(amp + 1)
        }
    }
}

/// Apply the escape-context bits for an opening tag.
///
/// The close path restores `state.escape_ctx` directly from `frame.prev_escape_ctx`
/// so a symmetric `remove_open_escape_ctx` is not needed.
#[inline]
fn apply_open_escape_ctx(state: &mut Tier1State, spec: &TagSpec) {
    if spec.kind == TagKind::Pre {
        state.escape_ctx |= EscapeCtx::PRE | EscapeCtx::CODE;
        return;
    }

    let bit = match spec.kind {
        TagKind::Code => EscapeCtx::CODE,
        TagKind::Link => EscapeCtx::LINK,
        TagKind::Blockquote => EscapeCtx::BLOCKQUOTE,
        TagKind::Heading(_) => EscapeCtx::HEADING,
        TagKind::Strong => EscapeCtx::STRONG,
        _ => return,
    };

    state.escape_ctx |= bit;
}

/// Report whether an attribute is present, with or without a value.
///
/// [`find_attr`] returns the attribute's *value* and so cannot tell an absent
/// attribute from a valueless one (`<td colspan>`).  Tier-2's spanning-cell test
/// (`block/table/scanner.rs`: `attrs.get("colspan").is_some()`) keys off presence
/// alone, so mirroring it needs this distinction.
fn has_attr(attrs: &[(&[u8], Option<&[u8]>)], key: &[u8]) -> bool {
    attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case(key))
}

/// Find an attribute value by (lowercase) key name.
fn find_attr<'a>(attrs: &[(&'a [u8], Option<&'a [u8]>)], key: &[u8]) -> Option<&'a [u8]> {
    for (k, v) in attrs {
        if k.eq_ignore_ascii_case(key) {
            return *v;
        }
    }
    None
}

/// Returns true when `name_lower` is a tag that *may* need preprocessing-skip
/// evaluation.  All other tags skip the more expensive `should_skip_preprocessing`
/// check entirely.
fn is_preprocessing_skip_candidate(name_lower: &[u8]) -> bool {
    matches!(name_lower, b"nav" | b"header" | b"footer" | b"aside" | b"form")
}

/// Mirrors `should_drop_for_preprocessing` (preprocessing_helpers.rs) for
/// the Tier-1 byte scanner.
///
/// Called only for tags that passed [`is_preprocessing_skip_candidate`].
/// Uses the raw attribute byte slices collected by [`parse::collect_attrs`]
/// instead of the Tier-2 `tl::HTMLTag` DOM node.
fn should_skip_preprocessing(
    name_lower: &[u8],
    attrs: &[(&[u8], Option<&[u8]>)],
    options: &ConversionOptions,
    is_page_header: bool,
) -> bool {
    use crate::options::PreprocessingPreset;

    if !options.preprocessing.enabled {
        return false;
    }

    if options.preprocessing.preset == PreprocessingPreset::Minimal {
        return false;
    }

    if options.preprocessing.remove_forms && name_lower == b"form" {
        return true;
    }

    if !options.preprocessing.remove_navigation {
        return false;
    }

    if name_lower == b"nav" {
        return true;
    }

    // ~keep <header> / <footer> / <aside> — drop only when navigation hints present.
    // (Aggressive would drop footer/aside unconditionally, but Aggressive routes
    // through Tier-2 via the existing router gate so Tier-1 only needs the
    // Standard-preset behaviour: nav-hint check.)
    if name_lower == b"header" && is_page_header {
        return true;
    }

    if matches!(name_lower, b"header" | b"footer" | b"aside") {
        return byte_attrs_have_navigation_hint(attrs);
    }

    false
}

fn header_is_page_level(state: &Tier1State, html: &str) -> bool {
    let mut in_document_body = false;
    for frame in &state.stack {
        let Some(name) = html.as_bytes().get(frame.name_range.clone()) else {
            continue;
        };
        if matches_ignore_ascii_case(name, &[b"article", b"section", b"main"]) {
            return false;
        }
        in_document_body |= name.eq_ignore_ascii_case(b"body");
    }
    in_document_body
}

fn matches_ignore_ascii_case(value: &[u8], candidates: &[&[u8]]) -> bool {
    candidates.iter().any(|candidate| value.eq_ignore_ascii_case(candidate))
}

/// Byte-level equivalent of `element_has_navigation_hint` for use in the
/// Tier-1 scanner where attributes are raw `&[u8]` slices rather than a
/// parsed `tl::HTMLTag`.
fn byte_attrs_have_navigation_hint(attrs: &[(&[u8], Option<&[u8]>)]) -> bool {
    if let Some(role) = find_attr(attrs, b"role") {
        let role_lc = role.to_ascii_lowercase();
        if matches!(role_lc.as_slice(), b"navigation" | b"menubar" | b"tablist" | b"toolbar") {
            return true;
        }
    }

    if let Some(label) = find_attr(attrs, b"aria-label") {
        let label_lc = label.to_ascii_lowercase();
        const ARIA_SUBSTRINGS: &[&[u8]] = &[b"navigation", b"menu", b"contents", b"table of contents", b"toc"];
        if ARIA_SUBSTRINGS
            .iter()
            .any(|sub| label_lc.windows(sub.len()).any(|w| w == *sub))
        {
            return true;
        }
    }

    for attr_name in [b"class".as_slice(), b"id".as_slice()] {
        if let Some(value) = find_attr(attrs, attr_name) {
            if byte_value_has_nav_keyword(value) {
                return true;
            }
        }
    }

    false
}

/// Tokenize a raw attribute byte value and return true when any token matches
/// a keyword in [`NAV_KEYWORDS`].
///
/// Tokens are split on ASCII whitespace.  Each token is normalised by
/// replacing `_`, `:`, `.`, `/` with `-` and lowercasing before comparison.
fn byte_value_has_nav_keyword(value: &[u8]) -> bool {
    let mut start = 0;
    let len = value.len();
    loop {
        while start < len && value[start].is_ascii_whitespace() {
            start += 1;
        }
        if start >= len {
            break;
        }
        let mut end = start;
        while end < len && !value[end].is_ascii_whitespace() {
            end += 1;
        }
        let token_bytes = &value[start..end];
        let mut buf = [0u8; 64];
        let normalised: &[u8] = if token_bytes.len() <= buf.len() {
            let n = token_bytes.len();
            for (i, &b) in token_bytes.iter().enumerate() {
                buf[i] = match b {
                    b'_' | b':' | b'.' | b'/' => b'-',
                    _ => b.to_ascii_lowercase(),
                };
            }
            &buf[..n]
        } else {
            start = end;
            continue;
        };

        if NAV_KEYWORDS.iter().any(|kw| kw.as_bytes() == normalised) {
            return true;
        }

        start = end;
    }
    false
}

/// Extract `href` and `title` from the attribute list for a link.
fn extract_link_attrs(attrs: &[(&[u8], Option<&[u8]>)]) -> Result<(Option<String>, Option<String>), BailReason> {
    let href = find_attr(attrs, b"href").map(decode_attr).transpose()?;
    // ~keep Full decode, matching Tier-2's `decoded_attribute` (issue #494). This used to be
    // a deliberate half-decode that mirrored Tier-2 leaving named entities intact, so a
    // `title="A&amp;B"` reached the output as the literal text `A&amp;B` in both tiers.
    let title = find_attr(attrs, b"title").map(decode_attr).transpose()?;
    Ok((href, title))
}

/// Extract `start` attribute from `<ol>` (defaults to 1).
fn extract_ol_start(attrs: &[(&[u8], Option<&[u8]>)]) -> u16 {
    find_attr(attrs, b"start")
        .and_then(|b| std::str::from_utf8(b).ok())
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(1)
}

/// Decode an attribute value: entity-decode and convert to a String.
///
/// Returns `Err(BailReason::Classifier)` when the value is not valid UTF-8
/// (malformed bytes in attributes cannot be decoded faithfully).
/// Returns `Err(BailReason::UnknownEntity)` when the value contains an entity
/// that Tier-1 cannot decode (Tier-2 would decode it differently).
fn decode_attr(bytes: &[u8]) -> Result<String, BailReason> {
    let s = std::str::from_utf8(bytes).map_err(|_| BailReason::Classifier)?;
    if !s.contains('&') {
        return Ok(s.to_owned());
    }
    let mut out = String::with_capacity(s.len());
    decode_entities_into(&mut out, s, 0, ReferenceContext::Attribute)?;
    Ok(out)
}

/// Pop the topmost frame whose spec matches `spec`.
/// Tier-2 is lenient about close tags; we are strict in M3c: only pop the
/// Pop the topmost frame whose spec matches `spec`.
///
/// We compare by checking if the `TagKind` on the top frame maps to the same
/// "semantic group" as the spec being closed.  We are strict in M3c: only the
/// top frame is checked to avoid mismatched-close-tag complexity.
fn pop_matching_frame(stack: &mut Vec<OpenTag>, spec: &'static TagSpec) -> Option<OpenTag> {
    let top = stack.last()?;
    if kinds_match(&top.spec.kind, &spec.kind) {
        stack.pop()
    } else {
        None
    }
}

/// Return `true` if two `TagKind` values are the "same" for close-tag matching.
///
/// Uses pointer equality on the `&'static TagSpec` where possible for speed.
/// For kinds with inner data (`List`, `Heading`, `TableCell`) we use a
/// coarser match that still prevents cross-kind confusion:
/// - `List(Ordered)` only matches `List(Ordered)`, etc.
/// - `Heading(n)` matches `Heading(m)` for any n, m (HTML allows `</h3>` to
///   close `<h2>` in some parsers; we are lenient for headings since they
///   do not nest in practice).
fn kinds_match(a: &TagKind, b: &TagKind) -> bool {
    match (a, b) {
        (TagKind::List(la), TagKind::List(lb)) => la == lb,
        (TagKind::Heading(_), TagKind::Heading(_)) => true,
        (TagKind::TableCell { is_header: a_h }, TagKind::TableCell { is_header: b_h }) => a_h == b_h,
        _ => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}

/// Find the nearest enclosing list kind by walking the stack top-to-bottom.
fn find_parent_list_kind(stack: &[OpenTag]) -> Option<ListKind> {
    for frame in stack.iter().rev() {
        if let TagKind::List(kind) = frame.spec.kind {
            return Some(kind);
        }
    }
    None
}

/// Increment the ordered-list counter on the nearest `List(Ordered)` frame.
/// Returns the new counter value (1-based).
fn increment_ol_counter(stack: &mut [OpenTag]) -> u16 {
    for frame in stack.iter_mut().rev() {
        if frame.spec.kind == TagKind::List(ListKind::Ordered) {
            frame.list_index = frame.list_index.saturating_add(1);
            return frame.list_index;
        }
    }
    1
}

/// Get the `ol_start` value from the nearest `List(Ordered)` frame.
fn find_ol_start(stack: &[OpenTag]) -> u16 {
    for frame in stack.iter().rev() {
        if frame.spec.kind == TagKind::List(ListKind::Ordered) {
            return frame.ol_start;
        }
    }
    1
}

/// Return the ATX heading prefix for level `n` (1–6).
///
/// Uses the `HEADING_PREFIXES` table — no allocation.
fn heading_prefix(n: u8) -> &'static str {
    let idx = (n as usize).saturating_sub(1).min(5);
    HEADING_PREFIXES[idx]
}

/// Push the list-item indentation for `depth` into `out`.
///
/// Depth 0 → no indent; each level adds two spaces (matches the router's
/// `list_indent_width == 2` gate).  Depths 0–7 use the static `LIST_ITEM_INDENTS`
/// table; deeper nesting (rare) falls back to a runtime loop.
fn push_list_item_indent(out: &mut String, depth: u16) {
    let idx = depth as usize;
    if idx < LIST_ITEM_INDENTS.len() {
        out.push_str(LIST_ITEM_INDENTS[idx]);
    } else {
        out.reserve(idx * 2);
        for _ in 0..idx {
            out.push_str("  ");
        }
    }
}

/// Add `> ` prefix to every non-empty line of `content`, and `>` to empty
/// lines that are between non-empty ones (Tier-2 behaviour for multi-paragraph
/// blockquotes).
fn prefix_blockquote_lines(content: &str) -> String {
    let content = content.trim_end_matches('\n');
    if content.is_empty() {
        return String::new();
    }

    let lines: Vec<&str> = content.split('\n').collect();
    let mut result = String::with_capacity(content.len() + lines.len() * 2);

    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            result.push('>');
        } else {
            result.push_str("> ");
            result.push_str(line);
        }
        if i < lines.len() - 1 {
            result.push('\n');
        }
    }
    result.push('\n');
    result
}

// ~keep ── GFM table emission ────────────────────────────────────────────────────────
