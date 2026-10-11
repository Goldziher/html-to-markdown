//! Conversion context for HTML to Markdown conversion.
//!
//! The `Context` struct maintains state during the recursive tree walk that transforms
//! HTML to Markdown. It tracks nesting levels, element types, and feature-specific collectors
//! that are passed through the conversion pipeline.

use std::cell::{Cell, RefCell};
#[cfg(feature = "metadata")]
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::rc::Rc;

#[cfg(feature = "inline-images")]
use crate::inline_images::InlineImageCollector;

use crate::converter::reference_collector::ReferenceCollectorHandle;
use crate::options::InlineDataMedia;
use crate::types::structure_collector::StructureCollectorHandle;

/// Handle type for inline image collector when feature is enabled.
#[cfg(feature = "inline-images")]
pub type InlineCollectorHandle = Rc<RefCell<InlineImageCollector>>;
/// Placeholder type when inline-images feature is disabled.
#[cfg(not(feature = "inline-images"))]
pub type InlineCollectorHandle = ();

/// Payload type for image metadata extraction.
#[cfg(feature = "metadata")]
pub type ImageMetadataPayload = (BTreeMap<String, String>, Option<u32>, Option<u32>);

pub struct ContextParameters {
    pub inline_collector: Option<InlineCollectorHandle>,
    #[cfg(feature = "metadata")]
    pub metadata_collector: Option<crate::metadata::MetadataCollectorHandle>,
    #[cfg(feature = "visitor")]
    pub visitor: Option<crate::visitor::VisitorHandle>,
    pub structure_collector: Option<StructureCollectorHandle>,
    pub reference_collector: Option<ReferenceCollectorHandle>,
    pub base_url: Option<Rc<url::Url>>,
    pub own_page: crate::converter::url_resolve::OwnPage,
}

/// The kinds of metadata the collector asks for, read once when the context is made.
#[cfg(feature = "metadata")]
#[derive(Clone, Copy)]
pub struct MetadataPreferences {
    pub(crate) document: bool,
    pub(crate) headers: bool,
    pub(crate) links: bool,
    pub(crate) images: bool,
    pub(crate) structured_data: bool,
}

#[cfg(feature = "metadata")]
fn metadata_preferences(collector: Option<&crate::metadata::MetadataCollectorHandle>) -> MetadataPreferences {
    let Some(collector) = collector else {
        return MetadataPreferences {
            document: false,
            headers: false,
            links: false,
            images: false,
            structured_data: false,
        };
    };
    let guard = collector.borrow();
    MetadataPreferences {
        document: guard.wants_document(),
        headers: guard.wants_headers(),
        links: guard.wants_links(),
        images: guard.wants_images(),
        structured_data: guard.wants_structured_data(),
    }
}

#[derive(Clone)]
pub(super) struct DjotRuleLikeText {
    lines: Rc<[bool]>,
    current_line: Rc<Cell<usize>>,
}

impl DjotRuleLikeText {
    pub(super) fn new(lines: Vec<bool>) -> Self {
        Self {
            lines: lines.into(),
            current_line: Rc::new(Cell::new(0)),
        }
    }

    pub(super) fn current(&self) -> bool {
        self.lines.get(self.current_line.get()).copied().unwrap_or(false)
    }

    pub(super) fn advance(&self) {
        self.current_line.set(self.current_line.get().saturating_add(1));
    }
}

