//! The code blocks of finished Markdown.
//!
//! The passes that clean the finished document, and the containers that trim the Markdown they hold,
//! ask this module which lines are code. A line of code keeps its white space.

use crate::options::CodeBlockStyle;

/// Follows the code blocks of finished Markdown, one line at a time.
pub enum CodeScan<'a> {
    /// The style writes fenced code blocks.
    Fenced(FenceScan),
    /// The style writes indented code blocks.
    Indented(IndentedScan<'a>),
}

/// Follows the indented code blocks of finished Markdown, one line at a time.
///
/// ~keep The lines are read when the first answer is asked for. A block quote asks only about a
/// ~keep line of white space, so most quotes read nothing. That saves a constant factor: a quote
/// ~keep reads only the lines of its own level (see below), never the square of the depth.
///
/// ~keep With [`QuoteLines::Kept`] the read starts after the last line of a quote at the left
/// ~keep margin and stops before the next one: such a line closes every list item and every
/// ~keep paragraph, so the lines on its other side change no answer. Each quote reads the lines
/// ~keep of its own level, and no line of a quote inside it.
pub struct IndentedScan<'a> {
    output: &'a str,
    quotes: QuoteLines,
    /// The index of the first line that was read, and whether each line from there is code.
    lines: Option<(usize, Vec<bool>)>,
    /// The index of the next line, and where it starts in `output`.
    next: (usize, usize),
    /// The index and the start of the line that a read starts at.
    first: (usize, usize),
}

impl IndentedScan<'_> {
    /// Go past the next line. Gives its index, and whether it is the line of a quote that is
    /// not read.
    fn advance(&mut self) -> (usize, bool) {
        let (index, start) = self.next;
        let rest = self.output.get(start..).unwrap_or_default();
        let length = rest.find('\n').unwrap_or(rest.len());
        self.next = (index + 1, start + length + 1);
        let is_kept_quote_line = self.quotes == QuoteLines::Kept && rest.starts_with('>');
        if is_kept_quote_line {
            self.first = self.next;
            self.lines = None;
        }
        (index, is_kept_quote_line)
    }

    /// Whether the next line is a line of code.
    fn is_code(&mut self) -> bool {
        let start = self.next.1;
        let (index, is_kept_quote_line) = self.advance();
        if is_kept_quote_line {
            return true;
        }
        let (output, quotes, first) = (self.output, self.quotes, self.first);
        let (first_index, lines) = self.lines.get_or_insert_with(|| {
            let rest = output.get(first.1..).unwrap_or_default();
            let end = match quotes {
                QuoteLines::Read => rest.len(),
                QuoteLines::Kept => output
                    .get(start..)
                    .and_then(|after| after.find("\n>"))
                    .map_or(rest.len(), |length| start - first.1 + length),
            };
            (first.0, indented_code_lines(&rest[..end], quotes))
        });
        lines.get(index - *first_index).copied().unwrap_or(false)
    }
}

/// How a scan for indented code reads the lines of a block quote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuoteLines {
    /// The scan reads them. The text is a finished document, or Markdown whose block quotes did
    /// not trim their content.
    Read,
    /// The scan does not read them. The text is the content of a container, and a block quote
    /// in it trimmed its own content: its lines stay as they are.
    ///
    /// ~keep A read of the lines of every nested quote, at each level of nested quotes, costs
    /// ~keep the square of the depth: a line has one marker for each level.
    Kept,
}

impl<'a> CodeScan<'a> {
    /// A scan of `output` for the code blocks that `style` writes.
    ///
    /// ~keep Only the configured style opens a block. A line of indented code that looks like a
    /// ~keep fence opens nothing, and neither does a line of tildes in a document whose fences are
    /// ~keep backticks.
    pub const fn new(style: CodeBlockStyle, output: &'a str) -> Self {
        Self::with_quote_lines(style, output, QuoteLines::Read)
    }

    /// A scan of `content`, the Markdown that a block quote holds, for the code blocks that
    /// `style` writes. The lines of a quote in it are not read (see [`QuoteLines::Kept`]).
    pub const fn of_quote_content(style: CodeBlockStyle, content: &'a str) -> Self {
        Self::with_quote_lines(style, content, QuoteLines::Kept)
    }

