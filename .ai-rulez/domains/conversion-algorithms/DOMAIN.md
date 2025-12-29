# Conversion Algorithms Domain

## Purpose

The Conversion Algorithms domain implements the core HTML→Markdown transformation logic that converts parsed DOM trees into well-formatted Markdown output. This domain provides standardized conversion patterns for 60+ HTML element types, including block elements (headings, paragraphs, lists, tables, blockquotes), inline formatting (bold, italic, code), semantic HTML5 elements, forms, and media elements. The domain ensures consistent, semantically-accurate Markdown output with proper handling of nested structures, edge cases, and whitespace normalization.

## Key Responsibilities

### 1. Block Element Conversion

- **Headings** (`<h1>-<h6>`):
  - Convert to Markdown headings using `#` syntax (ATX) or underline (Setext)
  - Configurable heading style via `HeadingStyle` enum
  - Preserve heading attributes (id, class) as metadata

- **Paragraphs** (`<p>`):
  - Convert to Markdown paragraphs (text + blank line separator)
  - Preserve inline formatting within paragraphs
  - Handle nested block elements (rare but valid)

- **Block Quotes** (`<blockquote>`):
  - Convert to Markdown blockquotes using `>` prefix
  - Recursive nesting support (nested blockquotes)
  - Preserve formatting within blockquotes

- **Horizontal Rules** (`<hr>`):
  - Convert to Markdown horizontal rules (`---`, `***`, `___`)
  - Configurable style option

- **Preformatted Text** (`<pre>`):
  - Convert to code blocks with triple backticks
  - Preserve whitespace exactly
  - Configurable indentation style (backticks or indent)

- **Semantic HTML5 Elements** (`<article>`, `<section>`, `<nav>`, `<aside>`, `<header>`, `<footer>`):
  - Convert to headings or sections based on context
  - Insert descriptive labels or semantic markers
  - Preserve structural hierarchy

### 2. List Conversion

- **Unordered Lists** (`<ul>`):
  - Convert to Markdown unordered lists using `-`, `*`, or `+`
  - Configurable bullet style
  - Preserve nesting depth

- **Ordered Lists** (`<ol>`):
  - Convert to Markdown ordered lists using `1.`, `2.`, etc.
  - Preserve start attribute if specified
  - Handle custom list styles (roman numerals, letters - convert to decimal)

- **Definition Lists** (`<dl>`, `<dt>`, `<dd>`):
  - Convert to pseudo-table or structured text format
  - Preserve definition relationships
  - Handle multiple definitions per term

- **List Item Conversion** (`<li>`):
  - Convert list item content (with nested lists)
  - Preserve inline formatting
  - Tight vs. loose list detection
  - Multi-paragraph list items with indentation

- **Nested Lists**:
  - Proper indentation for nested levels
  - Preserve nesting structure (3+ levels deep)
  - Handle mixed list types (ul within ol, etc.)

- **Task Lists** (`<input type="checkbox">`):
  - Detect task list items
  - Convert to `- [ ]` and `- [x]` Markdown syntax
  - Preserve checked state

### 3. Table Conversion

- **Table Structure** (`<table>`, `<tr>`, `<td>`, `<th>`):
  - Convert to Markdown pipe tables (GFM format)
  - Preserve table headers (`<thead>`, `<th>`)
  - Handle table bodies (`<tbody>`)
  - Support table footers (`<tfoot>`)

- **Cell Formatting**:
  - Extract cell content with inline formatting preserved
  - Handle rowspan/colspan (convert to multiple cells or nested content)
  - Cell alignment preservation (left, center, right via `align` attr or CSS)
  - Cell padding/spacing handling

- **Table Edge Cases**:
  - Missing cells (uneven rows)
  - Merged cells (rowspan/colspan)
  - Nested tables (convert to alternatives)
  - Complex table layouts (data tables with captions)

- **Alternative Table Formats**:
  - GFM pipe tables (default)
  - Plain text table representation
  - CSV-style tables
  - Fallback to list format for complex tables

### 4. Inline Element Conversion

