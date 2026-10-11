/// Emit a completed table as GFM markdown, appending to `state.output`.
///
/// Format must match Tier-2 `convert_table_row` byte-for-byte:
/// - Each row: `|` + ` cell |` per cell → `| c1 | c2 |`
/// - After row 0: separator `| --- | --- |` (one `---` per column)
///
/// # Panics
///
/// Never — empty-table guard returns early.
fn emit_gfm_table(
    target: &mut String,
    ts: crate::converter::tier1::state::TableState,
    output_format: crate::options::OutputFormat,
) {
    // ~keep Emit caption (if any) BEFORE the table body.
    // ~keep
    // ~keep Mirrors Tier-2 builder.rs caption handling: `*escaped_text*\n\n`.
    // Tier-2 emits the caption as part of the table child loop, which runs
    // before the rows are rendered, so the caption appears even when there
    // are no table rows.  The caption text has already been trimmed and
    // hyphen-escaped when `</caption>` was processed.
    emit_table_caption(target, ts.caption_text.as_deref());

    if ts.rows.is_empty() {
        return;
    }

    // ~keep Pre-table separator: mirrors Tier-2's `convert_table` logic exactly.
    // Tier-2 (block/table/mod.rs): `if !output.is_empty() && !output.ends_with("\n\n")`
    // — only adds separator when there is existing output (no leading blank lines).
    ensure_table_separator(target);

    // ~keep Pre-compute max column widths across ALL rows (mirrors Tier-2's pre-pass).
    // Tier-2: separator dashes = max(col_content_char_count_across_all_rows, 3).
    // col_count is the colspan-expanded column count (sum of colspans per row).
    let col_widths = table_column_widths(&ts.rows);
    let col_count = col_widths.len();

    for (row_index, row) in ts.rows.iter().enumerate() {
        emit_table_row(target, row, &col_widths);

        // ~keep After row 0 (the header row), emit the separator row.
        // Tier-2: col_widths.get(i).unwrap_or(0).max(MIN_SEPARATOR_DASHES).
        if row_index == 0 {
            emit_table_header_separator(target, &col_widths, col_count, output_format);
        }
    }
}

fn emit_table_caption(target: &mut String, caption: Option<&str>) {
    let Some(caption) = caption.filter(|caption| !caption.is_empty()) else {
        return;
    };
    ensure_table_separator(target);
    target.push('*');
    target.push_str(caption);
    target.push_str("*\n\n");
}

fn ensure_table_separator(target: &mut String) {
    if target.is_empty() || target.ends_with("\n\n") {
        return;
    }
    if target.ends_with('\n') {
        target.push('\n');
    } else {
        target.push_str("\n\n");
    }
}

fn table_column_widths(rows: &[Vec<(String, u16)>]) -> Vec<usize> {
    let column_count = rows
        .iter()
        .map(|row| row.iter().map(|(_, span)| usize::from(*span)).sum::<usize>())
        .max()
        .unwrap_or(0);
    let mut widths = vec![0; column_count];
    for row in rows {
        let mut column = 0;
        for (cell, span) in row {
            if let Some(width) = widths.get_mut(column) {
                *width = (*width).max(cell.chars().count());
            }
            column += usize::from(*span);
        }
    }
    widths
}

fn emit_table_row(target: &mut String, row: &[(String, u16)], widths: &[usize]) {
    target.push('|');
    let mut column = 0;
    for (cell, span) in row {
        target.push(' ');
        target.push_str(cell);
        for _ in cell.chars().count()..widths.get(column).copied().unwrap_or(0) {
            target.push(' ');
        }
        for _ in 0..*span {
            target.push_str(" |");
        }
        column += usize::from(*span);
    }
    target.push('\n');
}

fn emit_table_header_separator(
    target: &mut String,
    widths: &[usize],
    column_count: usize,
    output_format: crate::options::OutputFormat,
) {
    let is_djot = output_format == crate::options::OutputFormat::Djot;
    target.push('|');
    if !is_djot {
        target.push(' ');
    }
    for column in 0..column_count.max(1) {
        if column > 0 {
            target.push_str(if is_djot { "|" } else { " | " });
        }
        let dash_count = widths.get(column).copied().unwrap_or(0).max(MIN_SEPARATOR_DASHES);
        target.extend(std::iter::repeat_n('-', dash_count));
    }
    if !is_djot {
        target.push(' ');
    }
    target.push_str("|\n");
}

