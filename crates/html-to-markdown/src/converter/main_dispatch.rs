use crate::converter::block::container::HandlerContext;
use crate::converter::handlers::{handle_blockquote, handle_code, handle_graphic, handle_img, handle_link, handle_pre};

struct TagDispatcher<'a, 'output> {
    tag_name: &'a str,
    node_handle: &'a tl::NodeHandle,
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    output: &'output mut String,
    handler: HandlerContext<'a>,
}

pub(super) fn dispatch_tag<'a>(
    tag_name: &'a str,
    node_handle: &'a tl::NodeHandle,
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    output: &mut String,
    handler: HandlerContext<'a>,
) -> bool {
    // ~keep In a table cell a code block is one code span, which has no lines.
    if handler.ctx.in_code_block
        && !handler.ctx.in_table_cell
        && crate::converter::handlers::code_block::is_line_element_in_pre(tag_name)
    {
        crate::converter::handlers::code_block::handle_line_element_in_pre(node_handle, parser, output, handler);
        return false;
    }
    let mut dispatcher = TagDispatcher {
        tag_name,
        node_handle,
        tag,
        parser,
        output,
        handler,
    };
    if dispatcher.dispatch_inline() || dispatcher.dispatch_blocks() {
        return false;
    }
    if let Some(stop) = dispatcher.dispatch_tables_and_lists() {
        return stop;
    }
    if dispatcher.dispatch_categories() {
        return false;
    }
    dispatcher.dispatch_fallback();
    false
}

impl TagDispatcher<'_, '_> {
    fn dispatch_inline(&mut self) -> bool {
        let handler = self.handler;
        match self.tag_name {
            "strong" | "b" | "em" | "i" | "mark" | "del" | "s" | "strike" | "ins" | "u" | "small" | "sub" | "sup"
            | "kbd" | "samp" | "var" | "dfn" | "abbr" | "ruby" | "rb" | "rt" | "rp" | "rtc" | "span" => {
                crate::converter::inline::dispatch_inline_handler(
                    self.tag_name,
                    crate::converter::inline::HandlerContext::new((
                        self.node_handle,
                        self.parser,
                        self.output,
                        handler.options,
                        handler.ctx,
                        handler.depth,
                        handler.dom_ctx,
                    )),
                );
            }
            "a" => handle_link(
                self.tag,
                crate::converter::inline::HandlerContext::new((
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler.options,
                    handler.ctx,
                    handler.depth,
                    handler.dom_ctx,
                )),
            ),
            "img" => handle_img(
                self.tag,
                crate::converter::inline::HandlerContext::new((
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler.options,
                    handler.ctx,
                    handler.depth,
                    handler.dom_ctx,
                )),
            ),
            "graphic" => handle_graphic(
                self.tag,
                crate::converter::inline::HandlerContext::new((
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler.options,
                    handler.ctx,
                    handler.depth,
                    handler.dom_ctx,
                )),
            ),
            "code" => handle_code(
                self.tag,
                crate::converter::inline::HandlerContext::new((
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler.options,
                    handler.ctx,
                    handler.depth,
                    handler.dom_ctx,
                )),
            ),
            _ => return false,
        }
        true
    }

    fn dispatch_blocks(&mut self) -> bool {
        let handler = self.handler;
        match self.tag_name {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => crate::converter::block::heading::handle(
                self.tag_name,
                self.node_handle,
                self.parser,
                self.output,
                handler,
            ),
            "p" => crate::converter::block::paragraph::handle(self.node_handle, self.parser, self.output, handler),
            "pre" => handle_pre(
                self.tag,
                crate::converter::inline::HandlerContext::new((
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler.options,
                    handler.ctx,
                    handler.depth,
                    handler.dom_ctx,
                )),
            ),
            "blockquote" => handle_blockquote(
                self.tag,
                crate::converter::inline::HandlerContext::new((
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler.options,
                    handler.ctx,
                    handler.depth,
                    handler.dom_ctx,
                )),
            ),
            "time" | "data" => crate::converter::block::container::handle_passthrough(
                self.node_handle,
                self.parser,
                self.output,
                handler,
            ),
            "wbr" | "thead" | "tbody" | "tfoot" | "tr" | "th" | "td" | "source" => {
                crate::converter::block::container::handle_noop();
            }
            "br" => crate::converter::block::line_break::handle(self.node_handle, self.parser, self.output, handler),
            "hr" => {
                crate::converter::block::horizontal_rule::handle(self.node_handle, self.parser, self.output, handler);
            }
            "div" | "address" | "search" | "hgroup" | "center" | "dialog" => self.dispatch_div_like(),
            _ => return false,
        }
        true
    }

