//! Image element handler for HTML to Markdown conversion.
//!
//! Handles `<img>` elements including:
//! - Basic image markdown output `![alt](src "title")`
//! - Metadata collection for image extraction
//! - Inline data URI image handling
//! - Visitor callback integration

use std::borrow::Cow;
#[cfg(any(feature = "metadata", feature = "inline-images"))]
use std::collections::BTreeMap;

use crate::converter::Context;
use crate::converter::dom_context::DomContext;
use crate::converter::handlers::srcset::{Descriptor, parse_descriptor, srcset_candidates};
use crate::converter::inline::HandlerContext;
use crate::converter::inline::link::{append_url_destination, escape_markdown_title};
use crate::converter::main_helpers::tag_name_eq;
use crate::converter::media::is_inline_data;
use crate::converter::utility::attributes::decoded_attribute;
use crate::converter::utility::escaping::escape_image_alt;
use crate::converter::utility::preprocessing::sanitize_markdown_url;
use crate::options::InlineDataMedia;

#[cfg(feature = "inline-images")]
use crate::converter::media::handle_inline_data_image;

#[cfg(feature = "visitor")]
use crate::converter::utility::serialization::serialize_node;

#[cfg(feature = "metadata")]
type ImageMetadataPayload = (BTreeMap<String, String>, Option<u32>, Option<u32>);

struct ImageData<'a> {
    src: Cow<'a, str>,
    alt: Cow<'a, str>,
    title: Option<Cow<'a, str>>,
}

/// Handle an `<img>` element and convert to Markdown.
///
/// This handler processes image elements including:
/// - Extracting src, alt, and title attributes
/// - Collecting metadata when the metadata feature is enabled
/// - Handling inline data URIs when the inline-images feature is enabled
/// - Invoking visitor callbacks when the visitor feature is enabled
/// - Generating appropriate markdown output
pub fn handle_img(tag: &tl::HTMLTag, handler: HandlerContext<'_>) {
    let data = image_data(tag, &handler);
    #[cfg(feature = "metadata")]
    let metadata = handler.context.metadata_wants.images.then(|| image_metadata(tag));
    #[cfg(feature = "inline-images")]
    collect_inline_image(tag, &data, handler.context);
    let inline_data = handler
        .context
        .inline_data_treatment(handler.options.inline_data_media, &data.src);
    let rendered = render_image(tag, &data, inline_data, &handler);
    if !handler.options.skip_images {
        if let Some(image_text) = rendered {
            handler.output.push_str(&image_text);
        }
    }
    #[cfg(feature = "metadata")]
    record_image_metadata(&data, metadata, handler.context);
    record_image_structure(&data, inline_data, handler.context);
}

fn image_data<'a>(tag: &'a tl::HTMLTag<'a>, handler: &HandlerContext<'_>) -> ImageData<'a> {
    let src = {
        let skip_inline_data = handler.options.inline_data_media != InlineDataMedia::Keep;
        let mut effective_src = resolve_effective_src(tag, skip_inline_data);
        if skip_inline_data && is_inline_data(&effective_src) {
            if let Some(source_src) = picture_source_src(handler.node_handle, handler.parser, handler.dom_context) {
                effective_src = Cow::Owned(source_src);
            }
        }
        let base_resolved =
            handler
                .context
                .resolve_url(&effective_src, handler.node_handle, handler.parser, handler.dom_context);
        Cow::Owned(sanitize_markdown_url(base_resolved.as_deref().unwrap_or(&effective_src)).into_owned())
    };

    let alt = decoded_attribute(tag, "alt").unwrap_or(Cow::Borrowed(""));

    // ~keep An empty `title=""` carries no information, and `[t](u "")` / `![a](i "")` is
    // ~keep noise that no Markdown serializer round-trips: re-rendering the output drops
    // ~keep the empty title, so the second pass no longer matches the first. Treat it as
    // ~keep absent, which is what it means.
    let title = decoded_attribute(tag, "title").filter(|v| !v.is_empty());

    ImageData { src, alt, title }
}