/// Trim trailing spaces and tabs from the end of the output (used before
/// closing block elements that trim trailing whitespace in Tier-2).
fn trim_trailing_inline_whitespace(state: &mut Tier1State) {
    let buf = state.cell_or_output_mut();
    while buf.ends_with(' ') || buf.ends_with('\t') {
        buf.pop();
    }
}

/// Decode a single HTML entity name (without `&` or `;`) from Tier-1's hot subset
/// directly into `out`.
///
/// Returns `true` when the entity was recognized and written; `false` for any
/// other name or a numeric reference, which the caller hands to Tier-2's decoder.
/// No `String` is allocated.
fn decode_entity_into(out: &mut String, name: &str) -> bool {
    let Some(entity) = basic_entity(name).or_else(|| latin1_entity(name)) else {
        return false;
    };
    out.push_str(entity);
    true
}

fn basic_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => "\u{00A0}",
        "copy" => "\u{00A9}",
        "reg" => "\u{00AE}",
        "trade" => "\u{2122}",
        "mdash" => "\u{2014}",
        "ndash" => "\u{2013}",
        "hellip" => "\u{2026}",
        "laquo" => "\u{00AB}",
        "raquo" => "\u{00BB}",
        "lsquo" => "\u{2018}",
        "rsquo" => "\u{2019}",
        "ldquo" => "\u{201C}",
        "rdquo" => "\u{201D}",
        "prime" => "\u{2032}",
        "Prime" => "\u{2033}",
        "bull" => "\u{2022}",
        "middot" => "\u{00B7}",
        "deg" => "\u{00B0}",
        "plusmn" => "\u{00B1}",
        "times" => "\u{00D7}",
        "divide" => "\u{00F7}",
        "frac12" => "\u{00BD}",
        "frac14" => "\u{00BC}",
        "frac34" => "\u{00BE}",
        "euro" => "\u{20AC}",
        "pound" => "\u{00A3}",
        "yen" => "\u{00A5}",
        "cent" => "\u{00A2}",
        "larr" => "\u{2190}",
        "rarr" => "\u{2192}",
        "uarr" => "\u{2191}",
        "darr" => "\u{2193}",
        "harr" => "\u{2194}",
        "infin" => "\u{221E}",
        "alpha" => "\u{03B1}",
        "beta" => "\u{03B2}",
        "gamma" => "\u{03B3}",
        "delta" => "\u{03B4}",
        "pi" => "\u{03C0}",
        "sigma" => "\u{03C3}",
        "omega" => "\u{03C9}",
        _ => return None,
    })
}

fn latin1_entity(name: &str) -> Option<&'static str> {
    latin1_upper_entity(name).or_else(|| latin1_lower_entity(name))
}

fn latin1_upper_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "iexcl" => "\u{00A1}",
        "brvbar" => "\u{00A6}",
        "sect" => "\u{00A7}",
        "uml" => "\u{00A8}",
        "ordf" => "\u{00AA}",
        "not" => "\u{00AC}",
        "shy" => "\u{00AD}",
        "macr" => "\u{00AF}",
        "sup2" => "\u{00B2}",
        "sup3" => "\u{00B3}",
        "acute" => "\u{00B4}",
        "micro" => "\u{00B5}",
        "para" => "\u{00B6}",
        "cedil" => "\u{00B8}",
        "sup1" => "\u{00B9}",
        "ordm" => "º",
        "iquest" => "\u{00BF}",
        "Agrave" => "\u{00C0}",
        "Aacute" => "\u{00C1}",
        "Acirc" => "\u{00C2}",
        "Atilde" => "\u{00C3}",
        "Auml" => "\u{00C4}",
        "Aring" => "\u{00C5}",
        "AElig" => "\u{00C6}",
        "Ccedil" => "\u{00C7}",
        "Egrave" => "\u{00C8}",
        "Eacute" => "\u{00C9}",
        "Ecirc" => "\u{00CA}",
        "Euml" => "\u{00CB}",
        "Igrave" => "\u{00CC}",
        "Iacute" => "\u{00CD}",
        "Icirc" => "\u{00CE}",
        "Iuml" => "\u{00CF}",
        "ETH" => "\u{00D0}",
        "Ntilde" => "\u{00D1}",
        "Ograve" => "\u{00D2}",
        "Oacute" => "\u{00D3}",
        "Ocirc" => "\u{00D4}",
        "Otilde" => "\u{00D5}",
        "Ouml" => "\u{00D6}",
        "Oslash" => "\u{00D8}",
        "Ugrave" => "\u{00D9}",
        "Uacute" => "\u{00DA}",
        "Ucirc" => "\u{00DB}",
        "Uuml" => "\u{00DC}",
        "Yacute" => "\u{00DD}",
        "THORN" => "\u{00DE}",
        "szlig" => "\u{00DF}",
        _ => return None,
    })
}

