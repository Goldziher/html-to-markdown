//! HTML sanitization using ammonia.

use ammonia::Builder;

use crate::error::Result;
use crate::options::{PreprocessingOptions, PreprocessingPreset};

/// Sanitize HTML using ammonia.
///
/// This function cleans HTML by removing unwanted elements and attributes
/// based on the preprocessing options.
pub fn sanitize(html: &str, options: &PreprocessingOptions) -> Result<String> {
    use std::collections::HashSet;

    let mut builder = match options.preset {
        PreprocessingPreset::Minimal => create_minimal_builder(),
        PreprocessingPreset::Standard => create_standard_builder(),
        PreprocessingPreset::Aggressive => create_aggressive_builder(),
    };

    let mut clean_content = HashSet::new();
    let mut allowed_tags = builder.clone_tags();

    clean_content.insert("script");
    clean_content.insert("style");
    allowed_tags.remove("script");
    allowed_tags.remove("style");

    if options.remove_navigation {
        clean_content.insert("nav");
        clean_content.insert("aside");
        clean_content.insert("header");
        clean_content.insert("footer");
        allowed_tags.remove("nav");
        allowed_tags.remove("aside");
        allowed_tags.remove("header");
        allowed_tags.remove("footer");
    }

    if options.remove_forms {
        clean_content.insert("form");
        clean_content.insert("input");
        clean_content.insert("button");
        clean_content.insert("select");
        clean_content.insert("textarea");
        clean_content.insert("label");
        clean_content.insert("fieldset");
        clean_content.insert("legend");
        allowed_tags.remove("form");
        allowed_tags.remove("input");
        allowed_tags.remove("button");
        allowed_tags.remove("select");
        allowed_tags.remove("textarea");
        allowed_tags.remove("label");
        allowed_tags.remove("fieldset");
        allowed_tags.remove("legend");
    }

    builder.tags(allowed_tags);
    builder.clean_content_tags(clean_content);

    Ok(builder.clean(html).to_string())
}

/// Create a minimal sanitization builder (keeps most elements).
fn create_minimal_builder() -> Builder<'static> {
    let mut builder = Builder::default();
    builder.strip_comments(false);
    builder
}

/// Create a standard sanitization builder (balanced cleaning).
fn create_standard_builder() -> Builder<'static> {
    let mut builder = Builder::default();
    builder.strip_comments(true);
    builder
}

/// Create an aggressive sanitization builder (heavy cleaning for web scraping).
fn create_aggressive_builder() -> Builder<'static> {
    let mut builder = Builder::default();
    builder.strip_comments(true);
    builder.link_rel(Some("nofollow noopener noreferrer"));
    builder
}
