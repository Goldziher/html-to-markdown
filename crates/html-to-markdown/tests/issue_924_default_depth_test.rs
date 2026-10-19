#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, ConversionResult, OutputFormat, TierStrategy, WarningKind, convert};

const THREAD_STACK_BYTES: usize = 2 * 1024 * 1024;
const DEFAULT_DEPTH: usize = 1024;

fn nested_paragraph(depth: usize) -> String {
    format!(
        "<html><body>{}<p>The paragraph survives.</p>{}</body></html>",
        "<div>".repeat(depth),
        "</div>".repeat(depth)
    )
}

fn convert_on_small_stack(html: String, options: Option<ConversionOptions>) -> ConversionResult {
    std::thread::Builder::new()
        .stack_size(THREAD_STACK_BYTES)
        .spawn(move || convert(&html, options).expect("conversion should succeed"))
        .expect("spawn conversion thread")
        .join()
        .expect("conversion should complete on a 2 MiB stack")
}

fn decoded_svg(result: &ConversionResult) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    let markdown = result.content.as_deref().expect("SVG image output");
    let encoded = markdown
        .split_once("base64,")
        .expect("base64 SVG URI")
        .1
        .split(')')
        .next()
        .expect("base64 payload");
    String::from_utf8(STANDARD.decode(encoded).expect("valid base64")).expect("UTF-8 SVG")
}

#[test]
fn should_keep_a_paragraph_inside_sixty_one_divs_with_default_options() {
    let result = convert_on_small_stack(nested_paragraph(61), None);
    assert_eq!(result.content.as_deref(), Some("The paragraph survives.\n"));
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
}

#[test]
fn should_keep_a_thousand_divs_on_a_two_mib_stack_in_each_conversion_mode() {
    for tier_strategy in [
        TierStrategy::Auto,
        TierStrategy::Tier2,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
    ] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Plain] {
            let result = convert_on_small_stack(
                nested_paragraph(1000),
                Some(ConversionOptions {
                    tier_strategy,
                    output_format,
                    ..Default::default()
                }),
            );
            assert_eq!(result.content.as_deref(), Some("The paragraph survives.\n"));
            assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        }
    }
}

#[test]
fn should_honor_an_explicit_low_depth_and_report_the_skipped_content() {
    let html = format!("<p>Before</p>{}<p>After</p>", nested_paragraph(12));
    let result = convert_on_small_stack(
        html,
        Some(ConversionOptions {
            max_depth: Some(8),
            ..Default::default()
        }),
    );
    assert_eq!(result.content.as_deref(), Some("Before\n\nAfter\n"));
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.warnings[0].kind, WarningKind::DepthLimitExceeded);
    assert_eq!(
        result.warnings[0].message,
        "DOM traversal reached the effective depth limit of 8; deeper nodes were skipped."
    );
}

#[test]
fn should_bound_default_and_explicit_depths_at_the_same_absolute_ceiling() {
    for max_depth in [None, Some(DEFAULT_DEPTH), Some(usize::MAX)] {
        let html = format!("<p>Before</p>{}<p>After</p>", nested_paragraph(DEFAULT_DEPTH + 16));
        let result = convert_on_small_stack(
            html,
            Some(ConversionOptions {
                max_depth,
                ..Default::default()
            }),
        );
        assert_eq!(result.content.as_deref(), Some("Before\n\nAfter\n"));
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].kind, WarningKind::DepthLimitExceeded);
        assert_eq!(
            result.warnings[0].message,
            "DOM traversal reached the effective depth limit of 1024; deeper nodes were skipped."
        );
    }
}

#[test]
fn should_keep_recursive_svg_serialization_within_its_separate_depth_guard() {
    for (max_depth, expected_groups) in [(None, 64), (Some(8), 8), (Some(usize::MAX), 64)] {
        let html = format!("<svg>{}{}</svg>", "<g>".repeat(1000), "</g>".repeat(1000));
        let result = convert_on_small_stack(
            html,
            Some(ConversionOptions {
                max_depth,
                ..Default::default()
            }),
        );
        let svg = decoded_svg(&result);
        assert_eq!(svg.matches("<g>").count(), expected_groups, "{max_depth:?}");
    }
}

#[test]
fn should_preserve_a_shallow_svg_inside_deep_html_wrappers_in_each_tier() {
    use html_to_markdown_rs::HighlightStyle;

    let html = format!(
        "{}<svg><title>label</title><path d=\"M0 0\"></path></svg>{}",
        "<div>".repeat(100),
        "</div>".repeat(100)
    );
    for tier_strategy in [
        TierStrategy::Auto,
        TierStrategy::Tier2,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
    ] {
        let result = convert_on_small_stack(
            html.clone(),
            Some(ConversionOptions {
                tier_strategy,
                extract_metadata: false,
                highlight_style: HighlightStyle::None,
                ..Default::default()
            }),
        );
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        let svg = decoded_svg(&result);
        assert_eq!(
            svg, "<svg><title>label</title><path d=\"M0 0\" /></svg>",
            "{tier_strategy:?}"
        );
    }
}

#[test]
fn should_limit_nested_svg_serialization_by_its_remaining_conversion_budget() {
    for (max_depth, expected_groups) in [(None, 64), (Some(105), 5)] {
        let html = format!(
            "{}<svg>{}{}</svg>{}",
            "<div>".repeat(100),
            "<g>".repeat(1000),
            "</g>".repeat(1000),
            "</div>".repeat(100)
        );
        let result = convert_on_small_stack(
            html,
            Some(ConversionOptions {
                max_depth,
                tier_strategy: TierStrategy::Tier2,
                ..Default::default()
            }),
        );
        assert_eq!(decoded_svg(&result).matches("<g>").count(), expected_groups);
    }
}
