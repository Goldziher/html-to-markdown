//! Main HTML to Markdown conversion API.
//!
//! This module provides the primary `convert()` function for converting HTML to Markdown.

use std::borrow::Cow;

#[cfg(any(feature = "metadata", feature = "inline-images"))]
use crate::ConversionError;
use crate::error::Result;
use crate::options::{ConversionOptions, WhitespaceMode};
use crate::text;
use crate::types::ConversionResult;
use crate::validation::{Utf16Encoding, detect_utf16_encoding, validate_input};

#[cfg(feature = "metadata")]
use crate::{HtmlMetadata, MetadataConfig};

/// Convert HTML to Markdown, Djot, or plain text.
///
/// Returns a [`ConversionResult`] with converted content plus optional metadata,
/// document structure, table data, inline images, and warnings depending on the
/// enabled features and conversion options.
///
/// # Arguments
///
/// * `html` — the HTML string to convert.
/// * `options` — conversion options. Rust accepts bare [`ConversionOptions`],
///   `Some(options)`, or `None`. Language bindings expose the same option
///   fields through native constructors or optional parameters.
///
/// # Example
///
/// ```
/// use html_to_markdown_rs::{convert, ConversionOptions};
///
/// let html = "<h1>Hello World</h1>";
///
/// // Bare options — most ergonomic.
/// let result = convert(html, ConversionOptions::default()).unwrap();
/// assert!(result.content.as_deref().unwrap_or("").contains("Hello World"));
///
/// // `None` falls back to defaults.
/// let result = convert(html, None).unwrap();
/// assert!(result.content.as_deref().unwrap_or("").contains("Hello World"));
/// ```
///
/// # Errors
///
/// Returns an error if the configured input-size limit is exceeded, HTML parsing fails,
/// or the input contains invalid UTF-8.
///
/// # Observability
///
/// Emits an `html_to_markdown::convert` span at `INFO` level with fields `input_len`,
/// `output_format`, `wrap`, `extract_metadata`, `extract_images`, and `tier_strategy`. These
/// field names are part of the public observability contract and are kept stable across
/// releases. This crate never installs a `tracing` subscriber — attach one in the consuming
/// application to observe these spans and events.
#[tracing::instrument(
    level = "info",
    name = "html_to_markdown::convert",
    skip_all,
    fields(
        input_len = html.len(),
        output_format = tracing::field::Empty,
        wrap = tracing::field::Empty,
        extract_metadata = tracing::field::Empty,
        extract_images = tracing::field::Empty,
        tier_strategy = tracing::field::Empty,
    )
)]
pub fn convert(html: &str, options: impl Into<Option<ConversionOptions>>) -> Result<ConversionResult> {
    let options = options.into().unwrap_or_default();

    let span = tracing::Span::current();
    span.record("output_format", tracing::field::debug(options.output_format));
    span.record("wrap", options.wrap);
    span.record("extract_metadata", options.extract_metadata);
    span.record("extract_images", options.extract_images);
    span.record("tier_strategy", tracing::field::debug(options.tier_strategy));

    // ~keep Thin generic wrapper. Delegates to the non-generic `convert_inner` so the
    // ~keep large body monomorphises exactly once instead of once per `Into` impl
    // ~keep the caller picks. See xberg-io/html-to-markdown#398.
    convert_inner(html, options)
}

struct PreparedConversion<'a> {
    html: Cow<'a, str>,
    inert_bodies: std::collections::HashMap<String, String>,
    effective_base: Option<std::rc::Rc<url::Url>>,
    metadata_base_href: Option<String>,
    head_repair_limit: Option<crate::types::WarningKind>,
}

#[cfg(feature = "metadata")]
type MetadataCollectorHandle = std::rc::Rc<std::cell::RefCell<crate::metadata::MetadataCollector>>;
#[cfg(feature = "inline-images")]
type ImageCollectorHandle = std::rc::Rc<std::cell::RefCell<crate::inline_images::InlineImageCollector>>;