fn latin1_lower_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "agrave" => "\u{00E0}",
        "aacute" => "\u{00E1}",
        "acirc" => "\u{00E2}",
        "atilde" => "\u{00E3}",
        "auml" => "\u{00E4}",
        "aring" => "\u{00E5}",
        "aelig" => "\u{00E6}",
        "ccedil" => "\u{00E7}",
        "egrave" => "\u{00E8}",
        "eacute" => "\u{00E9}",
        "ecirc" => "\u{00EA}",
        "euml" => "\u{00EB}",
        "igrave" => "\u{00EC}",
        "iacute" => "\u{00ED}",
        "icirc" => "\u{00EE}",
        "iuml" => "\u{00EF}",
        "eth" => "\u{00F0}",
        "ntilde" => "\u{00F1}",
        "ograve" => "\u{00F2}",
        "oacute" => "\u{00F3}",
        "ocirc" => "\u{00F4}",
        "otilde" => "\u{00F5}",
        "ouml" => "\u{00F6}",
        "oslash" => "\u{00F8}",
        "ugrave" => "\u{00F9}",
        "uacute" => "\u{00FA}",
        "ucirc" => "\u{00FB}",
        "uuml" => "\u{00FC}",
        "yacute" => "\u{00FD}",
        "thorn" => "\u{00FE}",
        "yuml" => "\u{00FF}",
        _ => return None,
    })
}

/// Skip `<!--...-->`, `<!DOCTYPE...>`, or any `<!...>` construct.
/// Returns the position immediately after the closing `>`.
///
/// On failure returns `Err(BailReason::LiteralLt)`.
fn skip_bang(bytes: &[u8], pos: usize) -> Result<usize, BailReason> {
    let start = pos + 2;

    if bytes.get(start) == Some(&b'-') && bytes.get(start + 1) == Some(&b'-') {
        let comment_start = start + 2;
        let mut i = comment_start;
        while i + 2 < bytes.len() {
            if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>' {
                return Ok(i + 3);
            }
            i += 1;
        }
        // ~keep Unclosed comment — bail
        return Err(BailReason::LiteralLt { offset: pos });
    }

    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'>' {
            return Ok(i + 1);
        }
        i += 1;
    }
    Err(BailReason::LiteralLt { offset: pos })
}

/// Convert tag name bytes to lowercase in a fixed-size stack buffer.
/// Returns a slice into `buf`.  If the name is longer than `buf`, it is
/// truncated (names > `MAX_TAG_NAME_BYTES` won't appear in the spec table and
/// will be rejected as unknown).
fn lowercase_into<'b>(bytes: &[u8], buf: &'b mut [u8; MAX_TAG_NAME_BYTES]) -> &'b [u8] {
    let len = bytes.len().min(MAX_TAG_NAME_BYTES);
    for (i, &b) in bytes[..len].iter().enumerate() {
        buf[i] = b.to_ascii_lowercase();
    }
    &buf[..len]
}

/// Convert a byte slice to an owned `String` (lossy UTF-8).
fn bytes_to_string(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Peek the lowercased tag name of the upcoming OPEN tag at `bytes[lt_pos]`
/// (expected to be `<`), if there is one — `None` for a close tag (`</...`),
/// a non-tag `<` (comment, bang, literal), or EOF. Shared by
/// `upcoming_tag_is_list_open` and `upcoming_tag_is_named` so the main scan
/// loop can peek ahead, BEFORE the tag itself is parsed, to tell `flush_text`
/// what kind of tag the text about to be flushed sits directly in front of.
fn upcoming_open_tag_name<'b>(bytes: &[u8], lt_pos: usize, buf: &'b mut [u8; MAX_TAG_NAME_BYTES]) -> Option<&'b [u8]> {
    if bytes.get(lt_pos) != Some(&b'<') {
        return None;
    }
    let &next = bytes.get(lt_pos + 1)?;
    if !parse::is_tag_name_start(next) {
        return None;
    }
    let name_start = lt_pos + 1;
    let name_end = parse::scan_tag_name(bytes, name_start);
    Some(lowercase_into(&bytes[name_start..name_end], buf))
}

