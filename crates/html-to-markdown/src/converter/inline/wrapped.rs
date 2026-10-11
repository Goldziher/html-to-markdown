//! Shared emission for inline elements that wrap their trimmed content in a delimiter pair.
//!
//! Every such element has the same two edge cases, and both were originally fixed for
//! `<strong>`/`<em>` alone (issues #483 and #481) while the other handlers kept their own
//! byte-for-byte copy of the naive emission and kept the bugs:
//!
//! - A delimiter built from a repeated character (`*`, `**`, `~~`, `==`) that opens
//!   immediately after the matching close of a preceding sibling forms ONE longer
//!   delimiter run on reparse, not two spans.
//! - A body that is non-empty but entirely whitespace stands for a word separator, and a
//!   handler that only tests `!trimmed.is_empty()` drops it, joining the words either side.

use tl::{NodeHandle, Parser};

type DomContext = crate::converter::DomContext;
type Context = crate::converter::Context;
type ConversionOptions = crate::options::ConversionOptions;

/// Where a wrapper is being emitted: its DOM node, the parser and DOM lookup context, and the
/// conversion `Context` of the buffer it writes into.
///
/// ~keep Bundled because every emitter passes exactly these four together; `ctx` is needed for
/// ~keep `block_output_ptr`, which tells a block's own buffer apart from an enclosing wrapper's
/// ~keep scratch buffer (issue #504).
#[derive(Clone, Copy)]
pub struct InlineSite<'p> {
    pub node_handle: &'p NodeHandle,
    pub parser: &'p Parser<'p>,
    pub dom_ctx: &'p DomContext,
    pub ctx: &'p Context,
    pub options: &'p ConversionOptions,
}

/// Tag names that render with the single `strong_em_symbol` italic delimiter.
///
/// `<var>` and `<dfn>` wrap their content in exactly the same one-character pair `<em>`/`<i>`
/// do, so a `<var>` opening right after an `<em>` closes forms one longer delimiter run just
/// as two `<em>`s would -- the corruption issue #483 describes does not care which tag
/// emitted each half of the run. ~keep
pub const EMPHASIS_SIBLING_TAGS: [&str; 4] = ["em", "i", "var", "dfn"];

/// Tag names that render with the doubled `strong_em_symbol` bold delimiter.
pub const STRONG_SIBLING_TAGS: [&str; 2] = ["strong", "b"];

/// The delimiter pair an inline element wraps its trimmed content in, and what it may merge
/// that pair with.
///
/// `open`/`close` are the exact strings emitted around the content. Pass `("", "")` for an
/// element that renders with no marker of its own, such as a `<strong>` nested inside another
/// `<strong>`.
///
/// `merge_symbol` is the single character `open` is built from, used to detect and merge a
/// `CommonMark` adjacency with the immediately preceding sibling's close marker (issue #483,
/// see [`merge_adjacent_emphasis`](crate::converter::merge_adjacent_emphasis)) instead of
/// opening a second, textually-adjacent delimiter run. Use `None` for a delimiter that cannot
/// form a run at all -- Djot's `{-`/`-}` and `{+`/`+}` pairs, `<mark>`'s HTML style, or an
/// arbitrary caller-configured `sub_symbol`. A `Some` whose character does not make up the
/// whole of `open` is treated the same way, so a caller cannot accidentally pop bytes that are
/// not a delimiter run.
///
/// `sibling_tag_names` lists the HTML tag names (e.g. `["strong", "b"]`) that make this
/// element's immediately preceding DOM sibling a genuine candidate for that merge. Buffer
/// content alone is ambiguous: ordinary prose can coincidentally end in a literal `*`/`**`
/// (CommonMark spec example 442, `<p>*<em>foo</em></p>` -> `**foo*`, NOT a merge into `*foo*`)
/// that is indistinguishable, byte-for-byte, from a just-emitted close marker. Requiring the
/// DOM to actually show a matching sibling element resolves the ambiguity in favor of "no
/// merge" whenever the preceding content is plain text. ~keep
#[derive(Clone, Copy)]
pub struct InlineDelimiters<'a> {
    pub open: &'a str,
    pub close: &'a str,
    pub merge_symbol: Option<char>,
    pub sibling_tag_names: &'a [&'a str],
}

/// ~keep Wrap every non-empty Markdown block in its own delimiter pair so a blank line cannot
/// split one inline delimiter run into literal markers (issue #724).
pub fn wrap_block_runs(content: &str, open: &str, close: &str) -> String {
    let (prefix, suffix, trimmed) = crate::converter::chomp_inline(content);
    let mut wrapped = String::with_capacity(content.len() + open.len() + close.len());
    wrapped.push_str(prefix);

    for (index, block) in trimmed.split("\n\n").enumerate() {
        if index > 0 {
            wrapped.push_str("\n\n");
        }
        if block.trim_matches([' ', '\t']).is_empty() {
            wrapped.push_str(block);
        } else {
            let body_start = markdown_block_prefix_len(block);
            let body_end = block.trim_end_matches([' ', '\t']).len();
            if body_end <= body_start {
                wrapped.push_str(block);
                continue;
            }
            wrapped.push_str(&block[..body_start]);
            wrapped.push_str(open);
            wrapped.push_str(&block[body_start..body_end]);
            wrapped.push_str(close);
            wrapped.push_str(&block[body_end..]);
        }
    }

    wrapped.push_str(suffix);
    wrapped
}

