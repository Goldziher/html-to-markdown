//! The white space rule of running text, decided in one place for both converters.
//!
//! The rule is CSS `white-space: normal` as a browser applies it to the decoded text of the
//! parsed tree:
//!
//! 1. Collapsible white space is exactly the four characters of [`is_collapsible`]. A form feed,
//!    a no-break space and every other Unicode space is a character.
//! 2. A segment break (a line feed) is removed when the character before or after its run,
//!    across inline element boundaries and comments, is a zero-width space; otherwise it is
//!    a space.
//! 3. A run of collapsible white space, across inline element boundaries, comments and empty
//!    inline elements, is one space.
//! 4. That space is written only where it is owed ([`space_is_owed`]): not at the start of a
//!    line and not after a space already written. Markdown marks are not characters: the space
//!    goes outside the marks.
//! 5. Inside `pre`, code and in strict mode nothing above applies: the caller does not ask.
//!
//! The full converter also decides here what it writes for a no-break space and the other
//! Unicode spaces, which the rule above leaves as characters:
//!
//! - A text is [blank](is_blank) when a reader sees nothing in it. A blank text before the first
//!   visible content of a block writes nothing. A blank text elsewhere is written as it is.
//! - Inside a text with visible characters a Unicode space is a space: the [visible](visible)
//!   part of the text collapses it with the spaces around it, and at an [edge](edges) of the
//!   text it is the one space the edge owes. In a [cell](cell_text) a line end is such a space
//!   too.

use std::borrow::Cow;

use crate::converter::utility::content::ZERO_WIDTH_SPACE;
use crate::text;

/// Whether `character` is collapsible white space: a space, a tab, a line feed or a carriage
/// return. No other character is.
///
/// ~keep A form feed is a character in CSS text, as a browser shows it: the HTML parser treats
/// ~keep it as white space in a tag, but in running text it stays between the words.
#[must_use]
pub const fn is_collapsible(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r')
}

/// `decoded` with every carriage return a line feed: `\r\n` and `\r` are one `\n`.
///
/// ~keep In running text a carriage return is the same line end as a line feed, as a
/// ~keep character or as a reference (`one&#13;`): both converters ask here. Neither asks for
/// ~keep the text of code or `pre`, which is written as it is.
#[must_use]
pub fn with_line_feeds(decoded: Cow<'_, str>) -> Cow<'_, str> {
    if decoded.contains('\r') {
        Cow::Owned(decoded.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        decoded
    }
}

/// The source text `source` with every character reference to collapsible white space written
/// as the character it stands for (`&#10;`, `&#xA;`, `&NewLine;`, `&#13;`, `&#32;`, `&Tab;`).
/// A carriage return stays one: outside code the caller asks [`with_line_feeds`].
///
/// ~keep The fast converter decides on source text and decodes it when it writes it. With this
/// ~keep form every decision it makes on white space reads what the full converter reads in
/// ~keep its decoded text. No other reference is decoded, so the text is never decoded twice:
/// ~keep `&amp;#10;` stays the text `&#10;`. A reference without its `;` stays too: the fast
/// ~keep converter gives that page to the full one.
#[must_use]
pub fn with_white_space_references_decoded(source: &str) -> Cow<'_, str> {
    let mut buffer: Option<String> = None;
    let mut copied = 0;
    let mut from = 0;
    while let Some(found) = source[from..].find('&') {
        let amp = from + found;
        from = amp + 1;
        let Some((end, character, None)) = text::decode_character_reference(source, amp, text::ReferenceContext::Text)
        else {
            continue;
        };
        if !is_collapsible(character) || !source[..end].ends_with(';') {
            continue;
        }
        let target = buffer.get_or_insert_with(|| String::with_capacity(source.len()));
        target.push_str(&source[copied..amp]);
        target.push(character);
        copied = end;
        from = end;
    }
    let Some(mut decoded) = buffer else {
        return Cow::Borrowed(source);
    };
    decoded.push_str(&source[copied..]);
    Cow::Owned(decoded)
}

/// A run of collapsible white space at an edge of a text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    /// The run holds a segment break (a line feed).
    pub segment_break: bool,
}

