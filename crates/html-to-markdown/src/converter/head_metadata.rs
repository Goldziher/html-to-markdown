//! Metadata extraction by re-parsing the prescan's `head_range` slice.
//!
//! The prescan walks the input once and captures the byte range of the
//! `<head>…</head>` content (between the tags) in the **cleaned** buffer.
//! `extract_frontmatter` re-parses just the head slice (small) using
//! `tl::parse` and applies the same extraction rules as the Tier-2 path,
//! so both paths produce byte-identical YAML frontmatter output.
//!
//! Heads are typically small (< 50 KB), so the second parse is cheap.

use std::ops::Range;

use crate::converter::main_helpers::{extract_head_metadata, format_metadata_frontmatter};
use crate::options::ConversionOptions;

/// Attempt to extract YAML frontmatter from the document head.
///
/// `head_range` is the byte range of `<head>…</head>` content within `html`.
/// Originally captured by `prescan::run`; since Phase C the Tier-1 scanner
/// computes it directly during its single pass.
///
/// Returns `Some(frontmatter_string)` when `options.extract_metadata` is
/// true AND at least one metadata field was found.  Returns `None`
/// otherwise — callers should prepend the returned string to the body only
/// when `Some` is returned.
///
/// `document_base_href` is the document's `<base href>`, recorded as `base`
/// even when the source has no `<head>` tag (`head_range` is `None`).
pub fn extract_frontmatter(
    html: &str,
    head_range: Option<&Range<usize>>,
    options: &ConversionOptions,
    document_base_href: Option<&str>,
) -> Option<String> {
    if !options.extract_metadata {
        return None;
    }

    let head_content = head_range.and_then(|range| html.get(range.clone())).unwrap_or_default();

    // ~keep Wrap the extracted head content in a minimal HTML document so that
    // ~keep `tl::parse` has the correct context.  The wrapper tags are never
    // ~keep accessed during extraction — we only walk the direct children of
    // ~keep the synthesised `<head>` tag.
    let wrapped = format!("<html><head>{head_content}</head></html>");

    // ~keep SAFETY: we are re-parsing a small (head-only) slice of the same
    // ~keep already-cleaned HTML.  `tl::parse` is infallible for well-formed
    // ~keep UTF-8; any parse errors are silently ignored (empty metadata).
    let dom = match tl::parse(&wrapped, tl::ParserOptions::default()) {
        Ok(d) => d,
        Err(_) => return None,
    };

    // ~keep Delegate to the canonical Tier-2 extractor so the two tiers agree on
    // ~keep key names / casing / value normalisation byte-for-byte.
    let metadata = extract_head_metadata(dom.children(), dom.parser(), options, document_base_href);

    format_metadata_frontmatter(&metadata)
}
