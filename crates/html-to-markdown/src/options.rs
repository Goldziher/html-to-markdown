#![allow(clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::unused_self)]
//! Configuration options for HTML to Markdown conversion.

/// Heading style options for Markdown output.
///
/// Controls how headings (h1-h6) are rendered in the output Markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadingStyle {
    /// Underlined style (=== for h1, --- for h2).
    Underlined,
    /// ATX style (# for h1, ## for h2, etc.). Default.
    #[default]
    Atx,
    /// ATX closed style (# title #, with closing hashes).
    AtxClosed,
}

impl HeadingStyle {
    /// Parse a heading style from a string.
    ///
    /// Accepts "atx", "atxclosed", or defaults to Underlined.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "atx" => Self::Atx,
            "atxclosed" => Self::AtxClosed,
            _ => Self::Underlined,
        }
    }
}

/// List indentation character type.
///
/// Controls whether list items are indented with spaces or tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListIndentType {
    /// Use spaces for indentation. Default. Width controlled by `list_indent_width`.
    #[default]
    Spaces,
    /// Use tabs for indentation.
    Tabs,
}

impl ListIndentType {
    /// Parse a list indentation type from a string.
    ///
    /// Accepts "tabs" or defaults to Spaces.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "tabs" => Self::Tabs,
            _ => Self::Spaces,
        }
    }
}

/// Whitespace handling strategy during conversion.
///
/// Determines how sequences of whitespace characters (spaces, tabs, newlines) are processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhitespaceMode {
    /// Collapse multiple whitespace characters to single spaces. Default. Matches browser behavior.
    #[default]
    Normalized,
    /// Preserve all whitespace exactly as it appears in the HTML.
    Strict,
}

impl WhitespaceMode {
    /// Parse a whitespace mode from a string.
    ///
    /// Accepts "strict" or defaults to Normalized.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "strict" => Self::Strict,
            _ => Self::Normalized,
        }
    }
}

/// Line break syntax in Markdown output.
///
/// Controls how soft line breaks (from `<br>` or line breaks in source) are rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewlineStyle {
    /// Two trailing spaces at end of line. Default. Standard Markdown syntax.
    #[default]
    Spaces,
    /// Backslash at end of line. Alternative Markdown syntax.
    Backslash,
}

impl NewlineStyle {
    /// Parse a newline style from a string.
    ///
    /// Accepts "backslash" or defaults to Spaces.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "backslash" => Self::Backslash,
            _ => Self::Spaces,
        }
    }
}

/// Code block fence style in Markdown output.
///
/// Determines how code blocks (`<pre><code>`) are rendered in Markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeBlockStyle {
    /// Indented code blocks (4 spaces). Default. `CommonMark` standard.
    #[default]
    Indented,
    /// Fenced code blocks with backticks (```). Supports language hints.
    Backticks,
    /// Fenced code blocks with tildes (~~~). Supports language hints.
    Tildes,
}

impl CodeBlockStyle {
    /// Parse a code block style from a string.
    ///
    /// Accepts "backticks", "tildes", or defaults to Indented.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "backticks" => Self::Backticks,
            "tildes" => Self::Tildes,
            _ => Self::Indented,
        }
    }
}

/// Highlight rendering style for `<mark>` elements.
///
/// Controls how highlighted text is rendered in Markdown output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HighlightStyle {
    /// Double equals syntax (==text==). Default. Pandoc-compatible.
    #[default]
    DoubleEqual,
    /// Preserve as HTML (==text==). Original HTML tag.
    Html,
    /// Render as bold (**text**). Uses strong emphasis.
    Bold,
    /// Strip formatting, render as plain text. No markup.
    None,
}

impl HighlightStyle {
    /// Parse a highlight style from a string.
    ///
    /// Accepts "doubleequal", "html", "bold", "none", or defaults to None.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
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

/// HTML preprocessing aggressiveness level.
///
/// Controls the extent of cleanup performed before conversion. Higher levels remove more elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreprocessingPreset {
    /// Minimal cleanup. Remove only essential noise (scripts, styles).
    Minimal,
    /// Standard cleanup. Default. Removes navigation, forms, and other auxiliary content.
    #[default]
    Standard,
    /// Aggressive cleanup. Remove extensive non-content elements and structure.
    Aggressive,
}

impl PreprocessingPreset {
    /// Parse a preprocessing preset from a string.
    ///
    /// Accepts "minimal", "aggressive", or defaults to Standard.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "minimal" => Self::Minimal,
            "aggressive" => Self::Aggressive,
            _ => Self::Standard,
        }
    }
}