#[cfg(feature = "metadata")]
fn image_metadata(tag: &tl::HTMLTag<'_>) -> ImageMetadataPayload {
    let mut attributes = BTreeMap::new();
    let mut width = None;
    let mut height = None;
    for (key, value) in tag.attributes().iter() {
        let key = key.to_string();
        if key == "src" {
            continue;
        }
        let value = value.map(|value| value.to_string()).unwrap_or_default();
        match key.as_str() {
            "width" => width = value.parse().ok(),
            "height" => height = value.parse().ok(),
            _ => {}
        }
        attributes.insert(key, value);
    }
    (attributes, width, height)
}

#[cfg(feature = "inline-images")]
fn collect_inline_image(tag: &tl::HTMLTag<'_>, data: &ImageData<'_>, context: &Context) {
    let Some(collector) = context.inline_collector.as_ref() else {
        return;
    };
    if !is_inline_data(&data.src) {
        return;
    }
    let mut attributes = BTreeMap::new();
    for (key, value) in tag.attributes().iter() {
        let key = key.to_string();
        let keep = matches!(key.as_str(), "width" | "height" | "filename" | "aria-label") || key.starts_with("data-");
        if keep {
            attributes.insert(key, value.map(|value| value.to_string()).unwrap_or_default());
        }
    }
    handle_inline_data_image(
        collector,
        data.src.as_ref(),
        data.alt.as_ref(),
        data.title.as_deref(),
        attributes,
    );
}

#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
fn render_image(
    tag: &tl::HTMLTag<'_>,
    data: &ImageData<'_>,
    inline_data: InlineDataMedia,
    handler: &HandlerContext<'_>,
) -> Option<String> {
    // ~keep #492: `|| ctx.link_allow_inline_images` is purely additive relative to the
    // ~keep pre-#492 expression -- it can only turn an image ON, never off, because
    // ~keep `link_allow_inline_images` is `false` whenever "a" is absent from
    // ~keep `keep_inline_images_in` (its only source, `handlers/link.rs`). So callers who
    // ~keep never set the option see byte-identical output. Deliberately NOT mirrored with
    // ~keep the heading pattern's negative clause (`|| (in_link && !link_allow)`): the no-`<p>`
    // ~keep control already renders an image through the inline branch with
    // ~keep `convert_as_inline == false`, so `should_use_alt_text` is `false` there today and
    // ~keep the image survives -- a negative term would delete it, which is a real regression.
    let context = handler.context;
    let keep_as_markdown = (context.in_heading && context.heading_allow_inline_images)
        || context.cell_allow_inline_images
        || context.link_allow_inline_images;

    let should_use_alt_text = inline_data == InlineDataMedia::AltTextOnly
        || (!keep_as_markdown
            && (context.convert_as_inline || (context.in_heading && !context.heading_allow_inline_images)));
    #[cfg(feature = "visitor")]
    if let std::ops::ControlFlow::Break(result) = visit_image(tag, data, handler) {
        return result;
    }
    // ~keep Code shows every character as text, and a browser shows no text for an image. Only
    // ~keep what a visitor writes for the image is written in code.
    if handler.context.in_code {
        return None;
    }
    render_image_default(data, inline_data, should_use_alt_text, handler)
}

fn render_image_default(
    data: &ImageData<'_>,
    inline_data: InlineDataMedia,
    use_alt_only: bool,
    handler: &HandlerContext<'_>,
) -> Option<String> {
    (inline_data != InlineDataMedia::DropElement).then(|| {
        let rendered = format_image_markdown(
            &data.src,
            &data.alt,
            data.title.as_deref(),
            ImageFormatOptions {
                use_alt_only,
                link_style: handler.options.link_style,
                url_escape_style: handler.options.url_escape_style,
                reference_collector: handler.context.reference_collector.as_ref(),
            },
        );
        crate::converter::utility::escaping::escape_djot_table_cell_literal(
            &rendered,
            handler.options.output_format,
            handler.context.in_table_cell,
        )
        .into_owned()
    })
}

