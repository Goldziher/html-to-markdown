//! Validation and parsing utilities for option enums.
//!
//! This module provides parsing and serialization logic for configuration
//! enums (`HeadingStyle`, `ListIndentType`, etc.) with string conversion support.

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
    #[cfg_attr(alef, alef(skip))]
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
    #[cfg_attr(alef, alef(skip))]
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
    #[cfg_attr(alef, alef(skip))]
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
    #[cfg_attr(alef, alef(skip))]
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
    /// Indented code blocks (4 spaces). `CommonMark` standard.
    Indented,
    /// Fenced code blocks with triple backticks. Default (GFM). Supports language hints.
    #[default]
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
    #[cfg_attr(alef, alef(skip))]
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
    #[cfg_attr(alef, alef(skip))]
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

/// Link rendering style in Markdown output.
///
/// Controls whether links and images use inline `[text](url)` syntax or
/// reference-style `[text][1]` syntax with definitions collected at the end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkStyle {
    /// Inline links: `[text](url)`. Default.
    #[default]
    Inline,
    /// Reference-style links: `[text][1]` with `[1]: url` at end of document.
    Reference,
}

impl LinkStyle {
    /// Parse a link style from a string.
    ///
    /// Accepts "reference" or defaults to Inline.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    #[cfg_attr(alef, alef(skip))]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "reference" => Self::Reference,
            _ => Self::Inline,
        }
    }
}

/// URL encoding strategy for link and image destinations.
///
/// Controls how special characters in URL destinations are handled when they
/// require escaping to produce valid Markdown.
///
/// The `Angle` variant (default) wraps the destination in angle brackets:
/// `[text](<url with spaces>)`. This is the CommonMark-specified escape hatch
/// but breaks when the URL itself contains `>`.
///
/// The `Percent` variant percent-encodes every character that is not an RFC 3986
/// unreserved character or `/`, producing a destination safe for all Markdown
/// parsers: `[text](url%20with%20spaces)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UrlEscapeStyle {
    /// Wrap destinations that contain spaces or newlines in angle brackets. Default.
    #[default]
    Angle,
    /// Percent-encode all characters that are not RFC 3986 unreserved or `/`.
    Percent,
}

impl UrlEscapeStyle {
    /// Parse a URL escape style from a string.
    ///
    /// Accepts "percent" or defaults to Angle.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    #[cfg_attr(alef, alef(skip))]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "percent" => Self::Percent,
            _ => Self::Angle,
        }
    }
}

/// What the output shows for an image, an embedded media element, or a link, whose address is
/// an inline `data:` URL.
///
/// Applies to `<img>` (including its lazy-load attributes, its `srcset` candidates and the
/// `<source>` elements of a `<picture>` around it), `<graphic>`, inline `<svg>`, `<video>`,
/// `<audio>` (including their nested `<source>` elements), `<iframe>`, and a link (`<a href>`)
/// whose own address is a `data:` URL.
///
/// With `AltTextOnly` or `DropElement`, an element that also has an address that is not `data:`
/// uses that address instead. The document structure follows the markdown: a dropped image has
/// no node, and an image written as its alt text has no address. A link whose only content the
/// choice removed is dropped with it.
///
/// A link has no attribute separate from its own text to use as a caption, so `AltTextOnly` and
/// `DropElement` do the same thing to a link's `data:` address: write the link's text with no
/// destination. Unlike an image, a link's text is never dropped along with the address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InlineDataMedia {
    /// Write the `data:` URL as the destination, payload included. Default.
    #[default]
    Keep,
    /// Write the alt text (the text of an `<svg>`, the fallback content for `<video>` and
    /// `<audio>`) without a destination.
    AltTextOnly,
    /// Write nothing for the element.
    DropElement,
}

impl InlineDataMedia {
    /// Parse the choice from a string.
    ///
    /// Accepts "alttextonly" or "dropelement" or defaults to Keep.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    #[cfg_attr(alef, alef(skip))]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "alttextonly" => Self::AltTextOnly,
            "dropelement" => Self::DropElement,
            _ => Self::Keep,
        }
    }
}

