#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, WarningKind, WhitespaceMode, convert};

#[test]
fn should_skip_flat_table_sections_and_rows_at_the_depth_limit() {
    let html = "<table><tr><td>before<table><tbody><tr><td>payload</td></tr></tbody></table>after</td><td>side</td></tr></table>";
    for max_depth in [4, 5] {
        let result = convert(
            html,
            ConversionOptions {
                max_depth: Some(max_depth),
                compact_tables: true,
                extract_metadata: false,
                whitespace_mode: WhitespaceMode::Strict,
                tier_strategy: TierStrategy::Tier2,
                ..Default::default()
            },
        )
        .expect("conversion succeeds");
        assert_eq!(
            result.content.as_deref(),
            Some("| before after | side |\n| --- | --- |\n")
        );
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].kind, WarningKind::DepthLimitExceeded);
    }
}

#[test]
fn should_walk_flat_table_caption_children_at_their_original_depth() {
    let html = "<table><tr><td>before<table><caption><span>x</span></caption><tr><td>payload</td></tr></table>after</td><td>side</td></tr></table>";
    let result = convert(
        html,
        ConversionOptions {
            max_depth: Some(6),
            compact_tables: true,
            extract_metadata: false,
            whitespace_mode: WhitespaceMode::Strict,
            tier_strategy: TierStrategy::Tier2,
            ..Default::default()
        },
    )
    .expect("conversion succeeds");
    assert_eq!(
        result.content.as_deref(),
        Some("| before payload after | side |\n| --- | --- |\n")
    );
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.warnings[0].kind, WarningKind::DepthLimitExceeded);
}
