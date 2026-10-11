//! SVG and `MathML` element handling with serialization and base64 encoding.

use crate::converter::main_helpers::effective_max_depth;
use crate::converter::media::MediaContext;
use crate::converter::utility::content::normalized_tag_name;
use crate::converter::utility::escaping::escape_link_label;
use crate::converter::utility::serialization::escape_html_attribute_value;
use crate::converter::utility::svg_attrs::canonical_svg_attr;
use crate::options::conversion::NATIVE_STACK_SAFE_DEPTH;
// ~keep reason: BTreeMap is only used when the inline-images feature is active.
#[allow(unused_imports)]
use std::collections::BTreeMap;
use tl::{NodeHandle, Parser};

#[cfg(feature = "inline-images")]
use crate::inline_images::{InlineImageBuild, InlineImageCollector, InlineImageFormat, InlineImageSource};

#[cfg(feature = "inline-images")]
type InlineCollectorHandle = std::rc::Rc<std::cell::RefCell<InlineImageCollector>>;

/// Handle inline SVG elements with size limits and base64 encoding.
///
/// # Features
/// - SVG serialization to HTML string
/// - Size validation with configurable limits
/// - Base64 encoding for data URI
/// - Metadata extraction (aria-label, title, dimensions)
#[cfg(feature = "inline-images")]
#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::option_if_let_else)]
pub fn handle_inline_svg(
    collector_ref: &InlineCollectorHandle,
    node_handle: &NodeHandle,
    parser: &Parser,
    title_opt: Option<String>,
    attributes: BTreeMap<String, String>,
) {
    let max_size = {
        let borrow = collector_ref.borrow();
        if !borrow.capture_svg() {
            return;
        }
        borrow.max_decoded_size()
    };

    if max_size == 0 {
        let mut collector = collector_ref.borrow_mut();
        let index = collector.next_index();
        collector.warn_skip(index, "max SVG payload size is zero");
        return;
    }

    let mut collector = collector_ref.borrow_mut();
    let index = collector.next_index();

    let serialized = serialize_element(node_handle, parser);
    if serialized.is_empty() {
        collector.warn_skip(index, "unable to serialize SVG element");
        return;
    }

    let data = serialized.into_bytes();
    if data.len() as u64 > max_size {
        collector.warn_skip(
            index,
            format!(
                "serialized SVG payload ({} bytes) exceeds configured max ({})",
                data.len(),
                max_size
            ),
        );
        return;
    }

    let description = attributes
        .get("aria-label")
        .and_then(|value| non_empty_trimmed(value))
        .or_else(|| title_opt.as_deref().and_then(non_empty_trimmed));

    let filename_candidate = attributes
        .get("data-filename")
        .cloned()
        .or_else(|| attributes.get("filename").cloned())
        .or_else(|| attributes.get("data-name").cloned());

    let image = collector.build_image(InlineImageBuild {
        data,
        format: InlineImageFormat::Svg,
        filename: filename_candidate,
        description,
        dimensions: None,
        source: InlineImageSource::SvgElement,
        attributes,
    });

    collector.push_image(index, image);
}

/// Serialize an element to HTML string (for SVG and Math elements).
///
/// Attributes are sorted by name to guarantee deterministic output across
/// process invocations (the underlying parser stores them in a `HashMap`).
///
/// Depth-guarded by [`NATIVE_STACK_SAFE_DEPTH`]. Callers that already track the
/// element's depth in the wider DOM walk (`handle_svg`, `handle_math`) should call
/// [`serialize_element_at_depth`] instead, so the same recursion budget the caller is
/// already spending against `effective_max_depth` is honored here too.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_element(node_handle: &NodeHandle, parser: &Parser) -> String {
    serialize_element_at_depth(node_handle, parser, 0, NATIVE_STACK_SAFE_DEPTH)
}

/// Serialize a node to HTML string.
///
/// See [`serialize_element`] for the depth-guard contract.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_node(node_handle: &NodeHandle, parser: &Parser) -> String {
    serialize_node_at_depth(node_handle, parser, 0, NATIVE_STACK_SAFE_DEPTH)
}