    /// ~keep These content-bearing block containers mirror Tier-1's generic block handling;
    /// routing them through the div handler preserves table-cell and list-item behavior.
    fn dispatch_div_like(&mut self) {
        crate::converter::block::div::handle(self.node_handle, self.parser, self.output, self.handler);
    }

    fn dispatch_tables_and_lists(&mut self) -> Option<bool> {
        match self.tag_name {
            "caption" => {
                crate::converter::block::table::handle_caption(
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.handler,
                );
            }
            "table" => return Some(self.dispatch_table()),
            "ul" | "ol" | "li" | "dl" | "dt" | "dd" => {
                crate::converter::list::dispatch_list_handler(
                    self.tag_name,
                    self.node_handle,
                    self.tag,
                    self.parser,
                    self.output,
                    crate::converter::list::ListContext {
                        options: self.handler.options,
                        ctx: self.handler.ctx,
                        depth: self.handler.depth,
                        dom_ctx: self.handler.dom_ctx,
                    },
                );
            }
            _ => return None,
        }
        Some(false)
    }

    /// ~keep A nested table's width pre-pass uses descendant text instead of recursively
    /// measuring another table, which keeps issue #406's deeply nested layouts linear.
    fn dispatch_table(&mut self) -> bool {
        if self.handler.ctx.measure_width_only {
            let text = self.handler.dom_ctx.text_content(*self.node_handle, self.parser);
            self.output.push_str(text.as_str());
            return true;
        }
        crate::converter::block::table::handle_table_with_context(
            self.node_handle,
            self.parser,
            self.output,
            self.handler,
        );
        false
    }

    fn dispatch_categories(&mut self) -> bool {
        match self.tag_name {
            "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" | "q" | "figure" | "figcaption"
            | "details" | "summary" | "menu" => {
                crate::converter::semantic::dispatch_semantic_handler(
                    self.tag_name,
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.handler,
                );
            }
            "audio" | "video" | "picture" | "iframe" | "svg" | "math" => {
                crate::converter::media::dispatch_media_handler(
                    self.tag_name,
                    self.node_handle,
                    self.parser,
                    self.output,
                    crate::converter::media::MediaContext {
                        options: self.handler.options,
                        ctx: self.handler.ctx,
                        depth: self.handler.depth,
                        dom_ctx: self.handler.dom_ctx,
                    },
                );
            }
            "form" | "fieldset" | "legend" | "label" | "input" | "textarea" | "select" | "option" | "optgroup"
            | "button" | "progress" | "meter" | "output" | "datalist" => {
                crate::converter::form::dispatch_form_handler(
                    self.tag_name,
                    self.node_handle,
                    self.parser,
                    self.output,
                    crate::converter::form::FormContext {
                        options: self.handler.options,
                        ctx: self.handler.ctx,
                        depth: self.handler.depth,
                        dom_ctx: self.handler.dom_ctx,
                    },
                );
            }
            // ~keep Template content is inert, and noscript does not render in the
            // scripting-enabled browser model used by conversion.
            "template" | "noscript" => {}
            _ => return false,
        }
        true
    }

    fn dispatch_fallback(&mut self) {
        match self.tag_name {
            "head" | "script" | "style" => crate::converter::metadata::handle(
                self.tag_name,
                self.node_handle,
                self.parser,
                self.output,
                self.handler,
            ),
            "body" | "html" => crate::converter::block::container::handle_structural_container(
                self.node_handle,
                self.parser,
                self.output,
                self.handler,
            ),
            _ => crate::converter::block::unknown::handle(self.node_handle, self.parser, self.output, self.handler),
        }
    }
}
