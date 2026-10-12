//! Handler for metadata and script elements (head, script, style, math).
//!
//! Converts various metadata-related elements:
//! - **head**: Document metadata container; processes script[type="application/ld+json"]
//! - **script**: Script elements; extracts JSON-LD structured data when appropriate
//! - **style**: CSS stylesheet elements; skipped in conversion
//! - **math**: Visible text of `MathML` elements

use crate::converter::block::container::HandlerContext;
#[cfg(feature = "metadata")]
use crate::converter::utility::preprocessing::{KeptScripts, ScriptType, script_type};
use crate::options::ConversionOptions;
use crate::text::escape;
use tl::{NodeHandle, Parser};

type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handles metadata elements: head, script, style, math.
///
/// Processes various metadata-related elements:
/// - head: Scans for structured data in script[type="application/ld+json"]
/// - script: Extracts JSON-LD for structured data collection
/// - style: Skipped (CSS not relevant in markdown)
/// - math: Outputs visible `MathML` text
pub fn handle(
    tag_name: &str,
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    match tag_name {
        "head" => {
            handle_head(node_handle, parser, output, handler);
        }
        "script" => {
            handle_script(node_handle, parser, output, handler.ctx);
        }
        "style" => {}
        "math" => {
            handle_math(
                node_handle,
                parser,
                output,
                handler.options,
                handler.ctx,
                handler.dom_ctx,
            );
        }
        _ => {}
    }
}

/// True for the name (lower case) of a child of `head` that makes the converter write the
/// content of that `head`: a head with no end tag holds the body of the page.
pub fn is_body_content_in_head(name: &str) -> bool {
    matches!(name, "body" | "main" | "article" | "section" | "div" | "p")
}

/// Handle head element.
///
/// Head elements contain metadata. We process them to extract structured data from
/// nested script[type="application/ld+json"] elements if metadata collection is enabled.
fn handle_head(node_handle: &NodeHandle, parser: &Parser, output: &mut String, handler: HandlerContext<'_>) {
    use crate::converter::walk_node;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    let children = tag.children();
    let has_body_like = children.top().iter().any(|child_handle| {
        if let Some(child_name) = handler.dom_ctx.tag_name_for(*child_handle, parser) {
            is_body_content_in_head(child_name.as_ref())
        } else {
            false
        }
    });

    #[cfg(feature = "metadata")]
    if handler.ctx.metadata_wants.structured_data {
        collect_json_ld_from_children(&children, parser, handler);
    }

    if has_body_like {
        for child_handle in children.top().iter() {
            walk_node(
                child_handle,
                parser,
                output,
                crate::converter::block::container::HandlerContext::new(
                    handler.options,
                    handler.ctx,
                    handler.depth + 1,
                    handler.dom_ctx,
                ),
            );
        }
    }
}

#[cfg(feature = "metadata")]
fn collect_json_ld_from_children(children: &tl::Children, parser: &Parser, handler: HandlerContext<'_>) {
    let Some(collector) = handler.ctx.metadata_collector.as_ref() else {
        return;
    };
    for child_handle in children.top().iter() {
        let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) else {
            continue;
        };
        let child_name = handler
            .dom_ctx
            .tag_name_for(*child_handle, parser)
            .unwrap_or_else(|| crate::converter::normalized_tag_name(child_tag.name().as_utf8_str()));
        if child_name.as_ref() != "script" {
            continue;
        }
        if let Some(json) = json_ld_from_tag(child_handle.get_inner(), child_tag, parser, &handler.ctx.kept_scripts) {
            collector.borrow_mut().add_json_ld(json);
        }
    }
}

/// The structured data of a script, as the page has it. `node` is the id of the script.
///
/// ~keep The tree holds the text of a kept script with the escape of the first pass.
/// ~keep `KeptScripts` takes it back; this function decodes nothing, so a character reference
/// ~keep of the JSON stays as written.
#[cfg(feature = "metadata")]
fn json_ld_from_tag(node: u32, tag: &tl::HTMLTag<'_>, parser: &Parser, kept_scripts: &KeptScripts) -> Option<String> {
    let type_value = crate::converter::utility::attributes::decoded_attribute(tag, "type")?;
    if script_type(&type_value) != ScriptType::StructuredData {
        return None;
    }
    let text = tag.inner_text(parser);
    let json = kept_scripts.as_written(node, text.trim()).into_owned();
    (!json.is_empty()).then_some(json)
}

