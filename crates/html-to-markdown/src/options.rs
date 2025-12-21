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

impl HeadingStyle {
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "atx" => Self::Atx,
            "atxclosed" => Self::AtxClosed,
            _ => Self::Underlined,
        }
    }
}

/// List indentation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListIndentType {
    #[default]
    Spaces,
    Tabs,
}

impl ListIndentType {
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "tabs" => Self::Tabs,
            _ => Self::Spaces,
        }
    }
}

/// Whitespace handling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhitespaceMode {
    #[default]
    Normalized,
    Strict,
}

impl WhitespaceMode {
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "strict" => Self::Strict,
            _ => Self::Normalized,
        }
    }
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

impl NewlineStyle {
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "backslash" => Self::Backslash,
            _ => Self::Spaces,
        }
    }
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

impl CodeBlockStyle {
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "backticks" => Self::Backticks,
            "tildes" => Self::Tildes,
            _ => Self::Indented,
        }
    }
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

impl HighlightStyle {
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "doubleequal" => Self::DoubleEqual,
            "html" => Self::Html,
            "bold" => Self::Bold,
            "none" => Self::None,
            _ => Self::None,
        }
    }
}

/// Preprocessing preset levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreprocessingPreset {
    Minimal,
    #[default]
    Standard,
    Aggressive,
}

impl PreprocessingPreset {
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "minimal" => Self::Minimal,
            "aggressive" => Self::Aggressive,
            _ => Self::Standard,
        }
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

    /// List of HTML tags to preserve as-is in the output (keep original HTML)
    /// Useful for complex elements like tables that don't convert well to Markdown
    pub preserve_tags: Vec<String>,
}

/// Partial update for ConversionOptions.
#[derive(Debug, Clone, Default)]
#[cfg_attr(any(feature = "serde", feature = "metadata"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "serde", feature = "metadata"), serde(rename_all = "camelCase"))]
pub struct ConversionOptionsUpdate {
    pub heading_style: Option<HeadingStyle>,
    pub list_indent_type: Option<ListIndentType>,
    pub list_indent_width: Option<usize>,
    pub bullets: Option<String>,
    pub strong_em_symbol: Option<char>,
    pub escape_asterisks: Option<bool>,
    pub escape_underscores: Option<bool>,
    pub escape_misc: Option<bool>,
    pub escape_ascii: Option<bool>,
    pub code_language: Option<String>,
    pub autolinks: Option<bool>,
    pub default_title: Option<bool>,
    pub br_in_tables: Option<bool>,
    pub hocr_spatial_tables: Option<bool>,
    pub highlight_style: Option<HighlightStyle>,
    pub extract_metadata: Option<bool>,
    pub whitespace_mode: Option<WhitespaceMode>,
    pub strip_newlines: Option<bool>,
    pub wrap: Option<bool>,
    pub wrap_width: Option<usize>,
    pub convert_as_inline: Option<bool>,
    pub sub_symbol: Option<String>,
    pub sup_symbol: Option<String>,
    pub newline_style: Option<NewlineStyle>,
    pub code_block_style: Option<CodeBlockStyle>,
    pub keep_inline_images_in: Option<Vec<String>>,
    pub preprocessing: Option<PreprocessingOptionsUpdate>,
    pub encoding: Option<String>,
    pub debug: Option<bool>,
    pub strip_tags: Option<Vec<String>>,
    pub preserve_tags: Option<Vec<String>>,
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

impl ConversionOptions {
    pub fn apply_update(&mut self, update: ConversionOptionsUpdate) {
        if let Some(heading_style) = update.heading_style {
            self.heading_style = heading_style;
        }
        if let Some(list_indent_type) = update.list_indent_type {
            self.list_indent_type = list_indent_type;
        }
        if let Some(list_indent_width) = update.list_indent_width {
            self.list_indent_width = list_indent_width;
        }
        if let Some(bullets) = update.bullets {
            self.bullets = bullets;
        }
        if let Some(strong_em_symbol) = update.strong_em_symbol {
            self.strong_em_symbol = strong_em_symbol;
        }
        if let Some(escape_asterisks) = update.escape_asterisks {
            self.escape_asterisks = escape_asterisks;
        }
        if let Some(escape_underscores) = update.escape_underscores {
            self.escape_underscores = escape_underscores;
        }
        if let Some(escape_misc) = update.escape_misc {
            self.escape_misc = escape_misc;
        }
        if let Some(escape_ascii) = update.escape_ascii {
            self.escape_ascii = escape_ascii;
        }
        if let Some(code_language) = update.code_language {
            self.code_language = code_language;
        }
        if let Some(autolinks) = update.autolinks {
            self.autolinks = autolinks;
        }
        if let Some(default_title) = update.default_title {
            self.default_title = default_title;
        }
        if let Some(br_in_tables) = update.br_in_tables {
            self.br_in_tables = br_in_tables;
        }
        if let Some(hocr_spatial_tables) = update.hocr_spatial_tables {
            self.hocr_spatial_tables = hocr_spatial_tables;
        }
        if let Some(highlight_style) = update.highlight_style {
            self.highlight_style = highlight_style;
        }
        if let Some(extract_metadata) = update.extract_metadata {
            self.extract_metadata = extract_metadata;
        }
        if let Some(whitespace_mode) = update.whitespace_mode {
            self.whitespace_mode = whitespace_mode;
        }
        if let Some(strip_newlines) = update.strip_newlines {
            self.strip_newlines = strip_newlines;
        }
        if let Some(wrap) = update.wrap {
            self.wrap = wrap;
        }
        if let Some(wrap_width) = update.wrap_width {
            self.wrap_width = wrap_width;
        }
        if let Some(convert_as_inline) = update.convert_as_inline {
            self.convert_as_inline = convert_as_inline;
        }
        if let Some(sub_symbol) = update.sub_symbol {
            self.sub_symbol = sub_symbol;
        }
        if let Some(sup_symbol) = update.sup_symbol {
            self.sup_symbol = sup_symbol;
        }
        if let Some(newline_style) = update.newline_style {
            self.newline_style = newline_style;
        }
        if let Some(code_block_style) = update.code_block_style {
            self.code_block_style = code_block_style;
        }
        if let Some(keep_inline_images_in) = update.keep_inline_images_in {
            self.keep_inline_images_in = keep_inline_images_in;
        }
        if let Some(preprocessing) = update.preprocessing {
            self.preprocessing.apply_update(preprocessing);
        }
        if let Some(encoding) = update.encoding {
            self.encoding = encoding;
        }
        if let Some(debug) = update.debug {
            self.debug = debug;
        }
        if let Some(strip_tags) = update.strip_tags {
            self.strip_tags = strip_tags;
        }
        if let Some(preserve_tags) = update.preserve_tags {
            self.preserve_tags = preserve_tags;
        }
    }

