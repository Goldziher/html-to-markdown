// ~keep reason: CLI application modules do not expose docs to users; doc coverage not required
#![allow(missing_docs)]
// ~keep reason: enum names repeat the type name intentionally for clap ValueEnum derivation
// ~keep (e.g. CliHeadingStyle::Atx mirrors HeadingStyle::Atx one-to-one)
#![allow(clippy::enum_variant_names)]

use clap::ValueEnum;
use html_to_markdown_rs::{
    CodeBlockStyle, HeadingStyle, HiddenContent, HighlightStyle, InlineDataMedia, LinkStyle, ListIndentType,
    NewlineStyle, OutputFormat, PreprocessingPreset, TierStrategy, UrlEscapeStyle, WhitespaceMode,
};

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliHeadingStyle {
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
            CliHeadingStyle::Atx => Self::Atx,
            CliHeadingStyle::Underlined => Self::Underlined,
            CliHeadingStyle::AtxClosed => Self::AtxClosed,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliListIndentType {
    /// Use spaces for indentation
    Spaces,
    /// Use tabs for indentation
    Tabs,
}

impl From<CliListIndentType> for ListIndentType {
    fn from(indent_type: CliListIndentType) -> Self {
        match indent_type {
            CliListIndentType::Spaces => Self::Spaces,
            CliListIndentType::Tabs => Self::Tabs,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliNewlineStyle {
    /// Two spaces at end of line (default)
    Spaces,
    /// Backslash at end of line
    Backslash,
}

impl From<CliNewlineStyle> for NewlineStyle {
    fn from(style: CliNewlineStyle) -> Self {
        match style {
            CliNewlineStyle::Spaces => Self::Spaces,
            CliNewlineStyle::Backslash => Self::Backslash,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliCodeBlockStyle {
    /// Indented code blocks: 4 spaces
    Indented,
    /// Fenced code blocks (backtick style) (default)
    Backticks,
    /// Fenced code blocks: ~~~
    Tildes,
}

impl From<CliCodeBlockStyle> for CodeBlockStyle {
    fn from(style: CliCodeBlockStyle) -> Self {
        match style {
            CliCodeBlockStyle::Indented => Self::Indented,
            CliCodeBlockStyle::Backticks => Self::Backticks,
            CliCodeBlockStyle::Tildes => Self::Tildes,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliHighlightStyle {
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
            CliHighlightStyle::DoubleEqual => Self::DoubleEqual,
            CliHighlightStyle::Html => Self::Html,
            CliHighlightStyle::Bold => Self::Bold,
            CliHighlightStyle::None => Self::None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliWhitespaceMode {
    /// Normalize whitespace (default)
    Normalized,
    /// Preserve whitespace as-is
    Strict,
}

impl From<CliWhitespaceMode> for WhitespaceMode {
    fn from(mode: CliWhitespaceMode) -> Self {
        match mode {
            CliWhitespaceMode::Normalized => Self::Normalized,
            CliWhitespaceMode::Strict => Self::Strict,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliPreprocessingPreset {
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
            CliPreprocessingPreset::Minimal => Self::Minimal,
            CliPreprocessingPreset::Standard => Self::Standard,
            CliPreprocessingPreset::Aggressive => Self::Aggressive,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliOutputFormat {
    /// Standard Markdown (`CommonMark` compatible)
    Markdown,
    /// Djot lightweight markup language
    Djot,
    /// Plain text (no markup)
    Plain,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(format: CliOutputFormat) -> Self {
        match format {
            CliOutputFormat::Markdown => Self::Markdown,
            CliOutputFormat::Djot => Self::Djot,
            CliOutputFormat::Plain => Self::Plain,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliLinkStyle {
    /// Inline links: [text](url) (default)
    Inline,
    /// Reference-style links: [text][1] with definitions at end
    Reference,
}

impl From<CliLinkStyle> for LinkStyle {
    fn from(style: CliLinkStyle) -> Self {
        match style {
            CliLinkStyle::Inline => Self::Inline,
            CliLinkStyle::Reference => Self::Reference,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliInlineDataMedia {
    /// Write the data: URL with its payload (default)
    Keep,
    /// Write the alt text without a destination
    AltTextOnly,
    /// Write nothing for the element
    DropElement,
}

impl From<CliInlineDataMedia> for InlineDataMedia {
    fn from(choice: CliInlineDataMedia) -> Self {
        match choice {
            CliInlineDataMedia::Keep => Self::Keep,
            CliInlineDataMedia::AltTextOnly => Self::AltTextOnly,
            CliInlineDataMedia::DropElement => Self::DropElement,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliHiddenContent {
    /// Keep none of the text a browser does not show at first (default)
    Drop,
    /// Keep elements hidden by the hidden attribute or an inline style, and declarative shadow roots
    Reachable,
    /// Also keep the content of template and noscript elements
    All,
}

impl From<CliHiddenContent> for HiddenContent {
    fn from(choice: CliHiddenContent) -> Self {
        match choice {
            CliHiddenContent::Drop => Self::Drop,
            CliHiddenContent::Reachable => Self::Reachable,
            CliHiddenContent::All => Self::All,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliUrlEscapeStyle {
    /// Wrap destinations containing spaces or newlines in angle brackets (default)
    Angle,
    /// Percent-encode all characters that are not RFC 3986 unreserved or `/`
    Percent,
}

impl From<CliUrlEscapeStyle> for UrlEscapeStyle {
    fn from(style: CliUrlEscapeStyle) -> Self {
        match style {
            CliUrlEscapeStyle::Angle => Self::Angle,
            CliUrlEscapeStyle::Percent => Self::Percent,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliTierStrategy {
    /// Automatically pick the best conversion tier for the input (default)
    Auto,
    /// Always use the Tier-2 (DOM-walk) path, skipping Tier-1
    #[value(name = "tier2")]
    Tier2,
}

impl From<CliTierStrategy> for TierStrategy {
    fn from(strategy: CliTierStrategy) -> Self {
        match strategy {
            CliTierStrategy::Auto => Self::Auto,
            CliTierStrategy::Tier2 => Self::Tier2,
        }
    }
}

pub fn validate_bullets(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("bullets cannot be empty".to_string());
    }
    if s.len() > 10 {
        return Err("bullets string too long (max 10 characters)".to_string());
    }
    Ok(s.to_string())
}

pub fn validate_strong_em_symbol(s: &str) -> Result<char, String> {
    if s.len() != 1 {
        return Err("strong_em_symbol must be exactly one character".to_string());
    }
    let c = s.chars().next().expect("length already validated");
    if c != '*' && c != '_' {
        return Err("strong_em_symbol must be '*' or '_'".to_string());
    }
    Ok(c)
}
