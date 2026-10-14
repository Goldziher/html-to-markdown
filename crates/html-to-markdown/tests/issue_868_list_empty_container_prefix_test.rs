//! Regression coverage for the list continuation spacing noted in review #871. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, convert};
const EMPTY_BLOCKS: [&str; 8] = ["h1", "h3", "h4", "h5", "h6", "article", "figure", "dl"];
fn assert_routes(source: &str, expected: &str) -> usize {
    assert_options(source, expected, &ConversionOptions::default())
}
fn assert_options(source: &str, expected: &str, options: &ConversionOptions) -> usize {
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
                    "{source:?} {tier_strategy:?} {newline_style:?}"
                );
                rows += 1;
            }
        }
    }
    rows
}
#[test]
fn should_keep_nbsp_content_after_empty_headings_and_containers_in_list_spans() {
    let mut rows = 0;
    for tag in EMPTY_BLOCKS {
        for tail in ["&nbsp;b", "\u{a0}b"] {
            rows += assert_routes(
                &format!("<ul><li>a \n<span><{tag}></{tag}></span>{tail}</li></ul>"),
                "- a\n   b\n",
            );
        }
    }
    assert_eq!(rows, 192);
}
#[test]
fn should_keep_literal_backslash_n_distinct_from_a_source_line_ending() {
    let mut rows = 0;
    for tag in EMPTY_BLOCKS {
        for tail in ["&nbsp;b", "\u{a0}b"] {
            rows += assert_routes(
                &format!("<ul><li>a \\n<span><{tag}></{tag}></span>{tail}</li></ul>"),
                "- a \\n\n   b\n",
            );
        }
    }
    assert_eq!(rows, 192);
}
#[test]
fn should_merge_ascii_whitespace_after_empty_list_blocks() {
    let mut rows = 0;
    for tag in ["h1", "article", "dl"] {
        rows += assert_routes(
            &format!("<ul><li>a \n<span><{tag}></{tag}></span> b</li></ul>"),
            "- a\n  b\n",
        );
    }
    assert_eq!(rows, 36);
}
#[test]
fn should_keep_literal_nbsp_in_strict_list_whitespace() {
    let mut rows = 0;
    for tag in ["h1", "article", "dl"] {
        rows += assert_options(
            &format!("<ul><li>a \n<span><{tag}></{tag}></span>&nbsp;b</li></ul>"),
            "- a\n  \u{a0}b\n",
            &ConversionOptions {
                whitespace_mode: html_to_markdown_rs::WhitespaceMode::Strict,
                ..Default::default()
            },
        );
    }
    assert_eq!(rows, 36);
}
#[test]
fn should_keep_preserved_empty_block_markup_and_nbsp_content() {
    let mut rows = 0;
    for tag in ["h1", "article", "dl"] {
        rows += assert_options(
            &format!("<ul><li>a \n<span><{tag}></{tag}></span>&nbsp;b</li></ul>"),
            &format!("- a\n\n  <{tag}></{tag}>\n\n   b\n"),
            &ConversionOptions {
                preserve_tags: vec![tag.into()],
                ..Default::default()
            },
        );
    }
    assert_eq!(rows, 36);
}
#[test]
fn should_keep_authored_code_and_preformatted_list_bytes() {
    let mut rows = 0;
    for tag in ["h1", "article", "dl"] {
        let body = format!("<ul><li>a \n<span><{tag}></{tag}></span>&nbsp;b</li></ul>");
        rows += assert_routes(&format!("<code>{body}</code>"), "`- a  \u{a0}b`\n");
        rows += assert_routes(&format!("<pre>{body}</pre>"), "```\n- a \n\u{a0}b\n```\n");
    }
    assert_eq!(rows, 72);
}
#[test]
fn should_keep_plain_text_and_djot_list_continuation_bytes() {
    let mut rows = 0;
    for tag in EMPTY_BLOCKS {
        for tail in ["&nbsp;b", "\u{a0}b"] {
            let source = format!("<ul><li>a \n<span><{tag}></{tag}></span>{tail}</li></ul>");
            for output_format in [OutputFormat::Plain, OutputFormat::Djot] {
                let expected = match (output_format, tag) {
                    (OutputFormat::Djot, _) => "- a\n   b\n",
                    (_, "dl") => "- a\n b\n",
                    _ => "- a\n\n b\n",
                };
                rows += assert_options(
                    &source,
                    expected,
                    &ConversionOptions {
                        output_format,
                        ..Default::default()
                    },
                );
            }
        }
    }
    assert_eq!(rows, 384);
}
#[cfg(feature = "visitor")]
mod visitor_controls {
    use super::*;
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};
    #[derive(Clone, Copy, Debug)]
    enum Action {
        Continue,
        Skip,
        Custom,
        Preserve,
    }
    #[derive(Debug)]
    struct Hooks {
        tag: &'static str,
        action: Action,
        starts: usize,
        ends: usize,
    }
    impl HtmlVisitor for Hooks {
        fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
            if context.tag_name != self.tag {
                return VisitResult::Continue;
            }
            self.starts += 1;
            match self.action {
                Action::Continue => VisitResult::Continue,
                Action::Skip => VisitResult::Skip,
                Action::Custom => VisitResult::Custom("visible".into()),
                Action::Preserve => VisitResult::PreserveHtml,
            }
        }
        fn visit_element_end(&mut self, context: &NodeContext, _: &str) -> VisitResult {
            if context.tag_name == self.tag {
                self.ends += 1;
            }
            VisitResult::Continue
        }
    }
    fn assert_hook_routes(tag: &'static str, action: Action) -> usize {
        let source = format!("<ul><li>a \n<span><{tag}></{tag}></span>&nbsp;b</li></ul>");
        let (expected, ends) = match action {
            Action::Custom => ("- a\n  visible b\n", 1),
            Action::Skip => ("- a\n   b\n", 0),
            _ => ("- a\n   b\n", 1),
        };
        let mut rows = 0;
        for newline_style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
            for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
                for extract_metadata in [false, true] {
                    let hooks = Arc::new(Mutex::new(Hooks {
                        tag,
                        action,
                        starts: 0,
                        ends: 0,
                    }));
                    let result = convert(
                        &source,
                        ConversionOptions {
                            tier_strategy,
                            extract_metadata,
                            newline_style,
                            visitor: Some(hooks.clone()),
                            ..Default::default()
                        },
                    )
                    .expect("observed list continuation");
                    assert_eq!(
                        result.content.as_deref(),
                        Some(expected),
                        "{tag} {action:?} {tier_strategy:?}"
                    );
                    let counts = {
                        let guard = hooks.lock().expect("callback count");
                        (guard.starts, guard.ends)
                    };
                    assert_eq!(counts, (1, ends));
                    rows += 1;
                }
            }
        }
        rows
    }
    #[test]
    fn should_keep_empty_block_visitor_output_and_callback_counts() {
        let mut rows = 0;
        for tag in ["h1", "article", "dl"] {
            for action in [Action::Continue, Action::Skip, Action::Custom, Action::Preserve] {
                rows += assert_hook_routes(tag, action);
            }
        }
        assert_eq!(rows, 144);
    }
}
