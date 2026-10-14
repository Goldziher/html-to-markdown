//! Inline-code visitor omissions keep strict source separators distinct from authored breaks. ~keep
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

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
fn should_keep_empty_code_source_separators_without_visitors() {
    let mut rows = 0;
    for mut options in markup_options() {
        for whitespace_mode in [WhitespaceMode::Normalized, WhitespaceMode::Strict] {
            options.whitespace_mode = whitespace_mode;
            let result = convert("<div>a<b>\n<code></code></b>z</div>", options.clone()).expect("empty code");
            assert_eq!(result.content.as_deref(), Some("a z\n"));
            rows += 1;
        }
    }
    assert_eq!(rows, 24);
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
        inline: usize,
        block: usize,
        starts: usize,
        ends: usize,
        breaks: usize,
        code_text: Vec<String>,
        rows: Vec<Vec<String>>,
    }

    impl Hooks {
        fn code(&mut self, content: &str) -> VisitResult {
            self.code_text.push(content.into());
            match self.action {
                Action::Skip => VisitResult::Skip,
                Action::Custom(value) => VisitResult::Custom(value.into()),
                Action::PreserveHtml => VisitResult::PreserveHtml,
                Action::Replay if self.inline == 1 => VisitResult::Skip,
                Action::Continue | Action::Replay => VisitResult::Continue,
            }
        }
    }

    impl HtmlVisitor for Hooks {
        fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
            self.starts += usize::from(matches!(context.tag_name.as_ref(), "code" | "pre"));
            VisitResult::Continue
        }

        fn visit_element_end(&mut self, context: &NodeContext, _: &str) -> VisitResult {
            self.ends += usize::from(matches!(context.tag_name.as_ref(), "code" | "pre"));
            VisitResult::Continue
        }

        fn visit_code_inline(&mut self, _: &NodeContext, content: &str) -> VisitResult {
            self.inline += 1;
            self.code(content)
        }

        fn visit_code_block(&mut self, _: &NodeContext, _: Option<&str>, content: &str) -> VisitResult {
            self.block += 1;
            self.code(content)
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
            inline: 0,
            block: 0,
            starts: 0,
            ends: 0,
            breaks: 0,
            code_text: Vec::new(),
            rows: Vec::new(),
        }))
    }

    fn assert_case(
        source: &str,
        mut options: ConversionOptions,
        action: Action,
        expected: &str,
        counts: (usize, usize, usize, usize, usize),
        code_text: &[&str],
    ) {
        let hooks = hooks(action);
        options.visitor = Some(hooks.clone());
        let result = convert(source, options).expect("code visitor conversion");
        assert_eq!(result.content.as_deref(), Some(expected), "{source:?} {action:?}");
        let hooks = hooks.lock().expect("code callback counts");
        assert_eq!(
            (hooks.inline, hooks.block, hooks.breaks, hooks.starts, hooks.ends),
            counts
        );
        assert_eq!(hooks.code_text, code_text);
        drop(hooks);
    }

    fn forced_options() -> ConversionOptions {
        markup_options().pop().expect("forced code options")
    }

    #[test]
    fn should_collapse_the_reviewed_inline_code_skip_separator() {
        assert_case(
            "<div>a<b>\n<code><br></code></b>z</div>",
            forced_options(),
            Action::Skip,
            "a z\n",
            (1, 0, 1, 1, 1),
            &["\n"],
        );
    }

    #[test]
    fn should_collapse_omitted_inline_code_breaks_on_all_markup_routes() {
        let mut rows = 0;
        for body in ["<br>", "<span><br></span>"] {
            let source = format!("<div>a<b>\n<code>{body}</code></b>z</div>");
            for options in markup_options() {
                for action in [
                    Action::Skip,
                    Action::Custom(""),
                    Action::Custom(" "),
                    Action::Custom("\t "),
                ] {
                    assert_case(&source, options.clone(), action, "a z\n", (1, 0, 1, 1, 1), &["\n"]);
                    rows += 1;
                }
            }
        }
        assert_eq!(rows, 96);
    }

    #[test]
    fn should_keep_empty_code_callbacks_and_retained_inline_breaks() {
        let mut rows = 0;
        for options in markup_options() {
            assert_case(
                "<div>a<b>\n<code></code></b>z</div>",
                options.clone(),
                Action::Continue,
                "a z\n",
                (0, 0, 0, 1, 1),
                &[],
            );
            let expected =
                if options.output_format == OutputFormat::Markdown && options.newline_style == NewlineStyle::Spaces {
                    "a  \nz\n"
                } else {
                    "a\\\nz\n"
                };
            assert_case(
                "<div>a<b>\n<code><br></code></b>z</div>",
                options,
                Action::Continue,
                expected,
                (1, 0, 1, 1, 1),
                &["\n"],
            );
            rows += 2;
        }
        assert_eq!(rows, 24);
    }

    #[test]
    fn should_keep_literal_inline_code_and_visible_custom_outputs() {
        let mut rows = 0;
        for options in markup_options()
            .into_iter()
            .filter(|options| options.tier_strategy == TierStrategy::Tier2)
        {
            for (action, expected) in [
                (Action::Continue, "a` x  y `z\n"),
                (Action::Custom("V"), "aVz\n"),
                (Action::Custom("\n"), "a\nz\n"),
                (Action::Custom("\u{a0}"), "a\u{a0}z\n"),
                (Action::Custom("\r"), "a\rz\n"),
                (Action::Custom("\u{c}"), "a\u{c}z\n"),
                (Action::PreserveHtml, "a<code> x\n y </code>z\n"),
            ] {
                assert_case(
                    "<div>a<code> x\n y </code>z</div>",
                    options.clone(),
                    action,
                    expected,
                    (1, 0, 0, 1, 1),
                    &[" x  y "],
                );
                rows += 1;
            }
        }
        assert_eq!(rows, 28);
    }

    #[test]
    fn should_keep_authored_code_split_markers_with_continue() {
        let mut rows = 0;
        for options in markup_options() {
            let expected =
                if options.output_format == OutputFormat::Markdown && options.newline_style == NewlineStyle::Spaces {
                    "a`x`  \n`y`z\n"
                } else if options.output_format == OutputFormat::Markdown {
                    "a`x`\\\n`y`z\n"
                } else {
                    "a`x`\\\n\\`y`z\n"
                };
            let fast = options.tier_strategy == TierStrategy::Tier1;
            assert_case(
                "<div>a<code>x<br>y</code>z</div>",
                options,
                Action::Continue,
                expected,
                if fast { (0, 0, 0, 0, 0) } else { (1, 0, 1, 1, 1) },
                if fast { &[] } else { &["x\ny"] },
            );
            rows += 1;
        }
        assert_eq!(rows, 12);
    }

    #[test]
    fn should_keep_nonblank_code_replacements_and_preserved_break_html() {
        let mut rows = 0;
        for options in markup_options()
            .into_iter()
            .filter(|options| options.tier_strategy == TierStrategy::Tier2)
        {
            for (action, expected) in [
                (Action::Custom("V"), "aVz\n"),
                (Action::Custom("\n"), "a\nz\n"),
                (Action::Custom("\u{a0}"), "a\u{a0}z\n"),
                (Action::Custom("\r"), "a\rz\n"),
                (Action::Custom("\u{c}"), "a\u{c}z\n"),
                (Action::PreserveHtml, "a<code><br /></code>z\n"),
            ] {
                assert_case(
                    "<div>a<code><br></code>z</div>",
                    options.clone(),
                    action,
                    expected,
                    (1, 0, 1, 1, 1),
                    &["\n"],
                );
                rows += 1;
            }
        }
        assert_eq!(rows, 24);
    }

    #[test]
    fn should_keep_normalized_and_plain_code_omission_contracts() {
        let mut rows = 0;
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
            for action in [
                Action::Skip,
                Action::Custom(""),
                Action::Custom(" "),
                Action::Custom("\t "),
            ] {
                assert_case(
                    "<div>a<b>\n<code><br></code></b>z</div>",
                    options.clone(),
                    action,
                    if plain { "a\n\nz\n" } else { "a z\n" },
                    (1, 0, 1, if plain { 2 } else { 1 }, if plain { 2 } else { 1 }),
                    &["\n"],
                );
                rows += 1;
            }
        }
        assert_eq!(rows, 16);
    }

    #[test]
    fn should_keep_a_retained_break_sibling_of_omitted_code() {
        let mut rows = 0;
        for options in markup_options() {
            let expected =
                if options.output_format == OutputFormat::Markdown && options.newline_style == NewlineStyle::Spaces {
                    "a  \nz\n"
                } else {
                    "a\\\nz\n"
                };
            assert_case(
                "<div>a<b>\n<code><br></code><br></b>z</div>",
                options,
                Action::Skip,
                expected,
                (1, 0, 2, 1, 1),
                &["\n"],
            );
            rows += 1;
        }
        assert_eq!(rows, 12);
    }

    #[test]
    fn should_restore_code_breaks_on_a_later_table_pass() {
        let mut rows = 0;
        for br_in_tables in [false, true] {
            let hooks = hooks(Action::Replay);
            let result = convert(
                "<table><tr><th>H</th></tr><tr><td>a<b>\n<code><br></code></b>z</td></tr></table>",
                ConversionOptions {
                    compact_tables: true,
                    br_in_tables,
                    visitor: Some(hooks.clone()),
                    output_format: OutputFormat::Markdown,
                    ..forced_options()
                },
            )
            .expect("code table replay");
            assert_eq!(result.content.as_deref(), Some("| H |\n| --- |\n| a**` `**z |\n"));
            let hooks = hooks.lock().expect("replayed code counts");
            assert_eq!(
                (hooks.inline, hooks.block, hooks.breaks, hooks.starts, hooks.ends),
                (2, 0, 2, 2, 2)
            );
            assert_eq!(hooks.rows, vec![vec!["H"], vec!["a z"]]);
            assert_eq!(hooks.code_text, [" ", " "]);
            drop(hooks);
            rows += 1;
        }
        assert_eq!(rows, 2);
    }

    #[test]
    fn should_keep_css_code_omissions_during_table_replay() {
        let mut rows = 0;
        for br_in_tables in [false, true] {
            let hooks = hooks(Action::Continue);
            let result = convert(
                "<table><tr><th>H</th></tr><tr><td>a<b>\n<code id=target><br></code></b>z</td></tr></table>",
                ConversionOptions {
                    compact_tables: true,
                    br_in_tables,
                    visitor: Some(hooks.clone()),
                    exclude_selectors: vec!["#target".into()],
                    output_format: OutputFormat::Markdown,
                    ..forced_options()
                },
            )
            .expect("excluded code");
            assert_eq!(result.content.as_deref(), Some("| H |\n| --- |\n| a z |\n"));
            let hooks = hooks.lock().expect("excluded code counts");
            assert_eq!(
                (hooks.inline, hooks.block, hooks.breaks, hooks.starts, hooks.ends),
                (0, 0, 0, 2, 0)
            );
            assert_eq!(hooks.rows, vec![vec!["H"], vec!["a z"]]);
            assert_eq!(hooks.code_text, Vec::<String>::new());
            drop(hooks);
            rows += 1;
        }
        assert_eq!(rows, 2);
    }

    #[test]
    fn should_keep_preformatted_code_and_block_callback_contracts() {
        let mut rows = 0;
        for (action, expected) in [
            (Action::Continue, "```\n x\n y \n```\n"),
            (Action::Skip, ""),
            (Action::Custom(""), ""),
            (Action::Custom(" "), ""),
            (Action::Custom("\t "), ""),
            (Action::Custom("V"), "V\n"),
            (Action::Custom("\n"), ""),
            (Action::Custom("\u{a0}"), "\u{a0}\n"),
            (Action::PreserveHtml, "<pre><code> x\n y </code></pre>\n"),
        ] {
            let mut options = forced_options();
            options.output_format = OutputFormat::Markdown;
            assert_case(
                "<pre><code> x\n y </code></pre>",
                options,
                action,
                expected,
                (0, 1, 0, 2, 2),
                &[" x\n y "],
            );
            rows += 1;
        }
        assert_eq!(rows, 9);
    }
}
