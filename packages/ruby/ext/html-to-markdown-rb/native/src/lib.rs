use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions, ConversionOptionsUpdate, DEFAULT_INLINE_IMAGE_LIMIT, HeadingStyle,
    HighlightStyle, HtmlExtraction, InlineImage, InlineImageConfig, InlineImageConfigUpdate, InlineImageWarning,
    ListIndentType, NewlineStyle, PreprocessingOptionsUpdate, PreprocessingPreset, WhitespaceMode,
    convert as convert_inner, convert_with_inline_images as convert_with_inline_images_inner, error::ConversionError,
    safety::guard_panic,
};

#[cfg(feature = "metadata")]
use html_to_markdown_rs::convert_with_metadata as convert_with_metadata_inner;
mod profiling;
#[cfg(feature = "metadata")]
use html_to_markdown_rs::metadata::{
    DocumentMetadata as RustDocumentMetadata, ExtendedMetadata as RustExtendedMetadata,
    HeaderMetadata as RustHeaderMetadata, ImageMetadata as RustImageMetadata, LinkMetadata as RustLinkMetadata,
    MetadataConfig as RustMetadataConfig, StructuredData as RustStructuredData, TextDirection as RustTextDirection,
};
use magnus::prelude::*;
use magnus::r_hash::ForEach;
use magnus::{Error, RArray, RHash, Ruby, Symbol, TryConvert, Value, function, scan_args::scan_args};
use std::path::PathBuf;

#[derive(Clone)]
#[magnus::wrap(class = "HtmlToMarkdown::Options", free_immediately)]
struct OptionsHandle(ConversionOptions);

fn conversion_error(err: ConversionError) -> Error {
    match err {
        ConversionError::ConfigError(msg) => arg_error(msg),
        ConversionError::Panic(message) => {
            runtime_error(format!("html-to-markdown panic during conversion: {message}"))
        }
        other => runtime_error(other.to_string()),
    }
}

fn arg_error(message: impl Into<String>) -> Error {
    let ruby = Ruby::get().expect("Ruby not initialised");
    Error::new(ruby.exception_arg_error(), message.into())
}

fn runtime_error(message: impl Into<String>) -> Error {
    let ruby = Ruby::get().expect("Ruby not initialised");
    Error::new(ruby.exception_runtime_error(), message.into())
}

fn symbol_to_string(value: Value) -> Result<String, Error> {
    if let Some(symbol) = Symbol::from_value(value) {
        Ok(symbol.name()?.to_string())
    } else {
        String::try_convert(value)
    }
}

fn parse_heading_style(value: Value) -> Result<HeadingStyle, Error> {
    match symbol_to_string(value)?.as_str() {
        "underlined" => Ok(HeadingStyle::Underlined),
        "atx" => Ok(HeadingStyle::Atx),
        "atx_closed" => Ok(HeadingStyle::AtxClosed),
        other => Err(arg_error(format!("invalid heading_style: {other}"))),
    }
}

fn parse_list_indent_type(value: Value) -> Result<ListIndentType, Error> {
    match symbol_to_string(value)?.as_str() {
        "spaces" => Ok(ListIndentType::Spaces),
        "tabs" => Ok(ListIndentType::Tabs),
        other => Err(arg_error(format!("invalid list_indent_type: {other}"))),
    }
}

fn parse_highlight_style(value: Value) -> Result<HighlightStyle, Error> {
    match symbol_to_string(value)?.as_str() {
        "double_equal" => Ok(HighlightStyle::DoubleEqual),
        "html" => Ok(HighlightStyle::Html),
        "bold" => Ok(HighlightStyle::Bold),
        "none" => Ok(HighlightStyle::None),
        other => Err(arg_error(format!("invalid highlight_style: {other}"))),
    }
}

fn parse_whitespace_mode(value: Value) -> Result<WhitespaceMode, Error> {
    match symbol_to_string(value)?.as_str() {
        "normalized" => Ok(WhitespaceMode::Normalized),
        "strict" => Ok(WhitespaceMode::Strict),
        other => Err(arg_error(format!("invalid whitespace_mode: {other}"))),
    }
}

