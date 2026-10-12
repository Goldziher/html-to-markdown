// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]

//! Regression tests for where a list item starts: text inside a list before an item (issue #625),
//! an item after a real item or a quote between inline markers (issue #633), the first block of a
//! task item (issue #634) and an underlined heading in a list item (issue #635).

use html_to_markdown_rs::options::{CodeBlockStyle, HeadingStyle, ListIndentType, NewlineStyle};
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn tier2_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    }
}

fn convert_with(html: &str, options: &ConversionOptions) -> String {
    convert(html, Some(options.clone()))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn render(markdown: &str) -> String {
    comrak::markdown_to_html(markdown, &comrak::Options::default())
}

/// Render `markdown` with GFM task list items, which shows whether a checkbox survives.
fn render_tasklist(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.tasklist = true;
    comrak::markdown_to_html(markdown, &options)
}

/// Convert `html` with `options`, assert the exact markdown, and assert that its rendering holds
/// every one of `rendered`.
fn assert_converts(html: &str, options: &ConversionOptions, expected: &str, rendered: &[&str]) {
    let markdown = convert_with(html, options);
    assert_eq!(markdown, expected, "{html}");
    let html_out = render(&markdown);
    for fragment in rendered {
        assert!(html_out.contains(fragment), "{html}: {fragment:?} not in {html_out:?}");
    }
}

#[test]
fn should_start_a_list_item_on_its_own_line_after_text_in_the_list() {
    let options = tier2_options();
    let html5lib = include_str!("../../../test_documents/html/html5lib/unclosed-li-siblings-nested-ul.html");
    for strategy in [TierStrategy::Tier2, TierStrategy::Auto] {
        let options = ConversionOptions {
            tier_strategy: strategy,
            ..tier2_options()
        };
        assert_converts(
            html5lib,
            &options,
            "- hello\n- world\n\nhow\n  - do\n\n  you\n",
            &["<p>how</p>", "<li>do</li>"],
        );
    }
    let wrapped = ConversionOptions {
        wrap: true,
        ..tier2_options()
    };
    let markdown = convert_with(html5lib, &wrapped);
    assert!(render(&markdown).contains("<li>do</li>"), "{markdown:?}");
    assert_converts(
        "<ul>how<li>do</li></ul>",
        &options,
        "how\n- do\n",
        &["<p>how</p>", "<li>do</li>"],
    );
    assert_converts(
        "<ul><li>a<ul>how<li>do</li></ul></li></ul>",
        &options,
        "- a\n  how\n  * do\n",
        &["<li>do</li>"],
    );
    assert_converts(
        "<ul><li>a</li>x<li>b</li></ul>",
        &options,
        "- a\nx\n- b\n",
        &["<li>b</li>"],
    );
    assert_eq!(
        convert_with("<table><tr><td><ul>how<li>do</li></ul></td></tr></table>", &options),
        "| how do |\n| ------ |\n"
    );
    assert_converts(
        "<ul><li>a<ol>how<li>do</li></ol></li></ul>",
        &options,
        "- a\n  how\n  1. do\n",
        &["<ol>\n<li>do</li>"],
    );
}

#[cfg(feature = "testkit")]
#[test]
fn should_start_a_list_item_after_text_in_the_list_the_same_way_in_tier_1() {
    let tier1 = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..tier2_options()
    };
    for (html, expected) in [
        ("<ul><li>a</li>x<li>b</li></ul>", "- a\nx\n- b\n"),
        (r#"<ol start="3"><li>a</li>x<li>b</li></ol>"#, "3. a\nx\n\n4. b\n"),
    ] {
        assert_eq!(convert_with(html, &tier1), expected, "{html}");
        assert_eq!(convert_with(html, &tier2_options()), expected, "{html}");
    }
}

#[test]
fn should_leave_a_blank_line_before_an_item_that_cannot_interrupt_the_text() {
    let options = tier2_options();
    assert_converts(
        r#"<ol start="3">how<li>do</li></ol>"#,
        &options,
        "how\n\n3. do\n",
        &["<p>how</p>", "<ol start=\"3\">\n<li>do</li>"],
    );
    assert_converts(
        r#"<ul><li>a<ol start="3">how<li>do</li></ol></li></ul>"#,
        &options,
        "- a\n  how\n\n  3. do\n",
        &["<ol start=\"3\">\n<li>do</li>"],
    );
}

#[cfg(feature = "testkit")]
#[test]
fn should_keep_text_after_a_quote_in_a_sibling_item_between_markers() {
    let options = tier2_options();
    for (html, expected) in [
        (
            "<b><ul><li>a<ol><li>x</li><li>y<blockquote>q</blockquote>t</li></ol></li></ul></b>",
            "**- a\n  1. x\n  2. y\n     > q\n\n     t**\n",
        ),
        (
            "<em><ul><li>a<ol><li>x</li><li>y<blockquote>q</blockquote>t</li></ol></li></ul></em>",
            "*- a\n  1. x\n  2. y\n     > q\n\n     t*\n",
        ),
        (
            "<b><ul><li>a<ol><li>x</li><li>y</li><li>z<blockquote>q</blockquote>t</li></ol></li></ul></b>",
            "**- a\n  1. x\n  2. y\n  3. z\n     > q\n\n     t**\n",
        ),
        (
            "<b><ul><li>a<ol><li>x<ol><li>m<ul><li>n</li></ul></li></ol></li><li>y<blockquote>q</blockquote>t</li></ol></li></ul></b>",
            "**- a\n  1. x\n     1. m\n        * n\n  2. y\n     > q\n\n     t**\n",
        ),
    ] {
        assert_converts(html, &options, expected, &["<blockquote>\n<p>q</p>\n</blockquote>"]);
    }
    let tier1 = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..tier2_options()
    };
    assert_converts(
        "<b><ul><li>a<ol><li>x</li><li>y<blockquote>q</blockquote>t</li></ol></li></ul></b>",
        &tier1,
        "**- a\n  1. x\n  2. y\n     > q\n\n     t**\n",
        &["<blockquote>\n<p>q</p>\n</blockquote>"],
    );
}

