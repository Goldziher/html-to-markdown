use std::rc::Rc;

use super::{builder, cell, cells, scanner, utils};
use crate::converter::Context;
use crate::converter::block::container::HandlerContext;
use crate::converter::walk::{Child, ChildHook, Close, HookSite, Open};

#[derive(Clone, Copy)]
enum CellKind {
    Render,
    #[cfg(feature = "visitor")]
    Preview,
    Layout,
    Caption,
}

impl CellKind {
    const fn is_preview(self) -> bool {
        #[cfg(feature = "visitor")]
        {
            matches!(self, Self::Preview)
        }
        #[cfg(not(feature = "visitor"))]
        {
            false
        }
    }
}

#[derive(Clone, Copy)]
struct CellSpec {
    node: tl::NodeHandle,
    depth: usize,
    kind: CellKind,
    defer: bool,
}

enum Step {
    Row {
        node: tl::NodeHandle,
        header: Option<bool>,
    },
    Cell(CellSpec),
    Content {
        node: tl::NodeHandle,
        depth: usize,
    },
    EndCell,
    #[cfg(feature = "visitor")]
    PreviewEnd,
    RowEnd,
    Direct {
        depth: usize,
    },
}

enum Mode {
    Wrapper,
    Data(Box<DataState>),
    Layout,
    Flat,
}

struct DataState {
    scan: scanner::TableScan,
    total_cols: usize,
    widths: Vec<usize>,
    cache: cells::CellTextCache,
    row_index: usize,
    rowspan: Vec<Option<usize>>,
    has_content: bool,
    deferred: Vec<String>,
    rendered_widths: Option<cells::RenderedWidths>,
}

struct ActiveCell {
    spec: CellSpec,
    scope: Rc<Context>,
    text: String,
    table_ends: Vec<usize>,
    skip: bool,
}

struct TableWalk {
    mode: Mode,
    steps: Vec<Step>,
    text: String,
    active: Option<ActiveCell>,
    row: Option<(tl::NodeHandle, Option<bool>)>,
    row_cells: Vec<tl::NodeHandle>,
    row_texts: Vec<String>,
    skip_row: bool,
    #[cfg(feature = "visitor")]
    visitor: Option<builder::TableVisitorState>,
}

const fn handler<'a>(site: &'a HookSite<'a>) -> HandlerContext<'a> {
    HandlerContext::new(site.options, site.context, site.depth, site.dom_context)
}

pub(super) enum Opened {
    Complete(String),
    Children(Open),
}

pub(super) fn open(
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    handler: HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Opened {
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return Opened::Complete(String::new());
    };
    #[cfg(feature = "visitor")]
    let mut text = String::new();
    #[cfg(not(feature = "visitor"))]
    let text = String::new();
    #[cfg(feature = "visitor")]
    let Some(visitor) = builder::begin_table_visit(node, parser, &mut text, tag, handler) else {
        return Opened::Complete(text);
    };
    let scan = scanner::scan_table(node, parser, handler.dom_ctx, handler.ctx, handler.options.br_in_tables);
    let wrapper = builder::nested_table_wrapper_cell(tag, parser, &scan, handler.ctx).filter(|(_, depth)| {
        handler.depth + depth + 1 < crate::converter::main_helpers::effective_max_depth(handler.options)
    });
    let flat = handler.ctx.in_table_cell && !handler.ctx.in_layout_cell && !handler.ctx.allow_nested_table_markup;
    let layout = !flat && builder::is_layout_table(tag, &scan, wrapper);
    let layout_has_content = scan.has_text || scan.link_count > 0;
    let mode = if flat {
        Mode::Flat
    } else if layout {
        Mode::Layout
    } else if wrapper.is_some() {
        Mode::Wrapper
    } else {
        data_mode(node, tag, parser, handler, scan)
    };
    let mut walk = TableWalk {
        mode,
        steps: Vec::new(),
        text,
        active: None,
        row: None,
        row_cells: Vec::new(),
        row_texts: Vec::new(),
        skip_row: false,
        #[cfg(feature = "visitor")]
        visitor: Some(visitor),
    };
    if let Some((cell, depth)) = wrapper.filter(|_| matches!(walk.mode, Mode::Wrapper)) {
        if let Some(tl::Node::Tag(cell)) = cell.get(parser) {
            for child in cell.children().top().iter() {
                walk.push(
                    *child,
                    Step::Direct {
                        depth: handler.depth + depth + 1,
                    },
                    children,
                );
            }
        }
    } else if !layout || layout_has_content {
        walk.plan(tag, parser, handler, children);
    }
    Opened::Children(Open {
        scope: None,
        close: Close::Hooked(Box::new(walk)),
    })
}

