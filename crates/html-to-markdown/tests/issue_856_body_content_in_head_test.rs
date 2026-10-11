//! Issue #856: content of the body that is written before the end tag of the head ends the
//! head, as in a browser. The converter writes it in front of the body.

#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn run(html: &str, extract_metadata: bool, tier_strategy: TierStrategy) -> String {
    let options = ConversionOptions {
        extract_metadata,
        tier_strategy,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("the conversion succeeds")
        .content
        .unwrap_or_default()
}

/// The text of the default route, after a check that the full and the fast converter give it too.
fn on_every_route(html: &str, extract_metadata: bool) -> String {
    let auto = run(html, extract_metadata, TierStrategy::Auto);
    let full = run(html, extract_metadata, TierStrategy::Tier2);
    let fast = run(html, extract_metadata, TierStrategy::Tier1);
    assert_eq!(full, auto, "full against default for {html:?}");
    assert_eq!(fast, auto, "fast against default for {html:?}");
    auto
}

const IMAGE: &str = r#"<img src="p.gif" alt="A1">"#;

#[test]
fn the_head_ends_at_an_element_that_is_not_head_content() {
    let cases = [
        (format!("<head>{IMAGE}</head><p>End</p>"), "![A1](p.gif)\n\nEnd\n"),
        (
            format!("<html><head>{IMAGE}</head><body><p>End</p></body></html>"),
            "![A1](p.gif)\n\nEnd\n",
        ),
        ("<head><span>S</span></head><p>End</p>".to_owned(), "S\n\nEnd\n"),
        ("<head><h1>H</h1></head><p>End</p>".to_owned(), "# H\n\nEnd\n"),
        ("<head>text</head><p>End</p>".to_owned(), "text\n\nEnd\n"),
    ];
    for (html, expected) in cases {
        for extract_metadata in [true, false] {
            assert_eq!(on_every_route(&html, extract_metadata), expected, "{html:?}");
        }
    }
}

#[test]
fn the_metadata_before_the_body_content_of_a_head_stays_metadata() {
    let html = format!("<head><title>T</title>{IMAGE}</head><p>End</p>");
    assert_eq!(
        on_every_route(&html, true),
        "---\ntitle: T\n---\n\n![A1](p.gif)\n\nEnd\n"
    );
    assert_eq!(on_every_route(&html, false), "![A1](p.gif)\n\nEnd\n");
}

#[test]
fn the_fast_converter_writes_the_body_content_of_a_head() {
    for inner in ["<p>In</p>", "<div>In</div>"] {
        let html = format!("<head>{IMAGE}{inner}</head><p>End</p>");
        assert_eq!(on_every_route(&html, false), "![A1](p.gif)\n\nIn\n\nEnd\n", "{html:?}");
    }
}

#[test]
fn head_content_stays_in_the_head() {
    let cases = [
        format!("<head><title>T</title><noscript>{IMAGE}</noscript></head><p>End</p>"),
        format!("<head><title>T</title><template>{IMAGE}</template></head><p>End</p>"),
        "<head><!-- <img> --><title>T</title></head><p>End</p>".to_owned(),
        "<head> \n <title>T</title> \n </head>\n<p>End</p>".to_owned(),
        r#"<head><title>T</title><script>var a="<p>";</script><style>p{}</style></head><p>End</p>"#.to_owned(),
    ];
    for html in cases {
        assert_eq!(on_every_route(&html, true), "---\ntitle: T\n---\n\nEnd\n", "{html:?}");
        assert_eq!(on_every_route(&html, false), "End\n", "{html:?}");
    }
}

#[test]
fn the_metadata_after_the_body_content_of_a_head_stays_metadata() {
    let image = "<head><title>T</title><img src=i.png><meta name=description content=D></head><p>one";
    assert_eq!(
        on_every_route(image, true),
        "---\nmeta-description: D\ntitle: T\n---\n\n![](i.png)\n\none\n"
    );
    assert_eq!(on_every_route(image, false), "![](i.png)\n\none\n");
    let office = "<head><title>T</title><o:p></o:p><meta name=description content=D></head><p>one";
    assert!(on_every_route(office, true).starts_with("---\nmeta-description: D\ntitle: T\n---\n"));
}

#[test]
fn a_title_outside_the_source_head_or_in_an_svg_is_no_page_title() {
    let in_body = "<head><title>T</title></head><body><title>B</title><p>one</p></body>";
    assert!(on_every_route(in_body, true).starts_with("---\ntitle: T\n---\n"));
    let in_svg = "<head><svg><title>X</title></svg></head><p>one</p>";
    assert!(!on_every_route(in_svg, true).contains("title: X"));
}
