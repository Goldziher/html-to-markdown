//! Parser review regressions for preserved inert content and source boundaries. ~keep

#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

fn assert_routes(html: &str, options: &ConversionOptions, expected: &str) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            html,
            ConversionOptions {
                tier_strategy,
                ..options.clone()
            },
        )
        .expect("convert protected HTML");
        assert_eq!(result.content.as_deref(), Some(expected), "{tier_strategy:?}");
    }
}

#[test]
fn should_preserve_uppercase_raw_elements_and_original_entities() {
    for tag in ["NOSCRIPT", "IFRAME", "TEMPLATE"] {
        let html = format!("<P>before</P><{tag}><p>x &amp; y</{tag}><p>visible</p>");
        let lower = tag.to_ascii_lowercase();
        let expected = format!("before\n\n<{lower}><p>x &amp; y</{lower}>\n\nvisible\n");
        assert_routes(
            &html,
            &ConversionOptions {
                preserve_tags: vec![lower],
                ..ConversionOptions::default()
            },
            &expected,
        );
    }
}

#[test]
fn should_preserve_inert_bodies_when_preserving_an_ancestor() {
    assert_routes(
        "<div><noscript><p>x</noscript><template><!-- comment --><i>y</template></div><p>visible</p>",
        &ConversionOptions {
            preserve_tags: vec!["div".to_owned()],
            ..ConversionOptions::default()
        },
        "<div><noscript><p>x</noscript><template><!-- comment --><i>y</template></div>\n\nvisible\n",
    );
}

#[test]
fn should_keep_plain_output_free_of_preservation_markers_and_fallback_html() {
    assert_routes(
        "<p>before</p><noscript><p>x</noscript><iframe><p>fallback</iframe><p>visible</p>",
        &ConversionOptions {
            output_format: OutputFormat::Plain,
            preserve_tags: vec!["noscript".to_owned(), "iframe".to_owned()],
            ..ConversionOptions::default()
        },
        "before\n\nvisible\n",
    );
}

#[test]
fn should_apply_exclusions_before_preserving_an_inert_element() {
    assert_routes(
        "<noscript id='hidden'><p>x</noscript><p>visible</p>",
        &ConversionOptions {
            preserve_tags: vec!["noscript".to_owned()],
            exclude_selectors: vec!["#hidden".to_owned()],
            ..ConversionOptions::default()
        },
        "visible\n",
    );
}

#[test]
fn should_keep_literal_marker_text_separate_from_restored_inert_bodies_after_repair() {
    assert_routes(
        "<div><span><p>h2m_inert_body_0_end h2m_inert_body_&#49;_end</p></span><noscript><p>x</noscript></div><p>visible</p>",
        &ConversionOptions {
            preserve_tags: vec!["noscript".to_owned()],
            ..ConversionOptions::default()
        },
        "h2m_inert_body_0_end h2m_inert_body_1_end\n\n<noscript><p>x</noscript>\n\nvisible\n",
    );
}

#[cfg(feature = "metadata")]
#[test]
fn should_extract_real_metadata_without_interpreting_preserved_raw_body_markup() {
    let html = "<head><title>real &amp; title</title></head><body><noscript><meta name='description' content='fake'><base href='/fake'><img src='fake'><h1>fake</h1></noscript><p>visible</p></body>";
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            html,
            ConversionOptions {
                tier_strategy,
                extract_metadata: true,
                preserve_tags: vec!["noscript".to_owned()],
                ..ConversionOptions::default()
            },
        )
        .expect("preserve inert body with metadata");
        assert_eq!(result.metadata.document.title.as_deref(), Some("real & title"));
        assert_eq!(result.metadata.document.description, None);
        assert!(result.metadata.headers.is_empty());
        assert!(result.metadata.images.is_empty());
        assert_eq!(
            result.content.as_deref(),
            Some(
                "---\ntitle: real & title\n---\n<noscript><meta name='description' content='fake'><base href='/fake'><img src='fake'><h1>fake</h1></noscript>\n\nvisible\n"
            )
        );
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_expose_original_inert_bytes_to_a_preserving_visitor() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct PreserveBlockquote;
    impl HtmlVisitor for PreserveBlockquote {
        fn visit_blockquote(&mut self, _: &NodeContext<'_>, _: &str, _: usize) -> VisitResult {
            VisitResult::PreserveHtml
        }
    }
    assert_routes(
        "<blockquote><noscript><p>x &amp; y</noscript></blockquote><p>visible</p>",
        &ConversionOptions {
            visitor: Some(Arc::new(Mutex::new(PreserveBlockquote))),
            ..ConversionOptions::default()
        },
        "<blockquote><noscript><p>x &amp; y</noscript></blockquote>\n\nvisible\n",
    );
}

#[test]
fn should_strip_an_inert_wrapper_before_applying_preservation() {
    assert_routes(
        "<noscript><p>x</noscript><p>visible</p>",
        &ConversionOptions {
            preserve_tags: vec!["noscript".to_owned()],
            strip_tags: vec!["noscript".to_owned()],
            ..ConversionOptions::default()
        },
        "<p>x\n\nvisible\n",
    );
}

#[test]
fn should_preserve_an_inert_body_inside_select_when_repairing_misnested_markup() {
    assert_routes(
        "<em><p>before</p></em><custom-x><select>A<iframe><p>x</iframe>B<option>C</option></select></custom-x><p>visible</p>",
        &ConversionOptions {
            preserve_tags: vec!["custom-x".to_owned()],
            ..ConversionOptions::default()
        },
        "*before*<custom-x><select>A<iframe><p>x</iframe>B<option>C</option></select></custom-x>\n\nvisible\n",
    );
}

#[test]
fn should_preserve_a_plain_template_body_when_normalization_is_skipped() {
    assert_routes(
        "<p>before</p><template>plain</template><p>visible</p>",
        &ConversionOptions {
            preserve_tags: vec!["template".to_owned()],
            ..ConversionOptions::default()
        },
        "before\n\n<template>plain</template>\n\nvisible\n",
    );
}
