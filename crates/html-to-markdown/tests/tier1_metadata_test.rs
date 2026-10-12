//! Integration tests for Tier-1 metadata extraction (M5).
//!
//! Each test verifies that Tier-1 with `Tier1` and `extract_metadata: true`
//! produces output byte-identical to Tier-2 (`Tier2`, same options).
//!
//! These tests cover YAML frontmatter produced for:
//!   - `<title>` (including HTML entities and multi-word titles)
//!   - `<meta name="...">` (description, keywords, author, viewport)
//!   - `<meta property="og:...">` (Open Graph tags: title, description, image)
//!   - `<meta property="twitter:...">` (Twitter Card tags)
//!   - `<link rel="canonical">`, `<link rel="author">`, `<link rel="license">`
//!   - `<base href="...">`
//!   - Documents without a `<head>` element
//!   - Documents with an empty `<head>` element
//!   - Multiple concurrent meta tags
//!   - No-metadata case (option disabled)

#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

/// Convert with `Tier1` + `extract_metadata: true`.
fn t1(html: &str) -> String {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    convert(html, Some(opts))
        .expect("tier-1 conversion must succeed")
        .content
        .unwrap_or_default()
}

/// Convert with `Tier2` + `extract_metadata: true` (same other defaults).
fn t2(html: &str) -> String {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    convert(html, Some(opts))
        .expect("tier-2 conversion must succeed")
        .content
        .unwrap_or_default()
}

#[test]
fn tier1_extracts_title() {
    let html = "<html><head><title>Hello</title></head><body><p>body</p></body></html>";
    assert_eq!(t1(html), t2(html), "title metadata must be byte-identical");
}

#[test]
fn tier1_extracts_title_with_spaces() {
    let html = "<html><head><title>My Page Title Here</title></head><body><p>x</p></body></html>";
    assert_eq!(t1(html), t2(html), "multi-word title metadata must be byte-identical");
}

#[test]
fn tier1_extracts_title_with_leading_trailing_whitespace() {
    let html = "<html><head><title>  Padded Title  </title></head><body><p>x</p></body></html>";
    assert_eq!(
        t1(html),
        t2(html),
        "title with whitespace padding must be byte-identical"
    );
}

#[test]
fn tier1_extracts_meta_description() {
    let html = r#"<html><head><meta name="description" content="a desc"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "meta description must be byte-identical");
}

#[test]
fn tier1_extracts_meta_keywords() {
    let html =
        r#"<html><head><meta name="keywords" content="rust, markdown, html"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "meta keywords must be byte-identical");
}

#[test]
fn tier1_extracts_meta_author() {
    let html = r#"<html><head><meta name="author" content="Jane Doe"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "meta author must be byte-identical");
}

#[test]
fn tier1_extracts_og_image() {
    let html = r#"<html><head><meta property="og:image" content="foo.png"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "og:image must be byte-identical");
}

#[test]
fn tier1_extracts_og_title() {
    let html =
        r#"<html><head><meta property="og:title" content="Open Graph Title"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "og:title must be byte-identical");
}

#[test]
fn tier1_extracts_og_description() {
    let html =
        r#"<html><head><meta property="og:description" content="OG Description"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "og:description must be byte-identical");
}

#[test]
fn tier1_extracts_canonical_url() {
    let html =
        r#"<html><head><link rel="canonical" href="https://example.com/page"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "canonical link must be byte-identical");
}

#[test]
fn tier1_extracts_link_author() {
    let html =
        r#"<html><head><link rel="author" href="https://example.com/author"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "link-author must be byte-identical");
}

#[test]
fn tier1_extracts_base_href() {
    let html = r#"<html><head><base href="https://example.com/"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "base-href must be byte-identical");
}

#[test]
fn tier1_extracts_multiple_meta_tags() {
    let html = r#"<html><head>
        <title>My Site</title>
        <meta name="description" content="site description">
        <meta property="og:image" content="image.png">
        <link rel="canonical" href="https://example.com/">
    </head><body><p>content</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "multiple meta tags must be byte-identical");
}

