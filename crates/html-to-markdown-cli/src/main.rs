use clap::Parser;
use html_to_markdown::{
    convert, ConversionOptions, HeadingStyle, HighlightStyle, ListIndentType, NewlineStyle,
    ParsingOptions, PreprocessingOptions, PreprocessingPreset, WhitespaceMode,
};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "html-to-markdown")]
#[command(version, about = "Convert HTML to Markdown with comprehensive customization options", long_about = None)]
struct Cli {
    /// Input HTML file (use "-" or omit for stdin)
    #[arg(value_name = "FILE")]
    input: Option<String>,

    /// Header style: 'atx' (#), 'atx_closed' (# #), or 'underlined' (===)
    #[arg(long, value_name = "STYLE", default_value = "underlined")]
    heading_style: String,

    /// Characters for bullet points, alternates by nesting level
    #[arg(short, long, value_name = "CHARS", default_value = "*+-")]
    bullets: String,

    /// Symbol for bold/italic text: '*' or '_'
    #[arg(long, value_name = "SYMBOL", default_value = "*")]
    strong_em_symbol: char,

    /// Characters to surround subscript text
    #[arg(long, value_name = "SYMBOL", default_value = "")]
    sub_symbol: String,

    /// Characters to surround superscript text
    #[arg(long, value_name = "SYMBOL", default_value = "")]
    sup_symbol: String,

    /// Line break style: 'spaces' (two spaces) or 'backslash' (\)
    #[arg(long, value_name = "STYLE", default_value = "spaces")]
    newline_style: String,

    /// Default language for code blocks
    #[arg(long, value_name = "LANG", default_value = "")]
    code_language: String,

    /// Don't escape asterisk (*) characters
    #[arg(long)]
    no_escape_asterisks: bool,

    /// Don't escape underscore (_) characters
    #[arg(long)]
    no_escape_underscores: bool,

    /// Don't escape other special Markdown characters
    #[arg(long)]
    no_escape_misc: bool,

    /// Convert URLs to automatic links when text matches href
    #[arg(short, long)]
    autolinks: bool,

    /// Use href as link title when no title is provided
    #[arg(long)]
    default_title: bool,

    /// Use <br> tags for line breaks in table cells instead of spaces
    #[arg(long)]
    br_in_tables: bool,

    /// Enable text wrapping at --wrap-width characters
    #[arg(short, long)]
    wrap: bool,

    /// Column width for text wrapping
    #[arg(long, value_name = "WIDTH", default_value = "80")]
    wrap_width: usize,

    /// Remove newlines from HTML input (helps with messy HTML formatting)
    #[arg(long)]
    strip_newlines: bool,

    /// Treat all content as inline elements (no paragraph breaks)
    #[arg(long)]
    convert_as_inline: bool,

    /// Don't extract metadata (title, meta tags) as comment header
    #[arg(long)]
    no_extract_metadata: bool,

    /// Highlighting style: 'double-equal' (==), 'html' (<mark>), or 'bold' (**)
    #[arg(long, value_name = "STYLE", default_value = "double-equal")]
    highlight_style: String,

    /// List indentation: 'spaces' or 'tabs'
    #[arg(long, value_name = "TYPE", default_value = "spaces")]
    list_indent_type: String,

    /// Spaces per list indent level (use 2 for Discord/Slack)
    #[arg(long, value_name = "WIDTH", default_value = "4")]
    list_indent_width: usize,

    /// Whitespace handling: 'normalized' (clean) or 'strict' (preserve)
    #[arg(long, value_name = "MODE", default_value = "normalized")]
    whitespace_mode: String,

    /// Clean messy HTML (removes navigation, ads, forms, etc)
    #[arg(long)]
    preprocess_html: bool,

    /// Cleaning level: 'minimal', 'standard', or 'aggressive'
    #[arg(long, value_name = "PRESET", default_value = "standard")]
    preprocessing_preset: String,

    /// Keep form elements when preprocessing (normally removed)
    #[arg(long)]
    no_remove_forms: bool,

    /// Keep navigation elements when preprocessing (normally removed)
    #[arg(long)]
    no_remove_navigation: bool,

    /// Encoding for reading input files (e.g. 'utf-8', 'latin-1')
    #[arg(long, value_name = "ENCODING")]
    source_encoding: Option<String>,
}

