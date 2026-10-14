//! Generic visitor omissions distinguish source separators from retained authored breaks. ~keep
#![cfg(all(feature = "testkit", feature = "visitor"))]

use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug)]
enum Action {
    Continue,
    Skip,
    Custom(&'static str),
    PreserveHtml,
    ReplaySkip,
    ReplayBlank,
    ReplaySkipVisible,
    ReplayBlankVisible,
}

impl Action {
    fn result(self, call: usize) -> VisitResult {
        match self {
            Self::Skip => VisitResult::Skip,
            Self::Custom(value) => VisitResult::Custom(value.into()),
            Self::PreserveHtml => VisitResult::PreserveHtml,
            Self::ReplaySkip | Self::ReplaySkipVisible if call == 1 => VisitResult::Skip,
            Self::ReplayBlank | Self::ReplayBlankVisible if call == 1 => VisitResult::Custom(String::new()),
            Self::ReplaySkipVisible | Self::ReplayBlankVisible => VisitResult::Custom("V".into()),
            Self::Continue | Self::ReplaySkip | Self::ReplayBlank => VisitResult::Continue,
        }
    }
}

#[derive(Debug)]
struct Hooks {
    start: Action,
    end: Action,
    outer_end: Action,
    typed: Action,
    typed_calls: usize,
    starts: usize,
    ends: usize,
    breaks: usize,
    rows: Vec<Vec<String>>,
    end_outputs: Vec<String>,
}

impl Hooks {
    fn typed(&mut self, context: &NodeContext) -> VisitResult {
        if context.attributes().get("id").map(String::as_str) == Some("omit") {
            self.typed_calls += 1;
            self.typed.result(self.typed_calls)
        } else {
            VisitResult::Continue
        }
    }
}

impl HtmlVisitor for Hooks {
    fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
        if context.attributes().get("id").map(String::as_str) == Some("omit") {
            self.starts += 1;
            self.start.result(self.starts)
        } else {
            VisitResult::Continue
        }
    }

    fn visit_element_end(&mut self, context: &NodeContext, output: &str) -> VisitResult {
        match context.attributes().get("id").map(String::as_str) {
            Some("omit") => {
                self.ends += 1;
                self.end_outputs.push(output.into());
                self.end.result(self.ends)
            }
            Some("outer") => self.outer_end.result(1),
            _ => VisitResult::Continue,
        }
    }

    fn visit_emphasis(&mut self, context: &NodeContext, _: &str) -> VisitResult {
        self.typed(context)
    }

    fn visit_code_inline(&mut self, context: &NodeContext, _: &str) -> VisitResult {
        self.typed(context)
    }

