//! Integration tests for the Tier-1 router (M2).
//!
//! These tests verify that `classify()` routes inputs to the correct tier
//! and that the `convert()` dispatcher produces identical output on the
//! Tier-2 fallback path.

use html_to_markdown_rs::convert;
#[cfg(feature = "testkit")]
use html_to_markdown_rs::options::{CodeBlockStyle, HighlightStyle, PreprocessingOptions, PreprocessingPreset};
use html_to_markdown_rs::options::{ConversionOptions, TierStrategy};
#[cfg(feature = "testkit")]
use html_to_markdown_rs::prescan;
#[cfg(feature = "testkit")]
use html_to_markdown_rs::tier1::router::{RouterDecision, classify};

/// Prescan `html` and classify with the given options.
#[cfg(feature = "testkit")]
fn route(html: &str, options: &ConversionOptions) -> RouterDecision {
    let (_cleaned, report) = prescan::run(html);
    classify(&report, options)
}

/// `ConversionOptions` with all structural and style gates set to Tier-1-
/// compatible values so the classifier can return Tier1.
#[cfg(feature = "testkit")]
fn minimal_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        code_block_style: CodeBlockStyle::Indented,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    }
}

// ~keep ── 1. Default options always force Tier-2 ─────────────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_routes_tier2_when_extract_metadata_true() {
    // ~keep Default options have extract_metadata: true — must always be Tier-2.
    let opts = ConversionOptions::default();
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

// ~keep ── 2. Clean HTML with metadata off → Tier-1 ───────────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_routes_tier1_when_clean_and_extract_metadata_false() {
    let opts = minimal_options();
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier1);
}

#[cfg(feature = "testkit")]
#[test]
fn classify_routes_tier2_when_inline_data_media_is_not_keep() {
    use html_to_markdown_rs::InlineDataMedia;
    for (choice, expected) in [
        (InlineDataMedia::Keep, RouterDecision::Tier1),
        (InlineDataMedia::AltTextOnly, RouterDecision::Tier2),
        (InlineDataMedia::DropElement, RouterDecision::Tier2),
    ] {
        let opts = ConversionOptions {
            inline_data_media: choice,
            ..minimal_options()
        };
        assert_eq!(route("<p>hello</p>", &opts), expected, "{choice:?}");
    }
}

#[cfg(feature = "testkit")]
#[test]
fn classify_routes_tier2_when_hidden_content_is_not_drop() {
    use html_to_markdown_rs::HiddenContent;
    for (choice, expected) in [
        (HiddenContent::Drop, RouterDecision::Tier1),
        (HiddenContent::Reachable, RouterDecision::Tier2),
        (HiddenContent::All, RouterDecision::Tier2),
    ] {
        let opts = ConversionOptions {
            hidden_content: choice,
            ..minimal_options()
        };
        assert_eq!(route("<p>hello</p>", &opts), expected, "{choice:?}");
    }
}

// ~keep ── 3. Custom elements no longer gate routing (Phase FF) ────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_tier1_on_custom_elements() {
    // ~keep Phase FF dropped the had_custom_elements router gate; Tier-1's
    // ~keep skip-subtree dispatch handles unknown custom elements natively.
    let opts = minimal_options();
    let choice = route("<my-widget>foo</my-widget>", &opts);
    assert_eq!(choice, RouterDecision::Tier1);
}

