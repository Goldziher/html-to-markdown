# Rust API Reference

Crate name: `html-to-markdown-rs`

## Cargo.toml

```toml
[dependencies]
html-to-markdown-rs = "3"
# Default features include: metadata
# Available feature flags:
#   metadata       - HtmlMetadata extraction (default)
#   inline-images  - Inline image/SVG extraction
#   visitor        - Custom element visitor pattern
#   serde          - Serde serialize/deserialize for options/results
#   full           - All features: inline-images, metadata, visitor, serde
```

## Primary Function

```rust
pub fn convert(
    html: &str,
    options: Option<ConversionOptions>,
) -> Result<ConversionResult>
```

Returns a `ConversionResult` containing converted text, tables, metadata, images, and warnings. This is the single entry point for all conversions.

```rust
use html_to_markdown_rs::{convert, ConversionOptions};

// Simple conversion
let result = convert("<h1>Hello</h1>", None)?;
println!("{}", result.content.unwrap_or_default());

// With options
let opts = ConversionOptions::builder()
    .heading_style(HeadingStyle::Atx)
    .build();
let result = convert("<h1>Hello</h1>", Some(opts))?;
```

## ConversionResult

`ConversionResult` derives `Serialize` and `Deserialize` (always, not gated on a feature).

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversionResult {
    /// Converted text output (Markdown, Djot, or plain text).
    /// None in extraction-only mode (content output suppressed). `OutputFormat`
    /// has only Markdown, Djot, and Plain — there is no `None` variant.
    pub content: Option<String>,

    /// Structured document tree. Populated when include_document_structure = true.
    pub document: Option<DocumentStructure>,

    /// Extracted HTML metadata. Requires "metadata" feature (default).
    #[cfg(feature = "metadata")]
    pub metadata: HtmlMetadata,

    /// Extracted tables with structured cell data and markdown representation.
    pub tables: Vec<TableData>,

    /// Extracted inline images (data URIs and SVGs). Requires "inline-images" feature.
    #[cfg(feature = "inline-images")]
    pub images: Vec<InlineImage>,

    /// Non-fatal processing warnings.
    pub warnings: Vec<ProcessingWarning>,
}
```

## ConversionOptions Builder

`ConversionOptions::builder()` returns `ConversionOptionsBuilder` (now public, exported from crate root). Call `.build()` to produce final options.

```rust
use html_to_markdown_rs::{
    ConversionOptions, HeadingStyle, ListIndentType, CodeBlockStyle,
    NewlineStyle, HighlightStyle, OutputFormat, WhitespaceMode,
    PreprocessingOptions, PreprocessingPreset,
};

let options = ConversionOptions::builder()
    // Output control
    .output_format(OutputFormat::Markdown)      // Markdown | Djot | Plain
    .include_document_structure(false)
    .extract_metadata(true)
    .extract_images(false)                       // requires inline-images feature

    // Markdown formatting
    .heading_style(HeadingStyle::Atx)           // Atx | Underlined | AtxClosed
    .list_indent_type(ListIndentType::Spaces)   // Spaces | Tabs
    .list_indent_width(2usize)
    .bullets("-*+")                               // default "-*+"; cycles per nesting level
    .strong_em_symbol('*')                       // '*' or '_'
    .code_block_style(CodeBlockStyle::Backticks) // Indented | Backticks | Tildes (default: Backticks)
    .newline_style(NewlineStyle::Spaces)         // Spaces | Backslash
    .highlight_style(HighlightStyle::DoubleEqual) // DoubleEqual | Html | Bold | None
    .code_language("")                           // default language for fenced code blocks
    .autolinks(true)
    .default_title(false)
    .br_in_tables(false)
    .sub_symbol("")                              // e.g. "~"
    .sup_symbol("")                              // e.g. "^"

    // Escaping
    .escape_asterisks(false)
    .escape_underscores(false)
    .escape_misc(false)
    .escape_ascii(false)

    // Whitespace / wrapping
    .whitespace_mode(WhitespaceMode::Normalized) // Normalized | Strict
    .strip_newlines(false)
    .wrap(false)
    .wrap_width(80usize)

    // Element handling
    .convert_as_inline(false)
    .skip_images(false)
    .strip_tags(vec![])                          // tag names to strip (text only)
    .preserve_tags(vec![])                       // tag names to preserve as HTML
    .keep_inline_images_in(vec![])               // parent tags where images stay inline

    // Inline image extraction (requires inline-images feature)
    .max_image_size(5_242_880u64)               // 5 MiB default
    .capture_svg(false)
    .infer_dimensions(true)

    // Preprocessing
    .preprocessing(PreprocessingOptions {
        enabled: false,
        preset: PreprocessingPreset::Standard,  // Minimal | Standard | Aggressive
        remove_navigation: true,
        remove_forms: true,
    })

    // Encoding and debug
    .encoding("utf-8")
    .debug(false)

    .build();
```

## ConversionOptions Fields (Direct Access)

`ConversionOptions` is a plain struct — all fields are public. You can also construct it directly:

```rust
let options = ConversionOptions {
    heading_style: HeadingStyle::Atx,
    list_indent_width: 4,
    ..ConversionOptions::default()
};
```

## Accessing Metadata, Images, and Tables

All structured data is returned in the single `ConversionResult` from `convert()`. Enable the relevant options to populate each field:

```rust
use html_to_markdown_rs::{convert, ConversionOptions};

