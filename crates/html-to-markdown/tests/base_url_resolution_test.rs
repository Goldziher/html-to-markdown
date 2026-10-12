//! Coverage for `ConversionOptions::base_url`, which resolves relative `href`/`src`
//! destinations against a caller-supplied base URL.
//!
//! The Tier-1 (byte scanner) and Tier-2 (DOM walk) conversion paths implement URL
//! resolution independently (`converter/tier1/scanner.rs` and the `converter/handlers/*`
//! / `converter/inline/link.rs` / `converter/media/embedded.rs` call sites), both
//! reading the SAME pre-computed `Option<Rc<url::Url>>` that `convert_api.rs` builds once
//! per conversion (see `converter::url_resolve::compute_effective_base`). `assert_tier1_matches_tier2`
//! below is the parity check that would catch a resolve-only-one-tier regression: temporarily
//! reverting either of `converter/tier1/scanner.rs`'s two `state.resolve_url(...)` call sites
//! and rerunning this file fails 7 of these 13 tests with an explicit tier1-vs-tier2 diff
//! (verified manually; a "make it fail on purpose" run cannot itself be committed as a
//! passing test).

#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn convert_with(html: &str, base_url: Option<&str>, tier_strategy: TierStrategy) -> String {
    let options = ConversionOptions {
        base_url: base_url.map(str::to_string),
        tier_strategy,
        // ~keep Metadata/frontmatter extraction is unrelated to URL resolution and has its
        // own pre-existing Tier-1/Tier-2 blank-line divergence when a `<base>` tag is
        // present (reproduced independently of `base_url`, on unmodified `main`) --
        // disabled here so these tests isolate `base_url` resolution specifically.
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

/// Runs `html` through both tiers (forcing each explicitly via the testkit-only
/// `TierStrategy::Tier1`/`Tier2`, not the auto-routing default) with the same
/// `base_url`, and asserts the two tiers produced byte-identical output.
fn assert_tier1_matches_tier2(html: &str, base_url: &str) -> String {
    let tier1_output = convert_with(html, Some(base_url), TierStrategy::Tier1);
    let tier2_output = convert_with(html, Some(base_url), TierStrategy::Tier2);
    assert_eq!(
        tier1_output, tier2_output,
        "tier1 diverged from tier2 for base_url resolution\ninput: {html:?}\nbase_url: {base_url:?}\ntier1: {tier1_output:?}\ntier2: {tier2_output:?}"
    );
    tier1_output
}

#[test]
fn should_leave_output_byte_identical_when_base_url_is_unset() {
    let html = r#"<a href="rel/child.html">child</a> <img src="pic.png" alt="a">"#;
    let without_base = convert_with(html, None, TierStrategy::Auto);
    assert_eq!(without_base, "[child](rel/child.html) ![a](pic.png)\n");
}

#[test]
fn should_resolve_relative_link_against_base_url_on_both_tiers() {
    let html = r#"<a href="rel/child.html">child</a>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/blog/index.html");
    assert_eq!(out, "[child](https://example.com/blog/rel/child.html)\n");
}

#[test]
fn should_not_report_caller_base_url_as_document_frontmatter() {
    let html = r#"<html><head></head><body><a href="child.html">child</a></body></html>"#;

    for tier_strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
        let options = ConversionOptions {
            base_url: Some("https://example.com/blog/index.html".to_string()),
            extract_metadata: true,
            tier_strategy,
            ..ConversionOptions::default()
        };
        let output = convert(html, Some(options))
            .expect("conversion should succeed")
            .content
            .unwrap_or_default();

        assert_eq!(
            output, "[child](https://example.com/blog/child.html)\n",
            "tier: {tier_strategy:?}"
        );
    }
}

