#![allow(clippy::all, clippy::pedantic, clippy::nursery, missing_docs)]
#![cfg_attr(windows, feature(abi_vectorcall))]
#![deny(clippy::all)]

use std::collections::BTreeMap;
use std::convert::TryFrom;

use ext_php_rs::binary::Binary;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ArrayKey, ZendHashTable, Zval};
use html_to_markdown_bindings_common::error::error_message;
#[cfg(feature = "metadata")]
use html_to_markdown_rs::metadata::{
    DocumentMetadata, ExtendedMetadata, HeaderMetadata, ImageMetadata, LinkMetadata, MetadataConfig, StructuredData,
    TextDirection,
};
use html_to_markdown_rs::safety::guard_panic;
mod profiling;
#[cfg(feature = "visitor")]
mod visitor_support;
use html_to_markdown_rs::{
    CodeBlockStyle, ConversionError, ConversionOptions, ConversionOptionsUpdate, DEFAULT_INLINE_IMAGE_LIMIT,
    HeadingStyle, HighlightStyle, HtmlExtraction, InlineImage, InlineImageConfig, InlineImageConfigUpdate,
    InlineImageWarning, ListIndentType, MetadataConfigUpdate, NewlineStyle, PreprocessingOptionsUpdate,
    PreprocessingPreset, WhitespaceMode,
};
use std::path::PathBuf;

fn to_php_exception(err: ConversionError) -> PhpException {
    PhpException::default(error_message(&err))
}

#[php_function]
#[php(name = "html_to_markdown_convert")]
pub fn convert_html(html: String, options: Option<&ZendHashTable>, _visitor: Option<&Zval>) -> PhpResult<String> {
    let rust_options = match options {
        Some(table) => Some(parse_conversion_options(table)?),
        None => None,
    };

    // NOTE: PHP visitor support is not yet fully implemented with ext-php-rs.
    // The visitor parameter is accepted for API compatibility but is currently ignored.
    convert_with_options(&html, rust_options)
}

#[inline]
fn convert_with_options(html: &str, options: Option<ConversionOptions>) -> PhpResult<String> {
    // NOTE: The PHP binding always enables the visitor feature, so convert always takes 3 args
    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(html, options.clone())))
        .map_err(to_php_exception)
}

#[php_function]
#[php(name = "html_to_markdown_profile_start")]
pub fn profile_start(output_path: String, frequency: Option<i64>) -> PhpResult<bool> {
    let freq = frequency.unwrap_or(1000) as i32;
    profiling::start(PathBuf::from(output_path), freq).map_err(to_php_exception)?;
    Ok(true)
}

#[php_function]
#[php(name = "html_to_markdown_profile_stop")]
pub fn profile_stop() -> PhpResult<bool> {
    profiling::stop().map_err(to_php_exception)?;
    Ok(true)
}

#[php_function]
#[php(name = "html_to_markdown_convert_with_inline_images")]
pub fn convert_html_with_inline_images(
    html: String,
    options: Option<&ZendHashTable>,
    image_config: Option<&ZendHashTable>,
    _visitor: Option<&Zval>,
) -> PhpResult<ZBox<ZendHashTable>> {
    let rust_options = match options {
        Some(table) => Some(parse_conversion_options(table)?),
        None => None,
    };

    let config = match image_config {
        Some(table) => parse_inline_image_config(table)?,
        None => InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT),
    };

    // NOTE: PHP visitor support is not yet fully implemented with ext-php-rs.
    // The visitor parameter is accepted for API compatibility but is currently ignored.
    let extraction = convert_with_inline_images_impl(&html, rust_options, config)?;

    build_html_extraction(extraction)
}

#[inline]
fn convert_with_inline_images_impl(
    html: &str,
    options: Option<ConversionOptions>,
    config: InlineImageConfig,
) -> PhpResult<HtmlExtraction> {
    // NOTE: The PHP binding always enables the visitor feature
    guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, options, config, None))
        .map_err(to_php_exception)
}

