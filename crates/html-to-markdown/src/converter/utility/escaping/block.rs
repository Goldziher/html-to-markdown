use std::borrow::Cow;

/// `CommonMark` HTML-block start conditions of type 1: raw-text elements, whose *opening*
/// tag alone on a line starts a block. Their closing tags do not.
const HTML_BLOCK_RAW_TEXT_TAGS: [&str; 4] = ["pre", "script", "style", "textarea"];

/// `CommonMark` HTML-block start conditions of type 6 (spec 0.31.2): either an opening or a
/// closing tag with one of these names starts a block, and type 6 -- unlike type 7 -- may
/// interrupt a paragraph.
const HTML_BLOCK_TAGS: [&str; 62] = [
    "address",
    "article",
    "aside",
    "base",
    "basefont",
    "blockquote",
    "body",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hr",
    "html",
    "iframe",
    "legend",
    "li",
    "link",
    "main",
    "menu",
    "menuitem",
    "nav",
    "noframes",
    "ol",
    "optgroup",
    "option",
    "p",
    "param",
    "search",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
];

/// Escape a `CommonMark` block-structure marker that opens a *continuation* line of a link
/// label or image alt text.
///
/// Block structure is parsed before inline structure (spec appendix A), so a line inside a
/// multi-line label that reads as a block opener terminates the paragraph the label lives in
/// and the `[` / `![` never pairs with its `]`. `![A\n-\n](S)` therefore produces no image at
/// all -- it produces `<h2>![A</h2><p>](S)</p>`, with the image simply gone (issue #496).
/// A hard line break does not protect the next line either: hard breaks are inline, and block
/// parsing has already finished by the time they are looked at.
///
/// Only continuation lines are examined. A label's first line is preceded on that same line by
/// the caller's `[` / `![`, so it cannot start a block however it begins.
///
/// Returns `Cow::Borrowed` when `text` is single-line or no continuation line opens a block.
pub(super) fn escape_block_openers_on_continuation_lines(text: &str) -> Cow<'_, str> {
    if !text.contains('\n') {
        return Cow::Borrowed(text);
    }

    let mut escape_at: Vec<usize> = Vec::new();
    let mut line_start = 0usize;
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            if let Some(offset) = block_opener_escape_offset(line) {
                escape_at.push(line_start + offset);
            }
        }
        line_start += line.len() + 1;
    }

    if escape_at.is_empty() {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len() + escape_at.len());
    let mut copied = 0usize;
    for position in escape_at {
        result.push_str(&text[copied..position]);
        result.push('\\');
        copied = position;
    }
    result.push_str(&text[copied..]);
    Cow::Owned(result)
}

/// Byte offset within `line` of the character to backslash-escape so the line stops opening a
/// block, or `None` when the line opens no block that can interrupt a paragraph.
fn block_opener_escape_offset(line: &str) -> Option<usize> {
    let (indent, column) = leading_indent(line);
    // ~keep Four columns of indent is an indented code block, and an indented code block
    // ~keep cannot interrupt a paragraph -- such a line is already inert.
    if column >= 4 {
        return None;
    }
    let rest = line.get(indent..)?;
    block_opener_offset(rest).map(|offset| indent + offset)
}

/// Escape the character that makes `rest`, the first line of a paragraph without its
/// indentation, start a block, when the next line is a setext underline of `underline` (`=` or `-`).
///
/// ~keep At the start of a paragraph more lines start a block than can interrupt one: an empty
/// ~keep list item (`-`, `*`, `1.`), an ordered list at any number (spec section 5.2) and a link
/// ~keep reference definition (spec section 4.7). An underlined heading writes its text as such a
/// ~keep line (issues #653, #661).
pub fn escape_paragraph_start(rest: &str, underline: u8) -> Cow<'_, str> {
    fresh_block_opener_offset(rest)
        .or_else(|| starts_link_reference_definition(rest, underline).then_some(0))
        .map_or(Cow::Borrowed(rest), |at| {
            Cow::Owned(format!("{}\\{}", &rest[..at], &rest[at..]))
        })
}

fn fresh_block_opener_escape_offset(line: &str) -> Option<usize> {
    let (indent, column) = leading_indent(line);
    if column >= 4 {
        return None;
    }
    fresh_block_opener_offset(line.get(indent..)?).map(|offset| indent + offset)
}