/// Serialize an element to HTML string, stopping descent once `depth` reaches `max_depth`.
///
/// Mutually recursive with [`serialize_node_at_depth`] over `tag.children()`. Follows the
/// same convention as the main DOM walker (`walk_node` in `converter/main.rs`): the depth
/// counter increments on every descent into a child, and reaching the limit stops further
/// descent rather than erroring, so a pathologically nested `<svg>`/`<math>` subtree cannot
/// overflow the stack (audit #23). The element's own opening tag and attributes are always
/// emitted; only its descendants are dropped once the budget is exhausted.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_element_at_depth(node_handle: &NodeHandle, parser: &Parser, depth: usize, max_depth: usize) -> String {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let tag_name = normalized_tag_name(tag.name().as_utf8_str());
        let mut html = String::with_capacity(256);
        html.push('<');
        html.push_str(&tag_name);

        let mut attrs: Vec<_> = tag.attributes().iter().collect();
        attrs.sort_by(|(a, _), (b, _)| a.as_ref().cmp(b.as_ref()));
        for (key, value_opt) in attrs {
            html.push(' ');
            // ~keep Restore camelCase for SVG/MathML attributes whose canonical
            // ~keep WHATWG spelling is mixed-case.  tl lowercases all attribute
            // ~keep names when it re-parses a wrapped fragment (Tier-1 path via
            // ~keep emit_svg_from_slice), but preserves case on a full-document
            // ~keep parse (Tier-2 path).  Applying the lookup in both paths is
            // ~keep safe: Tier-2 already has the correct spelling so the lookup
            // ~keep returns None and the original key is used unchanged.
            let key_str = key.as_ref();
            let canonical = canonical_svg_attr(key_str);
            html.push_str(canonical.unwrap_or(key_str));
            if let Some(value) = value_opt {
                // ~keep Treat empty value identically to a bare attribute.  When tl
                // ~keep re-parses a wrapped SVG slice (Tier-1's emit_svg_from_slice)
                // ~keep it yields `None` for `attr=""` while a single full-document
                // ~keep parse (Tier-2) yields `Some("")`.  Both forms are
                // ~keep HTML5-equivalent; normalise here so both tiers produce
                // ~keep byte-identical output.
                if !value.is_empty() {
                    html.push_str("=\"");
                    html.push_str(&escape_html_attribute_value(&value));
                    html.push('"');
                }
            }
        }

        let has_children = !tag.children().top().is_empty();
        if has_children {
            html.push('>');
            if depth >= max_depth {
                tracing::warn!(
                    target: "html_to_markdown::convert",
                    max_depth,
                    tag = %tag_name,
                    "SVG/MathML serialization reached the effective depth limit; descendants were skipped"
                );
            } else {
                let children = tag.children();
                for child_handle in children.top().iter() {
                    html.push_str(&serialize_node_at_depth(child_handle, parser, depth + 1, max_depth));
                }
            }
            html.push_str("</");
            html.push_str(&tag_name);
            html.push('>');
        } else {
            html.push_str(" />");
        }
        return html;
    }
    String::new()
}

/// Serialize a node to HTML string, stopping descent once `depth` reaches `max_depth`.
///
/// See [`serialize_element_at_depth`] for the depth-guard contract.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_node_at_depth(node_handle: &NodeHandle, parser: &Parser, depth: usize, max_depth: usize) -> String {
    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Raw(bytes) => bytes.as_utf8_str().to_string(),
            tl::Node::Tag(_) => serialize_element_at_depth(node_handle, parser, depth, max_depth),
            _ => String::new(),
        }
    } else {
        String::new()
    }
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

/// Where the walk of [`graphic_text`] is, which decides what a child gives.
#[derive(Clone, Copy)]
enum GraphicScope {
    /// Among drawn elements. `of_svg`: the parent is an `svg` element, whose `title` and `desc`
    /// name the graphic. `named`: no `aria-hidden="true"` hides that name.
    Drawing { of_svg: bool, named: bool },
    /// Inside a `title` or a `desc`: every text node.
    Label,
    /// Inside a `text` element: every text node but the tooltip of a part.
    Text,
    /// Inside a `foreignObject`: HTML.
    Html,
}

