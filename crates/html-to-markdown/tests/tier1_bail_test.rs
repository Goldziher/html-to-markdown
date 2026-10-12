//! Comprehensive tests for Tier-1 bail tripwires (M6).
//!
//! Verifies that:
//!   (a) Each tripwire fires with the correct `BailReason` variant.
//!   (b) `convert()` with `Auto` or `Tier1` transparently falls back to
//!       Tier-2 and produces output byte-identical to the `Tier2` path.

#![cfg(feature = "testkit")]

use html_to_markdown_rs::prescan::{self, PrescanReport};
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

/// Prescan `html` then run the Tier-1 scanner directly.  Returns the
/// `BailReason` or the successful Markdown string.
fn tier1_run(html: &str) -> Result<String, BailReason> {
    let (cleaned, report) = prescan::run(html);
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    tier1::run(cleaned.as_ref(), &report, &opts)
}

/// Run `tier1::run` WITHOUT prescanning so constructs like CDATA survive to the
/// scanner.  Useful for testing tripwires that would be pre-sanitized otherwise.
fn tier1_raw(html: &str) -> Result<String, BailReason> {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    tier1::run(html, &PrescanReport::default(), &opts)
}

/// `convert()` via the Tier-2-only path.
fn tier2(html: &str) -> String {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    convert(html, Some(opts)).unwrap().content.unwrap_or_default()
}

/// `convert()` via the Auto path (classifier decides; falls back on bail).
fn auto(html: &str) -> String {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    convert(html, Some(opts)).unwrap().content.unwrap_or_default()
}

/// `convert()` with `Tier1` — bails silently and falls back to Tier-2.
fn force_tier1(html: &str) -> String {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    convert(html, Some(opts)).unwrap().content.unwrap_or_default()
}

// ~keep ── Tripwire 1: lenient literal `<` handling ──────────────────────────────────
// ~keep
// ~keep Tier-1 used to bail with `BailReason::LiteralLt` on any `<` not followed by
// ~keep a tag-name-start byte. The scanner now emits the bare `<` as text and
// ~keep continues, matching Tier-2's html5ever / astral-tl behaviour (both parse
// ~keep `<x` where x is whitespace/digit as a text node). Document the new
// ~keep behaviour with byte-equality assertions against Tier-2.

