#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

#[test]
fn should_ignore_source_indentation_between_mathml_tokens() {
    for html in [
        "<p>a <math><mrow><mi>x</mi><mo>=</mo><mi>y</mi></mrow></math> b</p>",
        "<p>a <math>\n  <mrow>\n    <mi>x</mi>\n    <mo>=</mo>\n    <mi>y</mi>\n  </mrow>\n</math> b</p>",
    ] {
        let actual = convert(
            html,
            Some(ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                ..ConversionOptions::default()
            }),
        )
        .expect("math conversion should succeed")
        .content
        .unwrap_or_default();
        assert_eq!(actual, "a x=y b\n");
    }
}

#[test]
fn should_keep_a_source_formatted_fraction_in_its_paragraph() {
    let html = "<p>a <math>\n <mfrac>\n  <mrow>\n   <mi>d</mi>\n   <mi>x</mi>\n  </mrow>\n  <mrow>\n   <mi>d</mi>\n   <mi>y</mi>\n  </mrow>\n </mfrac>\n</math> b</p>";
    let actual = convert(html, None)
        .expect("math conversion should succeed")
        .content
        .unwrap_or_default();
    assert_eq!(actual, "a dxdy b\n");
}

#[test]
fn should_preserve_spaces_inside_mathml_text_tokens() {
    for html in [
        "<p>a <math>\n <mtext>for all x</mtext>\n <mo>=</mo>\n <mi>y</mi>\n</math> b</p>",
        "<p>a <math>\n <mtext>for\n  all x</mtext>\n <mo>=</mo>\n <mi>y</mi>\n</math> b</p>",
        "<p>a <math>\n <mtext>for <b>all</b> x</mtext>\n <mo>=</mo>\n <mi>y</mi>\n</math> b</p>",
    ] {
        let actual = convert(html, None)
            .expect("math conversion should succeed")
            .content
            .unwrap_or_default();
        assert_eq!(actual, "a for all x=y b\n");
    }
}

#[test]
fn should_preserve_explicit_nonbreaking_spaces_in_mathml_tokens() {
    let html = "<math><mi>x</mi><mtext>&nbsp;</mtext><mi>y</mi></math>";
    let actual = convert(html, None)
        .expect("math conversion should succeed")
        .content
        .unwrap_or_default();
    assert_eq!(actual, "x\u{a0}y\n");
}
