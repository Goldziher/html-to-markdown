# Configuration Reference

All `ConversionOptions` fields with their types, defaults, and descriptions.

In Rust, use `ConversionOptions::builder()` or direct struct construction.
In Python, use the `ConversionOptions` dataclass.
In TypeScript/Node.js, use the `ConversionOptions` interface (camelCase fields, PascalCase enum string values).

## ConversionOptions Fields

| Rust Field                   | Python                       | TypeScript                               | Type       | Default       | Description                                                                                                                  |
| ---------------------------- | ---------------------------- | ---------------------------------------- | ---------- | ------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `heading_style`              | `heading_style`              | `headingStyle`                           | enum       | `atx`         | Heading format: `atx` (`# h1`), `underlined` (`===`), `atxClosed` (`# h1 #`)                                                 |
| `list_indent_type`           | `list_indent_type`           | `listIndentType`                         | enum       | `spaces`      | List indentation: `spaces` or `tabs`                                                                                         |
| `list_indent_width`          | `list_indent_width`          | `listIndentWidth`                        | int        | `2`           | Spaces per list indent level (ignored when `list_indent_type = tabs`)                                                        |
| `bullets`                    | `bullets`                    | `bullets`                                | string     | `"-*+"`       | Bullet characters cycling through nesting levels. Characters cycle across nesting levels. Use `"*+-"` for a different order. |
| `strong_em_symbol`           | `strong_em_symbol`           | `strongEmSymbol`                         | char       | `'*'`         | Symbol for bold/italic emphasis: `'*'` or `'_'`                                                                              |
| `escape_asterisks`           | `escape_asterisks`           | `escapeAsterisks`                        | bool       | `false`       | Escape `*` in plain text to prevent unintended formatting                                                                    |
| `escape_underscores`         | `escape_underscores`         | `escapeUnderscores`                      | bool       | `false`       | Escape `_` in plain text                                                                                                     |
| `escape_misc`                | `escape_misc`                | `escapeMisc`                             | bool       | `false`       | Escape `[]()#` and other Markdown metacharacters                                                                             |
| `escape_ascii`               | `escape_ascii`               | `escapeAscii`                            | bool       | `false`       | Escape all ASCII punctuation (strict CommonMark compliance)                                                                  |
| `code_language`              | `code_language`              | `codeLanguage`                           | string     | `""`          | Default language hint for fenced code blocks with no language annotation                                                     |
| `autolinks`                  | `autolinks`                  | `autolinks`                              | bool       | `true`        | Convert bare URLs to `<url>` autolinks when link text equals href                                                            |
| `link_style`                 | `link_style`                 | `linkStyle`                              | enum       | `inline`      | Link rendering: `inline` (`[text](url)`) or `reference` (`[text][1]` with definitions at end)                               |
| `default_title`              | `default_title`              | `defaultTitle`                           | bool       | `false`       | Use href as link title when no `title` attribute exists                                                                      |
| `br_in_tables`               | `br_in_tables`               | `brInTables`                             | bool       | `false`       | Render `<br>` inside table cells as literal `<br>` tags instead of spaces                                                    |
| `compact_tables`             | `compact_tables`             | `compactTables`                          | bool       | `false`       | Emit compact (unpadded) GFM tables: single space around cell content, no alignment padding                                  |
| `highlight_style`            | `highlight_style`            | `highlightStyle`                         | enum       | `doubleEqual` | `<mark>` rendering: `doubleEqual` (`==text==`), `html`, `bold`, `none`                                                       |
| `extract_metadata`           | `extract_metadata`           | `extractMetadata`                        | bool       | `true`        | Extract `<head>` metadata into result. Requires `metadata` feature in Rust.                                                  |
| `whitespace_mode`            | `whitespace_mode`            | `whitespaceMode`                         | enum       | `normalized`  | Whitespace handling: `normalized` (collapse to single space) or `strict` (preserve as-is)                                    |
| `strip_newlines`             | `strip_newlines`             | `stripNewlines`                          | bool       | `false`       | Remove all newlines from HTML input before processing (useful for minified HTML)                                             |
| `wrap`                       | `wrap`                       | `wrap`                                   | bool       | `false`       | Enable line wrapping at `wrap_width` columns                                                                                 |
| `wrap_width`                 | `wrap_width`                 | `wrapWidth`                              | int        | `80`          | Column width for wrapping when `wrap = true`. Range: 20–500.                                                                 |
| `convert_as_inline`          | `convert_as_inline`          | `convertAsInline`                        | bool       | `false`       | Treat all block elements as inline content (no paragraph breaks)                                                             |
| `sub_symbol`                 | `sub_symbol`                 | `subSymbol`                              | string     | `""`          | Symbol wrapping `<sub>` text. E.g. `"~"` → `~text~`. Empty = no wrapping.                                                    |
| `sup_symbol`                 | `sup_symbol`                 | `supSymbol`                              | string     | `""`          | Symbol wrapping `<sup>` text. E.g. `"^"` → `^text^`. Empty = no wrapping.                                                    |
| `newline_style`              | `newline_style`              | `newlineStyle`                           | enum       | `spaces`      | `<br>` representation: `spaces` (two trailing spaces + newline) or `backslash` (`\` + newline)                               |
| `code_block_style`           | `code_block_style`           | `codeBlockStyle`                         | enum       | `backticks`   | Code block style: `backticks` (```), `indented`(4 spaces),`tildes` (~~~).                                                    |
| `keep_inline_images_in`      | `keep_inline_images_in`      | `keepInlineImagesIn`                     | list/array | `[]`          | HTML tag names where `<img>` children remain as Markdown (not converted to alt text)                                         |
| `preprocessing`              | (separate param)             | `preprocessing`                          | object     | see below     | HTML preprocessing config. In Python, pass as separate `PreprocessingOptions` argument.                                      |
| `encoding`                   | `encoding`                   | `encoding`                               | string     | `"utf-8"`     | Expected character encoding for input HTML                                                                                   |
| `debug`                      | `debug`                      | `debug`                                  | bool       | `false`       | Emit diagnostic warnings about unhandled elements                                                                            |
| `strip_tags`                 | `strip_tags`                 | `stripTags`                              | list/array | `[]`          | HTML tag names whose content is stripped entirely from output (text not preserved)                                           |
| `preserve_tags`              | `preserve_tags`              | `preserveTags`                           | list/array | `[]`          | HTML tag names preserved verbatim as raw HTML in output                                                                      |
| `skip_images`                | `skip_images`                | `skipImages`                             | bool       | `false`       | Omit all `<img>` elements from output entirely                                                                               |
| `max_depth`                  | `max_depth`                  | `maxDepth`                               | int/null   | `null`        | Silently truncate subtrees beyond this DOM nesting depth. `null` = unlimited depth.                                          |
| `output_format`              | `output_format`              | `outputFormat`                           | enum       | `markdown`    | Output format: `markdown` (CommonMark), `djot`, `plain` (text only)                                                          |
| `include_document_structure` | `include_document_structure` | `includeDocumentStructure`               | bool       | `false`       | Include structured document tree in `ConversionResult.document`. Available in Rust, Python, and Node.                                           |
| `extract_images`             | `extract_images`             | `extractImages`                          | bool       | `false`       | Extract inline data URI images and SVGs. Requires `inline-images` Rust feature.                                              |
| `max_image_size`             | `max_image_size`             | `maxImageSize`                           | u64/int    | `5242880`     | Maximum decoded image size in bytes (default 5 MiB). Requires `inline-images`.                                               |
| `capture_svg`                | `capture_svg`                | `captureSvg`                             | bool       | `false`       | Capture `<svg>` elements as images. Requires `inline-images`.                                                                |
| `infer_dimensions`           | `infer_dimensions`           | `inferDimensions`                        | bool       | `true`        | Infer image width/height from data URI. Requires `inline-images`.                                                            |

