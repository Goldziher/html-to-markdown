//! MCP request parameter types.
//!
//! [`ConvertConfig`] is a typed, schema-bearing mirror of the settable fields of
//! [`crate::options::ConversionOptions`]. It exists so the `convert_html` tool can
//! advertise every conversion option to MCP clients through its generated
//! `inputSchema` (rmcp requires the tool parameter type to implement
//! [`schemars::JsonSchema`], which the core option types do not). Every field is
//! optional; only the fields a client sends override the corresponding default.
//!
//! The mirror is kept honest by the `mirror_covers_all_core_fields` drift-guard
//! test below, which fails if a field is ever added to `ConversionOptions` without
//! being mirrored here.

use crate::options::{
    CodeBlockStyle, ConversionOptions, ConversionOptionsUpdate, HeadingStyle, HighlightStyle, InlineDataMedia,
    LinkStyle, ListIndentType, NewlineStyle, OutputFormat, PreprocessingOptionsUpdate, PreprocessingPreset,
    TierStrategy, UrlEscapeStyle, WhitespaceMode,
};
use rmcp::schemars;

/// Typed conversion options for the `convert_html` tool.
///
/// Mirrors the settable fields of [`crate::options::ConversionOptions`]. Every
/// field is optional and defaults to the engine default when omitted. Enum-valued
/// options are accepted as strings (case-insensitive) and parsed with the same
/// parsers the core library uses.
#[cfg_attr(alef, alef(skip))]
#[derive(Debug, Default, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ConvertConfig {
    /// Heading style: `"atx"` (default), `"atxclosed"`, or `"underlined"`.
    pub heading_style: Option<String>,
    /// List indentation: `"spaces"` (default) or `"tabs"`.
    pub list_indent_type: Option<String>,
    /// Spaces (or tabs) per list-indent level. Default `2`.
    pub list_indent_width: Option<usize>,
    /// Bullet characters for unordered lists. Default `"-*+"`.
    pub bullets: Option<String>,
    /// Emphasis marker; first character is used (`"*"` default or `"_"`).
    pub strong_em_symbol: Option<String>,
    /// Escape `*` in plain text. Default `false`.
    pub escape_asterisks: Option<bool>,
    /// Escape `_` in plain text. Default `false`.
    pub escape_underscores: Option<bool>,
    /// Escape miscellaneous Markdown metacharacters (`[]()#` etc.). Default `false`.
    pub escape_misc: Option<bool>,
    /// Escape ASCII characters special in some Markdown dialects. Default `false`.
    pub escape_ascii: Option<bool>,
    /// Default language for fenced code blocks with no hint. Default empty.
    pub code_language: Option<String>,
    /// Convert bare URLs into autolinks. Default `true`.
    pub autolinks: Option<bool>,
    /// Append the title of an `<abbr>` in parentheses. Default `true`.
    pub expand_abbreviations: Option<bool>,
    /// Append the `cite` URL after a blockquote. Default `true`.
    pub include_blockquote_citations: Option<bool>,
    /// Emit a default title when no `<title>` is present. Default `false`.
    pub default_title: Option<bool>,
    /// Render `<br>` inside table cells as literal line breaks. Default `false`.
    pub br_in_tables: Option<bool>,
    /// Emit compact (unpadded) GFM tables. Default `false`.
    pub compact_tables: Option<bool>,
    /// `<mark>` rendering: `"doubleequal"` (default), `"html"`, `"bold"`, or `"none"`.
    pub highlight_style: Option<String>,
    /// Populate `metadata` from `<head>`/`<meta>`. Default `true`.
    pub extract_metadata: Option<bool>,
    /// Whitespace handling: `"normalized"` (default) or `"strict"`.
    pub whitespace_mode: Option<String>,
    /// Strip all newlines, producing single-line output. Default `false`.
    pub strip_newlines: Option<bool>,
    /// Wrap long lines at `wrap_width`. Default `false`.
    pub wrap: Option<bool>,
    /// Maximum line width when `wrap` is enabled. Default `80`.
    pub wrap_width: Option<usize>,
    /// Treat the whole document as inline content. Default `false`.
    pub convert_as_inline: Option<bool>,
    /// Markdown notation for subscript text (e.g. `"~"`). Default empty.
    pub sub_symbol: Option<String>,
    /// Markdown notation for superscript text (e.g. `"^"`). Default empty.
    pub sup_symbol: Option<String>,
    /// Hard line-break syntax: `"spaces"` (default) or `"backslash"`.
    pub newline_style: Option<String>,
    /// Fenced code block style: `"backticks"` (default), `"tildes"`, or `"indented"`.
    pub code_block_style: Option<String>,
    /// HTML tags whose `<img>` children stay inline.
    pub keep_inline_images_in: Option<Vec<String>>,
    /// HTML preprocessing (cleanup) options.
    pub preprocessing: Option<PreprocessingParams>,
    /// Expected input character encoding. Default `"utf-8"`.
    pub encoding: Option<String>,
    /// Emit debug information during conversion. Default `false`.
    pub debug: Option<bool>,
    /// HTML tags to strip (remove wrapper, keep children).
    pub strip_tags: Option<Vec<String>>,
    /// HTML tags to preserve verbatim in the output.
    pub preserve_tags: Option<Vec<String>>,
    /// Omit all `<img>` elements from the output. Default `false`.
    pub skip_images: Option<bool>,
    /// What to write for an image or media element whose address is a `data:` URL:
    /// `"keep"` (default, the URL with its payload), `"alt_text_only"`, or `"drop_element"`.
    pub inline_data_media: Option<String>,
    /// URL escaping: `"angle"` (default) or `"percent"`.
    pub url_escape_style: Option<String>,
    /// Link rendering: `"inline"` (default) or `"reference"`.
    pub link_style: Option<String>,
    /// Output format: `"markdown"` (default), `"djot"`, or `"plain"`.
    pub output_format: Option<String>,
    /// Include the structured document tree in the result. Default `false`.
    pub include_document_structure: Option<bool>,
    /// Extract inline images from data URIs and SVGs. Default `false`.
    pub extract_images: Option<bool>,
    /// Maximum decoded image size in bytes. Default `5242880` (5 MB).
    pub max_image_size: Option<u64>,
    /// Maximum accepted HTML input size in bytes. Omission uses the platform default.
    pub max_input_size: Option<u64>,
    /// Capture inline `<svg>` elements as images. Default `false`.
    pub capture_svg: Option<bool>,
    /// Infer image dimensions from data. Default `true`.
    pub infer_dimensions: Option<bool>,
    /// Maximum DOM traversal depth; omission uses the native-stack safety limit.
    pub max_depth: Option<u64>,
    /// CSS selectors for elements to exclude entirely (element + descendants).
    pub exclude_selectors: Option<Vec<String>>,
    /// Conversion tier: `"auto"` (default) or `"tier2"`.
    pub tier_strategy: Option<String>,
    /// Base URL to resolve relative `href`/`src` destinations against. Omitted (default):
    /// no resolution.
    pub base_url: Option<String>,
}