struct Tier2Collectors {
    #[cfg(feature = "metadata")]
    metadata: Option<MetadataCollectorHandle>,
    #[cfg(feature = "inline-images")]
    images: Option<ImageCollectorHandle>,
}

type ConvertOutput = (
    String,
    Option<crate::types::DocumentStructure>,
    Vec<crate::types::TableData>,
    Option<crate::types::ProcessingWarning>,
);

fn convert_inner(html: &str, options: ConversionOptions) -> Result<ConversionResult> {
    validate_input_size(html, options.max_input_size)?;
    let prepared = prepare_conversion(html, &options)?;
    if let Some(result) = try_tier1(&prepared, &options) {
        return Ok(result);
    }
    if !options.wrap
        && let Some(markdown) = fast_text_only(prepared.html.as_ref(), &options)
    {
        return Ok(ConversionResult {
            content: Some(markdown),
            ..ConversionResult::default()
        });
    }
    run_tier2(prepared, options)
}

fn validate_input_size(html: &str, max_size: Option<u64>) -> Result<()> {
    if let Some(max_size) = max_size {
        let observed_size = u64::try_from(html.len()).unwrap_or(u64::MAX);
        if observed_size > max_size {
            return Err(crate::error::ConversionError::InputTooLarge {
                observed_size,
                max_size,
            });
        }
    }
    Ok(())
}

fn prepare_conversion<'a>(html: &'a str, options: &ConversionOptions) -> Result<PreparedConversion<'a>> {
    // ~keep Both tiers must see the same repaired tree when head content precedes an
    // ~keep explicit `<head>` (issue #592) or a title precedes the body of a page with no
    // ~keep `<head>` (issue #816); browsers place that content in the implicit head.
    let normalized_input = normalize_input(html)?;
    #[cfg(feature = "visitor")]
    let has_visitor = options.visitor.is_some();
    #[cfg(not(feature = "visitor"))]
    let has_visitor = false;
    let preserve_bodies = options.output_format != crate::options::OutputFormat::Plain
        && (has_visitor || !options.preserve_tags.is_empty());
    let (inert_html, mut inert_bodies) =
        crate::converter::utility::preprocessing::normalize_inert_bodies_with_preservation(
            &normalized_input,
            preserve_bodies,
            options.hidden_content,
        );
    let normalized_input = match inert_html {
        Cow::Borrowed(_) => normalized_input,
        Cow::Owned(normalized) => Cow::Owned(normalized),
    };
    // ~keep Recover the JSON boundary before routing or raw-text stripping can consume visible siblings (#875).
    let preserves_script = preserve_bodies
        && options
            .preserve_tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case("script"));
    let bodies_before_repair = preserves_script.then(|| inert_bodies.clone());
    let mut repaired_page = written_page(
        crate::converter::utility::preprocessing::repair_unclosed_structured_data_scripts(
            normalized_input.as_ref(),
            preserves_script.then_some(&mut inert_bodies),
        ),
    );
    let mut after_repair = after_script_repair(
        repaired_page.as_deref().unwrap_or_else(|| normalized_input.as_ref()),
        options,
        has_visitor,
    );
    // ~keep A page with no repaired script whose bytes are kept, which is most pages, is repaired once.
    if let Some(bodies) = bodies_before_repair.filter(|bodies| bodies.len() < inert_bodies.len()) {
        let first_repair = after_repair
            .page
            .as_deref()
            .or(repaired_page.as_deref())
            .unwrap_or_else(|| normalized_input.as_ref());
        inert_bodies = bodies;
        repaired_page = repair_with_reserved_owner_names(&normalized_input, first_repair, &mut inert_bodies, options)?;
        after_repair = after_script_repair(
            repaired_page.as_deref().unwrap_or_else(|| normalized_input.as_ref()),
            options,
            has_visitor,
        );
    }
    let AfterScriptRepair {
        page,
        head_repair_limit,
        document_base_href,
    } = after_repair;
    let normalized_html = page.or(repaired_page).map_or(normalized_input, Cow::Owned);
    let effective_base = options
        .base_url
        .as_deref()
        .and_then(|base| crate::converter::url_resolve::compute_effective_base(document_base_href.as_deref(), base))
        .map(std::rc::Rc::new);
    let metadata_base_href = document_base_href.as_ref().map(|document_base_href| {
        effective_base
            .as_deref()
            .map_or_else(|| document_base_href.clone(), |base| base.as_str().to_owned())
    });
    Ok(PreparedConversion {
        html: normalized_html,
        inert_bodies,
        effective_base,
        metadata_base_href,
        head_repair_limit,
    })
}

