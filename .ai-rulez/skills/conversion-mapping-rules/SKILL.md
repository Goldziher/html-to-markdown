# Conversion Mapping Rules: HTML Elements to Markdown

## Overview

This skill documents how html-to-markdown maps 60+ HTML element types to their Markdown equivalents. The conversion logic respects Markdown syntax variations (ATX vs Setext headings, fenced vs indented code, etc.) and maintains semantic accuracy.

## Heading Elements (h1-h6)

### ATX Style (Default)

```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6
```

**Implementation:**
- Option: `HeadingStyle::Atx` (default)
- Each heading level uses n hashes
- Single space after hashes required
- Trailing hashes optional (ATX closed style adds them)

**HTML Example:**
```html
<h1>Title</h1>          → # Title
<h2 id="intro">Intro</h2> → ## Intro
<h3>Detail</h3>         → ### Detail
```

### Setext/Underlined Style

```markdown
Heading 1
=========

Heading 2
---------
```

**Implementation:**
- Option: `HeadingStyle::Underlined`
- H1: `=` characters for full line width
- H2: `-` characters for full line width
- H3+ not supported in Setext (fallback to ATX)

**HTML Example:**
```html
<h1>Main Title</h1>  → Main Title\n===========
<h2>Subtitle</h2>    → Subtitle\n---------
<h3>Detail</h3>      → ### Detail (fallback to ATX)
```

### ATX Closed Style

```markdown
# Heading 1 #
## Heading 2 ##
### Heading 3 ###
```

**Implementation:**
- Option: `HeadingStyle::AtxClosed`
- Closing hashes must match opening count
- Single space before closing hashes
- Less common, but valid Markdown

## Block-Level Elements

### Paragraph (`<p>`)

**Mapping:**
- Text content extracted and escaped
- Trailing/leading whitespace trimmed
- Single newline after paragraph

**Example:**
```html
<p>This is a paragraph with <strong>bold</strong> text.</p>
→ This is a paragraph with **bold** text.\n
```

### Division (`<div>`)

**Behavior:**
- Transparent wrapper for Markdown
- Content treated as block-level
- No wrapping markers in output
- Preserves child semantics

**Example:**
```html
<div>
  <p>Paragraph inside div</p>
</div>
→ Paragraph inside div\n
```

### Blockquote (`<blockquote>`)

**Mapping:**
- Each line prefixed with `> `
- Nested blockquotes: `> > `
- Handles multiple paragraphs

**Example:**
```html
<blockquote>
  <p>Quote line 1</p>
  <p>Quote line 2</p>
</blockquote>
→ > Quote line 1\n>\n> Quote line 2\n
```

### Preformatted Text (`<pre>`)

**Behavior:**
- Whitespace preserved exactly
- Treated as code block (see Code Blocks below)
- No entity decoding in content
- Trimmed and indented

**Example:**
```html
<pre>    code with spaces</pre>
→ (indented code or fenced, depends on CodeBlockStyle)
```

### Code Blocks

**Indented Style (Default):**
```markdown
    line 1
    line 2
    line 3
```

**Implementation:**
- Option: `CodeBlockStyle::Indented`
- Each line prefixed with 4 spaces
- Requires blank line before/after
- CommonMark default

**Fenced Backtick Style:**
```markdown
```language
code here
```
```

**Implementation:**
- Option: `CodeBlockStyle::Backticks`
- Triple backticks with optional language specifier
- Language from HTML class (e.g., `language-rust` → `rust`)
- Can contain blank lines

**Fenced Tilde Style:**
```markdown
~~~rust
code here
~~~
```

**Implementation:**
- Option: `CodeBlockStyle::Tildes`
- Triple tildes with optional language specifier
- Less common variant of fenced style

**HTML Mapping:**
```html
<pre><code>simple code</code></pre>
<pre><code class="language-python">def foo(): pass</code></pre>
<pre>indented code</pre>
```

### Horizontal Rule (`<hr>`)

**Output:** `---\n` (three dashes)

**Alternatives:** `***`, `___` all valid but standardized to `---`

