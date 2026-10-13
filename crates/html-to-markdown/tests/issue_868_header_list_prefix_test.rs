//! Header prefix repair preserves rendered text and existing block boundaries. ~keep
#![cfg(feature = "testkit")]
#[cfg(feature = "visitor")]
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

type Counts = (usize, usize, usize);

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

fn preprocessing_options(enabled: bool) -> Vec<ConversionOptions> {
    route_options()
        .into_iter()
        .map(|mut options| {
            options.preprocessing.enabled = enabled;
            options
        })
        .collect()
}

const fn observers() -> &'static [bool] {
    if cfg!(feature = "visitor") {
        &[false, true]
    } else {
        &[false]
    }
}

fn normalized_markup(options: &ConversionOptions) -> bool {
    options.whitespace_mode == WhitespaceMode::Normalized && options.output_format != OutputFormat::Plain
}

#[cfg(feature = "visitor")]
#[derive(Debug, Default)]
struct Hooks {
    counts: Counts,
}
#[cfg(feature = "visitor")]
impl HtmlVisitor for Hooks {
    fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
        self.counts.0 += 1;
        self.counts.1 += usize::from(context.tag_name == "header");
        VisitResult::Continue
    }
    fn visit_line_break(&mut self, _: &NodeContext) -> VisitResult {
        self.counts.2 += 1;
        VisitResult::Continue
    }
}

