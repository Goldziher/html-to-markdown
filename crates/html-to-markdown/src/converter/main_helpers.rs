//! Helper functions for HTML to Markdown conversion.
//!
//! This module contains utility functions used by the main conversion pipeline,
//! including preprocessing helpers, HTML repair, and metadata formatting.

use crate::converter::code_scan::CodeScan;
use crate::options::CodeBlockStyle;
use crate::options::ConversionOptions;
use crate::options::NewlineStyle;
use crate::options::conversion::{MAX_CONFIGURABLE_DEPTH, NATIVE_STACK_SAFE_DEPTH};

mod head;

use head::document_head_search;
pub use head::{
    document_head, extract_head_metadata, format_metadata_frontmatter, is_ignorable_before_head, starts_body,
};

/// Resolve the effective traversal-depth ceiling.
///
/// When the caller does not set `max_depth`, the conservative native default is
/// used. An explicit `max_depth` is honored — allowing callers to raise the
/// ceiling above the native default (issue #434) — but is clamped to
/// [`MAX_CONFIGURABLE_DEPTH`] so the recursive walker cannot overflow the stack.
pub fn effective_max_depth(options: &ConversionOptions) -> usize {
    match options.max_depth {
        None => NATIVE_STACK_SAFE_DEPTH,
        Some(explicit) => explicit.min(MAX_CONFIGURABLE_DEPTH),
    }
}

/// Compare two tag names case-insensitively.
pub fn tag_name_eq(a: impl AsRef<str>, b: &str) -> bool {
    a.as_ref().eq_ignore_ascii_case(b)
}

/// Remove trailing spaces and tabs from a string.
pub fn trim_trailing_whitespace(output: &mut String) {
    while output.ends_with([' ', '\t']) {
        output.pop();
    }
}

/// Whether `text` is empty or composed only of plain ASCII whitespace (`' '`,
/// `'\t'`, `'\n'`, `'\r'`).
///
/// ~keep Unlike `str::trim().is_empty()`, this does NOT treat other Unicode
/// ~keep whitespace (a decoded `&nbsp;`, a thin space, etc.) as "empty": such a
/// ~keep character trims to nothing under `str::trim`'s Unicode-aware definition,
/// ~keep but it is still significant, visible content. A caller deciding whether a
/// ~keep whitespace-looking text node carries zero content -- safe to collapse or
/// ~keep drop outright -- must use this instead, or it silently swallows real
/// ~keep content that merely LOOKS like formatting whitespace. `block/paragraph.rs`'s
/// ~keep empty-inline-neighbour skip used to do exactly that: a lone nbsp between
/// ~keep two `<br>`s vanished because the skip used the Unicode-aware check.
pub fn is_ascii_whitespace_only(text: &str) -> bool {
    text.chars().all(|c| matches!(c, ' ' | '\t' | '\n' | '\r'))
}

/// Strip a trailing run of backslash hard-break markers from the end of a block's content.
///
/// CommonMark's hard-break rule has no effect at the end of a block: there is no next line
/// for the break to reach. For the two-space style the leftover marker is invisible trailing
/// whitespace, so it is harmless and left in place. For the backslash style the leftover `\`
/// is a literal, visible character, so it must go — but each `"\\\n"` marker in the run keeps
/// its `\n` (only the `\` is dropped). The freed newline is what a block boundary is already
/// made of, so callers that then run their own "how many blank lines are already here"
/// separator logic (every block handler's `needs_leading_sep`-shaped check) keep working
/// unmodified, and any surplus blank lines a run of several `<br>` collapses into are mopped
/// up by `collapse_excess_blank_lines` at the end of the document (issue #464). `block_start`
/// bounds the strip to the current block so an earlier block's content already in `output` is
/// never touched.
pub fn strip_trailing_backslash_breaks(output: &mut String, block_start: usize) {
    // ~keep `block_content_start` is set only by the paragraph handler, so a handler that
    // ~keep walks its children into a fresh buffer while inheriting that context leaves the
    // ~keep index pointing into the wrong buffer. That is not hypothetical: it is the root
    // ~keep cause of the reported panics in issues #216/#217, and `text_node.rs` clamps the
    // ~keep same index the same way for the same reason. The `output.len() > block_start`
    // ~keep loop guard below already rules out an out-of-bounds slice, but not a stale index
    // ~keep that lands mid-UTF-8-character, which slicing would also panic on.
    let block_start = crate::converter::utility::content::floor_char_boundary(output, block_start.min(output.len()));
    let mut stripped_breaks = 0usize;
    while output.len() > block_start && output[block_start..].ends_with("\\\n") {
        let mut new_len = output.len() - "\\\n".len();
        // ~keep A marker on a line of its own in a list item follows the item's indent
        // ~keep (issue #681); left behind, the indent hid the marker before it from this loop.
        let indent_start = output[..new_len].trim_end_matches([' ', '\t']).len();
        if indent_start >= block_start && output[..indent_start].ends_with('\n') {
            new_len = indent_start;
        }
        output.truncate(new_len);
        stripped_breaks += 1;
    }
    for _ in 0..stripped_breaks {
        output.push('\n');
    }
}

