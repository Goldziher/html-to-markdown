//! Handlers for HTML5 sectioning elements.
//!
//! Processes semantic sectioning elements:
//! - `<article>` - Independent, self-contained content
//! - `<section>` - Generic grouping of thematic content
//! - `<nav>` - Navigation links (typically rendered inline or in sidebars)
//! - `<aside>` - Peripheral content (sidebars, callouts)
//! - `<header>` - Introductory content (page headers)
//! - `<footer>` - End content (page footers)
//! - `<main>` - Primary content area
//!
//! All these elements are treated as block-level containers.
//! Their content is extracted and formatted with proper spacing.

/// Handles sectioning elements (article, section, nav, aside, header, footer, main).
///
/// Sectioning elements are rendered as block-level containers. When in inline
/// conversion mode, their content is rendered inline without block spacing.
/// Otherwise, content is wrapped with proper blank lines to separate from other blocks.
///
/// # Behavior
///
/// - **Inline mode**: The dispatcher gives the children to the walk; this handler does not run
/// - **Block mode**: Content is collected, trimmed, and formatted with blank lines
/// - **Empty content**: Empty sections are skipped entirely
///
/// # Implementation Note
///
/// Sectioning elements act as transparent containers—their presence doesn't
/// add any special formatting beyond structural grouping.
pub fn handle(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    let mut children = Vec::new();
    let Some(opened) = open(node_handle, parser, output, handler, &mut children) else {
        return;
    };
    let mut content = String::new();
    let child_ctx = opened.scope.as_deref().unwrap_or(handler.ctx);
    for child in &children {
        crate::converter::walk_node(
            child,
            parser,
            &mut content,
            super::HandlerContext::new(handler.options, child_ctx, handler.depth + 1, handler.dom_ctx),
        );
    }
    close(&content, output, handler);
}

pub fn open(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &str,
    handler: super::HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<crate::converter::walk::Open> {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    let section_ctx = crate::converter::list::utils::nested_block_context(output, handler.ctx, handler.options);
    Some(crate::converter::walk::Open {
        scope: Some(std::rc::Rc::new(section_ctx)),
        close: crate::converter::walk::Close::Capture(crate::converter::walk::CaptureEnd::Section),
    })
}

pub fn close(content: &str, output: &mut String, handler: super::HandlerContext<'_>) {
    let mut content = content.to_owned();
    // ~keep A trailing <br> run with no following sibling has no next dispatch to catch
    // ~keep it in `walk_node`'s pre-block-dispatch strip, since this sectioning element's
    // ~keep content is simply finished here — so this closes its own trailing run the
    // ~keep same way `paragraph.rs` closes its own (issue #464 follow-up). `content` is
    // ~keep pushed to `output` raw below (not `.trim()`-ed), so the strip has to run here.
    crate::converter::main_helpers::strip_trailing_backslash_breaks_from_fresh_buffer(
        &mut content,
        handler.options.newline_style,
    );

    if content.trim().is_empty() {
        return;
    }
    let content = without_leading_space(&content, handler.options, handler.ctx);

    // ~keep Inside a list item the section starts at the item's content column (issue #583).
    if handler.ctx.in_list_item && !handler.ctx.in_table_cell && !output.is_empty() {
        crate::converter::list::utils::start_block_in_list_item(output, handler.ctx, handler.options);
    } else if !output.is_empty() && !output.ends_with("\n\n") {
        if !output.ends_with('\n') {
            output.push('\n');
        }
        output.push('\n');
    }

    crate::converter::block::horizontal_rule::separate_leading_rule(output, content, handler.ctx);
    output.push_str(content);

    if content.ends_with('\n') && !content.ends_with("\n\n") {
        output.push('\n');
    } else if !content.ends_with('\n') {
        output.push_str("\n\n");
    }
}

/// The content of a sectioning element without the one space it can start with.
///
/// ~keep White space at the start of a block is no space. The children are written into a
/// ~keep buffer of their own, so nothing before them shows that the block starts there: text
/// ~keep and inline elements write the space that they start with. Two spaces or more are an
/// ~keep indent (a code block), and the strict mode and code keep all white space.
fn without_leading_space<'content>(
    content: &'content str,
    options: &crate::options::ConversionOptions,
    ctx: &crate::converter::context::Context,
) -> &'content str {
    if crate::converter::utility::content::keeps_source_white_space(options, ctx) {
        return content;
    }
    match content.strip_prefix(' ') {
        Some(rest) if !rest.starts_with(' ') => rest,
        _ => content,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_render_a_section_through_the_synchronous_handler_bridge() {
        let html = "<section>x</section>";
        let dom = tl::parse(html, tl::ParserOptions::default()).expect("section parses");
        let parser = dom.parser();
        let dom_ctx = crate::converter::utility::caching::build_dom_context(&dom, parser, html.len());
        let options = crate::options::ConversionOptions::default();
        let context = crate::converter::Context::default();
        let mut output = String::new();
        super::handle(
            "section",
            dom.children().first().expect("section exists"),
            parser,
            &mut output,
            crate::converter::block::container::HandlerContext::new(&options, &context, 0, &dom_ctx),
        );
        assert_eq!(output, "x\n\n");
    }
}