#[test]
fn should_report_the_effective_document_base_in_frontmatter() {
    let html = r#"<html><head><base href="/other/"></head><body><a href="x">x</a></body></html>"#;

    for tier_strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
        let options = ConversionOptions {
            base_url: Some("https://example.com/docs/index.html".to_string()),
            tier_strategy,
            ..ConversionOptions::default()
        };
        let output = convert(html, Some(options))
            .expect("conversion should succeed")
            .content
            .unwrap_or_default();

        let expected = match tier_strategy {
            TierStrategy::Tier1 => "---\nbase: https://example.com/other/\n---\n\n[x](https://example.com/other/x)\n",
            TierStrategy::Tier2 => "---\nbase: https://example.com/other/\n---\n[x](https://example.com/other/x)\n",
            TierStrategy::Auto => unreachable!("the test enumerates forced tiers only"),
        };
        assert_eq!(output, expected, "tier: {tier_strategy:?}");
    }
}

#[test]
fn should_resolve_absolute_path_link_against_base_url_on_both_tiers() {
    let html = r#"<a href="/about">about</a>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/blog/index.html");
    assert_eq!(out, "[about](https://example.com/about)\n");
}

#[test]
fn should_resolve_relative_image_src_against_base_url_on_both_tiers() {
    let html = r#"<img src="images/pic.png" alt="a photo">"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/blog/index.html");
    assert_eq!(out, "![a photo](https://example.com/blog/images/pic.png)\n");
}

#[test]
fn should_resolve_protocol_relative_url_against_base_scheme_on_both_tiers() {
    let html = r#"<a href="//cdn.example.com/x">x</a>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/");
    assert_eq!(out, "[x](https://cdn.example.com/x)\n");
}

#[test]
fn should_resolve_fragment_only_href_against_full_page_url_on_both_tiers() {
    let html = r##"<a href="#section">jump</a>"##;
    let out = assert_tier1_matches_tier2(html, "https://example.com/blog/post.html");
    assert_eq!(out, "[jump](https://example.com/blog/post.html#section)\n");
}

#[test]
fn should_leave_already_absolute_url_unchanged_on_both_tiers() {
    let html = r#"<a href="https://other.example/x">x</a>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/");
    assert_eq!(out, "[x](https://other.example/x)\n");
}

#[test]
fn should_leave_mailto_href_unchanged_on_both_tiers() {
    let html = r#"<a href="mailto:foo@bar.com">mail</a>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/");
    assert_eq!(out, "[mail](mailto:foo@bar.com)\n");
}

#[test]
fn should_resolve_empty_href_against_the_base_on_both_tiers() {
    let html = r#"<a href="">empty</a>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/");
    assert_eq!(out, "[empty](https://example.com/)\n");
}

#[test]
fn should_leave_malformed_relative_reference_unchanged_without_panicking_on_both_tiers() {
    let html = r#"<a href="http://[not-a-valid-host">bad</a>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/");
    assert_eq!(out, "[bad](http://[not-a-valid-host)\n");
}

#[test]
fn should_leave_output_unchanged_when_base_url_itself_is_malformed() {
    let html = r#"<a href="rel/child.html">child</a>"#;
    let out = assert_tier1_matches_tier2(html, "not a url");
    assert_eq!(out, "[child](rel/child.html)\n");
}

#[test]
fn should_resolve_document_base_href_relative_to_caller_base_url_on_both_tiers() {
    let html = r#"<html><head><base href="/assets/"></head><body><a href="child.html">c</a></body></html>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/blog/post.html");
    assert_eq!(out, "[c](https://example.com/assets/child.html)\n");
}

#[test]
fn should_let_absolute_document_base_href_override_caller_base_url_on_both_tiers() {
    let html = r#"<html><head><base href="https://cdn.example.com/"></head><body><a href="x.png">x</a></body></html>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/blog/post.html");
    assert_eq!(out, "[x](https://cdn.example.com/x.png)\n");
}