#[cfg(feature = "metadata")]
#[php_function]
#[php(name = "html_to_markdown_convert_with_metadata")]
pub fn convert_html_with_metadata(
    html: String,
    options: Option<&ZendHashTable>,
    metadata_config: Option<&ZendHashTable>,
    _visitor: Option<&Zval>,
) -> PhpResult<ZBox<ZendHashTable>> {
    let rust_options = match options {
        Some(table) => Some(parse_conversion_options(table)?),
        None => None,
    };

    let config = match metadata_config {
        Some(table) => parse_metadata_config(table)?,
        None => MetadataConfig::default(),
    };

    // NOTE: PHP visitor support is not yet fully implemented with ext-php-rs.
    // The visitor parameter is accepted for API compatibility but is currently ignored.
    let (markdown, metadata) = convert_with_metadata_impl(&html, rust_options, config)?;

    build_metadata_extraction(markdown, metadata)
}

#[cfg(feature = "metadata")]
#[inline]
fn convert_with_metadata_impl(
    html: &str,
    options: Option<ConversionOptions>,
    config: MetadataConfig,
) -> PhpResult<(String, ExtendedMetadata)> {
    // NOTE: The PHP binding always enables the visitor feature
    guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, options, config, None)).map_err(to_php_exception)
}

#[cfg(feature = "visitor")]
#[php_function]
#[php(name = "html_to_markdown_convert_with_visitor")]
pub fn convert_html_with_visitor(
    html: String,
    options: Option<&ZendHashTable>,
    _visitor: Option<&Zval>,
) -> PhpResult<String> {
    let rust_options = match options {
        Some(table) => Some(parse_conversion_options(table)?),
        None => None,
    };

    // NOTE: PHP visitor support is not yet fully implemented with ext-php-rs.
    // The visitor parameter is accepted for API compatibility but is currently ignored.
    // TODO: Implement proper PHP visitor callback mechanism with ext-php-rs
    guard_panic(|| {
        profiling::maybe_profile(|| html_to_markdown_rs::convert_with_visitor(&html, rust_options.clone(), None))
    })
    .map_err(to_php_exception)
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    let mut builder = module
        .name("html_to_markdown")
        .version(env!("CARGO_PKG_VERSION"))
        .function(wrap_function!(convert_html))
        .function(wrap_function!(convert_html_with_inline_images))
        .function(wrap_function!(profile_start))
        .function(wrap_function!(profile_stop));

    #[cfg(feature = "metadata")]
    {
        builder = builder.function(wrap_function!(convert_html_with_metadata));
    }

    #[cfg(feature = "visitor")]
    {
        builder = builder.function(wrap_function!(convert_html_with_visitor));
    }

    builder
}