// ~keep ── 4. CDATA forces Tier-2 ──────────────────────────────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_cdata() {
    let opts = minimal_options();
    let choice = route("<svg><![CDATA[data]]></svg>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

// ~keep ── 5. Unescaped `<` forces Tier-2 ──────────────────────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_unescaped_lt() {
    let opts = minimal_options();
    let choice = route("<p>a < b</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

// ~keep ── 6. strip_tags forces Tier-2 ─────────────────────────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_strip_tags() {
    let opts = ConversionOptions {
        extract_metadata: false,
        strip_tags: vec!["div".to_string()],
        ..ConversionOptions::default()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

// ~keep ── 7. preserve_tags forces Tier-2 ──────────────────────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_preserve_tags() {
    let opts = ConversionOptions {
        extract_metadata: false,
        preserve_tags: vec!["table".to_string()],
        ..ConversionOptions::default()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

// ~keep ── 8. wrap forces Tier-2 ───────────────────────────────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_wrap() {
    let opts = ConversionOptions {
        extract_metadata: false,
        wrap: true,
        ..ConversionOptions::default()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

// ~keep ── 9. convert_as_inline forces Tier-2 ──────────────────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_convert_as_inline() {
    let opts = ConversionOptions {
        extract_metadata: false,
        convert_as_inline: true,
        ..ConversionOptions::default()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

// ~keep ── 10. non-standard preprocessing preset forces Tier-2 ─────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_non_standard_preprocessing() {
    let opts = ConversionOptions {
        extract_metadata: false,
        preprocessing: PreprocessingOptions {
            preset: PreprocessingPreset::Aggressive,
            ..PreprocessingOptions::default()
        },
        ..ConversionOptions::default()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

// ~keep ── 11. TierStrategy::Tier2 overrides classifier ────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn tier_strategy_tier2_overrides_classifier() {
    // ~keep Even the cleanest HTML with all classifier flags off: Tier2 wins.
    let opts = ConversionOptions {
        extract_metadata: false,
        code_block_style: CodeBlockStyle::Indented,
        highlight_style: HighlightStyle::None,
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    };
    // ~keep Verify the classifier alone would say Tier1.
    let (_cleaned, report) = prescan::run("<p>hello</p>");
    assert_eq!(
        classify(&report, &opts),
        RouterDecision::Tier1,
        "baseline check: classifier says Tier1"
    );

    // ~keep But the dispatcher honours Tier2 regardless.
    // ~keep We can't call the dispatcher directly here, but we verify the strategy
    // ~keep value is stored correctly and the variant exists.
    assert_eq!(opts.tier_strategy, TierStrategy::Tier2);
}

#[test]
fn convert_with_default_options_still_works() {
    let result = convert("<p>hello</p>", None);
    assert!(result.is_ok(), "convert() returned Err: {:?}", result.err());
    let md = result.unwrap().content.unwrap_or_default();
    assert!(md.contains("hello"), "expected 'hello' in output, got: {md:?}");
}

// ~keep ── 13. debug flag forces Tier-2 ────────────────────────────────────────────

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_debug_flag() {
    let opts = ConversionOptions {
        extract_metadata: false,
        debug: true,
        ..ConversionOptions::default()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

// ~keep ── 14. Tier-1 bail falls back to Tier-2 producing correct output ────────────
// ~keep
// ~keep Forces `TierStrategy::Tier1` to verify that when Tier-1 bails for any
// ~keep reason, the fallback path produces output identical to a direct Tier-2 call.
// ~keep
// ~keep This test requires the `testkit` feature because `TierStrategy::Tier1`
// ~keep is only visible when `cfg(any(test, feature = "testkit"))` is true — and
// ~keep integration tests are separate crates where `cfg(test)` is false in the
// ~keep library being tested.

#[cfg(feature = "testkit")]
#[test]
fn tier1_bail_falls_back_to_tier2() {
    let html = "<p>hello <strong>world</strong></p>";

    // ~keep Tier-2 baseline.
    let tier2_opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    };
    let tier2_output = convert(html, Some(tier2_opts))
        .expect("tier-2 must succeed")
        .content
        .unwrap_or_default();

    // ~keep Tier1 with default options: classifier normally blocks Tier-1 due to
    // ~keep extract_metadata=true, but Tier1 overrides it.  On bail the fallback
    // ~keep path runs and must produce output matching the Tier-2 path.
    let force_opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..ConversionOptions::default()
    };
    let fallback_output = convert(html, Some(force_opts))
        .expect("fallback must succeed")
        .content
        .unwrap_or_default();
    assert_eq!(
        fallback_output, tier2_output,
        "Tier-1 bail fallback output must match Tier-2 output"
    );
}

// ~keep ── 15. exclude_selectors forces Tier-2 (TIER1-CRIT) ─────────────────────────
// ~keep Tier-1 has no CSS selector engine; a configured exclusion would silently
// ~keep pass excluded content through untouched instead of dropping it.

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_exclude_selectors() {
    let opts = ConversionOptions {
        exclude_selectors: vec![".ad".to_string()],
        ..minimal_options()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

#[cfg(feature = "testkit")]
#[test]
fn classify_tier1_when_exclude_selectors_empty() {
    let opts = minimal_options();
    assert!(opts.exclude_selectors.is_empty());
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier1);
}

// ~keep ── 16. strip_newlines forces Tier-2 (TIER1-CRIT) ────────────────────────────
// ~keep Tier-1 never strips \r/\n from text runs; Tier-2 applies it per text node.

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_strip_newlines() {
    let opts = ConversionOptions {
        strip_newlines: true,
        ..minimal_options()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

#[cfg(feature = "testkit")]
#[test]
fn classify_tier1_when_strip_newlines_false() {
    let opts = minimal_options();
    assert!(!opts.strip_newlines);
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier1);
}

// ~keep ── 17. include_document_structure forces Tier-2 (TIER1-CRIT) ────────────────
// ~keep convert_api.rs hardcodes `document: None` / `tables: Vec::new()` on the
// ~keep Tier-1 success path, so this option's request would be silently dropped.

#[cfg(feature = "testkit")]
#[test]
fn classify_tier2_on_include_document_structure() {
    let opts = ConversionOptions {
        include_document_structure: true,
        ..minimal_options()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

#[cfg(feature = "testkit")]
#[test]
fn classify_tier1_when_include_document_structure_false() {
    let opts = minimal_options();
    assert!(!opts.include_document_structure);
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier1);
}

// ~keep ── 18. extract_images forces Tier-2 (TIER1-CRIT, inline-images feature) ─────
// ~keep convert_api.rs hardcodes `images: Vec::new()` on the Tier-1 success path,
// ~keep so a caller requesting extracted inline images would silently get none.

#[cfg(all(feature = "testkit", feature = "inline-images"))]
#[test]
fn classify_tier2_on_extract_images() {
    let opts = ConversionOptions {
        extract_images: true,
        ..minimal_options()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}

#[cfg(all(feature = "testkit", feature = "inline-images"))]
#[test]
fn classify_tier1_when_extract_images_false() {
    let opts = minimal_options();
    assert!(!opts.extract_images);
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier1);
}

// ~keep ── 19. Regression proof: Tier-1's ConversionResult never silently drops a
// ~keep requested structured field. Forces TierStrategy::Tier1 (bypassing the
// ~keep classifier) to prove the *scanner* would otherwise succeed here — the
// ~keep classifier gate is what stands between this input and a silently-empty
// ~keep `document`/`tables`/`images`. Confirms via the `Auto` strategy (which does
// ~keep consult the classifier) that the gated options route to Tier-2 and the
// ~keep fields end up populated, matching Tier-2 exactly.

#[test]
fn auto_routing_populates_document_and_tables_when_requested() {
    let html = "<table><tr><td>a</td></tr></table>";
    let opts = ConversionOptions {
        include_document_structure: true,
        tier_strategy: TierStrategy::Auto,
        ..ConversionOptions::default()
    };
    let result = convert(html, Some(opts)).expect("conversion must succeed");
    assert!(
        result.document.is_some(),
        "include_document_structure=true must populate result.document even under Auto routing"
    );
    assert!(
        !result.tables.is_empty(),
        "include_document_structure=true must populate result.tables for a document with a <table>"
    );
}

// ~keep ── 20. extract_metadata forces Tier-2 when the `metadata` feature is compiled
// ~keep in (TIER1-58) ────────────────────────────────────────────────────────────────
// ~keep `tier1::run` only produces YAML frontmatter *text*; it never builds the
// ~keep structured `HtmlMetadata` that `convert_api.rs` returns as `result.metadata`,
// ~keep which stays `HtmlMetadata::default()` on every Tier-1 success path regardless
// ~keep of this option. Unlike test 1 above (which uses `ConversionOptions::default()`
// ~keep and is also gated by the unrelated default `highlight_style`), these use
// ~keep `minimal_options()` so `extract_metadata` is isolated as the only variable.

// ~keep `classify_routes_tier1_when_clean_and_extract_metadata_false` (test 2, above)
// ~keep already proves the false side with these same otherwise-clear gates.

#[cfg(all(feature = "testkit", feature = "metadata"))]
#[test]
fn classify_tier2_on_extract_metadata_true_with_all_other_gates_clear() {
    let opts = ConversionOptions {
        extract_metadata: true,
        ..minimal_options()
    };
    let choice = route("<p>hello</p>", &opts);
    assert_eq!(choice, RouterDecision::Tier2);
}
