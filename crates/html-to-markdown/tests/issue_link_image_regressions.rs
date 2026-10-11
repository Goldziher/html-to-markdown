#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, InlineDataMedia, TierStrategy, convert};

fn content(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion succeeds")
        .content
        .unwrap_or_default()
}

fn paths() -> [ConversionOptions; 2] {
    let fast = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    [
        fast.clone(),
        ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..fast
        },
    ]
}

#[test]
fn should_preserve_empty_image_links_when_only_the_alt_is_requested() {
    for (name, label) in [
        ("", "/report.pdf"),
        (" aria-label=\"Report\"", "Report"),
        (" title=\"Report\"", "Report"),
    ] {
        let html =
            format!("<p>See <a href=\"/report.pdf\"{name}><img src=\"data:image/png;base64,AAAA\"></a> today.</p>");
        let options = ConversionOptions {
            inline_data_media: InlineDataMedia::AltTextOnly,
            ..ConversionOptions::default()
        };
        let expected = if name.contains("title") {
            format!("See [{label}](/report.pdf \"Report\") today.\n")
        } else {
            format!("See [{label}](/report.pdf) today.\n")
        };
        assert_eq!(content(&html, options), expected);
        let drop = ConversionOptions {
            inline_data_media: InlineDataMedia::DropElement,
            ..ConversionOptions::default()
        };
        assert_eq!(content(&html, drop), "See  today.\n");
    }
}

#[test]
fn should_name_icon_and_heading_image_links_on_both_paths() {
    for options in paths() {
        assert_eq!(
            content(
                "<p>Go to <a href=\"/cart\"><i class=\"icon-cart\"></i></a> now</p>",
                options.clone()
            ),
            "Go to [/cart](/cart) now\n"
        );
        assert_eq!(
            content("<h2>Head <a href=\"#x\"><img src=\"/i.png\"></a></h2>", options.clone()),
            "## Head [#x](#x)\n"
        );
        assert_eq!(
            content(
                "<p><a href=\"/cart\" aria-label=\"Cart\"><i></i></a></p>",
                options.clone()
            ),
            "[Cart](/cart)\n"
        );
        assert_eq!(
            content("<p><a href=\"/cart\" title=\"Cart\"><i></i></a></p>", options),
            "[Cart](/cart \"Cart\")\n"
        );
    }
}

#[test]
fn should_keep_invisible_span_link_labels_empty_without_adding_url_words() {
    for mut options in paths() {
        options.base_url = Some("http://example.test/wiki/Main".to_string());
        assert_eq!(
            content(
                "<p>before</p><a href=\"#\"><span class=\"icon\"></span></a><p>after</p>",
                options.clone()
            ),
            "before\n\n[](http://example.test/wiki/Main#)\n\nafter\n"
        );
        assert_eq!(
            content("<p><a href=\"/wiki/Help\"></a></p>", options.clone()),
            "[](http://example.test/wiki/Help)\n"
        );
        assert_eq!(
            content("<p><a href=\"#\" aria-label=\"Top\"><span></span></a></p>", options),
            "[Top](http://example.test/wiki/Main#)\n"
        );
    }
}

#[test]
fn should_write_empty_destinations_as_text_and_resolve_empty_href_against_base() {
    for options in paths() {
        let html = "<p><a href=\"\">empty</a> and <a>no href</a></p>";
        assert_eq!(content(html, options.clone()), "empty and no href\n");
        let based = ConversionOptions {
            base_url: Some("https://example.test/page".to_string()),
            ..options.clone()
        };
        assert_eq!(
            content(html, based.clone()),
            "[empty](https://example.test/page) and no href\n"
        );
        let images = "<p>a <img alt=\"logo\"> b <img src=\"\" alt=\"blank\"> c <img src=\"\" alt=\"\"> d</p>";
        assert_eq!(content(images, options), "a logo b blank c d\n");
        assert_eq!(content(images, based), "a logo b blank c d\n");
    }
}

#[test]
fn should_handle_deep_empty_span_links_without_recursive_fallback_walks() {
    let html = format!(
        "<p><a href=\"#\">{}{}</a></p>",
        "<span>".repeat(2000),
        "</span>".repeat(2000)
    );
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    };
    assert_eq!(content(&html, options), "[](#)\n");
}

#[test]
fn should_name_an_empty_icon_link_inside_a_table_cell_on_both_paths() {
    for mut options in paths() {
        options.compact_tables = true;
        assert_eq!(
            content(
                "<table><tr><th>File</th></tr><tr><td><a href=\"/report.pdf\"><i class=\"icon-pdf\"></i></a></td></tr></table>",
                options
            ),
            "| File |\n| --- |\n| [/report.pdf](/report.pdf) |\n"
        );
    }
}

#[test]
fn should_preserve_whitespace_before_an_absent_image_when_next_text_has_none() {
    for options in paths() {
        assert_eq!(content("<p>before <img src=\"\">after</p>", options), "before after\n");
    }
}
