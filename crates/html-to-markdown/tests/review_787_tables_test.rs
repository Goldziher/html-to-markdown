#![allow(missing_docs)]
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert, prescan, tier1};
#[test]
fn nested_table_keeps_separation_on_both_sides() {
    for strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
        let out = convert(
            "<table><tr><td><table><tr><td>x</td></tr></table>b</td><td>z</td></tr></table>",
            Some(ConversionOptions {
                tier_strategy: strategy,
                ..Default::default()
            }),
        )
        .unwrap()
        .content
        .unwrap();
        assert_eq!(out, "| x b | z |\n| --- | --- |\n", "{strategy:?}");
    }
}
#[test]
fn empty_nested_cells_add_no_spurious_spaces() {
    for strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
        let out=convert("<table><tr><td>o<table><tr><td></td><td>b</td></tr><tr><td>c</td><td></td></tr></table></td><td>x</td></tr></table>",Some(ConversionOptions{tier_strategy:strategy,..Default::default()})).unwrap().content.unwrap();
        assert_eq!(out.lines().next(), Some("| o b c | x |"), "{strategy:?}");
        assert_eq!(out.lines().count(), 2);
    }
}
#[test]
fn stray_rows_and_sections_keep_their_cells() {
    for wrapper in ["tr", "tbody", "thead", "tfoot"] {
        let html = format!("<div><{wrapper}><td>One</td></{wrapper}>items</div>");
        for strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
            let out = convert(
                &html,
                Some(ConversionOptions {
                    tier_strategy: strategy,
                    ..Default::default()
                }),
            )
            .unwrap()
            .content
            .unwrap();
            assert_eq!(out, "Oneitems\n", "{strategy:?} {wrapper}");
        }
    }
}
#[test]
fn a_visible_label_with_aria_label_stays_on_the_fast_path() {
    let html = r#"<p><a href="/x" aria-label="n">text</a></p>"#;
    let (clean, report) = prescan::run(html);
    assert_eq!(
        tier1::run(&clean, &report, &ConversionOptions::default()).unwrap(),
        "[text](/x)\n"
    );
}
#[test]
fn an_empty_aria_label_link_still_hands_over_for_its_name() {
    let html = r#"<p><a href="/x" aria-label="n"></a></p>"#;
    let (clean, report) = prescan::run(html);
    assert!(matches!(
        tier1::run(&clean, &report, &ConversionOptions::default()),
        Err(tier1::BailReason::LinkEmptyLabel)
    ));
    assert_eq!(convert(html, None).unwrap().content.unwrap(), "[n](/x)\n");
}
#[test]
fn nested_table_boundaries_follow_the_table_break_option() {
    for strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
        for br_in_tables in [false, true] {
            let out = convert(
                "<table><tr><td>a<table><tr><td>x</td></tr></table><code>b</code></td><td>z</td></tr></table>",
                Some(ConversionOptions {
                    tier_strategy: strategy,
                    br_in_tables,
                    ..Default::default()
                }),
            )
            .unwrap()
            .content
            .unwrap();
            let expected = if br_in_tables {
                "| a<br>x<br>`b` | z |"
            } else {
                "| a x `b` | z |"
            };
            assert_eq!(out.lines().next(), Some(expected), "{strategy:?} {br_in_tables}");
        }
    }
}
#[test]
fn empty_nested_rows_do_not_add_a_break() {
    for strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
        for br_in_tables in [false, true] {
            let out=convert("<table><tr><td>a<table><tr><td></td><td></td></tr><tr><td>x</td></tr></table></td><td>z</td></tr></table>",Some(ConversionOptions{tier_strategy:strategy,br_in_tables,..Default::default()})).unwrap().content.unwrap();
            let expected = if br_in_tables { "| a<br>x | z |" } else { "| a x | z |" };
            assert_eq!(out.lines().next(), Some(expected), "{strategy:?} {br_in_tables}");
        }
    }
}
#[test]
fn a_caption_only_nested_table_keeps_its_text_and_boundaries() {
    for strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
        for br_in_tables in [false, true] {
            let out = convert(
                "<table><tr><td>a<table><caption>x</caption></table>b</td><td>z</td></tr></table>",
                Some(ConversionOptions {
                    tier_strategy: strategy,
                    br_in_tables,
                    ..Default::default()
                }),
            )
            .unwrap()
            .content
            .unwrap();
            let expected = if br_in_tables {
                "| a<br>x<br>b | z |"
            } else {
                "| a x b | z |"
            };
            assert_eq!(out.lines().next(), Some(expected), "{strategy:?} {br_in_tables}");
        }
    }
}
#[test]
fn a_wrapped_nested_table_separates_text_inside_its_wrapper() {
    for strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
        for br_in_tables in [false, true] {
            let out = convert(
                "<table><tr><td><div><table><tr><td>x</td></tr></table>b</div></td><td>z</td></tr></table>",
                Some(ConversionOptions {
                    tier_strategy: strategy,
                    br_in_tables,
                    ..Default::default()
                }),
            )
            .unwrap()
            .content
            .unwrap();
            let expected = if br_in_tables { "| x<br>b | z |" } else { "| x b | z |" };
            assert_eq!(out.lines().next(), Some(expected), "{strategy:?} {br_in_tables}");
        }
    }
}