/// Main conversion options for HTML to Markdown conversion.
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Heading style (Underlined, Atx, `AtxClosed`)
    pub heading_style: HeadingStyle,

    /// List indentation type (Spaces or Tabs)
    pub list_indent_type: ListIndentType,

    /// List indentation width in spaces (applied if using spaces indentation)
    pub list_indent_width: usize,

    /// Bullet characters for unordered lists (e.g., "-", "*", "+")
    pub bullets: String,

    /// Symbol for strong/emphasis emphasis rendering (* or _)
    pub strong_em_symbol: char,

    /// Escape asterisks (*) in text to prevent accidental formatting
    pub escape_asterisks: bool,

    /// Escape underscores (_) in text to prevent accidental formatting
    pub escape_underscores: bool,

    /// Escape miscellaneous markdown characters (\ & < ` [ > ~ # = + | -)
    pub escape_misc: bool,

    /// Escape all ASCII punctuation characters (for `CommonMark` spec compliance tests)
    pub escape_ascii: bool,

    /// Default code language for fenced code blocks when not specified
    pub code_language: String,

    /// Use autolinks syntax for bare URLs (<http://example.com>)
    pub autolinks: bool,

    /// Add default title element to HTML if none exists before conversion
    pub default_title: bool,

    /// Use HTML <br> elements in tables instead of spaces for line breaks
    pub br_in_tables: bool,

    /// Enable spatial table reconstruction in hOCR documents (via spatial positioning analysis)
    pub hocr_spatial_tables: bool,

    /// Highlight style for <mark> elements (`DoubleEqual`, Html, Bold, None)
    pub highlight_style: HighlightStyle,

    /// Extract metadata from HTML (title, description, images, links, etc.)
    pub extract_metadata: bool,

    /// Whitespace handling mode (Normalized collapses multiple spaces, Strict preserves)
    pub whitespace_mode: WhitespaceMode,

    /// Strip newline characters from HTML before processing
    pub strip_newlines: bool,

    /// Enable automatic text wrapping at `wrap_width`
    pub wrap: bool,

    /// Text wrapping width in characters (default 80)
    pub wrap_width: usize,

    /// Treat block-level elements as inline during conversion
    pub convert_as_inline: bool,

    /// Custom symbol for subscript content (e.g., "~")
    pub sub_symbol: String,

    /// Custom symbol for superscript content (e.g., "^")
    pub sup_symbol: String,

    /// Newline style in markdown output (Spaces adds two spaces, Backslash adds \)
    pub newline_style: NewlineStyle,

    /// Code block fence style (Indented, Backticks, Tildes)
    pub code_block_style: CodeBlockStyle,

    /// HTML elements where images should remain as markdown links (not converted to alt text)
    pub keep_inline_images_in: Vec<String>,

    /// HTML preprocessing options (remove nav, forms, etc.)
    pub preprocessing: PreprocessingOptions,

    /// Source document encoding (informational, typically "utf-8")
    pub encoding: String,

    /// Enable debug mode with diagnostic warnings on conversion issues
    pub debug: bool,

    /// HTML tags to strip (extract text content, no markdown conversion)
    pub strip_tags: Vec<String>,

    /// HTML tags to preserve as-is in output (keep original HTML, useful for complex tables)
    pub preserve_tags: Vec<String>,
}

/// Partial update for `ConversionOptions`.
///
/// This struct uses `Option<T>` to represent optional fields that can be selectively updated.
/// Only specified fields (Some values) will override existing options; None values leave the
/// corresponding fields unchanged when applied via [`ConversionOptions::apply_update`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(any(feature = "serde", feature = "metadata"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "serde", feature = "metadata"), serde(rename_all = "camelCase"))]
pub struct ConversionOptionsUpdate {
    /// Optional heading style override (Underlined, Atx, `AtxClosed`)
    pub heading_style: Option<HeadingStyle>,

    /// Optional list indentation type override (Spaces or Tabs)
    pub list_indent_type: Option<ListIndentType>,

    /// Optional list indentation width override in spaces
    pub list_indent_width: Option<usize>,

    /// Optional bullet characters override for unordered lists
    pub bullets: Option<String>,

    /// Optional strong/emphasis symbol override (* or _)
    pub strong_em_symbol: Option<char>,

    /// Optional asterisk escaping override in text content
    pub escape_asterisks: Option<bool>,

    /// Optional underscore escaping override in text content
    pub escape_underscores: Option<bool>,

    /// Optional miscellaneous character escaping override (\ & < ` [ > ~ # = + | -)
    pub escape_misc: Option<bool>,

    /// Optional ASCII punctuation escaping override (for spec compliance testing)
    pub escape_ascii: Option<bool>,

    /// Optional default code language override for fenced code blocks
    pub code_language: Option<String>,

    /// Optional autolinks syntax override for bare URLs
    pub autolinks: Option<bool>,

    /// Optional default title element injection override
    pub default_title: Option<bool>,

    /// Optional HTML <br> usage in tables override
    pub br_in_tables: Option<bool>,