- **Text Formatting**:
  - **Bold** (`<strong>`, `<b>`): Convert to `**text**`
  - **Italic** (`<em>`, `<i>`): Convert to `*text*`
  - **Strikethrough** (`<s>`, `<del>`): Convert to `~~text~~` (GFM)
  - **Underline** (`<u>`): Convert to `__text__` or alt syntax
  - **Superscript/Subscript** (`<sup>`, `<sub>`): Convert to `^text` or `~text` (markdown extensions)

- **Code & Monospace**:
  - **Inline Code** (`<code>`): Convert to backtick-wrapped text
  - **Keyboard Input** (`<kbd>`): Convert to backticks or alt syntax
  - **Sample Output** (`<samp>`): Convert to backticks
  - **Variable** (`<var>`): Convert to backticks or italics
  - **Preserve Code Content**: No interpretation of code text (no formatting)

- **Emphasis Nesting**:
  - Detect nested emphasis (e.g., `<strong><em>text</em></strong>`)
  - Convert to `***text***` (or alternate syntax)
  - Avoid over-escaped output

- **Links** (`<a>`):
  - Extract href and convert to `[text](url)` format
  - Preserve link title as optional `"title"` parameter
  - Handle fragment URLs (#anchor)
  - Detect and normalize URL schemes (http, https, mailto, ftp)
  - Fallback for links without href (treat as plain text)

- **Images** (`<img>`):
  - Extract src and alt text, convert to `![alt](src)` format
  - Preserve image title as optional parameter
  - Handle responsive images (`srcset`, `sizes` attributes)
  - Fallback for images without src (use alt text or URL)
  - Support data URIs and inline base64 images (optional)

- **Abbreviations** (`<abbr>`):
  - Preserve abbreviation with title
  - Convert to `text (full form)` or Markdown footnotes
  - Handle acronyms

- **Marks** (`<mark>`):
  - Highlight syntax (` ==text== ` or `~~text~~`)
  - Configurable highlight style

### 5. Form Element Conversion

- **Input Fields** (`<input>`):
  - Text inputs: Convert to fenced code or Markdown form notation
  - Checkboxes: Convert to task list syntax
  - Radio buttons: Convert to list with indicator
  - Hidden inputs: Skip or note in comments

- **Text Areas** (`<textarea>`):
  - Convert to code blocks with explicit label

- **Select Dropdowns** (`<select>`):
  - Convert to list of options
  - Indicate selected option

- **Buttons** (`<button>`, `<input type="button">`):
  - Convert to `[Button: label]` notation or skip
  - Preserve button type (submit, reset, button)

- **Fieldsets** (`<fieldset>`):
  - Convert to labeled sections
  - Preserve legend text

- **Labels** (`<label>`):
  - Associate with form inputs
  - Include label text in output

### 6. Media & Embedded Content

- **Audio** (`<audio>`):
  - Extract src, preserve audio link
  - Convert to `[Audio: filename]` notation

- **Video** (`<video>`):
  - Extract src, preserve video link
  - Convert to `[Video: filename]` notation
  - Support poster attribute (thumbnail description)

- **IFrames** (`<iframe>`):
  - Extract src URL
  - Convert to link or `[Embedded: URL]` notation
  - Preserve title attribute

- **Embed & Object** (`<embed>`, `<object>`):
  - Handle various media types
  - Extract URL or data attribute
  - Preserve type information

- **Picture Elements** (`<picture>`, `<source>`):
  - Select best source based on media queries
  - Fall back to img element
  - Preserve responsive image information

### 7. Special Elements

- **Line Breaks** (`<br>`):
  - Convert to two spaces + newline (`  \n`) or backslash newline (`\\\n`)
  - Configurable via `NewlineStyle` option

- **Comments** (`<!-- -->`):
  - Skip by default
  - Preserve with `<!-- -->` syntax if configured
  - Handle multi-line comments

- **Scripts & Styles** (`<script>`, `<style>`):
  - Skip entirely (no content extraction)
  - Warn if configured

- **Metadata** (`<meta>`, `<head>`):
  - Skip entirely
  - Optionally extract and preserve metadata

- **SVG Inline** (`<svg>`):
  - Extract text content
  - Preserve or convert to ASCII representation
  - Fall back to alt text if available

- **Ruby Annotations** (`<ruby>`, `<rt>`, `<rp>`):
  - Preserve annotation text
  - Convert to `text(annotation)` format

## Core Components

### Element Conversion Registry (`converter.rs`)

```rust
pub struct ElementConverter {
    converters: HashMap<&'static str, fn(&Element) -> Result<String>>,
}

impl ElementConverter {
    pub fn convert_element(&self, tag: &str, elem: &Element) -> Result<String>;
    pub fn register_converter(&mut self, tag: &str, fn: fn(&Element) -> Result<String>);
}
```

### Block Element Converters

```rust
pub fn convert_heading(level: u8, text: &str, opts: &ConversionOptions) -> String;
pub fn convert_paragraph(content: &str, opts: &ConversionOptions) -> String;
pub fn convert_blockquote(content: &str, opts: &ConversionOptions) -> String;
pub fn convert_list(
    items: Vec<String>,
    ordered: bool,
    nested_depth: usize,
    opts: &ConversionOptions,
) -> String;
pub fn convert_list_item(content: &str, depth: usize, opts: &ConversionOptions) -> String;
pub fn convert_table(
    rows: Vec<Vec<String>>,
    headers: Vec<String>,
    alignments: Vec<Alignment>,
    opts: &ConversionOptions,
) -> String;
```

### Inline Element Converters

```rust
pub fn convert_bold(content: &str) -> String;
pub fn convert_italic(content: &str) -> String;
pub fn convert_inline_code(content: &str) -> String;
pub fn convert_link(text: &str, href: &str, title: Option<&str>) -> String;
pub fn convert_image(alt: &str, src: &str, title: Option<&str>) -> String;
pub fn convert_strikethrough(content: &str) -> String;
```

### Text Processing (`text.rs`)

```rust
pub fn escape_markdown(text: &str) -> String;
pub fn chomp_whitespace(text: &str) -> String;
pub fn normalize_whitespace(text: &str, mode: WhitespaceMode) -> String;
pub fn preserve_linebreaks(text: &str) -> String;
```

### List Handling

```rust
pub fn is_tight_list(items: &[String]) -> bool;
pub fn calculate_list_indent(depth: usize, opts: &ConversionOptions) -> String;
pub fn detect_task_list_item(content: &str) -> Option<bool>; // None = not task, Some(checked)
```

### Table Utilities

```rust
pub fn detect_table_alignment(td: &TableCell) -> Alignment;
pub fn escape_table_cell(content: &str) -> String;
pub fn build_table_separator(columns: usize, alignments: &[Alignment]) -> String;
```

## Integration with html-to-markdown Conversion

### Conversion Pipeline

```
Parsed DOM Tree
    ↓
walk_tree() [Depth-first traversal]
    ↓
visit_element(tag) [Match element type]
    ↓
Dispatch to converter:
  - convert_heading(h1, content) → "# content\n"
  - convert_paragraph(content) → "content\n\n"
  - convert_link(text, href) → "[text](href)"
  - convert_table(rows, headers) → "| h1 | h2 |\n|---|---|\n..."
    ↓
Accumulate Markdown fragments in buffer
    ↓
Join and normalize whitespace
    ↓
Final Markdown Output
```

### Visitor Pattern Integration

Converters invoked through visitor pattern in `visitor.rs`:

```rust
impl HtmlVisitor for MarkdownConverter {
    fn visit_h1(&mut self, ctx: &NodeContext, content: &str) -> VisitResult {
        VisitResult::Custom(convert_heading(1, content, &self.opts))
    }

    fn visit_p(&mut self, ctx: &NodeContext, content: &str) -> VisitResult {
        VisitResult::Custom(convert_paragraph(content, &self.opts))
    }

    fn visit_a(&mut self, ctx: &NodeContext, href: &str, content: &str, title: Option<&str>) -> VisitResult {
        VisitResult::Custom(convert_link(content, href, title))
    }
}
```

### Configurable Conversion Behavior

Options control conversion output via `ConversionOptions`:

```rust
pub struct ConversionOptions {
    pub heading_style: HeadingStyle,         // ATX or Setext
    pub list_indent_type: ListIndentType,    // Spaces or Tab
    pub code_block_style: CodeBlockStyle,    // Backticks or Indent
    pub newline_style: NewlineStyle,         // TwoSpaces or Backslash
    pub table_format: TableFormat,           // GFM or ASCII
    pub preserve_attributes: bool,           // Keep class, id in output
    pub link_reference_style: bool,          // [text][ref] vs [text](url)
}
```

## Data Flow

### Element Processing

1. **Element Dispatch**: Identify HTML tag type
2. **Content Extraction**: Recursively convert child elements
3. **Attribute Extraction**: Get href, src, title, etc.
4. **Conversion Selection**: Choose appropriate converter function
5. **Format Application**: Apply Markdown syntax
6. **Whitespace Normalization**: Clean up surrounding whitespace
7. **Return Markdown**: Append to output buffer

### Nested Structure Handling

```
<ul>
  <li>Item 1
    <ul>
      <li>Nested 1.1</li>
      <li>Nested 1.2</li>
    </ul>
  </li>
  <li>Item 2</li>
</ul>

↓ convert_list(
    items: [
      "Item 1\n  - Nested 1.1\n  - Nested 1.2",
      "Item 2"
    ],
    ordered: false,
    depth: 0
  )

↓

- Item 1
  - Nested 1.1
  - Nested 1.2
- Item 2
```

## Dependencies & Relationships

### Upstream Dependencies

- **HTML Parsing Domain**: Provides parsed DOM tree and element navigation
- **Safety-Sanitization Domain**: Validates element attributes (URLs, content)
- **Text Processing**: Markdown escaping, whitespace handling

### Downstream Dependencies

- **Visitor Pattern**: Conversion functions called through visitor interface
- **Output Formatting**: Whitespace normalization and final output generation
- **Metadata Extraction**: Extract link/image metadata alongside conversion

## Performance Characteristics

### Conversion Speed

- **Per-Element**: 0.1-1ms per element (1000 elements/second)
- **Text Processing**: 0.01ms per 1000 characters
- **Table Conversion**: 1-10ms per table (depends on size)
- **Full Document**: 100KB HTML → 2-50ms conversion time

### Memory Usage

- **Conversion Buffer**: ~2x input HTML size (temporary strings)
- **Fragment Cache**: Optional LRU cache for repeated elements
- **Stack Usage**: Proportional to nesting depth

## Testing & Validation

### Conversion Correctness

- **Element Coverage**: Test all 60+ supported HTML tags
- **Markdown Validity**: Generated Markdown must be valid
- **Round-trip**: HTML → Markdown → HTML should preserve structure
- **Edge Cases**: Empty elements, deeply nested, malformed
- **Whitespace**: Verify whitespace handling per mode

### Output Quality

- **Semantic Accuracy**: Converted Markdown represents original intent
- **Formatting Preservation**: Bold, italic, links rendered correctly
- **Table Structure**: Tables converted with correct alignment
- **Code Blocks**: Preserve code content verbatim
- **Special Characters**: Proper escaping of Markdown metacharacters

### Performance Benchmarks

- **Large Documents**: 1-10MB HTML conversion time
- **Deeply Nested**: 1000+ levels of nesting
- **Many Elements**: 10,000+ elements per document
- **Complex Tables**: 100+ rows/columns per table

## Future Enhancements

- Custom element converters via plugin system
- Advanced table conversion (complex colspan/rowspan)
- Footnote support for links and references
- Definition list variants
- Automatic alt text generation for images
- Markdown flavor detection and output targeting
- Math formula support (LaTeX, MathML)
- Diagram/flowchart conversion (mermaid, plantuml)
- Performance optimization with converter caching
- Parallel element conversion for large documents
