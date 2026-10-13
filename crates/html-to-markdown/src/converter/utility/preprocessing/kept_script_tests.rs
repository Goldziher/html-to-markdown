//! Tests of the record of the kept scripts.

#![cfg(feature = "metadata")]

use super::{KeptScripts, strip_script_and_style_tags, strip_script_and_style_tags_with_kept_scripts};

const OPEN: &str = r#"<script type="application/ld+json">"#;

/// A script of a tree after the owner marks left it: the text that the reader of the structured
/// data gets, and each attribute with its value.
struct Script {
    as_written: String,
    attributes: Vec<(String, Option<String>)>,
}

/// The scripts of `page` in the order of the tree, read as the conversion reads them.
fn scripts(page: &str, kept_scripts: KeptScripts) -> Vec<Script> {
    let mut dom = tl::parse(page, tl::ParserOptions::default()).expect("the page");
    let kept_scripts = kept_scripts.with_owner_marks_taken(&mut dom).expect("the owner marks");
    let parser = dom.parser();
    let mut scripts = Vec::new();
    for (node, slot) in dom.nodes().iter().enumerate() {
        let Some(tag) = slot.as_tag() else {
            continue;
        };
        if tag.name().as_bytes() != b"script" {
            continue;
        }
        let node = u32::try_from(node).expect("the id of a node");
        let text = tag.inner_text(parser);
        let mut attributes: Vec<(String, Option<String>)> = tag
            .attributes()
            .iter()
            .map(|(name, value)| (name.into_owned(), value.map(std::borrow::Cow::into_owned)))
            .collect();
        attributes.sort();
        scripts.push(Script {
            as_written: kept_scripts.as_written(node, text.trim()).to_owned(),
            attributes,
        });
    }
    scripts
}

fn type_only() -> Vec<(String, Option<String>)> {
    vec![("type".to_string(), Some("application/ld+json".to_string()))]
}

