#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

#[cfg(feature = "testkit")]
const ROUTES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2];
#[cfg(not(feature = "testkit"))]
const ROUTES: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];

fn assert_output(html: &str, expected: &str) {
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
        for tier_strategy in ROUTES {
            for extract_metadata in [false, true] {
                let result = convert(
                    html,
                    ConversionOptions {
                        output_format,
                        tier_strategy,
                        extract_metadata,
                        ..Default::default()
                    },
                )
                .expect("form conversion");
                assert_eq!(
                    result.content.as_deref(),
                    Some(expected),
                    "{html:?}, {output_format:?}/{tier_strategy:?}/metadata={extract_metadata}"
                );
            }
        }
    }
}

#[test]
fn should_keep_the_text_after_an_explicit_form_end_with_an_open_child() {
    let mut rows = 0;
    for tag in ["span", "b", "em", "a", "div"] {
        for (prefix, suffix, expected) in [
            ("", "c", "c\n"),
            ("t", "c", "tc\n"),
            ("<p>t</p>", "<p>c</p>", "t\n\nc\n"),
        ] {
            assert_output(&format!("{prefix}<form>a<{tag}>b</form>{suffix}"), expected);
            rows += 1;
        }
    }
    assert_eq!(rows, 15);
}

#[test]
fn should_end_removal_at_real_form_end_tags_in_the_previously_pinned_label_cases() {
    assert_output(
        "<p>before<form><label>name<input></form>after<p>next",
        "before\n\nafter\n\nnext\n",
    );
    assert_output("<form><label>a</form>b<form>c</form>d</label>e", "bde\n");
}

#[test]
fn should_keep_removing_the_html_subtree_when_the_form_has_no_end_tag() {
    assert_output("<p>before</p><form><label>name<input>after<p>next</p>", "before\n");
    assert_output("<form>a<span>b<p>tail</p>", "");
}

#[test]
fn should_ignore_nested_form_starts_until_the_first_real_form_end() {
    assert_output("<form>a<form>b</form>c</form>d", "cd\n");
    assert_output("<form><div><form>x</form></div>y</form>z", "yz\n");
}

#[test]
fn should_keep_retained_and_preprocessing_disabled_form_content() {
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            for disabled in [false, true] {
                let mut options = ConversionOptions {
                    output_format,
                    tier_strategy,
                    ..Default::default()
                };
                options.preprocessing.remove_forms = false;
                options.preprocessing.enabled = !disabled;
                assert_eq!(
                    convert("<form>a<span>b</form>c", options)
                        .expect("retained form")
                        .content
                        .as_deref(),
                    Some("abc\n")
                );
            }
        }
    }
}

#[test]
fn should_ignore_fake_form_end_tags_in_opaque_text_comments_and_attributes() {
    for html in [
        "<form>a<span>b<!-- </form> -->c</form>d",
        "<form title='</form>'>a<span>b</form>d",
        "<form>a<script>\"</form>\"</script><span>b</form>d",
        "<form>a<template></form></template><span>b</form>d",
        "<form>a<textarea></form></textarea><span>b</form>d",
        "<form>a<svg><form>x</form></svg><span>b</form>d",
        "<form>a<svg><foreignObject><form>x</form></foreignObject></svg><span>b</form>d",
    ] {
        assert_output(html, "d\n");
    }
    let html = format!(
        "é<form>a<script>{}\"</form>\"</script><span>b</form>c",
        "x".repeat(9000)
    );
    assert_output(&html, "éc\n");
}

#[test]
fn should_keep_original_form_boundaries_and_case_after_removing_the_body() {
    assert_output("<FoRm>a<span>b</fOrM>c", "c\n");
    assert_output("<FORM\x0c>a<span>b</FORM\x0c>c", "c\n");
    assert_output("<form>a<span>b</form><title>tail</title><p>next</p>", "tail\n\nnext\n");
    assert_output(
        "<form>a<template><noscript></form></noscript><template>x</template></template><span>b</form>c<form>d<span>e</form>f",
        "cf\n",
    );
}

