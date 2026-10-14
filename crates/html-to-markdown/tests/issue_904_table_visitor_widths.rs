//! Widths follow emitted visitor text while callback walks keep their existing contract (#904). ~keep
#![cfg(feature = "visitor")]
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)]

use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{ConversionOptions, ConversionResult, OutputFormat, TierStrategy, convert};
use std::sync::{Arc, Mutex};

const BREAD: &str = concat!(
    "<table><tr><th>Item</th><th>Price</th></tr>",
    "<tr><td>Bread <i>with extra rye flour</i> loaf</td><td>3</td></tr></table>"
);
const CHEESE: &str = concat!(
    "<table><tr><th>Item</th><th>Price</th></tr>",
    "<tr><td>Cheese <span>from the mountain farm</span></td><td>3</td></tr></table>"
);
const REPLACEMENT: &str = concat!(
    "<table><tr><th>Item</th><th>Price</th></tr>",
    "<tr><td><span>obsolete text source</span></td><td>3</td></tr></table>"
);

#[derive(Debug, Default, Clone)]
struct Trace {
    starts: usize,
    ends: usize,
    links: usize,
    rows: Vec<Vec<String>>,
    generic_table_outputs: Vec<String>,
    generic_div_outputs: Vec<String>,
    table_outputs: Vec<String>,
}

#[derive(Debug)]
struct Visitor {
    target: &'static str,
    answers: Vec<VisitResult>,
    row_answer: VisitResult,
    trace: Arc<Mutex<Trace>>,
}

impl HtmlVisitor for Visitor {
    fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if ctx.tag_name != self.target {
            return VisitResult::Continue;
        }
        let mut trace = self.trace.lock().expect("trace lock");
        let index = trace.starts.min(self.answers.len() - 1);
        trace.starts += 1;
        drop(trace);
        self.answers[index].clone()
    }

    fn visit_element_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if ctx.tag_name == self.target {
            self.trace.lock().expect("trace lock").ends += 1;
        }
        if ctx.tag_name == "table" {
            self.trace
                .lock()
                .expect("trace lock")
                .generic_table_outputs
                .push(output.into());
        }
        if ctx.tag_name == "div" {
            self.trace
                .lock()
                .expect("trace lock")
                .generic_div_outputs
                .push(output.into());
        }
        VisitResult::Continue
    }

    fn visit_link(&mut self, _: &NodeContext, _: &str, _: &str, _: Option<&str>) -> VisitResult {
        self.trace.lock().expect("trace lock").links += 1;
        VisitResult::Continue
    }

    fn visit_table_row(&mut self, _: &NodeContext, cells: &[String], is_header: bool) -> VisitResult {
        self.trace.lock().expect("trace lock").rows.push(cells.to_vec());
        if is_header {
            VisitResult::Continue
        } else {
            self.row_answer.clone()
        }
    }

    fn visit_table_end(&mut self, _: &NodeContext, output: &str) -> VisitResult {
        self.trace.lock().expect("trace lock").table_outputs.push(output.into());
        VisitResult::Continue
    }
}

fn configurations(compact: bool) -> Vec<ConversionOptions> {
    let mut options = Vec::new();
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
            for extract_metadata in [false, true] {
                options.push(ConversionOptions {
                    output_format,
                    tier_strategy,
                    extract_metadata,
                    compact_tables: compact,
                    ..Default::default()
                });
            }
        }
    }
    assert_eq!(options.len(), 8);
    options
}

fn all_configurations() -> Vec<ConversionOptions> {
    configurations(false).into_iter().chain(configurations(true)).collect()
}

fn observed(
    html: &str,
    mut options: ConversionOptions,
    target: &'static str,
    answers: Vec<VisitResult>,
    row_answer: VisitResult,
) -> (html_to_markdown_rs::Result<ConversionResult>, Trace) {
    assert!(!answers.is_empty());
    let trace = Arc::new(Mutex::new(Trace::default()));
    options.visitor = Some(Arc::new(Mutex::new(Visitor {
        target,
        answers,
        row_answer,
        trace: Arc::clone(&trace),
    })));
    let result = convert(html, options);
    let calls = trace.lock().expect("trace lock").clone();
    (result, calls)
}

