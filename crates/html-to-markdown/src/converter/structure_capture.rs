use crate::converter::Context;
use crate::types::structure_collector::{TextCaptureId, TextCaptureKind};

pub(super) struct BlockTextCapture {
    id: TextCaptureId,
    kind: TextCaptureKind,
}

pub(super) struct ElementTextCapture {
    suspended: Vec<TextCaptureId>,
    block: Option<BlockTextCapture>,
}

pub(super) fn append_text(ctx: &Context, text: &str) {
    if let Some(collector) = ctx.structure_collector.as_ref() {
        let mut collector = collector.borrow_mut();
        if ctx.in_heading && !ctx.in_code {
            collector.append_heading_text(text);
        } else {
            collector.append_text(text);
        }
    }
}

pub(super) fn is_text_capture_active(ctx: &Context) -> bool {
    ctx.structure_collector
        .as_ref()
        .is_some_and(|collector| collector.borrow().has_active_text_capture())
}

pub(super) fn begin_element(tag_name: &str, tag: &tl::HTMLTag<'_>, ctx: &Context) -> ElementTextCapture {
    if ctx.structure_collector.is_none() {
        return ElementTextCapture {
            suspended: Vec::new(),
            block: None,
        };
    }
    let kind = crate::types::structure_builder::annotation_kind_for_tag(tag_name, tag);
    if ends_loose_paragraph(tag_name, ctx) {
        finish_loose_paragraph(ctx);
    } else {
        // ~keep The checkpoint must include an empty capture before a visitor can replace its text.
        begin_loose_paragraph(ctx);
    }
    let suspended = suspend_parent_list_item(tag_name, ctx);
    let block = begin_block(tag_name, ctx);
    if let Some(collector) = ctx.structure_collector.as_ref() {
        collector.borrow_mut().begin_element(kind);
    }
    ElementTextCapture { suspended, block }
}

pub(super) fn finish_element(capture: ElementTextCapture, tag_name: &str, tag: &tl::HTMLTag<'_>, ctx: &Context) {
    if let Some(collector) = ctx.structure_collector.as_ref() {
        let allow_loose_text = !ctx.in_table_cell && !ctx.in_code_block && !ctx.convert_as_inline;
        collector.borrow_mut().finish_element(allow_loose_text);
    }
    resume(ctx, &capture.suspended);
    finish_block(capture.block, tag_name, tag, ctx);
    if ends_loose_paragraph(tag_name, ctx) {
        finish_loose_paragraph(ctx);
    }
}

fn ends_loose_paragraph(tag_name: &str, ctx: &Context) -> bool {
    !ctx.convert_as_inline && crate::converter::utility::content::is_block_level_element(tag_name)
}

/// Start collecting text that no paragraph, heading or list item holds as a paragraph of its
/// own. Table cells and code blocks have their own nodes, and inline conversion has no blocks.
pub(super) fn begin_loose_paragraph(ctx: &Context) {
    if ctx.in_table_cell || ctx.in_code_block || ctx.convert_as_inline {
        return;
    }
    if let Some(collector) = ctx.structure_collector.as_ref() {
        collector.borrow_mut().begin_loose_paragraph();
    }
}

fn finish_loose_paragraph(ctx: &Context) {
    if let Some(collector) = ctx.structure_collector.as_ref() {
        collector.borrow_mut().finish_loose_paragraph();
    }
}

pub(super) fn replace_element(ctx: &Context, replacement: Option<&str>) {
    if let Some(boundary) = ctx.media_boundary.as_ref() {
        boundary.set(Default::default());
    }

    if let Some(collector) = ctx.structure_collector.as_ref() {
        collector.borrow_mut().replace_current_element(replacement);
    }
}

pub(super) fn replace_element_at_start(tag_name: &str, tag: &tl::HTMLTag<'_>, ctx: &Context, replacement: &str) {
    let capture = begin_element(tag_name, tag, ctx);
    replace_element(ctx, Some(replacement));
    finish_element(capture, tag_name, tag, ctx);
}

