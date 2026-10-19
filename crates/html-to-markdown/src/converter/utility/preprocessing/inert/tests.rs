use super::normalize_inert_bodies;
use std::borrow::Cow;

#[test]
fn should_detect_only_unclosed_noscript_markup_in_a_bounded_fragment() {
    for source in [
        "<noscript>",
        "<NOSCRIPT data-x='>'>hidden",
        "<noscript>x</noscript><noscript>",
        "<script>const s = '<noscript>';</script><noscript>",
    ] {
        assert!(super::has_unclosed_noscript(source), "{source}");
    }
    for source in [
        "<noscript>x</noscript>",
        "<!-- <noscript> -->",
        "<p data-x='<noscript>'>x</p>",
        "<script>const s = '<noscript>';</script>",
        "<style>/* <noscript> */</style>",
        "<xmp><noscript></xmp>",
        "<textarea><noscript></textarea>",
        "<title><noscript></title>",
        "<iframe><noscript></iframe>",
    ] {
        assert!(!super::has_unclosed_noscript(source), "{source}");
    }
}
#[test]
fn should_skip_bogus_end_tag_comments_before_protected_bodies() {
    for next in [" ", "\t", "\n", "0", "!", "?", "/", "'", "é", " data='"] {
        for tag in ["title", "xmp", "iframe", "noscript", "template", "plaintext", "script"] {
            let before = format!("</{next}<{tag}>>");
            let source = format!("{before}<noscript><p>fallback</noscript><p>visible</p>");
            assert_eq!(
                normalize_inert_bodies(&source),
                format!("{before}<noscript></noscript><p>visible</p>"),
                "{next:?}/{tag}"
            );
        }
    }
}

#[test]
fn should_not_match_a_comment_terminator_across_its_body_boundary() {
    for opener in ["<!--!>", "<!---!>", "<!-->!>", "<!--->!>"] {
        let before = format!("{opener}<plaintext><noscript>fake</noscript></plaintext>-->");
        let source = format!("{before}<xmp><b>literal</b></xmp><noscript><p>fallback</noscript><p>visible</p>");
        let expected = format!("{before}<xmp>&lt;b>literal&lt;/b></xmp><noscript></noscript><p>visible</p>");
        if matches!(opener, "<!--!>" | "<!---!>") {
            assert_eq!(normalize_inert_bodies(&source), expected, "{opener}");
        } else {
            assert_eq!(
                normalize_inert_bodies(&source),
                source,
                "an abrupt empty comment exposes plaintext"
            );
        }
    }
}

#[test]
fn should_tokenize_only_the_inert_body_amid_large_html_gaps() {
    let paragraph = "<p data-note='quoted <noscript> fake'>ordinary text</p>";
    let prefix = paragraph.repeat(20_000);
    let suffix = paragraph.repeat(20_000);
    let source = format!("{prefix}<noscript><p>fallback</noscript>{suffix}");
    super::TOKEN_BYTES.with(|bytes| bytes.set(0));
    let normalized = normalize_inert_bodies(&source);
    assert_eq!(normalized, format!("{prefix}<noscript></noscript>{suffix}"));
    super::TOKEN_BYTES.with(|bytes| {
        assert!(bytes.get() > 0, "the candidate must actually be tokenized");
        assert!(bytes.get() <= 64, "tokenized {} of {} bytes", bytes.get(), source.len());
    });
}

#[test]
fn should_keep_fake_candidates_outside_protected_bodies_opaque() {
    for before in [
        "a < b ",
        "<<",
        "<!bogus data-x='fake> ",
        "<?bogus data-x='fake> ",
        "<p data-x='quoted <noscript> fake'></p>",
        "<!-- <noscript><p>fake</noscript> -->",
        "<!--> ",
        "<!---> ",
        "<!-- <noscript> --!>",
        "<script><!--<script><noscript>fake</noscript></script>--></script>",
        "<style><noscript>fake</noscript></style>",
        "<textarea><noscript>fake</noscript></textarea>",
        "<p data-x=unquoted'quote><span></span></p>",
        "<p ='quote><span></span></p>",
    ] {
        let source = format!("{before}<noscript><p>fallback</noscript><p>visible</p>");
        assert_eq!(
            normalize_inert_bodies(&source),
            format!("{before}<noscript></noscript><p>visible</p>")
        );
    }
}