enum GraphicStep {
    Visit(tl::NodeHandle, GraphicScope),
    Space,
    EndPart,
}

/// Collects the parts of [`graphic_text`], each with its white space collapsed.
#[derive(Default)]
struct GraphicParts {
    label: Option<String>,
    parts: Vec<String>,
    current: String,
}

impl GraphicParts {
    fn end_part(&mut self) {
        let part = self.current.split_whitespace().collect::<Vec<_>>().join(" ");
        self.current.clear();
        // ~keep A title that repeats the `aria-label` is one name, not two.
        if !part.is_empty() && self.label.as_deref() != Some(part.as_str()) {
            self.parts.push(part);
        }
    }
}

fn is_aria_hidden(tag: &tl::HTMLTag<'_>) -> bool {
    tag.attributes()
        .get("aria-hidden")
        .flatten()
        .is_some_and(|value| value.as_utf8_str().trim().eq_ignore_ascii_case("true"))
}

/// Whether the `display` or `visibility` attribute of an element of a graphic hides it.
///
/// ~keep The two attributes are the SVG spelling of the style declarations with the same names.
/// ~keep The function that judges a `style` attribute judges them, so text in a graphic and text
/// ~keep outside one are hidden by one decision. An element hidden by its `style` or by the
/// ~keep `hidden` attribute is already gone when the graphic is parsed.
fn is_hidden_in_graphic(tag: &tl::HTMLTag<'_>) -> bool {
    use crate::converter::utility::preprocessing::{HiddenStyleReason, style_value_hidden_reason};

    ["display", "visibility"].iter().any(|property| {
        tag.attributes().get(*property).flatten().is_some_and(|value| {
            let declaration = format!("{property}:{}", value.as_utf8_str());
            style_value_hidden_reason(&declaration) == Some(HiddenStyleReason::Definitive)
        })
    })
}

/// Whether the `systemLanguage` test of a child of a `switch` holds for a reader of English.
fn system_language_holds(tag: &tl::HTMLTag<'_>) -> bool {
    // ~keep The parser keeps the case of the name in a page and lowers it in a fragment.
    let Some(value) = ["systemLanguage", "systemlanguage"]
        .iter()
        .find_map(|key| tag.attributes().get(*key))
    else {
        return true;
    };
    value.is_some_and(|value| {
        value.as_utf8_str().split(',').any(|language| {
            let language = language.trim();
            language.eq_ignore_ascii_case("en")
                || language
                    .get(..3)
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case("en-"))
        })
    })
}

/// The text of an inline `<svg>` for a reader who does not get the picture, on one line.
///
/// In reading order: the `aria-label` of the graphic, then the `title` and `desc` of each `svg`
/// element, each `text` element and each `foreignObject`, joined by one space. Only elements that
/// draw their children are entered (`svg`, `g`, `a`, the child of `switch` that a reader of
/// English gets), so `defs`, `symbol`, `style`, `script` and `metadata` give nothing, and a `use`
/// reference is not followed. An element with `display="none"` or `visibility="hidden"` gives
/// nothing. `aria-hidden="true"` removes the label, the title and the description, and keeps
/// drawn text. A graphic with none of these gives the empty string.
pub fn graphic_text(svg: &tl::HTMLTag<'_>, parser: &Parser<'_>) -> String {
    if is_hidden_in_graphic(svg) {
        return String::new();
    }
    let named = !is_aria_hidden(svg);
    let mut out = GraphicParts::default();
    if named {
        if let Some(label) = crate::converter::utility::attributes::decoded_attribute(svg, "aria-label") {
            out.current.push_str(&label);
            out.end_part();
            out.label = out.parts.first().cloned();
        }
    }
    // ~keep An explicit stack: the nesting is the page's, and native recursion over it can overflow.
    let mut stack = Vec::new();
    push_graphic_children(&mut stack, svg, GraphicScope::Drawing { of_svg: true, named });
    while let Some(step) = stack.pop() {
        let (handle, scope) = match step {
            GraphicStep::Visit(handle, scope) => (handle, scope),
            GraphicStep::Space => {
                out.current.push(' ');
                continue;
            }
            GraphicStep::EndPart => {
                out.end_part();
                continue;
            }
        };
        match handle.get(parser) {
            Some(tl::Node::Raw(bytes)) if !matches!(scope, GraphicScope::Drawing { .. }) => {
                out.current
                    .push_str(&crate::text::decode_html_entities_cow(bytes.as_utf8_str().as_ref()));
            }
            Some(tl::Node::Tag(tag)) => visit_graphic_tag(tag, parser, scope, &mut stack, &mut out),
            _ => {}
        }
    }
    out.end_part();
    out.parts.join(" ")
}