/// Conversion context that tracks state during HTML to Markdown conversion.
///
/// This context is passed through the recursive tree walker and maintains information
/// about the current position in the document tree, nesting levels, and enabled features.
#[derive(Clone)]
pub struct Context {
    /// Are we inside a code-like element (pre, code, kbd, samp)?
    pub(crate) in_code: bool,
    /// Are we inside a `<pre>` code BLOCK specifically, as opposed to an inline code SPAN
    /// (`<code>`/`<kbd>`/`<samp>` outside a `<pre>`)?
    ///
    /// Set only by `handle_pre` (`handlers/code_block.rs`); `handle_code` and
    /// `inline::code::handle` never set it themselves, so it propagates unchanged through
    /// their `..ctx.clone()` spread into any code nested inside a `<pre>`. `line_break.rs`
    /// needs this distinction that `in_code` alone cannot make: a `<br>` inside a code
    /// BLOCK keeps its literal `\n` (real content structure), while a `<br>` inside a code
    /// SPAN must split the span in two rather than embed a newline inside the backticks
    /// (issue #487). ~keep
    pub(crate) in_code_block: bool,
    /// Output offsets of real `<br>` nodes while a table-cell `<pre>` is collected. ~keep
    pub(crate) pre_cell_break_offsets: Option<Rc<RefCell<Vec<usize>>>>,
    /// Current list item counter for ordered lists.
    ///
    /// Signed so a negative `start` attribute (permitted by the HTML spec and honored by
    /// browsers) can be represented without a separate sign flag.
    pub(crate) list_counter: i64,
    /// Are we in an ordered list (vs unordered)?
    pub(crate) in_ordered_list: bool,
    /// Blockquote nesting depth
    pub(crate) blockquote_depth: usize,
    /// Are we inside a table cell (td/th)?
    pub(crate) in_table_cell: bool,
    /// Are we inside a table cell (td/th) that holds only inputs and white space? A checkbox
    /// writes its state there. [`Self::for_cell`] decides it where the cell opens.
    pub(crate) in_cell_of_inputs: bool,
    /// Are we inside a *layout*-table cell, whose row renders as a list item rather than a
    /// pipe row?
    ///
    /// Such a cell converts as inline, so neither the ordinary block separator nor the
    /// `in_table_cell` continuation rule applied and adjacent `<p>`/`<div>` children abutted
    /// (issue #470). This flag routes only the sibling-boundary decision through
    /// `emit_table_cell_break`; it deliberately does not enable `in_table_cell`'s pipe and
    /// emphasis escaping, which a list item does not need.
    pub(crate) in_layout_cell: bool,
    /// Should we convert block elements as inline?
    pub(crate) convert_as_inline: bool,
    /// Depth of inline formatting elements (strong/emphasis/span/etc).
    pub(crate) inline_depth: usize,
    /// Are we inside a list item?
    pub(crate) in_list_item: bool,
    /// Whether the list item is still open where the current output buffer will be written:
    /// a block written after other content then starts at the item's content column.
    ///
    /// ~keep A container that renders its children into a buffer of its own (a definition
    /// ~keep list, a sectioning element, a figure, a details element, a form) writes that buffer
    /// ~keep as a whole, so the buffer cannot show whether the item has already ended. The
    /// ~keep container works that out from its own output before it renders the children, or
    /// ~keep passes false when it writes at the start of the line (issue #583).
    pub(crate) list_item_open: bool,
    /// Whether the current output buffer is written between inline markers (a summary's `**`,
    /// a caption's `*`): the first line of a list rendered into it is text, so its items are
    /// not open and write no content column for a block (issues #583, #615).
    pub(crate) text_in_markers: bool,
    /// Whether the current output buffer is an inline wrapper's own buffer that does not count in
    /// `inline_depth`, and its first line follows the wrapper's opening marker (a `<mark>`'s
    /// `==`, a `<del>`'s `~~`) or text on the line the wrapper is written on.
    ///
    /// ~keep A rule written there is text (issue #603), and so is the first line of a list, as
    /// ~keep under `text_in_markers`.
    pub(crate) in_marker_span: bool,
    /// The column where the current output buffer starts when it is an inline wrapper's own
    /// buffer with no text or opening marker before it on its line (after a list item's marker,
    /// say): a list marker at the start of the empty buffer is written at that column, not at
    /// the start of a line.
    pub(crate) inline_buffer_column: Option<usize>,
    /// ~keep Whether an inline wrapper's detached buffer starts immediately after a hard break.
    pub(crate) inline_buffer_after_hard_break: bool,
    /// Whether the current output buffer escapes every `-` once it is written (a table
    /// caption): a `-` list marker there is text.
    pub(crate) escapes_hyphens: bool,
    /// List nesting depth (for indentation)
    pub(crate) list_depth: usize,
    /// The content column of the innermost list item: the column its marker is written at plus
    /// its marker width (`"- "` = 2, `"1. "` = 3, `"10. "` = 4, ...), honouring
    /// `list_indent_width` as a floor. With the `Tabs` indent type the marker column is the one
    /// its tabs reach.
    ///
    /// Uniform per-depth indentation (`list_depth * list_indent_width`) is only correct when
    /// every ancestor list is unordered — an ordered ancestor's marker is wider than 2 columns,
    /// so a nested list must be indented to that marker's actual content column or CommonMark
    /// parses the child as a sibling instead of nested content.
    pub(crate) list_indent_columns: usize,
    /// The `list_indent_columns` of the innermost enclosing item whose marker line starts a list
    /// item in the output (an open item, or a marker between markers that starts its own line),
    /// or 0 where none does.
    ///
    /// ~keep A line between the markers starts a block only within 3 columns of that item's
    /// ~keep content column; further in, it is the paragraph's text. The first item's marker
    /// ~keep follows the opening marker, so it is text, but a nested item's marker starts its
    /// ~keep own line and is a real list item (issue #615).
    pub(crate) real_item_columns: usize,
    /// Whether the first line of the innermost quote follows an opening inline marker, so that
    /// line is text outside the quote.
    ///
    /// ~keep The quote's other lines then hold no paragraph that a line of a list item in it
    /// ~keep can continue, so a block 4 or more columns past the column of the item whose marker
    /// ~keep starts a list item is an indented code block there, not paragraph text.
    pub(crate) quote_starts_after_markers: bool,
    /// Whether a paragraph was open before the previous marker line of the lists: the next
    /// marker's check stops there instead of walking back over every earlier item.
    pub(crate) previous_marker: crate::converter::list::utils::PreviousMarker,
    /// Where the last ordered list ended, and the delimiter it wrote.
    pub(crate) last_list: crate::converter::list::utils::LastList,
    /// The delimiter of the innermost ordered list when it is not `.`.
    pub(crate) ordered_delimiter: Option<char>,
    /// Whether the innermost list item is still open after the lines of a buffer checked so
    /// far: each block checks only the lines written since.
    pub(crate) item_lines: crate::converter::list::utils::ItemLineScan,
    /// The element of a task item whose render writes the item's first content, or `None`
    /// outside a task item.
    pub(crate) first_writer: Option<crate::converter::list::item::FirstWriter>,
    /// ~keep List and blockquote depths of the task item whose children are being collected into
    /// the detached `task_text` buffer. Text at this exact scope is later placed after the
    /// checkbox marker; nested list items and blockquotes keep normal block-start escaping.
    pub(crate) task_item_scope: Option<(usize, usize)>,
    /// Unordered list nesting depth (for bullet cycling)
    pub(crate) ul_depth: usize,
    /// Are we inside any list (ul or ol)?
    pub(crate) in_list: bool,
    /// Is this a "loose" list where all items should have blank lines?
    pub(crate) loose_list: bool,
    /// Did a previous list item have block children?
    pub(crate) prev_item_had_blocks: bool,
    /// Are we inside a heading element (h1-h6)?
    pub(crate) in_heading: bool,
    /// Whether inline images should remain markdown inside the current heading.
    pub(crate) heading_allow_inline_images: bool,
    /// Whether inline images should remain markdown inside the current
    /// layout-table cell (its tag name is in `keep_inline_images_in`).
    pub(crate) cell_allow_inline_images: bool,
    /// Whether inline images should remain markdown inside the current `<a>` link label
    /// ("a" is in `keep_inline_images_in`). Set once per `handle_link` call and carried
    /// unchanged into both the block-label and inline-label branches, so the option means
    /// the same thing whether or not the anchor happens to contain a block child (#492).
    pub(crate) link_allow_inline_images: bool,
    /// Are we inside a paragraph element?
    pub(crate) in_paragraph: bool,
    /// ~keep `None` before a Djot text container is classified, then the rule-like status of
    /// each logical line. Inline descendants share the cursor so DOM segmentation cannot change
    /// which literal stars are escaped, while a `<br>` can start an ordinary following line.
    pub(super) djot_rule_like_text: Option<DjotRuleLikeText>,
    /// Output buffer position where the current block's content starts.
    /// Used to distinguish paragraph-break newlines from a previous block
    /// vs. newlines generated within the current block.
    pub(crate) block_content_start: usize,
    /// Address of the `output` buffer `block_content_start` was measured against (set
    /// alongside it in `block/paragraph.rs`), as `*const String as usize`.
    ///
    /// `block_content_start` alone cannot tell "this text node's `output` is the SAME
    /// buffer the paragraph started measuring" from "it coincidentally has the same
    /// length" -- an inline wrapper (`em`/`strong`/link) builds its content into a
    /// *different, fresh local `String`* before splicing it into the real output, and that
    /// scratch buffer's length can coincidentally equal the ANCESTOR paragraph's
    /// `block_content_start` even when real content already precedes this point in the
    /// actual document (issue #481: `<p>A<i> </i>B</p>` saw the space inside `<i>`'s own
    /// empty scratch buffer misidentified as the paragraph's own leading whitespace and
    /// dropped). A raw address comparison, unlike `ctx.inline_depth == 0`, correctly
    /// still fires when a block-level tag-soup wrapper (e.g. a stray `<b>` wrapping whole
    /// paragraphs, as Google Docs exports) puts a genuine, buffer-sharing paragraph body
    /// at non-zero `inline_depth`.
    pub(crate) block_output_ptr: usize,
    /// Shared flag: true until the first node writes content that is not white space. The node
    /// is text, or an element that writes content of its own (an image, a graphic, a form
    /// control). `convert_node` clears the flag for every kind of node (issue #762).
    ///
    /// ~keep `block_content_start` cannot answer "is this text node at the very start of a
    /// ~keep fresh block" on its own: it is a byte offset compared against `output.len()`,
    /// ~keep but every inline wrapper (`sub`/`sup`/`em`/`strong`/links/...) builds its content
    /// ~keep into a *fresh local `String`* before splicing it into the real output, so that
    /// ~keep comparison sees an empty scratch buffer (len 0) that can coincidentally equal a
    /// ~keep real block start of 0 -- indistinguishable from genuine document start. This
    /// ~keep flag is `Rc<Cell<bool>>` instead so every clone of `Context` -- scratch-buffer
    /// ~keep handlers included -- observes one shared, buffer-independent truth about
    /// ~keep whether real content already precedes this point in the actual document.
    /// ~keep Currently reset to `true` only at document start (`Context::new`); nothing
    /// ~keep flips it back to `true` mid-document, so it only ever fires exactly once, but
    /// ~keep the mechanism is ready for additional reset points (e.g. per-block handlers) to
    /// ~keep extend the same "insignificant leading whitespace" treatment to fresh blocks
    /// ~keep beyond the very first one, without changing how consumers read it.
    pub(crate) at_fresh_block_start: Rc<Cell<bool>>,
    /// Are we inside a ruby element?
    pub(crate) in_ruby: bool,
    /// Are we inside a `<strong>` / `<b>` element?
    pub(crate) in_strong: bool,
    /// Are we inside a link element (collecting link label text)?
    pub(crate) in_link: bool,
    /// Tag names that should be stripped during conversion.
    pub(crate) strip_tags: Rc<HashSet<String>>,
    /// Tag names that should be preserved as raw HTML.
    pub(crate) preserve_tags: Rc<HashSet<String>>,
    /// Tag names that allow inline images inside headings.
    pub(crate) keep_inline_images_in: Rc<HashSet<String>>,
    /// Node IDs matching `exclude_selectors` — these nodes and all descendants are dropped.
    pub(crate) excluded_node_ids: Rc<HashSet<u32>>,
    /// Shared flag set when the guarded DOM walk reaches its effective depth limit.
    pub(crate) depth_limit_reached: Rc<Cell<bool>>,
    /// Shared flag set when `inline_data_media` replaced or dropped an element. A link reads it to
    /// tell a label emptied by that option from an empty one.
    pub(crate) inline_data_replaced: Rc<Cell<bool>>,
    #[cfg(feature = "inline-images")]
    /// Shared collector for inline images when enabled.
    pub(crate) inline_collector: Option<InlineCollectorHandle>,
    #[cfg(feature = "metadata")]
    /// Shared collector for metadata when enabled.
    pub(crate) metadata_collector: Option<crate::metadata::MetadataCollectorHandle>,
    #[cfg(feature = "metadata")]
    /// The kinds of metadata the collector asks for; all false without a collector.
    pub(crate) metadata_wants: MetadataPreferences,
    #[cfg(feature = "visitor")]
    /// Optional visitor for custom HTML traversal callbacks.
    pub(crate) visitor: Option<crate::visitor::VisitorHandle>,
    #[cfg(feature = "visitor")]
    /// Stores the first visitor error encountered during traversal.
    pub(crate) visitor_error: Rc<RefCell<Option<String>>>,
    #[cfg(feature = "visitor")]
    /// When `true`, the visitor of this context is not the visitor of the caller: the walk is
    /// one that [`Self::unobserved_outside_code`] started, and nothing it writes or calls is seen.
    pub(crate) is_unobserved: bool,
    /// Optional structure collector for building a [`crate::types::DocumentStructure`].
    ///
    /// Populated when `options.include_document_structure == true`.
    pub(crate) structure_collector: Option<StructureCollectorHandle>,
    /// Optional reference collector for reference-style links.
    pub(crate) reference_collector: Option<ReferenceCollectorHandle>,
    /// When `true`, all visitor-hook Mutex acquisitions are skipped.
    ///
    /// Set during the table column-width pre-pass, which walks every cell to
    /// measure rendered text width but never uses visitor results.  The pre-pass
    /// runs before the main rendering pass, so acquiring the Mutex there is pure
    /// overhead (especially costly post-`Arc<Mutex>` migration in 7f6178f25).
    pub(crate) skip_visitor_hooks: bool,
    /// When `true`, `walk_node` short-circuits nested-table dispatch — the
    /// outer table is currently in its width-measurement pre-pass and the
    /// nested table's full markdown rendering (which itself runs a pre-pass on
    /// every descendant cell) would explode CPU on layout-heavy HTML.
    ///
    /// The per-cell width cap (`MAX_CELL_WIDTH = 200` in
    /// `cells.rs::collect_row_cell_widths`) bounds the discarded *output*, but
    /// the underlying *measurement* still walked every nested table fully.
    /// Skipping the nested-table dispatch during measurement keeps the
    /// pre-pass linear in descendant text length without changing rendered
    /// output (the column width is slightly underestimated for cells
    /// containing tables, which is fine — the cap dominates anyway).
    ///
    /// Set together with `skip_visitor_hooks` when constructing `prepass_ctx`
    /// in `block::table::builder::handle_table`. Resolves issue #406.
    pub(crate) measure_width_only: bool,
    /// Effective base URL for resolving relative `href`/`src` destinations, already
    /// combined with any document `<base href>` (see `converter::url_resolve`).
    /// `None` when `options.base_url` is unset -- every resolution call becomes a
    /// no-op then, so output is byte-identical to before this option existed.
    pub(crate) base_url: Option<Rc<url::Url>>,
    /// The address of the page, as far as it is known. A link asks it whether it points into
    /// the page (see `converter::url_resolve::OwnPage`).
    pub(crate) own_page: Rc<crate::converter::url_resolve::OwnPage>,
}

