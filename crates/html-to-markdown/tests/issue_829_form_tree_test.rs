#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

const OPEN_LABEL: &str = "<p>before<form><label>name<input></form>after<p>next";
const CLOSED_LABEL: &str = "<p>before<form><label>name<input></label></form>after<p>next";

#[test]
fn should_follow_html5_form_tree_when_removing_forms_on_every_route() {
    let cases = [
        (OPEN_LABEL, "before\n"),
        (CLOSED_LABEL, "before\n\nafter\n\nnext\n"),
        (
            "<p>before<form>outer<form>inner</form>after</form><p>next",
            "before\n\nafter\n\nnext\n",
        ),
    ];
    let mut failures = Vec::new();
    for (html, expected) in cases {
        for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
            for extract_metadata in [false, true] {
                for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
                    let result = convert(
                        html,
                        ConversionOptions {
                            output_format,
                            tier_strategy,
                            extract_metadata,
                            ..Default::default()
                        },
                    )
                    .expect("form document converts");
                    if result.content.as_deref() != Some(expected) {
                        failures.push(format!(
                            "{tier_strategy:?}/{output_format:?}/metadata={extract_metadata}: {html}: {:?}",
                            result.content
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn should_leave_removed_form_tree_construction_to_full_converter() {
    let report = html_to_markdown_rs::prescan::PrescanReport::default();
    for html in [OPEN_LABEL, CLOSED_LABEL] {
        let options = ConversionOptions::default();
        assert!(
            matches!(
                html_to_markdown_rs::tier1::run(html, &report, &options),
                Err(html_to_markdown_rs::tier1::BailReason::Classifier)
            ),
            "removed form requires HTML5 tree construction: {html}"
        );
    }
}

#[test]
fn should_keep_form_text_when_form_removal_is_disabled() {
    for (html, expected_content) in [
        (OPEN_LABEL, "before\n\nnameafter\n\nnext\n"),
        (CLOSED_LABEL, "before\n\nname\n\nafter\n\nnext\n"),
    ] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
            let mut options = ConversionOptions {
                output_format,
                extract_metadata: false,
                tier_strategy: TierStrategy::Tier2,
                ..Default::default()
            };
            options.preprocessing.remove_forms = false;
            let expected = convert(html, options.clone()).expect("full conversion").content;
            assert_eq!(expected.as_deref(), Some(expected_content));
            for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1] {
                options.tier_strategy = tier_strategy;
                assert_eq!(convert(html, options.clone()).expect("conversion").content, expected);
            }
        }
    }
}

#[test]
fn should_keep_scanning_well_formed_retained_forms() {
    let html = "<form><label>name</label></form><p>after</p>";
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        let mut options = ConversionOptions {
            output_format,
            extract_metadata: false,
            ..Default::default()
        };
        options.preprocessing.remove_forms = false;
        let report = html_to_markdown_rs::prescan::PrescanReport::default();
        assert_eq!(
            html_to_markdown_rs::tier1::run(html, &report, &options).expect("retained form stays on fast scanner"),
            "name\n\nafter\n"
        );
    }
}

#[test]
fn should_keep_retained_forms_transparent_in_plain_output_on_every_route() {
    let cases = [
        (CLOSED_LABEL, "before\n\nnameafter\n\nnext\n"),
        (
            "<div>before<form><label>name</label></form>after</div>",
            "beforenameafter\n",
        ),
        (
            "<div>before<form><label>name</label></form><span>after</span></div>",
            "beforenameafter\n",
        ),
        (
            "<div>before <form><label>name</label></form> after</div>",
            "before name after\n",
        ),
        (
            "<div>before\n<form><label>name</label></form>after</div>",
            "before\nnameafter\n",
        ),
        ("<form><p>name</p></form>after", "name\n\nafter\n"),
        ("<form><p>name</p></form><span>after</span>", "name\n\nafter\n"),
    ];
    let mut options = ConversionOptions {
        output_format: OutputFormat::Plain,
        extract_metadata: false,
        ..Default::default()
    };
    options.preprocessing.remove_forms = false;
    let report = html_to_markdown_rs::prescan::PrescanReport::default();
    let mut failures = Vec::new();
    for (html, expected) in cases {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            options.tier_strategy = tier_strategy;
            let result = convert(html, options.clone()).expect("retained plain form converts");
            if result.content.as_deref() != Some(expected) {
                failures.push(format!(
                    "{tier_strategy:?}: {html}: {:?}, expected {expected:?}",
                    result.content
                ));
            }
        }
        let scanned = html_to_markdown_rs::tier1::run(html, &report, &options).expect("plain form stays on scanner");
        if scanned != expected {
            failures.push(format!("raw scanner: {html}: {scanned:?}, expected {expected:?}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
