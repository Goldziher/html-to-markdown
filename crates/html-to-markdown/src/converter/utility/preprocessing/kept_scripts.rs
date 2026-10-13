//! The kept scripts of a page and the owner mark that names each of them.

/// The `<` of a kept element, as the first pass writes it.
pub(super) const ESCAPED_LT: &str = "&lt;";

/// The start of the name of the owner mark of a kept script. A number ends the name.
const OWNER_MARK_PREFIX: &str = "data-h2m-script-";

/// The kept scripts of one page whose text the tree cannot give back as written.
///
/// ~keep `push_kept_element` writes each `<` of a kept script as `&lt;`, and `as_written` takes
/// ~keep that back. No other code knows of the escape: the tree holds the escaped text, and a
/// ~keep writer of HTML writes it as it is. A body that holds `&lt;` itself reads the same in
/// ~keep the tree as a body that holds `<`, so the escaped text does not name a script. The
/// ~keep owner of a body goes with its element: the first pass writes an owner mark, an
/// ~keep attribute, in the start tag of each script whose escaped text holds `&lt;`, and keeps
/// ~keep the body by the number of the mark. A pass that removes or moves an element takes its
/// ~keep mark with it, so no script is found again by its place or by its text.
/// ~keep `with_owner_marks_taken` takes each mark out of the final tree before a reader sees
/// ~keep an attribute.
#[derive(Debug)]
pub struct KeptScripts {
    /// The number of each owner mark with the body of its script as written, with no white
    /// space at the two ends, in the order of the numbers.
    #[cfg(feature = "metadata")]
    bodies: Vec<(usize, Box<str>)>,
    /// The id of a script node of the final tree with the index of its body in `bodies`, in
    /// the order of the ids.
    #[cfg(feature = "metadata")]
    of_node: Vec<(u32, usize)>,
    /// Whether the page has a kept script with no end tag.
    ///
    /// ~keep The rest of the page is the text of the first such script, and a later start tag
    /// ~keep is in that text, so the first pass marks one script for them: the read stays
    /// ~keep linear on a page of such start tags.
    #[cfg(feature = "metadata")]
    pub(super) has_no_end_tag: bool,
    /// Whether the pass writes owner marks. The pass over a repaired page writes none: the
    /// tree builder writes the marks of the first page again.
    #[cfg(feature = "metadata")]
    pub(super) marks: bool,
    /// The numbers that the page has after `OWNER_MARK_PREFIX`, read at the first mark.
    #[cfg(feature = "metadata")]
    taken: Option<ahash::AHashSet<usize>>,
    /// The lowest number that no mark of this pass has.
    #[cfg(feature = "metadata")]
    next: usize,
}

impl KeptScripts {
    /// The kept scripts of a pass that writes no owner mark.
    pub(super) const fn new() -> Self {
        Self {
            #[cfg(feature = "metadata")]
            bodies: Vec::new(),
            #[cfg(feature = "metadata")]
            of_node: Vec::new(),
            #[cfg(feature = "metadata")]
            has_no_end_tag: false,
            #[cfg(feature = "metadata")]
            marks: false,
            #[cfg(feature = "metadata")]
            taken: None,
            #[cfg(feature = "metadata")]
            next: 0,
        }
    }

    /// The kept scripts of a pass that writes an owner mark for each script whose escaped
    /// text holds `&lt;`.
    #[cfg(feature = "metadata")]
    pub(super) const fn with_marks() -> Self {
        let mut kept_scripts = Self::new();
        kept_scripts.marks = true;
        kept_scripts
    }