/// Decide what a text node's trailing *bare* `\n` (no accompanying space/tab,
/// not part of a `\n\n` run) collapses to.
///
/// Mirrors Tier-2's `has_trailing_single_newline` follow-up step
/// (`text_node.rs`, the `else if has_trailing_single_newline` arm): `chomp()`
/// itself reduces that trailing run to nothing, but a supplementary check then
/// puts a joining character back unless the block already ends in a blank
/// line. Only reachable from the non-inline, non-table-cell Phase Y branch in
/// `flush_text`, so `state.output` — not `cell_or_output_mut()` — is always
/// the right buffer to inspect here (mirrors Tier-2's `ctx.block_content_start`
/// slice of the real `output`, which the same non-inline/non-cell precondition
/// guarantees is `state.output` too).
///
/// - Before an inline element the line end is white space between two words (issue #778):
///   a space, in every container (`line_end_before_element`).
/// - A blank-line break already in place needs nothing. The
///   "already" is scoped to the enclosing `<p>`/`<div>`'s OWN content (Tier-2's
///   `ctx.block_content_start`, i.e. `nearest_block_content_start` here) —
///   never the whole document buffer. A paragraph that just opened right
///   after a preceding one leaves the DOCUMENT ending in "\n\n" (its own
///   leading separator) while its OWN content is still empty; scoping the
///   check avoids reading that separator as "this text node already touches a
///   blank line" and wrongly swallowing the join.
/// - Otherwise: a paragraph ancestor, or a `<strong>`/`<em>` (Tier-2's
///   `inline_depth`-incrementing wrappers) ancestor, joins with a single
///   space; anything else (e.g. a bare `<div>`) joins with a literal newline,
///   which a `<br>` that follows removes again (`Tier1State::pending_newline_join`).
fn trailing_single_newline_join(state: &Tier1State, next: NextElement) -> &'static str {
    let block_start = clamp_to_char_boundary(&state.output, nearest_block_content_start(state));
    if state.output[block_start..].ends_with("\n\n") {
        return "";
    }
    let in_paragraph_or_inline_wrapper = state.stack.iter().any(|frame| {
        matches!(
            frame.spec.kind,
            TagKind::Paragraph | TagKind::Strong | TagKind::Emphasis
        )
    });
    // ~keep The one decision of both converters: before an inline element the line end is a space.
    match crate::converter::utility::content::line_end_before_element(in_paragraph_or_inline_wrapper, next) {
        ' ' => " ",
        _ => "\n",
    }
}

/// Whether the content of the inline element that opens at `bytes[lt_pos]`, or after the
/// comments there, starts with a block. White space and comments are no content, and the scan
/// looks into the elements that only wrap text (`<a><b><div>`).
///
/// ~keep Mirrors Tier-2's `content_starts_with_block`, which asks the same of the tree.
fn upcoming_inline_starts_with_block(bytes: &[u8], lt_pos: usize) -> bool {
    use crate::converter::utility::content::is_text_wrapper;

    let mut pos = lt_pos;
    let mut inside_marks = false;
    loop {
        if bytes.get(pos..).is_some_and(|rest| rest.starts_with(b"<!--")) {
            let Ok(after) = skip_bang(bytes, pos) else {
                return false;
            };
            pos = parse::skip_ws(bytes, after);
            continue;
        }
        let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
        let Some(name) = upcoming_open_tag_name(bytes, pos, &mut name_buf) else {
            return false;
        };
        if is_block_tag(name) {
            return inside_marks;
        }
        if !std::str::from_utf8(name).is_ok_and(is_text_wrapper) {
            return false;
        }
        let Some((close, self_closing)) = parse::find_tag_close(bytes, pos + 1 + name.len()) else {
            return false;
        };
        if self_closing {
            return false;
        }
        // ~keep A `<span>` writes no marks: a block that only spans wrap breaks the line itself.
        inside_marks |= name != b"span";
        pos = parse::skip_ws(bytes, close + 1);
    }
}

/// Whether an inline element opens after the comments at `bytes[lt_pos]`, with only white
/// space between them.
///
/// ~keep Tier-2 asks its tree for the next element (`DomContext::next_tag_id`), and the tree
/// ~keep passes comments and white space. A `<br>` is inline here as it is without a comment
/// ~keep (`upcoming_tag_is_inline`): the line break removes the white space before it.
fn inline_follows_comments(bytes: &[u8], lt_pos: usize) -> bool {
    let mut pos = lt_pos;
    while bytes.get(pos..).is_some_and(|rest| rest.starts_with(b"<!--")) {
        let Ok(after) = skip_bang(bytes, pos) else {
            return false;
        };
        pos = parse::skip_ws(bytes, after);
    }
    let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
    pos != lt_pos && upcoming_open_tag_name(bytes, pos, &mut name_buf).is_some_and(is_inline_tag)
}

