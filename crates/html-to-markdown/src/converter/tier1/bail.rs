//! Bail reasons emitted by the Tier-1 scanner.
//!
//! When the scanner encounters a condition it cannot handle correctly it returns
//! one of these variants. The dispatcher in `convert_api.rs::convert_inner` catches the
//! error, logs it at `tracing::warn!` (a fallback was taken), and falls back to the
//! Tier-2 path with the original (pre-prescan) input.

use std::fmt;

/// Reasons the Tier-1 scanner may bail out and hand off to Tier-2.
#[derive(Debug, Clone)]
pub enum BailReason {
    /// The classifier decided Tier-2 is required for this input / option set.
    Classifier,

    /// An open-tag stack mismatch was detected mid-stream.
    DepthMismatch {
        /// Tag name where the mismatch was detected.
        tag: String,
        /// Depth expected by the scanner's stack.
        expected: u8,
        /// Depth seen in the source.
        actual: u8,
    },

    /// Reached end-of-file with one or more unclosed block elements.
    EofWithOpenBlock {
        /// Number of unclosed block elements at EOF.
        open_count: usize,
    },

    /// A literal `<` (not a valid tag open) was encountered in the stream.
    LiteralLt {
        /// Byte offset of the `<` in the input.
        offset: usize,
    },

    /// A CDATA section was encountered.
    Cdata {
        /// Byte offset of `<![CDATA[` in the input.
        offset: usize,
    },

    /// An unknown custom element (tag name containing `-`) was encountered.
    UnknownCustomElement {
        /// The element name.
        name: Box<str>,
        /// Byte offset of the `<` in the input.
        offset: usize,
    },

    /// Two `<script>`/`<style>` (raw-text-ignored) elements were found directly
    /// adjacent, with no separating whitespace.  Tier-2's script/style-stripping
    /// preprocessing pass collapses such pairs into a single whitespace-only DOM
    /// text node whose downstream handling produces a byte pattern Tier-1 does
    /// not replicate; bail so Tier-2 (authoritative) handles it.
    AdjacentRawTextTags {
        /// Byte offset of the second element's `<` in the input.
        offset: usize,
    },

    // ~keep ── Table-specific bail reasons ───────────────────────────────────────────
    /// A `<td>` or `<th>` had a `rowspan` or `colspan` attribute with a value
    /// other than 1 (absent attribute counts as 1).
    TableRowspanColspan,

    /// A block-level element was opened inside a `<td>` or `<th>` (e.g.
    /// `<td><p>text</p></td>`).  Tier-1 only supports inline cell content.
    TableBlockChildInCell,

    /// A nested `<table>` was opened while a table is already being assembled
    /// (i.e. `table_stack` is non-empty).
    TableNestedTable,

    /// A `<tr>` whose only cell holds a nested `<table>` closed inside an outer table
    /// that is not itself a one-cell wrapper (i.e. `TableNestedTable` did not already
    /// cover it — the outer table has other rows and/or a `<th>` somewhere).
    ///
    /// Tier-2's `block/table/builder::handle_table` defers that nested table, rendering
    /// it as its own separate GFM table immediately after the enclosing one, rather than
    /// flattening it into a line of escaped pipes (issue #484). Reproducing "render this
    /// row's cell now, but hold its nested table's markdown until the whole outer table
    /// has finished" needs the same kind of end-of-table deferred buffer Tier-2 uses;
    /// this single-pass scanner has no equivalent buffering. Bail so Tier-2
    /// (authoritative) handles it.
    TableNestedTableInSingleCellRow,

    /// A `<caption>` element was encountered inside a table.
    TableCaption,

    /// Table sections appear in an unsupported order, e.g. `<tbody>` after
    /// `<tfoot>` close, or `<thead>` after any section that already closed.
    TableSectionOrder,