/// Strip a trailing backslash hard-break run from a self-contained block-content buffer,
/// before that buffer is trimmed and spliced back into the shared output.
///
/// Several block handlers (blockquote, sectioning elements, details/summary, figure/
/// figcaption, dl/dt/dd, list items) walk their children into a fresh, empty local
/// `String` rather than the shared `output`, then splice the trimmed result back.
/// `str::trim` cannot repair a trailing `"\\\n"` marker by itself — trim only removes
/// whitespace, and `\` is not whitespace, so it eats the newline and leaves the
/// backslash stranded (issue #464 follow-up). Since the buffer is fresh, position `0`
/// is always its own start, so callers only need to name the buffer and the style.
pub fn strip_trailing_backslash_breaks_from_fresh_buffer(content: &mut String, newline_style: NewlineStyle) {
    if newline_style == NewlineStyle::Backslash {
        strip_trailing_backslash_breaks(content, 0);
    }
}

/// Emit a line break for a `<br>`, `<div>`, or `<p>` continuation inside a table cell.
///
/// A Markdown table cell cannot contain a hard line break: neither `newline_style` form
/// (two-space or backslash) is valid there, and a raw newline corrupts the row by
/// splitting its pipe syntax across physical lines. So this always trims trailing
/// whitespace first, then emits a literal `<br>` when `br_in_tables` is true, or
/// collapses to a single space otherwise — `newline_style` is never consulted inside a
/// cell (issue #453, issue #454). The `output.is_empty()` guard suppresses a leading
/// space when the continuation is the first content in the cell; a leading `<br>` is
/// still emitted under `br_in_tables`, preserving the pre-existing `<br>` behaviour.
pub fn emit_table_cell_break(output: &mut String, br_in_tables: bool) {
    trim_trailing_whitespace(output);
    if br_in_tables {
        output.push_str("<br>");
    } else if !output.is_empty() {
        output.push(' ');
    }
}

/// Separate a block in a table cell from the cell content before it (issue #645).
///
/// A cell holds one line, so the cell break of [`emit_table_cell_break`] stands for the line a
/// block starts or ends outside a cell. Nothing is written when the cell holds no content yet or
/// already ends with a `<br>`. A line end that a block left folds into the break.
pub fn separate_block_in_cell(output: &mut String, br_in_tables: bool) {
    let content_end = output.trim_end_matches([' ', '\t', '\n']).len();
    output.truncate(content_end);
    if !output.is_empty() && !output.ends_with("<br>") {
        emit_table_cell_break(output, br_in_tables);
    }
}

/// Emit a cell break while keeping a table-cell `<pre>` break outside its code spans. ~keep
pub fn emit_table_cell_break_in_context(output: &mut String, br_in_tables: bool, ctx: &crate::converter::Context) {
    if br_in_tables && let Some(ref offsets) = ctx.pre_cell_break_offsets {
        trim_trailing_whitespace(output);
        offsets.borrow_mut().push(output.len());
        output.push('\n');
    } else {
        emit_table_cell_break(output, br_in_tables);
    }
}

/// The literal hard-break marker `line_break.rs` emits for a real `<br>`.
///
/// Shared by `line_break.rs`'s own non-code fallback and by the split code-span emitters
/// (`handlers::code_block::emit_inline_code`, `inline::code::handle_kbd_samp`) that join a
/// `<code>`/`<kbd>`/`<samp>` element's `<br>`-separated segments with this same marker,
/// now placed OUTSIDE the backticks where it is syntax rather than span content
/// (issue #487).
#[must_use]
pub const fn hard_break_marker(options: &ConversionOptions) -> &'static str {
    if matches!(options.output_format, crate::options::OutputFormat::Djot) {
        return "\\\n";
    }
    match options.newline_style {
        NewlineStyle::Spaces => "  \n",
        NewlineStyle::Backslash => "\\\n",
    }
}

