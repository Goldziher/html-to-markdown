#![allow(missing_docs)]

//! Issue #750: an inline `<svg>` keeps the text a reader gets from it (label, title, description,
//! text elements, foreign HTML), and a graphic with no text adds no word to the page.

use html_to_markdown_rs::{ConversionOptions, InlineDataMedia, OutputFormat, convert};

const SVG_URL: &str = "](data:image/svg+xml;base64,";
const ICON: &str = r#"<svg width="9" height="9"><path d="M1 1"/></svg>"#;
const STYLED_ICON: &str = r#"<svg><style>.cls{fill:#000}</style><path d="M1 1"/></svg>"#;

fn convert_plain(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

/// Converts `html` with `options`. Every input of this file also goes through both converters
/// with the options that select the fast one, and the two results must be equal (#766).
fn convert_with(html: &str, options: ConversionOptions) -> String {
    #[cfg(feature = "testkit")]
    {
        let on_tier = |tier_strategy| {
            convert_plain(
                html,
                ConversionOptions {
                    extract_metadata: false,
                    highlight_style: html_to_markdown_rs::HighlightStyle::None,
                    tier_strategy,
                    ..ConversionOptions::default()
                },
            )
        };
        assert_eq!(
            on_tier(html_to_markdown_rs::TierStrategy::Tier1),
            on_tier(html_to_markdown_rs::TierStrategy::Tier2),
            "the two converters differ for {html}"
        );
    }
    convert_plain(html, options)
}

fn text_only() -> ConversionOptions {
    ConversionOptions {
        inline_data_media: InlineDataMedia::AltTextOnly,
        ..ConversionOptions::default()
    }
}

fn assert_text_only(cases: &[(&str, &str)]) {
    let wrong: Vec<String> = cases
        .iter()
        .filter_map(|(html, expected)| {
            let actual = convert_with(html, text_only());
            (actual != *expected).then(|| format!("{html}\n  expected {expected:?}\n  actual   {actual:?}"))
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "{} of {} inputs are wrong:\n{}",
        wrong.len(),
        cases.len(),
        wrong.join("\n")
    );
}

#[test]
fn should_convert_the_inputs_of_the_issue() {
    assert_text_only(&[
        (
            r#"<p>before</p><svg width="90" height="20"><title>Sales chart</title><desc>described words</desc><text x="1" y="10">axis label words</text></svg><p>after</p>"#,
            "before\n\nSales chart described words axis label words\n\nafter\n",
        ),
        (
            r#"<p><svg width="9" height="9"><path d="M1 1"/></svg>note</p><p><a href="/x">2.3.1<svg width="9" height="9"><path d="M1 1"/></svg></a></p>"#,
            "note\n\n[2.3.1](/x)\n",
        ),
        (
            r#"<p>a<svg aria-hidden="true" width="9" height="9"><path d="M1 1"/></svg>b</p>"#,
            "ab\n",
        ),
    ]);
}

#[test]
fn should_never_write_the_placeholder_for_a_graphic_with_no_text() {
    let pages = [
        format!("<p>a {ICON} b</p>"),
        format!("<h2>Head {ICON}</h2>"),
        format!(r##"<h2>Head<a href="#x">{ICON}</a></h2>"##),
        format!(r#"<a href="/x">{ICON}</a>"#),
        format!(r#"<a href="/x"><div>{ICON}</div></a>"#),
        format!("<button>{ICON}</button>"),
        format!("<table><tr><th>A</th></tr><tr><td>{ICON}</td></tr></table>"),
        format!("<ul><li>{ICON} item</li></ul>"),
        ICON.to_string(),
    ];
    let option_sets = [
        ConversionOptions::default(),
        text_only(),
        ConversionOptions {
            output_format: OutputFormat::Plain,
            ..ConversionOptions::default()
        },
    ];
    for html in &pages {
        for options in &option_sets {
            let markdown = convert_with(html, options.clone());
            assert!(!markdown.contains("SVG Image"), "{html} -> {markdown:?}");
        }
    }
    // ~keep The same words are kept when the page holds them.
    assert_text_only(&[("<p>x <svg><title>SVG Image</title></svg> y</p>", "x SVG Image y\n")]);
}

#[test]
fn should_keep_the_text_of_a_graphic_in_reading_order() {
    assert_text_only(&[
        ("<p>a <svg><title>T</title></svg> b</p>", "a T b\n"),
        (
            concat!(
                "<svg><title>Sales</title><g><text x=\"0\" y=\"5\">Q1</text><text x=\"9\" y=\"5\">Q2</text></g>",
                "<g><text><tspan x=\"0\" dy=\"1em\">First line</tspan><tspan x=\"0\" dy=\"1em\">second line</tspan></text>",
                "<text>H<tspan dy=\"2\">2</tspan>O</text></g></svg>"
            ),
            "Sales Q1 Q2 First line second line H2O\n",
        ),
        (
            "<p>x <svg><svg><title>Inner</title><text>in</text></svg><text>out</text></svg> y</p>",
            "x Inner in out y\n",
        ),
        (
            "<svg><title>one\n  two</title><text>A &amp; B</text></svg>",
            "one two A & B\n",
        ),
        (
            r#"<p>x <svg aria-label="Label words"><title>T</title></svg> y</p>"#,
            "x Label words T y\n",
        ),
        (
            r#"<p>x <svg aria-label="Search"><title>Search</title></svg> y</p>"#,
            "x Search y\n",
        ),
        (
            "<svg><foreignObject><div>first <b>bold</b></div><div>second<br>third</div><style>.a{}</style></foreignObject></svg>",
            "first bold second third\n",
        ),
        (
            "<svg><foreignObject><p>html</p><svg><title>inner graphic</title></svg></foreignObject></svg>",
            "html inner graphic\n",
        ),
        (
            "<svg><switch><foreignObject><div>label html</div></foreignObject><text>label fallback</text></switch></svg>",
            "label html\n",
        ),
    ]);
}

#[test]
fn should_leave_out_what_the_graphic_does_not_draw() {
    assert_text_only(&[
        (
            concat!(
                "<p>x <svg><style>.a{fill:red}</style><script>var a=1</script><metadata>meta words</metadata>",
                "<defs><text id=\"d\">defs words</text></defs><symbol><text>symbol words</text></symbol>",
                "<clipPath><text>clip words</text></clipPath><text>drawn</text></svg> y</p>"
            ),
            "x drawn y\n",
        ),
        (
            "<p>x <svg><g><title>tip</title><path d=\"M1 1\"/></g><text><title>tip of text</title>shown</text></svg> y</p>",
            "x shown y\n",
        ),
        (
            r#"<p>a <svg aria-hidden="true"><title>Hidden name</title><desc>hidden words</desc><text>seen</text></svg> b</p>"#,
            "a seen b\n",
        ),
        (
            r#"<p>a <svg aria-hidden="true" aria-label="Hidden label"><path d="M1 1"/></svg> b</p>"#,
            "a b\n",
        ),
        (
            r##"<svg width="0" height="0"><symbol id="i"><title>Search</title><path d="M1 1"/></symbol></svg><p>go <svg><use href="#i"/></svg> now</p>"##,
            "go now\n",
        ),
        (
            r##"<a href="/s"><svg aria-label="Search"><use href="#i"/></svg></a>"##,
            "[Search](/s)\n",
        ),
    ]);
}

#[test]
fn should_convert_a_graphic_in_a_link_a_button_a_heading_a_cell_and_a_list() {
    assert_text_only(&[
        (r#"<a href="/x"><svg><path d="M1 1"/></svg> Home</a>"#, "[Home](/x)\n"),
        (r#"<a href="/x"><svg><path d="M1 1"/></svg></a>"#, "[/x](/x)\n"),
        (
            r#"<a href="/x"><svg><title>Home</title><path d="M1 1"/></svg></a>"#,
            "[Home](/x)\n",
        ),
        (r#"<a href="/x"><svg><text>drawn</text></svg></a>"#, "[drawn](/x)\n"),
        (r#"<button><svg><path d="M1 1"/></svg> Copy</button>"#, "Copy\n"),
        (r#"<button><svg><path d="M1 1"/></svg></button>"#, ""),
        (r#"<h2>Head <svg><path d="M1 1"/></svg></h2>"#, "## Head\n"),
        ("<h2>Head <svg><title>T</title></svg></h2>", "## Head T\n"),
        (
            r##"<h2>Head<a href="#x"><svg><path d="M1 1"/></svg></a></h2>"##,
            "## Head\n",
        ),
        (
            r#"<table><tr><th>A</th><th>B</th></tr><tr><td><svg><path d="M1 1"/></svg></td><td>x <svg><title>T</title></svg> y</td></tr></table>"#,
            "| A | B     |\n| --- | ----- |\n|   | x T y |\n",
        ),
        (
            r#"<ul><li><svg><path d="M1 1"/></svg> item</li><li><svg><title>T</title></svg></li></ul>"#,
            "- item\n- T\n",
        ),
    ]);
}

#[test]
fn should_write_no_text_that_the_graphic_hides() {
    assert_text_only(&[
        (
            concat!(
                "<svg><text display=\"none\">unseen</text><g display=\"none\"><text>unseen</text></g>",
                "<text visibility=\"hidden\">unseen</text><foreignObject visibility=\"hidden\"><div>unseen</div></foreignObject>",
                "<text style=\"display:none\">unseen</text><text>seen</text></svg>"
            ),
            "seen\n",
        ),
        (
            r#"<p>a <svg display="none"><title>T</title><text>unseen</text></svg> b</p>"#,
            "a b\n",
        ),
        (
            "<p>a <svg><title display=\"NONE\">T</title><text>seen</text></svg> b</p>",
            "a seen b\n",
        ),
        // ~keep `display="inline"` hides nothing, and neither does a transparent element: the
        // ~keep converter keeps the same text outside a graphic.
        (
            "<p><span style=\"opacity:0\">faint</span> <svg><text display=\"inline\" opacity=\"0\">faint too</text></svg></p>",
            "faint faint too\n",
        ),
    ]);
}

#[test]
fn should_draw_the_child_of_a_switch_that_a_reader_of_english_gets() {
    assert_text_only(&[
        (
            r#"<svg><switch><text systemLanguage="de">Hallo Welt</text><text systemLanguage="en">Hello world</text><text>Fallback</text></switch></svg>"#,
            "Hello world\n",
        ),
        (
            r#"<svg><switch><text systemLanguage="fr, en-GB">Hello world</text><text systemLanguage="de">Hallo Welt</text><text>Fallback</text></switch></svg>"#,
            "Hello world\n",
        ),
        (
            r#"<svg><switch><g systemLanguage="zz"><text>never shown</text></g><g><text>default shown</text></g></switch></svg>"#,
            "default shown\n",
        ),
        (
            r#"<p>a <svg><switch><text systemLanguage="de">Hallo Welt</text><text systemLanguage="">leer</text></switch></svg> b</p>"#,
            "a b\n",
        ),
    ]);
}

#[test]
fn should_escape_the_text_of_a_graphic_at_the_start_of_a_line_as_a_text_node_is() {
    let mut wrong = Vec::new();
    for words in [
        "# heading words",
        "1. one",
        "1) one",
        "- item",
        "+ item",
        "* item",
        "&gt; quote",
        "| a | b |",
        "```",
        "~~~",
        "---",
        "===",
        "[a]: b",
        "plain words",
    ] {
        for template in [
            "<p>{X}</p>",
            "<p>before<br>{X}</p>",
            "<ul><li>{X}</li></ul>",
            "<blockquote>{X}</blockquote>",
            "<div>{X}</div>",
        ] {
            let graphic = template.replace("{X}", &format!("<svg><text>{words}</text></svg>"));
            let span = template.replace("{X}", &format!("<span>{words}</span>"));
            let (graphic_out, span_out) = (convert_with(&graphic, text_only()), convert_with(&span, text_only()));
            if graphic_out != span_out {
                wrong.push(format!("{graphic}\n  graphic {graphic_out:?}\n  span    {span_out:?}"));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} inputs differ from a span:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    assert_text_only(&[
        ("<p><svg><text># heading words</text></svg></p>", "\\# heading words\n"),
        ("<p><svg><text>1. one</text></svg></p>", "1\\. one\n"),
        ("<p><svg><text>- item</text></svg></p>", "\\- item\n"),
    ]);
}

#[test]
fn should_write_the_text_as_the_alt_text_of_a_kept_image() {
    for (html, start, end) in [
        (
            "<p>x <svg><title>T</title><text>label</text></svg> y</p>",
            "x ![T label](data:image/svg+xml;base64,",
            ") y\n",
        ),
        (
            r#"<p>a <svg><path d="M1 1"/></svg> b</p>"#,
            "a ![](data:image/svg+xml;base64,",
            ") b\n",
        ),
        (
            "<svg><text>x](https://evil.example)y</text></svg>",
            "![x\\](https://evil.example)y](data:image/svg+xml;base64,",
            ")\n",
        ),
    ] {
        let markdown = convert_with(html, ConversionOptions::default());
        assert!(
            markdown.starts_with(start) && markdown.ends_with(end),
            "{html} -> {markdown:?}"
        );
        assert_eq!(markdown.matches(SVG_URL).count(), 1, "{html} -> {markdown:?}");
    }
}

#[test]
fn should_escape_the_text_like_a_text_node() {
    let options = ConversionOptions {
        escape_asterisks: true,
        escape_underscores: true,
        ..text_only()
    };
    assert_eq!(
        convert_with("<p>x <svg><title>a*b_c</title></svg> y a*b_c</p>", options),
        "x a\\*b\\_c y a\\*b\\_c\n"
    );
}

#[test]
fn should_leave_the_other_image_options_as_they_are() {
    let html = "<p>before</p><svg><title>Sales chart</title><text>axis label</text></svg><p>after</p>";
    for options in [
        ConversionOptions {
            inline_data_media: InlineDataMedia::DropElement,
            ..ConversionOptions::default()
        },
        ConversionOptions {
            skip_images: true,
            ..ConversionOptions::default()
        },
        ConversionOptions {
            skip_images: true,
            output_format: OutputFormat::Plain,
            ..ConversionOptions::default()
        },
    ] {
        assert_eq!(convert_with(html, options), "before\n\nafter\n");
    }
}

#[test]
fn should_write_the_text_of_a_graphic_in_plain_text_output() {
    let plain = || ConversionOptions {
        output_format: OutputFormat::Plain,
        ..ConversionOptions::default()
    };
    assert_eq!(
        convert_with(
            "<p>a <svg><style>.a { fill: red }</style><title>T</title><text>label</text></svg> b</p>",
            plain()
        ),
        "a T label b\n"
    );
    assert_eq!(
        convert_with(r#"<p>a <svg><path d="M1 1"/></svg> b</p>"#, plain()),
        "a  b\n"
    );
}

#[test]
fn should_read_an_image_element_from_its_alt_attribute_only() {
    // ~keep An `<img>` does not show the text inside the file it loads, so its alt text is its text.
    assert_text_only(&[
        (r#"<p>a <img src="/a.svg" alt="logo"> b</p>"#, "a ![logo](/a.svg) b\n"),
        (
            r#"<p>a <img src="data:image/svg+xml;base64,PHN2Zy8+" alt="logo"> b</p>"#,
            "a logo b\n",
        ),
        (
            r#"<p>y <img src="data:image/svg+xml,%3Csvg%3E%3Ctext%3Ehi%3C/text%3E%3C/svg%3E"> x</p>"#,
            "y x\n",
        ),
    ]);
}

#[cfg(feature = "inline-images")]
#[test]
fn should_describe_an_extracted_graphic_by_its_label_or_title() {
    for (html, expected) in [
        ("<svg><title>Logo</title><text>drawn</text></svg>", Some("Logo")),
        (r#"<svg aria-label="Label"><title>Logo</title></svg>"#, Some("Label")),
        (r#"<svg><path d="M1 1"/></svg>"#, None),
        // ~keep These words were the mark for "no title", so a graphic with this title lost it.
        ("<svg><title>SVG Image</title></svg>", Some("SVG Image")),
    ] {
        let options = ConversionOptions {
            extract_images: true,
            capture_svg: true,
            ..ConversionOptions::default()
        };
        let result = convert(html, Some(options)).expect("conversion should succeed");
        assert_eq!(result.images.len(), 1, "{html}");
        assert_eq!(result.images[0].description.as_deref(), expected, "{html}");
    }
}

#[cfg(feature = "testkit")]
#[test]
fn should_write_the_same_alt_text_in_both_tiers() {
    use html_to_markdown_rs::prescan::PrescanReport;
    use html_to_markdown_rs::{HighlightStyle, TierStrategy, tier1};

    let tier1_options = || ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    for (html, start) in [
        (
            r#"<p>before</p><svg aria-label="Chart"><title>Sales</title><desc>by quarter</desc><text>Q1</text><defs><text>hidden</text></defs></svg>"#,
            "before\n\n![Chart Sales by quarter Q1](data:image/svg+xml;base64,",
        ),
        (
            r#"<p>before</p><svg><path d="M1 1"/></svg>"#,
            "before\n\n![](data:image/svg+xml;base64,",
        ),
        (
            "<p>before</p><svg><title>A &amp; B</title></svg>",
            "before\n\n![A & B](data:image/svg+xml;base64,",
        ),
    ] {
        let fast = tier1::run(html, &PrescanReport::default(), &tier1_options())
            .unwrap_or_else(|reason| panic!("Tier 1 bailed on {html:?}: {reason:?}"));
        let full = convert_with(
            html,
            ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                ..tier1_options()
            },
        );
        assert!(fast.starts_with(start), "{html} -> {fast:?}");
        assert_eq!(fast, full, "tier divergence for {html:?}");
    }

    // ~keep Through `convert`, a page the fast converter takes reads as the full converter writes it.
    let mut converted = 0;
    let mut wrong = Vec::new();
    for html in [
        r#"<p>a <svg ARIA-LABEL="Upper &amp; lower"><TITLE>T</TITLE></svg> b</p>"#,
        r#"<p>a <svg><text style="display:none">unseen</text><text>seen</text></svg> b</p>"#,
        "<p>a <svg><text hidden>unseen</text><text>seen</text></svg> b</p>",
        r#"<p>a <svg><g><text>seen</text><text style="visibility: hidden">unseen</text></g></svg> b</p>"#,
        r#"<p>a <svg aria-hidden="true"><title>T</title><text>seen</text></svg> b</p>"#,
        "<p>a <svg><title>one <b>two</b>\nthree</title></svg> b</p>",
        "<p>a <svg><svg><title>Inner</title></svg><text>out</text></svg> b</p>",
        "<p>a <svg><foreignObject><div>html</div><p>more</p></foreignObject></svg> b</p>",
        r#"<p><a href="/x"><svg><title>T</title></svg> Home</a></p>"#,
        r#"<p><a href="/x"><svg><path d="M1 1"/></svg></a></p>"#,
        "<ul><li><svg><title>T</title></svg> item</li></ul>",
        "<table><tr><th>H</th></tr><tr><td><svg><title>T</title></svg></td></tr></table>",
        "<p>a <svg><text>x<tspan x=\"0\">y</tspan></text><style>.a{}</style></svg> b</p>",
    ] {
        converted += usize::from(tier1::run(html, &PrescanReport::default(), &tier1_options()).is_ok());
        let fast = convert_plain(
            html,
            ConversionOptions {
                tier_strategy: TierStrategy::Tier1,
                ..tier1_options()
            },
        );
        let full = convert_plain(
            html,
            ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                ..tier1_options()
            },
        );
        if fast != full {
            wrong.push(format!("{html}\n  fast {fast:?}\n  full {full:?}"));
        }
    }
    assert!(converted > 0, "the fast converter converted none of the inputs");
    assert!(
        wrong.is_empty(),
        "{} tier divergences:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[cfg(feature = "testkit")]
#[test]
fn should_leave_a_graphic_with_a_hidden_element_to_the_full_converter_and_no_other() {
    use html_to_markdown_rs::prescan::PrescanReport;
    use html_to_markdown_rs::{HighlightStyle, tier1};

    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    let fast = |html: &str| tier1::run(html, &PrescanReport::default(), &options);
    for html in [
        r#"<p>a <svg><text style="display:none">unseen</text><text>seen</text></svg> b</p>"#,
        "<p>a <svg><text hidden>unseen</text><text>seen</text></svg> b</p>",
    ] {
        assert!(fast(html).is_err(), "the fast converter kept {html}");
    }
    // ~keep A `<` inside a tag is part of an attribute, so the scan does not read it as a tag.
    for html in [
        r#"<p>a <svg><text data-note="<b hidden>">seen</text></svg> b</p>"#,
        r#"<p>a <svg><text data-note="<b style='display:none'>">seen</text></svg> b</p>"#,
        "<p>a <svg><title>T</title><text>seen</text></svg> b</p>",
    ] {
        let markdown = fast(html).unwrap_or_else(|reason| panic!("the fast converter left {html}: {reason:?}"));
        assert!(
            markdown.starts_with("a ![") && markdown.ends_with(") b\n"),
            "{html} -> {markdown:?}"
        );
    }
}

#[cfg(feature = "testkit")]
#[test]
fn should_leave_only_a_named_link_with_no_text_to_the_full_converter() {
    use html_to_markdown_rs::prescan::PrescanReport;
    use html_to_markdown_rs::{HighlightStyle, tier1};

    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    let fast = |html: &str| tier1::run(html, &PrescanReport::default(), &options);
    for html in [
        r#"<p><a href="/page" aria-label="Next page"><i class="fa"></i></a></p>"#,
        r#"<p><a href="/page" title="Go to page"><i class="fa"></i></a></p>"#,
    ] {
        assert!(fast(html).is_err(), "the fast converter kept {html}");
    }
    // ~keep A link with no text and no name is left to the full converter only when it points
    // ~keep into its own page. With a `<base>` element and no address of the page, `#x` is not
    // ~keep known to be the page, so the fast converter converts it.
    assert!(
        fast(r##"<p>a <a href="#x"><i class="fa"></i></a> b</p>"##).is_err(),
        "the fast converter kept a link into its own page with no text"
    );
    let with_base_element = r##"<html><head><base href="https://other.example/page"></head><body><p>a <a href="#x"><i class="fa"></i></a> b</p></body></html>"##;
    assert!(
        fast(with_base_element).is_ok(),
        "the fast converter left {with_base_element}: {:?}",
        fast(with_base_element)
    );
    // ~keep A link that has text keeps its text as its label, so the fast converter converts it.
    for (html, expected) in [
        (
            r#"<p><a href="/page" aria-label="Next page" title="Go to page">Next</a></p>"#,
            "[Next](/page \"Go to page\")\n",
        ),
        (r#"<p><a href="/page">Next</a></p>"#, "[Next](/page)\n"),
    ] {
        let markdown = fast(html).unwrap_or_else(|reason| panic!("the fast converter left {html}: {reason:?}"));
        assert_eq!(markdown, expected, "{html}");
    }
}

#[cfg(feature = "testkit")]
#[test]
fn should_write_a_graphic_in_a_heading_as_text_whatever_the_other_options_are() {
    use html_to_markdown_rs::{HighlightStyle, TierStrategy};

    for (html, expected) in [
        (
            r#"<h2>Head <svg width="40" height="20"><title>T</title></svg></h2>"#.to_string(),
            "## Head T\n",
        ),
        (format!("<h2>Head {ICON}</h2>"), "## Head\n"),
        (format!(r##"<h2>Title <a href="#title">{ICON}</a></h2>"##), "## Title\n"),
        (
            format!(r#"<h2>Title <a href="/page">{ICON}</a></h2>"#),
            "## Title [/page](/page)\n",
        ),
    ] {
        for (name, options) in [
            ("default", ConversionOptions::default()),
            (
                "no metadata",
                ConversionOptions {
                    extract_metadata: false,
                    ..ConversionOptions::default()
                },
            ),
            (
                "no metadata, no highlight",
                ConversionOptions {
                    extract_metadata: false,
                    highlight_style: HighlightStyle::None,
                    ..ConversionOptions::default()
                },
            ),
            (
                "fast converter asked for",
                ConversionOptions {
                    extract_metadata: false,
                    highlight_style: HighlightStyle::None,
                    tier_strategy: TierStrategy::Tier1,
                    ..ConversionOptions::default()
                },
            ),
        ] {
            assert_eq!(convert_with(&html, options), expected, "{name}: {html}");
        }
    }
}

include!("support/issue_750_graphic_links.rs");