fn assert_case(source: &str, options: ConversionOptions, observed: bool, expected: &str, expected_counts: Counts) {
    #[cfg(feature = "visitor")]
    let (options, hooks) = {
        let hooks = std::sync::Arc::new(std::sync::Mutex::new(Hooks::default()));
        let mut options = options;
        if observed {
            options.visitor = Some(hooks.clone());
        }
        (options, hooks)
    };
    #[cfg(not(feature = "visitor"))]
    {
        assert!(!observed);
        let _ = expected_counts;
    }
    let result = convert(source, options).expect("header list continuation");
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

fn assert_header_prefix(enabled: bool) {
    let mut rows = 0;
    for before in ["a \n", "a\n"] {
        for tail in ["&nbsp;b", "\u{a0}b"] {
            let source = format!("<ul><li>{before}<span><header></header></span>{tail}</li></ul>");
            for options in preprocessing_options(enabled).into_iter().filter(normalized_markup) {
                for &observed in observers() {
                    assert_case(&source, options.clone(), observed, "- a\n   b\n", (5, 1, 0));
                    rows += 1;
                }
            }
        }
    }
    assert_eq!(rows, 48 * observers().len());
}

#[test]
fn should_keep_header_prefix_when_preprocessing_is_disabled() {
    assert_header_prefix(false);
}
#[test]
fn should_keep_header_prefix_without_flattening_default_block_boundaries() {
    assert_header_prefix(true);
}

#[test]
fn should_preserve_strict_and_plain_header_prefixes() {
    let mut rows = 0;
    for enabled in [false, true] {
        for tail in ["&nbsp;b", "\u{a0}b"] {
            let source = format!("<ul><li>a \n<span><header></header></span>{tail}</li></ul>");
            for options in preprocessing_options(enabled)
                .into_iter()
                .filter(|options| !normalized_markup(options))
            {
                let (expected, counts) = if options.output_format == OutputFormat::Plain {
                    ("- a\n\n b\n", (9, 2, 0))
                } else {
                    ("- a\n  \u{a0}b\n", (5, 1, 0))
                };
                for &observed in observers() {
                    assert_case(&source, options.clone(), observed, expected, counts);
                    rows += 1;
                }
            }
        }
    }
    assert_eq!(rows, 96 * observers().len());
}

struct PreservationCase {
    source: &'static str,
    expected: [&'static str; 3],
    counts: [Counts; 2],
    tier1: Option<&'static str>,
}
impl PreservationCase {
    fn expectation(&self, options: &ConversionOptions) -> (&'static str, Counts) {
        if options.tier_strategy == TierStrategy::Tier1
            && let Some(expected) = self.tier1
        {
            return (expected, (0, 0, 0));
        }
        let plain = options.output_format == OutputFormat::Plain;
        let index = if plain {
            2
        } else {
            usize::from(options.whitespace_mode == WhitespaceMode::Strict)
        };
        (self.expected[index], self.counts[usize::from(plain)])
    }
}
const PRESERVATION_CASES: [PreservationCase; 10] = [
    PreservationCase {
        source: "<ul><li>a \n<span><header></header></span> b</li></ul>",
        expected: ["- a\n  b\n", "- a\n   b\n", "- a\n\n b\n"],
        counts: [(5, 1, 0), (9, 2, 0)],
        tier1: None,
    },
    PreservationCase {
        source: "<ul><li>a <span><header></header></span>&nbsp;b</li></ul>",
        expected: ["- a\n   b\n", "- a\n  \u{a0}b\n", "- a\n\n b\n"],
        counts: [(5, 1, 0), (9, 2, 0)],
        tier1: None,
    },
    PreservationCase {
        source: "<ul><li>a \n<header></header>&nbsp;b</li></ul>",
        expected: ["- a\n   b\n", "- a\n  \u{a0}b\n", "- a\n\n b\n"],
        counts: [(3, 1, 0), (6, 2, 0)],
        tier1: None,
    },
    PreservationCase {
        source: "<ul><li>a\n<header></header>&nbsp;b</li></ul>",
        expected: ["- a\n   b\n", "- a\n  \u{a0}b\n", "- a\n\n b\n"],
        counts: [(3, 1, 0), (6, 2, 0)],
        tier1: None,
    },
    PreservationCase {
        source: "<div>a \n<span><header></header></span>&nbsp;b</div>",
        expected: ["a b\n", "a\n\n\u{a0}b\n", "a\n\n b\n"],
        counts: [(4, 1, 0), (7, 2, 0)],
        tier1: Some("a\n\nb\n"),
    },
    PreservationCase {
        source: "<ul><li>a \n<span><header>x</header></span>&nbsp;b</li></ul>",
        expected: ["- a\n\n  x\n\n  b\n", "- a\n\n  x\n\n  \u{a0}b\n", "- a\n\nx\n\n b\n"],
        counts: [(5, 1, 0), (9, 2, 0)],
        tier1: None,
    },
    PreservationCase {
        source: "<div>a \n<span><header>x</header></span>&nbsp;b</div>",
        expected: ["a\n\nx\n\nb\n", "a\n\nx\n\n\u{a0}b\n", "a\n\nx\n\n b\n"],
        counts: [(4, 1, 0), (7, 2, 0)],
        tier1: Some("a\n\nx\n\nb\n"),
    },
    PreservationCase {
        source: "<ul><li>a \n<span><section></section></span>&nbsp;b</li></ul>",
        expected: ["- a\n   b\n", "- a\n  \u{a0}b\n", "- a\n\n b\n"],
        counts: [(5, 0, 0), (9, 0, 0)],
        tier1: None,
    },
    PreservationCase {
        source: "<ul><li>a \n<span><x-y></x-y></span>&nbsp;b</li></ul>",
        expected: ["- a b\n", "- a\n  \u{a0}b\n", "- a\n b\n"],
        counts: [(5, 0, 0), (9, 0, 0)],
        tier1: None,
    },
    PreservationCase {
        source: "<ul><li>a \n<span><x-y>x</x-y></span>&nbsp;b</li></ul>",
        expected: ["- a x b\n", "- a\n  x\u{a0}b\n", "- a\nx b\n"],
        counts: [(5, 0, 0), (9, 0, 0)],
        tier1: Some("- a x b\n"),
    },
];

#[test]
fn should_preserve_direct_nolf_retained_text_and_custom_header_contracts() {
    let mut rows = 0;
    for case in PRESERVATION_CASES {
        for enabled in [false, true] {
            for options in preprocessing_options(enabled) {
                let (expected, counts) = case.expectation(&options);
                for &observed in observers() {
                    assert_case(case.source, options.clone(), observed, expected, counts);
                    rows += 1;
                }
            }
        }
    }
    assert_eq!(rows, 720 * observers().len());
}
