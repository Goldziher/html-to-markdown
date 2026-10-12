//! Regression coverage for lazy-loaded `<img>` source resolution.
//!
//! An `<img>` whose real address lives in a lazy-loading attribute (`data-src` and its
//! common variants) rather than `src` previously converted to `![alt]()` -- the image was
//! effectively invisible in the output. See `resolve_effective_src` in
//! `src/converter/handlers/image.rs` for the precedence rule these tests pin.

// ~keep Rust inner attribute below is a crate-level attribute, not a shell shebang.
#![allow(missing_docs)]

fn convert(html: &str) -> String {
    html_to_markdown_rs::convert(html, None)
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

#[test]
fn should_leave_a_plain_img_with_only_src_byte_identical() {
    let html = r#"<img src="https://example.com/photo.jpg" alt="A photo" />"#;
    assert_eq!(convert(html), "![A photo](https://example.com/photo.jpg)\n");
}

#[test]
fn should_leave_a_plain_img_with_src_and_title_byte_identical() {
    let html = r#"<img src="https://example.com/photo.jpg" alt="A photo" title="Nice" />"#;
    assert_eq!(convert(html), "![A photo](https://example.com/photo.jpg \"Nice\")\n");
}

#[test]
fn should_fall_back_to_data_src_when_src_is_absent() {
    let html = r#"<img data-src="https://cdn.example.com/real.jpg" alt="Real image" />"#;
    assert_eq!(convert(html), "![Real image](https://cdn.example.com/real.jpg)\n");
}

#[test]
fn should_fall_back_to_data_src_when_src_is_empty() {
    let html = r#"<img src="" data-src="https://cdn.example.com/real.jpg" alt="Real image" />"#;
    assert_eq!(convert(html), "![Real image](https://cdn.example.com/real.jpg)\n");
}

#[test]
fn should_fall_back_to_data_src_when_src_is_a_data_uri_placeholder() {
    let placeholder = "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
    let html = format!(r#"<img src="{placeholder}" data-src="https://cdn.example.com/real.jpg" alt="Real image" />"#);
    assert_eq!(convert(&html), "![Real image](https://cdn.example.com/real.jpg)\n");
}

#[test]
fn should_fall_back_to_data_lazy_src_when_data_src_is_absent() {
    let html = r#"<img data-lazy-src="https://cdn.example.com/lazy.jpg" alt="Lazy image" />"#;
    assert_eq!(convert(html), "![Lazy image](https://cdn.example.com/lazy.jpg)\n");
}

#[test]
fn should_fall_back_to_data_original_when_other_attributes_are_absent() {
    let html = r#"<img data-original="https://cdn.example.com/original.jpg" alt="Original image" />"#;
    assert_eq!(
        convert(html),
        "![Original image](https://cdn.example.com/original.jpg)\n"
    );
}

#[test]
fn should_prefer_data_src_over_data_lazy_src_and_data_original() {
    let html = r#"<img data-src="https://cdn.example.com/preferred.jpg"
                       data-lazy-src="https://cdn.example.com/second.jpg"
                       data-original="https://cdn.example.com/third.jpg"
                       alt="Preferred image" />"#;
    assert_eq!(
        convert(html),
        "![Preferred image](https://cdn.example.com/preferred.jpg)\n"
    );
}

#[test]
fn should_fall_back_to_srcset_highest_resolution_candidate_when_src_is_absent() {
    let html = r#"<img srcset="https://cdn.example.com/small.jpg 480w, https://cdn.example.com/large.jpg 1200w" alt="Responsive image" />"#;
    assert_eq!(
        convert(html),
        "![Responsive image](https://cdn.example.com/large.jpg)\n"
    );
}

#[test]
fn should_fall_back_to_srcset_highest_density_candidate_when_src_is_absent() {
    let html = r#"<img srcset="https://cdn.example.com/1x.jpg 1x, https://cdn.example.com/3x.jpg 3x, https://cdn.example.com/2x.jpg 2x" alt="Retina image" />"#;
    assert_eq!(convert(html), "![Retina image](https://cdn.example.com/3x.jpg)\n");
}

#[test]
fn should_prefer_data_srcset_over_plain_srcset() {
    let html = r#"<img srcset="https://cdn.example.com/placeholder.gif 1x"
                       data-srcset="https://cdn.example.com/real-480.jpg 480w, https://cdn.example.com/real-960.jpg 960w"
                       alt="Lazy responsive image" />"#;
    assert_eq!(
        convert(html),
        "![Lazy responsive image](https://cdn.example.com/real-960.jpg)\n"
    );
}

#[test]
fn should_prefer_single_url_lazy_attributes_over_srcset_candidates() {
    let html = r#"<img data-src="https://cdn.example.com/precise.jpg"
                       srcset="https://cdn.example.com/wide-1200.jpg 1200w"
                       alt="Precise image" />"#;
    assert_eq!(convert(html), "![Precise image](https://cdn.example.com/precise.jpg)\n");
}

#[test]
fn should_trust_a_populated_non_data_src_even_alongside_a_data_placeholder_in_srcset() {
    // ~keep Mirrors the "jetpack-lazy-image" pattern from
    // ~keep test_documents/html/issues/gh-190/rbloggers.html: `src` already holds the real
    // ~keep photo while `srcset` holds only the lazy-load `data:` placeholder. A populated,
    // ~keep non-`data:` `src` must win -- this crate cannot fetch the URL to check whether it
    // ~keep is itself a placeholder graphic.
    let html = r#"<img src="https://cdn.example.com/already-real.gif"
                       srcset="data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7"
                       data-lazy-src="https://cdn.example.com/second.gif"
                       alt="Feedburner count" />"#;
    assert_eq!(
        convert(html),
        "![Feedburner count](https://cdn.example.com/already-real.gif)\n"
    );
}

#[test]
fn should_keep_a_pure_data_uri_src_when_no_fallback_attribute_exists() {
    let data_uri = "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
    let html = format!(r#"<img src="{data_uri}" alt="Inline pixel" />"#);
    assert_eq!(convert(&html), format!("![Inline pixel]({data_uri})\n"));
}

#[test]
fn should_use_alt_text_when_an_img_has_no_usable_source() {
    // ~keep With no usable source, the alternative text stands for the image (#756).
    let html = r#"<img alt="No source anywhere" />"#;
    assert_eq!(convert(html), "No source anywhere\n");
}

#[test]
fn should_resolve_lazy_src_for_an_img_nested_inside_a_picture_element() {
    let html = r#"<picture>
        <source srcset="https://cdn.example.com/small.webp" type="image/webp">
        <img data-src="https://cdn.example.com/real.jpg" alt="Picture fallback" />
    </picture>"#;
    assert_eq!(convert(html), "![Picture fallback](https://cdn.example.com/real.jpg)\n");
}

#[test]
fn should_keep_a_parenthesised_comma_inside_its_srcset_candidate() {
    let html = r#"<img srcset="https://x/a.png (x, https://x/b.png 3x ), https://x/c.png 2x" alt="Paren" />"#;
    assert_eq!(convert(html), "![Paren](https://x/c.png)\n");
}

#[test]
fn should_keep_a_comma_inside_a_srcset_candidate_url() {
    let html = r#"<img srcset="https://x/a.png?w=1,2 2x, https://x/b.png 1x" alt="Comma" />"#;
    assert_eq!(convert(html), "![Comma](https://x/a.png?w=1,2)\n");
}

#[test]
fn should_read_the_img_srcset_and_not_a_picture_source_srcset() {
    let html = r#"<picture>
        <source srcset="https://x/wrong.webp 9x, https://x/also-wrong.webp (a, b)" type="image/webp">
        <img srcset="https://x/a.png (x, https://x/b.png 3x ), https://x/c.png 2x" alt="Picture" />
    </picture>"#;
    assert_eq!(convert(html), "![Picture](https://x/c.png)\n");
}

#[test]
fn should_skip_a_srcset_candidate_whose_descriptor_is_invalid() {
    let html = r#"<img srcset="https://x/a.png foo, https://x/b.png" alt="Invalid" />"#;
    assert_eq!(convert(html), "![Invalid](https://x/b.png)\n");
}

#[test]
fn should_skip_a_srcset_density_that_is_not_a_spec_number() {
    let html = r#"<img srcset="https://x/a.png NaNx, https://x/b.png infx, https://x/c.png 2x" alt="Number" />"#;
    assert_eq!(convert(html), "![Number](https://x/c.png)\n");
}

#[test]
fn should_not_compare_a_srcset_width_with_a_density() {
    let html = r#"<img srcset="https://x/a.png 900x, https://x/b.png 800w" alt="Kinds" />"#;
    assert_eq!(convert(html), "![Kinds](https://x/b.png)\n");
}

#[test]
fn should_resolve_every_data_src_image_in_the_squarespace_fixture() {
    let path = [
        env!("CARGO_MANIFEST_DIR"),
        "../../test_documents/html/squarespace/squarespace-layout-grid-page.html",
    ]
    .iter()
    .collect::<std::path::PathBuf>();
    let html = std::fs::read_to_string(&path).expect("read squarespace fixture");

    let markdown = convert(&html);

    for expected in [
        "https://images.squarespace-cdn.com/content/v1/abc123/1690000000001-KILNDRUM/kiln-drum-lined.jpg",
        "https://images.squarespace-cdn.com/content/v1/abc123/1690000000002-BURNER/burner-port.jpg",
        "https://images.squarespace-cdn.com/content/v1/abc123/1690000000003-FIRST-FIRE/first-fire.jpg",
        "https://images.squarespace-cdn.com/content/v1/abc123/1690000000004-RAKU-POTS/raku-pots-cooling.jpg",
    ] {
        assert!(
            markdown.contains(expected),
            "expected resolved data-src URL {expected} in output, got:\n{markdown}"
        );
    }
}