impl Run {
    fn of(white_space: &str) -> Option<Self> {
        (!white_space.is_empty()).then(|| Self {
            segment_break: white_space.contains('\n'),
        })
    }

    /// Whether the run is written as one space. It is not when it holds a segment break that
    /// the zero-width space `before` or `after` the run removes.
    #[must_use]
    pub fn writes_space(self, before: Option<char>, after: Option<char>) -> bool {
        !(self.segment_break && segment_break_is_removed(before, after))
    }
}

/// The decoded text of a text node, classified by the collapsible white space at its edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextClass<'text> {
    /// Every character of the text is collapsible white space (or the text is empty).
    WhiteSpaceOnly(Option<Run>),
    /// The text has at least one character that is not collapsible white space.
    Text {
        /// The collapsible white space before the first character.
        prefix: Option<Run>,
        /// The text between the two runs, the first and the last character not collapsible.
        core: &'text str,
        /// The collapsible white space after the last character.
        suffix: Option<Run>,
    },
}

/// Classifies `decoded` by rule 1: only the five collapsible characters count as white space.
#[must_use]
pub fn classify_text(decoded: &str) -> TextClass<'_> {
    let core = decoded.trim_matches(is_collapsible);
    if core.is_empty() {
        return TextClass::WhiteSpaceOnly(Run::of(decoded));
    }
    let start = decoded.len() - decoded.trim_start_matches(is_collapsible).len();
    let end = start + core.len();
    TextClass::Text {
        prefix: Run::of(&decoded[..start]),
        core,
        suffix: Run::of(&decoded[end..]),
    }
}

/// Whether a reader sees nothing in `decoded`: every character is Unicode white space, a
/// no-break space included (`char::is_whitespace`). An empty text is blank.
///
/// ~keep A blank text before the first visible content of a block writes nothing, as a browser
/// ~keep shows none: `<p>&nbsp;<b>a</b></p>` is `**a**`. Between inline content and after the
/// ~keep last content of a block the caller writes it as it is (`<b>a</b>&nbsp;<b>b</b>`).
#[must_use]
pub fn is_blank(decoded: &str) -> bool {
    decoded.chars().all(char::is_whitespace)
}

/// The white space a text with visible characters writes at its edges: one space before it
/// when it starts with white space of any kind (a line end too), and after it a blank line when
/// it ends with two line ends, one space when it ends with a space, a tab or a Unicode space,
/// nothing when it ends with one line end (the caller decides that one by its siblings).
///
/// ~keep A no-break space at an edge is the one space of that edge: `<p>a<em>&nbsp;x</em></p>`
/// ~keep is `a *x*`, and `<p>a&nbsp;</p>` is `a` because the block drops the space at its end.
#[must_use]
pub fn edges(decoded: &str) -> (&'static str, &'static str) {
    let is_space = |character: char| matches!(character, ' ' | '\t') || text::is_unicode_space(character);
    let before = if decoded.starts_with(char::is_whitespace) {
        " "
    } else {
        ""
    };
    let after = if decoded.ends_with("\n\n") || decoded.ends_with("\r\n\r\n") {
        "\n\n"
    } else if decoded.ends_with(is_space) {
        " "
    } else {
        ""
    };
    (before, after)
}

/// The visible characters of a text as written: without the white space of any kind at its
/// edges, every run of spaces, tabs and Unicode spaces inside it one space, and no space after
/// a line end.
///
/// ~keep `<p>a<em>&nbsp;&nbsp;x</em></p>` is `a *x*`: the run is the one space of the edge.
#[must_use]
pub fn visible(decoded: &str) -> Cow<'_, str> {
    text::normalize_block_whitespace_cow(decoded.trim())
}

/// A text in a table cell as written: a cell has no line, so a line end is a space, and every
/// run of spaces, tabs and Unicode spaces is one space. The edges stay.
///
/// ~keep `<td>a<em>&nbsp;x</em></td>` is `a *x*` as in a paragraph; `<td>quick &nbsp; amet</td>`
/// ~keep is `quick amet`, as the fast converter writes it.
#[must_use]
pub fn cell_text(decoded: &str) -> Cow<'_, str> {
    text::normalize_cell_whitespace_cow(decoded)
}

