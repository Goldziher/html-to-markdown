//! Leading whitespace at the very start of the document must not survive conversion.
//!
//! `CommonMark` 4.8 treats whitespace before the first real content of a block as
//! insignificant, so any conforming parser strips it on reparse. Tier-2 only ever
//! protected this for `<p>` (`ctx.in_paragraph && output.len() == ctx.block_content_start`,
//! checked only in the whitespace-only text-node branch). An unknown/custom tag, a
//! `<style>`/`<textarea>`, or a stray closing tag at document start never sets
//! `in_paragraph`, and the main (non-whitespace-only) branch never consulted that guard
//! at all — so a leading space or blank line before their first real text child leaked
//! into the output verbatim, making `convert -> render -> convert` unstable.
//!
//! These are regression tests for `Context::at_fresh_block_start`, the shared
//! `Rc<Cell<bool>>` signal `text_node.rs` now consults instead: unlike comparing
//! `output.len()` to `ctx.block_content_start`, it stays correct even when `output` is
//! a fresh local `String` an inline wrapper (`sub`/`sup`/`em`/...) is building its
//! content into, which is why `test_subscript_leading_whitespace` /
//! `test_superscript_leading_whitespace` in `integration_test.rs` still pass unchanged.

#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, convert};

fn to_markdown(html: &str) -> String {
    convert(html, None).unwrap().content.unwrap_or_default()
}

fn to_markdown_escaped(html: &str) -> String {
    let options = ConversionOptions {
        escape_misc: true,
        escape_asterisks: true,
        escape_underscores: true,
        ..Default::default()
    };
    convert(html, Some(options)).unwrap().content.unwrap_or_default()
}

#[test]
fn should_drop_leading_whitespace_before_a_custom_tag_at_document_start() {
    // ~keep CommonMark spec example 163: an unknown/custom tag never sets `in_paragraph`.
    assert_eq!(to_markdown_escaped("<Warning>\n*bar*\n</Warning>\n"), "\\*bar\\*\n");
}

#[test]
fn should_drop_leading_whitespace_after_a_stray_closing_tag_at_document_start() {
    // ~keep CommonMark spec example 151: a stray closing tag leaves a bare leading-newline
    // ~keep text node as the very first thing in the document.
    assert_eq!(to_markdown_escaped("</div>\n*foo*\n"), "\\*foo\\*\n");
}

#[test]
fn should_drop_leading_whitespace_inside_a_textarea_at_document_start() {
    // ~keep CommonMark spec example 171.
    assert_eq!(
        to_markdown_escaped("<textarea>\n\n*foo*\n\n_bar_\n\n</textarea>\n"),
        "\\*foo\\*\n\n\\_bar\\_\n"
    );
}

#[test]
fn should_drop_leading_whitespace_after_a_style_element_at_document_start() {
    // ~keep The page of CommonMark spec example 173 with the `style` closed: the text after
    // ~keep the element is the first text of the document.
    assert_eq!(
        to_markdown_escaped("<style\n  type=\"text/css\">\n</style>\n\nfoo\n"),
        "foo\n"
    );
}

#[test]
fn should_show_no_text_of_a_style_tag_with_no_end_tag() {
    // ~keep CommonMark spec example 173 as written. The `style` has no end tag, so the HTML
    // ~keep tree builder reads the rest of the input as its text, and a browser shows none of it.
    assert_eq!(to_markdown_escaped("<style\n  type=\"text/css\">\n\nfoo\n"), "");
}

#[test]
fn should_drop_a_whitespace_only_text_node_with_no_newline_before_an_unknown_tag() {
    // ~keep CommonMark spec example 150: a single leading space (no newline at all) before
    // ~keep an unknown tag is a *separate* code path from the had_newlines branch above --
    // ~keep it falls through to the verbatim-push fallback and was never guarded before.
    assert_eq!(
        to_markdown_escaped(" <div>\n  *hello*\n         <foo><a>\n"),
        "\\*hello\\*\n"
    );
}

#[test]
fn should_drop_leading_whitespace_before_an_entity_reference_paragraph() {
    // ~keep CommonMark spec example 40: a leading tab inside `<p>` at document start went
    // ~keep through the main (non-whitespace-only) branch, which never consulted the
    // ~keep paragraph-start guard at all (that guard lived only in the whitespace-only
    // ~keep branch).
    assert_eq!(to_markdown("<p>\tfoo</p>\n"), "foo\n");
}

#[test]
fn should_still_preserve_a_leading_space_inside_subscript_after_real_text() {
    // ~keep Regression guard for the fix that was tried and reverted before this one: a
    // ~keep naive `output.is_empty()` check also fires for the *scratch buffer* `<sub>`
    // ~keep builds its content into, dropping a legitimately needed separating space.
    // ~keep `at_fresh_block_start` is shared via `Rc<Cell<bool>>`, so it already reflects
    // ~keep "hello" having been written to the real document and is unaffected by the
    // ~keep scratch buffer being empty.
    let opts = ConversionOptions {
        sub_symbol: "~".to_string(),
        ..Default::default()
    };
    assert_eq!(
        convert("<p>hello<sub> world</sub></p>", Some(opts))
            .unwrap()
            .content
            .unwrap_or_default(),
        "hello ~world~\n"
    );
}
