//! The text and the control ends of a `PlainOutput`, each behind the functions that read or
//! move it.
//!
//! ~keep The fields are private to this module, so the output has no other way to read or to
//! ~keep move a byte of the text or a control end. Under `cfg(test)` each function adds its
//! ~keep work to a counter: a test of the output then sees all the work, and a loop that reads
//! ~keep the whole text or every end for each element cannot stay out of the count.

/// The bytes of the text of one output.
#[derive(Default)]
pub(super) struct OutputText {
    text: String,
    /// The count of bytes that the functions here wrote, moved or gave to a reader.
    #[cfg(test)]
    work: std::cell::Cell<usize>,
}

impl OutputText {
    /// A text with room for `capacity` bytes.
    pub(super) fn with_capacity(capacity: usize) -> Self {
        let mut output = Self::default();
        output.text.reserve(capacity);
        output
    }

    /// The length in bytes.
    pub(super) const fn len(&self) -> usize {
        self.text.len()
    }

    /// The text, at the end of the buffer.
    pub(super) fn into_string(self) -> String {
        self.text
    }

    /// Writes `text` at the end.
    pub(super) fn push_str(&mut self, text: &str) {
        #[cfg(test)]
        self.count(text.len());
        self.text.push_str(text);
    }

    /// Writes `text` at `at`: the bytes after `at` move.
    pub(super) fn insert_str(&mut self, at: usize, text: &str) {
        #[cfg(test)]
        self.count(self.text.len() - at + text.len());
        self.text.insert_str(at, text);
    }

    /// Removes the bytes of `range`: the bytes after it move.
    #[cfg(feature = "visitor")]
    pub(super) fn remove(&mut self, range: std::ops::Range<usize>) {
        #[cfg(test)]
        self.count(self.text.len() - range.end);
        self.text.drain(range);
    }

    /// Removes the bytes after `len`. No byte moves.
    pub(super) fn truncate(&mut self, len: usize) {
        self.text.truncate(len);
    }

    /// The character right before `end`, as text. Empty when `end` is 0.
    pub(super) fn char_before(&self, end: usize) -> &str {
        #[cfg(test)]
        self.count(1);
        let head = &self.text[..end];
        head.char_indices().next_back().map_or("", |(start, _)| &head[start..])
    }

    /// The character at `at`, as text. Empty when `at` is the end of the text.
    pub(super) fn char_at(&self, at: usize) -> &str {
        #[cfg(test)]
        self.count(1);
        let tail = &self.text[at..];
        tail.chars().next().map_or("", |first| &tail[..first.len_utf8()])
    }

    /// The text from `at` to the end.
    #[cfg(feature = "visitor")]
    pub(super) fn from(&self, at: usize) -> &str {
        #[cfg(test)]
        self.count(self.text.len() - at);
        &self.text[at..]
    }

    /// Adds `bytes` to the count of work.
    #[cfg(test)]
    fn count(&self, bytes: usize) {
        self.work.set(self.work.get() + bytes);
    }

    /// The count of bytes written, moved or given to a reader so far.
    #[cfg(all(test, feature = "visitor"))]
    pub(super) const fn work(&self) -> usize {
        self.work.get()
    }

    /// The whole text, for an assertion. Not counted.
    #[cfg(all(test, feature = "visitor"))]
    pub(super) fn whole(&self) -> &str {
        &self.text
    }
}

/// The lengths of the text right after each form control wrote its text, in rising order.
#[derive(Default)]
pub(super) struct ControlEnds {
    ends: Vec<usize>,
    /// The count of ends that the functions here read or moved.
    #[cfg(test)]
    touched: std::cell::Cell<usize>,
}

impl ControlEnds {
    /// The count of ends.
    pub(super) const fn len(&self) -> usize {
        self.ends.len()
    }

    /// The last end.
    pub(super) fn last(&self) -> Option<usize> {
        self.ends.last().copied()
    }

    /// Adds `end`, which is not below the last end.
    pub(super) fn push(&mut self, end: usize) {
        self.ends.push(end);
    }

    /// The end at `index`.
    pub(super) fn get(&self, index: usize) -> usize {
        #[cfg(test)]
        self.count(1);
        self.ends[index]
    }

    /// The count of the ends that are not after `place`: the position of the first end after
    /// it. A binary search, because the ends rise.
    pub(super) fn first_after(&self, place: usize) -> usize {
        let (mut low, mut high) = (0, self.len());
        while low < high {
            let middle = low + (high - low) / 2;
            if self.get(middle) <= place {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low
    }

    /// Keeps the first `count` ends.
    pub(super) fn truncate(&mut self, count: usize) {
        self.ends.truncate(count);
    }

    /// Adds `by` to each end from the position `first` on.
    pub(super) fn raise_from(&mut self, first: usize, by: usize) {
        #[cfg(test)]
        self.count(self.len() - first);
        for end in &mut self.ends[first..] {
            *end += by;
        }
    }

    /// Takes `by` from each end from the position `first` on.
    #[cfg(feature = "visitor")]
    pub(super) fn lower_from(&mut self, first: usize, by: usize) {
        #[cfg(test)]
        self.count(self.len() - first);
        for end in &mut self.ends[first..] {
            *end -= by;
        }
    }

    /// Adds `ends` to the count of work.
    #[cfg(test)]
    fn count(&self, ends: usize) {
        self.touched.set(self.touched.get() + ends);
    }

    /// The count of ends read or moved so far.
    #[cfg(all(test, feature = "visitor"))]
    pub(super) const fn touched(&self) -> usize {
        self.touched.get()
    }

    /// Every end, for an assertion. Not counted.
    #[cfg(all(test, feature = "visitor"))]
    pub(super) fn as_slice(&self) -> &[usize] {
        &self.ends
    }
}
