#![allow(missing_docs)]

use base64::Engine;

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

#[test]
fn should_keep_paragraph_markers_formed_by_processing_instruction_removal() {
    let json = r#"{"a":"<b> & c"}"#;
    for prefix in ["h2m_inert_body_", "h2m_script_body_"] {
        for tier_strategy in ROUTES {
            let converted = convert(
                &format!("<p>{prefix}<?x?>0_end</p><script type='application/ld+json'>{json}<p>t</p>"),
                ConversionOptions {
                    preserve_tags: vec!["script".into()],
                    ..options(tier_strategy, OutputFormat::Markdown, true)
                },
            )
            .expect("marker-shaped paragraph");
            assert_eq!(
                converted.content.as_deref(),
                Some(format!("{prefix}0_end\n\n<script type=\"application/ld+json\">{json}</script>\n\nt\n").as_str())
            );
            #[cfg(feature = "metadata")]
            assert_eq!(converted.metadata.structured_data.len(), 0);
        }
    }
}

#[test]
fn should_not_restore_a_marker_forged_in_a_closed_script_body() {
    let json = r#"{"a":"<b> & c"}"#;
    for prefix in ["h2m_inert_body_", "h2m_script_body_"] {
        let converted = convert(
            &format!("<script type='application/ld+json'>{prefix}<?x?>0_end</script><script type='application/ld+json'>{json}<p>t</p>"),
            ConversionOptions { preserve_tags: vec!["script".into()], ..Default::default() },
        )
        .expect("closed marker-shaped script");
        assert_eq!(converted.content.as_deref(), Some(format!("<script type=\"application/ld+json\">{prefix}0_end</script>\n\n<script type=\"application/ld+json\">{json}</script>\n\nt\n").as_str()));
    }
}

#[test]
fn should_keep_authored_script_attributes_while_removing_only_transport_ownership() {
    let json = r#"{"a":"x"}"#;
    let attributes = "DATA-H2M-SCRIPT-0='authored' data-h2m-script-1='second'";
    let options = ConversionOptions {
        preserve_tags: vec!["script".into()],
        ..Default::default()
    };
    let repaired = convert(
        &format!("<script {attributes} type='application/ld+json'>{json}<p>tail</p>"),
        options.clone(),
    )
    .expect("authored attributes repaired");
    let closed = convert(
        &format!("<script {attributes} type='application/ld+json'>{json}</script><p>tail</p>"),
        options,
    )
    .expect("authored attributes closed");
    for converted in [repaired, closed] {
        let content = converted.content.as_deref().expect("content");
        assert!(content.ends_with("tail\n"));
        let dom = tl::parse(content, tl::ParserOptions::default()).expect("preserved HTML");
        let handle = dom.query_selector("script").expect("selector").next().expect("script");
        let tag = handle.get(dom.parser()).and_then(tl::Node::as_tag).expect("script tag");
        assert_eq!(tag.attributes().len(), 3);
        for (key, value) in [
            ("data-h2m-script-0", "authored"),
            ("data-h2m-script-1", "second"),
            ("type", "application/ld+json"),
        ] {
            assert_eq!(
                tag.attributes()
                    .get(key)
                    .flatten()
                    .expect("authored attribute")
                    .as_utf8_str(),
                value
            );
        }
        assert_eq!(tag.inner_text(dom.parser()), json);
    }
}

#[test]
fn should_activate_repaired_script_transport_only_for_script_preservation() {
    let json = r#"{"a":"&lt; &amp; 雪"}"#;
    let input = format!("<div><script type='application/ld+json'>{json}<p>tail</p></div>");
    let script = convert(
        &input,
        ConversionOptions {
            preserve_tags: vec!["script".into()],
            ..Default::default()
        },
    )
    .expect("script preservation");
    assert_eq!(
        script.content.as_deref(),
        Some(format!("<script type=\"application/ld+json\">{json}</script>\n\ntail\n").as_str())
    );
    let ancestor = convert(
        &input,
        ConversionOptions {
            preserve_tags: vec!["div".into()],
            ..Default::default()
        },
    )
    .expect("ancestor preservation");
    // ~keep Ancestor encoding is an inherited queued limitation; this fix scopes transport to script preservation.
    assert_eq!(
        ancestor.content.as_deref(),
        Some("<div><script type=\"application/ld+json\">{\"a\":\"&amp;lt; &amp;amp; 雪\"}</script><p>tail</p></div>\n")
    );
}

