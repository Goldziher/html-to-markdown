use clap::Parser;
use html_to_markdown::{
    convert, ConversionOptions, HeadingStyle, HighlightStyle, ListIndentType, NewlineStyle, ParsingOptions,
    PreprocessingOptions, PreprocessingPreset, WhitespaceMode,
};
use std::fs;
use std::io::{self, Read, Write as IoWrite};
use std::path::PathBuf;

/// Convert HTML to Markdown
///
/// A fast, powerful HTML to Markdown converter with comprehensive
/// customization options. Uses the html5ever parser for standards-compliant
/// HTML processing.
#[derive(Parser)]
#[command(name = "html-to-markdown")]
#[command(version)]
#[command(about, long_about = None)]
#[command(after_help = "EXAMPLES:
    # Basic conversion from stdin
    echo '<h1>Title</h1><p>Content</p>' | html-to-markdown

    # Convert file to stdout
    html-to-markdown input.html

    # Convert and save to file
    html-to-markdown input.html -o output.md
    html-to-markdown input.html --output output.md

    # Web scraping with preprocessing
    html-to-markdown page.html --preprocess --preset aggressive

    # Discord/Slack-friendly (2-space indents)
    html-to-markdown input.html --list-indent-width 2

    # Custom heading and list styles
    html-to-markdown input.html \\
        --heading-style atx \\
        --bullets '*' \\
        --list-indent-width 2

For more information and documentation: https://github.com/Goldziher/html-to-markdown
")]
struct Cli {
    /// Input HTML file (use \"-\" or omit for stdin)
    #[arg(value_name = "FILE")]
    input: Option<String>,

    /// Output file (default: stdout)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Heading style
    ///
    /// Controls how headings are formatted in the output:
    /// - 'underlined': h1 uses ===, h2 uses --- (default)
    /// - 'atx': # for h1, ## for h2, etc.
    /// - 'atx_closed': # Title # with closing hashes
    #[arg(long, value_name = "STYLE", default_value = "underlined")]
    #[arg(help_heading = "Heading Options")]
    #[arg(value_parser = ["underlined", "atx", "atx_closed"])]
    heading_style: String,

    /// List indentation type
    #[arg(long, value_name = "TYPE", default_value = "spaces")]
    #[arg(help_heading = "List Options")]
    #[arg(value_parser = ["spaces", "tabs"])]
    list_indent_type: String,

    /// Spaces per list indent level
    ///
    /// Use 2 for Discord/Slack compatibility, 4 for standard Markdown
    #[arg(long, value_name = "N", default_value = "4")]
    #[arg(help_heading = "List Options")]
    list_indent_width: usize,

    /// Bullet characters for unordered lists
    ///
    /// Characters cycle through nesting levels. Default "*+-" uses * for
    /// level 1, + for level 2, - for level 3, then repeats.
    #[arg(short = 'b', long, value_name = "CHARS", default_value = "*+-")]
    #[arg(help_heading = "List Options")]
    bullets: String,

    /// Symbol for bold and italic
    ///
    /// Choose '*' (default) or '_' for **bold** and *italic* text
    #[arg(long, value_name = "CHAR", default_value = "*")]
    #[arg(help_heading = "Text Formatting")]
    strong_em_symbol: char,

    /// Don't escape asterisk (*) characters
    #[arg(long)]
    #[arg(help_heading = "Text Formatting")]
    no_escape_asterisks: bool,

    /// Don't escape underscore (_) characters
    #[arg(long)]
    #[arg(help_heading = "Text Formatting")]
    no_escape_underscores: bool,

    /// Don't escape misc Markdown characters
    ///
    /// When disabled, characters like [, ], <, >, #, etc. won't be escaped
    #[arg(long)]
    #[arg(help_heading = "Text Formatting")]
    no_escape_misc: bool,

    /// Symbol to wrap subscript text
    ///
    /// Example: "~" wraps <sub>text</sub> as ~text~
    #[arg(long, value_name = "SYMBOL", default_value = "")]
    #[arg(help_heading = "Text Formatting")]
    sub_symbol: String,

