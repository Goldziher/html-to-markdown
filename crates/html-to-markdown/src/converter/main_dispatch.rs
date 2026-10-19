use crate::converter::block::container::HandlerContext;
use crate::converter::handlers::{handle_graphic, handle_img, open_link};
use crate::converter::walk::{CaptureEnd, Close, Open};

#[cfg(feature = "visitor")]
pub(super) fn walks_kept_inert_element(
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    handler: HandlerContext<'_>,
) -> bool {
    if handler.ctx.visitor.is_none() || handler.options.hidden_content != crate::options::HiddenContent::All {
        return false;
    }
    // ~keep Raw-text lookalikes inside these legacy containers were never unwrapped as HTML.
    let mut parent = handler.dom_ctx.parent_of(node.get_inner());
    while let Some(id) = parent {
        if handler
            .dom_ctx
            .tag_info(id, parser)
            .is_some_and(|info| matches!(info.name.as_str(), "plaintext" | "noembed" | "noframes"))
        {
            return false;
        }
        parent = handler.dom_ctx.parent_of(id);
    }
    true
}

#[cfg(not(feature = "visitor"))]
pub(super) const fn walks_kept_inert_element(
    _node: &tl::NodeHandle,
    _parser: &tl::Parser,
    _handler: HandlerContext<'_>,
) -> bool {
    false
}

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
        return opened_or_complete(crate::converter::handlers::code_block::open_line_element_in_pre(
            node_handle,
            parser,
            output,
            children,
        ));
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
    if handler.ctx.in_heading
        && matches!(
            tag_name,
            "p" | "section" | "article" | "nav" | "aside" | "header" | "footer" | "main" | "form"
        )
    {
        // ~keep A form in a heading is written as a `div`, but the visitor still sees it (#906).
        #[cfg(feature = "visitor")]
        if tag_name == "form"
            && crate::converter::form::elements::dispatch_form_visitor(
                node_handle,
                tag,
                parser,
                dispatcher.output,
                crate::converter::form::FormContext {
                    options: handler.options,
                    ctx: handler.ctx,
                    depth: handler.depth,
                    dom_ctx: handler.dom_ctx,
                },
            )
        {
            return Dispatched::Complete { stop: false };
        }
        dispatcher.dispatch_div_like();
        return opened_or_complete(dispatcher.opened);
    }
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
        Close::WithOptions(options, inner) => close_tag(
            *inner,
            parser,
            output,
            HandlerContext::new(&options, handler.ctx, handler.depth, handler.dom_ctx),
        ),
        Close::PreLine => crate::converter::handlers::code_block::close_line_element_in_pre(output),
        Close::CodeGroup(end) => crate::converter::form::elements::close_code_group(&end, output),
        Close::Button(end) => crate::converter::form::elements::close_button(&end, parser, output, handler),
        Close::LineEndControl(end) => {
            crate::converter::form::elements::close_line_end_control(&end, parser, output, handler);
        }
        Close::Div(end) => crate::converter::block::div::close(end, output, handler),
        Close::InlineDiv(end) => crate::converter::block::div::close_inline(&end, output),
        Close::Paragraph(end) => crate::converter::block::paragraph::close(&end, output, handler),
        Close::Unknown(end) => crate::converter::block::unknown::close(&end, output),
        Close::CollectedQuote(end) => crate::converter::semantic::close_collected_q(&end, output, handler),
        // ~keep The walk holds the buffer of a capturing element and ends it with `close_capture`.
        // ~keep The hook of an element ends it.
        Close::Capture(_) | Close::Hooked(_) => {}
    }
}