/// What the passes after the script repair give: the page, when one of them wrote it, the limit
/// that the head repair reached, and the base of the document.
struct AfterScriptRepair {
    page: Option<String>,
    head_repair_limit: Option<crate::types::WarningKind>,
    document_base_href: Option<String>,
}

/// The passes over `input`, the page after the script repair, in their order: the form decision,
/// the base read, the hidden strip, the form removal and the head repair.
///
/// ~keep The second script repair runs these passes again on its page, so the two repairs give
/// ~keep one page shape.
fn after_script_repair(input: &str, options: &ConversionOptions, has_visitor: bool) -> AfterScriptRepair {
    let removes_forms = crate::converter::utility::preprocessing::removes_closed_forms(input, options, has_visitor);
    // ~keep A document base stays effective when hidden stripping or form removal removes its container.
    let read_base = |source: &str| {
        (options.base_url.is_some() || options.extract_metadata)
            .then(|| crate::converter::url_resolve::document_base_href(source))
            .flatten()
    };
    let base_before_removal = removes_forms.then(|| read_base(input)).flatten();
    // ~keep Hidden content still loses form-body ownership before the form-removal pass (#873).
    let stripped = if removes_forms && options.hidden_content == crate::options::HiddenContent::Drop {
        crate::converter::utility::preprocessing::strip_hidden_elements(input)
    } else {
        Cow::Borrowed(input)
    };
    let without_forms = if removes_forms {
        crate::converter::utility::preprocessing::remove_closed_form_bodies(&stripped, options.hidden_content)
    } else {
        Cow::Borrowed(stripped.as_ref())
    };
    // ~keep The page of `drop` says whether the head is repaired, for each choice: a `<head>` tag
    // ~keep in kept content is no head of the page, and a browser ignores it there.
    let head = repair_head(&without_forms, options.hidden_content);
    let document_base_href = if removes_forms {
        base_before_removal
    } else {
        read_base(
            head.as_ref()
                .map_or_else(|| without_forms.as_ref(), |(html, _)| html.as_str()),
        )
    };
    let page_before_head = written_page(without_forms);
    let (page, head_repair_limit) = match head {
        Some((html, limit)) => (Some(html), limit),
        None => (page_before_head.or_else(|| written_page(stripped)), None),
    };
    AfterScriptRepair {
        page,
        head_repair_limit,
        document_base_href,
    }
}

/// The repair of the scripts of `input` with no end tag a second time. `first_repair` is the
/// page that the first repair and the passes after it wrote. `bodies` holds the kept bodies as
/// they were before the first repair.
///
/// ~keep The first repair wrote an owner mark. The cleanup of the conversion removes bytes, and
/// ~keep two parts of a name of the author can meet there and spell that mark. So the repair
/// ~keep writes its marks a second time, with no number that the cleaned page has: the
/// ~keep reservation of the kept scripts (`KeptScripts::reserving_owner_names_of`).
fn repair_with_reserved_owner_names(
    input: &str,
    first_repair: &str,
    bodies: &mut std::collections::HashMap<String, String>,
    options: &ConversionOptions,
) -> Result<Option<String>> {
    let reserved = crate::converter::owner_mark_numbers_after_cleanup(first_repair, options)?;
    Ok(written_page(
        crate::converter::utility::preprocessing::repair_unclosed_structured_data_scripts_reserving(
            input, bodies, &reserved,
        ),
    ))
}

/// The page of a head repair, with the limit that the repair reached.
type HeadRepair = Option<(String, Option<crate::types::WarningKind>)>;

