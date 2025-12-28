# Metadata Extraction Guide

The metadata extraction feature enables comprehensive document analysis during HTML-to-Markdown conversion. Extract document properties, headers, links, images, and structured data in a single pass with zero overhead when disabled.

## What is Metadata Extraction?

Metadata extraction collects information about HTML document structure and content without requiring a second parsing pass. During conversion, the engine walks the HTML tree and simultaneously:

1. **Captures document metadata** (title, description, author, canonical URL, social tags)
2. **Tracks heading hierarchy** (h1-h6 with depth and IDs)
3. **Analyzes links** (classification by type: anchor, internal, external, email, phone)
4. **Catalogs images** (dimensions, alt text, source type detection)
5. **Extracts structured data** (JSON-LD, Microdata, RDFa blocks)

This is useful for SEO analysis, content migration, accessibility audits, table-of-contents generation, and link validation.

## Quick Start by Language

### Python

```python
from html_to_markdown import convert_with_metadata

html = '<html lang="en"><head><title>My Article</title></head><body><h1>Hello</h1></body></html>'
markdown, metadata = convert_with_metadata(html)

print(metadata["document"]["title"])       # "My Article"
print(metadata["document"]["language"])    # "en"
print(metadata["headers"][0]["text"])      # "Hello"
```

### TypeScript

```typescript
import { convertWithMetadata } from 'html-to-markdown';

const html = '<html lang="en"><head><title>My Article</title></head><body><h1>Hello</h1></body></html>';
const { markdown, metadata } = convertWithMetadata(html);

console.log(metadata.document.title);       // "My Article"
console.log(metadata.document.language);    // "en"
console.log(metadata.headers[0].text);      // "Hello"
```

### Ruby

```ruby
require 'html_to_markdown'

html = '<html lang="en"><head><title>My Article</title></head><body><h1>Hello</h1></body></html>'
markdown, metadata = HtmlToMarkdown.convert_with_metadata(html)

puts metadata[:document][:title]            # "My Article"
puts metadata[:document][:language]         # "en"
puts metadata[:headers][0][:text]           # "Hello"
```

## Configuration API

Control which metadata types are extracted using `MetadataConfig`:

### Python

```python
from html_to_markdown import MetadataConfig, convert_with_metadata

config = MetadataConfig(
    extract_headers=True,              # h1-h6 elements (default: True)
    extract_links=True,                # <a> hyperlinks (default: True)
    extract_images=True,               # <img> elements (default: True)
    extract_structured_data=True,      # JSON-LD, Microdata, RDFa (default: True)
    max_structured_data_size=1_000_000, # Max bytes for structured data
)

markdown, metadata = convert_with_metadata(html, metadata_config=config)
```

### TypeScript

```typescript
import { convertWithMetadata } from 'html-to-markdown';

const config = {
  extractHeaders: true,
  extractLinks: true,
  extractImages: true,
  extractStructuredData: true,
  maxStructuredDataSize: 1_000_000,
};

const { markdown, metadata } = convertWithMetadata(html, undefined, config);
```

### Ruby

```ruby
config = {
  extract_headers: true,
  extract_links: true,
  extract_images: true,
  extract_structured_data: true,
  max_structured_data_size: 1_000_000
}

markdown, metadata = HtmlToMarkdown.convert_with_metadata(html, nil, config)
```

## Metadata Structure Reference

The returned metadata dictionary contains five categories:

### Document Metadata

```python
metadata["document"] = {
    "title": str | None,                    # <title> tag
    "description": str | None,              # meta[name="description"]
    "keywords": list[str],                  # Comma-separated keywords
    "author": str | None,                   # meta[name="author"]
    "canonical_url": str | None,            # link[rel="canonical"]
    "base_href": str | None,                # <base href="">
    "language": str | None,                 # html[lang] attribute
    "text_direction": str | None,           # "ltr", "rtl", "auto"
    "open_graph": dict[str, str],           # og:* meta properties
    "twitter_card": dict[str, str],         # twitter:* meta properties
    "meta_tags": dict[str, str],            # Other meta tags
}
```

### Headers

```python
metadata["headers"] = [
    {
        "level": int,           # 1-6
        "text": str,            # Normalized text content
        "id": str | None,       # HTML id attribute
        "depth": int,           # Nesting depth in document
        "html_offset": int,     # Byte offset in original HTML
    },
    # ... more headers
]
```

