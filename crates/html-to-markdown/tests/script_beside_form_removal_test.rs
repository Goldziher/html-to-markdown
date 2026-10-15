//! A structured data script whose JSON holds a `<`, with an end tag and with none, on a page
//! where another pass removes or reads bytes before the first pass: the removal of a form, the
//! read of a `base` in a hidden element, and an exclusion beside `svg` and `math`. The JSON is
//! a record as written where the script stays, and nothing of it is text of the page.

#![allow(missing_docs)]
// ~keep The converter is chosen by name, which the crate offers under this feature only.
#![cfg(all(feature = "testkit", feature = "metadata"))]

use html_to_markdown_rs::{ConversionOptions, HiddenContent, OutputFormat, TierStrategy, convert};

const ROUTES: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];

const OPEN: &str = r#"<script type="application/ld+json">"#;
const JSON: &str = r#"{"a":"<b> & c"}"#;

/// The script of [`JSON`] with an end tag, and with none.
const END_TAGS: [&str; 2] = ["</script>", ""];

/// The content of a page and the JSON of each of its scripts, by the automatic choice and by
/// the full converter, with the options that `configure` sets.
fn content_and_raw_json(html: &str, configure: impl Fn(&mut ConversionOptions)) -> Vec<(String, Vec<String>)> {
    ROUTES
        .into_iter()
        .map(|tier_strategy| {
            let mut options = ConversionOptions {
                tier_strategy,
                ..ConversionOptions::default()
            };
            configure(&mut options);
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

/// A script in a form that is removed is removed with the form: the page has no record of it
/// and no text of it, and the text after the form is written.
#[test]
fn a_script_in_a_removed_form_gives_no_record_and_no_text() {
    for end_tag in END_TAGS {
        let html = format!(r#"<p>Va</p><form>Fa{OPEN}{JSON}{end_tag}<input value="Fv"></form><p>Tb</p>"#);
        assert_eq!(
            content_and_raw_json(&html, |_| {}),
            on_each_route("Va\n\nTb", &[]),
            "{html:?}"
        );
    }
}

/// A form that leaves an element open ends at its end tag (#879). The script after it keeps
/// its record as written, and the text after the script is written.
#[test]
fn a_script_after_a_removed_form_with_an_open_element_keeps_its_record() {
    for end_tag in END_TAGS {
        let html = format!("<p>Va</p><form>Fa<span>Fb</form>{OPEN}{JSON}{end_tag}<p>Tb</p><b>Tc</b>");
        assert_eq!(
            content_and_raw_json(&html, |_| {}),
            on_each_route("Va\n\nTb\n\n**Tc**", &[JSON]),
            "{html:?}"
        );
    }
}

/// A form that the caller keeps is read as written: its script gives its record as written,
/// and the text of the form is written with no text of the script.
#[test]
fn a_script_in_a_kept_form_gives_its_record_as_written() {
    for end_tag in END_TAGS {
        let html = format!(r#"<p>Va</p><form>Fa{OPEN}{JSON}{end_tag}<input value="Fv"></form><p>Tb</p>"#);
        assert_eq!(
            content_and_raw_json(&html, |options| {
                options.preprocessing.remove_forms = false;
            }),
            on_each_route("Va\n\nFa\n\nTb", &[JSON]),
            "{html:?}"
        );
    }
}

/// The `base` of a hidden element is the base of the page when a form is removed. The script
/// beside it gives its record as written.
#[test]
fn a_script_beside_a_hidden_base_gives_its_record_and_the_base_stays() {
    for end_tag in END_TAGS {
        let html = format!(
            "<div hidden><base href='https://b.test/d/'></div><form>Fa<input value=\"Fv\"></form>\
             {OPEN}{JSON}{end_tag}<a href='p'>Tl</a>"
        );
        assert_eq!(
            content_and_raw_json(&html, |options| {
                options.base_url = Some("https://o.test/x/".to_string());
            }),
            on_each_route("---\nbase: https://b.test/d/\n---\n\n[Tl](https://b.test/d/p)", &[JSON]),
            "{html:?}"
        );
    }
}

/// An exclusion that removes a `template` of the page reads a page with a script in `svg` or
/// in `math`. The script outside gives its record as written under each choice of hidden
/// content, the script in the graphic has no record, and no text of the `template` is written.
#[test]
fn a_script_beside_a_graphic_with_a_script_and_an_excluded_template_keeps_its_record() {
    const SVG: &str = "a\n\n![s](data:image/svg+xml;base64,\
                       PHN2Zz48c2NyaXB0IHR5cGU9ImFwcGxpY2F0aW9uL2xkK2pzb24iPnsiaW4iOjF9PC9zY3JpcHQ+PHRleHQ+czwvdGV4dD48L3N2Zz4=)\n\nt";
    for (graphic, content) in [
        (format!(r#"<svg>{OPEN}{{"in":1}}</script><text>s</text></svg>"#), SVG),
        (
            format!(r#"<math><mi>m</mi>{OPEN}{{"in":"x &amp; y"}}</script></math>"#),
            "a\n\nm{\"in\":\"x & y\"}\n\nt",
        ),
    ] {
        for hidden_content in [HiddenContent::Drop, HiddenContent::All] {
            for end_tag in END_TAGS {
                let html =
                    format!(r#"<p>a</p>{graphic}<template class="x"><p>X</p></template>{OPEN}{JSON}{end_tag}<p>t</p>"#);
                assert_eq!(
                    content_and_raw_json(&html, |options| {
                        options.exclude_selectors = vec![".x".to_string()];
                        options.hidden_content = hidden_content;
                    }),
                    on_each_route(content, &[JSON]),
                    "{html:?} {hidden_content:?}"
                );
            }
        }
    }
}

/// The page of the decision: a script before `<html>`, with an end tag and with none.
fn script_before_html(end_tag: &str) -> String {
    format!("{OPEN}{JSON}{end_tag}<html><head><title>T</title></head><body><p>v</p></body></html>")
}

/// A script with no end tag before `<html>` keeps its record as plain text with
/// `preserve_tags = ["script"]`, as on the integration branch. The same script with an end tag
/// has no record there, as on the integration branch. No text of the script is written.
#[test]
fn a_preserved_script_before_html_gives_as_plain_text_no_record_with_an_end_tag_and_its_record_with_none() {
    for (end_tag, raw_json) in END_TAGS.into_iter().zip([&[][..], &[JSON][..]]) {
        let html = script_before_html(end_tag);
        assert_eq!(
            content_and_raw_json(&html, |options| {
                options.output_format = OutputFormat::Plain;
                options.preserve_tags = vec!["script".to_string()];
            }),
            on_each_route("v", raw_json),
            "{html:?}"
        );
    }
}

/// A script with no end tag before the document element keeps its record with
/// `preserve_tags = ["script"]`, as plain text and as Markdown: before `<html>`, before
/// `<head>` and before the doctype. Nothing of the script is written.
#[test]
fn a_preserved_script_with_no_end_tag_before_the_document_element_keeps_its_record() {
    for (rest, plain, markdown) in [
        (
            "<html><head><title>T</title></head><body><p>v</p></body></html>",
            "v",
            "---\ntitle: T\n---\n\nv",
        ),
        (
            "<head><title>x</title></head><body><p>t</p>",
            "t",
            "---\ntitle: x\n---\n\nt",
        ),
        (
            "<!DOCTYPE html><html><head><title>T</title></head><body><p>v</p></body></html>",
            "v",
            "---\ntitle: T\n---\n\nv",
        ),
    ] {
        let html = format!("{OPEN}{JSON}{rest}");
        for (as_plain_text, content) in [(true, plain), (false, markdown)] {
            assert_eq!(
                content_and_raw_json(&html, |options| {
                    if as_plain_text {
                        options.output_format = OutputFormat::Plain;
                    }
                    options.preserve_tags = vec!["script".to_string()];
                }),
                on_each_route(content, &[JSON]),
                "{html:?} plain text: {as_plain_text}"
            );
        }
    }
}

#[cfg(feature = "visitor")]
mod with_a_visitor {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use html_to_markdown_rs::NodeContext;
    use html_to_markdown_rs::visitor::{HtmlVisitor, VisitResult};

    use super::{END_TAGS, JSON, content_and_raw_json, on_each_route, script_before_html};

    /// The count of the scripts that the visitor was shown.
    type Shown = Arc<AtomicUsize>;

    /// Counts each script that it is shown and changes nothing.
    #[derive(Debug)]
    struct CountScripts {
        shown: Shown,
    }

    impl HtmlVisitor for CountScripts {
        fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
            if ctx.tag_name == "script" {
                self.shown.fetch_add(1, Ordering::Relaxed);
            }
            VisitResult::Continue
        }
    }

    /// Decision: a visitor is shown a script with no end tag before `<html>` as often as the
    /// same script with an end tag, and the page gives the same content and the same record.
    #[test]
    fn a_visitor_is_shown_a_script_before_html_with_an_end_tag_and_with_none() {
        let shown = [Shown::default(), Shown::default()];
        for (end_tag, shown) in END_TAGS.into_iter().zip(&shown) {
            let html = script_before_html(end_tag);
            assert_eq!(
                content_and_raw_json(&html, |options| {
                    options.visitor = Some(Arc::new(Mutex::new(CountScripts {
                        shown: Arc::clone(shown),
                    })));
                }),
                on_each_route("---\ntitle: T\n---\n\nv", &[JSON]),
                "{html:?}"
            );
        }
        let [with_an_end_tag, with_none] = shown.map(|count| count.load(Ordering::Relaxed));
        assert_ne!(with_an_end_tag, 0, "the visitor is shown the script with an end tag");
        assert_eq!(with_an_end_tag, with_none, "with an end tag, and with none");
    }

    /// A script with no end tag before `<html>` keeps its record as plain text with a visitor,
    /// with `preserve_tags = ["script"]` and without it.
    #[test]
    fn a_script_with_no_end_tag_before_html_keeps_its_record_as_plain_text_with_a_visitor() {
        let html = script_before_html("");
        for preserves_script in [true, false] {
            let records: Vec<Vec<String>> = content_and_raw_json(&html, |options| {
                options.output_format = html_to_markdown_rs::OutputFormat::Plain;
                if preserves_script {
                    options.preserve_tags = vec!["script".to_string()];
                }
                options.visitor = Some(Arc::new(Mutex::new(CountScripts {
                    shown: Shown::default(),
                })));
            })
            .into_iter()
            .map(|(_, raw_json)| raw_json)
            .collect();
            assert_eq!(
                records,
                vec![vec![JSON.to_string()]; 2],
                "{html:?} preserve_tags: {preserves_script}"
            );
        }
    }
}