## PreprocessingOptions Fields

Preprocessing runs before conversion to clean noisy HTML (ads, navigation, forms).

| Rust Field          | Python              | TypeScript         | Type | Default    | Description                                         |
| ------------------- | ------------------- | ------------------ | ---- | ---------- | --------------------------------------------------- |
| `enabled`           | `enabled`           | `enabled`          | bool | `true`     | Enable HTML preprocessing globally                  |
| `preset`            | `preset`            | `preset`           | enum | `standard` | Aggressiveness: `minimal`, `standard`, `aggressive` |
| `remove_navigation` | `remove_navigation` | `removeNavigation` | bool | `true`     | Remove `<nav>`, breadcrumbs, menus, sidebars        |
| `remove_forms`      | `remove_forms`      | `removeForms`      | bool | `true`     | Remove `<form>`, `<input>`, `<button>`, etc.        |

## Enum Values

### HeadingStyle

TypeScript values are PascalCase strings (NAPI-RS `const enum`).

| Value      | Rust                       | Python         | TypeScript     | Output               |
| ---------- | -------------------------- | -------------- | -------------- | -------------------- |
| ATX        | `HeadingStyle::Atx`        | `"atx"`        | `'Atx'`        | `# H1`, `## H2`, ... |
| Underlined | `HeadingStyle::Underlined` | `"underlined"` | `'Underlined'` | `H1\n===`, `H2\n---` |
| ATX Closed | `HeadingStyle::AtxClosed`  | `"atx_closed"` | `'AtxClosed'`  | `# H1 #`, `## H2 ##` |

### CodeBlockStyle

