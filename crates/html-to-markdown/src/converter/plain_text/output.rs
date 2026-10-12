//! The output of the plain text writer: the text of one buffer, the places in it where a
//! form control ended, the places where the open controls started and the places a visitor can
//! return to.
//!
//! ~keep The text is private to this module and no function gives a reference to all of it
//! ~keep away, so every change of the text is a function here: `append`, `insert_at`,
//! ~keep `shrink_to`, `hide_from` and `cut`. Each keeps every control end, every start of an
//! ~keep open control and every open checkpoint a true place in the text. A caller holds a
//! ~keep token for a place, never the place.
//!
//! ~keep The end of a block removes the spaces at the end of its line. While a checkpoint
//! ~keep stands in those spaces, the element of that checkpoint can still be removed, and the
//! ~keep spaces must be there again. So the output does not remove them then: they stay in
//! ~keep the text as a hole, bytes that are not part of the text. A return to the checkpoint
//! ~keep cuts the text at the place of the checkpoint, which never moves, and the hole is
//! ~keep text again. A commit of the checkpoint removes the bytes of the hole. No function
//! ~keep reads or copies a run of spaces once for each element after it.

mod counted;
#[cfg(all(test, feature = "visitor"))]
mod tests;

use crate::converter::form::spacing;
use counted::{ControlEnds, OutputText};

/// The text of one buffer of the plain text writer (the page, a table cell, an option list).
#[derive(Default)]
pub(super) struct PlainOutput {
    /// The bytes of the text, the holes included.
    text: OutputText,
    /// The lengths of the text right after each form control wrote its text. While the text
    /// has the last length and does not end with white space, that control is the last thing
    /// in it. After other text, a line end or a block the last length says nothing.
    control_ends: ControlEnds,
    /// The controls that are open, the outer one first, in rising order of their starts.
    open_controls: Vec<ControlPlace>,
    /// The checkpoints that are open, the outer one first, in rising order of their places.
    #[cfg(feature = "visitor")]
    open_checkpoints: Vec<OpenCheckpoint>,
    /// The holes, in rising order. Each belongs to an open checkpoint.
    #[cfg(feature = "visitor")]
    holes: Vec<Hole>,
    /// The count of spaces and tabs at the end of the text. `append`, `insert_at`, the end of
    /// a block and a return to a checkpoint keep it, so the end of a block does not read the
    /// text to find it.
    blank_tail: usize,
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
    /// The place: the length of the text when the checkpoint was opened. Only `insert_at`
    /// moves it, with the text it stands in.
    at: usize,
    /// The count of spaces and tabs at the end of the text when the checkpoint was opened.
    blank_tail: usize,
    /// The count of holes when the checkpoint was opened.
    holes: usize,
}