/// Typed mirror of [`crate::options::PreprocessingOptions`] for MCP input.
#[cfg_attr(alef, alef(skip))]
#[derive(Debug, Default, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct PreprocessingParams {
    /// Enable preprocessing globally. Default `true`.
    pub enabled: Option<bool>,
    /// Preset: `"minimal"`, `"standard"` (default), or `"aggressive"`.
    pub preset: Option<String>,
    /// Remove navigation elements (nav, breadcrumbs, menus, sidebars). Default `true`.
    pub remove_navigation: Option<bool>,
    /// Remove form elements (forms, inputs, buttons). Default `true`.
    pub remove_forms: Option<bool>,
}

/// An enum-valued [`ConvertConfig`] (or [`PreprocessingParams`]) field carried
/// a string that is not one of the accepted wire values.
///
/// Every enum field is accepted as a string over MCP; before this type
/// existed, an unrecognized string silently resolved to whichever variant a
/// core `X::parse` fell back to (not necessarily that field's documented
/// default — see the `mirror_covers_all_core_fields` drift-guard test module
/// for context). Constructing this error instead names the offending field,
/// echoes the value the client sent, and lists every accepted (snake_case,
/// matching the `serde(rename_all = "snake_case")` wire format) value, so the
/// client gets an actionable `invalid_params` error rather than a silent
/// substitution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidEnumValue {
    /// The `ConvertConfig` field name (as it appears on the wire).
    pub field: &'static str,
    /// The value the client sent.
    pub value: String,
    /// Every value `field` accepts.
    pub accepted: &'static [&'static str],
}

