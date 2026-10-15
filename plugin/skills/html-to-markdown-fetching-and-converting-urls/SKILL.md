---
description: Use when fetching a live URL and converting it to Markdown. Covers --url, custom user agents, preprocessing for noisy pages, and the --json ConversionResult shape.
name: html-to-markdown-fetching-and-converting-urls
---

<!--
AI-RULEZ :: GENERATED FILE — DO NOT EDIT
Content-Hash: blake3:f46c6d2411f814c68ba4f0cb91b2efe2104f01ae7129dd2b84e0fa6ef94169e2
Source-Hash: blake3:993543f9225ea202d6e97003900394a417d430ff4e7e1a9412b8f7a844713c9f
Schema-Version: v1
-->

# Fetching and converting URLs

Use this when the user gives a URL instead of HTML and wants the page as
Markdown (or its metadata/tables). The CLI fetches the page over HTTP and
converts it in one step via `--url`. `--url` conflicts with a positional `FILE`.

For crawling *many* pages or following links, use `crawlberg` instead — this
skill is for a single URL.

## Fetch and convert

```bash
# Fetch a URL, print Markdown to stdout
html-to-markdown --url https://example.com

# Save to a file
html-to-markdown --url https://example.com -o page.md

# Custom User-Agent (default mimics a real browser)
html-to-markdown --url https://example.com --user-agent "MyBot/1.0"
```

`--user-agent` requires `--url`.

## Clean noisy pages

Real web pages carry navigation, ads, cookie banners, and forms. Preprocess
before converting:

```bash
html-to-markdown --url https://example.com/article --preprocess --preset aggressive

# Keep the nav or forms if the page content lives there
html-to-markdown --url https://example.com --preprocess --keep-navigation
```

Presets: `minimal`, `standard` (default), `aggressive`. `--preset` and the
`--keep-*` flags require `--preprocess`.

## JSON output (ConversionResult)

Add `--json` to get the full structured result instead of plain Markdown:

```bash
html-to-markdown --url https://example.com --json
```

```json
{
  "content": "# Title\n\nContent\n",
  "metadata": {
    "document": { "title": "...", "language": "en" },
    "headers": [],
    "links": [],
    "images": [],
    "structured_data": []
  },
  "tables": [],
  "images": [],
  "warnings": []
}
```

Useful combinations (all require `--json`):

```bash
# Page title + outline, no Markdown body
html-to-markdown --url https://example.com --json --no-content \
  | jq '{title: .metadata.document.title, headings: [.metadata.headers[].text]}'

# Include the document-structure tree
html-to-markdown --url https://example.com --json --include-structure | jq '.document'

# Extract inline image data
html-to-markdown --url https://example.com --json --extract-inline-images | jq '.images | length'
```

## Surface warnings

```bash
html-to-markdown --url https://example.com --show-warnings > page.md
# non-fatal warnings (truncation, malformed markup) go to stderr
```

## Exit codes

| Code | Meaning |
| ---- | ------- |
| 0 | Success |
| 1 | Conversion or I/O error (including a failed fetch) |
| 2 | Invalid arguments |

See `references/cli-reference.md` for the full flag set and
JSON shape.