### List Elements

#### Unordered Lists (`<ul>`)

**Default Syntax (dashes):**
```markdown
- Item 1
- Item 2
  - Nested item
    - Deeply nested
```

**Implementation:**
- `-` marker (could be `*` or `+`, but `-` is default)
- Indentation for nesting: spaces or tabs
- Option: `ListIndentType::Spaces` (default) or `ListIndentType::Tabs`

#### Ordered Lists (`<ol>`)

```markdown
1. First item
2. Second item
3. Third item
```

**Implementation:**
- `1.` through `9.` for first 9 items (reset per list)
- Number must be followed by `. ` (dot space)
- Indentation matches unordered for nesting

#### List Items (`<li>`)

**Behavior:**
- Content can include block elements (paragraphs, code blocks)
- Continuation lines indented to match marker
- Multi-line items:

```markdown
- First paragraph

  Second paragraph (indented)
```

**HTML Example:**
```html
<ul>
  <li>
    <p>Item with paragraph</p>
    <p>Second paragraph</p>
  </li>
</ul>
```

#### Definition Lists (`<dl>`, `<dt>`, `<dd>`)

```markdown
Term
:   Definition

Another Term
:   Definition 1
:   Definition 2
```

**Implementation:**
- `<dt>`: Term on its own line
- `<dd>`: Definition with `:` prefix and indentation
- Multiple definitions per term supported

### Tables (`<table>`, `<tr>`, `<td>`, `<th>`)

**Mapping:**
```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
```

**Implementation:**
- `<table>` → GFM (GitHub Flavored Markdown) table
- `<thead>` content becomes header row
- `<tbody>` rows become data rows
- Cells separated by `|` pipes
- Separator row: `|---|---|` (minimum 3 dashes)
- Right-alignment: `:---|` Left: `|:--` Center: `:--:`

**Cell Content:**
- Escaped for pipe characters (`|` → `\|`)
- Nested elements converted (e.g., `<strong>` → `**`)
- Newlines converted to `<br>` representation

### Semantic HTML5 Elements

#### Article (`<article>`)
- Treated as transparent block wrapper
- No semantic markers in Markdown
- Content flows as-is

#### Section (`<section>`)
- Transparent block wrapper
- Could insert heading separator in future

#### Nav (`<nav>`)
- List-like wrapper
- Children converted normally
- Could insert navigation markers

#### Aside (`<aside>`)
- Optional blockquote prefix (configurable)
- Or treated as transparent block

#### Header (`<header>`)
- Transparent wrapper
- Content converted normally

#### Footer (`<footer>`)
- Transparent wrapper
- Could insert footer marker (e.g., `---\n`)

#### Main (`<main>`)
- Transparent wrapper
- Content flows normally

## Inline Elements

### Emphasis (`<em>`, `<i>`)

**Mapping:** `*text*` or `_text_`

**Implementation:**
- Default: `*` (asterisk italic)
- No underscore escaping needed in this context
- Trimmed of excess whitespace

**Example:**
```html
<em>emphasized</em>  → *emphasized*
<i>italic</i>        → *italic*
```

### Strong (`<strong>`, `<b>`)

**Mapping:** `**text**`

**Implementation:**
- Double asterisks (bold)
- Trimmed of excess whitespace
- Can be nested with emphasis

**Example:**
```html
<strong>bold</strong>           → **bold**
<b>bold</b>                     → **bold**
<strong><em>bold italic</em></strong> → ***bold italic***
```

### Code (`<code>`)

**Mapping:** `` `text` `` (backtick inline code)

**Implementation:**
- Single backticks for inline
- Escaped if backticks present in content
- No entity decoding within code

