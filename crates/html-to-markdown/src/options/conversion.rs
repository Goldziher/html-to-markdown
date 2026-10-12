//! Main conversion options with builder pattern.

use crate::options::preprocessing::PreprocessingOptions;
use crate::options::validation::{
    CodeBlockStyle, HeadingStyle, HighlightStyle, InlineDataMedia, LinkStyle, ListIndentType, NewlineStyle,
    OutputFormat, UrlEscapeStyle, WhitespaceMode,
};

/// Native recursion guard used as the default when the caller does not set an
/// explicit `max_depth`. Kept conservative so untrusted input cannot overflow
/// the process stack.
pub(crate) const NATIVE_STACK_SAFE_DEPTH: usize = 64;

/// Absolute ceiling a caller may raise `max_depth` to. Callers can exceed the
/// conservative native default (issue #434) but not this backstop, which still
/// guards the recursive walker against a genuine stack overflow.
pub(crate) const MAX_CONFIGURABLE_DEPTH: usize = 1024;

/// Default HTML input limit for WebAssembly builds: 2 MiB. ~keep
pub const DEFAULT_WASM_MAX_INPUT_SIZE: u64 = 2 * 1024 * 1024;

#[cfg(target_arch = "wasm32")]
const fn default_max_input_size() -> Option<u64> {
    Some(DEFAULT_WASM_MAX_INPUT_SIZE)
}

#[cfg(not(target_arch = "wasm32"))]
const fn default_max_input_size() -> Option<u64> {
    None
}

/// Controls which conversion tier is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(
    any(feature = "serde", feature = "metadata"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(any(feature = "serde", feature = "metadata"), serde(rename_all = "snake_case"))]
pub enum TierStrategy {
    /// Automatically pick the best tier for the input (default).
    ///
    /// Runs the classifier against the prescan report and uses Tier-1 when
    /// eligible; falls back to Tier-2 on bail or when the classifier routes
    /// to Tier-2.
    #[default]
    Auto,
    /// Always use the Tier-2 (`tl::parse` + walk) path, skipping Tier-1.
    Tier2,
    /// Force the Tier-1 byte scanner; if it bails, fall back to Tier-2.
    /// Testkit-only; not stable API.
    #[cfg(any(test, feature = "testkit"))]
    Tier1,
}