fn push_graphic_children(stack: &mut Vec<GraphicStep>, tag: &tl::HTMLTag<'_>, scope: GraphicScope) {
    stack.extend(
        tag.children()
            .top()
            .as_slice()
            .iter()
            .rev()
            .map(|child| GraphicStep::Visit(*child, scope)),
    );
}

/// Schedules the children of `tag` as one part of the text.
fn push_graphic_part(stack: &mut Vec<GraphicStep>, out: &mut GraphicParts, tag: &tl::HTMLTag<'_>, scope: GraphicScope) {
    out.end_part();
    stack.push(GraphicStep::EndPart);
    push_graphic_children(stack, tag, scope);
}

fn visit_graphic_tag(
    tag: &tl::HTMLTag<'_>,
    parser: &Parser<'_>,
    scope: GraphicScope,
    stack: &mut Vec<GraphicStep>,
    out: &mut GraphicParts,
) {
    if is_hidden_in_graphic(tag) {
        return;
    }
    let name = normalized_tag_name(tag.name().as_utf8_str());
    let nested_svg = |named: bool| GraphicScope::Drawing {
        of_svg: true,
        named: named && !is_aria_hidden(tag),
    };
    match (scope, &*name) {
        (GraphicScope::Drawing { of_svg, named }, "title" | "desc") => {
            if of_svg && named {
                push_graphic_part(stack, out, tag, GraphicScope::Label);
            }
        }
        (GraphicScope::Drawing { .. }, "text") => push_graphic_part(stack, out, tag, GraphicScope::Text),
        (GraphicScope::Drawing { .. }, "foreignobject") => push_graphic_part(stack, out, tag, GraphicScope::Html),
        (GraphicScope::Drawing { named, .. }, "svg") => push_graphic_children(stack, tag, nested_svg(named)),
        (GraphicScope::Drawing { named, .. }, "g" | "a") => {
            push_graphic_children(stack, tag, GraphicScope::Drawing { of_svg: false, named });
        }
        (GraphicScope::Drawing { named, .. }, "switch") => {
            // ~keep A `switch` draws one child: the first element whose test attributes hold. The
            // ~keep converter knows no language of the reader, so a `systemLanguage` holds only
            // ~keep when it names English, and a child without the attribute is the fallback.
            let first = tag
                .children()
                .top()
                .iter()
                .find(|child| matches!(child.get(parser), Some(tl::Node::Tag(child_tag)) if system_language_holds(child_tag)))
                .copied();
            stack.extend(first.map(|child| GraphicStep::Visit(child, GraphicScope::Drawing { of_svg: false, named })));
        }
        (GraphicScope::Drawing { .. }, _) => {}
        (GraphicScope::Label, _) => push_graphic_children(stack, tag, GraphicScope::Label),
        (GraphicScope::Text, "title" | "desc") | (GraphicScope::Html, "script" | "style" | "template") => {}
        (GraphicScope::Text, _) => {
            push_graphic_children(stack, tag, GraphicScope::Text);
            // ~keep A span with its own `x` or `y` starts at a new place, as a new line of a label does.
            if ["x", "y"].iter().any(|key| tag.attributes().get(*key).is_some()) {
                stack.push(GraphicStep::Space);
            }
        }
        (GraphicScope::Html, "svg") => {
            out.end_part();
            stack.push(GraphicStep::EndPart);
            push_graphic_children(stack, tag, nested_svg(true));
        }
        (GraphicScope::Html, "br") => out.current.push(' '),
        (GraphicScope::Html, html_name) => {
            let is_block = crate::converter::utility::content::is_block_level_element(html_name);
            if is_block {
                stack.push(GraphicStep::Space);
            }
            push_graphic_children(stack, tag, GraphicScope::Html);
            if is_block {
                stack.push(GraphicStep::Space);
            }
        }
    }
}