/// The page that a pass wrote, when it wrote one.
fn written_page(page: Cow<'_, str>) -> Option<String> {
    match page {
        Cow::Borrowed(_) => None,
        Cow::Owned(page) => Some(page),
    }
}

/// The head repair of `html`, the page after the inert-body pass of `hidden_content`.
///
/// ~keep A choice that keeps hidden content takes the repair only where `drop` takes it too.
/// ~keep Kept content can hold a `<head>` tag, which is no head of the page: a browser ignores it
/// ~keep there. Read as the head, it sent only the wider choice to the repair, which then put a
/// ~keep title of the body in the head. So a wider choice empties the kept elements again and
/// ~keep asks that page first.
fn repair_head(html: &str, hidden_content: crate::options::HiddenContent) -> HeadRepair {
    let drop = crate::options::HiddenContent::Drop;
    if hidden_content != drop {
        let emptied =
            crate::converter::utility::preprocessing::normalize_inert_bodies_with_preservation(html, false, drop).0;
        if !crate::converter::has_head_content_outside_head(&emptied) {
            return None;
        }
    }
    crate::converter::repair_head_content_outside_head(html, hidden_content)
}

fn try_tier1(prepared: &PreparedConversion<'_>, options: &ConversionOptions) -> Option<ConversionResult> {
    if !options.preserve_tags.is_empty() || !prepared.inert_bodies.is_empty() {
        return None;
    }
    // ~keep A page on which the head repair reached a limit is not a page for the fast
    // ~keep converter: the full converter converts it and gives the warning of the limit.
    if prepared.head_repair_limit.is_some() {
        return None;
    }
    let report = crate::converter::prescan::PrescanReport::default();
    match options.tier_strategy {
        crate::options::TierStrategy::Tier2 => None,
        crate::options::TierStrategy::Auto => {
            let decision = crate::converter::tier1::router::classify(&report, options);
            if decision == crate::converter::tier1::RouterDecision::Tier1 {
                run_tier1(prepared, options, &report)
            } else {
                tracing::debug!(target: "html_to_markdown::convert", "router selected tier-2 conversion path directly");
                None
            }
        }
        #[cfg(any(test, feature = "testkit"))]
        crate::options::TierStrategy::Tier1 => run_tier1(prepared, options, &report),
    }
}

fn run_tier1(
    prepared: &PreparedConversion<'_>,
    options: &ConversionOptions,
    report: &crate::converter::prescan::PrescanReport,
) -> Option<ConversionResult> {
    match crate::converter::tier1::run_with_base(
        prepared.html.as_ref(),
        report,
        options,
        prepared.effective_base.clone(),
        prepared.metadata_base_href.as_deref(),
    ) {
        Ok(markdown) => {
            tracing::debug!(target: "html_to_markdown::convert", output_len = markdown.len(), "tier-1 fast-path conversion completed");
            Some(ConversionResult {
                content: Some(markdown),
                document: None,
                tables: Vec::new(),
                warnings: Vec::new(),
                #[cfg(feature = "metadata")]
                metadata: crate::metadata::HtmlMetadata::default(),
                #[cfg(feature = "inline-images")]
                images: Vec::new(),
            })
        }
        Err(bail) => {
            tracing::warn!(target: "html_to_markdown::convert", reason = %bail, "tier-1 conversion bailed; falling back to tier-2");
            None
        }
    }
}

fn run_tier2(prepared: PreparedConversion<'_>, options: ConversionOptions) -> Result<ConversionResult> {
    let collectors = create_collectors(&options)?;
    let outcome = invoke_converter(&prepared, &options, &collectors);
    let (markdown, document, tables, depth_warning) = match outcome {
        Ok(result) => result?,
        Err(panic_payload) => {
            #[cfg(feature = "visitor")]
            if let Some(handle) = &options.visitor {
                handle.clear_poison();
            }
            return Err(crate::error::ConversionError::Panic(panic_message(&*panic_payload)));
        }
    };
    finish_conversion(markdown, document, tables, depth_warning, collectors)
}