#[test]
fn should_keep_repaired_script_ownership_when_a_non_void_opener_has_a_slash() {
    let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
    for slash in ["/", " /", "/  "] {
        for preserve_tags in [vec!["script".into()], vec!["script".into(), "div".into()]] {
            let converted = convert(
                &format!("<div><script type='application/ld+json'{slash}>{json}<p>tail</p></div>"),
                ConversionOptions {
                    preserve_tags: preserve_tags.clone(),
                    ..Default::default()
                },
            )
            .expect("non-void script opener");
            let expected = if preserve_tags.len() == 1 {
                format!("<script type=\"application/ld+json\">{json}</script>\n\ntail\n")
            } else {
                format!("<div><script type=\"application/ld+json\">{json}</script><p>tail</p></div>\n")
            };
            assert_eq!(converted.content.as_deref(), Some(expected.as_str()), "{slash}");
        }
    }
}

#[test]
fn should_keep_quoted_and_unquoted_slash_attribute_values_in_repaired_scripts() {
    let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
    for attributes in [
        "src='/path/>'",
        "src=/path/",
        "src=/path/ /",
        "src=relative/",
        "src=relative/ /",
        "data-note='/' /",
    ] {
        for preserve_tags in [vec!["script".into()], vec!["script".into(), "div".into()]] {
            let converted = convert(
                &format!("<div><script type='application/ld+json' {attributes}>{json}<p>tail</p></div>"),
                ConversionOptions {
                    preserve_tags,
                    ..Default::default()
                },
            )
            .expect("slash-valued attributes");
            let content = converted.content.as_deref().expect("content");
            assert!(
                content.ends_with("tail\n") || content.ends_with("<p>tail</p></div>\n"),
                "{attributes}: {content}"
            );
            // ~keep Raw JSON may itself contain script openers, so parse only the preserved opening tag.
            let script_start = content.find("<script ").expect("preserved script");
            let raw_start = content.find(json).expect("original JSON bytes");
            let opening = format!("{}</script>", &content[script_start..raw_start]);
            let dom = tl::parse(&opening, tl::ParserOptions::default()).expect("preserved opener");
            let tag = dom
                .query_selector("script")
                .expect("selector")
                .next()
                .expect("script")
                .get(dom.parser())
                .and_then(tl::Node::as_tag)
                .expect("script tag");
            assert_eq!(tag.attributes().len(), 2, "{attributes}");
            let (key, expected) = if attributes.starts_with("data-note") {
                ("data-note", "/")
            } else if attributes.contains("'/path/>'") {
                ("src", "/path/>")
            } else if attributes.starts_with("src=/path/") {
                ("src", "/path/")
            } else {
                ("src", "relative/")
            };
            assert_eq!(
                tag.attributes()
                    .get(key)
                    .flatten()
                    .expect("authored attribute")
                    .as_utf8_str(),
                expected,
                "{attributes}"
            );
        }
    }
}

#[test]
fn should_keep_authored_owner_attributes_from_claiming_another_scripts_body() {
    let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
    for attribute in ["data-h2m-script-0", "DATA-H2M-SCRIPT-0"] {
        let html = format!(
            "<script type='application/ld+json' {attribute}='h2m_script_body_0_end'>h2m_script_body_<?x?>0_end</script><script type='application/ld+json'>{json}<p>tail</p>"
        );
        let converted = convert(
            &html,
            ConversionOptions {
                preserve_tags: vec!["script".into()],
                ..Default::default()
            },
        )
        .expect("authored ownership lookalike");
        let content = converted.content.as_deref().expect("preserved content");
        let end = content.find("</script>").expect("authored script close") + "</script>".len();
        let dom = tl::parse(&content[..end], tl::ParserOptions::default()).expect("closed authored script");
        let tag = dom
            .query_selector("script")
            .expect("selector")
            .next()
            .expect("authored script")
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .expect("script tag");
        assert_eq!(tag.attributes().len(), 2);
        assert_eq!(
            tag.attributes()
                .get("data-h2m-script-0")
                .flatten()
                .expect("authored owner attribute")
                .as_utf8_str(),
            "h2m_script_body_0_end"
        );
        assert_eq!(
            tag.attributes()
                .get("type")
                .flatten()
                .expect("authored type")
                .as_utf8_str(),
            "application/ld+json"
        );
        assert_eq!(tag.inner_text(dom.parser()), "h2m_script_body_0_end");
        assert_eq!(
            &content[end..],
            format!("\n\n<script type=\"application/ld+json\">{json}</script>\n\ntail\n")
        );
    }
}

