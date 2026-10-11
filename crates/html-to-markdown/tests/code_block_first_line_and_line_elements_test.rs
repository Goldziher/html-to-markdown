#![allow(missing_docs)]

//! A code block starts where a browser starts it, and each plain block container in it is one line.
//!
//! Each expected block was compared with the lines that Chrome renders for the same `pre`. An HTML
//! parser drops one line feed right after the `pre` tag; a second one is a blank first line of code.

use html_to_markdown_rs::options::WhitespaceMode;
use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, TierStrategy, convert};

fn options(tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        tier_strategy,
        ..ConversionOptions::default()
    }
}

fn converted(html: &str, options: Option<ConversionOptions>) -> String {
    convert(html, options)
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn assert_none(failures: &[String], checked: usize) {
    assert!(
        failures.is_empty(),
        "{} of {checked} checks differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The tiers a test can ask for by name: Tier-1 only where the test kit exposes it.
fn tiers() -> Vec<TierStrategy> {
    vec![
        TierStrategy::Tier2,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
    ]
}

/// Converts each input in `style` on every tier and reports the outputs that differ.
fn assert_style(style: CodeBlockStyle, cases: &[(&str, &str)]) {
    let mut failures = Vec::new();
    let mut checked = 0;
    for (html, expected) in cases {
        for tier_strategy in tiers() {
            checked += 1;
            let options = ConversionOptions {
                code_block_style: style,
                ..options(tier_strategy)
            };
            let actual = converted(html, Some(options));
            if actual != *expected {
                failures.push(format!(
                    "{html:?} ({style:?}, {tier_strategy:?}):\n  expected {expected:?}\n  actual   {actual:?}"
                ));
            }
        }
    }
    assert_eq!(checked, cases.len() * tiers().len());
    assert_none(&failures, checked);
}

/// Every name of the list of plain block containers that the two converters read.
const LINE_ELEMENTS: [&str; 13] = [
    "address", "article", "aside", "center", "dialog", "div", "footer", "header", "hgroup", "main", "nav", "search",
    "section",
];

#[test]
fn should_write_one_line_for_each_plain_block_container_in_a_code_block() {
    const EXPECTED: &str = "```\na\n  b\n```\n";
    let mut failures = Vec::new();
    let mut checked = 0;
    for name in LINE_ELEMENTS {
        // A browser shows only an open dialog.
        let open = if name == "dialog" { " open" } else { "" };
        let html = format!("<pre><{name}{open}>a</{name}><{name}{open}>  b</{name}></pre>");
        for tier_strategy in tiers() {
            checked += 1;
            // The default options remove a `nav` element with its content.
            let mut options = options(tier_strategy);
            options.preprocessing.remove_navigation = false;
            #[cfg(feature = "testkit")]
            if matches!(tier_strategy, TierStrategy::Tier1) {
                let direct = html_to_markdown_rs::tier1::run(
                    &html,
                    &html_to_markdown_rs::prescan::PrescanReport::default(),
                    &options,
                )
                .map_err(|reason| reason.to_string());
                if direct.as_deref() != Ok(EXPECTED) {
                    failures.push(format!("{name} (tier 1 with no fallback): {direct:?}"));
                }
            }
            let actual = converted(&html, Some(options));
            if actual != EXPECTED {
                failures.push(format!("{name} ({tier_strategy:?}): {actual:?}"));
            }
        }
    }
    assert_eq!(checked, LINE_ELEMENTS.len() * tiers().len());
    assert_none(&failures, checked);
}

/// Chrome shows two blank lines here: a parser drops a line feed only right after the `pre` tag.
/// Both tiers drop the first line feed of the content, as the fast one did before.
#[test]
fn should_drop_one_line_feed_at_the_start_of_the_code_element_of_a_block() {
    assert_style(
        CodeBlockStyle::Backticks,
        &[("<pre><code>\n\na\n</code></pre>", "```\n\na\n```\n")],
    );
}

#[test]
fn should_not_start_an_indented_code_block_with_a_blank_line() {
    assert_style(
        CodeBlockStyle::Indented,
        &[
            ("<p>x</p><pre>\n\na\nb\n</pre><p>y</p>", "x\n\n    a\n    b\n\ny\n"),
            ("<pre>\n\na\nb\n</pre>", "    a\n    b\n"),
            ("<ul><li>t<pre>\n\na\nb\n</pre></li></ul>", "- t\n\n      a\n      b\n"),
            ("<blockquote><pre>\n\na\nb\n</pre></blockquote>", ">     a\n>     b\n"),
        ],
    );
}

/// Strict white space mode writes the line feeds at the start of a block as it did before the rule
/// of the default mode: one blank first line for one or more of them. Chrome shows that blank line
/// when the line feed is in the `code` element, and when two line feeds follow the `pre` tag.
#[test]
fn should_keep_a_blank_first_line_of_a_block_in_strict_white_space_mode() {
    let fenced = [
        ("<pre><code>\na\n</code></pre>", "\na"),
        ("<pre><code>\n\na\n</code></pre>", "\na"),
        ("<pre>\n\na\nb\n</pre>", "\na\nb"),
    ];
    let mut failures = Vec::new();
    let mut checked = 0;
    for (html, code) in fenced {
        for (style, fence) in [(CodeBlockStyle::Backticks, "```"), (CodeBlockStyle::Tildes, "~~~")] {
            let strict = ConversionOptions {
                whitespace_mode: WhitespaceMode::Strict,
                code_block_style: style,
                ..ConversionOptions::default()
            };
            let mut settings = vec![("default options".to_owned(), strict.clone())];
            for tier_strategy in tiers() {
                // The fast converter leaves a tilde fence to the full one, and has its own start
                // of a backtick fence in this mode (the next test).
                if matches!(tier_strategy, TierStrategy::Tier2) || matches!(style, CodeBlockStyle::Tildes) {
                    let forced = ConversionOptions {
                        extract_metadata: false,
                        tier_strategy,
                        ..strict.clone()
                    };
                    settings.push((format!("{tier_strategy:?} forced"), forced));
                }
            }
            let expected = format!("{fence}\n{code}\n{fence}\n");
            for (name, options) in settings {
                checked += 1;
                let actual = converted(html, Some(options));
                if actual != expected {
                    failures.push(format!("{html:?} ({style:?}, {name}): {actual:?} is not {expected:?}"));
                }
            }
        }
    }
    assert_eq!(checked, if cfg!(feature = "testkit") { 15 } else { 12 });
    assert_none(&failures, checked);
}

/// The start of a block in strict white space mode, as each converter wrote it before: the full
/// one keeps the line feed after the `pre` tag (Chrome does not show that one), the fast one drops
/// it, and more than one blank first line is one.
#[test]
fn should_write_the_start_of_a_block_in_strict_white_space_mode_as_each_tier_did() {
    let cases = [
        ("<pre>\na\nb\n</pre>", "```\n\na\nb\n```\n", "```\na\nb\n```\n"),
        ("<pre>\n\n\na\n</pre>", "```\n\na\n```\n", "```\n\na\n```\n"),
        ("<pre>\n\n\n\na\n</pre>", "```\n\na\n```\n", "```\n\na\n```\n"),
        ("<pre><code>\na\n</code></pre>", "```\n\na\n```\n", "```\na\n```\n"),
        (
            "<ul><li>t<pre>\n\na\nb\n</pre></li></ul>",
            "- t\n\n  ```\n\n  a\n  b\n  ```\n",
            "- t\n\n  ```\n\n  a\n  b\n  ```\n",
        ),
        ("<pre>a\n\nb\n</pre>", "```\na\n\nb\n```\n", "```\na\n\nb\n```\n"),
    ];
    for (html, full, fast) in cases {
        for tier_strategy in tiers() {
            let expected = if matches!(tier_strategy, TierStrategy::Tier2) {
                full
            } else {
                fast
            };
            let options = ConversionOptions {
                whitespace_mode: WhitespaceMode::Strict,
                ..options(tier_strategy)
            };
            assert_eq!(converted(html, Some(options)), expected, "{html:?} ({tier_strategy:?})");
        }
    }
}

/// The default mode drops one line feed and keeps the others, whatever their count.
#[test]
fn should_drop_only_the_first_line_feed_of_a_block_in_the_default_mode() {
    assert_style(
        CodeBlockStyle::Backticks,
        &[
            ("<pre>\na\nb\n</pre>", "```\na\nb\n```\n"),
            ("<pre>\n\n\na\n</pre>", "```\n\n\na\n```\n"),
            ("<pre><code>\na\n</code></pre>", "```\na\n```\n"),
        ],
    );
}

#[test]
fn should_drop_one_line_feed_before_content_of_white_space() {
    assert_style(
        CodeBlockStyle::Backticks,
        &[("<p>x</p><pre>\n  \n</pre><p>y</p>", "x\n\n```\n  \n```\n\ny\n")],
    );
}