#[cfg(feature = "visitor")]
fn visit_image(
    tag: &tl::HTMLTag<'_>,
    data: &ImageData<'_>,
    handler: &HandlerContext<'_>,
) -> std::ops::ControlFlow<Option<String>> {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(visitor_handle) = handler.context.visitor.as_ref() else {
        return std::ops::ControlFlow::Continue(());
    };
    let node_id = handler.node_handle.get_inner();
    let node_context = NodeContext::with_lazy_attributes(
        NodeType::Image,
        Cow::Borrowed("img"),
        tag,
        handler.depth,
        handler.dom_context.get_sibling_index(node_id).unwrap_or(0),
        handler
            .dom_context
            .parent_tag_name(node_id, handler.parser)
            .map(Cow::Borrowed),
        true,
    );
    let result = visitor_handle.lock().expect("visitor mutex poisoned").visit_image(
        &node_context,
        &data.src,
        &data.alt,
        data.title.as_deref(),
    );
    match result {
        VisitResult::Continue => std::ops::ControlFlow::Continue(()),
        VisitResult::Custom(custom) => std::ops::ControlFlow::Break(Some(custom)),
        VisitResult::Skip => std::ops::ControlFlow::Break(None),
        VisitResult::Error(error) => {
            if handler.context.visitor_error.borrow().is_none() {
                *handler.context.visitor_error.borrow_mut() = Some(error);
            }
            std::ops::ControlFlow::Break(None)
        }
        VisitResult::PreserveHtml => {
            std::ops::ControlFlow::Break(Some(serialize_node(handler.node_handle, handler.parser)))
        }
    }
}

#[cfg(feature = "metadata")]
fn record_image_metadata(data: &ImageData<'_>, metadata: Option<ImageMetadataPayload>, context: &Context) {
    let Some((attributes, width, height)) = metadata else {
        return;
    };
    let Some(collector) = context.metadata_collector.as_ref() else {
        return;
    };
    if data.src.is_empty() {
        return;
    }
    let dimensions = match (width, height) {
        (Some(width), Some(height)) => Some(crate::metadata::ImageDimensions { width, height }),
        _ => None,
    };
    collector.borrow_mut().add_image(
        data.src.to_string(),
        (!data.alt.is_empty()).then(|| data.alt.to_string()),
        data.title.as_deref().map(str::to_string),
        dimensions,
        attributes,
    );
}

fn record_image_structure(data: &ImageData<'_>, inline_data: InlineDataMedia, context: &Context) {
    // ~keep The structure shows the image the markdown shows: no node for a dropped element, and no
    // ~keep address when only the alt text is written.
    if let Some(ref sc) = context.structure_collector {
        if inline_data != InlineDataMedia::DropElement {
            let src_opt = if data.src.is_empty() || inline_data == InlineDataMedia::AltTextOnly {
                None
            } else {
                Some(data.src.as_ref())
            };
            let alt_opt = if data.alt.is_empty() {
                None
            } else {
                Some(data.alt.as_ref())
            };
            sc.borrow_mut().push_image(src_opt, alt_opt);
        }
    }
}

/// Single-URL attributes checked, in priority order, when `src` is missing or a
/// `data:` URI. See [`resolve_effective_src`] for the full precedence rule.
const LAZY_SINGLE_URL_ATTRIBUTES: [&str; 3] = ["data-src", "data-lazy-src", "data-original"];

