#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

#[test]
fn should_preserve_link_destinations_inside_preformatted_blocks() {
    let html = "<pre><a href=\"https://example.com/action\">Open</a></pre>";
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
        let actual = convert(
            html,
            Some(ConversionOptions {
                tier_strategy,
                ..ConversionOptions::default()
            }),
        )
        .expect("preformatted conversion should succeed")
        .content
        .unwrap_or_default();
        assert_eq!(actual, "```\n[Open](https://example.com/action)\n```\n");
    }
}

#[test]
fn should_omit_empty_local_anchors_inside_preformatted_blocks() {
    for (html, expected) in [
        ("<pre><a href=\"#L1\" id=\"L1\"></a>print</pre>", "```\nprint\n```\n"),
        ("<pre><a href=\"#L1\" id=\"L1\"> </a>print</pre>", "```\n print\n```\n"),
    ] {
        let actual = convert(html, None)
            .expect("preformatted conversion should succeed")
            .content
            .unwrap_or_default();
        assert_eq!(actual, expected);
    }
}

#[test]
fn should_keep_empty_preformatted_link_labels_as_text() {
    for (body, expected) in [
        ("<a href=/x></a>", "```\na  b\n```\n"),
        ("<a href=/x> </a>", "```\na   b\n```\n"),
        ("<a href=/x><video src=/media></video></a>", "```\na  b\n```\n"),
        ("<a href=/x><img src=/image></a>", "```\na  b\n```\n"),
    ] {
        let actual = convert(&format!("<pre>a {body} b</pre>"), None)
            .expect("preformatted conversion should succeed")
            .content
            .unwrap_or_default();
        assert_eq!(actual, expected);
    }
}
