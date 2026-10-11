---
title: "Changelog archive: 3.15.0–3.14.0"
---

## [3.15.0] - 2026-09-26

### Added

- **`ConversionOptions::base_url`** resolves relative `href`/`src` destinations against a
  caller-supplied base URL, honoring a document's own `<base href>` the way a browser does.
  Defaults to `None`, so output is byte-identical for callers who do not set it. Resolution
  happens identically in both the Tier 1 and Tier 2 rendering paths.

### Changed

- Upgraded `rmcp` to 3.4.1.
- Repinned the `alef` generator to 0.96.5 and regenerated every binding. Generator drift
  only; no public API of any binding changed.
- Split `converter/utility/content.rs` and `converter/inline/link.rs` (each over the
  1000-line quality gate) into smaller modules, and reduced `convert_table_row`'s cyclomatic
  complexity by extracting its visitor-hook pre-pass into a separate function. No behavior
  change; every existing import path is preserved via re-exports.

### Fixed

- **The R binding compiles again.** `ConversionOptions::base_url` is the struct's first
  `Option<String>`, and alef's extendr backend assigned it a bare `String`, failing the whole R
  package build with `error[E0308]: expected Option<String>, found String`. Fixed in alef 0.96.5.
  It went unnoticed for two commits because two independent mechanisms each suppressed the leg.
  On the commit that introduced the line, `Test: R` was **cancelled**: `ci-e2e.yaml`'s
  `cancel-in-progress: true` group is keyed on the branch, so the next push to `main` killed the
  run before R finished. On the commit after that, `Test: R` was **skipped**: the leg is gated on
  a `packages/r/**`/`crates/html-to-markdown/**` paths filter that commit did not match. Neither
  state is a failure, so `CI E2E` reported success twice without ever compiling R.

- CI's fixture-snippet validation now really validates the 341 Swift snippets. They had all
  been reporting `Unavailable` with `no such module 'HtmlToMarkdown'`: Swift 6.3 made
  `swiftbuild` the default build system, whose bin path holds the `.swiftmodule` files
  directly and emits no `Modules/` directory, while alef points `-I` at `<bin-path>/Modules`.
  The session now reconstructs that layout after building. The gate went red without any
  change to this tree, when the runner's preinstalled Swift moved to 6.4.

## [3.14.3] - 2026-09-19

### Fixed

