---
title: "Changelog archive: 3.13.0–3.11.5"
---

## [3.13.0] - 2026-09-14

### Changed

- **BREAKING (java): enum constants are now `SCREAMING_SNAKE_CASE`.** `LinkStyle.Inline` becomes
  `LinkStyle.INLINE`, `HeadingStyle.Atx` becomes `HeadingStyle.ATX`, and so on across every
  generated enum; builder default sites move with them. The JSON wire values are unchanged, so
  no serialized payload moves -- only the Java-facing constant names.

- **BREAKING: the `RDFa` structured-data variant is spelled consistently across bindings.** One
  Rust variant previously generated seven different names. Each binding now uses the correct
  acronym segmentation for its own convention: `rdfa` (python `options.py`, elixir, ruby,
  swift), `RDFA` (python stub, java, kotlin), `Rdfa` (csharp, go). Dart keeps `rdFa`, which is
  what flutter_rust_bridge itself declares. The wire value `"rdfa"` is unchanged everywhere.

- **BREAKING (python): the type stub declared an enum member the runtime does not have.** The
  `.pyi` emitted `StructuredDataType.RD_FA` while the extension declares `RDFA`, so a type
  checker accepted the name that raises `AttributeError` and rejected the one that works. Both
  now agree on `RDFA`.

