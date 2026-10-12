#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

fn check(html: &str, expected: &str, options: &ConversionOptions) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2] {
        let result = convert(
            html,
            Some(ConversionOptions {
                tier_strategy,
                ..options.clone()
            }),
        )
        .expect("conversion must succeed");
        assert_eq!(
            result.content.unwrap_or_default(),
            expected,
            "{tier_strategy:?}: {html}"
        );
    }
}

#[test]
fn should_keep_buttons_inline_with_surrounding_text() {
    check(
        "<p>Appearance <button>hide</button> now</p>",
        "Appearance hide now\n",
        &ConversionOptions::default(),
    );
    check(
        "<div><button>move to sidebar</button> <button>hide</button></div><p>After.</p>",
        "move to sidebar hide\n\nAfter.\n",
        &ConversionOptions::default(),
    );
}

#[test]
fn should_drop_comment_words_between_lists() {
    check(
        "<ul><li>a</li></ul>\n<!-- end of list -->\n<ul><li>b</li></ul>",
        "- a\n\n<!-- -->\n\n- b\n",
        &ConversionOptions::default(),
    );
}

#[test]
fn should_separate_definition_terms_and_descriptions_into_blocks() {
    for (html, expected) in [
        (
            "<dl><dt>Term</dt><dd>Description</dd></dl><p>After.</p>",
            "Term\n\nDescription\n\nAfter.\n",
        ),
        (
            "<dl><dt>Type:</dt><dd><code>bool</code></dd><dt>Default:</dt><dd><code>True</code></dd></dl><p>After.</p>",
            "Type:\n\n`bool`\n\nDefault:\n\n`True`\n\nAfter.\n",
        ),
        (
            "<dl><dt>Term</dt><dd><p>One.</p><p>Two.</p></dd></dl>",
            "Term\n\nOne.\n\nTwo.\n",
        ),
    ] {
        check(html, expected, &ConversionOptions::default());
    }
}

#[test]
fn should_label_media_links_with_titles_or_element_names() {
    for base_url in [None, Some("http://example.test/docs/page".to_owned())] {
        for (media, label, path, fallback) in [
            (
                "<audio controls src=\"/media/a.ogg\">fallback text</audio>",
                "audio",
                "/media/a.ogg",
                "fallback text\n\n",
            ),
            (
                "<audio controls><source src=\"/media/a.ogg\"></audio>",
                "audio",
                "/media/a.ogg",
                "",
            ),
            (
                "<video controls src=\"/media/v.mp4\"></video>",
                "video",
                "/media/v.mp4",
                "",
            ),
            (
                "<iframe src=\"/docs/embedded\" title=\"Embedded page\"></iframe>",
                "Embedded page",
                "/docs/embedded",
                "",
            ),
        ] {
            let prefix = if base_url.is_some() { "http://example.test" } else { "" };
            check(
                &format!("<p>before</p>{media}<p>after</p>"),
                &format!("before\n\n[{label}]({prefix}{path})\n\n{fallback}after\n"),
                &ConversionOptions {
                    base_url: base_url.clone(),
                    ..ConversionOptions::default()
                },
            );
        }
    }
}

#[test]
fn should_render_teletype_as_code_and_insertions_as_plain_text() {
    for (highlight_style, mark) in [(HighlightStyle::DoubleEqual, "==m=="), (HighlightStyle::None, "m")] {
        check(
            "<p><kbd>k</kbd> <samp>s</samp> <tt>t</tt> <code>c</code> <var>v</var> <mark>m</mark> <ins>i</ins> <u>u</u> <del>d</del></p>",
            &format!("`k` `s` `t` `c` *v* {mark} i u ~~d~~\n"),
            &ConversionOptions {
                highlight_style,
                ..ConversionOptions::default()
            },
        );
    }
}

#[test]
fn should_use_configured_emphasis_for_table_and_figure_captions() {
    check(
        "<table><caption>Table caption</caption><tr><th>h</th></tr><tr><td>v</td></tr></table><figure><img src=\"/a.png\" alt=\"a\"><figcaption>Figure caption</figcaption></figure>",
        "_Table caption_\n\n| h |\n| --- |\n| v |\n\n![a](/a.png)\n\n_Figure caption_\n",
        &ConversionOptions {
            strong_em_symbol: '_',
            ..ConversionOptions::default()
        },
    );
}

#[test]
fn should_drop_cdata_in_html_content() {
    check(
        "<p>before</p><![CDATA[cdata words]]><p>after</p>",
        "before\n\nafter\n",
        &ConversionOptions::default(),
    );
}

#[test]
fn should_emit_math_text_without_source_comments() {
    check(
        "<p>inline <math><mi>x</mi><mo>=</mo><mn>2</mn></math> end</p>",
        "inline x=2 end\n",
        &ConversionOptions::default(),
    );
    check(
        "<math display=\"block\"><mi>x</mi><mo>=</mo><mn>2</mn></math><p>After.</p>",
        "\n\nx=2\n\nAfter.\n",
        &ConversionOptions::default(),
    );
}

#[test]
fn should_keep_native_inline_button_and_definition_behavior_consistent() {
    for (html, expected) in [
        ("<p>A <button>B</button> C</p>", "A B C\n"),
        ("<dl><dt>Term</dt><dd>Definition</dd></dl>", "Term\n\nDefinition\n"),
        ("<p><tt>T</tt> <ins>I</ins></p>", "`T` I\n"),
    ] {
        let result = convert(
            html,
            Some(ConversionOptions {
                tier_strategy: TierStrategy::Tier1,
                extract_metadata: false,
                ..ConversionOptions::default()
            }),
        )
        .expect("native conversion succeeds");
        assert_eq!(result.content.unwrap_or_default(), expected, "{html}");
    }
}

#[test]
fn should_escape_markdown_block_openers_in_plain_inserted_text() {
    check("<p><ins>- item</ins></p>", "\\- item\n", &ConversionOptions::default());
}

#[test]
fn should_preserve_real_blocks_inside_plain_inserted_content() {
    check(
        "<ins><ul><li>x<blockquote>q</blockquote>t</li></ul></ins>",
        "- x\n  > q\n\n  t\n",
        &ConversionOptions::default(),
    );
}

#[test]
fn should_separate_a_stray_description_from_preceding_text() {
    let options = ConversionOptions {
        extract_metadata: false,
        ..Default::default()
    };
    for html in ["<div>a<dd>b</dd> c</div>", "<div>a <dd>b</dd> c</div>"] {
        let expected = "a\n\nb\n\nc\n";
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let result = convert(
                html,
                ConversionOptions {
                    tier_strategy,
                    ..options.clone()
                },
            )
            .expect("stray description converts");
            assert_eq!(result.content.as_deref(), Some(expected), "{tier_strategy:?}: {html}");
        }
        let (page, report) = html_to_markdown_rs::prescan::run(html);
        assert_eq!(
            html_to_markdown_rs::tier1::run(&page, &report, &options).expect("stray description scans"),
            expected
        );
    }
}

#[test]
fn should_keep_description_text_inline_in_a_heading() {
    for html in ["<h2>a<dd>b</dd> c</h2>", "<h2>a <dd>b</dd> c</h2>"] {
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let result = convert(
                html,
                ConversionOptions {
                    tier_strategy,
                    ..Default::default()
                },
            )
            .expect("heading description converts");
            assert_eq!(
                result.content.as_deref(),
                Some("## a b c\n"),
                "{tier_strategy:?}: {html}"
            );
        }
    }
}