/// Main conversion options for HTML to Markdown conversion.
///
/// Use [`ConversionOptions::builder()`] to construct, or [`Default::default()`] for defaults.
///
/// # Example
///
/// ```rust
/// use html_to_markdown_rs::{ConversionOptions, HeadingStyle};
///
/// let options = ConversionOptions::builder()
///     .heading_style(HeadingStyle::Atx)
///     .wrap(true)
///     .wrap_width(100)
///     .build();
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(
    any(feature = "serde", feature = "metadata"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(any(feature = "serde", feature = "metadata"), serde(default, deny_unknown_fields))]
pub struct ConversionOptions {
    /// Heading style to use in Markdown output (ATX `#` or Setext underline).
    pub heading_style: HeadingStyle,
    /// How to indent nested list items (spaces or tab).
    pub list_indent_type: ListIndentType,
    /// Number of spaces (or tabs) to use for each level of list indentation.
    pub list_indent_width: usize,
    /// Bullet character(s) to use for unordered list items (e.g. `"-"`, `"*"`).
    pub bullets: String,
    /// Character used for bold/italic emphasis markers (`*` or `_`).
    pub strong_em_symbol: char,
    /// Escape `*` characters in plain text to avoid unintended bold/italic.
    pub escape_asterisks: bool,
    /// Escape `_` characters in plain text to avoid unintended bold/italic.
    pub escape_underscores: bool,
    /// Escape miscellaneous Markdown metacharacters (`[]()#` etc.) in plain text.
    pub escape_misc: bool,
    /// Escape ASCII characters that have special meaning in certain Markdown dialects.
    pub escape_ascii: bool,
    /// Default language annotation for fenced code blocks that have no language hint.
    pub code_language: String,
    /// Automatically convert bare URLs into Markdown autolinks.
    pub autolinks: bool,
    /// Append the title of an `<abbr>` in parentheses (default `true`).
    pub expand_abbreviations: bool,
    /// Append the `cite` URL after a blockquote (default `true`).
    pub include_blockquote_citations: bool,
    /// Emit a default title when no `<title>` tag is present.
    pub default_title: bool,
    /// Render `<br>` elements inside table cells as literal line breaks.
    pub br_in_tables: bool,
    /// Emit tables without column padding (compact GFM format).
    ///
    /// When `true`, column widths are not computed and cells are emitted with
    /// no trailing spaces. Separator rows use exactly `---` per column.
    /// Produces token-efficient output suitable for RAG / LLM contexts.
    ///
    /// Default `false` (aligned padding preserved).
    pub compact_tables: bool,
    /// Style used for `<mark>` / highlighted text (e.g. `==text==`).
    pub highlight_style: HighlightStyle,
    /// Populate `result.metadata` with `<head>` / `<meta>` extraction
    /// (title, description, Open Graph, Twitter Card, JSON-LD, …).
    ///
    /// Default `true`. Disabling skips the metadata pass only — table
    /// extraction into `result.tables` runs unconditionally.
    pub extract_metadata: bool,
    /// Controls how whitespace sequences are normalised in the converted output.
    ///
    /// - [`WhitespaceMode::Normalized`] (default) — collapses consecutive whitespace characters
    ///   (spaces, tabs, newlines) to a single space, matching browser rendering behaviour.
    /// - [`WhitespaceMode::Strict`] — preserves all whitespace exactly as it appears in the
    ///   source HTML, including runs of spaces and embedded newlines.
    ///
    /// Choose `Strict` only when the source HTML uses deliberate whitespace (e.g. pre-formatted
    /// content outside `<pre>` tags). For most documents `Normalized` produces cleaner output.
    pub whitespace_mode: WhitespaceMode,
    /// Strip all newlines from the output, producing a single-line result.
    pub strip_newlines: bool,
    /// Wrap long lines at [`wrap_width`](Self::wrap_width) characters.
    pub wrap: bool,
    /// Maximum output line width in characters when [`wrap`](Self::wrap) is `true` (default `80`).
    ///
    /// Lines are broken at word boundaries so that no line exceeds this length. A value of `0`
    /// is treated as "no limit" — equivalent to leaving [`wrap`](Self::wrap) disabled. Has no
    /// effect when `wrap` is `false`.
    pub wrap_width: usize,
    /// Treat the entire document as inline content (no block-level wrappers).
    pub convert_as_inline: bool,
    /// Markdown notation for subscript text (e.g. `"~"`).
    pub sub_symbol: String,
    /// Markdown notation for superscript text (e.g. `"^"`).
    pub sup_symbol: String,
    /// How to encode hard line breaks (`<br>`) in Markdown.
    pub newline_style: NewlineStyle,
    /// Style used for fenced code blocks (backticks or tilde).
    pub code_block_style: CodeBlockStyle,
    /// HTML tag names whose `<img>` children are kept inline instead of block.
    pub keep_inline_images_in: Vec<String>,
    /// Options for the HTML pre-processing pass applied before conversion begins.
    ///
    /// Pre-processing runs before the HTML is handed to the converter and can perform operations
    /// such as unwrapping redundant wrapper elements, removing tracking pixels, and normalising
    /// vendor-specific markup. See [`PreprocessingOptions`] for the full set of knobs.
    ///
    /// Defaults to [`PreprocessingOptions::default()`], which enables the standard cleaning
    /// passes. Set individual fields on [`PreprocessingOptions`] (or construct via
    /// [`ConversionOptions::builder`]) to opt in or out of specific passes.
    pub preprocessing: PreprocessingOptions,
    /// Expected character encoding of the input HTML (default `"utf-8"`).
    pub encoding: String,
    /// Emit debug information during conversion.
    pub debug: bool,
    /// HTML tag names whose content is stripped from the output entirely.
    pub strip_tags: Vec<String>,
    /// HTML tag names that are preserved verbatim in the output.
    pub preserve_tags: Vec<String>,
    /// Skip conversion of `<img>` elements (omit images from output).
    pub skip_images: bool,
    /// What the output shows for an image or embedded media element whose address is an inline
    /// `data:` URL: the URL with its payload ([`InlineDataMedia::Keep`], the default), the alt
    /// text alone, or nothing. See [`InlineDataMedia`] for the elements it covers.
    pub inline_data_media: InlineDataMedia,
    /// URL encoding strategy for link and image destinations.
    ///
    /// Controls how special characters in URL destinations are escaped:
    /// - [`UrlEscapeStyle::Angle`] (default) — wraps the destination in angle brackets when it
    ///   contains spaces or newlines. Some parsers misinterpret `>` inside such a destination.
    /// - [`UrlEscapeStyle::Percent`] — percent-encodes every character that is not an RFC 3986
    ///   unreserved character or `/`, producing a destination that all Markdown parsers handle
    ///   correctly even when the URL contains `<`, `>`, spaces, or parentheses.
    pub url_escape_style: UrlEscapeStyle,
    /// Link rendering style (inline or reference).
    pub link_style: LinkStyle,
    /// Target output format (Markdown, plain text, etc.).
    pub output_format: OutputFormat,
    /// Include structured document tree in result.
    pub include_document_structure: bool,
    /// Extract inline images from data URIs and SVGs.
    pub extract_images: bool,
    /// Maximum decoded image size in bytes (default 5MB).
    pub max_image_size: u64,
    /// Maximum accepted HTML input size in bytes.
    ///
    /// WebAssembly builds default to [`DEFAULT_WASM_MAX_INPUT_SIZE`] (2 MiB) to avoid
    /// an uncatchable stack-exhaustion trap on unusually large DOMs. Native builds default
    /// to `None`. Set `Some(bytes)` to choose another limit or `None` to disable it.
    pub max_input_size: Option<u64>,
    /// Capture SVG elements as images.
    pub capture_svg: bool,
    /// Infer image dimensions from data.
    pub infer_dimensions: bool,
    /// Maximum DOM traversal depth.
    ///
    /// `None` uses the library's internal native-stack safety limit. Explicit
    /// values above that safety limit are clamped to prevent process-aborting
    /// stack overflows on pathologically deep DOM trees.
    pub max_depth: Option<usize>,
    /// CSS selectors for elements to exclude entirely (element + all content).
    ///
    /// Unlike `strip_tags` (which removes the tag wrapper but keeps children),
    /// excluded elements and all their descendants are dropped from the output.
    /// Supports any CSS selector that `tl` supports: tag names, `.class`,
    /// `#id`, `[attribute]`, etc.
    ///
    /// Invalid selectors are silently skipped at conversion time.
    ///
    /// Example: `vec![".cookie-banner".into(), "#ad-container".into(), "[role='complementary']".into()]`
    #[cfg_attr(any(feature = "serde", feature = "metadata"), serde(default))]
    pub exclude_selectors: Vec<String>,

    /// Which conversion tier to use.
    ///
    /// - [`TierStrategy::Auto`] (default) — automatically choose the best path.
    /// - [`TierStrategy::Tier2`] — always use the Tier-2 DOM-walk path.
    /// - `TierStrategy::Tier1` — always attempt Tier-1 (testkit only).
    #[cfg_attr(any(feature = "serde", feature = "metadata"), serde(default))]
    pub tier_strategy: TierStrategy,

    /// Base URL to resolve relative `href`/`src` destinations against.
    ///
    /// When set, every relative link and image/media destination (`a href`, `img src`
    /// and its lazy-load fallbacks, `srcset`, `graphic` `url`/`href`/`xlink:href`/`src`,
    /// `iframe`/`audio`/`video`/`source` `src`) is resolved to an absolute URL before
    /// being written to the Markdown output, so the result is followable without the
    /// reader knowing where the source HTML came from.
    ///
    /// A `<base href>` in the document, if present, is honored the way a browser
    /// honors it: it is itself resolved against `base_url`, and that combined result
    /// becomes the effective base every other relative reference resolves against.
    ///
    /// Non-hierarchical schemes (`mailto:`, `tel:`, `javascript:`, `data:`) and
    /// already-absolute URLs are left unchanged. An unset (default) or unparseable
    /// `base_url`, and a relative reference that fails to resolve, leave the original
    /// attribute text unchanged -- this option never panics and never corrupts a
    /// destination it cannot confidently resolve.
    ///
    /// Default `None` — no resolution, output byte-identical to versions before this
    /// option existed.
    pub base_url: Option<String>,

    /// Optional visitor for custom traversal logic.
    ///
    /// When set, the visitor's callbacks are invoked for matching HTML elements
    /// during conversion, allowing custom output, skipping, or HTML preservation.
    /// See [`crate::visitor::HtmlVisitor`].
    #[cfg(feature = "visitor")]
    #[cfg_attr(any(feature = "serde", feature = "metadata"), serde(skip))]
    // ~keep Must stay bindable. It looks unbindable from the Rust side -- it holds a live
    // ~keep handle with no default a target language could reproduce -- but that is not what
    // ~keep alef emits: each target gets its own callback-object type (node emits
    // ~keep `Option<Object>`), and the `[crates.*] visitor_callbacks` bridge reads THIS field
    // ~keep to construct the handle. Marking it `alef(skip)` removes it from the generated
    // ~keep options mirror while the bridge still reads it, so node and py stop compiling.
    pub visitor: Option<crate::visitor::VisitorHandle>,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            heading_style: HeadingStyle::default(),
            list_indent_type: ListIndentType::default(),
            list_indent_width: 2,
            bullets: "-*+".to_string(),
            strong_em_symbol: '*',
            escape_asterisks: false,
            escape_underscores: false,
            escape_misc: false,
            escape_ascii: false,
            code_language: String::new(),
            autolinks: true,
            expand_abbreviations: true,
            include_blockquote_citations: true,
            default_title: false,
            br_in_tables: false,
            compact_tables: false,
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
            skip_images: false,
            inline_data_media: InlineDataMedia::Keep,
            url_escape_style: UrlEscapeStyle::default(),
            link_style: LinkStyle::default(),
            output_format: OutputFormat::default(),
            include_document_structure: false,
            extract_images: false,
            max_image_size: 5_242_880,
            max_input_size: default_max_input_size(),
            capture_svg: false,
            infer_dimensions: true,
            max_depth: None,
            exclude_selectors: Vec::new(),
            tier_strategy: TierStrategy::Auto,
            base_url: None,
            #[cfg(feature = "visitor")]
            visitor: None,
        }
    }
}

