//! Graphic element handler for HTML to Markdown conversion.
//!
//! Handles `<graphic>` elements including:
//! - Alternative source attributes (url, href, xlink:href, src)
//! - Fallback alt text from filename attribute
//! - Metadata collection for graphic extraction
//! - Visitor callback integration

use std::borrow::Cow;
#[cfg(feature = "metadata")]
use std::collections::BTreeMap;

#[cfg(feature = "metadata")]
use crate::converter::Context;
use crate::converter::inline::HandlerContext;
use crate::converter::inline::link::{append_url_destination, escape_markdown_title};
use crate::converter::media::first_address;
use crate::converter::utility::escaping::escape_image_alt;
use crate::converter::utility::preprocessing::sanitize_markdown_url;
use crate::options::InlineDataMedia;

#[cfg(feature = "visitor")]
use crate::converter::utility::serialization::serialize_node;

#[cfg(feature = "metadata")]
type GraphicMetadataPayload = (BTreeMap<String, String>, Option<u32>, Option<u32>);

struct GraphicData<'a> {
    src: Cow<'a, str>,
    alt: Cow<'a, str>,
    title: Option<Cow<'a, str>>,
}

/// Handle a `<graphic>` element and convert to Markdown.
///
/// This handler processes graphic elements including:
/// - Extracting source from url, href, xlink:href, or src attributes
/// - Using alt attribute, with fallback to filename
/// - Collecting metadata when the metadata feature is enabled
/// - Invoking visitor callbacks when the visitor feature is enabled
/// - Generating appropriate markdown output
pub fn handle_graphic(tag: &tl::HTMLTag, handler: HandlerContext<'_>) {
    let data = graphic_data(tag, &handler);
    #[cfg(feature = "metadata")]
    let metadata = handler.context.metadata_wants.images.then(|| graphic_metadata(tag));
    let rendered = render_graphic(tag, &data, &handler);
    if !handler.options.skip_images {
        if let Some(graphic_text) = rendered {
            handler.output.push_str(&graphic_text);
        }
    }
    #[cfg(feature = "metadata")]
    record_graphic_metadata(&data, metadata, handler.context);
}

fn graphic_data<'a>(tag: &'a tl::HTMLTag<'a>, handler: &HandlerContext<'_>) -> GraphicData<'a> {
    let addresses = ["url", "href", "xlink:href", "src"]
        .into_iter()
        .filter_map(|name| crate::converter::utility::attributes::decoded_attribute(tag, name));
    let src = first_address(handler.options.inline_data_media, addresses).map_or(Cow::Borrowed(""), |s| {
        let resolved = handler
            .context
            .resolve_url(&s, handler.node_handle, handler.parser, handler.dom_context);
        Cow::Owned(sanitize_markdown_url(resolved.as_deref().unwrap_or(&s)).into_owned())
    });

    // ~keep Use "alt" attribute, fallback to "filename"
    let alt = crate::converter::utility::attributes::decoded_attribute(tag, "alt")
        .or_else(|| crate::converter::utility::attributes::decoded_attribute(tag, "filename"))
        .unwrap_or(Cow::Borrowed(""));

    // ~keep An empty `title=""` carries no information, and `[t](u "")` / `![a](i "")` is
    // ~keep noise that no Markdown serializer round-trips: re-rendering the output drops
    // ~keep the empty title, so the second pass no longer matches the first. Treat it as
    // ~keep absent, which is what it means.
    let title = crate::converter::utility::attributes::decoded_attribute(tag, "title").filter(|v| !v.is_empty());

    GraphicData { src, alt, title }
}

