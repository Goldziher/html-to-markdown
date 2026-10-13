//! Empty list blocks retain Unicode prefixes beside structural indentation. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

const TAGS: [&str; 6] = ["figcaption", "address", "fieldset", "center", "dialog", "hgroup"];

fn route_options() -> Vec<ConversionOptions> {
    let mut options = Vec::new();
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
            for (newline_style, whitespace_mode) in [
                (NewlineStyle::Spaces, WhitespaceMode::Normalized),
                (NewlineStyle::Spaces, WhitespaceMode::Strict),
                (NewlineStyle::Backslash, WhitespaceMode::Normalized),
                (NewlineStyle::Backslash, WhitespaceMode::Strict),
            ] {
                options.push(ConversionOptions {
                    tier_strategy,
                    output_format,
                    newline_style,
                    whitespace_mode,
                    extract_metadata: false,
                    ..Default::default()
                });
            }
        }
    }
    assert_eq!(options.len(), 36);
    options
}

const fn observers() -> &'static [bool] {
    if cfg!(feature = "visitor") {
        &[false, true]
    } else {
        &[false]
    }
}

#[cfg(feature = "visitor")]
#[derive(Debug)]
struct Hooks {
    tag: &'static str,
    counts: (usize, usize, usize),
}

#[cfg(feature = "visitor")]
impl html_to_markdown_rs::visitor::HtmlVisitor for Hooks {
    fn visit_element_start(
        &mut self,
        context: &html_to_markdown_rs::visitor::NodeContext,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        self.counts.0 += 1;
        self.counts.1 += usize::from(context.tag_name == self.tag);
        html_to_markdown_rs::visitor::VisitResult::Continue
    }
    fn visit_line_break(
        &mut self,
        _: &html_to_markdown_rs::visitor::NodeContext,
    ) -> html_to_markdown_rs::visitor::VisitResult {
        self.counts.2 += 1;
        html_to_markdown_rs::visitor::VisitResult::Continue
    }
}

fn assert_case(
    source: &str,
    tag: &'static str,
    options: ConversionOptions,
    observed: bool,
    expected: &str,
    expected_counts: (usize, usize, usize),
) {
    #[cfg(feature = "visitor")]
    let (options, hooks) = {
        let hooks = std::sync::Arc::new(std::sync::Mutex::new(Hooks { tag, counts: (0, 0, 0) }));
        let mut options = options;
        if observed {
            options.visitor = Some(hooks.clone());
        }
        (options, hooks)
    };
    #[cfg(not(feature = "visitor"))]
    {
        assert!(!observed);
        let _ = (tag, expected_counts);
    }
    let result = convert(source, options).expect("list block continuation");
    assert_eq!(
        result.content.as_deref(),
        Some(expected),
        "{source:?}, observed={observed}"
    );
    #[cfg(feature = "visitor")]
    assert_eq!(
        hooks.lock().expect("callback counts").counts,
        if observed { expected_counts } else { (0, 0, 0) },
        "{source:?}"
    );
}

fn list_counts(options: &ConversionOptions) -> (usize, usize, usize) {
    if options.output_format == OutputFormat::Plain {
        (9, 2, 0)
    } else {
        (5, 1, 0)
    }
}

fn normalized_markup(options: &ConversionOptions) -> bool {
    options.whitespace_mode == WhitespaceMode::Normalized && options.output_format != OutputFormat::Plain
}