fn check(actual: impl AsRef<str>, expected: impl AsRef<str>, label: &str, failures: &mut Vec<String>) {
    if actual.as_ref() != expected.as_ref() {
        failures.push(format!(
            "{label}: expected {:?}, got {:?}",
            expected.as_ref(),
            actual.as_ref()
        ));
    }
}

fn check_calls(trace: &Trace, starts: usize, ends: usize, rows: usize, failures: &mut Vec<String>) {
    let actual = (trace.starts, trace.ends, trace.rows.len());
    if actual != (starts, ends, rows) {
        failures.push(format!(
            "callback counts: expected {:?}, got {actual:?}",
            (starts, ends, rows)
        ));
    }
}

fn content(result: ConversionResult) -> String {
    result.content.expect("table content")
}

fn finish(label: &str, rows: usize, expected: usize, failures: &[String]) {
    eprintln!("ROW_COUNT {label} actual={rows} expected={expected}");
    assert!(expected > 0);
    assert_eq!(rows, expected);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn expected_table(item: &str, width: usize, options: &ConversionOptions) -> String {
    let (item_width, price_width) = if options.compact_tables { (0, 0) } else { (width, 5) };
    let header = format!("| {:item_width$} | {:price_width$} |\n", "Item", "Price");
    let separator = if options.output_format == OutputFormat::Djot {
        format!(
            "|{}|{}|\n",
            "-".repeat(item_width.max(3)),
            "-".repeat(price_width.max(3))
        )
    } else {
        format!(
            "| {} | {} |\n",
            "-".repeat(item_width.max(3)),
            "-".repeat(price_width.max(3))
        )
    };
    format!("{header}{separator}| {item:item_width$} | {:price_width$} |\n", "3")
}

#[test]
fn should_measure_surviving_bread_text_without_changing_paragraph_or_table_callbacks() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for options in all_configurations() {
        let (result, trace) = observed(
            BREAD,
            options.clone(),
            "i",
            vec![VisitResult::Skip],
            VisitResult::Continue,
        );
        rows += 1;
        check(
            content(result.expect("bread table")),
            expected_table("Bread loaf", 10, &options),
            "bread",
            &mut failures,
        );
        check_calls(&trace, 2, 0, 2, &mut failures);
        let (result, trace) = observed(
            "<p>Bread <i>with extra rye flour</i> loaf</p>",
            options,
            "i",
            vec![VisitResult::Skip],
            VisitResult::Continue,
        );
        rows += 1;
        check(
            content(result.expect("bread paragraph")),
            "Bread loaf\n",
            "paragraph",
            &mut failures,
        );
        check_calls(&trace, 1, 0, 0, &mut failures);
    }
    finish("bread-skip", rows, 32, &failures);
}

#[test]
fn should_measure_cheese_after_a_span_is_skipped() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for options in all_configurations() {
        let (result, trace) = observed(
            CHEESE,
            options.clone(),
            "span",
            vec![VisitResult::Skip],
            VisitResult::Continue,
        );
        rows += 1;
        check(
            content(result.expect("cheese table")),
            expected_table("Cheese", 6, &options),
            "cheese",
            &mut failures,
        );
        check_calls(&trace, 2, 0, 2, &mut failures);
    }
    finish("cheese-skip", rows, 16, &failures);
}

#[test]
fn should_size_columns_for_empty_short_and_long_custom_replacements() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for options in all_configurations() {
        for (replacement, width) in [
            ("", 4),
            ("X", 4),
            ("a replacement much longer than the original source", 50),
        ] {
            let (result, trace) = observed(
                REPLACEMENT,
                options.clone(),
                "span",
                vec![VisitResult::Custom(replacement.into())],
                VisitResult::Continue,
            );
            rows += 1;
            check(
                content(result.expect("replacement table")),
                expected_table(replacement, width, &options),
                "custom",
                &mut failures,
            );
            check_calls(&trace, 2, 2, 2, &mut failures);
        }
    }
    finish("custom-widths", rows, 48, &failures);
}