    /// Open-tag nesting reached the effective depth limit
    /// (`crate::converter::main_helpers::effective_max_depth`).
    ///
    /// The scanner's `state.stack` is an explicit `Vec`, not native recursion, so
    /// it has no stack-overflow risk of its own — but Tier-2's recursive
    /// `walk_node` silently truncates (skips deeper nodes and their content)
    /// once `depth >= effective_max_depth`. Continuing the scan past that same
    /// depth would produce the *untruncated* output, diverging from Tier-2's
    /// authoritative truncated output. Bail so Tier-2 (which truncates) wins.
    DepthLimitExceeded {
        /// Nesting depth (open-tag count) at which the limit was reached.
        depth: usize,
        /// The effective limit that was exceeded.
        max_depth: usize,
    },

    /// A character reference Tier-1 does not decode but Tier-2 does: a legacy
    /// named or a numeric reference without its `;` (e.g. `&copy 2024`, `&#39s`).
    ///
    /// Tier-1 would pass the reference through verbatim, but Tier-2 decodes it to
    /// the correct character, so the outputs would diverge.  Bail so the
    /// dispatcher falls back to Tier-2.
    ///
    /// The one call site that constructs this variant reaches it only after
    /// [`crate::text::decode_character_reference`] has already matched `name`, so
    /// the reference is always a known one missing its `;`, never a truly unknown
    /// name. [`fmt::Display`] still checks `name` itself before choosing the
    /// wording, rather than trusting that invariant, so a name that is not
    /// actually a recognized reference still reads as unknown.
    UnknownEntity {
        /// The reference after the `&` (e.g. `"copy"`, `"#39"`).
        name: Box<str>,
        /// Byte offset in the HTML input where the `&` was found.
        offset: usize,
    },

    /// An opening tag carries the `hidden` attribute, or an inline `style`
    /// declaration that hides the element (`display: none` / `visibility:
    /// hidden` / `font-size: 0`).
    ///
    /// `converter::utility::preprocessing::strip_hidden_elements` (outside
    /// tier1/) removes such elements — tag and all descendant content —
    /// before Tier-2 ever parses the document. Tier-1 has no equivalent pass
    /// and would otherwise emit the hidden element's content verbatim. Bail so
    /// Tier-2 (which already strips it) is authoritative.
    ///
    /// `font-size: 0` is the one conditional case: Tier-2 keeps the subtree when
    /// a descendant re-declares a non-zero size (issue #468). Tier-1's check is a
    /// single-tag test with no subtree awareness, so it bails on those too — the
    /// conservative direction, since the bail is what hands the document to the
    /// tier that can see the descendant.
    HiddenElement {
        /// Byte offset of the element's `<` in the input.
        offset: usize,
    },

    /// A list (`<ul>`/`<ol>`) was opened while already nested inside another
    /// list, where either the new list or an ancestor list is `<ol>`.
    ///
    /// A nested list's indent must equal the cumulative width of every
    /// ancestor marker (`"- "` = 2, `"1. "` = 3, `"10. "` = 4, ...). Tier-1's
    /// `push_list_item_indent` hardcodes a uniform 2-space-per-depth scheme,
    /// which only holds when every list in the ancestor chain is unordered.
    /// Bail so Tier-2 (which computes cumulative marker widths) is authoritative.
    ListNestedOrdered,

    /// An ordered list opened after only whitespace following an ordered list.
    ///
    /// Tier-2 writes it with the `)` delimiter so the two lists do not merge into one (issue #666).
    /// This scanner writes `.` only.
    OrderedListAfterOrderedList,

    /// A list item of a nested list, or one after text inside its list, closed with nothing after
    /// its marker on the marker line.
    ///
    /// Such a line cannot interrupt a paragraph, so Tier-2 writes a blank line before it when it
    /// follows text (issue #667). This scanner does not track open paragraphs.
    EmptyNestedListItem,

