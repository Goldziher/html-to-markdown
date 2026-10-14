//! Semantic visitor omissions cannot promote strict source line endings to authored breaks. ~keep
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

const TAGS: [&str; 6] = ["s", "del", "strike", "u", "ins", "mark"];

fn markup_options() -> Vec<ConversionOptions> {
    let mut routes = Vec::new();
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for newline_style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
            for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
                routes.push(ConversionOptions {
                    output_format,
                    newline_style,
                    tier_strategy,
                    whitespace_mode: WhitespaceMode::Strict,
                    extract_metadata: false,
                    ..Default::default()
                });
            }
        }
    }
    assert_eq!(routes.len(), 12);
    routes
}

#[test]
fn should_keep_source_separators_without_semantic_visitors() {
    let mut rows = 0;
    for tag in TAGS {
        for mut options in markup_options() {
            for whitespace_mode in [WhitespaceMode::Normalized, WhitespaceMode::Strict] {
                options.whitespace_mode = whitespace_mode;
                let source = format!("<div>a<b>\n<{tag}></{tag}></b>z</div>");
                let result = convert(&source, options.clone()).expect("unobserved semantic separator");
                assert_eq!(result.content.as_deref(), Some("a z\n"), "{source:?}");
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 144);
}

#[cfg(feature = "visitor")]
mod observed {
    use super::*;
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Copy, Debug)]
    enum Action {
        Skip,
        Custom(&'static str),
        Continue,
        PreserveHtml,
        Replay,
    }

    #[derive(Debug)]
    struct Hooks {
        action: Action,
        calls: usize,
        starts: usize,
        ends: usize,
        breaks: usize,
        rows: Vec<Vec<String>>,
    }

    impl Hooks {
        fn is_target(context: &NodeContext) -> bool {
            context.tag_name == "s" || context.attributes().get("id").map(String::as_str) == Some("target")
        }

        fn semantic(&mut self, context: &NodeContext) -> VisitResult {
            if !Self::is_target(context) {
                return VisitResult::Continue;
            }
            self.calls += 1;
            match self.action {
                Action::Skip => VisitResult::Skip,
                Action::Custom(value) => VisitResult::Custom(value.into()),
                Action::PreserveHtml => VisitResult::PreserveHtml,
                Action::Replay if self.calls == 1 => VisitResult::Skip,
                Action::Continue | Action::Replay => VisitResult::Continue,
            }
        }
    }

    impl HtmlVisitor for Hooks {
        fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
            self.starts += usize::from(Self::is_target(context));
            VisitResult::Continue
        }

        fn visit_element_end(&mut self, context: &NodeContext, _: &str) -> VisitResult {
            self.ends += usize::from(Self::is_target(context));
            VisitResult::Continue
        }

        fn visit_strikethrough(&mut self, context: &NodeContext, _: &str) -> VisitResult {
            self.semantic(context)
        }

        fn visit_underline(&mut self, context: &NodeContext, _: &str) -> VisitResult {
            self.semantic(context)
        }

        fn visit_mark(&mut self, context: &NodeContext, _: &str) -> VisitResult {
            self.semantic(context)
        }

        fn visit_line_break(&mut self, _: &NodeContext) -> VisitResult {
            self.breaks += 1;
            VisitResult::Continue
        }

        fn visit_table_row(&mut self, _: &NodeContext, cells: &[String], _: bool) -> VisitResult {
            self.rows.push(cells.to_vec());
            VisitResult::Continue
        }
    }

    fn hooks(action: Action) -> Arc<Mutex<Hooks>> {
        Arc::new(Mutex::new(Hooks {
            action,
            calls: 0,
            starts: 0,
            ends: 0,
            breaks: 0,
            rows: Vec::new(),
        }))
    }

    fn walks_before_callback(tag: &str, output_format: OutputFormat) -> bool {
        matches!(tag, "s" | "del" | "strike") || (tag == "ins" && output_format == OutputFormat::Djot)
    }

    fn assert_case(
        source: &str,
        mut options: ConversionOptions,
        action: Action,
        expected: &str,
        counts: (usize, usize, usize, usize),
    ) {
        let hooks = hooks(action);
        options.visitor = Some(hooks.clone());
        let result = convert(source, options).expect("semantic visitor conversion");
        assert_eq!(result.content.as_deref(), Some(expected), "{source:?} {action:?}");
        let hooks = hooks.lock().expect("semantic callback counts");
        assert_eq!(
            (hooks.calls, hooks.starts, hooks.ends, hooks.breaks),
            counts,
            "{source:?} {action:?}"
        );
        drop(hooks);
    }

    #[test]
    fn should_collapse_the_reviewed_strikethrough_skip_separator() {
        let mut options = markup_options();
        assert_case(
            "<div>a<b>\n<s><br></s></b>z</div>",
            options.remove(0),
            Action::Skip,
            "a z\n",
            (1, 1, 1, 1),
        );
    }

    #[test]
    fn should_collapse_skipped_or_ascii_blank_semantic_break_subtrees_on_all_markup_routes() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<b>\n<{tag} id=\"target\"><br></{tag}></b>z</div>");
            for options in markup_options() {
                let breaks = usize::from(walks_before_callback(tag, options.output_format));
                for action in [
                    Action::Skip,
                    Action::Custom(""),
                    Action::Custom(" "),
                    Action::Custom("\t "),
                ] {
                    assert_case(&source, options.clone(), action, "a z\n", (1, 1, 1, breaks));
                    rows += 1;
                }
            }
        }
        assert_eq!(rows, 288);
    }

    #[test]
    fn should_keep_empty_semantic_source_separators_with_continue_visitors() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<b>\n<{tag} id=\"target\"></{tag}></b>z</div>");
            for mut options in markup_options() {
                for whitespace_mode in [WhitespaceMode::Normalized, WhitespaceMode::Strict] {
                    options.whitespace_mode = whitespace_mode;
                    assert_case(&source, options.clone(), Action::Continue, "a z\n", (1, 1, 1, 0));
                    rows += 1;
                }
            }
        }
        assert_eq!(rows, 144);
    }

    #[test]
    fn should_keep_custom_semantic_content_and_preserved_html() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<{tag} id=\"target\"><br></{tag}>z</div>");
            for options in markup_options().into_iter().filter(|options| {
                options.tier_strategy == TierStrategy::Tier2 && options.newline_style == NewlineStyle::Spaces
            }) {
                let breaks = usize::from(walks_before_callback(tag, options.output_format));
                for (action, expected) in [
                    (Action::Custom("V"), "aVz\n"),
                    (Action::Custom("\n"), "a\nz\n"),
                    (Action::Custom("\u{a0}"), "a\u{a0}z\n"),
                    (Action::Custom("\r"), "a\rz\n"),
                    (Action::Custom("\u{c}"), "a\u{c}z\n"),
                ] {
                    assert_case(&source, options.clone(), action, expected, (1, 1, 1, breaks));
                    rows += 1;
                }
                let expected = format!("a<{tag} id=\"target\"><br /></{tag}>z\n");
                assert_case(&source, options, Action::PreserveHtml, &expected, (1, 1, 1, breaks));
                rows += 1;
            }
        }
        assert_eq!(rows, 72);
    }

    #[test]
    fn should_keep_authored_break_ownership_for_visible_custom_semantic_content() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<b>\n<{tag} id=\"target\"><br></{tag}></b>z</div>");
            for options in markup_options()
                .into_iter()
                .filter(|options| options.tier_strategy == TierStrategy::Tier2)
            {
                let expected = match (options.output_format, options.newline_style) {
                    (OutputFormat::Markdown, NewlineStyle::Spaces) => "a  \n**V**z\n",
                    (OutputFormat::Markdown, NewlineStyle::Backslash) => "a\\\n**V**z\n",
                    _ => "a\\\n*V*z\n",
                };
                let breaks = usize::from(walks_before_callback(tag, options.output_format));
                assert_case(&source, options, Action::Custom("V"), expected, (1, 1, 1, breaks));
                rows += 1;
            }
        }
        assert_eq!(rows, 24);
    }

    #[test]
    fn should_keep_normalized_and_plain_semantic_omission_contracts() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<b>\n<{tag} id=\"target\"><br></{tag}></b>z</div>");
            for (output_format, whitespace_mode) in [
                (OutputFormat::Markdown, WhitespaceMode::Normalized),
                (OutputFormat::Djot, WhitespaceMode::Normalized),
                (OutputFormat::Plain, WhitespaceMode::Normalized),
                (OutputFormat::Plain, WhitespaceMode::Strict),
            ] {
                let plain = output_format == OutputFormat::Plain;
                let options = ConversionOptions {
                    output_format,
                    whitespace_mode,
                    tier_strategy: TierStrategy::Tier2,
                    extract_metadata: false,
                    ..Default::default()
                };
                let breaks = usize::from(walks_before_callback(tag, output_format));
                let expected = if plain { "a\n\nz\n" } else { "a z\n" };
                for action in [
                    Action::Skip,
                    Action::Custom(""),
                    Action::Custom(" "),
                    Action::Custom("\t "),
                ] {
                    let passes = if plain { 2 } else { 1 };
                    assert_case(&source, options.clone(), action, expected, (1, passes, passes, breaks));
                    rows += 1;
                }
            }
        }
        assert_eq!(rows, 96);
    }

    #[test]
    fn should_keep_a_retained_break_sibling_when_a_semantic_subtree_is_omitted() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<b>\n<{tag} id=\"target\"><br></{tag}><br></b>z</div>");
            for options in markup_options()
                .into_iter()
                .filter(|options| options.tier_strategy == TierStrategy::Tier2)
            {
                let expected = if options.output_format == OutputFormat::Markdown
                    && options.newline_style == NewlineStyle::Spaces
                {
                    "a  \nz\n"
                } else {
                    "a\\\nz\n"
                };
                let breaks = 1 + usize::from(walks_before_callback(tag, options.output_format));
                assert_case(&source, options, Action::Skip, expected, (1, 1, 1, breaks));
                rows += 1;
            }
        }
        assert_eq!(rows, 24);
    }

    #[test]
    fn should_restore_semantic_breaks_on_a_later_table_pass() {
        let mut rows = 0;
        for (tag, expected) in [
            ("s", "| H |\n| --- |\n| a**~~<br>~~**z |\n"),
            ("u", "| H |\n| --- |\n| a**<br>**z |\n"),
            ("mark", "| H |\n| --- |\n| a**==<br>==**z |\n"),
        ] {
            let source = format!(
                "<table><tr><th>H</th></tr><tr><td>a<b>\n<{tag} id=\"target\"><br></{tag}></b>z</td></tr></table>"
            );
            let hooks = hooks(Action::Replay);
            let result = convert(
                &source,
                ConversionOptions {
                    tier_strategy: TierStrategy::Tier2,
                    whitespace_mode: WhitespaceMode::Strict,
                    compact_tables: true,
                    br_in_tables: true,
                    extract_metadata: false,
                    visitor: Some(hooks.clone()),
                    ..Default::default()
                },
            )
            .expect("semantic table replay");
            assert_eq!(result.content.as_deref(), Some(expected), "{tag}");
            let hooks = hooks.lock().expect("semantic replay counts");
            assert_eq!((hooks.calls, hooks.starts, hooks.ends), (2, 2, 2), "{tag}");
            assert_eq!(hooks.breaks, if tag == "s" { 2 } else { 1 }, "{tag}");
            assert_eq!(hooks.rows, vec![vec!["H"], vec!["a z"]], "{tag}");
            drop(hooks);
            rows += 1;
        }
        assert_eq!(rows, 3);
    }

    #[test]
    fn should_keep_css_omissions_during_semantic_table_replay() {
        let mut rows = 0;
        for tag in ["s", "u", "mark"] {
            let source = format!(
                "<table><tr><th>H</th></tr><tr><td>a<b>\n<{tag} id=\"target\"><br></{tag}></b>z</td></tr></table>"
            );
            for br_in_tables in [false, true] {
                let hooks = hooks(Action::Continue);
                let result = convert(
                    &source,
                    ConversionOptions {
                        tier_strategy: TierStrategy::Tier2,
                        whitespace_mode: WhitespaceMode::Strict,
                        compact_tables: true,
                        br_in_tables,
                        extract_metadata: false,
                        exclude_selectors: vec!["#target".into()],
                        visitor: Some(hooks.clone()),
                        ..Default::default()
                    },
                )
                .expect("CSS omission during semantic replay");
                assert_eq!(result.content.as_deref(), Some("| H |\n| --- |\n| a z |\n"), "{tag}");
                let hooks = hooks.lock().expect("excluded semantic callback counts");
                assert_eq!(
                    (hooks.calls, hooks.starts, hooks.ends, hooks.breaks),
                    (0, 2, 0, 0),
                    "{tag}"
                );
                assert_eq!(hooks.rows, vec![vec!["H"], vec!["a z"]], "{tag}");
                drop(hooks);
                rows += 1;
            }
        }
        assert_eq!(rows, 6);
    }
}
