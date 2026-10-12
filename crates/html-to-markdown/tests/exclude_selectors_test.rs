// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]

use html_to_markdown_rs::ConversionOptions;

fn convert(html: &str, opts: Option<ConversionOptions>) -> html_to_markdown_rs::error::Result<String> {
    html_to_markdown_rs::convert(html, opts).map(|r| r.content.unwrap_or_default())
}

#[test]
fn test_exclude_selectors_drops_matching_elements() {
    let html = r#"<body>
        <div class="cookie-banner">Accept cookies</div>
        <article><p>Main content here.</p></article>
        <div id="ad-container">Buy stuff</div>
    </body>"#;

    let options = ConversionOptions {
        exclude_selectors: vec![".cookie-banner".to_string(), "#ad-container".to_string()],
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    assert!(result.contains("Main content"), "Should keep main content");
    assert!(!result.contains("cookie"), "Should drop .cookie-banner element");
    assert!(!result.contains("Buy stuff"), "Should drop #ad-container element");
}

#[test]
fn test_exclude_selectors_drops_nested_content() {
    let html = r#"<body>
        <aside class="sidebar">
            <h2>Related articles</h2>
            <p>Some sidebar content</p>
        </aside>
        <main><p>Primary content.</p></main>
    </body>"#;

    let options = ConversionOptions {
        exclude_selectors: vec![".sidebar".to_string()],
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    assert!(result.contains("Primary content"), "Should keep main content");
    assert!(
        !result.contains("Related articles"),
        "Should drop heading inside excluded element"
    );
    assert!(
        !result.contains("sidebar content"),
        "Should drop paragraph inside excluded element"
    );
}

#[test]
fn test_exclude_selectors_empty_list_is_noop() {
    let html = r"<body><p>Hello world</p></body>";

    let options = ConversionOptions {
        exclude_selectors: vec![],
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();
    assert!(
        result.contains("Hello world"),
        "Empty exclude_selectors should not affect output"
    );
}

#[test]
fn test_exclude_selectors_invalid_selector_is_skipped() {
    let html = r"<body><p>Visible text</p></body>";

    let options = ConversionOptions {
        exclude_selectors: vec![String::new(), "p".to_string()],
        ..Default::default()
    };

    let _ = convert(html, Some(options));
}

#[test]
fn test_exclude_selectors_attribute_selector() {
    let html = r#"<body>
        <div role="complementary">Sidebar</div>
        <p>Main text</p>
    </body>"#;

    let options = ConversionOptions {
        exclude_selectors: vec!["[role='complementary']".to_string()],
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    assert!(result.contains("Main text"), "Should keep non-excluded content");
    assert!(
        !result.contains("Sidebar"),
        "Should drop element matching attribute selector"
    );
}

#[test]
fn test_exclude_selectors_plain_text_output() {
    let html = r#"<body>
        <div class="nav">Navigation links</div>
        <p>Article body text.</p>
    </body>"#;

    let options = ConversionOptions {
        exclude_selectors: vec![".nav".to_string()],
        output_format: html_to_markdown_rs::OutputFormat::Plain,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    assert!(
        result.contains("Article body text"),
        "Should keep body text in plain output"
    );
    assert!(
        !result.contains("Navigation links"),
        "Should drop excluded element in plain output"
    );
}

#[test]
fn should_exclude_inert_tags_before_retaining_their_hidden_content() {
    for tag in ["noscript", "template"] {
        for format in [
            html_to_markdown_rs::OutputFormat::Markdown,
            html_to_markdown_rs::OutputFormat::Plain,
        ] {
            let html = format!("<body><p>Visible.</p><{tag}><p>Excluded sentence.</p></{tag}></body>");
            let options = ConversionOptions {
                hidden_content: html_to_markdown_rs::HiddenContent::All,
                exclude_selectors: vec![tag.to_owned()],
                output_format: format,
                ..Default::default()
            };
            assert_eq!(
                convert(&html, Some(options)).expect("conversion succeeds").trim(),
                "Visible.",
                "{tag}, {format:?}"
            );
        }
    }
}

#[test]
fn should_apply_attribute_exclusions_to_kept_template_and_noscript_wrappers() {
    for tag in ["noscript", "template"] {
        for selector in [".secret", "#private", "[data-private]"] {
            let html = format!(
                "<body><p>Visible.</p><{tag} class=\"secret\" id=\"private\" data-private><p>Excluded sentence.</p></{tag}></body>"
            );
            let options = ConversionOptions {
                hidden_content: html_to_markdown_rs::HiddenContent::All,
                exclude_selectors: vec![selector.to_owned()],
                ..Default::default()
            };
            assert_eq!(
                convert(&html, Some(options)).expect("conversion succeeds").trim(),
                "Visible.",
                "{tag}, {selector}"
            );
        }
    }
}

#[test]
fn should_exclude_overlapping_hidden_subtrees_without_removing_adjacent_unicode_text() {
    let html = r#"<body><p>Before α.</p><section class="secret">
        <noscript class="secret"><p>Excluded fallback.</p></noscript>
        <template><p>Excluded template.</p></template><p>Excluded parent.</p>
        </section><p>After β.</p></body>"#;
    let options = ConversionOptions {
        hidden_content: html_to_markdown_rs::HiddenContent::All,
        exclude_selectors: vec![
            ".secret".to_owned(),
            "noscript".to_owned(),
            "template".to_owned(),
            ".secret".to_owned(),
        ],
        ..Default::default()
    };
    assert_eq!(
        convert(html, Some(options)).expect("conversion succeeds"),
        "Before α.\n\nAfter β.\n"
    );
}

#[test]
fn should_exclude_declarative_shadow_templates_when_retaining_reachable_content() {
    let html = r#"<body><div><template shadowrootmode="open"><p>Excluded shadow.</p></template>
        <p>Visible light.</p></div></body>"#;
    for choice in [
        html_to_markdown_rs::HiddenContent::Reachable,
        html_to_markdown_rs::HiddenContent::All,
    ] {
        let options = ConversionOptions {
            hidden_content: choice,
            exclude_selectors: vec!["template".to_owned()],
            ..Default::default()
        };
        assert_eq!(
            convert(html, Some(options)).expect("conversion succeeds"),
            "Visible light.\n",
            "{choice:?}"
        );
    }
}