/// Run the second half of the handler of an element that captured its children in `content`.
pub(super) fn close_capture<'a>(
    close: CaptureEnd,
    node_handle: &'a tl::NodeHandle,
    content: &str,
    parser: &'a tl::Parser<'a>,
    output: &'a mut String,
    handler: HandlerContext<'a>,
) {
    use crate::converter::inline::{emphasis, semantic};
    let inline = crate::converter::inline::HandlerContext::new((
        node_handle,
        parser,
        output,
        handler.options,
        handler.ctx,
        handler.depth,
        handler.dom_ctx,
    ));
    match close {
        CaptureEnd::Heading(level) => crate::converter::block::heading::close(
            &format!("h{level}"),
            content,
            node_handle,
            parser,
            inline.output,
            handler,
        ),
        CaptureEnd::Figcaption => {
            crate::converter::semantic::figure::close_figcaption(content, node_handle, parser, inline.output, handler);
        }
        CaptureEnd::Option(end) => {
            crate::converter::form::option::close_option(&end, content, node_handle, parser, inline.output, handler);
        }
        CaptureEnd::Emphasis(kind) => emphasis::close(kind, content, inline),
        CaptureEnd::Summary => {
            crate::converter::semantic::summary::close_summary(content, node_handle, parser, inline.output, handler);
        }
        CaptureEnd::Legend => {
            crate::converter::form::elements::close_legend(content, node_handle, parser, inline.output, handler);
        }
        CaptureEnd::Pre(end) => crate::converter::handlers::code_block::close_pre(&end, content, inline),
        CaptureEnd::Definition(kind) => close_definition(kind, content, node_handle, parser, inline.output, handler),
        CaptureEnd::Figure(start) => crate::converter::semantic::figure::close_figure(
            start,
            content,
            node_handle,
            parser,
            inline.output,
            handler,
        ),
        CaptureEnd::TaskItem(end) => crate::converter::list::item::close_task(&end, content, inline),
        CaptureEnd::Section => crate::converter::semantic::sectioning::close(content, inline.output, handler),
        CaptureEnd::Blockquote => crate::converter::handlers::blockquote::close_blockquote(content, inline),
        CaptureEnd::Mark => semantic::close_mark(content, inline),
        CaptureEnd::Strikethrough(tag_name) => semantic::close_strikethrough(tag_name, content, inline),
        CaptureEnd::Inserted => semantic::close_inserted(content, inline),
        CaptureEnd::Script(end) => semantic::close_script(&end, content, inline),
        CaptureEnd::ItalicSemantic => semantic::close_italic_semantic(content, inline),
        CaptureEnd::Abbreviation => semantic::close_abbreviation(content, inline),
        CaptureEnd::KbdSamp => crate::converter::inline::code::close(content, inline),
        CaptureEnd::RubyText => crate::converter::inline::ruby::close_text(content, inline.output),
        CaptureEnd::RubyAnnotation => crate::converter::inline::ruby::close_annotation(content, inline.output),
        CaptureEnd::Quote => crate::converter::semantic::close_q(content, inline.output, handler),
        CaptureEnd::MediaFallback => {
            crate::converter::media::embedded::close_media_fallback(content, inline.output, handler.ctx);
        }
        CaptureEnd::Code(end) => crate::converter::handlers::code_block::close_code(&end, content, inline),
    }
}

