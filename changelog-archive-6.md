## [3.14.1] - 2026-09-17

### Fixed

- **A multi-line link label or image `alt` no longer has one of its lines read as block
  structure** ([#496](https://github.com/xberg-io/html-to-markdown/issues/496)). `CommonMark`
  parses block structure before inline structure (spec 0.31.2 appendix A), so a continuation
  line that looks like a block opener ends the paragraph the label lives in and the `[`/`![`
  never reaches its `]`. `![A\n-\n](S)` produced no image at all -- it produced
  `<h2>![A</h2><p>](S)</p>`, with the image gone. A hard line break did not help, because it
  is inline and block parsing has already finished by then, so the reporter's escaping
  workarounds could not work either. Measured against comrak, 14 of 18 opener shapes destroyed
  the construct: setext `-` and `=`, all three thematic breaks, ATX headings, both code
  fences, block quotes, all three bullet markers, both ordered-list delimiters, and HTML
  blocks. The first non-blank character of a continuation line is now backslash-escaped when
  that line would open a block *that can interrupt a paragraph* -- ordered lists at their
  `.`/`)` delimiter, since a digit cannot carry an escape. Lines that open nothing are
  untouched: four columns of indent (indented code cannot interrupt a paragraph), a type-7
  HTML block such as a bare `<span>`, `2. x`, `-x`, seven `#`. The label's first line is never
  escaped -- it is preceded on that same line by the caller's own `[`/`![`. The escaping is
  unconditional, like the existing bracket escaping and unlike the `escape_misc` family: those
  decide whether text that merely *looks* like Markdown is emitted verbatim, this decides
  whether the image or link survives at all. Multi-line `alt` is not exotic -- TeX4ht emits it
  for every formula it cannot render. Both tiers were affected and both are fixed through the
  one `escape_link_label` helper they share.
- **A `<br>` at the very start or end of an `<a>` is no longer dropped**
  ([#497](https://github.com/xberg-io/html-to-markdown/issues/497)). `<a href="H">A<br></a>B`
  renders as A, a line break, then B; the converter emitted `[A](H)B`, losing the break
  entirely. `[A  \n](H)B` re-parses to exactly the original `<a href="H">A<br /></a>B`
  (verified against comrak), so the break belongs inside the label. Both tiers dropped it for
  the same reason expressed twice -- the whole label was whitespace-trimmed, and once
  flattened a `"  \n"` marker is indistinguishable from the incidental whitespace that really
  does belong before a `</a>` -- and Tier 2 additionally never emitted a *leading* break at
  all, because its "nothing on this line yet" test compares a fresh label buffer's length
  against the enclosing block's start offset. A run of breaks at one edge still collapses to a
  single break (two adjacent markers would put a blank line in the label, and a blank line
  ends the paragraph, destroying the link), and a label of nothing but breaks still collapses
  to empty. A heading and a pipe-table cell are single-line and still fold every break to a
  space. The issue's second example asks for `B[A  \n](H)`, which moves the break to the far
  side of the label text; the break is preserved where the `<br>` actually was instead --
  `B[  \nA](H)`, which comrak renders back to the input DOM.
- Tier 1 no longer emits three spaces where Tier 2 emits one for a `<br>` inside a link inside
  a table cell (`| [A   -   B](H) |` against `| [A - B](H) |`). Tier 1 folded its hard-break
  marker late, in `close_table_cell`, which left the marker's two spaces behind; it now folds
  in `close_link`, which is also what keeps the #496 escaping from firing on a label that is
  about to become single-line anyway.

- Generated Go and R e2e suites no longer assert against strings their fixtures never
  specified (alef pin 0.90.0 to 0.91.5). Two independent generator defects were corrupting
  fixture values on their way into test code: alef's Go emitter rendered a multi-line value as
  a raw backtick literal and its own writer then trimmed the trailing whitespace off every
  physical line, so the three assertions carrying a Markdown two-space hard break read
  `[Alpha\n](url)Beta` where the fixture said `[Alpha  \n](url)Beta`; and alef's R emitter ran
  every plain string argument through the PascalCase-to-snake_case transform meant only for
  enum wire values, so `Alpha<span ...>` was emitted as `alpha<span ...>` and
  `Beta<a ...><br>Alpha</a>` as `beta<a ...><br>_alpha</a>`. The R defect had been silently
  wrong since the 3.14.0 `paragraph_whitespace_only_span_separates_words` fixture landed; it
  went unnoticed because the E2E workflow had skipped every language test job on the three
  preceding commits, so "green" meant "nothing ran". Fixed upstream in alef 0.91.5 rather than
  by trimming the fixtures. The pin bump also carries alef 0.91.0-0.91.3, which for this repo
  is limited to a simpler argument-marshalling path in the Node visitor bridge (no API or
  behaviour change) and dropping its unused `tokio-util` dependency.

## [3.14.0] - 2026-09-16

### Changed

- **Upgraded `html5ever` to 0.40.1, which fixes silent content loss on the HTML repair path.**
  0.40.0's serializer dropped the leading `0xC2` byte of a two-byte UTF-8 sequence, corrupting
  every character in U+0080..U+00BF except NBSP -- `§`, `©`, `°`, `·` among them. Because
  `repair_with_html5ever` serializes the repaired tree back to a string for the primary parser to
  re-read, a single such character made the whole re-parse fail and everything after it vanish.
  This repo was pinned to 0.39.0 to avoid it; 0.40.1 carries the upstream fix, verified here by
  building the same tree against both versions rather than by trusting the release note. A
  regression test now pins the behaviour with non-ASCII fixtures -- the existing repair-path tests
  are ASCII-only and passed on the broken release, which is why the defect was invisible.
- Upgraded `rmcp` to 3.4.0. It deprecates the `ServerInfo` alias in favour of `ServerConfig`
  (the same type, renamed because it collided with the protocol's own `serverInfo` field), so
  the MCP server's `get_info` moved with it. No wire-visible change.

### Fixed

- **Character references in attribute values are now decoded**
  ([#494](https://github.com/xberg-io/html-to-markdown/issues/494)). `<a href>` was the only
  attribute in the converter that called the entity decoder, so every other user-visible
  attribute emitted its raw source text: `title="A&amp;B"` reached the output as the literal
  `A&amp;B`, and `alt="it&#39;s"` as `it&#39;s`. Twelve sites were affected -- `title` on `<a>`,
  `<img>`, `<graphic>` and `<abbr>`; `alt` on `<img>` and `<graphic>`; `src` on `<img>`,
  `<audio>`, `<video>`, `<iframe>` and `<source>`; `cite` on `<blockquote>`; the `language-`
  class on a code fence; and `<meta content>` in extracted metadata. They were found by probing
  every attribute that reaches output, not by reading the ones that looked likely.
  Attribute reads now go through one shared accessor so a new attribute cannot reopen the gap.
  The Markdown escaping is unchanged and was never at fault -- a literal `"` in a title was
  always escaped correctly; the decoded character simply never arrived. Tier 1 carried the same
  defect independently and moves in lockstep.

- **Three Tier-1 divergences that entity decoding made reachable.** Each was latent long before
  it could be triggered, and each is now pinned by a parity test. An `<img>` in a heading was
  emitted as `![alt](src)` by the fast scanner whenever `keepInlineImagesIn` was empty, where
  the DOM path correctly replaces it with its alt text -- an empty list names no heading, so it
  permits nothing. A heading whose body merely *ended* in whitespace was never trimmed, because
  the trim only ran for bodies containing a newline; a decoded `&nbsp;` therefore survived where
  the DOM path dropped it. And a link title containing a quote was escaped as `&quot;` rather
  than `\"`. The first two were found by the generated-corpus parity test over 3,000 documents,
  the third by a sweep over every character decoding newly makes reachable.

- **A wrapper element no longer defeats the nested-table fix**
  ([#488](https://github.com/xberg-io/html-to-markdown/issues/488)). A nested `<table>` inside a
  `<td>` was detected with a single-node tag-name test over the cell's *direct* children, so
  wrapping it in a `<div>` bypassed the 3.12.4 deferral, the pipe escaping and the row fold all at
  once. The inner table's raw `|` characters then read as the *outer* row's cell boundaries on
  reparse -- content loss, not a cosmetic diff. The nested-table *counter* has always descended
  through wrappers; the two now agree. The same line fixes the sibling-cell shape, which was
  corrupt in the same way.

- **Content after a table whose last row is never closed is no longer lost**
  ([#489](https://github.com/xberg-io/html-to-markdown/issues/489)). In
  `<table>...<tr></table><p>Visible footer</p>`, the primary parser discards a close tag that does
  not match the top of its open-element stack, so `</table>` vanished and the paragraph was
  adopted by the still-open `<tr>` -- where the cell collector, which keeps only `td`/`th`, dropped
  it. Such a document now takes the same html5ever repair path that #336, #479 and #486 already
  use. A row that yields no cells also stops emitting a phantom empty row, so the next real row
  becomes the header, matching Tier 1. Two further shapes are fixed by the same gate: a second
  `<tr>` opened without closing the first (its row was silently dropped) and a `<td>` placed
  directly inside `<tbody>` (which produced no output at all). Text-only children are deliberately
  excluded from the gate, so a `<tr>&nbsp;</tr>` spacer does not pay for a full re-parse.

- **An anchor wrapping a table renders the table, not an escaped link label**
  ([#490](https://github.com/xberg-io/html-to-markdown/issues/490)). Any block content inside an
  `<a>` became link-label content, so `<a href="..."><table>...</table></a>` crushed the whole
  table into a single label -- and the label escaper then correctly escaped the markdown that
  produced, leaving an unreadable run of `\|`. The escaping was never the bug; handing a table to
  the label builder was. The anchor's direct children are now partitioned, the inline half forms
  the label and the deferred half renders as blocks after it. Only a deferred subtree that
  actually contains a `<table>` triggers this, and never inside a heading or an inline context,
  so every other anchor shape is byte-identical.

- **A whitespace-only inline wrapper still separates the words around it**
  ([#491](https://github.com/xberg-io/html-to-markdown/issues/491)).
  `Alpha<span style="white-space:pre">\n</span>13` rendered as `Alpha13`. The predicate added for
  #430 asks whether the *next sibling is an element*, so a bare text node after the wrapper took
  the failing path and the newline vanished. Tier 1 was already correct, so the two tiers
  disagreed on this input; they are now pinned together by a parity test. Note that the reported
  `white-space: pre` is incidental -- that property is not implemented, and the defect reproduced
  without it, exactly as a browser collapses the newline to a space either way.
  The wrapper contributes a separator only when the next word butts straight up against it;
  text that already opens with whitespace supplies its own, and is left alone.

- **`keepInlineImagesIn` now means something for `<a>`**
  ([#492](https://github.com/xberg-io/html-to-markdown/issues/492)). The option was consulted for
  headings and for layout cells, but never for anchors, so an `<img>` inside a link that also held
  a block element was replaced by its alt text (or dropped entirely when it had none) no matter
  what the option said. Listing `"a"` now keeps the image as markdown, in both the block and inline
  anchor paths, and for `<graphic>` as well as `<img>`. The change is purely additive -- it can
  turn an image on, never off -- so output is byte-identical for anyone not naming `"a"` in the
  option. The Tier-1 scanner was updated in lockstep.
