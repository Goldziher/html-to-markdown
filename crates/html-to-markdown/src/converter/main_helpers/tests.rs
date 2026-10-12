//! Unit tests of the conversion helpers: the tree repair, the head repair and the output cleanup.

use super::*;

#[test]
fn should_give_the_tree_and_the_rest_of_the_input_when_the_repair_passes_the_depth_limit() {
    let deep = format!("{}x", "<div>".repeat(600));

    let (page, limit) = repair_with_html5ever_bounded(&deep).expect("a page");

    assert_eq!(limit, Some(crate::types::WarningKind::DepthLimitExceeded));
    assert_eq!(page.matches("<div>").count(), 600);
    assert!(
        page.ends_with(&format!("{}x", "<div>".repeat(80))),
        "{}",
        &page[page.len() - 40..]
    );
    assert!(repair_with_html5ever_bounded("<div><p>one</div>tail").is_some_and(|(_, limit)| limit.is_none()));
}

#[test]
fn should_preserve_head_repair_decisions_at_prefix_boundaries() {
    for html in [
        "<head></head><p title=\"<title>x\">é🦀</p>",
        "<!-- é🦀 --><head></head><p>body</p>",
        "<tit<head></head><p>body</p>",
        "<title<head></head><p>body</p>",
    ] {
        assert_eq!(
            repair_head_content_outside_head(html, HiddenContent::Drop),
            None,
            "{html}"
        );
    }
    for prefix in ["", "<!-- é🦀 -->"] {
        let html = format!("{prefix}<title>First</title><head><title>Second</title></head><p>x</p>");
        let (repaired, limit) =
            repair_head_content_outside_head(&html, HiddenContent::Drop).expect("repair implicit head");
        // ~keep The repair writes no `<html>` and no `<body>` that the input has no tag for.
        assert_eq!(
            repaired,
            format!("{prefix}<head><title>First</title><title>Second</title></head><p>x</p>")
        );
        assert_eq!(limit, None, "{html}");
    }
}

#[test]
fn should_write_the_content_of_a_template_back_in_the_head_repair_of_a_choice_that_keeps_it() {
    // ~keep The pass that removes the tags of a kept element reads the page after this repair.
    let page = |html: &str, choice| repair_head_content_outside_head(html, choice).map(|(page, _)| page);
    let html =
        "<title>t</title><template><meta></template><p>a</p><template shadowrootmode=open><p>b &amp; c</p></template>";
    let emptied =
        r#"<head><title>t</title><template></template></head><p>a</p><template shadowrootmode="open"></template>"#;
    assert_eq!(page(html, HiddenContent::Drop).as_deref(), Some(emptied));
    let kept = [
        "<head><title>t</title><template><meta></template></head><p>a</p>",
        r#"<template shadowrootmode="open"><p>b &amp; c</p></template>"#,
    ]
    .concat();
    assert_eq!(page(html, HiddenContent::Reachable), Some(kept.clone()));
    assert_eq!(page(html, HiddenContent::All), Some(kept));
    // ~keep The repair of the second parse reads the page after that pass: it writes no content.
    assert_eq!(
        repair_with_html5ever_bounded("<div><p>one</div><template><p>two</p></template>").map(|(page, _)| page),
        Some("<head></head><div><p>one</p></div><template></template>".to_owned())
    );
}

#[test]
fn should_limit_head_content_searches_to_input_before_the_head() {
    let html = format!("<head></head>{}", "<p>body</p>".repeat(10_000));
    START_TAG_CANDIDATE_CHECKS.with(|checks| checks.set(0));

    assert_eq!(repair_head_content_outside_head(&html, HiddenContent::Drop), None);
    // ~keep 20,002 checks find the last `<head>` from the end; the head content search reads
    // ~keep the empty prefix and adds none. The search for a title after `</head>` adds 2
    // ~keep checks to find `</head>` and one for each of the 20,001 tags from there.
    START_TAG_CANDIDATE_CHECKS.with(|checks| assert_eq!(checks.get(), 40_005));
}

#[test]
fn should_scan_only_markup_candidates_when_finding_head_tags() {
    let padding = "plain text ".repeat(10_000);
    let html = format!("{padding}<head>{padding}</head>{padding}<HEAD >{padding}");
    super::START_TAG_CANDIDATE_CHECKS.with(|checks| checks.set(0));

    assert_eq!(super::first_start_tag(html.as_bytes(), b"head"), Some(padding.len()));
    assert_eq!(
        super::last_start_tag(html.as_bytes(), b"head"),
        Some(padding.len() * 3 + "<head></head>".len())
    );
    super::START_TAG_CANDIDATE_CHECKS.with(|checks| assert_eq!(checks.get(), 2));
}

