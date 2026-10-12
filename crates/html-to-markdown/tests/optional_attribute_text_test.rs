#![allow(missing_docs)]
#![cfg(any(feature = "serde", feature = "metadata"))]

//! ~keep Attribute text options and quote spacing regressions (#780, #761).

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn markdown(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion succeeds")
        .content
        .unwrap_or_default()
}

#[test]
fn should_omit_abbreviation_titles_when_disabled_in_both_tiers() {
    let html = r#"<p>It displays <abbr title="last-in, first-out">LIFO</abbr>.</p><p><a href="/faq"><abbr title="Frequently asked questions">FAQ</abbr></a></p>"#;
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
        let mut options: ConversionOptions =
            serde_json::from_str(r#"{"expand_abbreviations":false}"#).expect("option is supported");
        options.tier_strategy = tier_strategy;
        assert_eq!(markdown(html, options), "It displays LIFO.\n\n[FAQ](/faq)\n");
        assert_eq!(
            markdown(
                html,
                ConversionOptions {
                    tier_strategy,
                    ..Default::default()
                }
            ),
            "It displays LIFO (last-in, first-out).\n\n[FAQ (Frequently asked questions)](/faq)\n"
        );
    }
}

#[test]
fn should_omit_blockquote_citations_when_disabled() {
    let html = r#"<blockquote cite="https://example.com/src"><p>quoted words</p></blockquote>"#;
    let options: ConversionOptions =
        serde_json::from_str(r#"{"include_blockquote_citations":false}"#).expect("option is supported");
    assert_eq!(markdown(html, options), "> quoted words\n");
    assert_eq!(
        markdown(html, ConversionOptions::default()),
        "> quoted words\n\n— <https://example.com/src>\n"
    );
}

#[test]
fn should_write_one_empty_quote_line_between_a_list_and_footer() {
    let html = "<blockquote><p>quoted words</p><ul><li>item</li></ul><footer>Someone</footer></blockquote>";
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
        assert_eq!(
            markdown(
                html,
                ConversionOptions {
                    tier_strategy,
                    ..Default::default()
                }
            ),
            "> quoted words\n>\n> - item\n>\n> Someone\n"
        );
    }
}

#[test]
fn should_preserve_authored_blank_lines_inside_quote_code() {
    let html = "<blockquote><pre><code>one\n\n\ntwo</code></pre></blockquote>";
    assert_eq!(
        markdown(html, ConversionOptions::default()),
        "> ```\n> one\n>\n>\n> two\n> ```\n"
    );
}

#[test]
fn should_apply_attribute_text_options_through_builders_and_partial_updates() {
    let options = ConversionOptions::builder()
        .expand_abbreviations(false)
        .include_blockquote_citations(false)
        .build();
    assert!(!options.expand_abbreviations);
    assert!(!options.include_blockquote_citations);
    let update = serde_json::from_str(r#"{"expand_abbreviations":false,"include_blockquote_citations":false}"#)
        .expect("partial options supported");
    let options = ConversionOptions::from_update(update);
    assert!(!options.expand_abbreviations);
    assert!(!options.include_blockquote_citations);
}
