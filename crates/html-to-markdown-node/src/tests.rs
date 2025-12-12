#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_style_conversion() {
        let atx: HeadingStyle = JsHeadingStyle::Atx.into();
        assert!(matches!(atx, HeadingStyle::Atx));
    }

    #[test]
    fn test_conversion_options_defaults() {
        let opts = JsConversionOptions {
            heading_style: Some(JsHeadingStyle::Atx),
            list_indent_type: None,
            list_indent_width: None,
            bullets: None,
            strong_em_symbol: None,
            escape_asterisks: None,
            escape_underscores: None,
            escape_misc: None,
            escape_ascii: None,
            code_language: None,
            autolinks: None,
            default_title: None,
            br_in_tables: None,
            hocr_spatial_tables: None,
            highlight_style: None,
            extract_metadata: None,
            whitespace_mode: None,
            strip_newlines: None,
            wrap: None,
            wrap_width: None,
            convert_as_inline: None,
            sub_symbol: None,
            sup_symbol: None,
            newline_style: None,
            code_block_style: None,
            keep_inline_images_in: None,
            preprocessing: None,
            encoding: None,
            debug: None,
            strip_tags: None,
        };

        let rust_opts: RustConversionOptions = opts.into();
        assert!(matches!(rust_opts.heading_style, HeadingStyle::Atx));
        assert_eq!(rust_opts.list_indent_width, 2);
    }

    #[test]
    fn test_preprocessing_options() {
        let opts = JsPreprocessingOptions {
            enabled: Some(true),
            preset: Some(JsPreprocessingPreset::Aggressive),
            remove_navigation: Some(false),
            remove_forms: Some(true),
        };

        let rust_opts: RustPreprocessingOptions = opts.into();
        assert!(rust_opts.enabled);
        assert!(matches!(rust_opts.preset, PreprocessingPreset::Aggressive));
        assert!(!rust_opts.remove_navigation);
        assert!(rust_opts.remove_forms);
    }

    #[test]
    fn test_inline_image_config() {
        let config = JsInlineImageConfig {
            max_decoded_size_bytes: Some(BigInt::from(1024u64)),
            filename_prefix: Some("test_".to_string()),
            capture_svg: Some(false),
            infer_dimensions: Some(true),
        };

        let rust_config: RustInlineImageConfig = config.into();
        assert!(!rust_config.capture_svg);
        assert!(rust_config.infer_dimensions);
        assert_eq!(rust_config.filename_prefix, Some("test_".to_string()));
    }
}
