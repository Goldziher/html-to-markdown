use crate::converter::block::container::HandlerContext;
use crate::converter::handlers::{handle_blockquote, handle_code, handle_graphic, handle_img, handle_link, handle_pre};
use crate::converter::walk::{Close, Open};

struct TagDispatcher<'a, 'output> {
    tag_name: &'a str,
    node_handle: &'a tl::NodeHandle,
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    output: &'output mut String,
    handler: HandlerContext<'a>,
    children: &'output mut Vec<tl::NodeHandle>,
    opened: Option<Open>,
}

/// What the handler of an element left to do.
pub(super) enum Dispatched {
    /// The handler wrote the element with its children. `stop` is set when the element has no end.
    Complete { stop: bool },
    /// The handler wrote the start of the element and listed its children for the walk.
    Opened(Open),
}

pub(super) fn dispatch_tag<'a>(
    tag_name: &'a str,
    node_handle: &'a tl::NodeHandle,
    parser: &'a tl::Parser<'a>,
    output: &mut String,
    handler: HandlerContext<'a>,
    children: &mut Vec<tl::NodeHandle>,
) -> Dispatched {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return Dispatched::Complete { stop: false };
    };
    // ~keep In a table cell a code block is one code span, which has no lines.
    if handler.ctx.in_code_block
        && !handler.ctx.in_table_cell
        && crate::converter::handlers::code_block::is_line_element_in_pre(tag_name)
    {
        crate::converter::handlers::code_block::handle_line_element_in_pre(node_handle, parser, output, handler);
        return Dispatched::Complete { stop: false };
    }
    // ~keep Inline-code link ranges belong to this buffer. Container handlers must not
    // ~keep collect children in detached buffers or insert block markup around those ranges (#813).
    if handler.ctx.inline_code_links.is_some() && !code_collection_handler(tag_name) {
        return opened_or_complete(crate::converter::block::container::open_children(
            node_handle,
            parser,
            children,
        ));
    }
    let mut dispatcher = TagDispatcher {
        tag_name,
        node_handle,
        tag,
        parser,
        output,
        handler,
        children,
        opened: None,
    };
    let stop = dispatcher.dispatch();
    match dispatcher.opened {
        Some(open) => Dispatched::Opened(open),
        None => Dispatched::Complete { stop },
    }
}

fn opened_or_complete(opened: Option<Open>) -> Dispatched {
    opened.map_or(Dispatched::Complete { stop: false }, Dispatched::Opened)
}

/// Run the second half of the handler that opened an element.
pub(super) fn close_tag(close: Close, parser: &tl::Parser, output: &mut String, handler: HandlerContext<'_>) {
    match close {
        Close::Nothing => {}
        Close::LineEndControl(end) => {
            crate::converter::form::elements::close_line_end_control(&end, parser, output, handler);
        }
        Close::Div(end) => crate::converter::block::div::close(end, output, handler),
        Close::InlineDiv(end) => crate::converter::block::div::close_inline(&end, output),
        Close::Paragraph(end) => crate::converter::block::paragraph::close(&end, output, handler),
        Close::Unknown(end) => crate::converter::block::unknown::close(&end, output),
    }
}

fn code_collection_handler(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "a" | "br"
            | "img"
            | "code"
            | "kbd"
            | "samp"
            | "strong"
            | "b"
            | "em"
            | "i"
            | "mark"
            | "del"
            | "s"
            | "strike"
            | "ins"
            | "u"
            | "small"
            | "sub"
            | "sup"
            | "var"
            | "dfn"
            | "abbr"
            | "span"
            | "q"
            | "template"
            | "noscript"
    )
}

impl TagDispatcher<'_, '_> {
    /// Run the handler of the element. Returns whether the element has no end.
    fn dispatch(&mut self) -> bool {
        if self.open_inline() || self.dispatch_inline() || self.dispatch_blocks() {
            return false;
        }
        if let Some(stop) = self.dispatch_tables_and_lists() {
            return stop;
        }
        if self.open_category() || self.dispatch_categories() {
            return false;
        }
        self.dispatch_fallback();
        false
    }