impl ConversionOptions {
    /// Create a [`ConversionOptionsBuilder`] pre-populated with default values.
    ///
    /// All fields start at their documented defaults. Use the setter methods on the returned
    /// builder to override individual fields, then call [`ConversionOptionsBuilder::build`] to
    /// produce the final [`ConversionOptions`].
    ///
    /// No fields are required — calling `.build()` immediately yields a valid options struct
    /// identical to [`ConversionOptions::default()`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use html_to_markdown_rs::{ConversionOptions, options::validation::{HeadingStyle, WhitespaceMode}};
    ///
    /// let options = ConversionOptions::builder()
    ///     .heading_style(HeadingStyle::AtxClosed)
    ///     .wrap(true)
    ///     .wrap_width(100)
    ///     .whitespace_mode(WhitespaceMode::Normalized)
    ///     .build();
    ///
    /// assert_eq!(options.wrap_width, 100);
    /// ```
    #[must_use]
    #[cfg_attr(alef, alef(skip))]
    pub fn builder() -> ConversionOptionsBuilder {
        ConversionOptionsBuilder(Self::default())
    }
}

/// Builder for [`ConversionOptions`].
///
/// All fields start with default values. Call `.build()` to produce the final options.
#[derive(Debug, Clone)]
#[cfg_attr(alef, alef(skip))]
pub struct ConversionOptionsBuilder(ConversionOptions);