/// The reader of the structured data gets the text of a kept script as the page has it: the
/// escape of the first pass is taken back and no character reference is decoded.
#[test]
fn the_text_of_a_kept_script_is_given_as_written() {
    for body in [
        r#"{"a":"?a=1&b=2 &amp; &#38; &amp;amp;"}"#,
        r#"{"a":"a < b"}"#,
        r#"{"a":"a &lt; b"}"#,
        r#"{"a":"a & b &amp; c < d &lt; e &amp;lt; f"}"#,
        "{\"a\":\"<p>x</p>\",\r\n\"b\":\"&lt;\"}",
    ] {
        let html = format!("<p>a</p>{OPEN} {body}\n</script><p>b</p>");
        let (page, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
        // ~keep A script whose escaped text holds `&lt;` has the owner mark of the number 0.
        let open = if body.contains('<') || body.contains("&lt;") {
            r#"<script data-h2m-script-0="h2m_script_body_0_end" type="application/ld+json">"#
        } else {
            OPEN
        };
        let in_tree = page
            .strip_prefix(&format!("<p>a</p>{open}"))
            .and_then(|rest| rest.strip_suffix("</script><p>b</p>"))
            .expect("the kept script");
        assert!(!in_tree.contains('<'), "{body:?}: one text for the parser");
        let found = scripts(&page, kept_scripts);
        assert_eq!(found.len(), 1, "{body:?}");
        assert_eq!(found[0].as_written, body, "{body:?}");
        assert_eq!(found[0].attributes, type_only(), "{body:?}: no owner mark stays");
        // ~keep The tree builder writes CR LF as LF.
        let (page, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
        let found = scripts(&page.replace("\r\n", "\n"), kept_scripts);
        assert_eq!(found[0].as_written, body, "{body:?}: a repaired page");
    }
}

/// The owner mark of a kept script goes with its element. A pass that removes or moves a
/// script changes the record of no other script, also when the tree holds two bodies as one
/// text.
#[test]
fn the_record_of_a_kept_script_goes_with_its_owner_mark() {
    let lt = r#"{"a":"<"}"#;
    let reference = r#"{"a":"&lt;"}"#;
    let other = r#"{"b":"&lt;"}"#;
    let bodies = [lt, reference, other, reference];
    let mut html = String::new();
    for body in bodies {
        html.push_str(OPEN);
        html.push_str(body);
        html.push_str("</script>");
    }
    // ~keep Each case: the scripts that a pass leaves, in the order in which it leaves them.
    let cases: [&[usize]; 9] = [
        &[0, 1, 2, 3],
        &[2, 0, 1, 3],
        &[3, 2, 1, 0],
        &[1, 2, 3],
        &[0, 2, 3],
        &[1],
        &[0],
        &[3, 0],
        &[],
    ];
    for kept in cases {
        let (page, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
        let elements: Vec<&str> = page.split_inclusive("</script>").collect();
        assert_eq!(elements.len(), bodies.len());
        let left: String = kept.iter().map(|at| elements[*at]).collect();
        let found = scripts(&left, kept_scripts);
        let as_written: Vec<&str> = found.iter().map(|script| script.as_written.as_str()).collect();
        let expected: Vec<&str> = kept.iter().map(|at| bodies[*at]).collect();
        assert_eq!(as_written, expected, "{kept:?}");
        for script in &found {
            assert_eq!(script.attributes, type_only(), "{kept:?}: no owner mark stays");
        }
    }
}

/// An attribute of the author with the name of an owner mark, in any case of its letters,
/// stays with its value: the pass takes a number that the page does not have.
#[test]
fn an_attribute_of_the_author_with_the_name_of_an_owner_mark_stays() {
    let html = format!(
        "<script type=\"application/ld+json\" DATA-H2M-SCRIPT-0=\"x\" data-h2m-script-2>{{\"a\":\"&lt;\"}}</script>\
         <p data-h2m-script-3=\"y\">text</p>{OPEN}{{\"a\":\"<\"}}</script>"
    );
    let (page, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
    assert_eq!(
        page,
        "<script data-h2m-script-1=\"h2m_script_body_1_end\" type=\"application/ld+json\" \
         DATA-H2M-SCRIPT-0=\"x\" data-h2m-script-2>{\"a\":\"&lt;\"}</script>\
         <p data-h2m-script-3=\"y\">text</p>\
         <script data-h2m-script-4=\"h2m_script_body_4_end\" type=\"application/ld+json\">{\"a\":\"&lt;\"}</script>"
    );
    let found = scripts(&page, kept_scripts);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].as_written, r#"{"a":"&lt;"}"#);
    assert_eq!(
        found[0].attributes,
        [
            ("data-h2m-script-0".to_string(), Some("x".to_string())),
            ("data-h2m-script-2".to_string(), None),
            ("type".to_string(), Some("application/ld+json".to_string())),
        ]
    );
    assert_eq!(found[1].as_written, r#"{"a":"<"}"#);
    assert_eq!(found[1].attributes, type_only());
}

/// A name that only looks like the name of an owner mark is no owner mark: the number of a
/// mark has no sign and no zero before it. The second script has no mark, and it does not take
/// the record of the first one.
#[test]
fn a_name_with_a_sign_or_a_zero_before_its_number_is_no_owner_mark() {
    for name in [
        "data-h2m-script-+0",
        "data-h2m-script-00",
        "data-h2m-script-0x",
        "data-h2m-script-",
    ] {
        let html = format!(
            "{OPEN}{{\"a\":\"&lt;\"}}</script>\
             <script type=\"application/ld+json\" {name}=\"x\">{{\"a\":\"b\"}}</script>"
        );
        let (page, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
        let found = scripts(&page, kept_scripts);
        assert_eq!(found.len(), 2, "{name}");
        assert_eq!(found[0].as_written, r#"{"a":"&lt;"}"#, "{name}");
        assert_eq!(found[1].as_written, r#"{"a":"b"}"#, "{name}");
        let mut expected = type_only();
        expected.push((name.to_string(), Some("x".to_string())));
        expected.sort();
        assert_eq!(found[1].attributes, expected, "{name}");
    }
}

/// The value of an owner mark is compared whole. A script with the name of a mark and another
/// value has no record: it reads as the tree has it, with no reference decoded, and the
/// attribute stays.
#[test]
fn a_script_with_no_owner_mark_reads_as_the_tree_has_it() {
    let html = format!("{OPEN}{{\"a\":\"<\"}}</script>");
    let (page, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
    let other_value = page.replace("h2m_script_body_0_end", "h2m_script_body_0_end ");
    assert_ne!(other_value, page);
    let found = scripts(&other_value, kept_scripts);
    assert_eq!(found[0].as_written, r#"{"a":"&lt;"}"#);
    assert_eq!(found[0].attributes.len(), 2);
    let (page, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
    assert_eq!(scripts(&page, kept_scripts)[0].as_written, r#"{"a":"<"}"#);
}

/// The start tag of a script that the tree has as text gives its owner mark to no text, and
/// the text of the author about it stays.
#[test]
fn an_owner_mark_in_the_text_of_the_tree_is_taken_out() {
    let html = format!("{OPEN}{{\"a\":\"<\"}}</script><p>b</p>{OPEN}{{\"a\":\"&lt;\"}}</script>");
    let (page, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
    assert_eq!(page.matches("data-h2m-script-").count(), 2);
    // ~keep The first script as the text of an element whose content is text.
    let as_text = page
        .replacen("<script", "<textarea>x data-h2m-script-7 &lt;script", 1)
        .replacen("</script>", "&lt;/script></textarea>", 1);
    let mut dom = tl::parse(&as_text, tl::ParserOptions::default()).expect("the page");
    kept_scripts.with_owner_marks_taken(&mut dom).expect("the owner marks");
    let texts: Vec<String> = dom
        .nodes()
        .iter()
        .filter_map(tl::Node::as_raw)
        .map(|text| text.as_utf8_str().into_owned())
        .collect();
    let attributes = &OPEN["<script".len()..];
    let expected = format!("x data-h2m-script-7 &lt;script{attributes}{{\"a\":\"&lt;\"}}&lt;/script>");
    assert!(texts.contains(&expected), "{texts:?}");
    assert_eq!(
        texts.iter().filter(|text| text.contains("data-h2m-script-")).count(),
        1,
        "{texts:?}"
    );
}

/// The pass over a repaired page writes no owner mark: the tree builder writes the marks of
/// the first page again, and the pass leaves them as they are.
#[test]
fn the_pass_over_a_repaired_page_writes_no_owner_mark() {
    let html = format!("{OPEN}{{\"a\":\"<\"}}</script>{OPEN}{{\"a\":\"&lt;\"}}</script>");
    assert!(!strip_script_and_style_tags(&html).contains("data-h2m-script-"));
    let (page, _) = strip_script_and_style_tags_with_kept_scripts(&html);
    assert_eq!(page.matches("data-h2m-script-").count(), 2);
    assert_eq!(strip_script_and_style_tags(&page), page);
}

/// A kept script with no `<` and no `&lt;` in its text has no owner mark: the tree holds its
/// text as written.
#[test]
fn a_kept_script_whose_text_the_tree_holds_as_written_has_no_owner_mark() {
    let html = format!("<p>a</p>{OPEN}{{\"a\":\"b &amp; c\"}}</script>");
    let (page, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
    assert_eq!(page, html);
    assert_eq!(scripts(&page, kept_scripts)[0].as_written, r#"{"a":"b &amp; c"}"#);
}
