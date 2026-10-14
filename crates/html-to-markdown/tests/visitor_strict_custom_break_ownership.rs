//! Strict custom-element omissions remove descendant BR provenance while preserving replacement bytes. ~keep
#![cfg(all(feature = "testkit", feature = "visitor"))]
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)]

use html_to_markdown_rs::error::ConversionError;
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};
use std::sync::{Arc, Mutex};

const FLOW: &str = "<div>a<b>\n<x-y><br></x-y></b>z</div>";
const CELL: &str = "<table><tr><th>H</th></tr><tr><td>a<b>\n<x-y><br></x-y></b>z</td></tr></table>";

#[derive(Clone, Copy, Debug)]
enum Action {
    Skip,
    Custom(&'static str),
    Continue,
    PreserveHtml,
    Error,
    ReplaySkip,
    ReplayBlank,
}

impl Action {
    fn result(self, call: usize) -> VisitResult {
        match self {
            Self::Skip => VisitResult::Skip,
            Self::Custom(value) => VisitResult::Custom(value.into()),
            Self::PreserveHtml => VisitResult::PreserveHtml,
            Self::Error => VisitResult::Error("custom element callback failed".into()),
            Self::ReplaySkip if call == 1 => VisitResult::Skip,
            Self::ReplayBlank if call == 1 => VisitResult::Custom(String::new()),
            Self::Continue | Self::ReplaySkip | Self::ReplayBlank => VisitResult::Continue,
        }
    }
}

#[derive(Debug)]
struct Hooks {
    action: Action,
    counts: [usize; 4],
    parents: Vec<Option<String>>,
    raw_html: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl HtmlVisitor for Hooks {
    fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
        self.counts[0] += usize::from(context.tag_name == "x-y");
        VisitResult::Continue
    }

    fn visit_element_end(&mut self, context: &NodeContext, _: &str) -> VisitResult {
        self.counts[1] += usize::from(context.tag_name == "x-y");
        VisitResult::Continue
    }

    fn visit_custom_element(&mut self, context: &NodeContext, tag_name: &str, html: &str) -> VisitResult {
        if tag_name != "x-y" {
            return VisitResult::Continue;
        }
        assert_eq!(context.tag_name, tag_name);
        self.counts[2] += 1;
        self.parents.push(context.parent_tag.as_deref().map(str::to_owned));
        self.raw_html.push(html.into());
        self.action.result(self.counts[2])
    }

    fn visit_line_break(&mut self, _: &NodeContext) -> VisitResult {
        self.counts[3] += 1;
        VisitResult::Continue
    }

    fn visit_table_row(&mut self, _: &NodeContext, cells: &[String], _: bool) -> VisitResult {
        self.rows.push(cells.to_vec());
        VisitResult::Continue
    }
}

#[derive(Debug)]
struct Observed {
    content: String,
    counts: [usize; 4],
    parents: Vec<Option<String>>,
    raw_html: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn observe(source: &str, mut options: ConversionOptions, action: Action) -> Result<Observed, ConversionError> {
    let hooks = Arc::new(Mutex::new(Hooks {
        action,
        counts: [0; 4],
        parents: Vec::new(),
        raw_html: Vec::new(),
        rows: Vec::new(),
    }));
    options.visitor = Some(hooks.clone());
    let result = convert(source, options)?;
    let hooks = hooks.lock().expect("custom callback evidence");
    Ok(Observed {
        content: result.content.expect("custom element content"),
        counts: hooks.counts,
        parents: hooks.parents.clone(),
        raw_html: hooks.raw_html.clone(),
        rows: hooks.rows.clone(),
    })
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
                    compact_tables: true,
                    ..Default::default()
                });
            }
        }
    }
    assert_eq!(routes.len(), 12);
    routes
}

fn active_options() -> Vec<ConversionOptions> {
    let routes: Vec<_> = markup_options()
        .into_iter()
        .filter(|options| options.tier_strategy != TierStrategy::Tier1)
        .collect();
    assert_eq!(routes.len(), 8);
    routes
}

