#[test]
fn should_not_label_an_icon_link_with_the_style_sheet_of_its_graphic() {
    let inline = format!(r#"<a href="/x">{STYLED_ICON}</a>"#);
    let in_block = format!(r#"<a href="/x"><div>{STYLED_ICON}</div></a>"#);
    for html in [&inline, &in_block] {
        for (name, options, expected) in [
            ("alt_text_only", text_only(), "[/x](/x)\n"),
            (
                "drop_element",
                ConversionOptions {
                    inline_data_media: InlineDataMedia::DropElement,
                    ..ConversionOptions::default()
                },
                "",
            ),
            (
                "skip_images",
                ConversionOptions {
                    skip_images: true,
                    ..ConversionOptions::default()
                },
                "[/x](/x)\n",
            ),
        ] {
            assert_eq!(convert_with(html, options), expected, "{name}: {html}");
        }
    }
    // ~keep The text of the link itself is still its label.
    assert_eq!(
        convert_with(
            &format!(r#"<a href="/x"><div>{STYLED_ICON}</div><div>Docs</div></a>"#),
            text_only()
        ),
        "[Docs](/x)\n"
    );
}

#[test]
fn should_keep_a_link_around_a_graphic_with_no_text_and_name_it_as_a_browser_does() {
    let with_base = || ConversionOptions {
        base_url: Some("https://example.org/doc".to_string()),
        ..ConversionOptions::default()
    };
    for (html, options, expected) in [
        // ~keep The label of the link, then its title, then its address: a link that leaves the
        // ~keep page is never lost.
        (
            format!(r#"<a href="https://example.org/meta" aria-label="Meta Open Source"><div>{ICON}</div></a>"#),
            ConversionOptions::default(),
            "[Meta Open Source](https://example.org/meta)\n",
        ),
        (
            format!(r#"<a href="/page" title="Open page"><div>{ICON}</div></a>"#),
            ConversionOptions::default(),
            "[Open page](/page \"Open page\")\n",
        ),
        (
            format!(r#"<a href="/page"><p>{ICON}</p></a>"#),
            ConversionOptions::default(),
            "[/page](/page)\n",
        ),
        (
            format!(r#"<h2>Head <a href="/page">{ICON}</a></h2>"#),
            ConversionOptions::default(),
            "## Head [/page](/page)\n",
        ),
        (
            format!(r#"<ul><li><a href="/prev"><div>{ICON}</div></a></li></ul>"#),
            ConversionOptions::default(),
            "- [/prev](/prev)\n",
        ),
        (
            format!(r#"<a href="/x"><div>{STYLED_ICON}</div></a>"#),
            ConversionOptions::default(),
            "[/x](/x)\n",
        ),
        (
            format!(r#"<p><a href="/files/report.pdf" aria-label="Report">{ICON}</a></p>"#),
            text_only(),
            "[Report](/files/report.pdf)\n",
        ),
        // ~keep The same link around an image with no alt text follows the same rule.
        (
            r#"<a href="/page"><div><img src="/i.png"></div></a>"#.to_string(),
            ConversionOptions::default(),
            "[/page](/page)\n",
        ),
        (
            r#"<a href="/page" aria-label="Photo"><div><img src="/i.png"></div></a>"#.to_string(),
            ConversionOptions::default(),
            "[Photo](/page)\n",
        ),
        // ~keep A link into its own page whose content gives no text is left out: the icon of a
        // ~keep heading permalink.
        (
            format!(r##"<h2>Head<a href="#x" title="Link for this heading" aria-label="Link">{ICON}</a></h2>"##),
            ConversionOptions::default(),
            "## Head\n",
        ),
        (
            format!(r#"<h2>Head<a href="https://example.org/doc#part">{ICON}</a></h2>"#),
            with_base(),
            "## Head\n",
        ),
        (
            format!(r#"<h2>Head<a href="https://example.org/other#part">{ICON}</a></h2>"#),
            with_base(),
            "## Head[https://example.org/other#part](https://example.org/other#part)\n",
        ),
        (
            r##"<h2>Head <a href="#x"><svg><title>Anchor</title></svg></a></h2>"##.to_string(),
            ConversionOptions::default(),
            "## Head [Anchor](#x)\n",
        ),
    ] {
        assert_eq!(convert_with(&html, options), expected, "{html}");
    }
}

#[test]
fn should_compare_a_resolved_link_with_the_address_of_the_page_and_not_with_the_base_element() {
    const PAGE: &str = "https://example.org/doc";
    const OTHER: &str = r#"<base href="https://other.example/page">"#;
    let page = |base_element: &str, href: &str| {
        format!(
            r#"<html><head>{base_element}</head><body><p>a <a href="{href}" aria-label="Part"><i class="fa"></i></a> b</p></body></html>"#
        )
    };
    let options = |base_url: Option<&str>| ConversionOptions {
        base_url: base_url.map(str::to_string),
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    for (base_element, href, base_url, expected) in [
        // ~keep With a `<base>` element, `#x` names the document of the base: another document.
        (OTHER, "#x", Some(PAGE), "a [Part](https://other.example/page#x) b\n"),
        (
            OTHER,
            "https://other.example/page#x",
            Some(PAGE),
            "a [Part](https://other.example/page#x) b\n",
        ),
        (
            r#"<base href="/">"#,
            "#x",
            Some(PAGE),
            "a [Part](https://example.org/#x) b\n",
        ),
        (
            r#"<base href="/">"#,
            "/#x",
            Some(PAGE),
            "a [Part](https://example.org/#x) b\n",
        ),
        // ~keep The page's own address is left out, whatever the base element is.
        (OTHER, "https://example.org/doc#x", Some(PAGE), "a b\n"),
        (r#"<base href="https://example.org/doc">"#, "#x", Some(PAGE), "a b\n"),
        ("", "#x", Some(PAGE), "a b\n"),
        ("", "https://example.org/doc#x", Some(PAGE), "a b\n"),
        // ~keep Another spelling of the address is another address, and so is one with no fragment.
        (
            "",
            "https://example.org/doc/#x",
            Some(PAGE),
            "a [Part](https://example.org/doc/#x) b\n",
        ),
        (
            "",
            "https://example.org/%64oc#x",
            Some(PAGE),
            "a [Part](https://example.org/%64oc#x) b\n",
        ),
        (
            "",
            "https://example.org/doc",
            Some(PAGE),
            "a [Part](https://example.org/doc) b\n",
        ),
        // ~keep No address of the page: `#x` is the page itself only with no `<base>` element.
        (OTHER, "#x", None, "a [Part](#x) b\n"),
        ("", "#x", None, "a b\n"),
    ] {
        let html = page(base_element, href);
        assert_eq!(
            convert_with(&html, options(base_url)),
            expected,
            "{html} with {base_url:?}"
        );
        // ~keep The same options on each converter.
        #[cfg(feature = "testkit")]
        {
            let on_tier = |tier_strategy| {
                convert_plain(
                    &html,
                    ConversionOptions {
                        highlight_style: html_to_markdown_rs::HighlightStyle::None,
                        tier_strategy,
                        ..options(base_url)
                    },
                )
            };
            assert_eq!(
                on_tier(html_to_markdown_rs::TierStrategy::Tier1),
                on_tier(html_to_markdown_rs::TierStrategy::Tier2),
                "the two converters differ for {html} with {base_url:?}"
            );
        }
    }
}

#[test]
fn should_leave_out_a_link_into_its_own_page_whose_content_gives_no_text() {
    let with_base = || ConversionOptions {
        base_url: Some("https://example.org/doc".to_string()),
        ..ConversionOptions::default()
    };
    // ~keep One rule, whatever the empty content is and whether or not the link has a name.
    // ~keep `convert_with` also compares the two converters for each input.
    for (html, options, expected) in [
        (
            r##"<p>a<a href="#top" aria-label="Back to top"></a>b</p>"##,
            ConversionOptions::default(),
            "ab\n",
        ),
        (
            r##"<h2>Title <a href="#id" aria-label="Link for this heading"></a></h2>"##,
            ConversionOptions::default(),
            "## Title\n",
        ),
        (
            r##"<h2>Title <a href="#id" title="Link for this heading"><span class="icon"></span></a></h2>"##,
            ConversionOptions::default(),
            "## Title\n",
        ),
        (
            r##"<h2>Head<a href="#x"><img src="/i.png"></a></h2>"##,
            ConversionOptions::default(),
            "## Head\n",
        ),
        (
            r##"<p>a<a href="#top"></a>b</p>"##,
            ConversionOptions::default(),
            "ab\n",
        ),
        (
            r##"<p>a<a href="#" aria-label="Menu"><i class="fa"></i></a>b</p>"##,
            ConversionOptions::default(),
            "ab\n",
        ),
        (
            r#"<p>a<a href="https://example.org/doc#part" aria-label="Part"></a>b</p>"#,
            with_base(),
            "ab\n",
        ),
        // ~keep A link that leaves the page is kept and named, and a link into the page that has
        // ~keep text is kept with its text.
        (
            r#"<p>a<a href="https://example.org/other#part" aria-label="Part"></a>b</p>"#,
            with_base(),
            "a[Part](https://example.org/other#part)b\n",
        ),
        (
            r##"<p><a href="#top" aria-label="Back to top">Top</a></p>"##,
            ConversionOptions::default(),
            "[Top](#top)\n",
        ),
        (
            r##"<p><a href="#top"><img src="/up.png" alt="Up"></a></p>"##,
            ConversionOptions::default(),
            "[![Up](/up.png)](#top)\n",
        ),
    ] {
        assert_eq!(convert_with(html, options), expected, "{html}");
    }
    // ~keep With a base, the two converters agree on the address that is the page's own.
    #[cfg(feature = "testkit")]
    for html in [
        r#"<p>a<a href="https://example.org/doc#part" aria-label="Part"></a>b</p>"#,
        r#"<p>a<a href="https://example.org/doc#part"><i class="fa"></i></a>b</p>"#,
        r#"<p>a<a href="https://example.org/other#part" aria-label="Part"></a>b</p>"#,
    ] {
        let on_tier = |tier_strategy| {
            convert_plain(
                html,
                ConversionOptions {
                    extract_metadata: false,
                    highlight_style: html_to_markdown_rs::HighlightStyle::None,
                    tier_strategy,
                    ..with_base()
                },
            )
        };
        assert_eq!(
            on_tier(html_to_markdown_rs::TierStrategy::Tier1),
            on_tier(html_to_markdown_rs::TierStrategy::Tier2),
            "the two converters differ for {html}"
        );
    }
}

#[test]
fn should_name_any_link_whose_content_gives_no_text_in_both_converters() {
    const PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
    let drop = || ConversionOptions {
        inline_data_media: InlineDataMedia::DropElement,
        ..ConversionOptions::default()
    };
    for (html, options, expected) in [
        // ~keep A link with no graphic: an icon font. `convert_with` also compares the converters.
        (
            r#"<p><a href="/page" aria-label="Next page"><i class="fa fa-cart"></i></a></p>"#.to_string(),
            ConversionOptions::default(),
            "[Next page](/page)\n",
        ),
        (
            r#"<h2>Head <a href="/page" aria-label="Next page"><i class="fa fa-cart"></i></a></h2>"#.to_string(),
            ConversionOptions::default(),
            "## Head [Next page](/page)\n",
        ),
        (
            r#"<table><tr><th>H</th></tr><tr><td><a href="/page" title="Go to page"><i class="fa"></i></a></td></tr></table>"#
                .to_string(),
            ConversionOptions::default(),
            "| H                                |\n| -------------------------------- |\n| [Go to page](/page \"Go to page\") |\n",
        ),
        // ~keep The `aria-label` comes before the `title`, as in the name a browser computes.
        (
            r#"<p><a href="/page" aria-label="Next page" title="Go to page"><i class="fa"></i></a></p>"#.to_string(),
            ConversionOptions::default(),
            "[Next page](/page \"Go to page\")\n",
        ),
        // ~keep An `aria-label` of white space only is no name.
        (
            r#"<p><a href="/page" aria-label="   " title="Go to page"><i class="fa"></i></a></p>"#.to_string(),
            ConversionOptions::default(),
            "[Go to page](/page \"Go to page\")\n",
        ),
        // ~keep A named link is kept when `drop_element` removes its content.
        (
            format!(r#"<p>a <a href="/files/report.pdf" aria-label="Download report">{ICON}</a> b</p>"#),
            drop(),
            "a [Download report](/files/report.pdf) b\n",
        ),
        (
            format!(r#"<p>a <a href="/files/report.pdf" aria-label="Download report"><img src="{PNG}"></a> b</p>"#),
            drop(),
            "a [Download report](/files/report.pdf) b\n",
        ),
        (
            format!(r#"<p>a <a href="/files/report.pdf"><img src="{PNG}"></a> b</p>"#),
            drop(),
            "a b\n",
        ),
    ] {
        assert_eq!(convert_with(&html, options), expected, "{html}");
    }
}

#[test]
fn should_name_a_link_that_has_no_content_at_all() {
    // ~keep `convert_with` also compares the two converters for each input.
    for (html, expected) in [
        (
            r#"<p>a <a href="/page" aria-label="Next page"></a> b</p>"#,
            "a [Next page](/page) b\n",
        ),
        (
            r#"<p>a <a href="/page" title="Go to page"></a> b</p>"#,
            "a [Go to page](/page \"Go to page\") b\n",
        ),
        (
            r#"<p>a <a href="/page" aria-label="Next page"> </a> b</p>"#,
            "a [Next page](/page) b\n",
        ),
        (
            r#"<p>a <a href="/page" aria-label="Next page"><!-- icon --></a> b</p>"#,
            "a [Next page](/page) b\n",
        ),
        (
            r#"<p>a <a href="/page" aria-label="Next page"><span></span></a> b</p>"#,
            "a [Next page](/page) b\n",
        ),
        (
            r#"<p>a <a href="/page" aria-label="Next page"><span hidden>h</span></a> b</p>"#,
            "a [Next page](/page) b\n",
        ),
        (
            r#"<ul><li><a href="https://x.com/mozdevnet" aria-label="MDN on X"></a></li></ul>"#,
            "- [MDN on X](https://x.com/mozdevnet)\n",
        ),
        // ~keep No name and no child node: the label stays empty, as before.
        (r#"<p>a <a href="/page"></a> b</p>"#, "a [](/page) b\n"),
        (r#"<p>a <a href="/page" aria-label="  "></a> b</p>"#, "a [](/page) b\n"),
    ] {
        assert_eq!(convert_with(html, ConversionOptions::default()), expected, "{html}");
    }
    // ~keep No name, and content that gives no text: the address, as before (full converter).
    for html in [
        r#"<p>a <a href="/page"> </a> b</p>"#,
        r#"<p>a <a href="/page"><span></span></a> b</p>"#,
    ] {
        assert_eq!(
            convert_plain(html, ConversionOptions::default()),
            "a [/page](/page) b\n",
            "{html}"
        );
    }
}

#[cfg(feature = "metadata")]
#[test]
fn should_record_a_kept_link_under_the_label_the_markdown_shows() {
    let html = format!(
        r#"<a href="https://example.org/meta" aria-label="Meta Open Source"><div>{ICON}</div></a><a href="/page"><p>{ICON}</p></a>"#
    );
    let result = convert(&html, None).expect("conversion should succeed");
    let links: Vec<(&str, &str)> = result
        .metadata
        .links
        .iter()
        .map(|link| (link.href.as_str(), link.text.as_str()))
        .collect();
    assert_eq!(
        links,
        [("https://example.org/meta", "Meta Open Source"), ("/page", "/page")]
    );
}