    const fn with_quote_lines(style: CodeBlockStyle, output: &'a str, quotes: QuoteLines) -> Self {
        match style {
            CodeBlockStyle::Indented => Self::Indented(IndentedScan {
                output,
                quotes,
                lines: None,
                next: (0, 0),
                first: (0, 0),
            }),
            CodeBlockStyle::Backticks => Self::Fenced(FenceScan {
                fence: b'`',
                quotes,
                open: None,
            }),
            CodeBlockStyle::Tildes => Self::Fenced(FenceScan {
                fence: b'~',
                quotes,
                open: None,
            }),
        }
    }

    /// Whether `line`, the next line of the output, is a line of code.
    pub fn is_code(&mut self, line: &str) -> bool {
        match self {
            Self::Fenced(fences) => fences.is_code(line),
            Self::Indented(scan) => scan.is_code(),
        }
    }

    /// Go past `line`, the next line of the output, when the caller does not need the answer.
    pub fn pass(&mut self, line: &str) {
        match self {
            Self::Fenced(fences) => {
                fences.is_code(line);
            }
            Self::Indented(scan) => {
                scan.advance();
            }
        }
    }
}

/// Follows the fenced code blocks of finished Markdown, one line at a time.
pub struct FenceScan {
    /// The fence character of the configured style.
    fence: u8,
    quotes: QuoteLines,
    open: Option<(u8, usize)>,
}

impl FenceScan {
    /// Whether `line` is a line of code: a line between the two fences of a code block.
    ///
    /// ~keep A fence is longer than every run of its character in the code, so no line of code
    /// ~keep closes it. A block quote and a list item put their prefix before every line.
    ///
    /// ~keep With [`QuoteLines::Kept`] a line of a quote at the left margin is not read while no
    /// ~keep block is open: that quote closed every block it opened. A read of its markers, at
    /// ~keep each level of nested quotes, costs the square of the depth.
    fn is_code(&mut self, line: &str) -> bool {
        use crate::converter::utility::escaping::code_fence;

        let Some((marker, length)) = self.open else {
            if self.quotes == QuoteLines::Kept && line.starts_with('>') {
                return false;
            }
            self.open = code_fence(after_container_markers(line)).filter(|(marker, _)| *marker == self.fence);
            return false;
        };
        let rest = line.trim_start_matches([' ', '\t', '>']);
        let closes = code_fence(rest).is_some_and(|(fence, run)| fence == marker && run >= length)
            && rest.trim_start_matches(char::from(marker)).trim().is_empty();
        if closes {
            self.open = None;
        }
        !closes
    }
}

/// `line` without the block quote and list item markers that can precede an opening fence.
fn after_container_markers(mut line: &str) -> &str {
    loop {
        line = line.trim_start_matches([' ', '\t', '>']);
        let Some(marker) = list_marker_length(line) else {
            return line;
        };
        line = &line[marker..];
    }
}

/// The length of the list item marker that `rest` starts with: a bullet, or a number of at most
/// nine digits with its delimiter, before a space, a tab or the end of the line.
fn list_marker_length(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let digits = bytes.iter().take_while(|byte| byte.is_ascii_digit()).count();
    let length = match bytes.first()? {
        b'-' | b'*' | b'+' => 1,
        _ if (1..=9).contains(&digits) && matches!(bytes.get(digits), Some(b'.' | b')')) => digits + 1,
        _ => return None,
    };
    matches!(bytes.get(length), None | Some(b' ' | b'\t')).then_some(length)
}

/// A block whose marker precedes its lines: a block quote, or a list item with the number of
/// columns its content is indented by.
enum Container {
    Quote,
    Item(usize),
}

/// A position in a line that counts columns: a tab reaches the next multiple of four.
struct LineCursor<'a> {
    rest: &'a str,
    column: usize,
    /// Columns of a tab that `skip` passed but did not use.
    spare: usize,
}