### Links

```python
metadata["links"] = [
    {
        "href": str,                        # The URL
        "text": str,                        # Link text
        "title": str | None,                # title attribute
        "link_type": str,                   # "anchor" | "internal" | "external" | "email" | "phone" | "other"
        "rel": list[str],                   # rel attribute values
        "attributes": dict[str, str],       # Other HTML attributes
    },
    # ... more links
]
```

### Images

```python
metadata["images"] = [
    {
        "src": str,                         # Image source (URL or data URI)
        "alt": str | None,                  # Alternative text
        "title": str | None,                # title attribute
        "dimensions": tuple[int, int] | None, # (width, height)
        "image_type": str,                  # "data_uri" | "inline_svg" | "external" | "relative"
        "attributes": dict[str, str],       # Other HTML attributes
    },
    # ... more images
]
```

### Structured Data

```python
metadata["structured_data"] = [
    {
        "data_type": str,                   # "json_ld" | "microdata" | "rdfa"
        "raw_json": str,                    # JSON string representation
        "schema_type": str | None,          # Detected schema type (e.g., "Article")
    },
    # ... more blocks
]
```

## Use Cases

### 1. SEO Metadata Extraction

Extract article SEO properties including title, description, Open Graph tags, and Twitter cards for social media optimization:

```python
from html_to_markdown import convert_with_metadata

html = """
<html>
  <head>
    <title>10 Rust Performance Tips</title>
    <meta name="description" content="Learn how to optimize Rust code">
    <meta property="og:image" content="https://example.com/og.jpg">
    <meta name="twitter:card" content="summary_large_image">
  </head>
  <body>
    <h1>Welcome to Rust</h1>
  </body>
</html>
"""

markdown, metadata = convert_with_metadata(html)
doc = metadata["document"]

seo_data = {
    "title": doc.get("title"),
    "description": doc.get("description"),
    "og_image": doc.get("open_graph", {}).get("image"),
    "twitter_card": doc.get("twitter_card", {}).get("card"),
}
```

### 2. Table of Contents Generation

Build nested table of contents from document headers:

```python
from html_to_markdown import convert_with_metadata

def build_toc(html: str) -> str:
    markdown, metadata = convert_with_metadata(html)
    lines = ["# Table of Contents\n"]

    for header in metadata["headers"]:
        indent = "  " * (header["level"] - 1)
        anchor = header.get("id") or header["text"].lower().replace(" ", "-")
        lines.append(f"{indent}- [{header['text']}](#{anchor})")

    return "\n".join(lines)
```

### 3. Link Validation and Classification

Find broken links, validate link types, and extract external references:

```python
from html_to_markdown import convert_with_metadata

def analyze_links(html: str) -> dict:
    markdown, metadata = convert_with_metadata(html)

    return {
        "external_links": [
            link for link in metadata["links"]
            if link["link_type"] == "external"
        ],
        "broken_anchors": [
            link for link in metadata["links"]
            if link["link_type"] == "anchor"
        ],
        "email_contacts": [
            link for link in metadata["links"]
            if link["link_type"] == "email"
        ],
    }
```

### 4. Accessibility Audit

Check for images without alt text, empty links, and heading hierarchy:

```python
from html_to_markdown import convert_with_metadata

def check_accessibility(html: str) -> dict:
    markdown, metadata = convert_with_metadata(html)

    return {
        "images_without_alt": [
            img for img in metadata["images"]
            if not img.get("alt") or not img["alt"].strip()
        ],
        "links_without_text": [
            link for link in metadata["links"]
            if not link.get("text") or not link["text"].strip()
        ],
        "images_without_dimensions": [
            img for img in metadata["images"]
            if not img.get("dimensions")
        ],
    }
```

### 5. Content Migration with Resource Manifest

Create a migration manifest documenting all external resources, links, and images for content preservation:

```python
from html_to_markdown import convert_with_metadata

def create_migration_manifest(html: str) -> dict:
    markdown, metadata = convert_with_metadata(html)

    return {
        "document": {
            "title": metadata["document"].get("title"),
            "language": metadata["document"].get("language"),
            "canonical": metadata["document"].get("canonical_url"),
        },
        "resources": {
            "external_links": len([
                l for l in metadata["links"]
                if l["link_type"] == "external"
            ]),
            "external_images": len([
                i for i in metadata["images"]
                if i["image_type"] == "external"
            ]),
            "json_ld_blocks": len([
                s for s in metadata["structured_data"]
                if s["data_type"] == "json_ld"
            ]),
        },
        "external_assets": {
            "images": [
                {"url": img["src"], "alt": img.get("alt")}
                for img in metadata["images"]
                if img["image_type"] == "external"
            ],
            "links": [
                {"url": link["href"], "text": link["text"]}
                for link in metadata["links"]
                if link["link_type"] == "external"
            ],
        },
    }
```

## Working Examples

This directory contains complete, runnable examples demonstrating real-world metadata extraction use cases:

### Python Examples

- **`seo-extraction.py`** - Extract SEO metadata (title, description, OG tags, Twitter cards)
- **`toc-generation.py`** - Build table of contents from headers with proper hierarchy
- **`link-validation.py`** - Find broken links, classify link types, validate structure
- **`accessibility-audit.py`** - Check images without alt text, empty links, heading hierarchy
- **`content-migration.py`** - Create migration manifest with all external resources

### TypeScript Examples

- **`seo-extraction.ts`** - Extract and format SEO metadata
- **`toc-generation.ts`** - Generate table of contents with anchor links

### Ruby Example

- **`seo-extraction.rb`** - Ruby version of SEO metadata extraction

## Performance Considerations

1. **Single-Pass Collection**: Metadata extraction happens during HTML parsing with zero overhead when disabled.
2. **Memory Efficient**: Collections use reasonable pre-allocations (32 headers, 64 links, 16 images typical).
3. **Selective Extraction**: Disable unused metadata types in `MetadataConfig` to reduce overhead.
4. **Structured Data Limits**: Large JSON-LD blocks are skipped if they exceed the size limit to prevent memory exhaustion.

```python
# Optimize for performance - only extract what you need
config = MetadataConfig(
    extract_headers=True,
    extract_links=False,  # Skip if not needed
    extract_images=False, # Skip if not needed
    extract_structured_data=False,  # Skip if not needed
)

markdown, metadata = convert_with_metadata(html, metadata_config=config)
```

## Running the Examples

### Python

```bash
python seo-extraction.py
python toc-generation.py
python link-validation.py
python accessibility-audit.py
python content-migration.py
```

### TypeScript

```bash
npm install
npm run seo
npm run toc
```

### Ruby

```bash
bundle install
ruby seo-extraction.rb
```

## Feature Detection

Check if metadata extraction is available at runtime:

### Python

```python
from html_to_markdown import convert_with_metadata, convert

try:
    markdown, metadata = convert_with_metadata(html)
except (NameError, TypeError):
    # Fallback for builds without metadata feature
    markdown = convert(html)
```

### TypeScript

```typescript
import { hasMetadataSupport, convertWithMetadata, convert } from 'html-to-markdown';

if (hasMetadataSupport()) {
  const { markdown, metadata } = convertWithMetadata(html);
} else {
  const markdown = convert(html);
}
```

### Ruby

```ruby
begin
  markdown, metadata = HtmlToMarkdown.convert_with_metadata(html)
rescue NoMethodError
  markdown = HtmlToMarkdown.convert(html)
end
```

## Error Handling

Metadata extraction is designed to be robust and won't fail on malformed input:

### Python

```python
from html_to_markdown import convert_with_metadata, MetadataConfig

config = MetadataConfig(
    extract_structured_data=True,
    max_structured_data_size=500_000,  # 500KB limit
)

try:
    markdown, metadata = convert_with_metadata(html, metadata_config=config)

    # Safe access with defaults
    title = metadata["document"].get("title", "Untitled")
    headers = metadata["headers"] or []
    images = metadata["images"] or []

except Exception as e:
    print(f"Extraction error: {e}")
```

## Links

- [GitHub Repository](https://github.com/kreuzberg-dev/html-to-markdown)
- [Python Package](https://pypi.org/project/html-to-markdown/)
- [TypeScript Package](https://www.npmjs.com/package/html-to-markdown-node)
- [Ruby Gem](https://rubygems.org/gems/html-to-markdown)
