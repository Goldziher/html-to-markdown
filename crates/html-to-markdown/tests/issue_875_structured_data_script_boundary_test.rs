#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

#[cfg(feature = "testkit")]
const ROUTES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2];
#[cfg(not(feature = "testkit"))]
const ROUTES: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];

const JSON_CASES: [&str; 4] = [
    r#"{"@type":"Article","name":"n"}"#,
    r#"{"@type":"Article","name":"<script> &lt; &amp; 雪","quoted":"\\\"{}[]"}"#,
    r#"[{"@type":"Article","items":[{"name":"<script>"},[true,false,null,1.5e2]]}]"#,
    r#"{"@type":"Article","escaped":"\u003cscript\u003e","nested":{"array":[{},[]]}}"#,
];

fn options(tier_strategy: TierStrategy, output_format: OutputFormat, extract_metadata: bool) -> ConversionOptions {
    ConversionOptions {
        tier_strategy,
        output_format,
        extract_metadata,
        ..Default::default()
    }
}

#[test]
fn should_keep_markup_after_complete_json_when_a_structured_script_has_no_end_tag() {
    for json in JSON_CASES {
        for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
            for extract_metadata in [false, true] {
                for tier_strategy in ROUTES {
                    let options = options(tier_strategy, output_format, extract_metadata);
                    let expected = convert("<p>a</p><p>b</p><b>c</b>", options.clone())
                        .expect("visible content converts")
                        .content;
                    let html = format!("<p>a</p><script type='application/ld+json'>{json}<p>b</p><b>c</b>");
                    assert_eq!(
                        convert(&html, options).expect("unclosed script converts").content,
                        expected,
                        "{tier_strategy:?}/{output_format:?}/metadata={extract_metadata}: {json}"
                    );
                }
            }
        }
    }
}

#[test]
fn should_keep_the_body_after_an_unclosed_structured_script_in_the_head() {
    for tier_strategy in ROUTES {
        for extract_metadata in [false, true] {
            let html = format!(
                "<html><head><script type='application/ld+json'>{}</head><body><p>visible</p></body></html>",
                JSON_CASES[0]
            );
            assert_eq!(
                convert(&html, options(tier_strategy, OutputFormat::Markdown, extract_metadata))
                    .expect("head script converts")
                    .content
                    .as_deref(),
                Some("visible\n")
            );
        }
    }
}

#[cfg(feature = "metadata")]
#[test]
fn should_extract_only_original_json_bytes_from_a_repaired_structured_script() {
    for json in JSON_CASES {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            for (prefix, suffix) in [
                ("<p>a</p>", "<p>b</p><b>c</b>"),
                ("<html><head>", "</head><body><p>visible</p></body></html>"),
            ] {
                let html = format!("{prefix}<script type='application/ld+json'> \n{json}{suffix}");
                let result =
                    convert(&html, options(tier_strategy, OutputFormat::Markdown, true)).expect("metadata converts");
                assert_eq!(result.metadata.structured_data.len(), 1, "{html}");
                assert_eq!(result.metadata.structured_data[0].raw_json, json, "{html}");
            }
        }
    }
}

#[test]
fn should_close_invalid_or_absent_json_at_the_script_start_without_removing_the_tail() {
    for body in [
        "",
        "not JSON",
        "{\"name\":",
        "{invalid}",
        "[1,]",
        "{\"name\":\"unterminated",
    ] {
        for tier_strategy in ROUTES {
            let options = options(tier_strategy, OutputFormat::Markdown, false);
            let tail = format!("{body}<p>tail</p>");
            let expected = convert(&tail, options.clone()).expect("fallback text converts").content;
            let html = format!("<script type='application/ld+json'>{tail}");
            assert_eq!(
                convert(&html, options).expect("invalid script converts").content,
                expected,
                "{html}"
            );
        }
    }
}

#[test]
fn should_keep_explicitly_closed_structured_scripts_and_plain_missing_close_behavior() {
    for tier_strategy in ROUTES {
        for extract_metadata in [false, true] {
            let options = options(tier_strategy, OutputFormat::Markdown, extract_metadata);
            let json = JSON_CASES[0];
            let closed = format!("<p>a</p><script type='application/ld+json'>{json}</script><p>b</p>");
            let expected = convert("<p>a</p><p>b</p>", options.clone())
                .expect("visible content converts")
                .content;
            assert_eq!(
                convert(&closed, options).expect("closed script converts").content,
                expected
            );
        }
    }
    let html = "<p>a</p><script>var a = \"<b>x</b>\";<p>b</p>";
    assert_eq!(
        convert(html, options(TierStrategy::Tier2, OutputFormat::Markdown, false))
            .expect("plain script converts")
            .content
            .as_deref(),
        Some("a\n\nvar a = \"**x**\";\n\nb\n")
    );
}