/// Handle SVG element conversion to Markdown.
///
/// Handles inline image collection, and writes either the text of the graphic (in inline mode
/// and for `alt_text_only`) or a base64-encoded image with that text as its alt text.
pub fn handle_svg(
    node_handle: &NodeHandle,
    tag: &tl::HTMLTag,
    parser: &Parser,
    output: &mut String,
    context: MediaContext<'_>,
) {
    let MediaContext { options, ctx, .. } = context;

    #[cfg(feature = "inline-images")]
    if let Some(ref collector_ref) = ctx.inline_collector {
        collect_inline_svg(collector_ref, *node_handle, tag, parser, context.dom_ctx);
    }

    // ~keep The converter writes an inline SVG as a `data:` URL it builds itself, so the
    // ~keep inline-data choice always applies.
    if options.skip_images {
        return;
    }
    let inline_data = super::inline_data_treatment(options.inline_data_media, "data:image/svg+xml");
    if inline_data == crate::options::InlineDataMedia::DropElement {
        // ~keep Only this choice removes the element. A link around a graphic that is written as
        // ~keep its text keeps its address also when that text is empty, so the flag stays unset.
        ctx.inline_data_replaced.set(true);
        return;
    }

    let title = graphic_text(tag, parser);
    // ~keep Code shows every character as text, so a graphic in code writes its text and no marks.
    if ctx.in_code {
        output.push_str(&title);
    } else if ctx.convert_as_inline || inline_data == crate::options::InlineDataMedia::AltTextOnly {
        write_graphic_as_text(*node_handle, parser, output, context, &title);
    } else {
        write_graphic_as_image(*node_handle, parser, output, context, &title);
    }
}

/// Gives an inline `<svg>` to the collector of inline images, with the text of its `title` child
/// and the attributes that describe it.
#[cfg(feature = "inline-images")]
fn collect_inline_svg(
    collector_ref: &InlineCollectorHandle,
    node_handle: NodeHandle,
    tag: &tl::HTMLTag,
    parser: &Parser,
    dom_ctx: &crate::converter::DomContext,
) {
    let title_opt = tag
        .children()
        .top()
        .iter()
        .find(|child| {
            matches!(child.get(parser), Some(tl::Node::Tag(child_tag)) if child_tag.name().as_utf8_str().eq_ignore_ascii_case("title"))
        })
        .map(|child| {
            crate::converter::utility::content::get_text_content(child, parser, dom_ctx)
                .trim()
                .to_string()
        });
    let mut attributes_map = BTreeMap::new();
    for (key, value_opt) in tag.attributes().iter() {
        let key_str = key.to_string();
        let keep = key_str == "width"
            || key_str == "height"
            || key_str == "filename"
            || key_str == "aria-label"
            || key_str.starts_with("data-");
        if keep {
            let value = value_opt.map(|value| value.to_string()).unwrap_or_default();
            attributes_map.insert(key_str, value);
        }
    }
    handle_inline_svg(collector_ref, &node_handle, parser, title_opt, attributes_map);
}

/// Writes the text of a graphic as running text.
fn write_graphic_as_text(
    node_handle: NodeHandle,
    parser: &Parser,
    output: &mut String,
    context: MediaContext<'_>,
    title: &str,
) {
    let MediaContext {
        options, ctx, dom_ctx, ..
    } = context;
    // ~keep Written as running text through the function a text node uses, so it gets the same
    // ~keep escaping, also where it starts a line.
    let escaped = crate::text::escape(
        title,
        options.escape_misc,
        options.escape_asterisks,
        options.escape_underscores,
        options.escape_ascii,
    );
    if !escaped.is_empty() {
        crate::converter::text_node::push_running_text(
            output,
            &escaped,
            crate::converter::text_node::TextSite {
                node_handle: &node_handle,
                parser,
                options,
                ctx,
                dom_ctx,
            },
        );
    }
}

