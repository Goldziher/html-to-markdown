use clap::{Parser, ValueEnum};
use encoding_rs::Encoding;
use html_to_markdown::{
    convert, CodeBlockStyle, ConversionOptions, HeadingStyle, HighlightStyle, ListIndentType, NewlineStyle,
    ParsingOptions, PreprocessingOptions, PreprocessingPreset, WhitespaceMode,
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

    # Generate shell completions
    html-to-markdown --generate-completion bash > html-to-markdown.bash
    html-to-markdown --generate-completion zsh > _html-to-markdown

    # Generate man page
    html-to-markdown --generate-man > html-to-markdown.1

    # Web scraping with preprocessing
    html-to-markdown page.html --preprocess --preset aggressive

    # Discord/Slack-friendly (2-space indents)
    html-to-markdown input.html --list-indent-width 2

    # Custom heading and list styles
    html-to-markdown input.html \\
        --heading-style atx \\
        --bullets '*' \\
        --list-indent-width 2

For more information: https://github.com/Goldziher/html-to-markdown
")]
struct Cli {
    /// Input HTML file (use \"-\" or omit for stdin)
    #[arg(value_name = "FILE")]
    input: Option<String>,

    /// Output file (default: stdout)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Generate shell completion script
    #[arg(long = "generate-completion", value_name = "SHELL", value_enum)]
    generate_completion: Option<Shell>,

    /// Generate man page
    #[arg(long = "generate-man")]
    generate_man: bool,

    /// Heading style
    ///
    /// Controls how headings are formatted in the output:
    /// - 'atx': # for h1, ## for h2, etc. (default, CommonMark)
    /// - 'underlined': h1 uses ===, h2 uses ---
    /// - 'atx-closed': # Title # with closing hashes
    #[arg(long, value_name = "STYLE")]
    #[arg(help_heading = "Heading Options")]
    heading_style: Option<CliHeadingStyle>,

    /// List indentation type
    #[arg(long, value_name = "TYPE")]
    #[arg(help_heading = "List Options")]
    list_indent_type: Option<CliListIndentType>,

    /// Spaces per list indent level
    ///
    /// Default is 2 (CommonMark standard). Use 4 for wider indentation.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u8).range(1..=8))]
    #[arg(help_heading = "List Options")]
    list_indent_width: Option<u8>,

    /// Bullet characters for unordered lists
    ///
    /// Characters cycle through nesting levels. Default "-" uses hyphen
    /// consistently. "*+-" uses * for level 1, + for level 2, - for level 3.
    #[arg(short = 'b', long, value_name = "CHARS")]
    #[arg(help_heading = "List Options")]
    #[arg(value_parser = validate_bullets)]
    bullets: Option<String>,

    /// Symbol for bold and italic
    ///
    /// Choose '*' (default) or '_' for **bold** and *italic* text
    #[arg(long, value_name = "CHAR")]
    #[arg(help_heading = "Text Formatting")]
    #[arg(value_parser = validate_strong_em_symbol)]
    strong_em_symbol: Option<char>,

    /// Escape asterisk (*) characters
    #[arg(long)]
    #[arg(help_heading = "Text Formatting")]
    escape_asterisks: bool,

    /// Escape underscore (_) characters
    #[arg(long)]
    #[arg(help_heading = "Text Formatting")]
    escape_underscores: bool,

    /// Escape misc Markdown characters
    ///
    /// Escape characters like [, ], <, >, #, etc.
    #[arg(long)]
    #[arg(help_heading = "Text Formatting")]
    escape_misc: bool,

    /// Escape all ASCII punctuation
    ///
    /// For strict CommonMark spec compliance (usually not needed)
    #[arg(long)]
    #[arg(help_heading = "Text Formatting")]
    escape_ascii: bool,

    /// Symbol to wrap subscript text
    ///
    /// Example: "~" wraps <sub>text</sub> as ~text~
    #[arg(long, value_name = "SYMBOL")]
    #[arg(help_heading = "Text Formatting")]
    sub_symbol: Option<String>,

    /// Symbol to wrap superscript text
    ///
    /// Example: "^" wraps <sup>text</sup> as ^text^
    #[arg(long, value_name = "SYMBOL")]
    #[arg(help_heading = "Text Formatting")]
    sup_symbol: Option<String>,

    /// Line break style
    ///
    /// How to represent <br> tags:
    /// - 'backslash': Backslash at end of line (default, CommonMark)
    /// - 'spaces': Two spaces at end of line
    #[arg(long, value_name = "STYLE")]
    #[arg(help_heading = "Text Formatting")]
    newline_style: Option<CliNewlineStyle>,

    /// Code block style
    ///
    /// How to format code blocks:
    /// - 'indented': 4-space indentation (default, CommonMark)
    /// - 'backticks': Fenced with backticks (```)
    /// - 'tildes': Fenced with tildes (~~~)
    #[arg(long, value_name = "STYLE")]
    #[arg(help_heading = "Code Blocks")]
    code_block_style: Option<CliCodeBlockStyle>,

    /// Default language for code blocks
    ///
    /// Sets the language for fenced code blocks when not specified in HTML
    #[arg(short = 'l', long, value_name = "LANG")]
    #[arg(help_heading = "Code Blocks")]
    code_language: Option<String>,

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

    /// Keep inline images in specific elements
    ///
    /// Comma-separated list of HTML elements where images should remain
    /// as markdown (not converted to alt text). Example: "a,strong"
    #[arg(long, value_name = "ELEMENTS", value_delimiter = ',')]
    #[arg(help_heading = "Images")]
    keep_inline_images_in: Option<Vec<String>>,

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
    #[arg(long, value_name = "STYLE")]
    #[arg(help_heading = "Highlighting")]
    highlight_style: Option<CliHighlightStyle>,

    /// Extract metadata from HTML
    ///
    /// Extract title and meta tags as HTML comment header
    #[arg(long)]
    #[arg(help_heading = "Metadata")]
    extract_metadata: bool,

    /// Whitespace handling mode
    ///
    /// How to handle whitespace in HTML:
    /// - 'normalized': Clean up excess whitespace (default)
    /// - 'strict': Preserve whitespace as-is
    #[arg(long, value_name = "MODE")]
    #[arg(help_heading = "Whitespace")]
    whitespace_mode: Option<CliWhitespaceMode>,

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
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u16).range(20..=500))]
    #[arg(help_heading = "Wrapping")]
    wrap_width: Option<u16>,

    /// Treat block elements as inline
    ///
    /// Convert block-level elements without adding paragraph breaks
    #[arg(long)]
    #[arg(help_heading = "Element Handling")]
    convert_as_inline: bool,

    /// HTML tags to strip
    ///
    /// Comma-separated list of HTML tags to strip (output only text content,
    /// no markdown conversion). Example: "script,style"
    #[arg(long, value_name = "TAGS", value_delimiter = ',')]
    #[arg(help_heading = "Element Handling")]
    strip_tags: Option<Vec<String>>,

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
    #[arg(long, value_name = "LEVEL")]
    #[arg(help_heading = "Preprocessing")]
    #[arg(requires = "preprocess")]
    preset: Option<CliPreprocessingPreset>,

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

    /// Enable hOCR table extraction
    ///
    /// Extract tables from hOCR (OCR output) documents
    #[arg(long)]
    #[arg(help_heading = "hOCR Options")]
    hocr_extract_tables: bool,

    /// hOCR column detection threshold
    ///
    /// Pixel threshold for detecting table columns in hOCR
    #[arg(long, value_name = "PIXELS", value_parser = clap::value_parser!(u32).range(1..=1000))]
    #[arg(help_heading = "hOCR Options")]
    hocr_table_column_threshold: Option<u32>,

    /// hOCR row detection threshold ratio
    ///
    /// Ratio threshold for detecting table rows in hOCR
    #[arg(long, value_name = "RATIO", value_parser = validate_ratio)]
    #[arg(help_heading = "hOCR Options")]
    hocr_table_row_threshold_ratio: Option<f64>,

    /// Input character encoding
    ///
    /// Encoding to use when reading input files (e.g., 'utf-8', 'latin-1')
    #[arg(short = 'e', long, value_name = "ENCODING", default_value = "utf-8")]
    #[arg(help_heading = "Parsing")]
    encoding: String,

    /// Enable debug mode
    ///
    /// Output diagnostic warnings and information
    #[arg(long)]
    #[arg(help_heading = "Debugging")]
    debug: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