fn fresh_block_opener_offset(rest: &str) -> Option<usize> {
    let structural_opener = match rest.as_bytes().first() {
        Some(b'<' | b'=') | None => None,
        Some(_) => block_opener_offset(rest),
    };
    structural_opener.or_else(|| list_marker_offset(rest))
}

/// Whether `rest`, the first line of a paragraph without its indentation, starts a link reference
/// definition when a setext underline of `underline` follows it: a label, a colon, then a
/// destination with an optional title and nothing after it, or nothing at all.
///
/// ~keep With nothing after the colon the destination is read from the next line. markdown-it
/// ~keep takes an `=` underline as that destination (`CommonMark` renderers keep the heading); a
/// ~keep `-` underline as long as a label and a colon is a thematic break, which ends the
/// ~keep definition first in both.
fn starts_link_reference_definition(rest: &str, underline: u8) -> bool {
    let Some(after_label) = link_label_len(rest).and_then(|len| rest[len..].strip_prefix(':')) else {
        return false;
    };
    let destination = after_label.trim_matches([' ', '\t']);
    if destination.is_empty() {
        return underline == b'=';
    }
    let Some(len) = link_destination_len(destination) else {
        return false;
    };
    let after_destination = &destination[len..];
    let title = after_destination.trim_start_matches([' ', '\t']);
    title.is_empty() || (title.len() < after_destination.len() && is_link_title(title))
}

/// Byte length of the link label, `[` to the first unescaped `]`, that starts `text`: at most 999
/// characters inside, no unescaped `[`, and not only spaces and tabs (spec section 4.7).
fn link_label_len(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.first() != Some(&b'[') {
        return None;
    }
    let mut at = 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            b'[' => return None,
            b']' => {
                let label = &text[1..at];
                let valid = !label.trim_matches([' ', '\t']).is_empty() && label.chars().count() <= 999;
                return valid.then_some(at + 1);
            }
            _ => at += 1,
        }
    }
    None
}

/// Byte length of the link destination that starts `text`: `<` to an unescaped `>` without a `<`
/// in between, or a run of characters other than spaces and controls with balanced parentheses.
fn link_destination_len(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.first() == Some(&b'<') {
        let mut at = 1;
        while at < bytes.len() {
            match bytes[at] {
                b'\\' => at += 2,
                b'>' => return Some(at + 1),
                b'<' => return None,
                _ => at += 1,
            }
        }
        return None;
    }
    let mut depth = 0usize;
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' if bytes.get(at + 1).is_some_and(u8::is_ascii_punctuation) => at += 1,
            b'(' => depth += 1,
            b')' if depth == 0 => break,
            b')' => depth -= 1,
            byte if byte <= b' ' || byte == 0x7f => break,
            _ => {}
        }
        at += 1;
    }
    (at > 0 && depth == 0).then_some(at)
}

/// Whether `text` is exactly one link title: `"..."`, `'...'` or `(...)`, with no unescaped
/// closing character before its end (and no unescaped `(` in a parenthesized title).
fn is_link_title(text: &str) -> bool {
    let bytes = text.as_bytes();
    let close = match bytes.first() {
        Some(b'"') => b'"',
        Some(b'\'') => b'\'',
        Some(b'(') => b')',
        _ => return false,
    };
    let mut at = 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            byte if byte == close => return at + 1 == bytes.len(),
            b'(' if close == b')' => return false,
            _ => at += 1,
        }
    }
    false
}

/// Byte offset within `text`, the trimmed text of an ATX heading, of the `#` run at its end when a parser
/// reads that run as the heading's closing sequence: the run is all of the text, or follows a
/// space or tab (spec section 4.2).
///
/// ~keep Escaping the run's first `#` keeps it as text: the run then follows a backslash
/// ~keep (issue #661).
pub fn atx_closing_sequence_offset(text: &str) -> Option<usize> {
    let run_start = text.trim_end_matches('#').len();
    let closes = run_start < text.len() && (run_start == 0 || matches!(text.as_bytes()[run_start - 1], b' ' | b'\t'));
    closes.then_some(run_start)
}

