<!--
AI-RULEZ :: GENERATED FILE — DO NOT EDIT
Content-Hash: blake3:60f12b55e155a214be5fcc59b943fc53731941f308b3ce7c9832adaab92901ba
Source-Hash: blake3:d85ad7bf79fa4cb1ed116e7fd2574bf5e6f64f503331a1bacf3cc3d495e3c873
Schema-Version: v1
-->

# TypeScript / Node.js API Reference

Package: `@xberg-io/html-to-markdown`

This is the NAPI-RS native binding. Platform-specific native binaries ship as
optional dependencies (`@xberg-io/html-to-markdown-<platform>`, e.g.
`@xberg-io/html-to-markdown-darwin-arm64`); the right one is selected
automatically at install time. The package includes generated `index.d.ts` type
definitions.

## Installation

```bash
pnpm add @xberg-io/html-to-markdown
# or
pnpm add @xberg-io/html-to-markdown
```

## Primary Function

```typescript
import { convert } from "@xberg-io/html-to-markdown";

// convert() returns a ConversionResult OBJECT (not a JSON string).
const result = convert(html, options?);
console.log(result.content);    // Markdown string or undefined
console.log(result.tables);     // array of table objects
console.log(result.warnings);   // array of warning objects
console.log(result.metadata);   // metadata object (when metadata is present)
```

`convert()` returns a parsed `ConversionResult` object directly — do **not**
`JSON.parse()` it. NAPI-RS maps the Rust struct straight to a JS object.

## Function Signature

```typescript
// The only exported function. Returns a ConversionResult object; throws on
// parse failure or invalid UTF-8.
export declare function convert(
  html: string,
  options?: ConversionOptions | undefined | null,
): ConversionResult;
```

There are no `convertFile` / `convertStream` helpers in this package — read the
file yourself and pass the string to `convert()`. The only other export is the
`VisitorHandle` class (see Visitor Pattern below), plus the option/result types
and string enums.

## Interfaces

### ConversionOptions

All fields are optional and camelCase. Defaults match the Rust core defaults.
Enum-typed fields take **PascalCase** string values (NAPI-RS string enums) — see
the note below.

```typescript
interface ConversionOptions {
  // Enum-typed fields are nominal: pass the enum member (HeadingStyle.Atx),
  // not the bare string "Atx", or tsc rejects it.
  headingStyle?: HeadingStyle;
  listIndentType?: "Spaces" | "Tabs";
  listIndentWidth?: number;
  bullets?: string;
  strongEmSymbol?: string; // '*' or '_'
  escapeAsterisks?: boolean;
  escapeUnderscores?: boolean;
  escapeMisc?: boolean;
  escapeAscii?: boolean;
  codeLanguage?: string;
  autolinks?: boolean;
  linkStyle?: "Inline" | "Reference";
  defaultTitle?: boolean;
  brInTables?: boolean;
  compactTables?: boolean;
  highlightStyle?: "DoubleEqual" | "Html" | "Bold" | "None";
  extractMetadata?: boolean;
  whitespaceMode?: "Normalized" | "Strict";
  stripNewlines?: boolean;
  wrap?: boolean;
  wrapWidth?: number;
  convertAsInline?: boolean;
  subSymbol?: string;
  supSymbol?: string;
  newlineStyle?: "Spaces" | "Backslash";
  codeBlockStyle?: "Indented" | "Backticks" | "Tildes";
  keepInlineImagesIn?: string[];
  stripTags?: string[];
  preserveTags?: string[];
  skipImages?: boolean;
  maxDepth?: number | null;
  urlEscapeStyle?: "Angle" | "Percent";
  outputFormat?: "Markdown" | "Djot" | "Plain";
  includeDocumentStructure?: boolean; // populates result.document
  extractImages?: boolean;
  maxImageSize?: number;
  captureSvg?: boolean;
  inferDimensions?: boolean;
  encoding?: string;
  debug?: boolean;
  preprocessing?: PreprocessingOptions;
  visitor?: VisitorHandle; // custom traversal (see below)
}
```

**Note on enum values:** the NAPI-RS string-enum fields use PascalCase values
matching the Rust variant names (e.g. `'Atx'` not `'atx'`, `'Spaces'` not
`'spaces'`, `'DoubleEqual'` not `'double-equal'`). Lowercase values are rejected.

### PreprocessingOptions

```typescript
interface PreprocessingOptions {
  enabled?: boolean;
  preset?: "Minimal" | "Standard" | "Aggressive";
  removeNavigation?: boolean;
  removeForms?: boolean;
}
```

## ConversionResult

```typescript
interface ConversionResult {
  content?: string;                 // converted Markdown/Djot/plain text
  document?: DocumentStructure;     // present only when includeDocumentStructure = true
  metadata?: HtmlMetadata;          // title, OG, headers, links, images, structured data
  tables?: TableData[];
  warnings?: ProcessingWarning[];
}

interface TableData {
  grid: TableGrid;     // structured cells
  markdown: string;    // rendered table in the target format
}

interface TableGrid {
  rows?: number;
  cols?: number;
  cells?: GridCell[];  // flat, sparse list ordered by (row, col); span origins only
}

interface GridCell {
  content: string;
  row: number;
  col: number;
  rowSpan: number;
  colSpan: number;
  isHeader: boolean;
}

interface ProcessingWarning {
  message: string;
  kind: string; // WarningKind
}
```

**Note on tables:** the Node binding exposes `grid: TableGrid` (matching the Rust
`TableData`), not a flat `cells: string[][]` array. `TableGrid.cells` is a sparse
list of `GridCell`s — only the top-left origin of a spanning cell is present.

**Note on inline images:** the Node `ConversionResult` has no top-level `images`
field. Image references are reported as an inventory in
`result.metadata?.images` (`ImageMetadata` with `src`, `alt`, `title`,
`dimensions`, `imageType`, `attributes`).

## Visitor Pattern

The visitor is supplied through `options.visitor` (a `VisitorHandle`), not as a
third argument to `convert()`. Build a `VisitorHandle` from an `HtmlVisitor`
callback object, whose methods return a `VisitResult`
(`Continue` | `Skip` | `PreserveHtml` | custom). This is an advanced surface;
consult the generated `index.d.ts` (`HtmlVisitor`, `VisitorHandle`,
`VisitResult`, `NodeContext`) for the exact shapes.

## Examples

```typescript
import { convert, HeadingStyle } from "@xberg-io/html-to-markdown";

// Simple conversion — result is an object
const result = convert("<h1>Hello</h1>");
console.log(result.content); // "# Hello\n"

// Metadata — enabled via extractMetadata (on by default); read result.metadata
const result2 = convert(html, { extractMetadata: true });
console.log(result2.metadata?.document?.title);
console.log(result2.metadata?.headers?.length);

// Tables — in result.tables when includeDocumentStructure is enabled
const result3 = convert(html, { includeDocumentStructure: true });
for (const table of result3.tables ?? []) {
  console.log(table.markdown);
}

// Document structure — enable includeDocumentStructure
const result4 = convert(html, { includeDocumentStructure: true });
console.log(result4.document);

// Reading a file: read it yourself, then convert
import { readFile } from "node:fs/promises";
const html5 = await readFile("./page.html", "utf8");
const result5 = convert(html5, { headingStyle: HeadingStyle.Atx });
console.log(result5.content);
```
