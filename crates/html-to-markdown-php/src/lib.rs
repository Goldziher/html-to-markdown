#![cfg_attr(windows, feature(abi_vectorcall))]
#![deny(clippy::all)]

use std::collections::BTreeMap;
use std::convert::TryFrom;

use ext_php_rs::binary::Binary;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ArrayKey, ZendHashTable, Zval};
use html_to_markdown_rs::{
    convert, convert_with_inline_images, CodeBlockStyle, ConversionOptions, HeadingStyle, HighlightStyle,
    HtmlExtraction, InlineImage, InlineImageConfig, InlineImageFormat, InlineImageSource, InlineImageWarning,
    ListIndentType, NewlineStyle, PreprocessingOptions, PreprocessingPreset, WhitespaceMode,
};

const DEFAULT_INLINE_IMAGE_LIMIT: u64 = 5 * 1024 * 1024;

#[php_function]
#[php(name = "html_to_markdown_convert")]
pub fn convert_html(html: String, options: Option<&ZendHashTable>) -> PhpResult<String> {
    let rust_options = match options {
        Some(table) => Some(parse_conversion_options(table)?),
        None => None,
    };

    convert(&html, rust_options).map_err(|err| PhpException::default(err.to_string()))
}

#[php_function]
#[php(name = "html_to_markdown_convert_with_inline_images")]
pub fn convert_html_with_inline_images(
    html: String,
    options: Option<&ZendHashTable>,
    image_config: Option<&ZendHashTable>,
) -> PhpResult<ZBox<ZendHashTable>> {
    let rust_options = match options {
        Some(table) => Some(parse_conversion_options(table)?),
        None => None,
    };

    let config = match image_config {
        Some(table) => parse_inline_image_config(table)?,
        None => InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT),
    };

    let extraction = convert_with_inline_images(&html, rust_options, config)
        .map_err(|err| PhpException::default(err.to_string()))?;

    build_html_extraction(extraction)
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .name("html_to_markdown")
        .version(env!("CARGO_PKG_VERSION"))
        .function(wrap_function!(convert_html))
        .function(wrap_function!(convert_html_with_inline_images))
}

fn parse_conversion_options(table: &ZendHashTable) -> PhpResult<ConversionOptions> {
    let mut opts = ConversionOptions::default();

    for (key, value) in table {
        let key_str = key_to_string(&key)?;

        if value.is_null() {
            continue;
        }

        match key_str.as_str() {
            "heading_style" => {
                opts.heading_style = parse_heading_style(value, &key_str)?;
            }
            "list_indent_type" => {
                opts.list_indent_type = parse_list_indent_type(value, &key_str)?;
            }
            "list_indent_width" => {
                opts.list_indent_width = read_usize(value, &key_str)?;
            }
            "bullets" => {
                opts.bullets = read_string(value, &key_str)?;
            }
            "strong_em_symbol" => {
                opts.strong_em_symbol = parse_single_char(value, &key_str)?;
            }
            "escape_asterisks" => {
                opts.escape_asterisks = read_bool(value, &key_str)?;
            }
            "escape_underscores" => {
                opts.escape_underscores = read_bool(value, &key_str)?;
            }
            "escape_misc" => {
                opts.escape_misc = read_bool(value, &key_str)?;
            }
            "escape_ascii" => {
                opts.escape_ascii = read_bool(value, &key_str)?;
            }
            "code_language" => {
                opts.code_language = read_string(value, &key_str)?;
            }
            "autolinks" => {
                opts.autolinks = read_bool(value, &key_str)?;
            }
            "default_title" => {
                opts.default_title = read_bool(value, &key_str)?;
            }
            "br_in_tables" => {
                opts.br_in_tables = read_bool(value, &key_str)?;
            }
            "hocr_spatial_tables" => {
                opts.hocr_spatial_tables = read_bool(value, &key_str)?;
            }
            "highlight_style" => {
                opts.highlight_style = parse_highlight_style(value, &key_str)?;
            }
            "extract_metadata" => {
                opts.extract_metadata = read_bool(value, &key_str)?;
            }
            "whitespace_mode" => {
                opts.whitespace_mode = parse_whitespace_mode(value, &key_str)?;
            }
            "strip_newlines" => {
                opts.strip_newlines = read_bool(value, &key_str)?;
            }
            "wrap" => {
                opts.wrap = read_bool(value, &key_str)?;
            }
            "wrap_width" => {
                opts.wrap_width = read_usize(value, &key_str)?;
            }
            "convert_as_inline" => {
                opts.convert_as_inline = read_bool(value, &key_str)?;
            }
            "sub_symbol" => {
                opts.sub_symbol = read_string(value, &key_str)?;
            }
            "sup_symbol" => {
                opts.sup_symbol = read_string(value, &key_str)?;
            }
            "newline_style" => {
                opts.newline_style = parse_newline_style(value, &key_str)?;
            }
            "code_block_style" => {
                opts.code_block_style = parse_code_block_style(value, &key_str)?;
            }
            "keep_inline_images_in" => {
                opts.keep_inline_images_in = read_string_list(value, &key_str)?;
            }
            "preprocessing" => {
                opts.preprocessing = parse_preprocessing_options(value, &key_str)?;
            }
            "encoding" => {
                opts.encoding = read_string(value, &key_str)?;
            }
            "debug" => {
                opts.debug = read_bool(value, &key_str)?;
            }
            "strip_tags" => {
                opts.strip_tags = read_string_list(value, &key_str)?;
            }
            "preserve_tags" => {
                opts.preserve_tags = read_string_list(value, &key_str)?;
            }
            _ => {}
        }
    }

    Ok(opts)
}

