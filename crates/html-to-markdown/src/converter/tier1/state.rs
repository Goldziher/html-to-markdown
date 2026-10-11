//! Tier-1 scanner state: escape context bitmask and the open-tag stack.

use crate::converter::tier1::tags::TagSpec;

/// Minimum output buffer capacity in bytes.
const OUTPUT_CAPACITY_MIN: usize = 1024;

/// Maximum output buffer capacity in bytes (256 KiB).
const OUTPUT_CAPACITY_MAX: usize = 256 * 1024;

/// Divisor applied to `input_len` to derive the initial output buffer capacity.
const OUTPUT_CAPACITY_DIVISOR: usize = 3;

/// Accumulated state for one `<table>` being assembled by the Tier-1 scanner.
///
/// When a `<table>` opens, a fresh `TableState` is pushed onto
/// `Tier1State::table_stack`.  Cells accumulate text into `current_cell`
/// instead of `Tier1State::output`.  On `</table>` the state is popped and
/// the completed GFM table is appended to the main output.
#[derive(Debug, Clone, Default)]
pub struct TableState {
    /// Completed rows; each cell is `(text, colspan)`.
    ///
    /// Tier-2's `convert_table_cell` (block/table/cell.rs:248) emits the cell
    /// text once then `colspan` trailing ` |` separators.  Storing the colspan
    /// here lets `emit_gfm_table` mirror that exactly while keeping the
    /// per-row vector small (no empty filler entries).
    pub rows: Vec<Vec<(String, u16)>>,
    /// Row currently being assembled (same `(text, colspan)` shape as `rows`).
    pub current_row: Vec<(String, u16)>,
    /// Cell text currently accumulating (active while inside a `<td>`/`<th>`).
    pub current_cell: String,
    /// True while the scanner is inside a `<thead>` section.
    pub in_thead: bool,
    /// True while the scanner is inside a `<td>` or `<th>`.
    pub in_cell: bool,
    /// True while the scanner is accumulating `<caption>` content.
    pub in_caption: bool,
    /// Raw text accumulated during a `<caption>` element.
    pub caption_buf: String,
    /// Trimmed, hyphen-escaped caption text ready for emission.
    ///
    /// `None` if no `<caption>` was seen; `Some("")` if the caption was empty
    /// (Tier-2 emits nothing for an empty caption, so we match that).
    pub caption_text: Option<String>,
    /// True after the first `<tbody>` has closed — used to detect
    /// `<tbody>` → `<tfoot>` → `<tbody>` ordering violations.
    pub seen_tbody_close: bool,
    /// True after a `<tfoot>` open has been seen.
    pub seen_tfoot: bool,
    /// True if at least one `<th>` cell has been seen in this table.
    ///
    /// Tier-2 only uses the GFM table rendering path when `table_has_header`
    /// returns true (i.e. at least one `<th>` exists).  Without a `<th>`,
    /// Tier-2 may render the table as a layout/bulleted-list table depending
    /// on other conditions (link count, row count, etc.).
    pub has_th: bool,
    /// Number of `<a>` (link) elements seen inside the table so far.
    ///
    /// Used to detect the "link-heavy navigation table" pattern that Tier-2
    /// renders as a layout table: `row_count <= 2 && link_count >= 3`.
    pub link_count: usize,
    /// Column count of the first row (used to detect inconsistent column counts
    /// across rows, which triggers Tier-2's layout-table path).
    pub first_row_col_count: Option<usize>,
    /// True when this table is nested inside another table's cell.
    ///
    /// On `</table>`, the rendered GFM markdown is appended to the parent
    /// cell buffer (rather than `state.output`).  The parent cell's later
    /// newline-collapse step then flattens the inner table to a single
    /// inline run, matching Tier-2's behaviour where inner tables emit
    /// full GFM into the cell text and the cell collapses `\n` → space.
    pub inline_mode: bool,
    /// True when at least one nested table has emitted GFM markdown into
    /// this table's current cell.  Set by `close_table` on the parent
    /// frame when popping an `inline_mode = true` child.  Read by
    /// `close_table_cell` to skip the literal-pipe bail — nested-table
    /// rendering legitimately introduces unescaped `|` characters that
    /// Tier-2 also emits without escaping.
    pub had_nested_table: bool,
    /// `colspan` attribute on the currently-open `<td>`/`<th>`.
    ///
    /// Defaults to 1.  On `</td>` / `</th>` close, `close_table_cell` pushes
    /// `(colspan - 1)` additional empty cells onto `current_row` so the row
    /// has the same column count Tier-2 sees after expanding colspan in
    /// `block/table/cells.rs`.  Without this expansion, infobox-style tables
    /// where a header row uses `<th colspan="2">Title</th>` would render as
    /// one column while the rest of the table has two, triggering Tier-2's
    /// layout-table fallback in close_table on what should be a normal GFM
    /// table (e.g. wikipedia/large_rust infobox).
    pub current_cell_colspan: u16,
    /// True while the row currently being assembled (`current_row`) holds exactly one
    /// finalized cell so far and that cell had a nested table flattened into it.
    ///
    /// Cleared as soon as a second cell closes into the same row (a sibling means Tier-2
    /// keeps the existing flatten-and-escape rendering, issue #469). Read by
    /// `close_table_row`, which folds a still-true value at a one-cell row's close into
    /// `had_single_cell_nested_table_row` below.
    pub pending_single_cell_nested_table: bool,
    /// True once any row in this table closed with exactly one cell that held a nested
    /// table (see `pending_single_cell_nested_table`).
    ///
    /// Checked by `close_table` — after its existing one-cell-*wrapper* check, which takes
    /// priority when both apply — rather than bailing immediately in `close_table_row`,
    /// because a row-level decision cannot yet know whether this row turns out to be the
    /// whole table (the wrapper shape, `TableNestedTable`) or one row among several (Tier-2
    /// defers just that row's nested table instead, issue #484): only `close_table` has
    /// seen every row.
    pub had_single_cell_nested_table_row: bool,
    /// True once a definition term or definition opened in the current cell. Tier-2 separates a
    /// definition from one before it in the same cell, so a second one goes to Tier-2.
    pub definition_in_cell: bool,
}