/// Collapse runs of three or more consecutive newlines into exactly two.
///
/// Block-level emitters append their own trailing newlines and the next block
/// emitter typically prepends a leading newline, which can produce `\n\n\n`
/// runs in transitions such as frontmatter → first block or list → next block.
/// markdownlint's MD012 rule forbids multiple consecutive blank lines, so the
/// final emission is normalized here. This intentionally preserves single
/// blank lines (`\n\n`) — only runs of three or more newlines are collapsed.
///
/// The lines of a code block, fenced or indented, are code, not block transitions, and stay as
/// they are (issue #783).
pub fn collapse_excess_blank_lines(output: &mut String, style: CodeBlockStyle) {
    if !output.contains("\n\n\n") {
        return;
    }
    let mut cleaned = String::with_capacity(output.len());
    let mut fences = CodeScan::new(style, output);
    let mut consecutive = 0usize;
    for line in output.split_inclusive('\n') {
        let content = line.strip_suffix('\n').unwrap_or(line);
        if fences.is_code(content) || !content.is_empty() {
            consecutive = 1;
            cleaned.push_str(line);
        } else {
            consecutive += 1;
            if consecutive <= 2 {
                cleaned.push_str(line);
            }
        }
    }
    *output = cleaned;
}

/// Remove trailing spaces/tabs from every line while preserving newlines.
///
/// The lines of a code block, fenced or indented, keep their line ends: they are code.
pub fn trim_line_end_whitespace(output: &mut String, style: CodeBlockStyle) {
    if output.is_empty() {
        return;
    }

    let mut cleaned = String::with_capacity(output.len());
    let mut fences = CodeScan::new(style, output);
    for line in output.split('\n') {
        if fences.is_code(line) {
            cleaned.push_str(line);
            cleaned.push('\n');
            continue;
        }
        let content = line.trim_end_matches([' ', '\t']);
        cleaned.push_str(content);
        // ~keep The two-space hard break is only meaningful after content on the same line;
        // ~keep on a blank line CommonMark treats it as ordinary trailing whitespace. Keeping
        // ~keep it there made whitespace-only documents render as "  \n" instead of "".
        if !content.is_empty() && line.ends_with("  ") {
            cleaned.push_str("  ");
        }
        cleaned.push('\n');
    }

    let trimmed = cleaned.trim_end_matches('\n');
    if trimmed.is_empty() {
        *output = String::new();
    } else {
        if trimmed.len() < cleaned.len() {
            cleaned.truncate(trimmed.len() + 1);
        } else {
            cleaned.push('\n');
        }
        *output = cleaned;
    }
}

/// Check for custom elements or paragraphs misnested inside namespaced elements.
pub fn has_custom_element_tags(html: &str) -> bool {
    // ~keep Custom elements must have a hyphen in their TAG NAME, not in attributes.
    // ~keep Namespaced Office tags also need HTML5 repair when paragraphs are misnested.
    // ~keep A markup declaration/comment (`<!...>`), a processing instruction (`<?...?>`),
    // ~keep or a CDATA section is NOT a tag — its "name" is arbitrary content that may
    // ~keep itself contain a hyphen (`<!--c-->` naively yields `!--c--`, which contains
    // ~keep one), so every comment-bearing document was misreported as having a custom
    // ~keep element. Those constructs are skipped wholesale instead of scanned for a name.
    // ~keep HTML5 begins a tag name only when `<`/`</` is IMMEDIATELY followed by an ASCII
    // ~keep letter (see `tier1::parse::is_tag_name_start`) — no whitespace skip first, so
    // ~keep `< div>`/`</ div>` are left as plain text rather than treated as tags.
    let bytes = html.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut namespaced_tags = Vec::new();

    while i < len {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }

        let after_lt = i + 1;
        let Some(&next) = bytes.get(after_lt) else {
            break;
        };

        if next == b'!' {
            i = skip_markup_declaration(bytes, i);
            continue;
        }

        if next == b'?' {
            i = skip_to_after_gt(bytes, after_lt);
            continue;
        }

        let name_start = if next == b'/' { after_lt + 1 } else { after_lt };

        if name_start >= len || !crate::converter::tier1::parse::is_tag_name_start(bytes[name_start]) {
            // ~keep Not a real tag start (e.g. `< div>`, `</ div>`, `</>`) — leave it as
            // ~keep text and resume scanning right after the `<`.
            i = after_lt;
            continue;
        }

        let tag_start = name_start;
        let mut tag_end = tag_start;
        while tag_end < len {
            let ch = bytes[tag_end];
            if ch == b'>' || ch == b'/' || ch.is_ascii_whitespace() {
                break;
            }
            tag_end += 1;
        }

        let name = &bytes[tag_start..tag_end];
        if name.contains(&b'-') {
            return true;
        }
        let (self_closing, terminator) = scan_tag_terminator(bytes, tag_end);
        if namespaced_paragraph_is_misnested(&mut namespaced_tags, name, next == b'/', self_closing) {
            return true;
        }
        i = terminator.saturating_add(if self_closing { 2 } else { 1 });
    }

    false
}