fn data_mode(
    node: &tl::NodeHandle,
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    handler: HandlerContext<'_>,
    scan: scanner::TableScan,
) -> Mode {
    let cols = builder::table_total_columns(node, parser, handler.dom_ctx, handler.ctx);
    let reuse = !handler.options.compact_tables && builder::cell_text_reuse_allowed(handler.ctx, &scan);
    let mut cache = cells::CellTextCache::new(reuse);
    let widths = builder::column_widths(tag, parser, handler, cols, reuse, &mut cache);
    cache.enable();
    Mode::Data(Box::new(DataState {
        scan,
        total_cols: cols,
        widths,
        cache,
        row_index: 0,
        rowspan: vec![None; cols],
        has_content: false,
        deferred: Vec::new(),
        rendered_widths: builder::visitor_rendered_widths(handler),
    }))
}

impl TableWalk {
    fn push(&mut self, node: tl::NodeHandle, step: Step, children: &mut Vec<tl::NodeHandle>) {
        children.push(node);
        self.steps.push(step);
    }

    fn plan(
        &mut self,
        tag: &tl::HTMLTag<'_>,
        parser: &tl::Parser,
        handler: HandlerContext<'_>,
        children: &mut Vec<tl::NodeHandle>,
    ) {
        let mut pending: Vec<_> = tag
            .children()
            .top()
            .iter()
            .map(|node| (*node, handler.depth + 1, None))
            .collect();
        pending.reverse();
        while let Some((node, depth, header)) = pending.pop() {
            if handler.ctx.excluded_node_ids.contains(&node.get_inner()) {
                continue;
            }
            let Some(tl::Node::Tag(tag)) = node.get(parser) else {
                continue;
            };
            if matches!(self.mode, Mode::Flat)
                && depth >= crate::converter::main_helpers::effective_max_depth(handler.options)
            {
                handler.ctx.depth_limit_reached.set(true);
                continue;
            }
            match utils::normalized_tag_name(tag.name().as_utf8_str()).as_ref() {
                "thead" | "tbody" | "tfoot" => {
                    let is_header = tag.name().as_utf8_str().eq_ignore_ascii_case("thead");
                    let first = pending.len();
                    pending.extend(
                        tag.children()
                            .top()
                            .iter()
                            .filter(|node| utils::is_tag_name(node, parser, handler.dom_ctx, "tr"))
                            .map(|node| (*node, depth + 1, Some(is_header))),
                    );
                    pending[first..].reverse();
                }
                "tr" | "row" => self.plan_row(node, depth, header, parser, handler, children),
                "caption" if !matches!(self.mode, Mode::Layout) => {
                    self.plan_cell(
                        CellSpec {
                            node,
                            depth: if matches!(self.mode, Mode::Flat) {
                                depth
                            } else {
                                handler.depth
                            },
                            kind: CellKind::Caption,
                            defer: false,
                        },
                        parser,
                        children,
                    );
                }
                "col" | "colgroup" => {}
                _ if !matches!(self.mode, Mode::Flat) => self.push(node, Step::Direct { depth }, children),
                _ => {}
            }
        }
    }