const fn omissions() -> [Action; 4] {
    [
        Action::Skip,
        Action::Custom(""),
        Action::Custom(" "),
        Action::Custom("\t "),
    ]
}

const fn hard_marker(options: &ConversionOptions) -> &'static str {
    match (options.output_format, options.newline_style) {
        (OutputFormat::Markdown, NewlineStyle::Spaces) => "  \n",
        _ => "\\\n",
    }
}

const fn strong_marker(options: &ConversionOptions) -> &'static str {
    match options.output_format {
        OutputFormat::Markdown => "**",
        _ => "*",
    }
}

fn table_output(cell: &str, options: &ConversionOptions) -> String {
    let separator = if options.output_format == OutputFormat::Markdown {
        "| --- |"
    } else {
        "|---|"
    };
    format!("| H |\n{separator}\n| {cell} |\n")
}

fn check_content(observed: &Observed, expected: &str, failures: &mut Vec<String>) {
    if observed.content != expected {
        failures.push(format!("expected {expected:?}, got {:?}", observed.content));
    }
}

fn finish_rows(label: &str, rows: usize, expected: usize, failures: &[String]) {
    assert_eq!(rows, expected, "{label} executed rows");
    eprintln!("{label} rows={rows} failures={}", failures.len());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn should_omit_strict_custom_descendant_breaks_without_visiting_them() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for options in active_options() {
        for (source, raw_html) in [
            (FLOW, "<x-y><br></x-y>"),
            ("<div>a<b>\n<x-y><i><br></i></x-y></b>z</div>", "<x-y><i><br></i></x-y>"),
        ] {
            for action in omissions() {
                let observed = observe(source, options.clone(), action).expect("strict custom omission");
                assert_eq!(observed.counts, [1, 1, 1, 0]);
                assert_eq!(observed.parents, [Some("b".into())]);
                assert_eq!(observed.raw_html, [raw_html]);
                check_content(&observed, "a z\n", &mut failures);
                rows += 1;
            }
        }
    }
    finish_rows("flow-omission", rows, 64, &failures);
}

#[test]
fn should_preserve_a_retained_break_after_custom_omission() {
    let mut rows = 0;
    let source = "<div>a<b>\n<x-y><br></x-y><br></b>z</div>";
    for options in active_options() {
        for action in omissions() {
            let observed = observe(source, options.clone(), action).expect("retained sibling break");
            assert_eq!(observed.content, format!("a{}z\n", hard_marker(&options)));
            assert_eq!(observed.counts, [1, 1, 1, 1]);
            rows += 1;
        }
    }
    finish_rows("retained-break", rows, 32, &[]);
}

#[test]
fn should_preserve_continue_visible_custom_and_original_html() {
    let mut rows = 0;
    for options in active_options() {
        let hard = hard_marker(&options);
        let strong = strong_marker(&options);
        for (action, expected, breaks) in [
            (Action::Continue, format!("a{hard}z\n"), 1),
            (Action::Custom("V"), format!("a{hard}{strong}V{strong}z\n"), 0),
            (
                Action::PreserveHtml,
                format!("a{hard}{strong}<x-y><br></x-y>{strong}z\n"),
                0,
            ),
        ] {
            let observed = observe(FLOW, options.clone(), action).expect("retained custom rendering");
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, [1, 1, 1, breaks]);
            rows += 1;
        }
    }
    finish_rows("retained-actions", rows, 24, &[]);
}

#[test]
fn should_preserve_lf_cr_and_nbsp_custom_replacements() {
    let mut rows = 0;
    for options in active_options() {
        for (replacement, expected) in [
            ("\n", String::from("a z\n")),
            ("\r", format!("a{}z\n", hard_marker(&options))),
            ("\u{a0}", String::from("a z\n")),
        ] {
            let observed = observe(FLOW, options.clone(), Action::Custom(replacement)).expect("custom line character");
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, [1, 1, 1, 0]);
            rows += 1;
        }
    }
    finish_rows("line-characters", rows, 24, &[]);
}

