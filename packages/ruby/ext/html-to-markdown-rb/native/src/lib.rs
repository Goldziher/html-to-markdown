use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions, HeadingStyle, HighlightStyle, HtmlExtraction, InlineImage, InlineImageConfig,
    InlineImageFormat, InlineImageSource, InlineImageWarning, ListIndentType, NewlineStyle, PreprocessingOptions,
    PreprocessingPreset, WhitespaceMode, convert as convert_inner,
    convert_with_inline_images as convert_with_inline_images_inner, error::ConversionError, safety::guard_panic,
};
use magnus::prelude::*;
use magnus::r_hash::ForEach;
use magnus::{Error, RArray, RHash, Ruby, Symbol, TryConvert, Value, function, scan_args::scan_args};

#[derive(Clone)]
#[magnus::wrap(class = "HtmlToMarkdown::Options", free_immediately)]
struct OptionsHandle(ConversionOptions);

const DEFAULT_INLINE_IMAGE_LIMIT: u64 = 5 * 1024 * 1024;

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

fn parse_preprocessing_options(_ruby: &Ruby, value: Value) -> Result<PreprocessingOptions, Error> {
    let hash = RHash::from_value(value).ok_or_else(|| arg_error("expected preprocessing to be a Hash"))?;

    let mut opts = PreprocessingOptions::default();

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "enabled" => {
                opts.enabled = bool::try_convert(val)?;
            }
            "preset" => {
                opts.preset = parse_preset(val)?;
            }
            "remove_navigation" => {
                opts.remove_navigation = bool::try_convert(val)?;
            }
            "remove_forms" => {
                opts.remove_forms = bool::try_convert(val)?;
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(opts)
}

fn build_conversion_options(ruby: &Ruby, options: Option<Value>) -> Result<ConversionOptions, Error> {
    let mut opts = ConversionOptions::default();

    let Some(options) = options else {
        return Ok(opts);
    };

    if options.is_nil() {
        return Ok(opts);
    }

    let hash = RHash::from_value(options).ok_or_else(|| arg_error("options must be provided as a Hash"))?;

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "heading_style" => {
                opts.heading_style = parse_heading_style(val)?;
            }
            "list_indent_type" => {
                opts.list_indent_type = parse_list_indent_type(val)?;
            }
            "list_indent_width" => {
                opts.list_indent_width = usize::try_convert(val)?;
            }
            "bullets" => {
                opts.bullets = String::try_convert(val)?;
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
                opts.strong_em_symbol = ch;
            }
            "escape_asterisks" => {
                opts.escape_asterisks = bool::try_convert(val)?;
            }
            "escape_underscores" => {
                opts.escape_underscores = bool::try_convert(val)?;
            }
            "escape_misc" => {
                opts.escape_misc = bool::try_convert(val)?;
            }
            "escape_ascii" => {
                opts.escape_ascii = bool::try_convert(val)?;
            }
            "code_language" => {
                opts.code_language = String::try_convert(val)?;
            }
            "autolinks" => {
                opts.autolinks = bool::try_convert(val)?;
            }
            "default_title" => {
                opts.default_title = bool::try_convert(val)?;
            }
            "br_in_tables" => {
                opts.br_in_tables = bool::try_convert(val)?;
            }
            "hocr_spatial_tables" => {
                opts.hocr_spatial_tables = bool::try_convert(val)?;
            }
            "highlight_style" => {
                opts.highlight_style = parse_highlight_style(val)?;
            }
            "extract_metadata" => {
                opts.extract_metadata = bool::try_convert(val)?;
            }
            "whitespace_mode" => {
                opts.whitespace_mode = parse_whitespace_mode(val)?;
            }
            "strip_newlines" => {
                opts.strip_newlines = bool::try_convert(val)?;
            }
            "wrap" => {
                opts.wrap = bool::try_convert(val)?;
            }
            "wrap_width" => {
                opts.wrap_width = usize::try_convert(val)?;
            }
            "convert_as_inline" => {
                opts.convert_as_inline = bool::try_convert(val)?;
            }
            "sub_symbol" => {
                opts.sub_symbol = String::try_convert(val)?;
            }
            "sup_symbol" => {
                opts.sup_symbol = String::try_convert(val)?;
            }
            "newline_style" => {
                opts.newline_style = parse_newline_style(val)?;
            }
            "code_block_style" => {
                opts.code_block_style = parse_code_block_style(val)?;
            }
            "keep_inline_images_in" => {
                opts.keep_inline_images_in = parse_vec_of_strings(val)?;
            }
            "preprocessing" => {
                opts.preprocessing = parse_preprocessing_options(ruby, val)?;
            }
            "encoding" => {
                opts.encoding = String::try_convert(val)?;
            }
            "debug" => {
                opts.debug = bool::try_convert(val)?;
            }
            "strip_tags" => {
                opts.strip_tags = parse_vec_of_strings(val)?;
            }
            "preserve_tags" => {
                opts.preserve_tags = parse_vec_of_strings(val)?;
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(opts)
}

fn build_inline_image_config(_ruby: &Ruby, config: Option<Value>) -> Result<InlineImageConfig, Error> {
    let mut cfg = InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT);

    let Some(config) = config else {
        return Ok(cfg);
    };

    if config.is_nil() {
        return Ok(cfg);
    }

    let hash = RHash::from_value(config).ok_or_else(|| arg_error("inline image config must be provided as a Hash"))?;

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "max_decoded_size_bytes" => {
                cfg.max_decoded_size_bytes = u64::try_convert(val)?;
            }
            "filename_prefix" => {
                cfg.filename_prefix = if val.is_nil() {
                    None
                } else {
                    Some(String::try_convert(val)?)
                };
            }
            "capture_svg" => {
                cfg.capture_svg = bool::try_convert(val)?;
            }
            "infer_dimensions" => {
                cfg.infer_dimensions = bool::try_convert(val)?;
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(cfg)
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

    let format_value = match format {
        InlineImageFormat::Png => "png".to_string(),
        InlineImageFormat::Jpeg => "jpeg".to_string(),
        InlineImageFormat::Gif => "gif".to_string(),
        InlineImageFormat::Bmp => "bmp".to_string(),
        InlineImageFormat::Webp => "webp".to_string(),
        InlineImageFormat::Svg => "svg".to_string(),
        InlineImageFormat::Other(other) => other,
    };
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

    let source_value = match source {
        InlineImageSource::ImgDataUri => "img_data_uri",
        InlineImageSource::SvgElement => "svg_element",
    };
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

    guard_panic(|| convert_inner(&html, Some(options))).map_err(conversion_error)
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

    guard_panic(|| convert_inner(&html, Some(options))).map_err(conversion_error)
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

    Ok(())
}