    /// A `<blockquote>`, `<div>` (or other generic block container), `<table>`,
    /// `<dl>`, or a paragraph-continuation `<p>` opened while inside an open
    /// list item, in a shape this scanner cannot render correctly.
    ///
    /// Tier-2 gives each of these kinds bespoke separator, indentation, and/or
    /// loose-vs-tight handling once `ctx.in_list_item` is true — a wider
    /// continuation indent for `<table>`, a same-line space-glued continuation
    /// for a `<p>` that continues the item's already-started text, an
    /// unindented description line for `<dl>`, and (for `<blockquote>`/`<div>`)
    /// a continuation separator that is a single newline, never a blank line.
    /// This scanner's generic block open/close helpers do not replicate any of
    /// that. Worse, a correct fix for the `<blockquote>`/`<div>` case would also
    /// have to rework `close_list_item`'s loose-list heuristic (which detects a
    /// "loose" item by scanning its content for a literal `"\n\n"`): Tier-2's
    /// correct continuation separator for those two never contains a blank
    /// line, silently defeating the very signal that heuristic relies on. That
    /// is exactly the kind of block-separator-deferral restructuring this
    /// scanner does not do — bail instead of guessing.
    ///
    /// `<p>` and `<pre>` are exempt in the one shape each already gets right
    /// without any of the above: as the item's own FIRST content, sitting
    /// directly after the bare bullet marker with nothing else on that line
    /// yet (Phase EE's existing inline-join for `<p>`; `<pre>`'s
    /// list-item-agnostic `ensure_blank_line` already coincides with Tier-2
    /// there). Only `<p>` opening as a CONTINUATION of already-started text
    /// bails.
    ///
    /// A heading or any other generic block container anywhere in a list item, an
    /// `<hr>` after the item's content, and inline content right after a block in a
    /// list item, bail too: Tier-2 starts each at the item's content column (issue
    /// #583), which this scanner does not track.
    ListItemUnsupportedBlockChild,

    /// A checkbox `<input>` opened inside a list item. Tier-2 writes an item that starts with a
    /// checkbox as a task item (`- [ ]`), which this scanner does not.
    ListItemCheckbox,

    /// An `<img>` had an empty (or whitespace-only) `src`, or a `src` that is a
    /// `data:` URI, while also carrying one of the lazy-load fallback attributes
    /// (`data-src`, `data-lazy-src`, `data-original`, `data-srcset`, `srcset`).
    ///
    /// Tier-2's `handle_img` (`handlers/image.rs`'s `resolve_effective_src`)
    /// resolves the real image URL from those attributes in that precedence
    /// order — including `pick_best_srcset_candidate`'s width/density-descriptor
    /// comparison for `srcset`/`data-srcset` — whenever `src` looks like a
    /// lazy-load placeholder. Reproducing that precedence and the `srcset`
    /// descriptor grammar byte-for-byte in a single-pass scanner duplicates
    /// non-trivial logic that would then have to stay in sync forever, for a
    /// shape that is not on any hot path. Bail so Tier-2 (authoritative) wins.
    ImageLazyLoadSrc,

    /// A `<blockquote>` carries a non-empty `cite` attribute.
    ///
    /// Tier-2's `handle_blockquote` emits the citation as a trailing
    /// `\u{2014} <url>` line AFTER the quoted content, resolved against
    /// `base_url`. Tier-1's `open_blockquote` reads no attributes at all, so it
    /// dropped the citation outright — the two tiers rendered the same markup
    /// differently, which every other bail in this file exists to prevent.
    ///
    /// Reproducing it here means stashing the cite across the whole quoted body
    /// and re-deriving Tier-2's separator, and `close_blockquote` computes that
    /// separator from the untouched pre-open tail of `output` (see its own
    /// note) — delicate enough that a second implementation would drift. A
    /// cited blockquote is rare and on no hot path, so bail and let Tier-2,
    /// which is authoritative, produce it.
    BlockquoteCite,