#[test]
fn tier1_no_head_element_produces_no_frontmatter() {
    let html = "<body><p>just a paragraph</p></body>";
    assert_eq!(t1(html), t2(html), "no-head document must be byte-identical");
}

#[test]
fn tier1_empty_head_produces_no_frontmatter() {
    let html = "<html><head></head><body><p>paragraph</p></body></html>";
    assert_eq!(
        t1(html),
        t2(html),
        "empty head must produce no frontmatter (byte-identical)"
    );
}

#[test]
fn tier1_no_frontmatter_when_extract_metadata_false() {
    let html = "<html><head><title>Hello</title></head><body><p>body</p></body></html>";
    let opts_t1 = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    let opts_t2 = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    let out_t1 = convert(html, Some(opts_t1)).unwrap().content.unwrap_or_default();
    let out_t2 = convert(html, Some(opts_t2)).unwrap().content.unwrap_or_default();
    assert_eq!(out_t1, out_t2, "disabled metadata must be byte-identical");
    assert!(
        !out_t1.starts_with("---\n"),
        "no YAML frontmatter expected when extract_metadata=false"
    );
}

#[test]
fn tier1_yaml_quotes_colon_in_value() {
    let html = r#"<html><head><meta name="description" content="Key: Value"></head><body><p>x</p></body></html>"#;
    assert_eq!(t1(html), t2(html), "YAML-quoted colon value must be byte-identical");
}

// ~keep ── 10. Auto-routing respects extract_metadata now (no longer forces Tier-2) ──

#[test]
fn auto_routing_with_extract_metadata_can_use_tier1() {
    // ~keep With no other Tier-2 signals, Auto should allow the classifier to pick
    // ~keep Tier-1 when extract_metadata=true (M5 removes that guard).  The output
    // ~keep must still match Tier-2.
    let html = "<html><head><title>AutoTest</title></head><body><p>content</p></body></html>";
    let opts_auto = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    let opts_t2 = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    let auto_out = convert(html, Some(opts_auto)).unwrap().content.unwrap_or_default();
    let t2_out = convert(html, Some(opts_t2)).unwrap().content.unwrap_or_default();
    assert_eq!(auto_out, t2_out, "Auto with extract_metadata must match Tier-2");
    assert!(
        auto_out.starts_with("---\n"),
        "Auto output must include YAML frontmatter"
    );
}

// ~keep ── 11. The document's first `<base href>` and first canonical link ──

#[test]
fn should_report_the_first_base_href_on_both_tiers() {
    let html = r#"<html><head><base href="/first/"><base href="/second/"></head><body><p>x</p></body></html>"#;
    let out = t2(html);
    assert!(out.contains("base: /first/\n"), "{out}");
    assert_eq!(t1(html), out);
}

#[test]
fn should_report_the_base_href_the_parser_keeps_on_both_tiers() {
    let html = r#"<html><head><title><base href="https://evil.example/"></title><base href="/real/"></head><body><p>x</p></body></html>"#;
    let out = t2(html);
    assert!(out.contains("base: /real/\n"), "{out}");
    assert_eq!(t1(html), out);
}

#[test]
fn should_report_a_base_href_from_the_body_with_an_empty_head_on_both_tiers() {
    let html = r#"<html><head></head><body><base href="/in-body/"><p>x</p></body></html>"#;
    let out = t2(html);
    assert!(out.contains("base: /in-body/\n"), "{out}");
    assert_eq!(t1(html), out);
}

#[test]
fn should_report_the_first_base_href_on_the_auto_path() {
    // ~keep Without the `metadata` feature, these options let the router pick Tier 1.
    let html = r#"<html><head><base href="/first/"><base href="/second/"></head><body><p>x</p></body></html>"#;
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        extract_metadata: true,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    let out = convert(html, Some(opts)).unwrap().content.unwrap_or_default();
    assert!(out.contains("base: /first/\n"), "{out}");
}

