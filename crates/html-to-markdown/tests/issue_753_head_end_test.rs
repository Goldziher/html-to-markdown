//! Coverage for where the document head ends under `ConversionOptions::hidden_content`
//! (issue #753).
//!
//! ~keep A `<template>` or a `<noscript>` of the head holds metadata, so its content is never
//! ~keep written. The same element after the head holds text for a reader, and
//! ~keep `HiddenContent::All` writes it. The end tag of the head is optional: a browser ends the
//! ~keep head at the first text that is not white space and at the first element that the head
//! ~keep cannot hold.

#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HiddenContent, TierStrategy, convert};

/// The front matter of a page whose title is `t`.
const TITLE: &str = "---\ntitle: t\n---\n";

/// The tiers that write the body of a page whose head has no end tag.
const DOM_TIERS: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];

fn tiers() -> Vec<TierStrategy> {
    vec![
        TierStrategy::Auto,
        TierStrategy::Tier2,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
    ]
}

fn convert_in(html: &str, choice: HiddenContent, tier_strategy: TierStrategy) -> String {
    let options = ConversionOptions {
        hidden_content: choice,
        tier_strategy,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

/// A page whose `<template>` or `<noscript>` is after a head that ends by implication.
struct Kept {
    /// The page.
    html: &'static str,
    /// The page with the two tags of the element removed.
    unwrapped: &'static str,
    /// The output of [`HiddenContent::Drop`], which is the output of the base.
    drop: String,
    /// The output of [`HiddenContent::All`].
    all: String,
}

/// One line for each output of `row` that is wrong.
///
/// ~keep In every tier, `HiddenContent::Reachable` writes what `HiddenContent::Drop` writes: no
/// ~keep row has content that a reader can reach. `HiddenContent::All` converts the content of
/// ~keep the element where it is written, so it writes what the page with no such tags gives.
/// ~keep The exact outputs hold for the tiers that write the body of such a page.
fn wrong_outputs(row: &Kept) -> Vec<String> {
    let mut wrong = Vec::new();
    let mut check = |name: &str, tier: TierStrategy, output: &str, expected: &str| {
        if output != expected {
            wrong.push(format!(
                "{name} {tier:?} {}\n  wrote    {output:?}\n  expected {expected:?}",
                row.html
            ));
        }
    };
    for tier in tiers() {
        let [drop, reachable, all] =
            [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All].map(|c| convert_in(row.html, c, tier));
        check("reachable against drop", tier, &reachable, &drop);
        check(
            "all against the page with no such tags",
            tier,
            &all,
            &convert_in(row.unwrapped, HiddenContent::Drop, tier),
        );
        if DOM_TIERS.contains(&tier) {
            check("drop", tier, &drop, &row.drop);
            check("all", tier, &all, &row.all);
        }
    }
    wrong
}

/// The paragraph that every page of the head tests has in its body.
const VISIBLE: &str = "<p>visible</p>";

/// `html` with a `<template>` after its visible paragraph, in the body.
///
/// ~keep `HiddenContent::All` must write that template. A test that expects the same output for
/// ~keep the three choices passes when the option is ignored: the template in the body makes it
/// ~keep fail then.
fn with_a_template_in_the_body(html: &str) -> String {
    assert!(html.contains(VISIBLE), "the page has the visible paragraph: {html}");
    html.replacen(VISIBLE, "<p>visible</p><template><p>BODY</p></template>", 1)
}

/// One line for each wrong output of `html`, whose head holds a `<template>` or a `<noscript>`.
///
/// ~keep `expected` is the output for `HiddenContent::Drop` and `HiddenContent::Reachable`.
/// ~keep `HiddenContent::All` writes it too, and after it the template that this function puts
/// ~keep in the body: nothing from the head.
fn wrong_in_the_head(html: &str, expected: &str) -> Vec<String> {
    wrong_in_the_head_in(&tiers(), html, expected)
}

fn wrong_in_the_head_in(tiers: &[TierStrategy], html: &str, expected: &str) -> Vec<String> {
    let html = with_a_template_in_the_body(html);
    let mut wrong = Vec::new();
    for &tier in tiers {
        for choice in [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All] {
            let output = convert_in(&html, choice, tier);
            let expected = match choice {
                HiddenContent::All => format!("{expected}\nBODY\n"),
                HiddenContent::Drop | HiddenContent::Reachable => expected.to_string(),
            };
            if output != expected {
                wrong.push(format!(
                    "{choice:?} {tier:?} {html}\n  wrote    {output:?}\n  expected {expected:?}"
                ));
            }
        }
    }
    wrong
}

fn assert_none(wrong: &[String]) {
    assert!(wrong.is_empty(), "{} wrong outputs:\n{}", wrong.len(), wrong.join("\n"));
}

#[test]
fn should_keep_a_template_after_text_that_ends_the_head() {
    assert_none(&wrong_outputs(&Kept {
        html: "<head><title>t</title>visible<template><p>BODY</p></template>",
        unwrapped: "<head><title>t</title>visible<p>BODY</p>",
        drop: format!("{TITLE}visible\n"),
        all: format!("{TITLE}visible\n\nBODY\n"),
    }));
}

#[test]
fn should_end_the_head_at_text_that_is_not_white_space() {
    let rows = [
        Kept {
            html: "<head><title>t</title>visible<noscript><p>BODY</p></noscript>",
            unwrapped: "<head><title>t</title>visible<p>BODY</p>",
            drop: format!("{TITLE}visible\n"),
            all: format!("{TITLE}visible\n\nBODY\n"),
        },
        Kept {
            html: "<head><title>t</title> <!-- c -->\n visible<template><p>BODY</p></template>",
            unwrapped: "<head><title>t</title> <!-- c -->\n visible<p>BODY</p>",
            drop: format!("{TITLE}visible\n"),
            all: format!("{TITLE}visible\n\nBODY\n"),
        },
        Kept {
            html: r#"<head><meta charset="utf-8">visible<template><p>BODY</p></template>"#,
            unwrapped: r#"<head><meta charset="utf-8">visible<p>BODY</p>"#,
            drop: "visible\n".to_string(),
            all: "visible\n\nBODY\n".to_string(),
        },
        // ~keep Text before the `<head>` tag starts the body: a browser ignores the tag.
        Kept {
            html: "visible<head><template><p>BODY</p></template></head>",
            unwrapped: "visible<head><p>BODY</p></head>",
            drop: "visible\n".to_string(),
            all: "visible\n\nBODY\n".to_string(),
        },
        // ~keep A `<` that starts no tag is text.
        Kept {
            html: "<head><title>t</title>1 < 2<template><p>BODY</p></template>",
            unwrapped: "<head><title>t</title>1 < 2<p>BODY</p>",
            drop: format!("{TITLE}1 < 2\n"),
            all: format!("{TITLE}1 < 2\n\nBODY\n"),
        },
    ];
    assert_none(&rows.iter().flat_map(wrong_outputs).collect::<Vec<_>>());
}

#[test]
fn should_end_the_head_at_an_element_of_the_body_and_at_the_body_tag() {
    let rows = [
        Kept {
            html: "<head><title>t</title><p>visible</p><template><p>BODY</p></template>",
            unwrapped: "<head><title>t</title><p>visible</p><p>BODY</p>",
            drop: format!("{TITLE}\nvisible\n"),
            all: format!("{TITLE}\nvisible\n\nBODY\n"),
        },
        Kept {
            html: "<head><title>t</title><div>visible</div><template><p>BODY</p></template>",
            unwrapped: "<head><title>t</title><div>visible</div><p>BODY</p>",
            drop: format!("{TITLE}\nvisible\n"),
            all: format!("{TITLE}\nvisible\n\nBODY\n"),
        },
        Kept {
            html: "<head><title>t</title><body><p>visible</p><template><p>BODY</p></template>",
            unwrapped: "<head><title>t</title><body><p>visible</p><p>BODY</p>",
            drop: format!("{TITLE}\nvisible\n"),
            all: format!("{TITLE}\nvisible\n\nBODY\n"),
        },
        Kept {
            html: "<head><title>t</title></head><noscript><p>BODY</p></noscript><p>visible</p>",
            unwrapped: "<head><title>t</title></head><p>BODY</p><p>visible</p>",
            drop: format!("{TITLE}\nvisible\n"),
            all: format!("{TITLE}\nBODY\n\nvisible\n"),
        },
    ];
    assert_none(&rows.iter().flat_map(wrong_outputs).collect::<Vec<_>>());
}

#[test]
fn should_not_end_the_head_at_white_space_a_comment_or_metadata() {
    let mut wrong = Vec::new();
    for (metadata, front_matter) in [
        (" \n\t\r", ""),
        ("<!-- a comment -->", ""),
        (r#"<meta name="a" content="b c">"#, "meta-a: b c\n"),
        (r#"<link rel="icon" href="i.png">"#, ""),
        (r#"<base href="/">"#, "base: /\n"),
        ("<script>var a = 1;</script>", ""),
        ("<style>p { color: red }</style>", ""),
        (r#"<noscript><img src="p.png"></noscript>"#, ""),
        (r#"<template><meta name="a" content="b"></template>"#, ""),
    ] {
        let html =
            format!("<head><title>t</title>{metadata}<template><p>HEAD</p></template></head><body><p>visible</p>");
        let expected = format!("---\n{front_matter}title: t\n---\n\nvisible\n");
        wrong.extend(wrong_in_the_head(&html, &expected));
    }
    let page = "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>t</title>\n<template><p>HEAD</p></template>\n</head>\n<body><p>visible</p></body></html>";
    wrong.extend(wrong_in_the_head(page, &format!("{TITLE}\nvisible\n")));
    assert_none(&wrong);
}

#[test]
fn should_not_end_the_head_at_a_tag_that_a_browser_ignores_there() {
    // ~keep A browser ignores a second `<head>` tag and a stray end tag in the head.
    let mut wrong = Vec::new();
    for html in [
        "<head><head><template><p>HEAD</p></template></head><body><p>visible</p>",
        "<head></p><template><p>HEAD</p></template></head><body><p>visible</p>",
    ] {
        wrong.extend(wrong_in_the_head(html, "visible\n"));
    }
    assert_none(&wrong);
}

#[test]
fn should_read_a_tag_in_a_noscript_of_the_head_as_its_content() {
    // ~keep A `<noscript>` of the head ends at its own end tag, not at the first end tag in it,
    // ~keep so the `<template>` after it is still metadata.
    let html = r#"<head><noscript></i><img src="p.png"></noscript><template><p>HEAD</p></template></head><body><p>visible</p>"#;
    assert_none(&wrong_in_the_head(html, "visible\n"));
}

// ~keep A browser decodes a character reference before it decides where text goes. A reference
// ~keep for a space, a tab, a line feed, a form feed or a carriage return is white space of the
// ~keep head, so the `<template>` after it is still metadata. A numeric reference is decoded
// ~keep with no semicolon too.
#[test]
fn should_not_end_the_head_at_a_character_reference_for_white_space() {
    let mut wrong = Vec::new();
    for reference in [
        "&#32;",
        "&#32",
        "&#x20;",
        "&#10;",
        "&#9;",
        "&Tab;",
        "&NewLine;",
        "&#13;",
        "&#12;",
        " &#32;\n&Tab; ",
        // ~keep The search for the document head reads a no-break space as white space, and
        // ~keep this scan has the same rule. A browser ends the head there.
        "&nbsp;",
        "&#160;",
        "&#xA0;",
    ] {
        for held in ["<p>HEAD</p>", r#"<meta name="a" content="b">"#] {
            let html = format!(
                "<head><title>t</title>{reference}<template>{held}</template></head><body><p>visible</p></body>"
            );
            wrong.extend(wrong_in_the_head(&html, &format!("{TITLE}\nvisible\n")));
        }
    }
    assert_none(&wrong);
}

// ~keep A reference for a character that is not white space is text, and so is a `&` that
// ~keep starts no reference: `&Tab` with no semicolon is not a reference. Each ends the head, so
// ~keep `HiddenContent::All` writes the content of the `<template>` after it where it is written.
#[test]
fn should_end_the_head_at_a_character_reference_for_other_text() {
    let mut wrong = Vec::new();
    for reference in ["&amp;", "&#65;", "&#x41;", "&Tab", "&zzz;", "&"] {
        let html = format!(
            "<head><title>t</title>{reference}<template><p>BODY</p></template></head><body><p>visible</p></body>"
        );
        let unwrapped = format!("<head><title>t</title>{reference}<p>BODY</p></head><body><p>visible</p></body>");
        for tier in DOM_TIERS {
            let [drop, reachable, all] = [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All]
                .map(|choice| convert_in(&html, choice, tier));
            let expected = convert_in(&unwrapped, HiddenContent::Drop, tier);
            if reachable != drop || all != expected || !all.contains("BODY") {
                wrong.push(format!(
                    "{tier:?} {html}\n  drop      {drop:?}\n  reachable {reachable:?}\n  all       {all:?}\n  expected  {expected:?}"
                ));
            }
        }
    }
    assert_none(&wrong);
}

/// The tracking pixel that a page holds in a `<noscript>` of its head.
const PIXEL: &str = r#"<noscript><img src="https://x.test/p.gif" alt="pixel"></noscript>"#;

// ~keep The `<head>` tag is optional. A document with none has a head all the same: a doctype,
// ~keep an `<html>` tag or an element that only the head holds opens it, and it ends where the
// ~keep head of a page with the tag ends.
#[test]
fn should_read_a_template_and_a_noscript_of_a_head_with_no_head_tag_as_metadata() {
    let mut wrong = Vec::new();
    for (html, expected) in [
        (
            format!(r#"<!DOCTYPE html><html><meta charset="utf-8">{PIXEL}<body><p>visible</p></body></html>"#),
            "visible\n",
        ),
        (
            format!(
                "<!DOCTYPE html><html><title>t</title>{PIXEL}<template><p>HEAD</p></template><body><p>visible</p></body></html>"
            ),
            "---\ntitle: t\n---\n\nvisible\n",
        ),
        (
            "<html><template><p>HEAD</p></template><body><p>visible</p></body></html>".to_string(),
            "visible\n",
        ),
        (
            "<HTML><TEMPLATE><P>HEAD</P></TEMPLATE><BODY><p>visible</p></BODY></HTML>".to_string(),
            "visible\n",
        ),
        (format!("<!doctype html>{PIXEL}<p>visible</p>"), "visible\n"),
        (
            format!(r#"<meta charset="utf-8">&#32;&#x20;&Tab;{PIXEL}<p>visible</p>"#),
            "visible\n",
        ),
        (
            format!(r#"<link rel="icon" href="i.png"><!-- <p> a comment -->{PIXEL}<p>visible</p>"#),
            "visible\n",
        ),
    ] {
        wrong.extend(wrong_in_the_head_in(&DOM_TIERS, &html, expected));
    }
    assert_none(&wrong);
}

// ~keep The head repair rebuilds a page whose `<title>` is in no `<head>` element (issue #816)
// ~keep and a page with head content before its `<head>` tag (issue #592). The tree builder
// ~keep writes no content of a `<template>` back, so the tags of a kept element are removed
// ~keep before it reads the page.
#[test]
fn should_keep_the_hidden_content_of_a_page_that_the_head_repair_rebuilds() {
    let mut wrong = Vec::new();
    for (html, unwrapped) in [
        (
            "<title>t</title><p>visible</p><template><p>BODY</p></template>",
            "<title>t</title><p>visible</p><p>BODY</p>",
        ),
        (
            "<title>t</title><p>visible</p><noscript><p>BODY</p></noscript>",
            "<title>t</title><p>visible</p><p>BODY</p>",
        ),
        (
            "<head></head><title>t</title><body><p>visible</p><template><p>BODY</p></template></body>",
            "<head></head><title>t</title><body><p>visible</p><p>BODY</p></body>",
        ),
        (
            r#"<meta charset="utf-8"><head></head><body><p>visible</p><template><p>BODY</p></template></body>"#,
            r#"<meta charset="utf-8"><head></head><body><p>visible</p><p>BODY</p></body>"#,
        ),
    ] {
        wrong.extend(wrong_after_the_head(html, unwrapped));
    }
    assert_none(&wrong);

    let template = "<title>t</title><p>visible</p><template><p>BODY</p></template>";
    let shadow_root =
        r#"<title>t</title><p>visible</p><div><template shadowrootmode="open"><p>BODY</p></template></div>"#;
    let (visible, kept) = (format!("{TITLE}\nvisible\n"), format!("{TITLE}\nvisible\n\nBODY\n"));
    let (visible, kept) = (visible.as_str(), kept.as_str());
    for tier in DOM_TIERS {
        let outputs = |html: &str| {
            [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All].map(|c| convert_in(html, c, tier))
        };
        assert_eq!(outputs(template), [visible, visible, kept], "{tier:?}");
        assert_eq!(outputs(shadow_root), [visible, kept, kept], "{tier:?}");
    }
}

// ~keep A fragment has no head: a `<template>` or a `<noscript>` that nothing of a document
// ~keep precedes is content, and so is one after an element whose name only starts like the name
// ~keep of a metadata element.
#[test]
fn should_keep_the_first_template_and_noscript_of_a_fragment() {
    let mut wrong = Vec::new();
    for (html, unwrapped) in [
        (
            "<template><p>BODY</p></template><p>visible</p>",
            "<p>BODY</p><p>visible</p>",
        ),
        (
            "<noscript><p>BODY</p></noscript><p>visible</p>",
            "<p>BODY</p><p>visible</p>",
        ),
        (
            " <!-- <html> -->\n<template><p>BODY</p></template><p>visible</p>",
            " <!-- <html> -->\n<p>BODY</p><p>visible</p>",
        ),
        (
            "<metadata></metadata><template><p>BODY</p></template><p>visible</p>",
            "<metadata></metadata><p>BODY</p><p>visible</p>",
        ),
        (
            "<head><titles></titles><template><p>BODY</p></template></head><p>visible</p>",
            "<head><titles></titles><p>BODY</p></head><p>visible</p>",
        ),
    ] {
        wrong.extend(wrong_after_the_head(html, unwrapped));
    }
    assert_none(&wrong);
}

/// One line for each wrong output of `html`, whose `<template>` or `<noscript>` is in the body:
/// `HiddenContent::All` writes what `unwrapped`, the page with the two tags removed, gives.
fn wrong_after_the_head(html: &str, unwrapped: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    for tier in DOM_TIERS {
        let [drop, reachable, all] =
            [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All].map(|c| convert_in(html, c, tier));
        let expected = convert_in(unwrapped, HiddenContent::Drop, tier);
        if reachable != drop || all != expected || !all.contains("BODY") || drop.contains("BODY") {
            wrong.push(format!(
                "{tier:?} {html}\n  drop      {drop:?}\n  reachable {reachable:?}\n  wrote    {all:?}\n  expected  {expected:?}"
            ));
        }
    }
    wrong
}

// ~keep A browser reads the end tags of the body, of the document and of a line break in the
// ~keep head as the first thing after it. Every other end tag there is ignored, and so are the
// ~keep three in a comment and in a longer name.
#[test]
fn should_end_the_head_at_an_end_tag_that_a_browser_reads_as_the_body() {
    let mut wrong = Vec::new();
    for end_tag in ["</body>", "</html>", "</br>", "</BODY>", "</body >"] {
        let html = format!(
            "<head><title>t</title>{end_tag}<template><p>BODY</p></template></head><body><p>visible</p></body>"
        );
        let unwrapped = format!("<head><title>t</title>{end_tag}<p>BODY</p></head><body><p>visible</p></body>");
        wrong.extend(wrong_after_the_head(&html, &unwrapped));
    }
    for ignored in ["</bodys>", "</brx>", "<!-- </body> -->", "</div>"] {
        let html = format!(
            "<head><title>t</title>{ignored}<template><p>HEAD</p></template></head><body><p>visible</p></body>"
        );
        wrong.extend(wrong_in_the_head(&html, &format!("{TITLE}\nvisible\n")));
    }
    assert_none(&wrong);
}

// ~keep The inert-body pass reads `<title/>` as a start tag, so the rest of the page is the text
// ~keep of the title, and each choice writes it as text. Issue #878 tracks that defect of the
// ~keep default: a page with a title that closes itself has an empty title and a body.
#[test]
fn should_write_what_the_default_writes_after_a_title_that_closes_itself() {
    let html = "<head><title/><template><p>HEAD</p></template></head><body><p>visible</p><template><p>BODY</p></template></body>";
    let rest = "<template><p>HEAD</p></template></head><body><p>visible</p><template><p>BODY</p></template></body>\n";
    for tier in DOM_TIERS {
        for choice in [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All] {
            assert_eq!(convert_in(html, choice, tier), rest, "{choice:?} {tier:?}");
        }
    }
}

// ~keep After the end tag of the head a browser still puts a `<template>` in the head. It puts a
// ~keep `<noscript>` there in the body.
#[test]
fn should_read_a_template_between_the_head_and_the_body_as_metadata() {
    let html =
        "<html><head><title>t</title></head>\n<template><p>HEAD</p></template>\n<body><p>visible</p></body></html>";
    assert_none(&wrong_in_the_head(html, &format!("{TITLE}\nvisible\n")));
    assert_none(&wrong_after_the_head(
        "<html><head><title>t</title></head>\n<noscript><p>BODY</p></noscript>\n<body><p>visible</p></body></html>",
        "<html><head><title>t</title></head>\n<p>BODY</p>\n<body><p>visible</p></body></html>",
    ));
}