/// Resolve the effective image address for an `<img>` element, falling back to
/// common lazy-loading attributes when `src` is empty or a `data:` URI.
///
/// ~keep Precedence, and why:
/// ~keep 1. `src`, when non-empty and not a `data:` URI — trusted as-is even if it
/// ~keep    happens to be a tiny placeholder graphic, because this crate cannot fetch
/// ~keep    the URL to inspect its pixel dimensions, and some real pages already have
/// ~keep    the genuine image in `src` while only the *other* attributes carry lazy
/// ~keep    -load scaffolding (see `test_documents/html/issues/gh-190/rbloggers.html`'s
/// ~keep    "jetpack-lazy-image" `<img>`: `src` is already the real photo URL, while
/// ~keep    `srcset` holds nothing but the lazy-load 1x1 `data:` placeholder — treating
/// ~keep    every populated `src` with suspicion would discard that real image).
/// ~keep 2. `data-src`, `data-lazy-src`, `data-original`, in that order — each holds
/// ~keep    exactly one URL, deliberately written by a lazy-load library as "the real
/// ~keep    address", so they outrank the multi-candidate srcset attributes below,
/// ~keep    which require guessing which listed candidate is "best".
/// ~keep 3. `data-srcset`, then `srcset` — each may list several URLs with width/
/// ~keep    density descriptors; the highest-resolution candidate is used (see
/// ~keep    `pick_best_srcset_candidate`). `data-srcset` (explicitly a lazy-load
/// ~keep    attribute) outranks plain `srcset`, which may itself have been
/// ~keep    placeholder-ized by the same lazy-load pass that placeholder-ized `src`.
/// ~keep 4. Otherwise, the original `src` (possibly empty, possibly a `data:` URI) is
/// ~keep    kept unchanged. This is also what makes a plain `<img src="...">` with none
/// ~keep    of the above attributes byte-identical to output produced before this
/// ~keep    fallback existed.
///
/// With `skip_inline_data`, every step first passes over a `data:` value or candidate in any case,
/// so a real address anywhere wins over an inline payload; when there is none, the address the
/// steps pick without skipping is used, so the caller sees the `data:` payload and not an empty `src`.
fn resolve_effective_src<'a>(tag: &'a tl::HTMLTag<'a>, skip_inline_data: bool) -> Cow<'a, str> {
    // ~keep Every read here goes through `decoded_attribute`: a URL attribute carries
    // ~keep character references like any other (`src="i.png?a=1&amp;b=2"`), and `srcset` is
    // ~keep parsed *after* decoding because the entity is not part of its comma/descriptor
    // ~keep grammar. Issue #494.
    let raw_src = decoded_attribute(tag, "src").unwrap_or(Cow::Borrowed(""));
    if !raw_src.trim().is_empty() && !is_inline_data(&raw_src) {
        return raw_src;
    }
    skip_inline_data
        .then(|| fallback_src(tag, true))
        .flatten()
        .or_else(|| fallback_src(tag, false))
        .unwrap_or(raw_src)
}