fn parse_newline_style(value: Value) -> Result<NewlineStyle, Error> {
    match symbol_to_string(value)?.as_str() {
        "spaces" => Ok(NewlineStyle::Spaces),
        "backslash" => Ok(NewlineStyle::Backslash),
        other => Err(arg_error(format!("invalid newline_style: {other}"))),
    }
}

fn parse_code_block_style(value: Value) -> Result<CodeBlockStyle, Error> {
    match symbol_to_string(value)?.as_str() {
        "indented" => Ok(CodeBlockStyle::Indented),
        "backticks" => Ok(CodeBlockStyle::Backticks),
        "tildes" => Ok(CodeBlockStyle::Tildes),
        other => Err(arg_error(format!("invalid code_block_style: {other}"))),
    }
}

fn parse_preset(value: Value) -> Result<PreprocessingPreset, Error> {
    match symbol_to_string(value)?.as_str() {
        "minimal" => Ok(PreprocessingPreset::Minimal),
        "standard" => Ok(PreprocessingPreset::Standard),
        "aggressive" => Ok(PreprocessingPreset::Aggressive),
        other => Err(arg_error(format!("invalid preprocessing preset: {other}"))),
    }
}

fn parse_vec_of_strings(value: Value) -> Result<Vec<String>, Error> {
    let array = RArray::from_value(value).ok_or_else(|| arg_error("expected an Array of strings"))?;

    array.to_vec::<String>()
}