#[test]
fn should_match_tokenizer_abrupt_doctype_identifier_endings() {
    for keyword in ["PUBLIC", "SYSTEM"] {
        let source = format!("<!DOCTYPE html {keyword} \"x><noscript><p>fallback</noscript>\"><p>visible</p>");
        assert_eq!(
            normalize_inert_bodies(&source),
            format!("<!DOCTYPE html {keyword} \"x><noscript></noscript>\"><p>visible</p>")
        );
    }
}

#[test]
fn should_keep_source_offsets_when_a_raw_body_crosses_unicode_chunks() {
    let text = "é🦀".repeat(2_000);
    let source = format!("<p>prefix</p><xmp>{text}<p>literal</xmp><p>visible</p>");
    assert_eq!(
        normalize_inert_bodies(&source),
        format!("<p>prefix</p><xmp>{text}&lt;p>literal</xmp><p>visible</p>")
    );
}

#[test]
fn should_preserve_source_outside_target_bodies() {
    let source = "<p data-x='a > b'>one</p><IFRAME Src=/f><p></IFRAME><p data-y=two>tail</p>";
    assert_eq!(
        normalize_inert_bodies(source),
        "<p data-x='a > b'>one</p><IFRAME Src=/f></IFRAME><p data-y=two>tail</p>"
    );
}

#[test]
fn should_leave_svg_title_markup_unchanged() {
    let source = "<svg><title>a <p> b</p></title></svg>";
    assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(value) if value == source));
}

#[test]
fn should_protect_raw_text_inside_foreign_object() {
    assert_eq!(
        normalize_inert_bodies("<svg><foreignObject><iframe><p></iframe></foreignObject></svg>"),
        "<svg><foreignObject><iframe></iframe></foreignObject></svg>"
    );
}

#[test]
fn should_ignore_unmatched_foreign_end_tags() {
    assert_eq!(
        normalize_inert_bodies("<svg></math><title>a <p> b</p></title></svg>"),
        "<svg></math><title>a <p> b</p></title></svg>"
    );
}

#[test]
fn should_restore_outer_namespace_after_dropping_a_template() {
    assert_eq!(
        normalize_inert_bodies("<template><svg></template><xmp><p></xmp>"),
        "<template></template><xmp>&lt;p></xmp>"
    );
}

#[test]
fn should_borrow_already_normalized_target_bodies() {
    assert!(matches!(
        normalize_inert_bodies("<head><title>safe title</title></head>"),
        Cow::Borrowed(_)
    ));
}
#[test]
fn should_not_count_a_foreign_template_as_an_html_template() {
    let source = "<template><svg><template></svg></template><p>visible</p>";
    assert_eq!(normalize_inert_bodies(source), "<template></template><p>visible</p>");
}

#[test]
fn should_break_out_of_svg_on_an_html_paragraph() {
    let source = "<svg><p><iframe><p></iframe><p>visible</p>";
    assert_eq!(
        normalize_inert_bodies(source),
        "<svg><p><iframe></iframe><p>visible</p>"
    );
    let (repaired, limit) = crate::converter::main_helpers::repair_with_html5ever_bounded(source).expect("repair HTML");
    assert_eq!(limit, None);
    assert_eq!(
        repaired,
        "<head></head><svg></svg><p><iframe><p></iframe></p><p>visible</p>"
    );
}

#[test]
fn should_not_treat_math_integration_names_as_svg_integration_points() {
    let source = "<svg><mi><xmp>a <b>x</b></xmp></mi></svg>";
    assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(value) if value == source));
}

#[test]
fn should_not_treat_svg_integration_names_as_math_integration_points() {
    let source = "<math><title><xmp>a <b>x</b></xmp></title></math>";
    assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(value) if value == source));
}

#[test]
fn should_keep_math_text_integration_exceptions_in_math_namespace() {
    for tag in ["mglyph", "malignmark"] {
        let source = format!("<math><mtext><{tag}><xmp>a &amp; b</xmp></{tag}></mtext></math>");
        assert!(matches!(normalize_inert_bodies(&source), Cow::Borrowed(value) if value == source));
    }
}

#[test]
fn should_treat_html_void_names_as_normal_foreign_elements() {
    let source = "<svg><source><title></source><xmp>a &amp; b</xmp></svg>";
    assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(value) if value == source));
}

