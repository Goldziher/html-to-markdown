//! Coverage for `ConversionOptions::hidden_content` (issue #753), which chooses whether the output
//! holds the text that a browser does not show at first.

#![allow(missing_docs)]

mod support;

use html_to_markdown_rs::{
    ConversionOptions, ConversionOptionsUpdate, HiddenContent, OutputFormat, TierStrategy, convert,
};

const CHOICES: [HiddenContent; 3] = [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All];

/// One way to run a conversion: the tier and the output format.
#[derive(Clone, Copy, Debug)]
struct Mode {
    tier_strategy: TierStrategy,
    output_format: OutputFormat,
}

fn modes() -> Vec<Mode> {
    vec![
        Mode {
            tier_strategy: TierStrategy::Auto,
            output_format: OutputFormat::Markdown,
        },
        Mode {
            tier_strategy: TierStrategy::Tier2,
            output_format: OutputFormat::Markdown,
        },
        Mode {
            tier_strategy: TierStrategy::Auto,
            output_format: OutputFormat::Plain,
        },
        #[cfg(feature = "testkit")]
        Mode {
            tier_strategy: TierStrategy::Tier1,
            output_format: OutputFormat::Markdown,
        },
    ]
}

fn convert_in(html: &str, choice: HiddenContent, mode: Mode) -> String {
    let options = ConversionOptions {
        hidden_content: choice,
        tier_strategy: mode.tier_strategy,
        output_format: mode.output_format,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn convert_with(html: &str, choice: HiddenContent) -> String {
    convert_in(html, choice, modes()[0])
}

fn all_choices(html: &str) -> [String; 3] {
    CHOICES.map(|choice| convert_with(html, choice))
}

/// Attributes that make the converter treat an element as hidden.
const HIDERS: &[&str] = &[
    "hidden",
    r#"hidden="""#,
    r#"hidden="hidden""#,
    r#"hidden="until-found""#,
    "HIDDEN",
    r#"style="display:none""#,
    r#"style="display: none !important""#,
    r#"style="color:red; display:none""#,
    r#"STYLE="DISPLAY:NONE""#,
    r#"style="visibility:hidden""#,
    r#"style="/* note */ visibility: hidden""#,
    r#"style="font-size:0""#,
    r#"style="font-size: 0px""#,
];

/// Documents with one element that takes a hider at `{H}`. Each has the text `visible` outside
/// the element and the text `BODY` inside it.
const CONTEXTS: &[&str] = &[
    r#"<p>visible</p><div role="tabpanel" {H}>BODY</div>"#,
    "<p>visible</p><div {H}><p>BODY</p><p>second</p></div><p>after</p>",
    "<p>visible <span {H}>BODY</span> after</p>",
    "<p>visible</p><p {H}>BODY <strong>bold</strong></p>",
    "<ul><li>visible</li><li {H}>BODY</li><li>third</li></ul>",
    "<ul><li>visible</li></ul><ul {H}><li>BODY</li></ul>",
    "<table><tr><th>visible</th></tr><tr {H}><td>BODY</td></tr><tr><td>shown</td></tr></table>",
    "<table><tr><th>visible</th><th {H}>BODY</th></tr><tr><td>one</td><td>two</td></tr></table>",
    "<h1>visible</h1><section {H}><h2>BODY</h2><p>answer</p></section>",
    r#"<p>visible <a href="/x" {H}>BODY</a></p>"#,
    "<p>visible</p><div {H}>outer <div {H}>BODY</div> tail</div>",
    "<p>visible</p><div {H}>BODY<div>inner</div>tail</div><p>after</p>",
    "<p>visible</p><blockquote {H}><p>BODY</p></blockquote>",
    "<p>visible</p><pre {H}>BODY  code</pre>",
    "<my-panel><p>visible</p><div {H}>BODY</div></my-panel>",
    "<p>visible</p><span><div {H}>BODY</div></span>",
];

fn check(failures: &mut Vec<String>, holds: bool, message: impl FnOnce() -> String) {
    if !holds {
        failures.push(message());
    }
}

fn report(failures: &[String], total: usize) {
    assert!(
        failures.is_empty(),
        "{} of {total} inputs are wrong:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn drop_is_the_default() {
    assert_eq!(ConversionOptions::default().hidden_content, HiddenContent::Drop);
    assert_eq!(HiddenContent::default(), HiddenContent::Drop);
}

#[test]
fn should_keep_the_hidden_panel_of_the_issue_only_when_asked() {
    let html = r#"<p>visible</p><div role="tabpanel" hidden>tab two body</div>"#;
    let [drop, reachable, all] = all_choices(html);
    assert_eq!(drop, "visible\n");
    assert_eq!(reachable, "visible\n\ntab two body\n");
    assert_eq!(all, "visible\n\ntab two body\n");
}

#[test]
fn should_convert_every_input_of_the_issue_as_its_table_says() {
    // ~keep (input, Drop, Reachable, All)
    let cases = [
        (
            r#"<p>visible</p><div style="display:none">collapsed body</div>"#,
            "visible\n",
            "visible\n\ncollapsed body\n",
            "visible\n\ncollapsed body\n",
        ),
        (
            r#"<p>visible</p><div style="visibility:hidden">invisible body</div>"#,
            "visible\n",
            "visible\n\ninvisible body\n",
            "visible\n\ninvisible body\n",
        ),
        (
            r#"<p>visible</p><p style="font-size:0">zero size body</p>"#,
            "visible\n",
            "visible\n\nzero size body\n",
            "visible\n\nzero size body\n",
        ),
        (
            "<p>visible</p><template><p>template body</p></template>",
            "visible\n",
            "visible\n",
            "visible\n\ntemplate body\n",
        ),
        (
            r#"<div><template shadowrootmode="open"><p>shadow body</p><slot></slot></template><span>light body</span></div>"#,
            "light body\n",
            "shadow body\n\nlight body\n",
            "shadow body\n\nlight body\n",
        ),
        (
            r#"<p>visible</p><div aria-hidden="true">aria hidden body</div>"#,
            "visible\n\naria hidden body\n",
            "visible\n\naria hidden body\n",
            "visible\n\naria hidden body\n",
        ),
    ];
    let mut failures = Vec::new();
    for (html, drop, reachable, all) in cases {
        let actual = all_choices(html);
        check(&mut failures, actual == [drop, reachable, all], || {
            format!("{html}\n  expected {:?}\n  actual   {actual:?}", [drop, reachable, all])
        });
    }
    report(&failures, cases.len());
}

#[test]
fn should_convert_a_kept_element_like_the_same_element_that_is_not_hidden() {
    let mut failures = Vec::new();
    let mut total = 0;
    for context in CONTEXTS {
        let shown_html = context.replace(" {H}", "");
        for hider in HIDERS {
            let hidden_html = context.replace("{H}", hider);
            for mode in modes() {
                total += 1;
                let shown = convert_in(&shown_html, HiddenContent::Drop, mode);
                let dropped = convert_in(&hidden_html, HiddenContent::Drop, mode);
                let reachable = convert_in(&hidden_html, HiddenContent::Reachable, mode);
                let all = convert_in(&hidden_html, HiddenContent::All, mode);
                let holds = shown.contains("BODY")
                    && dropped.contains("visible")
                    && !dropped.contains("BODY")
                    && reachable == shown
                    && all == shown;
                check(&mut failures, holds, || {
                    format!(
                        "{hidden_html} {mode:?}\n  shown     {shown:?}\n  dropped   {dropped:?}\n  reachable {reachable:?}\n  all       {all:?}"
                    )
                });
            }
        }
    }
    report(&failures, total);
}

#[test]
fn should_keep_template_and_noscript_content_only_with_all() {
    // ~keep (input, output with All). Drop and Reachable write `visible` alone.
    let cases = [
        ("<p>visible</p><template><p>BODY</p></template>", "visible\n\nBODY\n"),
        ("<p>visible</p><TEMPLATE><p>BODY</p></TEMPLATE>", "visible\n\nBODY\n"),
        ("<p>visible</p><noscript><p>BODY</p></noscript>", "visible\n\nBODY\n"),
        (
            r#"<p>visible</p><noscript><img src="/p.gif" alt="BODY"></noscript>"#,
            "visible\n\n![BODY](/p.gif)\n",
        ),
        (
            r#"<p>visible</p><template shadowrootmode="sideways"><p>BODY</p></template>"#,
            "visible\n\nBODY\n",
        ),
        (
            "<p>visible</p><template shadowrootmode><p>BODY</p></template>",
            "visible\n\nBODY\n",
        ),
        (
            r#"<p>visible</p><template shadowroot="open"><p>BODY</p></template>"#,
            "visible\n\nBODY\n",
        ),
        (
            "<p>visible</p><template><template><p>BODY</p></template></template>",
            "visible\n\nBODY\n",
        ),
    ];
    let mut failures = Vec::new();
    let mut total = 0;
    for (html, kept) in cases {
        for mode in modes() {
            total += 1;
            let [drop, reachable, all] = CHOICES.map(|choice| convert_in(html, choice, mode));
            let kept = if matches!(mode.output_format, OutputFormat::Plain) {
                kept.replace("![BODY](/p.gif)", "BODY")
            } else {
                kept.to_owned()
            };
            check(
                &mut failures,
                drop == "visible\n" && reachable == "visible\n" && all == kept,
                || {
                    format!(
                        "{html} {mode:?}\n  drop {drop:?}\n  reachable {reachable:?}\n  all {all:?}\n  expected all {kept:?}"
                    )
                },
            );
        }
    }
    report(&failures, total);
}

#[test]
fn should_keep_a_declarative_shadow_root_with_reachable() {
    // ~keep (input, output with Drop, output with Reachable and All)
    let cases = [
        (
            r#"<div><template shadowrootmode="open"><p>BODY</p><slot></slot></template><span>light</span></div>"#,
            "light\n",
            "BODY\n\nlight\n",
        ),
        (
            r#"<div><template shadowrootmode="closed"><p>BODY</p></template><span>light</span></div>"#,
            "light\n",
            "BODY\n\nlight\n",
        ),
        (
            r#"<div><template SHADOWROOTMODE="Open"><p>BODY</p></template><span>light</span></div>"#,
            "light\n",
            "BODY\n\nlight\n",
        ),
        (
            r#"<p>visible</p><my-card><template shadowrootmode="open"><h2>BODY</h2></template></my-card>"#,
            "visible\n",
            "visible\n\n## BODY\n",
        ),
    ];
    let mut failures = Vec::new();
    let mut total = 0;
    for (html, dropped, kept) in cases {
        for mode in modes() {
            total += 1;
            let [drop, reachable, all] = CHOICES.map(|choice| convert_in(html, choice, mode));
            let kept = if matches!(mode.output_format, OutputFormat::Plain) {
                kept.replace("## ", "")
            } else {
                kept.to_owned()
            };
            check(
                &mut failures,
                drop == dropped && reachable == kept && all == kept,
                || {
                    format!(
                        "{html} {mode:?}\n  drop {drop:?}\n  reachable {reachable:?}\n  all {all:?}\n  expected kept {kept:?}"
                    )
                },
            );
        }
    }
    report(&failures, total);
}

/// Documents with one element whose start tag is at `{OPEN}` and whose end tag is at `{CLOSE}`.
/// Each has the text `visible` outside the element and the text `BODY` inside it.
const WRAPPED: &[&str] = &[
    "<p>visible</p>{OPEN}<p>BODY</p>{CLOSE}",
    "<table><tr><th>visible</th></tr>{OPEN}<tr><td>BODY</td></tr>{CLOSE}<tr><td>shown</td></tr></table>",
    "<table><tbody><tr><th>visible</th></tr>{OPEN}<tr><td>BODY</td></tr>{CLOSE}</tbody></table>",
    "<table><tr><th>visible</th>{OPEN}<th>BODY</th>{CLOSE}</tr><tr><td>one</td><td>two</td></tr></table>",
    "<ul><li>visible</li>{OPEN}<li>BODY</li>{CLOSE}<li>third</li></ul>",
    "<ol><li>visible</li>{OPEN}<li>BODY</li><li>more</li>{CLOSE}</ol>",
    "<p>visible {OPEN}BODY{CLOSE} after</p>",
    "<dl><dt>visible</dt>{OPEN}<dd>BODY</dd>{CLOSE}</dl>",
    "<h1>visible</h1>{OPEN}<h2>BODY</h2><p>answer</p>{CLOSE}<p>after</p>",
    "<p>visible</p>{OPEN}<!-- {CLOSE} --><p>BODY</p>{CLOSE}<p>after</p>",
    "<my-el><p>visible</p></my-el>{OPEN}<ul><li>BODY</li></ul>{CLOSE}",
    "<p>visible</p><span><div>block</div></span>{OPEN}<p>BODY</p>{CLOSE}",
];

/// Start and end tags of elements that only `All` keeps.
const INERT_TAGS: &[(&str, &str)] = &[
    ("<template>", "</template>"),
    ("<TEMPLATE>", "</TEMPLATE>"),
    (r#"<template id="row" data-note="a > b">"#, "</template >"),
    (r#"<template shadowrootmode="sideways">"#, "</template>"),
    ("<template shadowrootmode>", "</template>"),
    (r#"<template shadowroot="open">"#, "</template>"),
    ("<template><template>", "</template></template>"),
    ("<noscript>", "</noscript>"),
    ("<NoScript>", "</NoScript>"),
];

/// Start and end tags of a declarative shadow root, which `Reachable` keeps too.
const SHADOW_ROOT_TAGS: &[(&str, &str)] = &[
    (r#"<template shadowrootmode="open">"#, "</template>"),
    (r#"<template shadowrootmode="closed">"#, "</template>"),
    (
        "<template SHADOWROOTMODE='Open' shadowrootdelegatesfocus>",
        "</template>",
    ),
    ("<template shadowrootmode=open>", "</template>"),
];

fn wrap(context: &str, open: &str, close: &str) -> String {
    context.replace("{OPEN}", open).replace("{CLOSE}", close)
}

#[test]
fn should_convert_kept_template_and_noscript_content_where_it_is_written() {
    let mut failures = Vec::new();
    let mut total = 0;
    for context in WRAPPED {
        for (tags, reachable_keeps) in [(INERT_TAGS, false), (SHADOW_ROOT_TAGS, true)] {
            for (open, close) in tags {
                let wrapped_html = wrap(context, open, close);
                // ~keep The two oracles: the document with the tags deleted, and with the element deleted.
                let bare_html = wrap(context, "", "");
                for mode in modes() {
                    total += 1;
                    let shown = convert_in(&bare_html, HiddenContent::Drop, mode);
                    let dropped = convert_in(&wrapped_html, HiddenContent::Drop, mode);
                    // ~keep A choice that is not Drop always converts in tier 2. For a template in
                    // ~keep a table the two tiers differ at the base, so the oracle for content
                    // ~keep that Reachable drops is Drop in tier 2.
                    let dropped_in_tier2 = convert_in(
                        &wrapped_html,
                        HiddenContent::Drop,
                        Mode {
                            tier_strategy: TierStrategy::Tier2,
                            ..mode
                        },
                    );
                    let reachable = convert_in(&wrapped_html, HiddenContent::Reachable, mode);
                    let all = convert_in(&wrapped_html, HiddenContent::All, mode);
                    // ~keep The plain text output of a table holds the text of a template in it
                    // ~keep with every choice. The base does that too, and Drop is the base.
                    let plain_table =
                        matches!(mode.output_format, OutputFormat::Plain) && context.starts_with("<table");
                    let holds = shown.contains("BODY")
                        && dropped.contains("visible")
                        && (plain_table || !dropped.contains("BODY"))
                        && all == shown
                        && reachable == if reachable_keeps { &shown } else { &dropped_in_tier2 }.as_str();
                    check(&mut failures, holds, || {
                        format!(
                            "{wrapped_html} {mode:?}\n  shown     {shown:?}\n  dropped   {dropped:?}\n  reachable {reachable:?}\n  all       {all:?}"
                        )
                    });
                }
            }
        }
    }
    report(&failures, total);
}

#[test]
fn should_close_a_kept_template_with_its_own_end_tag() {
    // ~keep A plain template around a shadow root, and the reverse: with Reachable only the
    // ~keep shadow root's tags go, so the plain template still drops what it holds.
    let outer_plain = r#"<p>visible</p><template><div><template shadowrootmode="open"><p>BODY</p></template></div><p>inert</p></template><p>after</p>"#;
    let outer_shadow = r#"<div><template shadowrootmode="open"><p>BODY</p><template><p>inert</p></template><p>tail</p></template></div><p>visible</p>"#;
    assert_eq!(
        convert_with(outer_plain, HiddenContent::Reachable),
        "visible\n\nafter\n"
    );
    assert_eq!(
        convert_with(outer_plain, HiddenContent::All),
        "visible\n\nBODY\n\ninert\n\nafter\n"
    );
    assert_eq!(
        convert_with(outer_shadow, HiddenContent::Reachable),
        "BODY\n\ntail\n\nvisible\n"
    );
    assert_eq!(
        convert_with(outer_shadow, HiddenContent::All),
        "BODY\n\ninert\n\ntail\n\nvisible\n"
    );
    assert_eq!(convert_with(outer_shadow, HiddenContent::Drop), "visible\n");

    // ~keep A shadow root that is a child of a plain template: if its end tag stays, that end
    // ~keep tag closes the plain template, and the text after it is written.
    let child = r#"<p>visible</p><template><template shadowrootmode="open"><p>BODY</p></template><p>inert</p></template><p>after</p>"#;
    assert_eq!(convert_with(child, HiddenContent::Reachable), "visible\n\nafter\n");
    assert_eq!(
        convert_with(child, HiddenContent::All),
        "visible\n\nBODY\n\ninert\n\nafter\n"
    );
}

#[test]
fn should_keep_the_hidden_text_of_a_graphic_only_when_asked() {
    // ~keep A graphic hides an element with the same attribute and styles as the page, and with
    // ~keep its own `display` and `visibility` attributes. One choice governs all of them.
    let graphics = [
        "<p>a <svg><text hidden>kept text</text><text>shown text</text></svg> b</p>",
        r#"<p>a <svg><g><text>shown text</text><text style="visibility: hidden">kept text</text></g></svg> b</p>"#,
        r#"<p>a <svg><text display="none">kept text</text><text>shown text</text></svg> b</p>"#,
        r#"<p>a <svg><g visibility="hidden"><text>kept text</text></g><text>shown text</text></svg> b</p>"#,
        r#"<p>a <svg><text>shown text</text><foreignObject display="none"><div>kept text</div></foreignObject></svg> b</p>"#,
        r#"<p>a <svg display="none"><text>shown text</text><text>kept text</text></svg> b</p>"#,
        r#"<p>shown text <a href="/page"><svg><text display="none">kept text</text></svg></a></p>"#,
    ];
    let mut failures = Vec::new();
    let mut total = 0;
    for html in graphics {
        // ~keep The whole graphic of this input is hidden, so `Drop` writes none of its text.
        let whole_graphic_hidden = html.contains("<svg display");
        for mode in modes() {
            for choice in CHOICES {
                total += 1;
                let output = convert_in(html, choice, mode);
                let keeps = choice != HiddenContent::Drop;
                check(&mut failures, output.contains("kept text") == keeps, || {
                    format!("{html} {choice:?} {mode:?}: hidden text kept={} in {output:?}", !keeps)
                });
                let shows = keeps || !whole_graphic_hidden;
                check(&mut failures, output.contains("shown text") == shows, || {
                    format!("{html} {choice:?} {mode:?}: shown text is wrong in {output:?}")
                });
            }
        }
    }
    report(&failures, total);
}

#[test]
fn should_write_the_text_of_a_graphic_as_its_alt_text_for_each_choice() {
    let html = r#"<p>a <svg><text display="none">kept text</text><text>shown text</text></svg> b</p>"#;
    let plain = Mode {
        tier_strategy: TierStrategy::Auto,
        output_format: OutputFormat::Plain,
    };
    assert_eq!(convert_in(html, HiddenContent::Drop, plain), "a shown text b\n");
    assert_eq!(
        convert_in(html, HiddenContent::Reachable, plain),
        "a kept text shown text b\n"
    );
    assert_eq!(
        convert_in(html, HiddenContent::All, plain),
        "a kept text shown text b\n"
    );
}

#[test]
fn should_keep_aria_hidden_content_with_every_choice() {
    // ~keep `aria-hidden` takes an element away from assistive technology. A browser shows it.
    let cases = [
        (
            r#"<p>visible</p><div aria-hidden="true">BODY</div>"#,
            "visible\n\nBODY\n",
        ),
        (
            r#"<p>visible <span aria-hidden="true">BODY</span> after</p>"#,
            "visible BODY after\n",
        ),
        (
            r#"<h2>visible <a href="/x" aria-hidden="true">BODY</a></h2>"#,
            "## visible [BODY](/x)\n",
        ),
    ];
    for (html, expected) in cases {
        for choice in CHOICES {
            assert_eq!(convert_with(html, choice), expected, "{html} {choice:?}");
        }
    }
}

#[test]
fn should_leave_the_text_of_an_attribute_value_as_it_is() {
    // ~keep The alt text names both tags. It is text, and no choice takes it out.
    let html = r#"<p>visible <img src="/a.png" alt="a <template> b </template> c <noscript> d"></p>"#;
    let [drop, reachable, all] = all_choices(html);
    assert!(
        drop.contains("visible") && drop.contains("template") && drop.contains("noscript"),
        "{drop:?}"
    );
    assert_eq!(reachable, drop);
    assert_eq!(all, drop);
}

#[test]
fn should_not_read_a_tag_in_an_attribute_value_or_a_comment_as_a_template() {
    let html = r#"<p>visible</p><div title="<template>">shown</div><!-- <template> --><p>next</p><template><p>BODY</p></template>"#;
    assert_eq!(convert_with(html, HiddenContent::Drop), "visible\n\nshown\n\nnext\n");
    assert_eq!(
        convert_with(html, HiddenContent::Reachable),
        "visible\n\nshown\n\nnext\n"
    );
    assert_eq!(
        convert_with(html, HiddenContent::All),
        "visible\n\nshown\n\nnext\n\nBODY\n"
    );
}

#[test]
fn should_not_pair_a_template_tag_with_one_in_a_comment_or_a_script() {
    // ~keep A template start tag in a comment or in a script body opens nothing. If the scan
    // ~keep read it, the end tag of the real template would close it and stay removed, and the
    // ~keep real template would hold the rest of the page.
    for opaque in [
        r#"<!-- <template shadowrootmode="open"> -->"#,
        r#"<script>var a = '<template shadowrootmode="open">';</script>"#,
    ] {
        let html = format!("<div><p>visible</p><template>{opaque}<p>BODY</p></template><p>after</p></div>");
        let [drop, reachable, _] = all_choices(&html);
        assert!(drop.contains("visible") && drop.contains("after"), "{html}: {drop:?}");
        assert!(!drop.contains("BODY") && !drop.contains("var a"), "{html}: {drop:?}");
        assert_eq!(reachable, drop, "{html}");
    }
}

#[test]
fn should_read_the_kept_text_of_a_graphic_as_the_label_of_its_link() {
    // ~keep A link whose text is its address has a short form. The link reads its text before
    // ~keep it writes its label, so the kept text of a graphic must reach that reading too.
    let hidden = r#"<p>visible <a href="https://example.com/page"><svg><text display="none">https://example.com/page</text></svg></a></p>"#;
    let shown = hidden.replace(r#" display="none""#, "");
    assert_ne!(shown, hidden);
    let expected = convert_with(&shown, HiddenContent::Drop);
    assert!(expected.contains("https://example.com/page"), "{expected:?}");
    assert_ne!(convert_with(hidden, HiddenContent::Drop), expected);
    assert_eq!(convert_with(hidden, HiddenContent::Reachable), expected);
    assert_eq!(convert_with(hidden, HiddenContent::All), expected);
}

#[test]
fn should_keep_noscript_content_with_every_preprocessing_preset() {
    use html_to_markdown_rs::{PreprocessingOptions, PreprocessingPreset};
    let html = "<p>visible</p><noscript><p>BODY</p></noscript>";
    for preset in [
        PreprocessingPreset::Minimal,
        PreprocessingPreset::Standard,
        PreprocessingPreset::Aggressive,
    ] {
        let output = |choice| {
            let options = ConversionOptions {
                hidden_content: choice,
                preprocessing: PreprocessingOptions {
                    preset,
                    ..PreprocessingOptions::default()
                },
                ..ConversionOptions::default()
            };
            convert(html, Some(options))
                .expect("conversion should succeed")
                .content
                .unwrap_or_default()
        };
        assert_eq!(output(HiddenContent::Drop), "visible\n", "{preset:?}");
        assert_eq!(output(HiddenContent::Reachable), "visible\n", "{preset:?}");
        assert_eq!(output(HiddenContent::All), "visible\n\nBODY\n", "{preset:?}");
    }
}

#[test]
fn should_write_no_text_for_metadata_in_the_document_head() {
    // ~keep The head holds metadata. A `<noscript>` or `<template>` there holds `<link>`, `<style>`
    // ~keep and `<meta>` elements, which are not text for a reader, with or without scripting.
    // ~keep Each page has a `<template>` in its body too, which `All` must write: without it the
    // ~keep loop passes when the option is ignored.
    let inputs = [
        r#"<html><head><title>t</title><noscript><link rel="stylesheet" href="x.css"><style>.CODE{}</style><meta name="CODE" content="CODE"></noscript></head><body><p>visible</p><template><p>BODY</p></template></body></html>"#,
        r#"<html><head><title>t</title><template><link rel="stylesheet" href="CODE.css"><meta name="CODE" content="CODE"></template></head><body><p>visible</p><template><p>BODY</p></template></body></html>"#,
    ];
    for html in inputs {
        let [drop, reachable, all] = all_choices(html);
        assert!(drop.ends_with("visible\n") && !drop.contains("CODE"), "{drop:?}");
        assert_eq!(reachable, drop, "{html}");
        assert_eq!(all, format!("{drop}\nBODY\n"), "{html}");
    }
    // ~keep The body starts where the head ends, with or without a body tag.
    for html in [
        "<html><head><title>t</title></head><body><p>visible</p><noscript><p>BODY</p></noscript></body></html>",
        "<head><title>t</title></head><p>visible</p><template><p>BODY</p></template>",
        "<head><title>t</title><body><p>visible</p><template><p>BODY</p></template>",
    ] {
        let [drop, reachable, all] = all_choices(html);
        assert!(drop.ends_with("visible\n"), "{drop:?}");
        assert_eq!(reachable, drop, "{html}");
        assert_eq!(all, format!("{drop}\nBODY\n"), "{html}");
    }
}

#[test]
fn should_end_the_document_head_where_a_browser_ends_it() {
    // ~keep The end tag of the head and the body tag are optional: the first element that the
    // ~keep head cannot hold starts the body.
    for html in [
        "<!doctype html><head><title>t</title><p>visible</p><template><p>BODY</p></template>",
        "<head><title>t</title><p>visible</p><noscript><p>BODY</p></noscript>",
        r#"<head><meta charset="utf-8"><link rel="icon" href="i.png"><script>var a;</script><title>t</title><p>visible</p><template><p>BODY</p></template>"#,
    ] {
        let [drop, reachable, all] = all_choices(html);
        assert!(drop.contains("visible") && !drop.contains("BODY"), "{drop:?}");
        assert_eq!(reachable, drop, "{html}");
        assert!(all.contains("visible") && all.contains("BODY"), "{html}: {all:?}");
    }
    let shadow = r#"<head><title>t</title><div>visible<template shadowrootmode="open"><p>BODY</p></template></div>"#;
    let [drop, reachable, all] = all_choices(shadow);
    assert!(drop.contains("visible") && !drop.contains("BODY"), "{drop:?}");
    assert!(
        reachable.contains("BODY") && all.contains("BODY"),
        "{reachable:?} {all:?}"
    );
    // ~keep Metadata elements and a tag written in the title text do not end the head.
    for html in [
        r#"<html><head><meta charset="utf-8"><link rel="icon" href="i.png"><base href="/"><script>var a;</script><noscript><meta name="CODE" content="CODE"><link rel="stylesheet" href="CODE.css"></noscript></head><body><p>visible</p></body></html>"#,
        r#"<html><head><title>a <b> c</title><template><meta name="CODE" content="CODE"><link rel="stylesheet" href="CODE.css"></template></head><body><p>visible</p></body></html>"#,
    ] {
        let [drop, reachable, all] = all_choices(html);
        assert!(drop.ends_with("visible\n") && !drop.contains("CODE"), "{drop:?}");
        assert_eq!(reachable, drop, "{html}");
        assert_eq!(all, drop, "{html}");
    }
}

#[test]
fn should_convert_a_document_that_ends_in_an_unterminated_comment() {
    // ~keep The comment starts before the last `>` and runs to the end of the input.
    let html = "<p>visible</p><template><p>BODY</p></template><!-- <b>open</b> tail";
    assert_eq!(convert_with(html, HiddenContent::Drop), "visible\n");
    assert_eq!(convert_with(html, HiddenContent::Reachable), "visible\n");
    assert_eq!(convert_with(html, HiddenContent::All), "visible\n\nBODY\n");
}

#[test]
fn should_stay_linear_on_unterminated_tags() {
    // ~keep No tag of the run ends: no `>` is left, or the quote before the last `>` has no
    // ~keep partner. A scan that looks for the end of each `<a` reads the rest of the input
    // ~keep every time.
    let run = "<a ".repeat(200_000);
    for tail in ["", "\">", "'>"] {
        let html = format!("<p>visible</p>{run}{tail}");
        let timed = |choice| {
            let started = std::time::Instant::now();
            let output = convert_with(&html, choice);
            assert!(output.contains("visible"), "the text before the run is kept");
            started.elapsed()
        };
        let dropped = timed(HiddenContent::Drop);
        for choice in [HiddenContent::Reachable, HiddenContent::All] {
            let kept = timed(choice);
            // ~keep A quadratic scan reads 60 GB here. The bound is a multiple of the same input
            // ~keep under Drop, so a loaded host moves both sides.
            assert!(
                kept < dropped * 20 + std::time::Duration::from_secs(5),
                "tail {tail:?}: {choice:?} took {kept:?}, Drop took {dropped:?}"
            );
        }
    }
}

#[test]
fn should_never_write_code_or_attribute_values_as_text() {
    // ~keep Each input holds `CODE` where no reader gets to it, and `BODY` in hidden text.
    let inputs = [
        "<p>visible</p><div hidden>BODY<script>var CODE = 1;</script></div>",
        r#"<p>visible</p><div hidden>BODY<script type="application/json">{"CODE":1}</script></div>"#,
        r#"<p>visible</p><div style="display:none">BODY<style>.CODE { margin: 0 }</style></div>"#,
        "<p>visible</p><template><style>.CODE{}</style><script>CODE()</script><p>BODY</p></template>",
        r#"<div><template shadowrootmode="open"><style>:host{--CODE:1}</style><p>BODY</p></template></div><p>visible</p>"#,
        "<p>visible</p><noscript><style>.CODE{}</style><p>BODY</p></noscript>",
        "<p>visible</p><div hidden>BODY<!-- CODE --></div>",
        r#"<p>visible</p><div hidden>BODY<input type="hidden" name="token" value="CODE"></div>"#,
        r#"<p>visible</p><div hidden data-state="CODE">BODY</div>"#,
        r#"<p>visible</p><script>var CODE = "<div hidden>CODE</div>";</script><div hidden>BODY</div>"#,
    ];
    let mut failures = Vec::new();
    let mut total = 0;
    for html in inputs {
        for mode in modes() {
            for choice in CHOICES {
                total += 1;
                let output = convert_in(html, choice, mode);
                let keeps_body = output.contains("BODY");
                let holds = !output.contains("CODE")
                    && output.contains("visible")
                    && (keeps_body || choice != HiddenContent::All)
                    && (!keeps_body || choice != HiddenContent::Drop);
                check(&mut failures, holds, || {
                    format!("{html} {choice:?} {mode:?}\n  {output:?}")
                });
            }
        }
    }
    report(&failures, total);
}

#[test]
fn should_not_change_markup_that_was_never_dropped() {
    let inputs = [
        r#"<p>visible</p><div aria-hidden="true">BODY</div>"#,
        "<p>visible</p><div inert>BODY</div>",
        "<p>visible</p><details><summary>sum</summary><p>BODY</p></details>",
        "<p>visible</p><dialog><p>BODY</p></dialog>",
        r#"<p>visible</p><input list="l"><datalist id="l"><option value="v">BODY</option></datalist>"#,
        "<p>visible</p><select><option>BODY</option></select>",
        r#"<p>visible</p><div style="opacity:0">BODY</div>"#,
        r#"<p>visible</p><div style="content-visibility:hidden">BODY</div>"#,
        r#"<p>visible</p><div style="position:absolute;left:-9999px">BODY</div>"#,
        r#"<p>visible</p><div class="hidden d-none sr-only">BODY</div>"#,
        r#"<p>visible</p><div style="font-size:0"><span style="font-size:14px">BODY</span></div>"#,
        r#"<p>visible</p><div title="hidden from search">BODY</div>"#,
        r#"<p>visible</p><div data-hidden="true">BODY</div>"#,
    ];
    let mut failures = Vec::new();
    let mut total = 0;
    for html in inputs {
        for mode in modes() {
            total += 1;
            let [drop, reachable, all] = CHOICES.map(|choice| convert_in(html, choice, mode));
            check(
                &mut failures,
                drop.contains("visible") && drop.contains("BODY") && reachable == drop && all == drop,
                || format!("{html} {mode:?}\n  drop {drop:?}\n  reachable {reachable:?}\n  all {all:?}"),
            );
        }
    }
    report(&failures, total);
}

#[test]
fn should_keep_hidden_content_when_the_markup_goes_through_the_repair_parser() {
    // ~keep A custom element, or a block inside an inline element, sends the document through
    // ~keep the HTML5 tree builder first. (input, Drop, Reachable, All)
    let cases = [
        (
            "<my-el><p>visible</p></my-el><template><p>BODY</p></template>",
            false,
            false,
            true,
        ),
        (
            "<p>visible</p><span><div>block</div></span><template><p>BODY</p></template>",
            false,
            false,
            true,
        ),
        (
            "<my-el><p>visible</p></my-el><noscript><p>BODY</p></noscript>",
            false,
            false,
            true,
        ),
        (
            r#"<my-el><template shadowrootmode="open"><p>BODY</p></template><p>visible</p></my-el>"#,
            false,
            true,
            true,
        ),
        (
            "<my-el><p>visible</p><template><ul><li>BODY</li></ul></template></my-el>",
            false,
            false,
            true,
        ),
        ("<my-el><p>visible</p><div hidden>BODY</div></my-el>", false, true, true),
    ];
    let mut failures = Vec::new();
    let mut total = 0;
    for (html, drop, reachable, all) in cases {
        for mode in modes() {
            total += 1;
            let outputs = CHOICES.map(|choice| convert_in(html, choice, mode));
            let keeps = outputs.clone().map(|output| output.contains("BODY"));
            check(
                &mut failures,
                keeps == [drop, reachable, all] && outputs.iter().all(|output| output.contains("visible")),
                || format!("{html} {mode:?}\n  {outputs:?}"),
            );
        }
    }
    report(&failures, total);
}

#[cfg(feature = "metadata")]
#[test]
fn should_report_the_links_of_kept_content_in_the_metadata() {
    let html = r#"<p>visible <a href="/shown">shown</a></p><div hidden><a href="/panel">BODY</a></div>"#;
    let links = |choice| {
        let options = ConversionOptions {
            hidden_content: choice,
            ..ConversionOptions::default()
        };
        let result = convert(html, Some(options)).expect("conversion should succeed");
        result
            .metadata
            .links
            .iter()
            .map(|link| link.href.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(links(HiddenContent::Drop), ["/shown"]);
    assert_eq!(links(HiddenContent::Reachable), ["/shown", "/panel"]);
}

#[test]
fn should_record_kept_content_in_the_document_structure() {
    let html = "<h1>visible</h1><div hidden><h2>BODY</h2><p>answer</p></div>";
    let node_count = |choice| {
        let options = ConversionOptions {
            hidden_content: choice,
            include_document_structure: true,
            ..ConversionOptions::default()
        };
        let result = convert(html, Some(options)).expect("conversion should succeed");
        result.document.expect("the structure was asked for").nodes.len()
    };
    let dropped = node_count(HiddenContent::Drop);
    let kept = node_count(HiddenContent::Reachable);
    assert!(dropped > 0, "the visible heading has a node");
    assert_eq!(
        kept,
        dropped + 3,
        "a group, a heading and a paragraph for the kept section"
    );
}

#[test]
fn should_parse_each_choice_and_refuse_an_unknown_value() {
    for (text, expected) in [
        ("drop", HiddenContent::Drop),
        ("reachable", HiddenContent::Reachable),
        ("Reachable", HiddenContent::Reachable),
        ("all", HiddenContent::All),
        ("ALL", HiddenContent::All),
    ] {
        assert_eq!(HiddenContent::parse(text).ok(), Some(expected), "{text:?}");
    }
    // ~keep A value with a typing error is not the default: the caller asked to keep text.
    for text in ["", "keep", "reachble", "al", "drop all", "none", "true"] {
        let error = HiddenContent::parse(text).expect_err("an unknown value is refused");
        assert_eq!(
            error.to_string(),
            format!("Invalid configuration: hidden_content is \"{text}\". Use \"drop\", \"reachable\" or \"all\"."),
        );
    }
}

#[cfg(any(feature = "serde", feature = "metadata"))]
#[test]
fn should_read_and_write_each_choice_as_a_lower_case_word() {
    for (text, choice) in [
        ("drop", HiddenContent::Drop),
        ("reachable", HiddenContent::Reachable),
        ("all", HiddenContent::All),
    ] {
        let json = format!(r#"{{"hidden_content":"{text}"}}"#);
        let options: ConversionOptions = serde_json::from_str(&json).expect("the documented value is accepted");
        assert_eq!(options.hidden_content, choice, "{text}");
        let written = serde_json::to_value(&options).expect("options serialize");
        assert_eq!(written["hidden_content"], text);
    }
    let options: ConversionOptions = serde_json::from_str("{}").expect("empty options are accepted");
    assert_eq!(options.hidden_content, HiddenContent::Drop);
    let options: ConversionOptions =
        serde_json::from_str(r#"{"hidden_content":"Reachable"}"#).expect("the name of the choice is accepted");
    assert_eq!(options.hidden_content, HiddenContent::Reachable);
    for unknown in ["reachble", "", "keep"] {
        let json = format!(r#"{{"hidden_content":"{unknown}"}}"#);
        let error = serde_json::from_str::<ConversionOptions>(&json).expect_err("an unknown value is refused");
        let message = error.to_string();
        assert!(
            message.contains(&format!(
                "hidden_content is \"{unknown}\". Use \"drop\", \"reachable\" or \"all\"."
            )),
            "{message}"
        );
    }
}

#[test]
fn should_apply_the_choice_from_an_update_and_keep_it_when_the_update_names_another_field() {
    let mut options = ConversionOptions {
        hidden_content: HiddenContent::Reachable,
        skip_images: true,
        ..ConversionOptions::default()
    };
    options.apply_update(ConversionOptionsUpdate {
        skip_images: Some(false),
        ..ConversionOptionsUpdate::default()
    });
    assert_eq!(
        options.hidden_content,
        HiddenContent::Reachable,
        "an update without the field keeps it"
    );
    assert!(!options.skip_images);

    options.apply_update(ConversionOptionsUpdate {
        hidden_content: Some(HiddenContent::All),
        ..ConversionOptionsUpdate::default()
    });
    assert_eq!(options.hidden_content, HiddenContent::All);
    assert!(!options.skip_images, "an update of the choice keeps the other field");

    let built = ConversionOptions::builder().hidden_content(HiddenContent::All).build();
    assert_eq!(built.hidden_content, HiddenContent::All);
}

#[test]
fn should_write_the_kept_text_of_a_graphic_in_code_without_marks() {
    let html = "<pre>a <svg><text>shown</text><text display=\"none\">kept</text></svg> b</pre>";
    let [dropped, reachable, all] = all_choices(html);
    assert!(dropped.contains("shown") && !dropped.contains("kept"), "{dropped:?}");
    for output in [&reachable, &all] {
        assert!(output.contains("shown kept"), "{output:?}");
    }
    for output in [&dropped, &reachable, &all] {
        assert!(!output.contains("![") && !output.contains("data:"), "{output:?}");
    }
}

#[test]
fn should_step_over_a_structured_data_script_that_holds_template_tags() {
    // ~keep The JSON text names both tags. It is the text of the script and not markup: it must
    // ~keep not end the head, open a template or close one.
    let script = r#"<script type="application/ld+json">{"zq":"</head><p>zqtext</p><template></template>"}</script>"#;
    let in_head = format!(
        "<html><head>{script}<template><p>headtext</p></template></head>\
         <body><p>one</p><template><p>bodytext</p></template><p>two</p></body></html>"
    );
    let in_template = format!("<p>one</p><template>{script}<p>bodytext</p></template><p>two</p>");
    for html in [in_head, in_template] {
        let [dropped, reachable, all] = all_choices(&html);
        assert_eq!(dropped, reachable, "{html:?}");
        assert!(!dropped.contains("bodytext"), "{dropped:?}");
        assert!(all.contains("bodytext"), "{all:?}");
        for output in [&dropped, &all] {
            assert!(!output.contains("zq") && !output.contains("headtext"), "{output:?}");
            let (one, two) = (output.find("one"), output.find("two"));
            assert!(one.is_some() && one < two, "{output:?}");
        }
    }
}

include!("support/issue_753_word_order.rs");