/// Byte offset of the delimiter of the list marker that starts `rest`, a line without its
/// indentation: a bullet, or one to nine digits then `.` or `)`, followed by a space, a tab or
/// the end of the line.
fn list_marker_offset(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let delimiter = match bytes.first()? {
        b'-' | b'*' | b'+' => 0,
        b'0'..=b'9' => {
            let digits = bytes.iter().take_while(|byte| byte.is_ascii_digit()).count();
            if digits > 9 || !matches!(bytes.get(digits), Some(b'.' | b')')) {
                return None;
            }
            digits
        }
        _ => return None,
    };
    matches!(bytes.get(delimiter + 1), None | Some(b' ' | b'\t')).then_some(delimiter)
}

/// Whether `line`, with its indentation, opens a block that can interrupt a paragraph.
pub fn line_opens_block(line: &str) -> bool {
    block_opener_escape_offset(line).is_some()
}

/// Escape the block opener at the start of `buffer[from..]`, text just written, when that text
/// starts a line that continues the paragraph above it.
///
/// ~keep A line after a hard break continues its paragraph only if it cannot interrupt it, the
/// ~keep rule a link label's continuation lines follow (issue #651). The text starts such a line
/// ~keep when only its container's indent is before it on the line and the line above holds text;
/// ~keep after a blank line it starts a paragraph of its own. The indent scan stops at the first
/// ~keep other byte, and the line above is read once per line, so the check stays linear.
pub fn escape_continuation_line_start(buffer: &mut String, from: usize, after_external_hard_break: bool) {
    let before = &buffer[..from];
    let continues_paragraph = if let Some(line_end) = before.trim_end_matches([' ', '\t']).strip_suffix('\n') {
        let line_above = &line_end[line_end.rfind('\n').map_or(0, |pos| pos + 1)..];
        !line_above.trim().is_empty()
    } else {
        after_external_hard_break && before.trim_matches([' ', '\t']).is_empty()
    };
    if !continues_paragraph {
        return;
    }
    let text = &buffer[from..];
    let line = &text[..text.find('\n').unwrap_or(text.len())];
    if let Some(offset) = block_opener_escape_offset(line) {
        buffer.insert(from + offset, '\\');
    }
}

/// ~keep Whether `buffer` ends immediately after a Markdown or Djot hard-break marker.
pub fn ends_with_hard_break(buffer: &str) -> bool {
    let Some(before_newline) = buffer.strip_suffix('\n') else {
        return false;
    };
    let line = &before_newline[before_newline.rfind('\n').map_or(0, |position| position + 1)..];
    let without_spaces = line.trim_end_matches(' ');
    let spaces = line.len() - without_spaces.len();
    if without_spaces.trim_matches([' ', '\t']).is_empty() {
        return false;
    }
    if spaces >= 2 {
        return true;
    }
    let backslashes = without_spaces.len() - without_spaces.trim_end_matches('\\').len();
    spaces == 0 && !backslashes.is_multiple_of(2)
}

/// ~keep Escape text just written at the start of a Markdown block or directly after a list-item
/// marker when that text would otherwise be parsed as block structure (issue #735).
pub fn escape_block_start(buffer: &mut String, from: usize, in_list_item: bool, followed_by_inline: bool) {
    let before = &buffer[..from];
    // ~keep A letter already written on this line makes every supported fresh block opener
    // inert or already escaped. Avoid rescanning/copying the growing line for each span (#773).
    if before
        .trim_end_matches([' ', '\t'])
        .as_bytes()
        .last()
        .is_some_and(u8::is_ascii_alphabetic)
    {
        return;
    }
    let line_start = before.rfind('\n').map_or(0, |position| position + 1);
    let prefix = &before[line_start..];
    let content_start = if in_list_item {
        list_item_content_start(prefix).map_or(from, |offset| line_start + offset)
    } else if prefix.trim().is_empty() || is_unfinished_ordered_marker(prefix) {
        line_start
    } else {
        return;
    };
    let text = &buffer[content_start..];
    let first_line = &text[..text.find('\n').unwrap_or(text.len())];
    let continues_paragraph = before
        .trim_end_matches([' ', '\t'])
        .strip_suffix('\n')
        .is_some_and(|preceding| {
            let preceding_line = &preceding[preceding.rfind('\n').map_or(0, |position| position + 1)..];
            !preceding_line.trim().is_empty()
        });

    if followed_by_inline {
        let mut continued = String::with_capacity(first_line.len() + 1);
        continued.push_str(first_line);
        continued.push('x');
        if fresh_block_opener_escape_offset(&continued).is_none() {
            return;
        }
    }

    let escape_at = if continues_paragraph {
        None
    } else {
        fresh_block_opener_escape_offset(first_line).map(|offset| content_start + offset)
    };

    if let Some(position) = escape_at {
        buffer.insert(position, '\\');
    }
}

