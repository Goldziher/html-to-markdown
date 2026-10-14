//! Tier router — decides whether an input goes to Tier-1 or Tier-2.

use crate::converter::prescan::PrescanReport;
use crate::options::{
    CodeBlockStyle, ConversionOptions, HeadingStyle, HiddenContent, HighlightStyle, InlineDataMedia, LinkStyle,
    ListIndentType, NewlineStyle, OutputFormat, PreprocessingPreset, UrlEscapeStyle, WhitespaceMode,
};

/// The routing decision produced by [`classify`] for a given input + options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouterDecision {
    /// Use the Tier-1 single-pass byte scanner.
    Tier1,
    /// Use the Tier-2 `tl::parse` + walk-node path.
    Tier2,
}

/// Classify the input against the given options and prescan report.
///
/// Returns `Tier2` if ANY of the conditions below are true; otherwise
/// returns `Tier1`.
///
/// # Structural / prescan gates (pre-existing)
///
/// - `report.had_cdata` — CDATA sections are not supported by Tier-1
/// - `report.had_unescaped_lt` — bare `<` that the prescan escaped
/// - `options.wrap` — wrapping logic lives in the Tier-2 path (for now)
/// - `options.convert_as_inline` — inline-conversion mode not yet in Tier-1
/// - `options.preprocessing.preset != PreprocessingPreset::Standard`
///   — non-standard preprocessing has Tier-2-specific semantics
/// - `!options.strip_tags.is_empty()` — tag stripping requires DOM awareness
/// - `!options.preserve_tags.is_empty()` — tag preservation requires DOM awareness
/// - `options.debug` — debug output is consistent only on Tier-2
/// - `!options.exclude_selectors.is_empty()` — Tier-1 is a byte scanner with no
///   CSS selector engine; it cannot evaluate `tl`-compatible selectors, so any
///   configured exclusion would silently pass excluded content straight
///   through instead of dropping it (see `converter/main.rs`'s
///   `tl::Selector`-based exclusion pass, outside tier1/)
/// - `options.strip_newlines` — Tier-1 never strips `\r`/`\n` from text runs;
///   Tier-2 applies it per text node (`converter/text_node.rs`)
///
/// # Result-shape gates (see doc block below the custom-element note)
///
/// `options.extract_metadata` no longer forces Tier-2: Tier-1 re-parses the
/// prescan's `head_range` slice and produces byte-identical YAML frontmatter.
///
/// `report.has_svg` no longer forces Tier-2: Phase I teaches the scanner to
/// emit `<svg>` elements as base64 data URIs matching Tier-2 byte-for-byte.
///
/// `options.keep_inline_images_in` (inline-images feature) is now handled
/// natively in Tier-1; it no longer forces a Tier-2 route.
///
/// `report.had_custom_elements` no longer forces Tier-2 (Phase FF): the scanner
/// passes unknown tags through transparently via `CUSTOM_ELEMENT_INLINE_SPEC`,
/// and Phase DD canonicalizes image alt/title entities for custom-element
/// pages via a separate `has_custom_element_tags(html)` recheck in `scan()`.
///
/// # Result-shape gates (TIER1-CRIT — silent `ConversionResult` field drops)
///
/// `convert_api.rs` (outside tier1/) hardcodes `document: None`, `tables:
/// Vec::new()`, and (with the `inline-images` feature) `images: Vec::new()` on
/// every successful Tier-1 conversion — it never threads `tier1::run`'s output
/// through the structure/image collectors Tier-2 uses to populate those
/// fields. Any option that asks for those fields to be populated must
/// therefore force Tier-2, or the caller silently gets an empty result where
/// Tier-2 would have returned real data — the exact "byte-equality contract
/// broken, feature honored by Tier-2 silently dropped by Tier-1" failure mode
/// this router exists to prevent, except at the `ConversionResult` level
/// rather than the `content` string:
///
/// - `options.include_document_structure` — gates both `result.document`
///   (`None` vs Tier-2's populated `DocumentStructure`) and `result.tables`
///   (always empty from Tier-1; Tier-2 populates it from the same structure
///   pass whenever `include_document_structure` is `true` — see
///   `ConversionResult::tables` doc comment).
/// - `options.extract_images` (`inline-images` feature) — gates
///   `result.images` (always empty from Tier-1; Tier-2 populates it via
///   `InlineImageCollector` during the DOM walk).
///
/// `options.extract_metadata` (TIER1-58 — supersedes the prior M5 decision
/// below) now DOES gate on this axis when the `metadata` feature is compiled
/// in. History: a prior phase (M5 — see `tests/tier1_metadata_test.rs`)
/// deliberately made `extract_metadata` (default `true`) stop forcing Tier-2,
/// reasoning that the YAML frontmatter embedded in `result.content` stays
/// byte-identical to Tier-2's on the fast path. That reasoning covered the
/// frontmatter *text* only. It did not cover `result.metadata` — the
/// structured `HtmlMetadata` struct — which `convert_api.rs` hardcodes to
/// `HtmlMetadata::default()` on every Tier-1 success path (`tier1::run` never
/// builds one). A caller with default options and the `metadata` feature
/// compiled in therefore silently got `{}` from `result.metadata` instead of
/// Tier-2's populated struct: a public-API correctness bug, not a documented
/// limitation.
///
/// Two fixes were considered:
/// - (a) Thread structured metadata out of `tier1::run` by building a
///   `MetadataCollector`-equivalent during the scan. Rejected: `HtmlMetadata`
///   collects headers (with computed hierarchy/IDs), links, images, and
///   structured data (JSON-LD/Microdata/RDFa) from the whole body, not just
///   the head slice `tier1::run` already re-parses for frontmatter. A second,
///   independent implementation of that surface is exactly the failure mode
///   Task A (this file's neighbours in `scanner.rs`) just fixed: two
///   hand-written copies of the same non-trivial logic silently drifting
///   apart. Higher risk, not actually cheap once the real surface is counted.
/// - (b) Gate `classify()` on `extract_metadata` (this fix). `extract_metadata`
///   is a plain `bool` on `ConversionOptions`, known synchronously before the
///   scan runs (unlike `report.had_svg` etc., which the scanner discovers
///   mid-walk) — so, unlike the `include_document_structure` gate above, this
///   one is cheap to evaluate here. Strictly correct: every routed Tier-2 call
///   returns the authoritative struct. The cost is real — it reverts M5's
///   fast path for the (feature-enabled, default-options) case — but a silent
///   `{}` is worse than a slower correct answer.
///
/// When the `metadata` feature is not compiled in, `result.metadata` does not
/// exist as a field, so there is nothing to diverge on and no gate is added.
///
/// # Style-option gates (A1 — router style-option gate)
///
/// Tier-1 hardcodes certain output style choices.  When a `ConversionOptions`
/// value deviates from Tier-1's hardcoded value, the output would silently
/// differ from Tier-2's, breaking the byte-equality contract.  The following
/// table documents each style option and the gate added here:
///
/// | Option               | Tier-1 hardcoded value                  | Gate added?                              |
/// |----------------------|-----------------------------------------|------------------------------------------|
/// | `output_format`      | `OutputFormat::Markdown`                | Yes — non-Markdown                       |
/// | `heading_style`      | `HeadingStyle::Atx`                     | Yes — non-Atx                            |
/// | `code_block_style`   | `CodeBlockStyle::Indented`              | Yes — non-Indented                       |
/// | `strong_em_symbol`   | `'*'` (asterisk)                        | Yes — any other char                     |
/// | `bullets`            | `"-*+"` (cycled by `ul_depth`)          | Yes — any other value                    |
/// | `list_indent_width`  | `2` spaces                              | Yes — `!= 2`                             |
/// | `list_indent_type`   | `ListIndentType::Spaces`                | Yes — Tabs                               |
/// | `escape_asterisks`   | `false` (no escaping)                   | Yes — `true`                             |
/// | `escape_underscores` | `false` (no escaping)                   | Yes — `true`                             |
/// | `escape_misc`        | `false` (no escaping)                   | Yes — `true`                             |
/// | `escape_ascii`       | `false` (no escaping)                   | Yes — `true`                             |
/// | `whitespace_mode`    | `WhitespaceMode::Normalized`            | Yes — Strict                             |
/// | `newline_style`      | `NewlineStyle::Spaces`                  | Yes — Backslash                          |
/// | `code_language`      | irrelevant (Indented style)             | No — gated via `code_block_style`        |
/// | `autolinks`          | implements the GFM autolink form (`<href>`) for a flat-text label; a label built from nested inline markup (e.g. `<b>`) bails (`BailReason::LinkAutolinkNestedMarkup`) instead of guessing — see `close_link`'s doc comment | No |
/// | `default_title`      | `false` (not honored)                   | Yes — `true`                             |
/// | `sub_symbol`         | `""` (transparent pass-through)         | Yes — non-empty                          |
/// | `sup_symbol`         | `""` (transparent pass-through)         | Yes — non-empty                          |
/// | `highlight_style`    | transparent (`<mark>` → plain text)     | Yes — non-None                           |
/// | `link_style`         | `LinkStyle::Inline`                     | Yes — Reference                          |
/// | `url_escape_style`   | `UrlEscapeStyle::Angle` (raw href)      | Yes — Percent                            |
/// | `compact_tables`     | `false` (padded cells: `\| cell \|`)    | Yes — `true`                             |
/// | `inline_data_media`  | `InlineDataMedia::Keep` (payload written) | Yes — AltTextOnly, DropElement         |
/// | `hidden_content`     | `HiddenContent::Drop` (hidden elements bail, templates skipped) | Yes — Reachable, All |
/// | `br_in_tables`       | honored in cells (literal `<br>` vs space) | No — scanner reads the option directly |
///
/// # Practical reachability & benchmark findings
///
/// The table and gates above answer "which options force Tier-2"; this section
/// answers the practical follow-up: how much of the remaining headroom is real.
///
/// ## Reachable option surface
///
/// With the `metadata` feature compiled in (every shipped binding), Tier-1 is
/// reachable via `TierStrategy::Auto` only when the caller sets
/// `extract_metadata: false` — and, with `inline-images` compiled in,
/// `extract_images: false` — on top of every other gate above already being
/// satisfied. `ConversionOptions::default()` sets `extract_metadata: true`, so
/// **the out-of-the-box default option set never reaches Tier-1**: every
/// default-options `Auto` call routes to Tier-2 via the `extract_metadata` gate
/// before the scanner ever runs.
///
/// ## Measured throughput (2026-08-11, this repo's 29-fixture corpus under
/// `tools/benchmark-harness/fixtures`, release build, `testkit` feature)
///
/// Commands: `htmbench run --force-tier1 --output <a>` / `--force-tier2
/// --output <b>` (3 replicate invocations, per-fixture median of each run's
/// internal calibrated best-of-3), and `cargo run --example tier1_routing -p
/// html-to-markdown-bench --features testkit` (options force
/// `extract_metadata: false` to see the option-gate-free routing behaviour).
///
/// - Aggregate: forced-Tier1 189.4ms vs forced-Tier2 181.2ms — **Tier-1 is
///   ~4.5% *slower* in aggregate** on this corpus, because 21/29 fixtures
///   (72%, in line with a prior audit's reported 69–82% bail rate) bail and
///   pay wasted-scan-then-full-Tier-2-fallback cost on top of Tier-2's own
///   cost.
/// - Bail distribution (`tier1_routing`, 21 bails): 8 `Classifier`
///   (option-gated — zero scan cost, since `classify` short-circuits before
///   the scanner runs), 7 `HiddenElement`, 3 `AdjacentRawTextTags`, 2
///   `TableBlockChildInCell`, 1 `Cdata`. `DepthLimitExceeded` was added this
///   lane but no corpus fixture is deeply-nested enough to exercise it.
/// - Native (no bail): 8/29 (28%). Six of those are sub-millisecond synthetic
///   edge-case fixtures (noisy, negligible absolute savings). The two
///   real-world wins are `mdream/github-markdown-complete.html` (0.83ms vs
///   9.73ms, ~11.7x) and `real-world/issues/gh-190/mitrade.html` (a modest,
///   sub-millisecond-absolute win).
/// - The 13 *structural* (non-`Classifier`) bails' wasted-scan time sums to
///   roughly 1ms total — under 1% of the corpus's aggregate Tier-2 cost. Even
///   a hypothetical, perfectly-accurate cheap pre-scan that skipped straight
///   to Tier-2 for those inputs could only recover that ~1%; it cannot make
///   Tier-1 beat Tier-2 in aggregate on this corpus, only shrink the gap.
///
/// ## Recommendation
///
/// On this corpus Tier-1 pays for itself only for documents resembling
/// `github-markdown-complete.html` — heavy custom-element / raw-HTML markup,
/// no hidden elements, no adjacent `<script>`/`<style>` pairs — and only when
/// the caller has already opted out of `extract_metadata` and
/// `extract_images`. Typical real-world pages in this corpus (Wikipedia
/// articles, blog posts) either bail (paying a small tax) or never reach
/// Tier-1 at all under default options. Widening structural bail coverage or
/// threading `MetadataCollector`/`InlineImageCollector` through the scanner
/// (see the rejected-alternatives note above) would pay off only if real
/// traffic looks more like the narrow native-win case than this corpus's
/// average — that has not been demonstrated. Hold Tier-1 at its current scope
/// rather than investing further, pending traffic evidence that changes this
/// picture.
#[must_use]
pub fn classify(report: &PrescanReport, options: &ConversionOptions) -> RouterDecision {
    if input_or_mode_requires_tier2(report, options)
        || structural_options_require_tier2(options)
        || formatting_options_require_tier2(options)
        || feature_options_require_tier2(options)
    {
        return RouterDecision::Tier2;
    }
    RouterDecision::Tier1
}

