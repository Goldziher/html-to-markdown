#![allow(missing_docs)]
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

fn assert_paths(html: &str, expected: &str) {
    for options in [
        ConversionOptions::default(),
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..ConversionOptions::default()
        },
        ConversionOptions {
            extract_metadata: false,
            highlight_style: HighlightStyle::None,
            ..ConversionOptions::default()
        },
    ] {
        let output = convert(html, Some(options))
            .expect("conversion succeeds")
            .content
            .unwrap_or_default();
        assert_eq!(output, expected, "{html}");
    }
}

#[test]
fn should_put_a_whole_inline_code_span_inside_its_link() {
    assert_paths(
        r#"<p>Use <code><a href="/wiki/While_loop" title="While loop">while</a></code> here.</p>"#,
        "Use [`while`](/wiki/While_loop \"While loop\") here.\n",
    );
    assert_paths(
        r#"<p>Use <a href="/wiki/While_loop"><code>while</code></a> here.</p>"#,
        "Use [`while`](/wiki/While_loop) here.\n",
    );
}

#[test]
fn should_split_code_around_partial_links_without_losing_destinations() {
    assert_paths(
        r#"<p>Call <code>list.<a href="/api/append">append</a>(x)</code> now.</p>"#,
        "Call `list.`[`append`](/api/append)`(x)` now.\n",
    );
    assert_paths(
        r#"<p><code>A<a href="/b">B</a>C<a href="/d">D</a>E</code></p>"#,
        "`A`[`B`](/b)`C`[`D`](/d)`E`\n",
    );
}

#[test]
fn should_keep_backticks_spaces_and_nested_styles_inside_link_code_labels() {
    assert_paths(
        r#"<p><code>x <span><a href="/b"><strong> b` </strong></a></span> y</code></p>"#,
        "`x `[``  b`  ``](/b)` y`\n",
    );
    assert_paths(r#"<p><code><a href="/b">`x`</a></code></p>"#, "[`` `x` ``](/b)\n");
}

#[test]
fn should_keep_code_blocks_literal_and_links_without_destinations_as_code() {
    assert_paths(
        r#"<pre><code>list.<a href="/api/append">append</a>(x)</code></pre>"#,
        "```\nlist.append(x)\n```\n",
    );
    assert_paths("<p><code>list.<a>append</a>(x)</code></p>", "`list.append(x)`\n");
    assert_paths(
        r#"<p><code>list.<a href="">append</a>(x)</code></p>"#,
        "`list.append(x)`\n",
    );
}

#[cfg(feature = "testkit")]
#[test]
fn should_route_code_links_from_the_scanner_to_the_full_converter() {
    use html_to_markdown_rs::{prescan, tier1};
    let html = r#"<p><code>a<a href="/b">b</a>c</code></p>"#;
    let (cleaned, report) = prescan::run(html);
    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    assert!(matches!(
        tier1::run(cleaned.as_ref(), &report, &options),
        Err(tier1::BailReason::Classifier)
    ));
    assert_paths(html, "`a`[`b`](/b)`c`\n");
}

#[test]
fn should_preserve_hard_breaks_around_and_inside_code_links() {
    assert_paths(
        r#"<p><code>a<br><a href="/b">b<br>c</a><br>d</code></p>"#,
        "`a`  \n[`b`  \n`c`](/b)  \n`d`\n",
    );
}

#[test]
fn should_keep_links_inside_nested_code_formatting_wrappers() {
    for wrapper in [
        "strong", "em", "span", "mark", "del", "ins", "small", "sub", "sup", "var", "dfn", "u", "code", "kbd", "samp",
        "abbr", "q",
    ] {
        let html = format!(r#"<p><code>prefix<{wrapper}><a href="/b">b</a></{wrapper}>tail</code></p>"#);
        let expected = if wrapper == "q" {
            "`prefix\"`[`b`](/b)`\"tail`\n"
        } else {
            "`prefix`[`b`](/b)`tail`\n"
        };
        assert_paths(&html, expected);
    }
}

#[test]
fn should_keep_code_labels_with_reference_links_and_resolved_destinations() {
    use html_to_markdown_rs::LinkStyle;
    let options = ConversionOptions {
        link_style: LinkStyle::Reference,
        base_url: Some("https://example.test/docs/".to_string()),
        ..ConversionOptions::default()
    };
    let output = convert(r#"<p><code>a<a href="b" title="B">b</a>c</code></p>"#, Some(options))
        .expect("conversion succeeds")
        .content
        .unwrap_or_default();
    assert_eq!(output, "`a`[`b`][1]`c`\n\n[1]: https://example.test/docs/b \"B\"\n");
}

#[test]
fn should_keep_link_offsets_through_block_containers_inside_inline_code() {
    for wrapper in [
        "div",
        "p",
        "blockquote",
        "ul",
        "ol",
        "li",
        "dl",
        "dt",
        "dd",
        "table",
        "tr",
        "td",
        "figure",
        "figcaption",
        "h1",
        "cite",
        "ruby",
    ] {
        let html = format!(r#"<p><code>pre<{wrapper}><a href="/a">x</a></{wrapper}>post</code></p>"#);
        assert_paths(&html, "`pre`[`x`](/a)`post`\n");
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_respect_visitors_that_skip_links_and_replace_containers() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct Visitor;
    impl HtmlVisitor for Visitor {
        fn visit_link(&mut self, _ctx: &NodeContext, href: &str, _text: &str, _title: Option<&str>) -> VisitResult {
            if href == "/skip" {
                VisitResult::Skip
            } else {
                VisitResult::Continue
            }
        }
        fn visit_element_end(&mut self, ctx: &NodeContext, _output: &str) -> VisitResult {
            if ctx.tag_name == "span" {
                VisitResult::Custom("custom".to_string())
            } else {
                VisitResult::Continue
            }
        }
    }
    let options = ConversionOptions {
        visitor: Some(Arc::new(Mutex::new(Visitor))),
        ..ConversionOptions::default()
    };
    let output = convert(
        r#"<p><code>pre<a href="/skip">skip</a><span><a href="/replace">replace</a></span>post</code></p>"#,
        Some(options),
    )
    .expect("conversion succeeds")
    .content
    .unwrap_or_default();
    assert_eq!(output, "`precustompost`\n");
}

#[test]
fn should_keep_collected_code_ranges_when_following_elements_are_excluded() {
    let options = ConversionOptions {
        exclude_selectors: vec![".omit".to_string()],
        ..ConversionOptions::default()
    };
    let output = convert(
        r#"<p><code>pre<a href="/a">x </a><span class="omit">omit</span></code></p>"#,
        Some(options),
    )
    .expect("conversion succeeds")
    .content
    .unwrap_or_default();
    assert_eq!(output, "`pre`[`x `](/a)\n");
}

#[test]
fn should_keep_link_ranges_and_code_whitespace_across_an_absent_image() {
    assert_paths(
        r#"<p><code>pre<a href="/a">x </a><img src=""> post</code></p>"#,
        "`pre`[`x `](/a)` post`\n",
    );
}
