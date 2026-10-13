//! Pull request 831: the owner of a kept script goes with its element. The tree holds the body
//! `{"a":"<"}` and the body `{"a":"&lt;"}` as one text, and a pass can remove one of two such
//! scripts after the first pass. The script that stays gives its own JSON as written, on the
//! default route and on the full converter, and the mark that names its owner is in no output.

#![allow(missing_docs)]
// ~keep The converter is chosen by name, which the crate offers under this feature only.
#![cfg(all(feature = "testkit", feature = "metadata"))]

use html_to_markdown_rs::{
    ConversionOptions, HiddenContent, OutputFormat, PreprocessingOptions, PreprocessingPreset, TierStrategy, convert,
};

const ROUTES: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];
const OPEN: &str = r#"<script type="application/ld+json">"#;
/// Two bodies that the tree holds as one text: the first pass writes each `<` as `&lt;`.
const LT: &str = r#"{"a":"<"}"#;
const REFERENCE: &str = r#"{"a":"&lt;"}"#;
const BOTH_ORDERS: [[&str; 2]; 2] = [[LT, REFERENCE], [REFERENCE, LT]];
/// The start of the name of the owner mark.
const MARK: &str = "data-h2m-script-";
/// The two ways in which a page hides an element, as its start tag and its end tag.
const HIDDEN: [[&str; 2]; 2] = [["<div hidden>", "</div>"], [r#"<div style="display:none">"#, "</div>"]];
/// A page as written, and the starts of a page that is parsed again: an omitted end tag, a
/// custom element, a misnest.
const STARTS: [&str; 4] = [
    "",
    "<p>one<div>two</div>",
    "<p>one<div>two</div><my-part>q</my-part>",
    "<b><p>one</b>two</p>",
];

fn script(body: &str) -> String {
    format!("{OPEN}{body}</script>")
}

/// The JSON of each script by each route, with the options that `configure` sets. No content
/// holds the owner mark.
fn raw_json_with(html: &str, configure: impl Fn(&mut ConversionOptions)) -> Vec<Vec<String>> {
    ROUTES
        .into_iter()
        .map(|tier_strategy| {
            let mut options = ConversionOptions {
                tier_strategy,
                ..ConversionOptions::default()
            };
            configure(&mut options);
            let result = convert(html, Some(options)).expect("conversion should succeed");
            let content = result.content.unwrap_or_default();
            assert!(!content.contains(MARK), "{tier_strategy:?}: {html:?} gave {content:?}");
            result
                .metadata
                .structured_data
                .into_iter()
                .map(|data| data.raw_json)
                .collect()
        })
        .collect()
}

fn raw_json(html: &str) -> Vec<Vec<String>> {
    raw_json_with(html, |_| {})
}

/// The bodies as the page has them, by each route.
fn as_written(bodies: &[&str]) -> Vec<Vec<String>> {
    vec![bodies.iter().map(|body| (*body).to_string()).collect(); ROUTES.len()]
}

/// The row of the review: the hidden script and the script that stays have unequal bodies and
/// one text in the tree. The script that stays gave `{"a":"<"}`.
#[test]
fn the_script_that_stays_after_a_hidden_script_with_its_text_gives_its_own_json() {
    let html = r#"<div hidden><script type="application/ld+json">{"a":"<"}</script></div><script type="application/ld+json">{"a":"&lt;"}</script><p>visible</p>"#;
    assert_eq!(raw_json(html), as_written(&[r#"{"a":"&lt;"}"#]));
}

/// The hidden cleanup removes one of two scripts with one text in the tree: the first one or
/// the second one, hidden by the attribute or by the style, on a page as written and on a page
/// that is parsed again.
#[test]
fn a_script_gives_its_own_json_when_the_hidden_cleanup_removes_a_script_with_its_text() {
    for [open, close] in HIDDEN {
        for [removed, kept] in BOTH_ORDERS {
            for start in STARTS {
                for html in [
                    format!("{start}{open}{}{close}{}<p>visible</p>", script(removed), script(kept)),
                    format!("{start}{}{open}{}{close}<p>visible</p>", script(kept), script(removed)),
                ] {
                    assert_eq!(raw_json(&html), as_written(&[kept]), "{html:?}");
                }
            }
        }
    }
}

/// The start tags of a structured data script that the converter reads: the media type in
/// another case, with a parameter, with white space at its two ends, and a `/` after the name.
const START_TAGS: [&str; 5] = [
    r#"<script type="application/ld+json">"#,
    r#"<script type="APPLICATION/LD+JSON">"#,
    r#"<script type="application/ld+json; charset=utf-8">"#,
    r#"<script type=" application/ld+json ">"#,
    r#"<script/type="application/ld+json">"#,
];
/// The white space on the two sides of the JSON in the script.
const AROUND_THE_JSON: [[&str; 2]; 3] = [["", ""], ["  ", " "], ["\n", "\n  "]];

/// The matrix of the review: a hidden script before the script that stays, for each way to
/// hide, each start tag and each white space around the JSON. A visible element in the place of
/// the hidden one is the control: each of the two scripts gives its own JSON.
#[test]
fn the_script_after_a_hidden_script_gives_its_own_json_for_each_start_tag_and_white_space() {
    for start_tag in START_TAGS {
        for [before, after] in AROUND_THE_JSON {
            for [first, second] in BOTH_ORDERS {
                let scripts = [first, second].map(|body| format!("{start_tag}{before}{body}{after}</script>"));
                for [open, close] in HIDDEN {
                    let html = format!("{open}{}{close}{}<p>visible</p>", scripts[0], scripts[1]);
                    assert_eq!(raw_json(&html), as_written(&[second]), "{html:?}");
                }
                let control = format!("<div>{}</div>{}<p>visible</p>", scripts[0], scripts[1]);
                assert_eq!(raw_json(&control), as_written(&[first, second]), "{control:?}");
            }
        }
    }
}

/// A script that the first pass does not write again has no owner mark and no record. It reads
/// as the tree has it: a character reference of the author is not decoded.
#[test]
fn a_script_with_no_record_keeps_the_references_of_the_author() {
    let html = format!("{OPEN}{{\"a\":\"&amp; &quot; &#60; &gt;\"}}</script><p>visible</p>");
    assert_eq!(raw_json(&html), as_written(&[r#"{"a":"&amp; &quot; &#60; &gt;"}"#]));
}

/// The hidden cleanup removes the middle one of three scripts.
#[test]
fn two_scripts_give_each_its_own_json_when_the_hidden_cleanup_removes_the_script_between_them() {
    for [open, close] in HIDDEN {
        for [first, middle, last] in [
            [LT, REFERENCE, LT],
            [REFERENCE, LT, REFERENCE],
            [LT, LT, REFERENCE],
            [REFERENCE, REFERENCE, LT],
        ] {
            let html = format!(
                "{}{open}{}{close}{}<p>visible</p>",
                script(first),
                script(middle),
                script(last)
            );
            assert_eq!(raw_json(&html), as_written(&[first, last]), "{html:?}");
        }
    }
}

/// A choice that keeps hidden content keeps the hidden script: each of the two scripts gives
/// its own JSON.
#[test]
fn a_kept_hidden_script_and_the_script_beside_it_give_each_its_own_json() {
    for hidden_content in [HiddenContent::Reachable, HiddenContent::All] {
        for [open, close] in HIDDEN {
            for [first, second] in BOTH_ORDERS {
                for start in ["", "<p>one<div>two</div><my-part>q</my-part>"] {
                    for html in [
                        format!("{start}{open}{}{close}{}<p>visible</p>", script(first), script(second)),
                        format!("{start}{}{open}{}{close}<p>visible</p>", script(first), script(second)),
                    ] {
                        assert_eq!(
                            raw_json_with(&html, |options| options.hidden_content = hidden_content),
                            as_written(&[first, second]),
                            "{hidden_content:?}: {html:?}"
                        );
                    }
                }
            }
        }
    }
}

/// The two pages with one script in `wrapper` and one script beside it: the wrapped script
/// first, and the wrapped script second. `wrapper` is a start tag and an end tag.
fn pages_with_one_wrapped_script(wrapper: [&str; 2], inside: &str, beside: &str) -> [String; 2] {
    let [open, close] = wrapper;
    [
        format!("{open}{}{close}{}<p>visible</p>", script(inside), script(beside)),
        format!("{}{open}{}{close}<p>visible</p>", script(beside), script(inside)),
    ]
}

/// A start tag of `svg` or `math` that closes itself opens nothing, so the script after it is a
/// script of the page: it gives its JSON as written, with a `<` and with a reference of the
/// author. The same holds after an `svg` with an end tag.
#[test]
fn a_script_after_a_self_closing_svg_or_math_gives_its_json_as_written() {
    for before in [
        "<svg/>",
        "<svg />",
        "<SVG/>",
        "<math/>",
        "<svg></svg>",
        "<svg><g/></svg>",
    ] {
        for body in [LT, REFERENCE] {
            let html = format!("{before}{}<p>t</p>", script(body));
            assert_eq!(raw_json(&html), as_written(&[body]), "{html:?}");
        }
    }
}

/// A `textarea` with no end tag has the rest of the page as its text. The start tag of a
/// script in that text is text, and no content holds the owner mark that the first pass gave
/// it: `raw_json_with` looks for the mark in each content.
#[test]
fn a_script_tag_in_the_text_of_a_textarea_with_no_end_tag_shows_no_owner_mark() {
    for body in [LT, REFERENCE] {
        for html in [
            format!(
                "<html><head><textarea></head><body><p>body</p>{}</body></html>",
                script(body)
            ),
            format!("<p>foo</p><div><textarea>t{}</div><p>after</p>", script(body)),
        ] {
            assert_eq!(raw_json(&html).len(), ROUTES.len(), "{html:?}");
        }
    }
}

/// An excluded selector removes one of the two scripts in the walk: the script itself, or the
/// element that holds it.
#[test]
fn a_script_gives_its_own_json_when_a_selector_excludes_a_script_with_its_text() {
    for [removed, kept] in BOTH_ORDERS {
        let excluded = format!(r#"<script id="gone" type="application/ld+json">{removed}</script>"#);
        let mut pages = vec![
            format!("{excluded}{}<p>visible</p>", script(kept)),
            format!("{}{excluded}<p>visible</p>", script(kept)),
        ];
        pages.extend(pages_with_one_wrapped_script(
            [r#"<div id="gone">"#, "</div>"],
            removed,
            kept,
        ));
        for html in pages {
            assert_eq!(
                raw_json_with(&html, |options| options.exclude_selectors = vec!["#gone".to_string()]),
                as_written(&[kept]),
                "{html:?}"
            );
        }
    }
}

/// A preprocessing preset removes the navigation that holds one of the two scripts.
#[test]
fn a_script_gives_its_own_json_when_a_preset_removes_a_script_with_its_text() {
    for [removed, kept] in BOTH_ORDERS {
        for html in pages_with_one_wrapped_script(["<nav>", "</nav>"], removed, kept) {
            let found = raw_json_with(&html, |options| {
                options.preprocessing = PreprocessingOptions {
                    enabled: true,
                    preset: PreprocessingPreset::Standard,
                    remove_navigation: true,
                    remove_forms: true,
                };
            });
            assert_eq!(found, as_written(&[kept]), "{html:?}");
        }
    }
}

/// `strip_tags` writes the content of an element and not its tags, so a script in a stripped
/// `div` stays: each of the two scripts gives its own JSON. With `script` in `strip_tags` no
/// script is read as structured data.
#[test]
fn the_scripts_give_each_its_own_json_beside_a_stripped_tag() {
    for [first, second] in BOTH_ORDERS {
        for html in [
            format!("<div>{}</div>{}<p>visible</p>", script(first), script(second)),
            format!("{}<div>{}</div><p>visible</p>", script(first), script(second)),
        ] {
            assert_eq!(
                raw_json_with(&html, |options| options.strip_tags = vec!["div".to_string()]),
                as_written(&[first, second]),
                "{html:?}"
            );
            assert_eq!(
                raw_json_with(&html, |options| options.strip_tags = vec!["script".to_string()]),
                as_written(&[]),
                "{html:?}"
            );
        }
    }
}

/// The inert-body pass takes the content of a `template` and of a `noscript` out of the page
/// when the choice of hidden content does not keep it. `all` keeps it.
#[test]
fn a_script_gives_its_own_json_beside_a_script_with_its_text_in_an_inert_element() {
    for [inert, beside] in BOTH_ORDERS {
        for html in pages_with_one_wrapped_script(["<template>", "</template>"], inert, beside) {
            for hidden_content in [HiddenContent::Drop, HiddenContent::Reachable] {
                assert_eq!(
                    raw_json_with(&html, |options| options.hidden_content = hidden_content),
                    as_written(&[beside]),
                    "{hidden_content:?}: {html:?}"
                );
            }
        }
        for html in pages_with_one_wrapped_script(["<noscript>", "</noscript>"], inert, beside) {
            assert_eq!(raw_json(&html), as_written(&[beside]), "{html:?}");
        }
        let kept = format!("<template>{}</template>{}<p>visible</p>", script(inert), script(beside));
        assert_eq!(
            raw_json_with(&kept, |options| options.hidden_content = HiddenContent::All),
            as_written(&[inert, beside]),
            "{kept:?}"
        );
    }
}

/// The head repair writes the page again before the first pass reads it, and the hidden
/// cleanup then removes one of the two scripts.
#[test]
fn a_script_of_a_page_with_a_rebuilt_head_gives_its_own_json_after_a_hidden_script_is_removed() {
    for [removed, kept] in BOTH_ORDERS {
        for html in [
            format!(
                "<title>T</title><div hidden>{}</div>{}<p>visible</p>",
                script(removed),
                script(kept)
            ),
            format!(
                "<title>T</title>{}<div hidden>{}</div><p>visible</p>",
                script(kept),
                script(removed)
            ),
        ] {
            assert_eq!(raw_json(&html), as_written(&[kept]), "{html:?}");
        }
    }
}

/// A script with no end tag holds the rest of the page. It gives its own JSON after the hidden
/// cleanup removed a script with its text.
#[test]
fn a_script_with_no_end_tag_gives_its_own_json_after_a_hidden_script_is_removed() {
    for [removed, kept] in BOTH_ORDERS {
        let html = format!("<div hidden>{}</div><p>visible</p>{OPEN}{kept}", script(removed));
        assert_eq!(raw_json(&html), as_written(&[kept]), "{html:?}");
    }
}

/// An attribute of the author with the name of an owner mark stays as the author wrote it, and
/// `preserve_tags` writes no owner mark. The two attributes of the script are written in the
/// one order or in the other.
#[test]
fn a_preserved_script_keeps_an_attribute_with_the_name_of_an_owner_mark() {
    let html = r#"<p>foo <script type="application/ld+json" data-h2m-script-0="mine">{"a":"<"}</script> bar</p>"#;
    assert_eq!(raw_json(html), as_written(&[LT]));
    let written = [
        r#"foo <script type="application/ld+json" data-h2m-script-0="mine">{"a":"&lt;"}</script> bar"#,
        r#"foo <script data-h2m-script-0="mine" type="application/ld+json">{"a":"&lt;"}</script> bar"#,
    ];
    for tier_strategy in ROUTES {
        let options = ConversionOptions {
            tier_strategy,
            preserve_tags: vec!["script".to_string()],
            ..ConversionOptions::default()
        };
        let content = convert(html, Some(options))
            .expect("conversion should succeed")
            .content
            .unwrap_or_default();
        assert!(written.contains(&content.trim_end()), "{tier_strategy:?}: {content:?}");
    }
}

const AUTHORED: &str = r#"{"a":"authored"}"#;
const WITH_TAG: &str = r#"{"a":"<b>"}"#;
/// The first owner mark of a page, as an attribute that the author wrote.
const AUTHORED_MARK: &str = r#"data-h2m-script-0="h2m_script_body_0_end""#;
/// What the cleanup removes after the first pass, with the text that hides a start tag from
/// the first pass: a processing instruction, two bogus comments, a CDATA section, a hidden
/// element.
const REMOVED: [[&str; 2]; 5] = [
    ["<?x?>", "<?y?>"],
    ["<?x?>", "<!x>"],
    ["<?x?>", "</ x>"],
    ["<?x?>", "<![CDATA[x]]>"],
    ["<span hidden>x</span>", "<span hidden>x</span>"],
];

/// A script of the author that the first pass does not read as a script: `lead` is between
/// its `<` and its name. `in_name` is in the name of an attribute and `in_value` is in its
/// value. With the three removed, the attribute is the first owner mark of a page.
fn script_with_a_mark_in_parts(lead: &str, in_name: &str, in_value: &str, body: &str) -> String {
    format!(
        r#"<{lead}script type="application/ld+json" data-h2m-script-{in_name}0="h2m_script_body_{in_value}0_end">{body}</script>"#
    )
}

/// The content of `html` by each route, with the options that `configure` sets.
fn content_with(html: &str, configure: impl Fn(&mut ConversionOptions)) -> Vec<String> {
    ROUTES
        .into_iter()
        .map(|tier_strategy| {
            let mut options = ConversionOptions {
                tier_strategy,
                ..ConversionOptions::default()
            };
            configure(&mut options);
            convert(html, Some(options))
                .expect("conversion should succeed")
                .content
                .unwrap_or_default()
        })
        .collect()
}

/// The content of `html` by each route, with the elements `tag` written as HTML.
fn preserved(html: &str, tag: &str) -> Vec<String> {
    content_with(html, |options| options.preserve_tags = vec![tag.to_string()])
}

/// The row of the second review: the cleanup removes three processing instructions, and the
/// first script then has the name and the value of the mark of the second one. Each script
/// gave `{"a":"<b>"}`, and `preserve_tags` wrote the first one with no attribute of the author.
#[test]
fn a_script_keeps_its_json_and_its_attribute_when_the_cleanup_joins_the_name_of_an_owner_mark() {
    let html = r#"<<?x?>script type="application/ld+json" data-h2m-script-<?y?>0="h2m_script_body_<?z?>0_end">{"a":"authored"}</script><script type="application/ld+json">{"a":"<b>"}</script><p>tail</p>"#;
    assert_eq!(raw_json(html), as_written(&[AUTHORED, WITH_TAG]));
    let written = format!(
        "<script {AUTHORED_MARK} type=\"application/ld+json\">{AUTHORED}</script>\n\n\
         <script type=\"application/ld+json\">{{\"a\":\"&lt;b>\"}}</script>\n\ntail\n"
    );
    assert_eq!(preserved(html, "script"), vec![written; ROUTES.len()]);
}

/// Each text that the cleanup removes, in the name of the attribute, in its value and in the
/// two, with the script of the author before and after the script of the page.
#[test]
fn a_script_keeps_its_json_for_each_text_that_the_cleanup_removes_from_a_name_or_a_value() {
    for [lead, removed] in REMOVED {
        for [in_name, in_value] in [[removed, ""], ["", removed], [removed, removed]] {
            let authored = script_with_a_mark_in_parts(lead, in_name, in_value, AUTHORED);
            let before = format!("{authored}{}<p>tail</p>", script(WITH_TAG));
            assert_eq!(raw_json(&before), as_written(&[AUTHORED, WITH_TAG]), "{before:?}");
            let after = format!("{}{authored}<p>tail</p>", script(WITH_TAG));
            assert_eq!(raw_json(&after), as_written(&[WITH_TAG, AUTHORED]), "{after:?}");
        }
    }
}

/// The script of the page is hidden and the cleanup removes it, so the page that the parser
/// reads has the name of its mark once, on the script of the author. That script keeps its own
/// JSON, a `&lt;` of it too, and its attribute.
#[test]
fn a_script_keeps_its_json_when_the_cleanup_joins_the_mark_of_a_script_that_it_removes() {
    for [gone, stays] in [[WITH_TAG, AUTHORED], [LT, REFERENCE]] {
        let authored = script_with_a_mark_in_parts("<?x?>", "<?y?>", "<?z?>", stays);
        let html = format!("<div hidden>{}</div>{authored}<p>tail</p>", script(gone));
        assert_eq!(raw_json(&html), as_written(&[stays]), "{html:?}");
        let written = format!("<script {AUTHORED_MARK} type=\"application/ld+json\">{stays}</script>\n\ntail\n");
        assert_eq!(preserved(&html, "script"), vec![written; ROUTES.len()], "{html:?}");
    }
}

/// An attribute that the author wrote whole, with no text for the cleanup to remove, in each
/// case of its letters, before and after the script of the page.
#[test]
fn a_script_keeps_an_attribute_of_the_author_with_the_name_and_the_value_of_an_owner_mark() {
    for name in ["data-h2m-script-0", "DATA-H2M-SCRIPT-0"] {
        let authored =
            format!(r#"<script type="application/ld+json" {name}="h2m_script_body_0_end">{AUTHORED}</script>"#);
        let before = format!("{authored}{}<p>tail</p>", script(WITH_TAG));
        assert_eq!(raw_json(&before), as_written(&[AUTHORED, WITH_TAG]), "{before:?}");
        let written = format!(
            "<script {AUTHORED_MARK} type=\"application/ld+json\">{AUTHORED}</script>\n\n\
             <script type=\"application/ld+json\">{{\"a\":\"&lt;b>\"}}</script>\n\ntail\n"
        );
        assert_eq!(preserved(&before, "script"), vec![written; ROUTES.len()], "{before:?}");
        let after = format!("{}{authored}<p>tail</p>", script(WITH_TAG));
        assert_eq!(raw_json(&after), as_written(&[WITH_TAG, AUTHORED]), "{after:?}");
    }
}

/// The name and the value of an owner mark on a `div`, written whole or joined by the
/// cleanup, stay on the `div`: only a script has an owner.
#[test]
fn a_div_keeps_an_attribute_with_the_name_and_the_value_of_an_owner_mark() {
    let whole = format!("<div {AUTHORED_MARK}>d</div>");
    let joined = r#"<<?x?>div data-h2m-script-<?y?>0="h2m_script_body_<?z?>0_end">d</div>"#;
    for div in [whole.as_str(), joined] {
        let html = format!("{div}{}<p>tail</p>", script(WITH_TAG));
        let written = format!("<div {AUTHORED_MARK}>d</div>\n\ntail\n");
        assert_eq!(preserved(&html, "div"), vec![written; ROUTES.len()], "{html:?}");
        assert_eq!(
            content_with(&html, |_| {}),
            vec!["d\n\ntail\n"; ROUTES.len()],
            "{html:?}"
        );
    }
}

/// The text of a `textarea` is the text of the author. A start tag of a script there, with
/// the name and the value of an owner mark, stays as written: in a `textarea` with an end tag
/// the cleanup removes nothing, and in one with no end tag it joins the name. The script of
/// the page is hidden, so the pass reads the text of the tree for its mark.
#[test]
fn the_text_of_a_textarea_keeps_the_name_and_the_value_of_an_owner_mark() {
    let hidden = format!("<div hidden>{}</div>", script(WITH_TAG));
    let authored = script_with_a_mark_in_parts("<?x?>", "<?y?>", "<?z?>", AUTHORED);
    let closed = format!("<textarea>{authored}</textarea>{hidden}<p>tail</p>");
    assert_eq!(
        content_with(&closed, |_| {}),
        vec![format!("{authored}\n\ntail\n"); ROUTES.len()],
        "{closed:?}"
    );
    let open = format!("{hidden}<textarea>{authored}<p>tail</p>");
    let written = format!("<script type=\"application/ld+json\" {AUTHORED_MARK}>{AUTHORED}</script><p>tail</p>\n");
    assert_eq!(content_with(&open, |_| {}), vec![written; ROUTES.len()], "{open:?}");
}

/// The plain writer writes no owner mark.
#[test]
fn the_plain_output_holds_no_owner_mark() {
    for [first, second] in BOTH_ORDERS {
        let html = format!("<p>foo {} and {} bar</p>", script(first), script(second));
        for tier_strategy in ROUTES {
            let options = ConversionOptions {
                tier_strategy,
                output_format: OutputFormat::Plain,
                ..ConversionOptions::default()
            };
            let content = convert(&html, Some(options))
                .expect("conversion should succeed")
                .content
                .unwrap_or_default();
            assert!(content.contains("foo"), "{tier_strategy:?}: {content:?}");
            assert!(!content.contains(MARK), "{tier_strategy:?}: {content:?}");
        }
    }
}

/// The conversion builds its context for the tree that it reads only (pull request 882): the
/// first parse of a page with each end tag, the second parse of a page of paragraphs with no
/// end tag, which the tree builder writes again. The owner marks are taken out of that tree
/// and the records go with it, so each of two scripts with one text in the tree gives its own
/// JSON.
#[test]
fn the_scripts_give_each_its_own_json_in_the_tree_that_the_conversion_reads() {
    for paragraph in ["<p>x </p>", "<p>x "] {
        for [first, second] in BOTH_ORDERS {
            let html = format!("{}{}{}", paragraph.repeat(20), script(first), script(second));
            assert_eq!(raw_json(&html), as_written(&[first, second]), "{html:?}");
        }
    }
}

/// The repair of a script with no end tag (#875) writes its owner mark before the cleanup of
/// the conversion. The cleanup joins the parts of a name, a value and a text of the author into
/// that mark. The repair reserves the names of the cleaned page, so the script of the author
/// does not get the JSON of the repaired script: it is written with its own text, and the
/// repaired script with its JSON. A preserved script of the body gives no metadata, so the
/// content is the proof.
#[test]
fn a_repaired_script_keeps_its_json_when_the_cleanup_joins_its_mark() {
    const GAP: &str = "<span hidden>x</span>";
    const JSON: &str = r#"{"a":"<b>"}"#;
    let forged = script_with_a_mark_in_parts(GAP, GAP, GAP, &format!("h2m_script_body_{GAP}0_end"));
    let html = format!("{forged}<p>m</p>{OPEN}{JSON}<p>tail</p>");
    let text_of_the_author = ">h2m_script_body_0_end</script>\n\nm\n\n";
    let repaired = format!("{}\n\ntail\n", script(JSON));
    for (route, content) in ROUTES.into_iter().zip(preserved(&html, "script")) {
        assert_eq!(content.matches(JSON).count(), 1, "{route:?}: {content:?}");
        assert!(content.contains(text_of_the_author), "{route:?}: {content:?}");
        assert!(content.ends_with(&repaired), "{route:?}: {content:?}");
    }
}

/// The first pass marks a script with an end tag whose text holds `<`. The repair of a script
/// with no end tag (#875) keeps the bytes of a preserved script under a mark of the same name.
/// No two marks of a page have one number: each script gives its own JSON as written, with one
/// script of each kind and with two, in the two orders. With `preserve_tags` a script of the
/// body gives no metadata, and each script is written with its own text and no mark: the kept
/// script as the tree has it, the repaired script as the page has it.
#[test]
fn a_marked_script_and_a_repaired_script_give_their_own_json() {
    const WITH_END_TAG: [&str; 2] = [r#"{"closed":"<1>"}"#, r#"{"closed":"<2> &lt;"}"#];
    const NO_END_TAG: [&str; 2] = [r#"{"unclosed":"<3> &amp;"}"#, r#"{"unclosed":"&lt;4>"}"#];
    for count in [1, 2] {
        let with_end_tag: String = WITH_END_TAG[..count].iter().map(|body| script(body)).collect();
        let no_end_tag = NO_END_TAG[..count]
            .iter()
            .map(|body| format!("{OPEN}{body}<p>tail</p>"))
            .collect::<Vec<_>>()
            .concat();
        let with_end_tag_first: Vec<&str> = WITH_END_TAG[..count]
            .iter()
            .chain(&NO_END_TAG[..count])
            .copied()
            .collect();
        let no_end_tag_first: Vec<&str> = NO_END_TAG[..count]
            .iter()
            .chain(&WITH_END_TAG[..count])
            .copied()
            .collect();
        let kept: Vec<String> = WITH_END_TAG[..count]
            .iter()
            .map(|body| script(&body.replace('<', "&lt;")))
            .collect();
        let repaired: Vec<String> = NO_END_TAG[..count]
            .iter()
            .flat_map(|body| [script(body), "tail".to_owned()])
            .collect();
        for (html, bodies, blocks) in [
            (
                format!("{with_end_tag}{no_end_tag}"),
                with_end_tag_first,
                [kept.clone(), repaired.clone()].concat(),
            ),
            (
                format!("{no_end_tag}{with_end_tag}"),
                no_end_tag_first,
                [repaired, kept].concat(),
            ),
        ] {
            assert_eq!(raw_json(&html), as_written(&bodies), "{html:?}");
            let written = format!("{}\n", blocks.join("\n\n"));
            for (route, content) in ROUTES.into_iter().zip(preserved(&html, "script")) {
                assert_eq!(content, written, "{route:?}: {html:?}");
            }
        }
    }
}

#[cfg(feature = "visitor")]
mod with_a_visitor {
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    use html_to_markdown_rs::NodeContext;
    use html_to_markdown_rs::visitor::{HtmlVisitor, VisitResult};

    use super::{BOTH_ORDERS, MARK, as_written, raw_json_with, script};

    /// The attributes of each script that the visitor was shown, in the order of the walk.
    type Shown = Arc<Mutex<Vec<BTreeMap<String, String>>>>;

    /// Skips the element with the id `gone` and keeps the attributes of each script.
    #[derive(Debug)]
    struct SkipVisitor {
        shown: Shown,
    }

    impl HtmlVisitor for SkipVisitor {
        fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
            if ctx.tag_name == "script" {
                self.shown
                    .lock()
                    .expect("the shown scripts")
                    .push(ctx.attributes().clone());
            }
            if ctx.attributes().get("id").is_some_and(|id| id == "gone") {
                VisitResult::Skip
            } else {
                VisitResult::Continue
            }
        }
    }

    /// A visitor skips one of the two scripts. The script that stays gives its own JSON, and
    /// the visitor is shown no owner mark.
    #[test]
    fn a_script_gives_its_own_json_when_a_visitor_skips_a_script_with_its_text() {
        for [removed, kept] in BOTH_ORDERS {
            let skipped = format!(r#"<script id="gone" type="application/ld+json">{removed}</script>"#);
            for html in [
                format!("{skipped}{}<p>visible</p>", script(kept)),
                format!("{}{skipped}<p>visible</p>", script(kept)),
            ] {
                let shown = Shown::default();
                let found = raw_json_with(&html, |options| {
                    options.visitor = Some(Arc::new(Mutex::new(SkipVisitor {
                        shown: Arc::clone(&shown),
                    })));
                });
                assert_eq!(found, as_written(&[kept]), "{html:?}");
                let shown = shown.lock().expect("the shown scripts");
                // ~keep Each of the two routes shows the two scripts.
                assert_eq!(shown.len(), 4, "{html:?}");
                for attributes in shown.iter() {
                    assert!(attributes.keys().all(|name| !name.starts_with(MARK)), "{attributes:?}");
                    assert_eq!(
                        attributes.get("type").map(String::as_str),
                        Some("application/ld+json"),
                        "{attributes:?}"
                    );
                    assert!(attributes.len() <= 2, "{attributes:?}");
                }
            }
        }
    }
}