    pub fn from_update(update: ConversionOptionsUpdate) -> Self {
        let mut options = Self::default();
        options.apply_update(update);
        options
    }
}

impl From<ConversionOptionsUpdate> for ConversionOptions {
    fn from(update: ConversionOptionsUpdate) -> Self {
        Self::from_update(update)
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

/// Partial update for PreprocessingOptions.
#[derive(Debug, Clone, Default)]
#[cfg_attr(any(feature = "serde", feature = "metadata"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "serde", feature = "metadata"), serde(rename_all = "camelCase"))]
pub struct PreprocessingOptionsUpdate {
    pub enabled: Option<bool>,
    pub preset: Option<PreprocessingPreset>,
    pub remove_navigation: Option<bool>,
    pub remove_forms: Option<bool>,
}

fn normalize_token(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        }
    }
    out
}

#[cfg(any(feature = "serde", feature = "metadata"))]
mod serde_impls {
    use super::*;
    use serde::Deserialize;

    macro_rules! impl_deserialize_from_parse {
        ($ty:ty, $parser:expr) => {
            impl<'de> Deserialize<'de> for $ty {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let value = String::deserialize(deserializer)?;
                    Ok($parser(&value))
                }
            }
        };
    }

    impl_deserialize_from_parse!(HeadingStyle, HeadingStyle::parse);
    impl_deserialize_from_parse!(ListIndentType, ListIndentType::parse);
    impl_deserialize_from_parse!(WhitespaceMode, WhitespaceMode::parse);
    impl_deserialize_from_parse!(NewlineStyle, NewlineStyle::parse);
    impl_deserialize_from_parse!(CodeBlockStyle, CodeBlockStyle::parse);
    impl_deserialize_from_parse!(HighlightStyle, HighlightStyle::parse);
    impl_deserialize_from_parse!(PreprocessingPreset, PreprocessingPreset::parse);
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

impl PreprocessingOptions {
    pub fn apply_update(&mut self, update: PreprocessingOptionsUpdate) {
        if let Some(enabled) = update.enabled {
            self.enabled = enabled;
        }
        if let Some(preset) = update.preset {
            self.preset = preset;
        }
        if let Some(remove_navigation) = update.remove_navigation {
            self.remove_navigation = remove_navigation;
        }
        if let Some(remove_forms) = update.remove_forms {
            self.remove_forms = remove_forms;
        }
    }

    pub fn from_update(update: PreprocessingOptionsUpdate) -> Self {
        let mut options = Self::default();
        options.apply_update(update);
        options
    }
}

impl From<PreprocessingOptionsUpdate> for PreprocessingOptions {
    fn from(update: PreprocessingOptionsUpdate) -> Self {
        Self::from_update(update)
    }
}