#[test]
fn should_use_the_actual_emission_answer_rather_than_the_row_callback_preview() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for options in all_configurations() {
        let (result, trace) = observed(
            BREAD,
            options.clone(),
            "i",
            vec![
                VisitResult::Custom("a deliberately long preview".into()),
                VisitResult::Custom("X".into()),
            ],
            VisitResult::Continue,
        );
        rows += 1;
        check(
            content(result.expect("stateful table")),
            expected_table("Bread X loaf", 12, &options),
            "second answer",
            &mut failures,
        );
        check_calls(&trace, 2, 2, 2, &mut failures);
        if trace.rows.get(1) != Some(&vec!["Bread a deliberately long preview loaf".into(), "3".into()]) {
            failures.push(format!("row preview changed: {:?}", trace.rows));
        }
    }
    finish("stateful-width", rows, 16, &failures);
}

#[test]
fn should_keep_the_separate_document_structure_callback_walk() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for mut options in all_configurations() {
        options.include_document_structure = true;
        let (result, trace) = observed(
            CHEESE,
            options.clone(),
            "span",
            vec![VisitResult::Skip],
            VisitResult::Continue,
        );
        rows += 1;
        let result = result.expect("structured table");
        check(
            result.content.as_deref().expect("structured content"),
            expected_table("Cheese", 6, &options),
            "structure width",
            &mut failures,
        );
        check_calls(&trace, 3, 0, 2, &mut failures);
        assert_eq!(result.tables.len(), 1);
        assert_eq!(result.tables[0].grid.cells[2].content, "Cheese");
    }
    finish("structure-width", rows, 16, &failures);
}

#[test]
fn should_preserve_continue_output_and_legacy_typed_hook_counts() {
    let mut rows = 0;
    let mut failures = Vec::new();
    let source = "<table><tr><th>Item</th><th>Price</th></tr><tr><td><span>Cheese</span> <a href=/u>link</a></td><td>3</td></tr></table>";
    for options in all_configurations() {
        let control = convert(source, options.clone()).expect("no-visitor control");
        rows += 1;
        let (result, trace) = observed(
            source,
            options.clone(),
            "span",
            vec![VisitResult::Continue],
            VisitResult::Continue,
        );
        rows += 1;
        let result = result.expect("continue table");
        check(
            result.content.as_deref().expect("continue content"),
            control.content.as_deref().expect("control content"),
            "continue",
            &mut failures,
        );
        check_calls(&trace, 2, 2, 2, &mut failures);
        assert_eq!(trace.links, if options.compact_tables { 2 } else { 3 });
        #[cfg(feature = "metadata")]
        if options.extract_metadata {
            assert_eq!(result.metadata.links.len(), 1);
        }
    }
    finish("continue-typed-controls", rows, 32, &failures);
}

#[test]
fn should_keep_colspan_rowspan_and_ragged_columns_after_cell_omissions() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for options in all_configurations() {
        for source in [
            "<table><tr><th>Item</th><th>Price</th></tr><tr><td colspan=2>Cheese<span>surplus long text</span></td></tr></table>",
            "<table><tr><th>Item</th><th>Price</th></tr><tr><td rowspan=2>Cheese<span>surplus long text</span></td><td>3</td></tr><tr><td>4</td></tr></table>",
            "<table><tr><th>Item</th></tr><tr><td>Cheese<span>surplus long text</span></td><td>3</td></tr></table>",
        ] {
            let control_source = source.replace("<span>surplus long text</span>", "");
            let control = content(convert(&control_source, options.clone()).expect("span control"));
            rows += 1;
            assert!(!control.is_empty());
            let (result, trace) = observed(
                source,
                options.clone(),
                "span",
                vec![VisitResult::Skip],
                VisitResult::Continue,
            );
            rows += 1;
            check(
                content(result.expect("spanned table")),
                control,
                "span positions",
                &mut failures,
            );
            let row_count = if source.contains("rowspan") { 3 } else { 2 };
            check_calls(&trace, 2, 0, row_count, &mut failures);
        }
    }
    finish("spanned-widths", rows, 96, &failures);
}

