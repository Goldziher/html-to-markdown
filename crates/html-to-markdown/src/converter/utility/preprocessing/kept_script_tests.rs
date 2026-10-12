//! Tests of the record of the kept scripts.

/// The reader of the structured data gets the text of a kept script as the page has it: the
/// escape of the first pass is taken back and no character reference is decoded.
#[cfg(feature = "metadata")]
#[test]
fn the_text_of_a_kept_script_is_given_as_written() {
    use super::strip_script_and_style_tags_with_kept_scripts;
    let open = r#"<script type="application/ld+json">"#;
    for body in [
        r#"{"a":"?a=1&b=2 &amp; &#38; &amp;amp;"}"#,
        r#"{"a":"a < b"}"#,
        r#"{"a":"a &lt; b"}"#,
        r#"{"a":"a & b &amp; c < d &lt; e &amp;lt; f"}"#,
        "{\"a\":\"<p>x</p>\",\r\n\"b\":\"&lt;\"}",
    ] {
        let html = format!("<p>a</p>{open} {body}\n</script><p>b</p>");
        let (page, _) = strip_script_and_style_tags_with_kept_scripts(&html);
        let in_tree = page
            .strip_prefix(&format!("<p>a</p>{open}"))
            .and_then(|rest| rest.strip_suffix("</script><p>b</p>"))
            .expect("the kept script");
        assert!(!in_tree.contains('<'), "{body:?}: one text for the parser");
        // ~keep The script is the node 7 of a tree that holds this one script.
        let as_written = |text: &str| {
            let (_, mut kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
            kept_scripts.match_to_tree(&[(7, text.to_string())]);
            kept_scripts.as_written(7, text).into_owned()
        };
        assert_eq!(as_written(in_tree.trim()), body, "{body:?}");
        // ~keep The tree builder writes CR LF as LF.
        let repaired = in_tree.replace("\r\n", "\n");
        assert_eq!(as_written(repaired.trim()), body, "{body:?}: a repaired page");
    }
}

/// The record of a kept script goes by the place of the script among the scripts with its
/// escaped text, so two bodies that escape to one text do not meet.
#[cfg(feature = "metadata")]
#[test]
fn the_record_of_a_kept_script_goes_by_its_place_among_the_scripts_with_its_text() {
    use super::strip_script_and_style_tags_with_kept_scripts;
    /// A script of the tree: the id of its node, its body in the page, the body that it reads.
    type InTree = (u32, &'static str, &'static str);
    let open = r#"<script type="application/ld+json">"#;
    let page = |bodies: &[&str]| {
        let mut page = String::new();
        for body in bodies {
            page.push_str(open);
            page.push_str(body);
            page.push_str("</script>");
        }
        page
    };
    let lt = r#"{"a":"<"}"#;
    let reference = r#"{"a":"&lt;"}"#;
    let other = r#"{"b":"&lt;"}"#;
    // ~keep Each case: the bodies of the page, then the nodes of the tree with the body that
    // ~keep each one reads. A node reads the text of the tree with `&lt;` as `<` when it has
    // ~keep no record.
    let cases: [(&[&str], &[InTree]); 7] = [
        (&[lt, reference], &[(3, lt, lt), (9, reference, reference)]),
        (&[reference, lt], &[(3, reference, reference), (9, lt, lt)]),
        (
            &[reference, reference],
            &[(3, reference, reference), (9, reference, reference)],
        ),
        // ~keep A pass moved a script with another text to the front.
        (
            &[lt, other, reference],
            &[(2, other, other), (3, lt, lt), (9, reference, reference)],
        ),
        // ~keep A pass dropped the script with another text: no later script takes its record.
        (&[other, lt, reference], &[(3, lt, lt), (9, reference, reference)]),
        // ~keep A pass dropped one of two equal bodies.
        (&[reference, reference], &[(9, reference, reference)]),
        // ~keep A pass dropped one of two bodies with one escaped text: the place names no
        // ~keep script, and the script that stays has no record.
        (&[lt, reference], &[(9, reference, lt)]),
    ];
    for (bodies, nodes) in cases {
        let html = page(bodies);
        let (_, mut kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&html);
        let texts: Vec<(u32, String)> = nodes
            .iter()
            .map(|(node, body, _)| (*node, body.replace('<', "&lt;")))
            .collect();
        kept_scripts.match_to_tree(&texts);
        for ((node, text), (_, _, expected)) in texts.iter().zip(nodes) {
            assert_eq!(
                kept_scripts.as_written(*node, text),
                *expected,
                "{bodies:?}: node {node}"
            );
        }
    }
}