/// Track only namespace subtrees, leaving well-formed Office documents on the normal parser path.
fn namespaced_paragraph_is_misnested<'a>(
    open_tags: &mut Vec<&'a [u8]>,
    name: &'a [u8],
    closing: bool,
    self_closing: bool,
) -> bool {
    if closing {
        if name.contains(&b':') {
            if let Some(position) = open_tags.iter().rposition(|open| open.eq_ignore_ascii_case(name)) {
                let unclosed_paragraph = open_tags[position + 1..]
                    .iter()
                    .any(|open| open.eq_ignore_ascii_case(b"p"));
                open_tags.truncate(position);
                return unclosed_paragraph;
            }
        } else if open_tags.last().is_some_and(|open| open.eq_ignore_ascii_case(name)) {
            open_tags.pop();
        }
    } else if !self_closing && (!open_tags.is_empty() || name.contains(&b':')) && !is_html5_void_element(name) {
        open_tags.push(name);
    }
    false
}

/// Skip a `<!...>` construct — an HTML comment, a CDATA section, or another
/// markup declaration such as `<!doctype html>` — starting at the `<` at
/// `lt_idx`. Returns the index just past the construct (or `bytes.len()` if
/// it is never terminated).
fn skip_markup_declaration(bytes: &[u8], lt_idx: usize) -> usize {
    // ~keep Reuses the exact comment/CDATA terminator scan `strip_hidden_elements`
    // ~keep relies on, rather than re-implementing it a third time in this crate.
    if let Some(end) = crate::converter::utility::preprocessing::skip_opaque_region(bytes, lt_idx) {
        return end;
    }
    skip_to_after_gt(bytes, lt_idx + 2)
}

/// Index just past the next `>` at or after `from`, or `bytes.len()` if none
/// exists (an unterminated declaration/processing-instruction runs to EOF).
fn skip_to_after_gt(bytes: &[u8], from: usize) -> usize {
    let start = from.min(bytes.len());
    match bytes[start..].iter().position(|&b| b == b'>') {
        Some(offset) => start + offset + 1,
        None => bytes.len(),
    }
}

/// HTML5 void elements that are self-closing by spec and must NOT be expanded.
///
/// These elements are always void in HTML5: they have no end tag, and `<br />` is
/// equivalent to `<br>`.  We must leave them as-is when pre-processing XML-style
/// self-closing syntax so that `repair_with_html5ever` can parse them correctly.
const HTML5_VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr",
];

/// Advance past a `</tag>` closing tag, starting at the `/` immediately after `<` (`i` points
/// at that `/`). Returns the index just past the `>`, or `len` if never terminated. Extracted
/// from `expand_xml_self_closing_tags` — identical scan, unchanged.
fn skip_closing_tag(bytes: &[u8], mut i: usize, len: usize) -> usize {
    while i < len && bytes[i] != b'>' {
        i += 1;
    }
    if i < len {
        i += 1;
    }
    i
}

/// Scan a tag name starting at `start`, stopping at `>`, `/`, or ASCII whitespace. Returns the
/// end index (exclusive) of the name. Extracted from `expand_xml_self_closing_tags` — identical
/// scan, unchanged.
pub fn scan_tag_name_end(bytes: &[u8], start: usize) -> usize {
    let len = bytes.len();
    let mut i = start;
    while i < len {
        let ch = bytes[i];
        if ch == b'>' || ch == b'/' || ch.is_ascii_whitespace() {
            break;
        }
        i += 1;
    }
    i
}

/// Whether `tag_name_bytes` (ASCII-case-insensitive) names an HTML5 void element. Extracted
/// from `expand_xml_self_closing_tags` — identical lowercase-and-compare, unchanged.
pub fn is_html5_void_element(tag_name_bytes: &[u8]) -> bool {
    let tag_name_lower = tag_name_bytes.iter().map(u8::to_ascii_lowercase).collect::<Vec<_>>();
    HTML5_VOID_ELEMENTS
        .iter()
        .any(|v| v.as_bytes() == tag_name_lower.as_slice())
}

