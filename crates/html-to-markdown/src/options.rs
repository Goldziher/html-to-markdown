//! Configuration options for HTML to Markdown conversion.

/// Heading style options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadingStyle {
    /// Underlined style (=== for h1, --- for h2)
    Underlined,
    /// ATX style (# for h1, ## for h2, etc.)
    #[default]
    Atx,
    /// ATX closed style (# title #)
    AtxClosed,
}

/// List indentation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListIndentType {
    #[default]
    Spaces,
    Tabs,
}

/// Whitespace handling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhitespaceMode {
    #[default]
    Normalized,
    Strict,
}

/// Newline style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewlineStyle {
    /// Two spaces at end of line
    #[default]
    Spaces,
    /// Backslash at end of line
    Backslash,
}

/// Code block style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeBlockStyle {
    /// Indented code blocks (4 spaces) - CommonMark default
    #[default]
    Indented,
    /// Fenced code blocks with backticks (```)
    Backticks,
    /// Fenced code blocks with tildes (~~~)
    Tildes,
}

/// Highlight style for `<mark>` elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HighlightStyle {
    /// ==text==
    #[default]
    DoubleEqual,
    /// <mark>text</mark>
    Html,
    /// **text**
    Bold,
    /// Plain text (no formatting)
    None,
}

/// Preprocessing preset levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreprocessingPreset {
    Minimal,
    #[default]
    Standard,
    Aggressive,
}

/// Main conversion options.
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Heading style
    pub heading_style: HeadingStyle,

    /// List indentation type
    pub list_indent_type: ListIndentType,

    /// List indentation width (spaces)
    pub list_indent_width: usize,

    /// Bullet characters for unordered lists
    pub bullets: String,

    /// Symbol for strong/emphasis (* or _)
    pub strong_em_symbol: char,

    /// Escape asterisks in text
    pub escape_asterisks: bool,

    /// Escape underscores in text
    pub escape_underscores: bool,

    /// Escape misc markdown characters
    pub escape_misc: bool,

    /// Escape all ASCII punctuation (for CommonMark spec compliance tests)
    pub escape_ascii: bool,

    /// Default code language
    pub code_language: String,

    /// Use autolinks for bare URLs
    pub autolinks: bool,

    /// Add default title if none exists
    pub default_title: bool,

    /// Use <br> in tables instead of spaces
    pub br_in_tables: bool,

    /// Enable spatial table reconstruction in hOCR documents
    pub hocr_spatial_tables: bool,

    /// Highlight style for <mark> elements
    pub highlight_style: HighlightStyle,

    /// Extract metadata from HTML
    pub extract_metadata: bool,

    /// Whitespace handling mode
    pub whitespace_mode: WhitespaceMode,

    /// Strip newlines from HTML before processing
    pub strip_newlines: bool,

    /// Enable text wrapping
    pub wrap: bool,

    /// Text wrap width
    pub wrap_width: usize,

    /// Treat block elements as inline
    pub convert_as_inline: bool,

    /// Subscript symbol
    pub sub_symbol: String,

    /// Superscript symbol
    pub sup_symbol: String,

    /// Newline style
    pub newline_style: NewlineStyle,

    /// Code block style
    pub code_block_style: CodeBlockStyle,

    /// Elements where images should remain as markdown (not converted to alt text)
    pub keep_inline_images_in: Vec<String>,

    /// Preprocessing options
    pub preprocessing: PreprocessingOptions,

    /// Source encoding (informational)
    pub encoding: String,

    /// Enable debug mode with diagnostic warnings
    pub debug: bool,

    /// List of HTML tags to strip (output only text content, no markdown conversion)
    pub strip_tags: Vec<String>,

    /// List of HTML tags to preserve as-is in the output (keep original HTML)
    /// Useful for complex elements like tables that don't convert well to Markdown
    pub preserve_tags: Vec<String>,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            heading_style: HeadingStyle::default(),
            list_indent_type: ListIndentType::default(),
            list_indent_width: 2,
            bullets: "-".to_string(),
            strong_em_symbol: '*',
            escape_asterisks: false,
            escape_underscores: false,
            escape_misc: false,
            escape_ascii: false,
            code_language: String::new(),
            autolinks: true,
            default_title: false,
            br_in_tables: false,
            hocr_spatial_tables: true,
            highlight_style: HighlightStyle::default(),
            extract_metadata: true,
            whitespace_mode: WhitespaceMode::default(),
            strip_newlines: false,
            wrap: false,
            wrap_width: 80,
            convert_as_inline: false,
            sub_symbol: String::new(),
            sup_symbol: String::new(),
            newline_style: NewlineStyle::Spaces,
            code_block_style: CodeBlockStyle::default(),
            keep_inline_images_in: Vec::new(),
            preprocessing: PreprocessingOptions::default(),
            encoding: "utf-8".to_string(),
            debug: false,
            strip_tags: Vec::new(),
            preserve_tags: Vec::new(),
        }
    }
}

/// HTML preprocessing options.
#[derive(Debug, Clone)]
pub struct PreprocessingOptions {
    /// Enable preprocessing
    pub enabled: bool,

    /// Preprocessing preset
    pub preset: PreprocessingPreset,

    /// Remove navigation elements
    pub remove_navigation: bool,

    /// Remove form elements
    pub remove_forms: bool,
}

impl Default for PreprocessingOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            preset: PreprocessingPreset::default(),
            remove_navigation: true,
            remove_forms: true,
        }
    }
}