#[test]
fn test_is_ignorable_before_head() {
    assert!(is_ignorable_before_head("", false));
    assert!(is_ignorable_before_head("   \n\t", false));
    assert!(is_ignorable_before_head("&nbsp;", false));
    assert!(is_ignorable_before_head("&#160;", false));
    assert!(!is_ignorable_before_head("hello", false));
    assert!(!is_ignorable_before_head("x", false));
    assert!(!is_ignorable_before_head("\u{FFFD}\u{FFFD}", false));
    // ~keep Any text sitting directly in front of the document's own `<html>` tag is
    // ~keep forgiven, garbage or genuine prose alike (#527 regression: a mangled byte
    // ~keep order mark reads as ordinary non-whitespace text by this point).
    assert!(is_ignorable_before_head("\u{FFFD}\u{FFFD}", true));
    assert!(is_ignorable_before_head("hello", true));
}

fn cleaned(markdown: &str, style: CodeBlockStyle) -> String {
    let mut output = markdown.to_owned();
    trim_line_end_whitespace(&mut output, style);
    collapse_excess_blank_lines(&mut output, style);
    output
}

#[test]
fn should_clean_the_prose_and_not_the_code_of_an_indented_style_document() {
    let cases = [
        // A fence line in indented code opens nothing: the prose after it is cleaned.
        (
            "    a \t\n    ```\n\n\n\n    b\n\n\n\nx \t\n\n\n\ny\n",
            "    a \t\n    ```\n\n\n\n    b\n\nx\n\ny\n",
        ),
        // The third level of a list is indented by four columns and is not code.
        (
            "- a\n  - b\n    - c \t\n\n\n\n    - d \t\n",
            "- a\n  - b\n    - c\n\n    - d\n",
        ),
        // Code in a list item is indented by four columns past the content of the item.
        (
            "1. t\n\n       a \t\n\n\n\n       b\n\n\n\n   u \t\n",
            "1. t\n\n       a \t\n\n\n\n       b\n\n   u\n",
        ),
        // A code block that starts on the line of the item marker.
        ("-     a \t\n\n\n\n      b\n", "-     a \t\n\n\n\n      b\n"),
        // The blank lines between the code blocks of two items belong to neither.
        ("-     a\n\n\n\n-     b\n", "-     a\n\n-     b\n"),
        // Code in a block quote, and prose after the quote.
        (
            ">     a \t\n>\n>\n>\n>     b\n\n\n\nx \t\n",
            ">     a \t\n>\n>\n>\n>     b\n\nx\n",
        ),
        // An indented line that continues a paragraph is not code.
        ("x\n    y \t\n", "x\n    y\n"),
        // A tab is four columns of indentation.
        ("\ta \t\n\n\n\n\tb\n", "\ta \t\n\n\n\n\tb\n"),
    ];
    for (markdown, expected) in cases {
        assert_eq!(cleaned(markdown, CodeBlockStyle::Indented), expected, "{markdown:?}");
    }
}

#[test]
fn should_open_a_block_only_on_the_fence_of_the_configured_style() {
    let markdown = "~~~\na \t\n\n\n\nb\n~~~\n";
    assert_eq!(cleaned(markdown, CodeBlockStyle::Tildes), markdown);
    assert_eq!(cleaned(markdown, CodeBlockStyle::Backticks), "~~~\na\n\nb\n~~~\n");
}

#[test]
fn test_trim_line_end_whitespace() {
    let mut s = String::new();
    trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
    assert_eq!("", s.as_str());

    let mut s = "\t\n\t\n".to_owned();
    trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
    assert_eq!("", s.as_str());

    let mut s = "hello, world  ".to_owned();
    trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
    assert_eq!("hello, world  \n", s.as_str());

    let mut s = "hello, world  \n".to_owned();
    trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
    assert_eq!("hello, world  \n", s.as_str());

    let mut s = "hello, world  ".to_owned();
    trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
    assert_eq!("hello, world  \n", s.as_str());

    let mut s = "hello, world  \n\n\n".to_owned();
    trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
    assert_eq!("hello, world  \n", s.as_str());

    let mut s = "hello  \n- world\n".to_owned();
    trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
    assert_eq!("hello  \n- world\n", s.as_str());

    let mut s = "hello, world\t\t  ".to_owned();
    trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
    assert_eq!("hello, world  \n", s.as_str());

    let mut s = "hello, world\t\t  \n.abc def \t \t".to_owned();
    trim_line_end_whitespace(&mut s, CodeBlockStyle::Backticks);
    assert_eq!("hello, world  \n.abc def\n", s.as_str());
}

