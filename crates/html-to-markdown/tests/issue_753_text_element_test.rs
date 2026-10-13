//! Coverage for `ConversionOptions::hidden_content` (issue #753) on the elements whose content
//! HTML reads as text.
//!
//! ~keep A `<template>` or a `<noscript>` tag in such text opens no element, so no choice has an
//! ~keep element to keep there.

#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HiddenContent, OutputFormat, TierStrategy, convert};

const CHOICES: [HiddenContent; 3] = [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All];

/// One way to run a conversion: the tier and the output format.
type Mode = (TierStrategy, OutputFormat);

fn modes() -> Vec<Mode> {
    vec![
        (TierStrategy::Auto, OutputFormat::Markdown),
        (TierStrategy::Tier2, OutputFormat::Markdown),
        (TierStrategy::Auto, OutputFormat::Plain),
        #[cfg(feature = "testkit")]
        (TierStrategy::Tier1, OutputFormat::Markdown),
    ]
}

fn convert_in(html: &str, choice: HiddenContent, (tier_strategy, output_format): Mode) -> String {
    let options = ConversionOptions {
        hidden_content: choice,
        tier_strategy,
        output_format,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

/// Convert `html` in each mode under each choice, and give one line for each output that is
/// not the expected one of its choice.
fn wrong_outputs(html: &str, expected: [&str; 3]) -> Vec<String> {
    let mut wrong = Vec::new();
    for mode in modes() {
        for (choice, expected) in CHOICES.into_iter().zip(expected) {
            let output = convert_in(html, choice, mode);
            if output != expected {
                wrong.push(format!("{html} {mode:?} {choice:?}: {output:?}, expected {expected:?}"));
            }
        }
    }
    wrong
}

/// The elements whose content HTML reads as text, and whether the converter writes that text.
const TEXT_ELEMENTS: [(&str, bool); 7] = [
    ("textarea", true),
    ("title", true),
    ("xmp", true),
    ("iframe", false),
    ("plaintext", false),
    ("noembed", false),
    ("noframes", false),
];

#[test]
fn should_leave_a_template_or_a_noscript_tag_in_the_text_of_an_element_as_text() {
    // ~keep Each choice writes what the default writes.
    let mut wrong = Vec::new();
    for (name, writes_its_text) in TEXT_ELEMENTS {
        for text in [
            "<template><p>x</p></template>",
            "<noscript><p>x</p></noscript>",
            r#"<template shadowrootmode="open"><p>x</p></template>"#,
        ] {
            let html = format!("<p>see</p><{name}>{text}</{name}><p>after</p>");
            let expected = if writes_its_text {
                format!("see\n\n{text}\n\nafter\n")
            } else {
                "see\n\nafter\n".to_owned()
            };
            wrong.extend(wrong_outputs(&html, [expected.as_str(); 3]));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn should_write_a_hidden_element_in_the_text_of_an_element_as_each_choice_reads_it() {
    // ~keep The text of a `<textarea>` is text (issue #828), so each choice leaves a tag in it
    // ~keep as written. The converter reads the content of `<plaintext>`, `<noembed>` and
    // ~keep `<noframes>` as markup, so a kept element there converts like the same element that
    // ~keep is not hidden.
    let (as_written, nothing, converted) = (
        "see\n\n<div hidden>x</div>\n\nafter\n",
        "see\n\nafter\n",
        "see\n\nx\n\nafter\n",
    );
    let mut wrong = Vec::new();
    for (name, dropped, kept) in [
        ("textarea", as_written, as_written),
        ("title", as_written, as_written),
        ("xmp", as_written, as_written),
        ("iframe", nothing, nothing),
        ("plaintext", nothing, converted),
        ("noembed", nothing, converted),
        ("noframes", nothing, converted),
    ] {
        let html = format!("<p>see</p><{name}><div hidden>x</div></{name}><p>after</p>");
        wrong.extend(wrong_outputs(&html, [dropped, kept, kept]));
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn should_write_what_a_kept_noscript_with_an_unclosed_iframe_writes_today() {
    // ~keep A limit, not a wanted output. The `<iframe>` has no end tag. A narrower choice
    // ~keep drops the `<noscript>` with the `<iframe>` in it and writes the text after it. With
    // ~keep `all` the tags of the `<noscript>` are removed, the text of the `<iframe>` runs to
    // ~keep the end of the page, and the text after the `<noscript>` is lost. A browser with
    // ~keep scripting off reads the page the same way. A change of this output must be seen.
    let wrong = wrong_outputs("<noscript>wa<iframe>wb</noscript>wc", ["wc\n", "wc\n", "wa\n"]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