fn parse_inline_image_config(table: &ZendHashTable) -> PhpResult<InlineImageConfig> {
    let mut config = InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT);

    for (key, value) in table {
        let key_str = key_to_string(&key)?;

        if value.is_null() {
            continue;
        }

        match key_str.as_str() {
            "max_decoded_size_bytes" => {
                let size = read_u64(value, &key_str)?;
                if size == 0 {
                    return Err(PhpException::default(
                        "max_decoded_size_bytes must be greater than zero".to_string(),
                    ));
                }
                config.max_decoded_size_bytes = size;
            }
            "filename_prefix" => {
                config.filename_prefix = Some(read_string(value, &key_str)?);
            }
            "capture_svg" => {
                config.capture_svg = read_bool(value, &key_str)?;
            }
            "infer_dimensions" => {
                config.infer_dimensions = read_bool(value, &key_str)?;
            }
            _ => {}
        }
    }

    Ok(config)
}

fn parse_preprocessing_options(value: &Zval, key: &str) -> PhpResult<PreprocessingOptions> {
    let table = value
        .array()
        .ok_or_else(|| PhpException::default(format!("'{key}' must be an associative array")))?;

    let mut opts = PreprocessingOptions::default();

    for (entry_key, entry_value) in table {
        let entry_name = key_to_string(&entry_key)?;

        if entry_value.is_null() {
            continue;
        }

        match entry_name.as_str() {
            "enabled" => {
                opts.enabled = read_bool(entry_value, &format!("{key}.enabled"))?;
            }
            "preset" => {
                opts.preset = parse_preprocessing_preset(entry_value, &format!("{key}.preset"))?;
            }
            "remove_navigation" => {
                opts.remove_navigation = read_bool(entry_value, &format!("{key}.remove_navigation"))?;
            }
            "remove_forms" => {
                opts.remove_forms = read_bool(entry_value, &format!("{key}.remove_forms"))?;
            }
            _ => {}
        }
    }

    Ok(opts)
}

fn parse_heading_style(value: &Zval, key: &str) -> PhpResult<HeadingStyle> {
    match read_string(value, key)?.as_str() {
        "underlined" => Ok(HeadingStyle::Underlined),
        "atx" => Ok(HeadingStyle::Atx),
        "atx_closed" => Ok(HeadingStyle::AtxClosed),
        other => Err(PhpException::default(format!("Invalid heading_style '{other}'"))),
    }
}