bitflags::bitflags! {
    /// Ambient escape contexts.  Set when we enter a tag that changes escape rules.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct EscapeCtx: u8 {
        /// Inside a `<code>` span.
        const CODE       = 1 << 1;
        /// Inside a `<pre>` block.
        const PRE        = 1 << 2;
        /// Inside an `<a>` link.
        const LINK       = 1 << 3;
        /// Inside a `<blockquote>`.
        const BLOCKQUOTE = 1 << 4;
        /// Inside a heading element.
        const HEADING    = 1 << 5;
        /// Inside a `<strong>`/`<b>` element.
        const STRONG     = 1 << 6;
    }
}

/// One frame on the open-tag stack.
#[derive(Debug, Clone)]
pub struct OpenTag {
    /// Static tag specification for this element.
    pub spec: &'static TagSpec,
    /// Where this tag's content begins in the input buffer (byte index).
    pub content_start: usize,
    /// Snapshot of `escape_ctx` BEFORE this tag set its bits, so we can restore on close.
    pub prev_escape_ctx: EscapeCtx,
    /// For ordered list items: the current item counter (1-based).
    pub list_index: u16,
    /// For ordered lists: the start counter value.
    pub ol_start: u16,
    /// Byte range of the tag name in the original input (original case; callers
    /// must lowercase before comparing).
    pub name_range: std::ops::Range<usize>,
    /// Set (`Strong`/`Emphasis` frames only) when `flush_text` dropped a whitespace-only
    /// text node at this frame's content start, i.e. before any other content was written.
    ///
    /// `close_inline_marker`'s body-emptiness check operates on buffer bytes alone, but by
    /// the time it runs, `flush_text`'s own block-edge whitespace drop (issue #481) has
    /// already erased those bytes — a genuinely empty `<em></em>` and a whitespace-only
    /// `<em> </em>` both look identical (buffer unchanged since `content_start`) from
    /// `close_inline_marker`'s point of view. This flag is the only surviving signal that
    /// distinguishes them, so `close_inline_marker` can bail (`WhitespaceOnlyInlineEmphasis`)
    /// for the latter instead of silently truncating the space away like the former.
    pub dropped_whitespace_only_text: bool,
    /// Whether Tier-2 renders this element's content into a buffer of its own, so that a block
    /// inside it sees only that content (see `scanner::cell_scratch_start`).
    pub own_buffer: bool,
    /// Whether the first content of this quote or heading in a table cell was whitespace, which
    /// Tier-2 keeps at the start of the element's own buffer (see `scanner::flush_text`).
    pub starts_with_whitespace: bool,
}