#[test]
fn should_keep_the_html_table_form_fostering_contract_and_reset_the_table_scope() {
    for (output_format, expected) in [
        (OutputFormat::Markdown, "| a |\n| --- |\n| b |\n\nafter\n"),
        (OutputFormat::Djot, "| a |\n|---|\n| b |\n\nafter\n"),
        (OutputFormat::Plain, "a\nb\n\nafter\n"),
    ] {
        for tier_strategy in ROUTES {
            let result = convert(
                "<table><tr><td>a</td></tr><form><tr><td>b</td></tr></form></table>after",
                ConversionOptions {
                    output_format,
                    tier_strategy,
                    ..Default::default()
                },
            )
            .expect("table form");
            assert_eq!(result.content.as_deref(), Some(expected));
        }
    }
    assert_output("<table></table><form>a<span>b</form>c", "c\n");
    assert_output("</table><form>a<span>b</form>c", "c\n");
}

#[test]
fn should_keep_preserved_ancestor_and_void_preservation_contracts() {
    let html = "<div><form>a<span>b</span></form>c</div><p>d</p>";
    let result = convert(
        html,
        ConversionOptions {
            preserve_tags: vec!["div".into()],
            ..Default::default()
        },
    )
    .expect("preserved ancestor");
    assert_eq!(
        result.content.as_deref(),
        Some("<div><form>a<span>b</span></form>c</div>\n\nd\n")
    );
    for preserved in ["form", "img"] {
        let result = convert(
            "<form>a<span>b</form>c",
            ConversionOptions {
                preserve_tags: vec![preserved.into()],
                ..Default::default()
            },
        )
        .expect("preserved tags");
        assert_eq!(result.content.as_deref(), Some("c\n"));
    }
}

#[test]
fn should_keep_minimal_preprocessing_form_content() {
    let mut options = ConversionOptions::default();
    options.preprocessing.preset = html_to_markdown_rs::PreprocessingPreset::Minimal;
    assert_eq!(
        convert("<form>a<span>b</form>c", options)
            .expect("minimal form")
            .content
            .as_deref(),
        Some("abc\n")
    );
}

#[test]
fn should_remove_forms_in_kept_template_content_without_changing_the_drop_choice() {
    for (choice, template, expected) in [
        (
            html_to_markdown_rs::HiddenContent::All,
            "template",
            "before\n\nc\n\nafter\n",
        ),
        (
            html_to_markdown_rs::HiddenContent::Reachable,
            "template shadowrootmode='open'",
            "before\n\nc\n\nafter\n",
        ),
        (
            html_to_markdown_rs::HiddenContent::Drop,
            "template",
            "before\n\nafter\n",
        ),
    ] {
        let html = format!("<p>before</p><{template}><form>a<span>b</form>c</template><p>after</p>");
        let result = convert(
            &html,
            ConversionOptions {
                hidden_content: choice,
                ..Default::default()
            },
        )
        .expect("template form");
        assert_eq!(result.content.as_deref(), Some(expected), "{html}");
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_preserve_the_existing_visitor_callback_and_malformed_form_contract() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};
    #[derive(Debug, Default)]
    struct FormVisitor(Vec<String>);
    impl HtmlVisitor for FormVisitor {
        fn visit_form(&mut self, _ctx: &NodeContext, action: Option<&str>, _method: Option<&str>) -> VisitResult {
            self.0.push(action.unwrap_or_default().into());
            VisitResult::Skip
        }
    }
    let visited = Arc::new(Mutex::new(FormVisitor::default()));
    let result = convert(
        "<form action='/outer'>a<span>b</form>c",
        ConversionOptions {
            visitor: Some(visited.clone()),
            ..Default::default()
        },
    )
    .expect("visitor form");
    // ~keep Visitor-owned removal retains its existing HTML-subtree contract; this source rule does not replace callbacks.
    assert_eq!(result.content.as_deref(), Some(""));
    assert_eq!(visited.lock().expect("visitor lock").0, ["/outer"]);
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_structured_data_and_literal_opaque_text_outside_the_removed_form() {
    let json = r#"{"@type":"Thing","name":"雪"}"#;
    for tier_strategy in ROUTES {
        let html = format!("<form>a<span>b</form><script type='application/ld+json'>{json}</script><p>tail</p>");
        let result = convert(
            &html,
            ConversionOptions {
                tier_strategy,
                extract_metadata: true,
                ..Default::default()
            },
        )
        .expect("form and metadata");
        assert_eq!(result.content.as_deref(), Some("tail\n"));
        assert_eq!(result.metadata.structured_data.len(), 1);
        assert_eq!(result.metadata.structured_data[0].raw_json, json);
    }
    assert_output(
        "<form>a<span>b</form><xmp>&lt; &amp; 雪</xmp><p>tail</p>",
        "&lt; &amp; 雪\n\ntail\n",
    );
}

#[test]
fn should_pair_nested_forms_inside_a_kept_template_independently_of_the_body_form_pointer() {
    for (html, expected) in [
        ("<template><form>a<form>b</form>c</form>d</template>", "d\n"),
        (
            "<p>before</p><template><form>a<form>b</form>c</form>d</template><p>after</p>",
            "before\n\nd\n\nafter\n",
        ),
    ] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
            for tier_strategy in ROUTES {
                let result = convert(
                    html,
                    ConversionOptions {
                        hidden_content: html_to_markdown_rs::HiddenContent::All,
                        output_format,
                        tier_strategy,
                        ..Default::default()
                    },
                )
                .expect("nested template forms");
                assert_eq!(result.content.as_deref(), Some(expected));
            }
        }
    }
}