impl Context {
    /// Set the pre-computed set of node IDs that match `exclude_selectors`.
    ///
    /// Called in `convert_html_impl` after DOM parsing, before the walk starts.
    pub(crate) fn set_excluded_node_ids(&mut self, ids: HashSet<u32>) {
        self.excluded_node_ids = Rc::new(ids);
    }

    /// Create a new conversion context from options and optional collectors.
    pub(crate) fn new(options: &crate::options::ConversionOptions, parameters: ContextParameters) -> Self {
        #[cfg(feature = "metadata")]
        let metadata = metadata_preferences(parameters.metadata_collector.as_ref());

        Self {
            in_code: false,
            in_code_block: false,
            pre_cell_break_offsets: None,
            list_counter: 0,
            in_ordered_list: false,
            blockquote_depth: 0,
            in_table_cell: false,
            in_cell_of_inputs: false,
            in_layout_cell: false,
            convert_as_inline: options.convert_as_inline,
            inline_depth: 0,
            in_list_item: false,
            list_item_open: false,
            text_in_markers: false,
            in_marker_span: false,
            inline_buffer_column: None,
            inline_buffer_after_hard_break: false,
            escapes_hyphens: false,
            list_depth: 0,
            list_indent_columns: 0,
            real_item_columns: 0,
            quote_starts_after_markers: false,
            previous_marker: crate::converter::list::utils::PreviousMarker::default(),
            last_list: crate::converter::list::utils::LastList::default(),
            ordered_delimiter: None,
            item_lines: crate::converter::list::utils::ItemLineScan::default(),
            first_writer: None,
            task_item_scope: None,
            ul_depth: 0,
            in_list: false,
            loose_list: false,
            prev_item_had_blocks: false,
            in_heading: false,
            heading_allow_inline_images: false,
            cell_allow_inline_images: false,
            link_allow_inline_images: false,
            in_paragraph: false,
            djot_rule_like_text: None,
            block_content_start: 0,
            block_output_ptr: 0,
            at_fresh_block_start: Rc::new(Cell::new(true)),
            in_ruby: false,
            in_strong: false,
            in_link: false,
            strip_tags: Rc::new(options.strip_tags.iter().cloned().collect()),
            preserve_tags: Rc::new(options.preserve_tags.iter().cloned().collect()),
            keep_inline_images_in: Rc::new(options.keep_inline_images_in.iter().cloned().collect()),
            excluded_node_ids: Rc::new(HashSet::new()),
            depth_limit_reached: Rc::new(Cell::new(false)),
            inline_data_replaced: Rc::new(Cell::new(false)),
            #[cfg(feature = "inline-images")]
            inline_collector: parameters.inline_collector,
            #[cfg(feature = "metadata")]
            metadata_collector: parameters.metadata_collector,
            #[cfg(feature = "metadata")]
            metadata_wants: metadata,
            #[cfg(feature = "visitor")]
            visitor: parameters.visitor,
            #[cfg(feature = "visitor")]
            visitor_error: Rc::new(RefCell::new(None)),
            #[cfg(feature = "visitor")]
            is_unobserved: false,
            structure_collector: parameters.structure_collector,
            reference_collector: parameters.reference_collector,
            skip_visitor_hooks: false,
            measure_width_only: false,
            base_url: parameters.base_url,
            own_page: Rc::new(parameters.own_page),
        }
    }

