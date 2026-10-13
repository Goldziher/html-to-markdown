//! The start tag of a structured data script ends at one place for the repair of a script with
//! no end tag, for the first pass and for the parser: the HTML rule of a `/` that stands alone
//! and of an `=` with no name of an attribute before it.

#![allow(missing_docs)]
// ~keep The converter is chosen by name, which the crate offers under this feature only.
#![cfg(all(feature = "testkit", feature = "metadata"))]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

const ROUTES: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];

/// The content of a page and the JSON of each of its scripts, by the automatic choice and by
/// the full converter.
fn content_and_raw_json(html: &str) -> Vec<(String, Vec<String>)> {
    ROUTES
        .into_iter()
        .map(|tier_strategy| {
            let options = ConversionOptions {
                tier_strategy,
                ..ConversionOptions::default()
            };
            let result = convert(html, Some(options)).expect("conversion should succeed");
            let content = result.content.unwrap_or_default().trim_end().to_string();
            let raw_json = result
                .metadata
                .structured_data
                .into_iter()
                .map(|data| data.raw_json)
                .collect();
            (content, raw_json)
        })
        .collect()
}

/// The same content and the same records on the two routes.
fn on_each_route(content: &str, raw_json: &[&str]) -> Vec<(String, Vec<String>)> {
    let raw_json: Vec<String> = raw_json.iter().map(|json| (*json).to_string()).collect();
    vec![(content.to_string(), raw_json); ROUTES.len()]
}

/// A `/` that stands alone in the start tag of a structured data script is nothing by the HTML
/// rule. The JSON is a record as written, with an end tag and with none, and no text of the
/// start tag is text of the page. The text after a script with no end tag is written (#875).
#[test]
fn a_slash_that_stands_alone_in_a_script_start_tag_ends_no_tag() {
    let json = r#"{"a":"<b> & c"}"#;
    for start_tag in [
        r#"<script//type="application/ld+json">"#,
        r#"<script/type="application/ld+json"/>"#,
        r#"<script type="application/ld+json"/>"#,
        r#"<script type="application/ld+json" / id="k">"#,
    ] {
        for end_tag in ["</script>", ""] {
            let html = format!("<p>a</p>{start_tag}{json}{end_tag}<p>t</p>");
            assert_eq!(
                content_and_raw_json(&html),
                on_each_route("a\n\nt", &[json]),
                "{html:?}"
            );
        }
    }
}

/// An `=` with no name of an attribute before it is a byte of a name by the HTML rule: the
/// quote after it opens no value, and the start tag ends at its `>`. The repair of a script
/// with no end tag and the first pass end the start tag at that one place, so the JSON is the
/// record and the text after the script is no part of it.
#[test]
fn an_equals_sign_with_no_name_before_it_opens_no_value_in_a_script_start_tag() {
    let json = r#"{"a":1}"#;
    for start_tag in [
        r#"<script type="application/ld+json" =">"#,
        r#"<script type="application/ld+json" x="1"=">"#,
    ] {
        for end_tag in ["</script>", ""] {
            let html = format!("<p>a</p>{start_tag}{json}{end_tag}<p>t</p>");
            assert_eq!(
                content_and_raw_json(&html),
                on_each_route("a\n\nt", &[json]),
                "{html:?}"
            );
        }
    }
}

/// White space between the name of an attribute and its `=` is no end of the name: the quoted
/// value holds the `>`, for a script with an end tag and for one with none.
#[test]
fn white_space_before_an_equals_sign_keeps_the_value_of_the_attribute() {
    let json = r#"{"a":1}"#;
    for end_tag in ["</script>", ""] {
        let html = format!("<p>a</p><script type=\"application/ld+json\" data-x  =\">\">{json}{end_tag}<p>t</p>");
        assert_eq!(
            content_and_raw_json(&html),
            on_each_route("a\n\nt", &[json]),
            "{html:?}"
        );
    }
}

/// Decision: a stray quote before an `=` is the name of an attribute by the HTML rule, so the
/// start tag of this script ends after `<b>` and the script has no JSON. The repair, the first
/// pass and the tree builder read it so: the page gives no record, and the text after the
/// start tag is written.
#[test]
fn a_script_whose_start_tag_holds_the_start_of_its_json_gives_no_record() {
    let html = "<p>a</p><script type=\"application/ld+json\" \"=\">{\"a\":\"<b> & c\"}<p>t</p>";
    assert_eq!(content_and_raw_json(html), on_each_route("a\n\n& c\"}\n\nt", &[]));
}