#[cfg(feature = "metadata")]
fn graphic_metadata(tag: &tl::HTMLTag<'_>) -> GraphicMetadataPayload {
    let mut attributes = BTreeMap::new();
    let mut width = None;
    let mut height = None;
    for (key, value) in tag.attributes().iter() {
        let key = key.to_string();
        if matches!(key.as_str(), "url" | "href" | "xlink:href" | "src") {
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

#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
fn render_graphic(tag: &tl::HTMLTag<'_>, data: &GraphicData<'_>, handler: &HandlerContext<'_>) -> Option<String> {
    // ~keep #492: byte-identical twin of `handlers/image.rs`'s `keep_as_markdown` -- see
    // ~keep that file's comment for why `|| ctx.link_allow_inline_images` is purely additive
    // ~keep and why the heading pattern's negative clause is deliberately not mirrored here.
    // ~keep Kept in lockstep so `<graphic>` and `<img>` never disagree inside the same anchor.
    let context = handler.context;
    let keep_as_markdown = (context.in_heading && context.heading_allow_inline_images)
        || context.cell_allow_inline_images
        || context.link_allow_inline_images;

    let inline_data = context.inline_data_treatment(handler.options.inline_data_media, &data.src);
    let should_use_alt_text = inline_data == InlineDataMedia::AltTextOnly
        || (!keep_as_markdown
            && (context.convert_as_inline || (context.in_heading && !context.heading_allow_inline_images)));
    let render = || {
        (inline_data != InlineDataMedia::DropElement).then(|| {
            let rendered = format_graphic_markdown(
                &data.src,
                &data.alt,
                data.title.as_deref(),
                GraphicFormatOptions {
                    use_alt_only: should_use_alt_text,
                    link_style: handler.options.link_style,
                    url_escape_style: handler.options.url_escape_style,
                    reference_collector: context.reference_collector.as_ref(),
                },
            );
            crate::converter::utility::escaping::escape_djot_table_cell_literal(
                &rendered,
                handler.options.output_format,
                context.in_table_cell,
            )
            .into_owned()
        })
    };

    #[cfg(feature = "visitor")]
    let graphic_output = if let Some(ref visitor_handle) = context.visitor {
        use crate::visitor::{NodeContext, NodeType, VisitResult};

        let node_id = handler.node_handle.get_inner();
        let parent_tag = handler.dom_context.parent_tag_name(node_id, handler.parser);
        let index_in_parent = handler.dom_context.get_sibling_index(node_id).unwrap_or(0);

        let node_ctx = NodeContext::with_lazy_attributes(
            NodeType::Image,
            Cow::Borrowed("graphic"),
            tag,
            handler.depth,
            index_in_parent,
            parent_tag.map(Cow::Borrowed),
            true,
        );

        let visit_result = {
            let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
            visitor.visit_image(&node_ctx, &data.src, &data.alt, data.title.as_deref())
        };
        match visit_result {
            VisitResult::Continue => render(),
            VisitResult::Custom(custom) => Some(custom),
            VisitResult::Skip => None,
            VisitResult::Error(err) => {
                if context.visitor_error.borrow().is_none() {
                    *context.visitor_error.borrow_mut() = Some(err);
                }
                None
            }
            VisitResult::PreserveHtml => Some(serialize_node(handler.node_handle, handler.parser)),
        }
    } else {
        render()
    };

    #[cfg(feature = "visitor")]
    return graphic_output;
    #[cfg(not(feature = "visitor"))]
    render()
}

#[cfg(feature = "metadata")]
fn record_graphic_metadata(data: &GraphicData<'_>, metadata: Option<GraphicMetadataPayload>, context: &Context) {
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

/// Format a graphic as Markdown syntax.
///
/// If `use_alt_only` is true, returns just the alt text.
/// Otherwise returns the full `![alt](src "title")` syntax.
fn format_graphic_markdown(src: &str, alt: &str, title: Option<&str>, format: GraphicFormatOptions<'_>) -> String {
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

struct GraphicFormatOptions<'a> {
    use_alt_only: bool,
    link_style: crate::options::validation::LinkStyle,
    url_escape_style: crate::options::validation::UrlEscapeStyle,
    reference_collector: Option<&'a crate::converter::reference_collector::ReferenceCollectorHandle>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::validation::{LinkStyle, UrlEscapeStyle};

    #[test]
    fn should_escape_alt_text_that_would_close_the_graphic_and_open_a_new_link() {
        // ~keep audit #24 finding 6: `<graphic alt>` had the same unescaped-label bug as `<img alt>`
        // (finding 1) — an inert `]`/`(` in alt must not manufacture a second, live link.
        let result = format_graphic_markdown(
            "x.svg",
            "a](https://evil.example/payload)",
            None,
            GraphicFormatOptions {
                use_alt_only: false,
                link_style: LinkStyle::Inline,
                url_escape_style: UrlEscapeStyle::Angle,
                reference_collector: None,
            },
        );
        assert_eq!(result, "![a\\](https://evil.example/payload)](x.svg)");
    }

    #[test]
    fn should_escape_a_quote_in_the_title_that_would_open_a_real_link_after_it() {
        // ~keep audit #24 finding 4: `<graphic title>` had zero quote escaping.
        let result = format_graphic_markdown(
            "a.svg",
            "diagram",
            Some("x\" [click](https://evil.example)"),
            GraphicFormatOptions {
                use_alt_only: false,
                link_style: LinkStyle::Inline,
                url_escape_style: UrlEscapeStyle::Angle,
                reference_collector: None,
            },
        );
        assert_eq!(result, "![diagram](a.svg \"x\\\" [click](https://evil.example)\")");
    }

    #[test]
    fn should_reject_out_of_order_parens_in_src() {
        // ~keep audit #24 finding 6/7: `<graphic src>` previously had no paren handling at all
        // (worse than `<img src>`'s naive count check).
        let result = format_graphic_markdown(
            "a)(b.svg",
            "alt",
            None,
            GraphicFormatOptions {
                use_alt_only: false,
                link_style: LinkStyle::Inline,
                url_escape_style: UrlEscapeStyle::Angle,
                reference_collector: None,
            },
        );
        assert_eq!(result, "![alt](a\\)\\(b.svg)");
    }

    #[test]
    fn should_sanitize_a_markdown_like_src_the_same_way_img_does() {
        // ~keep audit #24 finding 6: `<graphic src>` never went through `sanitize_markdown_url`,
        // unlike `<img src>` (image.rs). Verified end-to-end since sanitization happens at
        // attribute-extraction time in `handle_graphic`, before `format_graphic_markdown`.
        let html = r#"<p><graphic url="[p](https://example.com/real)" alt="pic" /></p>"#;
        let result = crate::convert(html, None).unwrap();
        let content = result.content.unwrap_or_default();
        assert_eq!(content, "![pic](https://example.com/real)\n");
    }
}
