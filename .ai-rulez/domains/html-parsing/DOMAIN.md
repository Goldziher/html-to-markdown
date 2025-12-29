# HTML Parsing Domain

## Purpose

The HTML Parsing domain manages the foundation of the html-to-markdown conversion library: selecting and configuring HTML parsers, implementing robust DOM tree traversal, and providing safe, efficient navigation through the parsed HTML structure. This domain ensures reliable HTML parsing across diverse HTML inputs (well-formed, malformed, legacy HTML, etc.) and provides the fundamental tree-walking infrastructure that all conversion algorithms depend on.

## Key Responsibilities

### 1. HTML Parser Selection & Configuration

- **Parser Architecture**: Support multiple HTML parser implementations (html5ever vs tl/astral-tl)
  - **html5ever**: Full HTML5 spec compliance, handles malformed HTML robustly, more memory overhead
  - **tl (astral-tl)**: Lightweight, fast, modern HTML parsing with lower memory footprint
  - **Parser Configuration**: DOM parser vs. tokenizer mode, namespace handling, entity resolution

- **Parser Capabilities**:
  - Automatic character encoding detection and UTF-8 conversion
  - Malformed HTML recovery (unclosed tags, attribute parsing, etc.)
  - Entity decoding (&nbsp;, &lt;, etc.)
  - Namespace support (SVG, MathML within HTML)
  - DOM tree construction with parent-child relationships

- **Parser Selection Heuristics**:
  - Choose html5ever for standards compliance and legacy HTML
  - Choose tl/astral-tl for performance when HTML quality is known
  - Configurable via `ParserType` enum with fallback behavior

### 2. DOM Tree Traversal

- **Tree Navigation Methods**:
  - Parent traversal: Walk up to document root
  - Child iteration: Process child nodes in document order
  - Sibling navigation: Move between adjacent elements
  - Depth-first traversal: Systematic DOM exploration
  - Filtered iteration: Skip text nodes, comments, etc.

- **Node Type Identification**:
  - Element nodes (60+ HTML tags): `<h1>`, `<div>`, `<p>`, `<table>`, etc.
  - Text nodes: Raw text content
  - Comment nodes: HTML comments (preserved or filtered)
  - Document nodes: Root element
  - Fragment nodes: Detached subtrees

- **Attribute Access**:
  - Retrieve attributes by name: `class`, `id`, `href`, `src`
  - Iterate all attributes
  - Case-insensitive attribute names (per HTML spec)
  - URL-encoded attribute values

### 3. Text Extraction & Whitespace Handling

- **Text Content Extraction**:
  - Extract all text nodes from element subtree
  - Preserve original whitespace (no HTML5 normalization)
  - Handle special text nodes (CDATA, etc.)
  - Decode HTML entities in text

- **Whitespace Handling Strategy**:
  - **Preserve**: All whitespace in text nodes retained as-is
  - **No Normalization**: Don't apply HTML5 whitespace collapsing
  - **Configure Via**: `WhitespaceMode` enum (Preserve, Minimal, Collapse)
  - **Newlines**: Preserve hard line breaks (newlines in source HTML)

- **Text Cleaning Options**:
  - Remove control characters (except tab, newline)
  - Collapse multiple spaces optionally
  - Trim leading/trailing whitespace per context

### 4. Safety Constraints

- **Safe Pointer Navigation**: No dangling pointers from tree walking
  - Parent pointers validated before use
  - Child lists checked for validity
  - Depth limits prevent stack overflow on deeply nested HTML

- **Memory Management**:
  - Efficient tree representation (avoid copying)
  - Reference counting for shared subtrees
  - Bounded memory allocation (configurable limits)

- **Input Validation**:
  - Reject binary data before parsing
  - Detect UTF-16/UTF-32 encoding mismatches
  - Validate HTML structure before traversal
  - Enforce size limits (max document size, max nesting depth)

## Core Components

### Parser Infrastructure (`converter.rs`, `wrapper.rs`)

Primary HTML parsing entry points:

- `parse_html()` - Initialize HTML parser with configuration
- `validate_input()` - Binary detection and encoding validation
- Parser state management and error recovery
- DOM tree construction from parser output

### DOM Traversal Helper (`visitor.rs`, `visitor_helpers.rs`)

Node navigation and element inspection:

```rust
pub trait DomWalker {
    fn walk_tree(&self, node: &Node, visitor: &mut dyn HtmlVisitor) -> Result<()>;
    fn visit_element(&self, elem: &Element) -> Result<()>;
    fn visit_children(&self, node: &Node) -> Result<()>;
    fn is_block_element(&self, tag: &str) -> bool;
    fn is_void_element(&self, tag: &str) -> bool;
}
```

### Element Context (`visitor.rs`)

Current node information during traversal:

```rust
pub struct NodeContext {
    pub node_type: NodeType,
    pub tag_name: Option<String>,
    pub depth: usize,
    pub index_in_parent: usize,
    pub attributes: BTreeMap<String, String>,
}
```

### Attribute Access

```rust
pub fn get_attribute(elem: &Element, name: &str) -> Option<String>;
pub fn get_attributes(elem: &Element) -> BTreeMap<String, String>;
pub fn has_class(elem: &Element, class: &str) -> bool;
pub fn get_classes(elem: &Element) -> Vec<String>;
```

### Text Extraction (`text.rs`)

```rust
pub fn extract_text(node: &Node) -> String;
pub fn extract_text_deep(node: &Node, whitespace_mode: WhitespaceMode) -> String;
pub fn chomp_whitespace(text: &str) -> String;
pub fn normalize_whitespace(text: &str, mode: WhitespaceMode) -> String;
```

### Tag Classification