impl std::fmt::Display for InvalidEnumValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid value \"{}\" for `{}`: expected one of {}",
            self.value,
            self.field,
            self.accepted.join(", ")
        )
    }
}

impl std::error::Error for InvalidEnumValue {}

const HEADING_STYLE_VALUES: &[&str] = &["atx", "atxclosed", "underlined"];
const LIST_INDENT_TYPE_VALUES: &[&str] = &["spaces", "tabs"];
const HIGHLIGHT_STYLE_VALUES: &[&str] = &["doubleequal", "html", "bold", "none"];
const WHITESPACE_MODE_VALUES: &[&str] = &["normalized", "strict"];
const NEWLINE_STYLE_VALUES: &[&str] = &["spaces", "backslash"];
const CODE_BLOCK_STYLE_VALUES: &[&str] = &["indented", "backticks", "tildes"];
const URL_ESCAPE_STYLE_VALUES: &[&str] = &["angle", "percent"];
const INLINE_DATA_MEDIA_VALUES: &[&str] = &["keep", "alttextonly", "dropelement"];
const LINK_STYLE_VALUES: &[&str] = &["inline", "reference"];
const OUTPUT_FORMAT_VALUES: &[&str] = &["markdown", "djot", "plain", "plaintext", "text"];
const TIER_STRATEGY_VALUES: &[&str] = &["auto", "tier2"];
const PREPROCESSING_PRESET_VALUES: &[&str] = &["minimal", "standard", "aggressive"];

/// Reject `raw` unless it normalizes to one of `accepted`.
///
/// Uses the same normalization (`lowercase`, alphanumeric-only) the core
/// `X::parse` constructors use, so anything this accepts, `parser` below
/// parses into the value the client actually asked for — never a fallback.
fn validate_enum_value(field: &'static str, raw: &str, accepted: &'static [&str]) -> Result<(), InvalidEnumValue> {
    let normalized = crate::options::validation::normalize_token(raw);
    if accepted.contains(&normalized.as_str()) {
        Ok(())
    } else {
        Err(InvalidEnumValue {
            field,
            value: raw.to_string(),
            accepted,
        })
    }
}

/// Validate and parse an optional enum-string field.
///
/// `None` (the client omitted the field) passes through unchanged; `Some`
/// must normalize to one of `accepted` or this returns [`InvalidEnumValue`].
fn validated_enum<T>(
    field: &'static str,
    raw: Option<String>,
    accepted: &'static [&str],
    parser: impl FnOnce(&str) -> T,
) -> Result<Option<T>, InvalidEnumValue> {
    match raw {
        Some(raw) => {
            validate_enum_value(field, &raw, accepted)?;
            Ok(Some(parser(&raw)))
        }
        None => Ok(None),
    }
}

impl TryFrom<PreprocessingParams> for PreprocessingOptionsUpdate {
    type Error = InvalidEnumValue;

    fn try_from(params: PreprocessingParams) -> Result<Self, Self::Error> {
        Ok(Self {
            enabled: params.enabled,
            preset: validated_enum(
                "preprocessing.preset",
                params.preset,
                PREPROCESSING_PRESET_VALUES,
                PreprocessingPreset::parse,
            )?,
            remove_navigation: params.remove_navigation,
            remove_forms: params.remove_forms,
        })
    }
}

impl TryFrom<ConvertConfig> for ConversionOptionsUpdate {
    type Error = InvalidEnumValue;

    fn try_from(config: ConvertConfig) -> Result<Self, Self::Error> {
        config.into_update()
    }
}

impl ConvertConfig {
    fn into_update(mut self) -> Result<ConversionOptionsUpdate, InvalidEnumValue> {
        let mut update = ConversionOptionsUpdate {
            preprocessing: self.preprocessing.take().map(TryInto::try_into).transpose()?,
            ..ConversionOptionsUpdate::default()
        };
        self.apply_basic_options(&mut update);
        self.apply_formatting_options(&mut update)?;
        self.apply_content_options(&mut update)?;
        Ok(update)
    }