/// ~keep General inline handlers split only plain paragraph-like runs. Structured Markdown
/// remains on its established path because moving a delimiter across nested list/quote syntax
/// changes the block tree; legend opts into that explicitly for its pinned list behavior.
pub fn block_runs_are_plain(content: &str) -> bool {
    block_runs_are_single_line(content)
        && content.split("\n\n").all(|block| {
            let indentation = block.bytes().take_while(|byte| matches!(byte, b' ' | b'\t')).count();
            markdown_block_prefix_len(block) == indentation
        })
}

/// ~keep Multi-line structured blocks need tree-aware delimiter placement; the shared wrapper
/// safely handles single-line blocks while preserving their Markdown prefixes.
pub fn block_runs_are_single_line(content: &str) -> bool {
    content.split("\n\n").all(|block| !block.contains('\n'))
}

/// ~keep A block quote that is a task item's first content keeps its opener outside inline
/// delimiters, so the checkbox line cannot turn the opener into literal text (#643). Other
/// block-like children retain their established inline-wrapper behavior.
pub fn emit_first_block_wrapped(
    output: &mut String,
    content: &str,
    open: &str,
    close: &str,
    ctx: &Context,
    parser: &Parser<'_>,
) -> bool {
    if open.is_empty()
        || !ctx
            .first_writer
            .as_ref()
            .is_some_and(|first_writer| first_writer.starts_with_blockquote(parser))
        || !block_runs_are_single_line(content.trim_end_matches('\n'))
        || !crate::converter::utility::escaping::opens_block(content.trim_start())
    {
        return false;
    }

    output.push_str(&wrap_block_runs(content, open, close));
    true
}

// ~keep Markdown block syntax stays outside the inline delimiters: `* **x**` remains a list,
// ~keep whereas `*** x**` is an ambiguous delimiter run rather than a bold list item.
fn markdown_block_prefix_len(block: &str) -> usize {
    let bytes = block.as_bytes();
    let mut position = bytes.iter().take_while(|byte| matches!(byte, b' ' | b'\t')).count();

    while bytes.get(position..position + 2) == Some(b"> ") {
        position += 2;
    }

    if matches!(bytes.get(position), Some(b'-' | b'+' | b'*')) && matches!(bytes.get(position + 1), Some(b' ' | b'\t'))
    {
        return position + 2;
    }

    let digits_start = position;
    while bytes.get(position).is_some_and(u8::is_ascii_digit) {
        position += 1;
    }
    if position > digits_start
        && matches!(bytes.get(position), Some(b'.' | b')'))
        && matches!(bytes.get(position + 1), Some(b' ' | b'\t'))
    {
        return position + 2;
    }

    let heading_start = position;
    while bytes.get(position) == Some(&b'#') && position - heading_start < 6 {
        position += 1;
    }
    if position > heading_start && bytes.get(position) == Some(&b' ') {
        return position + 1;
    }

    digits_start
}

fn move_leading_hard_breaks<'a>(
    output: &mut String,
    content: &'a str,
    ctx: &Context,
    options: &ConversionOptions,
) -> &'a str {
    // ~keep Table-cell block boundaries are normalized by the cell renderer; hoisting one from
    // ~keep an inline wrapper turns the folded separator into literal hard-break syntax (#645).
    if ctx.in_table_cell {
        return content;
    }

    let mut after_breaks = content;
    loop {
        let candidate = after_breaks.trim_start_matches([' ', '\t']);
        let Some(rest) = ["\\\n", "\n"]
            .into_iter()
            .find_map(|marker| candidate.strip_prefix(marker))
        else {
            break;
        };
        after_breaks = rest;
    }
    // ~keep A break-only wrapper separates adjacent words instead of authoring a hard break (#502).
    if after_breaks.trim().is_empty() {
        return content;
    }

    // ~keep The bare newline is an internal sentinel from a break at the start of a detached
    // ~keep wrapper. It becomes a hard break only when real paragraph content precedes the
    // ~keep wrapper; hoisting it into the paragraph's own empty block buffer recreates the
    // ~keep leading break that issue #572 drops before unwrapped text.
    let drop_at_paragraph_start = ctx.in_paragraph
        && output.len() == ctx.block_content_start
        && std::ptr::from_ref::<String>(output) as usize == ctx.block_output_ptr;
    let mut remaining = content;
    let mut moved = false;
    loop {
        let candidate = remaining.trim_start_matches([' ', '\t']);
        let Some((marker, rest)) = ["\\\n", "\n"]
            .into_iter()
            .find_map(|marker| candidate.strip_prefix(marker).map(|rest| (marker, rest)))
        else {
            break;
        };
        if !drop_at_paragraph_start {
            let starts_on_empty_line = output.ends_with('\n');
            crate::converter::list::utils::indent_list_item_line_start(output, ctx, options);
            let marker = if marker == "\n" {
                crate::converter::main_helpers::hard_break_marker(options)
            } else {
                marker
            };
            output.push_str(if marker == "  \n" && starts_on_empty_line {
                "\\\n"
            } else {
                marker
            });
        }
        remaining = rest;
        moved = true;
    }
    if moved && !drop_at_paragraph_start {
        crate::converter::list::utils::indent_list_item_line_start(output, ctx, options);
    }
    if moved {
        remaining = remaining.trim_start_matches([' ', '\t']);
    }
    remaining
}

