//! HTML sanitization using ammonia.

use ammonia::Builder;

use crate::error::Result;
use crate::options::{PreprocessingOptions, PreprocessingPreset};

/// Sanitize HTML using ammonia.
///
/// This function cleans HTML by removing unwanted elements and attributes
/// based on the preprocessing options.
pub fn sanitize(html: &str, options: &PreprocessingOptions) -> Result<String> {
    let builder = match options.preset {
        PreprocessingPreset::Minimal => create_minimal_builder(),
        PreprocessingPreset::Standard => create_standard_builder(),
        PreprocessingPreset::Aggressive => create_aggressive_builder(),
    };

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