/// Minimum capacity for each summary accumulation buffer.
const SUMMARY_BUF_CAPACITY: usize = 64;

/// Discriminator for entries in the wrap-buffer stack.
///
/// Both `<summary>` (Phase R) and `<figcaption>` (Phase FF-2) collect
/// children into an accumulation buffer before wrapping with delimiters,
/// but they differ on strong-marker suppression: Tier-2 sets
/// `in_strong: true` for summary children (suppresses nested `**…**`)
/// but uses the default context for figcaption children.  Tag each
/// buffer with its kind so `Tier1State::summary_at_top` can answer the
/// suppression check correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapKind {
    /// `<summary>` strong-wrap buffer.
    Summary,
    /// `<figcaption>` italic-wrap buffer.
    Figcaption,
}

/// Mutable state threaded through the entire Tier-1 scan pass.
pub struct Tier1State {
    /// Open-tag stack; one frame per currently-open element.
    pub stack: Vec<OpenTag>,
    /// Current escape-context bitmask.
    pub escape_ctx: EscapeCtx,
    /// Accumulated Markdown output.
    pub output: String,
    /// Current list nesting depth (0 = top level). Counts both `<ul>` and `<ol>`.
    pub list_depth: u16,
    /// Current unordered-list nesting depth.
    ///
    /// Counts only `<ul>` opens.  Used to cycle through `options.bullets` so
    /// that nested lists produce byte-identical output to Tier-2 (which selects
    /// `bullets[(ul_depth - 1) % bullets.len()]` per item).
    pub ul_depth: u16,
    /// Tracks the output length before the most recent block-open separator was appended,
    /// so we can detect whether any content was actually emitted inside a block.
    pub last_block_sep_pos: usize,
    /// Stack of in-progress table states.  Pushed on `<table>` open, popped on
    /// `</table>` close.  Depth > 1 is a nested-table bail.
    pub table_stack: Vec<TableState>,
    /// Stack of `title` attribute values for currently-open `<abbr>` elements.
    ///
    /// `None` if the element had no (or an empty) title; `Some(t)` if a
    /// non-empty title was present.  On `</abbr>` close, the trimmed title
    /// is emitted as `" (title)"` after the abbreviation text, mirroring
    /// Tier-2's `semantic/attributes.rs::handle_abbr` (line 104-111).
    pub abbr_titles: Vec<Option<String>>,
    /// Stack of `(href, title, has_nested_tag)` triples for currently-open `<a>`
    /// elements.
    ///
    /// HTML5 forbids nested `<a>`, but the stack handles malformed input safely.
    /// Holding link state off `OpenTag` saves two `Option<String>` slots per
    /// every non-link tag frame (24 bytes × every tag on Wikipedia pages with
    /// thousands of tags), and avoids per-frame `Clone` cost.
    ///
    /// `has_nested_tag` starts `false` and is set `true` by the scanner's tag-open
    /// dispatch the moment ANY child tag opens while this link is on the stack
    /// (every entry, not just the innermost, so malformed nested `<a>` still marks
    /// its ancestor). `close_link` reads it to bail (`BailReason::LinkAutolinkNestedMarkup`)
    /// rather than risk a false negative against Tier-2's tag-stripped autolink
    /// predicate — see that bail reason's doc comment.
    ///
    /// The last flag says that the link has a name of its own (`aria-label` or `title`) or
    /// points into its own page. `close_link` reads it to leave such a link with no text to
    /// Tier-2, which labels the first kind and leaves the second kind out.
    pub link_stack: Vec<(Option<String>, Option<String>, bool, bool)>,
    /// Byte range of `<head>…</head>` content (between the tags) in the
    /// input the scanner walked.  Populated by the `TagKind::Ignored`
    /// dispatch when a non-void Ignored tag (`<head>`) is encountered, or
    /// set to an empty range when the body starts before any `<head>`;
    /// `tier1::run` forwards the slice to `head_metadata::extract_frontmatter`
    /// so the YAML frontmatter pass still works without a `PrescanReport`.
    pub head_range: Option<std::ops::Range<usize>>,
    /// Language code extracted from the current `<pre>` or its nested `<code>`
    /// child's `class` attribute (`language-X` or `lang-X`).  Used by
    /// `close_pre` to emit the language tag after the opening backtick fence.
    /// Reset to `None` after each `</pre>` so nested same-level blocks don't
    /// inherit a stale language.
    pub pre_lang: Option<String>,
    /// Byte offsets of real `<br>` nodes inside the current table-cell `<pre>`. ~keep
    pub pre_cell_break_offsets: Vec<usize>,
    /// Stack of `<summary>` and `<figcaption>` accumulation buffers.
    ///
    /// Pushed when a non-cell `<summary>`/`<figcaption>` opens; all child
    /// text accumulates here instead of in `output`.  On close, the buffer
    /// is popped, trimmed, and emitted with the wrap delimiters into the
    /// parent destination (`**…**\n\n` for Summary, `*…*\n\n` for
    /// Figcaption).
    ///
    /// Each entry carries a [`WrapKind`] discriminator so the strong-marker
    /// suppression in scanner.rs can distinguish "currently inside a
    /// summary" from "currently inside a figcaption" — Tier-2 only sets the
    /// `in_strong: true` collection context for summary children.
    ///
    /// A stack (rather than a single `Option`) handles pathological nesting
    /// without panicking.
    pub summary_buf_stack: Vec<(WrapKind, String)>,
    /// `true` immediately after closing a custom element (unknown tag
    /// containing `-`), until the next scanner event consumes or clears it.
    ///
    /// ~keep Tier-2's `is_inline_element` whitespace-merge check
    /// ~keep (`main_helpers.rs`) is a fixed named-tag list that custom
    /// ~keep elements never match, so a whitespace-only text node right
    /// ~keep after a custom element's close is NOT treated as "between two
    /// ~keep inline siblings" and is preserved verbatim even when the
    /// ~keep custom element's own trailing content already ended in a
    /// ~keep (decoded-entity) space. Tier-1's general whitespace flush
    /// ~keep instead asks "does the output already end with a space" and
    /// ~keep skips a redundant one — correct for real inline tags, wrong
    /// ~keep here. This flag lets `flush_text` special-case exactly that
    /// ~keep one boundary instead of relaxing the general dedup rule.
    pub last_closed_custom_element: bool,

