---
description: Use when extracting metadata from HTML — title, description, language, Open Graph, JSON-LD / Microdata / RDFa, headers, links, and images. Covers the --json output shape and the --extract-metadata flag.
name: html-to-markdown-extracting-metadata
---

<!--
AI-RULEZ :: GENERATED FILE — DO NOT EDIT
Content-Hash: blake3:008ee34eae3d73ceed0d8a9c7fdb8ae8562558ccef3be60bac93f03dbf362e29
Source-Hash: blake3:61d35c074f5b61d82570d787b206cb51b12e061176e30f3fd2072e9c4b8b1b7a
Schema-Version: v1
-->

# Extracting metadata

Use this when the user wants structured metadata out of HTML rather than (or in
addition to) the Markdown body — page title, description, language, Open Graph
tags, structured data, the heading outline, links, or image references.

Metadata lives in `result.metadata` and is surfaced on the CLI through
`--json`. On the CLI, metadata extraction is **opt-in**: pass
`--extract-metadata` alongside `--json`, otherwise `result.metadata` comes back
empty (`document.title` is null, `headers`/`links`/`images`/`structured_data`
are `[]`). The library `convert()` call extracts metadata by default
(`extract_metadata=True`) — that default is a property of the API, not the CLI.
There are no per-field extraction flags: `--extract-metadata` populates all
sub-fields below at once.

## Get all metadata

```bash
html-to-markdown --json --extract-metadata input.html | jq '.metadata'

# Extraction-only (skip the Markdown body)
html-to-markdown --json --extract-metadata --no-content input.html | jq '.metadata'
```

## Metadata sub-fields

```json
{
  "metadata": {
    "document": { "title": "...", "description": "...", "language": "en", "open_graph": {"title": "..."} },
    "headers": [ { "level": 1, "text": "Main Heading" } ],
    "links":   [ { "href": "https://example.com", "link_type": "external" } ],
    "images":  [ { "src": "photo.jpg", "alt": "A photo", "image_type": "external" } ],
    "structured_data": [ /* JSON-LD, Microdata, RDFa blocks */ ]
  }
}
```

## Metadata flag

There is one metadata flag: `--extract-metadata`. With `--json` set it populates
all sub-fields above (`document`, `headers`, `links`, `images`,
`structured_data`) under `result.metadata` — select what you need with `jq`.

| Flag | Effect |
| ---- | ------ |
| `--extract-metadata` | With `--json`: populate `result.metadata`. In plain-text mode (no `--json`): prepend title + meta tags as a YAML frontmatter block (`---`-delimited) at the top of the Markdown output |

```bash
# Pull just the document-level metadata and the heading outline
html-to-markdown --json --extract-metadata --no-content input.html \
  | jq '{title: .metadata.document.title, lang: .metadata.document.language, outline: [.metadata.headers[].text]}'
```

## Common queries

```bash
# Title + canonical language
html-to-markdown --json --extract-metadata input.html | jq '{title: .metadata.document.title, lang: .metadata.document.language}'

# External links only
html-to-markdown --json --extract-metadata input.html | jq '[.metadata.links[] | select(.link_type == "external") | .href]'

# Open Graph card
html-to-markdown --json --extract-metadata input.html | jq '.metadata.document.open_graph'

# JSON-LD blocks
html-to-markdown --json --extract-metadata input.html | jq '.metadata.structured_data'
```

## Programmatic access

```python
from html_to_markdown import convert

html = """
<html lang="en">
<head><title>My Article</title></head>
<body>
<h1>Main Heading</h1>
<a href="https://example.com">External link</a>
<img src="photo.jpg" alt="A photo">
</body>
</html>
"""
result = convert(html)
meta = result.metadata
print(meta.document.title)        # "My Article"
print(meta.document.language)     # "en"
print(meta.headers[0].text)       # "Main Heading"
print(meta.links[0].link_type)    # "external"
print(meta.images[0].alt)         # "A photo"
```

Metadata is available from the single `convert()` call in Python, Rust, Go,
Ruby, and Elixir; in TypeScript read it off the returned result object. In Rust
it requires the `metadata` feature (on by default).

See `references/cli-reference.md` (Metadata section) and
`references/configuration.md` for field details.