/// Emit `content` wrapped in `delimiters`, handling delimiter-run adjacency with a preceding
/// sibling and a body that is non-empty but entirely whitespace.
pub fn emit_wrapped_inline(
    output: &mut String,
    content: &str,
    delimiters: &InlineDelimiters<'_>,
    site: InlineSite<'_>,
) {
    let InlineSite {
        node_handle,
        parser,
        dom_ctx,
        ctx,
        options,
    } = site;
    let InlineDelimiters {
        open,
        close,
        merge_symbol,
        sibling_tag_names,
    } = *delimiters;
    use crate::converter::utility::siblings::get_previous_sibling_tag;
    use crate::converter::{append_inline_suffix, chomp_inline, merge_adjacent_emphasis};

    let content = move_leading_hard_breaks(output, content, ctx, options);
    let (prefix, suffix, trimmed) = chomp_inline(content);
    if content.trim().is_empty() {
        if content.is_empty() {
            return;
        }
        // ~keep issue #481: a whitespace-only body (e.g. `<i> </i>`) is one space at most, and
        // ~keep none where the rule of running text owes none (after a space, at a line start).
        if crate::converter::utility::white_space::space_is_owed_in(output, ctx) {
            output.push_str(prefix);
        }
        append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
        return;
    }

    crate::converter::utility::content::push_inline_prefix(output, prefix, options, ctx);
    let sibling_is_matching_tag =
        get_previous_sibling_tag(node_handle, parser, dom_ctx).is_some_and(|name| sibling_tag_names.contains(&name));
    let merged = prefix.is_empty()
        && sibling_is_matching_tag
        && merge_symbol.is_some_and(|symbol| {
            open.chars().all(|c| c == symbol) && merge_adjacent_emphasis(output, symbol, open.chars().count())
        });
    if !merged {
        output.push_str(open);
    }
    output.push_str(trimmed);
    output.push_str(close);
    append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
}

/// Emit an already-rendered inline code `span`, merging it into an immediately preceding
/// sibling code span rather than letting the two backticks sit adjacent.
///
/// `CommonMark` reads a code span's closing backtick immediately followed by the next span's
/// opening backtick as interior text, not as two spans: `` `A` `` and `` `B` `` written
/// back-to-back reparse as ONE span whose content carries both literal backticks -- issue
/// #483's delimiter-run adjacency in its backtick form. There is no `CommonMark` spelling for
/// two genuinely adjacent code spans, so the faithful choice is the merged span: it preserves
/// every character of the content, which the corrupted reparse does not.
///
/// Only a span that rendered as a plain single-backtick pair can merge. One that needed a
/// longer fence or delimiter spaces (because its own content contains backticks or edge
/// spaces) is emitted untouched rather than half-rewritten. ~keep
pub fn emit_code_span(
    span: &str,
    content: &str,
    output: &mut String,
    node_handle: &NodeHandle,
    parser: &Parser,
    dom_ctx: &DomContext,
) {
    use crate::converter::merge_adjacent_emphasis;
    use crate::converter::utility::siblings::get_previous_sibling_tag;

    let is_plain_single_backtick_pair =
        span.len() == content.len() + 2 && span.starts_with('`') && span.ends_with('`') && !content.contains('`');
    let previous_is_code_sibling = get_previous_sibling_tag(node_handle, parser, dom_ctx)
        .is_some_and(|name| matches!(name, "code" | "kbd" | "samp"));

    if is_plain_single_backtick_pair && previous_is_code_sibling && merge_adjacent_emphasis(output, '`', 1) {
        output.push_str(&span[1..]);
    } else {
        output.push_str(span);
    }
}

/// Push the single space a non-empty, all-whitespace inline body stands for.
///
/// Suppressed when the buffer already ends with a space, mirroring `text_node.rs`'s
/// `!output.ends_with(' ')` guards. ~keep
pub fn emit_whitespace_only_inline_body(content: &str, output: &mut String) {
    use crate::converter::chomp_inline;

    let (prefix, _, _) = chomp_inline(content);
    if !output.ends_with(' ') {
        output.push_str(prefix);
    }
}