/// Which text that a browser does not show at first the output holds.
///
/// The choice answers one question: can a reader get to the text? It does not depend on how the
/// page hides it.
///
/// | Markup | `Drop` | `Reachable` | `All` |
/// | --- | --- | --- | --- |
/// | An element with the `hidden` attribute, any value (`hidden="until-found"` too) | dropped | kept | kept |
/// | An element with inline `display: none`, `visibility: hidden` or `font-size: 0` | dropped | kept | kept |
/// | An element of an inline `<svg>` with `display="none"` or `visibility="hidden"` | dropped | kept | kept |
/// | A declarative shadow root: a `<template>` whose `shadowrootmode` is `open` or `closed` | dropped | kept | kept |
/// | Any other `<template>`, and `<noscript>` | dropped | dropped | kept |
/// | `<script>`, `<style>`, comments, the value of `<input type="hidden">` | dropped | dropped | dropped |
///
/// `aria-hidden` is not hidden for this option. It takes an element away from assistive
/// technology only: a browser still shows the element, so a reader sees its text, and every
/// choice keeps it. One rule is older than this option and does not change with it: for an
/// inline `<svg aria-hidden="true">` no choice writes the title, the description or the label
/// of the graphic.
///
/// Every choice also keeps what the converter never treated as hidden: `inert`, a closed
/// `<details>` or `<dialog>`, `<datalist>` and `<option>` text, and an element hidden by a class
/// name, a style sheet rule, `opacity`, `content-visibility`, a zero size or an off-screen
/// position. The converter reads the inline `style` attribute only. It does no layout and reads no
/// style sheet, so it cannot tell that a rule in a style sheet hides an element.
///
/// A kept element converts like the same element without the attribute or style. Kept
/// `<template>` and `<noscript>` content converts where it is written, as if the two tags were
/// not there: a row in a `<template>` inside a `<table>` is a row of that table, and the content
/// of a declarative shadow root comes before the other children of its host element. Slots are
/// not resolved. `All` keeps `<noscript>` content with every preprocessing preset. A `<template>`
/// or `<noscript>` in the document head holds metadata (`<link>`, `<meta>`, `<style>`), not text
/// for a reader, and no choice keeps it. A document with no `<head>` tag has a head too: it
/// starts at the doctype, at the `<html>` tag or at the first metadata element, and it ends
/// where the body starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HiddenContent {
    /// Drop the text a browser does not show at first. Default.
    #[default]
    Drop,
    /// Keep the text of elements that are in the page and that the page can show: an inactive tab
    /// panel, a collapsed section, an answer shown on a click, a declarative shadow root.
    Reachable,
    /// Keep what `Reachable` keeps, and the content of `<template>` and `<noscript>`. A reader
    /// gets to that text only after a script copies it into the page, or with scripting off.
    All,
}

impl HiddenContent {
    /// Parse the choice from a string.
    ///
    /// Accepts "drop", "reachable" and "all".
    /// Input is normalized (lowercased, alphanumeric only), so "Reachable" and "ALL" are
    /// accepted too.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::ConfigError`] for any other value, the empty string too. A
    /// value with a typing error does not become `Drop`: that loses the text that the caller
    /// asked to keep, with no message.
    ///
    /// [`ConversionError::ConfigError`]: crate::error::ConversionError::ConfigError
    #[cfg_attr(alef, alef(skip))]
    pub fn parse(value: &str) -> Result<Self, crate::error::ConversionError> {
        match normalize_token(value).as_str() {
            "drop" => Ok(Self::Drop),
            "reachable" => Ok(Self::Reachable),
            "all" => Ok(Self::All),
            _ => Err(crate::error::ConversionError::ConfigError(format!(
                "hidden_content is \"{value}\". Use \"drop\", \"reachable\" or \"all\"."
            ))),
        }
    }
}

/// Output format for conversion.
///
/// Specifies the target markup language format for the conversion output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Standard Markdown (`CommonMark` compatible). Default.
    #[default]
    Markdown,
    /// Djot lightweight markup language.
    Djot,
    /// Plain text output (no markup, visible text only).
    Plain,
}

