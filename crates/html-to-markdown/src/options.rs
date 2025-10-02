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
        Self::Underlined
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

    /// Default code language
    pub code_language: String,

    /// Use autolinks for bare URLs
    pub autolinks: bool,

    /// Add default title if none exists
    pub default_title: bool,

    /// Use <br> in tables instead of spaces
    pub br_in_tables: bool,

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

    /// Elements where images should remain as markdown (not converted to alt text)
    pub keep_inline_images_in: Vec<String>,

    /// Preprocessing options
    pub preprocessing: PreprocessingOptions,

    /// Parsing options
    pub parsing: ParsingOptions,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            heading_style: HeadingStyle::default(),
            list_indent_type: ListIndentType::default(),
            list_indent_width: 4,
            bullets: "*+-".to_string(),
            strong_em_symbol: '*',
            escape_asterisks: true,
            escape_underscores: true,
            escape_misc: true,
            code_language: String::new(),
            autolinks: true,
            default_title: false,
            br_in_tables: false,
            highlight_style: HighlightStyle::default(),
            extract_metadata: true,
            whitespace_mode: WhitespaceMode::default(),
            strip_newlines: false,
            wrap: false,
            wrap_width: 80,
            convert_as_inline: false,
            sub_symbol: String::new(),
            sup_symbol: String::new(),
            newline_style: NewlineStyle::default(),
            keep_inline_images_in: Vec::new(),
            preprocessing: PreprocessingOptions::default(),
            parsing: ParsingOptions::default(),
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

/// HTML parsing options.
#[derive(Debug, Clone)]
pub struct ParsingOptions {
    /// Source encoding
    pub encoding: String,

    /// HTML parser to use
    pub parser: Option<String>,
}

impl Default for ParsingOptions {
    fn default() -> Self {
        Self {
            encoding: "utf-8".to_string(),
            parser: None,
        }
    }
}