    /// The number of the owner mark of the kept script with `body`, when its escaped text
    /// holds `&lt;`. `page` is the page that the pass reads.
    ///
    /// ~keep The page has no attribute with the name of a mark, in any case of its letters: a
    /// ~keep number that the page has after `OWNER_MARK_PREFIX` is not used. So an attribute of
    /// ~keep the author is never read as a mark and is never removed.
    #[cfg(feature = "metadata")]
    pub(super) fn mark(&mut self, page: &str, body: &str) -> Option<usize> {
        let escaped_text_holds_lt = body.contains('<') || body.contains(ESCAPED_LT);
        if !self.marks || !escaped_text_holds_lt {
            return None;
        }
        let taken = self.taken.get_or_insert_with(|| owner_mark_numbers(page.as_bytes()));
        while taken.contains(&self.next) {
            self.next += 1;
        }
        let number = self.next;
        self.next += 1;
        self.bodies.push((number, body.trim().into()));
        Some(number)
    }

    /// The index in `bodies` of the body whose owner mark is the attribute `name` with
    /// `value`.
    ///
    /// ~keep The name and the value are compared whole with the mark that the pass wrote.
    /// ~keep `parse` reads `+1` and `01` as 1, and no mark has such a name.
    #[cfg(feature = "metadata")]
    fn owner(&self, name: &str, value: Option<&str>) -> Option<usize> {
        let digits = name.strip_prefix(OWNER_MARK_PREFIX)?;
        let number: usize = digits.parse().ok()?;
        if number.to_string() != digits || value != Some(owner_mark_value(number).as_str()) {
            return None;
        }
        self.bodies.binary_search_by_key(&number, |(mark, _)| *mark).ok()
    }

    /// Take the owner mark out of each script of `dom`, the tree that the converter reads,
    /// and keep the node that had it.
    ///
    /// ~keep No parse comes after this function, so the id of a node is final. The metadata, a
    /// ~keep visitor and a writer of HTML read the tree after it, so none of them sees a mark.
    /// ~keep The read goes to each node once, and only on a page with a mark.
    #[cfg(feature = "metadata")]
    pub fn with_owner_marks_taken(mut self, dom: &mut tl::VDom<'_>) -> crate::error::Result<Self> {
        if self.bodies.is_empty() {
            return Ok(self);
        }
        for (node, slot) in dom.nodes_mut().iter_mut().enumerate() {
            let Some(tag) = slot.as_tag_mut() else {
                continue;
            };
            if !tag.name().as_bytes().eq_ignore_ascii_case(b"script") {
                continue;
            }
            let owned = tag
                .attributes()
                .iter()
                .find_map(|(name, value)| Some((self.owner(&name, value.as_deref())?, name.into_owned())));
            let Some((index, name)) = owned else {
                continue;
            };
            let key = owned_bytes(name)?;
            tag.attributes_mut().remove(key);
            if let Ok(node) = u32::try_from(node) {
                self.of_node.push((node, index));
            }
        }
        if self.of_node.len() < self.bodies.len() {
            self.take_owner_marks_out_of_text(dom)?;
        }
        Ok(self)
    }

    /// Take each owner mark out of the text of `dom`.
    ///
    /// ~keep The first pass does not know each place whose content is text: a `textarea` with
    /// ~keep no end tag has the rest of the page as its text. The start tag of a script there
    /// ~keep is text in the tree, and its mark with it. The read runs only when a mark of the
    /// ~keep pass is on no script of the tree.
    #[cfg(feature = "metadata")]
    fn take_owner_marks_out_of_text(&self, dom: &mut tl::VDom<'_>) -> crate::error::Result<()> {
        for slot in dom.nodes_mut() {
            let Some(text) = slot.as_raw_mut() else {
                continue;
            };
            let Some(without_marks) = self.without_owner_marks(&text.as_utf8_str()) else {
                continue;
            };
            *text = owned_bytes(without_marks)?;
        }
        Ok(())
    }