    fn visit_link(&mut self, context: &NodeContext, _: &str, _: &str, _: Option<&str>) -> VisitResult {
        self.typed(context)
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

struct Observed {
    content: String,
    typed_calls: usize,
    counts: (usize, usize, usize),
    rows: Vec<Vec<String>>,
    end_outputs: Vec<String>,
}

fn observe(source: &str, options: ConversionOptions, start: Action, end: Action, outer_end: Action) -> Observed {
    observe_typed(source, options, start, end, outer_end, Action::Continue)
}

fn observe_typed(
    source: &str,
    mut options: ConversionOptions,
    start: Action,
    end: Action,
    outer_end: Action,
    typed: Action,
) -> Observed {
    let hooks = Arc::new(Mutex::new(Hooks {
        start,
        end,
        outer_end,
        typed,
        typed_calls: 0,
        starts: 0,
        ends: 0,
        breaks: 0,
        rows: Vec::new(),
        end_outputs: Vec::new(),
    }));
    options.visitor = Some(hooks.clone());
    let result = convert(source, options).expect("generic visitor conversion");
    let hooks = hooks.lock().expect("generic callback evidence");
    Observed {
        typed_calls: hooks.typed_calls,
        content: result.content.expect("rendered generic content"),
        counts: (hooks.starts, hooks.ends, hooks.breaks),
        rows: hooks.rows.clone(),
        end_outputs: hooks.end_outputs.clone(),
    }
}

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

fn hard_break_output(options: &ConversionOptions) -> &'static str {
    if options.output_format == OutputFormat::Markdown && options.newline_style == NewlineStyle::Spaces {
        "a  \nz\n"
    } else {
        "a\\\nz\n"
    }
}

#[test]
fn should_collapse_generic_start_blank_custom_source_separators() {
    let mut rows = 0;
    for options in markup_options() {
        for action in [Action::Custom(""), Action::Custom(" "), Action::Custom("\t ")] {
            let observed = observe(
                "<div>a<b>\n<span id=omit><br></span></b>z</div>",
                options.clone(),
                action,
                Action::Continue,
                Action::Continue,
            );
            assert_eq!(observed.content, "a z\n");
            assert_eq!(observed.counts, (1, 1, 0));
            rows += 1;
        }
    }
    assert_eq!(rows, 36);
}

#[test]
fn should_collapse_generic_end_skip_and_blank_custom_source_separators() {
    let mut rows = 0;
    for options in markup_options() {
        for action in [
            Action::Skip,
            Action::Custom(""),
            Action::Custom(" "),
            Action::Custom("\t "),
        ] {
            let observed = observe(
                "<div>a<b>\n<span id=omit><br></span></b>z</div>",
                options.clone(),
                Action::Continue,
                action,
                Action::Continue,
            );
            assert_eq!(observed.content, "a z\n");
            assert_eq!(observed.counts, (1, 1, 1));
            rows += 1;
        }
    }
    assert_eq!(rows, 48);
}

#[test]
fn should_keep_generic_skip_continue_preserved_and_css_contracts() {
    let mut rows = 0;
    for options in markup_options() {
        for (action, expected, counts) in [
            (Action::Skip, "a z\n", (1, 0, 0)),
            (Action::Continue, hard_break_output(&options), (1, 1, 1)),
            (Action::PreserveHtml, hard_break_output(&options), (1, 1, 1)),
        ] {
            let observed = observe(
                "<div>a<b>\n<span id=omit><br></span></b>z</div>",
                options.clone(),
                action,
                Action::Continue,
                Action::Continue,
            );
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, counts);
            rows += 1;
        }
        let mut excluded = options;
        excluded.exclude_selectors = vec!["#omit".into()];
        let observed = observe(
            "<div>a<b>\n<span id=omit><br></span></b>z</div>",
            excluded,
            Action::Continue,
            Action::Continue,
            Action::Continue,
        );
        assert_eq!(observed.content, "a z\n");
        assert_eq!(observed.counts, (1, 0, 0));
        rows += 1;
    }
    assert_eq!(rows, 48);
}