#[cfg(feature = "metadata")]
#[test]
fn should_extract_equal_json_from_closed_and_repaired_scripts_without_literal_markup() {
    for json in [JSON_CASES[0], JSON_CASES[3]] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            let options = options(tier_strategy, OutputFormat::Markdown, true);
            let html = format!("<script type='application/ld+json'>{json}");
            let closed =
                convert(&format!("{html}</script><p>tail</p>"), options.clone()).expect("closed script metadata");
            let repaired = convert(&format!("{html}<p>tail</p>"), options).expect("repaired script metadata");
            assert_eq!(closed.content, repaired.content);
            assert_eq!(closed.metadata.structured_data.len(), 1);
            assert_eq!(repaired.metadata.structured_data.len(), 1);
            assert_eq!(closed.metadata.structured_data[0].raw_json, json);
            assert_eq!(repaired.metadata.structured_data[0].raw_json, json);
        }
    }
}

#[test]
fn should_keep_scripts_after_the_html_bogus_comment_end_of_cdata_shaped_markup() {
    for suffix in ["<p>tail</p>", "<p>tail</p>]]><p>after</p>"] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
            for extract_metadata in [false, true] {
                for tier_strategy in ROUTES {
                    let options = options(tier_strategy, output_format, extract_metadata);
                    let prefix = "<p>before</p><![CDATA[x > ";
                    let expected = convert(&format!("{prefix}{suffix}"), options.clone())
                        .expect("visible markup after a bogus comment")
                        .content;
                    let html = format!("{prefix}<script type='application/ld+json'>{}{suffix}", JSON_CASES[0]);
                    assert_eq!(
                        convert(&html, options).expect("script after bogus comment").content,
                        expected,
                        "{html}"
                    );
                }
            }
        }
    }
}

