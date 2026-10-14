#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

const ROUTES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2];
const FORMATS: [OutputFormat; 3] = [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot];
const SCRIPTS: [&str; 2] = [
    "<script>x</script>",
    r#"<script type="application/ld+json">{"name":"n"}</script>"#,
];

fn options(tier_strategy: TierStrategy, output_format: OutputFormat, extract_metadata: bool) -> ConversionOptions {
    ConversionOptions {
        tier_strategy,
        output_format,
        extract_metadata,
        ..Default::default()
    }
}

fn assert_matrix(html: &str, expected: &str) {
    for tier_strategy in ROUTES {
        for output_format in FORMATS {
            for extract_metadata in [false, true] {
                let result = convert(html, options(tier_strategy, output_format, extract_metadata))
                    .expect("script boundary converts");
                assert_eq!(
                    result.content.as_deref(),
                    Some(expected),
                    "{tier_strategy:?}/{output_format:?}/metadata={extract_metadata}: {html:?}"
                );
            }
        }
    }
}

#[test]
fn should_join_visible_words_across_removed_scripts_without_authored_space() {
    for script in SCRIPTS {
        assert_matrix(&format!("<p>foo{script}bar</p>"), "foobar\n");
    }
}

#[test]
fn should_join_words_across_removed_plain_script_in_forced_tier1() {
    let result = convert(
        "<p>foo<script>x</script>bar</p>",
        options(TierStrategy::Tier1, OutputFormat::Markdown, false),
    )
    .expect("forced Tier1 script boundary converts");
    assert_eq!(result.content.as_deref(), Some("foobar\n"));
}

#[test]
fn should_join_words_across_removed_plain_script_in_forced_tier2() {
    let result = convert(
        "<p>foo<script>x</script>bar</p>",
        options(TierStrategy::Tier2, OutputFormat::Markdown, false),
    )
    .expect("forced Tier2 script boundary converts");
    assert_eq!(result.content.as_deref(), Some("foobar\n"));
}

fn assert_route_outputs(html: &str, output_format: OutputFormat, expected: [&str; 3]) {
    for (tier_strategy, expected) in ROUTES.into_iter().zip(expected) {
        for extract_metadata in [false, true] {
            let result = convert(html, options(tier_strategy, output_format, extract_metadata))
                .expect("boundary control converts");
            assert_eq!(
                result.content.as_deref(),
                Some(expected),
                "{tier_strategy:?}/{output_format:?}/metadata={extract_metadata}: {html:?}"
            );
        }
    }
}

#[test]
fn should_preserve_authored_spaces_and_tabs_beside_removed_scripts() {
    for script in SCRIPTS {
        for (left, right) in [(" ", ""), ("", " "), (" ", " "), ("\t", "")] {
            let html = format!("<p>foo{left}{script}{right}bar</p>");
            for output_format in FORMATS {
                // ~keep This inherited route difference belongs to whitespace policy, outside issue #827.
                // ~keep In plain output the white space on the two sides of a kept script is one space.
                let expected = if script == SCRIPTS[0] && left == "\t" && output_format != OutputFormat::Plain {
                    ["foo\tbar\n", "foo bar\n", "foo\tbar\n"]
                } else {
                    ["foo bar\n"; 3]
                };
                assert_route_outputs(&html, output_format, expected);
            }
        }
    }
}

#[test]
fn should_keep_a_separator_when_removing_style_content_between_words() {
    assert_matrix("<p>foo<style>x</style>bar</p>", "foo bar\n");
}

#[test]
fn should_keep_preserved_json_script_html_and_join_scripts_that_are_still_removed() {
    for (script, preserved) in [
        (SCRIPTS[0], "foobar\n"),
        (
            r#"<script type="application/ld+json">{"a":1}</script>"#,
            "foo<script type=\"application/ld+json\">{\"a\":1}</script>bar\n",
        ),
    ] {
        let html = format!("<p>foo{script}bar</p>");
        for output_format in FORMATS {
            let expected = if output_format == OutputFormat::Plain {
                "foobar\n"
            } else {
                preserved
            };
            assert_preserved_script(&html, output_format, expected);
        }
    }
}

fn assert_preserved_script(html: &str, output_format: OutputFormat, expected: &str) {
    for tier_strategy in ROUTES {
        for extract_metadata in [false, true] {
            let mut options = options(tier_strategy, output_format, extract_metadata);
            options.preserve_tags = vec!["script".to_owned()];
            let result = convert(html, options).expect("preserved script converts");
            assert_eq!(
                result.content.as_deref(),
                Some(expected),
                "{tier_strategy:?}/{output_format:?}/metadata={extract_metadata}: {html:?}"
            );
        }
    }
}