fn close_definition(
    kind: &str,
    content: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    use crate::converter::list::definition::{close_dd, close_dl, close_dt};
    let list = crate::converter::list::ListContext {
        options: handler.options,
        ctx: handler.ctx,
        depth: handler.depth,
        dom_ctx: handler.dom_ctx,
    };
    match kind {
        "dt" => close_dt(content, node_handle, parser, output, list),
        "dd" => close_dd(content, node_handle, parser, output, list),
        _ => close_dl(content, node_handle, parser, output, list),
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
        if self.open_inline() || self.open_inline_styled() || self.dispatch_inline() || self.dispatch_blocks() {
            return false;
        }
        if let Some(stop) = self.dispatch_tables_and_lists() {
            return stop;
        }
        if self.open_category()
            || self.open_form_and_figure()
            || self.open_media_and_controls()
            || self.dispatch_categories()
        {
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
            "ruby" => {
                self.opened = crate::converter::inline::ruby::open(
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler.ctx,
                    self.children,
                );
            }
            "rb" | "rt" | "rp" | "rtc" => {
                self.opened = crate::converter::inline::ruby::open_part(
                    self.tag_name,
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler.ctx,
                    self.children,
                );
            }
            "code" => {
                self.opened = Some(crate::converter::handlers::code_block::open_code(
                    self.node_handle,
                    self.tag,
                    self.parser,
                    handler.ctx,
                    self.children,
                ));
            }
            "kbd" | "samp" | "tt" => {
                self.opened = crate::converter::inline::code::open(
                    crate::converter::inline::HandlerContext::new((
                        self.node_handle,
                        self.parser,
                        self.output,
                        handler.options,
                        handler.ctx,
                        handler.depth,
                        handler.dom_ctx,
                    )),
                    self.children,
                );
            }
            _ => return false,
        }
        true
    }

    /// The styled inline elements that give their children to the walk.
    fn open_inline_styled(&mut self) -> bool {
        let handler = self.handler;
        match self.tag_name {
            "strong" | "b" | "em" | "i" => {
                self.opened = crate::converter::inline::emphasis::open(
                    self.tag_name,
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler.ctx,
                    self.children,
                );
            }
            "mark" | "del" | "s" | "strike" | "ins" | "sub" | "sup" | "var" | "dfn" | "abbr" => {
                self.opened = crate::converter::inline::semantic::open(
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
                    self.children,
                );
            }
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
            "a" => {
                self.opened = open_link(
                    self.tag,
                    &mut crate::converter::inline::HandlerContext::new((
                        self.node_handle,
                        self.parser,
                        self.output,
                        handler.options,
                        handler.ctx,
                        handler.depth,
                        handler.dom_ctx,
                    )),
                    self.children,
                );
            }
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
            _ => return false,
        }
        true
    }

    fn dispatch_blocks(&mut self) -> bool {
        let handler = self.handler;
        match self.tag_name {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.opened = crate::converter::block::heading::open(
                    self.tag_name,
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler,
                    self.children,
                );
            }
            "p" => {
                self.opened = Some(crate::converter::block::paragraph::open(
                    self.node_handle,
                    self.parser,
                    self.output,
                    handler,
                    self.children,
                ));
            }
            "pre" => {
                self.opened = Some(crate::converter::handlers::code_block::open_pre(
                    self.tag,
                    self.parser,
                    handler.ctx,
                    handler.options,
                    self.children,
                ));
            }
            "blockquote" => {
                self.opened = crate::converter::handlers::blockquote::open_blockquote(
                    self.tag,
                    &crate::converter::inline::HandlerContext::new((
                        self.node_handle,
                        self.parser,
                        self.output,
                        handler.options,
                        handler.ctx,
                        handler.depth,
                        handler.dom_ctx,
                    )),
                    self.children,
                );
            }
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
            "ul" | "ol" | "li" => {
                let context = crate::converter::list::ListContext {
                    options: self.handler.options,
                    ctx: self.handler.ctx,
                    depth: self.handler.depth,
                    dom_ctx: self.handler.dom_ctx,
                };
                self.opened = match self.tag_name {
                    "ul" => crate::converter::list::unordered::open(
                        self.node_handle,
                        self.parser,
                        self.output,
                        context,
                        self.children,
                    ),
                    "ol" => crate::converter::list::ordered::open(
                        self.node_handle,
                        self.parser,
                        self.output,
                        context,
                        self.children,
                    ),
                    _ => crate::converter::list::item::open_regular(
                        self.node_handle,
                        self.parser,
                        self.output,
                        context,
                        self.children,
                    ),
                };
            }
            "dl" | "dt" | "dd" => {
                let kind = match self.tag_name {
                    "dt" => "dt",
                    "dd" => "dd",
                    _ => "dl",
                };
                self.opened = crate::converter::list::definition::open(
                    kind,
                    self.node_handle,
                    self.parser,
                    self.output,
                    crate::converter::list::ListContext {
                        options: self.handler.options,
                        ctx: self.handler.ctx,
                        depth: self.handler.depth,
                        dom_ctx: self.handler.dom_ctx,
                    },
                    self.children,
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
                self.handler.ctx,
                self.handler.options.br_in_tables,
            );
            self.output.push_str(text.as_str());
            return true;
        }
        self.opened = crate::converter::block::table::open_table_with_context(
            self.node_handle,
            self.parser,
            self.output,
            self.handler,
            self.children,
        );
        false
    }

    /// The semantic, media and form elements that give their children to the walk.
    fn open_category(&mut self) -> bool {
        match self.tag_name {
            "menu" => {
                let options = std::rc::Rc::new(crate::options::ConversionOptions {
                    bullets: "-".to_owned(),
                    ..self.handler.options.clone()
                });
                self.opened = crate::converter::block::div::open(
                    self.node_handle,
                    self.parser,
                    self.output,
                    HandlerContext::new(&options, self.handler.ctx, self.handler.depth, self.handler.dom_ctx),
                    self.children,
                )
                .map(|mut opened| {
                    opened.close = Close::WithOptions(options, Box::new(opened.close));
                    opened
                });
            }
            "figcaption" => {
                self.opened = crate::converter::semantic::figure::open_figcaption(
                    self.node_handle,
                    self.parser,
                    self.handler,
                    self.children,
                );
            }
            "select" | "datalist" | "option" => {
                self.opened = crate::converter::form::option::open_option(
                    self.tag_name,
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.handler,
                    self.children,
                );
            }

            // ~keep In an inline context a section and a figure write nothing of their own.
            "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" | "figure"
                if self.handler.ctx.convert_as_inline =>
            {
                self.open_children();
            }
            "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" => {
                self.opened = crate::converter::semantic::sectioning::open(
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.handler,
                    self.children,
                );
            }
            "summary" => {
                self.opened = crate::converter::semantic::summary::open_summary(
                    self.node_handle,
                    self.parser,
                    self.handler,
                    self.children,
                );
            }
            _ => return false,
        }
        true
    }

    /// The form and figure elements that give their children to the walk.
    fn open_form_and_figure(&mut self) -> bool {
        match self.tag_name {
            "legend" => {
                self.opened = crate::converter::form::elements::open_legend(
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.handler,
                    self.children,
                );
            }
            "fieldset" => {
                self.opened = crate::converter::form::elements::open_fieldset(
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.handler,
                    self.children,
                );
            }
            "label" => {
                self.opened = crate::converter::form::elements::open_label(
                    self.node_handle,
                    self.parser,
                    self.output,
                    crate::converter::form::FormContext {
                        options: self.handler.options,
                        ctx: self.handler.ctx,
                        depth: self.handler.depth,
                        dom_ctx: self.handler.dom_ctx,
                    },
                    self.children,
                );
            }
            "details" => {
                self.opened = crate::converter::semantic::summary::open_details(
                    self.tag_name,
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.handler,
                    self.children,
                );
            }
            "figure" => {
                self.opened = crate::converter::semantic::figure::open_figure(
                    crate::converter::inline::HandlerContext::new((
                        self.node_handle,
                        self.parser,
                        self.output,
                        self.handler.options,
                        self.handler.ctx,
                        self.handler.depth,
                        self.handler.dom_ctx,
                    )),
                    self.children,
                );
            }
            "optgroup" => self.open_children(),
            _ => return false,
        }
        true
    }

    /// The quote, media and control elements that give their children to the walk.
    fn open_media_and_controls(&mut self) -> bool {
        match self.tag_name {
            "q" => {
                self.opened = crate::converter::semantic::open_q(
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.handler,
                    self.children,
                );
            }
            "audio" | "video" => {
                self.opened = crate::converter::media::embedded::open_media(
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
                    self.children,
                );
            }
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
                if self.tag_name == "button" {
                    self.opened = crate::converter::form::elements::open_button(
                        self.node_handle,
                        self.parser,
                        self.output,
                        self.children,
                    );
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
            "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" | "figure" | "figcaption"
            | "details" | "summary" | "menu" => {
                crate::converter::semantic::dispatch_semantic_handler(
                    self.tag_name,
                    self.node_handle,
                    self.parser,
                    self.output,
                    self.handler,
                );
            }
            "iframe" | "svg" | "math" => {
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
            "template" | "noscript" => {
                if walks_kept_inert_element(self.node_handle, self.parser, self.handler) {
                    self.open_children();
                }
            }
            _ => return false,
        }
        true
    }

    fn dispatch_fallback(&mut self) -> bool {
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
                    return false;
                }
                self.opened =
                    crate::converter::block::unknown::open(self.node_handle, self.parser, self.output, self.children);
            }
        }
        false
    }
}