    /// Symbol to wrap superscript text
    ///
    /// Example: "^" wraps <sup>text</sup> as ^text^
    #[arg(long, value_name = "SYMBOL", default_value = "")]
    #[arg(help_heading = "Text Formatting")]
    sup_symbol: String,

    /// Line break style
    ///
    /// How to represent <br> tags:
    /// - 'spaces': Two spaces at end of line (default)
    /// - 'backslash': Backslash at end of line
    #[arg(long, value_name = "STYLE", default_value = "spaces")]
    #[arg(help_heading = "Text Formatting")]
    #[arg(value_parser = ["spaces", "backslash"])]
    newline_style: String,

    /// Default language for code blocks
    ///
    /// Sets the language for fenced code blocks when not specified in HTML
    #[arg(short = 'l', long, value_name = "LANG", default_value = "")]
    #[arg(help_heading = "Code Blocks")]
    code_language: String,

    /// Convert URLs to autolinks
    ///
    /// When link text equals href, use <url> instead of [url](url)
    #[arg(short = 'a', long)]
    #[arg(help_heading = "Links")]
    autolinks: bool,

    /// Add default title to links
    ///
    /// Use href as link title when no title attribute exists
    #[arg(long)]
    #[arg(help_heading = "Links")]
    default_title: bool,

    /// Use <br> in table cells
    ///
    /// Preserve line breaks in table cells using <br> tags instead of
    /// converting to spaces
    #[arg(long)]
    #[arg(help_heading = "Tables")]
    br_in_tables: bool,

    /// Style for <mark> elements
    ///
    /// How to represent highlighted text:
    /// - 'double-equal': ==text== (default)
    /// - 'html': <mark>text</mark>
    /// - 'bold': **text**
    /// - 'none': plain text
    #[arg(long, value_name = "STYLE", default_value = "double-equal")]
    #[arg(help_heading = "Highlighting")]
    #[arg(value_parser = ["double-equal", "html", "bold", "none"])]
    highlight_style: String,

    /// Don't extract metadata
    ///
    /// Skip extracting title and meta tags as HTML comment header
    #[arg(long)]
    #[arg(help_heading = "Metadata")]
    no_extract_metadata: bool,

    /// Whitespace handling mode
    ///
    /// How to handle whitespace in HTML:
    /// - 'normalized': Clean up excess whitespace (default)
    /// - 'strict': Preserve whitespace as-is
    #[arg(long, value_name = "MODE", default_value = "normalized")]
    #[arg(help_heading = "Whitespace")]
    #[arg(value_parser = ["normalized", "strict"])]
    whitespace_mode: String,

    /// Strip newlines from input
    ///
    /// Remove all newlines from HTML before processing (useful for
    /// minified HTML)
    #[arg(long)]
    #[arg(help_heading = "Whitespace")]
    strip_newlines: bool,

    /// Enable text wrapping
    ///
    /// Wrap output lines at --wrap-width columns
    #[arg(short = 'w', long)]
    #[arg(help_heading = "Wrapping")]
    wrap: bool,

    /// Wrap width in columns
    ///
    /// Column width for text wrapping when --wrap is enabled
    #[arg(long, value_name = "N", default_value = "80")]
    #[arg(help_heading = "Wrapping")]
    wrap_width: usize,

    /// Treat block elements as inline
    ///
    /// Convert block-level elements without adding paragraph breaks
    #[arg(long)]
    #[arg(help_heading = "Element Handling")]
    convert_as_inline: bool,

    /// Enable HTML preprocessing
    ///
    /// Clean up HTML before conversion (removes navigation, ads, forms, etc.)
    #[arg(short = 'p', long)]
    #[arg(help_heading = "Preprocessing")]
    preprocess: bool,

    /// Preprocessing aggressiveness preset
    ///
    /// How aggressively to clean HTML:
    /// - 'minimal': Basic cleanup only
    /// - 'standard': Balanced cleaning (default)
    /// - 'aggressive': Maximum cleaning for web scraping
    #[arg(long, value_name = "LEVEL", default_value = "standard")]
    #[arg(help_heading = "Preprocessing")]
    #[arg(requires = "preprocess")]
    #[arg(value_parser = ["minimal", "standard", "aggressive"])]
    preset: String,

