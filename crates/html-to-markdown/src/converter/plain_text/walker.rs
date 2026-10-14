//! Heap continuations keep DOM depth independent of the caller's native stack. ~keep

mod tables;
mod tags;

#[cfg(feature = "visitor")]
use super::output::Checkpoint;
use super::output::OpenControl;
use super::{ListContext, PlainOutput, SKIP_TAGS, WalkState};
use crate::converter::main_helpers::effective_max_depth;
use crate::converter::preprocessing_helpers::should_drop_for_preprocessing;

#[derive(Clone, Copy)]
struct Scope<'a> {
    state: WalkState<'a>,
    output: usize,
    list: usize,
}

enum CaptureKind {
    Caption,
    Cell,
    Option,
    Options { collapse_lines: bool, in_pre: bool },
}

enum Frame<'a> {
    Children {
        handles: std::vec::IntoIter<tl::NodeHandle>,
        scope: Scope<'a>,
        after: bool,
    },
    Node {
        handle: tl::NodeHandle,
        scope: Scope<'a>,
        after: bool,
    },
    EndBlock(usize),
    EndLine(usize),
    Mark {
        output: usize,
        mark: &'static str,
    },
    FinishControl {
        output: usize,
        control: OpenControl,
    },
    ReleaseList,
    Capture {
        handle: tl::NodeHandle,
        scope: Scope<'a>,
        kind: CaptureKind,
        visit_self: bool,
        own_list: bool,
    },
    FinishCapture {
        output: usize,
        own_list: bool,
        kind: CaptureKind,
    },
    #[cfg(feature = "visitor")]
    ElementEnd {
        handle: tl::NodeHandle,
        scope: Scope<'a>,
        checkpoint: Checkpoint,
    },
}

struct Walker<'a> {
    frames: Vec<Frame<'a>>,
    outputs: Vec<PlainOutput>,
    lists: Vec<ListContext>,
}

pub(super) fn extract(handles: &[tl::NodeHandle], parser: &tl::Parser, state: WalkState<'_>) -> String {
    let mut walker = Walker {
        frames: Vec::new(),
        outputs: vec![PlainOutput::with_capacity(1024)],
        lists: vec![ListContext::None],
    };
    walker.children(
        handles.iter().copied(),
        Scope {
            state,
            output: 0,
            list: 0,
        },
    );
    while let Some(frame) = walker.frames.pop() {
        walker.resume(frame, parser);
    }
    walker.outputs.pop().map_or_else(String::new, PlainOutput::into_text)
}

impl<'a> Walker<'a> {
    fn children(&mut self, handles: impl Iterator<Item = tl::NodeHandle>, scope: Scope<'a>) {
        self.frames.push(Frame::Children {
            handles: handles.collect::<Vec<_>>().into_iter(),
            scope,
            after: false,
        });
    }

    fn tag_children(&mut self, tag: &tl::HTMLTag<'_>, scope: Scope<'a>) {
        self.children(tag.children().top().iter().copied(), scope);
    }

    fn resume(&mut self, frame: Frame<'a>, parser: &tl::Parser) {
        match frame {
            Frame::Children { handles, scope, after } => self.next_child(handles, scope, after, parser),
            Frame::Node { handle, scope, after } => self.node(handle, scope, after, parser),
            Frame::EndBlock(output) => self.outputs[output].end_block(),
            Frame::EndLine(output) => self.outputs[output].end_line(),
            Frame::Mark { output, mark } => self.outputs[output].push_mark(mark),
            Frame::FinishControl { output, control } => self.outputs[output].finish_control(control),
            Frame::ReleaseList => {
                self.lists.pop();
            }
            Frame::Capture {
                handle,
                scope,
                kind,
                visit_self,
                own_list,
            } => {
                self.capture(handle, scope, kind, visit_self, own_list, parser);
            }
            Frame::FinishCapture { output, own_list, kind } => self.finish_capture(output, own_list, kind),
            #[cfg(feature = "visitor")]
            Frame::ElementEnd {
                handle,
                scope,
                checkpoint,
            } => self.element_end(handle, scope, checkpoint, parser),
        }
    }

