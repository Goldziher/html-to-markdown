// ~keep The inner attribute below is a crate-level Rust attribute, not a shell shebang.
#![allow(missing_docs)]

//! HTML5 *bogus comments* must render as nothing, like any other comment.
//!
//! The tokenizer enters the bogus-comment state from three places, and each produces a
//! comment token that renders as nothing:
//!
//! - `<?` — HTML has no processing instructions, so `<?php … ?>` is a comment
//!   (<https://html.spec.whatwg.org/#tag-open-state>).
//! - `<!` not beginning `--`, `DOCTYPE`, or `[CDATA[` — e.g. `<!bogus>`, and Word's
//!   downlevel-*revealed* conditional comments `<![if !vml]> … <![endif]>`.
//! - `</` followed by anything that is not an ASCII letter — `</3>`, `</ >`.
//!
//! All of these used to leak their text into the output, which was doubly wrong: real
//! comments already convert to nothing, so identical constructs rendered differently
//! depending only on which tokenizer state they happened to reach.

use html_to_markdown_rs::options::ConversionOptions;

fn convert(html: &str) -> String {
    html_to_markdown_rs::convert(html, Some(ConversionOptions::default()))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

#[test]
fn should_drop_a_question_mark_bogus_comment() {
    for (html, expected) in [
        ("<?php echo 1; ?>", ""),
        ("<?php echo 1; ?>text", "text\n"),
        ("<p>a<?php echo 1; ?>b</p>", "ab\n"),
        (r#"<?xml version="1.0"?>"#, ""),
        ("a<?b>c", "ac\n"),
        ("<?>", ""),
        // ~keep No `>` at all: the bogus-comment state runs to end of input, so everything
        // ~keep after `<?` is part of the comment.
        ("<?", ""),
        ("<p>keep</p><?unterminated", "keep\n"),
    ] {
        assert_eq!(convert(html), expected, "input: {html}");
    }
}

#[test]
fn should_drop_a_bang_bogus_comment_but_keep_doctype_handling() {
    for (html, expected) in [
        ("<!bogus>", ""),
        ("<!bogus>text", "text\n"),
        ("<!DOCTYPE html><p>x</p>", "x\n"),
        ("<!doctype html><p>x</p>", "x\n"),
    ] {
        assert_eq!(convert(html), expected, "input: {html}");
    }
}

#[test]
fn should_drop_an_end_tag_that_does_not_name_an_element() {
    for (html, expected) in [("</3>", ""), ("</ >text", "text\n"), ("</3>text", "text\n")] {
        assert_eq!(convert(html), expected, "input: {html}");
    }
}

#[test]
fn should_drop_word_downlevel_revealed_conditional_comments() {
    // ~keep Microsoft Word emits these around every image and footnote. They are not wrapped
    // ~keep in `<!--`, so they are bogus comments rather than real ones, and they used to
    // ~keep surface as literal `<![if !vml]>` noise around the content they bracket.
    assert_eq!(
        convert(r#"<![if !vml]><img src="i.gif" alt=""><![endif]>"#),
        "![](i.gif)\n"
    );
    assert_eq!(convert("<![if !supportFootnotes]>[1]<![endif]>"), "[1]\n");
}

#[test]
fn should_not_break_a_real_conditional_comment_by_eating_its_terminator() {
    // ~keep The regression this class of fix is most likely to cause, and did during
    // ~keep development. A downlevel-HIDDEN conditional comment is a real comment: it opens
    // ~keep with `<!--` and closes with the `-->` at the end of `<![endif]-->`. That
    // ~keep `<![endif]` looks exactly like a bogus comment, so stripping it removes the
    // ~keep comment's own terminator, leaving it unterminated and swallowing the entire rest
    // ~keep of the document. Word HTML is full of these -- it is why
    // ~keep `issue_190_regressions.rs`'s sjsu fixture went blank.
    assert_eq!(
        convert("<!--[if gte mso 9]><xml>junk</xml><![endif]--><p>kept</p>"),
        "kept\n"
    );
    assert_eq!(
        convert("<!--[if !mso]><style>x</style><![endif]--><p>kept</p>"),
        "kept\n"
    );
}

#[test]
fn should_not_touch_a_bogus_comment_lookalike_inside_an_attribute_value() {
    // ~keep Attribute values are not markup. Scanning into a tag would let `<?`/`<!`/`</`
    // ~keep inside a quoted value be mistaken for a bogus comment start, corrupting the tag.
    assert_eq!(
        convert(r#"<a href="x?a=1" title="a<?b>c">link</a>"#),
        "[link](x?a=1 \"a<?b>c\")\n"
    );
    assert_eq!(convert(r"<a title='single<?php ?>quote'>l</a>"), "l\n");
}

#[test]
fn should_drop_html_cdata_without_corrupting_foreign_content() {
    let svg = convert("<svg><![CDATA[<not-a-tag>]]></svg><p>after</p>");
    assert!(svg.contains("after"), "content after the SVG must survive: {svg:?}");
    assert_eq!(convert("<![CDATA[x]]>"), "");
}

#[test]
fn should_still_drop_real_comments() {
    assert_eq!(convert("<!-- real comment -->"), "");
    assert_eq!(convert("<p>a<!-- c -->b</p>"), "ab\n");
}
