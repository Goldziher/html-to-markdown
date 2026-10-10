---
name: extracting-tables
description: Use when extracting tabular data from HTML. Covers GFM Markdown tables, the structured tables array (grid cells plus pre-rendered markdown), and HTML line breaks in table cells.
---

<!--
AI-RULEZ :: GENERATED FILE — DO NOT EDIT
Content-Hash: blake3:9ecb90f7b724e95fc689cc1501dfe079d2b0c6be9c11dd0f0a78ace9293866d1
Source-Hash: blake3:568a4ea7e94a211bb55b0d3df2984bf570265f331b3dbf396e5b3b597fdb358e
Schema-Version: v1
-->

# Extracting tables

Use this when the user wants tabular data out of HTML — pricing tables, data
grids, spec sheets. html-to-markdown parses `<table>` elements into two
surfaces at once: inline GFM Markdown tables in the `content` stream, and a
structured `tables` array in the JSON output.

## Two surfaces

```bash
# Inline GFM tables appear in the Markdown body
html-to-markdown input.html

# Structured table data appears under result.tables (JSON)
html-to-markdown --json input.html | jq '.tables'
```

- **Markdown tables in `content`** — `| col | col |` blocks, good for LLM
  ingestion and human reading.
- **Structured `tables` array** — each entry has a `markdown` field
  (pre-rendered) and a `grid` of structured cells (rows × cols). Use this when
  downstream code needs exact cell access.

Both are populated from the same parse; you do not need a flag to enable table
parsing.

## Extraction-only

When you only care about tables (not the Markdown body):

```bash
html-to-markdown --json --no-content input.html | jq '.tables'
```

## Inspecting tables

```bash
# How many tables, and the row count of each
html-to-markdown --json input.html \
  | jq '.tables | to_entries | map({index: .key, rows: .value.grid.rows, cols: .value.grid.cols})'

# Just the rendered markdown of the first table
html-to-markdown --json input.html | jq -r '.tables[0].markdown'
```

## Line breaks in cells

By default `<br>` inside a cell is converted to a space. Keep hard breaks:

```bash
html-to-markdown input.html --br-in-tables
```

## Programmatic access

```python
from html_to_markdown import convert, ConversionOptions

html = "<table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>"
# result.tables is only populated when include_document_structure=True
result = convert(html, ConversionOptions(include_document_structure=True))
for table in result.tables:
    print(table.markdown)            # rendered GFM markdown
    print(table.grid.cells[0].content)  # first cell (grid is a TableGrid)
```

```typescript
import { convert } from "@xberg-io/html-to-markdown";

// Node's convert() returns a ConversionResult object directly.
const result = convert(html);
for (const table of result.tables ?? []) {
  console.log(table.markdown);   // rendered table
  // table.grid is a TableGrid: { rows, cols, cells: GridCell[] }
}
```

## Notes and limits

- **Nested tables** are flattened — the inner table is rendered inline within
  the parent cell.
- **Merged / spanning cells** are reconstructed positionally; the span itself
  is not preserved as separate metadata.
- **Malformed tables** still parse on a best-effort basis; check `result.warnings`
  (or `--show-warnings`) for non-fatal issues.

See `../html-to-markdown/references/cli-reference.md` (Tables section) for the
full flag set.