fn input_or_mode_requires_tier2(report: &PrescanReport, options: &ConversionOptions) -> bool {
    report.had_cdata
        || report.had_unescaped_lt
        || options.wrap
        || options.convert_as_inline
        || options.preprocessing.preset != PreprocessingPreset::Standard
        || options.debug
        || options.strip_newlines
}

const fn structural_options_require_tier2(options: &ConversionOptions) -> bool {
    // ~keep Tier-1 has no selector engine and does not build document structure.
    !options.strip_tags.is_empty()
        || !options.preserve_tags.is_empty()
        || !options.exclude_selectors.is_empty()
        || options.include_document_structure
}

fn formatting_options_require_tier2(options: &ConversionOptions) -> bool {
    if basic_formatting_differs(options) || escaping_differs(options) {
        return true;
    }
    destination_formatting_differs(options)
}

fn basic_formatting_differs(options: &ConversionOptions) -> bool {
    options.output_format != OutputFormat::Markdown
        || options.heading_style != HeadingStyle::Atx
        || options.code_block_style == CodeBlockStyle::Tildes
        || options.strong_em_symbol != '*'
        || options.bullets != "-*+"
        || options.list_indent_width != 2
        || options.list_indent_type != ListIndentType::Spaces
        || options.whitespace_mode != WhitespaceMode::Normalized
        || options.newline_style != NewlineStyle::Spaces
}

fn escaping_differs(options: &ConversionOptions) -> bool {
    options.escape_asterisks
        || options.escape_underscores
        || options.escape_misc
        || options.escape_ascii
        || options.default_title
        || !options.sub_symbol.is_empty()
        || !options.sup_symbol.is_empty()
        || options.highlight_style != HighlightStyle::None
}

fn destination_formatting_differs(options: &ConversionOptions) -> bool {
    options.link_style != LinkStyle::Inline
        || options.url_escape_style != UrlEscapeStyle::Angle
        || options.compact_tables
        || options.inline_data_media != InlineDataMedia::Keep
        || options.hidden_content != HiddenContent::Drop
}

const fn feature_options_require_tier2(options: &ConversionOptions) -> bool {
    let _ = options;
    // ~keep Tier-1 does not fire visitor callbacks.
    #[cfg(feature = "visitor")]
    if options.visitor.is_some() {
        return true;
    }

    // ~keep Tier-1 never builds the structured metadata result.
    #[cfg(feature = "metadata")]
    if options.extract_metadata {
        return true;
    }

    // ~keep Tier-1 never builds the extracted-images result.
    #[cfg(feature = "inline-images")]
    if options.extract_images {
        return true;
    }
    false
}