// Metadata — available by default (requires "metadata" feature, which is default)
let result = convert(html, None)?;
let meta = &result.metadata;
println!("{:?}", meta.document.title);

// Tables — populated when include_document_structure is enabled
for table in &result.tables {
    println!("{}", table.markdown);
    println!("{:?}", table.grid.cells);
}

// Inline images — requires "inline-images" feature and extract_images: true
let opts = ConversionOptions::builder()
    .extract_images(true)
    .build();
let result = convert(html, Some(opts))?;
for image in &result.images {
    println!("{:?}", image.format);
}

// Custom visitor — pass via options (requires "visitor" feature)
let opts = ConversionOptions::builder()
    // visitor is configured as part of options
    .build();

// Document structure
let opts = ConversionOptions::builder()
    .include_document_structure(true)
    .build();
let result = convert(html, Some(opts))?;
if let Some(doc) = &result.document {
    println!("{} nodes", doc.nodes.len());
}

// Plain string output
let result = convert(html, None)?;
let markdown: String = result.content.unwrap_or_default();
```

## JSON Configuration (requires `serde` or `metadata` feature)

`ConversionOptions` derives `Serialize`/`Deserialize`, so deserialize it with `serde_json`
directly — there are no `*_from_json` helper functions on the public surface.

```rust
let options: ConversionOptions = serde_json::from_str(json)?;
let result = convert(html, Some(options))?;
```

Unknown fields are rejected (`deny_unknown_fields`), so a typo in the JSON is an error rather
than a silently ignored key.

## DocumentStructure Types

```rust
pub struct DocumentStructure {
    pub nodes: Vec<DocumentNode>,
    pub source_format: Option<String>,  // always "html"
}

pub struct DocumentNode {
    pub id: String,
    pub content: NodeContent,
    pub parent: Option<u32>,
    pub children: Vec<u32>,
    pub annotations: Vec<TextAnnotation>,
    pub attributes: Option<HashMap<String, String>>,
}

pub enum NodeContent {
    Heading { level: u8, text: String },
    Paragraph { text: String },
    List { ordered: bool },
    ListItem { text: String },
    Table { grid: TableGrid },
    Image { description: Option<String>, src: Option<String>, image_index: Option<u32> },
    Code { text: String, language: Option<String> },
    Quote,
    DefinitionList,
    DefinitionItem { term: String, definition: String },
    RawBlock { format: String, content: String },
    MetadataBlock { entries: Vec<(String, String)> },
    Group { label: Option<String>, heading_level: Option<u8>, heading_text: Option<String> },
}

pub struct TextAnnotation {
    pub start: u32,
    pub end: u32,
    pub kind: AnnotationKind,
}

// AnnotationKind implements Default (default variant: Bold)
#[derive(Default)]
pub enum AnnotationKind {
    #[default]
    Bold, Italic, Underline, Strikethrough, Code,
    Subscript, Superscript, Highlight,
    Link { url: String, title: Option<String> },
}

// NodeContent implements Default (default: Heading { level: 1, text: String::new() })
pub enum NodeContent { ... }
```

## Error Types

```rust
pub enum ConversionError {
    ParseError(String),
    SanitizationError(String),
    ConfigError(String),
    IoError(std::io::Error),
    Panic(String),
    InvalidInput(String),          // binary data, invalid UTF-8
    #[cfg(feature = "visitor")]
    Visitor(String),
    Other(String),
}

pub type Result<T> = std::result::Result<T, ConversionError>;
```

## Metadata Types (requires `metadata` feature)

```rust
pub struct HtmlMetadata {
    pub document: DocumentMetadata,
    pub headers: Vec<HeaderMetadata>,
    pub links: Vec<LinkMetadata>,
    pub images: Vec<ImageMetadata>,
    pub structured_data: Vec<StructuredData>,
}

pub struct DocumentMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub author: Option<String>,
    pub canonical_url: Option<String>,
    pub base_href: Option<String>,
    pub language: Option<String>,
    pub text_direction: Option<TextDirection>,
    pub open_graph: BTreeMap<String, String>,
    pub twitter_card: BTreeMap<String, String>,
    pub meta_tags: BTreeMap<String, String>,
}

pub struct MetadataConfig {
    pub extract_document: bool,     // default: true
    pub extract_headers: bool,      // default: true
    pub extract_links: bool,        // default: true
    pub extract_images: bool,       // default: true
    pub extract_structured_data: bool,  // default: true
    pub max_structured_data_size: usize,
}
```

## TableData Types

`TableData` is exported from the crate root (`use html_to_markdown_rs::TableData`).

```rust
pub struct TableData {
    pub grid: TableGrid,
    pub markdown: String,
}

pub struct TableGrid {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<GridCell>,
}

pub struct GridCell {
    pub content: String,
    pub row: usize,
    pub col: usize,
    pub row_span: usize,
    pub col_span: usize,
    pub is_header: bool,
}
```