#[test]
fn should_keep_plain_tables_unpadded_with_the_existing_callback_walks() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for mut options in all_configurations()
        .into_iter()
        .filter(|options| options.output_format == OutputFormat::Markdown)
    {
        options.output_format = OutputFormat::Plain;
        for (answer, expected, ends) in [
            (VisitResult::Skip, "Item\tPrice\nCheese\t3\n", 0),
            (VisitResult::Custom("X".into()), "Item\tPrice\nCheese X\t3\n", 1),
            (
                VisitResult::Continue,
                "Item\tPrice\nCheese from the mountain farm\t3\n",
                1,
            ),
        ] {
            let (result, trace) = observed(CHEESE, options.clone(), "span", vec![answer], VisitResult::Continue);
            rows += 1;
            check(content(result.expect("plain table")), expected, "plain", &mut failures);
            check_calls(&trace, 3, 3 * ends, 2, &mut failures);
        }
    }
    finish("plain-controls", rows, 24, &failures);
}

#[test]
fn should_keep_row_skip_custom_preserve_html_and_error_decisions() {
    let mut rows = 0;
    let mut failures = Vec::new();
    let source =
        "<table><tr><th>Item</th><th>Price</th></tr><tr><td><span>old long source</span></td><td>3</td></tr></table>";
    for options in all_configurations() {
        for answer in [
            VisitResult::Skip,
            VisitResult::Custom("ROW\n".into()),
            VisitResult::PreserveHtml,
            VisitResult::Error("row failed".into()),
        ] {
            let (result, trace) = observed(
                source,
                options.clone(),
                "span",
                vec![VisitResult::Continue],
                answer.clone(),
            );
            rows += 1;
            check_calls(&trace, 1, 1, 2, &mut failures);
            if matches!(answer, VisitResult::Error(_)) {
                check(
                    result.expect_err("row error").to_string(),
                    "Visitor error: row failed",
                    "row error",
                    &mut failures,
                );
                continue;
            }
            let mut expected = expected_table("", 4, &options);
            let body_start = expected.find('\n').expect("header") + 1;
            let header_end = expected[body_start..].find('\n').expect("separator") + body_start + 1;
            expected.truncate(header_end);
            match answer {
                VisitResult::Custom(custom) => expected.push_str(&custom),
                VisitResult::PreserveHtml => {
                    expected.push_str("<tr><td><span>old long source</span></td><td>3</td></tr>\n");
                }
                _ => {}
            }
            check(
                content(result.expect("row decision")),
                expected,
                "row decision",
                &mut failures,
            );
        }
    }
    finish("row-decisions", rows, 64, &failures);
}

#[test]
fn should_measure_final_escaped_cells_by_characters() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for options in all_configurations() {
        for replacement in ["|a|b", "`a|b`", "ééééé"] {
            let (expected, width) = match (replacement, options.output_format) {
                ("|a|b", _) => (r"\|a\|b", 6),
                ("`a|b`", OutputFormat::Markdown) => (r"`a\|b`", 6),
                ("`a|b`", _) => ("`a|b`", 5),
                _ => ("ééééé", 5),
            };
            let (result, trace) = observed(
                REPLACEMENT,
                options.clone(),
                "span",
                vec![VisitResult::Custom(replacement.into())],
                VisitResult::Continue,
            );
            rows += 1;
            check(
                content(result.expect("escaped cell")),
                expected_table(expected, width, &options),
                "escaped width",
                &mut failures,
            );
            check_calls(&trace, 2, 2, 2, &mut failures);
        }
    }
    finish("escaped-character-widths", rows, 48, &failures);
}

#[test]
fn should_preserve_tables_inside_code_and_pre_contexts() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for options in all_configurations() {
        for wrapper in ["code", "pre"] {
            let source = format!("<{wrapper}>{CHEESE}</{wrapper}>");
            let expected = content(convert(&source, options.clone()).expect("literal control"));
            rows += 1;
            assert!(!expected.is_empty());
            let (result, trace) = observed(
                &source,
                options.clone(),
                "span",
                vec![VisitResult::Continue],
                VisitResult::Continue,
            );
            rows += 1;
            check(
                content(result.expect("literal visitor")),
                expected,
                "literal context",
                &mut failures,
            );
            check_calls(&trace, 2, 2, 2, &mut failures);
        }
    }
    finish("literal-context-controls", rows, 64, &failures);
}