fn is_unfinished_ordered_marker(prefix: &str) -> bool {
    let (indent_len, column) = leading_indent(prefix);
    if column >= 4 {
        return false;
    }
    let rest = &prefix[indent_len..];
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    (1..=9).contains(&digits) && matches!(rest.as_bytes().get(digits), Some(b'.' | b')')) && rest.len() == digits + 1
}

fn list_item_content_start(line: &str) -> Option<usize> {
    let (indent_len, _) = leading_indent(line);
    let mut position = indent_len;
    let mut found = false;
    loop {
        let rest = &line[position..];
        let marker_len = match rest.as_bytes().first() {
            Some(b'-' | b'*' | b'+') => 1,
            Some(b'0'..=b'9') => {
                let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
                let Some(delimiter) = rest.as_bytes().get(digits) else {
                    break;
                };
                if !matches!(delimiter, b'.' | b')') {
                    break;
                }
                digits + 1
            }
            _ => break,
        };
        let spacing = rest[marker_len..]
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
        if spacing == 0 {
            break;
        }
        found = true;
        position += marker_len + spacing;
    }
    found.then_some(position)
}

/// Escape numbered text at the start of a Djot list item so it stays literal text.
pub fn escape_djot_list_item_start(buffer: &mut String, from: usize, in_list_item: bool) {
    if !in_list_item {
        return;
    }
    let before = &buffer[..from];
    let line_start = before.rfind('\n').map_or(0, |position| position + 1);
    let Some(content_offset) = list_item_content_start(&before[line_start..]) else {
        return;
    };
    let content_start = line_start + content_offset;
    let text = &buffer[content_start..];
    let first_line = &text[..text.find('\n').unwrap_or(text.len())];
    let digits = first_line.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0
        || !matches!(first_line.as_bytes().get(digits), Some(b'.' | b')'))
        || !matches!(first_line.as_bytes().get(digits + 1), Some(b' ' | b'\t'))
    {
        return;
    }
    buffer.insert(content_start + digits, '\\');
}

/// ~keep Djot parses leading dash runs as typographic dashes and leading backticks as verbatim
/// markup even on a continuation line. Escape every marker in those runs after a hard break.
pub fn escape_djot_continuation_line_start(buffer: &mut String, from: usize, after_external_hard_break: bool) {
    let line_start = if let Some(position) = buffer.rfind('\n') {
        position + 1
    } else if after_external_hard_break && buffer[..from].trim_matches([' ', '\t']).is_empty() {
        from
    } else {
        return;
    };
    if line_start > 0 {
        let previous_line = &buffer[..line_start - 1];
        let previous_line_start = previous_line.rfind('\n').map_or(0, |position| position + 1);
        let previous_line = &previous_line[previous_line_start..];
        let trailing_backslashes = previous_line.len() - previous_line.trim_end_matches('\\').len();
        if trailing_backslashes.is_multiple_of(2) {
            return;
        }
    }

    let indent = buffer[line_start..]
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let run_start = line_start + indent;
    let Some(marker) = buffer.as_bytes().get(run_start).and_then(|byte| match byte {
        b'-' | b'`' => Some(*byte),
        b'\\' => buffer
            .as_bytes()
            .get(run_start + 1)
            .filter(|next| matches!(next, b'-' | b'`'))
            .copied(),
        _ => None,
    }) else {
        return;
    };

    let mut position = run_start;
    let mut count = 0usize;
    while position < buffer.len() {
        if buffer.as_bytes()[position] == marker {
            position += 1;
            count += 1;
        } else if buffer.as_bytes()[position] == b'\\' && buffer.as_bytes().get(position + 1) == Some(&marker) {
            position += 2;
            count += 1;
        } else {
            break;
        }
    }
    if count == 0 || (marker == b'-' && count < 2) {
        return;
    }

    let mut escaped = String::with_capacity(count * 2);
    for _ in 0..count {
        escaped.push('\\');
        escaped.push(char::from(marker));
    }
    buffer.replace_range(run_start..position, &escaped);
}