#[test]
fn should_enter_svg_inside_math_annotation_without_an_encoding() {
    let source =
        "<math><annotation-xml><svg><foreignObject><iframe><p></iframe></foreignObject></svg></annotation-xml></math>";
    let expected =
        "<math><annotation-xml><svg><foreignObject><iframe></iframe></foreignObject></svg></annotation-xml></math>";
    assert_eq!(normalize_inert_bodies(source), expected);
}
#[test]
fn should_skip_tokenization_for_bodies_without_literal_markup() {
    super::TOKEN_VISITS.with(|visits| visits.set(0));
    let source = "<TITLE>safe &amp; title</TITLE><noscript>plain</noscript><iframe>plain</iframe><template>plain</template><p>visible</p>";
    assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(_)));
    super::TOKEN_VISITS.with(|visits| assert_eq!(visits.get(), 0));
}

#[test]
fn should_bound_namespace_work_for_unmatched_foreign_closes() {
    const DEPTH: usize = 10_000;
    let source = "<title><p></title><svg>".to_owned() + &"<g>".repeat(DEPTH) + &"</zz>".repeat(DEPTH);
    super::NAMESPACE_VISITS.with(|visits| visits.set(0));
    super::TOKEN_VISITS.with(|visits| visits.set(0));
    let normalized = normalize_inert_bodies(&source);
    assert!(normalized.starts_with("<title>&lt;p></title><svg>"));
    super::TOKEN_VISITS.with(|visits| assert!(visits.get() >= 2 * DEPTH));
    super::NAMESPACE_VISITS.with(|visits| assert_eq!(visits.get(), DEPTH));
}

#[test]
fn should_skip_nested_candidate_shapes_inside_a_scanned_attribute() {
    let source = "<title a=".to_owned() + &"<title a=".repeat(10_000) + ">safe</title>";
    super::TAG_SCANS.with(|scans| scans.set(0));
    super::TOKEN_VISITS.with(|visits| visits.set(0));
    assert!(matches!(normalize_inert_bodies(&source), Cow::Borrowed(_)));
    super::TAG_SCANS.with(|scans| assert_eq!(scans.get(), 1));
    super::TOKEN_VISITS.with(|visits| assert_eq!(visits.get(), 0));
}

#[test]
fn should_preserve_closing_token_bytes_with_quoted_tag_shapes() {
    let source = "<TEMPLATE><p></TEMPLATE data-x='</TEMPLATE> >'>";
    assert_eq!(
        normalize_inert_bodies(source),
        "<TEMPLATE></TEMPLATE data-x='</TEMPLATE> >'>"
    );
    let source = "<TEMPLATE><div data-x='</template a=\"'></div></TEMPLATE>";
    assert_eq!(normalize_inert_bodies(source), "<TEMPLATE></TEMPLATE>");
    let source = "<NOSCRIPT><p></NOSCRIPT data=x='>";
    assert_eq!(normalize_inert_bodies(source), "<NOSCRIPT></NOSCRIPT data=x='>");
}

#[test]
fn should_bound_unmatched_template_close_lookups_in_foreign_markup() {
    const DEPTH: usize = 10_000;
    let source = "<title><p></title><svg>".to_owned() + &"<g>".repeat(DEPTH) + &"</template>".repeat(DEPTH);
    super::NAMESPACE_VISITS.with(|visits| visits.set(0));
    assert!(normalize_inert_bodies(&source).starts_with("<title>&lt;p></title>"));
    super::NAMESPACE_VISITS.with(|visits| assert_eq!(visits.get(), 2 * DEPTH));
}

#[test]
fn should_avoid_source_and_entity_decoded_body_marker_collisions() {
    let source = "<p>h2m_inert_body_0_end h2m_inert_body_&#49;_end</p><noscript><p>x</noscript>";
    let (normalized, bodies) =
        super::normalize_inert_bodies_with_preservation(source, true, crate::options::HiddenContent::Drop);
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies.get("h2m_inert_body_2_end").map(String::as_str), Some("<p>x"));
    assert!(normalized.contains("<noscript>h2m_inert_body_2_end</noscript>"));
    let mut dom = tl::parse(&normalized, tl::ParserOptions::default()).expect("parse protected bodies");
    crate::converter::utility::preprocessing::restore_inert_bodies(&mut dom, &bodies).expect("restore original bodies");
    assert_eq!(
        crate::converter::utility::serialization::serialize_tag_to_html(&dom.children()[1], dom.parser()),
        "<noscript><p>x</noscript>"
    );
}

