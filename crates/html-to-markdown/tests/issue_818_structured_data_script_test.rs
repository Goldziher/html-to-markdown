//! Issue 818: the content of a structured data script is raw text. A tag in a JSON string is not
//! markup, so it must not take the rest of the page.

#![allow(missing_docs)]
// ~keep The converter is chosen by name, which the crate offers under this feature only.
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

const STRATEGIES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2];

fn content(html: &str, tier_strategy: TierStrategy, extract_metadata: bool) -> String {
    let options = ConversionOptions {
        tier_strategy,
        extract_metadata,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
        .trim_end()
        .to_string()
}

/// Each converter and the automatic choice give `expected`, with the front matter on and off.
fn assert_page(html: &str, expected: &str) {
    for tier_strategy in STRATEGIES {
        for extract_metadata in [true, false] {
            assert_eq!(
                content(html, tier_strategy, extract_metadata),
                expected,
                "{tier_strategy:?}, extract_metadata {extract_metadata}: {html}"
            );
        }
    }
}

fn shop_page(open_tag: &str, body: &str, close_tag: &str) -> String {
    format!("<html><head>{open_tag}{body}{close_tag}</head><body><h1>Shop</h1><p>visible</p></body></html>")
}

const JSON_LD: &str = r#"<script type="application/ld+json">"#;

#[test]
fn the_page_of_the_issue_keeps_its_text() {
    assert_page(
        r#"<html><head><script type="application/ld+json">{"a":"<p>"}</script></head><body><p>visible</p></body></html>"#,
        "visible",
    );
}

#[test]
fn a_start_tag_in_a_text_value_does_not_take_the_page() {
    for value in [
        "Nice",
        "<p>Nice</p>",
        "<template>x</template>",
        "<template>",
        "<p>",
        "<li>one",
        "<b>",
        "<div>",
        "<div hidden>",
        "<span style='display:none'>",
        "<my-widget>",
        "<svg>",
        "<style>",
        "<script>",
        "<textarea>",
        "<title>",
        "<table><tr><td>",
        "<ul><li>",
        "<menu>",
        "<!-- note",
        "<![CDATA[",
        "<p",
        "<a href='",
        "a < b",
        "</p>",
        "</div></body></html>",
        "<img src='/a.png'>",
        "<br>",
    ] {
        let body = format!(r#"{{"a":"{value}"}}"#);
        assert_page(&shop_page(JSON_LD, &body, "</script>"), "# Shop\n\nvisible");
    }
}

#[test]
fn the_type_is_read_in_every_spelling() {
    for open_tag in [
        r#"<SCRIPT TYPE="APPLICATION/LD+JSON">"#,
        "<script type='application/ld+json'>",
        "<script type=application/ld+json>",
        r#"<script type="application/ld+json; charset=utf-8">"#,
        r#"<script id="data" type = "application/ld+json" data-a="1">"#,
        "<script\ntype=\"application/ld+json\"\n>",
        r#"<script type=" application/ld+json ">"#,
        r#"<script type="application/ld&#43;json">"#,
        r#"<script type="application/ld&#x2B;json">"#,
        r#"<script type="application/ld&plus;json">"#,
    ] {
        assert_page(&shop_page(open_tag, r#"{"a":"<P>"}"#, "</script>"), "# Shop\n\nvisible");
    }
}

/// A script ends at `</script` and then white space, `/` or `>`, in any letter case.
#[test]
fn the_end_tag_is_read_in_every_spelling() {
    for close_tag in [
        "</SCRIPT>",
        "</Script>",
        "</script >",
        "</script\n>",
        "</script\t>",
        "</script/>",
        "</SCRIPT />",
        r#"</script data-a="1">"#,
    ] {
        for open_tag in [JSON_LD, r#"<SCRIPT type="application/ld+json">"#, "<script>"] {
            assert_page(&shop_page(open_tag, r#"{"a":"<p>"}"#, close_tag), "# Shop\n\nvisible");
        }
        let style_close_tag = close_tag.to_ascii_lowercase().replace("script", "style");
        assert_page(
            &format!(r#"<p>before</p><style>p::before {{ content: "<p>"; }}{style_close_tag}<p>visible</p>"#),
            "before\n\nvisible",
        );
    }
}

/// A longer name with the same start does not end the script.
#[test]
fn a_longer_name_does_not_end_the_script() {
    for body in [
        r#"{"a":"</scripts><p>"}"#,
        r#"{"a":"</script-x><p>"}"#,
        r#"{"a":"</scriptx"}"#,
    ] {
        for open_tag in [JSON_LD, "<script>"] {
            assert_page(&shop_page(open_tag, body, "</script>"), "# Shop\n\nvisible");
        }
    }
}

#[test]
fn a_script_of_another_type_is_raw_text_too() {
    for open_tag in [
        "<script>",
        r#"<script type="application/json">"#,
        r#"<script type="text/template">"#,
        r#"<script type="module">"#,
        r#"<script type="application/ld+jsonx">"#,
        r#"<script type="x-application/ld+json">"#,
        r#"<script data-type="application/ld+json">"#,
    ] {
        assert_page(&shop_page(open_tag, r#"{"a":"<p>"}"#, "</script>"), "# Shop\n\nvisible");
    }
}

#[test]
fn a_structured_data_script_in_the_body_does_not_take_the_text_after_it() {
    assert_page(
        r#"<p>before</p><script type="application/ld+json">{"a":"<p>"}</script><p>after</p>"#,
        "before\n\nafter",
    );
    assert_page(
        r#"<ul><li>one<script type="application/ld+json">{"a":"<li>"}</script></li><li>two</li></ul>"#,
        "- one\n- two",
    );
}

#[test]
fn several_structured_data_scripts_keep_the_text_between_them() {
    assert_page(
        concat!(
            r#"<script type="application/ld+json">{"a":"<div>"}</script><p>one</p>"#,
            r#"<script type="application/ld+json">{"a":"<span>"}</script><p>two</p>"#,
            r#"<script>var a = "<p>";</script><p>three</p>"#,
        ),
        "one\n\ntwo\n\nthree",
    );
}

/// A style element and a comment: a browser does not read their content as markup.
#[test]
fn an_unclosed_tag_in_other_raw_text_does_not_take_the_page() {
    for hidden in [
        r#"<style>p::before { content: "<p>"; }</style>"#,
        "<!-- <p> -->",
        "<!-- <div> <script> -->",
    ] {
        assert_page(&format!("<p>before</p>{hidden}<p>visible</p>"), "before\n\nvisible");
    }
}

#[cfg(feature = "metadata")]
mod structured_data {
    use super::{JSON_LD, shop_page};
    use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

    /// The JSON of each script, by the automatic choice and by the full converter.
    fn raw_json(html: &str) -> Vec<Vec<String>> {
        [TierStrategy::Auto, TierStrategy::Tier2]
            .into_iter()
            .map(|tier_strategy| {
                let options = ConversionOptions {
                    tier_strategy,
                    ..ConversionOptions::default()
                };
                convert(html, Some(options))
                    .expect("conversion should succeed")
                    .metadata
                    .structured_data
                    .into_iter()
                    .map(|data| data.raw_json)
                    .collect()
            })
            .collect()
    }

    /// The JSON is the text of the script as written: a tag in a value stays in the value.
    #[test]
    fn the_json_keeps_the_tags_of_its_text_values() {
        for body in [
            r#"{"@type":"Product","description":"<p>"}"#,
            r#"{"@type":"Product","description":"<p>Nice</p>"}"#,
            r#"{"@type":"Product","description":"<ul><li>one<li>two</ul> a < b <"}"#,
            r#"{"@type":"Product","description":"<img src='/a.png'><br><!-- note"}"#,
            r#"{"@type":"Product","description":"<<p>>"}"#,
            r#"{"@type":"Product","description":"plain"}"#,
        ] {
            let found = raw_json(&shop_page(JSON_LD, body, "</script>"));
            assert_eq!(found, vec![vec![body.to_string()]; 2], "{body}");
        }
    }

    const BODIES_WITH_A_REFERENCE: [&str; 8] = [
        r#"{"a":"&lt;p&gt;"}"#,
        r#"{"a":"Tom &amp; Jerry"}"#,
        r#"{"a":"&amp;lt;"}"#,
        r#"{"a":"a < b &lt; c"}"#,
        r#"{"a":"1 <2 &lt;3 &#60;4 &#x3C;5"}"#,
        r#"{"a":"<b>x</b> &lt;b&gt;"}"#,
        r#"{"a":"Q&A & more &"}"#,
        r#"{"a":"&amp;amp;"}"#,
    ];

    /// Issue 824: a script is raw text, so a character reference in the JSON is not decoded.
    #[test]
    fn a_reference_in_the_json_is_kept_as_written() {
        for body in BODIES_WITH_A_REFERENCE {
            let found = raw_json(&shop_page(JSON_LD, body, "</script>"));
            assert_eq!(found, vec![vec![body.to_string()]; 2], "{body}");
        }
    }

    /// Issue 824: a page that the converter parses a second time, and a third time, gives the
    /// JSON as written too. An omitted end tag asks for one more parse and a custom element for
    /// another.
    #[test]
    fn a_reference_in_the_json_is_kept_as_written_on_a_repaired_page() {
        for page_body in [
            "<p>one<div>two</div>",
            "<my-part>one</my-part><p>two</p>",
            "<my-part><p>one<div>two</div></my-part>",
            "<b><p>one</b>two</p>",
        ] {
            for body in BODIES_WITH_A_REFERENCE {
                let html = format!("<html><head>{JSON_LD}{body}</script></head><body>{page_body}</body></html>");
                assert_eq!(raw_json(&html), vec![vec![body.to_string()]; 2], "{page_body} {body}");
            }
        }
    }

    /// The type is the decoded value of the attribute, and the end tag has any spelling.
    #[test]
    fn the_json_is_found_for_every_spelling_of_the_tags() {
        for (open_tag, close_tag) in [
            (JSON_LD, "</SCRIPT>"),
            (JSON_LD, "</script >"),
            (JSON_LD, "</script/>"),
            (r#"<SCRIPT type="APPLICATION/LD+JSON; charset=utf-8">"#, "</script\n>"),
            (r#"<script type=" application/ld+json ">"#, "</script>"),
            (r#"<script type="application/ld&#43;json">"#, "</script>"),
            (r#"<script type="application/ld&#x2B;json">"#, "</script>"),
            (r#"<script type="application/ld&plus;json">"#, "</script>"),
        ] {
            let found = raw_json(&shop_page(open_tag, r#"{"a":"<p>"}"#, close_tag));
            assert_eq!(
                found,
                vec![vec![r#"{"a":"<p>"}"#.to_string()]; 2],
                "{open_tag} {close_tag}"
            );
        }
        for open_tag in [
            r#"<script type="application/ld+jsonx">"#,
            r#"<script data-type="application/ld+json">"#,
        ] {
            let found = raw_json(&shop_page(open_tag, r#"{"a":"<p>"}"#, "</script>"));
            assert_eq!(found, vec![Vec::<String>::new(); 2], "{open_tag}");
        }
    }

    #[test]
    fn each_script_gives_its_own_json() {
        let html = concat!(
            r#"<html><head><script type="application/ld+json">{"a":"<div>"}</script></head><body><p>one</p>"#,
            r#"<script type="application/ld+json">{"b":"<p>"}</script><p>two</p></body></html>"#,
        );
        let expected = vec![r#"{"a":"<div>"}"#.to_string(), r#"{"b":"<p>"}"#.to_string()];
        assert_eq!(raw_json(html), vec![expected; 2]);
    }
}
