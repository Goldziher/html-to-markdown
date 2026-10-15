//! Tier-1: single-pass byte-scanner conversion path.
//!
//! `run` delegates to the byte scanner in `scanner.rs`.  When the scanner
//! encounters a construct it cannot handle, it returns `Err(BailReason::*)`
//! and the dispatcher in `lib.rs::convert` falls back to the Tier-2 path.
//!
//! When `options.extract_metadata` is true, `run` additionally re-parses the
//! source prefix before body content and prepends YAML frontmatter to the
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
pub fn run(html: &str, report: &PrescanReport, options: &ConversionOptions) -> Result<String, bail::BailReason> {
    let document_base_href = options
        .extract_metadata
        .then(|| crate::converter::url_resolve::document_base_href(html))
        .flatten();
    run_with_base(html, report, options, None, document_base_href.as_deref())
}

/// Run Tier-1 with caller-resolved URL context.
///
/// This is the same as [`run`], but resolves relative `href`/`src` destinations against
/// `effective_base` (see `converter::url_resolve::resolve_attribute_url`) and takes the
/// document's `<base href>` from the caller. Used by `convert_api.rs`'s production
/// conversion path.
///
/// # Errors
///
/// Returns `Err(BailReason::*)` when the scanner encounters a construct it cannot
/// handle.  The dispatcher falls back to Tier-2 transparently.
pub fn run_with_base(
    html: &str,
    _report: &PrescanReport,
    options: &ConversionOptions,
    effective_base: Option<std::rc::Rc<url::Url>>,
    document_base_href: Option<&str>,
) -> Result<String, bail::BailReason> {
    if !options.exclude_selectors.is_empty() {
        return Err(bail::BailReason::Classifier);
    }
    let scanner::ScanOutput { body, .. } = scanner::scan(html, options, effective_base)?;

    // ~keep Prepend YAML frontmatter when metadata extraction is requested.
    // ~keep The metadata reader parses the shared head prefix, including an implicit head.
    // ~keep
    // ~keep The frontmatter returned by the shared Tier-2 formatter ends with a
    // ~keep single `\n` after the closing `---`.  Tier-2's walker subsequently
    // ~keep gets `\n\n` from the first paragraph's leading separator (collapse
    // ~keep squashes the resulting `\n\n\n` to `\n\n`).  Tier-1 already trimmed
    // ~keep its body to a single trailing `\n` and has no walker pass to add the
    // ~keep separator, so insert one explicitly: emit `\n` between frontmatter
    // ~keep (`...---\n`) and the body (`first-paragraph...`) to land on
    // ~keep `...---\n\nfirst-paragraph...`.
    if let Some(frontmatter) = crate::converter::head_metadata::extract_frontmatter(html, options, document_base_href) {
        let mut output = String::with_capacity(frontmatter.len() + body.len() + 1);
        output.push_str(&frontmatter);
        if !body.is_empty() && !body.starts_with('\n') && !frontmatter.ends_with("\n\n") {
            output.push('\n');
        }
        output.push_str(&body);
        return Ok(output);
    }

    Ok(body)
}