    fn apply_basic_options(&mut self, update: &mut ConversionOptionsUpdate) {
        update.list_indent_width = self.list_indent_width.take();
        update.bullets = self.bullets.take();
        update.strong_em_symbol = self.strong_em_symbol.take().and_then(|value| value.chars().next());
        update.escape_asterisks = self.escape_asterisks.take();
        update.escape_underscores = self.escape_underscores.take();
        update.escape_misc = self.escape_misc.take();
        update.escape_ascii = self.escape_ascii.take();
        update.code_language = self.code_language.take();
        update.autolinks = self.autolinks.take();
        update.expand_abbreviations = self.expand_abbreviations.take();
        update.include_blockquote_citations = self.include_blockquote_citations.take();
        update.default_title = self.default_title.take();
        update.br_in_tables = self.br_in_tables.take();
        update.compact_tables = self.compact_tables.take();
        update.extract_metadata = self.extract_metadata.take();
        update.strip_newlines = self.strip_newlines.take();
        update.wrap = self.wrap.take();
        update.wrap_width = self.wrap_width.take();
        update.convert_as_inline = self.convert_as_inline.take();
        update.sub_symbol = self.sub_symbol.take();
        update.sup_symbol = self.sup_symbol.take();
    }

    fn apply_formatting_options(&mut self, update: &mut ConversionOptionsUpdate) -> Result<(), InvalidEnumValue> {
        update.heading_style = validated_enum(
            "heading_style",
            self.heading_style.take(),
            HEADING_STYLE_VALUES,
            HeadingStyle::parse,
        )?;
        update.list_indent_type = validated_enum(
            "list_indent_type",
            self.list_indent_type.take(),
            LIST_INDENT_TYPE_VALUES,
            ListIndentType::parse,
        )?;
        update.highlight_style = validated_enum(
            "highlight_style",
            self.highlight_style.take(),
            HIGHLIGHT_STYLE_VALUES,
            HighlightStyle::parse,
        )?;
        update.whitespace_mode = validated_enum(
            "whitespace_mode",
            self.whitespace_mode.take(),
            WHITESPACE_MODE_VALUES,
            WhitespaceMode::parse,
        )?;
        update.newline_style = validated_enum(
            "newline_style",
            self.newline_style.take(),
            NEWLINE_STYLE_VALUES,
            NewlineStyle::parse,
        )?;
        update.code_block_style = validated_enum(
            "code_block_style",
            self.code_block_style.take(),
            CODE_BLOCK_STYLE_VALUES,
            CodeBlockStyle::parse,
        )?;
        update.url_escape_style = validated_enum(
            "url_escape_style",
            self.url_escape_style.take(),
            URL_ESCAPE_STYLE_VALUES,
            UrlEscapeStyle::parse,
        )?;
        update.link_style = validated_enum(
            "link_style",
            self.link_style.take(),
            LINK_STYLE_VALUES,
            LinkStyle::parse,
        )?;
        update.output_format = validated_enum(
            "output_format",
            self.output_format.take(),
            OUTPUT_FORMAT_VALUES,
            OutputFormat::parse,
        )?;
        Ok(())
    }

    fn apply_content_options(&mut self, update: &mut ConversionOptionsUpdate) -> Result<(), InvalidEnumValue> {
        update.keep_inline_images_in = self.keep_inline_images_in.take();
        update.encoding = self.encoding.take();
        update.debug = self.debug.take();
        update.strip_tags = self.strip_tags.take();
        update.preserve_tags = self.preserve_tags.take();
        update.skip_images = self.skip_images.take();
        update.inline_data_media = validated_enum(
            "inline_data_media",
            self.inline_data_media.take(),
            INLINE_DATA_MEDIA_VALUES,
            InlineDataMedia::parse,
        )?;
        update.include_document_structure = self.include_document_structure.take();
        update.extract_images = self.extract_images.take();
        update.max_image_size = self.max_image_size.take();
        update.max_input_size = self.max_input_size.take().map(Some);
        update.capture_svg = self.capture_svg.take();
        update.infer_dimensions = self.infer_dimensions.take();
        update.max_depth = self.max_depth.take().map(|requested| Some(clamp_max_depth(requested)));
        update.exclude_selectors = self.exclude_selectors.take();
        update.tier_strategy = validated_enum(
            "tier_strategy",
            self.tier_strategy.take(),
            TIER_STRATEGY_VALUES,
            parse_tier_strategy,
        )?;
        update.base_url = self.base_url.take().map(Some);
        Ok(())
    }
}

