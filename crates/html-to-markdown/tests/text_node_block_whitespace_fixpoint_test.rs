//! Regression coverage for the leading-whitespace-on-continuation-lines fixpoint defect
//! (`CommonMark` spec example 182: `<![CDATA[ ... ]]>` whose body is indented source code).
//!
//! `normalize_whitespace_cow` used to collapse a run of spaces/tabs immediately after an
//! embedded `\n` down to a single space. That single space was not a fixed point: a
//! `CommonMark`-compliant parser drops a paragraph continuation line's leading whitespace
//! entirely on reparse (spec 4.9), so re-converting the rendered Markdown lost the space and
//! shrank the run again -- forever, on every subsequent pass. `text_node.rs` now runs
//! `normalize_block_whitespace_cow` on the already-boundary-trimmed core, collapsing such a
//! run straight to nothing so the very first pass already matches what the round trip forces.

#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn convert_with(html: &str, options: &ConversionOptions) -> String {
    convert(html, Some(options.clone()))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn escaping_options() -> ConversionOptions {
    ConversionOptions {
        escape_misc: true,
        escape_asterisks: true,
        escape_underscores: true,
        ..Default::default()
    }
}

fn render_markdown_to_html(md: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.render.r#unsafe = true;
    comrak::markdown_to_html(md, &options)
}

/// Convert twice through the round trip (`html -> md1 -> html2 -> md2`) and assert
/// `md1 == md2` -- the direct idempotency check.
fn assert_round_trip_is_a_fixpoint(html: &str, options: &ConversionOptions, context: &str) {
    let md1 = convert_with(html, options);
    let html2 = render_markdown_to_html(&md1);
    let md2 = convert_with(&html2, options);
    assert_eq!(
        md1, md2,
        "{context}: round trip is not a fixpoint\nmd1: {md1:?}\nmd2: {md2:?}"
    );
}

#[test]
fn commonmark_spec_example_182_reaches_a_conversion_fixpoint() {
    // ~keep The exact CommonMark spec example 182 input: a top-level CDATA section whose
    // ~keep body is indented source code, followed by a sibling paragraph.
    let html = concat!(
        "<![CDATA[\n",
        "function matchwo(a,b)\n",
        "{\n",
        "  if (a < b && a < 0) then {\n",
        "    return 1;\n",
        "\n",
        "  } else {\n",
        "\n",
        "    return 0;\n",
        "  }\n",
        "}\n",
        "]]>\n",
        "<p>okay</p>\n",
    );
    assert_round_trip_is_a_fixpoint(html, &escaping_options(), "spec example 182");
}

#[test]
fn indented_multiline_div_text_with_blank_lines_is_idempotent() {
    // ~keep A non-CDATA shape exercising the same code path: a single text node with
    // ~keep multiple indented continuation lines and embedded blank lines, directly under
    // ~keep a plain block element rather than parsed as a bogus-comment-adjacent CDATA body.
    let html = "<div>line one\n    line two\n\n    line three\n  line four</div>";
    assert_round_trip_is_a_fixpoint(html, &escaping_options(), "indented multiline div text");
}

#[test]
fn converting_twice_produces_identical_output_for_indented_continuation_lines() {
    // ~keep Literal "convert twice" framing: the second conversion is fed the first
    // ~keep conversion's own Markdown rendered back to HTML, and must reproduce it exactly.
    let html = "<p>foo\n    bar\n\n    baz</p>";
    let options = escaping_options();
    let first = convert_with(html, &options);
    let rendered = render_markdown_to_html(&first);
    let second = convert_with(&rendered, &options);
    assert_eq!(first, second, "first: {first:?}\nsecond: {second:?}");
}