/// Whether `rest`, a line without its indentation, opens a block that can interrupt a paragraph.
pub fn opens_block(rest: &str) -> bool {
    block_opener_offset(rest).is_some()
}

/// Whether `rest`, a line without its indentation, is a setext heading underline: nothing but `=`
/// or nothing but `-`.
pub fn is_heading_underline(rest: &str) -> bool {
    is_setext_underline(rest, b'=') || is_setext_underline(rest, b'-')
}

/// Whether `rest`, a line without its indentation, is a thematic break: three or more `-`, `*` or
/// `_` and nothing else but spaces/tabs.
pub fn is_rule(rest: &str) -> bool {
    rest.bytes()
        .next()
        .is_some_and(|marker| matches!(marker, b'-' | b'*' | b'_') && is_thematic_break(rest, marker))
}

/// The fence character and the length of its run when `rest`, a line without its indentation,
/// opens a fenced code block.
pub fn code_fence(rest: &str) -> Option<(u8, usize)> {
    let marker = *rest.as_bytes().first()?;
    let run = rest.bytes().take_while(|&byte| byte == marker).count();
    (matches!(marker, b'`' | b'~') && is_code_fence(rest, marker)).then_some((marker, run))
}

/// Byte offset within `rest`, a line without its indentation, of the character to backslash-escape
/// so the line stops opening a block, or `None` when it opens no block that can interrupt a
/// paragraph.
pub(super) fn block_opener_offset(rest: &str) -> Option<usize> {
    let marker = *rest.as_bytes().first()?;
    let opens = match marker {
        b'>' => true,
        b'#' => is_atx_heading(rest),
        b'`' | b'~' => is_code_fence(rest, marker),
        b'=' => is_setext_underline(rest, b'='),
        b'-' => is_setext_underline(rest, b'-') || is_thematic_break(rest, b'-') || is_bullet_list_item(rest),
        b'*' => is_thematic_break(rest, b'*') || is_bullet_list_item(rest),
        b'+' => is_bullet_list_item(rest),
        b'_' => is_thematic_break(rest, b'_'),
        b'<' => is_html_block_opener(rest),
        // ~keep A digit cannot carry a backslash escape, so an ordered-list marker is
        // ~keep defused at its `.`/`)` delimiter instead of at its number.
        b'0'..=b'9' => return ordered_list_delimiter_offset(rest),
        _ => false,
    };
    opens.then_some(0)
}

/// Split `line`'s leading indentation, returning `(byte length, column width)`.
pub fn leading_indent(line: &str) -> (usize, usize) {
    let mut length = 0usize;
    let mut column = 0usize;
    for &byte in line.as_bytes() {
        match byte {
            b' ' => column += 1,
            // ~keep A tab advances to the next multiple of four (spec section 2.2), so a
            // ~keep single leading tab is already four columns of indent.
            b'\t' => column += 4 - column % 4,
            _ => break,
        }
        length += 1;
    }
    (length, column)
}

/// An ATX heading: one to six `#`, then a space/tab or the end of the line.
fn is_atx_heading(rest: &str) -> bool {
    let hashes = rest.bytes().take_while(|&byte| byte == b'#').count();
    (1..=6).contains(&hashes) && matches!(rest.as_bytes().get(hashes), None | Some(b' ' | b'\t' | b'\r'))
}

/// An opening code fence: three or more of the same fence character.
fn is_code_fence(rest: &str, fence: u8) -> bool {
    let run = rest.bytes().take_while(|&byte| byte == fence).count();
    // ~keep A backtick fence's info string may not itself contain a backtick, so such a line
    // ~keep is ordinary text and needs no escape.
    run >= 3 && (fence != b'`' || !rest[run..].contains('`'))
}

/// A setext heading underline: nothing but `marker`, plus optional trailing whitespace.
fn is_setext_underline(rest: &str, marker: u8) -> bool {
    let trimmed = rest.trim_end();
    !trimmed.is_empty() && trimmed.bytes().all(|byte| byte == marker)
}

