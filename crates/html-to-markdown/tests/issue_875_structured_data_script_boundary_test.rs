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