#[cfg(any(feature = "metadata", feature = "inline-images"))]
fn create_collectors(options: &ConversionOptions) -> Result<Tier2Collectors> {
    #[cfg(feature = "metadata")]
    let metadata = options.extract_metadata.then(|| {
        std::rc::Rc::new(std::cell::RefCell::new(crate::metadata::MetadataCollector::new(
            MetadataConfig::default(),
        )))
    });
    #[cfg(feature = "inline-images")]
    let images = if options.extract_images {
        let mut config = crate::inline_images::InlineImageConfig::new(options.max_image_size);
        config.capture_svg = options.capture_svg;
        config.infer_dimensions = options.infer_dimensions;
        Some(std::rc::Rc::new(std::cell::RefCell::new(
            crate::inline_images::InlineImageCollector::new(config)?,
        )))
    } else {
        None
    };
    Ok(Tier2Collectors {
        #[cfg(feature = "metadata")]
        metadata,
        #[cfg(feature = "inline-images")]
        images,
    })
}

#[cfg(not(any(feature = "metadata", feature = "inline-images")))]
const fn create_collectors(_: &ConversionOptions) -> Result<Tier2Collectors> {
    Ok(Tier2Collectors {})
}

fn invoke_converter(
    prepared: &PreparedConversion<'_>,
    options: &ConversionOptions,
    collectors: &Tier2Collectors,
) -> std::thread::Result<Result<ConvertOutput>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        call_converter(prepared, options, collectors)
    }))
}

fn call_converter(
    prepared: &PreparedConversion<'_>,
    options: &ConversionOptions,
    collectors: &Tier2Collectors,
) -> Result<ConvertOutput> {
    #[cfg(feature = "visitor")]
    let visitor = options.visitor.clone();
    let structure = options
        .include_document_structure
        .then(|| std::rc::Rc::new(std::cell::RefCell::new(crate::types::StructureCollector::new())));
    crate::converter::convert_html_impl(
        prepared.html.as_ref(),
        options,
        crate::converter::main::ConversionParameters {
            inert_bodies: &prepared.inert_bodies,
            inline_collector: image_collector(collectors),
            #[cfg(feature = "metadata")]
            metadata_collector: metadata_collector(collectors),
            #[cfg(feature = "visitor")]
            visitor,
            structure_collector: structure,
            base_url: prepared.effective_base.clone(),
            document_base_href: prepared.metadata_base_href.as_deref(),
            head_repair_limit: prepared.head_repair_limit,
        },
    )
}

#[cfg(feature = "inline-images")]
fn image_collector(collectors: &Tier2Collectors) -> Option<ImageCollectorHandle> {
    collectors.images.as_ref().map(std::rc::Rc::clone)
}

#[cfg(not(feature = "inline-images"))]
const fn image_collector(_: &Tier2Collectors) -> Option<()> {
    None
}

#[cfg(feature = "metadata")]
fn metadata_collector(collectors: &Tier2Collectors) -> Option<MetadataCollectorHandle> {
    collectors.metadata.as_ref().map(std::rc::Rc::clone)
}

fn finish_conversion(
    markdown: String,
    document: Option<crate::types::DocumentStructure>,
    tables: Vec<crate::types::TableData>,
    depth_warning: Option<crate::types::ProcessingWarning>,
    collectors: Tier2Collectors,
) -> Result<ConversionResult> {
    let _ = &collectors;
    #[cfg(feature = "metadata")]
    let metadata = finish_metadata(collectors.metadata)?;
    #[cfg(feature = "inline-images")]
    let (images, mut warnings) = finish_images(collectors.images)?;
    #[cfg(not(feature = "inline-images"))]
    let mut warnings = Vec::new();
    if let Some(warning) = depth_warning {
        warnings.push(warning);
    }
    Ok(ConversionResult {
        content: Some(markdown),
        document,
        #[cfg(feature = "metadata")]
        metadata,
        tables,
        #[cfg(feature = "inline-images")]
        images,
        warnings,
    })
}