/// A thematic break: three or more `marker` characters and nothing else but spaces/tabs.
fn is_thematic_break(rest: &str, marker: u8) -> bool {
    let mut count = 0usize;
    for byte in rest.bytes() {
        if byte == marker {
            count += 1;
        } else if !matches!(byte, b' ' | b'\t' | b'\r') {
            return false;
        }
    }
    count >= 3
}

/// A bullet list item that can interrupt a paragraph: a marker, a space/tab, then content.
fn is_bullet_list_item(rest: &str) -> bool {
    // ~keep An empty list item cannot interrupt a paragraph (spec section 5.2), so both the
    // ~keep separating space/tab and non-blank content after it are required.
    matches!(rest.as_bytes().get(1), Some(b' ' | b'\t')) && rest.get(2..).is_some_and(|tail| !tail.trim().is_empty())
}

/// Byte offset of the `.`/`)` of an ordered list marker that can interrupt a paragraph.
fn ordered_list_delimiter_offset(rest: &str) -> Option<usize> {
    let digits = first_ordered_marker_len(rest)? - 1;
    let bytes = rest.as_bytes();
    let starts_a_list = matches!(bytes.get(digits + 1), Some(b' ' | b'\t'))
        && rest.get(digits + 2..).is_some_and(|tail| !tail.trim().is_empty());
    starts_a_list.then_some(digits)
}

/// The byte length of the ordered list marker at the start of `rest` when that marker starts a
/// list at 1: one to nine digits whose value is 1 (`1.`, `01)`, `000000001.`), then `.` or `)`.
///
/// ~keep Only a list starting at 1 can interrupt a paragraph, and the start number is the
/// ~keep marker's value, so leading zeros count; ten digits are no marker (spec section 5.2).
pub fn first_ordered_marker_len(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let digits = bytes.iter().take_while(|byte| byte.is_ascii_digit()).count();
    let value_is_one =
        (1..=9).contains(&digits) && bytes[digits - 1] == b'1' && bytes[..digits - 1].iter().all(|&byte| byte == b'0');
    (value_is_one && matches!(bytes.get(digits), Some(b'.' | b')'))).then_some(digits + 1)
}

/// An HTML block of type 1 to 6 -- the types that may interrupt a paragraph.
///
/// Type 7 (any other complete open tag alone on its line) deliberately may not, so inline
/// markup such as a `<span>` opening a continuation line is left alone.
fn is_html_block_opener(rest: &str) -> bool {
    let after_bracket = &rest[1..];
    // ~keep Types 2-5: comment, processing instruction, declaration, CDATA.
    if after_bracket.starts_with("!--")
        || after_bracket.starts_with('?')
        || after_bracket.starts_with("![CDATA[")
        || (after_bracket.starts_with('!') && after_bracket.as_bytes().get(1).is_some_and(u8::is_ascii_alphabetic))
    {
        return true;
    }

    let is_closing = after_bracket.starts_with('/');
    let name_start = usize::from(is_closing);
    let name: String = after_bracket[name_start..]
        .bytes()
        .take_while(u8::is_ascii_alphanumeric)
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect();
    if name.is_empty() {
        return false;
    }

    // ~keep Both start conditions require the tag name to end at whitespace, `>`, `/>` or the
    // ~keep end of the line; anything else (`<divx`, `<div=`) is not a tag at all.
    let tail = &after_bracket[name_start + name.len()..];
    let is_terminated =
        tail.is_empty() || tail.starts_with('>') || tail.starts_with("/>") || tail.starts_with(char::is_whitespace);

    is_terminated
        && (HTML_BLOCK_TAGS.contains(&name.as_str())
            || (!is_closing && HTML_BLOCK_RAW_TEXT_TAGS.contains(&name.as_str())))
}

/// Helper for block-level element detection.
pub fn is_block_level_name(tag_name: &str, is_inline: bool) -> bool {
    !is_inline
        && matches!(
            tag_name,
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "canvas"
                | "center"
                | "dd"
                | "details"
                | "dialog"
                | "div"
                | "dl"
                | "dt"
                | "fieldset"
                | "figcaption"
                | "figure"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "hgroup"
                | "hr"
                | "legend"
                | "li"
                | "main"
                | "menu"
                | "nav"
                | "ol"
                | "p"
                | "pre"
                | "search"
                | "section"
                | "summary"
                | "table"
                | "tfoot"
                | "ul"
        )
}
