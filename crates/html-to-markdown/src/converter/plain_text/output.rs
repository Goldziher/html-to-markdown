//! The output of the plain text writer: the text of one buffer, the places in it where a
//! form control ended, the places where the open controls started and the places a visitor can
//! return to.
//!
//! ~keep The text is private to this module and no function gives a mutable reference to it
//! ~keep away, so every change of the text is a function here: `append`, `insert_at` and
//! ~keep `shrink_to`. Each keeps every control end, every start of an open control and every
//! ~keep open checkpoint a true place in the text. A caller holds a token for a place, never
//! ~keep the place.

use crate::converter::form::spacing;

/// The text of one buffer of the plain text writer (the page, a table cell, an option list).
#[derive(Default)]
pub(super) struct PlainOutput {
    text: String,
    /// The lengths of `text` right after each form control wrote its text, in rising order.
    /// While `text` has the last length and does not end with white space, that control is
    /// the last thing in it. After other text, a line end or a block the last length says
    /// nothing.
    control_ends: Vec<usize>,
    /// The controls that are open, the outer one first, in rising order of their starts.
    open_controls: Vec<ControlPlace>,
    /// The checkpoints that are open, the outer one first, in rising order of their places.
    #[cfg(feature = "visitor")]
    open_checkpoints: Vec<OpenCheckpoint>,
    /// The count of control ends that `insert_at` and `shrink_to` read or moved.
    #[cfg(test)]
    ends_examined: std::cell::Cell<usize>,
}

/// The start of a control, as the output keeps it while the control is open.
struct ControlPlace {
    /// The place where the control started to write: a length of the text, never above its
    /// length now.
    at: usize,
    /// Whether the text before the control needed a space when the control started.
    separate: bool,
}

/// An open control of a `PlainOutput`. `finish_control` closes it.
#[must_use = "a control stays open until `finish_control` takes it"]
pub(super) struct OpenControl {
    /// The position of the control among the open controls of its output.
    index: usize,
}

impl OpenControl {
    /// The position of the control among the open controls of its output.
    const fn into_index(self) -> usize {
        self.index
    }
}

/// A place in the text to return to, as the output keeps it while the checkpoint is open.
#[cfg(feature = "visitor")]
struct OpenCheckpoint {
    /// The place: a length of the text, never above its length now.
    at: usize,
    /// The text that stood right before the place and that a shorter text removed since.
    removed_before: String,
}

/// An open checkpoint of a `PlainOutput`. `rollback` or `commit` closes it.
#[cfg(feature = "visitor")]
#[must_use = "a checkpoint stays open until `rollback` or `commit` takes it"]
pub(super) struct Checkpoint {
    /// The position of the checkpoint among the open checkpoints of its output.
    index: usize,
}

#[cfg(feature = "visitor")]
impl Checkpoint {
    /// The position of the checkpoint among the open checkpoints of its output.
    const fn into_index(self) -> usize {
        self.index
    }
}

