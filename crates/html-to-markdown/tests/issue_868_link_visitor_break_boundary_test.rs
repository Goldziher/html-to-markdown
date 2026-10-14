//! Link visitor omissions cannot promote strict source line endings to authored breaks. ~keep
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

const TAGS: [&str; 1] = ["a"];

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
fn should_keep_authored_link_breaks_without_visitors() {
    let mut rows = 0;
    for options in markup_options() {
        let expected = match (options.output_format, options.newline_style) {
            (OutputFormat::Markdown, NewlineStyle::Spaces) => "a  \n**[u](u)**z\n",
            (OutputFormat::Markdown, NewlineStyle::Backslash) => "a\\\n**[u](u)**z\n",
            _ => "a\\\n*[u](u)*z\n",
        };
        let result = convert("<div>a<b>\n<a href=\"u\"><br></a></b>z</div>", options).expect("unobserved link break");
        assert_eq!(result.content.as_deref(), Some(expected));
        rows += 1;
    }
    assert_eq!(rows, 12);
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
            context.tag_name == "a"
        }

        fn link(&mut self, context: &NodeContext) -> VisitResult {
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

        fn visit_link(&mut self, context: &NodeContext, _: &str, _: &str, _: Option<&str>) -> VisitResult {
            self.link(context)
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

    fn assert_case(
        source: &str,
        mut options: ConversionOptions,
        action: Action,
        expected: &str,
        counts: (usize, usize, usize, usize),
    ) {
        let hooks = hooks(action);
        options.visitor = Some(hooks.clone());
        let result = convert(source, options).expect("link visitor conversion");
        let hooks = hooks.lock().expect("link callback counts");
        let actual_counts = (hooks.calls, hooks.starts, hooks.ends, hooks.breaks);
        assert_eq!(actual_counts, counts, "{source:?} {action:?}");
        drop(hooks);
        assert_eq!(
            result.content.as_deref(),
            Some(expected),
            "{source:?} {action:?} callbacks={actual_counts:?}"
        );
    }

    #[test]
    fn should_collapse_link_skip_separator() {
        let mut options = markup_options();
        assert_case(
            "<div>a<b>\n<a href=\"u\"><br></a></b>z</div>",
            options.remove(0),
            Action::Skip,
            "a z\n",
            (1, 1, 1, 1),
        );
    }

    #[test]
    fn should_collapse_link_ascii_empty_separator() {
        let mut options = markup_options();
        assert_case(
            "<div>a<b>\n<a href=\"u\"><br></a></b>z</div>",
            options.remove(0),
            Action::Custom(""),
            "a z\n",
            (1, 1, 1, 1),
        );
    }

    #[test]
    fn should_collapse_skipped_or_ascii_blank_link_break_subtrees_on_all_markup_routes() {
        let mut rows = 0;
        let mut failures = Vec::new();
        for tag in TAGS {
            let source = format!("<div>a<b>\n<{tag} href=\"u\" id=\"target\"><br></{tag}></b>z</div>");
            for options in markup_options() {
                let breaks = 1;
                for action in [
                    Action::Skip,
                    Action::Custom(""),
                    Action::Custom(" "),
                    Action::Custom("\t "),
                ] {
                    let hooks = hooks(action);
                    let mut observed_options = options.clone();
                    observed_options.visitor = Some(hooks.clone());
                    let result = convert(&source, observed_options).expect("link omission route");
                    let hooks = hooks.lock().expect("link omission route counts");
                    let counts = (hooks.calls, hooks.starts, hooks.ends, hooks.breaks);
                    assert_eq!(counts, (1, 1, 1, breaks));
                    drop(hooks);
                    if result.content.as_deref() != Some("a z\n") {
                        failures.push(format!(
                            "{tag} {:?} {:?} {:?} {action:?} callbacks={counts:?}: {:?}",
                            options.output_format, options.newline_style, options.tier_strategy, result.content
                        ));
                    }
                    rows += 1;
                }
            }
        }
        assert_eq!(rows, 48);
        assert_eq!(failures, Vec::<String>::new(), "expected 48 literal a z\\n outputs");
    }

    #[test]
    fn should_keep_authored_breaks_with_continue_link_visitors() {
        let mut rows = 0;
        for options in markup_options() {
            let expected = match (options.output_format, options.newline_style) {
                (OutputFormat::Markdown, NewlineStyle::Spaces) => "a  \n**[u](u)**z\n",
                (OutputFormat::Markdown, NewlineStyle::Backslash) => "a\\\n**[u](u)**z\n",
                _ => "a\\\n*[u](u)*z\n",
            };
            assert_case(
                "<div>a<b>\n<a href=\"u\"><br></a></b>z</div>",
                options,
                Action::Continue,
                expected,
                (1, 1, 1, 1),
            );
            rows += 1;
        }
        assert_eq!(rows, 12);
    }

    #[test]
    fn should_keep_custom_link_content_and_preserved_html() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<{tag} href=\"u\" id=\"target\"><br></{tag}>z</div>");
            for options in markup_options().into_iter().filter(|options| {
                options.tier_strategy == TierStrategy::Tier2 && options.newline_style == NewlineStyle::Spaces
            }) {
                let breaks = 1;
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
                let expected = format!("a<{tag} href=\"u\" id=\"target\"><br /></{tag}>z\n");
                assert_case(&source, options, Action::PreserveHtml, &expected, (1, 1, 1, breaks));
                rows += 1;
            }
        }
        assert_eq!(rows, 12);
    }

    #[test]
    fn should_keep_authored_break_ownership_for_visible_custom_link_content() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<b>\n<{tag} href=\"u\" id=\"target\"><br></{tag}></b>z</div>");
            for options in markup_options()
                .into_iter()
                .filter(|options| options.tier_strategy == TierStrategy::Tier2)
            {
                let expected = match (options.output_format, options.newline_style) {
                    (OutputFormat::Markdown, NewlineStyle::Spaces) => "a  \n**V**z\n",
                    (OutputFormat::Markdown, NewlineStyle::Backslash) => "a\\\n**V**z\n",
                    _ => "a\\\n*V*z\n",
                };
                let breaks = 1;
                assert_case(&source, options, Action::Custom("V"), expected, (1, 1, 1, breaks));
                rows += 1;
            }
        }
        assert_eq!(rows, 4);
    }

    #[test]
    fn should_keep_normalized_and_plain_link_omission_contracts() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<b>\n<{tag} href=\"u\" id=\"target\"><br></{tag}></b>z</div>");
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
                let breaks = 1;
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
        assert_eq!(rows, 16);
    }

    #[test]
    fn should_keep_a_retained_break_sibling_when_a_link_subtree_is_omitted() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!("<div>a<b>\n<{tag} href=\"u\" id=\"target\"><br></{tag}><br></b>z</div>");
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
                let breaks = 2;
                assert_case(&source, options, Action::Skip, expected, (1, 1, 1, breaks));
                rows += 1;
            }
        }
        assert_eq!(rows, 4);
    }

    #[test]
    fn should_restore_link_breaks_on_a_later_table_pass() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!(
                "<table><tr><th>H</th></tr><tr><td>a<b>\n<{tag} href=\"u\" id=\"target\"><br></{tag}></b>z</td></tr></table>"
            );
            for br_in_tables in [false, true] {
                let hooks = hooks(Action::Replay);
                let result = convert(
                    &source,
                    ConversionOptions {
                        tier_strategy: TierStrategy::Tier2,
                        whitespace_mode: WhitespaceMode::Strict,
                        compact_tables: true,
                        br_in_tables,
                        extract_metadata: false,
                        visitor: Some(hooks.clone()),
                        ..Default::default()
                    },
                )
                .expect("link table replay");
                let expected = if br_in_tables {
                    "| H |\n| --- |\n| a**[<br>](u)**z |\n"
                } else {
                    "| H |\n| --- |\n| a**[u](u)**z |\n"
                };
                assert_eq!(result.content.as_deref(), Some(expected), "{tag} {br_in_tables}");
                let hooks = hooks.lock().expect("link replay counts");
                assert_eq!(
                    (hooks.calls, hooks.starts, hooks.ends, hooks.breaks),
                    (2, 2, 2, 2),
                    "{tag}"
                );
                assert_eq!(hooks.rows, vec![vec!["H"], vec!["a z"]], "{tag}");
                drop(hooks);
                rows += 1;
            }
        }
        assert_eq!(rows, 2);
    }

    #[test]
    fn should_keep_css_omissions_during_link_table_replay() {
        let mut rows = 0;
        for tag in TAGS {
            let source = format!(
                "<table><tr><th>H</th></tr><tr><td>a<b>\n<{tag} href=\"u\" id=\"target\"><br></{tag}></b>z</td></tr></table>"
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
                .expect("CSS omission during link replay");
                assert_eq!(result.content.as_deref(), Some("| H |\n| --- |\n| a z |\n"), "{tag}");
                let hooks = hooks.lock().expect("excluded link callback counts");
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
        assert_eq!(rows, 2);
    }
    #[test]
    fn should_keep_deferred_link_table_breaks_with_blank_custom_content() {
        let source = "<div>a<b>\n<a href=\"u\"><br><table><tr><th>H</th></tr><tr><td>x<b>\n<br></b>y</td></tr></table></a></b>z</div>";
        let mut rows = 0;
        for br_in_tables in [false, true] {
            let cell = if br_in_tables { "x<br>y" } else { "x y" };
            let expected = format!("a  \n\\\n**| H |\n| --- |\n| {cell} |**z\n");
            let hooks = hooks(Action::Custom(""));
            let result = convert(
                source,
                ConversionOptions {
                    tier_strategy: TierStrategy::Tier2,
                    whitespace_mode: WhitespaceMode::Strict,
                    compact_tables: true,
                    br_in_tables,
                    extract_metadata: false,
                    visitor: Some(hooks.clone()),
                    ..Default::default()
                },
            )
            .expect("deferred link table");
            assert_eq!(result.content.as_deref(), Some(expected.as_str()));
            let hooks = hooks.lock().expect("deferred link counts");
            assert_eq!((hooks.calls, hooks.starts, hooks.ends, hooks.breaks), (1, 1, 1, 3));
            assert_eq!(hooks.rows, vec![vec!["H"], vec![cell]]);
            drop(hooks);
            rows += 1;
        }
        assert_eq!(rows, 2);
    }
    #[test]
    fn should_collapse_link_omissions_in_later_paragraphs() {
        let mut rows = 0;
        for options in markup_options()
            .into_iter()
            .filter(|options| options.tier_strategy == TierStrategy::Tier2)
        {
            for action in [
                Action::Skip,
                Action::Custom(""),
                Action::Custom(" "),
                Action::Custom("\t "),
            ] {
                assert_case(
                    "<p>x</p><p>a<b>\n<a href=\"u\"><br></a></b>z</p>",
                    options.clone(),
                    action,
                    "x\n\na z\n",
                    (1, 1, 1, 1),
                );
                rows += 1;
            }
        }
        assert_eq!(rows, 16);
    }

    #[test]
    fn should_keep_wrapped_link_replacements_that_are_not_ascii_blank() {
        let mut rows = 0;
        for options in markup_options()
            .into_iter()
            .filter(|options| options.tier_strategy == TierStrategy::Tier2)
        {
            for (action, expected) in [
                (Action::Custom("\n"), "a z\n"),
                (Action::Custom("\u{a0}"), "a z\n"),
                (Action::Custom("\u{c}"), "a z\n"),
            ] {
                assert_case(
                    "<div>a<b>\n<a href=\"u\"><br></a></b>z</div>",
                    options.clone(),
                    action,
                    expected,
                    (1, 1, 1, 1),
                );
                rows += 1;
            }
            let expected =
                if options.output_format == OutputFormat::Markdown && options.newline_style == NewlineStyle::Spaces {
                    "a  \nz\n"
                } else {
                    "a\\\nz\n"
                };
            assert_case(
                "<div>a<b>\n<a href=\"u\"><br></a></b>z</div>",
                options,
                Action::Custom("\r"),
                expected,
                (1, 1, 1, 1),
            );
            rows += 1;
        }
        assert_eq!(rows, 16);
    }

    #[test]
    fn should_keep_source_breaks_with_blank_custom_deferred_layout_tables() {
        assert_case(
            "<div>a<b>\n<a href=\"u\"><table role=\"presentation\"><tr><td><br></td></tr></table></a></b>z</div>",
            ConversionOptions {
                whitespace_mode: WhitespaceMode::Strict,
                tier_strategy: TierStrategy::Tier2,
                extract_metadata: false,
                ..Default::default()
            },
            Action::Custom(""),
            "a  \n\nz\n",
            (1, 1, 1, 0),
        );
    }
}