#[test]
fn a_cdata_section_is_dropped_before_either_tier_reads_it() {
    // ~keep A CDATA section in HTML content is a bogus comment: it is removed from the source
    // ~keep before a converter runs, so neither tier sees it and no fallback happens here.
    let html = concat!(
        "<![CDATA[\n",
        "function matchwo(a,b)\n",
        "{\n",
        "  if (a < b && a < 0) then {\n",
        "    return 1;\n",
        "\n",
        "  } else {\n",
        "\n",
        "    return 0;\n",
        "  }\n",
        "}\n",
        "]]>\n",
        "<p>okay</p>\n",
    );
    let mut options = escaping_options();
    options.tier_strategy = TierStrategy::Tier1;
    let tier1_forced = convert_with(html, &options);
    options.tier_strategy = TierStrategy::Tier2;
    let tier2 = convert_with(html, &options);
    assert_eq!(tier1_forced, tier2, "both tiers must drop the CDATA section");
    assert_eq!(tier1_forced, "okay\n");
}

#[test]
fn br_in_heading_collapses_to_a_single_space_and_reaches_a_fixpoint() {
    // ~keep An ATX heading is single-line, so a `<br>` inside one can never carry a real
    // ~keep hard break -- any marker here is inherently lossy. The old two-space marker
    // ~keep survived the first conversion but a compliant renderer's own HTML whitespace
    // ~keep collapsing reduced it to one space on the next parse, so the round trip never
    // ~keep reached a fixpoint. `line_break.rs`'s `ctx.in_heading` branch now emits a
    // ~keep single space, matching what the renderer would collapse it to anyway (and
    // ~keep matching Tier-1's `close_heading`, which folds the same "  \n" marker down to
    // ~keep one space once the heading closes).
    let html = "<h5>foo<br>bar</h5>";
    let options = escaping_options();
    let converted = convert_with(html, &options);
    assert_eq!(converted, "##### foo bar\n");
    assert_round_trip_is_a_fixpoint(html, &options, "br inside heading");
}

#[test]
fn br_in_heading_matches_tier1_native_output() {
    use html_to_markdown_rs::HighlightStyle;
    use html_to_markdown_rs::prescan::PrescanReport;
    use html_to_markdown_rs::tier1;

    let html = "<h5>foo<br>bar</h5>";
    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..Default::default()
    };
    let report = PrescanReport::default();
    let tier1_output = tier1::run(html, &report, &options).expect("tier1 should handle a <br> inside a heading");
    let tier2_output = convert_with(html, &options);
    assert_eq!(tier1_output, tier2_output);
    assert_eq!(tier1_output, "##### foo bar\n");
}

#[test]
fn trailing_significant_whitespace_after_a_hard_break_is_preserved() {
    // ~keep A decoded `&nbsp;` is Unicode-whitespace by `str::trim`'s definition, so a text
    // ~keep node consisting only of a hard break's trailing newline plus one `&nbsp;`, with
    // ~keep no sibling following it, used to be dropped outright by the
    // ~keep no-next-sibling/no-separating-space-needed fallback in `text_node.rs`'s
    // ~keep `had_newlines` branch -- silently losing real content. Whether that newline was
    // ~keep present at all depended only on incidental HTML pretty-printing (a rendered
    // ~keep `<br>` is always followed by a literal newline before its next text node), so
    // ~keep the very same logical content survived the FIRST conversion (no newline yet)
    // ~keep but vanished on the SECOND (after a renderer inserted one), breaking the
    // ~keep round-trip fixpoint. A lone significant character now survives verbatim.
    let html = ";<br>&nbsp;";
    let options = escaping_options();
    let converted = convert_with(html, &options);
    assert_eq!(converted, ";  \n\u{a0}\n");
    assert_round_trip_is_a_fixpoint(html, &options, "trailing nbsp after a hard break");
}

#[test]
fn significant_whitespace_after_a_hard_break_survives_before_inline_content_too() {
    // ~keep Companion coverage for the same fix's sibling-is-inline branch: the lone
    // ~keep significant character must also survive when a next sibling exists and is
    // ~keep inline (here, `<em>`, a normal paired tag -- not a void element like `<br>`,
    // ~keep whose own adjacent-whitespace handling has a separate, unfixed defect noted
    // ~keep in the session report), rather than only in the no-next-sibling case.
    let html = ";<br>&nbsp;<em>x</em>";
    let options = escaping_options();
    let converted = convert_with(html, &options);
    assert!(
        converted.contains('\u{a0}'),
        "the nbsp before the <em> should survive: {converted:?}"
    );
    assert_round_trip_is_a_fixpoint(html, &options, "nbsp before inline content, after a hard break");
}

