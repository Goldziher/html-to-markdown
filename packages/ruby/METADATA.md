# Metadata Extraction for Ruby Bindings

Complete Ruby Magnus binding implementation for HTML-to-Markdown metadata extraction with full RBS type signatures.

## Features

The Ruby binding provides comprehensive metadata extraction during HTML-to-Markdown conversion:

- **Document Metadata**: title, description, keywords, author, canonical URL, language, text direction
- **Open Graph & Twitter Card**: social media metadata extraction
- **Headers**: h1-h6 extraction with hierarchy, ids, and depth tracking
- **Links**: hyperlink extraction with type classification (anchor, internal, external, email, phone)
- **Images**: image extraction with source type (data_uri, inline_svg, external, relative) and dimensions
- **Structured Data**: JSON-LD, Microdata, and RDFa extraction

## API

### Basic Usage

```ruby
require 'html_to_markdown'

html = '<html lang="en"><head><title>Test</title></head><body><h1>Hello</h1></body></html>'
markdown, metadata = HtmlToMarkdown.convert_with_metadata(html)

puts markdown
puts metadata[:document][:title]  # "Test"
puts metadata[:headers].length     # 1
```

### With Conversion Options

```ruby
conv_opts = { heading_style: :atx_closed }
metadata_opts = { extract_headers: true, extract_links: false }

markdown, metadata = HtmlToMarkdown.convert_with_metadata(
  html,
  conv_opts,
  metadata_opts
)
```

### Return Value

Returns a 2-element array: `[markdown_string, metadata_hash]`

The metadata hash contains:

```ruby
{
  document: {
    title: String?,
    description: String?,
    keywords: Array[String],
    author: String?,
    canonical_url: String?,
    base_href: String?,
    language: String?,
    text_direction: "ltr" | "rtl" | "auto" | nil,
    open_graph: Hash[String, String],
    twitter_card: Hash[String, String],
    meta_tags: Hash[String, String]
  },
  headers: [
    {
      level: Integer,          # 1-6
      text: String,
      id: String?,
      depth: Integer,
      html_offset: Integer
    }
  ],
  links: [
    {
      href: String,
      text: String,
      title: String?,
      link_type: "anchor" | "internal" | "external" | "email" | "phone" | "other",
      rel: Array[String],
      attributes: Hash[String, String]
    }
  ],
  images: [
    {
      src: String,
      alt: String?,
      title: String?,
      dimensions: [Integer, Integer]?,
      image_type: "data_uri" | "inline_svg" | "external" | "relative",
      attributes: Hash[String, String]
    }
  ],
  structured_data: [
    {
      data_type: "json_ld" | "microdata" | "rdfa",
      raw_json: String,
      schema_type: String?
    }
  ]
}
```

## Metadata Configuration

Pass a hash with the following options to control which metadata types are extracted:

```ruby
config = {
  extract_headers: true,           # Extract h1-h6 elements (default: true)
  extract_links: true,             # Extract <a> elements (default: true)
  extract_images: true,            # Extract <img> elements (default: true)
  extract_structured_data: true,   # Extract JSON-LD/Microdata/RDFa (default: true)
  max_structured_data_size: 1_000_000  # Max bytes for structured data (default: 1MB)
}

markdown, metadata = HtmlToMarkdown.convert_with_metadata(html, nil, config)
```

## Type Signatures

All types are defined in RBS format in `sig/html_to_markdown.rbs`:

- `document_metadata` - Document-level metadata structure
- `header_metadata` - Individual header element
- `link_metadata` - Individual link element
- `image_metadata` - Individual image element
- `structured_data` - Structured data block
- `extended_metadata` - Complete metadata extraction result

Uses strict RBS type checking with Steep for full type safety.

## Implementation Details

### Architecture

The Rust implementation uses a single-pass collector pattern for efficient metadata extraction:

1. **No duplication**: Core logic lives in Rust (`crates/html-to-markdown/src/metadata.rs`)
2. **Minimal wrapper layer**: Ruby binding in `crates/html-to-markdown-rb/src/lib.rs`
3. **Type translation**: Rust types → Ruby hashes with proper Magnus bindings
4. **Hash conversion**: Uses Magnus `RHash` API for efficient Ruby hash construction

### Hash Conversion Pattern

Following the inline_images pattern:

```rust
fn document_metadata_to_ruby(ruby: &Ruby, doc: RustDocumentMetadata) -> Result<Value, Error> {
    let hash = ruby.hash_new();
    hash.aset(ruby.intern("title"), opt_string_to_ruby(ruby, doc.title)?)?;
    hash.aset(ruby.intern("keywords"), keywords_array)?;
    // ... more fields
    Ok(hash.as_value())
}
```

### Feature Flag

The metadata feature is gated by a Cargo feature in `Cargo.toml`:

```toml
[features]
metadata = ["html-to-markdown-rs/metadata"]
```

This ensures:
- Zero overhead when metadata is not needed
- Clean integration with feature flag detection
- Consistent with Python binding implementation

## Tests

Comprehensive RSpec test suite in `spec/metadata_extraction_spec.rb`:

```bash
cd packages/ruby
bundle exec rake compile
bundle exec rspec spec/metadata_extraction_spec.rb
```

Tests cover:
- All metadata types extraction
- Configuration flags
- Edge cases (empty HTML, malformed input, special characters)
- Return value structure validation
- Integration with conversion options

## Language Parity

Implements the same API as the Python binding:

- Same method signature: `convert_with_metadata(html, options, metadata_config)`
- Same return type: `[markdown, metadata_dict]`
- Same metadata structures and field names
- Same enum values (link_type, image_type, data_type, text_direction)

Enables seamless migration and multi-language development.

## Performance

Single-pass collection during tree traversal:
- No additional parsing passes
- Minimal memory overhead
- Configurable extraction granularity
- Built-in size limits for safety

## Building and Testing

Build the extension with metadata support:

```bash
cd packages/ruby
bundle exec rake compile -- --release --features metadata
```

Run type checking:

```bash
steep check
```

Run tests:

```bash
bundle exec rspec spec/metadata_extraction_spec.rb
```