macro_rules! builder_setter {
    ($name:ident, $ty:ty) => {
        /// Set the value.
        #[must_use]
        pub const fn $name(mut self, value: $ty) -> Self {
            self.0.$name = value;
            self
        }
    };
}

macro_rules! builder_setter_into {
    ($name:ident, $ty:ty) => {
        /// Set the value.
        #[must_use]
        pub fn $name(mut self, value: impl Into<$ty>) -> Self {
            self.0.$name = value.into();
            self
        }
    };
}

#[cfg_attr(alef, alef(skip))]
impl ConversionOptionsBuilder {
    builder_setter!(output_format, OutputFormat);
    builder_setter!(include_document_structure, bool);
    builder_setter!(extract_metadata, bool);
    builder_setter!(extract_images, bool);

    builder_setter!(heading_style, HeadingStyle);
    builder_setter!(list_indent_type, ListIndentType);
    builder_setter!(list_indent_width, usize);
    builder_setter_into!(bullets, String);
    builder_setter!(strong_em_symbol, char);
    builder_setter!(code_block_style, CodeBlockStyle);
    builder_setter!(newline_style, NewlineStyle);
    builder_setter!(highlight_style, HighlightStyle);
    builder_setter_into!(code_language, String);
    builder_setter!(link_style, LinkStyle);
    builder_setter!(autolinks, bool);
    builder_setter!(expand_abbreviations, bool);
    builder_setter!(include_blockquote_citations, bool);
    builder_setter!(default_title, bool);
    builder_setter!(br_in_tables, bool);
    builder_setter!(compact_tables, bool);
    builder_setter_into!(sub_symbol, String);
    builder_setter_into!(sup_symbol, String);

