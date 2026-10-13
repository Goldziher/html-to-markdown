//! Empty nonflow list blocks retain the Unicode prefix beside structural indentation. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, convert};
const EMPTY_NONFLOW_TAGS: [&str; 4] = ["aside", "footer", "main", "details"];
const MARKUP_FORMATS: [OutputFormat; 2] = [OutputFormat::Markdown, OutputFormat::Djot];

fn assert_routes(source: &str, expected: &str, options: &ConversionOptions) -> usize {
    let mut rows = 0;
    for newline_style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            for extract_metadata in [false, true] {
                let result = convert(
                    source,
                    ConversionOptions {
                        tier_strategy,
                        extract_metadata,
                        newline_style,
                        ..options.clone()
                    },
                )
                .expect("list continuation");
                assert_eq!(
                    result.content.as_deref(),
                    Some(expected),
                    "{source:?} {tier_strategy:?} {newline_style:?} {:?}",
                    options.output_format
                );
                rows += 1;
            }
        }
    }
    rows
}

#[test]
fn should_keep_nbsp_prefix_after_empty_nonflow_list_blocks() {
    let mut rows = 0;
    for tag in EMPTY_NONFLOW_TAGS {
        for tail in ["&nbsp;b", "\u{a0}b"] {
            let source = format!("<ul><li>a \n<span><{tag}></{tag}></span>{tail}</li></ul>");
            for output_format in MARKUP_FORMATS {
                rows += assert_routes(
                    &source,
                    "- a\n   b\n",
                    &ConversionOptions {
                        output_format,
                        ..Default::default()
                    },
                );
            }
        }
    }
    assert_eq!(rows, 192);
}

#[test]
fn should_keep_strict_and_plain_nonflow_list_content_unchanged() {
    use html_to_markdown_rs::WhitespaceMode;
    let mut rows = 0;
    for tag in EMPTY_NONFLOW_TAGS {
        for tail in ["&nbsp;b", "\u{a0}b"] {
            let source = format!("<ul><li>a \n<span><{tag}></{tag}></span>{tail}</li></ul>");
            for (output_format, whitespace_mode, expected) in [
                (OutputFormat::Markdown, WhitespaceMode::Strict, "- a\n  \u{a0}b\n"),
                (OutputFormat::Djot, WhitespaceMode::Strict, "- a\n  \u{a0}b\n"),
                (OutputFormat::Plain, WhitespaceMode::Normalized, "- a\n\n b\n"),
                (OutputFormat::Plain, WhitespaceMode::Strict, "- a\n\n b\n"),
            ] {
                rows += assert_routes(
                    &source,
                    expected,
                    &ConversionOptions {
                        output_format,
                        whitespace_mode,
                        ..Default::default()
                    },
                );
            }
        }
    }
    assert_eq!(rows, 384);
}

#[test]
fn should_keep_covered_blocks_and_unrelated_list_boundaries_unchanged() {
    let mut rows = 0;
    for (tag, expected) in [
        ("h1", "- a\n   b\n"),
        ("section", "- a\n   b\n"),
        ("header", "- a\n  b\n"),
        ("x-y", "- a b\n"),
    ] {
        let source = format!("<ul><li>a \n<span><{tag}></{tag}></span>&nbsp;b</li></ul>");
        for output_format in MARKUP_FORMATS {
            rows += assert_routes(
                &source,
                expected,
                &ConversionOptions {
                    output_format,
                    ..Default::default()
                },
            );
        }
    }
    assert_eq!(rows, 96);
}

#[test]
fn should_merge_ascii_whitespace_after_empty_nonflow_list_blocks() {
    let mut rows = 0;
    for tag in EMPTY_NONFLOW_TAGS {
        let source = format!("<ul><li>a \n<span><{tag}></{tag}></span> b</li></ul>");
        for output_format in MARKUP_FORMATS {
            rows += assert_routes(
                &source,
                "- a\n  b\n",
                &ConversionOptions {
                    output_format,
                    ..Default::default()
                },
            );
        }
    }
    assert_eq!(rows, 96);
}

#[test]
fn should_keep_nonlist_empty_block_spacing_unchanged() {
    let mut rows = 0;
    for tag in EMPTY_NONFLOW_TAGS {
        let source = format!("<div>a \n<span><{tag}></{tag}></span>&nbsp;b</div>");
        for output_format in MARKUP_FORMATS {
            rows += assert_nonlist_routes(&source, tag, output_format);
        }
    }
    assert_eq!(rows, 96);
}

#[test]
fn should_keep_nonempty_list_block_spacing_unchanged() {
    let mut rows = 0;
    for tag in EMPTY_NONFLOW_TAGS {
        let source = format!("<ul><li>a \n<span><{tag}>x</{tag}></span>&nbsp;b</li></ul>");
        for output_format in MARKUP_FORMATS {
            rows += assert_routes(
                &source,
                "- a\n\n  x\n\n  b\n",
                &ConversionOptions {
                    output_format,
                    ..Default::default()
                },
            );
        }
    }
    assert_eq!(rows, 96);
}

fn assert_nonlist_routes(source: &str, tag: &str, output_format: OutputFormat) -> usize {
    let mut rows = 0;
    for newline_style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            for extract_metadata in [false, true] {
                let expected = if tag == "details" || tier_strategy == TierStrategy::Tier1 {
                    "a\n\nb\n"
                } else {
                    "a b\n"
                };
                let result = convert(
                    source,
                    ConversionOptions {
                        tier_strategy,
                        extract_metadata,
                        newline_style,
                        output_format,
                        ..Default::default()
                    },
                )
                .expect("nonlist continuation");
                assert_eq!(
                    result.content.as_deref(),
                    Some(expected),
                    "{source:?} {tier_strategy:?}"
                );
                rows += 1;
            }
        }
    }
    rows
}
