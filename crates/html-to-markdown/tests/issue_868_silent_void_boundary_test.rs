//! Silent descendant boundaries retain main parity without erasing visible breaks. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};
fn assert_routes(html: &str, expected: &str) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for extract_metadata in [false, true] {
            let result = convert(
                html,
                Some(ConversionOptions {
                    tier_strategy,
                    extract_metadata,
                    ..ConversionOptions::default()
                }),
            )
            .expect("boundary converts");
            assert_eq!(
                result.content.as_deref(),
                Some(expected),
                "{html:?}, {tier_strategy:?}, metadata={extract_metadata}"
            );
        }
    }
}
#[test]
fn should_join_a_line_end_to_nbsp_after_silent_void_descendants() {
    for tag in ["input", "wbr", "area", "source", "col", "embed", "meta", "link"] {
        for depth in [0, 2] {
            let body = format!("{}<{tag}>{}", "<em>".repeat(depth), "</em>".repeat(depth));
            for tail in ["&nbsp;", "\u{a0}"] {
                for (container, expected) in [("p", "a b\n"), ("h2", "## a b\n")] {
                    assert_routes(
                        &format!("<{container}>a\n<span>{body}</span>{tail}b</{container}>"),
                        expected,
                    );
                }
            }
        }
    }
}
#[test]
fn should_keep_visible_void_descendants_as_content_at_any_depth() {
    assert_routes("<p>a\n<span><b><br></b></span>&nbsp;b</p>", "a  \n\u{a0}b\n");
    assert_routes("<p>a\n<span><img alt='X' src='x'></span>&nbsp;b</p>", "a ![X](x) b\n");
}

const AUTHORED_VOID_CASES: [(&str, &str, &str); 7] = [
    ("<input value='X'>", "a b\n", "a <input value=\"X\"> b\n"),
    ("<input placeholder='X'>", "a b\n", "a <input placeholder=\"X\"> b\n"),
    (
        "<input type='checkbox' checked>",
        "a b\n",
        "a <input type=\"checkbox\" checked> b\n",
    ),
    (
        "<input type='button' value='X'>",
        "a b\n",
        "a <input type=\"button\" value=\"X\"> b\n",
    ),
    ("<embed src='/media'>", "a b\n", "a <embed src=\"/media\"> b\n"),
    ("<object data='/media'>X<embed></object>", "a X b\n", "a X<embed> b\n"),
    ("<wbr>", "a b\n", "a <wbr> b\n"),
];

#[test]
fn should_keep_authored_void_output_and_existing_control_ownership() {
    for (body, plain, preserved) in AUTHORED_VOID_CASES {
        for preprocessing in ["standard", "minimal", "disabled"] {
            for preserve in [false, true] {
                let mut options = ConversionOptions {
                    extract_metadata: false,
                    tier_strategy: TierStrategy::Tier2,
                    ..ConversionOptions::default()
                };
                options.preprocessing.remove_forms = false;
                options.preprocessing.enabled = preprocessing != "disabled";
                if preprocessing == "minimal" {
                    options.preprocessing.preset = html_to_markdown_rs::PreprocessingPreset::Minimal;
                }
                if preserve {
                    options.preserve_tags = vec!["input".into(), "embed".into(), "wbr".into()];
                }
                let source = format!("<p>a\n<span>{body}</span>&nbsp;b</p>");
                assert_eq!(
                    convert(&source, options).expect("authored void").content.as_deref(),
                    Some(if preserve { preserved } else { plain }),
                    "{source}, {preprocessing}, preserve={preserve}"
                );
            }
        }
    }
}