    /// A `<a>` element's href satisfies every structural autolink precondition
    /// (`options.autolinks`, `!options.default_title`, a non-empty href with a
    /// URI scheme) while at least one child tag opened inside the link before
    /// it closed.
    ///
    /// Tier-2's `is_autolink` predicate compares `href` against the label's
    /// TAG-STRIPPED text content (`get_text_content`/`collect_link_label_text`
    /// walk past inline tags and concatenate only their text), so a nested
    /// `<b>`/`<code>`/etc. is invisible to it. Tier-1's label is the RENDERED
    /// buffer (e.g. `**https://x.com**`), which can never equal the bare href
    /// even when Tier-2's stripped text would — so a literal buffer comparison
    /// can only produce a false negative here, never a false positive.
    /// Reproducing Tier-2's tag-stripping inside a single-pass scanner
    /// duplicates non-trivial logic that would then have to stay in sync
    /// forever; bail instead so Tier-2 (authoritative) decides.
    LinkAutolinkNestedMarkup,

    /// A `<strong>`/`<b>` or `<em>`/`<i>` opened while the output already ends with that
    /// same tag kind's matching close marker (`**` or a lone `*`), with no intervening
    /// content.
    ///
    /// `CommonMark` parses a closing delimiter run immediately followed by an opening one
    /// of the same character as a single, longer delimiter run, not as two independent
    /// emphasis spans -- so `<i>A</i><i>B</i>` reparses as `A**B` inside one `*…*` pair
    /// rather than as `A` and `B` each independently emphasized (issue #483). Tier-2 fixes
    /// this by merging the second element's open marker into the first's close marker
    /// (`merge_adjacent_emphasis`); Tier-1 does not replicate that merge, so it bails and
    /// lets Tier-2 (authoritative) handle it.
    AdjacentInlineEmphasis,

    /// A `<strong>`/`<b>` or `<em>`/`<i>` closed with a body that was non-empty but
    /// entirely whitespace (e.g. `<i> </i>`).
    ///
    /// Tier-2's `chomp_inline` folds such a body into a single space placed OUTSIDE the
    /// markers (issue #481). Tier-1's `close_inline_marker` instead erases the markers
    /// entirely for any whitespace-only body, which is only correct for a GENUINELY empty
    /// body (no bytes at all between the markers, e.g. `<i></i>`) -- for a whitespace-only
    /// one it silently drops the space Tier-2 preserves. Bail so Tier-2 (authoritative)
    /// handles it.
    WhitespaceOnlyInlineEmphasis,

    /// An inline element opened whose Tier-2 markers Tier-1 does not emit at all.
    ///
    /// `<q>` is wrapped in `"…"` by `semantic::attributes::handle_q`, and `<mark>` in the pair
    /// `options.highlight_style` selects. Tier-1 treats both as transparent inline elements, so
    /// it would drop those markers outright. The router already routes any non-`None`
    /// `highlight_style` to Tier-2, which makes the `<mark>` case unreachable under
    /// `TierStrategy::Auto` -- it is still checked here so the Tier-1/Tier-2 byte-equality
    /// contract holds under a forced Tier-1 run rather than resting on that gate.
    InlineMarkerNotReproduced,

    /// A form control whose text Tier-2 separates from the text around it: `<select>`,
    /// `<option>`, `<optgroup>`, `<datalist>`, and a `<button>`, `<output>`, `<meter>` or `<progress>` in a
    /// heading, a summary, a caption, a link, an inline element or code. The space after such a control depends
    /// on the text that follows it, which this scanner has not read yet.
    FormControl,

    /// An `<hr>` opened between inline markers: inside `<strong>`/`<b>`, `<em>`/`<i>`,
    /// `<var>`/`<dfn>`, `<del>`, `<ins>`, a `<summary>`, a `<figcaption>` or a table caption.
    ///
    /// Tier-2 writes such a rule as the text `---` in the running line, because on a line of
    /// its own it would end the paragraph between the markers (issue #603). Tier-1 writes it
    /// as a block, so it bails and lets Tier-2 (authoritative) handle it.
    RuleBetweenInlineMarkers,
}