    builder_setter!(escape_asterisks, bool);
    builder_setter!(escape_underscores, bool);
    builder_setter!(escape_misc, bool);
    builder_setter!(escape_ascii, bool);

    builder_setter!(whitespace_mode, WhitespaceMode);
    builder_setter!(strip_newlines, bool);
    builder_setter!(wrap, bool);
    builder_setter!(wrap_width, usize);

    builder_setter!(convert_as_inline, bool);
    builder_setter!(skip_images, bool);
    builder_setter!(inline_data_media, InlineDataMedia);
    builder_setter!(url_escape_style, UrlEscapeStyle);

    /// Set the list of HTML tag names whose content is stripped from output.
    #[must_use]
    pub fn strip_tags(mut self, tags: Vec<String>) -> Self {
        self.0.strip_tags = tags;
        self
    }

    /// Set the list of HTML tag names that are preserved verbatim in output.
    #[must_use]
    pub fn preserve_tags(mut self, tags: Vec<String>) -> Self {
        self.0.preserve_tags = tags;
        self
    }

    /// Set the list of HTML tag names whose `<img>` children are kept inline.
    #[must_use]
    pub fn keep_inline_images_in(mut self, tags: Vec<String>) -> Self {
        self.0.keep_inline_images_in = tags;
        self
    }

    builder_setter!(max_image_size, u64);
    builder_setter!(max_input_size, Option<u64>);
    builder_setter!(capture_svg, bool);
    builder_setter!(infer_dimensions, bool);
    builder_setter!(max_depth, Option<usize>);