impl LineCursor<'_> {
    /// The columns of white space at the cursor.
    fn indent(&self) -> usize {
        let mut column = self.column;
        for byte in self.rest.bytes() {
            match byte {
                b' ' => column += 1,
                b'\t' => column += 4 - column % 4,
                _ => break,
            }
        }
        self.spare + column - self.column
    }

    /// Move past `columns` columns of white space.
    fn skip(&mut self, mut columns: usize) {
        let used = columns.min(self.spare);
        self.spare -= used;
        columns -= used;
        while columns > 0 {
            let width = match self.rest.as_bytes().first() {
                Some(b' ') => 1,
                Some(b'\t') => 4 - self.column % 4,
                _ => return,
            };
            self.advance(1);
            self.column += width - 1;
            if width > columns {
                self.spare = width - columns;
                return;
            }
            columns -= width;
        }
    }

    /// Move past `bytes` bytes that are one column each.
    fn advance(&mut self, bytes: usize) {
        self.rest = &self.rest[bytes..];
        self.column += bytes;
    }

    /// Move past the `>` of a block quote and the one space that can follow it.
    fn skip_quote_marker(&mut self) {
        self.advance(1);
        if self.indent() > 0 {
            self.skip(1);
        }
    }

    fn is_blank(&self) -> bool {
        self.rest.trim_matches([' ', '\t']).is_empty()
    }

    /// Move past the markers of the `containers` that the line continues, the outer ones first.
    /// Gives the number of containers that the line continues.
    fn enter(&mut self, containers: &[Container]) -> usize {
        let mut depth = 0;
        for container in containers {
            match *container {
                Container::Quote => {
                    let indent = self.indent();
                    if indent > 3 || !self.rest.trim_start_matches([' ', '\t']).starts_with('>') {
                        break;
                    }
                    self.skip(indent);
                    self.skip_quote_marker();
                }
                Container::Item(content) => {
                    if self.indent() >= content {
                        self.skip(content);
                    } else if !self.is_blank() {
                        break;
                    }
                }
            }
            depth += 1;
        }
        depth
    }
}

/// Whether each line of `output` is a line of an indented code block (CommonMark 4.4).
///
/// ~keep A line is code when it is indented by four columns or more past the block quotes and the
/// ~keep list items it is in, and it does not continue a paragraph. The blank lines between two
/// ~keep lines of one code block belong to the block. The indentation alone does not say this:
/// ~keep the third level of a list is indented by four columns too.
///
/// ~keep With [`QuoteLines::Kept`] a line of a block quote is not read past its indentation: the
/// ~keep answer for it is `true`, and it opens nothing for the lines after it. The work for such
/// ~keep a line does not grow with the number of quote markers on it.
fn indented_code_lines(output: &str, quotes: QuoteLines) -> Vec<bool> {
    let mut code = Vec::new();
    let mut containers: Vec<Container> = Vec::new();
    let mut in_paragraph = false;
    let mut last_code: Option<usize> = None;
    for (index, line) in output.split('\n').enumerate() {
        code.push(false);
        let mut cursor = LineCursor {
            rest: line,
            column: 0,
            spare: 0,
        };
        let depth = cursor.enter(&containers);
        if cursor.is_blank() {
            in_paragraph = false;
            continue;
        }
        if depth < containers.len() {
            containers.truncate(depth);
            last_code = None;
        }
        let mut is_kept_quote_line = false;
        let is_code = loop {
            let indent = cursor.indent();
            if indent >= 4 {
                break !in_paragraph;
            }
            cursor.skip(indent);
            if cursor.rest.starts_with('>') {
                if quotes == QuoteLines::Kept {
                    is_kept_quote_line = true;
                    in_paragraph = false;
                    break false;
                }
                cursor.skip_quote_marker();
                containers.push(Container::Quote);
            } else if let Some(marker) = list_marker_length(cursor.rest) {
                cursor.advance(marker);
                // ~keep The content of an item starts after one to four spaces. With more, or with
                // ~keep nothing after the marker, it starts after one.
                let padding = match cursor.indent() {
                    spaces @ 1..=4 if !cursor.is_blank() => spaces,
                    _ => 1,
                };
                cursor.skip(padding);
                containers.push(Container::Item(indent + marker + padding));
            } else {
                in_paragraph = true;
                break false;
            }
            in_paragraph = false;
            last_code = None;
            if cursor.is_blank() {
                break false;
            }
        };
        if is_code {
            code[index] = true;
            if let Some(last) = last_code {
                code[last + 1..index].fill(true);
            }
            last_code = Some(index);
        } else {
            last_code = None;
            code[index] = is_kept_quote_line;
        }
    }
    code
}