- **BREAKING (elixir): `VisitResult`'s payload key matches the NIF struct.** `%{type: :custom,
  value: ...}` becomes `%{type: :custom, custom: ...}`, and `:error` likewise. The NIF struct has
  always declared `custom`/`error` fields, so the previous shape never round-tripped.

- **BREAKING (ruby): `VisitResult.from_hash` reads the real wire key.** It read `_0`; the core
  enum is adjacently tagged with `content = "output"`, so `_0` never matched and the payload was
  silently discarded in both directions.

### Fixed

- **`<br>` inside an inline code span no longer emits a raw newline inside the backticks**
  ([#487](https://github.com/xberg-io/html-to-markdown/issues/487)). `<code>A<br>B</code>`
  produced `` `A\nB` ``, which `CommonMark` does not read back as one code span. The span is now
  split and joined by a real hard break -- `` `A` `` + the `newline_style` marker + `` `B` `` --
  so the marker sits outside the span, where it is syntax rather than content, and `<code>`
  behaves like `<b>` and `<i>`, which already emitted hard breaks here. The output is round-trip
  stable through a `CommonMark` render; the previous form was not. `<kbd>` and `<samp>` had the
  same defect and are fixed with it.

  Both tiers were wrong and disagreed with each other -- Tier 1 emitted `` `A  \nB` ``, leaking
  the two-space marker into the code content -- and both are fixed, so the tiers now agree. A
  pre-existing Tier-1 bug surfaced on the way: inside a link label the `<br>` branch ran before
  the code-span check, trapping two spaces in the first segment.

  Contexts that cannot carry a hard break fold to a single space instead of splitting: headings
  and table cells. A link label splits, matching what a bare `<br>` in link text already did.

- **A literal line ending inside an inline code span folds to a space.** `<code>a\nb</code>`
  (a real newline in the source, no `<br>`) produced `` `a\nb` ``. `CommonMark` gives a line
  ending inside a code span no hard-break meaning and renders it as a space, so the old output
  was never round-trip stable. `<pre>` and fenced blocks are unaffected and keep every newline.

### Note

`html5ever` stays pinned at 0.39.0. 0.40.0's serializer drops the `C2` lead byte for every
codepoint in U+0080-U+00BF except NBSP (`§`, `©`, `°`, `±`, `»`, ...), emitting invalid UTF-8.
In this crate that makes `repair_with_html5ever` fail its `from_utf8` check and silently skip
the repair pass on any document containing one of those characters -- it fails safe, but
degrades quality invisibly. Fixed upstream in servo/html5ever#784, which is merged but not yet
published; this crate will adopt 0.40.x once it is.

## [3.12.4] - 2026-09-12

### Fixed

- Render `<strike>` as strikethrough. The Tier-2 inline dispatch listed only `del`/`s`, so
  `<strike>` fell through to a plain child walk and lost its `~~` markers entirely.
- Merge adjacent inline elements that share a repeated-character delimiter, extending the
  [#483](https://github.com/xberg-io/html-to-markdown/issues/483) fix from `<em>`/`<strong>`
  to every other family. `<del>A</del><del>B</del>` produced `~~A~~~~B~~`, which CommonMark
  reparses as one strikethrough containing `A~~~~B`; two adjacent code spans produced
  `` `A``B` ``, reparsed as one span carrying the literal backticks. `<ins>`, `<mark>`,
  `<var>` and `<dfn>` had the same defect.
- Keep the word separator a whitespace-only inline element stands for, extending the
  [#481](https://github.com/xberg-io/html-to-markdown/issues/481) fix to `<ins>`, `<sub>`,
  `<sup>`, `<var>`, `<dfn>`, `<abbr>` and `<q>`, each of which joined the words either side.
- Keep `<code> </code>` as a code span whose content is a space. It was dropped outright, and
  the delimiter-space padding that an all-spaces body used to get turned one space into three
  (CommonMark strips one space per end only when the content is not entirely spaces).
- Stop `<ins>`, `<kbd>`, `<samp>`, `<var>` and `<dfn>` from emitting their markers inside a
  code span or fenced block, where `==`, a second backtick pair and `*` are literal content
  rather than formatting. Every other inline handler already suppressed itself there.
- Close 22 Tier-1/Tier-2 output divergences across the inline elements, found by sweeping both
  converters over 26 tags. Tier-1 now reproduces `<var>`/`<dfn>`, bails on the shapes it cannot
  (`<q>`, `<mark>`, the new delimiter merges, a whitespace-only body), and suppresses
  `<strong>`/`<em>` markers inside a code span as Tier-2 does.

- Treat adjacent duplicate Rust warning flags as equivalent in benchmark provenance checks while preserving timing gates.
- Publish every native NuGet runtime package required by the managed package's runtime graph.
- Upload Dart native archives and checksums required by the generated package downloader.
- Publish Go installer archive aliases and required SHA-256 sidecars.
- Verify the installed Homebrew CLI and FFI versions directly in the registry smoke task.
- Stage original native archives through the shared Zig packager; the 3.12.3 source archive requires separate C FFI libraries.
- Merge adjacent inline emphasis into a single delimiter run, so `<i>A</i><i>B</i><i>C</i>`
  renders as `*ABC*` rather than the `*A**B**C*` CommonMark reparses as nested emphasis
  ([#483](https://github.com/xberg-io/html-to-markdown/issues/483)).
- Emit exactly one space for a whitespace-only inline element; `A<i> </i>B` duplicated it and,
  inside a paragraph, `<p>A<i> </i>B</p>` dropped it entirely
  ([#481](https://github.com/xberg-io/html-to-markdown/issues/481)).
- Drive the musl Node cross-compile through the shared build action, whose per-leg artifact
  staging replaces the `napi artifacts` call that `@napi-rs/cli` 3.9.1 made fail on any
  single-target matrix leg.
- Keep the visible text of a `<tr>` nested directly inside a `<td>`, a shape malformed
  newsletter HTML produces; the row and its content were dropped silently. The same fix
  repairs tbody-less tables with implicitly-closed cells, where
  `<table><tr><th>h1<th>h2<tr><td>a<td>b</table>` emitted only a one-column header and
  dropped both the second header and the entire data row
  ([#486](https://github.com/xberg-io/html-to-markdown/issues/486)).
- Render a data table's nested single-cell table as its own table instead of flattening it
  into a cell of escaped pipes ([#484](https://github.com/xberg-io/html-to-markdown/issues/484)).
- Apply the WHATWG numeric character reference replacement table, so `&#155;` decodes to
  `›` rather than a raw C1 control character; null, surrogate and out-of-range
  references now yield U+FFFD ([#485](https://github.com/xberg-io/html-to-markdown/issues/485)).

## [3.12.3] - 2026-09-09

### Fixed

- Preserve visible images inside `font-size: 0` wrappers ([#476](https://github.com/xberg-io/html-to-markdown/issues/476)).
- Recover table cells following malformed Office namespace tags
  ([#477](https://github.com/xberg-io/html-to-markdown/issues/477)).
- Render data tables inside one-cell layout wrappers as separate Markdown tables
  ([#478](https://github.com/xberg-io/html-to-markdown/issues/478)).
- Repair nested anchors so both link destinations remain clickable
  ([#479](https://github.com/xberg-io/html-to-markdown/issues/479)).
- Resolve unpublished native Node packages from the workspace during frozen documentation installs.
- Build prerelease PHP extensions against the local Rust core and committed dependency lockfile.
- Preserve native error codes on typed C# conversion exceptions.
- Prevent panics when nested list visitors shorten earlier output or replace it with multibyte text
  ([#474](https://github.com/xberg-io/html-to-markdown/issues/474)).
- Generate R and Ruby bindings that pass strict Clippy checks while preserving visitor and conversion behavior.
- Accept explicit `nil` for optional Ruby conversion fields.
- Use canonical URL-escape enum values in fixtures so Node and WebAssembly suites regenerate completely.
- Point Ruby dependency-update tasks at the native crate and remove the obsolete duplicate Rust workspace.

### Changed

- Refactor MCP helpers, image construction, and visitor state to remove three lint suppressions while preserving
  preprocessing validation order ([#466](https://github.com/xberg-io/html-to-markdown/pull/466)).
- Regenerate all bindings, end-to-end suites, and snippets with Alef 0.85.11.
- Refresh dependency lockfiles across Rust workspaces.

## [3.12.2] - 2026-09-07

### Fixed

- **`bullets` now applies to layout-table rows**
  ([#472](https://github.com/xberg-io/html-to-markdown/issues/472)). A table with inconsistent
  column counts and no `<th>`/`<caption>` is treated as a layout table and renders each row as a
  list item — but that renderer hardcoded `-` and never read `options.bullets`, so configuring
  the option had no effect on the markers actually emitted. Surfaced by the reporter of #470, who
  set `bullets` to `"*+-"` in 3.12.0 and still got hyphens.

  The marker now cycles through `bullets` by nesting depth the same way list items do, so a
  layout table nested inside a list takes the next marker rather than repeating its parent's. The
  prefix strip that prevents a doubled-up marker assumed a hyphen as well, and now accepts any
  configured bullet.

  Default output is unchanged — the default `bullets` string already starts with `-`. Tier-1
  bails on layout tables, so this is a Tier-2 path with no parity mirror.

## [3.12.1] - 2026-09-07

Correctness release covering the four defects reported against 3.12.0. Three are conversion bugs
in the core, one extends the hidden-element rule; all four were reported through the Java binding
and reproduce identically in every language.

### Fixed

- **An uppercase void element no longer swallows the rest of the document**
  ([#467](https://github.com/xberg-io/html-to-markdown/issues/467)). The bundled `astral-tl`
  parser matches void elements against an all-lowercase table using a tag's raw source bytes, so
  `<META>` missed it and was pushed onto the open-element stack as a *container*: the parent's
  close tag could not pop it and every following sibling was absorbed as its child. For
  `<head><META ...></head><body>` that left `<body>` a grandchild of `<head>`, out of reach of the
  direct-child rescue in `handle_head`, and the whole document converted to an empty string. The
  `charset` attribute in the report was incidental -- the crate has no charset handling, and every
  HTML5 void element (`<BR>`, `<IMG>`, `<HR>`, `<INPUT>`, `<LINK>`, ...) reproduced it. Tier-2 now
  lowercases void-element tag *names* during preprocessing; attribute names and values are left
  byte for byte alone. Tier-1 already lowercased before lookup, so this also closes a tier
  divergence.

- **A nested table inside a cell keeps its row boundaries under `br_in_tables`**
  ([#469](https://github.com/xberg-io/html-to-markdown/issues/469)). A Markdown cell cannot hold a
  nested table, so the inner table is flattened into the outer cell with its pipes escaped. Until
  3.11.2, `br_in_tables: true` merely skipped the whole-cell newline fold, letting the inner rows
  leak out of the cell as raw newlines -- malformed, but a GFM parser could still see two rows.
  Making that fold unconditional (correctly, for issues #456 and #457: a raw newline between two
  pipes splits the row across physical lines) collapsed the rows onto one line joined by spaces
  and erased the boundaries. The fold stays unconditional; the flattened rows are now joined with
  the literal `<br>` that `br_in_tables` already means everywhere else in a cell. A preceding
  sibling is separated from the nested table too, which previously ran straight into its first
  pipe (`Before\| ID`). **This restores row boundaries, not table structure** -- a real nested GFM
  table remains impossible and the inner pipes stay escaped.

- **Adjacent paragraphs in a layout-table cell are separated**
  ([#470](https://github.com/xberg-io/html-to-markdown/issues/470)). A table with inconsistent
  column counts and no `<th>`/`<caption>` renders each row as a list item, and those cells convert
  as inline. That suppressed the ordinary block separator while never reaching the table-cell
  continuation rule, so `<p><b>Alice Example</b></p><p><i>Customer Service</i></p>` emitted
  `**Alice Example***Customer Service*` -- merged words and invalid emphasis. Such cells now
  follow the settled cell rule from issues #453/#454: a literal `<br>` under `br_in_tables`, a single
  space otherwise. `<div>` continuations are covered by the same change. Layout rows are list
  items, so they still do not take table-cell pipe or emphasis escaping. Not a regression -- this
  predates 3.8.3.

### Changed

- **`font-size: 0` now marks an element as not rendered**
  ([#468](https://github.com/xberg-io/html-to-markdown/issues/468)), joining `display: none`,
  `visibility: hidden` and the `hidden` attribute. Reported against generated email banners whose
  marker text reached the Markdown despite being invisible in a browser. Any exact zero length is
  recognised (`0`, `0px`, `0.0em`, `.0%`, in any casing, with or without `!important`) and the CSS
  last-declaration-wins cascade applies as it already does to `display`.

  **This drops content that previous versions emitted.** One case is deliberately exempt: the same
  declaration is the classic inline-block/email spacing hack, where the wrapper kills inter-child
  whitespace and each child restores a readable size. When a descendant re-declares a non-zero
  `font-size`, the subtree is kept. That guard is a one-level-of-inheritance heuristic, not a
  cascade -- a size restored from a stylesheet is out of reach of a byte-level pass. Tier-1 has no
  subtree awareness and bails on any `font-size: 0`, deferring to the tier that can see the
  descendant.

  Detection remains unconditional, as it has always been for the other three: no option governs it.

## [3.12.0] - 2026-08-31

Correctness release. A broad pass over converter correctness: defects in the shipped output,
plus a run of cases where the Tier-1 fast scanner and the Tier-2 converter disagreed on the same
input -- the library picks a tier automatically, so those meant one document could convert two
ways.

Test coverage behind it: every one of the 652 `CommonMark` spec examples is now exercised through
a conversion-fixpoint oracle (the exact-match test compares against the spec's own rendering, so
it can only run on the 131 examples admitting a single valid form), a Tier-1/Tier-2 differential
oracle over the benchmark corpus and a generated document set, and a fuzz target.

Against that fixpoint oracle, 644 of the 652 examples now round-trip unchanged with escaping
enabled and 629 with the shipped defaults, up from 607 and 597. Of the 8 that remain, three are inherent to the round trip -- adjacent
block quotes, and adjacent lists sharing a bullet, merge under any compliant reparse whatever we
emit -- and the other five differ only in the byte spelling of a link destination.

### Removed

- **`htm_conversion_options_update_visitor` is gone from the C API.** The getter documented a
  non-null return as caller-owned data, but `ConversionOptionsUpdate` is only ever constructed
  through `htm_conversion_options_update_from_json`, and its `visitor` field carries a full
  `serde(skip)` -- so `Deserialize` could never populate it and no FFI setter existed. The symbol
  could only ever return `0`. **Breaking for C consumers:** it disappears from
  `html_to_markdown.h` and from the compiled library, so a call that used to yield `NULL` at
  runtime is now a link error. Nothing that worked stops working. `ConversionOptions.visitor` is
  unaffected -- it carries the same attribute but is genuinely reachable, because
  `htm_options_set_visitor` writes it directly, bypassing serde.

### Added

- CLI flags for six `ConversionOptions` fields that the library and the MCP surface already
  exposed but the CLI hardcoded: `--exclude-selectors`, `--url-escape-style`,
  `--max-image-size`, `--capture-svg`, `--no-infer-dimensions` and `--tier-strategy`. Each
  now defaults to the library's own default, so an invocation that omits them is unchanged.
  The three image flags require `--extract-inline-images`, which they are inert without.

### Performance

- `escape_link_label` returns `Cow<'_, str>` and skips its two allocations when the text
  contains no `[` or `]`, which is the common case for link labels, image alt text and
  titles. The two-pass design is unchanged -- it exists to avoid an O(n^2) `String::insert`
  shape.

### Fixed

- **Uppercase attribute names no longer discard link destinations and image sources.**
  HTML attribute names are case-insensitive, but the Tier-2 converter matched them
  byte-for-byte as written, so `<a HREF="up.html">link</a>` converted to bare `link` and
  `<img SRC="a.png" ALT="cat">` to `![](<>)` -- the destination and the alt text were
  dropped, not merely reformatted. The Tier-1 fast scanner has always compared attribute
  names case-insensitively, so these inputs were also a tier disagreement: the same
  document converted two ways depending on which tier ran. The `astral-tl` 0.8.0 parser
  upgrade lowercases attribute keys at parse time, which fixes both.
- **Tier-1 and Tier-2 disagreed on GFM autolinks.** `autolinks` defaults to `true` but was
  not gated in the Tier-1 router, and the Tier-1 scanner had no autolink branch, so
  `<a href="https://x.com">https://x.com</a>` converted to `<https://x.com>` on Tier-2 and
  `[https://x.com](https://x.com)` on Tier-1 -- the library picks a tier automatically, so
  the same document converted two ways. Tier-1 now implements the autolink form rather than
  being gated off it, since gating a default-`true` option would have made the fast path
  unreachable for ordinary input.
- **Tier-1 could return the wrong output for an autolink whose label carried inline markup.**
  Tier-2 tests its autolink predicate against the tag-stripped text, so
  `<a href="https://x.com"><b>https://x.com</b></a>` still autolinks there; Tier-1 compared
  its rendered label (`**https://x.com**`), never matched, and emitted
  `[**https://x.com**](https://x.com)`. Tier-1 now bails to Tier-2 on this shape. The bail is
  guarded by a subsequence precheck so it fires only when an autolink is actually possible:
  on the benchmark corpus, none of the 889 scheme-href links that carry nested markup trigger
  it, so decorated links keep the fast path.


- **`<address>`, `<search>`, `<hgroup>` and `<center>` no longer merge into their neighbours.**
  All four are block-level, but they reached a pass-through handler that emits no separator,
  so `<address>foo</address><address>bar</address>` produced `foobar` -- nothing in the output
  recorded that these were ever distinct blocks. `<colgroup>`, `<col>`, `<base>`, `<html>` and
  `<body>` share the same internal classification and are deliberately unchanged: the first
  three are table-internal or void metadata, and the last two wrap every document.

- **Two block containers in one table cell are separated by one space, not three.** The fast
  scanner emitted a hard line break between them regardless of `br_in_tables`, and the
  table-cell finalizer turned it into a three-space run, where the converter emits a single
  space under the default options.

- **Five Tier-1/Tier-2 divergences closed.** `<blockquote>` omitted its trailing blank line
  in the fast scanner, invisible to every existing test because they all placed the
  blockquote last; fixing it exposed a `<pre>` fence stripping one trailing newline where the
  converter strips all of them. A text node's leading space survived at the start of a bare
  `<span>`/`<u>`, producing a double space across the Google Docs and WordPress fixtures.
  `<address>`, `<search>`, `<hgroup>` and `<center>` emitted a block separator the converter
  does not. Images with a lazy-load `src` and flattened nested-table pipes now agree as well.

- **An HTML comment no longer forces a redundant reparse of the whole document.** Custom
  elements are detected by looking for a hyphen in a tag name, but the scan treated the text
  inside `<!--...-->` as a tag name -- `!--c--` contains a hyphen -- so any document with any
  comment was reparsed through the repair path for nothing. On a 200-element document
  producing byte-identical output, a single comment cost 1.8x. Comments are near-universal in
  real pages, so most documents were paying it.

- **A table nested inside another table's cell no longer destroys the inner cells.** The
  nested table's own row and separator syntax was flattened into the outer cell unescaped, so
  its bare `|` characters were read as additional cell boundaries for the *outer* row. GFM
  truncates a row to the header's column count, so the inner cells were dropped outright on
  reparse. The flattened content is now pipe-escaped outside code spans.

- **A run of `&nbsp;` between two inline elements survives.** A whitespace-only text node
  between inline siblings was collapsed to a single plain space unconditionally. `str::trim`
  is Unicode-aware and counts `U+00A0` as whitespace, so the run was destroyed on the first
  conversion, not merely on a round trip. The same collapse also ran without checking whether
  the output already ended in a space, stacking a real inter-element space against the
  synthetic one left behind by `<style>` removal into a literal double space.

- **Content written directly inside `<table>`, outside any row or cell, is no longer dropped.**
  HTML5's "in table" insertion mode foster-parents such content: it moves to just before the
  table and survives. The parser this crate uses performs no such fixup and the table builder
  recognised only `caption`/`colgroup`/`col`/`thead`/`tbody`/`tfoot`/`tr` there, so raw text
  was silently discarded and stray elements went through a no-op handler --
  `<table>abc</table>` converted to nothing at all.

  It appeared to work whenever a comment happened to sit nearby, but that was coincidence: a
  separate defect treats `!--c--` as a tag name, sees a hyphen, and reroutes any
  comment-bearing document through the html5ever repair path, which does implement foster
  parenting. The text survived by accident. That path is now entered deliberately for this
  shape. Comments themselves are excluded, since HTML5 keeps them as children of the table
  and nothing is lost.

- **Lazy-loaded images resolve to their real address instead of converting to nothing.**
  Lazy-loading libraries leave `src` empty or holding a 1x1 `data:` placeholder and put the
  actual URL in `data-src`, `data-lazy-src`, `data-original`, `data-srcset` or `srcset`, so
  those images came out as `![alt]()` or a base64 blob -- effectively invisible on a large
  share of modern pages. The fallback applies only when `src` is already empty or a `data:`
  URI, so a plain `<img src="...">` is byte-identical to before. A populated non-`data:` `src`
  is trusted even when it looks like a placeholder, since some pages carry the real photo
  there while `srcset` holds only the lazy-load stand-in.

- **A heading inside `<summary>` or `<figcaption>` no longer splices its `#` prefix into
  unrelated text.** Tier-1 records a heading's content offset against whichever buffer is
  active when the tag opens, and `<summary>`/`<figcaption>` accumulate into their own buffer.
  Only table cells were special-cased, so a heading in a `<summary>` took a small
  buffer-relative offset and used it to index the whole document output instead -- inserting
  heading prefixes into the middle of already-emitted text. Rustdoc's
  `<details><summary><h3 class="code-header">` shape turned `Sample` into `Samp#### #### le`.
  This corrupted adjacent content, not just the heading.

- **An unclosed `<table>` no longer discards its rows.** Every element still open at end of
  input is closed implicitly, but Tier-1's implicit-close path did nothing for `<table>`,
  dropping the entire accumulated table -- fully-formed rows included -- where the explicit
  `</table>` path rendered it correctly. Text sitting directly inside `<table>` outside any
  row or cell is now routed to Tier-2 rather than silently discarded.

- **An empty `title=""` is treated as absent instead of rendered as `(url "")`.** The empty
  annotation carries no information and no Markdown serializer round-trips it, so converting
  the re-rendered output produced different Markdown than the first pass. Applies to links,
  images and graphics alike. A whitespace-only title is still a title; only a genuinely empty
  attribute changed.

- **HTML5 bogus comments render as nothing instead of leaking their text.** `<?php echo 1; ?>`
  emitted `?php echo 1; ?>`, `<!bogus>` emitted a stray `>`, and `</3>` emitted `</3>`. All
  three are comment tokens under the tokenizer's tag-open, markup-declaration-open and
  end-tag-open states, so they render as nothing -- as real `<!-- -->` comments already did.
  Identical constructs were rendering differently based only on which tokenizer state they
  happened to reach.

  Most visibly this cleans up Microsoft Word HTML, whose downlevel-*revealed* conditional
  comments (`<![if !vml]> … <![endif]>`, not wrapped in `<!--`) were surfacing as literal noise
  around every image and footnote.

  Real comments, CDATA, and doctypes are stepped over rather than scanned into. That matters
  for downlevel-*hidden* conditional comments: `<!--[if gte mso 9]> … <![endif]-->` is a real
  comment whose terminator is the `-->` at the end of `<![endif]-->`, so treating that
  `<![endif]` as bogus would delete the comment's own terminator and swallow the rest of the
  document. `<![CDATA[` is left untouched entirely, since it is only a bogus comment outside
  foreign content and this pass has no element context to distinguish `<svg>` interiors.

- **`strip_hidden_elements` no longer treats a non-tag `<` as a tag, fixing both a
  content bug and a quadratic-time denial-of-service vector.** The hidden-element pre-pass
  accepted any `<` whose next byte was not `/` or `!`, then scanned forward for the next `>`
  anywhere in the document and treated whatever it spanned as a tag. HTML5 only begins a tag
  name when `<` is followed by an ASCII letter, so this was wrong in three ways:

  - **Visible text was silently deleted.** `<1div hidden>x</1div>` is entirely text, but the
    pre-pass matched it as a hidden element and removed the whole span including the `x`.
    Same for `<_div …>`, `<-div …>`, `< div …>` and any non-ASCII initial.
  - **Genuinely hidden content leaked.** In `<<div hidden>x</div>` the span began at the stray
    first `<`, so what got removed was `<<div hidden>` and the `x` inside the real hidden
    `<div>` survived into the output.
  - **Conversion was O(n²) in unterminated `<`.** With no `>` ahead, every candidate `<`
    re-scanned to end of input. Converting 40,000 bare `<` took 1.05 s and 80,000 took 4.2 s;
    around a megabyte of such input would have taken hours. This is reachable from untrusted
    pages -- the repo's own fixtures already carry a `fallthrough_bare_lt` group -- so it was
    a practical DoS. The same input now converts in roughly linear time; the guarded case
    dropped from 16.4 s to 0.26 s.

  Output is byte-identical across the whole fixture corpus; the only behaviour changes are
  the two defect classes above, each now pinned by test.

- **A `<br>` inside a code span or fenced code block no longer injects a `newline_style`
  marker into the code.** A code context reproduces its content literally, so the marker was
  not syntax there -- it was a character in the user's code. `newline_style="backslash"` put a
  literal `\` inside the span (`<pre><code>A<br>B</code></pre>` emitted `A\` on its own line),
  and the two-space style injected trailing spaces. `CommonMark` gives a line ending inside a
  code span no hard-break meaning at all, so the two styles now agree byte for byte there. This
  is the rule the table-cell branch already applied for the same reason. A `<br>` *between* two
  code spans is an ordinary hard break and is unaffected.

- **A `<br>` at the end of any block no longer leaves a stray `\` under
  `newline_style="backslash"` ([#464](https://github.com/xberg-io/html-to-markdown/issues/464)).**
  3.11.6 claimed this was fixed, but the strip ran only inside the paragraph handler, so it
  covered `<p>...<br></p>` and nothing else. Every other way an inline run can end still leaked
  the marker: before a following block of any kind (`A<br><p>B`, the reporter's follow-up case,
  plus headings, lists, blockquotes, rules, tables and `<div>`), at the end of the document
  (`A<br>`), at the end of a `<div>`, list item, blockquote, `<section>`, `<details>`, `<figure>`
  or `<dl>`, and through nested containers.

  `CommonMark` gives a hard line break no meaning at the end of a *block*, not merely at the end
  of a `<p>`. The check now runs wherever a block boundary is actually reached. Handlers that
  walk their children into a fresh buffer were each an independent instance of the same defect,
  because `str::trim` cannot repair a trailing `"\\\n"` -- it removes the newline and leaves the
  backslash stranded.

  Worst case fixed: in a `<figcaption>` the caption is wrapped in emphasis *after* the marker is
  appended, so the leftover backslash landed between the text and the closing `*` and escaped it
  -- `<figure><figcaption>A<br></figcaption></figure>` produced `*A\*`, malformed Markdown rather
  than a merely visible artifact.

  A `<br>` that precedes real inline content still emits its break, a leading top-level `<br>`
  still opens a line ([#112](https://github.com/xberg-io/html-to-markdown/issues/112)), and
  `newline_style="spaces"` output is unchanged -- its leftover marker is invisible trailing
  whitespace.

- **Four cases where the Tier-1 fast scanner produced different Markdown than the Tier-2
  converter for the same input.** The library picks a tier automatically, so a divergence means
  the same document could convert two ways depending on which path it qualified for. Tier-2 is
  authoritative and was correct in all four.

  A custom element (any unknown tag containing `-`) was treated as block-level, so one appearing
  inline in flowing text split the surrounding paragraph, list item or blockquote and could leave
  an orphaned bullet or emphasis marker behind. Unknown elements are inline by default in HTML --
  there is no block-level custom element absent a CSS rule this converter does not apply -- so
  they now pass through inline, as the DOM walker always did.

  Named character references outside Tier-1's hot subset (`&notin;`, `&there4;`, `&sup1;`,
  `&para;`, `&trade;`) passed through as literal text instead of decoding. Tier-1 now falls back
  to the same full WHATWG named-reference table Tier-2 decodes against, rather than a hand-picked
  subset.

  `<strong><strong>x</strong></strong>` emitted `****x****`, which `CommonMark` does not parse as
  strong emphasis at all; redundant same-type nesting now collapses to one marker pair. A
  trailing whitespace run before a closing inline marker -- most often a decoded `&nbsp;` --
  stayed inside the markers instead of moving outside them.

  A leading whitespace run inside `<strong>`/`<em>` was deleted rather than moved outside the
  markers, so `<p>a<em>&nbsp;x</em></p>` lost the space entirely: `a*x*` where the DOM converter
  gives `a *x*`. The trim is still applied to `<a>` labels, which really are trimmed, and to
  `<code>`, which is verbatim.

- **The published crate no longer ships two tests that cannot compile.** Both read the
  `CommonMark` spec fixture from a path outside the crate root, which `cargo package` does not
  carry, so `cargo test` on the packaged crate failed to build. They are excluded from the
  package and still run from a repository checkout.

- **Leading whitespace at the start of a document is stripped, matching every later block.**
  `<p>&nbsp;a</p>` kept its space while `<p>x</p><p>&nbsp;a</p>` dropped it, so the same markup
  converted differently depending only on whether anything preceded it. The converter had no way
  to tell "start of the real output" from "start of a handler's private scratch buffer" -- every
  inline wrapper builds its content into a fresh buffer before splicing it in -- so a naive
  emptiness check silently changed every wrapper whose first child begins with whitespace.

- **Block children of a list item stay inside the item.** Fenced code blocks and block quotes
  were emitted without the continuation indentation that keeps them in the item, so they escaped
  the list entirely on reparse. The indent sums each open ancestor's actual marker width rather
  than assuming two spaces per level, since ordered markers vary in width (`1.` against `10.`).

- **List looseness is detected as the whole-list property it is.** One blank line anywhere
  between items makes a compliant reparse wrap *every* item in `<p>`, but detection only looked
  for a literal `<p>` and missed `div`, `blockquote`, `pre`, `table`, `hr` and `dl` children and
  loose nested sublists. Alongside it: a nested list that is an item's sole leading child no
  longer double-counts the parent marker's width and pushes its content past the four-space
  indented-code threshold; a block quote following an item's bare leading text is no longer
  forced onto a blank line, which had made that text round-trip as its own paragraph; a bare
  `<!-- -->` between two adjacent same-type lists is preserved, being the one signal that stops
  them merging; and the bare-marker check now honours the configured bullets instead of
  hardcoding `*`, `-` and `.`, having missed `+` from the default cycle.

- **A nested list keeps its line break when the text before it ends in whitespace.**
  `<li><strong>b </strong><ul><li>sub</li></ul></li>` collapsed to `- **b** * sub`, flattening
  the sublist onto the parent line so it stopped being a list at all and its content was lost.
  The "are we already at a bare marker" test was a two-byte suffix check, and a closing `**b**`
  plus its trailing space ends in the same two bytes as a real `*` bullet plus its space. Both
  tiers were wrong in exactly the same
  way, so the differential oracle could not see it; the check now requires the whole line to
  decompose into bare markers.

- **Non-ASCII bytes in a link destination are percent-encoded.** A raw byte at or above 0x80 is
  not valid URI syntax under RFC 3986, and compliant renderers encode it when writing an `href`.
  Only non-ASCII bytes are touched, so reserved ASCII punctuation in query strings is untouched.
  Same-document `#fragment` destinations are exempt: they must byte-match an element id this
  converter does not generate, and encoding one side of that pair would break the anchor.

- **Link-shaped text in image alt text and link labels is escaped.** Alt text is parsed as full
  inline content on reparse, so `<img alt="[foo](uri2)">` had its alt silently become a real
  nested link and lost the destination. Both ends of the pair are escaped, since escaping only
  the closing bracket leaves the outer `[` to be captured by a later `]` and the image then fails
  to form at all. A genuine nested `![alt](src)` inside link text is left intact.

- **A link or image with no visible text no longer turns its own href into emphasis.** Such an
  element falls back to the href as its label, and that fallback bypassed the text escaper, so a
  raw `*` or `_` in the URL round-tripped into real emphasis. Two destination-escaping gaps
  closed with it: a literal backslash before the paren escaping in an unbalanced-parens
  destination was not itself escaped, and a raw line ending inside an angle-bracket-wrapped
  destination is not valid `CommonMark` at all and is now folded to a space.

- **The Tier-1 fast scanner matches the Tier-2 converter on every fix above, and on a further
  run of divergences found by the differential oracle.** Because the library selects a tier by
  input shape, each divergence was a case where one document could convert two ways. Three
  suppressions were also removed from the oracle's allow-list, so it now generates those shapes
  freely instead of avoiding them; the four that remain are each documented in place.

- **A hard line break inside a link's visible text survives a round trip.** A `<br>` in a link
  label was collapsed to a space, so converting, rendering back to HTML, and converting again
  lost the break -- yet a hard break inside link text is perfectly legal `CommonMark`. Ordinary
  soft newlines from wrapped source text still collapse to a space, and a break at the very
  start or end of a label is still dropped, having no line to break to.

- **Four more places mistook inline content for a bare list marker.** The check was a two-byte
  suffix test, and a closing `**bold**` plus its space ends in the same two bytes as a real `*`
  bullet plus its space. A block quote after inline text in a list item lost its continuation
  indent and fell out of the item entirely on reparse, since `CommonMark` matches containers per
  line; a fenced code block was glued onto the preceding inline line, where the fence is not a
  valid opener; a `<div>` ran straight onto the previous text with no separator at all; and
  Tier-1's paragraph handler ran the check without first confirming an open list item, so
  top-level text merely ending in a hyphen and a space lost the blank line before the next
  paragraph. Several of these also omitted `+`, the third bullet in the default cycle, so they
  misbehaved at every third nesting level.

- **An unclosed `<p>` or `<div>` in a list item no longer swallows the next item.**
  `<ul><li><div></li><li>foo</li></ul>` converted to `- - foo`, turning two sibling items into
  one item holding a nested list, and `<ul><li><p>x</li><li>foo</li></ul>` converted to
  `- x- foo`, running both onto one line so the second stopped being a list item at all.
  Closing the tag explicitly was already correct, so this hit precisely the shape the HTML5
  parsing algorithm resolves with an implied end tag -- ordinary markup, not an edge case. The
  primary parser nests the following item under the unclosed element; the misparse detector
  that already re-parses such trees with the spec-compliant fallback only recognized a block
  nested under an *inline* ancestor, so a `div` or `p` never triggered it. Genuinely nested
  lists are unaffected.

- **Leading whitespace on soft-wrapped continuation lines no longer shrinks on every pass.**
  A run of spaces after a newline in block text collapsed to one space, but `CommonMark`
  strips a line's leading whitespace entirely when assembling a paragraph, so a renderer
  dropped the space that was kept and the next conversion saw one fewer -- the text never
  stabilized. Such runs now collapse to nothing, which is what a round trip already forces.
  Whitespace at a text node's own edge is untouched, since that is what keeps adjacent words
  apart.

- **A literal backslash in a link destination is no longer swallowed.** Two shapes lost data.
  A backslash before ASCII punctuation was consumed as a `CommonMark` escape of that character
  on reparse, so `href="\*"` came back as `href="*"`. A backslash at the end of a destination
  with no title after it merged with the closing parenthesis into an escaped `\)`, so the
  destination never terminated -- `href="x\"` reparsed as the literal text `[t](x)`, losing
  the href and the link structure and leaving raw brackets in the rendered output. Both are now
  escaped; a trailing backslash followed by a title's space, which is harmless, still is not.

- **The Tier-1 fast scanner no longer emits broken output for block children of a list item.**
  A `<div>` inside an `<li>` came out with no continuation indent, so its content fell out of
  the list on reparse; `<blockquote>`, `<table>`, `<dl>`, a `<p>` continuing existing text and a
  `<pre>` as an item's first content were each wrong in their own way. Under the shipped
  defaults Tier-1 is never reached for these, because metadata extraction and highlight styling
  both force the DOM converter first -- but with those disabled, all six shapes diverged.
  Rendering them correctly needs Tier-1 to defer its separator decisions the way the DOM
  converter does, which also entangles the loose-list heuristic, so the scanner now declines
  these shapes and falls back instead. No corpus coverage is lost: the new bail never fires
  across either parity corpus.

- **A lone significant character after a `<br>` is no longer dropped.** `<p>a<br>&nbsp;</p>`
  kept its non-breaking space, but the same markup with a newline after the `<br>` lost it --
  so whether content survived depended on nothing but whether the source HTML happened to be
  pretty-printed. A rendered `<br>` is always followed by a literal newline, which flipped the
  handler into a branch that discarded the run wholesale. Separately, a `<br>` inside a heading
  emitted the two-space hard-break marker, which has no meaning on a single line: a renderer
  collapses it, so the next conversion saw different bytes. It now emits one space.

- **`br_in_tables: false` is honoured for lists inside a table cell.** Both converters emitted
  a literal `<br>` between sibling `<li>` elements in a cell regardless of the option, so the
  setting silently did nothing for the shape it most often applies to -- a list in a cell, as
  MediaWiki sidebars produce. This also caused a round-trip instability: once a renderer
  flattens the list, that same `<br>` took the ordinary option path on the second pass and
  collapsed to a space, so the two passes disagreed.

- **A non-breaking space between two `<br>` tags is no longer dropped.** The paragraph
  pre-filter discarded a text node outright when trimming found it empty and both neighbours
  were empty inline elements -- and trimming is Unicode-aware, so a lone U+00A0 counted as
  empty even though it is visible content. It surfaced only on a second conversion, because the
  source spells it `&nbsp;` (six ASCII bytes) while a renderer re-serializes it as the literal
  character. Only genuine ASCII whitespace is dropped there now.

## [3.11.6] - 2026-08-28

Re-release of 3.11.5, which never reached any registry: the publish run failed and crates.io
still tops out at 3.11.4. Also carries the release-gate fixes below.

### Fixed

- **`<br>` runs emit one hard break each, and none trailing a block.** A run of consecutive `<br>`
  elements collapsed inconsistently, and a `<br>` immediately before a block boundary added a
  stray break to the output.

- **`packages/r/src/Makevars` is tracked instead of gitignored.** The file is alef-generated and
  alef-owned, but `packages/r/.gitignore` discarded it, so `alef verify --exit-code` -- the
  "Verify binding freshness" step of CI Rust -- failed on every run. `src/Makevars.win` stays
  ignored.

- **The Dart e2e before hook installs the pinned `flutter_rust_bridge_codegen`.** It hard-coded
  2.12.0 while the project pinned flutter_rust_bridge 2.13.0, so CI Dart aborted on alef's
  version-disagreement check. `[crates.dart] frb_version` now declares the pin explicitly next to
  the hook that has to match it.

### Removed

- Dropped a stray Python `enum_module` from the node and java e2e call overrides; neither emitter
  reads it.

## [3.11.5] - 2026-08-25

### Changed

- Regenerated all language bindings on alef 0.68.0.

### Changed

- **`alef` pinned to `0.67.5` (from `0.67.2`) and the tree regenerated.** Every one of the 609
  regenerated files carries a real content change; none is a bare provenance restamp. The
  substantive ones: the tagged `VisitResult` wire shape is now decoded consistently across
  languages -- Swift gains hand-written `Codable` conformance keyed on `{type, output}`, and the
  PyO3 bridge reads the `output` member instead of treating the whole envelope as the payload;
  Java's `VisitorBridge` derives its `NodeContext` field offsets from the declared `MemoryLayout`
  rather than from hand-maintained byte constants, and a throwing visitor callback now returns
  `VISIT_RESULT_CONTINUE` instead of `VISIT_RESULT_ERROR`, so one failing callback no longer
  aborts the whole conversion; Kotlin/Android e2e imports lose a doubled
  `io.xberg.android.io.xberg.android` package prefix; the Zig e2e suite asserts exact output
  instead of trimming it first; and the TypeScript snippets optional-chain nullable metadata
  (`result.metadata?.links`).

- **Snippet gap checking is configured, so `--strict` runs it instead of erroring.**
  `[workspace.docs.snippets]` gained `docs_dirs` and `required_languages`, mirroring the flags
  `task docs:snippets:gaps` already passed. Neither was set, so the gap pass -- unreferenced
  snippets and missing language variants -- was skipped entirely, and alef 0.67.5 correctly
  refuses to let a strict run pass on a check that compared nothing. `alef.toml` is hashed as a
  generation input as a whole rather than per-surface, so adding two snippet-only keys restamped
  the provenance marker of all 5044 content-verified files without changing a single line of
  generated content.

### Fixed

- **`RustBridgeC.h` is whole again after a regeneration silently truncated it by 193 lines.**
  The header is a concatenation of `SwiftBridgeCore.h` and `html-to-markdown-rs-swift.h`, both
  produced by `cargo build -p html-to-markdown-rs-swift`. When those build artifacts are absent
  -- as they are right after `alef all --clean` wipes them -- alef emits the crate half alone and
  reports success, so the file lost `RustStr`, `__private__FfiSlice`, every `__private__Option*`
  struct and `__swift_bridge__null_pointer`. Regenerating against real build artifacts restores
  content byte-identical to the pre-upgrade file. Nothing about this was an alef 0.67.5 behavior
  change; an earlier entry in this section wrongly credited it as one.

- **The eleven hand-written TypeScript and WASM snippets now declare which session validates
  them.** Two sessions (`node` and `wasm`) both validate `typescript` snippets, and alef 0.67.5
  stopped guessing between them; the six snippets under `docs-site/src/snippets/typescript/` now
  carry `target: node` and the five under `docs-site/src/snippets/wasm/` carry `target: wasm`.
  These eleven had never actually been compiled -- they were reported as errors, not failures --
  which is how the WASM visitor example kept four implicitly-`any` callback parameters that
  `noImplicitAny` rejects the moment the snippet is really typechecked. It is now a `typescript`
  fence with its parameters annotated, matching the `node` example beside it and every generated
  WASM snippet. Snippet results move from 4550 passed / 11 errored to 4561 passed / 0 errored.

- **A skipped `publish-crates` no longer reads as a passing gate, and a release that published
  nothing can no longer report success.** Eight places in `.github/workflows/publish.yaml` gated
  downstream build, publish and release-promotion jobs on
  `needs.publish-crates.result != 'failure'`. That expression is TRUE when the dependency was
  *skipped*, and `publish-crates` skips for two opposite reasons: the version is already on
  crates.io (a re-run or a resumed release, where downstream must proceed) or an upstream gate
  such as version validation or crate packaging failed (where downstream must not). `result`
  alone cannot separate them, so every one of those conditions was gating on nothing --
  tree-sitter-language-pack v1.15.5 promoted a GitHub release to `Latest` with 40+ failed jobs and
  every registry publish skipped, and still reported success.

  A new always-running `crates-gate` job resolves the ambiguity once, into an explicit
  `outcome` (`published` / `already-present` / `not-required` / `dry-run` / `blocked`) and an
  `ok` flag that every consumer now tests instead of `result`. Because the job always runs, its
  outputs always exist; because it never fails, depending on it cannot skip a consumer. The
  legitimate already-published path stays exactly as permissive as before, and only the
  gate-failed path is newly blocked.

  `release-report` also stopped being able to see this class of failure: it inspected only jobs
  whose conclusion was `failure`, so a run where every publish job merely *skipped* passed. It now
  judges each target on its own result and treats `skipped` as a failure unless the target is not
  enabled for this release or its registry probe already found this exact version published.

- **The R vendoring script deleted the core crate's `[lints]` without inlining anything, so the
  vendored copy compiled under a different lint configuration than its sources.**
  `scripts/ci/r/vendor-core-crate.py` copies `crates/html-to-markdown/` out of the workspace and
  rewrites `workspace = true` inheritance into explicit values — except for `[lints]`, which it
  stripped outright. The `[workspace.lints.rust]` `unexpected_cfgs` check-cfg allowlist went with
  it; that allowlist is what declares the crate's own `#[cfg(...)]` gates as expected cfg names, so
  every gate in the vendored copy became an `unexpected_cfgs` diagnostic — silent in a default
  build, a hard error under `RUSTFLAGS="-D warnings"`. `missing_docs`, `unused_must_use`,
  `unsafe_code = "forbid"` and the whole `[workspace.lints.clippy]` policy were dropped the same
  way. The script now materializes the entire `[workspace.lints]` sub-tree into the vendored
  manifest verbatim, matching the fix alef shipped in 0.67.x for `alef publish prepare`. The
  extracted text is re-parsed and compared against the authoritative parse of the root manifest, so
  a mis-extraction fails loudly rather than silently emitting different lints. Nothing is specific
  to a lint or cfg name: adding a lint or a check-cfg entry to the workspace needs no change here.
  The script is not replaced by alef's implementation because it runs from `packages/r/configure` at
  R configure time — including on end-user machines installing the source tarball from GitHub, where
  only Python 3 and a Rust toolchain exist.

- **The coding-agent plugin version gate never ran on the commits that cause drift.**
  `plugin/` sat at 3.11.3 while core shipped 3.11.4, so all 13 version declarations under
  `plugin/` — OpenCode, Hermes, Claude Code, Cursor, Codex, Gemini, Kimi, Factory — were a
  release stale. The checker for this already existed (`scripts/sync_plugin_version.py --check`,
  run by `CI Plugin`), but `ci-plugin.yaml`'s `paths:` filter did not list `Cargo.toml`. A
  release commit bumps `Cargo.toml` and nothing under `plugin/`, so the workflow was never
  triggered on exactly the commits that cause drift, and the gate reported nothing rather than
  failing. `Cargo.toml` and `.task/tools/version-sync.yml` are now in the filter, so any core
  bump re-runs the gate. `sync_plugin_version.py` also grew `--expect <version>`, which asserts
  that core *and* the plugin both equal the version being released; `publish.yaml`'s
  `validate-versions` job runs it against the tag, so a drifted plugin now fails the release
  instead of publishing a bundle that lags the version it claims to be. The 13 stale declarations
  are re-synced to 3.11.4.

- `task test` now runs. `tools/benchmark-harness/examples/profile_visitor.rs` imports
  `NoOpVisitor`, which only exists under the harness's `visitor` feature, but the task builds the
  workspace with `--no-default-features` — and `cargo test` builds examples. The example is now
  declared with `required-features = ["visitor"]`, matching the existing `testkit` examples.
  Before: `task test` exited 201 with `E0432: unresolved import
  html_to_markdown_bench::bench::NoOpVisitor`. After: exit 0, 1422 passed / 0 failed / 126 suites.
- `task test:ci`'s `--exclude benchmark-harness` excluded nothing — that is the crate's *directory*
  name; the package is `html-to-markdown-bench`. cargo reports an unmatched exclude as a warning,
  not an error, so the dead flag survived silently and the crate was covered on every run. Fixed
  in all four `rust:test:ci` invocations and sabotage-verified: with the bench crate deliberately
  made unparseable, `--exclude benchmark-harness` still fails the build while
  `--exclude html-to-markdown-bench` never compiles it.
- Dropped the trailing `fix-cjs-named-exports.mjs` step from `task alef:generate`. `@napi-rs/cli`
  3.x already emits an explicit `module.exports.<name> = nativeBinding.<name>` for every runtime
  export, so named ESM imports resolve without it (#450). The script had also gone stale in a way
  that corrupts output: napi now declares enums as `export declare const enum X`, which its regex
  reads as a const *named* `enum`, appending a bogus `module.exports.enum = nativeBinding.enum`
  and re-exporting only 7 of the 20 names it used to cover.
- `alef` pinned to `0.67.2` (from `0.66.0`) and the tree regenerated. Four real codegen changes
  landed, plus a hash restamp of the fixture-driven snippet corpus:
  - `DocumentNode.children` and `DocumentNode.annotations` are no longer `@Nullable` in the Java
    record. Both mirror a plain `Vec<T>` on the Rust side carrying
    `#[serde(default, skip_serializing_if = ...)]`; the Rust value is never `null`, so the old
    signature described a state the binding cannot produce. A compact constructor now normalizes
    a `null` deserialization to `List.of()`. All 283 `e2e/java` tests still pass — no test in this
    repo asserted the old null-valued contract.
  - `flutter_rust_bridge` moved to `2.13.0` across `packages/dart/{pubspec.yaml,rust/Cargo.toml}`,
    `Cargo.lock`, and the `frb_generated.*` outputs. `e2e/dart/pubspec.lock` also picked up the
    `ffi: ^2.2.0` entry its `pubspec.yaml` already declared. All 283 `e2e/dart` tests pass.
  - `crates/html-to-markdown-ffi/build.rs` was regenerated with the header-export path split into
    named helpers, which drops the `clippy::collapsible_if` hit.
    `cargo clippy -p html-to-markdown-ffi --all-targets -- -D warnings` is clean and all 283
    `e2e/c` tests pass.
  - `packages/swift/Sources/RustBridgeC/RustBridgeC.h` is now clang-formatted. Token-normalized,
    the file is identical to the previous content apart from `#include` ordering.
  - The generated snippets under `docs-site/src/snippets/generated/` now print the fields their
    fixture actually asserts (`result.content`, `result.metadata.document.author`, ...) instead of
    dumping the whole result object. This exposes an alef codegen bug: the emitter dereferences
    optional intermediate fields without guarding them, so the 24 `metadata_*` TypeScript
    snippets now fail `tsc` with `TS18048: 'result.metadata' is possibly 'undefined'` and the
    three Open Graph / Twitter-card Swift snippets fail on
    `expression implicitly coerced from 'Optional<RustString>' to 'Any'`. These are generated
    files, so they cannot be corrected here — the fix belongs in alef's snippet emitter.

  `alef verify --exit-code` and `poly lint .` both pass on the resulting tree.

- `alef` pinned to `0.65.0` (from `0.64.0`) and the tree regenerated with the released crates.io
  build (`cargo install alef --version 0.65.0 --locked`), not a local `alef` checkout — unlike the
  0.64.0 repin, this is not byte-identical: `packages/elixir/native/html_to_markdown_nif/src/lib.rs`,
  `packages/elixir/lib/html_to_markdown/native.ex`, `packages/r/src/rust/src/lib.rs`,
  `packages/r/R/extendr-wrappers.R`, `packages/r/NAMESPACE`,
  `packages/java/src/main/java/io/xberg/htmltomarkdown/{ConversionOptions,DocumentNode}.java`, and
  the Dart/Swift structure e2e tests all picked up real codegen differences. `packages/r/NAMESPACE`
  and `packages/r/R/extendr-wrappers.R` gained `.alef-ownership.toml` records (formats with no
  comment syntax, so they cannot carry an inline `alef:hash:` marker) alongside the existing
  `packages/r/DESCRIPTION` entry. `alef verify --exit-code` and `poly lint .` both pass on the
  resulting tree.

- `[workspace.docs.snippets]` `dirs` only listed `docs-site/src/snippets/generated`, so the 96
  hand-written snippets under `docs-site/src/snippets/<lang>/{getting-started,error-handling,
  table-extraction,metadata,visitor}` — the ones `usage.mdx`, `errors.mdx`, `tables.mdx` and
  `visitor.mdx` actually import — were never discovered by `alef snippets check`. Nothing
  validated they still compile and nothing regenerates them, so a reader could be shown code that
  no longer builds. `alef e2e snippets-migrate docs-site/src/snippets` confirms none of the 96
  have a fixture-generated equivalent: the generated corpus is organized by fixture topic
  (`conversion`, `edge-cases`, `metadata`, `options`, `real-world`, `result`, `smoke`,
  `structure`, `visitor`) and emits a test-style snippet (frontmatter + `main()` + `print(result)`)
  aimed at compile validation, not the field-access idiom (`result.content` / `result.tables` /
  `result.metadata`) the narrative docs teach — so they're kept rather than replaced. Added each
  language's snippet root (not a glob) to `dirs`; this reaches only the hand-written topic dirs
  and does not overlap `generated/<lang>/...`, which lives one level deeper under its own subtree.

- `alef adopt "**/*" --converged-only --write` claimed ownership of 13 files that already matched
  (or, for `packages/r/DESCRIPTION`, could only ever match via `.alef-ownership.toml` since its
  format carries no comment syntax) current `alef generate` output byte-for-byte apart from the
  provenance marker: `.clang-format`, `packages/r/DESCRIPTION`,
  `crates/html-to-markdown-ffi/cmake/html-to-markdown-ffi-config.cmake`,
  `packages/dart/rust/Cargo.toml`, `packages/{elixir,java,kotlin-android,zig}/README.md`,
  `packages/ruby/html_to_markdown.gemspec`, and
  `test_apps/{node,wasm}/pnpm-workspace.yaml`, `test_apps/php/phpunit.xml`,
  `test_apps/python/pyproject.toml`. 11 of those had drifted (stale `3.11.0` version pins in
  READMEs/gemspec/build metadata, indentation, quoting) — `adopt` stamps the marker and leaves the
  drifted content as-is by design; a later `alef generate` will resync it now that ownership is
  recorded. `packages/kotlin-android/gradle/wrapper/gradle-wrapper.jar` could not be adopted even
  with `--write`: alef refuses to stamp or ownership-record any binary file, so it stays unmarked
  — a tool limitation, not a decision made here.

- Left every create-once seed alone: `config.m4`, `crates/html-to-markdown-php/src/HtmlVisitor.php`,
  `e2e/java/pom.xml`, and `e2e/kotlin_android/build.gradle.kts` (the four paths `alef generate`
  refused to write) plus ~20 siblings alef's own `adopt` scan flagged the same way (Cargo/npm/gem
  scaffold manifests under `crates/html-to-markdown-node/{package.json,index.js,npm/*/package.json}`,
  `packages/ruby/spec/html_to_markdown_spec.rb`, `packages/java/checkstyle-suppressions.xml`,
  `packages/csharp/HtmlToMarkdown{,.Runtime}/*.csproj`, `Package.swift` /
  `packages/swift/Package.swift`, `packages/dart/{pubspec.yaml,CHANGELOG.md,example/*.dart}`,
  `packages/zig/{build.zig,build.zig.zon,test/*.zig}`). Each is a create-once seed: alef writes it
  only when absent, and every one on disk is substantial hand-maintained content (24-431 lines),
  not a placeholder. `alef adopt --clobber-create-once-seeds` would stamp them, but that flag
  explicitly consents to alef replacing the file with a placeholder seed on the next `generate` —
  the wrong trade for real content, so none of these were adopted.

- Ran `alef generate` then `alef all` to actually resync the 13 adopted files and everything else
  the `[workspace.docs.snippets]` `dirs` config edit (above) left stale: every alef-owned file's
  provenance marker fingerprints the full config, so that one-line config change invalidated the
  whole generated tree's markers even where rendered content hadn't changed. 5 of the 13 adopted
  files had real drift: `packages/dart/rust/Cargo.toml` (hardcoded `version = "3.11.4"` instead of
  `version.workspace = true`, and missing the `[lints.clippy]` block its swift sibling carries —
  meaning `dbg_macro`/`print_stdout`/`print_stderr` were silently unenforced for that crate),
  `packages/ruby/html_to_markdown.gemspec` (stale file-exclusion glob),
  `crates/html-to-markdown-ffi/cmake/html-to-markdown-ffi-config.cmake` (indentation), and
  `test_apps/python/pyproject.toml` and `.clang-format` (missing hash marker / stale dependency
  pin). The other 8 already matched generated output byte-for-byte. Checked every other workspace
  member's `Cargo.toml` for the same missing-lints gap: `packages/dart/rust/Cargo.toml` was the
  only one lacking `[lints]` entirely; all ten others already carry their own `[lints.clippy]`
  block or `workspace = true`. `alef verify --exit-code` now reports zero stale bindings; the
  remaining failure is ~99 pre-existing scaffold files (`LICENSE`, `.gitignore`, `package.json`,
  gradle wrapper files, …) that were never run through `alef adopt` and so carry no provenance
  marker — a separate, long-standing gap outside this fix's scope, left for a future adoption pass.

- Compile-validated the 96 hand-written snippets `alef generate` (above) had just made visible to
  `alef snippets check`. 6 C snippets under `docs-site/src/snippets/c/` still declared
  `HTMConversionResult *`/`HTMConversionOptions *`/etc. as raw pointer types and compared them
  against `NULL`; the FFI moved to opaque `HTMAlefHandle` (`uint64_t`) handles a while ago and
  these were never updated, so 5 failed to compile and a 6th (`visitor/basic_visitor.md`, comparing
  `HTMHtmVisitor *` against a type that no longer exists) was misclassified as an unresolved
  dependency by alef's error-pattern heuristic rather than a real failure. Rewrote all 6 to the
  handle-based idiom (`HTMAlefHandle result = htm_convert(html, 0);`). One zig snippet
  (`docs-site/src/snippets/zig/visitor/basic_visitor.md`) used `orelse` on `c.htm_convert`'s return
  value as if it were optional; the binding returns a plain `u64` (0 on failure), so `orelse` no
  longer type-checks — replaced with an explicit `if (result == 0) return error.ConvertFailed;`.
  Both languages are now 578/578 with zero failures and zero unavailable.

  The hand-written `docs-site/src/snippets/typescript/*` snippets import the published package
  name (`@xberg-io/html-to-markdown`), same as a real consumer would, but the node and wasm
  snippet sessions' `cwd` IS that package (or its sibling), so nothing had ever installed it into
  its own `node_modules` the way `pnpm add @xberg-io/html-to-markdown` would for a consumer.
  Added a `before` hook to both sessions in `alef.toml` that self-links the package
  (`mkdir -p node_modules/@xberg-io && ln -sfn ../.. node_modules/@xberg-io/html-to-markdown` for
  node; the wasm session links to the sibling node crate since the snippets never import the wasm
  package by name). Verified by hand that this actually fixes module resolution — `tsc --noEmit`
  against alef's own synthesized session `tsconfig.json` passes clean once the symlink exists — but
  `alef snippets check` itself still reports these same 11 snippets (6 hand-written × node and wasm
  sessions, minus one overlap) as `unresolved_dependency` even after the fix and with `--cache off`.
  This reproduces identically across repeated fresh runs, so it isn't cache staleness; it looks like
  a defect in how alef disambiguates a single fence-tag ("typescript") claimed by two sessions when
  validating frontmatter-less (hand-written, not fixture-generated) snippets, since the equivalent
  283-file fixture-generated batch (which does carry `target: node` frontmatter) passes cleanly
  through both sessions. Not fixed from this repo; documented here for upstream triage.

  5 hand-written `docs-site/src/snippets/kotlin_android/*` snippets (`import io.xberg.android.*`)
  also remain `unresolved_dependency` even after `./gradlew assembleDebug -Palef.skipHostJni=true`
  succeeds and the target classes are confirmably present in
  `packages/kotlin-android/build/intermediates/aar_main_jar/debug/syncDebugLibJars/classes.jar`.
  `./gradlew publishToMavenLocal -Palef.skipHostJni=true` gets further (compiles and bundles the
  release AAR) but fails at `signMavenPublication` for lack of a configured signatory — this
  environment has no GPG signing key, and none was fabricated. Left unresolved rather than faked;
  the 6th kotlin_android snippet (`visitor/basic_visitor.md`) is comment-only (the visitor pattern
  isn't supported on this binding yet) and reports `PASS` trivially.

  `docs-site/src/snippets/feedback.md` was deliberately left out of `[workspace.docs.snippets]`
  `dirs`: it's a shared MDX `<Content>` partial (a static "found a bug?" callout imported by ~10
  doc pages), not a per-language code sample — it contains zero code fences, so there is nothing
  for `alef snippets check` to discover or validate there.

  Final `alef snippets check --strict --cache off` result across all 16 configured languages:
  4571 total, 4555 passed, 0 downgraded, 0 failed, 0 skipped, 0 errors, 16 unavailable (kotlin 5,
  typescript 11, both detailed above). `--strict`'s bar is zero *incomplete*
  (`Skip | Unavailable | Downgraded`), not zero failures, so this run still fails that bar —
  reported plainly rather than as a pass.

- The 5 kotlin_android `unresolved_dependency` snippets above were misdiagnosed: the real cause
  was not the classpath or the unsigned AAR, it was plain Kotlin syntax. `error-handling/basic_usage.md`,
  `table-extraction/basic_extraction.md`, and `metadata/basic_extraction.md` opened with (or ended
  in) bare top-level statements — `try { }`, a top-level `for` loop, trailing `println(...)` calls
  — which a plain `.kt` file (unlike a `.kts` script) does not permit outside a function body;
  `kotlinc` rejected them with `error: syntax error: Expecting a top level declaration`, which
  alef's error-pattern heuristic then classified as `unresolved_dependency` rather than a real
  failure, the same misclassification already seen and corrected for the C/zig visitor snippets
  above. `getting-started/basic_usage.md` and `getting-started/with_options.md` were already
  declarations-only (`val`/`import`) and passed without change. Wrapped the executable body of the
  three broken snippets in `fun main() { ... }`, matching the convention the Java/C# snippets
  already use for the same requirement. `alef all --clean` (0.65.0, with `packages/kotlin-android`
  built via `cargo build --release -p html-to-markdown-rs-jni` +
  `./gradlew assembleDebug -Palef.skipHostJni=true`) now reports kotlin at 0 failed / 0 unavailable
  across all 3 (all 6 including the two already-passing and the trivial visitor snippet). The
  typescript disambiguation defect (11 unavailable, both `node` and `wasm` sessions claiming the
  same `typescript` fence tag) is unchanged at 0.65.0 and remains upstream-triaged, not fixed here.