#[test]
fn should_keep_table_checkbox_and_preserved_control_output() {
    for (preserve, expected) in [
        (false, "| b |\n| --- |\n"),
        (
            true,
            "| <input type=\"checkbox\" checked> b |\n| --------------------------------- |\n",
        ),
    ] {
        let source = "<table><tr><td><span><input type='checkbox' checked></span>&nbsp;b</td></tr></table>";
        let mut options = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..ConversionOptions::default()
        };
        options.preprocessing.remove_forms = false;
        if preserve {
            options.preserve_tags = vec!["input".into()];
        }
        assert_eq!(
            convert(source, options).expect("table checkbox").content.as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn should_join_heading_line_end_to_nbsp_after_an_image_that_renders_nothing() {
    let mut rows = 0;
    for (tag, prefix) in [("h1", "# "), ("h2", "## "), ("h6", "###### ")] {
        for image in ["<img>", "<img src='' alt=''>", "<img src='x'>"] {
            for depth in [0, 2] {
                let body = format!("{}{}{}", "<span>".repeat(depth), image, "</span>".repeat(depth));
                for tail in ["&nbsp;", "\u{a0}"] {
                    let source = format!("<{tag}>a\n<span>{body}</span>{tail}b</{tag}>");
                    assert_routes(&source, &format!("{prefix}a b\n"));
                    rows += 1;
                }
            }
        }
    }
    assert_eq!(rows, 36);
}

#[cfg(feature = "visitor")]
#[test]
fn should_keep_unobserved_image_probes_out_of_preformatted_boundary_ownership() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct EmptyImages {
        custom: bool,
        calls: usize,
    }
    impl HtmlVisitor for EmptyImages {
        fn visit_image(&mut self, _: &NodeContext<'_>, _: &str, _: &str, _: Option<&str>) -> VisitResult {
            self.calls += 1;
            if self.custom {
                VisitResult::Custom(String::new())
            } else {
                VisitResult::Continue
            }
        }
    }
    let source = "<h2>a\n<span><pre><a href='#x'><img></a></pre></span>&nbsp;b</h2>";
    let mut rows = 0;
    for custom in [false, true] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            for extract_metadata in [false, true] {
                let visitor = Arc::new(Mutex::new(EmptyImages { custom, calls: 0 }));
                let result = convert(
                    source,
                    ConversionOptions {
                        tier_strategy,
                        extract_metadata,
                        visitor: Some(visitor.clone()),
                        ..Default::default()
                    },
                )
                .expect("preformatted image probe converts");
                assert_eq!(
                    result.content.as_deref(),
                    Some("## a  b\n"),
                    "{tier_strategy:?}, custom={custom}"
                );
                assert_eq!(visitor.lock().expect("visitor state").calls, 1);
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 12);
}

// ~keep Forced-fast skip_images behavior is inherited; this repair preserves its current output.
const HEADING_IMAGE_CONTROLS: [(&str, &str, &str, Option<&str>); 16] = [
    (
        "<h2>a\n<span><img alt='X' src='x'></span>&nbsp;b</h2>",
        "default",
        "## a X b\n",
        None,
    ),
    (
        "<h2>a\n<span><img alt='X'></span>&nbsp;b</h2>",
        "default",
        "## a X b\n",
        None,
    ),
    (
        "<h2>a\n<span><img alt='&nbsp;'></span>&nbsp;b</h2>",
        "default",
        "## a   b\n",
        None,
    ),
    (
        "<h2>a\n<span><img src='x'></span>&nbsp;b</h2>",
        "inline",
        "## a ![](x) b\n",
        None,
    ),
    (
        "<h2>a\n<span><img></span>&nbsp;b</h2>",
        "preserve",
        "## a <img> b\n",
        None,
    ),
    ("<h2>a\n<span><img></span>&nbsp;b</h2>", "strict", "## a  b\n", None),
    ("<p>a\n<span><img></span>&nbsp;b</p>", "default", "a  b\n", None),
    ("<h2>a <span><img></span>&nbsp;b</h2>", "default", "## a  b\n", None),
    (
        "<h2>a\n<span><code><img></code></span>&nbsp;b</h2>",
        "default",
        "## a  b\n",
        None,
    ),
    (
        "<h2>a\n<span><pre><img></pre></span>&nbsp;b</h2>",
        "default",
        "## a  b\n",
        None,
    ),
    ("<h2>a\n<span><img></span>&nbsp;b</h2>", "skip", "## a  b\n", None),
    (
        "<h2>a\n<span><img alt='X' src='x'></span>&nbsp;b</h2>",
        "skip",
        "## a  b\n",
        Some("## a X b\n"),
    ),
    (
        "<h2>a\n<span><img src='data:image/svg+xml;base64,PHN2Zz48L3N2Zz4='></span>&nbsp;b</h2>",
        "drop",
        "## a  b\n",
        None,
    ),
    (
        "<h2>a\n<span><img alt='V' src='data:image/svg+xml;base64,PHN2Zz48L3N2Zz4='></span>&nbsp;b</h2>",
        "inline",
        "## a ![V](data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=) b\n",
        None,
    ),
    (
        "<h2>a\n<span><img alt='&#x2003;X&#160;' src='x'></span>&nbsp;b</h2>",
        "default",
        "## a  X  b\n",
        None,
    ),
    (
        "<h2>a\n<span><img><br></span>&nbsp;b</h2>",
        "default",
        "## a  b\n",
        None,
    ),
];
#[test]
fn should_keep_visible_literal_excluded_and_nonheading_image_boundaries() {
    let mut rows = 0;
    for (source, mode, expected, fast) in HEADING_IMAGE_CONTROLS {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            for extract_metadata in [false, true] {
                let mut options = ConversionOptions {
                    tier_strategy,
                    extract_metadata,
                    ..Default::default()
                };
                match mode {
                    "inline" => options.keep_inline_images_in = vec!["h2".into()],
                    "preserve" => options.preserve_tags = vec!["img".into()],
                    "strict" => options.whitespace_mode = html_to_markdown_rs::WhitespaceMode::Strict,
                    "skip" => options.skip_images = true,
                    "drop" => options.inline_data_media = html_to_markdown_rs::InlineDataMedia::DropElement,
                    _ => {}
                }
                let expected = if tier_strategy == TierStrategy::Tier1 {
                    fast.unwrap_or(expected)
                } else {
                    expected
                };
                assert_eq!(
                    convert(source, options)
                        .expect("image boundary control")
                        .content
                        .as_deref(),
                    Some(expected),
                    "{source}, {mode}, {tier_strategy:?}"
                );
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 96);
}
