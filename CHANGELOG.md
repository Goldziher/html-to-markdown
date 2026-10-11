# Changelog

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

- Plain text output separates the text of a form control from the words beside it, as the
  Markdown output does. This shows only with `remove_forms` off.
  `<p>aaa<select><option>Xone</option><option>Xtwo</option></select>bbb</p>` gave
  `aaaXoneXtwobbb` and now gives `aaa Xone Xtwo bbb`. Each option of a select list or a data
  list is a word of its own. A button, an output, a meter, a progress bar or a text area is a
  word of its own too: `<h1>aaa<button>Xone</button><button>Xtwo</button>bbb</h1>` gave
  `aaaXoneXtwobbb` and now gives `aaa Xone Xtwo bbb`. In a `pre` element a button adds no
  space, and the options of a select list stay separate words
  ([#814](https://github.com/xberg-io/html-to-markdown/issues/814)).
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

- A line break in the source before an inline element is one space between two words, in every
  container. Before, the words on the two sides of a `<span>` were joined, with an empty span and
  with a span that holds text. This changes the default output:
  `<p>hard to read\n<span id="index-0"></span>docstrings</p>` gave `hard to readdocstrings` and
  gives `hard to read docstrings`, `<p>one\n<span>two</span> three</p>` gave `onetwo three` and
  gives `one two three`, and `<ul><li>one\n<span id="a"></span>two</li></ul>` gave `- onetwo` and
  gives `- one two`. Outside a paragraph the line break before any other inline element was a line
  break, and it is a space now, as a browser shows it: `<div>one\n<b>y</b></div>` gave `one` and
  `**y**` on two lines and gives `one **y**`. A line break before a block stays a line break, and
  so does a line break before an inline element whose content starts with a block
  ([#778](https://github.com/xberg-io/html-to-markdown/issues/778)).

- White space on the two sides of the start of an inline element is one space, and it goes
  outside the marks of the element: `<p>one <b> y</b>two</p>` gave `one  **y**two` with two spaces
  and gives `one **y**two`. The same holds for `<kbd>`, `<samp>`, `<abbr>`, `<dfn>`, `<del>`,
  `<ins>`, `<mark>`, `<sub>` and `<sup>`. At the start of a line such an element starts with no
  space: `<div>a<br><sup> x</sup></div>` gave a space and `x` on the second line and gives `x`.

- A line break is no space when an element or a comment stands between it and a zero-width space,
  as in a browser: the zero-width space is a place where a line can break, so the two parts are
  one word. `<p>one \n <span></span>&#8203;two</p>` gave `one`, a space, the zero-width space and
  `two`, and gives `one`, the zero-width space and `two`. The line break can be the last content
  of an element that only wraps text: `<p><span>one\n</span>&#8203;two</p>` gave `one`, a space,
  the zero-width space and `two`, and gives `one`, the zero-width space and `two`. Two or more
  line breaks are one line break there: `<p>one\n\n<span></span>&#8203;two</p>` gave two
  paragraphs and gives one word. A space that is not a line break stays. The converter keeps the
  space before a zero-width space only when the `style` attribute of the element between them, or
  of an element around them, sets `display` or `white-space`:
  `<p>one\n<span style="display:inline-block"></span>&#8203;two</p>` keeps the space, and
  `<p>one\n<span style="color:red"></span>&#8203;two</p>` gives one word. A line break directly
  before the zero-width space in the same text is not changed: `<p>one\n&#8203;two</p>` keeps its
  line break.

- A `<footer>`, a `<section>`, an `<article>`, an `<aside>`, a `<header>` and a `<main>` start with
  no space. White space at the start of a block is no space, and the full converter wrote one
  there. This changes the default output: `x<footer> Logo</footer>` gave `x` and then `Logo` with
  a space at the start of its line, and gives `x` and then `Logo` with none. The same holds for an
  inline element at the start: `x<footer><b> Logo</b></footer>` gave a space and `**Logo**` and
  gives `**Logo**`.

- The strict white space mode writes what it wrote before these changes.

- The white space after an image, an inline graphic, a video or a form control at the start of a
  document is kept. This changes the default output: `<p><img src="/i.png" alt="alt"> text</p>`
  gave `![alt](/i.png)text` and gives `![alt](/i.png) text`, and
  `<p><a href="/x"><img src="/i.png" alt="alt"> text</a></p>` gave `[![alt](/i.png)text](/x)` and
  gives `[![alt](/i.png) text](/x)`. With no white space in the source the output has none
  ([#762](https://github.com/xberg-io/html-to-markdown/issues/762)).

- Blocks inside a link label or a heading are separate words at every depth of nesting. This
  changes the default output:
  `<a href="/x"><div><div>Next</div><div>Cross-references</div></div></a>` gave
  `[NextCross-references](/x)` and gives `[Next Cross-references](/x)`, and
  `<h2><p>a</p><p>b</p></h2>` gave `## ab` and gives `## a b`. Inline content beside a block in a
  link no longer gets a space that the source does not have:
  `<a href="/x"><b>H</b>ello<div>x</div></a>` gave `[**H** ello x](/x)` and gives
  `[**H**ello x](/x)`. The text of an image stays a word of its own there:
  `<a href="/x"><img src="/i.png" alt="Logo"><span>Docs</span><p>desc</p></a>` gives
  `[Logo Docs desc](/x)`, as before
  ([#751](https://github.com/xberg-io/html-to-markdown/issues/751)).

- An inline element that holds only a tab is one space: `<p>one<span>\t</span>two</p>` gave `one`,
  a tab and `two`, and gives `one two`.

- White space on the two sides of an element that writes nothing is one space, also in a table
  cell: `<p>one <span id="a"></span> two</p>` gave `one  two` with two spaces and gives `one two`,
  and `<p><span>one </span> two</p>` does the same. The rule holds for every element that writes
  nothing: an image that `inline_data_media: DropElement` removes, a link with no name, an
  element that a visitor skips. With `DropElement`,
  `<p>Before <img src="data:image/png;base64,AAAA" alt="icon"> after</p>` gave `Before  after`
  and gives `Before after`.

- A no-break space in an element of its own is kept after a space:
  `<p>one <span>&nbsp;</span> two</p>` gave `one  two` and gives `one`, a space, the no-break
  space, a space and `two`. A no-break space in running text is one space, as before. In a link
  label a no-break space beside a space is one space on both converters.

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

## Archives

- [3.16.0 through 3.14.0](changelog-archive-6.md)
- [3.13.0 through 3.11.5](changelog-archive-1.md)
- [3.11.4 through 3.6.21](changelog-archive-2.md)
- [3.6.20 through 3.2.0](changelog-archive-3.md)
- [3.1.0 through 2.14.1](changelog-archive-4.md)
- [2.14.0 through 1.x](changelog-archive-5.md)