/// Scan forward from `start` (the position right after the tag name) to find where the tag's
/// attribute list ends, honoring quoted attribute values so a `/` or `>` inside a quoted string
/// is not mistaken for the tag terminator. Returns `(self_closing, end)` where `end` is the
/// index of the terminating `/` (when `self_closing` is true) or `>` (when false), or `len` if
/// the tag is never closed. Extracted from `expand_xml_self_closing_tags` — identical
/// quote-tracking scan, unchanged.
fn scan_tag_terminator(bytes: &[u8], start: usize) -> (bool, usize) {
    let len = bytes.len();
    let mut i = start;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while i < len {
        match bytes[i] {
            b'"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                i += 1;
            }
            b'\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                i += 1;
            }
            b'/' if !in_single_quote && !in_double_quote => {
                if i + 1 < len && bytes[i + 1] == b'>' {
                    return (true, i);
                }
                i += 1;
            }
            b'>' if !in_single_quote && !in_double_quote => {
                return (false, i);
            }
            _ => {
                i += 1;
            }
        }
    }

    (false, len)
}

/// Expand XML-style self-closing tags to explicit open+close pairs.
///
/// HTML5 does not honour the `/>` self-close syntax for non-void elements.  When
/// `repair_with_html5ever` re-parses content that contains custom / namespaced tags
/// written as `<ac:parameter name="foo" />`, the HTML5 parser treats the `/>` as `>`
/// and leaves the element open.  Subsequent siblings then nest inside it, breaking
/// visitor pre-order/post-order start/end pairing.
///
/// This function scans the input byte-by-byte and rewrites any `<tag ... />` where
/// `tag` is not a known HTML5 void element into `<tag ...></tag>`.  Known void
/// elements are left unchanged because they must not receive an explicit close tag.
///
/// # Correctness guarantees
/// - Non-ASCII bytes are never interpreted as structural characters; all multi-byte
///   UTF-8 sequences pass through unmodified via `&input[byte_offset..]` slicing.
/// - Attribute values containing `/>` are skipped correctly (the scanner tracks
///   whether it is inside a quoted attribute).
/// - `</closing>` tags are never modified.
/// - The function is pure and returns a new `String`; if no substitution is needed
///   the allocation is still performed (cheap given repair is already rare).
pub fn expand_xml_self_closing_tags(input: &str) -> String {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut output = String::with_capacity(len);
    let mut copy_start = 0usize;
    let mut i = 0;

    while i < len {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }

        let tag_open = i;
        i += 1;

        if i < len && bytes[i] == b'/' {
            i = skip_closing_tag(bytes, i, len);
            continue;
        }

        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        let name_start = i;
        i = scan_tag_name_end(bytes, name_start);
        let tag_name_bytes = &bytes[name_start..i];

        if tag_name_bytes.is_empty() {
            continue;
        }

        let is_void = is_html5_void_element(tag_name_bytes);

        let attrs_start = i;
        let (self_closing, terminator) = scan_tag_terminator(bytes, attrs_start);
        i = terminator;

        if self_closing && !is_void {
            output.push_str(&input[copy_start..tag_open]);

            let tag_name_str = std::str::from_utf8(tag_name_bytes).unwrap_or("");
            let attrs_part = &input[attrs_start..i];

            output.push('<');
            output.push_str(tag_name_str);
            output.push_str(attrs_part);
            output.push('>');
            output.push('<');
            output.push('/');
            output.push_str(tag_name_str);
            output.push('>');

            i += 2;
            copy_start = i;
        } else {
            if i < len && bytes[i] == b'/' {
                i += 2;
            } else if i < len && bytes[i] == b'>' {
                i += 1;
            }
        }
    }

    output.push_str(&input[copy_start..]);
    output
}

/// Try to repair HTML using html5ever parser.
///
/// Returns `Some(repaired_html)` if repair was successful, None otherwise.
///
/// Before feeding the input to the HTML5 parser, XML-style self-closing tags on
/// non-void elements (e.g. `<ac:parameter name="foo" />`) are expanded to explicit
/// open+close pairs.  This preserves the intended document structure because HTML5
/// semantics do not honour `/>` on unknown elements — without the expansion, the
/// element would be left open and subsequent siblings would nest inside it, breaking
/// visitor start/end event pairing (issue #331).
pub fn repair_with_html5ever(input: &str) -> Option<String> {
    use crate::converter::anchor_origin::parse_with_anchor_origins;

    let expanded = expand_xml_self_closing_tags(input);
    write_repaired(parse_with_anchor_origins(&expanded), input.len())
}