/// Rule 2: a segment break is removed when the character `before` or `after` its run is a
/// zero-width space. `None` is a block boundary or the end of the text.
#[must_use]
pub fn segment_break_is_removed(before: Option<char>, after: Option<char>) -> bool {
    before == Some(ZERO_WIDTH_SPACE) || after == Some(ZERO_WIDTH_SPACE)
}

/// Rule 4: whether white space of the source owes one space at the end of `written`. Both
/// converters ask here before they write the space of a text or of an inline element.
///
/// ~keep White space collapses as in CSS: a space after a space is the same run, whatever
/// ~keep element boundary lies between the two, and the start of a line keeps none.
#[must_use]
pub fn space_is_owed(written: &str) -> bool {
    !written.ends_with([' ', '\n'])
}

/// Rule 4 in the full converter, which builds the body of an inline element in a scratch
/// buffer of its own: an empty scratch buffer is mid-line and owes the space, an empty block
/// buffer is a line start and owes none.
///
/// ~keep `<strong><em><br></em></strong>` lost its one space when an empty scratch buffer
/// ~keep counted as a line start (issue #504); `<p>A</p><p><i> </i>B</p>` opened its second
/// ~keep paragraph with a stray space when it did not (issue #501).
#[must_use]
pub fn space_is_owed_in(output: &String, ctx: &crate::converter::context::Context) -> bool {
    let is_block_buffer = std::ptr::from_ref::<String>(output) as usize == ctx.block_output_ptr;
    space_is_owed(output) && !(output.is_empty() && is_block_buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUN: Option<Run> = Some(Run { segment_break: false });
    const BREAK: Option<Run> = Some(Run { segment_break: true });

    #[test]
    fn a_text_of_unicode_white_space_is_blank() {
        for blank in ["", " \t\r\n", "\u{a0}", " \u{a0}\n", "\u{2003}\u{3000}"] {
            assert!(is_blank(blank), "{blank:?}");
        }
        for text in ["a", " a ", "\u{a0}a", "\u{200b}", "\u{feff}"] {
            assert!(!is_blank(text), "{text:?}");
        }
    }

    #[test]
    fn an_edge_of_a_text_is_one_space_for_white_space_of_any_kind() {
        assert_eq!(edges("one"), ("", ""));
        assert_eq!(edges(" one "), (" ", " "));
        assert_eq!(edges("\u{a0}one\u{a0}"), (" ", " "));
        assert_eq!(edges("\none\t"), (" ", " "));
        assert_eq!(edges("one\n"), ("", ""));
        assert_eq!(edges("one\n\n"), ("", "\n\n"));
        assert_eq!(edges("one\r\n\r\n"), ("", "\n\n"));
        assert_eq!(edges("one\u{c}"), ("", ""));
    }

    #[test]
    fn the_visible_part_of_a_text_collapses_a_no_break_space_with_the_spaces_around_it() {
        assert_eq!(visible(" one \u{a0} two\u{a0}"), "one two");
        assert_eq!(visible("one\n  two"), "one\ntwo");
        assert_eq!(visible("\u{a0}\u{a0}"), "");
    }

    #[test]
    fn a_cell_text_folds_a_line_end_to_a_space_and_keeps_its_edges() {
        assert_eq!(cell_text(" one\n\u{a0}two "), " one two ");
        assert_eq!(cell_text("quick \u{a0} amet"), "quick amet");
    }

    #[test]
    fn a_reference_to_collapsible_white_space_is_written_as_the_character() {
        for (source, decoded) in [
            ("one&#10;", "one\n"),
            ("one&#xA;two", "one\ntwo"),
            ("&#x0a;", "\n"),
            ("one&NewLine;", "one\n"),
            ("one&#13;", "one\r"),
            ("one&#13;&#10;two", "one\r\ntwo"),
            ("one&#32;&Tab;&#9;two", "one \t\ttwo"),
            ("a &amp; b&#10;", "a &amp; b\n"),
            ("é&#10;é", "é\né"),
        ] {
            assert_eq!(with_white_space_references_decoded(source), decoded, "{source:?}");
        }
    }

    #[test]
    fn a_text_with_no_reference_to_collapsible_white_space_is_not_changed() {
        for source in [
            "",
            "one\n",
            "one &",
            "one&amp;#10;",
            "one&nbsp;",
            "one&#12;",
            "one&#8203;",
            "one&NewLines;",
            "one&#10",
        ] {
            assert!(
                matches!(with_white_space_references_decoded(source), Cow::Borrowed(same) if same == source),
                "{source:?}"
            );
        }
    }

    #[test]
    fn a_carriage_return_is_a_line_feed() {
        assert_eq!(
            with_line_feeds(Cow::Borrowed("one\r\ntwo\rthree\n")),
            "one\ntwo\nthree\n"
        );
        assert!(matches!(
            with_line_feeds(Cow::Borrowed("one\n")),
            Cow::Borrowed("one\n")
        ));
    }

    #[test]
    fn the_four_characters_are_collapsible() {
        for character in [' ', '\t', '\n', '\r'] {
            assert!(is_collapsible(character), "{character:?}");
        }
    }

    #[test]
    fn a_form_feed_a_no_break_space_and_the_other_unicode_spaces_are_characters() {
        for character in [
            '\u{c}', '\u{a0}', '\u{2002}', '\u{2003}', '\u{3000}', '\u{200b}', '\u{feff}', 'a',
        ] {
            assert!(!is_collapsible(character), "{character:?}");
        }
    }

    #[test]
    fn a_text_of_the_four_characters_is_white_space_only() {
        assert_eq!(classify_text(" \t\r"), TextClass::WhiteSpaceOnly(RUN));
        assert_eq!(classify_text(" \n "), TextClass::WhiteSpaceOnly(BREAK));
        assert_eq!(classify_text(""), TextClass::WhiteSpaceOnly(None));
    }

    #[test]
    fn a_no_break_space_alone_is_text() {
        assert_eq!(
            classify_text("\u{a0}"),
            TextClass::Text {
                prefix: None,
                core: "\u{a0}",
                suffix: None
            }
        );
        assert_eq!(
            classify_text(" \u{a0} "),
            TextClass::Text {
                prefix: RUN,
                core: "\u{a0}",
                suffix: RUN
            }
        );
    }

    #[test]
    fn the_edges_of_a_text_are_its_runs() {
        assert_eq!(
            classify_text("\r\none"),
            TextClass::Text {
                prefix: BREAK,
                core: "one",
                suffix: None
            }
        );
        assert_eq!(
            classify_text("one\u{c}\n"),
            TextClass::Text {
                prefix: None,
                core: "one\u{c}",
                suffix: BREAK
            }
        );
        assert_eq!(
            classify_text("one two\t"),
            TextClass::Text {
                prefix: None,
                core: "one two",
                suffix: RUN
            }
        );
    }

    #[test]
    fn a_segment_break_beside_a_zero_width_space_is_removed_on_either_side() {
        assert!(segment_break_is_removed(Some('\u{200b}'), Some('G')));
        assert!(segment_break_is_removed(Some('s'), Some('\u{200b}')));
        assert!(segment_break_is_removed(Some('\u{200b}'), None));
        assert!(segment_break_is_removed(None, Some('\u{200b}')));
        assert!(!segment_break_is_removed(Some('s'), Some('G')));
        assert!(!segment_break_is_removed(None, None));
        assert!(!segment_break_is_removed(Some('\u{a0}'), Some('\u{feff}')));
    }

    #[test]
    fn a_run_writes_one_space_unless_its_segment_break_is_removed() {
        let run = Run { segment_break: false };
        assert!(run.writes_space(Some('\u{200b}'), Some('\u{200b}')));
        let with_break = Run { segment_break: true };
        assert!(with_break.writes_space(Some('s'), Some('G')));
        assert!(!with_break.writes_space(Some('\u{200b}'), Some('G')));
        assert!(!with_break.writes_space(Some('s'), Some('\u{200b}')));
    }

    #[test]
    fn a_space_is_owed_except_after_a_space_or_at_a_line_start() {
        assert!(space_is_owed("one"));
        assert!(space_is_owed("**one**"));
        assert!(space_is_owed("one\u{a0}"));
        assert!(!space_is_owed("one "));
        assert!(!space_is_owed("one\n"));
    }
}