#[test]
fn should_keep_the_nested_table_form_fallback_and_reset_after_the_outer_table() {
    assert_output(
        "<table><tr><td><table></table><form>a<span>b</form>c</td></tr></table><p>d</p>",
        "d\n",
    );
    assert_output(
        "<table><tr><td><table></table></td></tr></table><form>a<span>b</form>c",
        "c\n",
    );
    assert_output("<table></table></table><form>a<span>b</form>c", "c\n");
}

#[test]
fn should_resolve_links_against_the_document_base_before_removing_its_form() {
    for tier_strategy in ROUTES {
        for extract_metadata in [false, true] {
            let options = ConversionOptions {
                tier_strategy,
                extract_metadata,
                base_url: Some("https://o.test/x/".into()),
                ..Default::default()
            };
            let result = convert("<form><base href='https://b.test/d/'></form><a href='p'>l</a>", options)
                .expect("base in removed form");
            assert_eq!(
                result.content.as_deref().and_then(|text| text.lines().last()),
                Some("[l](https://b.test/d/p)")
            );
        }
    }
}

#[test]
fn should_keep_text_outside_the_select_that_contains_a_form_start() {
    for html in [
        "<select><option>o<form>a</select>x</form>c",
        "<select><option>o<form>a</option></select>x</form>c",
    ] {
        assert_output(html, "o xc\n");
    }
}

#[test]
fn should_keep_form_content_when_preprocessing_is_disabled_but_remove_forms_is_set() {
    for tier_strategy in ROUTES {
        let mut options = ConversionOptions {
            tier_strategy,
            ..Default::default()
        };
        options.preprocessing.enabled = false;
        assert!(options.preprocessing.remove_forms);
        assert_eq!(
            convert("<form>a<span>b</form>c", options)
                .expect("disabled preprocessing")
                .content
                .as_deref(),
            Some("abc\n")
        );
    }
}

#[test]
fn should_remove_select_descendants_of_an_outer_form_without_losing_its_tail() {
    assert_output("<form><span>a<select><option>o</select>x</form>c", "c\n");
    assert_output("<form>a<select><option>o</select>x</form>c", "c\n");
}

#[test]
fn should_keep_real_base_priority_while_ignoring_opaque_and_foreign_bases() {
    for opaque in [
        "<!-- <base href='https://fake.test/'> -->",
        r#"<script>"<base href='https://fake.test/'>"</script>"#,
        "<template><base href='https://fake.test/'></template>",
        "<svg><base href='https://fake.test/'></svg>",
        r#"<div title="<base href='https://fake.test/'>"></div>"#,
    ] {
        for tier_strategy in ROUTES {
            let html = format!(
                "<form>{opaque}<base href='https://b.test/d/'></form><base href='https://later.test/'><a href='p'>l</a>"
            );
            let result = convert(
                &html,
                ConversionOptions {
                    tier_strategy,
                    base_url: Some("https://o.test/x/".into()),
                    ..Default::default()
                },
            )
            .expect("base priority");
            assert_eq!(
                result.content.as_deref().and_then(|text| text.lines().last()),
                Some("[l](https://b.test/d/p)"),
                "{opaque:?}/{tier_strategy:?}"
            );
        }
    }
}

