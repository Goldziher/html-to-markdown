//! Shared enum wrapper types for language bindings.
//!
//! These wrapper types provide a bridge between Rust enums and language-specific
//! enum representations. They support serde serialization/deserialization for
//! JSON parsing in bindings.

use html_to_markdown_rs::{
    CodeBlockStyle, HeadingStyle, HighlightStyle, ListIndentType, NewlineStyle, PreprocessingPreset, WhitespaceMode,
};
use serde::{Deserialize, Serialize};

/// Wrapper for `HeadingStyle` enum with serde support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadingStyleWrapper {
    /// Underlined headings (`====` and `----`)
    Underlined,
    /// ATX-style headings (`# H1`, `## H2`, etc.)
    Atx,
    /// ATX-style with closing hashes (`# H1 #`, `## H2 ##`, etc.)
    AtxClosed,
}

impl From<HeadingStyleWrapper> for HeadingStyle {
    fn from(wrapper: HeadingStyleWrapper) -> Self {
        match wrapper {
            HeadingStyleWrapper::Underlined => Self::Underlined,
            HeadingStyleWrapper::Atx => Self::Atx,
            HeadingStyleWrapper::AtxClosed => Self::AtxClosed,
        }
    }
}

impl From<HeadingStyle> for HeadingStyleWrapper {
    fn from(style: HeadingStyle) -> Self {
        match style {
            HeadingStyle::Underlined => Self::Underlined,
            HeadingStyle::Atx => Self::Atx,
            HeadingStyle::AtxClosed => Self::AtxClosed,
        }
    }
}

/// Wrapper for `ListIndentType` enum with serde support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListIndentTypeWrapper {
    /// Use spaces for indentation
    Spaces,
    /// Use tabs for indentation
    Tabs,
}

impl From<ListIndentTypeWrapper> for ListIndentType {
    fn from(wrapper: ListIndentTypeWrapper) -> Self {
        match wrapper {
            ListIndentTypeWrapper::Spaces => Self::Spaces,
            ListIndentTypeWrapper::Tabs => Self::Tabs,
        }
    }
}

impl From<ListIndentType> for ListIndentTypeWrapper {
    fn from(indent_type: ListIndentType) -> Self {
        match indent_type {
            ListIndentType::Spaces => Self::Spaces,
            ListIndentType::Tabs => Self::Tabs,
        }
    }
}

/// Wrapper for `WhitespaceMode` enum with serde support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhitespaceModeWrapper {
    /// Normalize whitespace (default)
    Normalized,
    /// Preserve strict whitespace
    Strict,
}

impl From<WhitespaceModeWrapper> for WhitespaceMode {
    fn from(wrapper: WhitespaceModeWrapper) -> Self {
        match wrapper {
            WhitespaceModeWrapper::Normalized => Self::Normalized,
            WhitespaceModeWrapper::Strict => Self::Strict,
        }
    }
}

impl From<WhitespaceMode> for WhitespaceModeWrapper {
    fn from(mode: WhitespaceMode) -> Self {
        match mode {
            WhitespaceMode::Normalized => Self::Normalized,
            WhitespaceMode::Strict => Self::Strict,
        }
    }
}

/// Wrapper for `NewlineStyle` enum with serde support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewlineStyleWrapper {
    /// Two spaces for line breaks
    Spaces,
    /// Backslash for line breaks
    Backslash,
}

impl From<NewlineStyleWrapper> for NewlineStyle {
    fn from(wrapper: NewlineStyleWrapper) -> Self {
        match wrapper {
            NewlineStyleWrapper::Spaces => Self::Spaces,
            NewlineStyleWrapper::Backslash => Self::Backslash,
        }
    }
}

impl From<NewlineStyle> for NewlineStyleWrapper {
    fn from(style: NewlineStyle) -> Self {
        match style {
            NewlineStyle::Spaces => Self::Spaces,
            NewlineStyle::Backslash => Self::Backslash,
        }
    }
}

/// Wrapper for `CodeBlockStyle` enum with serde support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeBlockStyleWrapper {
    /// Indented code blocks (4 spaces)
    Indented,
    /// Fenced code blocks with backticks
    Backticks,
    /// Fenced code blocks with tildes
    Tildes,
}

impl From<CodeBlockStyleWrapper> for CodeBlockStyle {
    fn from(wrapper: CodeBlockStyleWrapper) -> Self {
        match wrapper {
            CodeBlockStyleWrapper::Indented => Self::Indented,
            CodeBlockStyleWrapper::Backticks => Self::Backticks,
            CodeBlockStyleWrapper::Tildes => Self::Tildes,
        }
    }
}

impl From<CodeBlockStyle> for CodeBlockStyleWrapper {
    fn from(style: CodeBlockStyle) -> Self {
        match style {
            CodeBlockStyle::Indented => Self::Indented,
            CodeBlockStyle::Backticks => Self::Backticks,
            CodeBlockStyle::Tildes => Self::Tildes,
        }
    }
}

/// Wrapper for `HighlightStyle` enum with serde support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HighlightStyleWrapper {
    /// Double equals (`==highlighted==`)
    DoubleEqual,
    /// HTML `<mark>` tag
    Html,
    /// Bold/strong (`**highlighted**`)
    Bold,
    /// No highlighting
    None,
}

impl From<HighlightStyleWrapper> for HighlightStyle {
    fn from(wrapper: HighlightStyleWrapper) -> Self {
        match wrapper {
            HighlightStyleWrapper::DoubleEqual => Self::DoubleEqual,
            HighlightStyleWrapper::Html => Self::Html,
            HighlightStyleWrapper::Bold => Self::Bold,
            HighlightStyleWrapper::None => Self::None,
        }
    }
}

impl From<HighlightStyle> for HighlightStyleWrapper {
    fn from(style: HighlightStyle) -> Self {
        match style {
            HighlightStyle::DoubleEqual => Self::DoubleEqual,
            HighlightStyle::Html => Self::Html,
            HighlightStyle::Bold => Self::Bold,
            HighlightStyle::None => Self::None,
        }
    }
}

/// Wrapper for `PreprocessingPreset` enum with serde support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreprocessingPresetWrapper {
    /// Minimal preprocessing
    Minimal,
    /// Standard preprocessing (default)
    Standard,
    /// Aggressive preprocessing
    Aggressive,
}

impl From<PreprocessingPresetWrapper> for PreprocessingPreset {
    fn from(wrapper: PreprocessingPresetWrapper) -> Self {
        match wrapper {
            PreprocessingPresetWrapper::Minimal => Self::Minimal,
            PreprocessingPresetWrapper::Standard => Self::Standard,
            PreprocessingPresetWrapper::Aggressive => Self::Aggressive,
        }
    }
}

impl From<PreprocessingPreset> for PreprocessingPresetWrapper {
    fn from(preset: PreprocessingPreset) -> Self {
        match preset {
            PreprocessingPreset::Minimal => Self::Minimal,
            PreprocessingPreset::Standard => Self::Standard,
            PreprocessingPreset::Aggressive => Self::Aggressive,
        }
    }
}