    /// Keep navigation elements
    ///
    /// Don't remove <nav>, menus, etc. during preprocessing
    #[arg(long)]
    #[arg(help_heading = "Preprocessing")]
    #[arg(requires = "preprocess")]
    keep_navigation: bool,

    /// Keep form elements
    ///
    /// Don't remove <form>, <input>, etc. during preprocessing
    #[arg(long)]
    #[arg(help_heading = "Preprocessing")]
    #[arg(requires = "preprocess")]
    keep_forms: bool,

    /// Input character encoding
    ///
    /// Encoding to use when reading input files (e.g., 'utf-8', 'latin-1')
    #[arg(short = 'e', long, value_name = "ENCODING", default_value = "utf-8")]
    #[arg(help_heading = "Parsing")]
    encoding: String,
}

fn parse_heading_style(s: &str) -> HeadingStyle {
    match s {
        "atx" => HeadingStyle::Atx,
        "atx_closed" => HeadingStyle::AtxClosed,
        "underlined" => HeadingStyle::Underlined,
        _ => HeadingStyle::Underlined,
    }
}

fn parse_newline_style(s: &str) -> NewlineStyle {
    match s {
        "spaces" => NewlineStyle::Spaces,
        "backslash" => NewlineStyle::Backslash,
        _ => NewlineStyle::Spaces,
    }
}

fn parse_highlight_style(s: &str) -> HighlightStyle {
    match s {
        "double-equal" => HighlightStyle::DoubleEqual,
        "html" => HighlightStyle::Html,
        "bold" => HighlightStyle::Bold,
        "none" => HighlightStyle::None,
        _ => HighlightStyle::DoubleEqual,
    }
}

fn parse_list_indent_type(s: &str) -> ListIndentType {
    match s {
        "spaces" => ListIndentType::Spaces,
        "tabs" => ListIndentType::Tabs,
        _ => ListIndentType::Spaces,
    }
}

fn parse_whitespace_mode(s: &str) -> WhitespaceMode {
    match s {
        "normalized" => WhitespaceMode::Normalized,
        "strict" => WhitespaceMode::Strict,
        _ => WhitespaceMode::Normalized,
    }
}

fn parse_preprocessing_preset(s: &str) -> PreprocessingPreset {
    match s {
        "minimal" => PreprocessingPreset::Minimal,
        "standard" => PreprocessingPreset::Standard,
        "aggressive" => PreprocessingPreset::Aggressive,
        _ => PreprocessingPreset::Standard,
    }
}

fn main() {
    let cli = Cli::parse();

    let html = match cli.input.as_deref() {
        None | Some("-") => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap_or_else(|e| {
                eprintln!("Error reading from stdin: {}", e);
                std::process::exit(1);
            });
            buffer
        }
        Some(path) => {
            let path = PathBuf::from(path);
            fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("Error reading file '{}': {}", path.display(), e);
                std::process::exit(1);
            })
        }
    };

    let preprocessing = PreprocessingOptions {
        enabled: cli.preprocess,
        preset: parse_preprocessing_preset(&cli.preset),
        remove_navigation: !cli.keep_navigation,
        remove_forms: !cli.keep_forms,
    };

    let parsing = ParsingOptions {
        encoding: cli.encoding,
        parser: None,
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
        keep_inline_images_in: vec![],
        hocr_extract_tables: true,
        hocr_table_column_threshold: 50,
        hocr_table_row_threshold_ratio: 0.5,
        preprocessing,
        parsing,
        debug: false,
    };

    let markdown = match convert(&html, Some(options)) {
        Ok(md) => md,
        Err(e) => {
            eprintln!("Error converting HTML: {}", e);
            std::process::exit(1);
        }
    };

    match cli.output {
        Some(path) => {
            let mut file = fs::File::create(&path).unwrap_or_else(|e| {
                eprintln!("Error creating output file '{}': {}", path.display(), e);
                std::process::exit(1);
            });
            file.write_all(markdown.as_bytes()).unwrap_or_else(|e| {
                eprintln!("Error writing to file '{}': {}", path.display(), e);
                std::process::exit(1);
            });
        }
        None => {
            print!("{}", markdown);
        }
    }
}