/// Whether the text that follows the markup at `bytes[lt_pos]` starts with a zero-width space.
/// Inside an open element whose `style` attribute sets `display` or `white-space` the answer
/// is no.
///
/// ~keep Mirrors Tier-2's `zero_width_space_follows`, which asks the same of the tree.
fn zero_width_space_is_upcoming(html: &str, lt_pos: usize, stack: &[OpenTag]) -> bool {
    use crate::converter::utility::content::ZERO_WIDTH_SPACE;

    upcoming_text_starts_with(html, lt_pos, stack, |text| text.starts_with(ZERO_WIDTH_SPACE))
        && !open_element_sets_display_or_white_space(html.as_bytes(), stack)
}

/// Whether the text that follows the markup at `bytes[lt_pos]` starts with white space of any
/// kind, a no-break space included.
///
/// ~keep The content of a link with no address starts with white space when its first text
/// ~keep does, through a comment, an empty element or the open tag of an inline child:
/// ~keep `<a name="n"><i> Go</i></a>` and `<a name="n">&#32;Go</a>` as `<a name="n"> Go</a>`.
fn white_space_is_upcoming(html: &str, lt_pos: usize, stack: &[OpenTag]) -> bool {
    upcoming_text_starts_with(html, lt_pos, stack, |text| text.starts_with(char::is_whitespace))
}

/// Whether `starts_with` holds for the decoded text that follows the markup at
/// `bytes[lt_pos]`. The scan looks into the elements that only wrap text, leaves such an open
/// element of `stack` at its end tag and passes the empty ones, comments and `<wbr>`. Any
/// other element, an element whose `style` attribute sets `display` or `white-space` and the
/// end of any other parent end the scan with no.
fn upcoming_text_starts_with(html: &str, lt_pos: usize, stack: &[OpenTag], starts_with: impl Fn(&str) -> bool) -> bool {
    let bytes = html.as_bytes();
    let mut pos = lt_pos;
    let mut open_wrappers = 0usize;
    let mut enclosing = stack.iter().rev();
    while pos < bytes.len() {
        if bytes[pos] != b'<' {
            // ~keep The first character decides, and no character reference is longer than this.
            let window_end = clamp_to_char_boundary(html, (pos + 40).min(bytes.len()));
            let text_end = memchr::memchr(b'<', &bytes[pos..window_end]).map_or(window_end, |offset| pos + offset);
            let text = crate::text::decode_html_entities_cow(&html[pos..text_end]);
            return starts_with(&text);
        }
        match bytes.get(pos + 1) {
            Some(b'!') => match skip_bang(bytes, pos) {
                Ok(after) => pos = after,
                Err(_) => return false,
            },
            Some(b'/') => {
                if open_wrappers > 0 {
                    open_wrappers -= 1;
                } else if !enclosing
                    .next()
                    .is_some_and(|frame| closes_plain_text_wrapper(bytes, pos, frame))
                {
                    return false;
                }
                let Some((close, _)) = parse::find_tag_close(bytes, pos + 2) else {
                    return false;
                };
                pos = close + 1;
            }
            _ => {
                let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
                let Some(name) = upcoming_open_tag_name(bytes, pos, &mut name_buf) else {
                    return false;
                };
                let is_wbr = name == b"wbr";
                let Some((close, self_closing)) = plain_text_wrapper_open_end(bytes, name, pos + 1 + name.len()) else {
                    return false;
                };
                if !is_wbr && !self_closing {
                    open_wrappers += 1;
                }
                pos = close + 1;
            }
        }
    }
    false
}

/// Where the open tag of an element that only wraps text (or of a `<wbr>`) ends, and whether
/// it closes itself. `name` is the lowercase name of the tag and its attributes start at
/// `attributes_start`. `None` for any other element and for one whose `style` attribute sets
/// `display` or `white-space`.
fn plain_text_wrapper_open_end(bytes: &[u8], name: &[u8], attributes_start: usize) -> Option<(usize, bool)> {
    use crate::converter::utility::content::is_text_wrapper;

    if name != b"wbr" && !std::str::from_utf8(name).is_ok_and(is_text_wrapper) {
        return None;
    }
    let (close, self_closing) = parse::find_tag_close(bytes, attributes_start)?;
    (!open_tag_sets_display_or_white_space(bytes, attributes_start, close)).then_some((close, self_closing))
}