#[test]
fn lenient_literal_lt_in_text() {
    let html = "<p>a < b</p>";
    assert!(tier1_raw(html).is_ok());
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn lenient_literal_lt_numeric() {
    let html = "<p>3 < 5</p>";
    assert!(tier1_raw(html).is_ok());
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn scanner_bails_on_cdata_direct() {
    // ~keep Call tier1::run without prescanning so `<![CDATA[` survives to the scanner.
    let html = "<p><![CDATA[data]]></p>";
    let err = tier1_raw(html).unwrap_err();
    assert!(matches!(err, BailReason::Cdata { .. }), "expected Cdata, got {err:?}");
}

// ~keep ── issue #458: a literal backslash stays on the Tier-1 fast path ──────────
// ~keep
// ~keep Tier-2 escapes a source backslash unconditionally, so Tier-1 has to apply the
// ~keep same rule at emission time rather than bail — a backslash is not rare enough
// ~keep in prose to justify surrendering the fast path for a whole document. These
// ~keep assertions are the byte-equality contract for that rule.

#[test]
fn literal_backslash_stays_native_and_matches_tier2_across_contexts() {
    for html in &[
        r"<p>3\*4</p>",
        r"<p>a\3b</p>",
        r"<p>abc\</p>",
        "<p>abc\\\ndef</p>",
        r"<p>C:\Users\Alice</p>",
        r"<p><strong>abc\</strong></p>",
        r"<p><em>a\*b</em></p>",
        r#"<p><a href="https://e.com">a\*b</a></p>"#,
        r"<ul><li>a\*b</li></ul>",
        r"<blockquote><p>a\*b</p></blockquote>",
        r"<p><code>a\*b</code></p>",
        "<pre><code>a\\*b</code></pre>",
        // ~keep A cell without `*`/`_`: Tier-1 does not escape those in cells at all
        // ~keep (a pre-existing divergence unrelated to #458), which would mask the
        // ~keep backslash comparison this test is making.
        r"<table><tr><td>abc\</td></tr></table>",
        r"<table><tr><td>a\3b</td></tr></table>",
        r"<p>a&#92;*b</p>",
    ] {
        // ~keep Assert the scanner completed natively first: `force_tier1` falls back to
        // ~keep Tier-2 on a bail, so a byte-equality check on its own would pass even if
        // ~keep the fast path had silently given up on every one of these inputs.
        assert!(tier1_run(html).is_ok(), "tier-1 bailed on {html:?}");
        assert_eq!(force_tier1(html), tier2(html), "tier mismatch for {html:?}");
    }
}

#[test]
fn backslash_inside_code_and_pre_is_left_verbatim_by_tier1() {
    // ~keep Tier-2 bypasses `text::escape` entirely for code/pre content, so Tier-1
    // ~keep must not apply the escape there either.
    let code = tier1_run(r"<p><code>a\*b</code></p>").expect("code span should stay on tier-1");
    assert!(code.contains(r"`a\*b`"), "expected verbatim code span, got {code:?}");
    let pre = tier1_run("<pre><code>a\\*b</code></pre>").expect("code block should stay on tier-1");
    assert!(pre.contains(r"a\*b"), "expected verbatim code block, got {pre:?}");
}

#[test]
fn svg_with_cdata_handled_natively_via_prescan() {
    // ~keep After prescanning, CDATA inside SVG is escaped to `&lt;![CDATA[...]` so
    // ~keep the scanner no longer sees raw CDATA.  Phase I teaches Tier-1 to emit
    // ~keep `<svg>` elements as base64 data URIs, so this input is now handled
    // ~keep natively (no bail).  The final output from Tier-1 must match Tier-2.
    let html = "<svg><![CDATA[xxx]]></svg><p>x</p>";
    // ~keep tier1_run prescans first — CDATA is escaped, then SVG is emitted as base64.
    assert!(
        tier1_run(html).is_ok(),
        "expected success: SVG with CDATA handled after prescan"
    );
    assert_eq!(force_tier1(html), tier2(html));
}

// ~keep ── Custom-element block passthrough (Phase E) ───────────────────────────────
// ~keep
// ~keep Custom elements (tag names containing `-`) are now handled natively by
// ~keep Tier-1 as generic block containers rather than bailed immediately.
// ~keep Tier-2 emits their inner content as plain block text; Tier-1's Block
// ~keep dispatch produces byte-identical output.

#[test]
fn custom_element_handled_natively() {
    // ~keep Tier-1 should NOT bail — it handles <my-thing> as a Block container
    // ~keep and emits the inner content as-is.
    let html = "<my-thing>x</my-thing>";
    let result = tier1_run(html);
    assert!(result.is_ok(), "expected success, got bail: {:?}", result.unwrap_err());
    assert_eq!(result.unwrap(), tier2(html), "Tier-1 output must match Tier-2");
}

#[test]
fn custom_element_content_matches_tier2() {
    // ~keep Verify a range of custom-element patterns all produce Tier-2-identical output.
    for html in &[
        "<data-widget>content</data-widget>",
        "<x-button>click</x-button>",
        "<ui-card><p>hi</p></ui-card>",
        "<foo-bar baz=\"1\">text</foo-bar>",
        "<my-thing>x</my-thing>",
    ] {
        let t1 = force_tier1(html);
        let t2 = tier2(html);
        assert_eq!(t1, t2, "output mismatch for {html:?}");
    }
}

#[test]
fn bails_on_custom_element_fallback_matches_tier2() {
    // ~keep This test verifies that even when Tier-1 handles custom elements natively
    // ~keep (no bail), the output is identical to Tier-2.  Kept for regression coverage.
    for html in &[
        "<x-button>click</x-button>",
        "<ui-card><p>hi</p></ui-card>",
        "<foo-bar baz=\"1\">text</foo-bar>",
    ] {
        assert_eq!(force_tier1(html), tier2(html), "fallback mismatch for {html:?}");
    }
}

// ~keep ── Tripwire 4: EOF implicit-close (was: EofWithOpenBlock) ───────────────────
// ~keep
// ~keep Phase N2: the scanner now closes every remaining open frame at EOF instead
// ~keep of bailing, mirroring the HTML5 parser's behaviour.  Trailing whitespace is
// ~keep trimmed from the output buffer before each implicit close so inline markers
// ~keep like `</strong>` land flush against the content (no `world\n**`).

#[test]
fn handles_eof_with_open_block_inline() {
    let html = "<p>hello <strong>world";
    assert!(tier1_run(html).is_ok(), "Tier-1 should auto-close at EOF (no bail)");
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn handles_eof_with_single_unclosed_div() {
    let html = "<div>some text without closing tag";
    assert!(
        tier1_run(html).is_ok(),
        "Tier-1 should auto-close <div> at EOF (no bail)"
    );
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn handles_eof_nested_open_inline() {
    // ~keep Three levels open: ul > li > strong. Phase N2: scanner closes all
    // ~keep three at EOF instead of bailing.
    let html = "<ul><li>item <strong>bold";
    assert!(
        tier1_run(html).is_ok(),
        "Tier-1 should auto-close nested elements at EOF"
    );
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn bails_on_depth_mismatch_close_without_open() {
    // ~keep The end tag stands in the body: the HTML tree builder drops an end tag in front of
    // ~keep the body, and the page that the scanner reads does not hold it.
    let html = "x</p>orphan close";
    let err = tier1_run(html).unwrap_err();
    assert!(
        matches!(err, BailReason::DepthMismatch { .. }),
        "expected DepthMismatch, got {err:?}"
    );
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn bails_on_depth_mismatch_wrong_close() {
    let html = "<p>text</div>";
    let err = tier1_run(html).unwrap_err();
    assert!(
        matches!(err, BailReason::DepthMismatch { .. }),
        "expected DepthMismatch, got {err:?}"
    );
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn depth_mismatch_contains_tag_name() {
    let html = "<p>text</div>";
    if let Err(BailReason::DepthMismatch { tag, .. }) = tier1_run(html) {
        assert_eq!(tag, "div", "expected tag name 'div', got {tag:?}");
    } else {
        panic!("expected DepthMismatch");
    }
}

#[test]
fn table_now_handled_by_tier1() {
    // ~keep M9: simple tables are now handled by Tier-1 (no longer bail).
    // ~keep Tier-1 output must be byte-identical to Tier-2.
    let html = "<table><tr><td>a</td></tr></table>";
    let result = tier1_run(html);
    assert!(result.is_ok(), "expected success, got {result:?}");
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn handles_definition_list_inline() {
    // ~keep Phase K: scanner emits <dl>/<dt>/<dd> natively, matching Tier-2's
    // ~keep plain-block format (no Pandoc colon syntax). dt → trimmed + "\n";
    // ~keep dd → trimmed + "\n\n"; dl wrapper trims and ensures "\n\n" boundaries.
    let html = "<dl><dt>key</dt><dd>value</dd></dl>";
    assert!(tier1_run(html).is_ok(), "Tier-1 should handle dl inline (no bail)");
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn handles_link_text_across_newline() {
    // ~keep Phase Q.3: text inside an `<a>` that spans a newline (e.g. pretty-printed
    // ~keep HTML wrapping link text across lines) must collapse the newline to a
    // ~keep single space, matching Tier-2's `normalize_link_label`.  Tier-1's
    // ~keep text-handler previously preserved the newline verbatim, producing
    // ~keep `[Skip to main\n  content](...)` instead of `[Skip to main content](...)`.
    let html = "<a href=\"#main-content\">Skip to main\n  content</a>";
    assert!(tier1_run(html).is_ok());
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn handles_strong_text_across_newline() {
    // ~keep Q.3 sanity: `<strong>` does NOT normalize newlines (only `<a>` does).
    // ~keep Tier-2 keeps the literal newline in strong text.
    let html = "<p><strong>bold\ntext</strong></p>";
    assert!(tier1_run(html).is_ok());
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn handles_script_inline() {
    // ~keep Phase B: scanner skips `<script>` content inline by jumping to the
    // ~keep matching close tag.  Prescan also pre-strips script content
    // ~keep (defence-in-depth).  Either way Tier-1 produces the same bytes as
    // ~keep Tier-2 with no bail.
    let html = "<script>var x = 1;</script><p>ok</p>";
    assert!(tier1_run(html).is_ok(), "Tier-1 should handle script inline (no bail)");
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn bails_on_textarea() {
    // ~keep textarea's text content is preserved by Tier-2 (the DOM walker emits
    // ~keep its text node).  Tier-1 must therefore bail to Tier-2 rather than
    // ~keep silent-skipping — see the Phase B narrowing in scanner.rs.
    let html = "<textarea>some text</textarea>";
    let err = tier1_run(html).unwrap_err();
    assert!(
        matches!(err, BailReason::Classifier),
        "expected Classifier, got {err:?}"
    );
    assert_eq!(force_tier1(html), tier2(html));
}

#[test]
fn auto_routes_through_tier1_for_clean_paragraph() {
    // ~keep Clean paragraph with no complex constructs → Auto picks Tier-1 when
    // ~keep extract_metadata=false.
    // ~keep Both tiers must produce identical output.
    let html = "<p>Hello <strong>world</strong></p>";
    assert_eq!(auto(html), tier2(html));
}

#[test]
fn auto_routes_correctly_for_complex_input() {
    // ~keep M9: tables are handled by Tier-1. Auto, Tier1, and Tier2
    // ~keep must all produce byte-identical output for a simple table.
    let html = "<table><tr><td>cell</td></tr></table>";
    assert_eq!(auto(html), tier2(html));
    assert_eq!(force_tier1(html), tier2(html));
}

// ~keep ── BailReason Display ────────────────────────────────────────────────────────

#[test]
fn bail_reason_display_is_non_empty() {
    let reasons: Vec<BailReason> = vec![
        BailReason::Classifier,
        BailReason::DepthMismatch {
            tag: "div".into(),
            expected: 1,
            actual: 0,
        },
        BailReason::EofWithOpenBlock { open_count: 3 },
        BailReason::LiteralLt { offset: 42 },
        BailReason::Cdata { offset: 10 },
        BailReason::UnknownCustomElement {
            name: "x-button".into(),
            offset: 0,
        },
    ];
    for reason in &reasons {
        let s = reason.to_string();
        assert!(!s.is_empty(), "Display for {reason:?} produced empty string");
    }
}

#[test]
fn bail_reason_display_contains_contextual_info() {
    let r = BailReason::DepthMismatch {
        tag: "section".into(),
        expected: 1,
        actual: 0,
    };
    let s = r.to_string();
    assert!(s.contains("section"), "Display should contain tag name: {s}");

    let r2 = BailReason::LiteralLt { offset: 99 };
    let s2 = r2.to_string();
    assert!(s2.contains("99"), "Display should contain offset: {s2}");

    let r3 = BailReason::EofWithOpenBlock { open_count: 5 };
    let s3 = r3.to_string();
    assert!(s3.contains('5'), "Display should contain open_count: {s3}");
}

// ~keep ── Phase D': preprocessing skip (nav / nav-hinted block elements) ────────────

#[test]
fn nav_subtree_skipped() {
    let html = "<nav>content</nav>";
    assert_eq!(force_tier1(html), tier2(html));
    assert_eq!(force_tier1(html), "");
}

#[test]
fn header_with_nav_hint_skipped() {
    let html = "<header class=\"site-header\">content</header>";
    assert_eq!(force_tier1(html), tier2(html));
    assert_eq!(force_tier1(html), "");
}

#[test]
fn header_without_nav_hint_kept() {
    let html = "<header><h1>Title</h1></header>";
    assert_eq!(force_tier1(html), tier2(html));
    assert!(
        !force_tier1(html).is_empty(),
        "plain <header> content must not be dropped"
    );
}

#[test]
fn aside_with_role_navigation_skipped() {
    let html = "<aside role=\"navigation\">content</aside>";
    assert_eq!(force_tier1(html), tier2(html));
    assert_eq!(force_tier1(html), "");
}