- **A whitespace-only inline wrapper nested inside another wrapper keeps its separator**
  ([#504](https://github.com/xberg-io/html-to-markdown/issues/504)).
  `<h2><strong>Alpha</strong><strong><em><br></em></strong>Beta</h2>` rendered `## **Alpha**Beta`
  from 3.14.2 on, where 3.14.1 kept the space, and a paragraph or table cell joined the words the
  same way. Issue #501 taught the shared wrapper emitter that a whitespace-only body at a line
  start contributes nothing, keyed on the destination buffer being empty -- but the destination
  can also be an enclosing wrapper's fresh scratch buffer, which is empty mid-line, so the inner
  `<em>`'s one space was dropped there and the outer `<strong>` came out empty. The line-start
  rule now fires only on the block's own buffer, using the address test the text-node fallback
  already uses for the same distinction, so `<mark>`, `<ins>`, `<del>`, `<sub>` and `<sup>`
  wrappers move with it. A `<br>` inside a wrapper inside a table cell had the same shape on its
  own since before 3.14.2 -- `<td>Alpha<em><br></em>Beta</td>` -- and now keeps its space too; a
  cell's own leading `<br>` still contributes nothing. Tier 1 bails on adjacent emphasis, so the
  change is Tier-2 only.
- **A newline inside nested transparent inline wrappers still separates the words around it**
  ([#505](https://github.com/xberg-io/html-to-markdown/issues/505)).
  `<p><i>Alpha</i><span><span>\n</span></span>Beta</p>` rendered `*Alpha*Beta` where a browser
  shows a space. Issues #430 and #491 taught the text-node fallback that a lone newline inside
  an inline wrapper separates words when the wrapper is followed by inline content, but the check
  looked one level up only: with a second wrapper the inner `<span>` is the last child of the
  outer one and the newline was dropped. The check now climbs through every transparent inline
  ancestor that has nothing after it and stops at the first block. Tier 1 already emitted the
  space, so this was a live cross-tier divergence; the tiers now agree.
- **An anchor that html5ever's adoption agency splits around a block is emitted once, not twice**
  ([#493](https://github.com/xberg-io/html-to-markdown/issues/493)).
  `<a href="/o"><div><a href="/i">Inner</a></div></a>` rendered `[](/o)` and then
  `[](/o)[Inner](/i)`: the repair legitimately closes the outer `<a>` at the `<div>` and
  reconstructs it inside, and the clone reached the renderer indistinguishable from an authored
  element. A renderer rule keyed on shape would either drop a genuine empty anchor (`CommonMark`
  example 484) or a deliberately authored duplicate, so the fix is at parse time: every `<a>`
  start tag is stamped with a private origin id before the tree builder sees it, the clones
  inherit it, and on the repaired tree the halves of a split anchor with no content of their own
  are unwrapped in place. When no half has content the authored one is kept, so the destination
  still appears once as `[](/o)`; `<a href="/o"><div>Text<a href="/i">Inner</a></div></a>` keeps
  the half that carries `Text` and renders `[Text](/o)[Inner](/i)`. Input the repair never runs
  on, and an anchor the repair leaves whole, are unchanged.

## [3.14.2] - 2026-09-18

### Fixed

- **An `&` that would read as a character reference in a link destination or title now stays
  literal** ([#498](https://github.com/xberg-io/html-to-markdown/issues/498)). `CommonMark`
  decodes entity and numeric character references inside destinations and titles, so the
  decoded `?&plus;` that `<img src="?&amp;plus;">` produced from 3.14.0 on re-parsed as `?+` --
  a different URL. `<a href>` and `title` had carried the same defect since 3.12, when they
  started decoding; only `src` was new to it. Such an `&` is now written back as `&amp;`, the
  form 3.13 emitted and the one non-`CommonMark` consumers of the URL also read correctly. Only a
  reference the HTML5 decoder really recognises is touched: `?a&b` and `&foo;` are unchanged.
  Tier 1 wrote a link's `href` straight into the output with no escaping at all, so it now
  shares Tier 2's `append_url_destination`; that also ends a divergence unrelated to entities,
  where Tier 1 emitted `[T](/a b(c)` and `[T](/a(b)` against Tier 2's `[T](</a b(c>)` and
  `[T](/a\(b)`.
- **An unmatched `[` in a link label or image `alt` is now escaped**
  ([#499](https://github.com/xberg-io/html-to-markdown/issues/499)). `CommonMark` accepts a
  bracket in a label only escaped or as a matched pair; an unmatched `]` was already escaped,
  an unmatched `[` was not. `<img alt="[A;B)" src="S">` rendered `![[A;B)](S)`, which
  re-parses as a dangling `!` followed by a real `[A;B)](S)` link -- the image was gone. The
  shared `escape_link_label` helper now escapes every opener left without a closer, so both
  tiers and every caller (links, images, `<graphic>`, SVG titles, embedded media) move together.
- **A run of ASCII whitespace at the start of a line is no longer emitted**
  ([#501](https://github.com/xberg-io/html-to-markdown/issues/501)).
  `<p>P</p><div><span>    </span><img alt="A" src="S"></div>` rendered the image as
  `![A](S)` indented by four columns, which `CommonMark` reads as an indented code block, so
  the image was lost. Issue #460 dropped leading whitespace at the start of a
  *paragraph*; a `<div>` never set that flag and the run fell through to the verbatim fallback.
  Leading ASCII whitespace on a line is never Markdown content, so the text-node fallback now
  drops it at a line start of the block's own buffer, the `<div>` handler measures its content
  start the way `<p>` does so an inline wrapper's empty scratch buffer is not mistaken for one
  (its single space must still reach the #481 handling), and a whitespace-only wrapper spliced
  in at a line start contributes nothing -- `<p>A</p><p><i> </i>B</p>` no longer opens its
  second paragraph with a stray space. A multi-space run mid-line collapses to one space, as it
  already did between two inline siblings. A run carrying a decoded `&nbsp;` is untouched. Tier 1
  already emitted every case this way.
- **A whitespace-only inline wrapper body no longer vanishes and joins the words around it**
  ([#502](https://github.com/xberg-io/html-to-markdown/issues/502)).
  `<b>Alpha</b><b><span>\n</span></b><b>Beta</b>` rendered `**AlphaBeta**`, and
  `Alpha<i>\n</i>Beta` rendered `AlphaBeta`, where a browser shows a space. Two drops of one
  shape: a newline-only text node returned early because the wrapper's scratch buffer was empty
  -- the buffer is fresh per wrapper, so its length says nothing about the document -- and a
  `<br>` with nothing before it in that buffer left a bare newline that `chomp_inline` did not
  count as a space. Both now surface as the single separating space issue #481 already gives a
  literal `<b> </b>`, still suppressed after an existing space; a truly empty `<b></b>` still
  emits nothing. Tier 1 bails on every one of these shapes, so the change is Tier-2 only.
- **A headerless table whose rows have different cell counts is padded, not turned into a
  list** ([#500](https://github.com/xberg-io/html-to-markdown/issues/500)).
  `<table><tr><td>A</td><td>B</td></tr><tr><td>C</td></tr></table>` rendered `- A B` / `- C`:
  ragged row lengths alone classified a table as a *layout* table. A headerless table with a
  short row is ordinary tabular data far more often than it is an email-signature grid, and the
  regular renderer already pads a short row to the table's width (issue #13), so it now renders
  `| A | B |`, `| --- | --- |`, `| C |   |`. Layout still triggers on more than one nested table,
  `colspan`/`rowspan` combined with `border="0"`, a blank table, or a short table dense with
  links. Tier 1 keeps its stricter bail on ragged rows -- it has no padding of its own -- which
  only ever sends more input to the path that does. A layout row also **keeps an image as
  `![alt](src)`** instead of degrading it to alt text: the row is a list item, and list items and
  data cells keep images by default; only headings degrade them. `keepInlineImagesIn` is no
  longer needed for that (issue #433), though it still governs headings and links.
- **A layout row stays on one line across a `<br>` or a blank nested table.** A `<br>` inside a
  layout cell emitted a hard-break marker, and a nested table that rendered to nothing still
  emitted the blank line meant to separate content; either put a bare newline inside a list
  item's line and ended the item. Both were latent -- the Hacker News footer in the gh-121
  fixture is `<img><table>bar</table><br>links`, and while the spacer image degraded to nothing
  every separator stayed suppressed -- and keeping the image surfaced them. A `<br>` now follows
  the settled cell rule its `<div>`/`<p>` continuations already use (issue #470), and an empty
  table output writes no separator. Across the benchmark corpus this rejoined three split list
  items in one fixture and changed nothing else; the other movements are images now kept in
  layout rows and leading whitespace dropped at line starts (#501).
- **An anchor wrapping a table inside a layout cell keeps its inner links clickable**
  ([#503](https://github.com/xberg-io/html-to-markdown/issues/503)). Issue #490 renders a
  wrapped table as a separate block after the link, but refused inside inline contexts, and a
  cell of a table the layout heuristic turns into a bullet list converts as inline. The nested
  table was walked into the label instead, where the bracket escaping turned `[One](/one)` and
  `[Two](/two)` into text; only the outer destination survived. A layout cell is a list item's
  text, not a link label, and it already holds a bare nested table on the lines after its
  bullet, so the deferred table now lands there the same way. Headings and true inline labels
  keep refusing. Tier 1 bails on a table opened inside a link, so this is Tier-2 only.

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