    /// Optional spatial table reconstruction for hOCR documents override
    pub hocr_spatial_tables: Option<bool>,

    /// Optional highlight style override for <mark> elements
    pub highlight_style: Option<HighlightStyle>,

    /// Optional metadata extraction override (title, description, images, links)
    pub extract_metadata: Option<bool>,

    /// Optional whitespace handling mode override (Normalized or Strict)
    pub whitespace_mode: Option<WhitespaceMode>,

    /// Optional newline stripping override before processing
    pub strip_newlines: Option<bool>,

    /// Optional automatic text wrapping override
    pub wrap: Option<bool>,

    /// Optional text wrapping width override in characters
    pub wrap_width: Option<usize>,

    /// Optional block-level to inline conversion override
    pub convert_as_inline: Option<bool>,

    /// Optional subscript symbol override
    pub sub_symbol: Option<String>,

    /// Optional superscript symbol override
    pub sup_symbol: Option<String>,

    /// Optional newline style override for markdown output
    pub newline_style: Option<NewlineStyle>,

    /// Optional code block fence style override (Indented, Backticks, Tildes)
    pub code_block_style: Option<CodeBlockStyle>,

    /// Optional context elements where images remain as markdown links override
    pub keep_inline_images_in: Option<Vec<String>>,

    /// Optional preprocessing options partial update
    pub preprocessing: Option<PreprocessingOptionsUpdate>,

    /// Optional source document encoding override
    pub encoding: Option<String>,

    /// Optional debug mode override for diagnostic warnings
    pub debug: Option<bool>,

    /// Optional HTML tags to strip override (extract text, no conversion)
    pub strip_tags: Option<Vec<String>>,

    /// Optional HTML tags to preserve as-is override in output
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
    /// Apply a partial update to these conversion options.
    ///
    /// Any specified fields in the update will override the current values.
    /// Unspecified fields (None) are left unchanged.
    ///
    /// # Arguments
    ///
    /// * `update` - Partial options update with fields to override
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

    /// Create new conversion options from a partial update.
    ///
    /// Creates a new `ConversionOptions` struct with defaults, then applies the update.
    /// Fields not specified in the update keep their default values.
    ///
    /// # Arguments
    ///
    /// * `update` - Partial options update with fields to set
    ///
    /// # Returns
    ///
    /// New `ConversionOptions` with specified updates applied to defaults
    #[must_use]
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

/// HTML preprocessing options for document cleanup before conversion.
#[derive(Debug, Clone)]
pub struct PreprocessingOptions {
    /// Enable HTML preprocessing globally
    pub enabled: bool,

    /// Preprocessing preset level (Minimal, Standard, Aggressive)
    pub preset: PreprocessingPreset,

    /// Remove navigation elements (nav, breadcrumbs, menus, sidebars)
    pub remove_navigation: bool,

    /// Remove form elements (forms, inputs, buttons, etc.)
    pub remove_forms: bool,
}

/// Partial update for `PreprocessingOptions`.
///
/// This struct uses `Option<T>` to represent optional fields that can be selectively updated.
/// Only specified fields (Some values) will override existing options; None values leave the
/// corresponding fields unchanged when applied via [`PreprocessingOptions::apply_update`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(any(feature = "serde", feature = "metadata"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "serde", feature = "metadata"), serde(rename_all = "camelCase"))]
pub struct PreprocessingOptionsUpdate {
    /// Optional global preprocessing enablement override
    pub enabled: Option<bool>,

    /// Optional preprocessing preset level override (Minimal, Standard, Aggressive)
    pub preset: Option<PreprocessingPreset>,

    /// Optional navigation element removal override (nav, breadcrumbs, menus, sidebars)
    pub remove_navigation: Option<bool>,

    /// Optional form element removal override (forms, inputs, buttons, etc.)
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
    use super::{
        CodeBlockStyle, HeadingStyle, HighlightStyle, ListIndentType, NewlineStyle, PreprocessingPreset, WhitespaceMode,
    };
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
    /// Apply a partial update to these preprocessing options.
    ///
    /// Any specified fields in the update will override the current values.
    /// Unspecified fields (None) are left unchanged.
    ///
    /// # Arguments
    ///
    /// * `update` - Partial preprocessing options update
    #[allow(clippy::needless_pass_by_value)]
    pub const fn apply_update(&mut self, update: PreprocessingOptionsUpdate) {
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

    /// Create new preprocessing options from a partial update.
    ///
    /// Creates a new `PreprocessingOptions` struct with defaults, then applies the update.
    /// Fields not specified in the update keep their default values.
    ///
    /// # Arguments
    ///
    /// * `update` - Partial preprocessing options update
    ///
    /// # Returns
    ///
    /// New `PreprocessingOptions` with specified updates applied to defaults
    #[must_use]
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