/// The part of `content`, the Markdown inside a block quote, without the white space around it.
///
/// ~keep With the indented style the indentation of a first line of code stays, and so does the
/// ~keep line end of a last line of code: without its four columns the line is running text.
pub fn quote_content_range(content: &str, style: CodeBlockStyle) -> std::ops::Range<usize> {
    let start = quote_content_start(content, style);
    start..quote_content_end(content, style, QuoteLines::Read).max(start)
}

/// The same part of `content` when each block quote in it trimmed its own content, as the
/// containers of the full converter do (see [`QuoteLines::Kept`]).
pub fn trimmed_quotes_content_range(content: &str, style: CodeBlockStyle) -> std::ops::Range<usize> {
    let start = quote_content_start(content, style);
    start..quote_content_end(content, style, QuoteLines::Kept).max(start)
}

/// Where the content of a block quote starts: at its first character that is not white space, or
/// at the start of that line when the line is indented code.
///
/// ~keep No line before the first one opens a list item or a paragraph, so the line alone says
/// ~keep whether it is code. The lines after it are not read.
pub fn quote_content_start(content: &str, style: CodeBlockStyle) -> usize {
    let start = content.len() - content.trim_start().len();
    if style != CodeBlockStyle::Indented {
        return start;
    }
    let first_line = content[..start].rfind('\n').map_or(0, |index| index + 1);
    let line_end = content[start..].find('\n').map_or(content.len(), |index| start + index);
    if indented_code_lines(&content[first_line..line_end], QuoteLines::Read) == [true] {
        first_line
    } else {
        start
    }
}