fn parse_conversion_options(table: &ZendHashTable) -> PhpResult<ConversionOptions> {
    let mut update = ConversionOptionsUpdate::default();

    for (key, value) in table {
        let key_str = key_to_string(&key)?;

        if value.is_null() {
            continue;
        }

        match key_str.as_str() {
            "heading_style" => {
                update.heading_style = Some(parse_heading_style(value, &key_str)?);
            }
            "list_indent_type" => {
                update.list_indent_type = Some(parse_list_indent_type(value, &key_str)?);
            }
            "list_indent_width" => {
                update.list_indent_width = Some(read_usize(value, &key_str)?);
            }
            "bullets" => {
                update.bullets = Some(read_string(value, &key_str)?);
            }
            "strong_em_symbol" => {
                update.strong_em_symbol = Some(parse_single_char(value, &key_str)?);
            }
            "escape_asterisks" => {
                update.escape_asterisks = Some(read_bool(value, &key_str)?);
            }
            "escape_underscores" => {
                update.escape_underscores = Some(read_bool(value, &key_str)?);
            }
            "escape_misc" => {
                update.escape_misc = Some(read_bool(value, &key_str)?);
            }
            "escape_ascii" => {
                update.escape_ascii = Some(read_bool(value, &key_str)?);
            }
            "code_language" => {
                update.code_language = Some(read_string(value, &key_str)?);
            }
            "autolinks" => {
                update.autolinks = Some(read_bool(value, &key_str)?);
            }
            "default_title" => {
                update.default_title = Some(read_bool(value, &key_str)?);
            }
            "br_in_tables" => {
                update.br_in_tables = Some(read_bool(value, &key_str)?);
            }
            "hocr_spatial_tables" => {
                update.hocr_spatial_tables = Some(read_bool(value, &key_str)?);
            }
            "highlight_style" => {
                update.highlight_style = Some(parse_highlight_style(value, &key_str)?);
            }
            "extract_metadata" => {
                update.extract_metadata = Some(read_bool(value, &key_str)?);
            }
            "whitespace_mode" => {
                update.whitespace_mode = Some(parse_whitespace_mode(value, &key_str)?);
            }
            "strip_newlines" => {
                update.strip_newlines = Some(read_bool(value, &key_str)?);
            }
            "wrap" => {
                update.wrap = Some(read_bool(value, &key_str)?);
            }
            "wrap_width" => {
                update.wrap_width = Some(read_usize(value, &key_str)?);
            }
            "convert_as_inline" => {
                update.convert_as_inline = Some(read_bool(value, &key_str)?);
            }
            "sub_symbol" => {
                update.sub_symbol = Some(read_string(value, &key_str)?);
            }
            "sup_symbol" => {
                update.sup_symbol = Some(read_string(value, &key_str)?);
            }
            "newline_style" => {
                update.newline_style = Some(parse_newline_style(value, &key_str)?);
            }
            "code_block_style" => {
                update.code_block_style = Some(parse_code_block_style(value, &key_str)?);
            }
            "keep_inline_images_in" => {
                update.keep_inline_images_in = Some(read_string_list(value, &key_str)?);
            }
            "preprocessing" => {
                update.preprocessing = Some(parse_preprocessing_options(value, &key_str)?);
            }
            "encoding" => {
                update.encoding = Some(read_string(value, &key_str)?);
            }
            "debug" => {
                update.debug = Some(read_bool(value, &key_str)?);
            }
            "skip_images" => {
                update.skip_images = Some(read_bool(value, &key_str)?);
            }
            "strip_tags" => {
                update.strip_tags = Some(read_string_list(value, &key_str)?);
            }
            "preserve_tags" => {
                update.preserve_tags = Some(read_string_list(value, &key_str)?);
            }
            _ => {}
        }
    }

    Ok(ConversionOptions::from(update))
}

fn parse_inline_image_config(table: &ZendHashTable) -> PhpResult<InlineImageConfig> {
    let mut update = InlineImageConfigUpdate::default();

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
                update.max_decoded_size_bytes = Some(size);
            }
            "filename_prefix" => {
                update.filename_prefix = Some(read_string(value, &key_str)?);
            }
            "capture_svg" => {
                update.capture_svg = Some(read_bool(value, &key_str)?);
            }
            "infer_dimensions" => {
                update.infer_dimensions = Some(read_bool(value, &key_str)?);
            }
            _ => {}
        }
    }

    Ok(InlineImageConfig::from_update(update))
}

#[cfg(feature = "metadata")]
fn parse_metadata_config(table: &ZendHashTable) -> PhpResult<MetadataConfig> {
    let mut update = MetadataConfigUpdate::default();

    for (key, value) in table {
        let key_str = key_to_string(&key)?;

        if value.is_null() {
            continue;
        }

        match key_str.as_str() {
            "extract_document" => {
                update.extract_document = Some(read_bool(value, &key_str)?);
            }
            "extract_headers" => {
                update.extract_headers = Some(read_bool(value, &key_str)?);
            }
            "extract_links" => {
                update.extract_links = Some(read_bool(value, &key_str)?);
            }
            "extract_images" => {
                update.extract_images = Some(read_bool(value, &key_str)?);
            }
            "extract_structured_data" => {
                update.extract_structured_data = Some(read_bool(value, &key_str)?);
            }
            "max_structured_data_size" => {
                update.max_structured_data_size = Some(read_usize(value, &key_str)?);
            }
            _ => {}
        }
    }

    Ok(MetadataConfig::from(update))
}