    /// Set the list of CSS selectors for elements to exclude entirely from output.
    #[must_use]
    pub fn exclude_selectors(mut self, selectors: Vec<String>) -> Self {
        self.0.exclude_selectors = selectors;
        self
    }

    /// Set the visitor used during conversion.
    #[cfg(feature = "visitor")]
    #[must_use]
    pub fn visitor(mut self, visitor: Option<crate::visitor::VisitorHandle>) -> Self {
        self.0.visitor = visitor;
        self
    }

    /// Set the pre-processing options applied to the HTML before conversion.
    #[must_use]
    pub const fn preprocessing(mut self, preprocessing: PreprocessingOptions) -> Self {
        self.0.preprocessing = preprocessing;
        self
    }

    builder_setter_into!(encoding, String);

    builder_setter!(debug, bool);

    builder_setter!(tier_strategy, TierStrategy);

    /// Set the base URL used to resolve relative `href`/`src` destinations.
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<Option<String>>) -> Self {
        self.0.base_url = base_url.into();
        self
    }

    /// Build the final [`ConversionOptions`].
    #[must_use]
    pub fn build(self) -> ConversionOptions {
        self.0
    }
}

use crate::options::preprocessing::PreprocessingOptionsUpdate;

/// Partial update for `ConversionOptions`.
///
/// Uses `Option<T>` fields for selective updates. Bindings use this to construct
/// options from language-native types. Prefer [`ConversionOptionsBuilder`] for Rust code.
#[derive(Debug, Clone, Default)]
#[cfg_attr(
    any(feature = "serde", feature = "metadata"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(any(feature = "serde", feature = "metadata"), serde(deny_unknown_fields))]
