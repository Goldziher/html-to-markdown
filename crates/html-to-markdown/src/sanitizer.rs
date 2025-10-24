//! HTML sanitization using ammonia.

use ammonia::Builder;

use crate::error::Result;
use crate::options::{PreprocessingOptions, PreprocessingPreset};

/// Sanitize HTML using ammonia.
///
/// This function cleans HTML by removing unwanted elements and attributes
/// based on the preprocessing options.
///
/// Tags in `preserve_tags` are allowed along with common attributes (id, class, style, etc.)
/// to ensure they can be output as HTML later in the conversion process.
pub fn sanitize(html: &str, options: &PreprocessingOptions, preserve_tags: &[String]) -> Result<String> {
    use std::collections::HashSet;

    // First, determine which additional tags and attributes we need based on preserve_tags
    let needs_extra_attrs = !preserve_tags.is_empty();
    let preserve_tag_set: HashSet<&str> = preserve_tags.iter().map(|s| s.as_str()).collect();

    let mut builder = match options.preset {
        PreprocessingPreset::Minimal => create_minimal_builder(),
        PreprocessingPreset::Standard => create_standard_builder(),
        PreprocessingPreset::Aggressive => create_aggressive_builder(),
    };

    // Get the current tags and convert to a HashSet<String> so we can add preserve_tags
    let current_tags = builder.clone_tags();
    let mut allowed_tags: HashSet<String> = current_tags.iter().map(|&s| s.to_string()).collect();

    // Add preserve_tags
    for tag in preserve_tags {
        allowed_tags.insert(tag.clone());
    }

    let mut clean_content = HashSet::new();

    clean_content.insert("script".to_string());
    clean_content.insert("style".to_string());
    allowed_tags.remove("script");
    allowed_tags.remove("style");

    if options.remove_navigation {
        clean_content.insert("nav".to_string());
        clean_content.insert("aside".to_string());
        clean_content.insert("header".to_string());
        clean_content.insert("footer".to_string());
        allowed_tags.remove("nav");
        allowed_tags.remove("aside");
        allowed_tags.remove("header");
        allowed_tags.remove("footer");
    }

    if options.remove_forms {
        // Don't remove forms if they're in preserve_tags
        if !preserve_tag_set.contains("form") {
            clean_content.insert("form".to_string());
            allowed_tags.remove("form");
        }
        // Note: Don't remove "input" here - we'll keep checkboxes for task lists
        clean_content.insert("button".to_string());
        clean_content.insert("select".to_string());
        clean_content.insert("textarea".to_string());
        clean_content.insert("label".to_string());
        clean_content.insert("fieldset".to_string());
        clean_content.insert("legend".to_string());
        allowed_tags.remove("button");
        allowed_tags.remove("select");
        allowed_tags.remove("textarea");
        allowed_tags.remove("label");
        allowed_tags.remove("fieldset");
        allowed_tags.remove("legend");
    }

    // Convert back to static strings by leaking
    // This is necessary because ammonia::Builder requires 'static lifetimes
    let allowed_tags_static: HashSet<&'static str> = allowed_tags
        .iter()
        .map(|s| Box::leak(s.clone().into_boxed_str()) as &'static str)
        .collect();

    builder.tags(allowed_tags_static);

    let clean_content_static: HashSet<&'static str> = clean_content
        .iter()
        .map(|s| Box::leak(s.clone().into_boxed_str()) as &'static str)
        .collect();
    builder.clean_content_tags(clean_content_static);

    // For preserve_tags, add common HTML attributes to generic attributes
    // This allows these attributes on all tags, ensuring preserved tags keep their attributes
    if needs_extra_attrs {
        let mut generic_attrs = builder.clone_generic_attributes();
        generic_attrs.insert("id");
        generic_attrs.insert("class");
        generic_attrs.insert("style");
        generic_attrs.insert("title");
        generic_attrs.insert("name");
        generic_attrs.insert("value");
        generic_attrs.insert("type");
        generic_attrs.insert("href");
        generic_attrs.insert("src");
        generic_attrs.insert("alt");
        generic_attrs.insert("width");
        generic_attrs.insert("height");
        generic_attrs.insert("colspan");
        generic_attrs.insert("rowspan");
        generic_attrs.insert("role");
        builder.generic_attributes(generic_attrs);
    }

    Ok(builder.clean(html).to_string())
}

/// Create a minimal sanitization builder (keeps most elements).
fn create_minimal_builder() -> Builder<'static> {
    use std::collections::HashSet;

    let mut builder = Builder::default();
    builder.strip_comments(false);

    // Allow data: URLs for inline images (base64 encoded images)
    let mut url_schemes = builder.clone_url_schemes();
    url_schemes.insert("data");
    builder.url_schemes(url_schemes);

    // Add input, meta, and SVG tags for checkbox support, hOCR detection, and inline images
    let mut tags = builder.clone_tags();
    tags.insert("input");
    tags.insert("meta");
    tags.insert("head");
    tags.insert("svg");
    tags.insert("circle");
    tags.insert("rect");
    tags.insert("path");
    tags.insert("line");
    tags.insert("polyline");
    tags.insert("polygon");
    tags.insert("ellipse");
    tags.insert("g");
    builder.tags(tags);

    // Allow type and checked attributes on input elements
    // Allow name and content on meta tags for hOCR detection
    // Allow class attribute on all tags for hOCR detection
    let mut tag_attrs = builder.clone_tag_attributes();
    let input_attrs: HashSet<&str> = ["type", "checked"].iter().copied().collect();
    tag_attrs.insert("input", input_attrs);
    let meta_attrs: HashSet<&str> = ["name", "content"].iter().copied().collect();
    tag_attrs.insert("meta", meta_attrs);
    builder.tag_attributes(tag_attrs);

    // Add class attribute globally for hOCR support, plus SVG attributes
    let mut generic_attrs = builder.clone_generic_attributes();
    generic_attrs.insert("class");
    generic_attrs.insert("width");
    generic_attrs.insert("height");
    generic_attrs.insert("viewBox");
    generic_attrs.insert("cx");
    generic_attrs.insert("cy");
    generic_attrs.insert("r");
    generic_attrs.insert("x");
    generic_attrs.insert("y");
    generic_attrs.insert("d");
    generic_attrs.insert("fill");
    generic_attrs.insert("stroke");
    builder.generic_attributes(generic_attrs);

    builder
}

/// Create a standard sanitization builder (balanced cleaning).
fn create_standard_builder() -> Builder<'static> {
    use std::collections::HashSet;

    let mut builder = Builder::default();
    builder.strip_comments(true);

    // Allow data: URLs for inline images (base64 encoded images)
    let mut url_schemes = builder.clone_url_schemes();
    url_schemes.insert("data");
    builder.url_schemes(url_schemes);

    // Add input, meta, and SVG tags for checkbox support, hOCR detection, and inline images
    let mut tags = builder.clone_tags();
    tags.insert("input");
    tags.insert("meta");
    tags.insert("head");
    tags.insert("svg");
    tags.insert("circle");
    tags.insert("rect");
    tags.insert("path");
    tags.insert("line");
    tags.insert("polyline");
    tags.insert("polygon");
    tags.insert("ellipse");
    tags.insert("g");
    builder.tags(tags);

    // Allow type and checked attributes on input elements
    // Allow name and content on meta tags for hOCR detection
    let mut tag_attrs = builder.clone_tag_attributes();
    let input_attrs: HashSet<&str> = ["type", "checked"].iter().copied().collect();
    tag_attrs.insert("input", input_attrs);
    let meta_attrs: HashSet<&str> = ["name", "content"].iter().copied().collect();
    tag_attrs.insert("meta", meta_attrs);
    builder.tag_attributes(tag_attrs);

    // Add class attribute globally for hOCR support, plus SVG attributes
    let mut generic_attrs = builder.clone_generic_attributes();
    generic_attrs.insert("class");
    generic_attrs.insert("width");
    generic_attrs.insert("height");
    generic_attrs.insert("viewBox");
    generic_attrs.insert("cx");
    generic_attrs.insert("cy");
    generic_attrs.insert("r");
    generic_attrs.insert("x");
    generic_attrs.insert("y");
    generic_attrs.insert("d");
    generic_attrs.insert("fill");
    generic_attrs.insert("stroke");
    builder.generic_attributes(generic_attrs);

    builder
}