fn assert_unicode_prefix(tag: &'static str) {
    let mut rows = 0;
    for tail in ["&nbsp;b", "\u{a0}b"] {
        let source = format!("<ul><li>a \n<span><{tag}></{tag}></span>{tail}</li></ul>");
        for options in route_options().into_iter().filter(normalized_markup) {
            for &observed in observers() {
                assert_case(&source, tag, options.clone(), observed, "- a\n   b\n", (5, 1, 0));
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 24 * observers().len());
}

#[test]
fn should_keep_unicode_prefix_after_empty_figcaptions() {
    assert_unicode_prefix("figcaption");
}
#[test]
fn should_keep_unicode_prefix_after_empty_addresses() {
    assert_unicode_prefix("address");
}
#[test]
fn should_keep_unicode_prefix_after_empty_fieldsets() {
    assert_unicode_prefix("fieldset");
}
#[test]
fn should_keep_unicode_prefix_after_empty_centers() {
    assert_unicode_prefix("center");
}
#[test]
fn should_keep_unicode_prefix_after_empty_dialogs() {
    assert_unicode_prefix("dialog");
}
#[test]
fn should_keep_unicode_prefix_after_empty_heading_groups() {
    assert_unicode_prefix("hgroup");
}

fn plain_empty_list(tag: &str) -> &'static str {
    if tag == "fieldset" { "- a\n b\n" } else { "- a\n\n b\n" }
}

#[test]
fn should_preserve_strict_and_plain_empty_block_continuations() {
    let mut rows = 0;
    for tag in TAGS {
        for tail in ["&nbsp;b", "\u{a0}b"] {
            let source = format!("<ul><li>a \n<span><{tag}></{tag}></span>{tail}</li></ul>");
            for options in route_options()
                .into_iter()
                .filter(|options| !normalized_markup(options))
            {
                let expected = if options.output_format == OutputFormat::Plain {
                    plain_empty_list(tag)
                } else {
                    "- a\n  \u{a0}b\n"
                };
                for &observed in observers() {
                    assert_case(&source, tag, options.clone(), observed, expected, list_counts(&options));
                    rows += 1;
                }
            }
        }
    }
    assert_eq!(rows, 288 * observers().len());
}

#[test]
fn should_preserve_ascii_spaces_after_empty_list_blocks() {
    let mut rows = 0;
    for tag in TAGS {
        let source = format!("<ul><li>a \n<span><{tag}></{tag}></span> b</li></ul>");
        for options in route_options() {
            let expected = if options.output_format == OutputFormat::Plain {
                plain_empty_list(tag)
            } else if options.whitespace_mode == WhitespaceMode::Strict {
                "- a\n   b\n"
            } else {
                "- a\n  b\n"
            };
            for &observed in observers() {
                assert_case(&source, tag, options.clone(), observed, expected, list_counts(&options));
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 216 * observers().len());
}

fn nonlist_expected(tag: &str, options: &ConversionOptions) -> (&'static str, (usize, usize, usize)) {
    if options.tier_strategy == TierStrategy::Tier1 && tag != "figcaption" {
        return ("a\n\nb\n", (0, 0, 0));
    }
    if options.output_format == OutputFormat::Plain {
        return (if tag == "fieldset" { "a\n b\n" } else { "a\n\n b\n" }, (7, 2, 0));
    }
    let expected = if options.whitespace_mode == WhitespaceMode::Strict {
        "a\n\n\u{a0}b\n"
    } else if tag == "figcaption" {
        "a b\n"
    } else {
        "a\n\nb\n"
    };
    (expected, (4, 1, 0))
}

#[test]
fn should_preserve_existing_nonlist_block_spacing_and_route_fallbacks() {
    let mut rows = 0;
    for tag in TAGS {
        let source = format!("<div>a \n<span><{tag}></{tag}></span>&nbsp;b</div>");
        for options in route_options() {
            let (expected, counts) = nonlist_expected(tag, &options);
            for &observed in observers() {
                assert_case(&source, tag, options.clone(), observed, expected, counts);
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 216 * observers().len());
}

fn retained_expected(tag: &str, options: &ConversionOptions) -> &'static str {
    if options.output_format == OutputFormat::Plain {
        return if tag == "fieldset" {
            "- a\nx b\n"
        } else {
            "- a\n\nx\n\n b\n"
        };
    }
    match (tag, options.whitespace_mode) {
        ("figcaption", WhitespaceMode::Normalized) => "- a\n\n*x*\n\nb\n",
        ("figcaption", WhitespaceMode::Strict) => "- a\n\n*x*\n\n\u{a0}b\n",
        (_, WhitespaceMode::Normalized) => "- a\n\n  x\n\n  b\n",
        (_, WhitespaceMode::Strict) => "- a\n\n  x\n\n  \u{a0}b\n",
    }
}

#[test]
fn should_preserve_prefixes_after_blocks_with_retained_text() {
    let mut rows = 0;
    for tag in TAGS {
        let source = format!("<ul><li>a \n<span><{tag}>x</{tag}></span>&nbsp;b</li></ul>");
        for options in route_options() {
            let expected = retained_expected(tag, &options);
            for &observed in observers() {
                assert_case(&source, tag, options.clone(), observed, expected, list_counts(&options));
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 216 * observers().len());
}

#[test]
fn should_keep_header_custom_and_covered_block_prefix_contracts() {
    let mut rows = 0;
    for (tag, expected) in [
        ("header", "- a\n   b\n"),
        ("x-y", "- a b\n"),
        ("h1", "- a\n   b\n"),
        ("section", "- a\n   b\n"),
    ] {
        let source = format!("<ul><li>a \n<span><{tag}></{tag}></span>&nbsp;b</li></ul>");
        for options in route_options().into_iter().filter(normalized_markup) {
            for &observed in observers() {
                assert_case(&source, tag, options.clone(), observed, expected, (5, 1, 0));
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 48 * observers().len());
}
