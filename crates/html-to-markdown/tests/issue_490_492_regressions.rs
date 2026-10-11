#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #490 (an `<a>` wrapping a `<table>` crushes the whole GFM table
//! into the link label instead of rendering it as a block) and issue #492 (`keep_inline_images_in`
//! is never consulted for `<a>` -- the option's `<a>` handler moved from the dead
//! `converter/inline/link.rs` to `handlers/link.rs`, and the option was left behind: headings
//! and layout cells honored it, `<a>` never did).
//!
//! Part C, an adjacent empty-anchor duplication defect surfaced while investigating #490
//! (`<a href="/o"><div><a href="/i">Inner</a></div></a>` emitted the outer link twice, once
//! empty), became issue #493 and is fixed at parse time rather than in the renderer: a renderer
//! rule that drops a childless `<a>` regresses `commonmark_compliance_test`'s mandatory spec
//! example 484 (`<a href="./target.md"></a>` -> `[](./target.md)`). See
//! `converter::anchor_origin` and `issue_493_split_anchor_collapse_test.rs`.

use html_to_markdown_rs::prescan;
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn content(html: &str) -> String {
    convert(html, Some(ConversionOptions::default()))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn content_with(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn tier1_run(html: &str, options: &ConversionOptions) -> Result<String, BailReason> {
    let (cleaned, report) = prescan::run(html);
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..options.clone()
    };
    tier1::run(cleaned.as_ref(), &report, &opts)
}

fn keep_options(keep: &[&str]) -> ConversionOptions {
    ConversionOptions {
        keep_inline_images_in: keep.iter().map(ToString::to_string).collect(),
        ..ConversionOptions::default()
    }
}

// ~keep ── Issue #490: <a> wrapping a <table> ───────────────────────────────────────────

#[test]
fn should_emit_the_wrapped_table_as_a_block_instead_of_crushing_it_into_the_label() {
    let html = concat!(
        r#"<a href="https://example.com/outer"><table><tr><td>"#,
        r#"<a href="https://example.com/inner">Inner</a></td></tr></table></a>"#
    );
    assert_eq!(
        content(html),
        "[https://example.com/outer](https://example.com/outer)\n\n\
         | [Inner](https://example.com/inner) |\n\
         | ---------------------------------- |\n"
    );
}

#[test]
fn should_emit_the_wrapped_table_as_a_block_when_the_anchor_has_no_inner_link() {
    let html = r#"<a href="/o"><table><tr><td>Cell</td></tr></table></a>"#;
    assert_eq!(content(html), "[/o](/o)\n\n| Cell |\n| ---- |\n");
}

#[test]
fn should_keep_inline_anchor_text_as_the_label_when_a_table_follows_it() {
    let html = r#"<a href="/o">Label<table><tr><td>Cell</td></tr></table></a>"#;
    assert_eq!(content(html), "[Label](/o)\n\n| Cell |\n| ---- |\n");
}

#[test]
fn should_keep_an_inline_image_in_the_label_when_a_table_follows_it() {
    let html = r#"<a href="/o"><img src="p.png" alt="A"><table><tr><td>Cell</td></tr></table></a>"#;
    assert_eq!(content(html), "[![A](p.png)](/o)\n\n| Cell |\n| ---- |\n");
}

#[test]
fn should_emit_a_table_buried_under_a_block_child_as_a_block() {
    let html = r#"<a href="/o"><div><table><tr><td>Cell</td></tr></table></div></a>"#;
    assert_eq!(content(html), "[/o](/o)\n\n| Cell |\n| ---- |\n");
}

#[test]
fn should_not_autolink_away_a_wrapped_table_whose_text_equals_the_href() {
    // ~keep Without `!emit_blocks_separately` gating `is_autolink`, `raw_text` (whole-subtree
    // ~keep text, including the deferred table's own cell text) equals `href` here, and the
    // ~keep table is silently dropped in favor of a bare autolink.
    let html = r#"<a href="https://x.com"><table><tr><td>https://x.com</td></tr></table></a>"#;
    assert_eq!(
        content(html),
        "[https://x.com](https://x.com)\n\n| https://x.com |\n| ------------- |\n"
    );
}

#[test]
fn should_still_inline_a_paragraph_only_anchor() {
    let html = r#"<a href="/o"><p>text</p></a>"#;
    assert_eq!(content(html), "[text](/o)\n");
}

#[test]
fn should_still_inline_a_div_only_anchor() {
    let html = r#"<a href="/o"><div>x</div></a>"#;
    assert_eq!(content(html), "[x](/o)\n");
}

#[test]
fn should_not_change_the_nested_sibling_anchor_repair_output() {
    // ~keep Issue #479 control: html5ever's adoption agency splits this into two sibling
    // ~keep anchors. No `<table>` is involved, so #490's new branch must not touch it.
    let html = r#"<a href="/o">Outer <a href="/i">Inner</a></a>"#;
    assert_eq!(content(html), "[Outer](/o) [Inner](/i)\n");
}

#[test]
fn should_still_crush_a_wrapped_table_inside_a_heading() {
    // ~keep `!ctx.in_heading` is load-bearing: `normalize_heading_text` folds `\n` to spaces,
    // ~keep so a block table inside a heading is no better than the crushed-label bug. This
    // ~keep case is deliberately left un-fixed (unchanged from pre-#490 output).
    let html = r#"<h1><a href="/o"><table><tr><td>Cell</td></tr></table></a></h1>"#;
    assert_eq!(content(html), "# [| Cell | | ---- |](/o)\n");
}

#[test]
fn should_bail_with_classifier_when_a_table_opens_inside_a_link() {
    // ~keep #490 needs no Tier-1 change: a `<table>` opening inside a link already bails to
    // ~keep Tier-2 (scanner.rs:1111-1128, `TagKind::Table` is one of the block kinds guarded
    // ~keep there) both before and after this fix. Assert that stays true.
    let html = r#"<a href="/o"><table><tr><td>Cell</td></tr></table></a>"#;
    let err = tier1_run(html, &ConversionOptions::default()).expect_err("a table opening inside a link must bail");
    assert!(
        matches!(err, BailReason::Classifier),
        "expected BailReason::Classifier, got {err:?}"
    );
}

// ~keep ── Issue #492: keep_inline_images_in never consulted for <a> ────────────────────

#[test]
fn should_keep_a_linked_image_as_markdown_when_a_is_in_keep_inline_images_in() {
    let html = r#"<a href="https://example.com/file.pdf"><img src="pdf.png"><p>Download PDF</p></a>"#;
    assert_eq!(
        content_with(html, keep_options(&["a"])),
        "[![](pdf.png) Download PDF](https://example.com/file.pdf)\n"
    );
}

#[test]
fn should_not_change_the_default_output_when_a_is_not_in_keep_inline_images_in() {
    // ~keep `link_allow_inline_images` is `false` whenever "a" is absent from
    // ~keep `keep_inline_images_in` (its only source), so the added `||` disjunct in
    // ~keep `image.rs`/`graphic.rs` is a no-op and this must be byte-identical to pre-#492.
    let html = r#"<a href="https://example.com/file.pdf"><img src="pdf.png"><p>Download PDF</p></a>"#;
    assert_eq!(content(html), "[Download PDF](https://example.com/file.pdf)\n");
}

#[test]
fn should_keep_an_image_in_a_block_free_anchor_regardless_of_the_keep_list() {
    // ~keep A block-free anchor already renders through the inline branch with
    // ~keep `convert_as_inline == false`, so `should_use_alt_text` is `false` regardless of
    // ~keep `keep_as_markdown` -- the image survives with or without "a" in the keep list.
    let html = r#"<a href="https://example.com/file.pdf"><img src="pdf.png">Download PDF</a>"#;
    let expected = "[![](pdf.png)Download PDF](https://example.com/file.pdf)\n";
    assert_eq!(content(html), expected, "default (no keep list) must be unaffected");
    assert_eq!(
        content_with(html, keep_options(&["a"])),
        expected,
        "keep=[a] must not change a block-free anchor's image either"
    );
}

#[test]
fn should_keep_a_linked_image_inside_a_non_matching_heading_when_a_is_in_the_keep_list() {
    // ~keep The Tier-2 divergence case: `link_allow_inline_images` is OR'd into
    // ~keep `keep_as_markdown` independent of any heading, so a matching `<a>` keeps the
    // ~keep image even though the wrapping `<h1>` itself is not in the keep list.
    let html = r#"<h1><a href="x"><img src="i.png" alt="A"></a></h1>"#;
    assert_eq!(content_with(html, keep_options(&["a"])), "# [![A](i.png)](x)\n");
    // ~keep Control: with no keep list at all, the pre-#492 heading-only rule still strips
    // ~keep to alt text -- confirms the divergence is caused by "a" being in the list, not
    // ~keep by some unrelated change to heading handling.
    assert_eq!(content(html), "# [A](x)\n");
}

#[test]
fn should_bail_with_classifier_when_a_paragraph_opens_inside_a_link() {
    // ~keep #492's actual reported input shape: Tier-1 bails on this (scanner.rs:1111-1128)
    // ~keep both before and after the fix, which is why the Tier-1 lockstep change only
    // ~keep needed to handle the heading-ancestor case, never this one.
    let html = r#"<a href="https://example.com/file.pdf"><img src="pdf.png"><p>Download PDF</p></a>"#;
    let err = tier1_run(html, &ConversionOptions::default()).expect_err("a paragraph opening inside a link must bail");
    assert!(
        matches!(err, BailReason::Classifier),
        "expected BailReason::Classifier, got {err:?}"
    );
}