```rust
pub enum ElementCategory {
    Block,       // div, p, h1-h6, ul, ol, dl, table, etc.
    Inline,      // span, strong, em, code, a, img, etc.
    Void,        // br, img, input, hr, etc.
    FormControl, // input, button, textarea, select, etc.
    Semantic,    // article, section, nav, aside, header, footer
}

pub fn categorize_element(tag: &str) -> ElementCategory;
pub fn is_block_element(tag: &str) -> bool;
pub fn is_inline_element(tag: &str) -> bool;
pub fn is_void_element(tag: &str) -> bool;
```

## Integration with html-to-markdown Conversion

### Parsing Pipeline

The overall conversion flow from HTML to Markdown:

```
Input HTML String
    ↓
validate_input() [Binary detection, encoding check]
    ↓
parse_html() [html5ever or tl parser]
    ↓
DOM Tree (parent-child relationships, attributes)
    ↓
walk_tree() [Depth-first traversal]
    ↓
visit_element() [Per-element conversion logic]
    ↓
Markdown Output String
```

### Converter Integration Points

1. **Initialization**: `parse_html()` builds DOM tree from input
2. **Traversal**: `walk_tree()` iterates through DOM
3. **Element Dispatch**: Determine element type and call appropriate converter
4. **Text Extraction**: Use `extract_text()` for element content
5. **Attribute Access**: Retrieve `href`, `src`, `title`, `class` via attribute helpers

### Parser-Specific Implementations

**html5ever-based path** (`crates/html-to-markdown/src/`):
- Uses `html5ever::parse_document()` for standards-compliant parsing
- Returns `RcDom` tree with reference-counted nodes
- Handles malformed HTML recovery

**tl-based path** (potential `converter/mod.rs`):
- Lightweight `astral-tl` parser for high performance
- Direct DOM access without reference counting overhead
- Suitable for pre-validated HTML

### Configuration Cascade

Parser behavior configured through `ConversionOptions`:

```rust
pub struct ConversionOptions {
    pub parser: ParserType,        // html5ever | tl
    pub encoding: Option<String>,  // UTF-8, ISO-8859-1, etc.
    pub whitespace_mode: WhitespaceMode, // Preserve, Minimal, Collapse
    pub max_depth: usize,          // Stack overflow protection
    pub max_document_size: usize,  // Memory protection
}
```

## Data Flow

### Parsing Phase

1. **Input Validation**: Check for binary data (gzip, PDF, ZIP, UTF-16)
2. **Character Encoding**: Detect and convert to UTF-8
3. **HTML Parsing**: Parse with selected parser
4. **DOM Construction**: Build tree structure with parent/child pointers
5. **Tree Validation**: Check depth limits, node counts

### Traversal Phase

1. **Root Node Discovery**: Start from document element
2. **Recursive Descent**: Depth-first traversal
3. **Element Processing**: Apply visitor pattern
4. **Text Extraction**: Collect text node content
5. **Attribute Access**: Retrieve element metadata
6. **Result Generation**: Accumulate Markdown output

## Dependencies & Relationships

### Upstream Dependencies

- **html5ever**: HTML5 spec-compliant parsing (for standards compliance)
- **astral-tl**: Lightweight, fast HTML parsing (for performance)
- **encoding_rs**: Character encoding detection and conversion
- **regex**: HTML entity decoding
- **rcdom** (markup5ever_rcdom): Reference-counted DOM nodes

### Downstream Dependencies

- **Conversion Algorithms Domain**: Consumes DOM tree for HTML→Markdown conversion
- **Safety-Sanitization Domain**: Input validation (binary detection)
- **Visitor Pattern Implementation**: Uses traversal infrastructure
- **Metadata Extraction**: DOM navigation for link/image analysis

## Performance Characteristics

### Parser Overhead

- **html5ever**: 1-5ms for typical documents (100KB HTML)
- **tl parser**: 0.5-2ms for typical documents (100KB HTML)
- **DOM Tree Memory**: 2-4x input HTML size (node overhead)

### Traversal Performance

- **Element Iteration**: O(n) where n = total node count
- **Attribute Lookup**: O(1) hash table access
- **Text Extraction**: O(m) where m = total text characters
- **Depth-First Walk**: Single pass through tree (no backtracking)

### Memory Usage

- **Parser State**: ~10MB baseline for html5ever
- **DOM Tree**: Proportional to HTML size + node overhead
- **Text Buffers**: Temporary during text extraction

## Testing & Validation

### Parser Correctness

- **HTML Conformance**: Test against HTML5 spec test suite
- **Malformed HTML Recovery**: Verify handling of unclosed tags, broken attributes
- **Character Encoding**: Test UTF-8, Latin-1, other encodings
- **Large Documents**: Verify performance on 10MB+ HTML files
- **Deep Nesting**: Test stack safety with 1000+ nesting depth

### Traversal Correctness

- **Tree Navigation**: Verify parent/child relationships
- **Sibling Access**: Test sibling list iteration
- **Void Elements**: Verify void elements have no children
- **Text Nodes**: Verify text node content preservation
- **Whitespace Preservation**: Confirm whitespace not collapsed

### Edge Cases

- **Empty Documents**: Verify handling of empty/whitespace-only input
- **Binary Data**: Confirm rejection of non-HTML binary
- **Entity Decoding**: Test HTML entities (&#123;, &nbsp;, etc.)
- **Namespace Elements**: Verify SVG/MathML within HTML
- **Comments & CDATA**: Test comment handling, CDATA preservation

## Future Enhancements

- Alternative parser backends (htmlparser2, html-rs)
- Streaming HTML parsing for very large documents
- Incremental DOM updates (live updates from script injections)
- Parallel DOM traversal for multi-core performance
- DOM mutation tracking (what changed from original parse)
- Parser profiling and performance metrics
- Interactive HTML debugging tools
