#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, convert};

fn markdown(html: &str) -> String {
    let options = ConversionOptions {
        extract_metadata: false,
        br_in_tables: true,
        compact_tables: true,
        ..ConversionOptions::default()
    };
    markdown_with_options(html, options)
}

fn markdown_with_options(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

#[test]
fn should_escape_literal_asterisks_inside_bold_table_text() {
    assert_eq!(
        markdown("<table><tr><td><b>LUX* Chongzuo</b></td></tr></table>"),
        "| **LUX\\* Chongzuo** |\n| --- |\n"
    );
}

#[test]
fn should_omit_presentation_tables_containing_only_breaks() {
    for role in ["presentation", "PRESENTATION"] {
        for cell in ["<br>", " <br><br> ", "<span><br></span>"] {
            let html = format!("<table role=\"{role}\"><tr><td>{cell}</td></tr></table>");
            assert_eq!(markdown(&html), "", "{html}");
        }
    }
}

#[test]
fn should_keep_break_only_data_tables() {
    assert_eq!(markdown("<table><tr><td><br></td></tr></table>"), "| <br> |\n| --- |\n");
}

#[test]
fn should_keep_presentation_tables_with_visible_content() {
    assert_eq!(
        markdown("<table role=\"presentation\"><tr><td>Text<br>more</td></tr></table>"),
        "| Text<br>more |\n| --- |\n"
    );
}

#[test]
fn should_escape_bold_table_text_once_with_either_asterisk_option() {
    for escape_asterisks in [false, true] {
        for text in ["LUX* Chongzuo", "LUX&#42; Chongzuo"] {
            let html = format!("<table><tr><td><strong>{text}</strong></td></tr></table>");
            let options = ConversionOptions {
                extract_metadata: false,
                br_in_tables: true,
                compact_tables: true,
                escape_asterisks,
                ..ConversionOptions::default()
            };
            assert_eq!(
                markdown_with_options(&html, options),
                "| **LUX\\* Chongzuo** |\n| --- |\n"
            );
        }
    }
}

#[test]
fn should_leave_code_asterisks_inside_bold_table_text_literal() {
    assert_eq!(
        markdown("<table><tr><td><b>LUX <code>*</code> Chongzuo</b></td></tr></table>"),
        "| **LUX `*` Chongzuo** |\n| --- |\n"
    );
}

#[cfg(feature = "testkit")]
#[test]
fn should_match_table_regressions_across_tiers() {
    use html_to_markdown_rs::TierStrategy;

    let cases = [
        (
            "<table><tr><td><b>LUX* Chongzuo</b></td></tr></table>",
            "| **LUX\\* Chongzuo** |\n| ------------------ |\n",
        ),
        ("<table role=\"presentation\"><tr><td><br></td></tr></table>", ""),
        ("<table><tr><td><br></td></tr></table>", "| <br> |\n| ---- |\n"),
    ];
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for escape_asterisks in [false, true] {
            for (html, expected) in cases {
                let options = ConversionOptions {
                    extract_metadata: false,
                    br_in_tables: true,
                    compact_tables: false,
                    tier_strategy,
                    escape_asterisks,
                    ..ConversionOptions::default()
                };
                assert_eq!(
                    markdown_with_options(html, options),
                    expected,
                    "{tier_strategy:?}: {html}"
                );
            }
        }
    }
}

#[cfg(feature = "testkit")]
#[test]
fn should_match_bold_table_escaping_for_backslashes_and_entities() {
    use html_to_markdown_rs::TierStrategy;

    for text in [
        r"LUX\* Chongzuo",
        r"LUX\\* Chongzuo",
        "LUX&#42; Chongzuo",
        "LUX** Chongzuo",
    ] {
        let html = format!("<table><tr><td><strong>{text}</strong></td></tr></table>");
        let options = ConversionOptions {
            extract_metadata: false,
            br_in_tables: true,
            compact_tables: false,
            tier_strategy: TierStrategy::Tier2,
            ..ConversionOptions::default()
        };
        let expected = markdown_with_options(&html, options.clone());
        let options = ConversionOptions {
            tier_strategy: TierStrategy::Tier1,
            ..options
        };
        assert_eq!(markdown_with_options(&html, options), expected, "{html}");
    }
}
