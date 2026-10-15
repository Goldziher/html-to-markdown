#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

const STRATEGIES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2];

fn content(html: &str, tier_strategy: TierStrategy) -> String {
    convert(
        html,
        Some(ConversionOptions {
            tier_strategy,
            extract_metadata: false,
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed")
    .content
    .expect("conversion should return Markdown")
}

#[test]
fn should_keep_slashes_in_unquoted_link_attributes() {
    for strategy in STRATEGIES {
        for href in ["/x", "https://example.com/a/b", "/search?q='tis", "/x/"] {
            let html = format!("<p>before</p><a href={href}>link</a><p>after</p>");
            assert_eq!(
                content(&html, strategy),
                format!("before\n\n[link]({href})\n\nafter\n"),
                "{strategy:?}: {html}"
            );
        }
        assert_eq!(content("<p>one<br class=a/>two</p>", strategy), "one  \ntwo\n");
    }
}

#[test]
fn should_treat_quotes_inside_unquoted_values_as_literal_bytes() {
    for strategy in STRATEGIES {
        let html = "<p>before</p><a href=/x title=it's>link</a><p>it's after</p>";
        assert_eq!(
            content(html, strategy),
            "before\n\n[link](/x \"it's\")\n\nit's after\n",
            "{strategy:?}"
        );
        let html = "<p>before</p><a href=/x title=it's>link</a><textarea><b>x</b></textarea><p>it's after</p>";
        assert_eq!(
            content(html, strategy),
            "before\n\n[link](/x \"it's\") <b>x</b>\n\nit's after\n",
            "{strategy:?}"
        );
    }
}

#[test]
fn should_recover_unterminated_raw_text_consistently() {
    for strategy in STRATEGIES {
        for start in [
            "<script>",
            "<script data=it's>",
            "<script\x0C>",
            "<style>",
            "<style media=it's>",
            "<style\x0C>",
        ] {
            let html = format!("<p>a</p>{start}<p>b</p>");
            assert_eq!(content(&html, strategy), "a\n\nb\n", "{strategy:?}: {html}");
        }
        assert_eq!(
            content("<p>a</p><script>var x = 1; <p>b</p>", strategy),
            "a\n\nvar x = 1;\n\nb\n",
            "{strategy:?}"
        );
        assert_eq!(
            content("<style\n  type=\"text/css\">\n\nfoo", strategy),
            "foo\n",
            "{strategy:?}"
        );
    }
}

#[test]
fn should_remove_raw_text_without_inserting_spaces() {
    for strategy in STRATEGIES {
        for element in ["<script>x</script>", "<style>p{}</style>"] {
            for (before, after, expected) in [
                ("<p>foo", "bar</p>", "foobar\n"),
                ("<pre>foo", "bar</pre>", "```\nfoobar\n```\n"),
            ] {
                if before == "<p>foo" && element.starts_with("<style>") {
                    continue;
                }
                let html = format!("{before}{element}{after}");
                assert_eq!(content(&html, strategy), expected, "{strategy:?}: {html}");
            }
        }
    }
}

#[test]
fn should_preserve_blank_first_pre_line_after_head_repair() {
    for strategy in STRATEGIES {
        assert_eq!(
            content("<title>T</title><pre>\n\nx\n</pre>", strategy),
            "```\n\nx\n```\n",
            "{strategy:?}"
        );
        assert_eq!(
            content("<head><title>T</title></head><pre>\n\nx\n</pre>", strategy),
            "```\n\nx\n```\n",
            "{strategy:?}"
        );
        #[cfg(feature = "metadata")]
        {
            let result = convert(
                "<title>T</title><pre>\n\nx\n</pre>",
                Some(ConversionOptions {
                    tier_strategy: strategy,
                    extract_metadata: true,
                    ..ConversionOptions::default()
                }),
            )
            .expect("convert with metadata");
            assert_eq!(result.content.as_deref(), Some("---\ntitle: T\n---\n\n```\n\nx\n```\n"));
        }
    }
}

#[test]
fn should_repair_paragraph_boundaries_beside_unusual_raw_text_tags() {
    for strategy in STRATEGIES {
        let html = "<p>Hello<form>name</form><script/x>var a = \"<b>x</b>\";</script/><form>mail</form>world</p>";
        assert_eq!(content(html, strategy), "Hello\n\nworld\n", "{strategy:?}");
        assert_eq!(
            content(
                "<p>Hello<form>name</form><script>x</script><form>mail</form>world</p>",
                strategy
            ),
            "Hello\n\nworld\n",
            "{strategy:?}"
        );
        for html in [
            "<p>Hello<dt><textarea\x0Cclass=c>some <b>bold</b> text</textarea\x0C></dt>world</p>",
            "<p>Hello<dt><textarea class=c>some <b>bold</b> text</textarea></dt>world</p>",
        ] {
            assert_eq!(
                content(html, strategy),
                "Hello\n\nsome <b>bold</b> text\n\nworld\n",
                "{strategy:?}: {html}"
            );
        }
    }
}

#[test]
fn should_keep_authored_frameset_text_when_paragraph_repair_discards_it() {
    let html = "<frameset><frame></frameset><p>a<p>b";
    for strategy in STRATEGIES {
        assert_eq!(content(html, strategy), "a\n\nb\n", "{strategy:?}");
    }
}