impl TryFrom<ConvertConfig> for ConversionOptions {
    type Error = InvalidEnumValue;

    fn try_from(config: ConvertConfig) -> Result<Self, Self::Error> {
        Ok(Self::from_update(config.try_into()?))
    }
}

/// Convert an MCP client's requested `max_depth` (wire type `u64`) to the
/// engine's native `usize`.
///
/// On 64-bit targets `usize == u64` and this is always exact. On 32-bit
/// targets a client-requested value above `usize::MAX` cannot be represented;
/// rather than silently wrapping or rejecting the whole request, it is
/// clamped to `usize::MAX` and the clamp is surfaced via a `WARN` event so the
/// degradation is observable instead of silent.
fn clamp_max_depth(requested: u64) -> usize {
    if let Ok(depth) = usize::try_from(requested) {
        depth
    } else {
        tracing::warn!(
            target: "html_to_markdown::mcp",
            requested,
            clamped_value = usize::MAX,
            "max_depth exceeds this platform's usize range; clamping to usize::MAX"
        );
        usize::MAX
    }
}

/// Parse a [`TierStrategy`] from a string already validated against
/// [`TIER_STRATEGY_VALUES`].
///
/// `TierStrategy` has no public `parse` constructor, so this matches on the
/// normalised token the same way the other option enums do (case- and
/// separator-insensitive), accepting `"auto"` and `"tier2"`.
fn parse_tier_strategy(value: &str) -> TierStrategy {
    match crate::options::validation::normalize_token(value).as_str() {
        "tier2" => TierStrategy::Tier2,
        _ => TierStrategy::Auto,
    }
}

/// Parameters for the `convert_html` MCP tool.
#[cfg_attr(alef, alef(skip))]
#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConvertHtmlParams {
    /// The HTML string to convert to Markdown.
    pub html: String,

    /// Optional typed conversion options. Omit for engine defaults.
    #[serde(default)]
    pub config: Option<ConvertConfig>,

    /// When `true`, return the full `ConversionResult` serialised as JSON
    /// (content, tables, document structure, metadata, warnings) instead of the
    /// bare Markdown string.
    #[serde(default)]
    pub json: bool,
}

