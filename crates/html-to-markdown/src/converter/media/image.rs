//! Image element handling (img, data URIs, inline image collection).

#[allow(unused_imports)]
use std::collections::BTreeMap;

#[cfg(feature = "inline-images")]
use crate::inline_images::{InlineImageCollector, InlineImageFormat, InlineImageSource};

#[cfg(feature = "inline-images")]
type InlineCollectorHandle = std::rc::Rc<std::cell::RefCell<InlineImageCollector>>;

/// Handle inline data URIs in img elements with base64 encoding validation.
///
/// # Features
/// - Base64 decoding with size limits
/// - MIME type validation
/// - Metadata extraction (width, height, alt text)
/// - Format detection (PNG, JPEG, GIF, BMP, WebP, SVG)
#[cfg(feature = "inline-images")]
#[allow(clippy::items_after_statements)]
#[allow(clippy::manual_let_else)]
pub(crate) fn handle_inline_data_image(
    collector_ref: &InlineCollectorHandle,
    src: &str,
    alt: &str,
    title: Option<&str>,
    attributes: BTreeMap<String, String>,
) {
    let trimmed_src = src.trim();
    if !trimmed_src.starts_with("data:") {
        return;
    }

    let mut collector = collector_ref.borrow_mut();
    let index = collector.next_index();

    let Some((meta, payload)) = trimmed_src.split_once(',') else {
        collector.warn_skip(index, "missing data URI separator");
        return;
    };

    if payload.trim().is_empty() {
        collector.warn_skip(index, "empty data URI payload");
        return;
    }

    if !meta.starts_with("data:") {
        collector.warn_skip(index, "invalid data URI scheme");
        return;
    }

    let header = &meta["data:".len()..];
    if header.is_empty() {
        collector.warn_skip(index, "missing MIME type");
        return;
    }

    let mut segments = header.split(';');
    let mime = segments.next().unwrap_or("");
    let Some((top_level, subtype_raw)) = mime.split_once('/') else {
        collector.warn_skip(index, "missing MIME subtype");
        return;
    };

    if !top_level.eq_ignore_ascii_case("image") {
        collector.warn_skip(index, format!("unsupported MIME type {mime}"));
        return;
    }

    let subtype_raw = subtype_raw.trim();
    if subtype_raw.is_empty() {
        collector.warn_skip(index, "missing MIME subtype");
        return;
    }

    let mut is_base64 = false;
    let mut inline_name: Option<String> = None;
    for segment in segments {
        if segment.eq_ignore_ascii_case("base64") {
            is_base64 = true;
        } else if let Some(value) = segment.strip_prefix("name=") {
            inline_name = non_empty_trimmed(value.trim_matches('"'));
        } else if let Some(value) = segment.strip_prefix("filename=") {
            inline_name = non_empty_trimmed(value.trim_matches('"'));
        }
    }

    if !is_base64 {
        collector.warn_skip(index, "missing base64 encoding marker");
        return;
    }

    use base64::{Engine as _, engine::general_purpose::STANDARD};

    let payload_clean = payload.trim();
    let max_size = collector.max_decoded_size();
    let max_encoded = max_size.saturating_div(3).saturating_mul(4).saturating_add(4);
    if payload_clean.len() as u64 > max_encoded {
        collector.warn_skip(
            index,
            format!(
                "encoded payload ({} bytes) exceeds configured max ({})",
                payload_clean.len(),
                max_size
            ),
        );
        return;
    }

    let decoded = if let Ok(bytes) = STANDARD.decode(payload_clean) {
        bytes
    } else {
        collector.warn_skip(index, "invalid base64 payload");
        return;
    };

    if decoded.is_empty() {
        collector.warn_skip(index, "empty base64 payload");
        return;
    }

    if decoded.len() as u64 > max_size {
        collector.warn_skip(
            index,
            format!(
                "decoded payload ({} bytes) exceeds configured max ({})",
                decoded.len(),
                max_size
            ),
        );
        return;
    }

    let format = if subtype_raw.eq_ignore_ascii_case("png") {
        InlineImageFormat::Png
    } else if subtype_raw.eq_ignore_ascii_case("jpeg") || subtype_raw.eq_ignore_ascii_case("jpg") {
        InlineImageFormat::Jpeg
    } else if subtype_raw.eq_ignore_ascii_case("gif") {
        InlineImageFormat::Gif
    } else if subtype_raw.eq_ignore_ascii_case("bmp") {
        InlineImageFormat::Bmp
    } else if subtype_raw.eq_ignore_ascii_case("webp") {
        InlineImageFormat::Webp
    } else if subtype_raw.eq_ignore_ascii_case("svg+xml") {
        InlineImageFormat::Svg
    } else {
        InlineImageFormat::Other(subtype_raw.to_ascii_lowercase())
    };

    let description = non_empty_trimmed(alt).or_else(|| title.and_then(non_empty_trimmed));

    let filename_candidate = attributes
        .get("data-filename")
        .cloned()
        .or_else(|| attributes.get("filename").cloned())
        .or_else(|| attributes.get("data-name").cloned())
        .or(inline_name);

    let dimensions = collector.infer_dimensions(index, &decoded, &format);

    let image = collector.build_image(
        decoded,
        format,
        filename_candidate,
        description,
        dimensions,
        InlineImageSource::ImgDataUri,
        attributes,
    );

    collector.push_image(index, image);
}

/// Check if heading tag allows inline images based on configuration.
pub(crate) fn heading_allows_inline_images(
    tag_name: &str,
    keep_inline_images_in: &std::collections::HashSet<String>,
) -> bool {
    keep_inline_images_in.contains(tag_name)
}

/// Extract non-empty trimmed string or return None.
#[cfg(feature = "inline-images")]
fn non_empty_trimmed(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
