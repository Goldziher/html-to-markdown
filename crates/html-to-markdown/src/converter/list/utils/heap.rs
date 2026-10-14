use super::{
    Context, ListChildProcessor, ListChildrenContext, ListContext, MarkerPosition, add_nested_list_trailing_separator,
    is_whitespace_node,
};
pub struct ListWalkEnd {
    pub ordered: bool,
    pub delimiter: Option<char>,
    #[cfg(feature = "visitor")]
    pub visitor: crate::converter::list::ListVisitorOutput,
}

pub fn open_children_walk(
    node: tl::NodeHandle,
    parser: &tl::Parser,
    context: ListChildrenContext<'_>,
    end: ListWalkEnd,
    children: &mut Vec<tl::NodeHandle>,
) -> crate::converter::walk::Open {
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return crate::converter::walk::Open::children_only();
    };
    let handles: Vec<_> = tag
        .children()
        .top()
        .iter()
        .copied()
        .filter(|handle| !is_whitespace_node(handle, parser))
        .collect();
    children.extend(handles.iter().copied());
    let processor = ListChildProcessor::new(context);
    crate::converter::walk::Open {
        scope: None,
        close: crate::converter::walk::Close::Hooked(Box::new(ListHook {
            parent: std::rc::Rc::new(context.list.ctx.clone()),
            item_ctx: processor.item_ctx,
            counter: processor.counter,
            counter_saturated: false,
            first_item: true,
            marker: MarkerPosition::default(),
            loose: context.loose,
            nested_depth: context.nested_depth,
            end,
            handles,
        })),
    }
}

struct ListHook {
    parent: std::rc::Rc<Context>,
    item_ctx: Context,
    counter: i64,
    counter_saturated: bool,
    first_item: bool,
    marker: MarkerPosition,
    loose: bool,
    nested_depth: usize,
    end: ListWalkEnd,
    handles: Vec<tl::NodeHandle>,
}

impl ListHook {
    fn processor<'a>(&'a self, site: &crate::converter::walk::HookSite<'a>) -> ListChildProcessor<'a> {
        let mut processor = ListChildProcessor::new(ListChildrenContext {
            list: ListContext {
                options: site.options,
                ctx: &self.parent,
                depth: site.depth,
                dom_ctx: site.dom_context,
            },
            ordered: self.end.ordered,
            loose: self.loose,
            nested_depth: self.nested_depth,
            start_counter: self.counter,
            delimiter: self.end.delimiter,
        });
        processor.item_ctx = self.item_ctx.clone();
        processor.counter_saturated = self.counter_saturated;
        processor.first_item = self.first_item;
        processor
    }
}

impl crate::converter::walk::ChildHook for ListHook {
    fn before_child(
        &mut self,
        index: usize,
        site: &mut crate::converter::walk::HookSite<'_>,
    ) -> crate::converter::walk::Child {
        let (marker, context, first) = {
            let mut processor = self.processor(site);
            if self.end.ordered {
                processor.item_ctx.list_counter = self.counter;
            }
            let marker = processor.marker_before(self.handles[index], site.parser, site.output);
            (marker, processor.item_ctx, processor.first_item)
        };
        self.first_item = first;
        self.marker = marker;
        self.item_ctx = context.clone();
        crate::converter::walk::Child::Walk {
            depth: None,
            buffer: None,
            scope: Some(std::rc::Rc::new(context)),
        }
    }
    fn after_child(&mut self, index: usize, _own: Option<String>, site: &mut crate::converter::walk::HookSite<'_>) {
        let marker = std::mem::take(&mut self.marker);
        let (counter, saturated) = {
            let mut processor = self.processor(site);
            processor.marker_after(site.output, marker);
            processor.advance_counter(self.handles[index], site.parser);
            (processor.counter, processor.counter_saturated)
        };
        self.counter = counter;
        self.counter_saturated = saturated;
    }
    fn close(self: Box<Self>, site: &mut crate::converter::walk::HookSite<'_>) {
        let Some(tl::Node::Tag(tag)) = site.node_handle.get(site.parser) else {
            return;
        };
        if !site.context.in_table_cell {
            if let Some(collector) = site.context.structure_collector.as_ref() {
                collector.borrow_mut().push_list_end();
            }
        }
        add_nested_list_trailing_separator(site.output, site.context);
        #[cfg(feature = "visitor")]
        crate::converter::list::visit_list_end(
            site.node_handle,
            tag,
            site.parser,
            site.output,
            ListContext {
                options: site.options,
                ctx: site.context,
                depth: site.depth,
                dom_ctx: site.dom_context,
            },
            self.end.visitor,
        );
        #[cfg(not(feature = "visitor"))]
        let _ = tag;
        if self.end.ordered {
            site.context
                .last_list
                .set(site.output, site.context, self.end.delimiter.unwrap_or('.'));
        }
    }
}