    fn plan_row(
        &mut self,
        node: tl::NodeHandle,
        depth: usize,
        header: Option<bool>,
        parser: &tl::Parser,
        handler: HandlerContext<'_>,
        children: &mut Vec<tl::NodeHandle>,
    ) {
        let mut row_cells = Vec::new();
        cell::collect_table_cells(&node, parser, handler.dom_ctx, handler.ctx, &mut row_cells);
        if row_cells.is_empty() {
            return;
        }
        self.push(node, Step::Row { node, header }, children);
        let row_depth = if matches!(self.mode, Mode::Flat) {
            depth
        } else {
            handler.depth + 1
        };
        #[cfg(feature = "visitor")]
        if matches!(self.mode, Mode::Data(_)) && handler.ctx.visitor.is_some() {
            for &cell in &row_cells {
                self.plan_cell(
                    CellSpec {
                        node: cell,
                        depth: row_depth,
                        kind: CellKind::Preview,
                        defer: false,
                    },
                    parser,
                    children,
                );
            }
            self.push(node, Step::PreviewEnd, children);
        }
        let kind = if matches!(self.mode, Mode::Layout) {
            CellKind::Layout
        } else {
            CellKind::Render
        };
        for &cell in &row_cells {
            self.plan_cell(
                CellSpec {
                    node: cell,
                    depth: row_depth + 1,
                    kind,
                    defer: matches!(self.mode, Mode::Data(_)) && row_cells.len() == 1,
                },
                parser,
                children,
            );
        }
        self.push(node, Step::RowEnd, children);
    }

    fn plan_cell(&mut self, spec: CellSpec, parser: &tl::Parser, children: &mut Vec<tl::NodeHandle>) {
        self.push(spec.node, Step::Cell(spec), children);
        if let Some(tl::Node::Tag(tag)) = spec.node.get(parser) {
            for &node in tag.children().top().iter() {
                self.push(
                    node,
                    Step::Content {
                        node,
                        depth: spec.depth + 1,
                    },
                    children,
                );
            }
        }
        self.push(spec.node, Step::EndCell, children);
    }

    fn start_cell(&mut self, spec: CellSpec, site: &HookSite<'_>) {
        let mut ctx = site.context.clone();
        if spec.kind.is_preview() {
            ctx.in_table_cell = true;
            detach_collectors(&mut ctx);
        } else if matches!(spec.kind, CellKind::Layout) {
            ctx.convert_as_inline = true;
            ctx.in_layout_cell = true;
            ctx.cell_allow_inline_images = true;
        } else if !matches!(spec.kind, CellKind::Caption) {
            ctx.in_table_cell = true;
        } else {
            ctx.text_in_markers = true;
            ctx.escapes_hyphens = true;
        }
        let ctx = if matches!(spec.kind, CellKind::Caption) {
            ctx
        } else {
            ctx.for_cell(spec.node, site.parser, site.dom_context, site.options.br_in_tables)
                .into_owned()
        };
        let mut cached = match &mut self.mode {
            Mode::Data(state) if matches!(spec.kind, CellKind::Render) => state.cache.take(spec.node.get_inner()),
            _ => None,
        };
        if cached.is_none() && matches!(spec.kind, CellKind::Render) && !self.skip_row {
            if let Some(tl::Node::Tag(tag)) = spec.node.get(site.parser) {
                if !tag
                    .children()
                    .top()
                    .iter()
                    .any(|node| matches!(node.get(site.parser), Some(tl::Node::Tag(_) | tl::Node::Comment(_))))
                {
                    cached = Some(cell::render_cell_text(
                        &spec.node,
                        site.parser,
                        HandlerContext::new(site.options, &ctx, spec.depth, site.dom_context),
                        None,
                    ));
                }
            }
        }
        let skip = self.skip_row || cached.is_some();
        self.active = Some(ActiveCell {
            spec,
            scope: Rc::new(ctx),
            text: cached.unwrap_or_default(),
            table_ends: Vec::new(),
            skip,
        });
    }

