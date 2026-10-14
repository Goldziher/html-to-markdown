use super::super::{BLOCK_TAGS, ListContext, WalkState};
use super::{CaptureKind, Frame, Scope, Walker};
use crate::converter::form::spacing;
use crate::converter::media::svg::graphic_text;

impl<'a> Walker<'a> {
    pub(super) fn dispatch(&mut self, name: &str, tag: &tl::HTMLTag<'_>, scope: Scope<'a>, parser: &tl::Parser) {
        let child = Scope {
            state: WalkState {
                in_document_body: scope.state.in_document_body || name == "body",
                in_content_header_scope: scope.state.in_content_header_scope
                    || matches!(name, "article" | "section" | "main"),
                ..scope.state.descend()
            },
            ..scope
        };
        if self.leaf(name, tag, scope, parser) {
            return;
        }
        match name {
            "pre" => self.pre(tag, child),
            "table" => self.table(tag, child, parser),
            "ul" => self.list(tag, child, ListContext::Unordered),
            "ol" => {
                let start = tag
                    .attributes()
                    .get("start")
                    .flatten()
                    .and_then(|value| value.as_utf8_str().parse::<u32>().ok())
                    .unwrap_or(1);
                self.list(tag, child, ListContext::Ordered { next_index: start });
            }
            "li" => self.list_item(tag, child),
            _ if BLOCK_TAGS.contains(&name) => self.block(tag, child),
            "select" | "datalist" => self.options(name, tag, child),
            "option" if !scope.state.in_option_list => self.options(name, tag, child),
            "option" => self.option(tag, child),
            _ if spacing::writes_own_text(name) => self.control(tag, child),
            _ => self.tag_children(tag, child),
        }
    }

    fn leaf(&mut self, name: &str, tag: &tl::HTMLTag<'_>, scope: Scope<'a>, parser: &tl::Parser) -> bool {
        let buf = &mut self.outputs[scope.output];
        match name {
            "br" => buf.push_mark("\n"),
            "hr" => buf.end_block(),
            "img" => super::super::emit_plain_image(tag, buf, scope.state.options),
            "svg" => {
                if !scope.state.options.skip_images {
                    buf.push_text(&graphic_text(tag, parser, scope.state.options.hidden_content));
                }
            }
            _ => return false,
        }
        true
    }

    fn pre(&mut self, tag: &tl::HTMLTag<'_>, mut scope: Scope<'a>) {
        scope.state.in_pre = true;
        self.outputs[scope.output].end_block();
        self.frames.push(Frame::EndBlock(scope.output));
        self.tag_children(tag, scope);
    }

    fn block(&mut self, tag: &tl::HTMLTag<'_>, scope: Scope<'a>) {
        if scope.state.item_marker_end != Some(self.outputs[scope.output].len()) {
            self.outputs[scope.output].end_block();
        }
        self.frames.push(Frame::EndBlock(scope.output));
        self.tag_children(tag, scope);
    }

    fn list(&mut self, tag: &tl::HTMLTag<'_>, mut scope: Scope<'a>, context: ListContext) {
        self.outputs[scope.output].end_line();
        scope.state.in_pre = false;
        scope.list = self.lists.len();
        self.lists.push(context);
        self.frames.push(Frame::ReleaseList);
        self.frames.push(Frame::EndLine(scope.output));
        self.tag_children(tag, scope);
    }

    fn list_item(&mut self, tag: &tl::HTMLTag<'_>, mut scope: Scope<'a>) {
        let buf = &mut self.outputs[scope.output];
        buf.end_line();
        match &mut self.lists[scope.list] {
            ListContext::Unordered | ListContext::None => buf.push_mark("- "),
            ListContext::Ordered { next_index } => {
                buf.push_mark(&format!("{next_index}. "));
                *next_index += 1;
            }
        }
        scope.state.item_marker_end = Some(buf.len());
        scope.state.in_pre = false;
        self.frames.push(Frame::EndLine(scope.output));
        self.tag_children(tag, scope);
    }

    fn control(&mut self, tag: &tl::HTMLTag<'_>, scope: Scope<'a>) {
        if !scope.state.in_pre {
            let control = self.outputs[scope.output].start_control();
            self.frames.push(Frame::FinishControl {
                output: scope.output,
                control,
            });
        }
        self.tag_children(tag, scope);
    }

    fn options(&mut self, name: &str, tag: &tl::HTMLTag<'_>, mut scope: Scope<'a>) {
        let kind = CaptureKind::Options {
            collapse_lines: !scope.state.in_pre && matches!(name, "select" | "datalist"),
            in_pre: scope.state.in_pre,
        };
        self.frames.push(Frame::FinishCapture {
            output: scope.output,
            own_list: false,
            kind,
        });
        scope.output = self.outputs.len();
        self.outputs.push(super::super::PlainOutput::default());
        scope.state = WalkState {
            in_option_list: true,
            ..scope.state.for_own_buffer()
        };
        if name == "option" {
            self.option(tag, scope);
        } else {
            self.tag_children(tag, scope);
        }
    }

    fn option(&mut self, tag: &tl::HTMLTag<'_>, mut scope: Scope<'a>) {
        self.frames.push(Frame::FinishCapture {
            output: scope.output,
            own_list: false,
            kind: CaptureKind::Option,
        });
        scope.output = self.outputs.len();
        self.outputs.push(super::super::PlainOutput::default());
        scope.state = scope.state.for_own_buffer();
        self.tag_children(tag, scope);
    }
}