    /// `text` with no owner mark of this pass, when it holds one. A mark is the text that
    /// `push_owner_mark` writes, with the space before its name.
    ///
    /// ~keep The read goes to each byte of `text` a fixed number of times: the value is
    /// ~keep compared at its own length, and no search for a quote starts at a name.
    #[cfg(feature = "metadata")]
    fn without_owner_marks(&self, text: &str) -> Option<String> {
        let mut out: Option<String> = None;
        let mut last = 0;
        for (name, _) in text.match_indices(OWNER_MARK_PREFIX) {
            let Some(start) = name.checked_sub(1).filter(|&start| start >= last) else {
                continue;
            };
            let number = name + OWNER_MARK_PREFIX.len();
            let digits = text[number..].bytes().take_while(u8::is_ascii_digit).count();
            let name_end = number + digits;
            let Ok(mark) = text[number..name_end].parse::<usize>() else {
                continue;
            };
            let value = owner_mark_value(mark);
            let end = name_end + value.len() + 3;
            let written = text
                .get(name_end..end)
                .and_then(|rest| rest.strip_prefix("=\"")?.strip_suffix('"'));
            if text.as_bytes()[start] != b' '
                || written != Some(value.as_str())
                || self.owner(&text[name..name_end], written).is_none()
            {
                continue;
            }
            let out = out.get_or_insert_with(|| String::with_capacity(text.len()));
            out.push_str(&text[last..start]);
            last = end;
        }
        let mut out = out?;
        out.push_str(&text[last..]);
        Some(out)
    }

    /// The text of a kept script as the page has it. `node` is the id of the script in the
    /// final tree and `text` is the text that the tree has for it, with no white space at the
    /// two ends.
    ///
    /// ~keep A script with no owner mark reads as the tree has it, and nothing is decoded: a
    /// ~keep `&lt;` of the author stays `&lt;`. The first pass did not write that script again.
    #[cfg(feature = "metadata")]
    #[must_use]
    pub fn as_written<'a>(&'a self, node: u32, text: &'a str) -> &'a str {
        match self.of_node.binary_search_by_key(&node, |(id, _)| *id) {
            Ok(at) => &self.bodies[self.of_node[at].1].1,
            Err(_) => text,
        }
    }
}

/// `text` as bytes that a node of the tree owns.
#[cfg(feature = "metadata")]
fn owned_bytes(text: String) -> crate::error::Result<tl::Bytes<'static>> {
    tl::Bytes::try_from(text).map_err(|error| {
        crate::error::ConversionError::Other(format!("taking an owner mark out of the tree: {error:?}"))
    })
}

/// The value of the owner mark with the number `mark`.
fn owner_mark_value(mark: usize) -> String {
    format!("h2m_script_body_{mark}_end")
}

/// Each number that `page` has after `OWNER_MARK_PREFIX`, in any case of its letters.
///
/// ~keep The parser and the tree builder write the name of an attribute in lower case, so a
/// ~keep name of the author in upper case is the name of a mark in the tree. The `2` of the
/// ~keep prefix has no case and is not frequent in a page, so the read looks for it.
#[cfg(feature = "metadata")]
fn owner_mark_numbers(page: &[u8]) -> ahash::AHashSet<usize> {
    let prefix = OWNER_MARK_PREFIX.as_bytes();
    let Some(two) = memchr::memchr(b'2', prefix) else {
        return ahash::AHashSet::new();
    };
    memchr::memchr_iter(b'2', page)
        .filter_map(|at| {
            let start = at.checked_sub(two)?;
            let number = start + prefix.len();
            if !page.get(start..number)?.eq_ignore_ascii_case(prefix) {
                return None;
            }
            let digits = page[number..].iter().take_while(|byte| byte.is_ascii_digit()).count();
            std::str::from_utf8(&page[number..number + digits]).ok()?.parse().ok()
        })
        .collect()
}

/// Write the owner mark with the number `mark` as an attribute of a start tag, after the name
/// of the tag.
///
/// ~keep The mark has a value of its own, so a `=` after it is not read as its value, and the
/// ~keep reader of the tree compares the value too.
pub(super) fn push_owner_mark(out: &mut String, mark: usize) {
    out.push(' ');
    out.push_str(OWNER_MARK_PREFIX);
    out.push_str(&mark.to_string());
    out.push_str("=\"");
    out.push_str(&owner_mark_value(mark));
    out.push('"');
}