    fn finish_cell(&mut self, site: &HookSite<'_>) {
        let Some(mut active) = self.active.take() else { return };
        if self.skip_row {
            return;
        }
        match active.spec.kind {
            #[cfg(feature = "visitor")]
            CellKind::Preview => self
                .row_texts
                .push(crate::text::normalize_whitespace_cow(&active.text).trim().to_string()),
            CellKind::Layout => self
                .row_texts
                .push(crate::text::normalize_whitespace_cow(&active.text).trim().to_string()),
            CellKind::Caption => {
                let text = active.text.trim();
                if !text.is_empty() {
                    if matches!(self.mode, Mode::Flat) {
                        append_flat(&mut self.text, text, site.options.br_in_tables);
                    } else {
                        self.text.push(site.options.strong_em_symbol);
                        self.text.push_str(&text.replace('-', r"\-"));
                        self.text.push(site.options.strong_em_symbol);
                        self.text.push_str("\n\n");
                    }
                }
            }
            CellKind::Render => {
                cell::separate_nested_table_following_content(
                    &mut active.text,
                    &active.table_ends,
                    site.options.br_in_tables,
                );
                let text = cell::finalize_cell_text(active.text, site.options);
                if let Mode::Data(state) = &mut self.mode {
                    // ~keep This cache contains the actual emission walk, including visitor edits (#904).
                    state.cache.store(active.spec.node.get_inner(), text);
                } else {
                    self.row_texts.push(text);
                }
            }
        }
    }

    #[cfg(feature = "visitor")]
    fn finish_preview(&mut self, site: &HookSite<'_>) {
        let (Some((node, header)), Mode::Data(state)) = (self.row, &mut self.mode) else {
            return;
        };
        let before = self.text.len();
        let result = cells::run_row_visitor_with_contents(
            &node,
            &mut self.text,
            cells::RowVisit {
                env: row_env(site),
                cells: &self.row_cells,
                row_index: state.row_index,
                is_header: header.unwrap_or(state.row_index == 0),
                depth: site.depth + 1,
            },
            &self.row_texts,
        );
        self.row_texts.clear();
        self.skip_row = result.is_some();
        if self.skip_row {
            state.has_content |= self.text.len() > before;
            state.row_index += 1;
        }
    }

    fn finish_row(&mut self, site: &HookSite<'_>) {
        if self.skip_row {
            return;
        }
        match &mut self.mode {
            Mode::Data(state) => {
                let Some((node, header)) = self.row else { return };
                let emitted = cells::convert_table_row(
                    &node,
                    &mut self.text,
                    row_env(site),
                    &mut cells::RowRender {
                        visitor_is_complete: true,
                        row_index: state.row_index,
                        has_content: &mut state.has_content,
                        has_span: state.scan.has_span,
                        rowspan_tracker: &mut state.rowspan,
                        total_cols: state.total_cols,
                        header_cols: state.total_cols,
                        depth: site.depth + 1,
                        is_header: header.unwrap_or(state.row_index == 0),
                        col_widths: &state.widths,
                        cell_cache: &mut state.cache,
                        deferred_tables: &mut state.deferred,
                        rendered_widths: state.rendered_widths.as_mut(),
                    },
                );
                if emitted {
                    state.row_index += 1;
                }
            }
            Mode::Flat => append_flat(
                &mut self.text,
                &self
                    .row_texts
                    .iter()
                    .filter(|text| !text.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" "),
                site.options.br_in_tables,
            ),
            Mode::Layout => cells::append_layout_text(
                &mut self.text,
                &self
                    .row_texts
                    .iter()
                    .filter(|text| !text.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" "),
                site.options,
                site.context,
            ),
            Mode::Wrapper => {}
        }
    }
}

fn detach_collectors(ctx: &mut Context) {
    #[cfg(feature = "metadata")]
    {
        ctx.metadata_collector = None;
    }
    ctx.structure_collector = None;
    #[cfg(feature = "inline-images")]
    {
        ctx.inline_collector = None;
    }
}

const fn row_env<'a>(site: &'a HookSite<'a>) -> cells::RowEnv<'a> {
    cells::RowEnv {
        parser: site.parser,
        options: site.options,
        ctx: site.context,
        dom_ctx: site.dom_context,
    }
}

fn append_flat(output: &mut String, content: &str, br: bool) {
    if content.is_empty() {
        return;
    }
    if !output.is_empty() {
        crate::converter::emit_table_cell_break(output, br);
    }
    output.push_str(content);
}

