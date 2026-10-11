//! Tier-1 single-pass byte scanner.
//!
//! Walks `html.as_bytes()` once and emits Markdown directly to a pre-sized
//! output buffer.  On any construct it cannot handle exactly, returns a
//! [`BailReason`] so the dispatcher can fall back to Tier-2.
//!
//! # Supported subset (M9 + Phase E + Phase I)
//!
//! Paragraph, Heading(1-6), Strong, Emphasis, Code (inline), Pre, Hr,
//! `LineBreak`, Link, Image, List(Unordered), List(Ordered), `ListItem`,
//! Blockquote, Block (div/section/article/center/etc.), Inline (span/etc.),
//! Table (GFM — conservative bail set, inline-only cell content),
//! SVG (emitted as base64 data URI — Phase I), and custom elements (tag names
//! containing `-`, treated as Block containers).
//!
//! Bails on: RawText(script/style/textarea/etc.), `DefinitionTerm`,
//! `DefinitionDescription`, List(Definition), Ignored (head/meta/link),
//! nested tables, non-inlineable block children in cells (heading/list/blockquote/pre),
//! section-order violations, and any HTML construct with in-text whitespace
//! complexity or unclosed tags.

use crate::converter::inline::link::has_uri_scheme;
use crate::converter::tier1::bail::BailReason;
use crate::converter::tier1::parse;
use crate::converter::tier1::spec_rules;
use crate::converter::tier1::state::{EscapeCtx, OpenTag, Tier1State};
use crate::converter::tier1::tags::{ListKind, TagKind, TagSpec};
use crate::converter::tier1::{self};
use crate::converter::utility::attributes::NAV_KEYWORDS;
use crate::options::ConversionOptions;
use crate::text::ReferenceContext;

use memchr::{memchr2, memchr3};

/// Maximum byte length of a tag name lowercased into a stack buffer.
///
/// Names longer than this are silently truncated and will not match any
/// entry in the spec table, causing an `UnknownCustomElement` bail.
const MAX_TAG_NAME_BYTES: usize = 32;

/// Minimum number of dashes in a GFM separator cell.
///
/// Matches Tier-2's `col_widths.get(i).unwrap_or(0).max(MIN_SEPARATOR_DASHES)`.
const MIN_SEPARATOR_DASHES: usize = 3;

/// Static `TagSpec` used for all unknown custom elements (tag names containing
/// `-`, e.g. `<x-foo>`, `<my-component>`).
///
/// ~keep Unknown/custom elements are inline by default in HTML (there is no
/// such thing as a block-level custom element absent a UA stylesheet
/// rule or CSS `display` override, neither of which this converter
/// applies) and Tier-2's DOM walk treats them exactly that way: an
/// `<x-widget>` inside flowing text stays inline in the surrounding
/// paragraph/list-item/blockquote instead of splitting it. A `Block`
/// spec here previously matched Tier-2 only for the common case of a
/// custom element as a whole top-level document/child; it diverged the
/// moment one appeared inline.
///
/// The static reference `&CUSTOM_ELEMENT_INLINE_SPEC` is used anywhere the
/// scanner needs a `&'static TagSpec` for a custom element open/close tag.
static CUSTOM_ELEMENT_INLINE_SPEC: TagSpec = TagSpec {
    kind: TagKind::Inline,
    is_void: false,
    is_block: false,
    optional_close: None,
    is_rawtext: false,
};

/// ATX heading prefixes indexed by level − 1 (0 = `h1`, 5 = `h6`).
const HEADING_PREFIXES: [&str; 6] = ["# ", "## ", "### ", "#### ", "##### ", "###### "];

/// List-item indentation strings indexed by depth (0 = top-level, no indent).
///
/// Depths beyond the table size fall back to a runtime allocation.
const LIST_ITEM_INDENTS: [&str; 8] = [
    "",
    "  ",
    "    ",
    "      ",
    "        ",
    "          ",
    "            ",
    "              ",
];

/// Successful output of the Tier-1 scanner.
#[derive(Debug, Clone, Default)]
pub struct ScanOutput {
    /// Accumulated Markdown body.
    pub body: String,
    /// Byte range of `<head>…</head>` content (if a `<head>` was seen) in
    /// the input the scanner walked.  Forwarded by `tier1::run` to
    /// `head_metadata::extract_frontmatter` so the YAML frontmatter step
    /// works without a `PrescanReport`.
    pub head_range: Option<std::ops::Range<usize>>,
}

include!("scanner/scan_core.rs");
include!("scanner/open.rs");
include!("scanner/void.rs");
include!("scanner/close.rs");
include!("scanner/code.rs");
include!("scanner/table.rs");
include!("scanner/text.rs");
include!("scanner/text_helpers.rs");
include!("scanner/tail.rs");