    /// True immediately after an `<img>` is emitted, for exactly the next
    /// `flush_text` call (read-then-clear, same convention as
    /// `last_closed_custom_element`). Tier-2's `paragraph.rs` skips a
    /// whitespace-only text node entirely — not just deduping it to a single
    /// space — when it sits directly between two "empty inline" siblings
    /// (`br`/`hr`/`img`/`input`/`meta`/`link`) that are BOTH direct children
    /// of the same `<p>`; this flag lets `flush_text` detect the "previous
    /// direct sibling was an `<img>`" half of that condition (the "next
    /// sibling is an `<img>`" half comes from the scan loop's tag peek, same
    /// as `next_tag_is_list`).
    pub last_emitted_was_img: bool,

    /// True right after the closing tag of a block element or a `<hr>` (block-level by Tier-2's
    /// test), until the next opening or void tag, the next closing tag that is not inline, or the
    /// next text flush with content (read-then-clear, same convention as
    /// `last_closed_custom_element`). Mirrors Tier-2's check in `walk_node` that inline content
    /// after a block, or after an inline element ending in one, starts a new paragraph (issues
    /// #570, #571, #585).
    pub last_closed_block: bool,
    /// The length of the output right after a text node's trailing source newline was written
    /// as a `'\n'` join. Cleared by an opening tag other than `<br>`, `<script>` or `<style>`,
    /// and by the closing tag of a block or of a form element that Tier-2 writes on a line of its
    /// own; any other output leaves it behind the end. An inline closing tag that leaves the join
    /// last moves it to the new end. A `<br>` that finds the output still that long removes the
    /// join: the break ends the text's line, as in Tier-2 (issue #683). Deciding at the `<br>`
    /// rather than looking ahead from the text reads each byte of the markup between them once.
    pub pending_newline_join: Option<usize>,
    /// The end of the text of the last ordered list closed outside a table cell: an ordered list
    /// that opens after only whitespace there continues it.
    pub last_ordered_list_end: Option<usize>,

