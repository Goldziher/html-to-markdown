//! Coverage for `ConversionOptions::inline_data_media`, which chooses what the markdown shows for
//! an image, `<graphic>`, inline `<svg>`, `<video>`, `<audio>` or `<iframe>` whose address is an
//! inline `data:` URL.

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, InlineDataMedia, NodeContent, convert};

const PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
const MP4: &str = "data:video/mp4;base64,AAAAIGZ0eXBpc29t";
const MP3: &str = "data:audio/mpeg;base64,SUQzBAAAAAAA";
const HTML_PAGE: &str = "data:text/html,%3Cp%3Ehi%3C%2Fp%3E";

fn convert_with(html: &str, choice: InlineDataMedia) -> String {
    let options = ConversionOptions {
        inline_data_media: choice,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn all_choices(html: &str) -> [String; 3] {
    [
        convert_with(html, InlineDataMedia::Keep),
        convert_with(html, InlineDataMedia::AltTextOnly),
        convert_with(html, InlineDataMedia::DropElement),
    ]
}

#[test]
fn keep_is_the_default() {
    assert_eq!(ConversionOptions::default().inline_data_media, InlineDataMedia::Keep);
}

#[test]
fn an_image_prints_the_payload_the_alt_text_or_nothing() {
    let html = format!(r#"<p>Before <img src="{PNG}" alt="icon"> after</p>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert_eq!(keep, format!("Before ![icon]({PNG}) after\n"));
    assert_eq!(alt, "Before icon after\n");
    assert_eq!(drop, "Before  after\n");
}

#[test]
fn an_image_uses_a_real_lazy_address_before_a_data_one() {
    let html = format!(r#"<p><img src="{PNG}" data-src="{PNG}" srcset="https://example.com/real.png 2x" alt="p"></p>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert_eq!(keep, format!("![p]({PNG})\n"));
    assert_eq!(alt, "![p](https://example.com/real.png)\n");
    assert_eq!(drop, "![p](https://example.com/real.png)\n");
}

#[test]
fn an_image_with_only_data_addresses_gets_the_choice() {
    let html = format!(r#"<p><img src="{PNG}" data-src="{PNG}" alt="p"></p>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert_eq!(keep, format!("![p]({PNG})\n"));
    assert_eq!(alt, "p\n");
    assert_eq!(drop, "");
}

#[test]
fn an_image_with_a_real_src_is_unchanged() {
    let html = r#"<p><img src="https://example.com/a.png" srcset="data:image/gif;base64,R0lG 1x" alt="a"></p>"#;
    for output in all_choices(html) {
        assert_eq!(output, "![a](https://example.com/a.png)\n");
    }
}

#[test]
fn the_scheme_matches_in_any_case() {
    let html = r#"<p><img src="DATA:image/png;base64,AAAA" alt="icon"></p>"#;
    assert_eq!(convert_with(html, InlineDataMedia::AltTextOnly), "icon\n");
    let html = r#"<p><img src="DATA:image/png;base64,AAAA" data-src="https://example.com/r.png" alt="i"></p>"#;
    assert_eq!(
        convert_with(html, InlineDataMedia::AltTextOnly),
        "![i](https://example.com/r.png)\n"
    );
}

#[test]
fn an_image_whose_only_address_is_a_lazy_data_one_gets_the_choice() {
    for html in [
        r#"<p><img alt="a" data-src="data:image/png;base64,AAAA"></p>"#,
        r#"<p><img alt="a" src="" data-src="data:image/png;base64,AAAA"></p>"#,
        r#"<p><img alt="a" data-lazy-src="data:image/png;base64,AAAA"></p>"#,
    ] {
        let [keep, alt, drop] = all_choices(html);
        assert_eq!(keep, "![a](data:image/png;base64,AAAA)\n", "{html}");
        assert_eq!(alt, "a\n", "{html}");
        assert_eq!(drop, "", "{html}");
    }
}

#[test]
fn an_image_whose_only_srcset_candidates_are_data_gets_the_choice() {
    for (html, payload) in [
        (
            r#"<p><img alt="a" srcset="data:image/png;base64,AAAA 1x"></p>"#,
            "data:image/png;base64,AAAA",
        ),
        (
            r#"<p><img alt="a" src="data:image/gif;base64,R0lG" data-srcset="data:image/png;base64,AAAA 2x"></p>"#,
            "data:image/png;base64,AAAA",
        ),
        (
            r#"<p><img alt="a" srcset="DATA:image/png;base64,AAAA 1x"></p>"#,
            "DATA:image/png;base64,AAAA",
        ),
    ] {
        let [keep, alt, drop] = all_choices(html);
        assert_eq!(keep, format!("![a]({payload})\n"), "{html}");
        assert_eq!(alt, "a\n", "{html}");
        assert_eq!(drop, "", "{html}");
    }
}

#[test]
fn whitespace_before_the_data_scheme_does_not_hide_it() {
    for html in [
        r#"<p><img alt="a" src=" data:image/png;base64,AAAA"></p>"#,
        "<p><img alt=\"a\" src=\"\tdata:image/png;base64,AAAA\"></p>",
    ] {
        assert_eq!(convert_with(html, InlineDataMedia::AltTextOnly), "a\n", "{html}");
        assert_eq!(convert_with(html, InlineDataMedia::DropElement), "", "{html}");
    }
}

#[test]
fn a_data_src_in_any_case_gives_way_to_a_real_lazy_address_under_every_choice() {
    let lower = r#"<p><img src="data:image/png;base64,AAAA" data-src="https://example.com/r.png" alt="i"></p>"#;
    for scheme in ["DATA:", "Data:", "dAtA:"] {
        let html = lower.replace("data:image", &format!("{scheme}image"));
        for (output, choice) in all_choices(&html).into_iter().zip(["keep", "alt", "drop"]) {
            assert_eq!(output, "![i](https://example.com/r.png)\n", "{scheme} {choice}");
        }
    }
    assert_eq!(all_choices(lower)[0], "![i](https://example.com/r.png)\n");
}

#[test]
fn a_data_src_in_any_case_gives_way_to_a_real_lazy_address_on_the_fast_path_too() {
    // ~keep Metadata off and no `<mark>` styling make the document eligible for the Tier-1
    // ~keep scanner, which must hand a lazy-loaded image to Tier 2 whatever the scheme's case.
    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    for scheme in ["data:", "DATA:", "Data:"] {
        let html =
            format!(r#"<p><img src="{scheme}image/png;base64,AAAA" data-src="https://example.com/r.png" alt="i"></p>"#);
        let output = convert(&html, Some(options.clone()))
            .unwrap()
            .content
            .unwrap_or_default();
        assert_eq!(output, "![i](https://example.com/r.png)\n", "{scheme}");
    }
}

#[test]
fn a_picture_uses_the_address_of_its_first_source_with_a_real_one() {
    let one = format!(
        r#"<p><picture><source srcset="https://example.com/real.webp"><img src="{PNG}" alt="a"></picture></p>"#
    );
    let [keep, alt, drop] = all_choices(&one);
    assert_eq!(keep, format!("![a]({PNG})\n"));
    assert_eq!(alt, "![a](https://example.com/real.webp)\n");
    assert_eq!(drop, "![a](https://example.com/real.webp)\n");

    let several = format!(
        r#"<p><picture><source srcset="data:image/webp;base64,UklG 1x"><source srcset="https://example.com/1.webp 1x, https://example.com/2.webp 2x"><source srcset="https://example.com/3.webp"><img src="{PNG}" alt="a"></picture></p>"#
    );
    let [keep, alt, drop] = all_choices(&several);
    assert_eq!(keep, format!("![a]({PNG})\n"));
    assert_eq!(alt, "![a](https://example.com/2.webp)\n");
    assert_eq!(drop, "![a](https://example.com/2.webp)\n");
}

#[test]
fn a_picture_with_no_usable_source_gets_the_choice() {
    for html in [
        format!(
            r#"<p><picture><source srcset="data:image/webp;base64,UklG 1x"><img src="{PNG}" alt="a"></picture></p>"#
        ),
        format!(r#"<p><picture><source media="(min-width: 1px)"><img src="{PNG}" alt="a"></picture></p>"#),
        format!(
            r#"<p><picture><img src="{PNG}" alt="a"><source srcset="https://example.com/after.webp"></picture></p>"#
        ),
    ] {
        let [keep, alt, drop] = all_choices(&html);
        assert_eq!(keep, format!("![a]({PNG})\n"), "{html}");
        assert_eq!(alt, "a\n", "{html}");
        assert_eq!(drop, "", "{html}");
    }
}

#[test]
fn a_picture_source_does_not_replace_a_real_image_address() {
    let html = r#"<p><picture><source srcset="https://example.com/s.webp"><img src="https://example.com/i.png" alt="a"></picture></p>"#;
    for output in all_choices(html) {
        assert_eq!(output, "![a](https://example.com/i.png)\n");
    }
}

fn structure_images(html: &str, choice: InlineDataMedia) -> Vec<(Option<String>, Option<String>)> {
    let options = ConversionOptions {
        inline_data_media: choice,
        include_document_structure: true,
        ..ConversionOptions::default()
    };
    let document = convert(html, Some(options))
        .unwrap()
        .document
        .expect("document structure");
    document
        .nodes
        .into_iter()
        .filter_map(|node| match node.content {
            NodeContent::Image { src, description, .. } => Some((src, description)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_document_structure_shows_the_image_the_markdown_shows() {
    let html = format!(r#"<p>x <img src="{PNG}" alt="icon"> y</p>"#);
    let icon = Some("icon".to_string());
    assert_eq!(
        structure_images(&html, InlineDataMedia::Keep),
        [(Some(PNG.to_string()), icon.clone())]
    );
    assert_eq!(structure_images(&html, InlineDataMedia::AltTextOnly), [(None, icon)]);
    assert_eq!(structure_images(&html, InlineDataMedia::DropElement), []);

    let real = r#"<p><img src="https://example.com/a.png" alt="a"></p>"#;
    for choice in [InlineDataMedia::AltTextOnly, InlineDataMedia::DropElement] {
        assert_eq!(
            structure_images(real, choice),
            [(Some("https://example.com/a.png".to_string()), Some("a".to_string()))]
        );
    }
}

#[test]
fn a_link_around_a_replaced_image_keeps_its_alt_text_or_goes_with_it() {
    let html = format!(r#"<p><a href="https://example.com/x.html"><img src="{PNG}" alt="a"></a></p>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert_eq!(keep, format!("[![a]({PNG})](https://example.com/x.html)\n"));
    assert_eq!(alt, "[a](https://example.com/x.html)\n");
    assert_eq!(drop, "");

    let wrapped =
        format!(r#"<p><a href="https://example.com/x.html"> <span><img src="{PNG}" alt="a"></span> </a></p>"#);
    assert_eq!(
        convert_with(&wrapped, InlineDataMedia::AltTextOnly),
        "[a](https://example.com/x.html)\n"
    );
    assert_eq!(convert_with(&wrapped, InlineDataMedia::DropElement), "");

    let no_alt = format!(r#"<p><a href="https://example.com/x.html"><img src="{PNG}"></a></p>"#);
    assert_eq!(
        convert_with(&no_alt, InlineDataMedia::AltTextOnly),
        "[https://example.com/x.html](https://example.com/x.html)\n"
    );
}

#[test]
fn a_link_around_any_dropped_media_element_goes_with_it() {
    for media in [
        format!(r#"<graphic src="{PNG}"></graphic>"#),
        format!(r#"<video src="{MP4}"></video>"#),
        format!(r#"<audio src="{MP3}"></audio>"#),
        format!(r#"<iframe src="{HTML_PAGE}"></iframe>"#),
        "<svg><rect/></svg>".to_string(),
    ] {
        let html = format!(r#"<p><a href="https://example.com/x.html">{media}</a></p>"#);
        assert_eq!(convert_with(&html, InlineDataMedia::DropElement), "", "{media}");
    }
}

#[test]
fn a_link_keeps_its_other_content_and_an_unrelated_empty_link_is_unchanged() {
    let html = format!(r#"<p><a href="https://example.com/x.html">see <img src="{PNG}" alt="a"></a></p>"#);
    assert_eq!(
        convert_with(&html, InlineDataMedia::DropElement),
        "[see](https://example.com/x.html)\n"
    );

    let empty = format!(r#"<p><img src="{PNG}" alt="a"> <a href="https://example.com/x.html"><span></span></a></p>"#);
    let [_, _, drop] = all_choices(&empty);
    assert_eq!(
        drop,
        convert_with(
            r#"<p><a href="https://example.com/x.html"><span></span></a></p>"#,
            InlineDataMedia::Keep
        )
    );
}

#[test]
fn a_graphic_prints_the_payload_the_alt_text_or_nothing() {
    let html = format!(r#"<p><graphic url="{PNG}" alt="g" /></p>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert_eq!(keep, format!("![g]({PNG})\n"));
    assert_eq!(alt, "g\n");
    assert_eq!(drop, "");
}

#[test]
fn a_graphic_uses_its_next_address_attribute_that_is_not_data() {
    let html = format!(r#"<p><graphic url="{PNG}" src="https://example.com/g.png" alt="g" /></p>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert_eq!(keep, format!("![g]({PNG})\n"));
    assert_eq!(alt, "![g](https://example.com/g.png)\n");
    assert_eq!(drop, "![g](https://example.com/g.png)\n");
}

#[test]
fn a_video_drops_the_link_and_keeps_or_drops_its_fallback_text() {
    let html = format!(r#"<video src="{MP4}">Your browser cannot play this.</video>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert!(keep.contains(MP4), "Keep writes the payload: {keep:?}");
    assert_eq!(alt, "Your browser cannot play this.\n");
    assert_eq!(drop, "");
}

#[test]
fn a_video_falls_back_to_a_nested_source_that_is_not_data() {
    let real = "https://example.com/v.mp4";
    let html = format!(r#"<video src="{MP4}"><source src="{MP4}"><source src="{real}"></video>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert!(keep.contains(MP4), "Keep writes the payload: {keep:?}");
    assert_eq!(alt, format!("[{real}]({real})\n"));
    assert_eq!(drop, format!("[{real}]({real})\n"));
}

#[test]
fn an_audio_element_gets_the_choice() {
    let html = format!(r#"<audio src="{MP3}">No audio.</audio>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert!(keep.contains(MP3), "Keep writes the payload: {keep:?}");
    assert_eq!(alt, "No audio.\n");
    assert_eq!(drop, "");
}

#[test]
fn an_iframe_loses_its_link() {
    let html = format!(r#"<iframe src="{HTML_PAGE}"></iframe>"#);
    let [keep, alt, drop] = all_choices(&html);
    assert!(keep.contains(HTML_PAGE), "Keep writes the payload: {keep:?}");
    assert_eq!(alt, "");
    assert_eq!(drop, "");
}

#[test]
fn an_inline_svg_prints_the_payload_the_title_or_nothing() {
    let html = r#"<p><svg width="1" height="1"><title>Logo</title><rect width="1" height="1"/></svg></p>"#;
    let [keep, alt, drop] = all_choices(html);
    assert!(keep.starts_with("![Logo](data:image/svg+xml;base64,"), "{keep:?}");
    assert_eq!(alt, "Logo\n");
    assert_eq!(drop, "");
}

#[test]
fn a_link_to_a_data_url_keeps_the_payload_or_just_its_text() {
    let html = r#"<p><a href="data:text/plain,hello">file</a></p>"#;
    let [keep, alt, drop] = all_choices(html);
    assert_eq!(keep, "[file](data:text/plain,hello)\n");
    assert_eq!(alt, "file\n");
    assert_eq!(drop, "file\n");
}

#[test]
fn a_link_to_a_data_url_wrapping_an_image_keeps_the_image_and_drops_the_link_address() {
    let html = r#"<p><a href="data:text/plain,hello"><img src="pic.png" alt="pic"></a></p>"#;
    let [keep, alt, drop] = all_choices(html);
    assert_eq!(keep, "[![pic](pic.png)](data:text/plain,hello)\n");
    assert_eq!(alt, "![pic](pic.png)\n");
    assert_eq!(drop, "![pic](pic.png)\n");
}

#[test]
fn a_data_link_whose_text_equals_its_own_address_does_not_autolink() {
    // ~keep The GFM autolink form (`<href>`) writes `href` as the link's own visible text, which
    // ~keep would put the payload right back into the output a dropped address asked to remove.
    let html = r#"<p><a href="data:text/plain,hi">data:text/plain,hi</a></p>"#;
    let [keep, alt, drop] = all_choices(html);
    assert_eq!(keep, "<data:text/plain,hi>\n");
    assert_eq!(alt, "data:text/plain,hi\n");
    assert_eq!(drop, "data:text/plain,hi\n");
}

#[test]
fn a_data_link_wrapping_a_heading_drops_the_address_not_the_heading_text() {
    let html = r#"<a href="data:text/plain,hello"><h2>Title</h2></a>"#;
    let [keep, alt, drop] = all_choices(html);
    assert_eq!(keep, "## [Title](data:text/plain,hello)\n");
    assert_eq!(alt, "Title\n");
    assert_eq!(drop, "Title\n");
}

#[test]
fn a_data_link_with_no_text_content_writes_nothing_once_the_address_is_dropped() {
    // ~keep Empty span wrappers add no URL label (#771). Once the data address is removed,
    // ~keep the payload must not reappear as visible text.
    let html = r#"<p><a href="data:text/plain,hello"><span></span></a></p>"#;
    let [keep, alt, drop] = all_choices(html);
    assert_eq!(keep, "[](data:text/plain,hello)\n");
    assert_eq!(alt, "");
    assert_eq!(drop, "");
}

#[test]
fn a_normal_link_is_unaffected_by_the_data_link_choice() {
    let html = r#"<p><a href="https://example.com/page">text</a></p>"#;
    for output in all_choices(html) {
        assert_eq!(output, "[text](https://example.com/page)\n");
    }
}

#[test]
fn a_link_to_an_upper_case_data_url_is_treated_the_same() {
    let html = r#"<p><a href="DATA:text/plain,hello">file</a></p>"#;
    let [keep, alt, drop] = all_choices(html);
    assert_eq!(keep, "[file](DATA:text/plain,hello)\n");
    assert_eq!(alt, "file\n");
    assert_eq!(drop, "file\n");
}

#[test]
fn the_choice_holds_on_the_fast_path_too() {
    // ~keep With metadata extraction off and no `<mark>` styling, a plain document is eligible
    // ~keep for the Tier-1 scanner, which does not implement the choice; the router must send it
    // ~keep to Tier 2.
    let html = format!(r#"<p>Before <img src="{PNG}" alt="icon"> after</p>"#);
    let options = ConversionOptions {
        inline_data_media: InlineDataMedia::AltTextOnly,
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    let output = convert(&html, Some(options)).unwrap().content.unwrap_or_default();
    assert_eq!(output, "Before icon after\n");

    let svg = "<p><svg><title>Logo</title><rect/></svg></p>";
    let options = ConversionOptions {
        inline_data_media: InlineDataMedia::DropElement,
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    let output = convert(svg, Some(options)).unwrap().content.unwrap_or_default();
    assert_eq!(output, "");
}

#[cfg(feature = "inline-images")]
#[test]
fn extracted_images_do_not_depend_on_the_choice() {
    let html = format!(r#"<p><img src="{PNG}" alt="icon"></p>"#);
    let options = ConversionOptions {
        inline_data_media: InlineDataMedia::DropElement,
        extract_images: true,
        ..ConversionOptions::default()
    };
    let result = convert(&html, Some(options)).unwrap();
    assert_eq!(result.content.unwrap_or_default(), "");
    assert_eq!(result.images.len(), 1);
}

#[cfg(feature = "inline-images")]
#[test]
fn an_image_is_extracted_whatever_the_case_of_its_data_scheme() {
    for scheme in ["data:", "DATA:", "Data:"] {
        let html = format!(r#"<p><img src="{scheme}{}" alt="icon"></p>"#, &PNG[5..]);
        let options = ConversionOptions {
            extract_images: true,
            ..ConversionOptions::default()
        };
        let result = convert(&html, Some(options)).unwrap();
        assert_eq!(result.images.len(), 1, "{scheme}");
    }
}

#[cfg(feature = "metadata")]
#[test]
fn the_metadata_reports_a_data_image_whatever_the_case_of_its_scheme() {
    for scheme in ["data:", "DATA:", "Data:"] {
        let html = format!(r#"<p><img src="{scheme}{}" alt="icon"></p>"#, &PNG[5..]);
        let options = ConversionOptions {
            extract_metadata: true,
            ..ConversionOptions::default()
        };
        let metadata = convert(&html, Some(options)).unwrap().metadata;
        assert_eq!(metadata.images.len(), 1, "{scheme}");
        assert_eq!(
            metadata.images[0].image_type,
            html_to_markdown_rs::ImageType::DataUri,
            "{scheme}"
        );
    }
}

#[test]
fn parse_accepts_any_spelling_and_falls_back_to_keep() {
    assert_eq!(InlineDataMedia::parse("alt_text_only"), InlineDataMedia::AltTextOnly);
    assert_eq!(InlineDataMedia::parse("AltTextOnly"), InlineDataMedia::AltTextOnly);
    assert_eq!(InlineDataMedia::parse("drop-element"), InlineDataMedia::DropElement);
    assert_eq!(InlineDataMedia::parse("keep"), InlineDataMedia::Keep);
    assert_eq!(InlineDataMedia::parse("bogus"), InlineDataMedia::Keep);
}

#[cfg(feature = "serde")]
#[test]
fn the_option_round_trips_through_serde() {
    let options: ConversionOptions = serde_json::from_str(r#"{"inline_data_media":"drop_element"}"#).unwrap();
    assert_eq!(options.inline_data_media, InlineDataMedia::DropElement);
    let json = serde_json::to_value(InlineDataMedia::AltTextOnly).unwrap();
    assert_eq!(
        serde_json::from_value::<InlineDataMedia>(json).unwrap(),
        InlineDataMedia::AltTextOnly
    );
}