#[test]
fn should_recognize_form_feed_as_html_tag_whitespace() {
    assert_eq!(
        normalize_inert_bodies("<NOSCRIPT\x0c><p></NOSCRIPT\x0c><p>visible</p>"),
        "<NOSCRIPT\x0c></NOSCRIPT\x0c><p>visible</p>"
    );
}

#[test]
fn should_finish_unterminated_protected_bodies_at_eof() {
    for (source, expected) in [
        ("<title><p>", "<title>&lt;p></title>"),
        ("<xmp>a &amp;", "<xmp>a &amp;amp;</xmp>"),
        ("<iframe><p>x", "<iframe></iframe>"),
        ("<noscript><p>x", "<noscript></noscript>"),
        ("<template><p>x", "<template></template>"),
    ] {
        assert_eq!(normalize_inert_bodies(source), expected, "{source}");
    }
}

#[test]
fn should_identify_html_inert_starts_with_source_namespace_ownership() {
    for source in [
        r"<svg><template class=x>foreign</template></svg><template id=real>html</template>",
        r"<math><noscript>foreign</noscript></math><template id=real>html</template>",
        r"<svg><foreignObject><template id=inside>html</template></foreignObject></svg><template id=real>page</template>",
        r"<svg><desc><template id=inside>html</template></desc></svg><template id=real>page</template>",
        r"<math><mtext><template id=inside>html</template></mtext></math><template id=real>page</template>",
        r"<math><mtext><mglyph><template>foreign</template></mglyph><template id=inside>html</template></mtext></math><template id=real>page</template>",
        r"<math><annotation-xml encoding=text/html><template id=inside>html</template></annotation-xml></math><template id=real>page</template>",
        r"<math><annotation-xml><template>foreign</template></annotation-xml></math><template id=real>html</template>",
        r"<svg><foreignObject><math><template>foreign</template></math><template id=inside>html</template></foreignObject></svg><template id=real>page</template>",
        r"<!-- <template>fake</template> --><script><template>fake</template></script><template id=real>html</template>",
        r"<script><svg><template>fake</template></script><template id=real>html</template>",
        r"<svg><p><template>graphic</template></p></svg><template id=real>html</template>",
        r"<svg><foreignObject><math><template>graphic</template></math></foreignObject></SVG><template id=real>html</template>",
        r"<svg/><math/><template id=real>html</template>",
        r"<textarea><template>fake</template></textarea><iframe><template>fake</template></iframe><template id=real>html</template>",
        r"<title><template>fake</template></title><p data-x='<template>fake</template>'>x</p><template id=real>html</template>",
    ] {
        let expected = vec![source.find("<template id=real>").expect("real source start")];
        assert_eq!(
            super::html_inert_starts(source, crate::options::HiddenContent::All),
            expected,
            "{source}"
        );
    }
}

#[test]
fn should_keep_inert_source_offsets_after_unicode_and_large_opaque_regions() {
    let html = format!(
        "<script>{}</script><math><template>foreign</template></math><template id=real>html</template>",
        "雪<svg>".repeat(1500)
    );
    assert_eq!(
        super::html_inert_starts(&html, crate::options::HiddenContent::All),
        vec![html.find("<template id=real>").expect("real source start")]
    );
}

#[test]
fn should_tokenize_only_relevant_tags_in_the_source_form_pass() {
    use crate::options::HiddenContent;
    for paragraphs in [100, 1000, 4000] {
        let prefix = "<p>ordinary text <b>bold</b></p>".repeat(paragraphs);
        for (suffix, tokens, bytes) in [("", 0, 0), ("<form>x</form>", 3, 13)] {
            let source = format!("{prefix}{suffix}");
            super::TOKEN_VISITS.with(|counter| counter.set(0));
            super::TOKEN_BYTES.with(|counter| counter.set(0));
            let _ = super::remove_closed_form_bodies(&source, HiddenContent::Drop);
            assert_eq!(super::TOKEN_VISITS.with(std::cell::Cell::get), tokens);
            assert_eq!(super::TOKEN_BYTES.with(std::cell::Cell::get), bytes);
        }
    }
}