/// The repair of [`repair_with_html5ever`], given up on a document that nests deeper than a
/// browser builds (512 open elements), so its cost stays linear in the size of the input.
///
/// ~keep For the second parse that an omitted end tag asks for (issue #772) only. A caller that
/// ~keep made the repair before that issue keeps the repair with no limit: its first parse can
/// ~keep hide content (a table cell with no row), so giving up there loses text and the depth
/// ~keep warning.
pub fn repair_with_html5ever_within_depth(input: &str) -> Option<String> {
    use crate::converter::anchor_origin::parse_with_anchor_origins_within_depth;

    let expanded = expand_xml_self_closing_tags(input);
    write_repaired(parse_with_anchor_origins_within_depth(&expanded)?, input.len())
}

fn write_repaired(dom: crate::rcdom::RcDom, capacity: usize) -> Option<String> {
    use crate::converter::anchor_origin::collapse_split_anchors;
    use crate::rcdom::SerializableHandle;
    use html5ever::serialize::{SerializeOpts, serialize};

    // ~keep The adoption agency splits an `<a>` around a block into an authored element and
    // ~keep clones; collapse the empty halves before the tree is flattened to a string and
    // ~keep the provenance is gone (issue #493).
    collapse_split_anchors(&dom.document);

    let mut buf = Vec::with_capacity(capacity);
    let handle = SerializableHandle::from(dom.document);
    serialize(&mut buf, &handle, SerializeOpts::default()).ok()?;
    String::from_utf8(buf).ok()
}

/// Rebuild a document when head content precedes an authored `<head>` element.
///
/// The HTML tree builder places that content in an implicit head and ignores the later head
/// start tag. The lightweight parser preserves source nesting, so its tree needs this targeted
/// repair before either conversion tier reads metadata or renders the body.
pub fn repair_head_content_before_explicit_head(input: &str) -> Option<String> {
    let head_start = last_start_tag(input.as_bytes(), b"head")?;
    const HEAD_CONTENT: [&[u8]; 11] = [
        b"base",
        b"basefont",
        b"bgsound",
        b"link",
        b"meta",
        b"noframes",
        b"noscript",
        b"script",
        b"style",
        b"template",
        b"title",
    ];
    if !HEAD_CONTENT
        .iter()
        .any(|name| first_start_tag(input.as_bytes(), name).is_some_and(|start| start < head_start))
    {
        return None;
    }

    let dom = tl::parse(input, tl::ParserOptions::default()).ok()?;
    if !document_head_search(dom.children(), dom.parser()).1 {
        return None;
    }
    repair_with_html5ever(input)
}

fn first_start_tag(bytes: &[u8], name: &[u8]) -> Option<usize> {
    bytes
        .iter()
        .enumerate()
        .find_map(|(start, _)| is_start_tag_at(bytes, name, start).then_some(start))
}

fn last_start_tag(bytes: &[u8], name: &[u8]) -> Option<usize> {
    bytes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(start, _)| is_start_tag_at(bytes, name, start).then_some(start))
}

fn is_start_tag_at(bytes: &[u8], name: &[u8], start: usize) -> bool {
    if bytes.get(start) != Some(&b'<') {
        return false;
    }
    let tag_start = start + 1;
    let tag_end = tag_start + name.len();
    bytes
        .get(tag_start..tag_end)
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name))
        && bytes
            .get(tag_end)
            .is_some_and(|next| next.is_ascii_whitespace() || matches!(next, b'/' | b'>'))
}