/// The kept scripts of the page with the record of each script of the final tree. `dom` is the
/// tree that the converter reads, after the last parse.
///
/// ~keep The first pass records a script outside `svg` only, it keeps a script by the rule of
/// ~keep `script_type`, and it writes the body as one text, so this function takes the same
/// ~keep scripts: a script with an element in it is not one that the first pass wrote. The
/// ~keep read is in the order of the page and goes to each node once, with no recursion.
#[cfg(feature = "metadata")]
#[must_use]
pub fn kept_scripts_of_tree(dom: &tl::VDom<'_>, mut kept_scripts: KeptScripts) -> KeptScripts {
    if kept_scripts.is_empty() {
        return kept_scripts;
    }
    let parser = dom.parser();
    let mut in_tree = Vec::new();
    // ~keep The nodes to read, the next one last, each with whether it is in an `svg`.
    let mut open: Vec<(NodeHandle, bool)> = dom.children().iter().rev().map(|child| (*child, false)).collect();
    while let Some((handle, in_svg)) = open.pop() {
        let Some(tl::Node::Tag(tag)) = handle.get(parser) else {
            continue;
        };
        let name = tag.name().as_bytes();
        let children = tag.children();
        let children = children.top().as_slice();
        if !in_svg
            && name.eq_ignore_ascii_case(b"script")
            && let [only] = children
            && let Some(tl::Node::Raw(text)) = only.get(parser)
        {
            let text = text.as_utf8_str();
            let text = text.trim();
            if text.contains("&lt;") && is_kept_script(tag) {
                in_tree.push((handle.get_inner(), text.to_string()));
            }
        }
        let in_svg = in_svg || name.eq_ignore_ascii_case(b"svg");
        open.extend(children.iter().rev().map(|child| (*child, in_svg)));
    }
    kept_scripts.match_to_tree(&in_tree);
    kept_scripts
}

/// Whether the first pass keeps a script with the start tag `tag`.
#[cfg(feature = "metadata")]
fn is_kept_script(tag: &tl::HTMLTag<'_>) -> bool {
    crate::converter::utility::attributes::decoded_attribute(tag, "type")
        .is_some_and(|type_value| script_type(&type_value) != ScriptType::Removed)
}

/// Handle script element.
///
/// Script elements are processed to extract JSON-LD structured data when
/// the type is "application/ld+json" and metadata collection is enabled.
#[cfg_attr(not(feature = "metadata"), allow(unused_variables))]
fn handle_script(node_handle: &NodeHandle, parser: &Parser, _output: &mut String, ctx: &Context) {
    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    #[cfg(feature = "metadata")]
    if ctx.metadata_wants.structured_data
        && let Some(collector) = ctx.metadata_collector.as_ref()
        && let Some(json) = json_ld_from_tag(node_handle.get_inner(), tag, parser, &ctx.kept_scripts)
    {
        collector.borrow_mut().add_json_ld(json);
    }
}

/// Handle math element.
///
/// `MathML` elements contribute their visible text.
/// The text content of the element is also output as plain text.
fn handle_math(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    dom_ctx: &DomContext,
) {
    let text_content = crate::converter::get_text_content(node_handle, parser, dom_ctx)
        .trim()
        .to_string();

    if text_content.is_empty() {
        return;
    }

    let escaped_text = escape(
        &text_content,
        options.escape_misc,
        options.escape_asterisks,
        options.escape_underscores,
        options.escape_ascii,
    );

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    let is_display_block = tag
        .attributes()
        .get("display")
        .flatten()
        .is_some_and(|v| v.as_utf8_str() == "block");

    if is_display_block && !ctx.in_paragraph && !ctx.convert_as_inline {
        output.push_str("\n\n");
    }

    output.push_str(&escaped_text);

    if is_display_block && !ctx.in_paragraph && !ctx.convert_as_inline {
        output.push_str("\n\n");
    }
}