#[test]
fn should_keep_select_scope_with_selector_exclusion_and_preserved_ancestors() {
    for tier_strategy in ROUTES {
        for preserve in [false, true] {
            for exclusion in [false, true] {
                let options = ConversionOptions {
                    tier_strategy,
                    preserve_tags: if preserve { vec!["div".into()] } else { Vec::new() },
                    exclude_selectors: if exclusion { vec!["form".into()] } else { Vec::new() },
                    ..Default::default()
                };
                assert_eq!(
                    convert("<select><option>o<form>a</select>x</form>c", options)
                        .expect("select ownership")
                        .content
                        .as_deref(),
                    Some("o xc\n")
                );
            }
        }
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_keep_select_scope_with_a_visitor() {
    use html_to_markdown_rs::visitor::HtmlVisitor;
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct Observer;
    impl HtmlVisitor for Observer {
        fn visit_form(
            &mut self,
            _ctx: &html_to_markdown_rs::visitor::NodeContext,
            _action: Option<&str>,
            _method: Option<&str>,
        ) -> html_to_markdown_rs::visitor::VisitResult {
            html_to_markdown_rs::visitor::VisitResult::Skip
        }
    }
    for tier_strategy in ROUTES {
        assert_eq!(
            convert(
                "<select><option>o<form>a</select>x</form>c",
                ConversionOptions {
                    tier_strategy,
                    visitor: Some(Arc::new(Mutex::new(Observer))),
                    ..Default::default()
                }
            )
            .expect("select visitor")
            .content
            .as_deref(),
            Some("o xc\n")
        );
    }
}

#[test]
fn should_keep_hidden_base_priority_before_removing_visible_forms() {
    for hidden in ["hidden", "style='display:none'"] {
        for tier_strategy in ROUTES {
            let html = format!(
                "<form>a</form><div {hidden}><base href='https://fake.test/'></div><base href='https://real.test/'><a href='p'>l</a>"
            );
            let result = convert(
                &html,
                ConversionOptions {
                    tier_strategy,
                    base_url: Some("https://o.test/x/".into()),
                    ..Default::default()
                },
            )
            .expect("hidden base priority");
            assert_eq!(
                result.content.as_deref().and_then(|text| text.lines().last()),
                Some("[l](https://fake.test/p)")
            );
        }
    }
}

#[test]
fn should_keep_the_literal_form_end_policy_when_it_precedes_the_select_end() {
    assert_output("<select><option>o<form><span>a</form>c", "oc\n");
    assert_output("<select><option>o<form><span>a</form>c</select>x", "oc x\n");
    assert_output("<select><form><span>a</form>c</select>", "c\n");
}

#[test]
fn should_retain_the_form_pointer_after_select_scope_releases_it() {
    for html in [
        "<select><form>a</select><form>b</form>c</form>d",
        "<select><form>a<input><form>b</form>c</form>d",
        "<select><form>a<select><form>b</form>c</form>d",
    ] {
        assert_output(html, "bcd\n");
    }
}

#[test]
fn should_keep_literal_form_tails_when_a_scope_blocker_prevents_select_release() {
    for blocker in ["object", "applet", "marquee"] {
        for ending in ["</select>", "<input>", "<select>"] {
            assert_output(
                &format!("<select><{blocker}><form><span>a{ending}b</form>c</{blocker}></select>d"),
                "c d\n",
            );
        }
    }
}

#[test]
fn should_keep_form_tails_when_table_scope_ignores_a_select_end() {
    assert_output(
        "<select><form><span>a<table><tbody><tr><td></select>x</td></tr></tbody></table>b</form>c</select>d",
        "c d\n",
    );
}

#[test]
fn should_ignore_mismatched_scope_blocker_ends_without_losing_form_tails() {
    assert_output(
        "<select><object><applet><form><span>a</object></select>b</form>c</applet></object></select>d",
        "c d\n",
    );
}

#[test]
fn should_use_the_same_hidden_base_with_and_without_removed_forms() {
    use html_to_markdown_rs::HiddenContent;
    for hidden_content in [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All] {
        for remove_forms in [false, true] {
            for form in ["", "<form>a</form>"] {
                for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
                    for tier_strategy in ROUTES {
                        let mut options = ConversionOptions {
                            output_format,
                            tier_strategy,
                            hidden_content,
                            base_url: Some("https://o.test/x/".into()),
                            extract_metadata: true,
                            ..Default::default()
                        };
                        options.preprocessing.remove_forms = remove_forms;
                        let html = format!("<div hidden><base href='https://b.test/d/'></div>{form}<a href='p'>l</a>");
                        let result = convert(&html, options).expect("hidden base and form policy");
                        let last = if output_format == OutputFormat::Plain {
                            if !remove_forms && !form.is_empty() { "al" } else { "l" }
                        } else {
                            "[l](https://b.test/d/p)"
                        };
                        assert_eq!(
                            result.content.as_deref().and_then(|s| s.lines().last()),
                            Some(last),
                            "{hidden_content:?}/{remove_forms}/{form}/{output_format:?}/{tier_strategy:?}"
                        );
                        #[cfg(feature = "metadata")]
                        assert_eq!(result.metadata.document.base_href.as_deref(), Some("https://b.test/d/"));
                    }
                }
            }
        }
    }
}

#[test]
fn should_read_hidden_base_before_form_removal_without_collecting_metadata() {
    for extract_metadata in [false, true] {
        for tier_strategy in ROUTES {
            let options = ConversionOptions {
                base_url: Some("https://o.test/x/".into()),
                extract_metadata,
                tier_strategy,
                ..Default::default()
            };
            let result = convert(
                "<div hidden><base href='https://b.test/d/'></div><form>a</form><a href='p'>l</a>",
                options,
            )
            .expect("hidden base");
            let expected = if extract_metadata {
                "---\nbase: https://b.test/d/\n---\n\n[l](https://b.test/d/p)\n"
            } else {
                "[l](https://b.test/d/p)\n"
            };
            assert_eq!(result.content.as_deref(), Some(expected));
        }
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_keep_hidden_base_when_a_visitor_removes_a_form() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct Observer {
        forms: usize,
    }
    impl HtmlVisitor for Observer {
        fn visit_form(&mut self, _: &NodeContext, _: Option<&str>, _: Option<&str>) -> VisitResult {
            self.forms += 1;
            VisitResult::Skip
        }
    }
    for tier_strategy in ROUTES {
        let recorder = Arc::new(Mutex::new(Observer { forms: 0 }));
        let result = convert(
            "<div hidden><base href='https://b.test/d/'></div><form>a</form><a href='p'>l</a>",
            ConversionOptions {
                tier_strategy,
                base_url: Some("https://o.test/x/".into()),
                extract_metadata: true,
                visitor: Some(recorder.clone()),
                ..Default::default()
            },
        )
        .expect("visited form hidden base");
        assert_eq!(
            result.content.as_deref(),
            Some("---\nbase: https://b.test/d/\n---\n\n[l](https://b.test/d/p)\n")
        );
        assert_eq!(recorder.lock().expect("observer").forms, 1);
        #[cfg(feature = "metadata")]
        assert_eq!(result.metadata.document.base_href.as_deref(), Some("https://b.test/d/"));
    }
}

#[cfg(feature = "metadata")]
#[test]
fn should_collect_hidden_base_before_form_removal_without_a_caller_base() {
    for tier_strategy in ROUTES {
        let result = convert(
            "<div hidden><base href='https://b.test/d/'></div><form>a</form><a href='p'>l</a>",
            ConversionOptions {
                tier_strategy,
                extract_metadata: true,
                ..Default::default()
            },
        )
        .expect("metadata-only base");
        assert_eq!(
            result.content.as_deref(),
            Some("---\nbase: https://b.test/d/\n---\n\n[l](p)\n")
        );
        assert_eq!(result.metadata.document.base_href.as_deref(), Some("https://b.test/d/"));
    }
}
