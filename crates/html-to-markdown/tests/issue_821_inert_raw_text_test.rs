//! Regression coverage for inert and raw-text boundaries in issue #821. ~keep

#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn assert_routes(html: &str, expected: &str, extract_metadata: bool) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            html,
            ConversionOptions {
                tier_strategy,
                extract_metadata,
                ..ConversionOptions::default()
            },
        )
        .expect("convert inert or raw-text HTML");
        assert_eq!(result.content.as_deref(), Some(expected), "{tier_strategy:?}: {html}");
    }
}

#[test]
fn should_not_open_raw_or_inert_elements_inside_bogus_end_tag_comments() {
    for tag in ["iframe", "noscript", "template", "title", "xmp"] {
        let html = format!("</ <{tag}>><b>Shown</b><p>tail</p>");
        for extract_metadata in [false, true] {
            assert_routes(&html, "\\>**Shown**\n\ntail\n", extract_metadata);
        }
    }
}

#[test]
fn should_not_end_a_comment_using_dashes_from_its_opener() {
    for opener in ["<!--!>", "<!---!>"] {
        let html = format!(
            "<p>a</p>{opener}<plaintext><b>in</b></plaintext>--><p>mid</p><xmp><b>lit</b></xmp><noscript><p>hid</noscript><p>tail</p>"
        );
        for extract_metadata in [false, true] {
            assert_routes(&html, "a\n\nmid\n\n<b>lit</b>\n\ntail\n", extract_metadata);
        }
    }
}

#[test]
fn should_keep_visible_siblings_after_noscript_with_an_unclosed_child() {
    assert_routes(
        "<p>before</p><noscript><p></noscript><p>visible</p>",
        "before\n\nvisible\n",
        false,
    );
}

#[test]
fn should_keep_visible_siblings_after_iframe_with_an_unclosed_child() {
    assert_routes(
        "<p>before</p><iframe><p></iframe><p>visible</p>",
        "before\n\nvisible\n",
        false,
    );
}

#[test]
fn should_keep_frame_address_and_following_visible_siblings() {
    assert_routes(
        "<p>before</p><iframe src='/f.html'><p></iframe><p>visible</p>",
        "before\n\n[iframe](/f.html)\n\nvisible\n",
        false,
    );
}

#[test]
fn should_drop_template_markup_without_swallowing_visible_siblings() {
    assert_routes(
        "<p>before</p><template><p></template><p>visible</p>",
        "before\n\nvisible\n",
        false,
    );
}

#[test]
fn should_keep_title_rcdata_and_following_body() {
    assert_routes(
        "<html><head><title>a <p> b &amp; c</title></head><body><p>visible</p></body></html>",
        "---\ntitle: a <p> b & c\n---\n\nvisible\n",
        true,
    );
}

#[test]
fn should_drop_title_when_metadata_is_disabled_without_losing_body() {
    assert_routes(
        "<html><head><title>a <p> b</title></head><body><p>visible</p></body></html>",
        "visible\n",
        false,
    );
}

#[test]
fn should_preserve_xmp_literal_markup_and_entities() {
    assert_routes(
        "<p>before</p><xmp><p> &amp;</xmp><p>visible</p>",
        "before\n\n<p> &amp;\n\nvisible\n",
        false,
    );
}

#[test]
fn should_ignore_target_tag_shapes_in_comments_and_quoted_attributes() {
    assert_routes(
        "<p data-note='<template><p>'>before</p><!-- <iframe><p> --><p>visible</p>",
        "before\n\nvisible\n",
        false,
    );
}

#[test]
fn should_balance_nested_templates_and_ignore_quoted_closes() {
    assert_routes(
        "<p>before</p><template><div data-x='</template>'><template><p></template></div></template><p>visible</p>",
        "before\n\nvisible\n",
        false,
    );
}

#[test]
fn should_not_close_template_on_comment_or_raw_text_tag_shapes() {
    assert_routes(
        "<p>before</p><template><!-- </template> --><script>'</template>'</script><p></template><p>visible</p>",
        "before\n\nvisible\n",
        false,
    );
}

#[test]
fn should_handle_unquoted_attribute_equals_without_opening_a_quote() {
    assert_routes(
        "<p>before</p><template><div data=x='><p></template><p>visible</p>",
        "before\n\nvisible\n",
        false,
    );
}

#[test]
fn should_keep_visible_siblings_after_a_foreign_template_in_an_inert_template() {
    assert_routes(
        "<template><svg><template></svg></template><p>visible</p>",
        "visible\n",
        false,
    );
}

#[test]
fn should_keep_visible_siblings_after_uppercase_inert_elements() {
    for tag in ["NOSCRIPT", "IFRAME", "TEMPLATE"] {
        let html = format!("<P>before</P><{tag}><p></{tag}><p>visible</p>");
        assert_routes(&html, "before\n\nvisible\n", false);
    }
}

#[test]
fn should_preserve_noscript_raw_body_and_visible_siblings() {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            "<p>before</p><noscript><p>x</noscript><p>visible</p>",
            ConversionOptions {
                tier_strategy,
                preserve_tags: vec!["noscript".to_owned()],
                ..ConversionOptions::default()
            },
        )
        .expect("preserve inert raw HTML");
        assert_eq!(
            result.content.as_deref(),
            Some("before\n\n<noscript><p>x</noscript>\n\nvisible\n"),
            "{tier_strategy:?}"
        );
    }
}
