use std::collections::HashMap;

use html_to_markdown_rs::{
    CodeBlockStyle, ConversionError, ConversionOptions, HeadingStyle, ListIndentType, NewlineStyle,
    PreprocessingOptions, PreprocessingPreset, WhitespaceMode, convert as convert_html,
};
use rustler::{Error, NifResult, Term, types::atom::Atom};

mod atoms {
    rustler::atoms! {
        ok,
        error,
        invalid_option,
        conversion_failed,
        atx,
        atx_closed,
        underlined,
        spaces,
        tabs,
        normalized,
        strict,
        minimal,
        standard,
        aggressive,
        backslash,
        indented,
        backticks,
        tildes,
    }
}

rustler::init!("Elixir.HtmlToMarkdown.Native", [convert, convert_with_options]);

#[rustler::nif(schedule = "DirtyCpu")]
fn convert(html: String) -> NifResult<(Atom, String)> {
    let markdown = convert_html(&html, None).map_err(map_error)?;
    Ok((atoms::ok(), markdown))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn convert_with_options(html: String, opts: Term) -> NifResult<(Atom, String)> {
    let options = decode_options(opts)?;
    let markdown = convert_html(&html, Some(options)).map_err(map_error)?;
    Ok((atoms::ok(), markdown))
}

fn decode_options(term: Term) -> NifResult<ConversionOptions> {
    let map: HashMap<String, Term> = term.decode()?;
    apply_options(map)
}

fn apply_options(map: HashMap<String, Term>) -> NifResult<ConversionOptions> {
    let mut options = ConversionOptions::default();

    for (key, value) in map.iter() {
        match key.as_str() {
            "wrap" => options.wrap = value.decode::<bool>()?,
            "wrap_width" => options.wrap_width = ensure_positive(value.decode::<i64>()?, "wrap_width")? as usize,
            "list_indent_width" => {
                options.list_indent_width = ensure_positive(value.decode::<i64>()?, "list_indent_width")? as usize
            }
            "heading_style" => options.heading_style = decode_heading_style(value)?,
            "list_indent_type" => options.list_indent_type = decode_list_indent(value)?,
            "newline_style" => options.newline_style = decode_newline_style(value)?,
            "code_block_style" => options.code_block_style = decode_code_block_style(value)?,
            "whitespace" => options.whitespace_mode = decode_whitespace_mode(value)?,
            "bullets" => options.bullets = value.decode::<String>()?,
            "strong_em_symbol" => options.strong_em_symbol = decode_char(value, "strong_em_symbol")?,
            "convert_as_inline" => options.convert_as_inline = value.decode::<bool>()?,
            "preprocess" => options.preprocessing.enabled = value.decode::<bool>()?,
            "preprocessing_preset" => options.preprocessing.preset = decode_preset(value)?,
            "debug" => options.debug = value.decode::<bool>()?,
            "preprocessing" => apply_preprocessing(&mut options.preprocessing, value)?,
            _ => {}
        }
    }

    Ok(options)
}

fn ensure_positive(value: i64, field: &str) -> NifResult<i64> {
    if value <= 0 { Err(bad_option(field)) } else { Ok(value) }
}

fn decode_char(term: &Term, field: &str) -> NifResult<char> {
    let value = term.decode::<String>()?;
    value.chars().next().ok_or_else(|| bad_option(field))
}

fn decode_heading_style(term: &Term) -> NifResult<HeadingStyle> {
    let atom: Atom = term.decode()?;
    if atom == atoms::atx() {
        Ok(HeadingStyle::Atx)
    } else if atom == atoms::atx_closed() {
        Ok(HeadingStyle::AtxClosed)
    } else if atom == atoms::underlined() {
        Ok(HeadingStyle::Underlined)
    } else {
        Err(bad_option("heading_style"))
    }
}

fn decode_list_indent(term: &Term) -> NifResult<ListIndentType> {
    let atom: Atom = term.decode()?;
    if atom == atoms::spaces() {
        Ok(ListIndentType::Spaces)
    } else if atom == atoms::tabs() {
        Ok(ListIndentType::Tabs)
    } else {
        Err(bad_option("list_indent_type"))
    }
}

fn decode_newline_style(term: &Term) -> NifResult<NewlineStyle> {
    let atom: Atom = term.decode()?;
    if atom == atoms::spaces() {
        Ok(NewlineStyle::Spaces)
    } else if atom == atoms::backslash() {
        Ok(NewlineStyle::Backslash)
    } else {
        Err(bad_option("newline_style"))
    }
}

fn decode_code_block_style(term: &Term) -> NifResult<CodeBlockStyle> {
    let atom: Atom = term.decode()?;
    if atom == atoms::indented() {
        Ok(CodeBlockStyle::Indented)
    } else if atom == atoms::backticks() {
        Ok(CodeBlockStyle::Backticks)
    } else if atom == atoms::tildes() {
        Ok(CodeBlockStyle::Tildes)
    } else {
        Err(bad_option("code_block_style"))
    }
}

fn decode_whitespace_mode(term: &Term) -> NifResult<WhitespaceMode> {
    let atom: Atom = term.decode()?;
    if atom == atoms::normalized() {
        Ok(WhitespaceMode::Normalized)
    } else if atom == atoms::strict() {
        Ok(WhitespaceMode::Strict)
    } else {
        Err(bad_option("whitespace"))
    }
}

fn decode_preset(term: &Term) -> NifResult<PreprocessingPreset> {
    let atom: Atom = term.decode()?;
    if atom == atoms::minimal() {
        Ok(PreprocessingPreset::Minimal)
    } else if atom == atoms::aggressive() {
        Ok(PreprocessingPreset::Aggressive)
    } else if atom == atoms::standard() {
        Ok(PreprocessingPreset::Standard)
    } else {
        Err(bad_option("preprocessing_preset"))
    }
}

fn apply_preprocessing(options: &mut PreprocessingOptions, term: &Term) -> NifResult<()> {
    let map: HashMap<String, Term> = term.decode()?;
    if let Some(value) = map.get("enabled") {
        options.enabled = value.decode::<bool>()?;
    }
    if let Some(value) = map.get("preset") {
        options.preset = decode_preset(value)?;
    }
    if let Some(value) = map.get("remove_navigation") {
        options.remove_navigation = value.decode::<bool>()?;
    }
    if let Some(value) = map.get("remove_forms") {
        options.remove_forms = value.decode::<bool>()?;
    }
    Ok(())
}

fn map_error(err: ConversionError) -> Error {
    let message = err.to_string();
    Error::Term(Box::new((atoms::conversion_failed(), message)))
}

fn bad_option(name: &str) -> Error {
    let name = name.to_string();
    Error::Term(Box::new((atoms::invalid_option(), name)))
}