| Value     | Rust                        | Python        | TypeScript    | Output         |
| --------- | --------------------------- | ------------- | ------------- | -------------- |
| Indented  | `CodeBlockStyle::Indented`  | `"indented"`  | `'Indented'`  | 4-space indent |
| Backticks | `CodeBlockStyle::Backticks` | `"backticks"` | `'Backticks'` | ` ``` ` fence  |
| Tildes    | `CodeBlockStyle::Tildes`    | `"tildes"`    | `'Tildes'`    | `~~~` fence    |

### OutputFormat

| Value    | Rust                     | Python       | TypeScript   | Description                   |
| -------- | ------------------------ | ------------ | ------------ | ----------------------------- |
| Markdown | `OutputFormat::Markdown` | `"markdown"` | `'Markdown'` | CommonMark Markdown (default) |
| Djot     | `OutputFormat::Djot`     | `"djot"`     | `'Djot'`     | Djot lightweight markup       |
| Plain    | `OutputFormat::Plain`    | `"plain"`    | `'Plain'`    | Visible text only, no markup  |

### LinkStyle

| Value     | Rust                   | Python        | TypeScript    | Output                          |
| --------- | ---------------------- | ------------- | ------------- | ------------------------------- |
| Inline    | `LinkStyle::Inline`    | `"inline"`    | `'Inline'`    | `[text](url)` (default)         |
| Reference | `LinkStyle::Reference` | `"reference"` | `'Reference'` | `[text][1]` + definitions block |

### WhitespaceMode

| Value      | Rust                         | Python         | TypeScript     | Description                                         |
| ---------- | ---------------------------- | -------------- | -------------- | --------------------------------------------------- |
| Normalized | `WhitespaceMode::Normalized` | `"normalized"` | `'Normalized'` | Collapse to single space (default, matches browser) |
| Strict     | `WhitespaceMode::Strict`     | `"strict"`     | `'Strict'`     | Preserve all whitespace as-is                       |

### NewlineStyle

| Value     | Rust                      | Python        | TypeScript    | `<br>` becomes             |
| --------- | ------------------------- | ------------- | ------------- | -------------------------- |
| Spaces    | `NewlineStyle::Spaces`    | `"spaces"`    | `'Spaces'`    | `\n` (two trailing spaces) |
| Backslash | `NewlineStyle::Backslash` | `"backslash"` | `'Backslash'` | `\\\n`                     |

### HighlightStyle

| Value       | Rust                          | Python           | TypeScript      | `<mark>text</mark>` becomes |
| ----------- | ----------------------------- | ---------------- | --------------- | --------------------------- |
| DoubleEqual | `HighlightStyle::DoubleEqual` | `"double_equal"` | `'DoubleEqual'` | `==text==` (default)        |
| Html        | `HighlightStyle::Html`        | `"html"`         | `'Html'`        | `<mark>text</mark>`         |
| Bold        | `HighlightStyle::Bold`        | `"bold"`         | `'Bold'`        | `**text**`                  |
| None        | `HighlightStyle::None`        | `"none"`         | `'None'`        | `text` (plain)              |

### PreprocessingPreset

| Value      | Rust                              | Python         | TypeScript     | Description                               |
| ---------- | --------------------------------- | -------------- | -------------- | ----------------------------------------- |
| Minimal    | `PreprocessingPreset::Minimal`    | `"minimal"`    | `'Minimal'`    | Remove only scripts and styles            |
| Standard   | `PreprocessingPreset::Standard`   | `"standard"`   | `'Standard'`   | Remove navigation, forms, noise (default) |
| Aggressive | `PreprocessingPreset::Aggressive` | `"aggressive"` | `'Aggressive'` | Maximum cleanup for web scraping          |

## MetadataConfig Fields

Controls what metadata is extracted. Metadata is returned in `ConversionResult.metadata` from the single `convert()` call (requires the `metadata` feature, which is enabled by default).

| Field                      | Type  | Default | Description                                 |
| -------------------------- | ----- | ------- | ------------------------------------------- |
| `extract_document`         | bool  | `true`  | Title, description, language, OG tags, etc. |
| `extract_headers`          | bool  | `true`  | h1-h6 headers with level, text, id          |
| `extract_links`            | bool  | `true`  | Anchor tags with href, text, link_type      |
| `extract_images`           | bool  | `true`  | img tags with src, alt, title, image_type   |
| `extract_structured_data`  | bool  | `true`  | JSON-LD, Microdata, RDFa blocks             |
| `max_structured_data_size` | usize | `10000` | Max bytes per structured data block         |

## Notes

- In **Python**, `PreprocessingOptions` is a separate `@dataclass` passed as the second argument to `convert()`, not nested inside `ConversionOptions`.
- In **TypeScript/Node.js**, preprocessing is nested inside `ConversionOptions.preprocessing`.
- In **Rust**, preprocessing is a field inside `ConversionOptions` struct (`options.preprocessing`).
- The `metadata` feature is **enabled by default** in the Rust crate (`features = ["metadata"]`). In Python and Node.js bindings, metadata is always available.
- TypeScript/Node.js enum values are **PascalCase strings** (e.g. `'Atx'` not `'atx'`). This is a NAPI-RS `const enum` requirement — lowercase values will be rejected at runtime.
- The Node.js `ConversionResult` has no top-level `images` array. Image references appear as an inventory in `result.metadata.images` (`ImageMetadata`: `src`, `alt`, `title`, `dimensions`, `imageType`, `attributes`). The `extractImages` / `maxImageSize` / `captureSvg` / `inferDimensions` options exist on `ConversionOptions`, but extracted inline-image binary data is exposed only in the Rust core (the `inline-images` feature), not in the Node result.