    fn next_child(
        &mut self,
        mut handles: std::vec::IntoIter<tl::NodeHandle>,
        scope: Scope<'a>,
        after: bool,
        parser: &tl::Parser,
    ) {
        let Some(handle) = handles.next() else {
            return;
        };
        let passed_over = scope.state.depth < effective_max_depth(scope.state.options)
            && match handle.get(parser) {
                Some(tl::Node::Comment(_)) => true,
                Some(tl::Node::Tag(tag)) if !scope.state.excludes(&handle) => SKIP_TAGS
                    .iter()
                    .any(|skip| tag.name().as_utf8_str().eq_ignore_ascii_case(skip)),
                _ => false,
            };
        self.frames.push(Frame::Children {
            handles,
            scope,
            after: passed_over,
        });
        self.frames.push(Frame::Node { handle, scope, after });
    }

    fn node(&mut self, handle: tl::NodeHandle, scope: Scope<'a>, after: bool, parser: &tl::Parser) {
        if scope.state.depth >= effective_max_depth(scope.state.options) {
            return;
        }
        match handle.get(parser) {
            Some(tl::Node::Raw(bytes)) => {
                super::emit_raw_text(bytes, &mut self.outputs[scope.output], &scope.state, after);
            }
            Some(tl::Node::Tag(tag)) if !scope.state.excludes(&handle) => self.tag(handle, tag, scope, parser),
            _ => {}
        }
    }

    fn tag(&mut self, handle: tl::NodeHandle, tag: &tl::HTMLTag<'_>, scope: Scope<'a>, parser: &tl::Parser) {
        let name = tag.name().as_utf8_str().to_ascii_lowercase();
        if SKIP_TAGS.contains(&name.as_str())
            || should_drop_for_preprocessing(
                &name,
                tag,
                scope.state.options,
                name == "header" && scope.state.in_document_body && !scope.state.in_content_header_scope,
            )
        {
            return;
        }
        #[cfg(feature = "visitor")]
        {
            if super::visit_plain_element_start(&name, tag, &mut self.outputs[scope.output], &scope.state) {
                return;
            }
            if scope.state.visitor.is_some() {
                let checkpoint = self.outputs[scope.output].open();
                self.frames.push(Frame::ElementEnd {
                    handle,
                    scope,
                    checkpoint,
                });
            }
        }
        #[cfg(not(feature = "visitor"))]
        let _ = handle;
        self.dispatch(&name, tag, scope, parser);
    }

    #[cfg(feature = "visitor")]
    fn element_end(&mut self, handle: tl::NodeHandle, scope: Scope<'a>, checkpoint: Checkpoint, parser: &tl::Parser) {
        if let Some(visitor) = scope.state.visitor
            && let Some(tl::Node::Tag(tag)) = handle.get(parser)
        {
            let name = tag.name().as_utf8_str().to_ascii_lowercase();
            super::visit_plain_element_end(
                visitor,
                &name,
                tag,
                &mut self.outputs[scope.output],
                scope.state.depth,
                checkpoint,
            );
        }
    }

    fn capture(
        &mut self,
        handle: tl::NodeHandle,
        mut scope: Scope<'a>,
        kind: CaptureKind,
        visit_self: bool,
        own_list: bool,
        parser: &tl::Parser,
    ) {
        self.frames.push(Frame::FinishCapture {
            output: scope.output,
            own_list,
            kind,
        });
        scope.output = self.outputs.len();
        self.outputs.push(PlainOutput::default());
        if own_list {
            scope.list = self.lists.len();
            self.lists.push(ListContext::None);
        }
        if visit_self {
            self.frames.push(Frame::Node {
                handle,
                scope,
                after: false,
            });
        } else if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
            self.tag_children(tag, scope);
        }
    }

    fn finish_capture(&mut self, output: usize, own_list: bool, kind: CaptureKind) {
        let Some(captured) = self.outputs.pop() else {
            return;
        };
        let text = captured.into_text();
        if own_list {
            self.lists.pop();
        }
        let buf = &mut self.outputs[output];
        match kind {
            CaptureKind::Caption => {
                if !text.trim().is_empty() {
                    buf.push_text(text.trim());
                    buf.end_block();
                }
            }
            CaptureKind::Cell => buf.push_text(text.trim()),
            CaptureKind::Option => buf.write_option(&text),
            CaptureKind::Options { collapse_lines, in_pre } => {
                let text = if collapse_lines {
                    text.split_whitespace().collect::<Vec<_>>().join(" ")
                } else {
                    text
                };
                buf.write_option_list(&text, in_pre);
            }
        }
    }
}