/// A run of spaces and tabs that the end of a block removed from the text while a checkpoint
/// stood in the run. The bytes from `start` to `end` are in the buffer and are not text.
#[cfg(feature = "visitor")]
struct Hole {
    /// The place of the first byte of the hole.
    start: usize,
    /// The place after the last byte of the hole.
    end: usize,
    /// The position of the outer checkpoint in the run among the open checkpoints. Its commit
    /// removes the bytes.
    owner: usize,
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

/// The count of spaces and tabs at the end of `text`.
fn blank_at_end(text: &str) -> usize {
    text.len() - text.trim_end_matches([' ', '\t']).len()
}

impl PlainOutput {
    /// An output with room for `capacity` bytes of text.
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            text: OutputText::with_capacity(capacity),
            ..Self::default()
        }
    }

    /// The length of the text so far.
    #[cfg(feature = "visitor")]
    pub(super) fn len(&self) -> usize {
        self.text.len() - self.holes.iter().map(|hole| hole.end - hole.start).sum::<usize>()
    }

    /// The length of the text so far.
    #[cfg(not(feature = "visitor"))]
    pub(super) const fn len(&self) -> usize {
        self.text.len()
    }

    /// Whether the text ends with a line end.
    pub(super) fn ends_with_line_end(&self) -> bool {
        self.text.char_before(self.visible_len()) == "\n"
    }

    /// Whether the text ends with a space.
    pub(super) fn ends_with_space(&self) -> bool {
        self.text.char_before(self.visible_len()) == " "
    }

    /// The text, at the end of the buffer. Every control and every checkpoint is closed by
    /// then, so no hole is left.
    pub(super) fn into_text(self) -> String {
        debug_assert!(
            self.open_controls.is_empty(),
            "a control is open at the end of the buffer"
        );
        #[cfg(feature = "visitor")]
        debug_assert!(
            self.open_checkpoints.is_empty() && self.holes.is_empty(),
            "a checkpoint is open at the end of the buffer"
        );
        self.text.into_string()
    }

    /// Writes `text`. When a control ends right before it and `text` starts with a word, one
    /// space separates the two.
    ///
    /// ~keep Every arm except `select`, `datalist` and `option` writes the text of the page
    /// ~keep here; those three separate their text through the shared spacing functions. The
    /// ~keep space after a control is written when the next text arrives, so no arm decides
    /// ~keep ahead what follows a control.
    pub(super) fn push_text(&mut self, text: &str) {
        let end = self.visible_len();
        if self.control_ends.last() == Some(end)
            && spacing::needs_space_before(self.text.char_before(end))
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
            separate: spacing::needs_space_before(self.text.char_before(self.visible_len())),
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
        let Some(start) = self.open_controls.pop() else {
            return;
        };
        let at = start.at;
        #[cfg(feature = "visitor")]
        let at = self.past_holes(at);
        if let Some(space_owed) = spacing::space_owed_before(start.separate, self.text.char_at(at)) {
            if space_owed {
                self.insert_at(at, " ");
            }
            self.control_ends.push(self.text.len());
        }
    }

    /// Writes `text`, the text of one option, to this list of options.
    pub(super) fn write_option(&mut self, text: &str) {
        let end = self.visible_len();
        if let Some((separator, word)) = spacing::option_word(self.text.char_before(end), text) {
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
        let end = self.visible_len();
        if !(end == 0 || self.text.char_before(end) == "\n") {
            self.append("\n");
        }
    }

    /// Ends the block: the text ends with a blank line, and the line before it does not end
    /// with a space or a tab. An empty text stays empty.
    ///
    /// ~keep The work does not grow with the spaces at the end or with the line ends at the
    /// ~keep end: the count of the spaces is kept, and at most two line ends are read.
    pub(super) fn end_block(&mut self) {
        let end = self.visible_len();
        if end == 0 {
            return;
        }
        if self.blank_tail > 0 {
            let kept = end - self.blank_tail;
            #[cfg(feature = "visitor")]
            let hidden = self.hide_from(kept);
            #[cfg(not(feature = "visitor"))]
            let hidden = false;
            if !hidden {
                self.shrink_to(kept);
            }
            self.blank_tail = 0;
        }
        for _ in self.line_ends_at_end()..2 {
            self.append("\n");
        }
    }

    /// Opens a checkpoint at the end of the text.
    #[cfg(feature = "visitor")]
    pub(super) fn open(&mut self) -> Checkpoint {
        self.open_checkpoints.push(OpenCheckpoint {
            at: self.text.len(),
            blank_tail: self.blank_tail,
            holes: self.holes.len(),
        });
        Checkpoint {
            index: self.open_checkpoints.len() - 1,
        }
    }

    /// The text written since `checkpoint`, the last checkpoint opened.
    #[cfg(feature = "visitor")]
    pub(super) fn since(&self, checkpoint: &Checkpoint) -> &str {
        let start = self
            .open_checkpoints
            .get(checkpoint.index)
            .map_or(self.text.len(), |open| self.past_holes(open.at));
        self.text.from(start)
    }

    /// Closes `checkpoint` and keeps the text written since. The spaces that the end of a
    /// block removed before the checkpoint are removed from the buffer now.
    ///
    /// ~keep The bytes after a hole move once for the hole. A checkpoint has one hole at
    /// ~keep most, so a byte moves at most once for each element around it.
    #[cfg(feature = "visitor")]
    pub(super) fn commit(&mut self, checkpoint: Checkpoint) {
        let owner = checkpoint.index;
        let Some(closed) = self.close(checkpoint) else {
            return;
        };
        let mut index = self.holes.len();
        while index > closed.holes {
            index -= 1;
            if self.holes[index].owner == owner {
                let hole = self.holes.remove(index);
                self.cut(hole.start, hole.end);
            }
        }
    }

    /// Closes `checkpoint` and makes the text what it was when the checkpoint was opened. A
    /// control that ended in the removed text is not in the text any more; a control that
    /// ended before it is the last control again.
    ///
    /// ~keep The end of a block removes the spaces at the end of its line, and those can
    /// ~keep stand before the checkpoint. They are a hole in the buffer since then, and the
    /// ~keep hole is dropped here: the element that is removed leaves nothing, and the text
    /// ~keep before it stays as it was. No byte is written again.
    #[cfg(feature = "visitor")]
    pub(super) fn rollback(&mut self, checkpoint: Checkpoint) {
        let Some(closed) = self.close(checkpoint) else {
            return;
        };
        self.holes.truncate(closed.holes);
        self.shrink_to(closed.at);
        self.blank_tail = closed.blank_tail;
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

    /// The length of the buffer without the holes at its end: the end of the text.
    #[cfg(feature = "visitor")]
    fn visible_len(&self) -> usize {
        self.before_holes(self.text.len())
    }

    /// The length of the buffer: the end of the text.
    #[cfg(not(feature = "visitor"))]
    const fn visible_len(&self) -> usize {
        self.text.len()
    }

    /// The place where the text before `place` ends: before each hole that ends at it.
    #[cfg(feature = "visitor")]
    fn before_holes(&self, mut place: usize) -> usize {
        for hole in self.holes.iter().rev() {
            if hole.end == place {
                place = hole.start;
            }
        }
        place
    }

    /// The place where the text after `place` starts: after each hole that `place` stands in
    /// or at the start of.
    #[cfg(feature = "visitor")]
    fn past_holes(&self, mut place: usize) -> usize {
        for hole in &self.holes {
            if (hole.start..=hole.end).contains(&place) {
                place = hole.end;
            }
        }
        place
    }

    /// The count of line ends at the end of the text, 2 at most.
    fn line_ends_at_end(&self) -> usize {
        let mut end = self.visible_len();
        let mut count = 0;
        while count < 2 && self.text.char_before(end) == "\n" {
            count += 1;
            end -= 1;
            #[cfg(feature = "visitor")]
            {
                end = self.before_holes(end);
            }
        }
        count
    }

    /// Writes `text` at the end of the buffer.
    fn append(&mut self, text: &str) {
        let blank = blank_at_end(text);
        self.blank_tail = if blank == text.len() {
            self.blank_tail + blank
        } else {
            blank
        };
        self.text.push_str(text);
    }

    /// Writes `text` at `at`, a character boundary of the text that is in no hole. Each
    /// control end, each start of an open control, each open checkpoint and each hole after
    /// `at` moves with the text it stands in.
    ///
    /// ~keep The control ends rise, so the ends after `at` are the last ones: a binary search
    /// ~keep finds the first, and only those move. A scan of every end made a line of controls
    /// ~keep cost the square of their count. The open controls, the open checkpoints and the
    /// ~keep holes are as many as the elements around the place, so each is read.
    fn insert_at(&mut self, at: usize, text: &str) {
        if at >= self.text.len() - self.blank_tail {
            let blank = blank_at_end(text);
            self.blank_tail = if blank == text.len() {
                self.blank_tail + blank
            } else {
                self.text.len() - at + blank
            };
        }
        self.text.insert_str(at, text);
        let after = self.control_ends.first_after(at);
        self.control_ends.raise_from(after, text.len());
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
        #[cfg(feature = "visitor")]
        for hole in &mut self.holes {
            if hole.start >= at {
                hole.start += text.len();
                hole.end += text.len();
            }
        }
    }

    /// Removes the text after `len`, with the record of each control that ended in it. The
    /// start of an open control after `len` moves to `len`. No open checkpoint stands after
    /// `len`: the end of a block calls `hide_from` then, and a return removes the text after
    /// its own checkpoint, the last one.
    ///
    /// ~keep This is the one function that makes the text shorter at its end.
    fn shrink_to(&mut self, len: usize) {
        #[cfg(feature = "visitor")]
        debug_assert!(
            self.open_checkpoints.iter().all(|open| open.at <= len),
            "an open checkpoint stands in the text that is removed"
        );
        for control in &mut self.open_controls {
            control.at = control.at.min(len);
        }
        self.text.truncate(len);
        let kept = self.control_ends.first_after(len);
        self.control_ends.truncate(kept);
    }

    /// Makes the spaces and tabs after `kept`, which are the end of the buffer, a hole, when
    /// an open checkpoint stands in them. Returns whether it did. The record of each control
    /// that ended in them is removed, and the start of an open control in them moves to
    /// `kept`, as `shrink_to` does. No byte is read, written or moved.
    #[cfg(feature = "visitor")]
    fn hide_from(&mut self, kept: usize) -> bool {
        let Some(owner) = self.open_checkpoints.iter().position(|open| open.at > kept) else {
            return false;
        };
        for control in &mut self.open_controls {
            control.at = control.at.min(kept);
        }
        let ends = self.control_ends.first_after(kept);
        self.control_ends.truncate(ends);
        self.holes.push(Hole {
            start: kept,
            end: self.text.len(),
            owner,
        });
        true
    }

    /// Removes the bytes from `start` to `end`, a hole whose checkpoint is committed. Each
    /// control end and each start of an open control after it moves with its text. Every open
    /// checkpoint stands before the hole, and so does every other hole: a hole after it has a
    /// checkpoint after it, which is closed by now.
    #[cfg(feature = "visitor")]
    fn cut(&mut self, start: usize, end: usize) {
        debug_assert!(
            self.open_checkpoints.iter().all(|open| open.at <= start),
            "an open checkpoint stands in or after a hole that is removed"
        );
        debug_assert!(
            self.holes.iter().all(|hole| hole.end <= start),
            "a hole stands after a hole that is removed"
        );
        let by = end - start;
        self.text.remove(start..end);
        let after = self.control_ends.first_after(start);
        self.control_ends.lower_from(after, by);
        for control in &mut self.open_controls {
            if control.at > start {
                control.at -= by;
            }
        }
    }
}