    /// Byte width of each currently-open list item's own marker (`"- "` = 2,
    /// `"1. "` = 3, `"10. "` = 4, ...), one entry per open `<li>` frame,
    /// pushed by `open_list_item` and popped by `close_list_item`.
    ///
    /// A block child (`<pre>`, `<blockquote>`, a text sibling after a
    /// heading, ...) of a list item must indent every physical line to the
    /// item's continuation column, per CommonMark's per-line list-container
    /// matching. That column is the SUM of every ancestor `<li>`'s own
    /// marker width — not a uniform `2 * depth` — because an ordered-list
    /// marker's width varies with its digit count (`"1. "` vs `"10. "`).
    /// Summing this stack gives that column directly; see
    /// `Tier1State::list_continuation_indent_width`.
    pub list_item_marker_widths: Vec<usize>,

    /// Whether each currently-open list item's marker line follows text inside its list on the
    /// line above, one entry per open `<li>` frame, pushed and popped with
    /// `list_item_marker_widths`.
    pub list_items_after_text: Vec<bool>,

    /// `true` until the first text node carrying real (non-whitespace)
    /// content has been processed anywhere in the document, or an image or
    /// a graphic has written content (`end_document_start_if_written`), then
    /// permanently `false`.
    ///
    /// Mirrors Tier-2's `Context::at_fresh_block_start` (an
    /// `Rc<Cell<bool>>` shared across the whole conversion): CommonMark
    /// 4.8 makes leading whitespace at the very start of a document
    /// insignificant, so `flush_text` strips it there the same way it
    /// already strips leading whitespace after a block separator.
    /// Deliberately NOT derived from buffer emptiness — an inline
    /// wrapper (`<sub>`, `<em>`, a link) accumulates into a fresh local
    /// buffer via `cell_or_output_mut`, so an empty buffer there means
    /// "this wrapper's own scratch space is empty", not "we are at
    /// document start". A dedicated flag distinguishes the two.
    pub at_document_start: bool,

    /// Effective base URL for resolving relative `href`/`src` destinations, computed
    /// once in `convert_api.rs` and shared with the Tier-2 path's `Context::base_url`
    /// so both tiers resolve the same input identically. `None` when `options.base_url`
    /// is unset (the default), making every resolution call a no-op.
    pub effective_base: Option<std::rc::Rc<url::Url>>,

    /// The address of the page, as far as it is known; read when the first link opens. The
    /// same value Tier-2 holds in `Context::own_page`.
    pub own_page: std::cell::OnceCell<crate::converter::url_resolve::OwnPage>,
}

