//! Tier-1: single-pass byte-scanner conversion path.
//!
//! `run` delegates to the byte scanner in `scanner.rs`.  When the scanner
//! encounters a construct it cannot handle, it returns `Err(BailReason::*)`
//! and the dispatcher in `lib.rs::convert` falls back to the Tier-2 path.
//!
//! When `options.extract_metadata` is true, `run` additionally re-parses the
//! head slice captured by the prescan and prepends YAML frontmatter to the
//! scanner's output (matching Tier-2 behaviour byte-for-byte).

// ~keep All submodules are declared `pub` here.  The `tier1` module itself lives
// ~keep inside `pub(crate) mod converter`, so the effective visibility is already
// ~keep crate-internal; `pub(crate)` would be redundant and triggers the
// ~keep `clippy::redundant_pub_crate` lint.  The `tier1` module is only re-exported
// ~keep from `lib.rs` under `#[cfg(any(test, feature = "testkit"))]`, so these
// ~keep submodules remain invisible outside the crate in normal builds.
pub mod bail;
pub mod parse;
pub mod router;
pub mod scanner;
pub mod spec_rules;
pub mod state;
pub mod tags;

// ~keep `lookup` is called by scanner.rs as `tier1::lookup(...)`.
pub use tags::lookup;

// ~keep `BailReason` re-export for testkit/bench consumers who pattern-match on it.
// ~keep Not needed by production code (convert_api.rs discards the bail value), so
// ~keep gate it to avoid widening the non-testkit API surface.
#[cfg(any(test, feature = "testkit"))]
pub use bail::BailReason;

// ~keep Convenience re-exports for testkit consumers that import via
// ~keep `html_to_markdown_rs::tier1::{ListKind, TagKind, …}` rather than the full
// ~keep module path.
#[cfg(any(test, feature = "testkit"))]
pub use tags::{ListKind, OptionalCloseRule, RawKind, TagKind, TagSpec};

// ~keep `RouterDecision` is compared in production code (convert_api.rs line 82)
// ~keep via the path `tier1::RouterDecision::Tier1`, so this re-export is ungated.
pub use router::RouterDecision;

use crate::converter::prescan::PrescanReport;
use crate::options::ConversionOptions;

/// Attempt a Tier-1 conversion.
///
/// Returns the complete output string (optional YAML frontmatter followed by
/// the markdown body) on success, or `Err(BailReason::*)` when the scanner
/// encounters a construct it cannot handle.  The dispatcher falls back to
/// Tier-2 transparently.
///
/// # Errors
///
/// Returns `Err(BailReason::*)` when the scanner encounters a construct it
/// cannot handle.  The dispatcher falls back to Tier-2 transparently.
///
/// ~keep Kept at this exact 3-argument signature: it is re-exported (gated on
/// ~keep `test`/`testkit`) as `html_to_markdown_rs::tier1::run` and called directly by
/// ~keep dozens of integration tests under `tests/`. `convert_api.rs`'s production path
/// ~keep needs to pass a resolved `base_url` through to the scanner, which those tests
/// ~keep have no reason to exercise -- so that need is served by
/// ~keep [`run_with_base`] instead of widening this signature.
///
/// ~keep A direct call reads the head of the page as `convert` does: the HTML tree builder
/// ~keep gives the page that the scanner reads and the metadata of the front matter.
pub fn run(html: &str, _report: &PrescanReport, options: &ConversionOptions) -> Result<String, bail::BailReason> {
    let head = crate::converter::read_head(html);
    let body = run_with_base(&head.page, options, None)?;
    let document_base_href = options.extract_metadata.then(|| head.base_href(html)).flatten();
    let front_matter = crate::convert_api::front_matter(&head, options, document_base_href.as_deref());
    Ok(crate::convert_api::join_front_matter(&front_matter, body))
}

/// Run the scanner over `html`, the page that the HTML tree builder gave, with the URL context
/// of the caller. The result is the body of the document: the caller writes the front matter.
///
/// Relative `href`/`src` destinations resolve against `effective_base` (see
/// `converter::url_resolve::resolve_attribute_url`).
///
/// # Errors
///
/// Returns `Err(BailReason::*)` when the scanner encounters a construct it cannot
/// handle.  The dispatcher falls back to Tier-2 transparently.
pub fn run_with_base(
    html: &str,
    options: &ConversionOptions,
    effective_base: Option<std::rc::Rc<url::Url>>,
) -> Result<String, bail::BailReason> {
    Ok(scanner::scan(html, options, effective_base)?.body)
}
