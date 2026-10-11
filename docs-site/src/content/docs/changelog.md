---
title: "Changelog"
---

All notable changes to html-to-markdown will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- A checkbox outside a task list item writes nothing.
  `<p>Agree <input type="checkbox"> to the terms</p>` gave `Agree [ ] to the terms` and now gives
  `Agree to the terms`. Bracket pairs in running text are link syntax: `[x](optional)` rendered as
  a link. A table cell that holds only inputs keeps its task brackets: `| [x] |`. A label or a
  `span` around the checkbox of such a cell does not change that. A list item is a
  task item only when the checkbox is its first content:
  `<li>Text <input type="checkbox"> more</li>` gave `- [ ] Text  more` and now gives `- Text more`.
  The menu switch of many themes, a checkbox before its label, no longer writes `[ ]`
  ([#757](https://github.com/xberg-io/html-to-markdown/issues/757)).
- A button, an output, a meter, a progress bar or a text area is a word of its own in running
  text, also when the source glues it to a word:
  `<p>Please re<button>load</button>ed the page</p>` gave `Please reload` and now gives
  `Please re load`. The line still ends after the control
  ([#752](https://github.com/xberg-io/html-to-markdown/issues/752)).
- A link whose content gives no text is labelled with the name of the link: its `aria-label`,
  then its `title`. Before, the label was the address of the link. This changes the default
  output for every such link, with or without a graphic:
  `<a href="/page" aria-label="Next page"><i class="fa fa-arrow"></i></a>` gave `[/page](/page)`
  and gives `[Next page](/page)`. A link with no child node is named too:
  `<a href="/page" aria-label="Next page"></a>` gave `[](/page)` and gives `[Next page](/page)`.
  A link with no name keeps its address as its label, or its empty label when it has no child
  node. A link with
  a name is also kept when `inline_data_media = drop_element` removes its content.
- A link into its own page whose content gives no text is left out, whatever that content is
  (a graphic with no text, an empty element, nothing) and whether or not the link has a name. It
  is the icon of a heading permalink or of a "back to top" link, and it leads nowhere else:
  `<h2>Title <a href="#title"><svg>...</svg></a></h2>` gives `## Title`, and
  `<a href="#top" aria-label="Back to top"></a>` gave `[](#top)` and gives nothing. A link is
  into its own page when its address, after it is resolved, differs from the address of the page
  (`base_url`) only by a fragment. A `<base>` element changes what `#part` resolves to, so on such
  a page `#part` can name another document, and that link is kept. With no `base_url`, a written
  `#part` is into its own page when the document has no `<base>` element; with one, the link is
  kept. Such a link that has text is kept with its text.
- A link whose only content is an inline `<svg>` with no text is no longer labelled `SVG Image`.
  The link is kept and labelled by the first rule above:
  `<a href="/page" aria-label="Next"><div><svg>...</svg></div></a>` gives `[Next](/page)`
  ([#750](https://github.com/xberg-io/html-to-markdown/issues/750)).
- A code block keeps the indentation that all its lines share. The converter removed it since the
  fix for [#134](https://github.com/xberg-io/html-to-markdown/issues/134):
  `<pre>  a = 1\n    b = 2\n</pre>` wrote the lines `"a = 1\n  b = 2"` and now writes
  `"  a = 1\n    b = 2"`, as the page shows it. Blank lines at the end of a block are dropped. In a
  list item one blank line at the end stayed and is now dropped too: the middle item of
  `<ul><li>a</li><li><pre><code>b\n\n\n</code></pre></li><li>c</li></ul>` wrote the fence, `b`, a blank
  line and the fence, and now writes the fence, `b` and the fence. A code block in a table cell is
  still one code span and does not keep that indentation
  ([#782](https://github.com/xberg-io/html-to-markdown/issues/782)).
- A fenced code block keeps a blank first line. A browser drops one line feed after the `<pre>` tag
  and shows a second one as a blank line. The converter dropped every line feed at the start of a
  block: `<pre>\n\na\n</pre>` wrote the fence and `a`, and now writes the fence, a blank line and
  `a`. An indented code block does not start with a blank line. Strict white space mode writes the
  start of a block as before.
- A link, an image, highlighted text and an abbreviation inside a code block or a code span write
  their text and no Markdown marks, because code shows every character as text:
  `<pre>Guido &lt;<a href="/mail">guido</a>&gt;</pre>` wrote `Guido <[guido](/mail)>` and now writes
  `Guido <guido>`. An image in code writes nothing, and an inline `<svg>` in code writes its text in
  place of a `data:` image. The link and the image stay in the metadata
  ([#781](https://github.com/xberg-io/html-to-markdown/issues/781)).
- With `extract_metadata` off, an empty `pre` writes nothing in place of an empty fence, as with the
  default options.

### Fixed

- A page with a structured data script keeps its text when a text value of the JSON holds a tag.
  `<script type="application/ld+json">{"a":"<p>"}</script>` in the head made the whole page
  convert to an empty string, and the same script in the body removed the text after it. The
  JSON in the metadata keeps the tags of its values: `{"a":"<p>Nice</p>"}` gave `{"a":"Nice"}`
  and now gives the JSON as written. The end tag of the script can have any spelling a browser
  reads (`</SCRIPT>`, `</script >`, `</script/>`); each of these also gave an empty page. A type
  written with a character reference (`application/ld&#43;json`) is read as structured data
  ([#818](https://github.com/xberg-io/html-to-markdown/issues/818)).
- The options of a select list are separate words instead of one joined word. A select list
  with the options `Quickstart`, `Installation` and `Ruby 101` gave
  `QuickstartInstallationRuby 101` and now gives `Quickstart Installation Ruby 101`; a label, a
  select list and a text area in one form gave `NameOneTwoarea words` and now give
  `Name One Two area words`. Both converters give the same text for output, meter and progress
  elements. Such a control in an inline element (a label, a span, bold text, a custom element)
  stays in its line, as a browser shows it: `<label><button>One</button></label>items` gave
  `Oneitems` and now gives `One items`, and `<span><button>One</button></span>items` gave `One`, a
  blank line and `items`, and now gives `One items`. A block that holds such a control still ends
  the line after it. An input writes no text and adds no space: `<p>Name<input>Mail</p>` gives
  `NameMail`. Inside `pre` and code, a button, an output, a meter, a progress bar or a text area
  adds no space: `<pre>before<button>Go</button>after</pre>` gives `beforeGoafter`. A select list
  there keeps its options as separate words
  ([#752](https://github.com/xberg-io/html-to-markdown/issues/752)).
- The `label` attribute of an option group is no longer written. It is not text of the page: a
  browser shows it only inside the open list. A select list with the groups `Getting Started`
  (option `Quickstart`) and `Build` (option `Commands`) gave `**Getting Started**`, a line break,
  `Quickstart**Build**`, a line break and `Commands`, and now gives `Quickstart Commands`
  ([#776](https://github.com/xberg-io/html-to-markdown/issues/776)).
- An element with no end tag ends where the HTML standard says: at the end tag of its parent, or
  at a start tag that cannot be its content. Before, a paragraph with no end tag stayed open after
  its parent ended, so each `<div class="note"><p>Note</div>` put the rest of the page two levels
  deeper, and after 30 such blocks the page was cut at the depth limit with a
  `depth_limit_exceeded` warning. A page of 80 such blocks and a footer gave 30 notes and no
  footer; it gives 80 notes and the footer, with no warning. This changes the default output for
  other pages that omit an end tag: `<div><p>one</div>tail` gave `onetail` and gives `one`, a
  blank line, `tail`. `<blockquote><p>quoted</blockquote><p>after</p>` gave `after` inside the
  quote and gives it after the quote. A paragraph after a table whose last cell has no end tag
  was written into that cell and is written after the table.
  `<nav><p>one</nav><aside><p>two</aside>tail` gave an empty document and gives `two` and `tail`.
  `<table><thead><tr><th>a<th>b<tbody><tr><td>c<td>d</table>` gave a table of one cell and gives
  both rows. A document with no `</head>` wrote its title into the body, or wrote nothing with
  `extract_metadata = false`; it writes its body. `<p><ruby>a<rt>b<rt>c</ruby> after` gave
  `a(b(c after))` and gives `a(b)(c) after`. `<h2>one<h3>two</h3>three` gave `## onetwothree` and
  gives two headings and a paragraph. `<p>one<blockquote>two</blockquote>three` gave a blank line
  between `one` and the quote and gives none, as the same input with `</p>` written always did.
  A page that ends with an element open converts as the same page with every end tag written:
  `<body><article><p>one</article><header>site</header><p>two</p>` gave `one`, `site` and `two`,
  and gives `one` and `two`, because a `<header>` directly in the body is a page header.
  `<body><main><p>one</main><header>late</header>` loses `late` by the same rule. A page that
  omits an end tag is parsed a second time, by the HTML tree builder. Each of 11 recorded
  Docusaurus pages of 33 to 143 KB takes 1.3 to 6.1 ms more (median 3.2 ms), which is 1.9 to 2.5
  times as long, and the 70 recorded pages together take 290 ms in place of 252 ms. Pages with
  every end tag are not affected. The second parse is given up when it holds more than 512
  elements open, which is the depth a browser builds. Such a page converts as it did before, so
  its omitted end tags are not repaired and it can still lose its tail. A repair that would
  lose text is not used: the text of the page is counted before and after the second parse, and
  a page that comes back with less text converts as it did before. A page that a `<frameset>`
  replaces is such a page: the tree builder drops the text after `</frameset>`, and the
  converter writes it. The omitted end tags of such a page are not repaired. A fragment with no
  `<body>` tag keeps its top-level `<header>` when it is parsed a second time. Before, the second
  parse read every fragment as a document and dropped that `<header>` as a page header:
  `<header>h</header><b><p>one</p></b>` gave `**one**` and gives `h` and `**one**`, and
  `<header>h</header><div><p>one</div>tail` gives `h`, `one` and `tail`. The depth limit and its
  warning are unchanged for a page that really nests that deep. Past the depth limit, where the
  converter warns, the number of empty tables before the cut can differ; no text is lost. Two
  inputs are not repaired: an element that its parent ended and that gets its own end tag later
  (`<div><p>one</div>two</p>rest</div>tail`), and an unquoted attribute value that runs into an
  end tag (`<div class=a</div>`)
  ([#772](https://github.com/xberg-io/html-to-markdown/issues/772)).
- An inline `<svg>` keeps its text and no longer adds the words `SVG Image`. Its text is its
  `aria-label`, the `<title>` and `<desc>` of the graphic, its `<text>` elements and the HTML in a
  `<foreignObject>`, in document order. Style sheets, scripts, metadata and the content of `<defs>`
  and `<symbol>` are not text, and a `<use>` reference is not followed. A `<switch>` gives the child
  that a reader of English gets. An element with `display="none"` or `visibility="hidden"` gives
  nothing, as hidden text outside a graphic gives nothing. `aria-hidden="true"` removes the label,
  the title and the description. That text is the alt text of the image, and it is what
  `inline_data_media = alt_text_only`, a heading and plain text output write, with the escaping of
  any other text. A graphic with no text writes nothing there, and an image with an empty alt text
  when the payload is kept. An icon link no longer takes the style sheet of its graphic as its
  label ([#750](https://github.com/xberg-io/html-to-markdown/issues/750)).
- A graphic in a heading is written as its text also with `extract_metadata = false` and
  `highlight_style = none`. With those options it was written as a `data:` image, because the fast
  converter has no rule for a heading. It now leaves a page with a graphic in a heading, with a
  hidden element in a graphic, or with a link that has a name and no text, to the full converter
  ([#766](https://github.com/xberg-io/html-to-markdown/issues/766)).
- Two page shapes no longer take quadratic time in the pass that removes hidden elements: many
  tag starts with no end (`<a` and a space, repeated), and many attribute values with no closing quote
  (`<g a="` repeated). A 120 KB page with 40,000 such tag starts in a graphic took 16 seconds;
  200,000 of them now take 20 milliseconds. The pass reads each tag once. A `<` inside a tag is
  part of an attribute value there, so an attribute value that holds markup is no longer cut.
  Other page shapes are still slow
  ([#765](https://github.com/xberg-io/html-to-markdown/issues/765)).

- Text that is only white space no longer makes the conversion fail when `include_document_structure`
  is on, and `build_document_structure` no longer panics on it. The Markdown is the same as with the
  structure off. Neither builder records an empty heading, paragraph or list item
  ([#749](https://github.com/xberg-io/html-to-markdown/issues/749)).
- A code block inside a list item keeps the indentation of every line. A line after a blank line got
  four spaces for each list level in place of its own indentation, so
  `<ol><li><p>Save this:</p><pre>def f(x):\n    if x:\n\n        return 1\n</pre></li></ol>` wrote
  `return 1` at the level of `if x:`. It now writes it eight spaces in, as in the page
  ([#769](https://github.com/xberg-io/html-to-markdown/issues/769)).
- A code block whose lines are separate block elements has one line for each. A `div`, `section` or
  other plain container inside a `pre` starts a line and ends a line, as in a browser, and adds no
  blank line: `<pre><div>a<br></div><div>b<br></div></pre>` wrote `a`, a blank line, `b` and now writes
  `a`, `b`. An empty container is no line. A line feed between two containers is still a blank line,
  because a browser shows one ([#770](https://github.com/xberg-io/html-to-markdown/issues/770)).
- Two or more blank lines in a row inside a fenced code block stay, and so do the spaces at the end of
  a line of code: `<pre>import a\n\n\nclass B:\n</pre>` wrote one blank line and now writes two. The
  rules that fold blank lines and trim line ends still apply to the text outside of code
  ([#783](https://github.com/xberg-io/html-to-markdown/issues/783)).
- With `code_block_style = indented`, a code block that starts a block quote keeps the four spaces of
  its first line. Without them a Markdown reader takes the code for a paragraph:
  `<blockquote><pre>a\nb\n</pre></blockquote>` wrote `> a`, `>     b` and now writes `>     a`,
  `>     b`. With that style the blank lines and the spaces at the end of a line of code also stay, at
  the top level, in a block quote and in a list item: `<pre>a   \n\n\nb\n</pre>` wrote
  `"    a  \n\n    b"` and now writes `"    a   \n\n\n    b"`
  ([#799](https://github.com/xberg-io/html-to-markdown/issues/799)).
- With `code_block_style = indented`, a code block that starts the description of a definition list
  keeps the four spaces of its first line, and a blank line goes between the term and the code:
  the full converter wrote `"t\na\n    b"` for `<dl><dt>t</dt><dd><pre>a\nb\n</pre></dd></dl>`, and
  the full and fast converters now write `"t\n\n    a\n    b"`. The spaces at the end of the last
  line of that code stay too
  ([#799](https://github.com/xberg-io/html-to-markdown/issues/799)).
- Add `expand_abbreviations` and `include_blockquote_citations` options, enabled by default.
- Keep one blank quote line between lists and blocks, preserving blank lines inside code.
- Convert repeated unfinished tags, unclosed hidden elements, and inline list fragments with linear scans.
- Repair a page that ends a paragraph again after the end tag of its parent, or whose head has no end tag, from the first parse with no second tokenizer pass; past 512 levels or past the node budget of the repair, keep the first parse with every word of the input and a `malformed_html` warning.

## [3.17.2] - 2026-10-06

### Changed

- Rust dependencies and all generated bindings, documentation, test applications and E2E suites
  were refreshed with Alef 0.105.0.

### Fixed

- HTML highlight output now preserves leading and trailing whitespace inside `<mark>` boundaries
  ([#744](https://github.com/xberg-io/html-to-markdown/issues/744)).
- Whitespace between adjacent inline images is preserved instead of joining their Markdown output
  ([#746](https://github.com/xberg-io/html-to-markdown/issues/746)).
- Blank lines in image alternative text are encoded without losing line feeds or producing invalid
  Markdown image labels ([#747](https://github.com/xberg-io/html-to-markdown/issues/747)).

## [3.17.1] - 2026-10-05

### Fixed

- Caller-provided `base_url` values now resolve relative URLs without being emitted as document
  `base` metadata when the source has no `<base href>`
  ([#743](https://github.com/xberg-io/html-to-markdown/issues/743)).

## [3.17.0] - 2026-10-04

### Added

- Structure extraction now records inline annotations during the conversion traversal, including
  links and inline formatting, while preserving visitor, exclusion and depth semantics
  ([#741](https://github.com/xberg-io/html-to-markdown/issues/741)).
- The WASM converter now rejects inputs above a configurable 2 MiB default limit with a typed
  error, preventing unexpectedly large allocations in browser and edge runtimes
  ([#508](https://github.com/xberg-io/html-to-markdown/issues/508)).

### Changed

- WASM class-valued getters and setters now use explicit detached-copy semantics; callers reassign
  modified values rather than mutating an ineffective borrowed wrapper
  ([#729](https://github.com/xberg-io/html-to-markdown/issues/729)).
- Conversion reuses validated parser state and precomputed sibling context, and raw-text stripping
  uses byte-search skipping, reducing repeated DOM and scan work without changing output
  ([#649](https://github.com/xberg-io/html-to-markdown/issues/649),
  [#715](https://github.com/xberg-io/html-to-markdown/issues/715)).
- Rust and polyglot dependencies, supported toolchains and generated bindings were refreshed; Alef
  is pinned to 0.103.14.
- CI now uses baseline-free quality gates, stricter delegated-workflow checks and reliable benchmark
  timing and regression thresholds
  ([#736](https://github.com/xberg-io/html-to-markdown/issues/736),
  [#738](https://github.com/xberg-io/html-to-markdown/issues/738)).

### Fixed

- Lists, task items, block quotes and wrapped output now preserve markers, indentation, hard breaks
  and block boundaries across nested and inline containers
  ([#594](https://github.com/xberg-io/html-to-markdown/issues/594),
  [#603](https://github.com/xberg-io/html-to-markdown/issues/603),
  [#604](https://github.com/xberg-io/html-to-markdown/issues/604),
  [#607](https://github.com/xberg-io/html-to-markdown/issues/607),
  [#611](https://github.com/xberg-io/html-to-markdown/issues/611),
  [#612](https://github.com/xberg-io/html-to-markdown/issues/612),
  [#613](https://github.com/xberg-io/html-to-markdown/issues/613),
  [#614](https://github.com/xberg-io/html-to-markdown/issues/614),
  [#615](https://github.com/xberg-io/html-to-markdown/issues/615),
  [#616](https://github.com/xberg-io/html-to-markdown/issues/616),
  [#617](https://github.com/xberg-io/html-to-markdown/issues/617),
  [#619](https://github.com/xberg-io/html-to-markdown/issues/619),
  [#620](https://github.com/xberg-io/html-to-markdown/issues/620),
  [#622](https://github.com/xberg-io/html-to-markdown/issues/622),
  [#623](https://github.com/xberg-io/html-to-markdown/issues/623),
  [#625](https://github.com/xberg-io/html-to-markdown/issues/625),
  [#626](https://github.com/xberg-io/html-to-markdown/issues/626),
  [#627](https://github.com/xberg-io/html-to-markdown/issues/627),
  [#629](https://github.com/xberg-io/html-to-markdown/issues/629),
  [#630](https://github.com/xberg-io/html-to-markdown/issues/630),
  [#631](https://github.com/xberg-io/html-to-markdown/issues/631),
  [#633](https://github.com/xberg-io/html-to-markdown/issues/633),
  [#634](https://github.com/xberg-io/html-to-markdown/issues/634),
  [#635](https://github.com/xberg-io/html-to-markdown/issues/635),
  [#636](https://github.com/xberg-io/html-to-markdown/issues/636),
  [#637](https://github.com/xberg-io/html-to-markdown/issues/637),
  [#643](https://github.com/xberg-io/html-to-markdown/issues/643),
  [#650](https://github.com/xberg-io/html-to-markdown/issues/650),
  [#651](https://github.com/xberg-io/html-to-markdown/issues/651),
  [#653](https://github.com/xberg-io/html-to-markdown/issues/653),
  [#654](https://github.com/xberg-io/html-to-markdown/issues/654),
  [#655](https://github.com/xberg-io/html-to-markdown/issues/655),
  [#657](https://github.com/xberg-io/html-to-markdown/issues/657),
  [#658](https://github.com/xberg-io/html-to-markdown/issues/658),
  [#659](https://github.com/xberg-io/html-to-markdown/issues/659),
  [#662](https://github.com/xberg-io/html-to-markdown/issues/662),
  [#668](https://github.com/xberg-io/html-to-markdown/issues/668),
  [#669](https://github.com/xberg-io/html-to-markdown/issues/669),
  [#674](https://github.com/xberg-io/html-to-markdown/issues/674),
  [#675](https://github.com/xberg-io/html-to-markdown/issues/675),
  [#678](https://github.com/xberg-io/html-to-markdown/issues/678),
  [#680](https://github.com/xberg-io/html-to-markdown/issues/680),
  [#681](https://github.com/xberg-io/html-to-markdown/issues/681),
  [#686](https://github.com/xberg-io/html-to-markdown/issues/686),
  [#734](https://github.com/xberg-io/html-to-markdown/issues/734)).
- Markdown and Djot escaping now keeps literal markers, rule-like text, table delimiters, code-span
  backticks and hard breaks valid in their surrounding container
  ([#624](https://github.com/xberg-io/html-to-markdown/issues/624),
  [#638](https://github.com/xberg-io/html-to-markdown/issues/638),
  [#661](https://github.com/xberg-io/html-to-markdown/issues/661),
  [#688](https://github.com/xberg-io/html-to-markdown/issues/688),
  [#689](https://github.com/xberg-io/html-to-markdown/issues/689),
  [#697](https://github.com/xberg-io/html-to-markdown/issues/697),
  [#705](https://github.com/xberg-io/html-to-markdown/issues/705),
  [#706](https://github.com/xberg-io/html-to-markdown/issues/706),
  [#707](https://github.com/xberg-io/html-to-markdown/issues/707),
  [#708](https://github.com/xberg-io/html-to-markdown/issues/708),
  [#709](https://github.com/xberg-io/html-to-markdown/issues/709),
  [#710](https://github.com/xberg-io/html-to-markdown/issues/710),
  [#713](https://github.com/xberg-io/html-to-markdown/issues/713),
  [#735](https://github.com/xberg-io/html-to-markdown/issues/735)).
- Tables and headings now retain surrounding text, block boundaries, checkboxes and nested content
  consistently between the full and fast converters
  ([#628](https://github.com/xberg-io/html-to-markdown/issues/628),
  [#645](https://github.com/xberg-io/html-to-markdown/issues/645),
  [#646](https://github.com/xberg-io/html-to-markdown/issues/646),
  [#647](https://github.com/xberg-io/html-to-markdown/issues/647),
  [#679](https://github.com/xberg-io/html-to-markdown/issues/679),
  [#692](https://github.com/xberg-io/html-to-markdown/issues/692),
  [#702](https://github.com/xberg-io/html-to-markdown/issues/702),
  [#722](https://github.com/xberg-io/html-to-markdown/issues/722),
  [#723](https://github.com/xberg-io/html-to-markdown/issues/723),
  [#724](https://github.com/xberg-io/html-to-markdown/issues/724),
  [#725](https://github.com/xberg-io/html-to-markdown/issues/725),
  [#730](https://github.com/xberg-io/html-to-markdown/issues/730),
  [#732](https://github.com/xberg-io/html-to-markdown/issues/732)).
- Hard line breaks now remain in place across source whitespace, inline buffers, links and both
  Markdown and Djot output
  ([#690](https://github.com/xberg-io/html-to-markdown/issues/690),
  [#691](https://github.com/xberg-io/html-to-markdown/issues/691),
  [#693](https://github.com/xberg-io/html-to-markdown/issues/693),
  [#696](https://github.com/xberg-io/html-to-markdown/issues/696),
  [#698](https://github.com/xberg-io/html-to-markdown/issues/698),
  [#704](https://github.com/xberg-io/html-to-markdown/issues/704),
  [#711](https://github.com/xberg-io/html-to-markdown/issues/711),
  [#718](https://github.com/xberg-io/html-to-markdown/issues/718)).
- Metadata, preprocessing and URL handling now preserve effective base URLs and titles, remove page
  headers as configured, and ignore link-like text inside raw-text elements
  ([#608](https://github.com/xberg-io/html-to-markdown/issues/608),
  [#716](https://github.com/xberg-io/html-to-markdown/issues/716),
  [#719](https://github.com/xberg-io/html-to-markdown/issues/719),
  [#739](https://github.com/xberg-io/html-to-markdown/issues/739),
  [#740](https://github.com/xberg-io/html-to-markdown/issues/740)).
- The fast converter builds cleanly without default features, and generated R reference pages now
  match the public API
  ([#593](https://github.com/xberg-io/html-to-markdown/issues/593),
  [#596](https://github.com/xberg-io/html-to-markdown/issues/596)).

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

## Archives

- [3.15.0 through 3.14.0](changelog-archive-6.md)
- [3.13.0 through 3.11.5](changelog-archive-1.md)
- [3.11.4 through 3.6.21](changelog-archive-2.md)
- [3.6.20 through 3.2.0](changelog-archive-3.md)
- [3.1.0 through 2.14.1](changelog-archive-4.md)
- [2.14.0 through 1.x](changelog-archive-5.md)