/// The address a browser loads from the `<source>` elements of the `<picture>` that holds this
/// `<img>`: the best `srcset` candidate that is not a `data:` URL, from the first `<source>` before
/// the image that has one. `media` and `type` are not evaluated, so the first such source wins.
fn picture_source_src(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> Option<String> {
    let parent_id = dom_ctx.parent_of(node_handle.get_inner())?;
    let Some(tl::Node::Tag(picture)) = dom_ctx.node_handle(parent_id)?.get(parser) else {
        return None;
    };
    if !tag_name_eq(picture.name().as_utf8_str(), "picture") {
        return None;
    }
    picture
        .children()
        .top()
        .iter()
        .take_while(|child| child.get_inner() != node_handle.get_inner())
        .filter_map(|child| match child.get(parser) {
            Some(tl::Node::Tag(source)) if tag_name_eq(source.name().as_utf8_str(), "source") => Some(source),
            _ => None,
        })
        .find_map(|source| {
            let srcset = decoded_attribute(source, "srcset")?;
            pick_best_srcset_candidate(&srcset, true).map(str::to_owned)
        })
}

/// Steps 2 and 3 of [`resolve_effective_src`]: the first lazy-load address, then the best
/// `srcset` candidate, passing over `data:` values when `skip_inline_data` is set.
fn fallback_src<'a>(tag: &'a tl::HTMLTag<'a>, skip_inline_data: bool) -> Option<Cow<'a, str>> {
    let is_placeholder = |value: &str| value.trim().is_empty() || (skip_inline_data && is_inline_data(value));

    for attr_name in LAZY_SINGLE_URL_ATTRIBUTES {
        if let Some(value) = decoded_attribute(tag, attr_name) {
            if !is_placeholder(&value) {
                return Some(value);
            }
        }
    }

    for attr_name in ["data-srcset", "srcset"] {
        if let Some(value) = decoded_attribute(tag, attr_name) {
            if let Some(candidate) = pick_best_srcset_candidate(&value, skip_inline_data) {
                return Some(Cow::Owned(candidate.to_string()));
            }
        }
    }

    None
}

/// Parse a `srcset`-shaped attribute value and return the URL of its highest
/// -resolution candidate.
///
/// ~keep `srcset` lists `"<url> [descriptors]"` candidates separated by commas. Markdown has
/// ~keep no responsive-image equivalent, so this picks one URL, the highest-quality image
/// ~keep offered (a high-resolution image degrades gracefully wherever it is viewed; a
/// ~keep low-resolution one does not). Candidates are split and their descriptors parsed as the
/// ~keep HTML spec does (see `srcset_candidates` and `parse_descriptor`), and a candidate the
/// ~keep spec drops is never chosen. A browser compares a width with a density only through the
/// ~keep `sizes` value and the viewport, which a converter does not have, so the kinds are not
/// ~keep compared: when any candidate has a width, the largest width wins (widths keep their
/// ~keep order under every `sizes` value); otherwise the largest density wins, a candidate with no
/// ~keep descriptor counting as `1x`. On a tie the first candidate wins, as in the spec.
fn pick_best_srcset_candidate(value: &str, skip_inline_data: bool) -> Option<&str> {
    let mut widest: Option<(&str, f64)> = None;
    let mut densest: Option<(&str, f64)> = None;

    for (url, descriptor) in srcset_candidates(value) {
        let (best, score) = match parse_descriptor(descriptor) {
            Some(Descriptor::Width(width)) => (&mut widest, width),
            Some(Descriptor::Density(density)) => (&mut densest, density),
            None => continue,
        };
        if skip_inline_data && is_inline_data(url) {
            continue;
        }
        if best.is_none_or(|(_, best_score)| score > best_score) {
            *best = Some((url, score));
        }
    }

    widest.or(densest).map(|(url, _)| url)
}

/// Format an image as Markdown syntax.
///
/// If `use_alt_only` is true, returns just the alt text.
/// Otherwise returns the full `![alt](src "title")` syntax.
///
/// The `url_escape_style` controls how the `src` URL is escaped:
/// - [`UrlEscapeStyle::Angle`] (default) — wraps `src` in angle brackets when it contains spaces.
/// - [`UrlEscapeStyle::Percent`] — percent-encodes every non-unreserved character.
fn format_image_markdown(src: &str, alt: &str, title: Option<&str>, format: ImageFormatOptions<'_>) -> String {
    if format.use_alt_only {
        return alt.to_string();
    }
    let escaped_alt = escape_image_alt(alt);
    if format.link_style == crate::options::validation::LinkStyle::Reference {
        if let Some(collector) = format.reference_collector {
            let ref_num = collector.borrow_mut().get_or_insert(src, title);
            let mut buf = String::with_capacity(escaped_alt.len() + 10);
            buf.push_str("![");
            buf.push_str(&escaped_alt);
            buf.push_str("][");
            buf.push_str(&ref_num.to_string());
            buf.push(']');
            return buf;
        }
    }
    let mut buf = String::with_capacity(src.len() + escaped_alt.len() + 10);
    buf.push_str("![");
    buf.push_str(&escaped_alt);
    buf.push_str("](");

    append_url_destination(&mut buf, src, format.url_escape_style, title.is_some());

    if let Some(title_text) = title {
        buf.push_str(" \"");
        buf.push_str(&escape_markdown_title(title_text));
        buf.push('"');
    }
    buf.push(')');
    buf
}

struct ImageFormatOptions<'a> {
    use_alt_only: bool,
    link_style: crate::options::validation::LinkStyle,
    url_escape_style: crate::options::validation::UrlEscapeStyle,
    reference_collector: Option<&'a crate::converter::reference_collector::ReferenceCollectorHandle>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::validation::{LinkStyle, UrlEscapeStyle};

    fn image_format(
        link_style: LinkStyle,
        url_escape_style: UrlEscapeStyle,
        reference_collector: Option<&crate::converter::reference_collector::ReferenceCollectorHandle>,
    ) -> ImageFormatOptions<'_> {
        ImageFormatOptions {
            use_alt_only: false,
            link_style,
            url_escape_style,
            reference_collector,
        }
    }

    #[test]
    fn a_srcset_skips_data_candidates_only_when_asked() {
        let srcset = "real.png 1x, data:x 3x";
        assert_eq!(pick_best_srcset_candidate(srcset, false), Some("data:x"));
        assert_eq!(pick_best_srcset_candidate(srcset, true), Some("real.png"));
        assert_eq!(pick_best_srcset_candidate("DATA:x 1x", true), None);
    }

    #[test]
    fn format_image_markdown_handles_multiline_alt_text() {
        let result = format_image_markdown(
            "image.png",
            "line one\nline two\nline three",
            None,
            image_format(LinkStyle::Inline, UrlEscapeStyle::Angle, None),
        );

        assert_eq!(result, "![line one\nline two\nline three](image.png)");
    }

    #[test]
    fn format_image_markdown_escapes_blank_lines_in_alt_text() {
        let result = format_image_markdown(
            "S",
            "A\n\n B C",
            None,
            image_format(LinkStyle::Inline, UrlEscapeStyle::Angle, None),
        );

        assert_eq!(result, "![A&#10;\n B C](S)");
    }

    #[test]
    fn format_image_markdown_angle_wraps_space() {
        let result = format_image_markdown(
            "/img (1).png",
            "alt",
            None,
            image_format(LinkStyle::Inline, UrlEscapeStyle::Angle, None),
        );
        assert_eq!(result, "![alt](</img (1).png>)");
    }

    #[test]
    fn format_image_markdown_percent_encodes_space_and_parens() {
        let result = format_image_markdown(
            "/img (1).png",
            "alt",
            None,
            image_format(LinkStyle::Inline, UrlEscapeStyle::Percent, None),
        );
        assert_eq!(result, "![alt](/img%20%281%29.png)");
    }

    // ~keep Regression for the CommonMark spec fixpoint gap (spec example 520):
    // ~keep `<img alt="[foo](uri2)">` previously emitted `![[foo](uri2)](uri3)`, which
    // ~keep reparses with the alt text's `[foo](uri2)` becoming a real nested link --
    // ~keep CommonMark parses an image's alt as full inline content, so only `foo`
    // ~keep survived into the alt attribute on a second conversion and `uri2` was lost.
    #[test]
    fn format_image_markdown_escapes_link_shaped_alt_text() {
        let result = format_image_markdown(
            "uri3",
            "[foo](uri2)",
            None,
            image_format(LinkStyle::Inline, UrlEscapeStyle::Angle, None),
        );
        assert_eq!(result, "![\\[foo\\](uri2)](uri3)");
    }

    #[test]
    fn format_image_markdown_percent_encodes_angle_brackets() {
        let result = format_image_markdown(
            "/img (1) <draft>.png",
            "alt",
            None,
            image_format(LinkStyle::Inline, UrlEscapeStyle::Percent, None),
        );
        assert_eq!(result, "![alt](/img%20%281%29%20%3Cdraft%3E.png)");
    }

    #[test]
    fn format_image_markdown_angle_plain_url_unchanged() {
        let result = format_image_markdown(
            "https://example.com/img.png",
            "photo",
            None,
            image_format(LinkStyle::Inline, UrlEscapeStyle::Angle, None),
        );
        assert_eq!(result, "![photo](https://example.com/img.png)");
    }

    #[test]
    fn should_escape_alt_text_that_would_close_the_image_and_open_a_new_link() {
        // ~keep audit #24 finding 1: an inert `alt` containing `]` and `(` must not be able to
        // terminate the image label early and start a second, attacker-controlled link.
        let result = format_image_markdown(
            "x.png",
            "a](https://evil.example/payload)",
            None,
            image_format(LinkStyle::Inline, UrlEscapeStyle::Angle, None),
        );
        assert_eq!(result, "![a\\](https://evil.example/payload)](x.png)");
    }

    #[test]
    fn should_escape_alt_text_in_reference_style_images_too() {
        // The same alt-escaping bug existed independently in the reference-style branch.
        let collector = crate::converter::reference_collector::ReferenceCollector::new();
        let handle = std::rc::Rc::new(std::cell::RefCell::new(collector));
        let result = format_image_markdown(
            "x.png",
            "a](https://evil.example/payload)",
            None,
            image_format(LinkStyle::Reference, UrlEscapeStyle::Angle, Some(&handle)),
        );
        assert_eq!(result, "![a\\](https://evil.example/payload)][1]");
    }

    #[test]
    fn should_escape_a_quote_in_the_title_that_would_open_a_real_link_after_it() {
        // ~keep audit #24 finding 4: an inert `title` containing `"` must not be able to close the
        // title early, turning the rest of the title text into document markdown (e.g. a link).
        let result = format_image_markdown(
            "a.png",
            "photo",
            Some("x\" [click](https://evil.example)"),
            image_format(LinkStyle::Inline, UrlEscapeStyle::Angle, None),
        );
        assert_eq!(result, "![photo](a.png \"x\\\" [click](https://evil.example)\")");
    }

    #[test]
    fn a_comma_inside_a_parenthesised_descriptor_does_not_start_a_candidate() {
        assert_eq!(
            pick_best_srcset_candidate("a.png (x, b.png 3x ), c.png 2x", false),
            Some("c.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png 1x (x, y.png 9x ), b.png 2x", false),
            Some("b.png")
        );
    }

    #[test]
    fn a_comma_inside_a_candidate_url_stays_in_the_url() {
        assert_eq!(
            pick_best_srcset_candidate("a.png?x=1,2 2x, b.png 1x", false),
            Some("a.png?x=1,2")
        );
        assert_eq!(
            pick_best_srcset_candidate("data:image/gif;base64,R0lG 2x, b.png 1x", false),
            Some("data:image/gif;base64,R0lG")
        );
        assert_eq!(pick_best_srcset_candidate("a.png,b.png 2x", false), Some("a.png,b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png,2x", false), Some("a.png,2x"));
    }

    #[test]
    fn commas_around_candidates_separate_them() {
        assert_eq!(
            pick_best_srcset_candidate(",,, a.png 2x, b.png 1x", false),
            Some("a.png")
        );
        assert_eq!(pick_best_srcset_candidate("a.png,, b.png 2x,,", false), Some("b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png, b.png", false), Some("a.png"));
        assert_eq!(pick_best_srcset_candidate("", false), None);
        assert_eq!(pick_best_srcset_candidate(" , ,, ", false), None);
    }

    #[test]
    fn the_largest_descriptor_of_one_kind_wins() {
        assert_eq!(pick_best_srcset_candidate("a.png", false), Some("a.png"));
        assert_eq!(pick_best_srcset_candidate("a.png, b.png 2x", false), Some("b.png"));
        assert_eq!(
            pick_best_srcset_candidate("a.png 480w, b.png 1200w", false),
            Some("b.png")
        );
        assert_eq!(pick_best_srcset_candidate("a.png 800w, b.png 2x", false), Some("a.png"));
    }

    #[test]
    fn only_the_five_ascii_whitespace_characters_separate_a_descriptor() {
        assert_eq!(
            pick_best_srcset_candidate("a.png\t1x,\nb.png\x0C2x", false),
            Some("b.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png\r\n3x ,\tb.png 2x", false),
            Some("a.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png\u{a0}3x, b.png 2x", false),
            Some("b.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png \u{a0}9x, b.png 2x", false),
            Some("b.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a\u{a0}b.png 2x", false),
            Some("a\u{a0}b.png")
        );
    }

    #[test]
    fn a_candidate_with_invalid_descriptors_is_never_chosen() {
        assert_eq!(pick_best_srcset_candidate("a.png foo", false), None);
        assert_eq!(pick_best_srcset_candidate("a.png foo, b.png", false), Some("b.png"));
        assert_eq!(
            pick_best_srcset_candidate("a.png 1x 2x, b.png 0.5x", false),
            Some("b.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png 3x 900w, b.png 1x", false),
            Some("b.png")
        );
        assert_eq!(pick_best_srcset_candidate("a.png 0w, b.png 10w", false), Some("b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png 0w", false), None);
        assert_eq!(
            pick_best_srcset_candidate("a.png -1x, b.png 0.5x", false),
            Some("b.png")
        );
        assert_eq!(pick_best_srcset_candidate("a.png 0x, b.png 0.5x", false), Some("b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png 0x", false), Some("a.png"));
    }

    #[test]
    fn descriptor_numbers_follow_the_spec_grammar() {
        assert_eq!(pick_best_srcset_candidate("a.png NaNx, b.png 2x", false), Some("b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png infx, b.png 2x", false), Some("b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png +3x, b.png 2x", false), Some("b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png 3.x, b.png 2x", false), Some("b.png"));
        assert_eq!(
            pick_best_srcset_candidate("a.png 1e400x, b.png 2x", false),
            Some("b.png")
        );
        assert_eq!(pick_best_srcset_candidate("a.png 1.5w, b.png 1w", false), Some("b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png +5w, b.png 1w", false), Some("b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png 1e1x, b.png 2x", false), Some("a.png"));
        assert_eq!(
            pick_best_srcset_candidate("a.png .5x, b.png 0.25x", false),
            Some("a.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png 2.5E-1x, b.png 0.2x", false),
            Some("a.png")
        );
    }

    #[test]
    fn widths_and_densities_are_not_compared_on_one_scale() {
        assert_eq!(
            pick_best_srcset_candidate("a.png 900x, b.png 800w", false),
            Some("b.png")
        );
        assert_eq!(pick_best_srcset_candidate("a.png, b.png 10w", false), Some("b.png"));
        assert_eq!(pick_best_srcset_candidate("a.png, b.png 0.5x", false), Some("a.png"));
        assert_eq!(
            pick_best_srcset_candidate("a.png 1x, b.png 2x, c.png 2x", false),
            Some("b.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png 900w, b.png 900w", false),
            Some("a.png")
        );
    }

    #[test]
    fn a_height_descriptor_needs_a_width() {
        assert_eq!(
            pick_best_srcset_candidate("a.png 50h 100w, b.png 90w", false),
            Some("a.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png 100w 50h, b.png 90w", false),
            Some("a.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png 2x 50h, b.png 1x", false),
            Some("b.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png 50h, b.png 0.5x", false),
            Some("b.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png 100w 50h 60h, b.png 90w", false),
            Some("b.png")
        );
        assert_eq!(
            pick_best_srcset_candidate("a.png 100w 0h, b.png 90w", false),
            Some("b.png")
        );
    }

    #[test]
    fn should_reject_out_of_order_parens_in_src_like_append_markdown_link_does() {
        // ~keep audit #24 finding 7: a naive open-count == close-count check treats ")(" as
        // balanced. Reuses `append_url_destination`, which applies the same
        // `parens_are_balanced` check already used (and tested) for `<a href>`.
        let result = format_image_markdown(
            "a)(b.png",
            "alt",
            None,
            image_format(LinkStyle::Inline, UrlEscapeStyle::Angle, None),
        );
        assert_eq!(result, "![alt](a\\)\\(b.png)");
    }
}
