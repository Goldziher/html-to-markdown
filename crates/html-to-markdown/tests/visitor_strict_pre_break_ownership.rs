//! Strict PRE omissions remove descendant BR provenance without collapsing literal code or block gaps. ~keep
#![cfg(all(feature = "testkit", feature = "visitor"))]
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)]

use html_to_markdown_rs::error::ConversionError;
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};
use std::sync::{Arc, Mutex};

const MALFORMED_FLOW: &str = "<div>a<b>\n<pre id=omit><code><br></code></pre></b>z</div>";
const MALFORMED_CELL: &str =
    "<table><tr><th>H</th></tr><tr><td>a<b>\n<pre id=omit><code><br></code></pre></b>z</td></tr></table>";

#[derive(Clone, Copy, Debug)]
enum Action {
    Continue,
    Skip,
    Custom(&'static str),
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
            Self::Error => VisitResult::Error("PRE ownership callback failed".into()),
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
    code_text: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn is_target(context: &NodeContext) -> bool {
    context.attributes().get("id").map(String::as_str) == Some("omit")
}

impl HtmlVisitor for Hooks {
    fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
        if is_target(context) {
            self.counts[0] += 1;
            self.parents.push(context.parent_tag.as_deref().map(str::to_owned));
        }
        VisitResult::Continue
    }

    fn visit_element_end(&mut self, context: &NodeContext, _: &str) -> VisitResult {
        self.counts[1] += usize::from(is_target(context));
        VisitResult::Continue
    }

    fn visit_line_break(&mut self, _: &NodeContext) -> VisitResult {
        self.counts[2] += 1;
        VisitResult::Continue
    }

    fn visit_code_block(&mut self, context: &NodeContext, _: Option<&str>, code_text: &str) -> VisitResult {
        if is_target(context) {
            self.counts[3] += 1;
            self.code_text.push(code_text.into());
            self.action.result(self.counts[3])
        } else {
            VisitResult::Continue
        }
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
    code_text: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn observe(source: &str, mut options: ConversionOptions, action: Action) -> Result<Observed, ConversionError> {
    let hooks = Arc::new(Mutex::new(Hooks {
        action,
        counts: [0; 4],
        parents: Vec::new(),
        code_text: Vec::new(),
        rows: Vec::new(),
    }));
    options.visitor = Some(hooks.clone());
    let result = convert(source, options)?;
    let hooks = hooks.lock().expect("PRE callback evidence");
    Ok(Observed {
        content: result.content.expect("PRE content"),
        counts: hooks.counts,
        parents: hooks.parents.clone(),
        code_text: hooks.code_text.clone(),
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

fn active_flow_options() -> Vec<ConversionOptions> {
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

fn hard_marker(options: &ConversionOptions) -> &'static str {
    if options.output_format == OutputFormat::Markdown && options.newline_style == NewlineStyle::Spaces {
        "  \n"
    } else {
        "\\\n"
    }
}

fn strong_marker(options: &ConversionOptions) -> &'static str {
    if options.output_format == OutputFormat::Markdown {
        "**"
    } else {
        "*"
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

const fn omitted_cell(options: &ConversionOptions) -> &'static str {
    if options.br_in_tables { "a<br>z" } else { "a z" }
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
fn should_omit_typed_pre_break_ownership_under_inline_ancestors() {
    let mut rows = 0;
    let mut failures = Vec::new();
    let sources = [
        MALFORMED_FLOW,
        "<div>a<b>\r<pre id=omit><code><br></code></pre></b>z</div>",
        "<div>a<b>\n<pre id=omit><code><span><br></span></code></pre></b>z</div>",
    ];
    for options in active_flow_options() {
        for source in sources {
            for action in omissions() {
                let observed = observe(source, options.clone(), action).expect("inline PRE omission");
                assert_eq!(observed.counts, [1, 1, 1, 1]);
                assert_eq!(observed.parents, [Some("b".into())]);
                assert_eq!(observed.code_text, ["\n"]);
                check_content(&observed, "a z\n", &mut failures);
                rows += 1;
            }
        }
    }
    finish_rows("inline-omission", rows, 96, &failures);
}

#[test]
fn should_omit_typed_pre_break_ownership_in_table_row_payloads() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for mut options in markup_options() {
        for br_in_tables in [false, true] {
            options.br_in_tables = br_in_tables;
            let cell = omitted_cell(&options);
            for action in omissions() {
                let observed = observe(MALFORMED_CELL, options.clone(), action).expect("cell PRE omission");
                assert_eq!(observed.counts, [2, 2, 2, 2]);
                assert_eq!(observed.parents, [Some("b".into()), Some("b".into())]);
                assert_eq!(observed.code_text, ["\n", "\n"]);
                check_content(&observed, &table_output(cell, &options), &mut failures);
                if observed.rows != [vec![String::from("H")], vec![String::from(cell)]] {
                    failures.push(format!("unexpected row payload {:?}", observed.rows));
                }
                rows += 1;
            }
        }
    }
    finish_rows("cell-omission", rows, 96, &failures);
}

#[test]
fn should_clear_typed_pre_omission_when_table_replay_retains_the_pre() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for mut options in markup_options() {
        for br_in_tables in [false, true] {
            options.br_in_tables = br_in_tables;
            for action in [Action::ReplaySkip, Action::ReplayBlank] {
                let observed = observe(MALFORMED_CELL, options.clone(), action).expect("PRE replay");
                assert_eq!(observed.counts, [2, 2, 2, 2]);
                let cell = omitted_cell(&options);
                if observed.rows != [vec![String::from("H")], vec![String::from(cell)]] {
                    failures.push(format!("unexpected replay row payload {:?}", observed.rows));
                }
                let retained = if hard_marker(&options) == "  \n" {
                    cell.into()
                } else {
                    format!("a\\{}z", if br_in_tables { "<br>" } else { " " })
                };
                check_content(&observed, &table_output(&retained, &options), &mut failures);
                rows += 1;
            }
        }
    }
    finish_rows("cell-replay", rows, 48, &failures);
}

#[test]
fn should_keep_valid_flow_pre_block_gaps_when_omitted() {
    let mut rows = 0;
    for options in active_flow_options() {
        for (action, expected) in [
            (Action::Skip, "a\n\nz\n"),
            (Action::Custom(""), "a\n\nz\n"),
            (Action::Custom(" "), "a\n z\n"),
            (Action::Custom("\t "), "a\n\t z\n"),
        ] {
            let source = "<div>a\n<pre id=omit><code><br></code></pre>z</div>";
            let observed = observe(source, options.clone(), action).expect("valid flow PRE");
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, [1, 1, 1, 1]);
            assert_eq!(observed.parents, [Some("div".into())]);
            rows += 1;
        }
    }
    finish_rows("valid-block-gap", rows, 32, &[]);
}

#[test]
fn should_keep_literal_newlines_and_custom_bytes_inside_an_enclosing_pre() {
    let mut rows = 0;
    let source = "<pre>a<b>\n<pre id=omit><code><br></code></pre></b>z</pre>";
    for options in active_flow_options() {
        for (action, expected) in [
            (Action::Skip, "```\na\nz\n```\n"),
            (Action::Custom(""), "```\na\nz\n```\n"),
            (Action::Custom(" "), "```\na\n z\n```\n"),
            (Action::Custom("\t "), "```\na\n\t z\n```\n"),
        ] {
            let observed = observe(source, options.clone(), action).expect("literal enclosing PRE");
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, [1, 1, 1, 1]);
            rows += 1;
        }
    }
    finish_rows("literal-enclosing-pre", rows, 32, &[]);
}

#[test]
fn should_keep_standalone_pre_literal_content_and_empty_omissions() {
    let mut rows = 0;
    for options in active_flow_options() {
        let source = "<pre id=omit><code> x\n y </code></pre>";
        let observed = observe(source, options.clone(), Action::Continue).expect("literal standalone PRE");
        assert_eq!(observed.content, "```\n x\n y \n```\n");
        assert_eq!(observed.counts, [1, 1, 0, 1]);
        assert_eq!(observed.code_text, [" x\n y "]);
        rows += 1;
        for action in omissions() {
            let observed = observe("<pre id=omit><code><br></code></pre>", options.clone(), action)
                .expect("standalone PRE omission");
            assert_eq!(observed.content, "");
            assert_eq!(observed.counts, [1, 1, 1, 1]);
            rows += 1;
        }
    }
    finish_rows("standalone-literals", rows, 40, &[]);
}

#[test]
fn should_keep_a_retained_break_after_an_omitted_pre() {
    let mut rows = 0;
    let source = "<div>a<b>\n<pre id=omit><code><br></code></pre><br></b>z</div>";
    for options in active_flow_options() {
        for action in omissions() {
            let observed = observe(source, options.clone(), action).expect("retained sibling BR");
            assert_eq!(observed.content, format!("a{}z\n", hard_marker(&options)));
            assert_eq!(observed.counts, [1, 1, 2, 1]);
            rows += 1;
        }
    }
    finish_rows("retained-after-break", rows, 32, &[]);
}

#[test]
fn should_keep_continue_nonblank_custom_and_preserved_html_rendering() {
    let mut rows = 0;
    for options in active_flow_options() {
        let strong = strong_marker(&options);
        let hard = hard_marker(&options);
        for (action, expected) in [
            (
                Action::Continue,
                format!("a{strong}```{strong}\n\n{strong}```{strong}z\n"),
            ),
            (Action::Custom("V"), format!("a{hard}{strong}V{strong}z\n")),
            (
                Action::PreserveHtml,
                format!("a{hard}{strong}<pre id=\"omit\"><code><br /></code></pre>{strong}z\n"),
            ),
        ] {
            let observed = observe(MALFORMED_FLOW, options.clone(), action).expect("retained PRE rendering");
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, [1, 1, 1, 1]);
            rows += 1;
        }
    }
    finish_rows("retained-pre-actions", rows, 24, &[]);
}

#[test]
fn should_keep_lf_cr_and_nbsp_custom_replacement_contracts() {
    let mut rows = 0;
    for options in active_flow_options() {
        for (replacement, expected) in [
            ("\n", String::from("a z\n")),
            ("\r", format!("a{}\nz\n", hard_marker(&options))),
            ("\u{a0}", String::from("a z\n")),
        ] {
            let observed = observe(MALFORMED_FLOW, options.clone(), Action::Custom(replacement))
                .expect("non-ASCII-blank PRE replacement");
            assert_eq!(observed.content, expected);
            assert_eq!(observed.counts, [1, 1, 1, 1]);
            rows += 1;
        }
    }
    finish_rows("custom-line-characters", rows, 24, &[]);
}

#[test]
fn should_propagate_typed_pre_errors() {
    let mut rows = 0;
    for options in active_flow_options() {
        let error = observe(MALFORMED_FLOW, options, Action::Error).expect_err("typed PRE error");
        assert!(matches!(error, ConversionError::Visitor(ref message) if message == "PRE ownership callback failed"));
        assert_eq!(error.to_string(), "Visitor error: PRE ownership callback failed");
        rows += 1;
    }
    finish_rows("typed-pre-errors", rows, 8, &[]);
}

#[test]
fn should_preserve_forced_tier1_pre_callback_bypass() {
    let mut rows = 0;
    for options in markup_options()
        .into_iter()
        .filter(|options| options.tier_strategy == TierStrategy::Tier1)
    {
        for action in omissions() {
            let observed = observe(MALFORMED_FLOW, options.clone(), action).expect("forced Tier1 PRE");
            assert_eq!(observed.content, "a**```**\n\n**```**z\n");
            assert_eq!(observed.counts, [0; 4]);
            assert_eq!(observed.parents, Vec::<Option<String>>::new());
            assert_eq!(observed.code_text, Vec::<String>::new());
            rows += 1;
        }
    }
    finish_rows("forced-tier1-bypass", rows, 16, &[]);
}

#[test]
fn should_keep_no_br_typed_pre_omission_equivalent_to_source_separation() {
    let mut rows = 0;
    for options in active_flow_options() {
        for body in ["\n", "\r"] {
            let source = format!("<div>a<b>\n<pre id=omit><code>{body}</code></pre></b>z</div>");
            for action in omissions() {
                let observed = observe(&source, options.clone(), action).expect("literal-only PRE omission");
                assert_eq!(observed.content, "a z\n");
                assert_eq!(observed.counts, [1, 1, 0, 1]);
                assert_eq!(observed.code_text, ["\n"]);
                rows += 1;
            }
        }
    }
    finish_rows("no-br-equivalence", rows, 64, &[]);
}

#[test]
fn should_keep_valid_cell_boundaries_when_a_pre_is_omitted() {
    let mut rows = 0;
    let source = "<table><tr><th>H</th></tr><tr><td>a\n<pre id=omit><code><br></code></pre>z</td></tr></table>";
    for mut options in active_flow_options() {
        for br_in_tables in [false, true] {
            options.br_in_tables = br_in_tables;
            let cell = omitted_cell(&options);
            for action in omissions() {
                let observed = observe(source, options.clone(), action).expect("valid cell PRE");
                assert_eq!(observed.content, table_output(cell, &options));
                assert_eq!(observed.rows, [vec![String::from("H")], vec![String::from(cell)]]);
                assert_eq!(observed.counts, [2, 2, 2, 2]);
                assert_eq!(observed.parents, [Some("td".into()), Some("td".into())]);
                rows += 1;
            }
        }
    }
    finish_rows("valid-cell-boundaries", rows, 64, &[]);
}