#[test]
fn should_preserve_normalized_custom_omission_behavior() {
    let mut rows = 0;
    for mut options in active_options() {
        options.whitespace_mode = WhitespaceMode::Normalized;
        for action in omissions() {
            let observed = observe(FLOW, options.clone(), action).expect("normalized custom omission");
            assert_eq!(observed.content, "a z\n");
            assert_eq!(observed.counts, [1, 1, 1, 0]);
            rows += 1;
        }
    }
    finish_rows("normalized-omission", rows, 32, &[]);
}

#[test]
fn should_preserve_literal_pre_newlines_and_custom_blank_bytes() {
    let mut rows = 0;
    let source = "<pre>a<b>\n<x-y><br></x-y></b>z</pre>";
    for options in active_options() {
        for (action, expected) in [
            (Action::Skip, "```\na\nz\n```\n"),
            (Action::Custom(""), "```\na\nz\n```\n"),
            (Action::Custom(" "), "```\na\n z\n```\n"),
            (Action::Custom("\t "), "```\na\n\t z\n```\n"),
        ] {
            let observed = observe(source, options.clone(), action).expect("literal PRE custom omission");
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, [1, 1, 1, 0]);
            rows += 1;
        }
    }
    finish_rows("literal-pre", rows, 32, &[]);
}

#[test]
fn should_propagate_custom_element_errors() {
    let mut rows = 0;
    for options in active_options() {
        let error = observe(FLOW, options, Action::Error).expect_err("custom visitor error");
        assert!(matches!(error, ConversionError::Visitor(ref message) if message == "custom element callback failed"));
        assert_eq!(error.to_string(), "Visitor error: custom element callback failed");
        rows += 1;
    }
    finish_rows("custom-errors", rows, 8, &[]);
}

#[test]
fn should_omit_custom_breaks_in_table_outputs_and_row_payloads() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for mut options in active_options() {
        for br_in_tables in [false, true] {
            options.br_in_tables = br_in_tables;
            for action in omissions() {
                let observed = observe(CELL, options.clone(), action).expect("table custom omission");
                assert_eq!(observed.counts, [2, 2, 2, 0]);
                assert_eq!(observed.raw_html, ["<x-y><br></x-y>", "<x-y><br></x-y>"]);
                check_content(&observed, &table_output("a z", &options), &mut failures);
                if observed.rows != [vec![String::from("H")], vec![String::from("a z")]] {
                    failures.push(format!("unexpected row payload {:?}", observed.rows));
                }
                rows += 1;
            }
        }
    }
    finish_rows("table-omission", rows, 64, &failures);
}

#[test]
fn should_restore_custom_break_ownership_when_table_replay_continues() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for mut options in active_options() {
        for br_in_tables in [false, true] {
            options.br_in_tables = br_in_tables;
            for action in [Action::ReplaySkip, Action::ReplayBlank] {
                let observed = observe(CELL, options.clone(), action).expect("custom table replay");
                assert_eq!(observed.counts, [2, 2, 2, 1]);
                if observed.rows != [vec![String::from("H")], vec![String::from("a z")]] {
                    failures.push(format!("unexpected replay row payload {:?}", observed.rows));
                }
                let retained_cell = if br_in_tables {
                    format!("a{}<br>{}z", strong_marker(&options), strong_marker(&options))
                } else {
                    String::from("a z")
                };
                check_content(&observed, &table_output(&retained_cell, &options), &mut failures);
                rows += 1;
            }
        }
    }
    finish_rows("table-replay", rows, 32, &failures);
}

#[test]
fn should_execute_custom_omissions_after_forced_tier1_dom_fallback() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for options in markup_options()
        .into_iter()
        .filter(|options| options.tier_strategy == TierStrategy::Tier1)
    {
        for action in omissions() {
            let observed = observe(FLOW, options.clone(), action).expect("forced Tier1 custom fallback");
            assert_eq!(observed.counts, [1, 1, 1, 0]);
            check_content(&observed, "a z\n", &mut failures);
            rows += 1;
        }
    }
    finish_rows("forced-tier1-fallback", rows, 16, &failures);
}