pub struct ConversionOptionsUpdate {
    /// Optional override for [`ConversionOptions::heading_style`].
    pub heading_style: Option<HeadingStyle>,
    /// Optional override for [`ConversionOptions::list_indent_type`].
    pub list_indent_type: Option<ListIndentType>,
    /// Optional override for [`ConversionOptions::list_indent_width`].
    pub list_indent_width: Option<usize>,
    /// Optional override for [`ConversionOptions::bullets`].
    pub bullets: Option<String>,
    /// Optional override for [`ConversionOptions::strong_em_symbol`].
    pub strong_em_symbol: Option<char>,
    /// Optional override for [`ConversionOptions::escape_asterisks`].
    pub escape_asterisks: Option<bool>,
    /// Optional override for [`ConversionOptions::escape_underscores`].
    pub escape_underscores: Option<bool>,
    /// Optional override for [`ConversionOptions::escape_misc`].
    pub escape_misc: Option<bool>,
    /// Optional override for [`ConversionOptions::escape_ascii`].
    pub escape_ascii: Option<bool>,
    /// Optional override for [`ConversionOptions::code_language`].
    pub code_language: Option<String>,
    /// Optional override for [`ConversionOptions::autolinks`].
    pub autolinks: Option<bool>,
    /// Optional override for [`ConversionOptions::expand_abbreviations`].
    pub expand_abbreviations: Option<bool>,
    /// Optional override for [`ConversionOptions::include_blockquote_citations`].
    pub include_blockquote_citations: Option<bool>,
    /// Optional override for [`ConversionOptions::default_title`].
    pub default_title: Option<bool>,
    /// Optional override for [`ConversionOptions::br_in_tables`].
    pub br_in_tables: Option<bool>,
    /// Optional override for [`ConversionOptions::compact_tables`].
    pub compact_tables: Option<bool>,
    /// Optional override for [`ConversionOptions::highlight_style`].
    pub highlight_style: Option<HighlightStyle>,
    /// Optional override for [`ConversionOptions::extract_metadata`].
    pub extract_metadata: Option<bool>,
    /// Optional override for [`ConversionOptions::whitespace_mode`].
    pub whitespace_mode: Option<WhitespaceMode>,
    /// Optional override for [`ConversionOptions::strip_newlines`].
    pub strip_newlines: Option<bool>,
    /// Optional override for [`ConversionOptions::wrap`].
    pub wrap: Option<bool>,
    /// Optional override for [`ConversionOptions::wrap_width`].
    pub wrap_width: Option<usize>,
    /// Optional override for [`ConversionOptions::convert_as_inline`].
    pub convert_as_inline: Option<bool>,
    /// Optional override for [`ConversionOptions::sub_symbol`].
    pub sub_symbol: Option<String>,
    /// Optional override for [`ConversionOptions::sup_symbol`].
    pub sup_symbol: Option<String>,
    /// Optional override for [`ConversionOptions::newline_style`].
    pub newline_style: Option<NewlineStyle>,
    /// Optional override for [`ConversionOptions::code_block_style`].
    pub code_block_style: Option<CodeBlockStyle>,
    /// Optional override for [`ConversionOptions::keep_inline_images_in`].
    pub keep_inline_images_in: Option<Vec<String>>,
    /// Optional override for [`ConversionOptions::preprocessing`].
    pub preprocessing: Option<PreprocessingOptionsUpdate>,
    /// Optional override for [`ConversionOptions::encoding`].
    pub encoding: Option<String>,
    /// Optional override for [`ConversionOptions::debug`].
    pub debug: Option<bool>,
    /// Optional override for [`ConversionOptions::strip_tags`].
    pub strip_tags: Option<Vec<String>>,
    /// Optional override for [`ConversionOptions::preserve_tags`].
    pub preserve_tags: Option<Vec<String>>,
    /// Optional override for [`ConversionOptions::skip_images`].
    pub skip_images: Option<bool>,
    /// Optional override for [`ConversionOptions::inline_data_media`].
    pub inline_data_media: Option<InlineDataMedia>,
    /// Optional override for [`ConversionOptions::url_escape_style`].
    pub url_escape_style: Option<UrlEscapeStyle>,
    /// Optional override for [`ConversionOptions::link_style`].
    pub link_style: Option<LinkStyle>,
    /// Optional override for [`ConversionOptions::output_format`].
    pub output_format: Option<OutputFormat>,
    /// Optional override for [`ConversionOptions::include_document_structure`].
    pub include_document_structure: Option<bool>,
    /// Optional override for [`ConversionOptions::extract_images`].
    pub extract_images: Option<bool>,
    /// Optional override for [`ConversionOptions::max_image_size`].
    pub max_image_size: Option<u64>,
    /// Optional override for [`ConversionOptions::max_input_size`].
    pub max_input_size: Option<Option<u64>>,
    /// Optional override for [`ConversionOptions::capture_svg`].
    pub capture_svg: Option<bool>,
    /// Optional override for [`ConversionOptions::infer_dimensions`].
    pub infer_dimensions: Option<bool>,
    /// Optional override for [`ConversionOptions::max_depth`].
    pub max_depth: Option<Option<usize>>,
    /// Optional override for [`ConversionOptions::exclude_selectors`].
    pub exclude_selectors: Option<Vec<String>>,
    /// Optional override for [`ConversionOptions::tier_strategy`].
    pub tier_strategy: Option<TierStrategy>,
    /// Optional override for [`ConversionOptions::base_url`].
    pub base_url: Option<Option<String>>,
    /// Optional override for [`ConversionOptions::visitor`].
    #[cfg(feature = "visitor")]
    #[cfg_attr(any(feature = "serde", feature = "metadata"), serde(skip))]
    pub visitor: Option<crate::visitor::VisitorHandle>,
}