    fn open_children(&mut self) {
        self.opened = crate::converter::block::container::open_children(self.node_handle, self.parser, self.children);
    }

    /// The inline elements that give their children to the walk.
    fn open_inline(&mut self) -> bool {
        let handler = self.handler;
        match self.tag_name {
            "small" => self.open_children(),
            // ~keep Code shows every character as text: no title in parentheses, no script marks.
            "abbr" | "sub" | "sup" if handler.ctx.in_code => self.open_children(),
            "u" => {
                #[cfg(feature = "visitor")]
                if crate::converter::inline::semantic::visit_underline(
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler,
                ) {
                    return true;
                }
                self.open_children();
            }
            "span" => {
                self.opened = crate::converter::inline::semantic::open_span(
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.children,
                );
            }
            _ => return false,
        }
        true
    }

    fn dispatch_inline(&mut self) -> bool {
        let handler = self.handler;
        match self.tag_name {
            "strong" | "b" | "em" | "i" | "mark" | "del" | "s" | "strike" | "ins" | "sub" | "sup" | "kbd" | "samp"
            | "var" | "dfn" | "abbr" | "ruby" | "rb" | "rt" | "rp" | "rtc" => {
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
            "p" => {
                self.opened = Some(crate::converter::block::paragraph::open(
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler,
                    self.children,
                ));
            }
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
            // ~keep A table handler walks its own rows and cells, so a part that arrives here is
            // ~keep stray, in a cell too: its tag is ignored and its text is kept (#805).
            "time" | "data" | "td" | "th" | "tr" | "thead" | "tbody" | "tfoot" => self.open_children(),
            "wbr" | "source" => {
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
        self.opened =
            crate::converter::block::div::open(self.node_handle, self.parser, self.output, self.handler, self.children);
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
            let text = crate::converter::block::table::cell::nested_table_width_text(
                self.node_handle,
                self.parser,
                self.handler.dom_ctx,
                self.handler.options.br_in_tables,
            );
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

    /// The semantic, media and form elements that give their children to the walk.
    fn open_category(&mut self) -> bool {
        match self.tag_name {
            // ~keep In an inline context a section and a figure write nothing of their own.
            "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" | "figure"
                if self.handler.ctx.convert_as_inline =>
            {
                self.open_children();
            }
            "optgroup" => self.open_children(),
            "picture" => {
                self.opened = Some(crate::converter::media::embedded::open_picture(
                    self.tag,
                    self.parser,
                    self.children,
                ));
            }
            "button" | "textarea" | "output" | "meter" | "progress" => {
                #[cfg(feature = "visitor")]
                if self.tag_name == "button"
                    && crate::converter::form::elements::visit_button(
                        self.node_handle,
                        self.parser,
                        self.output,
                        crate::converter::form::FormContext {
                            options: self.handler.options,
                            ctx: self.handler.ctx,
                            depth: self.handler.depth,
                            dom_ctx: self.handler.dom_ctx,
                        },
                    )
                {
                    return true;
                }
                self.opened = crate::converter::form::elements::open_line_end_control(
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.children,
                );
            }
            _ => return false,
        }
        true
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
            "audio" | "video" | "iframe" | "svg" | "math" => {
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
            "form" | "fieldset" | "legend" | "label" | "input" | "select" | "option" | "datalist" => {
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
            "head" => {
                self.opened =
                    crate::converter::metadata::open_head(self.node_handle, self.parser, self.handler, self.children);
            }
            "script" | "style" => crate::converter::metadata::handle(
                self.tag_name,
                self.node_handle,
                self.parser,
                self.output,
                self.handler,
            ),
            "body" | "html" => self.open_children(),
            _ => {
                #[cfg(feature = "visitor")]
                if crate::converter::block::unknown::visit(self.node_handle, self.parser, self.output, self.handler) {
                    return;
                }
                self.opened =
                    crate::converter::block::unknown::open(self.node_handle, self.parser, self.output, self.children);
            }
        }
    }
}
