//! Configuration options for HTML to Markdown conversion.

/// Heading style options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingStyle {
    /// Underlined style (=== for h1, --- for h2)
    Underlined,
    /// ATX style (# for h1, ## for h2, etc.)
    Atx,
    /// ATX closed style (# title #)
    AtxClosed,
}

impl Default for HeadingStyle {
    fn default() -> Self {
        Self::Atx
    }
}

/// List indentation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListIndentType {
    Spaces,
    Tabs,
}

impl Default for ListIndentType {
    fn default() -> Self {
        Self::Spaces
    }
}

/// Whitespace handling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceMode {
    Normalized,
    Strict,
}

impl Default for WhitespaceMode {
    fn default() -> Self {
        Self::Normalized
    }
}

/// Newline style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewlineStyle {
    /// Two spaces at end of line
    Spaces,
    /// Backslash at end of line
    Backslash,
}

impl Default for NewlineStyle {
    fn default() -> Self {
        Self::Spaces
    }
}

/// Code block style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeBlockStyle {
    /// Indented code blocks (4 spaces) - CommonMark default
    Indented,
    /// Fenced code blocks with backticks (```)
    Backticks,
    /// Fenced code blocks with tildes (~~~)
    Tildes,
}

impl Default for CodeBlockStyle {
    fn default() -> Self {
        Self::Indented
    }
}

/// Highlight style for `<mark>` elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightStyle {
    /// ==text==
    DoubleEqual,
    /// <mark>text</mark>
    Html,
    /// **text**
    Bold,
    /// Plain text (no formatting)
    None,
}

impl Default for HighlightStyle {
    fn default() -> Self {
        Self::DoubleEqual
    }
}

/// Preprocessing preset levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreprocessingPreset {
    Minimal,
    Standard,
    Aggressive,
}

impl Default for PreprocessingPreset {
    fn default() -> Self {
        Self::Standard
    }
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
            enabled: true,
            preset: PreprocessingPreset::default(),
            remove_navigation: true,
            remove_forms: true,
        }
    }
}
