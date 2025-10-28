use html_to_markdown_rs::{
    convert as convert_inner, convert_with_inline_images as convert_with_inline_images_inner, error::ConversionError,
    CodeBlockStyle, ConversionOptions, HeadingStyle, HighlightStyle, HtmlExtraction, InlineImage, InlineImageConfig,
    InlineImageFormat, InlineImageSource, InlineImageWarning, ListIndentType, NewlineStyle, PreprocessingOptions,
    PreprocessingPreset, WhitespaceMode,
};
use magnus::prelude::*;
use magnus::{function, scan_args::scan_args, Error, RArray, RHash, Ruby, Symbol, TryConvert, Value};

const DEFAULT_INLINE_IMAGE_LIMIT: u64 = 5 * 1024 * 1024;

fn conversion_error(err: ConversionError) -> Error {
    match err {
        ConversionError::ConfigError(msg) => arg_error(msg),
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

fn get_kw(ruby: &Ruby, hash: RHash, name: &str) -> Option<Value> {
    let sym = ruby.intern(name);
    hash.get(sym).or_else(|| hash.get(name))
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

fn parse_preprocessing_options(ruby: &Ruby, value: Value) -> Result<PreprocessingOptions, Error> {
    let hash = RHash::from_value(value).ok_or_else(|| arg_error("expected preprocessing to be a Hash"))?;

    let mut opts = PreprocessingOptions::default();

    if let Some(enabled) = get_kw(ruby, hash, "enabled") {
        opts.enabled = bool::try_convert(enabled)?;
    }

    if let Some(preset) = get_kw(ruby, hash, "preset") {
        opts.preset = parse_preset(preset)?;
    }

    if let Some(remove_navigation) = get_kw(ruby, hash, "remove_navigation") {
        opts.remove_navigation = bool::try_convert(remove_navigation)?;
    }

    if let Some(remove_forms) = get_kw(ruby, hash, "remove_forms") {
        opts.remove_forms = bool::try_convert(remove_forms)?;
    }

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

    if let Some(heading_style) = get_kw(ruby, hash, "heading_style") {
        opts.heading_style = parse_heading_style(heading_style)?;
    }

    if let Some(list_indent_type) = get_kw(ruby, hash, "list_indent_type") {
        opts.list_indent_type = parse_list_indent_type(list_indent_type)?;
    }

    if let Some(list_indent_width) = get_kw(ruby, hash, "list_indent_width") {
        opts.list_indent_width = usize::try_convert(list_indent_width)?;
    }

    if let Some(bullets) = get_kw(ruby, hash, "bullets") {
        opts.bullets = String::try_convert(bullets)?;
    }

    if let Some(strong_em_symbol) = get_kw(ruby, hash, "strong_em_symbol") {
        let value = String::try_convert(strong_em_symbol)?;
        let mut chars = value.chars();
        let ch = chars
            .next()
            .ok_or_else(|| arg_error("strong_em_symbol must not be empty"))?;
        if chars.next().is_some() {
            return Err(arg_error("strong_em_symbol must be a single character"));
        }
        opts.strong_em_symbol = ch;
    }

    if let Some(escape_asterisks) = get_kw(ruby, hash, "escape_asterisks") {
        opts.escape_asterisks = bool::try_convert(escape_asterisks)?;
    }

    if let Some(escape_underscores) = get_kw(ruby, hash, "escape_underscores") {
        opts.escape_underscores = bool::try_convert(escape_underscores)?;
    }

    if let Some(escape_misc) = get_kw(ruby, hash, "escape_misc") {
        opts.escape_misc = bool::try_convert(escape_misc)?;
    }

    if let Some(escape_ascii) = get_kw(ruby, hash, "escape_ascii") {
        opts.escape_ascii = bool::try_convert(escape_ascii)?;
    }

    if let Some(code_language) = get_kw(ruby, hash, "code_language") {
        opts.code_language = String::try_convert(code_language)?;
    }

    if let Some(autolinks) = get_kw(ruby, hash, "autolinks") {
        opts.autolinks = bool::try_convert(autolinks)?;
    }

    if let Some(default_title) = get_kw(ruby, hash, "default_title") {
        opts.default_title = bool::try_convert(default_title)?;
    }

    if let Some(br_in_tables) = get_kw(ruby, hash, "br_in_tables") {
        opts.br_in_tables = bool::try_convert(br_in_tables)?;
    }

    if let Some(hocr_spatial_tables) = get_kw(ruby, hash, "hocr_spatial_tables") {
        opts.hocr_spatial_tables = bool::try_convert(hocr_spatial_tables)?;
    }

    if let Some(highlight_style) = get_kw(ruby, hash, "highlight_style") {
        opts.highlight_style = parse_highlight_style(highlight_style)?;
    }

    if let Some(extract_metadata) = get_kw(ruby, hash, "extract_metadata") {
        opts.extract_metadata = bool::try_convert(extract_metadata)?;
    }

    if let Some(whitespace_mode) = get_kw(ruby, hash, "whitespace_mode") {
        opts.whitespace_mode = parse_whitespace_mode(whitespace_mode)?;
    }

    if let Some(strip_newlines) = get_kw(ruby, hash, "strip_newlines") {
        opts.strip_newlines = bool::try_convert(strip_newlines)?;
    }

    if let Some(wrap) = get_kw(ruby, hash, "wrap") {
        opts.wrap = bool::try_convert(wrap)?;
    }

    if let Some(wrap_width) = get_kw(ruby, hash, "wrap_width") {
        opts.wrap_width = usize::try_convert(wrap_width)?;
    }

    if let Some(convert_as_inline) = get_kw(ruby, hash, "convert_as_inline") {
        opts.convert_as_inline = bool::try_convert(convert_as_inline)?;
    }

    if let Some(sub_symbol) = get_kw(ruby, hash, "sub_symbol") {
        opts.sub_symbol = String::try_convert(sub_symbol)?;
    }

    if let Some(sup_symbol) = get_kw(ruby, hash, "sup_symbol") {
        opts.sup_symbol = String::try_convert(sup_symbol)?;
    }

    if let Some(newline_style) = get_kw(ruby, hash, "newline_style") {
        opts.newline_style = parse_newline_style(newline_style)?;
    }

    if let Some(code_block_style) = get_kw(ruby, hash, "code_block_style") {
        opts.code_block_style = parse_code_block_style(code_block_style)?;
    }

    if let Some(keep_inline_images_in) = get_kw(ruby, hash, "keep_inline_images_in") {
        opts.keep_inline_images_in = parse_vec_of_strings(keep_inline_images_in)?;
    }

    if let Some(preprocessing) = get_kw(ruby, hash, "preprocessing") {
        opts.preprocessing = parse_preprocessing_options(ruby, preprocessing)?;
    }

    if let Some(encoding) = get_kw(ruby, hash, "encoding") {
        opts.encoding = String::try_convert(encoding)?;
    }

    if let Some(debug) = get_kw(ruby, hash, "debug") {
        opts.debug = bool::try_convert(debug)?;
    }

    if let Some(strip_tags) = get_kw(ruby, hash, "strip_tags") {
        opts.strip_tags = parse_vec_of_strings(strip_tags)?;
    }

    if let Some(preserve_tags) = get_kw(ruby, hash, "preserve_tags") {
        opts.preserve_tags = parse_vec_of_strings(preserve_tags)?;
    }

    Ok(opts)
}

fn build_inline_image_config(ruby: &Ruby, config: Option<Value>) -> Result<InlineImageConfig, Error> {
    let mut cfg = InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT);

    let Some(config) = config else {
        return Ok(cfg);
    };

    if config.is_nil() {
        return Ok(cfg);
    }

    let hash = RHash::from_value(config).ok_or_else(|| arg_error("inline image config must be provided as a Hash"))?;

    if let Some(limit) = get_kw(ruby, hash, "max_decoded_size_bytes") {
        cfg.max_decoded_size_bytes = u64::try_convert(limit)?;
    }

    if let Some(prefix) = get_kw(ruby, hash, "filename_prefix") {
        cfg.filename_prefix = if prefix.is_nil() {
            None
        } else {
            Some(String::try_convert(prefix)?)
        };
    }

    if let Some(capture_svg) = get_kw(ruby, hash, "capture_svg") {
        cfg.capture_svg = bool::try_convert(capture_svg)?;
    }

    if let Some(infer_dimensions) = get_kw(ruby, hash, "infer_dimensions") {
        cfg.infer_dimensions = bool::try_convert(infer_dimensions)?;
    }

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

    convert_inner(&html, Some(options)).map_err(conversion_error)
}

fn convert_with_inline_images_fn(ruby: &Ruby, args: &[Value]) -> Result<Value, Error> {
    let parsed = scan_args::<(String,), (Option<Value>, Option<Value>), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let options = build_conversion_options(ruby, parsed.optional.0)?;
    let config = build_inline_image_config(ruby, parsed.optional.1)?;

    let extraction = convert_with_inline_images_inner(&html, Some(options), config).map_err(conversion_error)?;

    extraction_to_value(ruby, extraction)
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("HtmlToMarkdown")?;
    module.define_singleton_method("convert", function!(convert_fn, -1))?;
    module.define_singleton_method(
        "convert_with_inline_images",
        function!(convert_with_inline_images_fn, -1),
    )?;

    Ok(())
}