/// A `<base>` a browser reads as text or as a comment does not set the base.
#[test]
fn should_ignore_a_document_base_href_inside_a_comment_or_title_on_both_tiers() {
    for head in [
        r#"<!-- <base href="https://evil.example/"> -->"#,
        r#"<title><base href="https://evil.example/"></title>"#,
    ] {
        let html = format!(r#"<html><head>{head}</head><body><a href="child.html">c</a></body></html>"#);
        let out = assert_tier1_matches_tier2(&html, "https://example.com/blog/post.html");
        assert_eq!(out, "[c](https://example.com/blog/child.html)\n", "head: {head}");
    }
}

/// A `<blockquote cite>` is a destination the converter renders, so `base_url` must resolve it.
///
/// ~keep Found while adopting `base_url` in crawlberg, whose own link pre-pass
/// (`html/link_targets.rs`) resolves `blockquote cite` and would have to stay alive purely
/// for this one attribute otherwise. `cite` is the only destination in that pre-pass's
/// target list that 3.15.0 left unresolved.
#[test]
fn should_resolve_blockquote_cite_against_base_url_on_both_tiers() {
    let html = r#"<blockquote cite="/source.html">quoted</blockquote>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/blog/index.html");
    assert!(
        out.contains("https://example.com/source.html"),
        "a relative blockquote cite must resolve against base_url, got {out:?}"
    );
}

/// An already-absolute `cite` passes through untouched, like every other destination.
#[test]
fn should_leave_an_absolute_blockquote_cite_unchanged_on_both_tiers() {
    let html = r#"<blockquote cite="https://other.example/s.html">quoted</blockquote>"#;
    let out = assert_tier1_matches_tier2(html, "https://example.com/blog/index.html");
    assert!(
        out.contains("https://other.example/s.html"),
        "an absolute cite must survive verbatim, got {out:?}"
    );
}

/// Tier 1 and Tier 2 must agree on a cited blockquote even with no `base_url` at all.
///
/// ~keep This is the parity half, separable from resolution: Tier 1's `open_blockquote` took no
/// attributes and never read `cite`, so it dropped a citation Tier 2 emitted as
/// `\u{2014} <url>`. Without `base_url` in play the two tiers still have to agree.
#[test]
fn should_render_a_cited_blockquote_identically_on_both_tiers_without_a_base_url() {
    let html = r#"<blockquote cite="https://other.example/s.html">quoted</blockquote>"#;
    let tier1 = convert_with(html, None, TierStrategy::Tier1);
    let tier2 = convert_with(html, None, TierStrategy::Tier2);
    assert_eq!(tier1, tier2, "tier1 dropped or reshaped the blockquote citation");
    assert!(
        tier1.contains("https://other.example/s.html"),
        "a blockquote citation must reach the markdown, got {tier1:?}"
    );
}

#[test]
fn should_not_resolve_link_like_text_inside_textarea() {
    let html = r#"<textarea>see <a href="x.html">here</a> <img src="pic.png" alt="pic"></textarea><p><a href="real.html">real</a></p>"#;
    let out = convert_with(html, Some("https://example.com/dir/page.html"), TierStrategy::Tier2);

    assert_eq!(
        out,
        "see [here](x.html) ![pic](pic.png)\n\n[real](https://example.com/dir/real.html)\n"
    );
}

#[test]
fn should_not_resolve_link_like_text_inside_other_raw_text_elements() {
    for (element, expected) in [
        ("xmp", "see [here](x.html)\n\nafter\n"),
        ("iframe", "after\n"),
        ("noscript", "after\n"),
        ("noembed", "see [here](x.html)\n\nafter\n"),
        ("noframes", "see [here](x.html)\n\nafter\n"),
    ] {
        let html = format!(r#"<{element}>see <a href="x.html">here</a></{element}><p>after</p>"#);
        let out = convert_with(&html, Some("https://example.com/dir/page.html"), TierStrategy::Tier2);

        assert_eq!(out, expected, "element: {element}");
    }
}

#[test]
fn should_keep_title_script_and_style_contents_outside_url_resolution() {
    let html = r#"
        <html>
          <head>
            <title>see <a href="title.html">title</a></title>
            <script>const link = '<a href="script.html">script</a>';</script>
            <style>.x { background: url("style.html"); }</style>
          </head>
          <body><p><a href="real.html">real</a></p></body>
        </html>
    "#;
    let out = convert_with(html, Some("https://example.com/dir/page.html"), TierStrategy::Tier2);

    assert_eq!(out, "[real](https://example.com/dir/real.html)\n");
}
