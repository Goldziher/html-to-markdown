//! Emphasis Custom omissions preserve strict source separators and retained authored breaks. ~keep
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

const TAGS: [&str; 4] = ["strong", "b", "em", "i"];

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

fn source(tag: &str, body: &str, suffix: &str) -> String {
    let outer = if tag == "b" { "strong" } else { "b" };
    format!("<div>a<{outer}>\n<{tag} id=\"target\">{body}</{tag}>{suffix}</{outer}>z</div>")
}

#[test]
fn should_keep_public_dom_source_separators_without_emphasis_visitors() {
    let mut rows = 0;
    for tag in TAGS {
        for mut options in markup_options() {
            for whitespace_mode in [WhitespaceMode::Normalized, WhitespaceMode::Strict] {
                options.whitespace_mode = whitespace_mode;
                let result = convert(&source(tag, "", ""), options.clone()).expect("public DOM separator");
                assert_eq!(result.content.as_deref(), Some("a z\n"));
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 96);
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
        ReplaySkip,
    }

    #[derive(Debug)]
    struct Hooks {
        tag: &'static str,
        action: Action,
        calls: usize,
        starts: usize,
        ends: usize,
        breaks: usize,
        text: Vec<String>,
        rows: Vec<Vec<String>>,
    }

    impl Hooks {
        fn emphasis(&mut self, context: &NodeContext, text: &str) -> VisitResult {
            if context.tag_name.as_ref() != self.tag {
                return VisitResult::Continue;
            }
            self.calls += 1;
            self.text.push(text.into());
            match self.action {
                Action::Skip => VisitResult::Skip,
                Action::Custom(value) => VisitResult::Custom(value.into()),
                Action::PreserveHtml => VisitResult::PreserveHtml,
                Action::Replay if self.calls == 1 => VisitResult::Custom(String::new()),
                Action::ReplaySkip if self.calls == 1 => VisitResult::Skip,
                Action::Continue | Action::Replay | Action::ReplaySkip => VisitResult::Continue,
            }
        }
    }

    impl HtmlVisitor for Hooks {
        fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
            self.starts += usize::from(context.tag_name.as_ref() == self.tag);
            VisitResult::Continue
        }

        fn visit_element_end(&mut self, context: &NodeContext, _: &str) -> VisitResult {
            self.ends += usize::from(context.tag_name.as_ref() == self.tag);
            VisitResult::Continue
        }

        fn visit_strong(&mut self, context: &NodeContext, text: &str) -> VisitResult {
            self.emphasis(context, text)
        }

        fn visit_emphasis(&mut self, context: &NodeContext, text: &str) -> VisitResult {
            self.emphasis(context, text)
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

    fn hooks(tag: &'static str, action: Action) -> Arc<Mutex<Hooks>> {
        Arc::new(Mutex::new(Hooks {
            tag,
            action,
            calls: 0,
            starts: 0,
            ends: 0,
            breaks: 0,
            text: Vec::new(),
            rows: Vec::new(),
        }))
    }

    fn observe(
        tag: &'static str,
        source: &str,
        mut options: ConversionOptions,
        action: Action,
    ) -> (String, Arc<Mutex<Hooks>>) {
        let hooks = hooks(tag, action);
        options.visitor = Some(hooks.clone());
        let result = convert(source, options).expect("emphasis visitor conversion");
        (result.content.expect("emphasis output"), hooks)
    }

    fn assert_case(
        tag: &'static str,
        source: &str,
        options: ConversionOptions,
        action: Action,
        expected: &str,
        counts: (usize, usize, usize, usize),
    ) {
        let (content, hooks) = observe(tag, source, options, action);
        let hooks = hooks.lock().expect("emphasis callbacks");
        assert_eq!(
            (hooks.calls, hooks.starts, hooks.ends, hooks.breaks),
            counts,
            "{source:?} {action:?}"
        );
        assert_eq!(hooks.text, vec![String::new(); counts.0]);
        drop(hooks);
        assert_eq!(content, expected, "{source:?} {action:?}");
    }

    fn retained_break(options: &ConversionOptions) -> &'static str {
        if options.output_format == OutputFormat::Markdown && options.newline_style == NewlineStyle::Spaces {
            "a  \nz\n"
        } else {
            "a\\\nz\n"
        }
    }

    #[test]
    fn should_collapse_ascii_blank_emphasis_break_subtrees_on_all_markup_routes() {
        let mut rows = 0;
        let mut failures = Vec::new();
        for tag in TAGS {
            for body in ["<br>", "<span><br></span>"] {
                for options in markup_options() {
                    for action in [Action::Custom(""), Action::Custom(" "), Action::Custom("\t ")] {
                        let (content, hooks) = observe(tag, &source(tag, body, ""), options.clone(), action);
                        let hooks = hooks.lock().expect("emphasis omission counts");
                        assert_eq!((hooks.calls, hooks.starts, hooks.ends, hooks.breaks), (1, 1, 1, 1));
                        assert_eq!(hooks.text, vec![String::new()]);
                        drop(hooks);
                        if content != "a z\n" {
                            failures.push(format!("{tag} {body} {options:?} {action:?}: {content:?}"));
                        }
                        rows += 1;
                    }
                }
            }
        }
        assert_eq!(rows, 288);
        assert_eq!(failures.len(), 0, "expected 288 literal a z\\n outputs: {failures:?}");
    }

    #[test]
    fn should_keep_skip_and_continue_emphasis_controls() {
        let mut rows = 0;
        for tag in TAGS {
            for options in markup_options() {
                assert_case(
                    tag,
                    &source(tag, "<br>", ""),
                    options.clone(),
                    Action::Skip,
                    "a z\n",
                    (1, 1, 1, 1),
                );
                assert_case(
                    tag,
                    &source(tag, "", ""),
                    options.clone(),
                    Action::Continue,
                    "a z\n",
                    (1, 1, 1, 0),
                );
                let expected = retained_break(&options);
                assert_case(
                    tag,
                    &source(tag, "<br>", ""),
                    options,
                    Action::Continue,
                    expected,
                    (1, 1, 1, 1),
                );
                rows += 3;
            }
        }
        assert_eq!(rows, 144);
    }

    #[test]
    fn should_keep_nonhorizontal_custom_content_and_preserved_emphasis_html() {
        let mut rows = 0;
        for tag in TAGS {
            let input = format!("<div>a<{tag} id=\"target\"><br></{tag}>z</div>");
            for options in markup_options().into_iter().filter(|options| {
                options.tier_strategy == TierStrategy::Tier2 && options.newline_style == NewlineStyle::Spaces
            }) {
                for (custom, expected) in [
                    ("V", "aVz\n"),
                    ("\n", "a\nz\n"),
                    ("\u{a0}", "a\u{a0}z\n"),
                    ("\r", "a\rz\n"),
                    ("\u{c}", "a\u{c}z\n"),
                ] {
                    assert_case(
                        tag,
                        &input,
                        options.clone(),
                        Action::Custom(custom),
                        expected,
                        (1, 1, 1, 1),
                    );
                    rows += 1;
                }
                let expected = format!("a<{tag} id=\"target\"><br /></{tag}>z\n");
                assert_case(tag, &input, options, Action::PreserveHtml, &expected, (1, 1, 1, 1));
                rows += 1;
            }
        }
        assert_eq!(rows, 48);
    }

    #[test]
    fn should_keep_authored_break_ownership_for_visible_custom_emphasis() {
        let mut rows = 0;
        for tag in TAGS {
            for options in markup_options()
                .into_iter()
                .filter(|options| options.tier_strategy == TierStrategy::Tier2)
            {
                let expected = match (options.output_format, options.newline_style) {
                    (OutputFormat::Markdown, NewlineStyle::Spaces) => "a  \n**V**z\n",
                    (OutputFormat::Markdown, NewlineStyle::Backslash) => "a\\\n**V**z\n",
                    _ => "a\\\n*V*z\n",
                };
                assert_case(
                    tag,
                    &source(tag, "<br>", ""),
                    options,
                    Action::Custom("V"),
                    expected,
                    (1, 1, 1, 1),
                );
                rows += 1;
            }
        }
        assert_eq!(rows, 16);
    }

    #[test]
    fn should_keep_normalized_and_plain_emphasis_omission_contracts() {
        let mut rows = 0;
        for tag in TAGS {
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
                    let passes = if plain { 2 } else { 1 };
                    let expected = if plain { "a\n\nz\n" } else { "a z\n" };
                    assert_case(
                        tag,
                        &source(tag, "<br>", ""),
                        options.clone(),
                        action,
                        expected,
                        (1, passes, passes, 1),
                    );
                    rows += 1;
                }
            }
        }
        assert_eq!(rows, 64);
    }

    #[test]
    fn should_keep_retained_break_siblings_after_emphasis_custom_omissions() {
        let mut rows = 0;
        for tag in TAGS {
            for options in markup_options() {
                let expected = retained_break(&options);
                for action in [
                    Action::Skip,
                    Action::Custom(""),
                    Action::Custom(" "),
                    Action::Custom("\t "),
                ] {
                    assert_case(
                        tag,
                        &source(tag, "<br>", "<br>"),
                        options.clone(),
                        action,
                        expected,
                        (1, 1, 1, 2),
                    );
                    rows += 1;
                }
            }
        }
        assert_eq!(rows, 192);
    }

    fn table_source(tag: &str) -> String {
        let cell = source(tag, "<br>", "").replace("<div>", "").replace("</div>", "");
        format!("<table><tr><th>H</th></tr><tr><td>{cell}</td></tr></table>")
    }

    fn table_options(br_in_tables: bool) -> ConversionOptions {
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            whitespace_mode: WhitespaceMode::Strict,
            compact_tables: true,
            br_in_tables,
            extract_metadata: false,
            ..Default::default()
        }
    }

    #[test]
    fn should_omit_empty_custom_emphasis_breaks_from_table_row_payloads() {
        let mut rows = 0;
        for tag in TAGS {
            let (content, hooks) = observe(tag, &table_source(tag), table_options(false), Action::Custom(""));
            let hooks = hooks.lock().expect("emphasis table omission");
            assert_eq!((hooks.calls, hooks.starts, hooks.ends, hooks.breaks), (2, 2, 2, 2));
            assert_eq!(hooks.text, vec![String::new(); 2]);
            assert_eq!(hooks.rows, vec![vec!["H"], vec!["a z"]]);
            drop(hooks);
            assert_eq!(content, "| H |\n| --- |\n| a z |\n");
            rows += 1;
        }
        assert_eq!(rows, 4);
    }

    #[test]
    fn should_restore_emphasis_breaks_on_a_later_table_pass() {
        let mut rows = 0;
        for tag in TAGS {
            for br_in_tables in [false, true] {
                let (content, hooks) = observe(tag, &table_source(tag), table_options(br_in_tables), Action::Replay);
                let expected = match (br_in_tables, tag) {
                    (true, "em" | "i") => "| H |\n| --- |\n| a***<br>***z |\n",
                    (true, _) => "| H |\n| --- |\n| a**<br>**z |\n",
                    (false, _) => "| H |\n| --- |\n| a   z |\n",
                };
                assert_eq!(content, expected, "{tag} {br_in_tables}");
                let hooks = hooks.lock().expect("emphasis table replay");
                assert_eq!((hooks.calls, hooks.starts, hooks.ends, hooks.breaks), (2, 2, 2, 2));
                assert_eq!(hooks.text, vec![String::new(); 2]);
                assert_eq!(hooks.rows, vec![vec!["H"], vec!["a z"]]);
                drop(hooks);
                rows += 1;
            }
        }
        assert_eq!(rows, 8);
    }

    #[test]
    fn should_keep_skipped_emphasis_breaks_on_a_later_table_pass() {
        let mut rows = 0;
        for tag in TAGS {
            for br_in_tables in [false, true] {
                let (content, hooks) =
                    observe(tag, &table_source(tag), table_options(br_in_tables), Action::ReplaySkip);
                let expected = match (br_in_tables, tag) {
                    (true, "em" | "i") => "| H |\n| --- |\n| a***<br>***z |\n",
                    (true, _) => "| H |\n| --- |\n| a**<br>**z |\n",
                    (false, _) => "| H |\n| --- |\n| a   z |\n",
                };
                assert_eq!(content, expected, "{tag} {br_in_tables}");
                let hooks = hooks.lock().expect("emphasis table replay");
                assert_eq!((hooks.calls, hooks.starts, hooks.ends, hooks.breaks), (2, 2, 2, 2));
                assert_eq!(hooks.text, vec![String::new(); 2]);
                assert_eq!(hooks.rows, vec![vec!["H"], vec!["a z"]]);
                drop(hooks);
                rows += 1;
            }
        }
        assert_eq!(rows, 8);
    }

    #[test]
    fn should_keep_css_omissions_during_emphasis_table_replay() {
        let mut rows = 0;
        for tag in TAGS {
            for br_in_tables in [false, true] {
                let mut options = table_options(br_in_tables);
                options.exclude_selectors = vec!["#target".into()];
                let (content, hooks) = observe(tag, &table_source(tag), options, Action::Continue);
                assert_eq!(content, "| H |\n| --- |\n| a z |\n");
                let hooks = hooks.lock().expect("CSS-excluded emphasis callbacks");
                assert_eq!((hooks.calls, hooks.starts, hooks.ends, hooks.breaks), (0, 2, 0, 0));
                assert_eq!(hooks.text, Vec::<String>::new());
                assert_eq!(hooks.rows, vec![vec!["H"], vec!["a z"]]);
                drop(hooks);
                rows += 1;
            }
        }
        assert_eq!(rows, 8);
    }
}
