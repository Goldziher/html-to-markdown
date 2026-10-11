// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #493: an anchor wrapping a block that holds another anchor was
//! emitted twice, once with an empty label -- `<a href="/o"><div><a href="/i">Inner</a></div></a>`
//! rendered `[](/o)` and then `[](/o)[Inner](/i)`.
//!
//! html5ever's adoption agency legitimately closes the outer `<a>` at the `<div>` and
//! reconstructs it inside; the repaired DOM is a browser's. The clone is created from the
//! original start-tag token's attributes, so `converter::anchor_origin` stamps every `<a>` start
//! tag with an origin id before the tree builder sees it and, on the repaired tree, unwraps the
//! members of a split origin that have no content of their own -- keeping the authored one when
//! none has, so a destination is never lost. Nothing changes for input the repair never runs on,
//! and an origin the repair leaves whole is untouched.

use html_to_markdown_rs::{ConversionOptions, convert};

const ISSUE_493: &str = r#"<a href="/o"><div><a href="/i">Inner</a></div></a>"#;

fn content(html: &str) -> String {
    convert(html, Some(ConversionOptions::default()))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

#[test]
fn should_emit_the_split_outer_anchor_once_when_no_half_has_content() {
    let out = content(ISSUE_493);
    assert_eq!(out, "[](/o)\n\n[Inner](/i)\n", "actual: {out:?}");
}

#[test]
fn should_keep_the_authored_half_that_carries_text() {
    let out = content(r#"<a href="/o">Text<div><a href="/i">Inner</a></div></a>"#);
    assert_eq!(out, "[Text](/o)\n\n[Inner](/i)\n", "actual: {out:?}");
}

#[test]
fn should_keep_the_reconstructed_half_that_carries_text_and_drop_the_empty_authored_one() {
    let out = content(r#"<a href="/o"><div>Text<a href="/i">Inner</a></div></a>"#);
    assert_eq!(out, "[Text](/o)[Inner](/i)\n", "actual: {out:?}");
}

#[test]
fn should_not_treat_the_inner_anchor_as_the_clone_own_content() {
    let out = content(r#"<a href="/o"><div><span><a href="/i">Inner</a></span></div></a>"#);
    assert_eq!(out, "[](/o)\n\n[Inner](/i)\n", "actual: {out:?}");
}

#[test]
fn should_keep_an_image_bearing_half() {
    let out = content(r#"<a href="/o"><div><img alt="A" src="s"><a href="/i">Inner</a></div></a>"#);
    assert_eq!(out, "[![A](s)](/o)[Inner](/i)\n", "actual: {out:?}");
}

#[test]
fn should_leave_authored_duplicate_empty_anchors_alone() {
    let out = content(r#"<a href="/o"></a><div><a href="/o"></a>real</div>"#);
    assert_eq!(out, "[](/o)\n\n[](/o)real\n", "actual: {out:?}");
}

#[test]
fn should_still_render_a_genuinely_empty_anchor_as_commonmark_does() {
    let out = content(r#"<a href="./target.md"></a>"#);
    assert_eq!(out, "[](./target.md)\n", "actual: {out:?}");
}

#[test]
fn should_not_change_a_nested_anchor_with_no_block_boundary() {
    let out = content(r#"<a href="/o">Outer <a href="/i">Inner</a></a>"#);
    assert_eq!(out, "[Outer](/o) [Inner](/i)\n", "actual: {out:?}");
}

#[test]
fn should_still_inline_a_block_only_anchor() {
    let out = content(r#"<a href="/o"><div>Text</div></a>"#);
    assert_eq!(out, "[Text](/o)\n", "actual: {out:?}");
}
