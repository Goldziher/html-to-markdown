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
    document_head, end_tag_starts_body, extract_head_metadata, format_metadata_frontmatter, is_ignorable_before_head,
    starts_body,
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
/// self-closing syntax so that `repair_with_html5ever_bounded` can parse them correctly.
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
/// `repair_with_html5ever_bounded` re-parses content that contains custom / namespaced tags
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
///
/// ~keep The tree builder has a node budget and a depth limit on every route (issues #808 and
/// ~keep #809), and the result names the limit that it reached. At the node budget the result
/// ~keep is `input` as written. At the depth limit the tree ends at the tag that reached it,
/// ~keep and the input after that tag follows the tree as written. No word is lost.
pub(super) fn repair_with_html5ever_bounded(input: &str) -> Option<(String, Option<crate::types::WarningKind>)> {
    use crate::converter::anchor_origin::{RepairTree, collapse_split_anchors, parse_with_anchor_origins};
    use crate::rcdom::SerializableHandle;
    use html5ever::serialize::{SerializeOpts, serialize};

    let expanded = expand_xml_self_closing_tags(input);
    let RepairTree { dom, limit, unparsed } = parse_with_anchor_origins(&expanded);
    if limit == Some(crate::types::WarningKind::TruncatedInput) {
        tracing::warn!(
            target: "html_to_markdown::convert",
            "HTML tree repair reached its node budget; continuing with the original structure"
        );
        // ~keep Abandon the amplified repair and keep the authored text (issue #809).
        return Some((input.to_owned(), limit));
    }
    // ~keep The adoption agency splits an `<a>` around a block into an authored element and
    // ~keep clones; collapse the empty halves before the tree is flattened to a string and
    // ~keep the provenance is gone (issue #493).
    collapse_split_anchors(&dom.document);

    let mut buf = Vec::with_capacity(input.len());
    let handle = SerializableHandle::from(dom.document);
    serialize(&mut buf, &handle, SerializeOpts::default()).ok()?;
    let mut html = String::from_utf8(buf).ok()?;
    if limit.is_some() {
        tracing::warn!(
            target: "html_to_markdown::convert",
            "HTML tree repair reached its depth limit; the rest of the input keeps its original structure"
        );
        // ~keep The input after the last tag of the tree goes into the body as written (issue
        // ~keep #808). The tree has a `<body>` end tag only when the input wrote a `<body>`.
        const DOCUMENT_END: &str = "</body></html>";
        let end = if html.ends_with(DOCUMENT_END) {
            DOCUMENT_END.len()
        } else {
            0
        };
        html.insert_str(html.len() - end, &unparsed);
    }
    Some((html, limit))
}

/// Rebuild a document when head content precedes an authored `<head>` element, or when a
/// `<title>` precedes the body and is in no `<head>` element: the page has no `<head>` tag, or
/// the title follows `</head>`.
///
/// The HTML tree builder places that content in an implicit head and ignores a later head
/// start tag. The lightweight parser preserves source nesting, so its tree needs this targeted
/// repair before either conversion tier reads metadata or renders the body.
pub fn repair_head_content_outside_head(input: &str) -> Option<(String, Option<crate::types::WarningKind>)> {
    if !may_hold_head_content_outside_head(input.as_bytes()) {
        return None;
    }
    let dom = tl::parse(input, tl::ParserOptions::default()).ok()?;
    if !document_head_search(dom.children(), dom.parser()).content_outside_head {
        return None;
    }
    repair_with_html5ever_bounded(input)
}

/// A cheap check of the source in front of the parse of [`repair_head_content_outside_head`];
/// the parsed tree decides.
///
/// ~keep The check reads bytes, not tags. A `<head>` or a `<body>` inside a comment makes it
/// ~keep false for a page that the parsed tree would rebuild: that page keeps its title in the
/// ~keep body.
fn may_hold_head_content_outside_head(bytes: &[u8]) -> bool {
    let Some(head_start) = last_start_tag(bytes, b"head") else {
        return first_start_tag(bytes, b"title").is_some();
    };
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
    HEAD_CONTENT
        .iter()
        .any(|name| first_start_tag(&bytes[..head_start], name).is_some())
        || title_follows_head_end(bytes)
}

/// True when a `<title>` tag follows the first `</head>` with no `<body>` tag between them.
fn title_follows_head_end(bytes: &[u8]) -> bool {
    // ~keep `/head` as a name: the end tag has the shape of a start tag with that name.
    let Some(head_end) = first_start_tag(bytes, b"/head") else {
        return false;
    };
    let rest = &bytes[head_end..];
    let Some(title) = first_start_tag(rest, b"title") else {
        return false;
    };
    first_start_tag(rest, b"body").is_none_or(|body| title < body)
}

fn first_start_tag(bytes: &[u8], name: &[u8]) -> Option<usize> {
    memchr::memchr_iter(b'<', bytes).find(|&start| is_start_tag_at(bytes, name, start))
}

fn last_start_tag(bytes: &[u8], name: &[u8]) -> Option<usize> {
    memchr::memchr_iter(b'<', bytes)
        .rev()
        .find(|&start| is_start_tag_at(bytes, name, start))
}

#[cfg(test)]
thread_local! {
    static START_TAG_CANDIDATE_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn is_start_tag_at(bytes: &[u8], name: &[u8], start: usize) -> bool {
    #[cfg(test)]
    START_TAG_CANDIDATE_CHECKS.with(|checks| checks.set(checks.get() + 1));
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

/// Build the warning for a repair that stopped at `limit`.
///
/// `limit` is the reason the tree builder reports: `TruncatedInput` for the node budget (the
/// first parse is kept), `DepthLimitExceeded` for the depth limit (the first parse is kept
/// too). No text is lost in either case, so the warning kind is `MalformedHtml` for both.
pub(super) fn repair_limit_warning(limit: crate::types::WarningKind) -> crate::types::ProcessingWarning {
    use crate::converter::anchor_origin::MAX_REPAIR_DEPTH;
    crate::types::ProcessingWarning {
        kind: crate::types::WarningKind::MalformedHtml,
        message: if limit == crate::types::WarningKind::TruncatedInput {
            "HTML tree repair exceeded its node budget; continued with the original structure.".to_owned()
        } else {
            format!(
                "HTML tree repair reached the depth limit of {MAX_REPAIR_DEPTH}; continued with the original structure."
            )
        },
    }
}

#[cfg(test)]
mod tests;
