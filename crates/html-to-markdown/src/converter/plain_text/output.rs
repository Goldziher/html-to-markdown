//! The output of the plain text writer: the text of one buffer and the places in it where a
//! form control ended.
//!
//! ~keep The text is private to this module, so every way it becomes shorter is a function
//! ~keep here: the return to a checkpoint and the removal of the white space at a line end.
//! ~keep Both drop the record of a control that ended in the removed text, and both keep the
//! ~keep record of a control before it.

use crate::converter::form::spacing::{self, ControlStart};

/// The text of one buffer of the plain text writer (the page, a table cell, an option list).
#[derive(Default)]
pub(super) struct PlainOutput {
    text: String,
    /// The lengths of `text` right after each form control wrote its text, in rising order.
    /// While `text` has the last length and does not end with white space, that control is
    /// the last thing in it. After other text, a line end or a block the last length says
    /// nothing.
    control_ends: Vec<usize>,
}

/// A place in a `PlainOutput` to return to.
#[cfg(feature = "visitor")]
pub(super) struct Checkpoint {
    len: usize,
}

impl PlainOutput {
    /// An output with room for `capacity` bytes of text.
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            text: String::with_capacity(capacity),
            control_ends: Vec::new(),
        }
    }

    /// The text so far.
    pub(super) fn as_str(&self) -> &str {
        &self.text
    }

    /// The text, at the end of the buffer.
    pub(super) fn into_text(self) -> String {
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
            self.text.push(' ');
        }
        self.text.push_str(text);
    }

    /// Writes `mark`, a part of the layout that is not text of the page: a line end, the tab
    /// between two table cells, the marker of a list item.
    pub(super) fn push_mark(&mut self, mark: &str) {
        self.text.push_str(mark);
    }

    /// The place where a control starts to write: the end of the text.
    pub(super) fn start_control(&self) -> ControlStart {
        ControlStart::new(&self.text)
    }

    /// Separates what a control wrote since `start` from the text before it, and records that
    /// the control ends at the end of the text. A control that wrote nothing leaves no record.
    pub(super) fn finish_control(&mut self, start: &ControlStart) {
        if start.finish(&mut self.text) {
            self.end_control();
        }
    }

    /// Writes `text`, the text of one option, to this list of options.
    pub(super) fn write_option(&mut self, text: &str) {
        spacing::write_option(&mut self.text, text);
    }

    /// Writes `options`, the text of the options of a list, as one control. In code
    /// (`in_code`) the white space around the options is text of the source.
    pub(super) fn write_option_list(&mut self, options: &str, in_code: bool) {
        if spacing::write_option_list(&mut self.text, options, in_code) {
            self.end_control();
        }
    }

    /// Ends the line: the text ends with at least one line end. An empty text stays empty.
    pub(super) fn end_line(&mut self) {
        if !(self.text.is_empty() || self.text.ends_with('\n')) {
            self.text.push('\n');
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
            self.text.push('\n');
        }
    }

    /// The place to return to: the end of the text now.
    #[cfg(feature = "visitor")]
    pub(super) const fn checkpoint(&self) -> Checkpoint {
        let len = self.text.len();
        Checkpoint { len }
    }

    /// The text written since `checkpoint`.
    #[cfg(feature = "visitor")]
    pub(super) fn since(&self, checkpoint: &Checkpoint) -> &str {
        &self.text[self.start_of(checkpoint)..]
    }

    /// Removes the text written since `checkpoint`. A control that ended in that text is not
    /// in the text any more; a control that ended before it is the last control again.
    #[cfg(feature = "visitor")]
    pub(super) fn rollback(&mut self, checkpoint: &Checkpoint) {
        self.shrink_to(self.start_of(checkpoint));
    }

    /// The offset of `checkpoint` in the text now.
    ///
    /// ~keep The end of a block removes the spaces at the end of its line, so the text can be
    /// ~keep shorter than it was at the checkpoint, and other text can stand where the spaces
    /// ~keep were. The offset is then the nearest character start at or before the checkpoint.
    #[cfg(feature = "visitor")]
    fn start_of(&self, checkpoint: &Checkpoint) -> usize {
        let mut start = checkpoint.len.min(self.text.len());
        while !self.text.is_char_boundary(start) {
            start -= 1;
        }
        start
    }

    /// Records that a control ends at the end of the text.
    fn end_control(&mut self) {
        self.control_ends.push(self.text.len());
    }

    /// Removes the text after `len`, with the record of each control that ended in it.
    ///
    /// ~keep This is the one function that makes the text shorter.
    fn shrink_to(&mut self, len: usize) {
        self.text.truncate(len);
        let kept = self.control_ends.partition_point(|end| *end <= len);
        self.control_ends.truncate(kept);
    }
}