    /// Whether the current output buffer is written between inline markers: an inline wrapper's
    /// (`inline_depth`, a link's brackets included), a summary's or caption's (`text_in_markers`)
    /// or a marker-only wrapper's (`in_marker_span`). A rule written there is text, since the
    /// markers cannot span it.
    pub(crate) const fn in_marker_text(&self) -> bool {
        self.inline_depth > 0 || self.text_in_markers || self.in_marker_span
    }

    /// The context for the children of an inline wrapper that renders them into a buffer of its
    /// own and then writes that buffer at the end of `output`, after its opening marker if
    /// `opens_with_marker`.
    ///
    /// ~keep The buffer continues the line `output` ends on. After an opening marker or text on
    /// ~keep that line, the first line of a list in the buffer is text, as in bold (issue #615).
    /// ~keep After only indent or a list item's marker, the list's first marker starts an item
    /// ~keep where that line ends, as it would with no wrapper.
    pub(crate) fn inline_buffer(&self, output: &str, opens_with_marker: bool) -> Self {
        let line = &output[output.rfind('\n').map_or(0, |pos| pos + 1)..];
        let line_holds_text =
            !line.trim().is_empty() && !crate::converter::list::utils::line_is_bare_list_marker(output);
        let (indent_length, indent_column) = crate::converter::utility::escaping::leading_indent(line);
        Self {
            in_marker_span: self.in_marker_span || opens_with_marker || line_holds_text,
            inline_buffer_column: if output.is_empty() {
                self.inline_buffer_column
            } else {
                Some(indent_column + line[indent_length..].chars().count())
            },
            inline_buffer_after_hard_break: if output.is_empty() {
                self.inline_buffer_after_hard_break
            } else {
                crate::converter::utility::escaping::ends_with_hard_break(output)
            },
            ..self.clone()
        }
    }

