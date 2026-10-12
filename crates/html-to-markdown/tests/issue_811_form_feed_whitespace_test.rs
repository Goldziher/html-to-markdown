#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, WhitespaceMode, convert};

fn check(html: &str, expected: &str, options: &ConversionOptions) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            html,
            Some(ConversionOptions {
                tier_strategy,
                ..options.clone()
            }),
        )
        .expect("conversion must succeed");
        let markdown = result.content.unwrap_or_default();
        assert_eq!(markdown, expected, "{tier_strategy:?}: {html:?}");
    }
}

#[test]
fn should_preserve_form_feeds_and_adjacent_word_spaces() {
    for (html, expected) in [
        ("<p>one<i>y</i>\u{c}\n<b>two</b></p>", "one*y*\u{c} **two**\n"),
        (
            "<div>one<i>y</i>\u{c}\n<!-- c --><b>two</b></div>",
            "one*y*\u{c} **two**\n",
        ),
        ("<p>one\u{c}<b>two</b></p>", "one\u{c}**two**\n"),
        ("<p>one\u{c}two</p>", "one\u{c}two\n"),
        ("<p>one<i>y</i>\u{c}<b>two</b></p>", "one*y*\u{c}**two**\n"),
        ("<p>one<i>y</i> \n<b>two</b></p>", "one*y* **two**\n"),
    ] {
        check(html, expected, &ConversionOptions::default());
    }
}

#[test]
fn should_preserve_a_form_feed_written_as_a_character_reference() {
    for reference in ["&#12;", "&#x0c;", "&#xC;"] {
        check(
            &format!("<p>one{reference}<b>two</b></p>"),
            "one\u{c}**two**\n",
            &ConversionOptions::default(),
        );
    }
}

#[test]
fn should_preserve_form_feeds_at_paragraph_boundaries() {
    for (html, expected) in [
        ("<p>\u{c}text</p>", "\u{c}text\n"),
        ("<p>text\u{c}</p>", "text\u{c}\n"),
        ("<p>\u{c}</p>", "\u{c}\n"),
        ("<p>\u{c}<!-- c --><b>text</b></p>", "\u{c}**text**\n"),
        ("<p><b>text</b><!-- c -->\u{c}</p>", "**text**\u{c}\n"),
    ] {
        for whitespace_mode in [WhitespaceMode::Normalized, WhitespaceMode::Strict] {
            check(
                html,
                expected,
                &ConversionOptions {
                    whitespace_mode,
                    ..Default::default()
                },
            );
        }
    }
}

#[test]
fn should_preserve_form_feeds_between_many_comment_fragments() {
    let fragments = "<!--c-->\u{c}\n".repeat(64);
    check(
        &format!("<p>a{fragments}<b>b</b></p>"),
        &format!("a{}**b**\n", "\u{c} ".repeat(64)),
        &ConversionOptions::default(),
    );
}