#[test]
fn should_join_words_across_removed_scripts_inside_code_and_pre() {
    for script in SCRIPTS {
        for (prefix, suffix, expected) in [
            ("<p><code>foo", "bar</code></p>", "`foobar`\n"),
            ("<pre>foo", "bar</pre>", "```\nfoobar\n```\n"),
        ] {
            let html = format!("{prefix}{script}{suffix}");
            for output_format in FORMATS {
                let expected = if output_format == OutputFormat::Plain {
                    ["foobar\n", expected, "foobar\n"]
                } else {
                    [expected; 3]
                };
                assert_route_outputs(&html, output_format, expected);
            }
        }
    }
}

#[test]
fn should_preserve_escaped_script_looking_text_inside_pre() {
    let html = "<pre>foo&lt;script&gt;x&lt;/script&gt;bar</pre>";
    let fenced = "```\nfoo<script>x</script>bar\n```\n";
    for output_format in FORMATS {
        let expected = if output_format == OutputFormat::Plain {
            ["foo<script>x</script>bar\n", fenced, "foo<script>x</script>bar\n"]
        } else {
            [fenced; 3]
        };
        assert_route_outputs(html, output_format, expected);
    }
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_trimmed_json_metadata_when_joining_visible_words() {
    let html = "<p>foo<script type=\"application/ld+json\"> \n{\"name\":\"n\"}\t </script>bar</p>";
    for tier_strategy in ROUTES {
        let result =
            convert(html, options(tier_strategy, OutputFormat::Markdown, true)).expect("structured metadata converts");
        assert_eq!(result.content.as_deref(), Some("foobar\n"), "{tier_strategy:?}");
        let raw_json: Vec<&str> = result
            .metadata
            .structured_data
            .iter()
            .map(|data| data.raw_json.as_str())
            .collect();
        // ~keep Forced Tier1 currently omits metadata collection; joining must not change that API behavior.
        let expected: &[&str] = if tier_strategy == TierStrategy::Tier1 {
            &[]
        } else {
            &[r#"{"name":"n"}"#]
        };
        assert_eq!(raw_json, expected, "{tier_strategy:?}");
    }
}

#[cfg(feature = "visitor")]
mod visitors {
    use super::*;
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Copy, Debug)]
    enum ScriptAnswer {
        Continue,
        Skip,
        Custom,
    }

    #[derive(Debug)]
    struct ScriptVisitor {
        answer: ScriptAnswer,
        calls: usize,
    }

    impl HtmlVisitor for ScriptVisitor {
        fn visit_element_start(&mut self, context: &NodeContext<'_>) -> VisitResult {
            if context.tag_name != "script" {
                return VisitResult::Continue;
            }
            self.calls += 1;
            match self.answer {
                ScriptAnswer::Continue => VisitResult::Continue,
                ScriptAnswer::Skip => VisitResult::Skip,
                ScriptAnswer::Custom => VisitResult::Custom("CUSTOM".to_owned()),
            }
        }
    }

    fn assert_visitor_routes(script: &str, answer: ScriptAnswer) {
        let html = format!("<p>foo{script}bar</p>");
        for tier_strategy in ROUTES {
            for output_format in FORMATS {
                let visitor = Arc::new(Mutex::new(ScriptVisitor { answer, calls: 0 }));
                let mut options = options(tier_strategy, output_format, false);
                options.visitor = Some(visitor.clone());
                let result = convert(&html, options).expect("script visitor converts");
                let calls = usize::from(script == SCRIPTS[1] && tier_strategy != TierStrategy::Tier1);
                // ~keep Plain ignores the custom script result; forced Tier1 never invokes this callback.
                let expected =
                    if calls == 1 && matches!(answer, ScriptAnswer::Custom) && output_format != OutputFormat::Plain {
                        "fooCUSTOMbar\n"
                    } else {
                        "foobar\n"
                    };
                assert_eq!(
                    result.content.as_deref(),
                    Some(expected),
                    "{tier_strategy:?}/{output_format:?}/{answer:?}: {html}"
                );
                assert_eq!(visitor.lock().expect("visitor state").calls, calls, "{html}");
            }
        }
    }

    #[test]
    fn should_keep_script_visitor_continue_skip_and_custom_behavior_when_joining_words() {
        for script in SCRIPTS {
            for answer in [ScriptAnswer::Continue, ScriptAnswer::Skip, ScriptAnswer::Custom] {
                assert_visitor_routes(script, answer);
            }
        }
    }
}
