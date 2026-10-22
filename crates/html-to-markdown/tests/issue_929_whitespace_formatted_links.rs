#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

#[test]
fn should_preserve_word_separation_around_whitespace_only_formatted_links() {
    for body in [
        "<i> </i>",
        "<em> </em>",
        "<strong> </strong>",
        "<b><i> </i></b>",
        "<span>&nbsp;</span>",
        "<i> \t\n </i>",
    ] {
        for tier_strategy in [
            TierStrategy::Auto,
            #[cfg(feature = "testkit")]
            TierStrategy::Tier1,
            TierStrategy::Tier2,
        ] {
            let html = format!("<p><i>employees</i><a href=\"https://example.com\">{body}</a><i>other</i></p>");
            let actual = convert(
                &html,
                Some(ConversionOptions {
                    tier_strategy,
                    extract_metadata: false,
                    ..ConversionOptions::default()
                }),
            )
            .expect("conversion should succeed")
            .content
            .unwrap_or_default();
            assert_eq!(actual, "*employees* [](https://example.com)*other*\n", "body: {body}");
        }
    }
}

#[test]
fn should_keep_empty_and_nonempty_link_labels_unchanged() {
    for (body, expected) in [
        ("<i></i>", "a[](https://example.com)b\n"),
        ("<i>text</i>", "a[*text*](https://example.com)b\n"),
        ("<i> </i>", "a [](https://example.com)b\n"),
    ] {
        let html = format!("<p>a<a href=\"https://example.com\">{body}</a>b</p>");
        let actual = convert(&html, None)
            .expect("conversion should succeed")
            .content
            .unwrap_or_default();
        assert_eq!(actual, expected);
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_add_source_space_only_when_the_visitor_keeps_the_converted_link() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct LinkVisitor(VisitResult);

    impl HtmlVisitor for LinkVisitor {
        fn visit_link(&mut self, _: &NodeContext<'_>, _: &str, text: &str, _: Option<&str>) -> VisitResult {
            assert_eq!(text, "");
            self.0.clone()
        }
    }

    for (decision, expected) in [
        (VisitResult::Continue, "a [](https://example.com)b\n"),
        (VisitResult::Skip, "ab\n"),
        (VisitResult::Custom("X".to_string()), "aXb\n"),
    ] {
        let actual = convert(
            "<p>a<a href=\"https://example.com\"><i> </i></a>b</p>",
            Some(ConversionOptions {
                visitor: Some(Arc::new(Mutex::new(LinkVisitor(decision)))),
                ..ConversionOptions::default()
            }),
        )
        .expect("conversion should succeed")
        .content
        .unwrap_or_default();
        assert_eq!(actual, expected);
    }
}

#[test]
fn should_avoid_adding_spaces_at_empty_anchor_boundaries() {
    for (html, expected) in [
        ("<p>a <a href=/x><i> </i></a>b</p>", "a [](/x)b\n"),
        ("<p><a href=/x><i> </i></a>b</p>", "[](/x)b\n"),
        ("<p>a<a href=\"#here\"><i> </i></a>b</p>", "ab\n"),
    ] {
        let actual = convert(html, None)
            .expect("conversion should succeed")
            .content
            .unwrap_or_default();
        assert_eq!(actual, expected);
    }
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_whitespace_outside_the_collected_link_label() {
    let result =
        convert("<p>a<a href=\"https://example.com\"><i> </i></a>b</p>", None).expect("conversion should succeed");
    assert_eq!(result.content.as_deref(), Some("a [](https://example.com)b\n"));
    assert_eq!(result.metadata.links.len(), 1);
    assert_eq!(result.metadata.links[0].href, "https://example.com");
    assert_eq!(result.metadata.links[0].text, "");
}