#[test]
fn significant_whitespace_between_two_hard_breaks_survives_a_round_trip() {
    // ~keep The void-element follow-up flagged above: a text node consisting solely of a
    // ~keep decoded `&nbsp;` sandwiched between two `<br>`s is whitespace-only by
    // ~keep `str::trim`'s Unicode-aware definition, so `block/paragraph.rs`'s
    // ~keep empty-inline-neighbour pre-filter (`is_empty_inline_element` matches `br`)
    // ~keep used to skip walking that child NODE ENTIRELY -- not merely collapse it --
    // ~keep whenever both its neighbours were themselves `<br>`/`hr`/`img`/`input`/`link`.
    // ~keep The first conversion never hits that filter (the source spells the nbsp as
    // ~keep the `&nbsp;` ENTITY, six plain ASCII bytes that do not trim to empty), but a
    // ~keep compliant HTML renderer serialises it back out as the literal `\u{a0}` byte,
    // ~keep which does -- so the character survived pass one and silently vanished on
    // ~keep pass two, breaking round-trip stability (real content loss, not just a
    // ~keep formatting difference). Fixed by checking `is_ascii_whitespace_only`
    // ~keep (main_helpers.rs) instead, which does not treat significant Unicode
    // ~keep whitespace as "safe to drop".
    let html = ";<br>&nbsp;<br>&amp;";
    let options = escaping_options();
    let md1 = convert_with(html, &options);
    assert_eq!(
        md1, ";  \n\u{a0}  \n\\&\n",
        "first conversion should already preserve the nbsp"
    );
    assert_round_trip_is_a_fixpoint(html, &options, "nbsp between two hard breaks");
}

#[test]
fn significant_whitespace_between_two_hard_breaks_matches_tier1_when_tier1_handles_it() {
    // ~keep With no escape overrides `router.rs::classify` allows Tier-1 to handle this
    // ~keep shape (unlike `significant_whitespace_between_two_hard_breaks_survives_a_
    // ~keep round_trip` above, whose `escape_misc`/`escape_asterisks`/`escape_underscores`
    // ~keep gate Tier-1 off entirely per the option table in `router.rs`). Tier-1's byte
    // ~keep scanner never had this bug in the first place: `raw_is_whitespace`
    // ~keep (scanner.rs) checks ASCII whitespace BYTES only, so a raw nbsp byte sequence
    // ~keep always fails that check and the text is routed through ordinary content
    // ~keep emission rather than the whitespace-collapse fast path. Pin that Tier-1 and
    // ~keep the now-fixed Tier-2 agree byte for byte on this shape.
    use html_to_markdown_rs::TierStrategy;
    use html_to_markdown_rs::prescan::PrescanReport;
    use html_to_markdown_rs::tier1;

    let html = "<p>a<br>&nbsp;<br>b</p>";
    let options = ConversionOptions {
        extract_metadata: false,
        ..Default::default()
    };

    let mut tier2_options = options.clone();
    tier2_options.tier_strategy = TierStrategy::Tier2;
    let tier2 = convert_with(html, &tier2_options);

    let mut tier1_options = options;
    tier1_options.tier_strategy = TierStrategy::Tier1;
    let report = PrescanReport::default();
    let tier1_forced = tier1::run(html, &report, &tier1_options).expect("Tier-1 should not bail on this shape");

    assert_eq!(tier1_forced, tier2, "Tier-1 and Tier-2 must agree byte-for-byte");
    assert!(
        tier2.contains('\u{a0}'),
        "the nbsp between the two <br>s must survive: {tier2:?}"
    );
}