impl Tier1State {
    /// Create a new `Tier1State` pre-allocating output capacity based on `input_len`.
    #[must_use]
    pub fn new(input_len: usize, effective_base: Option<std::rc::Rc<url::Url>>) -> Self {
        Self {
            stack: Vec::with_capacity(16),
            escape_ctx: EscapeCtx::empty(),
            output: String::with_capacity(
                (input_len / OUTPUT_CAPACITY_DIVISOR).clamp(OUTPUT_CAPACITY_MIN, OUTPUT_CAPACITY_MAX),
            ),
            list_depth: 0,
            ul_depth: 0,
            last_block_sep_pos: 0,
            table_stack: Vec::new(),
            link_stack: Vec::new(),
            abbr_titles: Vec::new(),
            head_range: None,
            pre_lang: None,
            pre_cell_break_offsets: Vec::new(),
            summary_buf_stack: Vec::new(),
            last_closed_custom_element: false,
            last_emitted_was_img: false,
            last_closed_block: false,
            pending_newline_join: None,
            last_ordered_list_end: None,
            list_item_marker_widths: Vec::new(),
            list_items_after_text: Vec::new(),
            at_document_start: true,
            effective_base,
            own_page: std::cell::OnceCell::new(),
        }
    }

    /// Resolve a `href`/`src` attribute value against [`Self::effective_base`].
    ///
    /// Returns `None` (meaning: use the original text unchanged) whenever
    /// `effective_base` is unset, `value` is empty, already absolute, or fails to
    /// resolve. See `converter::url_resolve::resolve_attribute_url` for the full
    /// contract. Mirrors `Context::resolve_url` on the Tier-2 path.
    #[must_use]
    pub fn resolve_url(&self, value: &str) -> Option<String> {
        crate::converter::url_resolve::resolve_attribute_url(self.effective_base.as_deref()?, value)
    }

    /// The body starts at `pos`: the head is over, empty if no `<head>` came before, and the
    /// parser ignores a later `<head>` tag, as Tier 2's head walk does.
    pub const fn start_body(&mut self, pos: usize) {
        if self.head_range.is_none() {
            self.head_range = Some(pos..pos);
        }
    }

    /// Total continuation-indent width (in columns) for a block child of the
    /// innermost currently-open list item, or `0` when not inside one.
    ///
    /// See `list_item_marker_widths`'s doc comment for why this is a sum of
    /// real marker widths rather than a uniform `2 * depth`.
    #[must_use]
    pub fn list_continuation_indent_width(&self) -> usize {
        self.list_item_marker_widths.iter().sum()
    }

    /// Return a mutable reference to the current accumulation target.
    ///
    /// Priority order (highest first):
    /// 1. Summary buffer top — when a `<summary>` accumulation buffer is active,
    ///    all text (including text from inside table cells) accumulates here.
    ///    This mirrors Tier-2's behaviour where `handle_summary` processes
    ///    children into a local `content` buffer regardless of outer context.
    /// 2. Table cell — when inside a `<td>`/`<th>`, text accumulates in the
    ///    cell buffer (only when not already inside a summary).
    /// 3. Table caption — when inside a `<caption>`, text accumulates in the
    ///    caption buffer.
    /// 4. `self.output` — the main output buffer (default).
    ///
    /// This is the single dispatch point for "where does inline text land."
    pub fn cell_or_output_mut(&mut self) -> &mut String {
        if let Some((_, buf)) = self.summary_buf_stack.last_mut() {
            return buf;
        }
        if let Some(ts) = self.table_stack.last_mut() {
            if ts.in_cell {
                return &mut ts.current_cell;
            }
            if ts.in_caption {
                return &mut ts.caption_buf;
            }
        }
        &mut self.output
    }

    /// Ends the start of the document when an element wrote content of its own (an image, a
    /// graphic) from `written_from` on in the buffer that inline text lands in. Mirrors the
    /// check that Tier-2's `convert_node` does for every node (issue #762).
    pub fn end_document_start_if_written(&mut self, written_from: usize) {
        if !self.at_document_start {
            return;
        }
        let wrote_content = self
            .cell_or_output_mut()
            .get(written_from..)
            .is_some_and(|written| !written.trim().is_empty());
        if wrote_content {
            self.at_document_start = false;
        }
    }