#[cfg(feature = "metadata")]
fn finish_metadata(collector: Option<MetadataCollectorHandle>) -> Result<HtmlMetadata> {
    collector.map_or_else(
        || Ok(HtmlMetadata::default()),
        |collector| {
            Ok(std::rc::Rc::try_unwrap(collector)
                .map_err(|_| ConversionError::Other("failed to recover metadata state".to_string()))?
                .into_inner()
                .finish())
        },
    )
}

#[cfg(feature = "inline-images")]
fn finish_images(
    collector: Option<ImageCollectorHandle>,
) -> Result<(
    Vec<crate::inline_images::InlineImage>,
    Vec<crate::types::ProcessingWarning>,
)> {
    let Some(collector) = collector else {
        return Ok((Vec::new(), Vec::new()));
    };
    let collector = std::rc::Rc::try_unwrap(collector)
        .map_err(|_| ConversionError::Other("failed to recover inline image state".to_string()))?
        .into_inner();
    let (images, warnings) = collector.finish();
    let warnings = warnings
        .into_iter()
        .map(|warning| crate::types::ProcessingWarning {
            kind: crate::types::WarningKind::ImageExtractionFailed,
            message: warning.message,
        })
        .collect();
    Ok((images, warnings))
}

/// Extract a human-readable message from a `catch_unwind` panic payload.
///
/// Panics started via `panic!("{}", msg)` carry a `String`; panics started via a string
/// literal (`panic!("msg")`) carry a `&'static str`. Anything else falls back to a generic
/// message rather than losing the error entirely.
fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "visitor callback panicked during conversion".to_string()
    }
}

/// Validate and normalize HTML input for conversion. A leading byte order mark is dropped, as
/// a browser's decoder drops it, so it is not text that starts the body.
fn normalize_input(html: &str) -> Result<Cow<'_, str>> {
    let decoded = decode_utf16_if_needed(html);
    match decoded {
        Cow::Borrowed(borrowed) => {
            let borrowed = borrowed.strip_prefix('\u{FEFF}').unwrap_or(borrowed);
            validate_input(borrowed)?;
            let sanitized = strip_nul_bytes(borrowed);
            let line_normalized = match sanitized {
                Cow::Borrowed(b) => normalize_line_endings(b),
                Cow::Owned(o) => Cow::Owned(normalize_line_endings(&o).into_owned()),
            };
            Ok(fix_xhtml_self_closing(line_normalized))
        }
        Cow::Owned(mut owned) => {
            validate_input(&owned)?;
            if owned.contains('\0') {
                owned = owned.replace('\0', "");
            }
            if owned.contains('\r') {
                owned = owned.replace("\r\n", "\n").replace('\r', "\n");
            }
            Ok(fix_xhtml_self_closing(Cow::Owned(owned)))
        }
    }
}

/// Insert a space before `/>` in XHTML-style self-closing tags so the underlying
/// HTML parser does not greedily consume the trailing slash as part of the tag name.
///
/// The bundled astral-tl parser treats `/` as an identifier character, so `<td/>`
/// is parsed as a tag literally named `"td/"` and subsequent siblings become its
/// children — silently truncating the table and dropping the rest of the document.
/// Rewriting to `<td />` (with a space) lets the parser recognise the self-closing
/// syntax correctly. EPUB/XHTML-derived HTML uses this form heavily for empty
/// table cells; see issue #391.
fn fix_xhtml_self_closing(html: Cow<'_, str>) -> Cow<'_, str> {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"<([a-zA-Z][a-zA-Z0-9_:.\-]*)/>").expect("XHTML self-closing regex is well-formed")
    });
    if !html.contains("/>") {
        return html;
    }
    match re.replace_all(html.as_ref(), "<$1 />") {
        Cow::Borrowed(_) => html,
        Cow::Owned(s) => Cow::Owned(s),
    }
}