fn parse_preprocessing_options(value: &Zval, key: &str) -> PhpResult<PreprocessingOptionsUpdate> {
    let table = value
        .array()
        .ok_or_else(|| PhpException::default(format!("'{key}' must be an associative array")))?;

    let mut update = PreprocessingOptionsUpdate::default();

    for (entry_key, entry_value) in table {
        let entry_name = key_to_string(&entry_key)?;

        if entry_value.is_null() {
            continue;
        }

        match entry_name.as_str() {
            "enabled" => {
                update.enabled = Some(read_bool(entry_value, &format!("{key}.enabled"))?);
            }
            "preset" => {
                update.preset = Some(parse_preprocessing_preset(entry_value, &format!("{key}.preset"))?);
            }
            "remove_navigation" => {
                update.remove_navigation = Some(read_bool(entry_value, &format!("{key}.remove_navigation"))?);
            }
            "remove_forms" => {
                update.remove_forms = Some(read_bool(entry_value, &format!("{key}.remove_forms"))?);
            }
            _ => {}
        }
    }

    Ok(update)
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
    entry.insert("format", image.format.to_string())?;

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
            dims.push(i64::from(width))?;
            dims.push(i64::from(height))?;
            entry.insert("dimensions", dims)?;
        }
        None => entry.insert("dimensions", ())?,
    }

    entry.insert("source", image.source.to_string())?;
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

fn key_to_string(key: &ArrayKey<'_>) -> PhpResult<String> {
    String::try_from(key.clone())
        .map_err(|_| PhpException::default("Option keys must be representable as strings".to_string()))
}

fn table_capacity(len: usize) -> u32 {
    len.min(u32::MAX as usize) as u32
}

#[cfg(feature = "metadata")]
fn build_metadata_extraction(markdown: String, metadata: ExtendedMetadata) -> PhpResult<ZBox<ZendHashTable>> {
    let mut result = ZendHashTable::new();
    result.insert("markdown", markdown)?;
    result.insert("metadata", build_extended_metadata(metadata)?)?;
    Ok(result)
}

#[cfg(feature = "metadata")]
fn build_extended_metadata(metadata: ExtendedMetadata) -> PhpResult<ZBox<ZendHashTable>> {
    let mut table = ZendHashTable::new();
    table.insert("document", build_document_metadata(metadata.document)?)?;
    table.insert("headers", build_headers_array(metadata.headers)?)?;
    table.insert("links", build_links_array(metadata.links)?)?;
    table.insert("images", build_images_array(metadata.images)?)?;
    table.insert(
        "structured_data",
        build_structured_data_array(metadata.structured_data)?,
    )?;
    Ok(table)
}

#[cfg(feature = "metadata")]
fn build_document_metadata(doc: DocumentMetadata) -> PhpResult<ZBox<ZendHashTable>> {
    let mut table = ZendHashTable::new();

    match doc.title {
        Some(title) => table.insert("title", title)?,
        None => table.insert("title", ())?,
    }

    match doc.description {
        Some(description) => table.insert("description", description)?,
        None => table.insert("description", ())?,
    }

    table.insert("keywords", doc.keywords)?;

    match doc.author {
        Some(author) => table.insert("author", author)?,
        None => table.insert("author", ())?,
    }

    match doc.canonical_url {
        Some(url) => table.insert("canonical_url", url)?,
        None => table.insert("canonical_url", ())?,
    }

    match doc.base_href {
        Some(href) => table.insert("base_href", href)?,
        None => table.insert("base_href", ())?,
    }

    match doc.language {
        Some(lang) => table.insert("language", lang)?,
        None => table.insert("language", ())?,
    }

    table.insert("text_direction", text_direction_to_string(doc.text_direction))?;
    table.insert("open_graph", build_string_map(doc.open_graph)?)?;
    table.insert("twitter_card", build_string_map(doc.twitter_card)?)?;
    table.insert("meta_tags", build_string_map(doc.meta_tags)?)?;

    Ok(table)
}