/// Create an aggressive sanitization builder (heavy cleaning for web scraping).
fn create_aggressive_builder() -> Builder<'static> {
    use std::collections::HashSet;

    let mut builder = Builder::default();
    builder.strip_comments(true);
    builder.link_rel(Some("nofollow noopener noreferrer"));

    // Allow data: URLs for inline images (base64 encoded images)
    let mut url_schemes = builder.clone_url_schemes();
    url_schemes.insert("data");
    builder.url_schemes(url_schemes);

    // Add input, meta, and SVG tags for checkbox support, hOCR detection, and inline images
    let mut tags = builder.clone_tags();
    tags.insert("input");
    tags.insert("meta");
    tags.insert("head");
    tags.insert("svg");
    tags.insert("circle");
    tags.insert("rect");
    tags.insert("path");
    tags.insert("line");
    tags.insert("polyline");
    tags.insert("polygon");
    tags.insert("ellipse");
    tags.insert("g");
    builder.tags(tags);

    // Allow type and checked attributes on input elements
    // Allow name and content on meta tags for hOCR detection
    let mut tag_attrs = builder.clone_tag_attributes();
    let input_attrs: HashSet<&str> = ["type", "checked"].iter().copied().collect();
    tag_attrs.insert("input", input_attrs);
    let meta_attrs: HashSet<&str> = ["name", "content"].iter().copied().collect();
    tag_attrs.insert("meta", meta_attrs);
    builder.tag_attributes(tag_attrs);

    // Add class attribute globally for hOCR support, plus SVG attributes
    let mut generic_attrs = builder.clone_generic_attributes();
    generic_attrs.insert("class");
    generic_attrs.insert("width");
    generic_attrs.insert("height");
    generic_attrs.insert("viewBox");
    generic_attrs.insert("cx");
    generic_attrs.insert("cy");
    generic_attrs.insert("r");
    generic_attrs.insert("x");
    generic_attrs.insert("y");
    generic_attrs.insert("d");
    generic_attrs.insert("fill");
    generic_attrs.insert("stroke");
    builder.generic_attributes(generic_attrs);

    builder
}
