//! Omitted BRs must not make strict source whitespace into authored breaks. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

const MARKUP_FORMATS: [OutputFormat; 2] = [OutputFormat::Markdown, OutputFormat::Djot];
#[cfg(feature = "visitor")]
const ALL_FORMATS: [OutputFormat; 3] = [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain];

fn configurations(formats: &[OutputFormat]) -> Vec<(OutputFormat, NewlineStyle)> {
    let mut cases = Vec::new();
    for format in formats {
        for style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
            cases.push((*format, style));
        }
    }
    cases
}

fn strict_options(format: OutputFormat, style: NewlineStyle, tier: TierStrategy, excluded: bool) -> ConversionOptions {
    ConversionOptions {
        whitespace_mode: WhitespaceMode::Strict,
        tier_strategy: tier,
        output_format: format,
        newline_style: style,
        exclude_selectors: if excluded { vec!["i".into()] } else { Vec::new() },
        ..Default::default()
    }
}

fn assert_content(source: &str, options: ConversionOptions, expected: &str) {
    let result = convert(source, options).expect("strict break conversion");
    assert_eq!(result.content.as_deref(), Some(expected), "{source}");
}

fn break_marker(format: OutputFormat, style: NewlineStyle) -> &'static str {
    if format == OutputFormat::Djot || style == NewlineStyle::Backslash {
        "\\\n"
    } else {
        "  \n"
    }
}

#[cfg(feature = "visitor")]
#[derive(Clone, Copy, Debug)]
enum Response {
    Continue,
    Newline,
    Text,
    Empty,
    EmptyAt(usize),
}

#[cfg(feature = "visitor")]
#[derive(Debug)]
struct BreakVisitor {
    response: Response,
    calls: usize,
}

#[cfg(feature = "visitor")]
impl html_to_markdown_rs::visitor::HtmlVisitor for BreakVisitor {
    fn visit_line_break(
        &mut self,
        _: &html_to_markdown_rs::visitor::NodeContext,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        use html_to_markdown_rs::visitor::VisitResult;
        let index = self.calls;
        self.calls += 1;
        match self.response {
            Response::Newline => VisitResult::Custom("\n".into()),
            Response::Text => VisitResult::Custom("V".into()),
            Response::Empty => VisitResult::Custom(String::new()),
            Response::EmptyAt(empty_at) if index == empty_at => VisitResult::Custom(String::new()),
            Response::Continue | Response::EmptyAt(_) => VisitResult::Continue,
        }
    }
}

#[cfg(feature = "visitor")]
fn assert_callback(source: &str, mut options: ConversionOptions, response: Response, expected: &str, calls: usize) {
    use std::sync::{Arc, Mutex};
    let visitor = Arc::new(Mutex::new(BreakVisitor { response, calls: 0 }));
    options.visitor = Some(visitor.clone());
    assert_content(source, options, expected);
    assert_eq!(
        visitor.lock().expect("callback count").calls,
        calls,
        "{source} {response:?}"
    );
}

#[cfg(feature = "visitor")]
fn expected_break_control(format: OutputFormat, style: NewlineStyle, response: Response) -> String {
    if format == OutputFormat::Plain {
        return "a\n\nz\n".into();
    }
    if matches!(response, Response::Newline | Response::Empty) {
        return "a z\n".into();
    }
    let body = match (response, format) {
        (Response::Text, OutputFormat::Djot) => "*V*",
        (Response::Text, _) => "**V**",
        _ => "",
    };
    format!("a{}{body}z\n", break_marker(format, style))
}

#[cfg(feature = "visitor")]
#[test]
fn should_collapse_strict_blank_wrappers_when_the_break_callback_is_empty() {
    let mut rows = 0;
    for source in ["<div>a<b>\n<br></b>z</div>", "<div>a<b>\n<span><br></span></b>z</div>"] {
        for (format, style) in configurations(&MARKUP_FORMATS) {
            assert_callback(
                source,
                strict_options(format, style, TierStrategy::Tier2, false),
                Response::Empty,
                "a z\n",
                1,
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 8);
}

#[test]
fn should_collapse_strict_blank_wrappers_when_the_break_subtree_is_excluded() {
    let mut rows = 0;
    for source in [
        "<div>a<b>\n<i><br></i></b>z</div>",
        "<div>a<b>\n<i><span><br></span></i></b>z</div>",
    ] {
        for (format, style) in configurations(&MARKUP_FORMATS) {
            assert_content(
                source,
                strict_options(format, style, TierStrategy::Tier2, true),
                "a z\n",
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 8);
}

#[cfg(feature = "visitor")]
#[test]
fn should_retain_nonempty_break_callbacks_and_plain_empty_controls() {
    let mut rows = 0;
    for (format, style) in configurations(&ALL_FORMATS) {
        for response in [Response::Continue, Response::Newline, Response::Text, Response::Empty] {
            assert_callback(
                "<div>a<b>\n<br></b>z</div>",
                strict_options(format, style, TierStrategy::Tier2, false),
                response,
                &expected_break_control(format, style, response),
                1,
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 24);
}

#[test]
fn should_preserve_plain_text_spacing_with_excluded_breaks() {
    for style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
        assert_content(
            "<div>a<b>\n<i><br></i></b>z</div>",
            strict_options(OutputFormat::Plain, style, TierStrategy::Tier2, true),
            "a\nz\n",
        );
    }
}

#[cfg(feature = "visitor")]
fn omitted_break_output(format: OutputFormat, excluded: bool) -> &'static str {
    if format != OutputFormat::Plain {
        return "a z\n";
    }
    if excluded { "a\nz\n" } else { "a\n\nz\n" }
}

#[cfg(feature = "visitor")]
#[test]
fn should_collapse_omitted_breaks_on_all_routes_with_plain_controls() {
    let mut rows = 0;
    for (source, excluded) in [
        ("<div>a<b>\n<br></b>z</div>", false),
        ("<div>a<b>\n<span><br></span></b>z</div>", false),
        ("<div>a<b>\n<i><br></i></b>z</div>", true),
        ("<div>a<b>\n<i><span><br></span></i></b>z</div>", true),
    ] {
        for tier in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            for (format, style) in configurations(&ALL_FORMATS) {
                let mut options = strict_options(format, style, tier, excluded);
                options.extract_metadata = true;
                options.include_document_structure = true;
                assert_callback(
                    source,
                    options,
                    Response::Empty,
                    omitted_break_output(format, excluded),
                    usize::from(!excluded),
                );
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 72);
}

#[cfg(feature = "visitor")]
#[test]
fn should_keep_retained_sibling_breaks_in_either_callback_order() {
    let mut rows = 0;
    for empty_at in [0, 1] {
        for (format, style) in configurations(&MARKUP_FORMATS) {
            assert_callback(
                "<div>a<b>\n<span><br></span><i><br></i></b>z</div>",
                strict_options(format, style, TierStrategy::Tier2, false),
                Response::EmptyAt(empty_at),
                &expected_break_control(format, style, Response::Continue),
                2,
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 8);
}

#[test]
fn should_keep_retained_sibling_breaks_beside_excluded_subtrees() {
    let mut rows = 0;
    for source in [
        "<div>a<b>\n<i><br></i><span><br></span></b>z</div>",
        "<div>a<b>\n<span><br></span><i><br></i></b>z</div>",
    ] {
        for (format, style) in configurations(&MARKUP_FORMATS) {
            assert_content(
                source,
                strict_options(format, style, TierStrategy::Tier2, true),
                &format!("a{}z\n", break_marker(format, style)),
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 8);
}