/// Attempt to decode UTF-16 HTML that was provided as a lossy UTF-8 string.
///
/// Some callers read raw bytes and convert with `from_utf8_lossy`, which preserves
/// the NUL-byte pattern of UTF-16 input. When we detect that pattern, we can
/// recover the original HTML instead of rejecting it as binary data.
fn decode_utf16_if_needed(html: &str) -> Cow<'_, str> {
    let bytes = html.as_bytes();
    if !bytes.contains(&0) {
        return Cow::Borrowed(html);
    }

    let Some(encoding) = detect_utf16_encoding(bytes) else {
        return Cow::Borrowed(html);
    };

    let decoded = decode_utf16_bytes(bytes, encoding);
    if decoded.is_empty() {
        Cow::Borrowed(html)
    } else {
        Cow::Owned(decoded)
    }
}

fn decode_utf16_bytes(bytes: &[u8], encoding: Utf16Encoding) -> String {
    let (is_little_endian, skip_bom) = match encoding {
        Utf16Encoding::BomLe => (true, true),
        Utf16Encoding::BomBe => (false, true),
        Utf16Encoding::NoBomLe => (true, false),
        Utf16Encoding::NoBomBe => (false, false),
    };

    let mut units = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let unit = if is_little_endian {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        };
        units.push(unit);
    }

    let mut decoded = String::from_utf16_lossy(&units);
    if skip_bom {
        decoded = decoded.trim_start_matches('\u{FEFF}').to_string();
    }
    decoded
}

/// Strip NUL bytes that can appear in malformed HTML inputs.
fn strip_nul_bytes(html: &str) -> Cow<'_, str> {
    if html.contains('\0') {
        Cow::Owned(html.replace('\0', ""))
    } else {
        Cow::Borrowed(html)
    }
}

/// Normalize line endings in HTML input.
///
/// Converts CRLF and CR line endings to LF for consistent processing.
fn normalize_line_endings(html: &str) -> Cow<'_, str> {
    if html.contains('\r') {
        Cow::Owned(html.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(html)
    }
}

/// Fast path for plain text (no HTML) conversion.
///
/// Skips HTML parsing if no angle brackets are present.
fn fast_text_only(html: &str, options: &ConversionOptions) -> Option<String> {
    if html.contains('<') {
        return None;
    }

    let mut decoded = text::decode_html_entities_cow(html);
    if options.strip_newlines && (decoded.contains('\n') || decoded.contains('\r')) {
        decoded = Cow::Owned(decoded.replace(&['\r', '\n'][..], " "));
    }
    let trimmed = decoded.trim_end_matches('\n');
    // ~keep Whitespace-only input renders as nothing in a browser, so it converts to "" —
    // ~keep not to the lone "\n" the trailing-space pop plus unconditional newline below
    // ~keep would otherwise emit.
    if trimmed.bytes().all(|byte| byte.is_ascii_whitespace()) {
        return Some(String::new());
    }

    let normalized = if options.whitespace_mode == WhitespaceMode::Normalized {
        text::normalize_whitespace_cow(trimmed)
    } else {
        Cow::Borrowed(trimmed)
    };

    let escape_asterisks = options.escape_asterisks
        || (options.output_format == crate::options::OutputFormat::Djot
            && text::is_djot_rule_like(normalized.as_ref()));
    let escaped = if options.output_format == crate::options::OutputFormat::Plain {
        normalized.into_owned()
    } else if options.escape_misc
        || escape_asterisks
        || options.escape_underscores
        || options.escape_ascii
        // ~keep `text::escape` escapes a literal backslash regardless of the flags above,
        // ~keep so this gate must open on a bare backslash too or the tag-free fast path
        // ~keep would silently skip it under default options.
        || normalized.contains('\\')
    {
        text::escape(
            normalized.as_ref(),
            options.escape_misc,
            escape_asterisks,
            options.escape_underscores,
            options.escape_ascii,
        )
        .into_owned()
    } else {
        normalized.into_owned()
    };

    let mut output = String::with_capacity(escaped.len() + 1);
    output.push_str(&escaped);
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
    output.push('\n');
    Some(output)
}