#[test]
fn test_is_ascii_whitespace_only() {
    assert!(is_ascii_whitespace_only(""));
    assert!(is_ascii_whitespace_only(" \t\n\r"));
    assert!(is_ascii_whitespace_only("\n\n"));
    // ~keep A decoded `&nbsp;` trims to empty under `str::trim`'s Unicode-aware
    // ~keep definition but must NOT be treated as safe-to-drop formatting whitespace.
    assert!(!is_ascii_whitespace_only("\u{a0}"));
    assert!(!is_ascii_whitespace_only("\n\u{a0}"));
    assert!(!is_ascii_whitespace_only("a"));
    assert!(!is_ascii_whitespace_only(" a "));
}

#[test]
fn test_has_custom_element_tags_ignores_comments() {
    // ~keep Regression for the bug this module fixed: a comment's own text
    // ~keep (`!--c--`) contains a hyphen and must not be mistaken for a tag name.
    assert!(!has_custom_element_tags("<!--c-->"));
    assert!(!has_custom_element_tags("<p><!-- a routine comment --></p>"));
    assert!(!has_custom_element_tags("<!-- multiple -- dashes -- here -->"));
}

#[test]
fn should_detect_namespaced_elements_for_html5_repair() {
    assert!(has_custom_element_tags("<o:p><P></o:p></P>"));
    assert!(!has_custom_element_tags("<o:p><P></P></o:p>"));
    assert!(!has_custom_element_tags("<o:p>Office text</o:p>"));
    assert!(!has_custom_element_tags("<o:p/><P>text</P>"));
    assert!(!has_custom_element_tags("<!-- <o:p>comment</o:p> -->"));
    assert!(!has_custom_element_tags("<div title='o:p'>text</div>"));
}

#[test]
fn test_has_custom_element_tags_detects_real_custom_element() {
    assert!(has_custom_element_tags("<my-widget></my-widget>"));
    assert!(has_custom_element_tags("<p></p><my-widget></my-widget>"));
    assert!(has_custom_element_tags("<div></my-widget>"));
}

#[test]
fn test_has_custom_element_tags_detects_custom_element_alongside_comment() {
    assert!(has_custom_element_tags(
        "<!-- a routine comment --><my-widget>hi</my-widget>"
    ));
    assert!(has_custom_element_tags(
        "<my-widget><!-- nested comment -->hi</my-widget>"
    ));
}

#[test]
fn test_has_custom_element_tags_ignores_comment_mentioning_custom_element() {
    // ~keep A custom-element-shaped tag written INSIDE a comment is text, not markup.
    assert!(!has_custom_element_tags("<!-- <my-widget>hi</my-widget> -->"));
}

#[test]
fn test_has_custom_element_tags_ignores_processing_instruction() {
    assert!(!has_custom_element_tags("<?php echo 'a-b'; ?>"));
    assert!(!has_custom_element_tags(
        "<?xml-stylesheet type=\"text/xsl\" href=\"a-b.xsl\"?>"
    ));
}

#[test]
fn test_has_custom_element_tags_ignores_cdata_section() {
    assert!(!has_custom_element_tags(
        "<svg><![CDATA[<my-widget>a-b</my-widget>]]></svg>"
    ));
}

#[test]
fn test_has_custom_element_tags_ignores_doctype() {
    assert!(!has_custom_element_tags("<!doctype html>"));
    assert!(!has_custom_element_tags("<!DOCTYPE html><p>no custom elements</p>"));
    // ~keep A real custom element after the doctype must still be detected.
    assert!(has_custom_element_tags("<!DOCTYPE html><my-widget></my-widget>"));
}

#[test]
fn test_has_custom_element_tags_rejects_whitespace_before_name() {
    // ~keep HTML5 does not skip whitespace after `<`/`</` before the tag name:
    // ~keep `< div>` and `</ div>` are plain text, not tags, in a real parser.
    assert!(!has_custom_element_tags("< my-widget>"));
    assert!(!has_custom_element_tags("</ my-widget>"));
}