impl ChildHook for TableWalk {
    fn before_child(&mut self, index: usize, site: &mut HookSite<'_>) -> Child {
        match self.steps[index] {
            Step::Row { node, header } => {
                self.row = Some((node, header));
                self.skip_row = false;
                self.row_texts.clear();
                cell::collect_table_cells(&node, site.parser, site.dom_context, site.context, &mut self.row_cells);
            }
            Step::Cell(spec) => self.start_cell(spec, site),
            Step::EndCell => self.finish_cell(site),
            #[cfg(feature = "visitor")]
            Step::PreviewEnd => {
                #[cfg(feature = "visitor")]
                self.finish_preview(site);
            }
            Step::RowEnd => self.finish_row(site),
            Step::Direct { depth } => {
                return Child::Walk {
                    buffer: Some(std::mem::take(&mut self.text)),
                    scope: None,
                    depth: Some(depth),
                };
            }
            Step::Content { node, depth } => {
                let Some(active) = &mut self.active else {
                    return Child::Skip;
                };
                if active.skip {
                    return Child::Skip;
                }
                let nested = matches!(active.spec.kind, CellKind::Render)
                    && utils::is_or_contains_table(&node, site.parser, site.dom_context);
                let scope = if nested {
                    Rc::new(Context {
                        allow_nested_table_markup: active.spec.defer,
                        ..(*active.scope).clone()
                    })
                } else {
                    Rc::clone(&active.scope)
                };
                let buffer = if nested {
                    String::new()
                } else {
                    std::mem::take(&mut active.text)
                };
                return Child::Walk {
                    buffer: Some(buffer),
                    scope: Some(scope),
                    depth: Some(depth),
                };
            }
        }
        Child::Skip
    }

    fn after_child(&mut self, index: usize, own: Option<String>, site: &mut HookSite<'_>) {
        let Some(mut text) = own else { return };
        if matches!(self.steps[index], Step::Direct { .. }) {
            self.text = text;
            return;
        }
        let Some(active) = &mut self.active else { return };
        let Step::Content { node, .. } = self.steps[index] else {
            return;
        };
        if !matches!(active.spec.kind, CellKind::Render)
            || !utils::is_or_contains_table(&node, site.parser, site.dom_context)
        {
            active.text = text;
            return;
        }
        if active.spec.defer {
            if let Mode::Data(state) = &mut self.mode {
                if !text.trim().is_empty() {
                    state.deferred.push(text.trim().to_string());
                }
            }
            return;
        }
        if text.contains('|') {
            text = crate::converter::utility::escaping::escape_bare_pipes_outside_code_spans(&text);
        }
        let text = cell::fold_nested_table_rows(&text, site.options.br_in_tables);
        if !text.is_empty() && !active.text.trim_end().is_empty() {
            crate::converter::emit_table_cell_break(&mut active.text, site.options.br_in_tables);
        }
        active.text.push_str(&text);
        active.table_ends.push(active.text.len());
    }

    fn close(mut self: Box<Self>, site: &mut HookSite<'_>) {
        if let Mode::Data(state) = &mut self.mode {
            if !state.has_content
                && !state.scan.has_header
                && !state.scan.has_caption
                && state.scan.nested_table_count == 0
                && state.deferred.is_empty()
            {
                self.text.clear();
            }
            if !self.text.is_empty() {
                if let Some(widths) = state.rendered_widths.take() {
                    widths.repad(&mut self.text, 0, site.options);
                }
            }
            builder::append_deferred_tables(&mut self.text, &state.deferred);
        } else if matches!(self.mode, Mode::Layout) && !self.text.is_empty() && !self.text.ends_with('\n') {
            self.text.push('\n');
        }
        #[cfg(feature = "visitor")]
        if !matches!(self.mode, Mode::Layout) {
            if let (Some(visitor), Some(tl::Node::Tag(tag))) = (self.visitor.take(), site.node_handle.get(site.parser))
            {
                builder::finish_table_visit(
                    site.node_handle,
                    site.parser,
                    &mut self.text,
                    tag,
                    handler(site),
                    visitor,
                );
            }
        }
        super::append_table_output(
            site.node_handle,
            site.parser,
            site.output,
            HandlerContext::new(site.options, site.context, site.depth, site.dom_context),
            &self.text,
        );
    }
}