fn parse_list_indent_type(value: &Zval, key: &str) -> PhpResult<ListIndentType> {
    match read_string(value, key)?.as_str() {
        "spaces" => Ok(ListIndentType::Spaces),
        "tabs" => Ok(ListIndentType::Tabs),
        other => Err(PhpException::default(format!("Invalid list_indent_type '{other}'"))),
    }
}

fn parse_highlight_style(value: &Zval, key: &str) -> PhpResult<HighlightStyle> {
    match read_string(value, key)?.as_str() {
        "double_equal" => Ok(HighlightStyle::DoubleEqual),
        "html" => Ok(HighlightStyle::Html),
        "bold" => Ok(HighlightStyle::Bold),
        "none" => Ok(HighlightStyle::None),
        other => Err(PhpException::default(format!("Invalid highlight_style '{other}'"))),
    }
}

fn parse_whitespace_mode(value: &Zval, key: &str) -> PhpResult<WhitespaceMode> {
    match read_string(value, key)?.as_str() {
        "normalized" => Ok(WhitespaceMode::Normalized),
        "strict" => Ok(WhitespaceMode::Strict),
        other => Err(PhpException::default(format!("Invalid whitespace_mode '{other}'"))),
    }
}

fn parse_newline_style(value: &Zval, key: &str) -> PhpResult<NewlineStyle> {
    match read_string(value, key)?.as_str() {
        "spaces" => Ok(NewlineStyle::Spaces),
        "backslash" => Ok(NewlineStyle::Backslash),
        other => Err(PhpException::default(format!("Invalid newline_style '{other}'"))),
    }
}

fn parse_code_block_style(value: &Zval, key: &str) -> PhpResult<CodeBlockStyle> {
    match read_string(value, key)?.as_str() {
        "indented" => Ok(CodeBlockStyle::Indented),
        "backticks" => Ok(CodeBlockStyle::Backticks),
        "tildes" => Ok(CodeBlockStyle::Tildes),
        other => Err(PhpException::default(format!("Invalid code_block_style '{other}'"))),
    }
}

fn parse_preprocessing_preset(value: &Zval, key: &str) -> PhpResult<PreprocessingPreset> {
    match read_string(value, key)?.as_str() {
        "minimal" => Ok(PreprocessingPreset::Minimal),
        "standard" => Ok(PreprocessingPreset::Standard),
        "aggressive" => Ok(PreprocessingPreset::Aggressive),
        other => Err(PhpException::default(format!("Invalid preprocessing preset '{other}'"))),
    }
}

fn read_bool(value: &Zval, key: &str) -> PhpResult<bool> {
    value
        .bool()
        .ok_or_else(|| PhpException::default(format!("'{key}' must be a boolean (got {:?})", value.get_type())))
}

fn read_string(value: &Zval, key: &str) -> PhpResult<String> {
    value
        .string()
        .ok_or_else(|| PhpException::default(format!("'{key}' must be a string (got {:?})", value.get_type())))
}

fn read_usize(value: &Zval, key: &str) -> PhpResult<usize> {
    let number = value
        .long()
        .ok_or_else(|| PhpException::default(format!("'{key}' must be an integer (got {:?})", value.get_type())))?;
    if number < 0 {
        return Err(PhpException::default(format!("'{key}' must be a non-negative integer")));
    }
    Ok(number as usize)
}

fn read_u64(value: &Zval, key: &str) -> PhpResult<u64> {
    let number = value
        .long()
        .ok_or_else(|| PhpException::default(format!("'{key}' must be an integer (got {:?})", value.get_type())))?;

    if number < 0 {
        return Err(PhpException::default(format!("'{key}' must be a non-negative integer")));
    }

    Ok(number as u64)
}

fn read_string_list(value: &Zval, key: &str) -> PhpResult<Vec<String>> {
    let array = value
        .array()
        .ok_or_else(|| PhpException::default(format!("'{key}' must be an array of strings")))?;

    let mut result = Vec::with_capacity(array.len());

    for (_, element) in array {
        if element.is_null() {
            continue;
        }

        result.push(read_string(element, key)?);
    }

    Ok(result)
}