impl PlainOutput {
    /// An output with room for `capacity` bytes of text.
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            text: String::with_capacity(capacity),
            ..Self::default()
        }
    }

    /// The text so far.
    pub(super) fn as_str(&self) -> &str {
        &self.text
    }

    /// The text, at the end of the buffer. Every control and every checkpoint is closed by
    /// then.
    pub(super) fn into_text(self) -> String {
        debug_assert!(
            self.open_controls.is_empty(),
            "a control is open at the end of the buffer"
        );
        #[cfg(feature = "visitor")]
        debug_assert!(
            self.open_checkpoints.is_empty(),
            "a checkpoint is open at the end of the buffer"
        );
        self.text
    }

    /// Writes `text`. When a control ends right before it and `text` starts with a word, one
    /// space separates the two.
    ///
    /// ~keep Every arm except `select`, `datalist` and `option` writes the text of the page
    /// ~keep here; those three separate their text through the shared spacing functions. The
    /// ~keep space after a control is written when the next text arrives, so no arm decides
    /// ~keep ahead what follows a control.
    pub(super) fn push_text(&mut self, text: &str) {
        if self.control_ends.last() == Some(&self.text.len())
            && spacing::needs_space_before(&self.text)
            && spacing::starts_with_word(text) == Some(true)
        {
            self.append(" ");
        }
        self.append(text);
    }

    /// Writes `mark`, a part of the layout that is not text of the page: a line end, the tab
    /// between two table cells, the marker of a list item.
    pub(super) fn push_mark(&mut self, mark: &str) {
        self.append(mark);
    }

    /// Opens a control at the end of the text: the place where it starts to write.
    pub(super) fn start_control(&mut self) -> OpenControl {
        self.open_controls.push(ControlPlace {
            at: self.text.len(),
            separate: spacing::needs_space_before(&self.text),
        });
        OpenControl {
            index: self.open_controls.len() - 1,
        }
    }

    /// Closes `control`, the last control opened: separates what it wrote from the text before
    /// it, and records that the control ends at the end of the text. A control that wrote
    /// nothing leaves no record.
    ///
    /// ~keep A block end in the control can remove spaces before its start. The output moved
    /// ~keep the start with the text, so what the control wrote then starts with that block end.
    pub(super) fn finish_control(&mut self, control: OpenControl) {
        debug_assert_eq!(
            control.into_index() + 1,
            self.open_controls.len(),
            "the control that finishes is not the last one opened"
        );
        if let Some(start) = self.open_controls.pop()
            && let Some(space_owed) = spacing::space_owed_before(start.separate, &self.text[start.at..])
        {
            if space_owed {
                self.insert_at(start.at, " ");
            }
            self.control_ends.push(self.text.len());
        }
    }

    /// Writes `text`, the text of one option, to this list of options.
    pub(super) fn write_option(&mut self, text: &str) {
        if let Some((separator, word)) = spacing::option_word(&self.text, text) {
            self.append(separator);
            self.append(word);
        }
    }

    /// Writes `options`, the text of the options of a list, as one control. In code
    /// (`in_code`) the white space around the options is text of the source.
    pub(super) fn write_option_list(&mut self, options: &str, in_code: bool) {
        if let Some(list) = spacing::option_list_text(options, in_code) {
            let control = self.start_control();
            self.append(list);
            self.finish_control(control);
        }
    }

    /// Ends the line: the text ends with at least one line end. An empty text stays empty.
    pub(super) fn end_line(&mut self) {
        if !(self.text.is_empty() || self.text.ends_with('\n')) {
            self.append("\n");
        }
    }

    /// Ends the block: the text ends with a blank line, and the line before it does not end
    /// with a space or a tab. An empty text stays empty.
    pub(super) fn end_block(&mut self) {
        if self.text.is_empty() {
            return;
        }
        let kept = self.text.trim_end_matches([' ', '\t']).len();
        self.shrink_to(kept);
        let line_ends = self.text.chars().rev().take_while(|&c| c == '\n').count();
        for _ in line_ends..2 {
            self.append("\n");
        }
    }

    /// Opens a checkpoint at the end of the text.
    #[cfg(feature = "visitor")]
    pub(super) fn open(&mut self) -> Checkpoint {
        self.open_checkpoints.push(OpenCheckpoint {
            at: self.text.len(),
            removed_before: String::new(),
        });
        Checkpoint {
            index: self.open_checkpoints.len() - 1,
        }
    }

    /// The text written since `checkpoint`.
    #[cfg(feature = "visitor")]
    pub(super) fn since(&self, checkpoint: &Checkpoint) -> &str {
        let start = self
            .open_checkpoints
            .get(checkpoint.index)
            .map_or(self.text.len(), |open| open.at);
        &self.text[start..]
    }

    /// Closes `checkpoint` and keeps the text written since.
    #[cfg(feature = "visitor")]
    pub(super) fn commit(&mut self, checkpoint: Checkpoint) {
        self.close(checkpoint);
    }

    /// Closes `checkpoint` and makes the text what it was when the checkpoint was opened. A
    /// control that ended in the removed text is not in the text any more; a control that
    /// ended before it is the last control again.
    ///
    /// ~keep The end of a block removes the spaces at the end of its line, and those can
    /// ~keep stand before the checkpoint. The output kept them for the checkpoint, and they
    /// ~keep are written again here: the element that is removed leaves nothing, and the text
    /// ~keep before it stays as it was.
    #[cfg(feature = "visitor")]
    pub(super) fn rollback(&mut self, checkpoint: Checkpoint) {
        let Some(closed) = self.close(checkpoint) else {
            return;
        };
        self.shrink_to(closed.at);
        self.append(&closed.removed_before);
        // ~keep An outer checkpoint that lost the same spaces has its first place again.
        for open in &mut self.open_checkpoints {
            if open.at == closed.at
                && !open.removed_before.is_empty()
                && closed.removed_before.starts_with(&open.removed_before)
            {
                open.at += open.removed_before.len();
                open.removed_before.clear();
            }
        }
    }

    /// Takes `checkpoint` from the open checkpoints, with every checkpoint opened after it.
    #[cfg(feature = "visitor")]
    fn close(&mut self, checkpoint: Checkpoint) -> Option<OpenCheckpoint> {
        let index = checkpoint.into_index();
        debug_assert_eq!(
            index + 1,
            self.open_checkpoints.len(),
            "the checkpoint that closes is not the last one opened"
        );
        self.open_checkpoints.truncate(index + 1);
        if self.open_checkpoints.len() > index {
            self.open_checkpoints.pop()
        } else {
            None
        }
    }

    /// Writes `text` at the end of the text.
    fn append(&mut self, text: &str) {
        self.text.push_str(text);
    }

    /// Writes `text` at `at`, a character boundary of the text. Each control end, each start
    /// of an open control and each open checkpoint after `at` moves with the text it stands in.
    ///
    /// ~keep The control ends rise, so the ends after `at` are the last ones: a binary search
    /// ~keep finds the first, and only those move. A scan of every end made a line of controls
    /// ~keep cost the square of their count. The open controls and the open checkpoints are as
    /// ~keep many as the elements around the place, so each is read.
    fn insert_at(&mut self, at: usize, text: &str) {
        self.text.insert_str(at, text);
        let after = self.ends_up_to(at);
        #[cfg(test)]
        self.ends_examined
            .set(self.ends_examined.get() + self.control_ends.len() - after);
        for end in &mut self.control_ends[after..] {
            *end += text.len();
        }
        for control in &mut self.open_controls {
            if control.at > at {
                control.at += text.len();
            }
        }
        #[cfg(feature = "visitor")]
        for open in &mut self.open_checkpoints {
            if open.at > at {
                open.at += text.len();
            }
        }
    }

    /// Removes the text after `len`, with the record of each control that ended in it. The
    /// start of an open control after `len` moves to `len`. An open checkpoint after `len`
    /// moves to `len` and keeps the text removed before it.
    ///
    /// ~keep This is the one function that makes the text shorter.
    fn shrink_to(&mut self, len: usize) {
        for control in &mut self.open_controls {
            control.at = control.at.min(len);
        }
        #[cfg(feature = "visitor")]
        for open in &mut self.open_checkpoints {
            if open.at > len {
                open.removed_before.insert_str(0, &self.text[len..open.at]);
                open.at = len;
            }
        }
        self.text.truncate(len);
        let kept = self.ends_up_to(len);
        self.control_ends.truncate(kept);
    }

    /// The count of the control ends that are not after `place`: the position of the first end
    /// after it. A binary search, because the ends rise.
    fn ends_up_to(&self, place: usize) -> usize {
        self.control_ends.partition_point(|end| {
            #[cfg(test)]
            self.ends_examined.set(self.ends_examined.get() + 1);
            *end <= place
        })
    }
}