**Example:**
```html
<code>variable_name</code>        → `variable_name`
<code>don't</code>                → `don't`
<code>`already_quoted`</code>     → `` `already_quoted` ``
```

### Link (`<a href>`)

**Mapping:** `[link text](url "title")`

**Implementation:**
- `href` attribute becomes URL
- Text content becomes link text
- `title` attribute becomes optional title (in quotes)
- URL preserved as-is (no extra encoding)
- Special link types:
  - `href="#section"` → Anchor link
  - `href="/page"` → Internal link (relative)
  - `href="https://external.com"` → External link
  - `href="mailto:user@example.com"` → Email link
  - `href="tel:+1234567890"` → Phone link

**Examples:**
```html
<a href="https://example.com">Link</a>
→ [Link](https://example.com)

<a href="/page" title="My Page">Internal</a>
→ [Internal](/page "My Page")

<a href="#section">Anchor</a>
→ [Anchor](#section)

<a href="mailto:test@example.com">Email</a>
→ [Email](mailto:test@example.com)
```

### Image (`<img>`)

**Mapping:** `![alt text](url "title")`

**Implementation:**
- `src` attribute becomes URL
- `alt` attribute becomes alt text
- `title` attribute becomes optional title
- Dimensions (`width`, `height`) captured in metadata
- Data URIs: `![alt](data:image/png;base64,...)`
- Relative paths preserved

**Examples:**
```html
<img src="photo.jpg" alt="A photo">
→ ![A photo](photo.jpg)

<img src="image.png" alt="Image" title="My Image" width="200" height="150">
→ ![Image](image.png "My Image")

<img src="data:image/png;base64,..." alt="Embedded">
→ ![Embedded](data:image/png;base64,...)
```

### Line Break (`<br>`)

**Mapping:**
- Two spaces + newline: `  \n`
- Or backslash + newline: `\\\n`

**Option:** `NewlineStyle::Spaces` (default) or `NewlineStyle::Backslash`

**Example:**
```html
<p>Line 1<br>Line 2</p>
→ Line 1  \nLine 2\n
```

### Strikethrough (`<s>`, `<del>`, `<strike>`)

**Mapping:** `~~strikethrough~~`

**Implementation:**
- GFM strikethrough syntax (double tilde)
- Not standard Markdown, but widely supported
- Trimmed of excess whitespace

**Example:**
```html
<del>removed text</del>  → ~~removed text~~
<s>strikethrough</s>     → ~~strikethrough~~
```

### Subscript/Superscript (`<sub>`, `<sup>`)

**Behavior:**
- No native Markdown support
- Typically converted to plain text or HTML passthrough
- Implementation: Extract text content, no markup

**Example:**
```html
H<sub>2</sub>O          → H2O (plain text)
E=mc<sup>2</sup>        → E=mc2 (plain text)
```

### Mark/Highlight (`<mark>`)

**Options:**
1. `HighlightStyle::DoubleEqual`: `==text==`
2. `HighlightStyle::Html`: `<mark>text</mark>`
3. `HighlightStyle::Bold`: `**text**`
4. `HighlightStyle::None`: plain text

**Example:**
```html
<mark>highlighted</mark>
→ ==highlighted==  (DoubleEqual mode)
→ <mark>highlighted</mark>  (Html mode)
→ **highlighted**  (Bold mode)
```

### Ruby Annotations (`<ruby>`, `<rt>`, `<rp>`)

**Mapping:**
- Japanese ruby text support
- Format: `text {rt_text}` or similar
- Implementation: Extract base text with rt annotation

**Example:**
```html
<ruby>漢字<rt>かんじ</rt></ruby>
→ 漢字 (かんじ)
```

## Media Elements

### Audio (`<audio>`)

**Behavior:**
- No direct Markdown equivalent
- Typically extracted as metadata or skipped
- Could insert link to source if `src` attribute

**Handling:**
```html
<audio src="sound.mp3">Audio</audio>
→ (Skipped or converted to link in metadata)
```

### Video (`<video>`)

**Behavior:**
- Similar to audio
- Could extract `poster` image
- Typically skipped in markdown output

### Picture/Source (`<picture>`, `<source>`)

**Behavior:**
- Responsive image container
- Extract from child `<img>` inside
- Or use first source `src`

## Form Elements

### Input (`<input>`)

**Behavior:**
- Generally skipped or marked as form element
- Could convert to metadata about form structure
- Types: text, checkbox, radio, button, hidden

**Implementation:**
- Placeholder preserved in metadata
- Value not typically included in markdown

### Select/Option (`<select>`, `<option>`)

**Behavior:**
- Converted to list or metadata
- Option text extracted
- Selected state noted

### Button (`<button>`)

**Behavior:**
- Text content extracted (ignores `<button>` wrapper)
- Click handlers ignored
- Treated as inline text

### Textarea (`<textarea>`)

**Behavior:**
- Content treated as code block or preformatted
- Whitespace preserved

## Special Elements

### SVG (`<svg>`)

**Behavior:**
- Can be preserved as inline image or skipped
- Feature: `inline-images` can extract inline SVG
- Typically rendered as-is in compatible markdown renderers

### MathML (`<math>`)

**Behavior:**
- Skipped in standard markdown
- Could be preserved with feature gate
- Converted to LaTeX or plain text fallback

### iframe (`<iframe>`)

**Behavior:**
- Generally skipped
- Could extract as metadata (video embeds, etc.)
- URL captured if needed

## Whitespace and Formatting Context

### Whitespace Mode

**Normalized (default):**
- Multiple spaces collapsed to single space
- Multiple newlines → single newline
- Leading/trailing whitespace trimmed per element

**Strict:**
- All whitespace preserved exactly
- Multiple spaces and newlines intact
- Useful for poetry, ASCII art, etc.

### Text Escaping

**Options:**
- `escape_asterisks`: `*` → `\*`
- `escape_underscores`: `_` → `\_`
- `escape_misc`: Special chars `\ & < ` [ > ~ # = + | -`
- `escape_ascii`: All ASCII punctuation (CommonMark spec)

**Example:**
```html
<p>Price: $10 & free shipping *limited time*</p>

escape_misc=true:
→ Price: $10 \& free shipping *limited time*

escape_asterisks=true:
→ Price: $10 & free shipping \*limited time\*

escape_ascii=true:
→ Price: \$10 \& free shipping \*limited time\*
```

## Implementation Details Location

**Key Files:**
- `/crates/html-to-markdown/src/converter.rs` - Element dispatch and conversion
- `/crates/html-to-markdown/src/options.rs` - Style configuration enums
- `/crates/html-to-markdown/src/text.rs` - Text escaping and normalization

## Element Dispatch Example

```rust
// From converter.rs pattern
match element.tag_name() {
    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => convert_heading(...),
    "p" => convert_paragraph(...),
    "a" => convert_link(...),
    "img" => convert_image(...),
    "strong" | "b" => convert_strong(...),
    "em" | "i" => convert_em(...),
    "code" => convert_code(...),
    "pre" => convert_pre(...),
    "blockquote" => convert_blockquote(...),
    "ul" | "ol" => convert_list(...),
    "li" => convert_list_item(...),
    "table" => convert_table(...),
    "br" => convert_br(...),
    "hr" => convert_hr(...),
    // ... 40+ more elements
    _ => convert_generic_element(...)
}
```

## Complete Element Reference

See `/crates/html-to-markdown/src/visitor.rs` for exhaustive `NodeType` enum covering all 60+ supported elements.

## Quick Reference Table

| HTML Element | Markdown Output | Notes |
|--------------|-----------------|-------|
| `<h1>` | `# text` | ATX style default |
| `<p>` | `text\n` | Paragraph |
| `<strong>` | `**text**` | Bold |
| `<em>` | `*text*` | Italic |
| `<a href>` | `[text](url)` | Link |
| `<img>` | `![alt](src)` | Image |
| `<ul>` | `- item` | Unordered list |
| `<ol>` | `1. item` | Ordered list |
| `<code>` | `` `text` `` | Inline code |
| `<pre>` | Indented or fenced | Code block |
| `<blockquote>` | `> text` | Quote |
| `<table>` | GFM table | Pipe-delimited |
| `<br>` | `  \n` | Line break |
| `<hr>` | `---` | Horizontal rule |
| `<del>` | `~~text~~` | Strikethrough |
| `<mark>` | `==text==` | Highlight (configurable) |