impl ConversionOptions {
    /// Apply a partial update to these conversion options.
    #[cfg_attr(alef, alef(skip))]
    pub fn apply_update(&mut self, update: ConversionOptionsUpdate) {
        macro_rules! apply {
            ($field:ident) => {
                if let Some(v) = update.$field {
                    self.$field = v;
                }
            };
        }
        apply!(heading_style);
        apply!(list_indent_type);
        apply!(list_indent_width);
        apply!(bullets);
        apply!(strong_em_symbol);
        apply!(escape_asterisks);
        apply!(escape_underscores);
        apply!(escape_misc);
        apply!(escape_ascii);
        apply!(code_language);
        apply!(autolinks);
        apply!(expand_abbreviations);
        apply!(include_blockquote_citations);
        apply!(default_title);
        apply!(br_in_tables);
        apply!(compact_tables);
        apply!(highlight_style);
        apply!(extract_metadata);
        apply!(whitespace_mode);
        apply!(strip_newlines);
        apply!(wrap);
        apply!(wrap_width);
        apply!(convert_as_inline);
        apply!(sub_symbol);
        apply!(sup_symbol);
        apply!(newline_style);
        apply!(code_block_style);
        apply!(keep_inline_images_in);
        apply!(encoding);
        apply!(debug);
        apply!(strip_tags);
        apply!(preserve_tags);
        apply!(skip_images);
        apply!(inline_data_media);
        apply!(url_escape_style);
        apply!(link_style);
        apply!(output_format);
        apply!(include_document_structure);
        apply!(extract_images);
        apply!(max_image_size);
        apply!(max_input_size);
        apply!(capture_svg);
        apply!(infer_dimensions);
        apply!(max_depth);
        apply!(exclude_selectors);
        apply!(tier_strategy);
        apply!(base_url);
        #[cfg(feature = "visitor")]
        if let Some(visitor) = update.visitor {
            self.visitor = Some(visitor);
        }
        if let Some(preprocessing) = update.preprocessing {
            self.preprocessing.apply_update(preprocessing);
        }
    }

    /// Create from a partial update, applying to defaults.
    #[must_use]
    #[cfg_attr(alef, alef(skip))]
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

#[cfg(all(test, any(feature = "serde", feature = "metadata")))]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_options_serde() {
        let options = ConversionOptions::builder()
            .heading_style(HeadingStyle::AtxClosed)
            .list_indent_width(4)
            .bullets("*")
            .escape_asterisks(true)
            .whitespace_mode(WhitespaceMode::Strict)
            .build();

        let json = serde_json::to_string(&options).expect("Failed to serialize");
        let deserialized: ConversionOptions = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.list_indent_width, 4);
        assert_eq!(deserialized.bullets, "*");
        assert!(deserialized.escape_asterisks);
        assert_eq!(deserialized.heading_style, HeadingStyle::AtxClosed);
        assert_eq!(deserialized.whitespace_mode, WhitespaceMode::Strict);
    }

    #[test]
    fn test_conversion_options_partial_deserialization() {
        let partial_json = r#"{
            "heading_style": "atxclosed",
            "list_indent_width": 4,
            "bullets": "*"
        }"#;

        let deserialized: ConversionOptions =
            serde_json::from_str(partial_json).expect("Failed to deserialize partial JSON");

        assert_eq!(deserialized.heading_style, HeadingStyle::AtxClosed);
        assert_eq!(deserialized.list_indent_width, 4);
        assert_eq!(deserialized.bullets, "*");
        assert!(!deserialized.escape_asterisks);
        assert!(!deserialized.escape_underscores);
        assert_eq!(deserialized.list_indent_type, ListIndentType::Spaces);
    }

    #[test]
    fn test_builder_pattern() {
        let options = ConversionOptions::builder()
            .heading_style(HeadingStyle::Underlined)
            .wrap(true)
            .wrap_width(100)
            .include_document_structure(true)
            .extract_images(true)
            .build();

        assert_eq!(options.heading_style, HeadingStyle::Underlined);
        assert!(options.wrap);
        assert_eq!(options.wrap_width, 100);
        assert!(options.include_document_structure);
        assert!(options.extract_images);
    }
}
