//! Handler for preformatted code elements (pre, code).
//!
//! Converts HTML preformatted and code tags to Markdown code blocks with support for:
//! - Language detection from `class` attributes (language-*, lang-*)
//! - Multiple code block styles (indented, backticks, tildes)
//! - Code dedentation and whitespace normalization
//! - Inline code formatting with backtick management
//! - Visitor callbacks for custom code processing

use crate::options::{CodeBlockStyle, ConversionOptions, WhitespaceMode};
#[allow(unused_imports)]
use std::collections::BTreeMap;
use tl::{NodeHandle, Parser};

// Type aliases for Context and DomContext to avoid circular imports
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handle preformatted code blocks (pre element).
pub(crate) fn handle_pre(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    use crate::converter::walk_node;

    let code_ctx = Context {
        in_code: true,
        ..ctx.clone()
    };

    let language = extract_language_from_pre(node_handle, parser);

    let mut content = String::with_capacity(256);
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let children = tag.children();
            for child_handle in children.top().iter() {
                walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    options,
                    &code_ctx,
                    depth + 1,
                    dom_ctx,
                );
            }
        }
    }

    if !content.is_empty() {
        let leading_newlines = content.chars().take_while(|&c| c == '\n').count();
        let trailing_newlines = content.chars().rev().take_while(|&c| c == '\n').count();
        let core = content.trim_matches('\n');
        let is_whitespace_only = core.trim().is_empty();

        let processed_content = if options.whitespace_mode == WhitespaceMode::Strict {
            content
        } else {
            let mut core_text = if leading_newlines > 0 {
                dedent_code_block(core)
            } else {
                core.to_string()
            };

            if is_whitespace_only {
                let mut rebuilt = String::new();
                for _ in 0..leading_newlines {
                    rebuilt.push('\n');
                }
                rebuilt.push_str(&core_text);
                for _ in 0..trailing_newlines {
                    rebuilt.push('\n');
                }
                rebuilt
            } else {
                for _ in 0..trailing_newlines {
                    core_text.push('\n');
                }
                core_text
            }
        };

        #[cfg(feature = "visitor")]
        {
            if let Some(ref visitor_handle) = ctx.visitor {
                use crate::visitor::{NodeContext, NodeType, VisitResult};

                if let Some(node) = node_handle.get(parser) {
                    if let tl::Node::Tag(tag) = node {
                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let node_id = node_handle.get_inner();
                        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::Pre,
                            tag_name: "pre".to_string(),
                            attributes,
                            depth,
                            index_in_parent,
                            parent_tag,
                            is_inline: false,
                        };

                        let mut visitor = visitor_handle.borrow_mut();
                        match visitor.visit_code_block(&node_ctx, language.as_deref(), &processed_content) {
                            VisitResult::Continue => {
                                format_code_block(output, options, ctx, &processed_content, language.as_deref());
                            }
                            VisitResult::Custom(custom) => {
                                output.push_str(&custom);
                            }
                            VisitResult::Skip => {
                                // Skip code block
                            }
                            VisitResult::PreserveHtml => {
                                format_code_block(output, options, ctx, &processed_content, language.as_deref());
                            }
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                            }
                        }
                    }
                }
            } else {
                format_code_block(output, options, ctx, &processed_content, language.as_deref());
            }
        }

        #[cfg(not(feature = "visitor"))]
        {
            format_code_block(output, options, ctx, &processed_content, language.as_deref());
        }
    }
}

/// Extract programming language from pre or nested code element's class attribute.
fn extract_language_from_pre(node_handle: &NodeHandle, parser: &Parser) -> Option<String> {
    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            // First, try to extract language from <pre> tag's class attribute
            if let Some(class_attr) = tag.attributes().get("class") {
                if let Some(class_bytes) = class_attr {
                    let class_str = class_bytes.as_utf8_str();
                    for cls in class_str.split_whitespace() {
                        if let Some(stripped) = cls.strip_prefix("language-") {
                            return Some(String::from(stripped));
                        } else if let Some(stripped) = cls.strip_prefix("lang-") {
                            return Some(String::from(stripped));
                        }
                    }
                }
            }

            // If not found on <pre>, try to extract from nested <code> tag's class attribute
            let children = tag.children();
            for child_handle in children.top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    if child_tag.name() == "code" {
                        if let Some(class_attr) = child_tag.attributes().get("class") {
                            if let Some(class_bytes) = class_attr {
                                let class_str = class_bytes.as_utf8_str();
                                for cls in class_str.split_whitespace() {
                                    if let Some(stripped) = cls.strip_prefix("language-") {
                                        return Some(String::from(stripped));
                                    } else if let Some(stripped) = cls.strip_prefix("lang-") {
                                        return Some(String::from(stripped));
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
    None
}

/// Format code block with appropriate markdown syntax.
fn format_code_block(
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    content: &str,
    language: Option<&str>,
) {
    match options.code_block_style {
        CodeBlockStyle::Indented => {
            if !ctx.convert_as_inline && !output.is_empty() && !output.ends_with("\n\n") {
                if output.ends_with('\n') {
                    output.push('\n');
                } else {
                    output.push_str("\n\n");
                }
            }

            let indented = content
                .lines()
                .map(|line| {
                    if line.is_empty() {
                        String::new()
                    } else {
                        format!("    {line}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            output.push_str(&indented);

            output.push_str("\n\n");
        }
        CodeBlockStyle::Backticks | CodeBlockStyle::Tildes => {
            if !ctx.convert_as_inline && !output.is_empty() && !output.ends_with("\n\n") {
                if output.ends_with('\n') {
                    output.push('\n');
                } else {
                    output.push_str("\n\n");
                }
            }

            let fence = if options.code_block_style == CodeBlockStyle::Backticks {
                "```"
            } else {
                "~~~"
            };

            output.push_str(fence);
            if let Some(lang) = language {
                output.push_str(lang);
            } else if !options.code_language.is_empty() {
                output.push_str(&options.code_language);
            }
            output.push('\n');
            output.push_str(content);
            output.push('\n');
            output.push_str(fence);
            output.push('\n');
        }
    }
}

/// Dedent code block by finding the minimum indentation and removing it.
fn dedent_code_block(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Find minimum indentation of non-empty lines
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    let dedented: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else if line.len() >= min_indent {
                line[min_indent..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect();

    dedented.join("\n")
}