#[test]
fn should_start_an_item_after_a_quote_between_markers() {
    assert_converts(
        r#"<b><ul><li>a<blockquote>p</blockquote><ol start="2"><li>x<blockquote>q</blockquote>t</li></ol></li></ul></b>"#,
        &tier2_options(),
        "**- a\n  > p\n  2. x\n     > q\n\n     t**\n",
        &[
            "<ol start=\"2\">\n<li>\n<p>x</p>",
            "<blockquote>\n<p>q</p>\n</blockquote>",
        ],
    );
}

#[test]
fn should_keep_items_that_cannot_interrupt_the_text_between_markers_as_text() {
    let options = tier2_options();
    for (html, expected, text) in [
        (
            r#"<b><ul><li>a<ol start="3"><li>x</li><li>y<blockquote>q</blockquote>t</li></ol></li></ul></b>"#,
            "**- a\n  3. x\n  4. y\n     > q\n     t**\n",
            "3. x\n4. y",
        ),
        (
            r#"<b><ul><li>a<ol start="2"><li>x<blockquote>q</blockquote>t</li></ol></li></ul></b>"#,
            "**- a\n  2. x\n     > q\n     t**\n",
            "2. x",
        ),
        (
            "<b><ol><li>x</li><li>y<blockquote>q</blockquote>t</li></ol></b>",
            "**1. x\n2. y\n   > q\n\nt**\n",
            "1. x\n2. y",
        ),
        (
            r#"<b>x<ul><li>a<ol start="3"><li>y<blockquote>q</blockquote>t</li></ol></li></ul></b>"#,
            "**x\n\n- a\n  3. y\n     > q\n\n  t**\n",
            "a\n3. y",
        ),
        (
            r#"<b><ul><li>a<blockquote>p</blockquote>u<ol start="2"><li>x<blockquote>q</blockquote>t</li></ol></li></ul></b>"#,
            "**- a\n  > p\n\nu\n  2. x\n     > q\n     t**\n",
            "u\n2. x",
        ),
        (
            r#"<b><ul><li>a<br>b<ol start="2"><li>x<blockquote>q</blockquote>t</li></ol></li></ul></b>"#,
            "**- a  \n  b\n  2. x\n     > q\n     t**\n",
            "b\n2. x",
        ),
    ] {
        let markdown = convert_with(html, &options);
        assert_eq!(markdown, expected, "{html}");
        assert!(render(&markdown).contains(text), "{markdown:?}");
    }
}

