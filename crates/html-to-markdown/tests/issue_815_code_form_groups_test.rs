#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, PreprocessingOptions, TierStrategy, convert};

fn markdown(html: &str, tier_strategy: TierStrategy) -> String {
    convert(
        html,
        Some(ConversionOptions {
            extract_metadata: false,
            compact_tables: true,
            tier_strategy,
            preprocessing: PreprocessingOptions {
                remove_forms: false,
                ..PreprocessingOptions::default()
            },
            ..ConversionOptions::default()
        }),
    )
    .expect("conversion should succeed")
    .content
    .unwrap_or_default()
}

#[test]
fn should_separate_legend_and_button_inside_heading_code() {
    let html = "<h2>ccc <code>aaa<fieldset><legend>Xone</legend><input><button>Xtwo</button></fieldset>bbb</code></h2>";
    for tier in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
        assert_eq!(markdown(html, tier), "## ccc `aaa Xone Xtwo bbb`\n", "{tier:?}");
    }
}

#[test]
fn should_keep_table_code_form_group_in_one_span_without_emphasis() {
    let html = "<table><tr><th>hhh</th></tr><tr><td><code>aaa<fieldset><legend>Xone</legend><input><button>Xtwo</button></fieldset>bbb</code></td></tr></table>";
    for tier in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
        assert_eq!(
            markdown(html, tier),
            "| hhh |\n| --- |\n| `aaa Xone Xtwo bbb` |\n",
            "{tier:?}"
        );
    }
}

#[test]
fn should_keep_form_group_block_emphasis_outside_code() {
    let html = "<div>aaa<fieldset><legend>Xone</legend><input><button>Xtwo</button></fieldset>bbb</div>";
    for tier in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
        assert_eq!(markdown(html, tier), "aaa\n\n**Xone**\n\nXtwo\n\nbbb\n", "{tier:?}");
    }
}

#[test]
fn should_not_add_spaces_for_an_empty_form_group_inside_code() {
    for tier in [TierStrategy::Tier1, TierStrategy::Tier2, TierStrategy::Auto] {
        assert_eq!(
            markdown(
                "<p><code>aaa<fieldset><legend></legend><input></fieldset>bbb</code></p>",
                tier
            ),
            "`aaabbb`\n",
            "{tier:?}"
        );
    }
}