#[test]
fn should_keep_repaired_script_bytes_after_a_slash_in_a_boolean_attribute_name() {
    let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
    for preserve_tags in [vec!["script".into()], vec!["script".into(), "div".into()]] {
        let converted = convert(
            &format!("<div><script type='application/ld+json' data-x/foo />{json}<p>tail</p></div>"),
            ConversionOptions {
                preserve_tags,
                ..Default::default()
            },
        )
        .expect("slash-shaped boolean attribute");
        let content = converted.content.as_deref().expect("content");
        let start = content.find("<script ").expect("preserved script");
        let body = content.find(json).expect("absolute original JSON bytes");
        let opening = format!("{}</script>", &content[start..body]);
        let dom = tl::parse(&opening, tl::ParserOptions::default()).expect("preserved attributes");
        let tag = dom
            .query_selector("script")
            .expect("selector")
            .next()
            .expect("script")
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .expect("script tag");
        assert_eq!(tag.attributes().len(), 2);
        assert_eq!(tag.attributes().get("data-x/foo"), Some(None));
        assert!(content.ends_with("tail\n") || content.ends_with("<p>tail</p></div>\n"));
    }
}

#[test]
fn should_keep_original_script_bytes_when_standalone_slashes_precede_an_attribute() {
    let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
    for attributes in [
        "/ data-x=foo /",
        "/data-x=foo /",
        "/ data-x='foo/' /",
        "/ data-x=foo/ /",
    ] {
        for preserve_tags in [vec!["script".into()], vec!["script".into(), "div".into()]] {
            let converted = convert(
                &format!("<div><script type='application/ld+json' {attributes}>{json}<p>tail</p></div>"),
                ConversionOptions {
                    preserve_tags,
                    ..Default::default()
                },
            )
            .expect("standalone slash tokens");
            let content = converted.content.as_deref().expect("content");
            let start = content.find("<script ").expect("preserved script");
            let body = content.find(json).expect("original JSON bytes");
            let opening = format!("{}</script>", &content[start..body]);
            let dom = tl::parse(&opening, tl::ParserOptions::default()).expect("script attributes");
            let tag = dom
                .query_selector("script")
                .expect("selector")
                .next()
                .expect("script")
                .get(dom.parser())
                .and_then(tl::Node::as_tag)
                .expect("script tag");
            assert_eq!(tag.attributes().len(), 2);
            let expected = if attributes.contains("foo/") { "foo/" } else { "foo" };
            assert_eq!(
                tag.attributes()
                    .get("data-x")
                    .flatten()
                    .expect("authored data-x")
                    .as_utf8_str(),
                expected
            );
            assert!(content.ends_with("tail\n") || content.ends_with("<p>tail</p></div>\n"));
        }
    }
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_original_head_json_bytes_with_repaired_slash_shaped_attributes() {
    let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
    for attributes in [
        "/",
        "src='/path/>' /",
        "src=/path/ /",
        "data-x/foo /",
        "/ data-x=foo/ /",
        "/data-x='foo/' /",
    ] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            let converted = convert(
                &format!("<head><script type='application/ld+json' {attributes}>{json}</head><p>tail</p>"),
                ConversionOptions {
                    preserve_tags: vec!["script".into()],
                    ..options(tier_strategy, OutputFormat::Markdown, true)
                },
            )
            .expect("head script ownership");
            assert_eq!(converted.metadata.structured_data.len(), 1, "{attributes}");
            assert_eq!(converted.metadata.structured_data[0].raw_json, json, "{attributes}");
            assert_eq!(converted.content.as_deref(), Some("tail\n"));
        }
    }
}