/// Whether the open tag whose attributes are `bytes[attributes_start..close]` has a `style`
/// attribute that sets `display` or `white-space`.
fn open_tag_sets_display_or_white_space(bytes: &[u8], attributes_start: usize, close: usize) -> bool {
    use crate::converter::utility::attributes::style_attribute_sets_display_or_white_space;

    // ~keep An attribute name has no letter case in HTML, and Tier-2's parser gives it in lower case.
    // ~keep A browser reads the first of two `style` attributes, so the first decides.
    parse::collect_attrs(bytes, attributes_start, close)
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(b"style"))
        .and_then(|(_, value)| *value)
        .is_some_and(style_attribute_sets_display_or_white_space)
}

/// Whether an open element of `stack` has a `style` attribute that sets `display` or
/// `white-space`.
///
/// ~keep Such an element can keep its line ends (`white-space: pre`), and its content inherits
/// ~keep that. Mirrors Tier-2's `is_inside_display_or_white_space_style`.
fn open_element_sets_display_or_white_space(bytes: &[u8], stack: &[OpenTag]) -> bool {
    stack.iter().any(|frame| {
        parse::find_tag_close(bytes, frame.name_range.end)
            .is_some_and(|(close, _)| open_tag_sets_display_or_white_space(bytes, frame.name_range.end, close))
    })
}

/// Whether the end tag at `bytes[lt_pos]` closes `frame`, and `frame` is an element that only
/// wraps text and has no `style` attribute that sets `display` or `white-space`.
fn closes_plain_text_wrapper(bytes: &[u8], lt_pos: usize, frame: &OpenTag) -> bool {
    let name_start = lt_pos + 2;
    let closed = &bytes[name_start..parse::scan_tag_name(bytes, name_start)];
    let Some(opened) = bytes.get(frame.name_range.clone()) else {
        return false;
    };
    let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
    let name = lowercase_into(opened, &mut name_buf);
    closed.eq_ignore_ascii_case(opened) && plain_text_wrapper_open_end(bytes, name, frame.name_range.end).is_some()
}

fn contains_blank_line(bytes: &[u8]) -> bool {
    let mut line_breaks = 0;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                line_breaks += 1;
                if bytes.get(index + 1) == Some(&b'\n') {
                    index += 1;
                }
            }
            b'\n' => line_breaks += 1,
            _ => {}
        }
        if line_breaks >= 2 {
            return true;
        }
        index += 1;
    }
    false
}

/// Position where the innermost enclosing `<p>`/`<div>` frame's OWN content
/// starts in `state.output` — Tier-1's equivalent of Tier-2's
/// `ctx.block_content_start` (set in `block/paragraph.rs`, which handles both
/// tags). Falls back to `0` (the whole buffer) when no such ancestor is open,
/// matching `Context::default`'s `block_content_start: 0`.
fn nearest_block_content_start(state: &Tier1State) -> usize {
    state
        .stack
        .iter()
        .rev()
        .find(|frame| matches!(frame.spec.kind, TagKind::Paragraph | TagKind::Block))
        .map_or(0, |frame| frame.content_start)
}

/// Peek whether the upcoming tag at `bytes[lt_pos]` is an opening `<ul>`/`<ol>`.
/// See `flush_text`'s `next_tag_is_list` parameter for why that distinction
/// matters.
fn upcoming_tag_is_list_open(bytes: &[u8], lt_pos: usize) -> bool {
    let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
    matches!(
        upcoming_open_tag_name(bytes, lt_pos, &mut name_buf),
        Some(b"ul" | b"ol")
    )
}

fn upcoming_tag_is_inline(bytes: &[u8], lt_pos: usize) -> bool {
    let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
    upcoming_open_tag_name(bytes, lt_pos, &mut name_buf).is_some_and(is_inline_tag)
}

/// Peek whether the upcoming tag at `bytes[lt_pos]` is an opening tag named
/// exactly `name` (already lowercase). See `flush_text`'s `next_tag_is_img`
/// parameter for why that distinction matters.
fn upcoming_tag_is_named(bytes: &[u8], lt_pos: usize, name: &[u8]) -> bool {
    let mut name_buf = [0u8; MAX_TAG_NAME_BYTES];
    upcoming_open_tag_name(bytes, lt_pos, &mut name_buf) == Some(name)
}
