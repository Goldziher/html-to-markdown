---
title: "Changelog archive: 3.16.0–3.14.0"
---

## [3.16.0] - 2026-10-02

### Added

- **`ConversionOptions::inline_data_media`** chooses what to write for an image or embedded media
  element whose address is an inline `data:` URL, so the payload no longer has to land in the
  output. `keep` (the default) writes the URL as before. `alt_text_only` writes the alt text: the
  title of an inline `<svg>`, the fallback content of `<video>` and `<audio>`, nothing for an
  `<iframe>`. `drop_element` writes nothing. It covers `<img>`, `<graphic>`, inline `<svg>`,
  `<video>`, `<audio>` and `<iframe>`. With either of the last two choices, a real address on the
  element wins over the `data:` one: a lazy-load attribute or `srcset` candidate on `<img>`,
  another address attribute on `<graphic>`, a nested `<source>` on `<video>` and `<audio>`, and
  a `<source>` of the `<picture>` that holds an `<img>`. The document structure follows the
  markdown: no image node for a dropped element, and no address when only the alt text is
  written. A link whose only content the option removed is dropped with it, instead of turning
  into a link labelled with its own address. A link whose own address is a `data:` URL gets the
  same choice: `alt_text_only` and `drop_element` both write the link's text with no destination,
  since a link has no caption separate from its text the way an image has alt text, and that text
  is never dropped along with the address. Extracted images do not change. The CLI takes it as
  `--inline-data-media` (#528).

### Changed

- **WASM: assigning `null` or `undefined` to five optional fields now throws.** The fields are
  `WasmConversionOptions.visitor`, `WasmConversionOptionsUpdate.visitor` and `preprocessing`,
  `WasmConversionResult.document` and `WasmImageMetadata.dimensions`. They borrow the value you
  assign instead of taking it, and an assignment of `null` cleared them before. To unset one, call
  `clearVisitor()`, `clearPreprocessing()`, `clearDocument()` or `clearDimensions()`.

- The `alef` pin in `alef.toml` is now 0.101.0, the version that generated the committed bindings.
  Regenerating with 0.97.0 dropped the Go binding's `runtime.LockOSThread` calls.

- The FFI Symbols CI gate now fails when a detector matches no call site, and names the silent
  language. Before, a detector that stopped matching read exactly like a clean pass, so a
  restyled binding dropped out of the diff with every check green. Each detector must also match
  in every one of its required roots: the binding package for C#, Java, Go and Zig, and both
  `e2e/c` and `test_apps/c` for C. So the alef-generated `e2e/zig` tests cannot keep a silent Zig
  binding looking covered, and the vendored Go copy of the C header cannot do the same for C. The
  PHP comparison fails the same way once the extension exports a function and its probe root is
  missing. The `--json` summary now carries the per-language counts, the per-root counts and the
  silent detectors.

- CI E2E now ends in one `E2E result` job that fails unless every other job in the workflow
  passed or was skipped because its path filters did not match. Before, a skipped leg left the
  run as green as a passing one, whatever the reason for the skip, and no single check covered
  every leg. Each leg's path filter condition is now written once, as an output of the change
  detection job, and both the leg and the result job read that output. A script test fails when a
  job is added to the workflow without being listed in the result job.
- Removed an unused copy of the `<a>` handler that no conversion path called. Links are
  converted by the one live handler, as before; output does not change.

### Fixed

- **Text after a `<center>`, `<search>`, `<hgroup>` or `<dialog>` in a table cell joined it
  (#692).** `<table><tr><td>a<center><h2>h</h2>x</center>y</td></tr></table>` gave `| a h xy |`
  in both converters, and the full converter wrote `| a h x  y |` for a `<dialog>`. These convert
  like a `<div>` but were not counted as blocks, so the text after them got no cell break. They
  now give `| a h x y |`, or `| a<br>h<br>x<br>y |` with `br_in_tables` on, as a `<div>` does. In a
  list item, text after one of them now starts a paragraph in the item, as after a `<div>`, instead
  of continuing the container's last line or leaving the list.
  Counting them as blocks changes three more outputs, again to match a `<div>`. With
  `newline_style: backslash`, the hard break before one of them is dropped: `<p>a<br><center>b</center>c</p>`
  gives `a`, `b` and `c` as paragraphs instead of `a\` then `b`. A dialog in a link label is set off
  by spaces, so `<a href="u">l<dialog>b</dialog>m</a>` gives `[l b m](u)` instead of `[lb m](u)`.
  Bold or italic around one of them is closed before it and opened again after it:
  `<p><b>a<center>b</center>c</b></p>` gives `**a**`, `**b**` and `**c**` instead of one bold run
  across three paragraphs. Visitors now get `is_inline` false for these four tags.
- **Text before a `<dialog>` ran into it (#692).** `<p>a<dialog>b</dialog>c</p>` gave `ab`, then
  `c`, and `<td>a<dialog>b</dialog>c</td>` gave `| ab  c |` in the full converter and `| a bc |` in
  the fast one: the dialog wrote no break before its content. A dialog now converts like a `<div>`, so the paragraph gives `a`,
  `b` and `c` as three paragraphs, the cell gives `| a b c |` in both converters, and in a list
  item the dialog content starts a paragraph in the item.
- **Plain text output joined a `<center>` or `<dialog>` to the text after it (#692).**
  `<p><b>a<center>b</center>c</b></p>` with `output_format: plain` gave `abc` on one line. Plain
  output now starts both on their own line, as it does for a `<div>`: `a`, `b` and `c`.
- **A `<details>`, `<summary>`, `<menu>` or `<legend>` joined the text around it (#692).** In a
  table cell, `<td>a<menu>b</menu>c</td>` gave `| ab  c |` and a summary gave `| a**b**  c |`. In
  a list item, the text after a details left the list, and a summary, menu or legend ran into the
  text before it. In a paragraph, `<p>a<summary>b</summary>c</p>` gave `a**b**`, then `c`. A
  details and a menu now convert like a `<div>`, and a summary or legend is placed the way a
  `<div>` is placed and stays bold: the cell gives `| a b c |` or `| a **b** c |` in both
  converters, the list item keeps `b` and `c` as paragraphs in the item, and the paragraph gives
  `a`, `b` and `c` apart. As with the tags above, these four now count as blocks: the backslash
  break before them is dropped, bold around a details, summary or menu is closed and reopened,
  plain output starts a menu or legend on its own line, and visitors get `is_inline` false for them. A summary that holds
  a list with a quote in it, inside a details in a list item, now stays in the item, so with the
  default or tab list indent its bold markers no longer span the quote.
- **A block in a heading ran into the words around it.** `<h1>a<div>b</div>c</h1>` gave `# abc`
  in the full converter and `# a b c` in the fast one. Both now give `# a b c`, for a `<div>` and for
  every tag that converts like one, a details and a menu included. In the fast converter, a block inside a summary, figcaption or
  table caption now writes its break into that element's text, so a rustdoc heading such as
  `impl Any for T<div class="where">where T: ...</div>` gives `for T where` instead of `for Twhere`.
- **In a heading in a table cell, the text after a block ran into it.**
  `<td><h2>a<div>b</div>c</h2></td>` gave `| a bc |`. The text after the block now gets the cell
  break, so the cell gives `| a b c |`, or `| a<br>b<br>c |` with `br_in_tables` on. The fast
  converter leaves this shape to the full converter.
- **In a link label in a table cell, the text after a block ran into it.**
  `<td><a href="u"><span>a<div>b</div>c</span></a></td>` gave `| [a bc](u) |`. The text after the
  block now gets the cell break, as in a heading in a cell, so the cell gives `| [a b c](u) |`, or
  `| [a<br>b<br>c](u) |` with `br_in_tables` on.
- **Plain text output left a list item marker alone on its line.** `<ol><li><div>a</div>b</li></ol>`
  with `output_format: plain` gave `1.` on a line of its own, then `a` and `b`. A block that opens a
  list item now starts on the marker line, so the item gives `1. a`, then `b`.
- **The fast converter dropped the bold of a `<legend>` and split a link label at a `<summary>`.**
  `<legend>x</legend>` gave `x` where the full converter gives `**x**`, and
  `<a href="u">l<summary>b</summary>m</a>` gave a link label across three paragraphs where the full
  converter gives `[l b m](u)`. The fast converter now writes a legend in bold, as it does a summary,
  and leaves a summary inside a link to the full converter, as it already did for a `<div>`.
- **The WASM binding used up a visitor handle on assignment, and `convert()` ignored
  `options.visitor` (#517).** Assigning a `WasmVisitorHandle` to `WasmConversionOptions.visitor`
  moved it into the options, so a second options object could not take the same handle. The
  setter now borrows the handle. `convert()` now uses `options.visitor` when you pass no visitor
  argument, and a visitor argument still wins over the options field.
- **A line break on its own source line became a paragraph break (#683).** `First\n<br>\nSecond`
  gave `First\n\nSecond`, two paragraphs: the newline before the `<br>` put its hard-break marker
  on a line of its own, and an empty line ends a paragraph. The break now ends the line of the text
  before it, `First  \nSecond`, in both converters and with both newline styles. This also holds
  when that text ends inside an element that writes no line end, such as `<span>`, `<font>` or a
  custom element: `<span>First\n</span><br>Second`.
- **A table cell ignored `escape_underscores` and `escape_asterisks` (#638).**
  The full converter always escaped `_` and `*` in a cell, so
  `<table><tr><td>sample_value</td></tr></table>` gave `sample\_value` while
  `<p>sample_value</p>` gave `sample_value`. Every escape option now acts the same in a cell as
  anywhere else. A `|` in a cell is still escaped whatever the options say, and with
  `escape_ascii` on and `escape_misc` off it is now escaped once, as `\|`, instead of as `\\|`,
  which some renderers show with a stray backslash.
- **A `|` in a code span, link or image in a table cell broke the table.**
  `<table><tr><td><code>a|b</code></td></tr></table>` gave ``| `a|b` |``. GFM splits a row on
  a pipe in a code span too, so the row no longer matched the delimiter row and the whole table
  became a paragraph. A pipe in a link destination or title, an image description, preformatted
  text or a code span in a nested table did the same. In Markdown output every pipe in a cell is
  now escaped as `\|`, which GFM reads as `|` in a code span too. In Djot output a pipe in a
  link or an image in a cell is now escaped as `\|`, and a pipe in a verbatim span stays bare,
  because Djot does not split a row there.
- **An empty task item lost its checkbox on GitHub.** `<ul><li><input type="checkbox"></li></ul>`
  gave `- [ ]`. GFM reads a checkbox only when content follows it, so cmark-gfm and markdown-it
  showed the text `[ ]`. It now gives `- [ ] &#32;`, a checkbox: the character reference renders
  as a space. Djot output does not change.
- **Text next to a paragraph, heading or other block in a table cell joined it (#645).**
  `<table><tr><td><p>a</p>b</td></tr></table>` gave `| ab |`, and so did a heading, a `<div>`,
  a list or a code block before the text. Text before a heading or a code block joined it too.
  A block in a cell is now separated from the cell content before and after it by the cell break:
  `| a b |`, or `| a<br>b |` with `br_in_tables` on. A block at the start of bold, a code span or
  another inline element still joins the text before that element, and a block inside a heading
  still joins the text next to it.
- **The two converters wrote a quote or a paragraph in a table cell differently (#647).** A quote
  in a cell now has no `>` marker in either converter, as a heading, a list and a code block in a
  cell have none: `<blockquote>a</blockquote>b` gives `| a b |`. The fast converter wrote a
  paragraph after other cell content as `<br>` with `br_in_tables` off; it now writes a space, as
  the full converter does.
- **The fast converter dropped a line break that no element encloses (#679).**
  `a<br>1) t` gave `a1) t`, so the two lines joined. The fast converter now writes the break as
  the full converter does, `a  \n1\) t`, the same as when the input sits in `<body>`. The line
  after a hard break also no longer keeps a leading space: `<div>a<br> b</div>` gives `a  \nb` in
  both converters.
- **Hard breaks that differed between the converters or lost their place.**
  A whitespace character reference after a break (`a<br>&#10;b`) no longer makes a paragraph
  break in the fast converter. A break in a heading followed by a space
  gives one space (`## a b`). With the backslash newline style, a line in a list item that holds
  only a break keeps the item's indent (#681). Wrap no longer cuts a list item's text at a `===`
  line left of the item's column, which dropped the hard breaks after it (#680), and keeps a
  hard break right before such a line.
- **Two ordered lists next to each other became one list (#666).**
  `<ol><li>a</li></ol><ol><li>b</li></ol>` gave `1. a\n\n1. b`, which CommonMark and Djot read
  as one list, because a blank line does not end a list. An ordered list that follows an ordered
  list with only blank lines between them now writes the other delimiter, so it gives
  `1. a\n\n1) b`, also when a section, article, figure or similar element wraps either list. A
  list written as text, in a heading or between inline markers, keeps `.`.
- **In Djot output a nested list directly under its item's text was text (#670).**
  Djot needs a blank line before a list that follows text, so `- a\n  * b` is one paragraph
  there. A nested list after its item's text now starts after a blank line in Djot output:
  `- a\n\n  * b`.
- **An empty nested list item after text turned the text into a heading (#667).**
  `<ol><li>a<ul><li></li></ul></li></ol>` gave `1. a\n   -\n`: an empty item cannot interrupt a
  paragraph, so the lone `-` read as a heading underline and the item was lost. A nested list whose
  first item writes nothing on its marker line now starts after a blank line: `1. a\n\n   -\n`.
  An empty item after text directly inside its list does the same: `<ul>t<li></li></ul>` gives
  `t\n\n-\n`, not `t\n-\n`. A list between inline markers, such as a highlight, is text and
  does not change. With custom `bullets` such as `-`, nested single-item lists that end in an
  empty item wrote `- - -`, a thematic break. The empty item's marker now starts the next line,
  at the column it had on the line of markers: `- -\n    -`.
- **A task item in an ordered list lost its number (#659).**
  `<ol><li><input type="checkbox">p</li></ol>` gave `- [ ] p`, a bullet list. A task item now
  writes the marker of its own list, so it gives `1. [ ] p`, and its content column follows the
  width of that number. Djot output keeps `- [ ] p`, because Djot has task items only in bullet
  lists.
- **A nested ordered list that does not start at 1 joined the text before it (#662).**
  CommonMark lets only a list that starts at 1 interrupt a paragraph, so in
  `<ol start="10"><li>a<ol start="100"><li>q</li></ol></li></ol>` the line `100. q` was part of
  the paragraph `a`. Such a list now starts after a blank line when a paragraph is open before
  it: `10. a\n\n    100. q`. The blank line makes the outer list loose.
- **Text after a line break became a list, quote or heading (#651).** `<p>a<br>1) t</p>` gave
  `a  \n1) t`, so `1) t` became a list item; in bold, in a list item, in a quote and at the top
  level alike. Text that starts the line after a hard break and would interrupt the paragraph
  (`1.`, `1)`, `01.`, `-`, `+`, `*`, `>`, `#`, a rule, a fence or an underline) now has that
  character escaped, `1\) t`, in both converters. A line that cannot interrupt a paragraph, like
  `2. t`, is left as it is.
- **Heading text that reads as a closing `#` or a link definition lost the heading (#661).**
  `<h1>#</h1>` gave `# #`, an empty heading, because CommonMark reads a `#` run at the end of the
  line as the heading's closing sequence; `a #` and `##` lost their `#` the same way. Such a run
  now has its first `#` escaped, `# \#`, in both converters. With `heading_style: underlined`,
  `<h1>[a]: b</h1>` gave `[a]: b` over its underline, which is a link reference definition, so
  the heading was lost. Text that starts a definition now has its `[` escaped, `\[a]: b`. A Djot
  heading and a closed ATX heading keep their `#` run as it is, and plain heading text is
  unchanged.
- **A table that starts a task item became the task's text (#630).**
  `<ul><li><input type="checkbox"><table><tr><td>c</td></tr></table></li></ul>` gave
  `- [ ] | c |`, so the header row was the task's text and the table was lost. A table that is a
  task item's first content now starts after a blank line at the content column, also inside a
  `<div>` or `<section>`. A block in an element that writes nothing before it, like a `<span>` or
  an `<hgroup>`, now also starts below the checkbox; in bold or a link it stays text on the
  checkbox line.
- **The fast converter dropped a task item's checkbox (#632).** With the fast converter forced,
  `<ul><li><input type="checkbox"><p>p</p></li></ul>` gave `- p`. A list item that holds a
  checkbox now goes to the full converter, which writes `- [ ] p`. The full converter also reads
  `type="CHECKBOX"` as a checkbox now, as a browser does.
- **Content after an HTML block that `preserve_tags` keeps joined the block (#655).**
  CommonMark ends an HTML block only at a blank line, and none followed a preserved block
  element, so in `<ul><li><input type="checkbox"><div></div><h2>h</h2>t</li></ul>` the heading
  and the text were part of the HTML. A blank line now follows a preserved element that starts
  an HTML block, in a list item and at the top level:
  `- [ ] &#32;\n  <div></div>\n\n  ## h\n  t`.
- **A table whose cells held only rules was dropped (#628).** The full converter took a table
  with no text and no image for a blank spacer table and wrote nothing, so
  `<table><tr><td><ul><li><hr></li></ul></td></tr></table>` gave an empty document. A rule now
  counts as content, and both converters write the table as `| --- |` over its delimiter row. Text
  in a cell that looks like a list marker no longer turns a rule after it into `___`, and the fast
  converter now trims the space before a rule in a cell as the full converter does.
- **A table whose only cell held a line break was dropped with `br_in_tables` on (#646).**
  `<table><tr><td><br></td></tr></table>` gave an empty document from the full converter while
  the fast converter kept the table as `| <br> |`. With `br_in_tables` on, a `<br>` in a cell
  writes a literal `<br>`, so it now counts as content, matching the fast converter; with the
  option off it still collapses to a space the cell trims away, so the table is still dropped
  there, as it was before.
- **A rule inside bold, italic, a summary or a caption split the emphasis markers (#603).** The
  rule was written after a blank line, which ended the paragraph between the markers, so the
  markers showed as literal text: `<summary>t<hr></summary>` gave `**t\n\n---**`, and a
  definition list that starts its definition with a rule did the same inside a caption, `<b>` or
  `<em>`. Markdown has no rule inside emphasis, so the rule is now the text `---` in the running
  line, as it already was in a link: `**t ---**`. This covers `<b>`, `<strong>`, `<em>`, `<i>`,
  `<summary>`, `<figcaption>`, a table caption, `<del>`, `<s>`, `<strike>`, `<ins>`, `<mark>`,
  `<var>`, `<dfn>`, `<q>`, and `<sub>` and `<sup>` when they write a symbol.
- **A list item took the checkbox of an item in its nested list (#604).**
  `<ul><li>X<ul><li><input type="checkbox" checked> A</li></ul>ZZ</li></ul>` gave `- [x] X AZZ`:
  the outer item became a task item and the nested list was joined into its text. A checkbox in a
  nested list now belongs to that list's item, so the output is `- X`, the nested `- [x] A` and
  then `ZZ`.
- **Text after a quote, in a list inside bold, italic or `<q>`, rendered inside the quote
  (#615).** `<b><ul><li>x<blockquote>q</blockquote>t</li></ul></b>` gave `**- x\n  > q\n  t**`,
  so `t**` continued the quote's paragraph. The first item's marker follows the opening marker,
  so that item is text and writes no content column, and the blank line after the quote went
  with the column. A nested item's marker starts its own line, so it is a real list item, and
  `**- a\n  * x\n    > q\n    t**` kept `t` in the quote too. Text after a quote or a nested
  list now starts after a blank line, at the column of the innermost real list item:
  `**- x\n  > q\n\nt**` and `**- a\n  * x\n    > q\n\n    t**`. A quote 4 or more columns
  past that item's column, or past the start of the line when no item is real, is the
  paragraph's own text, so nothing changes there and the markers stay in one paragraph.
- **A quote that starts a list item rendered outside the item (#617).**
  `<ul><li><blockquote>q</blockquote></li></ul>` gave `-\n> q`, an empty item and a quote after
  the list, also when the list is inside a quote. The quote now starts on the marker line,
  `- > q`.
- **A quote that starts a task item became the task's text (#622).** A checked task item whose
  first content is `<blockquote>q</blockquote>` gave `- [x] > q`, which renders the text
  `[x] > q`. The quote now starts on the next line at the item's content column,
  `- [x] &#32;\n  > q`. The checkbox line ends in a space written as a character reference: GFM
  reads a checkbox only when content follows it, so cmark-gfm shows a bare `[x]` line as text.
  A nested list, a heading or a code block that starts a task item does the same, also inside a
  `<div>` or a `<section>`, and also when the checkbox is in a `<p>` of its own. A rule there
  gave `- [ ] ---`, the text `[ ] ---`; it now comes after a blank line,
  `- [ ] &#32;\n\n  ---`, since `---` right under the checkbox line makes that line a heading.
- **A rule that starts a list item rendered outside the item (#623).**
  `<ul><li><hr></li></ul>` gave `-\n\n---`, an empty item and a rule after the list. The rule is
  now `___` on the marker line, `- ___`: `- ---` is a rule of its own.
- **Text inside a list before an item joined the item's marker (#625).** `<ul>how<li>do</li></ul>`
  gave `how- do`, one line of text, and the item was lost. The item now starts its own line,
  `how\n- do`. When its marker cannot interrupt the text above it, as with `3.`, a blank line
  comes first: `how\n\n3. do`.
- **Text after a quote stayed in the quote for a later item of a list inside bold or italic
  (#633).** `<b><ul><li>a<ol><li>x</li><li>y<blockquote>q</blockquote>t</li></ol></li></ul></b>`
  gave `**- a\n  1. x\n  2. y\n     > q\n     t**`, and `t**` continued the quote. The marker
  `2.` cannot interrupt a paragraph, but after the real item `1. x` no paragraph is open, so `2.`
  starts an item. The same holds after a quote of the enclosing item. Text
  after the quote now starts after a blank line: `**- a\n  1. x\n  2. y\n     > q\n\n     t**`.
- **Some first blocks of a task item still became the task's text (#634).** An ordered list that
  starts at a number other than 1 and a code block in the indented style cannot interrupt the
  checkbox line, so they now start after a blank line: `- [ ] &#32;\n\n  3. x` and
  `- [ ] &#32;\n\n      c`.
  The code block keeps its indent; before, `- [ ]\n  c` lost the code. A quote after an empty
  inline element, such as `<span></span>`, now starts on the next line like a quote right after
  the checkbox. An image before the quote is still text on the checkbox line, and so is an empty
  element that `preserve_tags` writes as HTML. An empty list, heading or code block writes
  nothing, so text after it stays on the checkbox line: `- [ ] t`. A fenced code block that holds
  only whitespace still writes its fences, so it starts on the next line and the text after it
  stays out of the code.
- **A quote after a line break in a task item stayed on the checkbox line (#650).**
  `<ul><li><input type="checkbox"><br><blockquote>q</blockquote></li></ul>` gave `- [ ] > q`, and
  the quote was text. The converter's own output now decides which element writes first, also
  inside a `<div>` or a `<section>`. A line break, a `&nbsp;`, a `<template>`, a `<noscript>`, an
  `<input>` that is not the checkbox, an empty `<picture>` or an image with `skip_images` writes
  nothing there, so the quote now starts on the next line: `- [ ] &#32;\n  > q`.
- **An underlined heading ended its list item (#635).** With `heading_style` set to `underlined`,
  the underline of a heading in a list item was written at column 0: `<ul><li><h2>q</h2></li></ul>`
  gave `- q\n-`, an item and an empty item. The underline now gets the item's content column,
  `- q\n  --`. After a line of the item, the heading starts after a blank line, so it does not
  continue that line's paragraph: `- a\n\n  q\n  --`. A block after a heading of one letter starts
  its own paragraph: `<ul><li><h2>q</h2><p>t</p></li></ul>` gives `- q\n  --\n\n  t`. In a list
  item the underline has at least two dashes, since a lone `-` line reads as an empty item.
- **Many blocks in one list item took quadratic time (#649).** Each block checked every earlier
  line of the item to see whether the item was still open, and each item of a list inside bold or
  italic checked every earlier item. `<ul><li>` with 5000 headings after text took seconds. Each
  check now reads only the lines written since the last one, so the time grows linearly.
- **A heading after text in a quote joined the text (#640).** `<blockquote>a<h2>q</h2></blockquote>`
  gave `> a## q`, a paragraph, and with `heading_style` set to `underlined` it gave `> aq\n> -`,
  one heading. The heading now starts after a blank quote line: `> a\n>\n> ## q`. An underlined
  heading does the same after any line of text in the quote, such as a line that ends in a hard
  break or the last item of a list.
- **An underlined heading whose text starts a block lost its heading (#653).** With
  `heading_style` set to `underlined`, `<h2>-</h2>` gave `-\n-`, two empty list items. The text of
  an underlined heading is now escaped where it would start a block: a list marker (`-`, `*`,
  `1.`, also on its own), a quote, a `#` heading or a rule. `<h2>-</h2>` gives `\-\n-`.
- **The lines of a list item in a quote or under tab indent left the item (#654).** A list inside
  a quote in a list item counted the markers outside the quote too, so its lines sat further in
  than the item, and the underline of a heading sat short of it. The quote now starts its content
  as a container of its own, so a list in it counts only its own markers. A block after text in a
  list item inside a quote now also starts its own line, as it does outside a quote. An
  underlined heading in a quote in a list item now gets the one-dash underline it gets in a
  quote elsewhere: `<ul><li><blockquote><h2>q</h2></blockquote></li></ul>` gives `- > q\n  > -`,
  where it gave `- > q\n  > --`. Bold or italic around the quote no longer changes the lists in
  it, so text after a nested quote in such a list leaves the nested quote. With
  `list_indent_type` set to `tabs`, the lines of a nested item were one tab short of its content
  column: `- a\n\t* q\n\n\tt` put `t` in the outer item. They now reach the column where the
  item's text starts, `- a\n\t* q\n\n\t\tt`. With `list_indent_width` set to 4 or with tab
  indent, a quote right after an opening bold marker, a summary's or a caption's, now writes a
  nested quote or list of its list at the column of the nearest real list item, where they became
  a code block. A list inside `<mark>` or `<del>` is now text after its marker, as inside bold,
  so text after a quote in it no longer becomes a code block.
- **Wrap mode joined a rule or a heading underline to the text next to it (#607).** With `wrap`
  on, a `---` line followed by text became one line of text, `--- B`, and the rule was lost. The
  underline of an underlined heading was joined to the heading text (`Heading -------`), or cut
  off from it by a blank line for `=======`, so the heading was lost too. A rule now stays on its
  own line, and an underline stays right under its heading text, which is not reflowed, as with a
  `#` heading. Both hold inside a quote too.
- **Wrap mode joined the keys of the frontmatter into one line.** With `wrap` on and metadata
  extraction on, the YAML frontmatter went through the reflow like body text, so
  `---\ntitle: My Page\n---` became `--- title: My Page ---` and the frontmatter was lost. Only
  the text after the frontmatter is wrapped now.
- **Wrap mode folded list items, code fences, headings and table rows inside a quote into text.**
  With `wrap` on, `<blockquote><ul><li>alpha</li><li>beta</li></ul></blockquote>` gave
  `> - alpha - beta`, one item, and a code block inside a quote lost its code. Outside a quote, a
  `~~~` code block was reflowed like a paragraph. Every line that starts a block now keeps its own
  line in and out of a quote, and a code block ends only at a fence that closes it.
- **Wrap mode turned a tight list loose when an item went on to a second line (#616).** A line
  right under a list item that starts no block of its own, at column 0 or indented, belongs to
  the item's text. With `wrap` on, the reflow wrote it as a paragraph of its own and put a blank
  line before the next item, so the list rendered loose. It now joins the item's text and is
  wrapped with it. For the same reason, a line that starts with a number such as `1990.` or
  `57)` stays in its paragraph, in a quote and in a list item: only a bullet or a number equal
  to 1, such as `1.` or `01)`, can end a paragraph and start a list. Under a list item, a number
  line left of the item's text still ends the item.
- **Wrap mode dropped a hard line break (#613).** With `wrap` on, `<p>a<br>b</p>` gave `a b`:
  the reflow joined the line after a `<br>` to the line before it, in a paragraph, a quote and a
  list item. A hard break is now a line end the reflow never joins across, so each side of it is
  wrapped on its own and the break stays, with both newline styles.
- **Wrap mode could start a line with a list marker and turn text into a list (#614).** With
  `wrap` on, a break before a `-`, `1.`, `#` or `>` in running text started a new line with it,
  which opened a list, a heading or a quote. A wrapped line now never starts with a word that
  opens a block there, also in a run such as `--- --- ---` or `* * *`; the word stays at the
  end of the line before it, which can then run past the wrap width. A number equal to 1 with
  leading zeros, such as `01.` or `001)`, opens a list like `1.` does, so it is kept off a line
  start too, and a link label or image alt line that starts with one is escaped. A number
  followed by non-breaking spaces, such as Word's `1.&nbsp;&nbsp; Cut`, is no longer read as a
  list marker, and the reflow no longer breaks a line at a non-breaking space.
- **Wrap mode broke a link whose address holds a space.** An address with a space is written in
  angle brackets, `[Share](<https://example.com/?text=a b>)`, and a line end inside the brackets
  ends the link. With `wrap` on, the reflow broke the line there. It now keeps the address in
  angle brackets on one line.
- **Wrap mode cut a nested list marker off from its text.** A list item that holds a nested list
  on its own line, such as `- 3. [vote](...) title`, wrapped to `- 3.` and the text on the next
  line. `- 3.` alone is an empty nested item, and the text below it left the nested list. The
  reflow now treats both markers as one, so the text stays in the nested item. An item whose
  text starts a heading or a code fence, such as `- ## Title`, is no longer reflowed: the heading
  kept only its first words, and the code lines were joined and wrapped like text.

- **A page whose bytes open with a mangled byte order mark lost its whole head.** A real leading
  U+FEFF is stripped before parsing, but one a wrong encoding guess mangles beyond recognition
  reads as ordinary text by the time #527's head search sees it, and that search treated any such
  text as the start of the body, so it gave up before it ever reached `<head>` and reported no
  title, no meta tags and no base or canonical link. A browser discards anything ahead of the
  document's own `<html>` tag without letting it block the real head inside, so the head search
  now does the same: text directly in front of `<html>` no longer ends the search, on both tiers.
  A head-only fragment with no `<html>` tag is unaffected: text ahead of `<head>` there still ends
  the search, as #527 intended.
- **An `<img>` whose `src` spelled the `data:` scheme in upper or mixed case ignored its lazy-load
  address.** The check that sends a `data:` source to the `data-src`, `data-lazy-src`,
  `data-original` and `srcset` fallbacks compared the scheme in lower case only, so
  `<img src="DATA:..." data-src="real.png">` kept the payload while the same image with `data:`
  used `real.png`. The scheme now matches in any case.
- **An image whose `data:` scheme was written in upper or mixed case was not extracted, and the
  metadata reported it as a relative image.** Inline image extraction and the metadata image type
  compared the scheme in lower case only. Every check now uses the one case-insensitive test the
  converter uses for its markdown output.
- **Text right after a list or a table rendered inside it (#570, #571).** Inline content that
  directly followed a list or a table in the same container continued the block's last line.
  After a list it became a lazy continuation of the last item, so
  `<div><ul><li>A</li></ul>para</div>` rendered `para` inside the item; after a table it became
  one more table row. Inline content after a block now starts its own paragraph after a blank
  line, the same as text after a paragraph, in both rendering paths. This includes a list item
  that ends in a line break, and a `<br>` right after a list with backslash line breaks, which
  also put the text inside the item. Text after a horizontal rule gets the same blank line.
- **A block inside a list item, and the text after it, left the item (#583).** A heading after
  the item's text joined that text (`<li>A<h3>H</h3>tail</li>` gave `- A### H`), a rule ended the
  list, and the text after a div, table, blockquote, nested list or definition list became a lazy
  continuation of that block. The text after a code block or paragraph, a definition list, and
  a section, article, header, footer, aside or main element fell out of the list. Inside a list
  item, a block after other content of the item now starts on its own line at the item's content
  column, and text after a block starts its own paragraph at that column. A paragraph or div
  after the item's text is now its own paragraph instead of joining the text, which makes the
  list loose. The column is written only while the item is still open, also inside a
  definition list or a section that the item holds. Whitespace between the marker and the
  item's first block no longer counts as content in strict whitespace mode, and with `wrap` a
  paragraph inside an item keeps its indent, also inside a blockquote. The fast conversion
  path hands these items to the full converter.
- **Text after a list or table at the end of an inline wrapper continued it (#585).** In
  `<div><span><ul><li>A</li></ul></span>para</div>`, `para` still continued the list's last item,
  because the rule from #570 looked only at the element right before the text. Text after an
  inline element whose last content is a block now starts its own paragraph too, in both
  rendering paths.
- **A horizontal rule right after a line of text turned the text into a heading (#584).** Inside
  a paragraph, and at the start of a definition, the rule was written on the line right after
  the text, so `<p>t<hr>B</p>` gave `t\n---` and `<dl><dt>t</dt><dd><hr></dd></dl>` gave the same.
  Markdown reads `---` under text as a heading underline, so `t` became a heading and the rule
  was lost. The rule now starts after a blank line there too, as it already did after text in a
  `<div>`.
- **The nightly benchmark guardrail scored timings on hardware it was never calibrated on.** The
  runner pool moved from the AMD EPYC 9V74 the baseline was calibrated on to an EPYC 7763, and
  every fixture read 15% to 45% slower. `htmbench compare` still scored each timing, printed 26
  `FAIL` lines, and then passed the run as advisory, so the job was green while measuring nothing.
  On a CPU other than the calibrated one no timing is scored now: the run reports
  `TIMINGS NOT SCORED` with both CPUs and fails, and under `--allow-host-mismatch` (the nightly
  job) it succeeds with a `Benchmark timings not scored` warning on the run page instead. The
  fixture inventory stays fatal on any host, and a regression on the calibrated CPU still fails.
- **The benchmark baseline recorded stale output sizes for five fixtures.** The 3.14.2
  conversion fixes moved the Markdown output of `gh-121-hacker-news`, `gh-127-issue`,
  `gh-190/firsteigen`, `gh-190/rbloggers` and `wikipedia/small_html`, and the baseline was never
  updated. Each change was traced to the fix that made it and reviewed: images kept in layout rows
  (5b26d732d, 31f2015b1), and whitespace no longer opening a line (c5b8d1baa), which also stops
  two lines rendering as indented code blocks. Only `output_bytes` changes; the calibrated timings
  stay as measured. `gh-190/plusblog` changed for a different reason, fixed below.
- **An `<img>` with no usable `src` could take its address from the middle of a `srcset`
  candidate.** The fallback split `srcset` and `data-srcset` on every comma, so a comma inside a
  parenthesised descriptor or inside a URL started a new candidate: `a.png (x, b.png 3x ), c.png 2x`
  produced `b.png`, and `a.png?w=1,2 2x` produced `2`. Candidates are now split with the HTML
  spec's srcset parsing steps, including the parentheses rule, and only the spec's five ASCII
  whitespace characters separate a URL from its descriptor.
- **The `srcset` fallback could choose a candidate a browser never loads, or compare a width with a
  density.** Descriptors were read with Rust's float parsing and nothing else, so `a.png foo` stayed
  eligible, `a.png infx` beat every other candidate, a first `NaNx` candidate could not be beaten,
  `+2x` counted as `2x`, and `900x` beat `800w`. Descriptors now go through the spec's descriptor parser: a candidate with an
  unknown token, a number outside the spec's grammar, a zero width, a second descriptor of one kind,
  or an `h` without a `w` is dropped, and a list with no valid candidate keeps `src`. Widths and
  densities are not compared with each other, because a browser needs `sizes` and the viewport to
  do that: when any candidate has a width, the largest width wins, and otherwise the largest density
  wins, a candidate with no descriptor counting as `1x`.
- **A `<br>` in a `<span>` after a list or a layout table pulled the next paragraph into the last
  list item.** A `<span>` removed the line break that ends the item's line, so the `<br>` became a
  hard break at the end of the item and the paragraph after it rendered inside the item. Without a
  `<br>`, the span's text was joined onto the item's last word, and a `<span>` right after a
  horizontal rule was joined onto the `---`. A `<span>` now leaves the line break before it in
  place (#546).
- **Head metadata kept its character references encoded.** The `<base href>`, the
  `<link rel="canonical">` href and the `<title>` text reached the frontmatter and the structured
  metadata as written, so `<base href="https://example.com/it&#x27;s/">` produced
  `base: https://example.com/it&#x27;s/`. They now go through the same decoder as `<meta content>`
  and body text, including the Windows-1252 mapping for numeric references 128-159.
- **A newline in a head value started a new frontmatter key.** The frontmatter wrote each
  `key: value` line as it was, so a `<title>` or `<meta content>` holding a newline, literal or
  written as `&#10;`, could add or override a key. Each key and value is now one YAML scalar: a
  value that plain YAML would misread (a newline, `: `, a space before `#`, a leading `-`, `#` or
  `@`, a control character) is written in double quotes with YAML escapes. Other values stay
  unquoted.
- **Legacy named references without a semicolon were not decoded.** The spec lets about a hundred
  names such as `&copy`, `&amp` and `&eacute` close without `;`, and browsers decode them in text:
  `&copy 2024` is `© 2024`. The converter kept them as written. They now decode on both tiers,
  with the spec's longest-name rule (`&notit;` is `¬it;`). In an attribute value a legacy name
  followed by `=` or a letter or digit stays as written, so `?a=1&copy=2` in an `href` is unchanged.
- **Frontmatter values that YAML reads as numbers, booleans, null or dates were not quoted.** A
  value such as `3`, `true`, `null` or `2024-01-01` is a valid plain scalar, so a YAML reader
  turned `meta-algolia-search-order: 3` into the number 3. A value that the YAML 1.2 core schema
  or a YAML 1.1 reader resolves to anything other than a string is now written in double quotes.
- **Numeric character references without a semicolon were not decoded.** The spec decodes `&#39`
  and `&#x27` without their `;`, in text and in attribute values, so `it&#39s` is `it's` in a
  browser. The converter kept the reference as written. It now decodes on both tiers.
- **Tier 1 kept a reference encoded after an unknown name.** When an unknown name such as `&foo`
  had a `;` a few bytes later, Tier 1 wrote the whole span as it was, so `&foo &amp;` kept
  `&amp;` where Tier 2 wrote `&`. Tier 1 now writes the `&` alone and reads on, as Tier 2 does.
- **Tier 1's fallback message added a `;` the page did not have.** When Tier 1 handed a reference
  without its `;` to Tier 2, the log message showed `&#39;` for an input of `&#39`. The message
  now shows the reference as written (#565).
- **Tier 1's fallback message called a known reference unknown.** When Tier 1 handed a reference
  without its `;` to Tier 2, the log message called it an unknown HTML entity even when the
  reference was one Tier 2's decoder knows, such as `&#39` or `&copy`. The message now says the
  reference is missing its `;` when the name is known, and keeps the unknown wording for names
  that really are unknown (#586).

- **`base_url` could pick a `<base href>` that a browser ignores.** The document base came from a
  byte scan for the first `<base` tag in the source, so a `<base>` inside a comment, inside
  `<title>`, `<textarea>`, `<script>`, `<style>` or another raw-text element, inside `<template>`
  or SVG, or in a body that a `<frameset>` replaces, set the base for every relative link. The base
  now comes from the first `<base>` with an `href` in tree order, read from an html5ever parse of
  the document. The parse stops at the first `<base href>` once no later markup can place a node
  in front of it or remove it, in `<head>` or in the body, and a page without a `<base` tag is not
  parsed at all. Found while adopting `base_url` downstream, where the same two mistakes had
  already been fixed once in a link pre-pass.

- **A `data:` or `javascript:` `<base href>` became the base for relative links.** A browser
  ignores such a base and resolves against the page's own URL, as the HTML "frozen base URL"
  steps require. `base_url` now does the same, so relative links on such a page resolve against
  the caller's `base_url` instead of failing to resolve.

- **The `base` and `canonical` metadata kept the last tag, not the first.** The head extractor
  overwrote each value when it met another tag, so the reported `base` could differ from the
  base that `base_url` resolves against. The `base` metadata (and `base_href` in the document
  metadata) is now the same first `<base href>` in tree order that the document base uses, read
  once, and the first `<link rel="canonical">` wins.

- **The `<meta>` metadata kept the last tag with a given name.** Each `<meta name>` or
  `<meta property>` overwrote the value an earlier tag with the same key had stored. The first
  tag per key now wins on both tiers, as the `base` and `canonical` metadata do.

- **A page without a `<head>` tag reported no `base` metadata.** The parser creates the head
  itself, so `<base href="https://a.example/"><p>x</p>` sets the document base, but the head
  extractor looked for a `<head>` tag in the source and found none. The `base` metadata now comes
  from the same parsed document as the document base on both tiers, with or without a `<head>`
  tag.

- **A `>` inside a `<base>` attribute value turned off the early stop of the base parse.** The
  parse is fed in pieces, and a piece could end inside the quoted value, so the `<base>` tag only
  completed in the next piece, which was never checked. The parse now notes each `<base href>`
  element when html5ever's tree builder creates it and checks only that element's ancestors, so
  the parse stops wherever the tag bytes fall, and the cost of the check does not grow with the
  size of the tree. The document base itself was always correct.

- **The `title` metadata kept the last `<title>`, not the first.** A head with the titles First
  and Second reported Second, while a browser shows First. The first title now wins on both
  tiers, as the `base`, `canonical` and `<meta>` metadata already do. An empty first title also
  wins, as in a browser, so the page reports no title.

- **`<meta>` names that differ only in letter case let the last tag win.** Meta names do not
  depend on letter case, but `<meta name="Description">` followed by `<meta name="description">`
  gave the second value in the document metadata, and the frontmatter printed both. The names are
  now compared in any letter case, and the first tag wins in the frontmatter and the document
  metadata.

- **A `<head>` tag inside the body gave the two tiers different metadata.** The parser ignores a
  `<head>` tag once the body has started. Tier 1 read such a stray head when the page had no head
  of its own, and Tier 2 read it when the real head was empty. Both tiers now read only the first
  head before the body. The body starts at a `<body>` tag, at text, or at any tag other than the
  ones a head can hold, so `<p>x</p><head><title>Stray</title></head>` has no title. The
  document structure gives a metadata block only for the head the metadata reads. A leading
  UTF-8 byte order mark is dropped first, as a browser's decoder drops it, so it does not start
  the body and no longer appears at the start of the output.

- **A `<meta>` tag named `title`, `base` or `canonical` replaced the document's title, base and
  canonical link.** `<meta name="base" content="/meta/">` next to `<base href="/real/">` set
  `base_href` in the document metadata to `/meta/`, while the links resolved against `/real/`, and
  `<meta name="title">` replaced the `<title>` text. The `base_href` and `canonical_url` fields
  now come only from the `<base>` element and `<link rel="canonical">`, and such a meta tag is an
  ordinary entry in `meta_tags`. A `<meta name="title">` still gives the `title` of a page without
  a `<title>` element, but it no longer replaces the text of one.

## [3.15.1] - 2026-09-27

### Fixed

- **`base_url` did not resolve a `<blockquote cite>`, and Tier 1 dropped the citation entirely.**
  `cite` is a destination the converter renders — Tier 2 emits it as a trailing `— <url>` line —
  but it was the one such destination `base_url` left alone, so a relative citation stayed
  relative. Tier 1's blockquote handling read no attributes at all, so the same markup produced
  the citation on one tier and nothing on the other; a cited blockquote now bails to Tier 2,
  which is authoritative for it. Found while adopting `base_url` downstream, where a converter
  that resolves every destination except this one forces the caller to keep a whole
  link-rewriting pre-pass alive for it.

- **The Go binding could read a different OS thread's FFI error slot.** `Convert`,
  `HeaderMetadata.IsValid` and the visitor entry point now pin the goroutine for the duration of
  the cgo call with `runtime.LockOSThread`. The FFI layer keeps its last-error state in a
  `thread_local!` (`crates/html-to-markdown-ffi/src/lib.rs:31`), so without pinning the Go runtime
  was free to reschedule the goroutine between the call that stamped the error and the
  `htm_last_error_code` read that reports it — surfacing a nil error for a call that had in fact
  failed. Emitted by alef 0.97.0; no Go API changed.

### Changed

- Repinned the `alef` generator to 0.97.0 and regenerated every binding. Apart from the Go thread
  pinning above, the only other generated change is the Kotlin Android Gradle wrapper moving from
  9.7.1 to 9.8.0; everything else in the regeneration is version strings and provenance hashes.
- CI now gates the `docs-site` changelog mirror against `CHANGELOG.md`. The two are compared from
  the first `## [` heading onward rather than over `[Unreleased]` alone, because this repo releases
  straight out of `[Unreleased]` and leaves it empty on `main` — an Unreleased-only comparison
  would compare zero lines and pass while a released section drifted.

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
