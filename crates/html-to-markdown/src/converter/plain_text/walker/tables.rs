use super::super::{WalkState, collect_descendant_handles};
use super::{CaptureKind, Frame, Scope, Walker};

impl<'a> Walker<'a> {
    pub(super) fn table(&mut self, tag: &tl::HTMLTag<'_>, scope: Scope<'a>, parser: &tl::Parser) {
        self.outputs[scope.output].end_block();
        self.frames.push(Frame::EndBlock(scope.output));
        let mut scheduled = Vec::new();
        let caption_scope = Scope {
            state: WalkState {
                item_marker_end: None,
                in_pre: false,
                ..scope.state
            },
            ..scope
        };
        for handle in tag.children().top().iter() {
            if let Some(tl::Node::Tag(child)) = handle.get(parser)
                && child.name().as_utf8_str().eq_ignore_ascii_case("caption")
            {
                scheduled.push(Frame::Capture {
                    handle: *handle,
                    scope: caption_scope,
                    kind: CaptureKind::Caption,
                    visit_self: true,
                    own_list: true,
                });
            }
        }
        let mut rows = Vec::new();
        collect_descendant_handles(tag, parser, "tr", &scope.state, &mut rows);
        let cell_scope = Scope {
            state: WalkState {
                item_marker_end: None,
                in_pre: false,
                ..scope.state.descend()
            },
            ..scope
        };
        for (row_index, handle) in rows.into_iter().enumerate() {
            if row_index > 0 {
                scheduled.push(Frame::Mark {
                    output: scope.output,
                    mark: "\n",
                });
            }
            if let Some(tl::Node::Tag(row)) = handle.get(parser) {
                self.schedule_cells(row, cell_scope, parser, &mut scheduled);
            }
        }
        self.frames.extend(scheduled.into_iter().rev());
    }

    fn schedule_cells(
        &self,
        row: &tl::HTMLTag<'_>,
        scope: Scope<'a>,
        parser: &tl::Parser,
        scheduled: &mut Vec<Frame<'a>>,
    ) {
        let mut first = true;
        for handle in row.children().top().iter() {
            if scope.state.excludes(handle) {
                continue;
            }
            let Some(tl::Node::Tag(cell)) = handle.get(parser) else {
                continue;
            };
            let name = cell.name().as_utf8_str();
            if !(name.eq_ignore_ascii_case("td") || name.eq_ignore_ascii_case("th")) {
                continue;
            }
            if !first {
                scheduled.push(Frame::Mark {
                    output: scope.output,
                    mark: "\t",
                });
            }
            first = false;
            scheduled.push(Frame::Capture {
                handle: *handle,
                scope,
                kind: CaptureKind::Cell,
                visit_self: false,
                own_list: true,
            });
        }
    }
}