fn suspend_parent_list_item(tag_name: &str, ctx: &Context) -> Vec<TextCaptureId> {
    if !matches!(tag_name, "ul" | "ol") {
        return Vec::new();
    }
    ctx.structure_collector.as_ref().map_or_else(Vec::new, |collector| {
        collector.borrow_mut().suspend_list_item_captures()
    })
}

fn resume(ctx: &Context, captures: &[TextCaptureId]) {
    if captures.is_empty() {
        return;
    }
    if let Some(collector) = ctx.structure_collector.as_ref() {
        collector.borrow_mut().resume_text_captures(captures);
    }
}

fn begin_block(tag_name: &str, ctx: &Context) -> Option<BlockTextCapture> {
    let kind = match tag_name {
        "p" if !ctx.in_table_cell && !ctx.in_list_item && !ctx.convert_as_inline => TextCaptureKind::Paragraph,
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" if !ctx.in_table_cell => TextCaptureKind::Heading,
        "li" if !ctx.in_table_cell => TextCaptureKind::ListItem,
        _ => return None,
    };
    let collector = ctx.structure_collector.as_ref()?;
    let id = collector.borrow_mut().begin_text_capture(kind);
    Some(BlockTextCapture { id, kind })
}

fn finish_block(capture: Option<BlockTextCapture>, tag_name: &str, tag: &tl::HTMLTag<'_>, ctx: &Context) {
    let Some(capture) = capture else { return };
    let Some(collector) = ctx.structure_collector.as_ref() else {
        return;
    };
    let (text, annotations) = collector.borrow_mut().finish_text_capture(capture.id);
    if text.is_empty() {
        return;
    }
    let mut collector = collector.borrow_mut();
    match capture.kind {
        TextCaptureKind::Paragraph => {
            collector.push_paragraph_with_annotations(&text, annotations);
        }
        TextCaptureKind::Heading => {
            let level = tag_name
                .chars()
                .last()
                .and_then(|character| character.to_digit(10))
                .unwrap_or(1) as u8;
            let id = tag.attributes().get("id").flatten().map(|value| value.as_utf8_str());
            collector.push_heading_with_annotations(level, &text, id.as_deref(), annotations);
        }
        TextCaptureKind::ListItem => {
            collector.push_list_item_with_annotations(&text, annotations);
        }
    }
}

#[cfg(feature = "visitor")]
pub(super) struct VisitorEndCapture<'a, 'parser> {
    pub state: Option<&'a crate::converter::visitor_hooks::VisitorElementState<'parser>>,
    pub tag_name: &'a str,
    pub tag: &'a tl::HTMLTag<'a>,
    pub output: &'a mut String,
    pub element_output_start: usize,
    pub ctx: &'a Context,
    pub depth: usize,
}

#[cfg(feature = "visitor")]
pub(super) fn handle_visitor_end(
    end: VisitorEndCapture<'_, '_>,
) -> Option<crate::converter::visitor_hooks::VisitAction> {
    use crate::converter::visitor_hooks::{VisitAction, VisitorElementEndContext, handle_visitor_element_end};

    let (Some(visitor_handle), Some(state)) = (end.ctx.visitor.as_ref(), end.state) else {
        return None;
    };
    let action = handle_visitor_element_end(
        visitor_handle,
        end.tag_name,
        state,
        end.tag,
        VisitorElementEndContext {
            output: end.output,
            element_output_start: end.element_output_start,
            ctx: end.ctx,
            depth: end.depth,
        },
    );
    if !matches!(action, VisitAction::Continue) {
        if let Some(links) = end.ctx.inline_code_links.as_ref() {
            // ~keep Replaced or removed subtrees no longer own link ranges in the raw code buffer (#813).
            links
                .borrow_mut()
                .retain(|link| link.range.start < end.element_output_start);
        }
    }
    match action {
        VisitAction::Custom => {
            let replacement = end.output.get(end.element_output_start..).map(str::to_string);
            replace_element(end.ctx, replacement.as_deref());
        }
        VisitAction::Skip | VisitAction::Error => replace_element(end.ctx, None),
        VisitAction::Continue => {}
    }
    Some(action)
}