fn assert_repaired_scripts_match_explicit_closes(input: &str, closed: &str) {
    for tier_strategy in ROUTES {
        for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
            for extract_metadata in [false, true] {
                let options = options(tier_strategy, output_format, extract_metadata);
                let repaired = convert(input, options.clone()).expect("repaired page");
                let expected = convert(closed, options).expect("closed page");
                assert_eq!(
                    repaired.content, expected.content,
                    "{tier_strategy:?}/{output_format:?}: {input}"
                );
                #[cfg(feature = "metadata")]
                assert_eq!(
                    repaired
                        .metadata
                        .structured_data
                        .iter()
                        .map(|item| &item.raw_json)
                        .collect::<Vec<_>>(),
                    expected
                        .metadata
                        .structured_data
                        .iter()
                        .map(|item| &item.raw_json)
                        .collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
fn should_not_let_synthetic_close_capture_an_earlier_plain_script() {
    assert_repaired_scripts_match_explicit_closes(
        r#"<script>x<p>a</p><script type="application/ld+json">{"a":1}<p>b</p>"#,
        r#"<script></script>x<p>a</p><script type="application/ld+json">{"a":1}</script><p>b</p>"#,
    );
}

#[test]
fn should_recover_a_structured_script_before_a_later_closed_plain_script() {
    assert_repaired_scripts_match_explicit_closes(
        r#"<p>a</p><script type="application/ld+json">{"a":1}<script>x</script><p>b</p>"#,
        r#"<p>a</p><script type="application/ld+json">{"a":1}</script><script>x</script><p>b</p>"#,
    );
}

#[test]
fn should_recover_two_structured_scripts_with_no_end_tags() {
    assert_repaired_scripts_match_explicit_closes(
        r#"<script type="application/ld+json">{"a":1}<script type="application/ld+json">{"b":2}<p>tail</p>"#,
        r#"<script type="application/ld+json">{"a":1}</script><script type="application/ld+json">{"b":2}</script><p>tail</p>"#,
    );
}

#[test]
fn should_preserve_original_json_bytes_in_a_repaired_preserved_script() {
    for json in [r#"{"a":"b & c"}"#, r#"{"a":"&lt; &amp; 雪"}"#] {
        for format in [OutputFormat::Markdown, OutputFormat::Djot] {
            let options = ConversionOptions {
                preserve_tags: vec!["script".into()],
                extract_metadata: true,
                output_format: format,
                ..Default::default()
            };
            let input = format!("<script type='application/ld+json'>{json}");
            let repaired = convert(&format!("{input}<p>tail</p>"), options.clone()).expect("preserved repaired");
            let closed = convert(&format!("{input}</script><p>tail</p>"), options).expect("preserved closed");
            assert_eq!(repaired.content, closed.content, "{json}");
            #[cfg(feature = "metadata")]
            {
                // ~keep Preserved body scripts bypass collection; head scripts have separate non-empty coverage below.
                assert_eq!(
                    closed.metadata.structured_data.len(),
                    0,
                    "closed preserved script: {json}"
                );
                assert_eq!(
                    repaired.metadata.structured_data.len(),
                    0,
                    "repaired preserved script: {json}"
                );
            }
            assert_eq!(
                repaired.content.as_deref(),
                Some(format!("<script type=\"application/ld+json\">{json}</script>\n\ntail\n").as_str())
            );
        }
    }
}

#[test]
fn should_support_valid_scalar_json_prefixes_with_the_same_tail_as_explicit_close() {
    for json in [r#""x""#, "null", "true", "false", "42", "1.5e2"] {
        for tier_strategy in ROUTES {
            let options = options(tier_strategy, OutputFormat::Markdown, true);
            let input = format!("<script type='application/ld+json'>{json} y<p>tail</p>");
            let repaired = convert(&input, options.clone()).expect("scalar repaired");
            let closed = convert(
                &format!("<script type='application/ld+json'>{json}</script> y<p>tail</p>"),
                options,
            )
            .expect("scalar closed");
            assert_eq!(repaired.content, closed.content, "{json}");
            #[cfg(feature = "metadata")]
            assert_eq!(
                repaired
                    .metadata
                    .structured_data
                    .iter()
                    .map(|item| &item.raw_json)
                    .collect::<Vec<_>>(),
                closed
                    .metadata
                    .structured_data
                    .iter()
                    .map(|item| &item.raw_json)
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_original_head_metadata_bytes_when_preserving_repaired_scripts() {
    for json in [r#"{"a":"b & c"}"#, r#"{"a":"&lt; &amp; 雪"}"#] {
        let options = ConversionOptions {
            preserve_tags: vec!["script".into()],
            extract_metadata: true,
            ..Default::default()
        };
        let input = format!("<head><script type='application/ld+json'>{json}");
        let repaired = convert(&format!("{input}</head><p>tail</p>"), options.clone()).expect("repaired head metadata");
        let closed = convert(&format!("{input}</script></head><p>tail</p>"), options).expect("closed head metadata");
        assert_eq!(closed.metadata.structured_data.len(), 1, "{json}");
        assert_eq!(repaired.metadata.structured_data.len(), 1, "{json}");
        assert_eq!(
            repaired.metadata.structured_data[0].raw_json, json,
            "original repaired head bytes: {json}"
        );
        assert_eq!(repaired.content, closed.content, "{json}");
    }
}

#[test]
fn should_keep_literal_script_openers_entities_and_unicode_in_repaired_preserved_bodies() {
    let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for tier_strategy in ROUTES {
            let options = ConversionOptions {
                preserve_tags: vec!["script".into()],
                ..options(tier_strategy, output_format, true)
            };
            let converted = convert(
                &format!("<script type='application/ld+json'>{json}<p>tail</p>"),
                options,
            )
            .expect("preserved repaired script");
            assert_eq!(
                converted.content.as_deref(),
                Some(format!("<script type=\"application/ld+json\">{json}</script>\n\ntail\n").as_str())
            );
        }
    }
}

#[test]
fn should_keep_repaired_script_bytes_across_document_head_and_body_prefixes() {
    for (prefix, suffix, in_head) in [
        ("", "", false),
        ("<head>", "</head>", true),
        ("<html><head>", "</head></html>", true),
        ("<html>", "</html>", false),
        ("<!doctype html><html>", "</html>", false),
        ("<title>prefix</title>", "", false),
        ("<body>", "</body>", false),
        ("<body><p>prefix</p><head>", "</head></body>", true),
    ] {
        for json in [
            r#"{"a":"b & c"}"#,
            r#"{"a":"&lt; &amp; 雪"}"#,
            r#"{"a":"<script> &lt; &amp; 雪"}"#,
        ] {
            for tier_strategy in ROUTES {
                let converted = convert(
                    &format!("{prefix}<script type='application/ld+json'>{json}{suffix}<p>tail</p>"),
                    ConversionOptions {
                        preserve_tags: vec!["script".into()],
                        ..options(tier_strategy, OutputFormat::Markdown, true)
                    },
                )
                .expect("repaired document prefix");
                let content = converted.content.as_deref().expect("content");
                assert!(content.ends_with("tail\n"), "{prefix}: {content}");
                if !in_head {
                    assert!(
                        content.contains(&format!("<script type=\"application/ld+json\">{json}</script>")),
                        "{prefix}: {content}"
                    );
                }
                #[cfg(feature = "metadata")]
                {
                    assert_eq!(
                        converted.metadata.structured_data.len(),
                        usize::from(in_head),
                        "{prefix}"
                    );
                    if in_head {
                        assert_eq!(converted.metadata.structured_data[0].raw_json, json, "{prefix}");
                    }
                }
            }
        }
    }
}
