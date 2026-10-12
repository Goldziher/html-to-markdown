//! Integration tests for the prescan module.
//!
//! Every test calls `prescan::run()` directly — only visible when
//! `cfg(any(test, feature = "testkit"))` is true — so the whole file is
//! gated on the `testkit` feature.
#![cfg(feature = "testkit")]

use html_to_markdown_rs::prescan::{self, PrescanReport};

fn report(html: &str) -> PrescanReport {
    prescan::run(html).1
}

fn cleaned(html: &str) -> String {
    prescan::run(html).0.into_owned()
}

#[test]
fn custom_elements_flagged() {
    let r = report("<my-thing attr=\"1\"></my-thing>");
    assert!(
        r.had_custom_elements,
        "tag name with hyphen should set had_custom_elements"
    );
}

#[test]
fn standard_tags_not_custom() {
    let r = report("<div><p class=\"x\"></p></div>");
    assert!(!r.had_custom_elements, "standard tags must not set had_custom_elements");
}

#[test]
fn x_foo_custom_element_flagged() {
    let r = report("<x-foo></x-foo>");
    assert!(r.had_custom_elements);
}

#[test]
fn cdata_flagged() {
    let r = report("<svg><![CDATA[raw content]]></svg>");
    assert!(r.had_cdata, "CDATA section should set had_cdata");
}

#[test]
fn no_cdata_clean() {
    let r = report("<svg><path d=\"M0 0\"/></svg>");
    assert!(!r.had_cdata);
}

#[test]
fn unescaped_lt_flagged() {
    let r = report("<p>a < b</p>");
    assert!(
        r.had_unescaped_lt,
        "bare `<` not followed by valid tag char should set had_unescaped_lt"
    );
}

#[test]
fn unescaped_lt_clean() {
    let r = report("<p>a &lt; b</p>");
    assert!(
        !r.had_unescaped_lt,
        "pre-escaped &lt; should not trigger had_unescaped_lt"
    );
}

#[test]
fn unescaped_lt_at_eof() {
    let r = report("<p>text<");
    assert!(r.had_unescaped_lt, "<p>text< should set had_unescaped_lt");
}

#[test]
fn script_flag() {
    let r = report("<script>alert(1)</script>");
    assert!(r.has_script_or_style);
}

#[test]
fn style_flag() {
    let r = report("<style>body { color: red; }</style>");
    assert!(r.has_script_or_style);
}

#[test]
fn no_script_or_style_clean() {
    let r = report("<div><p>text</p></div>");
    assert!(!r.has_script_or_style);
}

// ~keep ── has_svg ───────────────────────────────────────────────────────────────────

#[test]
fn svg_flag() {
    let r = report("<svg viewBox=\"0 0 10 10\"></svg>");
    assert!(r.has_svg, "<svg> should set has_svg");
}

#[test]
fn no_svg_clean() {
    let r = report("<div><p>text</p></div>");
    assert!(!r.has_svg);
}

#[test]
fn empty_comment_normalised() {
    let out = cleaned("<!---->");
    assert_eq!(out, "<!-- -->");
}

#[test]
fn self_closing_br_normalised() {
    let out = cleaned("<br/>");
    assert_eq!(out, "<br>");
}

#[test]
fn script_content_stripped() {
    let out = cleaned("<p>before</p><script>alert(1)</script><p>after</p>");
    assert!(out.contains("<p>before</p>"));
    assert!(out.contains("<p>after</p>"));
    assert!(!out.contains("alert(1)"), "script body should be stripped");
}

#[test]
fn doctype_removed() {
    let out = cleaned("<!DOCTYPE html><html><body>hi</body></html>");
    assert!(
        !out.to_lowercase().contains("doctype"),
        "DOCTYPE should be stripped; got: {out:?}"
    );
}

#[test]
fn cow_borrowed_when_no_change() {
    let html = "<p>hello world</p>";
    let (cow, _) = prescan::run(html);
    assert!(
        matches!(cow, std::borrow::Cow::Borrowed(_)),
        "no-op input should return Cow::Borrowed"
    );
}