impl OutputFormat {
    /// Parse an output format from a string.
    ///
    /// Accepts "djot" or defaults to Markdown.
    /// Input is normalized (lowercased, alphanumeric only).
    #[must_use]
    #[cfg_attr(alef, alef(skip))]
    pub fn parse(value: &str) -> Self {
        match normalize_token(value).as_str() {
            "djot" => Self::Djot,
            "plain" | "plaintext" | "text" => Self::Plain,
            _ => Self::Markdown,
        }
    }
}

/// Normalize a configuration string by lowercasing and removing non-alphanumeric characters.
pub(crate) fn normalize_token(value: &str) -> String {
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
        CodeBlockStyle, HeadingStyle, HiddenContent, HighlightStyle, InlineDataMedia, LinkStyle, ListIndentType,
        NewlineStyle, OutputFormat, UrlEscapeStyle, WhitespaceMode,
    };
    use serde::{Deserialize, Serialize, Serializer};

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
    impl_deserialize_from_parse!(LinkStyle, LinkStyle::parse);
    impl_deserialize_from_parse!(UrlEscapeStyle, UrlEscapeStyle::parse);
    impl_deserialize_from_parse!(OutputFormat, OutputFormat::parse);
    impl_deserialize_from_parse!(InlineDataMedia, InlineDataMedia::parse);

    impl<'de> Deserialize<'de> for HiddenContent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = String::deserialize(deserializer)?;
            Self::parse(&value).map_err(serde::de::Error::custom)
        }
    }

    impl Serialize for HeadingStyle {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Underlined => "underlined",
                Self::Atx => "atx",
                Self::AtxClosed => "atxclosed",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for ListIndentType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Spaces => "spaces",
                Self::Tabs => "tabs",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for WhitespaceMode {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Normalized => "normalized",
                Self::Strict => "strict",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for NewlineStyle {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Spaces => "spaces",
                Self::Backslash => "backslash",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for CodeBlockStyle {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Indented => "indented",
                Self::Backticks => "backticks",
                Self::Tildes => "tildes",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for HighlightStyle {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::DoubleEqual => "doubleequal",
                Self::Html => "html",
                Self::Bold => "bold",
                Self::None => "none",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for LinkStyle {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Inline => "inline",
                Self::Reference => "reference",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for UrlEscapeStyle {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Angle => "angle",
                Self::Percent => "percent",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for InlineDataMedia {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Keep => "keep",
                Self::AltTextOnly => "alttextonly",
                Self::DropElement => "dropelement",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for HiddenContent {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Drop => "drop",
                Self::Reachable => "reachable",
                Self::All => "all",
            };
            serializer.serialize_str(s)
        }
    }

    impl Serialize for OutputFormat {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                Self::Markdown => "markdown",
                Self::Djot => "djot",
                Self::Plain => "plain",
            };
            serializer.serialize_str(s)
        }
    }
}

#[cfg(all(test, any(feature = "serde", feature = "metadata")))]
mod tests {
    use super::*;

    #[test]
    fn test_enum_serialization() {
        let heading = HeadingStyle::AtxClosed;
        let json = serde_json::to_string(&heading).expect("Failed to serialize");
        assert_eq!(json, r#""atxclosed""#);

        let list_indent = ListIndentType::Tabs;
        let json = serde_json::to_string(&list_indent).expect("Failed to serialize");
        assert_eq!(json, r#""tabs""#);

        let whitespace = WhitespaceMode::Strict;
        let json = serde_json::to_string(&whitespace).expect("Failed to serialize");
        assert_eq!(json, r#""strict""#);
    }

    #[test]
    fn test_enum_deserialization() {
        let heading: HeadingStyle = serde_json::from_str(r#""atxclosed""#).expect("Failed");
        assert_eq!(heading, HeadingStyle::AtxClosed);

        let heading: HeadingStyle = serde_json::from_str(r#""ATXCLOSED""#).expect("Failed");
        assert_eq!(heading, HeadingStyle::AtxClosed);

        let list_indent: ListIndentType = serde_json::from_str(r#""tabs""#).expect("Failed");
        assert_eq!(list_indent, ListIndentType::Tabs);
    }
}