#[cfg(all(test, feature = "visitor"))]
mod tests {
    use super::{Checkpoint, OpenControl, PlainOutput};

    /// One change of an output.
    #[derive(Clone, Copy)]
    enum Step {
        Text(&'static str),
        Control(&'static str),
        Start,
        Finish,
        Options(&'static str),
        EndBlock,
        Open,
        Rollback,
        Commit,
    }

    /// Fails when the start of an open control is not a place in the text, or when the starts
    /// do not rise.
    fn assert_control_starts_are_true(output: &PlainOutput, context: &str) {
        let starts: Vec<usize> = output.open_controls.iter().map(|control| control.at).collect();
        assert!(
            starts
                .iter()
                .all(|at| *at <= output.text.len() && output.text.is_char_boundary(*at)),
            "{context}: the start of an open control is not a place in the text: {starts:?} in {:?}",
            output.text
        );
        assert!(
            starts.is_sorted(),
            "{context}: the starts of the open controls do not rise: {starts:?}"
        );
    }

    /// Fails when a control end, the start of an open control or an open checkpoint is not a
    /// place in the text, or when one of the three lists does not rise.
    fn assert_places_are_true(output: &PlainOutput, context: &str) {
        assert_control_starts_are_true(output, context);
        let len = output.text.len();
        assert!(
            output.control_ends.iter().all(|end| *end <= len),
            "{context}: a control end is past the text: {:?} in {:?}",
            output.control_ends,
            output.text
        );
        assert!(
            output.control_ends.is_sorted(),
            "{context}: the control ends do not rise: {:?}",
            output.control_ends
        );
        let places: Vec<usize> = output.open_checkpoints.iter().map(|open| open.at).collect();
        assert!(
            places.iter().all(|at| *at <= len && output.text.is_char_boundary(*at)),
            "{context}: an open checkpoint is not a place in the text: {places:?} in {:?}",
            output.text
        );
        assert!(
            places.is_sorted(),
            "{context}: the open checkpoints do not rise: {places:?}"
        );
    }

    /// Runs `steps` on a new output, checks the places after each step and returns the text.
    fn run(steps: &[Step]) -> String {
        let mut output = PlainOutput::default();
        let mut open: Vec<Checkpoint> = Vec::new();
        let mut controls: Vec<OpenControl> = Vec::new();
        for (index, step) in steps.iter().enumerate() {
            match *step {
                Step::Text(text) => output.push_text(text),
                Step::Control(text) => {
                    let control = output.start_control();
                    output.push_text(text);
                    output.finish_control(control);
                }
                Step::Start => controls.push(output.start_control()),
                Step::Finish => output.finish_control(controls.pop().expect("the table starts before it finishes")),
                Step::Options(options) => output.write_option_list(options, false),
                Step::EndBlock => output.end_block(),
                Step::Open => open.push(output.open()),
                Step::Rollback => output.rollback(open.pop().expect("the table opens before it closes")),
                Step::Commit => output.commit(open.pop().expect("the table opens before it closes")),
            }
            assert_places_are_true(&output, &format!("step {index}"));
        }
        output.into_text()
    }

    #[test]
    fn a_block_end_below_one_checkpoint_keeps_the_places_true() {
        use Step::{Commit, EndBlock, Open, Rollback, Text};
        let table: [(&[Step], &str); 3] = [
            // ~keep A block end removes spaces before a checkpoint; the rollback writes them again.
            (
                &[Text("aaa   "), Open, EndBlock, Text("bcd"), EndBlock, Rollback],
                "aaa   ",
            ),
            // ~keep The same with a commit: the block end stands.
            (
                &[Text("aaa   "), Open, EndBlock, Text("bcd"), EndBlock, Commit],
                "aaa\n\nbcd\n\n",
            ),
            // ~keep A checkpoint in an empty text.
            (&[Open, EndBlock, Text("b"), EndBlock, Rollback, Text("c")], "c"),
        ];
        for (steps, expected) in table {
            assert_eq!(run(steps), expected);
        }
    }

    #[test]
    fn a_block_end_below_two_checkpoints_keeps_the_places_true() {
        use Step::{Commit, EndBlock, Open, Rollback, Text};
        let table: [(&[Step], &str); 3] = [
            // ~keep Two checkpoints, a block end below both, the inner one kept, the outer one not.
            (
                &[
                    Text("aaa "),
                    Open,
                    Open,
                    EndBlock,
                    Text("b"),
                    Commit,
                    Text("c"),
                    Rollback,
                ],
                "aaa ",
            ),
            // ~keep Two checkpoints, a block end below both, both returned to.
            (
                &[
                    Text("aaa "),
                    Open,
                    Text("b  "),
                    Open,
                    EndBlock,
                    Text("c"),
                    Rollback,
                    Rollback,
                ],
                "aaa ",
            ),
            // ~keep The inner return writes the spaces again; a second block end removes them again.
            (
                &[
                    Text("aa  "),
                    Open,
                    Open,
                    EndBlock,
                    Text("b"),
                    Rollback,
                    EndBlock,
                    Text("c"),
                    Rollback,
                ],
                "aa  ",
            ),
        ];
        for (steps, expected) in table {
            assert_eq!(run(steps), expected);
        }
    }

    #[test]
    fn a_control_beside_a_checkpoint_keeps_the_places_true() {
        use Step::{Commit, Control, EndBlock, Open, Options, Rollback, Text};
        let table: [(&[Step], &str); 3] = [
            // ~keep A control before the checkpoint is the last control again after the return.
            (
                &[
                    Text("aaa"),
                    Control("X"),
                    Open,
                    Control("Y"),
                    Options("one two"),
                    Rollback,
                    Text("bbb"),
                ],
                "aaa X bbb",
            ),
            // ~keep A control that ends with spaces, a block end in a checkpoint, a return.
            (
                &[
                    Text("aaa"),
                    Control("X   "),
                    Open,
                    EndBlock,
                    Text("\u{e9}b"),
                    EndBlock,
                    Rollback,
                    Text("ccc"),
                ],
                "aaa X   ccc",
            ),
            // ~keep A control that holds a checkpoint: the space before the control moves no place wrongly.
            (
                &[Text("aaa"), Open, Control("W"), Control("Y"), Commit, Text("bbb")],
                "aaa W Y bbb",
            ),
        ];
        for (steps, expected) in table {
            assert_eq!(run(steps), expected);
        }
    }

    #[test]
    fn a_block_end_below_the_start_of_an_open_control_keeps_the_places_true() {
        use Step::{EndBlock, Finish, Open, Rollback, Start, Text};
        let table: [(&[Step], &str); 5] = [
            // ~keep A block end in a control removes the spaces before its start: the control
            // ~keep still ends at the end of its text, and the next word gets its space.
            (
                &[Text("b   "), Start, EndBlock, Text("e"), Finish, Text("ccc")],
                "b\n\ne ccc",
            ),
            // ~keep The same with a character of two bytes: the old start was inside it.
            (
                &[Text("b   "), Start, EndBlock, Text("\u{e9}"), Finish, Text("ccc")],
                "b\n\n\u{e9} ccc",
            ),
            // ~keep A control that holds a block end and no text.
            (&[Text("b   "), Start, EndBlock, Finish, Text("ccc")], "b\n\nccc"),
            // ~keep Two open controls: the block end moves the inner start, and the space of
            // ~keep the outer control is written at its own start.
            (
                &[
                    Text("a"),
                    Start,
                    Text("x  "),
                    Start,
                    EndBlock,
                    Text("e"),
                    Finish,
                    Finish,
                    Text("ccc"),
                ],
                "a x\n\ne ccc",
            ),
            // ~keep A return to a checkpoint in the control writes the spaces again.
            (
                &[
                    Text("b   "),
                    Start,
                    Open,
                    EndBlock,
                    Text("e"),
                    Rollback,
                    Finish,
                    Text("ccc"),
                ],
                "b   ccc",
            ),
        ];
        for (steps, expected) in table {
            assert_eq!(run(steps), expected);
        }
    }

    #[test]
    fn text_written_before_the_start_of_an_open_control_moves_the_start() {
        // ~keep No path of the writer writes text before an open control. `insert_at` keeps
        // ~keep the start true all the same: the control starts at 3, then at 5.
        let mut output = PlainOutput::default();
        output.push_text("aaa");
        let control = output.start_control();
        output.push_text("W");
        output.insert_at(1, "  ");
        assert_places_are_true(&output, "after the insert");
        output.finish_control(control);
        assert_eq!(output.as_str(), "a  aa W");
        assert_eq!(output.control_ends, [7]);
        assert_eq!(output.into_text(), "a  aa W");
    }

    #[test]
    fn an_outer_checkpoint_has_its_place_again_after_an_inner_return() {
        let mut output = PlainOutput::default();
        output.push_text("aa  ");
        let outer = output.open();
        let inner = output.open();
        output.end_block();
        output.push_text("b");
        assert_eq!(output.since(&outer), "\n\nb");
        output.rollback(inner);
        assert_eq!(output.as_str(), "aa  ");
        assert_eq!(output.since(&outer), "");
        output.push_text("c");
        assert_eq!(output.since(&outer), "c");
        output.rollback(outer);
        assert_eq!(output.into_text(), "aa  ");
    }

    #[test]
    fn a_space_written_before_a_control_moves_the_places_after_it() {
        // ~keep The outer control starts at 3 and holds a control that ends at 5 and a
        // ~keep checkpoint at 5. The space of the outer control is written at 3.
        let mut output = PlainOutput::default();
        output.push_text("aaa");
        let outer = output.start_control();
        output.push_text("W");
        let inner = output.start_control();
        output.push_text("Y");
        output.finish_control(inner);
        assert_eq!(output.as_str(), "aaaW Y");
        assert_eq!(output.control_ends, [6]);
        let checkpoint = output.open();
        output.push_text("zz");
        output.finish_control(outer);
        assert_eq!(output.as_str(), "aaa W Y zz");
        assert_eq!(output.control_ends, [7, 10]);
        assert_places_are_true(&output, "after the space");
        assert_eq!(output.since(&checkpoint), " zz");
        output.rollback(checkpoint);
        assert_eq!(output.as_str(), "aaa W Y");
        assert_eq!(output.control_ends, [7]);
        assert_eq!(output.into_text(), "aaa W Y");
    }

    /// The count of control ends that the output examines for `count` controls, each after a
    /// word and each with a space written before it. With `block_ends`, a block ends after
    /// each control.
    fn ends_examined_for(count: usize, block_ends: bool) -> usize {
        let mut output = PlainOutput::default();
        for _ in 0..count {
            output.push_text("z");
            let control = output.start_control();
            output.push_text("X");
            output.finish_control(control);
            if block_ends {
                output.end_block();
            }
        }
        assert_places_are_true(&output, "after the last control");
        assert_eq!(output.control_ends.len(), count);
        let examined = output.ends_examined.get();
        let line = if block_ends { "z X\n\n" } else { "z X " };
        assert_eq!(output.into_text(), line.repeat(count).trim_end_matches(' '));
        examined
    }

    #[test]
    fn controls_one_after_the_other_examine_a_bounded_count_of_control_ends() {
        // ~keep No control end stands after the place of a space, so only the binary search
        // ~keep reads ends: at most `ilog2(count) + 2` for each search, 11 and 13 here. A block
        // ~keep end makes a second search. A scan of every end reads `count * (count - 1) / 2`
        // ~keep ends for the spaces alone: 499,500 and 7,998,000.
        for (block_ends, searches) in [(false, 1_usize), (true, 2)] {
            for (count, reads) in [(1_000_usize, 11_usize), (4_000, 13)] {
                let examined = ends_examined_for(count, block_ends);
                let bound = count * reads * searches;
                assert!(
                    examined <= bound,
                    "{count} controls (block ends: {block_ends}) examined {examined} control ends, over {bound}"
                );
            }
        }
    }
}