#[test]
fn should_preserve_nested_preview_flattening_and_emission_walks_in_compact_output() {
    let mut rows = 0;
    let mut failures = Vec::new();
    let source = concat!(
        "<table><tr><th>Outer</th></tr><tr><td>",
        "<div><table><tr><th>Inner</th></tr><tr><td><span>Cheese</span></td></tr></table></div>",
        "</td></tr></table>"
    );
    for options in configurations(true) {
        let expected = content(convert(source, options.clone()).expect("nested control"));
        rows += 1;
        assert!(!expected.is_empty());
        let (result, trace) = observed(
            source,
            options,
            "span",
            vec![VisitResult::Continue],
            VisitResult::Continue,
        );
        rows += 1;
        check(
            content(result.expect("nested visitor")),
            expected,
            "nested table",
            &mut failures,
        );
        check_calls(&trace, 3, 3, 4, &mut failures);
    }
    finish("nested-compact-controls", rows, 16, &failures);
}

#[test]
fn should_preserve_sibling_text_and_expose_corrected_tables_to_end_callbacks() {
    let mut rows = 0;
    let mut failures = Vec::new();
    let source = format!("<div><p>préface</p>{CHEESE}<p>between</p>{CHEESE}<p>tail</p></div>");
    let control = source.replace("<span>from the mountain farm</span>", "");
    for options in all_configurations() {
        let (control_result, control_trace) = observed(
            &control,
            options.clone(),
            "span",
            vec![VisitResult::Continue],
            VisitResult::Continue,
        );
        rows += 1;
        let expected = content(control_result.expect("sibling control"));
        let (result, trace) = observed(
            &source,
            options.clone(),
            "span",
            vec![VisitResult::Skip],
            VisitResult::Continue,
        );
        rows += 1;
        check(
            content(result.expect("sibling tables")),
            &expected,
            "sibling text",
            &mut failures,
        );
        check_calls(&trace, 4, 0, 4, &mut failures);
        for (source, trace) in [("control", &control_trace), ("candidate", &trace)] {
            for (label, actual, expected) in [
                ("typed table end", trace.table_outputs.len(), 2),
                ("generic table end", trace.generic_table_outputs.len(), 0),
                ("generic div end", trace.generic_div_outputs.len(), 1),
            ] {
                if actual != expected {
                    failures.push(format!("{source} {label}: expected {expected} callbacks, got {actual}"));
                }
            }
        }
        let table = expected_table("Cheese", 6, &options);
        for output in trace.table_outputs.iter().chain(&control_trace.table_outputs) {
            check(
                output.trim(),
                table.trim(),
                "table end sees adjusted width",
                &mut failures,
            );
        }
        for (output, expected) in trace.generic_div_outputs.iter().zip(&control_trace.generic_div_outputs) {
            check(output, expected, "div end sees adjusted tables", &mut failures);
        }
    }
    finish("sibling-end-controls", rows, 32, &failures);
}

#[test]
fn should_adjust_nested_table_widths_before_appending_deferred_output() {
    let mut rows = 0;
    let mut failures = Vec::new();
    let source = concat!(
        "<table><tr><th>Outer</th></tr><tr><td>",
        "<div><table><tr><th>Inner</th></tr><tr><td>",
        "Cheese<span>surplus long text</span></td></tr></table></div>",
        "</td></tr></table>"
    );
    for options in configurations(false) {
        let separators = if options.output_format == OutputFormat::Djot {
            ("|-----|\n", "|------|\n")
        } else {
            ("| ----- |\n", "| ------ |\n")
        };
        let expected = format!(
            "| Outer |\n{}|       |\n\n| Inner  |\n{}| Cheese |\n",
            separators.0, separators.1,
        );
        let (result, trace) = observed(source, options, "span", vec![VisitResult::Skip], VisitResult::Continue);
        rows += 1;
        check(
            content(result.expect("nested widths")),
            expected,
            "nested widths",
            &mut failures,
        );
        check_calls(&trace, 3, 0, 4, &mut failures);
    }
    finish("nested-width-controls", rows, 8, &failures);
}