#[allow(clippy::enum_variant_names)]
enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CliHeadingStyle {
    /// ATX style: # for h1, ## for h2 (default)
    Atx,
    /// Underlined: === for h1, --- for h2
    Underlined,
    /// ATX closed: # Title #
    AtxClosed,
}

impl From<CliHeadingStyle> for HeadingStyle {
    fn from(style: CliHeadingStyle) -> Self {
        match style {
            CliHeadingStyle::Atx => HeadingStyle::Atx,
            CliHeadingStyle::Underlined => HeadingStyle::Underlined,
            CliHeadingStyle::AtxClosed => HeadingStyle::AtxClosed,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CliListIndentType {
    /// Use spaces for indentation
    Spaces,
    /// Use tabs for indentation
    Tabs,
}

impl From<CliListIndentType> for ListIndentType {
    fn from(indent_type: CliListIndentType) -> Self {
        match indent_type {
            CliListIndentType::Spaces => ListIndentType::Spaces,
            CliListIndentType::Tabs => ListIndentType::Tabs,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CliNewlineStyle {
    /// Two spaces at end of line
    Spaces,
    /// Backslash at end of line (default)
    Backslash,
}

impl From<CliNewlineStyle> for NewlineStyle {
    fn from(style: CliNewlineStyle) -> Self {
        match style {
            CliNewlineStyle::Spaces => NewlineStyle::Spaces,
            CliNewlineStyle::Backslash => NewlineStyle::Backslash,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CliCodeBlockStyle {
    /// Indented code blocks: 4 spaces (default)
    Indented,
    /// Fenced code blocks: ```
    Backticks,
    /// Fenced code blocks: ~~~
    Tildes,
}

impl From<CliCodeBlockStyle> for CodeBlockStyle {
    fn from(style: CliCodeBlockStyle) -> Self {
        match style {
            CliCodeBlockStyle::Indented => CodeBlockStyle::Indented,
            CliCodeBlockStyle::Backticks => CodeBlockStyle::Backticks,
            CliCodeBlockStyle::Tildes => CodeBlockStyle::Tildes,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CliHighlightStyle {
    /// ==text== (default)
    DoubleEqual,
    /// <mark>text</mark>
    Html,
    /// **text**
    Bold,
    /// Plain text
    None,
}

impl From<CliHighlightStyle> for HighlightStyle {
    fn from(style: CliHighlightStyle) -> Self {
        match style {
            CliHighlightStyle::DoubleEqual => HighlightStyle::DoubleEqual,
            CliHighlightStyle::Html => HighlightStyle::Html,
            CliHighlightStyle::Bold => HighlightStyle::Bold,
            CliHighlightStyle::None => HighlightStyle::None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CliWhitespaceMode {
    /// Normalize whitespace (default)
    Normalized,
    /// Preserve whitespace as-is
    Strict,
}

impl From<CliWhitespaceMode> for WhitespaceMode {
    fn from(mode: CliWhitespaceMode) -> Self {
        match mode {
            CliWhitespaceMode::Normalized => WhitespaceMode::Normalized,
            CliWhitespaceMode::Strict => WhitespaceMode::Strict,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CliPreprocessingPreset {
    /// Basic cleanup
    Minimal,
    /// Balanced cleaning (default)
    Standard,
    /// Maximum cleaning
    Aggressive,
}

impl From<CliPreprocessingPreset> for PreprocessingPreset {
    fn from(preset: CliPreprocessingPreset) -> Self {
        match preset {
            CliPreprocessingPreset::Minimal => PreprocessingPreset::Minimal,
            CliPreprocessingPreset::Standard => PreprocessingPreset::Standard,
            CliPreprocessingPreset::Aggressive => PreprocessingPreset::Aggressive,
        }
    }
}

fn validate_bullets(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("bullets cannot be empty".to_string());
    }
    if s.len() > 10 {
        return Err("bullets string too long (max 10 characters)".to_string());
    }
    Ok(s.to_string())
}

fn validate_strong_em_symbol(s: &str) -> Result<char, String> {
    if s.len() != 1 {
        return Err("strong_em_symbol must be exactly one character".to_string());
    }
    let c = s.chars().next().unwrap();
    if c != '*' && c != '_' {
        return Err("strong_em_symbol must be '*' or '_'".to_string());
    }
    Ok(c)
}

fn validate_ratio(s: &str) -> Result<f64, String> {
    let ratio: f64 = s.parse().map_err(|_| "ratio must be a number".to_string())?;
    if !(0.0..=1.0).contains(&ratio) {
        return Err("ratio must be between 0.0 and 1.0".to_string());
    }
    Ok(ratio)
}

fn decode_bytes(bytes: &[u8], encoding_name: &str) -> Result<String, String> {
    let lowercase = encoding_name.to_lowercase();
    let normalized = match lowercase.as_str() {
        "latin-1" | "latin1" => "iso-8859-1",
        "latin-2" | "latin2" => "iso-8859-2",
        "latin-3" | "latin3" => "iso-8859-3",
        "latin-4" | "latin4" => "iso-8859-4",
        "latin-5" | "latin5" => "iso-8859-5",
        "latin-6" | "latin6" => "iso-8859-6",
        "latin-7" | "latin7" => "iso-8859-7",
        "latin-8" | "latin8" => "iso-8859-8",
        "latin-9" | "latin9" => "iso-8859-9",
        "latin-10" | "latin10" => "iso-8859-10",
        _ => encoding_name,
    };

    let encoding =
        Encoding::for_label(normalized.as_bytes()).ok_or_else(|| format!("Unknown encoding '{}'", encoding_name))?;

    let (decoded, _, had_errors) = encoding.decode(bytes);
    if had_errors {
        eprintln!("Warning: Some characters could not be decoded correctly");
    }
    Ok(decoded.into_owned())
}

fn generate_completions(shell: Shell) {
    use clap::CommandFactory;
    use clap_complete::{generate, Shell as ClapShell};

    let mut cmd = Cli::command();
    let shell = match shell {
        Shell::Bash => ClapShell::Bash,
        Shell::Zsh => ClapShell::Zsh,
        Shell::Fish => ClapShell::Fish,
        Shell::PowerShell => ClapShell::PowerShell,
        Shell::Elvish => ClapShell::Elvish,
    };

    generate(shell, &mut cmd, "html-to-markdown", &mut io::stdout());
}

fn generate_man_page() -> Result<(), String> {
    use clap::CommandFactory;

    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer)
        .map_err(|e| format!("Failed to generate man page: {}", e))?;

    io::stdout()
        .write_all(&buffer)
        .map_err(|e| format!("Failed to write man page: {}", e))?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(shell) = cli.generate_completion {
        generate_completions(shell);
        return Ok(());
    }

    if cli.generate_man {
        generate_man_page()?;
        return Ok(());
    }

    let html = match cli.input.as_deref() {
        None | Some("-") => {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|e| format!("Error reading from stdin: {}", e))?;
            decode_bytes(&buffer, &cli.encoding)?
        }
        Some(path) => {
            let path = PathBuf::from(path);
            let bytes = fs::read(&path).map_err(|e| format!("Error reading file '{}': {}", path.display(), e))?;
            decode_bytes(&bytes, &cli.encoding)?
        }
    };

    let defaults = ConversionOptions::default();

    let preprocessing = PreprocessingOptions {
        enabled: cli.preprocess,
        preset: cli.preset.map(Into::into).unwrap_or_default(),
        remove_navigation: !cli.keep_navigation,
        remove_forms: !cli.keep_forms,
    };

    let parsing = ParsingOptions {
        encoding: cli.encoding,
        parser: None,
    };

    let options = ConversionOptions {
        heading_style: cli.heading_style.map(Into::into).unwrap_or(defaults.heading_style),
        list_indent_type: cli
            .list_indent_type
            .map(Into::into)
            .unwrap_or(defaults.list_indent_type),
        list_indent_width: cli
            .list_indent_width
            .map(|w| w as usize)
            .unwrap_or(defaults.list_indent_width),
        bullets: cli.bullets.unwrap_or(defaults.bullets),
        strong_em_symbol: cli.strong_em_symbol.unwrap_or(defaults.strong_em_symbol),
        escape_asterisks: cli.escape_asterisks,
        escape_underscores: cli.escape_underscores,
        escape_misc: cli.escape_misc,
        escape_ascii: cli.escape_ascii,
        code_language: cli.code_language.unwrap_or(defaults.code_language),
        autolinks: cli.autolinks,
        default_title: cli.default_title,
        br_in_tables: cli.br_in_tables,
        highlight_style: cli.highlight_style.map(Into::into).unwrap_or(defaults.highlight_style),
        extract_metadata: cli.extract_metadata,
        whitespace_mode: cli.whitespace_mode.map(Into::into).unwrap_or(defaults.whitespace_mode),
        strip_newlines: cli.strip_newlines,
        wrap: cli.wrap,
        wrap_width: cli.wrap_width.map(|w| w as usize).unwrap_or(defaults.wrap_width),
        convert_as_inline: cli.convert_as_inline,
        sub_symbol: cli.sub_symbol.unwrap_or(defaults.sub_symbol),
        sup_symbol: cli.sup_symbol.unwrap_or(defaults.sup_symbol),
        newline_style: cli.newline_style.map(Into::into).unwrap_or(defaults.newline_style),
        code_block_style: cli
            .code_block_style
            .map(Into::into)
            .unwrap_or(defaults.code_block_style),
        keep_inline_images_in: cli.keep_inline_images_in.unwrap_or(defaults.keep_inline_images_in),
        hocr_extract_tables: cli.hocr_extract_tables,
        hocr_table_column_threshold: cli
            .hocr_table_column_threshold
            .unwrap_or(defaults.hocr_table_column_threshold),
        hocr_table_row_threshold_ratio: cli
            .hocr_table_row_threshold_ratio
            .unwrap_or(defaults.hocr_table_row_threshold_ratio),
        preprocessing,
        parsing,
        debug: cli.debug,
        strip_tags: cli.strip_tags.unwrap_or(defaults.strip_tags),
    };

    let markdown = convert(&html, Some(options)).map_err(|e| format!("Error converting HTML: {}", e))?;

    match cli.output {
        Some(path) => {
            fs::write(&path, markdown.as_bytes())
                .map_err(|e| format!("Error writing to file '{}': {}", path.display(), e))?;
        }
        None => {
            print!("{}", markdown);
        }
    }

    Ok(())
}