/// Where the content of a block quote ends: after its last character that is not white space, or
/// at the end of that line when the line is indented code.
///
/// ~keep Only white space at the end of the last line depends on the answer. Without it the lines
/// ~keep are not read.
///
/// ~keep With [`QuoteLines::Kept`] the last line alone answers first. A line that is not code
/// ~keep alone is not code after other lines. A line of a nested quote that is code alone is
/// ~keep code: that quote trimmed its own content. So a quote in a quote reads one line, and only
/// ~keep the quote that holds the code block reads the lines before it.
fn quote_content_end(content: &str, style: CodeBlockStyle, quotes: QuoteLines) -> usize {
    let end = content.trim_end().len();
    if style != CodeBlockStyle::Indented || end == 0 {
        return end;
    }
    let last_line = content[end..].find('\n').map_or(content.len(), |index| end + index);
    if last_line == end {
        return end;
    }
    let start = content.len() - content.trim_start().len();
    let mut first_line = content[..start].rfind('\n').map_or(0, |index| index + 1);
    if quotes == QuoteLines::Kept {
        let line_start = content[..end].rfind('\n').map_or(0, |index| index + 1);
        let line = &content[line_start..last_line];
        if indented_code_lines(line, QuoteLines::Read) != [true] {
            return end;
        }
        // ~keep A line of a quote at the left margin closes every list item and every paragraph:
        // ~keep the read starts after the last one.
        if let Some(quote) = content[..line_start].rfind("\n>") {
            first_line = content[quote + 1..line_start]
                .find('\n')
                .map_or(line_start, |length| quote + length + 2);
        }
    }
    if indented_code_lines(&content[first_line..last_line], quotes).last() == Some(&true) {
        last_line
    } else {
        end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One character for each line of `markdown`: `1` for a line of indented code, `0` for another.
    fn code(markdown: &str) -> String {
        indented_code_lines(markdown, QuoteLines::Read)
            .into_iter()
            .map(|is_code| if is_code { '1' } else { '0' })
            .collect()
    }

    #[test]
    fn should_measure_indented_code_from_the_content_column_of_a_list_item() {
        let cases = [
            // A number without a delimiter is no item marker: the last line is code at the top level.
            ("7  x\n\n     c", "001"),
            // An item whose marker has three spaces after it has its content at column four.
            ("-   a\n\n      b", "000"),
            // A marker with only spaces after it has its content one column after the marker.
            ("-  \n      b", "01"),
            // The columns before a marker belong to the content column of its item.
            (" - a\n\n      b", "000"),
            // A line at the left margin closes the item, so five columns after it are code.
            ("- a\n\nb\n\n     c", "00001"),
        ];
        for (markdown, expected) in cases {
            assert_eq!(code(markdown), expected, "{markdown:?}");
        }
    }

    #[test]
    fn should_measure_indented_code_after_the_marker_of_a_block_quote() {
        let cases = [
            // One space after the marker belongs to the marker.
            (">    a", "0"),
            // A marker after three columns continues the quote and its code block.
            (">     a\n>\n   >     b", "111"),
            // A tab after the marker gives one column to the marker and two to the content
            // (CommonMark example 6).
            (">\t foo", "0"),
            (">\t\tfoo", "1"),
        ];
        for (markdown, expected) in cases {
            assert_eq!(code(markdown), expected, "{markdown:?}");
        }
    }

    #[test]
    fn should_count_a_tab_to_the_next_multiple_of_four_columns() {
        // The tab after two spaces is two columns wide, less than the content column of the item.
        assert_eq!(code("1234. a\n\n  \tb"), "001");

        let mut cursor = LineCursor {
            rest: "  \t\tx",
            column: 0,
            spare: 0,
        };
        assert_eq!(cursor.indent(), 8);
        cursor.skip(2);
        cursor.skip(1);
        // One column of the first tab is left, and the second tab is four columns wide.
        assert_eq!(cursor.indent(), 5);
        cursor.skip(1);
        assert_eq!(cursor.indent(), 4);
    }

    #[test]
    fn should_keep_the_white_space_of_code_at_the_edges_of_the_content_of_a_container() {
        let indented = CodeBlockStyle::Indented;
        assert_eq!(quote_content_range("\n    a  \n", indented), 1..8);
        assert_eq!(quote_content_range("\n  a  \n", indented), 3..4);
        assert_eq!(quote_content_range("  \n", indented), 3..3);
        assert_eq!(quote_content_range("    a\n", CodeBlockStyle::Backticks), 4..5);
    }

    /// The range that a read of every line of `content` gives.
    fn range_from_every_line(content: &str) -> std::ops::Range<usize> {
        let start = content.len() - content.trim_start().len();
        let end = content.trim_end().len();
        if start >= end {
            return start..end.max(start);
        }
        let first_line = content[..start].rfind('\n').map_or(0, |index| index + 1);
        let last_line = content[end..].find('\n').map_or(content.len(), |index| end + index);
        let code = indented_code_lines(&content[first_line..last_line], QuoteLines::Read);
        let start = if code.first() == Some(&true) { first_line } else { start };
        let end = if code.last() == Some(&true) { last_line } else { end };
        start..end
    }

    #[test]
    fn should_trim_the_content_of_a_container_as_a_read_of_every_line_does() {
        let cases = [
            ("", 0..0),
            (" \n\t\n", 4..4),
            ("\n    a\n\n    b  \n\n", 1..15),
            ("\n    a\n\nb  \n\n", 1..9),
            ("  a\n\n    b  \n", 2..12),
            ("  a\n    b  \n", 2..9),
            ("- a\n\n      b \t\n", 0..14),
            ("- a\n\n    b  \n", 0..10),
            (">     a  \n> b\n", 0..13),
            ("> b\n>\n>     a  \n", 0..15),
            ("\n\n\ta\n\tb\t\n", 2..8),
            ("    a  ", 0..7),
            ("a  ", 0..1),
        ];
        for (content, expected) in cases {
            let range = quote_content_range(content, CodeBlockStyle::Indented);
            assert_eq!(range, expected, "{content:?}");
            // No quote line of these cases depends on the lines of a quote before it.
            assert_eq!(
                trimmed_quotes_content_range(content, CodeBlockStyle::Indented),
                expected,
                "{content:?}"
            );
            assert_eq!(range, range_from_every_line(content), "{content:?}");
            assert_eq!(
                quote_content_start(content, CodeBlockStyle::Indented),
                range.start,
                "{content:?}"
            );
        }
        // A fenced style has no indented code: the white space at both ends goes.
        assert_eq!(quote_content_range("\n    a  \n", CodeBlockStyle::Tildes), 5..6);
        assert_eq!(quote_content_start("\n    a  \n", CodeBlockStyle::Backticks), 5);
    }

    #[test]
    fn should_not_read_the_lines_of_a_quote_that_trimmed_its_own_content() {
        let kept = |markdown: &str| -> String {
            indented_code_lines(markdown, QuoteLines::Kept)
                .into_iter()
                .map(|is_kept| if is_kept { '1' } else { '0' })
                .collect()
        };
        // The lines of a quote stay as they are. The lines after them are read as in a document.
        assert_eq!(kept("> > a\n\n    b\n      \n    c\n> d"), "101111");
        // A quote in a list item is a quote, and the item goes on after it.
        assert_eq!(kept("- a\n  > b\n\n    c\n\n      d"), "010001");
        // A line of white space between two quotes is not their code.
        assert_eq!(kept(">     a\n \n>     b"), "101");
        assert_eq!(code(">     a\n \n>     b"), "111");

        let indented = CodeBlockStyle::Indented;
        // The last line of a quote in the content keeps the line end that the quote kept.
        assert_eq!(trimmed_quotes_content_range("a\n\n> >     b  \n\n", indented), 0..14);
        // Text that starts with the marker is not the line of a quote that kept code.
        assert_eq!(trimmed_quotes_content_range("a\n> b  \n", indented), 0..5);
        // A line after a quote is read with the lines of its own level.
        assert_eq!(trimmed_quotes_content_range("> a\n\n    b  \n", indented), 0..12);
        assert_eq!(trimmed_quotes_content_range("- a\n  > b\n\n    c  \n", indented), 0..16);
        assert_eq!(trimmed_quotes_content_range("- a\n> b\n\n    c  \n", indented), 0..16);

        // A read starts after the last line of a quote at the left margin and stops before the
        // next one. The answers are those of one read of every line.
        let content = "    a\n \n    b\n> c\n\n    d\n      \n    e\n> f\n  ";
        assert_eq!(kept(content), "1111011110");
        let mut scan = CodeScan::of_quote_content(indented, content);
        let mut answers = String::new();
        for line in content.split('\n') {
            if line.trim().is_empty() {
                answers.push(if scan.is_code(line) { '1' } else { '0' });
            } else {
                scan.pass(line);
                answers.push('-');
            }
        }
        assert_eq!(answers, "-1--0-1--0");
        let mut scan = CodeScan::of_quote_content(indented, content);
        let every: String = content
            .split('\n')
            .map(|line| if scan.is_code(line) { '1' } else { '0' })
            .collect();
        assert_eq!(every, kept(content));
    }

    #[test]
    fn should_answer_for_the_next_line_after_lines_that_were_passed() {
        let markdown = "a\n\n    b\n      \n    c\nd";
        let mut scan = CodeScan::new(CodeBlockStyle::Indented, markdown);
        let mut answers = String::new();
        for (index, line) in markdown.split('\n').enumerate() {
            if index < 2 || index == 4 {
                scan.pass(line);
                answers.push('-');
            } else {
                answers.push(if scan.is_code(line) { '1' } else { '0' });
            }
        }
        assert_eq!(answers, "--11-0");
        // A line past the end of the output is not code.
        assert!(!scan.is_code(""));

        let fenced = "```\n  \n```\n  ";
        let mut scan = CodeScan::new(CodeBlockStyle::Backticks, fenced);
        let mut answers = String::new();
        for (index, line) in fenced.split('\n').enumerate() {
            if index % 2 == 0 {
                scan.pass(line);
                answers.push('-');
            } else {
                answers.push(if scan.is_code(line) { '1' } else { '0' });
            }
        }
        // The fences were passed, and the scan still knows that the block is closed.
        assert_eq!(answers, "-1-0");
    }

    #[test]
    fn should_not_read_the_fences_of_a_quote_that_closed_its_own_code_blocks() {
        let answers = |mut scan: CodeScan<'_>, markdown: &str| -> String {
            markdown
                .split('\n')
                .map(|line| if scan.is_code(line) { '1' } else { '0' })
                .collect()
        };
        for (style, fence) in [(CodeBlockStyle::Backticks, "```"), (CodeBlockStyle::Tildes, "~~~")] {
            // The code block of a quote, then a code block of this level with a line of spaces.
            let content = format!("> {fence}\n> a\n> {fence}\n  \n{fence}\n  \n> b\n{fence}\n  ");
            assert_eq!(
                answers(CodeScan::of_quote_content(style, &content), &content),
                "000001100"
            );
            // A read of every line gives the same answers after the quote, and reads the quote.
            assert_eq!(answers(CodeScan::new(style, &content), &content), "010001100");
            // A quote line that is not at the left margin is read: it can be code of a list item.
            let item = format!("- a\n  > {fence}\n  >   \n  > {fence}\n  ");
            assert_eq!(answers(CodeScan::of_quote_content(style, &item), &item), "00100");
        }
    }
}