    /// True when the scanner is currently accumulating `<summary>` or
    /// `<figcaption>` content (any wrap-buffer is on the stack).
    ///
    /// Whitespace and text-normalization paths use this — both wrap kinds
    /// share the same collection-mode semantics.
    #[must_use]
    pub const fn in_summary(&self) -> bool {
        !self.summary_buf_stack.is_empty()
    }

    /// True when the topmost wrap-buffer is specifically a `<summary>`
    /// (Phase R), as opposed to a `<figcaption>` (Phase FF-2).
    ///
    /// Used to suppress nested `**…**` strong markers, since Tier-2 only
    /// sets `in_strong: true` for summary children.
    #[must_use]
    pub fn summary_at_top(&self) -> bool {
        matches!(self.summary_buf_stack.last(), Some((WrapKind::Summary, _)))
    }

    /// Push a fresh wrap accumulation buffer onto the stack, tagged with
    /// the given [`WrapKind`].
    pub fn push_summary_buf(&mut self, kind: WrapKind) {
        self.summary_buf_stack
            .push((kind, String::with_capacity(SUMMARY_BUF_CAPACITY)));
    }

    /// Pop the top wrap accumulation buffer and return it (kind discarded).
    pub fn pop_summary_buf(&mut self) -> Option<String> {
        self.summary_buf_stack.pop().map(|(_, buf)| buf)
    }

    /// True when the scanner is currently accumulating `<caption>` content.
    #[must_use]
    pub fn in_table_caption(&self) -> bool {
        self.table_stack.last().is_some_and(|ts| ts.in_caption)
    }

    /// True when the scanner is currently inside a table cell.
    #[must_use]
    pub fn in_table_cell(&self) -> bool {
        self.table_stack.last().is_some_and(|ts| ts.in_cell)
    }

    /// True when ANY frame on the table stack has `in_cell = true`.
    ///
    /// With nested tables (Phase HH), the inner table's frame may have
    /// `in_cell = false` while an outer frame still has `in_cell = true`
    /// — for example between `<td>` siblings of the inner table while the
    /// outer cell that wraps it is still open.  Use this for inter-cell
    /// whitespace guards that must drop text outside *any* active cell.
    #[must_use]
    pub fn in_any_table_cell(&self) -> bool {
        self.table_stack.iter().any(|ts| ts.in_cell)
    }

    /// Ensure the output ends with exactly two newlines (blank-line separator).
    /// If the output is empty, do nothing.
    ///
    /// Trailing ASCII spaces / tabs (introduced by the inter-tag whitespace
    /// preservation in `flush_text`) are trimmed before the separator is
    /// appended.  Without this trim, `<span>foo</span> <div>bar</div>` would
    /// emit `foo \n\nbar` with a stray trailing space — a regression flagged
    /// during Phase U development.  Trimming here makes "preserve a space
    /// optimistically, drop on block boundary" safe.
    pub fn ensure_blank_line(&mut self) {
        let out = &mut self.output;
        if out.is_empty() {
            return;
        }
        while out.ends_with(' ') || out.ends_with('\t') {
            out.pop();
        }
        if out.ends_with("\n\n") {
            return;
        }
        if out.ends_with('\n') {
            out.push('\n');
        } else if out.is_empty() {
        } else {
            out.push_str("\n\n");
        }
    }

    /// Ensure the output ends with at least one newline.
    pub fn ensure_newline(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }
}

/// Trim trailing ASCII horizontal whitespace (spaces and tabs) from a string
/// buffer.  Used before emitting block separators to drop the optimistic
/// inter-tag space pushed by `flush_text` (Phase U-2).
pub fn trim_trailing_horizontal(buf: &mut String) {
    while buf.ends_with(' ') || buf.ends_with('\t') {
        buf.pop();
    }
}