fn parse_single_char(value: &Zval, key: &str) -> PhpResult<char> {
    let string = read_string(value, key)?;
    let mut chars = string.chars();
    if let Some(ch) = chars.next() {
        if chars.next().is_some() {
            Err(PhpException::default(format!("'{key}' must be a single character")))
        } else {
            Ok(ch)
        }
    } else {
        Err(PhpException::default(format!("'{key}' must not be empty")))
    }
}

fn build_html_extraction(extraction: HtmlExtraction) -> PhpResult<ZBox<ZendHashTable>> {
    let mut result = ZendHashTable::new();
    result.insert("markdown", extraction.markdown)?;
    result.insert("inline_images", build_inline_images(extraction.inline_images)?)?;
    result.insert("warnings", build_warnings(extraction.warnings)?)?;
    Ok(result)
}

fn build_inline_images(images: Vec<InlineImage>) -> PhpResult<ZBox<ZendHashTable>> {
    let mut table = ZendHashTable::with_capacity(table_capacity(images.len()));

    for image in images {
        table.push(build_inline_image_entry(image)?)?;
    }

    Ok(table)
}

fn build_inline_image_entry(image: InlineImage) -> PhpResult<ZBox<ZendHashTable>> {
    let mut entry = ZendHashTable::new();
    entry.insert("data", Binary::from(image.data))?;
    entry.insert("format", inline_image_format_to_string(&image.format))?;

    match image.filename {
        Some(filename) => entry.insert("filename", filename)?,
        None => entry.insert("filename", ())?,
    }
    match image.description {
        Some(description) => entry.insert("description", description)?,
        None => entry.insert("description", ())?,
    }

    match image.dimensions {
        Some((width, height)) => {
            let mut dims = ZendHashTable::with_capacity(2);
            dims.push(width as i64)?;
            dims.push(height as i64)?;
            entry.insert("dimensions", dims)?;
        }
        None => entry.insert("dimensions", ())?,
    }

    entry.insert("source", inline_image_source_to_string(&image.source))?;
    entry.insert("attributes", build_attribute_table(image.attributes)?)?;

    Ok(entry)
}

fn build_attribute_table(attributes: BTreeMap<String, String>) -> PhpResult<ZBox<ZendHashTable>> {
    let mut table = ZendHashTable::with_capacity(table_capacity(attributes.len()));

    for (key, value) in attributes {
        table.insert(key, value)?;
    }

    Ok(table)
}

fn build_warnings(warnings: Vec<InlineImageWarning>) -> PhpResult<ZBox<ZendHashTable>> {
    let mut table = ZendHashTable::with_capacity(table_capacity(warnings.len()));

    for warning in warnings {
        let mut entry = ZendHashTable::new();
        entry.insert("index", warning.index as i64)?;
        entry.insert("message", warning.message)?;
        table.push(entry)?;
    }

    Ok(table)
}

fn inline_image_format_to_string(format: &InlineImageFormat) -> String {
    match format {
        InlineImageFormat::Png => "png".to_string(),
        InlineImageFormat::Jpeg => "jpeg".to_string(),
        InlineImageFormat::Gif => "gif".to_string(),
        InlineImageFormat::Bmp => "bmp".to_string(),
        InlineImageFormat::Webp => "webp".to_string(),
        InlineImageFormat::Svg => "svg".to_string(),
        InlineImageFormat::Other(other) => other.clone(),
    }
}

fn inline_image_source_to_string(source: &InlineImageSource) -> &'static str {
    match source {
        InlineImageSource::ImgDataUri => "img_data_uri",
        InlineImageSource::SvgElement => "svg_element",
    }
}

fn key_to_string(key: &ArrayKey<'_>) -> PhpResult<String> {
    String::try_from(key.clone())
        .map_err(|_| PhpException::default("Option keys must be representable as strings".to_string()))
}

fn table_capacity(len: usize) -> u32 {
    len.min(u32::MAX as usize) as u32
}
