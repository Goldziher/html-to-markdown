//! Tree walk with a work list on the heap.
//!
//! [`drive`] visits a tree in document order and calls [`Walker::enter`] before the children of
//! a node and [`Walker::leave`] after them. The pending steps and the open frames live in two
//! vectors, so the depth of the tree costs heap and no native stack.

/// The two halves of a visit that [`drive`] calls for each node.
pub trait Walker {
    /// A handle to one node of the tree.
    type Node: Copy;
    /// What an entered node keeps until its children are done.
    type Frame;

    /// Enter `node` below `parent`, the nearest open frame.
    ///
    /// Returns the frame to leave after the children that this call wrote to `children`, in
    /// document order. Returns `None` for a node that is complete; such a call leaves
    /// `children` empty.
    fn enter(
        &mut self,
        node: Self::Node,
        parent: Option<&Self::Frame>,
        children: &mut Vec<Self::Node>,
    ) -> Option<Self::Frame>;

    /// Leave the node of `frame`: all of its children are complete.
    fn leave(&mut self, frame: Self::Frame);

    /// Whether [`drive`] calls [`Walker::before_child`] and [`Walker::after_child`] around each
    /// child of `frame`.
    fn hooks_children(&self, _frame: &Self::Frame) -> bool {
        false
    }

    /// Called before the child `index` of `frame` is entered, when `frame` hooks its children.
    /// Returns whether the walk visits the child. A child that is not visited gets no
    /// [`Walker::after_child`].
    fn before_child(&mut self, _frame: &mut Self::Frame, _index: usize) -> bool {
        true
    }

    /// Called after the child `index` of `frame` is complete with its subtree, when `frame` hooks
    /// its children.
    fn after_child(&mut self, _frame: &mut Self::Frame, _index: usize) {}
}

/// An element of the Markdown walk that its handler opened: the walk visits the children that the
/// handler listed, each one level deeper, and then ends the element with `close`. When `close`
/// captures, the children write to a buffer of their own and `close` gets its text.
pub struct Open {
    /// The conversion state of the children, when it differs from the state of the element.
    pub scope: Option<std::rc::Rc<crate::converter::Context>>,
    /// What the handler does after the children.
    pub close: Close,
}

impl Open {
    /// An element that writes nothing after its children, which read the state of the element.
    pub const fn children_only() -> Self {
        Self {
            scope: None,
            close: Close::Nothing,
        }
    }
}

/// The second half of a handler, with what its first half measured.
pub enum Close {
    /// ~keep A menu overrides bullets for every descendant; the walker unwraps this before the children.
    WithOptions(std::rc::Rc<crate::options::ConversionOptions>, Box<Self>),
    Nothing,
    PreLine,
    Div(crate::converter::block::div::DivEnd),
    InlineDiv(crate::converter::block::div::InlineDivEnd),
    LineEndControl(crate::converter::form::elements::LineEndControlEnd),
    Button(crate::converter::form::elements::ButtonEnd),
    CodeGroup(crate::converter::form::elements::CodeGroupEnd),
    Paragraph(crate::converter::block::paragraph::ParagraphEnd),
    Unknown(crate::converter::block::unknown::UnknownEnd),
    CollectedQuote(crate::converter::semantic::CollectedQuoteEnd),
    /// The children write to a buffer of their own, and the second half of the handler reads
    /// it. The children of every other element write to the buffer of the element.
    Capture(CaptureEnd),
    /// The walk calls the hook before and after each child, and the hook ends the element.
    Hooked(Box<dyn ChildHook>),
}