    /// The context for the children of the table cell `cell_handle`: `self` with the answer to
    /// "does this cell hold only inputs" for that cell.
    ///
    /// ~keep Each walk of a cell's children takes its context from here, so the cell is read one
    /// ~keep time for each walk and not one time for each control in it. A row shares one context
    /// ~keep between its cells: this copies it only for a cell whose answer differs.
    pub(crate) fn for_cell(
        &self,
        cell_handle: tl::NodeHandle,
        parser: &tl::Parser,
        dom_ctx: &crate::converter::DomContext,
        br_in_tables: bool,
    ) -> std::borrow::Cow<'_, Self> {
        let in_cell_of_inputs =
            crate::converter::block::table::scanner::cell_holds_only_inputs(cell_handle, parser, dom_ctx, br_in_tables);
        if in_cell_of_inputs == self.in_cell_of_inputs {
            std::borrow::Cow::Borrowed(self)
        } else {
            std::borrow::Cow::Owned(Self {
                in_cell_of_inputs,
                ..self.clone()
            })
        }
    }

    /// Whether the visitor of the caller sees this walk.
    ///
    /// ~keep A link in code asks what its content writes outside code only when this holds. In
    /// ~keep an unobserved walk no answer reaches the caller's visitor, so no link there starts
    /// ~keep an unobserved walk of its own: a link in `<code>` in a link in `<code>` cost twice
    /// ~keep the walks at each level before.
    #[cfg(feature = "visitor")]
    pub(crate) fn has_observer(&self) -> bool {
        self.visitor.is_some() && !self.is_unobserved
    }

    /// Whether a `code`, `kbd` or `samp` element writes its content and nothing of its own in
    /// this walk: it starts no code and adds no code marks.
    ///
    /// ~keep A code element in code is such an element. An unobserved walk answers what the
    /// ~keep content of a link in code writes outside code, so it stays outside code to its end,
    /// ~keep and a code element in it is still an element in code. When the element started code,
    /// ~keep an image in it wrote nothing and the link lost its call. When the element wrote code
    /// ~keep marks, a line break or a space in it was a label and the link got a call.
    #[cfg(feature = "visitor")]
    pub(crate) const fn code_element_is_transparent(&self) -> bool {
        self.in_code || self.is_unobserved
    }

    /// Whether a `code`, `kbd` or `samp` element writes its content and nothing of its own in
    /// this walk: a code element in code is such an element.
    #[cfg(not(feature = "visitor"))]
    pub(crate) const fn code_element_is_transparent(&self) -> bool {
        self.in_code
    }

    /// A copy of this context that writes as outside code and that nothing observes: its visitor
    /// continues at every call and records nothing, it has no collector, and it shares no flag,
    /// counter or list state with this context.
    ///
    /// ~keep A link in code asks its content what that content writes outside code. The answer
    /// ~keep comes from the normal traversal with this copy, so every rule of the traversal holds
    /// ~keep for the answer: a stripped tag, an excluded node, a preset, skipped images. The copy
    /// ~keep keeps a visitor because the traversal applies a preset only when it has none.
    #[cfg(feature = "visitor")]
    pub(crate) fn unobserved_outside_code(&self) -> Self {
        #[derive(Debug)]
        struct ContinuesEverywhere;
        impl crate::visitor::HtmlVisitor for ContinuesEverywhere {}

        Self {
            in_code: false,
            in_code_block: false,
            pre_cell_break_offsets: None,
            previous_marker: crate::converter::list::utils::PreviousMarker::default(),
            last_list: crate::converter::list::utils::LastList::default(),
            item_lines: crate::converter::list::utils::ItemLineScan::default(),
            first_writer: None,
            djot_rule_like_text: None,
            at_fresh_block_start: Rc::new(Cell::new(self.at_fresh_block_start.get())),
            depth_limit_reached: Rc::new(Cell::new(self.depth_limit_reached.get())),
            inline_data_replaced: Rc::new(Cell::new(false)),
            #[cfg(feature = "inline-images")]
            inline_collector: None,
            #[cfg(feature = "metadata")]
            metadata_collector: None,
            visitor: self.visitor.as_ref().map(|_| -> crate::visitor::VisitorHandle {
                std::sync::Arc::new(std::sync::Mutex::new(ContinuesEverywhere))
            }),
            visitor_error: Rc::new(RefCell::new(None)),
            is_unobserved: true,
            structure_collector: None,
            reference_collector: None,
            ..self.clone()
        }
    }

    /// What to write for an element whose chosen address is `address`, as
    /// [`crate::converter::media::inline_data_treatment`] decides, recording in
    /// [`Self::inline_data_replaced`] when it is not [`InlineDataMedia::Keep`].
    pub(crate) fn inline_data_treatment(&self, choice: InlineDataMedia, address: &str) -> InlineDataMedia {
        let treatment = crate::converter::media::inline_data_treatment(choice, address);
        if treatment != InlineDataMedia::Keep {
            self.inline_data_replaced.set(true);
        }
        treatment
    }

    /// Resolve a `href`/`src` attribute value against [`Self::base_url`].
    ///
    /// Returns `None` (meaning: use the original text unchanged) whenever `base_url`
    /// is unset, `value` is empty, already absolute, or fails to resolve. See
    /// `converter::url_resolve::resolve_attribute_url` for the full contract.
    pub(crate) fn resolve_url(
        &self,
        value: &str,
        node_handle: &tl::NodeHandle,
        parser: &tl::Parser,
        dom_ctx: &crate::converter::DomContext,
    ) -> Option<String> {
        if dom_ctx.has_raw_text_ancestor(node_handle.get_inner(), parser) {
            return None;
        }
        crate::converter::url_resolve::resolve_attribute_url(self.base_url.as_deref()?, value)
    }
}