#[cfg(feature = "metadata")]
fn build_headers_array(headers: Vec<HeaderMetadata>) -> PhpResult<ZBox<ZendHashTable>> {
    let mut array = ZendHashTable::with_capacity(table_capacity(headers.len()));

    for header in headers {
        let mut entry = ZendHashTable::new();
        entry.insert("level", i64::from(header.level))?;
        entry.insert("text", header.text)?;

        match header.id {
            Some(id) => entry.insert("id", id)?,
            None => entry.insert("id", ())?,
        }

        entry.insert("depth", header.depth as i64)?;
        entry.insert("html_offset", header.html_offset as i64)?;

        array.push(entry)?;
    }

    Ok(array)
}

#[cfg(feature = "metadata")]
fn build_links_array(links: Vec<LinkMetadata>) -> PhpResult<ZBox<ZendHashTable>> {
    let mut array = ZendHashTable::with_capacity(table_capacity(links.len()));

    for link in links {
        let mut entry = ZendHashTable::new();
        entry.insert("href", link.href)?;
        entry.insert("text", link.text)?;

        match link.title {
            Some(title) => entry.insert("title", title)?,
            None => entry.insert("title", ())?,
        }

        entry.insert("link_type", link.link_type.to_string())?;
        entry.insert("rel", link.rel)?;
        entry.insert("attributes", build_string_map(link.attributes)?)?;

        array.push(entry)?;
    }

    Ok(array)
}

#[cfg(feature = "metadata")]
fn build_images_array(images: Vec<ImageMetadata>) -> PhpResult<ZBox<ZendHashTable>> {
    let mut array = ZendHashTable::with_capacity(table_capacity(images.len()));

    for image in images {
        let mut entry = ZendHashTable::new();
        entry.insert("src", image.src)?;

        match image.alt {
            Some(alt) => entry.insert("alt", alt)?,
            None => entry.insert("alt", ())?,
        }

        match image.title {
            Some(title) => entry.insert("title", title)?,
            None => entry.insert("title", ())?,
        }

        match image.dimensions {
            Some((width, height)) => {
                let mut dims = ZendHashTable::with_capacity(2);
                dims.push(i64::from(width))?;
                dims.push(i64::from(height))?;
                entry.insert("dimensions", dims)?;
            }
            None => entry.insert("dimensions", ())?,
        }

        entry.insert("image_type", image.image_type.to_string())?;
        entry.insert("attributes", build_string_map(image.attributes)?)?;

        array.push(entry)?;
    }

    Ok(array)
}

#[cfg(feature = "metadata")]
fn build_structured_data_array(data: Vec<StructuredData>) -> PhpResult<ZBox<ZendHashTable>> {
    let mut array = ZendHashTable::with_capacity(table_capacity(data.len()));

    for item in data {
        let mut entry = ZendHashTable::new();
        entry.insert("data_type", item.data_type.to_string())?;
        entry.insert("raw_json", item.raw_json)?;

        match item.schema_type {
            Some(schema_type) => entry.insert("schema_type", schema_type)?,
            None => entry.insert("schema_type", ())?,
        }

        array.push(entry)?;
    }

    Ok(array)
}

#[cfg(feature = "metadata")]
fn build_string_map(map: BTreeMap<String, String>) -> PhpResult<ZBox<ZendHashTable>> {
    let mut table = ZendHashTable::with_capacity(table_capacity(map.len()));

    for (key, value) in map {
        table.insert(key, value)?;
    }

    Ok(table)
}

#[cfg(feature = "metadata")]
fn text_direction_to_string(direction: Option<TextDirection>) -> String {
    direction.map(|dir| dir.to_string()).unwrap_or_default()
}
