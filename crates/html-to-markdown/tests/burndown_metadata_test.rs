#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn routes() -> Vec<TierStrategy> {
    vec![
        TierStrategy::Auto,
        TierStrategy::Tier2,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
    ]
}

fn content(html: &str, tier_strategy: TierStrategy, extract_metadata: bool) -> String {
    convert(
        html,
        ConversionOptions {
            tier_strategy,
            extract_metadata,
            ..Default::default()
        },
    )
    .expect("conversion succeeds")
    .content
    .expect("content is requested")
}

#[test]
fn should_extract_metadata_without_an_explicit_head_on_every_route() {
    for (html, field) in [
        (
            r#"<meta name="description" content="D"><p>one</p>"#,
            "meta-description: D",
        ),
        (
            r#"<link rel="canonical" href="https://e.org/"><p>one</p>"#,
            "canonical: https://e.org/",
        ),
        (
            r#"<head></head><meta name="description" content="D"><p>one</p>"#,
            "meta-description: D",
        ),
        (
            r#"<!doctype html><meta name="description" content="D"><p>one</p>"#,
            "meta-description: D",
        ),
        (
            r#"<html><meta name="description" content="D"><body><p>one</p></body></html>"#,
            "meta-description: D",
        ),
        (
            r#"<!-- c --><meta name="description" content="D"><p>one</p>"#,
            "meta-description: D",
        ),
    ] {
        for tier in routes() {
            assert_eq!(
                content(html, tier, true),
                format!("---\n{field}\n---\n\none\n"),
                "{tier:?}: {html}"
            );
            assert_eq!(content(html, tier, false), "one\n", "{tier:?}: {html}");
        }
    }
}

#[cfg(feature = "testkit")]
#[test]
fn should_extract_implicit_head_metadata_in_the_direct_fast_converter() {
    for html in [
        r#"<meta name="description" content="D"><p>one</p>"#,
        r#"<head></head><meta name="description" content="D"><p>one</p>"#,
        r#"<!-- <head> --><meta name="description" content="D"><p>one</p>"#,
    ] {
        let options = ConversionOptions::default();
        let result = html_to_markdown_rs::tier1::run(html, &Default::default(), &options)
            .expect("the fast converter supports metadata and paragraphs");
        assert_eq!(result, "---\nmeta-description: D\n---\n\none\n", "{html}");
    }
}

#[test]
fn should_ignore_commented_head_and_body_tags_when_classifying_titles() {
    for html in [
        "<!-- <head> --><title>T</title><p>one</p>",
        "<head></head><!-- <body> --><title>T</title><p>one</p>",
        "<!-- <header> --><title>T</title><p>one</p>",
    ] {
        for tier in routes() {
            assert_eq!(
                content(html, tier, true),
                "---\ntitle: T\n---\n\none\n",
                "{tier:?}: {html}"
            );
            assert_eq!(content(html, tier, false), "one\n", "{tier:?}: {html}");
        }
    }
}

#[test]
fn should_not_collect_metadata_after_inert_elements_that_start_the_body() {
    for html in [
        r#"<template><p>inert</p></template><meta name="description" content="D"><p>body</p>"#,
        r#"<head></head><noscript></noscript><meta name="description" content="D"><p>body</p>"#,
    ] {
        for tier in routes() {
            let actual = content(html, tier, true);
            assert_eq!(actual, "body\n", "{tier:?}: {html}");
        }
        #[cfg(feature = "testkit")]
        match html_to_markdown_rs::tier1::run(html, &Default::default(), &ConversionOptions::default()) {
            Ok(output) => assert_eq!(output, "body\n", "{html}"),
            Err(html_to_markdown_rs::tier1::BailReason::Classifier) => {}
            Err(reason) => panic!("unexpected fast-route failure for {html}: {reason:?}"),
        }
    }
}

#[test]
fn should_keep_office_head_metadata_only_before_real_body_content() {
    let html = "<html><head><o:p></o:p><meta name='Generator' content='Word'><title>Document</title></head><body><p>Visible</p></body></html>";
    let after_body = "<head><o:p></o:p><p>Visible</p><meta name='Generator' content='Word'></head>";
    for tier in routes() {
        assert_eq!(
            content(html, tier, true),
            "---\nmeta-Generator: Word\ntitle: Document\n---\n\nVisible\n",
            "{tier:?}"
        );
        let body_metadata = content(after_body, tier, true);
        assert!(!body_metadata.starts_with("---\n"), "{tier:?}: {body_metadata:?}");
        assert!(!body_metadata.contains("meta-Generator"), "{tier:?}: {body_metadata:?}");
    }
}

#[test]
fn should_keep_an_image_that_implicitly_ends_the_head() {
    for html in [
        r#"<head><img src="p.gif" alt="A1"></head><p>End</p>"#,
        r#"<html><head><img src="p.gif" alt="A1"></head><body><p>End</p></body></html>"#,
    ] {
        for tier in routes() {
            for metadata in [false, true] {
                assert_eq!(
                    content(html, tier, metadata),
                    "![A1](p.gif)\n\nEnd\n",
                    "{tier:?}: {html}"
                );
            }
        }
    }
}

#[test]
fn should_separate_front_matter_from_bare_body_text_on_every_route() {
    for html in [
        "<head><title>T</title></head>tail",
        "<title>T</title>tail",
        "<html><head><title>T</title></head><body>tail</body></html>",
    ] {
        for tier in routes() {
            assert_eq!(
                content(html, tier, true),
                "---\ntitle: T\n---\n\ntail\n",
                "{tier:?}: {html}"
            );
            assert_eq!(content(html, tier, false), "tail\n", "{tier:?}: {html}");
        }
    }
}

#[test]
fn should_preserve_table_section_measurement_depth() {
    let sectioned = "<table><thead><tr><th>HEADER</th></tr></thead><tbody><tr><td>BODY</td></tr></tbody></table>";
    let direct = "<table><tr><th>HEADER</th></tr><tr><td>BODY</td></tr></table>";
    for max_depth in 3..=6 {
        let options = ConversionOptions {
            max_depth: Some(max_depth),
            tier_strategy: TierStrategy::Tier2,
            ..Default::default()
        };
        let actual = convert(sectioned, options.clone())
            .expect("conversion succeeds")
            .content;
        let expected = convert(direct, options).expect("conversion succeeds").content;
        assert_eq!(actual, expected, "max_depth={max_depth}");
        if max_depth == 6 {
            assert!(actual.as_deref().is_some_and(|text| text.contains("BODY")));
        }
    }
}