/// The handler of an element that works between two of its children.
///
/// The walk calls [`ChildHook::before_child`] before it enters the child with the index `index`
/// of the list that the first half of the handler gave, [`ChildHook::after_child`] after that
/// child is complete with its subtree, and [`ChildHook::close`] after the last child.
pub trait ChildHook {
    /// Say how the walk visits the child `index`. `element.output` is the buffer of the element.
    fn before_child(&mut self, index: usize, element: &mut HookSite<'_>) -> Child;

    /// The child `index` is complete. `own` is the buffer that [`ChildHook::before_child`] gave
    /// for it, with what the child wrote.
    fn after_child(&mut self, index: usize, own: Option<String>, element: &mut HookSite<'_>);

    /// End the element: the second half of the handler.
    fn close(self: Box<Self>, element: &mut HookSite<'_>);
}

/// The element that a [`ChildHook`] works for, with its buffer and its state.
pub type HookSite<'a> = crate::converter::inline::HandlerContext<'a>;

/// How the walk visits one child of an element with a [`ChildHook`].
pub enum Child {
    /// The walk does not visit the child, and calls no [`ChildHook::after_child`] for it.
    Skip,
    Walk {
        /// Absolute traversal depth when a handler flattens intermediate DOM nodes.
        depth: Option<usize>,
        /// The buffer that the child writes to. `None` for the buffer of the element.
        buffer: Option<String>,
        /// The conversion state of the child. `None` for the state of the element.
        scope: Option<std::rc::Rc<crate::converter::Context>>,
    },
}

impl Child {
    /// The child writes to the buffer of the element and reads the state of the element.
    pub const IN_PLACE: Self = Self::Walk {
        depth: None,
        buffer: None,
        scope: None,
    };
}

/// The second half of a handler that captures the text of its children.
pub enum CaptureEnd {
    Heading(usize),
    Figcaption,
    Option {
        list: bool,
        word: bool,
    },
    Summary,
    Legend,
    Pre(crate::converter::handlers::code_block::PreEnd),
    Definition(&'static str),
    Figure(usize),
    TaskItem(crate::converter::list::item::TaskItemEnd),
    Section,
    Blockquote,
    Emphasis(crate::converter::inline::emphasis::EmphasisKind),
    Mark,
    /// The name of the element: `del`, `s` or `strike`.
    Strikethrough(&'static str),
    Inserted,
    Script(crate::converter::inline::semantic::ScriptEnd),
    ItalicSemantic,
    Abbreviation,
    KbdSamp,
    /// An `rb` or an `rp` of a ruby.
    RubyText,
    /// An `rt` of a ruby.
    RubyAnnotation,
    Quote,
    /// The content of an `audio` or a `video` that is no `source`.
    MediaFallback,
    Code(crate::converter::handlers::code_block::CodeEnd),
}

impl Close {
    /// Whether the children write to a buffer of their own.
    pub const fn captures(&self) -> bool {
        matches!(self, Self::Capture(_))
    }

    /// Whether the walk calls a hook around each child.
    pub const fn hooks_children(&self) -> bool {
        matches!(self, Self::Hooked(_))
    }
}

enum Step<N> {
    Enter(N),
    Leave,
    /// A child of a frame that hooks its children, with its index among them.
    Child(N, usize),
    /// The end of the child with this index of a frame that hooks its children.
    After(usize),
}

/// The vectors of one walk. A walk leaves them empty, so the next walk can use their memory.
pub struct Scratch<N, F> {
    work: Vec<Step<N>>,
    frames: Vec<F>,
    children: Vec<N>,
}

impl<N, F> Default for Scratch<N, F> {
    fn default() -> Self {
        Self {
            work: Vec::new(),
            frames: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl<N, F> Scratch<N, F> {
    /// The same vectors for frames of the type `G`. The vectors are empty between two walks, so
    /// no frame changes its type. `F` and `G` differ in a lifetime only, so the memory stays.
    pub fn with_frames<G>(self) -> Scratch<N, G> {
        debug_assert!(self.frames.is_empty(), "a walk leaves no frame");
        Scratch {
            work: self.work,
            frames: self.frames.into_iter().filter_map(|_| None).collect(),
            children: self.children,
        }
    }
}

/// Walk each tree of `roots` in order.
pub fn drive<W: Walker>(walker: &mut W, roots: &[W::Node], scratch: &mut Scratch<W::Node, W::Frame>) {
    let Scratch { work, frames, children } = scratch;
    for &root in roots {
        // ~keep The root is entered with no push, so a walk of one complete node allocates nothing.
        let mut next = Some(Step::Enter(root));
        while let Some(step) = next {
            match step {
                Step::Enter(node) => {
                    if let Some(frame) = walker.enter(node, frames.last(), children) {
                        let hooked = walker.hooks_children(&frame);
                        frames.push(frame);
                        work.push(Step::Leave);
                        let first_child = work.len();
                        if hooked {
                            work.extend(
                                children
                                    .iter()
                                    .enumerate()
                                    .map(|(index, &child)| Step::Child(child, index)),
                            );
                        } else {
                            work.extend(children.iter().copied().map(Step::Enter));
                        }
                        work[first_child..].reverse();
                        children.clear();
                    }
                    debug_assert!(children.is_empty(), "a complete node gives no children");
                }
                Step::Leave => {
                    if let Some(frame) = frames.pop() {
                        walker.leave(frame);
                    }
                }
                Step::Child(node, index) => {
                    if frames.last_mut().is_some_and(|frame| walker.before_child(frame, index)) {
                        work.push(Step::After(index));
                        next = Some(Step::Enter(node));
                        continue;
                    }
                }
                Step::After(index) => {
                    if let Some(frame) = frames.last_mut() {
                        walker.after_child(frame, index);
                    }
                }
            }
            next = work.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Scratch, Walker, drive};

    /// A tree as a list of child lists: node `n` has the children `tree[n]`.
    struct Recorder<'a> {
        tree: &'a [Vec<usize>],
        events: Vec<String>,
        open: usize,
        most_open: usize,
        /// The nodes that hook their children.
        hooked: &'a [usize],
        /// The children that a hook does not visit, each as a node and an index.
        skipped: &'a [(usize, usize)],
    }

    impl Walker for Recorder<'_> {
        type Node = usize;
        type Frame = usize;

        fn enter(&mut self, node: usize, parent: Option<&usize>, children: &mut Vec<usize>) -> Option<usize> {
            let below = parent.map_or_else(|| "root".to_string(), ToString::to_string);
            self.events.push(format!("enter {node} below {below}"));
            if self.tree[node].is_empty() {
                return None;
            }
            children.extend(&self.tree[node]);
            self.open += 1;
            self.most_open = self.most_open.max(self.open);
            Some(node)
        }

        fn leave(&mut self, frame: usize) {
            self.open -= 1;
            self.events.push(format!("leave {frame}"));
        }

        fn hooks_children(&self, frame: &usize) -> bool {
            self.hooked.contains(frame)
        }

        fn before_child(&mut self, frame: &mut usize, index: usize) -> bool {
            self.events.push(format!("before child {index} of {frame}"));
            !self.skipped.contains(&(*frame, index))
        }

        fn after_child(&mut self, frame: &mut usize, index: usize) {
            self.events.push(format!("after child {index} of {frame}"));
        }
    }

    fn record<'a>(tree: &'a [Vec<usize>], roots: &[usize]) -> Recorder<'a> {
        record_hooked(tree, roots, &[], &[])
    }

    fn record_hooked<'a>(
        tree: &'a [Vec<usize>],
        roots: &[usize],
        hooked: &'a [usize],
        skipped: &'a [(usize, usize)],
    ) -> Recorder<'a> {
        let mut recorder = Recorder {
            tree,
            events: Vec::new(),
            open: 0,
            most_open: 0,
            hooked,
            skipped,
        };
        drive(&mut recorder, roots, &mut Scratch::default());
        recorder
    }

    #[test]
    fn should_enter_each_node_before_its_children_and_leave_it_after_them() {
        let tree = vec![vec![1, 4], vec![2, 3], vec![], vec![], vec![5], vec![], vec![]];
        let recorder = record(&tree, &[0, 6]);
        assert_eq!(
            recorder.events,
            [
                "enter 0 below root",
                "enter 1 below 0",
                "enter 2 below 1",
                "enter 3 below 1",
                "leave 1",
                "enter 4 below 0",
                "enter 5 below 4",
                "leave 4",
                "leave 0",
                "enter 6 below root",
            ]
        );
        assert_eq!(recorder.open, 0);
    }

    #[test]
    fn should_call_the_hook_before_and_after_each_child_of_a_hooked_node_only() {
        let tree = vec![vec![1, 4, 6], vec![2, 3], vec![], vec![], vec![5], vec![], vec![]];
        let recorder = record_hooked(&tree, &[0], &[0], &[(0, 1)]);
        assert_eq!(
            recorder.events,
            [
                "enter 0 below root",
                "before child 0 of 0",
                "enter 1 below 0",
                "enter 2 below 1",
                "enter 3 below 1",
                "leave 1",
                "after child 0 of 0",
                "before child 1 of 0",
                "before child 2 of 0",
                "enter 6 below 0",
                "after child 2 of 0",
                "leave 0",
            ]
        );
        assert_eq!(recorder.open, 0);
    }

    #[test]
    fn should_hold_one_frame_for_each_level_of_a_spine_on_a_small_thread_stack() {
        const LEVELS: usize = 10_000;
        let most_open = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let tree: Vec<Vec<usize>> = (0..=LEVELS)
                    .map(|node| if node < LEVELS { vec![node + 1] } else { Vec::new() })
                    .collect();
                let recorder = record(&tree, &[0]);
                assert_eq!(recorder.events.len(), 2 * LEVELS + 1);
                recorder.most_open
            })
            .expect("the test thread starts")
            .join()
            .expect("the walk of a spine does not overflow the stack");
        assert_eq!(most_open, LEVELS);
    }
}
