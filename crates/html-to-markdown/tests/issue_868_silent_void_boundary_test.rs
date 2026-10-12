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