#[test]
fn should_restore_owned_script_bytes_even_when_foreign_close_case_confuses_the_literal_dom() {
    let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
    for prefix in ["<SVG></svg>", "<svg></SVG>", "<math></MATH>", "<MATH></math>"] {
        let converted = convert(
            &format!("{prefix}<script type='application/ld+json'>{json}<p>tail</p>"),
            ConversionOptions {
                preserve_tags: vec!["script".into()],
                ..Default::default()
            },
        )
        .expect("mixed-case foreign boundary");
        let content = converted.content.as_deref().expect("content");
        let payload = if prefix.to_ascii_lowercase().starts_with("<svg") {
            let encoded = content
                .split_once("data:image/svg+xml;base64,")
                .expect("SVG image")
                .1
                .split_once(')')
                .expect("image end")
                .0;
            String::from_utf8(
                base64::engine::general_purpose::STANDARD
                    .decode(encoded)
                    .expect("SVG bytes"),
            )
            .expect("UTF-8 SVG")
        } else {
            content.to_owned()
        };
        let expected = format!("<script type=\"application/ld+json\">{json}</script><p>tail</p>");
        assert!(payload.contains(&expected), "{prefix}: {payload}");
        assert!(!payload.contains("data-h2m-script-"), "{prefix}: {payload}");
        assert!(!payload.contains("h2m_script_body_"), "{prefix}: {payload}");
    }
}

#[test]
fn should_keep_json_when_a_slash_separates_the_script_name_from_its_type() {
    let json = r#"{"a":"<b> & c"}"#;
    for opener in [
        "<script/type=\"application/ld+json\">",
        "<SCRIPT//type='application/ld+json'>",
    ] {
        for tier_strategy in ROUTES {
            for extract_metadata in [false, true] {
                let converted = convert(
                    &format!("<p>a</p>{opener}{json}<p>t</p>"),
                    ConversionOptions {
                        tier_strategy,
                        extract_metadata,
                        preserve_tags: vec!["script".into()],
                        ..Default::default()
                    },
                )
                .expect("slash-separated type");
                assert_eq!(
                    converted.content.as_deref(),
                    Some(format!("a\n\n<script type=\"application/ld+json\">{json}</script>\n\nt\n").as_str()),
                    "{tier_strategy:?}/{extract_metadata}/{opener}"
                );
            }
        }
    }
}

#[test]
fn should_reserve_owner_names_and_markers_formed_by_bogus_comment_removal() {
    let json = r#"{"a":"<b> & c"}"#;
    let authored = r#"<<?x?>script type="application/ld+json" data-h2m-script-<?y?>0="h2m_script_body_<?z?>0_end">h2m_script_body_<?w?>0_end</script>"#;
    for tier_strategy in ROUTES {
        for extract_metadata in [false, true] {
            let content = convert(
                &format!("{authored}<script type='application/ld+json'>{json}<p>t</p>"),
                ConversionOptions {
                    tier_strategy,
                    extract_metadata,
                    preserve_tags: vec!["script".into()],
                    ..Default::default()
                },
            )
            .expect("sanitized authored owner")
            .content
            .expect("content");
            let end = content.find("</script>").expect("first script close") + "</script>".len();
            let dom = tl::parse(&content[..end], tl::ParserOptions::default()).expect("authored DOM");
            let tag = dom
                .query_selector("script")
                .expect("selector")
                .next()
                .expect("first script")
                .get(dom.parser())
                .and_then(tl::Node::as_tag)
                .expect("tag");
            assert_eq!(
                tag.attributes()
                    .get("data-h2m-script-0")
                    .flatten()
                    .expect("authored owner")
                    .as_utf8_str(),
                "h2m_script_body_0_end"
            );
            assert_eq!(
                tag.inner_text(dom.parser()),
                "h2m_script_body_0_end",
                "{tier_strategy:?}/{extract_metadata}"
            );
            assert_eq!(
                &content[end..],
                format!("\n\n<script type=\"application/ld+json\">{json}</script>\n\nt\n")
            );
        }
    }
}

#[test]
fn should_keep_repaired_script_attribute_order_stable_across_conversions() {
    let json = r#"{"a":"<b> & c"}"#;
    let html = format!("<p>a</p><script type='application/ld+json' nonce='n'>{json}<p>t</p>");
    for tier_strategy in ROUTES {
        let mut observed = std::collections::BTreeSet::new();
        for _ in 0..60 {
            observed.insert(
                convert(
                    &html,
                    ConversionOptions {
                        tier_strategy,
                        preserve_tags: vec!["script".into()],
                        ..Default::default()
                    },
                )
                .expect("stable script")
                .content
                .expect("content"),
            );
        }
        assert_eq!(observed.len(), 1, "{tier_strategy:?}: {observed:?}");
        assert_eq!(
            observed.into_iter().next().expect("one output"),
            format!("a\n\n<script nonce=\"n\" type=\"application/ld+json\">{json}</script>\n\nt\n")
        );
    }
}
