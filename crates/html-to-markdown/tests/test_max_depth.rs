// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]

//! Tests for the `max_depth` traversal budget.

use html_to_markdown_rs::options::{OutputFormat, TierStrategy};
use html_to_markdown_rs::{ConversionOptions, ConversionResult, WarningKind};

fn convert_with_options(html: &str, options: ConversionOptions) -> ConversionResult {
    html_to_markdown_rs::convert(html, Some(options)).expect("conversion should not fail")
}

fn assert_depth_warning(result: &ConversionResult, max_depth: usize) {
    assert_eq!(result.warnings.len(), 1, "expected exactly one depth warning");
    let warning = &result.warnings[0];
    assert_eq!(warning.kind, WarningKind::DepthLimitExceeded);
    assert_eq!(
        warning.message,
        format!("DOM traversal reached the effective depth limit of {max_depth}; deeper nodes were skipped.")
    );
}

fn nested_divs(depth: usize, inner: &str) -> String {
    let mut html = inner.to_string();
    for _ in 0..depth {
        html = format!("<div>{html}</div>");
    }
    html
}

/// With the default `max_depth: None`, ordinary nesting below the traversal
/// limit should be fully converted.
#[test]
fn test_max_depth_none_converts_reasonably_nested_content() {
    let mut html = String::from("<p>deep</p>");
    for _ in 0..32 {
        html = format!("<div>{html}</div>");
    }

    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: None,
        ..Default::default()
    };

    let result = convert_with_options(&html, options);
    let content = result.content.as_deref().unwrap_or_default();
    assert!(
        content.contains("deep"),
        "Deeply nested text should be present when max_depth is None. Got:\n{content}"
    );
    assert!(result.warnings.is_empty());
}

/// Content at the configured limit is truncated and reported.
#[test]
fn test_max_depth_truncates_at_limit() {
    let html = "<div><p>shallow</p><div><p>deep</p></div></div>";

    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: Some(3),
        ..Default::default()
    };

    let result = convert_with_options(html, options);
    let content = result.content.as_deref().unwrap_or_default();
    assert!(
        content.contains("shallow"),
        "Content at depth < max_depth should be present. Got:\n{content}"
    );
    assert!(
        !content.contains("deep"),
        "Content at depth >= max_depth should be absent. Got:\n{content}"
    );
    assert_depth_warning(&result, 3);
}

/// Issue #434: callers can raise the ceiling above the former native default (64) by
/// setting an explicit `max_depth`, recovering content in deeply-nested email HTML.
#[test]
fn test_max_depth_can_be_raised_above_native_default_issue_434() {
    let html = nested_divs(90, "<p>deep-content</p>");

    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: Some(256),
        ..Default::default()
    };

    let result = convert_with_options(&html, options);
    assert!(
        result.content.as_deref().unwrap_or_default().contains("deep-content"),
        "Raising max_depth above 64 must recover content nested deeper than the native default."
    );
    // ~keep Nothing was truncated (90 < 256), so no depth warning should be emitted.
    assert!(
        result.warnings.is_empty(),
        "No DepthLimitExceeded warning expected when nothing is truncated: {:?}",
        result.warnings
    );
}

/// Issue #434: when the depth limit truncates content, a `DepthLimitExceeded`
/// warning is surfaced (previously the subtree was dropped silently).
#[test]
fn test_depth_truncation_emits_warning_issue_434() {
    let html = nested_divs(90, "<p>deep-content</p>");

    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: Some(64),
        ..Default::default()
    };

    let result = convert_with_options(&html, options);
    assert!(
        !result.content.as_deref().unwrap_or_default().contains("deep-content"),
        "Content nested beyond the explicit depth budget should be truncated."
    );
    assert_depth_warning(&result, 64);
}

/// Multiple truncated subtrees still produce one conversion-level warning.
#[test]
fn test_max_depth_warns_once_for_multiple_truncated_subtrees() {
    let html = "<div><div><p>first</p></div><div><p>second</p></div></div>";
    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: Some(2),
        ..Default::default()
    };

    let result = convert_with_options(html, options);
    let content = result.content.as_deref().unwrap_or_default();
    assert!(!content.contains("first"));
    assert!(!content.contains("second"));
    assert_depth_warning(&result, 2);
}

/// With `max_depth: Some(0)`, no nodes are processed and truncation is reported.
#[test]
fn test_max_depth_zero_produces_empty() {
    let html = "<p>hello</p>";

    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: Some(0),
        ..Default::default()
    };

    let result = convert_with_options(html, options);
    let content = result.content.as_deref().unwrap_or_default();
    assert!(
        content.trim().is_empty(),
        "max_depth: Some(0) should produce no output. Got:\n{content}"
    );
    assert_depth_warning(&result, 0);
}

/// Tier-2 applies its bounded traversal default when no explicit limit is configured.
#[test]
fn test_tier2_default_limit_truncates_and_warns() {
    let html = format!("{}leaf{}", "<div>".repeat(1025), "</div>".repeat(1025));
    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: None,
        tier_strategy: TierStrategy::Tier2,
        ..Default::default()
    };

    let result = convert_with_options(&html, options);
    let content = result.content.as_deref().unwrap_or_default();
    assert!(!content.contains("leaf"));
    assert_depth_warning(&result, 1024);
}

/// Tier-2 processes every node below the default limit without warning.
#[test]
fn test_tier2_default_limit_boundary_does_not_warn() {
    let html = format!("{}leaf{}", "<div>".repeat(1023), "</div>".repeat(1023));
    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: None,
        tier_strategy: TierStrategy::Tier2,
        ..Default::default()
    };

    let result = convert_with_options(&html, options);
    assert!(result.content.as_deref().unwrap_or_default().contains("leaf"));
    assert!(result.warnings.is_empty());
}

/// Plain-text output reports the same default Tier-2 truncation as Markdown output.
#[test]
fn test_plain_text_default_limit_truncates_and_warns() {
    let html = format!("{}leaf{}", "<div>".repeat(1025), "</div>".repeat(1025));
    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: None,
        output_format: OutputFormat::Plain,
        ..Default::default()
    };

    let result = convert_with_options(&html, options);
    assert!(!result.content.as_deref().unwrap_or_default().contains("leaf"));
    assert_depth_warning(&result, 1024);
}

/// A tree whose deepest node is below the configured limit is not truncated.
#[test]
fn test_max_depth_below_limit_does_not_warn() {
    let options = ConversionOptions {
        extract_metadata: false,
        max_depth: Some(3),
        ..Default::default()
    };

    let result = convert_with_options("<div><p>safe</p></div>", options);
    assert!(result.content.as_deref().unwrap_or_default().contains("safe"));
    assert!(result.warnings.is_empty());
}
