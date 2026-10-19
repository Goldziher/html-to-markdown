#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, InlineDataMedia, TierStrategy, convert};

fn assert_routes(html: &str, expected: &str, inline_data_media: InlineDataMedia) {
    for tier_strategy in [
        TierStrategy::Auto,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
        TierStrategy::Tier2,
    ] {
        let actual = convert(
            html,
            ConversionOptions {
                tier_strategy,
                inline_data_media,
                extract_metadata: false,
                highlight_style: HighlightStyle::None,
                ..ConversionOptions::default()
            },
        )
        .expect("media conversion succeeds")
        .content
        .expect("content is requested");
        assert_eq!(actual, expected, "{tier_strategy:?}: {html}");
    }
}

#[test]
fn should_separate_svg_labels_from_neighboring_link_and_list_words() {
    for (html, expected) in [
        (
            r#"<a href="https://github.blog">Blog<svg aria-label="External link" role="img"></svg></a>"#,
            "[Blog External link](https://github.blog)\n",
        ),
        (
            r#"<ul><li>Documentation<svg aria-label="External link" role="img"></svg></li><li>Blog</li></ul>"#,
            "- Documentation External link\n- Blog\n",
        ),
        (
            r#"<p><svg aria-label="Note" role="img"></svg>Remember to save</p>"#,
            "Note Remember to save\n",
        ),
        (
            r#"<p>Blog<strong><svg aria-label="External link"></svg></strong>Next</p>"#,
            "Blog **External link** Next\n",
        ),
        (r#"<p>Gruß<svg aria-label="外部"></svg>世界</p>"#, "Gruß 外部 世界\n"),
    ] {
        assert_routes(html, expected, InlineDataMedia::AltTextOnly);
    }
}

#[test]
fn should_separate_image_alt_words_inside_headings_and_code() {
    for (html, expected) in [
        (r#"<h2>hot<img src="i.png" alt="pic">end</h2>"#, "## hot pic end\n"),
        (
            r#"<pre>link hot<img src="i.png" alt="pic"> end</pre>"#,
            "```\nlink hot pic end\n```\n",
        ),
        (
            r#"<p>run <code>hot<img src="i.png" alt="pic"> end</code></p>"#,
            "run `hot pic end`\n",
        ),
        (
            r#"<p>hot<img src="data:image/png;base64,AAAA" alt="pic">end</p>"#,
            "hot pic end\n",
        ),
    ] {
        assert_routes(html, expected, InlineDataMedia::AltTextOnly);
    }
}

#[test]
fn should_separate_nested_and_entity_edges_when_heading_images_use_alt_text() {
    for (html, expected) in [
        (r#"<h2>hot<img src="i.png" alt="pic"></h2>"#, "## hot pic\n"),
        (
            r#"<h2><span><img src="i.png" alt="pic"></span>end</h2>"#,
            "## pic end\n",
        ),
        (
            r#"<h2>hot<img src="i.png" alt="pic"><span>end</span></h2>"#,
            "## hot pic end\n",
        ),
        (r#"<h2><img src="i.png" alt="pic">&#101;nd</h2>"#, "## pic end\n"),
        (r#"<h2>hot<img src="i.png" alt="pi&#99;">end</h2>"#, "## hot pic end\n"),
    ] {
        assert_routes(html, expected, InlineDataMedia::Keep);
    }
}

#[test]
fn should_keep_code_link_ranges_valid_after_inserting_a_media_boundary() {
    for (html, expected) in [
        (
            r#"<code><img src="i.png" alt="a"><a href="/x">日本語</a></code>"#,
            "`a `[`日本語`](/x)\n",
        ),
        (
            r#"<code><a href="/a">一</a><span><img src="i.png" alt="p"><a href="/b">二</a></span></code>"#,
            "[`一`](/a)` p `[`二`](/b)\n",
        ),
    ] {
        assert_routes(html, expected, InlineDataMedia::Keep);
    }
}

#[test]
fn should_keep_preformatted_cell_break_offsets_after_inserting_a_media_boundary() {
    let actual = convert(
        r#"<table><tr><th>h</th></tr><tr><td><pre><img src="i.png" alt="a"><span>日本語<br>after</span></pre></td></tr></table>"#,
        ConversionOptions {
            compact_tables: true,
            br_in_tables: true,
            extract_metadata: false,
            ..ConversionOptions::default()
        },
    )
    .expect("preformatted cell conversion succeeds")
    .content
    .expect("content is requested");
    assert_eq!(actual, "| h |\n| --- |\n| `a 日本語`<br>`after` |\n");
}

#[test]
fn should_preserve_authored_media_spacing_and_punctuation() {
    for (html, expected) in [
        (
            r#"<p>Blog <svg aria-label="External link"></svg> Next</p>"#,
            "Blog External link Next\n",
        ),
        (
            r#"<pre>hot  <img src="i.png" alt="pic">  end</pre>"#,
            "```\nhot  pic  end\n```\n",
        ),
        (r#"<p>(<svg aria-label="Note"></svg>).</p>"#, "(Note).\n"),
        ("<p>a<svg></svg>b</p>", "ab\n"),
        (r#"<p>a<svg aria-label="."></svg>b</p>"#, "a.b\n"),
        (r#"<p>a<svg hidden aria-label="Note"></svg>b</p>"#, "ab\n"),
    ] {
        assert_routes(html, expected, InlineDataMedia::AltTextOnly);
    }
}

#[test]
fn should_not_add_a_boundary_for_dropped_media() {
    assert_routes(
        r#"<p>a<svg aria-label="Note"></svg>b</p>"#,
        "ab\n",
        InlineDataMedia::DropElement,
    );
}

#[test]
fn should_keep_picture_text_and_nested_captions() {
    for (html, expected) in [
        (
            "<picture>Caption text here</picture><p>after</p>",
            "Caption text here\n\nafter\n",
        ),
        (
            r#"<p>before</p><picture><div><img src="a.png" alt=""></div><figcaption>Caption words</figcaption></picture><h2>after</h2>"#,
            "before\n\n![](a.png)\n\n*Caption words*\n\n## after\n",
        ),
        (
            "<p>one<picture><span>Caption</span></picture>end</p>",
            "oneCaptionend\n",
        ),
    ] {
        assert_routes(html, expected, InlineDataMedia::Keep);
    }
}

#[test]
fn should_keep_picture_sources_out_of_text_and_render_its_image_once() {
    assert_routes(
        r#"<picture><source srcset="alternative.webp 1x"><img src="fallback.png" alt="Image"></picture>"#,
        "![Image](fallback.png)\n",
        InlineDataMedia::Keep,
    );
}

#[cfg(feature = "testkit")]
#[test]
fn should_keep_text_and_ordinary_markdown_images_on_the_direct_fast_path() {
    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    for (html, expected) in [
        ("<p>plain <strong>text</strong></p>", "plain **text**\n"),
        (r#"<h4><img src="i.png" alt="pic"></h4>"#, "#### pic\n"),
        (r#"<H4><img src="i.png" alt="pic"></H4>"#, "#### pic\n"),
        (r#"<h1><img src="i.png" alt="fox &mdash; cafe"></h1>"#, "# fox — cafe\n"),
        (r#"<h2><img src="i.png" alt="pic"> end</h2>"#, "## pic end\n"),
        (r#"<p>hot<img src="i.png" alt="pic">end</p>"#, "hot![pic](i.png)end\n"),
    ] {
        let actual =
            html_to_markdown_rs::tier1::run(html, &html_to_markdown_rs::prescan::PrescanReport::default(), &options)
                .expect("ordinary content should stay on the fast path");
        assert_eq!(actual, expected, "{html}");
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_not_add_media_spacing_for_visitor_skips_or_custom_output() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct ImageDecision(VisitResult);

    impl HtmlVisitor for ImageDecision {
        fn visit_image(&mut self, _: &NodeContext<'_>, _: &str, _: &str, _: Option<&str>) -> VisitResult {
            self.0.clone()
        }
    }

    for (decision, expected) in [
        (VisitResult::Skip, "ab\n"),
        (VisitResult::Custom("replaced".to_owned()), "areplacedb\n"),
    ] {
        let actual = convert(
            r#"<p>a<img src="data:image/png;base64,AAAA" alt="pic">b</p>"#,
            ConversionOptions {
                visitor: Some(Arc::new(Mutex::new(ImageDecision(decision)))),
                inline_data_media: InlineDataMedia::AltTextOnly,
                ..ConversionOptions::default()
            },
        )
        .expect("visitor conversion succeeds")
        .content
        .expect("content is requested");
        assert_eq!(actual, expected);
    }
}