impl fmt::Display for BailReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Classifier => write!(f, "classifier forced tier-2"),
            Self::DepthMismatch { tag, expected, actual } => {
                write!(
                    f,
                    "depth mismatch for </{tag}>: expected {expected} open(s), got {actual}"
                )
            }
            Self::EofWithOpenBlock { open_count } => write!(f, "EOF with {open_count} unclosed block element(s)"),
            Self::LiteralLt { offset } => write!(f, "literal '<' at byte offset {offset}"),
            Self::Cdata { offset } => write!(f, "CDATA section at byte offset {offset}"),
            Self::UnknownCustomElement { name, offset } => {
                write!(f, "unknown custom element <{name}> at byte offset {offset}")
            }
            Self::AdjacentRawTextTags { offset } => {
                write!(
                    f,
                    "adjacent <script>/<style> tags with no separating whitespace at byte offset {offset}"
                )
            }
            Self::TableRowspanColspan => write!(f, "table cell has rowspan or colspan != 1"),
            Self::TableBlockChildInCell => write!(f, "block-level element inside table cell"),
            Self::TableNestedTable => write!(f, "nested <table> inside a table cell"),
            Self::TableNestedTableInSingleCellRow => write!(f, "nested <table> inside a data table's single-cell row"),
            Self::TableCaption => write!(f, "<caption> element in table"),
            Self::TableSectionOrder => write!(f, "table sections in unsupported order"),
            Self::UnknownEntity { name, offset } => {
                if is_known_reference_name(name) {
                    write!(
                        f,
                        "HTML entity &{name} is missing its closing semicolon at byte offset {offset}"
                    )
                } else {
                    write!(f, "unknown HTML entity &{name} at byte offset {offset}")
                }
            }
            Self::DepthLimitExceeded { depth, max_depth } => {
                write!(
                    f,
                    "open-tag nesting depth {depth} reached the effective limit of {max_depth}"
                )
            }
            Self::HiddenElement { offset } => {
                write!(f, "hidden element (hidden attribute or style) at byte offset {offset}")
            }
            Self::ListNestedOrdered => {
                write!(
                    f,
                    "nested list with an ordered ancestor or ordered self (cumulative indent width)"
                )
            }
            Self::OrderedListAfterOrderedList => {
                write!(f, "ordered list right after an ordered list (switched delimiter)")
            }
            Self::EmptyNestedListItem => {
                write!(f, "nested list item with nothing on its marker line")
            }
            Self::ListItemUnsupportedBlockChild => {
                write!(
                    f,
                    "block-level child of a list item in a shape this scanner cannot render correctly"
                )
            }
            Self::ListItemCheckbox => write!(f, "checkbox input inside a list item (a task item)"),
            Self::ImageLazyLoadSrc => write!(f, "<img> has a lazy-load placeholder src and a fallback src attribute"),
            Self::BlockquoteCite => write!(f, "<blockquote> carries a cite attribute that tier-1 does not render"),
            Self::LinkAutolinkNestedMarkup => {
                write!(
                    f,
                    "autolink-eligible <a> href had a nested tag inside the label before close"
                )
            }
            Self::AdjacentInlineEmphasis => write!(f, "adjacent strong/emphasis elements would form one delimiter run"),
            Self::WhitespaceOnlyInlineEmphasis => write!(f, "strong/emphasis element with a whitespace-only body"),
            Self::InlineMarkerNotReproduced => write!(f, "inline element whose tier-2 markers tier-1 does not emit"),
            Self::FormControl => write!(f, "form control whose text tier-2 separates from the text around it"),
            Self::RuleBetweenInlineMarkers => write!(f, "horizontal rule between inline markers"),
        }
    }
}

/// Whether `name` (the text after `&` stored in [`BailReason::UnknownEntity`]) is a character
/// reference [`crate::text::decode_character_reference`] recognizes, checked by feeding it back
/// through that same decoder with a closing `;` appended. Reuses the one table both tiers read
/// instead of a second copy of the legacy-name and numeric-reference rules.
fn is_known_reference_name(name: &str) -> bool {
    let closed = format!("&{name};");
    crate::text::decode_character_reference(&closed, 0, crate::text::ReferenceContext::Text).is_some()
}