#[test]
fn should_start_the_first_block_of_a_task_item_that_cannot_interrupt_the_checkbox_line_after_a_blank_line() {
    let options = tier2_options();
    for html in [
        r#"<ul><li><input type="checkbox"><div><ol start="3"><li>x</li></ol></div></li></ul>"#,
        r#"<ul><li><input type="checkbox"><ol start="3"><li>x</li></ol></li></ul>"#,
    ] {
        assert_converts(
            html,
            &options,
            "- [ ] &#32;\n\n  3. x\n",
            &["<ol start=\"3\">\n<li>x</li>"],
        );
    }
    let indented = ConversionOptions {
        code_block_style: CodeBlockStyle::Indented,
        ..tier2_options()
    };
    for html in [
        r#"<ul><li><input type="checkbox"><div><pre>c</pre></div></li></ul>"#,
        r#"<ul><li><input type="checkbox"><pre>c</pre></li></ul>"#,
    ] {
        assert_converts(
            html,
            &indented,
            "- [ ] &#32;\n\n      c\n",
            &["<pre><code>c\n</code></pre>"],
        );
    }
    assert_converts(
        r#"<ul><li><input type="checkbox"><pre>c&#10;d</pre>t</li></ul>"#,
        &indented,
        "- [ ] &#32;\n\n      c\n      d\n\n  t\n",
        &["<pre><code>c\nd\n</code></pre>", "<p>t</p>"],
    );
    for (html, expected) in [
        (
            r#"<ul><li><input type="checkbox"><pre>c</pre></li></ul>"#,
            "- [ ] &#32;\n  ```\n  c\n  ```\n",
        ),
        (
            r#"<ul><li><input type="checkbox"><ol><li>x</li></ol></li></ul>"#,
            "- [ ] &#32;\n  1. x\n",
        ),
        (
            r#"<ul><li><input type="checkbox"><hr></li></ul>"#,
            "- [ ] &#32;\n\n  ---\n",
        ),
        (
            r#"<ul><li><input type="checkbox"><blockquote></blockquote></li></ul>"#,
            "- [ ] &#32;\n",
        ),
        (r#"<ul><li><input type="checkbox"><ul></ul>t</li></ul>"#, "- [ ] t\n"),
        (r#"<ul><li><input type="checkbox"><h2></h2>t</li></ul>"#, "- [ ] t\n"),
        (r#"<ul><li><input type="checkbox"><pre></pre>t</li></ul>"#, "- [ ] t\n"),
        (
            r#"<ul><li><input type="checkbox"><h2><br></h2>t</li></ul>"#,
            "- [ ] t\n",
        ),
    ] {
        assert_eq!(convert_with(html, &options), expected, "{html}");
    }
}

#[test]
fn should_keep_task_text_after_a_code_block_of_only_whitespace_out_of_the_code_block() {
    let options = tier2_options();
    for html in [
        r#"<ul><li><input type="checkbox"><pre> </pre>t</li></ul>"#,
        "<ul><li><input type=\"checkbox\"><pre>\n\n</pre>t</li></ul>",
        r#"<ul><li><input type="checkbox"><div><pre> </pre>t</div></li></ul>"#,
    ] {
        let markdown = convert_with(html, &options);
        assert!(markdown.starts_with("- [ ] &#32;\n  ```\n"), "{html}: {markdown:?}");
        let rendered = render(&markdown);
        assert!(rendered.contains("<p>t</p>"), "{html}: {rendered:?}");
        assert!(!rendered.contains("t\n</code>"), "{html}: {rendered:?}");
    }
}

#[test]
fn should_keep_an_empty_preserved_element_on_the_checkbox_line() {
    let options = ConversionOptions {
        preserve_tags: ["span", "b", "a", "q", "abbr"].map(String::from).to_vec(),
        ..tier2_options()
    };
    for element in [
        "<span></span>",
        r#"<span title="x"></span>"#,
        r#"<a name="x"></a>"#,
        r#"<abbr title="x"></abbr>"#,
        "<q></q>",
        "<b> </b>",
    ] {
        for html in [
            format!(r#"<ul><li><input type="checkbox">{element}<blockquote>q</blockquote></li></ul>"#),
            format!(r#"<ul><li><input type="checkbox"><div>{element}<blockquote>q</blockquote></div></li></ul>"#),
        ] {
            assert_converts(
                &html,
                &options,
                &format!("- [ ] {element}\n  > q\n"),
                &["<blockquote>\n<p>q</p>\n</blockquote>"],
            );
        }
    }
}

#[test]
fn should_start_a_task_item_quote_after_an_element_that_writes_nothing_on_the_next_line() {
    let options = tier2_options();
    for element in [
        "<br>",
        "<span>&nbsp;</span>",
        "&nbsp;",
        "<template>x</template>",
        "<noscript>x</noscript>",
        "<h2><br></h2>",
    ] {
        for html in [
            format!(r#"<ul><li><input type="checkbox">{element}<blockquote>q</blockquote></li></ul>"#),
            format!(r#"<ul><li><input type="checkbox"><div>{element}<blockquote>q</blockquote></div></li></ul>"#),
        ] {
            assert_converts(
                &html,
                &options,
                "- [ ] &#32;\n  > q\n",
                &["<blockquote>\n<p>q</p>\n</blockquote>"],
            );
        }
    }
    let backslash = ConversionOptions {
        newline_style: NewlineStyle::Backslash,
        ..tier2_options()
    };
    assert_converts(
        r#"<ul><li><input type="checkbox"><br><blockquote>q</blockquote></li></ul>"#,
        &backslash,
        "- [ ] &#32;\n  > q\n",
        &["<blockquote>\n<p>q</p>\n</blockquote>"],
    );
    assert_converts(
        r#"<ul><li><input type="checkbox"><br>t<blockquote>q</blockquote></li></ul>"#,
        &options,
        "- [ ] t\n  > q\n",
        &["[ ] t\n<blockquote>"],
    );
}

#[test]
fn should_start_a_task_item_quote_after_an_element_that_writes_nothing_in_a_container_on_the_next_line() {
    let options = tier2_options();
    let skip_images = ConversionOptions {
        skip_images: true,
        ..tier2_options()
    };
    let elements = [
        (r#"<input type="text">"#, &options),
        ("<meta>", &options),
        (r#"<link rel="x">"#, &options),
        ("<source>", &options),
        ("<track>", &options),
        ("<embed>", &options),
        ("<area>", &options),
        ("<picture></picture>", &options),
        ("<math></math>", &options),
        (r#"<img src="i.png" alt="i">"#, &skip_images),
    ];
    for (element, options) in elements {
        for html in [
            format!(r#"<ul><li><input type="checkbox"><div>{element}<blockquote>q</blockquote></div></li></ul>"#),
            format!(
                r#"<ul><li><input type="checkbox"><section>{element}<blockquote>q</blockquote></section></li></ul>"#
            ),
            format!(
                r#"<ul><li><input type="checkbox"><section><div>{element}</div><blockquote>q</blockquote></section></li></ul>"#
            ),
        ] {
            assert_converts(
                &html,
                options,
                "- [ ] &#32;\n  > q\n",
                &["<blockquote>\n<p>q</p>\n</blockquote>"],
            );
        }
    }
    assert_converts(
        r#"<ul><li><input type="checkbox"><div><img src="i.png" alt="i"><blockquote>q</blockquote></div></li></ul>"#,
        &options,
        "- [ ] ![i](i.png)\n  > q\n",
        &["<img src=\"i.png\" alt=\"i\" />"],
    );
}

#[test]
fn should_start_a_preserved_block_element_of_a_task_item_on_the_next_line() {
    let options = ConversionOptions {
        preserve_tags: vec!["div".to_string()],
        ..tier2_options()
    };
    for html in [
        r#"<ul><li><input type="checkbox"><div><blockquote>q</blockquote></div></li></ul>"#,
        r#"<ul><li><input type="checkbox"><section><div><blockquote>q</blockquote></div></section></li></ul>"#,
    ] {
        let markdown = convert_with(html, &options);
        assert_eq!(
            markdown, "- [ ] &#32;\n  <div><blockquote>q</blockquote></div>\n",
            "{html}"
        );
        let rendered = render_tasklist(&markdown);
        assert!(
            rendered.contains("<li><input type=\"checkbox\" disabled=\"\" />  \n<!-- raw HTML omitted -->\n</li>"),
            "{html}: {rendered:?}"
        );
    }
}

#[test]
fn should_start_an_underlined_heading_that_starts_a_task_item_after_a_blank_line() {
    let options = ConversionOptions {
        heading_style: HeadingStyle::Underlined,
        ..tier2_options()
    };
    for html in [
        r#"<ul><li><input type="checkbox"><h2>q</h2></li></ul>"#,
        r#"<ul><li><input type="checkbox"><div><h2>q</h2></div></li></ul>"#,
    ] {
        let markdown = convert_with(html, &options);
        assert_eq!(markdown, "- [ ] &#32;\n\n  q\n  --\n", "{html}");
        let rendered = render_tasklist(&markdown);
        assert!(
            rendered.contains("<input type=\"checkbox\" disabled=\"\" /> \n<p> </p>\n<h2>q</h2>"),
            "{html}: {rendered:?}"
        );
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_start_a_task_item_quote_after_an_element_a_visitor_skips_on_the_next_line() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct SkipEmphasis;

    impl HtmlVisitor for SkipEmphasis {
        fn visit_element_end(&mut self, ctx: &NodeContext, _output: &str) -> VisitResult {
            if ctx.tag_name == "em" {
                return VisitResult::Skip;
            }
            VisitResult::Continue
        }
    }

    let options = ConversionOptions {
        visitor: Some(Arc::new(Mutex::new(SkipEmphasis))),
        ..tier2_options()
    };
    for html in [
        r#"<ul><li><input type="checkbox"><em>x</em><blockquote>q</blockquote></li></ul>"#,
        r#"<ul><li><input type="checkbox"><div><em>x</em><blockquote>q</blockquote></div></li></ul>"#,
    ] {
        assert_converts(
            html,
            &options,
            "- [ ] &#32;\n  > q\n",
            &["<blockquote>\n<p>q</p>\n</blockquote>"],
        );
    }
}

#[test]
fn should_start_a_table_whose_cell_holds_a_quote_after_a_blank_line() {
    let html =
        r#"<ul><li><input type="checkbox"><table><tr><td><blockquote>q</blockquote></td></tr></table></li></ul>"#;
    let markdown = convert_with(html, &tier2_options());
    assert!(markdown.starts_with("- [ ] &#32;\n\n  | q |\n"), "{html}: {markdown:?}");
}

#[test]
fn should_start_a_task_item_quote_after_an_empty_inline_element_on_the_next_line() {
    let options = tier2_options();
    for wrapper in [
        "<span></span>",
        "<b></b>",
        "<center></center>",
        "<span> </span>",
        "<span><b></b></span>",
        r#"<a id="x"></a>"#,
        r#"<i class="icon"></i>"#,
    ] {
        assert_converts(
            &format!(r#"<ul><li><input type="checkbox">{wrapper}<blockquote>q</blockquote></li></ul>"#),
            &options,
            "- [ ] &#32;\n  > q\n",
            &["<blockquote>\n<p>q</p>\n</blockquote>"],
        );
    }
    for (wrapper, expected) in [
        (r#"<img src="i.png" alt="i">"#, "- [ ] ![i](i.png)\n  > q\n"),
        ("<span>s</span>", "- [ ] s\n  > q\n"),
        ("<b>", "- [ ] &#32;\n  > **q**\n"),
    ] {
        let html = format!(r#"<ul><li><input type="checkbox">{wrapper}<blockquote>q</blockquote></li></ul>"#);
        assert_eq!(convert_with(&html, &options), expected, "{html}");
    }
    for (wrapper, first_line) in [
        (r#"<a href="u"></a>"#, "- [ ] [](u)\n"),
        ("<svg></svg>", "- [ ] ![](data:image/svg+xml;base64,"),
    ] {
        let html = format!(r#"<ul><li><input type="checkbox">{wrapper}<blockquote>q</blockquote></li></ul>"#);
        let markdown = convert_with(&html, &options);
        assert!(markdown.starts_with(first_line), "{html}: {markdown:?}");
    }
}

#[test]
fn should_write_an_underlined_heading_in_a_list_item_at_the_content_column() {
    let options = ConversionOptions {
        heading_style: HeadingStyle::Underlined,
        ..tier2_options()
    };
    for (html, expected, heading) in [
        (
            r#"<ul><li><input type="checkbox"><h2>q</h2></li></ul>"#,
            "- [ ] &#32;\n\n  q\n  --\n",
            "<h2>q</h2>",
        ),
        (
            r#"<ul><li><input type="checkbox"><h1>q</h1></li></ul>"#,
            "- [ ] &#32;\n\n  q\n  =\n",
            "<h1>q</h1>",
        ),
        ("<ul><li><h2>q</h2></li></ul>", "- q\n  --\n", "<h2>q</h2>"),
        (
            "<ul><li>a<ul><li><h2>q</h2></li></ul></li></ul>",
            "- a\n  * q\n    --\n",
            "<h2>q</h2>",
        ),
        (
            "<ul><li><p>a</p><h2>q</h2></li></ul>",
            "- a\n\n  q\n  --\n",
            "<h2>q</h2>",
        ),
        (
            "<ul><li><blockquote><h2>q</h2></blockquote></li></ul>",
            "- > q\n  > -\n",
            "<h2>q</h2>",
        ),
        (
            "<ul><li><blockquote><p>a</p><h2>q</h2></blockquote></li></ul>",
            "- > a\n  >\n  > q\n  > -\n",
            "<p>a</p>\n<h2>q</h2>",
        ),
    ] {
        assert_converts(html, &options, expected, &[heading]);
        assert!(!render(expected).contains("<li></li>"), "{html}");
    }
}

#[test]
fn should_start_an_underlined_heading_after_a_line_of_the_item_after_a_blank_line() {
    let options = ConversionOptions {
        heading_style: HeadingStyle::Underlined,
        ..tier2_options()
    };
    for (html, expected, heading) in [
        (
            "<ul><li>a<h2>q</h2></li></ul>",
            "- a\n\n  q\n  --\n",
            "<p>a</p>\n<h2>q</h2>",
        ),
        (
            "<ul><li>a<h1>q</h1></li></ul>",
            "- a\n\n  q\n  =\n",
            "<p>a</p>\n<h1>q</h1>",
        ),
        (
            "<ul><li>a<blockquote>p</blockquote><h2>q</h2></li></ul>",
            "- a\n  > p\n\n  q\n  --\n",
            "</blockquote>\n<h2>q</h2>",
        ),
        (
            "<ol><li>a<h2>q</h2>b</li><li>c</li></ol>",
            "1. a\n\n   q\n   --\n   b\n2. c\n",
            "<h2>q</h2>\n<p>b</p>",
        ),
    ] {
        assert_converts(html, &options, expected, &[heading]);
    }
    assert_eq!(
        convert_with("<ul><li>a<h3>q</h3></li></ul>", &options),
        "- a\n  ### q\n"
    );
    assert_eq!(
        convert_with("<ul><li>a<h2>q</h2></li></ul>", &tier2_options()),
        "- a\n  ## q\n"
    );
}

#[test]
fn should_write_an_underlined_heading_in_a_list_in_a_quote_at_the_content_column() {
    let options = ConversionOptions {
        heading_style: HeadingStyle::Underlined,
        ..tier2_options()
    };
    for (html, expected) in [
        (
            "<blockquote><ul><li><h2>q</h2></li></ul></blockquote>",
            "> - q\n>   --\n",
        ),
        (
            r#"<blockquote><ul><li><input type="checkbox"><h2>q</h2></li></ul></blockquote>"#,
            "> - [ ] &#32;\n>\n>   q\n>   --\n",
        ),
        (
            "<ul><li><blockquote><ul><li><h2>q</h2></li></ul></blockquote></li></ul>",
            "- > * q\n  >   --\n",
        ),
        (
            "<ul><li><blockquote><h2>q</h2></blockquote></li></ul>",
            "- > q\n  > -\n",
        ),
    ] {
        assert_converts(html, &options, expected, &["<h2>q</h2>"]);
        assert!(!render(expected).contains("<li></li>"), "{html}");
    }
    let tabs = ConversionOptions {
        list_indent_type: ListIndentType::Tabs,
        ..options
    };
    assert_converts(
        "<blockquote><ul><li><h2>q</h2></li></ul></blockquote>",
        &tabs,
        "> - q\n> \t--\n",
        &["<h2>q</h2>"],
    );
}

#[test]
fn should_start_a_block_after_an_underlined_heading_of_one_letter_in_its_own_paragraph() {
    let options = ConversionOptions {
        heading_style: HeadingStyle::Underlined,
        ..tier2_options()
    };
    for (html, expected) in [
        ("<ul><li><h2>q</h2><p>t</p></li></ul>", "- q\n  --\n\n  t\n"),
        (
            r#"<ul><li><input type="checkbox"><h2>q</h2><p>t</p></li></ul>"#,
            "- [ ] &#32;\n\n  q\n  --\n\n  t\n",
        ),
        ("<ol><li><h2>q</h2><p>t</p></li></ol>", "1. q\n   --\n\n   t\n"),
        (
            "<ul><li>a<ul><li><h2>q</h2><p>t</p></li></ul></li></ul>",
            "- a\n  * q\n    --\n\n    t\n",
        ),
    ] {
        assert_converts(html, &options, expected, &["<h2>q</h2>\n<p>t</p>"]);
    }
}