fn parse_preprocessing_options(_ruby: &Ruby, value: Value) -> Result<PreprocessingOptionsUpdate, Error> {
    let hash = RHash::from_value(value).ok_or_else(|| arg_error("expected preprocessing to be a Hash"))?;

    let mut update = PreprocessingOptionsUpdate::default();

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "enabled" => {
                update.enabled = Some(bool::try_convert(val)?);
            }
            "preset" => {
                update.preset = Some(parse_preset(val)?);
            }
            "remove_navigation" => {
                update.remove_navigation = Some(bool::try_convert(val)?);
            }
            "remove_forms" => {
                update.remove_forms = Some(bool::try_convert(val)?);
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(update)
}

fn build_conversion_options(ruby: &Ruby, options: Option<Value>) -> Result<ConversionOptions, Error> {
    let mut update = ConversionOptionsUpdate::default();

    let Some(options) = options else {
        return Ok(ConversionOptions::default());
    };

    if options.is_nil() {
        return Ok(ConversionOptions::default());
    }

    let hash = RHash::from_value(options).ok_or_else(|| arg_error("options must be provided as a Hash"))?;

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "heading_style" => {
                update.heading_style = Some(parse_heading_style(val)?);
            }
            "list_indent_type" => {
                update.list_indent_type = Some(parse_list_indent_type(val)?);
            }
            "list_indent_width" => {
                update.list_indent_width = Some(usize::try_convert(val)?);
            }
            "bullets" => {
                update.bullets = Some(String::try_convert(val)?);
            }
            "strong_em_symbol" => {
                let value = String::try_convert(val)?;
                let mut chars = value.chars();
                let ch = chars
                    .next()
                    .ok_or_else(|| arg_error("strong_em_symbol must not be empty"))?;
                if chars.next().is_some() {
                    return Err(arg_error("strong_em_symbol must be a single character"));
                }
                update.strong_em_symbol = Some(ch);
            }
            "escape_asterisks" => {
                update.escape_asterisks = Some(bool::try_convert(val)?);
            }
            "escape_underscores" => {
                update.escape_underscores = Some(bool::try_convert(val)?);
            }
            "escape_misc" => {
                update.escape_misc = Some(bool::try_convert(val)?);
            }
            "escape_ascii" => {
                update.escape_ascii = Some(bool::try_convert(val)?);
            }
            "code_language" => {
                update.code_language = Some(String::try_convert(val)?);
            }
            "autolinks" => {
                update.autolinks = Some(bool::try_convert(val)?);
            }
            "default_title" => {
                update.default_title = Some(bool::try_convert(val)?);
            }
            "br_in_tables" => {
                update.br_in_tables = Some(bool::try_convert(val)?);
            }
            "hocr_spatial_tables" => {
                update.hocr_spatial_tables = Some(bool::try_convert(val)?);
            }
            "highlight_style" => {
                update.highlight_style = Some(parse_highlight_style(val)?);
            }
            "extract_metadata" => {
                update.extract_metadata = Some(bool::try_convert(val)?);
            }
            "whitespace_mode" => {
                update.whitespace_mode = Some(parse_whitespace_mode(val)?);
            }
            "strip_newlines" => {
                update.strip_newlines = Some(bool::try_convert(val)?);
            }
            "wrap" => {
                update.wrap = Some(bool::try_convert(val)?);
            }
            "wrap_width" => {
                update.wrap_width = Some(usize::try_convert(val)?);
            }
            "convert_as_inline" => {
                update.convert_as_inline = Some(bool::try_convert(val)?);
            }
            "sub_symbol" => {
                update.sub_symbol = Some(String::try_convert(val)?);
            }
            "sup_symbol" => {
                update.sup_symbol = Some(String::try_convert(val)?);
            }
            "newline_style" => {
                update.newline_style = Some(parse_newline_style(val)?);
            }
            "code_block_style" => {
                update.code_block_style = Some(parse_code_block_style(val)?);
            }
            "keep_inline_images_in" => {
                update.keep_inline_images_in = Some(parse_vec_of_strings(val)?);
            }
            "preprocessing" => {
                update.preprocessing = Some(parse_preprocessing_options(ruby, val)?);
            }
            "encoding" => {
                update.encoding = Some(String::try_convert(val)?);
            }
            "debug" => {
                update.debug = Some(bool::try_convert(val)?);
            }
            "strip_tags" => {
                update.strip_tags = Some(parse_vec_of_strings(val)?);
            }
            "preserve_tags" => {
                update.preserve_tags = Some(parse_vec_of_strings(val)?);
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(ConversionOptions::from(update))
}

fn build_inline_image_config(_ruby: &Ruby, config: Option<Value>) -> Result<InlineImageConfig, Error> {
    let mut update = InlineImageConfigUpdate::default();

    let Some(config) = config else {
        return Ok(InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    };

    if config.is_nil() {
        return Ok(InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    }

    let hash = RHash::from_value(config).ok_or_else(|| arg_error("inline image config must be provided as a Hash"))?;

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "max_decoded_size_bytes" => {
                update.max_decoded_size_bytes = Some(u64::try_convert(val)?);
            }
            "filename_prefix" => {
                update.filename_prefix = if val.is_nil() {
                    None
                } else {
                    Some(String::try_convert(val)?)
                };
            }
            "capture_svg" => {
                update.capture_svg = Some(bool::try_convert(val)?);
            }
            "infer_dimensions" => {
                update.infer_dimensions = Some(bool::try_convert(val)?);
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(InlineImageConfig::from_update(update))
}

fn inline_image_to_value(ruby: &Ruby, image: InlineImage) -> Result<Value, Error> {
    let InlineImage {
        data,
        format,
        filename,
        description,
        dimensions,
        source,
        attributes,
    } = image;

    let hash = ruby.hash_new();
    let data_value = ruby.str_from_slice(&data);
    hash.aset(ruby.intern("data"), data_value)?;

    let format_value = format.to_string();
    hash.aset(ruby.intern("format"), format_value)?;

    match filename {
        Some(name) => hash.aset(ruby.intern("filename"), name)?,
        None => hash.aset(ruby.intern("filename"), ruby.qnil())?,
    }

    match description {
        Some(desc) => hash.aset(ruby.intern("description"), desc)?,
        None => hash.aset(ruby.intern("description"), ruby.qnil())?,
    }

    if let Some((width, height)) = dimensions {
        let dims = ruby.ary_new();
        dims.push(width as i64)?;
        dims.push(height as i64)?;
        hash.aset(ruby.intern("dimensions"), dims)?;
    } else {
        hash.aset(ruby.intern("dimensions"), ruby.qnil())?;
    }

    let source_value = source.to_string();
    hash.aset(ruby.intern("source"), source_value)?;

    let attrs = ruby.hash_new();
    for (key, value) in attributes {
        attrs.aset(key, value)?;
    }
    hash.aset(ruby.intern("attributes"), attrs)?;

    Ok(hash.as_value())
}

fn warning_to_value(ruby: &Ruby, warning: InlineImageWarning) -> Result<Value, Error> {
    let hash = ruby.hash_new();
    hash.aset(ruby.intern("index"), warning.index as i64)?;
    hash.aset(ruby.intern("message"), warning.message)?;
    Ok(hash.as_value())
}

fn extraction_to_value(ruby: &Ruby, extraction: HtmlExtraction) -> Result<Value, Error> {
    let hash = ruby.hash_new();
    hash.aset(ruby.intern("markdown"), extraction.markdown)?;

    let inline_images = ruby.ary_new();
    for image in extraction.inline_images {
        inline_images.push(inline_image_to_value(ruby, image)?)?;
    }
    hash.aset(ruby.intern("inline_images"), inline_images)?;

    let warnings = ruby.ary_new();
    for warning in extraction.warnings {
        warnings.push(warning_to_value(ruby, warning)?)?;
    }
    hash.aset(ruby.intern("warnings"), warnings)?;

    Ok(hash.as_value())
}

fn convert_fn(ruby: &Ruby, args: &[Value]) -> Result<String, Error> {
    let parsed = scan_args::<(String,), (Option<Value>,), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let options = build_conversion_options(ruby, parsed.optional.0)?;

    guard_panic(|| profiling::maybe_profile(|| convert_inner(&html, Some(options)))).map_err(conversion_error)
}

fn options_handle_fn(ruby: &Ruby, args: &[Value]) -> Result<OptionsHandle, Error> {
    let parsed = scan_args::<(), (Option<Value>,), (), (), (), ()>(args)?;
    let options = build_conversion_options(ruby, parsed.optional.0)?;
    Ok(OptionsHandle(options))
}

fn convert_with_options_handle_fn(_ruby: &Ruby, args: &[Value]) -> Result<String, Error> {
    let parsed = scan_args::<(String, &OptionsHandle), (), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let handle = parsed.required.1;
    let options = handle.0.clone();

    guard_panic(|| profiling::maybe_profile(|| convert_inner(&html, Some(options)))).map_err(conversion_error)
}

fn convert_with_inline_images_fn(ruby: &Ruby, args: &[Value]) -> Result<Value, Error> {
    let parsed = scan_args::<(String,), (Option<Value>, Option<Value>), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let options = build_conversion_options(ruby, parsed.optional.0)?;
    let config = build_inline_image_config(ruby, parsed.optional.1)?;

    let extraction =
        guard_panic(|| convert_with_inline_images_inner(&html, Some(options), config)).map_err(conversion_error)?;

    extraction_to_value(ruby, extraction)
}

#[cfg(feature = "metadata")]
fn build_metadata_config(_ruby: &Ruby, config: Option<Value>) -> Result<RustMetadataConfig, Error> {
    let mut cfg = RustMetadataConfig::default();

    let Some(config) = config else {
        return Ok(cfg);
    };

    if config.is_nil() {
        return Ok(cfg);
    }

    let hash = RHash::from_value(config).ok_or_else(|| arg_error("metadata_config must be provided as a Hash"))?;

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "extract_headers" => {
                cfg.extract_headers = bool::try_convert(val)?;
            }
            "extract_links" => {
                cfg.extract_links = bool::try_convert(val)?;
            }
            "extract_images" => {
                cfg.extract_images = bool::try_convert(val)?;
            }
            "extract_structured_data" => {
                cfg.extract_structured_data = bool::try_convert(val)?;
            }
            "max_structured_data_size" => {
                cfg.max_structured_data_size = usize::try_convert(val)?;
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(cfg)
}

#[cfg(feature = "metadata")]
fn opt_string_to_ruby(ruby: &Ruby, opt: Option<String>) -> Result<Value, Error> {
    match opt {
        Some(val) => Ok(ruby.str_from_slice(val.as_bytes()).as_value()),
        None => Ok(ruby.qnil().as_value()),
    }
}

#[cfg(feature = "metadata")]
fn btreemap_to_ruby_hash(ruby: &Ruby, map: std::collections::BTreeMap<String, String>) -> Result<Value, Error> {
    let hash = ruby.hash_new();
    for (k, v) in map {
        hash.aset(k, v)?;
    }
    Ok(hash.as_value())
}

#[cfg(feature = "metadata")]
fn text_direction_to_string(text_direction: Option<RustTextDirection>) -> Option<String> {
    text_direction.map(|direction| direction.to_string())
}

#[cfg(feature = "metadata")]
fn document_metadata_to_ruby(ruby: &Ruby, doc: RustDocumentMetadata) -> Result<Value, Error> {
    let hash = ruby.hash_new();

    hash.aset(ruby.intern("title"), opt_string_to_ruby(ruby, doc.title)?)?;
    hash.aset(ruby.intern("description"), opt_string_to_ruby(ruby, doc.description)?)?;

    let keywords = ruby.ary_new();
    for keyword in doc.keywords {
        keywords.push(keyword)?;
    }
    hash.aset(ruby.intern("keywords"), keywords)?;

    hash.aset(ruby.intern("author"), opt_string_to_ruby(ruby, doc.author)?)?;
    hash.aset(
        ruby.intern("canonical_url"),
        opt_string_to_ruby(ruby, doc.canonical_url)?,
    )?;
    hash.aset(ruby.intern("base_href"), opt_string_to_ruby(ruby, doc.base_href)?)?;
    hash.aset(ruby.intern("language"), opt_string_to_ruby(ruby, doc.language)?)?;

    match text_direction_to_string(doc.text_direction) {
        Some(dir) => hash.aset(ruby.intern("text_direction"), dir)?,
        None => hash.aset(ruby.intern("text_direction"), ruby.qnil())?,
    }

    hash.aset(ruby.intern("open_graph"), btreemap_to_ruby_hash(ruby, doc.open_graph)?)?;
    hash.aset(
        ruby.intern("twitter_card"),
        btreemap_to_ruby_hash(ruby, doc.twitter_card)?,
    )?;
    hash.aset(ruby.intern("meta_tags"), btreemap_to_ruby_hash(ruby, doc.meta_tags)?)?;

    Ok(hash.as_value())
}

#[cfg(feature = "metadata")]
fn headers_to_ruby(ruby: &Ruby, headers: Vec<RustHeaderMetadata>) -> Result<Value, Error> {
    let array = ruby.ary_new();
    for header in headers {
        let hash = ruby.hash_new();
        hash.aset(ruby.intern("level"), header.level)?;
        hash.aset(ruby.intern("text"), header.text)?;
        hash.aset(ruby.intern("id"), opt_string_to_ruby(ruby, header.id)?)?;
        hash.aset(ruby.intern("depth"), header.depth as i64)?;
        hash.aset(ruby.intern("html_offset"), header.html_offset as i64)?;
        array.push(hash)?;
    }
    Ok(array.as_value())
}

#[cfg(feature = "metadata")]
fn links_to_ruby(ruby: &Ruby, links: Vec<RustLinkMetadata>) -> Result<Value, Error> {
    let array = ruby.ary_new();
    for link in links {
        let hash = ruby.hash_new();
        hash.aset(ruby.intern("href"), link.href)?;
        hash.aset(ruby.intern("text"), link.text)?;
        hash.aset(ruby.intern("title"), opt_string_to_ruby(ruby, link.title)?)?;
        hash.aset(ruby.intern("link_type"), link.link_type.to_string())?;

        let rel_array = ruby.ary_new();
        for r in link.rel {
            rel_array.push(r)?;
        }
        hash.aset(ruby.intern("rel"), rel_array)?;

        hash.aset(ruby.intern("attributes"), btreemap_to_ruby_hash(ruby, link.attributes)?)?;
        array.push(hash)?;
    }
    Ok(array.as_value())
}

#[cfg(feature = "metadata")]
fn images_to_ruby(ruby: &Ruby, images: Vec<RustImageMetadata>) -> Result<Value, Error> {
    let array = ruby.ary_new();
    for image in images {
        let hash = ruby.hash_new();
        hash.aset(ruby.intern("src"), image.src)?;
        hash.aset(ruby.intern("alt"), opt_string_to_ruby(ruby, image.alt)?)?;
        hash.aset(ruby.intern("title"), opt_string_to_ruby(ruby, image.title)?)?;

        match image.dimensions {
            Some((width, height)) => {
                let dims = ruby.ary_new();
                dims.push(width as i64)?;
                dims.push(height as i64)?;
                hash.aset(ruby.intern("dimensions"), dims)?;
            }
            None => {
                hash.aset(ruby.intern("dimensions"), ruby.qnil())?;
            }
        }

        hash.aset(ruby.intern("image_type"), image.image_type.to_string())?;
        hash.aset(
            ruby.intern("attributes"),
            btreemap_to_ruby_hash(ruby, image.attributes)?,
        )?;
        array.push(hash)?;
    }
    Ok(array.as_value())
}

#[cfg(feature = "metadata")]
fn structured_data_to_ruby(ruby: &Ruby, data: Vec<RustStructuredData>) -> Result<Value, Error> {
    let array = ruby.ary_new();
    for item in data {
        let hash = ruby.hash_new();
        hash.aset(ruby.intern("data_type"), item.data_type.to_string())?;
        hash.aset(ruby.intern("raw_json"), item.raw_json)?;
        hash.aset(ruby.intern("schema_type"), opt_string_to_ruby(ruby, item.schema_type)?)?;
        array.push(hash)?;
    }
    Ok(array.as_value())
}

#[cfg(feature = "metadata")]
fn extended_metadata_to_ruby(ruby: &Ruby, metadata: RustExtendedMetadata) -> Result<Value, Error> {
    let hash = ruby.hash_new();

    hash.aset(
        ruby.intern("document"),
        document_metadata_to_ruby(ruby, metadata.document)?,
    )?;
    hash.aset(ruby.intern("headers"), headers_to_ruby(ruby, metadata.headers)?)?;
    hash.aset(ruby.intern("links"), links_to_ruby(ruby, metadata.links)?)?;
    hash.aset(ruby.intern("images"), images_to_ruby(ruby, metadata.images)?)?;
    hash.aset(
        ruby.intern("structured_data"),
        structured_data_to_ruby(ruby, metadata.structured_data)?,
    )?;

    Ok(hash.as_value())
}

#[cfg(feature = "metadata")]
fn convert_with_metadata_fn(ruby: &Ruby, args: &[Value]) -> Result<Value, Error> {
    let parsed = scan_args::<(String,), (Option<Value>, Option<Value>), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let options = build_conversion_options(ruby, parsed.optional.0)?;
    let metadata_config = build_metadata_config(ruby, parsed.optional.1)?;

    let (markdown, metadata) =
        guard_panic(|| convert_with_metadata_inner(&html, Some(options), metadata_config)).map_err(conversion_error)?;

    let array = ruby.ary_new();
    array.push(markdown)?;
    array.push(extended_metadata_to_ruby(ruby, metadata)?)?;

    Ok(array.as_value())
}

fn start_profiling_fn(_ruby: &Ruby, args: &[Value]) -> Result<bool, Error> {
    let output = args.first().ok_or_else(|| arg_error("output_path required"))?;
    let output: String = String::try_convert(*output)?;
    let freq = if let Some(value) = args.get(1) {
        i32::try_convert(*value)?
    } else {
        1000
    };
    profiling::start(PathBuf::from(output), freq).map_err(conversion_error)?;
    Ok(true)
}

fn stop_profiling_fn(_ruby: &Ruby, _args: &[Value]) -> Result<bool, Error> {
    profiling::stop().map_err(conversion_error)?;
    Ok(true)
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("HtmlToMarkdown")?;
    module.define_singleton_method("convert", function!(convert_fn, -1))?;
    module.define_singleton_method("options", function!(options_handle_fn, -1))?;
    module.define_singleton_method("convert_with_options", function!(convert_with_options_handle_fn, -1))?;
    module.define_singleton_method(
        "convert_with_inline_images",
        function!(convert_with_inline_images_fn, -1),
    )?;

    #[cfg(feature = "metadata")]
    module.define_singleton_method("convert_with_metadata", function!(convert_with_metadata_fn, -1))?;
    module.define_singleton_method("start_profiling", function!(start_profiling_fn, -1))?;
    module.define_singleton_method("stop_profiling", function!(stop_profiling_fn, -1))?;

    Ok(())
}