/// Writes a graphic as an image with a base64 `data:` URL, with the text of the graphic as its
/// alt text.
fn write_graphic_as_image(
    node_handle: NodeHandle,
    parser: &Parser,
    output: &mut String,
    context: MediaContext<'_>,
    title: &str,
) {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    let MediaContext { options, depth, .. } = context;
    let svg_html = serialize_element_at_depth(&node_handle, parser, depth, effective_max_depth(options));
    let base64_svg = STANDARD.encode(svg_html.as_bytes());

    output.push_str("![");
    output.push_str(&escape_link_label(title));
    output.push_str("](data:image/svg+xml;base64,");
    output.push_str(&base64_svg);
    output.push(')');
}

/// Handle `MathML` element conversion to Markdown.
///
/// Serializes `MathML` to HTML comment and outputs text content with escaping.
pub fn handle_math(
    node_handle: &NodeHandle,
    tag: &tl::HTMLTag,
    parser: &Parser,
    output: &mut String,
    context: MediaContext<'_>,
) {
    let MediaContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = context;
    use crate::converter::utility::content::get_text_content;
    use crate::text;

    let text_content = get_text_content(node_handle, parser, dom_ctx).trim().to_string();

    if text_content.is_empty() {
        return;
    }

    let math_html = serialize_element_at_depth(node_handle, parser, depth, effective_max_depth(options));

    let escaped_text = text::escape(
        &text_content,
        options.escape_misc,
        options.escape_asterisks,
        options.escape_underscores,
        options.escape_ascii,
    );

    let is_display_block = tag
        .attributes()
        .get("display")
        .flatten()
        .is_some_and(|v| v.as_utf8_str() == "block");

    if is_display_block && !ctx.in_paragraph && !ctx.convert_as_inline {
        output.push_str("\n\n");
    }

    output.push_str("<!-- MathML: ");
    output.push_str(&math_html);
    output.push_str(" --> ");
    output.push_str(&escaped_text);

    if is_display_block && !ctx.in_paragraph && !ctx.convert_as_inline {
        output.push_str("\n\n");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_escape_svg_title_caption_that_would_open_a_new_image_link() {
        // ~keep audit #24 finding 5: the `<svg><title>` used as the data-URI image caption was
        // pushed unescaped, so an inert `]`/`(` in it could manufacture a second, live image
        // pointing at an attacker-controlled URL — same mechanism as finding 1 (`<img alt>`).
        let html = "<svg><title>a](https://evil.example/payload)</title></svg>";
        let result = crate::convert(html, None).unwrap();
        let content = result.content.unwrap_or_default();
        assert_eq!(
            content,
            "![a\\](https://evil.example/payload)](data:image/svg+xml;base64,\
             PHN2Zz48dGl0bGU+YV0oaHR0cHM6Ly9ldmlsLmV4YW1wbGUvcGF5bG9hZCk8L3RpdGxlPjwvc3ZnPg==)\n"
        );
    }
}

#[cfg(test)]
mod attribute_escaping_tests {
    use super::serialize_element;

    #[test]
    fn should_escape_a_quote_inside_a_reconstructed_attribute_value() {
        // ~keep audit #24 finding 3 (media/svg.rs duplicate): the source HTML's `"` is valid,
        // inert content inside a single-quoted attribute. Reconstructing it into a
        // double-quoted attribute without escaping manufactures a real `onclick` that never
        // existed as an attribute in the original document.
        let html = r#"<foo title='x" onclick="alert(1)" y=' data-safe="1">"#;
        let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let node_handle = dom
            .children()
            .iter()
            .find(|handle| matches!(handle.get(parser), Some(tl::Node::Tag(_))))
            .expect("tag node");
        let result = serialize_element(node_handle, parser);
        assert_eq!(
            result,
            "<foo data-safe=\"1\" title=\"x&quot; onclick=&quot;alert(1)&quot; y=\" />"
        );
    }
}
