//! JSON parsing helpers for configuration objects.
//!
//! Provides reusable functions for parsing JSON configuration strings
//! into Rust configuration types, with consistent error handling.

#[cfg(feature = "metadata")]
use html_to_markdown_rs::metadata::MetadataConfig;
use html_to_markdown_rs::{ConversionError, ConversionOptions};
#[cfg(feature = "inline-images")]
use html_to_markdown_rs::{DEFAULT_INLINE_IMAGE_LIMIT, InlineImageConfig};

/// Parse JSON string into `ConversionOptions`.
///
/// Returns `None` if the JSON string is `None` or empty.
/// Returns an error if the JSON is invalid or cannot be parsed.
///
/// # Errors
///
/// Returns `ConversionError` if JSON parsing fails.
pub fn parse_conversion_options(json: Option<&str>) -> Result<Option<ConversionOptions>, ConversionError> {
    let Some(json_str) = json else {
        return Ok(None);
    };

    if json_str.trim().is_empty() {
        return Ok(None);
    }

    let options = html_to_markdown_rs::conversion_options_from_json(json_str)?;
    Ok(Some(options))
}

/// Parse JSON string into `InlineImageConfig`.
///
/// Returns default configuration if the JSON string is `None` or empty.
/// Returns an error if the JSON is invalid or cannot be parsed.
///
/// # Errors
///
/// Returns `ConversionError` if JSON parsing fails.
#[cfg(feature = "inline-images")]
pub fn parse_inline_image_config(json: Option<&str>) -> Result<InlineImageConfig, ConversionError> {
    let Some(json_str) = json else {
        return Ok(InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    };

    if json_str.trim().is_empty() {
        return Ok(InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    }

    html_to_markdown_rs::inline_image_config_from_json(json_str)
}

/// Parse JSON string into `MetadataConfig`.
///
/// Returns default configuration if the JSON string is `None` or empty.
/// Returns an error if the JSON is invalid or cannot be parsed.
///
/// # Errors
///
/// Returns `ConversionError` if JSON parsing fails.
#[cfg(feature = "metadata")]
pub fn parse_metadata_config(json: Option<&str>) -> Result<MetadataConfig, ConversionError> {
    let Some(json_str) = json else {
        return Ok(MetadataConfig::default());
    };

    if json_str.trim().is_empty() {
        return Ok(MetadataConfig::default());
    }

    html_to_markdown_rs::metadata_config_from_json(json_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conversion_options_none() {
        let result = parse_conversion_options(None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_conversion_options_empty() {
        let result = parse_conversion_options(Some(""));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_conversion_options_whitespace() {
        let result = parse_conversion_options(Some("  \n  "));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_conversion_options_valid_json() {
        let json = r#"{"heading_style": "atx"}"#;
        let result = parse_conversion_options(Some(json));
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_parse_conversion_options_invalid_json() {
        let json = r#"{"invalid": true"#; // Missing closing brace
        let result = parse_conversion_options(Some(json));
        assert!(result.is_err());
    }

    #[cfg(feature = "inline-images")]
    #[test]
    fn test_parse_inline_image_config_none() {
        let result = parse_inline_image_config(None);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.max_decoded_size_bytes, DEFAULT_INLINE_IMAGE_LIMIT);
    }

    #[cfg(feature = "inline-images")]
    #[test]
    fn test_parse_inline_image_config_empty() {
        let result = parse_inline_image_config(Some(""));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.max_decoded_size_bytes, DEFAULT_INLINE_IMAGE_LIMIT);
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_parse_metadata_config_none() {
        let result = parse_metadata_config(None);
        assert!(result.is_ok());
        // MetadataConfig doesn't implement PartialEq, so just verify it was created successfully
        let _config = result.unwrap();
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_parse_metadata_config_empty() {
        let result = parse_metadata_config(Some(""));
        assert!(result.is_ok());
        // MetadataConfig doesn't implement PartialEq, so just verify it was created successfully
        let _config = result.unwrap();
    }
}