#[test]
fn should_keep_authored_break_siblings_after_generic_omission() {
    let mut rows = 0;
    for options in markup_options() {
        for end_hook in [false, true] {
            for action in [
                Action::Skip,
                Action::Custom(""),
                Action::Custom(" "),
                Action::Custom("\t "),
            ] {
                let observed = observe(
                    "<div>a<b>\n<span id=omit><br></span><br></b>z</div>",
                    options.clone(),
                    if end_hook { Action::Continue } else { action },
                    if end_hook { action } else { Action::Continue },
                    Action::Continue,
                );
                assert_eq!(observed.content, hard_break_output(&options));
                let counts = if end_hook {
                    (1, 1, 2)
                } else if matches!(action, Action::Skip) {
                    (1, 0, 1)
                } else {
                    (1, 1, 1)
                };
                assert_eq!(observed.counts, counts);
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 96);
}

fn forced_options() -> ConversionOptions {
    ConversionOptions {
        output_format: OutputFormat::Markdown,
        newline_style: NewlineStyle::Backslash,
        ..markup_options().pop().expect("forced generic options")
    }
}

#[test]
fn should_keep_literal_nonblank_generic_replacements() {
    let mut rows = 0;
    for options in markup_options()
        .into_iter()
        .filter(|options| options.tier_strategy == TierStrategy::Tier2)
    {
        for (replacement, expected) in [
            ("V", "aVz\n"),
            ("\n", "a\nz\n"),
            ("\u{a0}", "a\u{a0}z\n"),
            ("\r", "a\rz\n"),
            ("\u{c}", "a\u{c}z\n"),
        ] {
            for end_hook in [false, true] {
                let action = Action::Custom(replacement);
                let observed = observe(
                    "<div>a<span id=omit><br></span>z</div>",
                    options.clone(),
                    if end_hook { Action::Continue } else { action },
                    if end_hook { action } else { Action::Continue },
                    Action::Continue,
                );
                assert_eq!(observed.content, expected);
                assert_eq!(observed.counts, (1, 1, usize::from(end_hook)));
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 40);
}

#[test]
fn should_keep_nonblank_replacement_flow_and_observational_end_policy() {
    let mut rows = 0;
    for options in markup_options()
        .into_iter()
        .filter(|options| options.tier_strategy == TierStrategy::Tier2)
    {
        let visible = if options.output_format == OutputFormat::Markdown {
            if options.newline_style == NewlineStyle::Spaces {
                "a  \n**V**z\n"
            } else {
                "a\\\n**V**z\n"
            }
        } else {
            "a\\\n*V*z\n"
        };
        for (start, end, expected, payload) in [
            (Action::Custom(""), Action::Custom("V"), "a z\n", ""),
            (Action::Custom(""), Action::Skip, "a z\n", ""),
            (Action::Custom("V"), Action::Custom(""), visible, "V"),
            (Action::Custom("V"), Action::Skip, visible, "V"),
        ] {
            let observed = observe(
                "<div>a<b>\n<span id=omit><br></span></b>z</div>",
                options.clone(),
                start,
                end,
                Action::Continue,
            );
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, (1, 1, 0));
            assert_eq!(observed.end_outputs, [payload]);
            rows += 1;
        }
    }
    assert_eq!(rows, 16);
}

#[test]
fn should_keep_final_visible_ancestor_replacements_after_child_omission() {
    let mut rows = 0;
    for options in markup_options() {
        let expected = if options.output_format == OutputFormat::Markdown {
            if options.newline_style == NewlineStyle::Spaces {
                "a  \n*V*z\n"
            } else {
                "a\\\n*V*z\n"
            }
        } else {
            "a\\\n_V_z\n"
        };
        for action in [Action::Skip, Action::Custom("")] {
            let observed = observe(
                "<div>a<em>\n<b id=outer><span id=omit><br></span></b></em>z</div>",
                options.clone(),
                action,
                Action::Continue,
                Action::Custom("V"),
            );
            assert_eq!(observed.content, expected);
            assert_eq!(
                observed.counts,
                if matches!(action, Action::Skip) {
                    (1, 0, 0)
                } else {
                    (1, 1, 0)
                }
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 24);
}

#[test]
fn should_keep_normalized_and_plain_generic_omission_contracts() {
    let mut rows = 0;
    for (output_format, whitespace_mode) in [
        (OutputFormat::Markdown, WhitespaceMode::Normalized),
        (OutputFormat::Djot, WhitespaceMode::Normalized),
        (OutputFormat::Plain, WhitespaceMode::Normalized),
        (OutputFormat::Plain, WhitespaceMode::Strict),
    ] {
        let options = ConversionOptions {
            output_format,
            whitespace_mode,
            ..forced_options()
        };
        for (action, plain) in [
            (Action::Skip, "a\nz\n"),
            (Action::Custom(""), "a\nz\n"),
            (Action::Custom(" "), "a\n z\n"),
            (Action::Custom("\t "), "a\n\t z\n"),
        ] {
            for end_hook in [false, true] {
                let observed = observe(
                    "<div>a<b>\n<span id=omit><br></span></b>z</div>",
                    options.clone(),
                    if end_hook { Action::Continue } else { action },
                    if end_hook { action } else { Action::Continue },
                    Action::Continue,
                );
                assert_eq!(
                    observed.content,
                    if output_format == OutputFormat::Plain {
                        plain
                    } else {
                        "a z\n"
                    }
                );
                let starts = if output_format == OutputFormat::Plain { 2 } else { 1 };
                assert_eq!(
                    observed.counts,
                    (
                        starts,
                        if !end_hook && matches!(action, Action::Skip) {
                            0
                        } else {
                            starts
                        },
                        usize::from(end_hook)
                    )
                );
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 32);
}

#[test]
fn should_collapse_nested_and_iterative_table_part_omissions() {
    let mut rows = 0;
    for options in markup_options() {
        for source in [
            "<div>a<b>\n<span id=omit><i><br></i></span></b>z</div>",
            "<div>a<b>\n<td id=omit><br></td></b>z</div>",
        ] {
            for end_hook in [false, true] {
                for action in [
                    Action::Skip,
                    Action::Custom(""),
                    Action::Custom(" "),
                    Action::Custom("\t "),
                ] {
                    let observed = observe(
                        source,
                        options.clone(),
                        if end_hook { Action::Continue } else { action },
                        if end_hook { action } else { Action::Continue },
                        Action::Continue,
                    );
                    assert_eq!(observed.content, "a z\n");
                    assert_eq!(
                        observed.counts,
                        if end_hook {
                            (1, 1, 1)
                        } else if matches!(action, Action::Skip) {
                            (1, 0, 0)
                        } else {
                            (1, 1, 0)
                        }
                    );
                    rows += 1;
                }
            }
        }
    }
    assert_eq!(rows, 192);
}

fn table_options() -> Vec<ConversionOptions> {
    let routes: Vec<_> = markup_options()
        .into_iter()
        .filter(|options| options.tier_strategy == TierStrategy::Tier2)
        .flat_map(|options| {
            [false, true].map(|br_in_tables| ConversionOptions {
                br_in_tables,
                compact_tables: true,
                ..options.clone()
            })
        })
        .collect();
    assert_eq!(routes.len(), 8);
    routes
}

const fn table_output(options: &ConversionOptions, retained: bool) -> &'static str {
    match (options.output_format, retained && options.br_in_tables) {
        (OutputFormat::Djot, true) => "| H |\n|---|\n| a*<br>*z |\n",
        (OutputFormat::Djot, false) => "| H |\n|---|\n| a z |\n",
        (_, true) => "| H |\n| --- |\n| a**<br>**z |\n",
        (_, false) => "| H |\n| --- |\n| a z |\n",
    }
}

#[test]
fn should_restore_generic_breaks_on_table_reentry_and_keep_css_exclusions() {
    let mut rows = 0;
    for options in table_options() {
        for (start, end, counts) in [
            (Action::ReplaySkip, Action::Continue, (2, 1, 1)),
            (Action::ReplayBlank, Action::Continue, (2, 2, 1)),
            (Action::Continue, Action::ReplaySkip, (2, 2, 2)),
            (Action::Continue, Action::ReplayBlank, (2, 2, 2)),
        ] {
            let observed = observe(
                "<table><tr><th>H</th></tr><tr><td>a<b>\n<span id=omit><br></span></b>z</td></tr></table>",
                options.clone(),
                start,
                end,
                Action::Continue,
            );
            assert_eq!(observed.content, table_output(&options, true));
            assert_eq!(observed.counts, counts);
            assert_eq!(observed.rows, vec![vec!["H"], vec!["a z"]]);
            rows += 1;
        }
        let mut excluded = options.clone();
        excluded.exclude_selectors = vec!["#omit".into()];
        let observed = observe(
            "<table><tr><th>H</th></tr><tr><td>a<b>\n<span id=omit><br></span></b>z</td></tr></table>",
            excluded,
            Action::Continue,
            Action::Continue,
            Action::Continue,
        );
        assert_eq!(observed.content, table_output(&options, false));
        assert_eq!(observed.counts, (2, 0, 0));
        assert_eq!(observed.rows, vec![vec!["H"], vec!["a z"]]);
        rows += 1;
    }
    assert_eq!(rows, 40);
}

#[test]
fn should_omit_generic_table_breaks_from_callback_rows_and_final_output() {
    let mut rows = 0;
    for options in table_options() {
        for end_hook in [false, true] {
            for action in [
                Action::Skip,
                Action::Custom(""),
                Action::Custom(" "),
                Action::Custom("\t "),
            ] {
                let observed = observe(
                    "<table><tr><th>H</th></tr><tr><td>a<b>\n<span id=omit><br></span></b>z</td></tr></table>",
                    options.clone(),
                    if end_hook { Action::Continue } else { action },
                    if end_hook { action } else { Action::Continue },
                    Action::Continue,
                );
                assert_eq!(observed.content, table_output(&options, false));
                assert_eq!(observed.rows, vec![vec!["H"], vec!["a z"]]);
                assert_eq!(
                    observed.counts,
                    if end_hook {
                        (2, 2, 2)
                    } else if matches!(action, Action::Skip) {
                        (2, 0, 0)
                    } else {
                        (2, 2, 0)
                    }
                );
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 64);
}

fn visible_output(options: &ConversionOptions) -> &'static str {
    if options.output_format == OutputFormat::Djot {
        "a\\\n*V*z\n"
    } else if options.newline_style == NewlineStyle::Spaces {
        "a  \n**V**z\n"
    } else {
        "a\\\n**V**z\n"
    }
}

#[test]
fn should_keep_same_node_typed_omissions_when_generic_end_returns_visible_text() {
    let mut rows = 0;
    for options in markup_options() {
        for body in [
            "<em id=omit><br></em>",
            "<i id=omit><br></i>",
            "<code id=omit><br></code>",
            "<a id=omit href=h><br></a>",
        ] {
            for typed in [Action::Skip, Action::Custom("")] {
                let source = format!("<div>a<b>\n{body}</b>z</div>");
                let observed = observe_typed(
                    &source,
                    options.clone(),
                    Action::Continue,
                    Action::Custom("V"),
                    Action::Continue,
                    typed,
                );
                assert_eq!(observed.content, visible_output(&options));
                assert_eq!(observed.counts, (1, 1, 1));
                assert_eq!(observed.typed_calls, 1);
                assert_eq!(observed.end_outputs, [""]);
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 96);
}

#[test]
fn should_keep_nonblank_start_replacements_under_css_exclusion() {
    let mut rows = 0;
    for mut options in markup_options() {
        options.exclude_selectors = vec!["#omit".into()];
        let observed = observe(
            "<div>a<b>\n<span id=omit><br></span></b>z</div>",
            options.clone(),
            Action::Custom("V"),
            Action::Continue,
            Action::Continue,
        );
        assert_eq!(observed.content, visible_output(&options));
        assert_eq!(observed.counts, (1, 1, 0));
        assert_eq!(observed.end_outputs, ["V"]);
        rows += 1;
    }
    assert_eq!(rows, 12);
}

#[test]
fn should_keep_visible_start_replacements_after_an_omitted_table_pass() {
    let mut rows = 0;
    for options in table_options() {
        let expected = if options.output_format == OutputFormat::Djot {
            "| H |\n|---|\n| a*V*z |\n"
        } else {
            "| H |\n| --- |\n| a**V**z |\n"
        };
        for (action, counts, payloads) in [
            (Action::ReplaySkipVisible, (2, 1, 0), vec!["V"]),
            (Action::ReplayBlankVisible, (2, 2, 0), vec!["", "V"]),
        ] {
            let observed = observe(
                "<table><tr><th>H</th></tr><tr><td>a<b>\n<span id=omit><br></span></b>z</td></tr></table>",
                options.clone(),
                action,
                Action::Continue,
                Action::Continue,
            );
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, counts);
            assert_eq!(observed.rows, vec![vec!["H"], vec!["a z"]]);
            assert_eq!(observed.end_outputs, payloads);
            rows += 1;
        }
    }
    assert_eq!(rows, 16);
}