/// Parameters for the `extract_metadata` MCP tool.
#[cfg_attr(alef, alef(skip))]
#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExtractMetadataParams {
    /// The HTML string to extract metadata from.
    pub html: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn json_keys(value: &serde_json::Value) -> BTreeSet<String> {
        value
            .as_object()
            .expect("expected a JSON object")
            .keys()
            .cloned()
            .collect()
    }

    /// Drift guard: every settable `ConversionOptions` field must be mirrored by
    /// `ConvertConfig`. `ConversionOptions` serialises every field except the
    /// serde-skipped `visitor`, and `ConvertConfig` serialises every field as
    /// `null` (no `skip_serializing_if`), so the key sets must match exactly.
    /// Fails loudly, naming the offending field, if a core option is added
    /// without being mirrored here.
    #[test]
    fn mirror_covers_all_core_fields() {
        let core = serde_json::to_value(ConversionOptions::default()).unwrap();
        let mirror = serde_json::to_value(ConvertConfig::default()).unwrap();
        assert_eq!(
            json_keys(&core),
            json_keys(&mirror),
            "ConvertConfig drifted from ConversionOptions — add/remove the mismatched field(s)"
        );
    }

    /// Same drift guard for the nested preprocessing mirror.
    #[test]
    fn preprocessing_mirror_covers_all_core_fields() {
        use crate::options::PreprocessingOptions;
        let core = serde_json::to_value(PreprocessingOptions::default()).unwrap();
        let mirror = serde_json::to_value(PreprocessingParams::default()).unwrap();
        assert_eq!(json_keys(&core), json_keys(&mirror));
    }

    #[test]
    fn test_convert_html_params_minimal() {
        let params: ConvertHtmlParams = serde_json::from_str(r#"{"html": "<h1>Hi</h1>"}"#).unwrap();
        assert_eq!(params.html, "<h1>Hi</h1>");
        assert!(params.config.is_none());
        assert!(!params.json);
    }

    #[test]
    fn test_enum_strings_parse_into_typed_options() {
        let config = ConvertConfig {
            heading_style: Some("atxclosed".into()),
            output_format: Some("djot".into()),
            code_block_style: Some("tildes".into()),
            ..ConvertConfig::default()
        };
        let opts: ConversionOptions = config.try_into().expect("all values are accepted");
        assert_eq!(opts.heading_style, HeadingStyle::AtxClosed);
        assert_eq!(opts.output_format, OutputFormat::Djot);
        assert_eq!(opts.code_block_style, CodeBlockStyle::Tildes);
    }

    #[test]
    fn should_reject_unknown_enum_string_instead_of_substituting_a_default() {
        let config = ConvertConfig {
            heading_style: Some("nonsense".into()),
            ..ConvertConfig::default()
        };
        let error = ConversionOptions::try_from(config).expect_err("unrecognized value must be rejected");
        assert_eq!(error.field, "heading_style");
        assert_eq!(error.value, "nonsense");
        assert_eq!(error.accepted, HEADING_STYLE_VALUES);
    }

    #[test]
    fn should_report_the_offending_field_and_accepted_values_in_the_message() {
        let config = ConvertConfig {
            output_format: Some("yaml".into()),
            ..ConvertConfig::default()
        };
        let error = ConversionOptions::try_from(config).expect_err("unrecognized value must be rejected");
        let message = error.to_string();
        assert!(
            message.contains("output_format"),
            "message must name the field: {message}"
        );
        assert!(
            message.contains("yaml"),
            "message must echo the offending value: {message}"
        );
        assert!(
            message.contains("markdown"),
            "message must list accepted values: {message}"
        );
    }

    #[test]
    fn should_reject_unknown_nested_preprocessing_preset() {
        let config = ConvertConfig {
            preprocessing: Some(PreprocessingParams {
                preset: Some("extreme".into()),
                ..PreprocessingParams::default()
            }),
            ..ConvertConfig::default()
        };
        let error = ConversionOptions::try_from(config).expect_err("unrecognized preset must be rejected");
        assert_eq!(error.field, "preprocessing.preset");
        assert_eq!(error.value, "extreme");
    }

    #[test]
    fn should_report_invalid_preprocessing_before_invalid_heading_style() {
        let config = ConvertConfig {
            heading_style: Some("nonsense".into()),
            preprocessing: Some(PreprocessingParams {
                preset: Some("extreme".into()),
                ..PreprocessingParams::default()
            }),
            ..ConvertConfig::default()
        };
        let error = ConversionOptions::try_from(config).expect_err("invalid preprocessing must be reported first");
        assert_eq!(
            error,
            InvalidEnumValue {
                field: "preprocessing.preset",
                value: "extreme".into(),
                accepted: PREPROCESSING_PRESET_VALUES,
            }
        );
    }

    #[test]
    fn should_reject_unknown_tier_strategy() {
        let config = ConvertConfig {
            tier_strategy: Some("tier3".into()),
            ..ConvertConfig::default()
        };
        let error = ConversionOptions::try_from(config).expect_err("unrecognized tier strategy must be rejected");
        assert_eq!(error.field, "tier_strategy");
    }

    #[test]
    fn inline_data_media_accepts_each_choice_and_rejects_others() {
        for (value, expected) in [
            ("keep", InlineDataMedia::Keep),
            ("alt_text_only", InlineDataMedia::AltTextOnly),
            ("drop_element", InlineDataMedia::DropElement),
        ] {
            let config = ConvertConfig {
                inline_data_media: Some(value.into()),
                ..ConvertConfig::default()
            };
            let opts = ConversionOptions::try_from(config).expect("documented value must be accepted");
            assert_eq!(opts.inline_data_media, expected, "{value}");
        }
        let config = ConvertConfig {
            inline_data_media: Some("strip".into()),
            ..ConvertConfig::default()
        };
        let error = ConversionOptions::try_from(config).expect_err("unrecognized value must be rejected");
        assert_eq!(error.field, "inline_data_media");
        assert_eq!(error.accepted, INLINE_DATA_MEDIA_VALUES);
    }

    #[test]
    fn should_accept_every_documented_heading_style_value() {
        for value in HEADING_STYLE_VALUES {
            let config = ConvertConfig {
                heading_style: Some((*value).to_string()),
                ..ConvertConfig::default()
            };
            assert!(
                ConversionOptions::try_from(config).is_ok(),
                "documented value {value:?} must be accepted"
            );
        }
    }

    #[test]
    fn test_partial_config_leaves_other_fields_at_default() {
        let config = ConvertConfig {
            wrap: Some(true),
            wrap_width: Some(100),
            ..ConvertConfig::default()
        };
        let opts: ConversionOptions = config.try_into().expect("no enum fields set");
        assert!(opts.wrap);
        assert_eq!(opts.wrap_width, 100);
        assert!(opts.autolinks);
        assert_eq!(opts.bullets, "-*+");
        assert_eq!(opts.heading_style, HeadingStyle::Atx);
    }

    #[test]
    fn should_pass_the_abbreviation_and_citation_options_to_the_converter() {
        let defaults: ConversionOptions = ConvertConfig::default().try_into().expect("no enum fields set");
        assert!(defaults.expand_abbreviations);
        assert!(defaults.include_blockquote_citations);

        let config: ConvertConfig =
            serde_json::from_str(r#"{"expand_abbreviations":false,"include_blockquote_citations":false}"#)
                .expect("both options are accepted");
        let opts: ConversionOptions = config.try_into().expect("no enum fields set");
        assert!(!opts.expand_abbreviations);
        assert!(!opts.include_blockquote_citations);
    }

    #[test]
    fn test_strong_em_symbol_takes_first_char() {
        let config = ConvertConfig {
            strong_em_symbol: Some("_".into()),
            ..ConvertConfig::default()
        };
        let opts: ConversionOptions = config.try_into().expect("no enum fields set");
        assert_eq!(opts.strong_em_symbol, '_');
    }

    #[test]
    fn test_max_depth_maps_through() {
        let config = ConvertConfig {
            max_depth: Some(5),
            ..ConvertConfig::default()
        };
        let opts: ConversionOptions = config.try_into().expect("no enum fields set");
        assert_eq!(opts.max_depth, Some(5));
    }

    #[test]
    fn test_max_input_size_maps_through() {
        let config = ConvertConfig {
            max_input_size: Some(1_024),
            ..ConvertConfig::default()
        };
        let opts: ConversionOptions = config.try_into().expect("no enum fields set");
        assert_eq!(opts.max_input_size, Some(1_024));
    }

    #[test]
    fn test_preprocessing_nested_config() {
        let config = ConvertConfig {
            preprocessing: Some(PreprocessingParams {
                preset: Some("aggressive".into()),
                remove_forms: Some(false),
                ..PreprocessingParams::default()
            }),
            ..ConvertConfig::default()
        };
        let opts: ConversionOptions = config.try_into().expect("\"aggressive\" is accepted");
        assert_eq!(opts.preprocessing.preset, PreprocessingPreset::Aggressive);
        assert!(!opts.preprocessing.remove_forms);
        assert!(opts.preprocessing.enabled);
        assert!(opts.preprocessing.remove_navigation);
    }

    #[test]
    fn test_tier_strategy_parses() {
        let config = ConvertConfig {
            tier_strategy: Some("tier2".into()),
            ..ConvertConfig::default()
        };
        let opts: ConversionOptions = config.try_into().expect("\"tier2\" is accepted");
        assert_eq!(opts.tier_strategy, TierStrategy::Tier2);
    }

    #[test]
    fn test_unknown_top_level_field_is_rejected() {
        let result: Result<ConvertConfig, _> = serde_json::from_str(r#"{"unknown_field_xyz": true}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_generation_exposes_config_fields() {
        let schema = schemars::schema_for!(ConvertHtmlParams);
        let json = serde_json::to_value(&schema).unwrap();
        let text = serde_json::to_string(&json).unwrap();
        assert!(text.contains("heading_style"), "schema must expose heading_style");
        assert!(text.contains("output_format"), "schema must expose output_format");
        assert!(text.contains("preprocessing"), "schema must expose preprocessing");
    }
}
