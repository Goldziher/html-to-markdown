#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

fn content(html: &str) -> String {
    let options = ConversionOptions {
        compact_tables: true,
        ..ConversionOptions::default()
    };
    let paths = [
        options.clone(),
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..options.clone()
        },
        ConversionOptions {
            extract_metadata: false,
            highlight_style: HighlightStyle::None,
            ..options
        },
    ];
    let mut outputs = paths.into_iter().map(|options| {
        convert(html, Some(options))
            .expect("conversion succeeds")
            .content
            .unwrap_or_default()
    });
    let output = outputs.next().expect("default conversion exists");
    for candidate in outputs {
        assert_eq!(candidate, output, "conversion paths agree for {html}");
    }
    output
}

#[test]
fn should_preserve_code_link_ranges_when_a_table_break_trims_whitespace() {
    assert_eq!(
        content("<table><tr><th>h</th></tr><tr><td><code>x<a href=\"/x\">a  </a><br></code></td></tr></table>"),
        "| h |\n| --- |\n| `x`[`a`](/x) |\n"
    );
}

#[test]
fn should_preserve_text_after_a_link_and_table_break() {
    assert_eq!(
        content("<table><tr><th>h</th></tr><tr><td><code><a href=\"/x\">a  </a><br>b</code></td></tr></table>"),
        "| h |\n| --- |\n| [`a`](/x) `b` |\n"
    );
}

#[test]
fn should_render_links_outside_keyboard_and_sample_code_spans() {
    for tag in ["kbd", "samp"] {
        assert_eq!(
            content(&format!("<p><{tag}><a href=\"/k\">Enter</a></{tag}></p>")),
            "[`Enter`](/k)\n"
        );
    }
}

#[test]
fn should_omit_empty_anchors_inside_code() {
    for tag in ["code", "kbd", "samp"] {
        assert_eq!(
            content(&format!("<p><{tag}>x<a href=\"/y\"></a>z</{tag}></p>")),
            "`xz`\n"
        );
    }
}

#[test]
fn should_render_only_image_alt_text_inside_linked_code() {
    assert_eq!(
        content("<p><code><a href=\"/y\"><img src=\"/i.png\" alt=\"pic\"></a></code></p>"),
        "[`pic`](/y)\n"
    );
}

#[test]
fn should_preserve_quotes_beside_anchors_inside_code() {
    assert_eq!(
        content("<p><code><a href=\"/x\">a</a><q>b</q></code></p>"),
        "[`a`](/x)`\"b\"`\n"
    );
}

#[test]
fn should_ignore_template_anchors_inside_code() {
    assert_eq!(
        content("<p><code>x<template><a href=\"/t\">t</a></template></code></p>"),
        "`x`\n"
    );
}

#[test]
fn should_use_the_same_label_for_empty_text_wrappers() {
    for body in ["", "<span></span>", "<b></b>", "<em></em>", "<span>&#32;</span>"] {
        assert_eq!(
            content(&format!("<p><a href=\"#\">{body}</a></p>")),
            "[](#)\n",
            "body {body}"
        );
    }
}

#[test]
fn should_preserve_link_ranges_at_a_heading_break() {
    assert_eq!(
        content("<h2><code>x<a href=\"/x\">a  </a><br>b</code></h2>"),
        "## `x`[`a`](/x) `b`\n"
    );
}

#[test]
fn should_preserve_quotes_inside_link_labels_and_ignore_noscript() {
    assert_eq!(
        content("<p><code><a href=\"/x\"><q>a</q></a><noscript><a href=\"/t\">t</a></noscript></code></p>"),
        "[`\"a\"`](/x)\n"
    );
}

#[test]
fn should_keep_backticks_and_punctuation_literal_in_image_alts() {
    assert_eq!(
        content("<p><code><a href=\"/x\"><img src=\"/i.png\" alt=\"x`[y]*\"></a></code></p>"),
        "[``x`[y]*``](/x)\n"
    );
}

#[cfg(feature = "visitor")]
#[test]
fn should_keep_keyboard_and_sample_visitor_behavior_with_links() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct Visitor;
    impl HtmlVisitor for Visitor {
        fn visit_code_inline(&mut self, _ctx: &NodeContext, _code: &str) -> VisitResult {
            VisitResult::Custom("replacement".to_string())
        }
    }
    for tag in ["kbd", "samp"] {
        let options = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            visitor: Some(Arc::new(Mutex::new(Visitor))),
            ..ConversionOptions::default()
        };
        assert_eq!(
            convert(&format!("<p><{tag}><a href=\"/x\">a</a></{tag}></p>"), Some(options))
                .expect("conversion succeeds")
                .content
                .unwrap_or_default(),
            "[`a`](/x)\n"
        );
    }
}

#[test]
fn should_preserve_empty_and_spaced_quote_siblings_of_code_links() {
    assert_eq!(
        content("<p><code><a href=\"/x\">a</a><q></q><q> b </q></code></p>"),
        "[`a`](/x)`\"b\"`\n"
    );
}