/// Check if an element is inline (not block-level).
pub fn is_inline_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "a" | "abbr"
            | "b"
            | "bdi"
            | "bdo"
            | "br"
            | "cite"
            | "code"
            | "data"
            | "dfn"
            | "em"
            | "i"
            | "kbd"
            | "mark"
            | "q"
            | "rp"
            | "rt"
            | "ruby"
            | "s"
            | "samp"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "sup"
            | "time"
            | "u"
            | "var"
            | "wbr"
            | "del"
            | "strike"
            | "ins"
            | "img"
            | "map"
            | "area"
            | "audio"
            | "video"
            | "picture"
            | "source"
            | "track"
            | "embed"
            | "object"
            | "param"
            | "input"
            | "label"
            | "button"
            | "select"
            | "textarea"
            | "output"
            | "progress"
            | "meter"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_repair_a_document_of_any_depth_and_give_up_only_within_the_depth_limit() {
        let deep = format!("{}x", "<div>".repeat(600));

        assert!(repair_with_html5ever(&deep).is_some_and(|repaired| repaired.matches("</div>").count() == 600));
        assert!(repair_with_html5ever_within_depth(&deep).is_none());
        assert!(repair_with_html5ever_within_depth("<div><p>one</div>tail").is_some());
    }

    #[test]
    fn test_is_ignorable_before_head() {
        assert!(is_ignorable_before_head("", false));
        assert!(is_ignorable_before_head("   \n\t", false));
        assert!(is_ignorable_before_head("&nbsp;", false));
        assert!(is_ignorable_before_head("&#160;", false));
        assert!(!is_ignorable_before_head("hello", false));
        assert!(!is_ignorable_before_head("x", false));
        assert!(!is_ignorable_before_head("\u{FFFD}\u{FFFD}", false));
        // ~keep Any text sitting directly in front of the document's own `<html>` tag is
        // ~keep forgiven, garbage or genuine prose alike (#527 regression: a mangled byte
        // ~keep order mark reads as ordinary non-whitespace text by this point).
        assert!(is_ignorable_before_head("\u{FFFD}\u{FFFD}", true));
        assert!(is_ignorable_before_head("hello", true));
    }

    fn cleaned(markdown: &str, style: CodeBlockStyle) -> String {
        let mut output = markdown.to_owned();
        trim_line_end_whitespace(&mut output, style);
        collapse_excess_blank_lines(&mut output, style);
        output
    }

    #[test]
    fn should_clean_the_prose_and_not_the_code_of_an_indented_style_document() {
        let cases = [
            // A fence line in indented code opens nothing: the prose after it is cleaned.
            (
                "    a \t\n    ```\n\n\n\n    b\n\n\n\nx \t\n\n\n\ny\n",
                "    a \t\n    ```\n\n\n\n    b\n\nx\n\ny\n",
            ),
            // The third level of a list is indented by four columns and is not code.
            (
                "- a\n  - b\n    - c \t\n\n\n\n    - d \t\n",
                "- a\n  - b\n    - c\n\n    - d\n",
            ),
            // Code in a list item is indented by four columns past the content of the item.
            (
                "1. t\n\n       a \t\n\n\n\n       b\n\n\n\n   u \t\n",
                "1. t\n\n       a \t\n\n\n\n       b\n\n   u\n",
            ),
            // A code block that starts on the line of the item marker.
            ("-     a \t\n\n\n\n      b\n", "-     a \t\n\n\n\n      b\n"),
            // The blank lines between the code blocks of two items belong to neither.
            ("-     a\n\n\n\n-     b\n", "-     a\n\n-     b\n"),
            // Code in a block quote, and prose after the quote.
            (
                ">     a \t\n>\n>\n>\n>     b\n\n\n\nx \t\n",
                ">     a \t\n>\n>\n>\n>     b\n\nx\n",
            ),
            // An indented line that continues a paragraph is not code.
            ("x\n    y \t\n", "x\n    y\n"),
            // A tab is four columns of indentation.
            ("\ta \t\n\n\n\n\tb\n", "\ta \t\n\n\n\n\tb\n"),
        ];
        for (markdown, expected) in cases {
            assert_eq!(cleaned(markdown, CodeBlockStyle::Indented), expected, "{markdown:?}");
        }
    }

    #[test]
    fn should_open_a_block_only_on_the_fence_of_the_configured_style() {
        let markdown = "~~~\na \t\n\n\n\nb\n~~~\n";
        assert_eq!(cleaned(markdown, CodeBlockStyle::Tildes), markdown);
        assert_eq!(cleaned(markdown, CodeBlockStyle::Backticks), "~~~\na\n\nb\n~~~\n");
    }

    #[test]
    fn test_trim_line_end_whitespace() {
        let mut s = String::new();
        trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
        assert_eq!("", s.as_str());

        let mut s = "\t\n\t\n".to_owned();
        trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
        assert_eq!("", s.as_str());

        let mut s = "hello, world  ".to_owned();
        trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
        assert_eq!("hello, world  \n", s.as_str());

        let mut s = "hello, world  \n".to_owned();
        trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
        assert_eq!("hello, world  \n", s.as_str());

        let mut s = "hello, world  ".to_owned();
        trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
        assert_eq!("hello, world  \n", s.as_str());

        let mut s = "hello, world  \n\n\n".to_owned();
        trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
        assert_eq!("hello, world  \n", s.as_str());

        let mut s = "hello  \n- world\n".to_owned();
        trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
        assert_eq!("hello  \n- world\n", s.as_str());

        let mut s = "hello, world\t\t  ".to_owned();
        trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
        assert_eq!("hello, world  \n", s.as_str());

        let mut s = "hello, world\t\t  \n.abc def \t \t".to_owned();
        trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
        assert_eq!("hello, world  \n.abc def\n", s.as_str());
    }

    #[test]
    fn test_is_ascii_whitespace_only() {
        assert!(is_ascii_whitespace_only(""));
        assert!(is_ascii_whitespace_only(" \t\n\r"));
        assert!(is_ascii_whitespace_only("\n\n"));
        // ~keep A decoded `&nbsp;` trims to empty under `str::trim`'s Unicode-aware
        // ~keep definition but must NOT be treated as safe-to-drop formatting whitespace.
        assert!(!is_ascii_whitespace_only("\u{a0}"));
        assert!(!is_ascii_whitespace_only("\n\u{a0}"));
        assert!(!is_ascii_whitespace_only("a"));
        assert!(!is_ascii_whitespace_only(" a "));
    }

    #[test]
    fn test_has_custom_element_tags_ignores_comments() {
        // ~keep Regression for the bug this module fixed: a comment's own text
        // ~keep (`!--c--`) contains a hyphen and must not be mistaken for a tag name.
        assert!(!has_custom_element_tags("<!--c-->"));
        assert!(!has_custom_element_tags("<p><!-- a routine comment --></p>"));
        assert!(!has_custom_element_tags("<!-- multiple -- dashes -- here -->"));
    }

    #[test]
    fn should_detect_namespaced_elements_for_html5_repair() {
        assert!(has_custom_element_tags("<o:p><P></o:p></P>"));
        assert!(!has_custom_element_tags("<o:p><P></P></o:p>"));
        assert!(!has_custom_element_tags("<o:p>Office text</o:p>"));
        assert!(!has_custom_element_tags("<o:p/><P>text</P>"));
        assert!(!has_custom_element_tags("<!-- <o:p>comment</o:p> -->"));
        assert!(!has_custom_element_tags("<div title='o:p'>text</div>"));
    }

    #[test]
    fn test_has_custom_element_tags_detects_real_custom_element() {
        assert!(has_custom_element_tags("<my-widget></my-widget>"));
        assert!(has_custom_element_tags("<p></p><my-widget></my-widget>"));
        assert!(has_custom_element_tags("<div></my-widget>"));
    }

    #[test]
    fn test_has_custom_element_tags_detects_custom_element_alongside_comment() {
        assert!(has_custom_element_tags(
            "<!-- a routine comment --><my-widget>hi</my-widget>"
        ));
        assert!(has_custom_element_tags(
            "<my-widget><!-- nested comment -->hi</my-widget>"
        ));
    }

    #[test]
    fn test_has_custom_element_tags_ignores_comment_mentioning_custom_element() {
        // ~keep A custom-element-shaped tag written INSIDE a comment is text, not markup.
        assert!(!has_custom_element_tags("<!-- <my-widget>hi</my-widget> -->"));
    }

    #[test]
    fn test_has_custom_element_tags_ignores_processing_instruction() {
        assert!(!has_custom_element_tags("<?php echo 'a-b'; ?>"));
        assert!(!has_custom_element_tags(
            "<?xml-stylesheet type=\"text/xsl\" href=\"a-b.xsl\"?>"
        ));
    }

    #[test]
    fn test_has_custom_element_tags_ignores_cdata_section() {
        assert!(!has_custom_element_tags(
            "<svg><![CDATA[<my-widget>a-b</my-widget>]]></svg>"
        ));
    }

    #[test]
    fn test_has_custom_element_tags_ignores_doctype() {
        assert!(!has_custom_element_tags("<!doctype html>"));
        assert!(!has_custom_element_tags("<!DOCTYPE html><p>no custom elements</p>"));
        // ~keep A real custom element after the doctype must still be detected.
        assert!(has_custom_element_tags("<!DOCTYPE html><my-widget></my-widget>"));
    }

    #[test]
    fn test_has_custom_element_tags_rejects_whitespace_before_name() {
        // ~keep HTML5 does not skip whitespace after `<`/`</` before the tag name:
        // ~keep `< div>` and `</ div>` are plain text, not tags, in a real parser.
        assert!(!has_custom_element_tags("< my-widget>"));
        assert!(!has_custom_element_tags("</ my-widget>"));
    }
}