fn parse_heading_style(s: &str) -> HeadingStyle {
    match s {
        "atx" => HeadingStyle::Atx,
        "atx_closed" => HeadingStyle::AtxClosed,
        "underlined" => HeadingStyle::Underlined,
        _ => {
            eprintln!("Invalid heading style '{}', using default 'underlined'", s);
            HeadingStyle::Underlined
        }
    }
}

fn parse_newline_style(s: &str) -> NewlineStyle {
    match s {
        "spaces" => NewlineStyle::Spaces,
        "backslash" => NewlineStyle::Backslash,
        _ => {
            eprintln!("Invalid newline style '{}', using default 'spaces'", s);
            NewlineStyle::Spaces
        }
    }
}

fn parse_highlight_style(s: &str) -> HighlightStyle {
    match s {
        "double-equal" => HighlightStyle::DoubleEqual,
        "html" => HighlightStyle::Html,
        "bold" => HighlightStyle::Bold,
        "none" => HighlightStyle::None,
        _ => {
            eprintln!(
                "Invalid highlight style '{}', using default 'double-equal'",
                s
            );
            HighlightStyle::DoubleEqual
        }
    }
}

fn parse_list_indent_type(s: &str) -> ListIndentType {
    match s {
        "spaces" => ListIndentType::Spaces,
        "tabs" => ListIndentType::Tabs,
        _ => {
            eprintln!("Invalid list indent type '{}', using default 'spaces'", s);
            ListIndentType::Spaces
        }
    }
}

fn parse_whitespace_mode(s: &str) -> WhitespaceMode {
    match s {
        "normalized" => WhitespaceMode::Normalized,
        "strict" => WhitespaceMode::Strict,
        _ => {
            eprintln!(
                "Invalid whitespace mode '{}', using default 'normalized'",
                s
            );
            WhitespaceMode::Normalized
        }
    }
}

fn parse_preprocessing_preset(s: &str) -> PreprocessingPreset {
    match s {
        "minimal" => PreprocessingPreset::Minimal,
        "standard" => PreprocessingPreset::Standard,
        "aggressive" => PreprocessingPreset::Aggressive,
        _ => {
            eprintln!(
                "Invalid preprocessing preset '{}', using default 'standard'",
                s
            );
            PreprocessingPreset::Standard
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // Read input HTML
    let html = match cli.input.as_deref() {
        None | Some("-") => {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .expect("Failed to read from stdin");
            buffer
        }
        Some(path) => {
            // Read from file
            let path = PathBuf::from(path);
            fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("Error reading file '{}': {}", path.display(), e);
                std::process::exit(1);
            })
        }
    };

    // Build conversion options
    let preprocessing = PreprocessingOptions {
        enabled: cli.preprocess_html,
        preset: parse_preprocessing_preset(&cli.preprocessing_preset),
        remove_navigation: !cli.no_remove_navigation,
        remove_forms: !cli.no_remove_forms,
    };

    let parsing = ParsingOptions {
        encoding: cli.source_encoding.unwrap_or_else(|| "utf-8".to_string()),
        parser: None, // Rust always uses html5ever
    };

    let options = ConversionOptions {
        heading_style: parse_heading_style(&cli.heading_style),
        list_indent_type: parse_list_indent_type(&cli.list_indent_type),
        list_indent_width: cli.list_indent_width,
        bullets: cli.bullets,
        strong_em_symbol: cli.strong_em_symbol,
        escape_asterisks: !cli.no_escape_asterisks,
        escape_underscores: !cli.no_escape_underscores,
        escape_misc: !cli.no_escape_misc,
        code_language: cli.code_language,
        autolinks: cli.autolinks,
        default_title: cli.default_title,
        br_in_tables: cli.br_in_tables,
        highlight_style: parse_highlight_style(&cli.highlight_style),
        extract_metadata: !cli.no_extract_metadata,
        whitespace_mode: parse_whitespace_mode(&cli.whitespace_mode),
        strip_newlines: cli.strip_newlines,
        wrap: cli.wrap,
        wrap_width: cli.wrap_width,
        convert_as_inline: cli.convert_as_inline,
        sub_symbol: cli.sub_symbol,
        sup_symbol: cli.sup_symbol,
        newline_style: parse_newline_style(&cli.newline_style),
        keep_inline_images_in: vec![], // Not supported in CLI yet
        preprocessing,
        parsing,
    };

    // Convert HTML to Markdown
    match convert(&html, Some(options)) {
        Ok(markdown) => {
            print!("{}", markdown);
        }
        Err(e) => {
            eprintln!("Error converting HTML: {}", e);
            std::process::exit(1);
        }
    }
}