#[test]
fn should_report_the_first_canonical_link_on_both_tiers() {
    let html = r#"<html><head><link rel="canonical" href="https://example.com/first"><link rel="canonical" href="https://example.com/second"></head><body><p>x</p></body></html>"#;
    let out = t2(html);
    assert!(out.contains("canonical: https://example.com/first\n"), "{out}");
    assert_eq!(t1(html), out);
}

#[test]
fn should_report_the_first_base_href_from_the_tier1_entry_point() {
    let html = r#"<html><head><base href="/first/"><base href="/second/"></head><body><p>x</p></body></html>"#;
    let opts = ConversionOptions {
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    let out = html_to_markdown_rs::tier1::run(html, &html_to_markdown_rs::prescan::PrescanReport::default(), &opts)
        .expect("tier-1 conversion must succeed");
    assert!(out.contains("base: /first/\n"), "{out}");
}

#[test]
fn should_report_the_first_meta_tag_per_name_on_both_tiers() {
    let html = r#"<html><head><meta name="description" content="first"><meta name="description" content="second"><meta property="og:title" content="first"><meta property="og:title" content="second"></head><body><p>x</p></body></html>"#;
    let out = t2(html);
    assert!(out.contains("meta-description: first\n"), "{out}");
    assert!(out.contains("meta-og:title: first\n"), "{out}");
    assert_eq!(t1(html), out);
}

#[test]
fn should_report_the_head_metadata_after_a_leading_doctype_and_comment_on_both_tiers() {
    let html = r#"<!DOCTYPE html><!-- note --><html><head><title>T</title><base href="/b/"></head><body><p>x</p></body></html>"#;
    let out = t2(html);
    assert!(out.contains("title: T\n") && out.contains("base: /b/\n"), "{out}");
    assert_eq!(t1(html), out);
}

#[test]
fn should_report_the_base_href_of_a_document_without_a_head_tag_on_both_tiers() {
    // ~keep The parser creates the head itself, and the `<base>` goes in it.
    let html = r#"<base href="https://a.example/"><p>x</p>"#;
    let out = t2(html);
    assert!(out.contains("base: https://a.example/\n"), "{out}");
    assert_eq!(t1(html), out);
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_the_first_base_href_and_canonical_link_in_the_document_metadata() {
    let html = r#"<html><head><base href="/first/"><base href="/second/"><link rel="canonical" href="https://example.com/first"><link rel="canonical" href="https://example.com/second"></head><body><p>x</p></body></html>"#;
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    let document = convert(html, Some(opts)).unwrap().metadata.document;
    assert_eq!(document.base_href.as_deref(), Some("/first/"));
    assert_eq!(document.canonical_url.as_deref(), Some("https://example.com/first"));
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_the_base_href_of_a_document_without_a_head_tag_in_the_document_metadata() {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    let result = convert(r#"<base href="https://a.example/"><p>x</p>"#, Some(opts)).unwrap();
    assert_eq!(
        result.metadata.document.base_href.as_deref(),
        Some("https://a.example/")
    );
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_the_first_meta_tag_per_name_in_the_document_metadata() {
    let html = r#"<html><head><meta name="description" content="first"><meta name="description" content="second"></head><body><p>x</p></body></html>"#;
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    let document = convert(html, Some(opts)).unwrap().metadata.document;
    assert_eq!(document.description.as_deref(), Some("first"));
}

// ~keep ── 12. First title, stray head, meta name case, normalized input ──

/// Convert with the Tier-1 scanner alone, which fails instead of falling back to Tier 2.
fn t1_only(html: &str) -> String {
    let opts = ConversionOptions {
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    html_to_markdown_rs::tier1::run(html, &html_to_markdown_rs::prescan::PrescanReport::default(), &opts)
        .expect("tier-1 conversion must succeed")
}

/// Convert with `base_url` set on the given tier.
fn with_base_url(html: &str, tier_strategy: TierStrategy) -> String {
    let opts = ConversionOptions {
        tier_strategy,
        extract_metadata: true,
        base_url: Some("https://example.com/dir/page".to_string()),
        ..ConversionOptions::default()
    };
    convert(html, Some(opts)).unwrap().content.unwrap_or_default()
}

/// `html` as UTF-16 bytes after a byte order mark, read the way a caller with raw bytes reads
/// them: through `String::from_utf8_lossy`.
fn utf16_with_bom(html: &str, little_endian: bool) -> String {
    let mut bytes = if little_endian {
        vec![0xFF, 0xFE]
    } else {
        vec![0xFE, 0xFF]
    };
    for unit in html.encode_utf16() {
        bytes.extend_from_slice(&if little_endian {
            unit.to_le_bytes()
        } else {
            unit.to_be_bytes()
        });
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[test]
fn should_report_the_first_title_on_both_tiers() {
    let html = "<html><head><title>First</title><title>Second</title></head><body><p>x</p></body></html>";
    let out = t2(html);
    assert!(out.contains("title: First\n"), "{out}");
    assert_eq!(t1_only(html), out);
}

#[test]
fn should_use_a_title_before_an_explicit_head_on_both_tiers() {
    let html = "<title>First</title><head><title>Second</title></head><p>x</p>";
    let expected = "---\ntitle: First\n---\n\nx\n";
    assert_eq!(t1(html), expected);
    assert_eq!(t2(html), expected);

    #[cfg(feature = "metadata")]
    {
        let options = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            extract_metadata: true,
            ..ConversionOptions::default()
        };
        let document = convert(html, Some(options)).unwrap().metadata.document;
        assert_eq!(document.title.as_deref(), Some("First"));
    }
}

#[test]
fn should_ignore_a_head_inside_the_body_on_both_tiers() {
    for html in [
        "<html><head></head><body><head><title>Stray</title></head><p>x</p></body></html>",
        "<html><body><head><title>Stray</title></head><p>x</p></body></html>",
    ] {
        let out = t2(html);
        assert!(!out.contains("Stray"), "{out}");
        assert_eq!(t1_only(html), out, "{html}");
    }
    // ~keep A second `<head>` tag in front of the body opens no element, and the HTML tree
    // ~keep builder puts the title after it into the head: it is the title of the document.
    let html = "<html><head></head><head><title>Second</title></head><body><p>x</p></body></html>";
    assert_eq!(t2(html), "---\ntitle: Second\n---\n\nx\n");
    assert_eq!(t1_only(html), t2(html));
}

#[test]
fn should_report_the_first_meta_tag_for_names_that_differ_in_case_on_both_tiers() {
    let html = r#"<html><head><meta name="Description" content="first"><meta name="description" content="second"><meta property="og:Title" content="first"><meta property="og:title" content="second"></head><body><p>x</p></body></html>"#;
    let out = t2(html);
    assert!(
        out.contains("meta-Description: first\n") && out.contains("meta-og:Title: first\n") && !out.contains("second"),
        "{out}"
    );
    assert_eq!(t1_only(html), out);
}

#[test]
fn should_read_the_base_href_of_utf16_input_on_both_tiers() {
    let html = r#"<html><head><base href="/b/"><title>T</title></head><body><p><a href="rel">l</a></p></body></html>"#;
    for little_endian in [true, false] {
        let input = utf16_with_bom(html, little_endian);
        for tier in [TierStrategy::Tier2, TierStrategy::Tier1] {
            let out = with_base_url(&input, tier);
            assert!(
                out.contains("base: https://example.com/b/\n"),
                "{little_endian} {tier:?}: {out}"
            );
            assert!(
                out.contains("(https://example.com/b/rel)"),
                "{little_endian} {tier:?}: {out}"
            );
        }
    }
}

#[test]
fn should_read_the_base_href_of_a_tag_name_holding_a_nul_byte_on_both_tiers() {
    let html = "<html><head><ba\0se href=\"/n/\"></head><body><p><a href=\"rel\">l</a></p></body></html>";
    for tier in [TierStrategy::Tier2, TierStrategy::Tier1] {
        let out = with_base_url(html, tier);
        assert!(out.contains("base: https://example.com/n/\n"), "{tier:?}: {out}");
        assert!(out.contains("(https://example.com/n/rel)"), "{tier:?}: {out}");
    }
}

#[test]
fn should_strip_a_nul_byte_on_the_auto_path() {
    // ~keep Without the `metadata` feature, these options let the router pick Tier 1.
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    let out = convert("<p>a\0b</p>", Some(opts)).unwrap().content.unwrap_or_default();
    assert_eq!(out, "ab\n");
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_the_first_title_and_the_first_meta_tag_in_any_case_in_the_document_metadata() {
    let html = r#"<html><head><title>First</title><title>Second</title><meta name="Description" content="first"><meta name="description" content="second"></head><body><p>x</p></body></html>"#;
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    let document = convert(html, Some(opts)).unwrap().metadata.document;
    assert_eq!(document.title.as_deref(), Some("First"));
    assert_eq!(document.description.as_deref(), Some("first"));
}

// ~keep ── 13. Empty first title, implicit body start, element-only base and canonical ──

#[test]
fn should_keep_an_empty_first_title_on_both_tiers() {
    for html in [
        "<html><head><title></title><title>Second</title></head><body><p>x</p></body></html>",
        r#"<html><head><title> </title><title>Second</title><meta name="description" content="d"></head><body><p>x</p></body></html>"#,
    ] {
        let out = t2(html);
        assert!(!out.contains("title:"), "{out}");
        assert_eq!(t1_only(html), out, "{html}");
    }
}

#[test]
fn should_ignore_a_head_after_implicit_body_content_on_both_tiers() {
    for html in [
        "<p>x</p><head><title>Stray</title></head><p>y</p>",
        "x<head><title>Stray</title></head><p>y</p>",
        "< <head><title>Stray</title></head><p>y</p>",
        "<html><div>x</div><head><title>Stray</title></head><p>y</p></html>",
    ] {
        let out = t2(html);
        assert!(!out.contains("Stray"), "{out}");
        assert_eq!(t1_only(html), out, "{html}");
    }
}

#[test]
fn should_read_a_head_after_head_content_and_whitespace_on_both_tiers() {
    let html = "<!DOCTYPE html>\n<meta charset=\"utf-8\">\n<head><title>Kept</title></head><p>x</p>";
    let out = t2(html);
    assert!(out.contains("title: Kept\n"), "{out}");
    assert_eq!(t1_only(html), out);
}

#[cfg(feature = "metadata")]
#[test]
fn should_take_the_base_href_and_canonical_link_from_their_elements_and_a_meta_title_only_without_a_title_element() {
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: true,
        base_url: Some("https://example.com/dir/page".to_string()),
        ..ConversionOptions::default()
    };
    let html = r#"<html><head><title>Real</title><meta name="title" content="Meta"><base href="/real/"><meta name="base" content="/meta/"><link rel="canonical" href="https://example.com/real"><meta name="canonical" content="https://example.com/meta"></head><body><p><a href="x">x</a></p></body></html>"#;
    let result = convert(html, Some(opts.clone())).unwrap();
    assert!(
        result
            .content
            .unwrap_or_default()
            .contains("(https://example.com/real/x)")
    );
    let document = result.metadata.document;
    assert_eq!(document.base_href.as_deref(), Some("https://example.com/real/"));
    assert_eq!(document.canonical_url.as_deref(), Some("https://example.com/real"));
    assert_eq!(document.title.as_deref(), Some("Real"));
    assert_eq!(document.meta_tags.get("base").map(String::as_str), Some("/meta/"));
    assert_eq!(document.meta_tags.get("title").map(String::as_str), Some("Meta"));

    let html = r#"<html><head><meta name="title" content="Meta"><meta name="base" content="/meta/"><meta name="canonical" content="https://example.com/meta"></head><body><p>x</p></body></html>"#;
    let document = convert(html, Some(opts)).unwrap().metadata.document;
    assert_eq!(document.title.as_deref(), Some("Meta"));
    assert!(!document.meta_tags.contains_key("title"));
    assert_eq!(document.base_href, None);
    assert_eq!(document.canonical_url, None);
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_an_empty_title_element_blocking_a_meta_title_fallback() {
    // ~keep An empty `<title></title>` is still a title element: it blocks the meta fallback,
    // ~keep and the page reports no title, whichever order the two tags come in (#527).
    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    for html in [
        r#"<html><head><title></title><meta name="title" content="Meta"></head><body><p>x</p></body></html>"#,
        r#"<html><head><meta name="title" content="Meta"><title></title></head><body><p>x</p></body></html>"#,
    ] {
        let document = convert(html, Some(opts.clone())).unwrap().metadata.document;
        assert_eq!(document.title, None, "{html}");
        assert_eq!(
            document.meta_tags.get("title").map(String::as_str),
            Some("Meta"),
            "{html}"
        );
        assert_eq!(t1_only(html), t2(html), "{html}");
        assert!(!t2(html).lines().any(|line| line.starts_with("title:")), "{}", t2(html));
    }
}

#[cfg(feature = "metadata")]
#[test]
fn should_give_the_document_structure_the_metadata_block_of_the_head_the_metadata_reads() {
    use html_to_markdown_rs::types::{NodeContent, build_document_structure};

    let opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    for (html, title) in [
        (
            "<html><head><title>Kept</title></head><body><p>x</p></body></html>",
            Some("Kept"),
        ),
        (
            "<html><body><head><title>Stray</title></head><p>x</p></body></html>",
            None,
        ),
        ("<p>x</p><head><title>Stray</title></head>", None),
        (
            "<html><head><title>Kept</title></head><head><title>Stray</title></head><body><p>x</p></body></html>",
            Some("Kept"),
        ),
    ] {
        let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let blocks: Vec<Option<String>> = build_document_structure(&dom)
            .nodes
            .into_iter()
            .filter_map(|node| match node.content {
                NodeContent::MetadataBlock { entries } => Some(
                    entries
                        .into_iter()
                        .find(|entry| entry.key == "title")
                        .map(|entry| entry.value),
                ),
                _ => None,
            })
            .collect();
        let expected: Vec<Option<String>> = title.map(|t| Some(t.to_string())).into_iter().collect();
        assert_eq!(blocks, expected, "{html}");
        let document = convert(html, Some(opts.clone())).unwrap().metadata.document;
        assert_eq!(document.title.as_deref(), title, "{html}");
    }
}

#[test]
fn should_read_the_head_after_a_utf8_byte_order_mark_on_both_tiers() {
    let html = "\u{FEFF}<html><head><title>T</title></head><body><p>x</p></body></html>";
    for tier in [TierStrategy::Tier2, TierStrategy::Tier1] {
        let opts = ConversionOptions {
            tier_strategy: tier,
            extract_metadata: true,
            ..ConversionOptions::default()
        };
        let out = convert(html, Some(opts)).unwrap().content.unwrap_or_default();
        assert_eq!(out, "---\ntitle: T\n---\n\nx\n", "{tier:?}");
    }
}

// ~keep ── 14. A mangled leading byte order mark no longer drops the whole head (#527 regression) ──

#[test]
fn should_recover_the_head_after_a_mangled_leading_byte_order_mark_on_both_tiers() {
    // ~keep A stand-in for `real-world/issues/gh-190/sjsu.html`'s own opening bytes: a byte
    // ~keep order mark a wrong encoding guess mangled beyond recognition reads as two ordinary
    // ~keep U+FFFD characters by the time parsing sees it (a real U+FEFF is stripped earlier
    // ~keep and never reaches here). See the fixture-accurate reproduction below.
    let html = "\u{FFFD}\u{FFFD}<html><head><title>T</title><meta name=\"description\" content=\"d\"></head><body><p>x</p></body></html>";
    let out = t2(html);
    assert!(
        out.contains("title: T\n") && out.contains("meta-description: d\n"),
        "{out}"
    );
    assert!(out.contains("x\n"), "{out}");

    // ~keep Tier 1 recovers the same metadata. Byte-for-byte parity with Tier 2 is not
    // ~keep asserted: Tier 1 always inserts a blank line after frontmatter, while Tier 2
    // ~keep leaves this bare, unwrapped leading text node exactly as authored, a combination
    // ~keep the real fixture never reaches (it is complex enough that Tier 1 bails to Tier 2
    // ~keep well before the head is at issue, confirmed by the oracle at zero failures).
    let t1_out = t1_only(html);
    assert!(
        t1_out.contains("title: T\n") && t1_out.contains("meta-description: d\n"),
        "{t1_out}"
    );
    assert!(t1_out.contains("x\n"), "{t1_out}");
}

#[test]
fn should_recover_the_head_from_the_sjsu_fixtures_opening_bytes() {
    // ~keep The exact byte shape `real-world/issues/gh-190/sjsu.html` opens with: its true
    // ~keep UTF-16LE byte order mark was already mangled into two U+FFFD's worth of UTF-8
    // ~keep bytes by an earlier, unrelated lossy conversion before this fixture was captured,
    // ~keep and the rest of the document is genuine UTF-16LE text with no BOM left to detect.
    // ~keep The no-BOM heuristic still decodes the whole thing as UTF-16LE, turning those six
    // ~keep bytes into three unrelated non-ASCII characters directly ahead of `<html>`: a fixed
    // ~keep instance of the same bug the simpler stand-in above covers.
    let mut bytes = vec![0xEFu8, 0xBF, 0xBD, 0xEF, 0xBF, 0xBD];
    let doc =
        "<html><head><title>T</title><meta name=\"description\" content=\"d\"></head><body><p>x</p></body></html>";
    for unit in doc.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    let html = String::from_utf8(bytes).expect("NUL bytes are valid single-byte UTF-8");
    let opts = ConversionOptions {
        extract_metadata: true,
        ..ConversionOptions::default()
    };
    let out = convert(&html, Some(opts)).unwrap().content.unwrap_or_default();
    assert!(
        out.contains("title: T\n") && out.contains("meta-description: d\n"),
        "{out}"
    );
    assert!(out.contains("x\n"), "{out}");
}

#[test]
fn should_report_the_head_after_leading_whitespace_on_both_tiers() {
    let html = "   \n\t <html><head><title>T</title></head><body><p>x</p></body></html>";
    let out = t2(html);
    assert!(out.contains("title: T\n"), "{out}");
    assert_eq!(t1_only(html), out);
}

#[test]
fn should_report_the_head_after_a_leading_comment_on_both_tiers() {
    let html = "<!-- note --><html><head><title>T</title></head><body><p>x</p></body></html>";
    let out = t2(html);
    assert!(out.contains("title: T\n"), "{out}");
    assert_eq!(t1_only(html), out);
}

#[test]
fn should_report_the_head_after_a_leading_doctype_on_both_tiers() {
    let html = "<!DOCTYPE html><html><head><title>T</title></head><body><p>x</p></body></html>";
    let out = t2(html);
    assert!(out.contains("title: T\n"), "{out}");
    assert_eq!(t1_only(html), out);
}

#[test]
fn should_report_the_head_after_leading_plain_text_directly_in_front_of_html_on_both_tiers() {
    // ~keep Whatever sits directly in front of the document's own `<html>` tag, garbage or
    // ~keep genuine prose, is forgiven the same way a browser discards it: the head search
    // ~keep only stays strict for a head-only fragment with no `<html>` tag at all (the
    // ~keep `x<head>...` cases in `should_ignore_a_head_after_implicit_body_content_on_both_tiers`).
    let html = "hello<html><head><title>Kept</title></head><body><p>x</p></body></html>";
    let out = t2(html);
    assert!(out.contains("title: Kept\n"), "{out}");
    // ~keep Not byte-for-byte parity with Tier 2 here: see the mangled-BOM test above for why
    // ~keep a bare, unwrapped leading text node is the one case this brief leaves unaligned.
    assert!(t1_only(html).contains("title: Kept\n"), "{}", t1_only(html));
}
