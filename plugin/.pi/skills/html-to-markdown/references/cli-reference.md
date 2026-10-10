<!--
AI-RULEZ :: GENERATED FILE — DO NOT EDIT
Content-Hash: blake3:dfaf03eca6acf6844233aed17bde4f7282aeea1213133a40331becfeb81f2bd2
Source-Hash: blake3:568a4ea7e94a211bb55b0d3df2984bf570265f331b3dbf396e5b3b597fdb358e
Schema-Version: v1
-->

# CLI Reference

Binary name: `html-to-markdown`

## Installation

```bash
brew install xberg-io/tap/html-to-markdown
# or run without a persistent install (self-installs the binary):
npx @xberg-io/html-to-markdown-cli --help
uvx --from html-to-markdown html-to-markdown --help
# or build from source:
cargo install html-to-markdown-cli
```

## Usage

```text
html-to-markdown [OPTIONS] [FILE]
```

`FILE` is the path to an input HTML file. Use `-` or omit to read from stdin.

## Input / Output

| Flag                | Short | Description                                                                            |
| ------------------- | ----- | -------------------------------------------------------------------------------------- |
| `[FILE]`            |       | Input HTML file. Omit or use `-` for stdin.                                            |
| `--url <URL>`       |       | Fetch HTML from a URL. Conflicts with FILE.                                            |
| `--user-agent <UA>` |       | User-Agent header when using `--url`. Default mimics a real browser. Requires `--url`. |
| `--output <FILE>`   | `-o`  | Output file. Default: stdout.                                                          |

## Heading Options

| Flag                      | Values                            | Default | Description                                                     |
| ------------------------- | --------------------------------- | ------- | --------------------------------------------------------------- |
| `--heading-style <STYLE>` | `atx`, `underlined`, `atx-closed` | `atx`   | `atx`: `# h1`. `underlined`: `h1\n===`. `atx-closed`: `# h1 #`. |

## List Options

| Flag                        | Short | Values           | Default  | Description                                                     |
| --------------------------- | ----- | ---------------- | -------- | --------------------------------------------------------------- |
| `--list-indent-type <TYPE>` |       | `spaces`, `tabs` | `spaces` | Indentation type for nested lists.                              |
| `--list-indent-width <N>`   |       | 1–8              | `2`      | Spaces per indent level. Ignored with `tabs`.                   |
| `--bullets <CHARS>`         | `-b`  | string           | `"-*+"`  | Bullet characters cycling through nesting levels. E.g. `"*+-"`. |

## Text Formatting

| Flag                        | Values                | Default  | Description                                                                         |
| --------------------------- | --------------------- | -------- | ----------------------------------------------------------------------------------- |
| `--strong-em-symbol <CHAR>` | `*`, `_`              | `*`      | Symbol for bold/italic emphasis.                                                    |
| `--escape-asterisks`        | flag                  | off      | Escape `*` in plain text.                                                           |
| `--escape-underscores`      | flag                  | off      | Escape `_` in plain text.                                                           |
| `--escape-misc`             | flag                  | off      | Escape `[]()#` and other Markdown metacharacters.                                   |
| `--escape-ascii`            | flag                  | off      | Escape all ASCII punctuation (strict CommonMark compliance).                        |
| `--sub-symbol <SYMBOL>`     | string                | `""`     | Symbol wrapping `<sub>` text. E.g. `"~"`.                                           |
| `--sup-symbol <SYMBOL>`     | string                | `""`     | Symbol wrapping `<sup>` text. E.g. `"^"`.                                           |
| `--newline-style <STYLE>`   | `backslash`, `spaces` | `spaces` | `<br>` representation: `backslash` (`\`+newline) or `spaces` (two trailing spaces). |

## Code Blocks

| Flag                         | Short | Values                            | Default     | Description                              |
| ---------------------------- | ----- | --------------------------------- | ----------- | ---------------------------------------- |
| `--code-block-style <STYLE>` |       | `backticks`, `indented`, `tildes` | `backticks` | Code block fence style.                  |
| `--code-language <LANG>`     | `-l`  | string                            | `""`        | Default language for fenced code blocks. |

## Links

| Flag                   | Values                  | Default  | Description                                                                                 |
| ---------------------- | ----------------------- | -------- | ------------------------------------------------------------------------------------------- |
| `--no-autolinks`       | flag                    | off      | Disable autolinks. Autolinks are on by default: when link text equals the href, output `<url>` syntax. Pass this flag to output `[url](url)` instead. |
| `--link-style <STYLE>` | `inline`, `reference`   | `inline` | Link rendering style. `inline`: `[text](url)`. `reference`: `[text][1]` with definitions at end. |
| `--default-title`      | flag                    | off      | Use href as link title when no `title` attribute exists.                                    |
| `--url-escape-style <STYLE>` | `angle`, `percent` | `angle` | Destination escaping. `angle`: wrap destinations containing spaces or newlines in `<...>`. `percent`: percent-encode every character that is not RFC 3986 unreserved or `/`. |

## Images

| Flag                                 | Values                    | Default | Description                                                          |
| ------------------------------------ | ------------------------- | ------- | -------------------------------------------------------------------- |
| `--keep-inline-images-in <ELEMENTS>` | comma-separated tag names | none    | Keep images as Markdown in these parent elements. E.g. `"a,strong"`. |
| `--max-image-size <BYTES>`           | integer                   | `5242880` | Skip inline images whose decoded payload exceeds this size. Requires `--extract-inline-images`. |
| `--capture-svg`                      | flag                      | off     | Capture inline `<svg>` elements as extracted images. Requires `--extract-inline-images`. |
| `--no-infer-dimensions`              | flag                      | off     | Skip inferring image width/height from the decoded payload when the HTML omits them. Inference is on by default. Requires `--extract-inline-images`. |
| `--inline-data-media <CHOICE>`      | `keep`, `alt-text-only`, `drop-element` | `keep` | What to write for an image or media element whose address is a `data:` URL. `keep`: the URL with its payload. `alt-text-only`: the alt text (the title of an inline `<svg>`, the fallback content of `<video>` and `<audio>`). `drop-element`: nothing. A real address on the element wins over the `data:` one. |

## Tables

| Flag               | Default | Description                                                                         |
| ------------------ | ------- | ---------------------------------------------------------------------------------- |
| `--br-in-tables`   | off     | Use `<br>` in table cells instead of converting to spaces.                         |
| `--compact-tables` | off     | Emit compact (unpadded) GFM tables: single space around cell content, no column-width alignment padding. |

## Highlighting

| Flag                        | Values                                 | Default        | Description                                      |
| --------------------------- | -------------------------------------- | -------------- | ------------------------------------------------ |
| `--highlight-style <STYLE>` | `double-equal`, `html`, `bold`, `none` | `double-equal` | `<mark>` rendering. `double-equal` → `==text==`. |

## Metadata

There is only one metadata flag. The metadata sub-fields (document, headers, links, images, structured data) populate automatically inside `result.metadata` when `--json` is used — there are no per-field extract flags.

| Flag                 | Description                                                                                  |
| -------------------- | ------------------------------------------------------------------------------------------- |
| `--extract-metadata` | Extract title and meta tags as an HTML comment header in Markdown output (plain text mode). |

Metadata is returned in `result.metadata` within the JSON output (use `--json` to see it):

```json
{
    "content": "# Title\n\nContent\n",
    "metadata": {
        "document": { "title": "...", "language": "en" },
        "headers": [...],
        "links": [...],
        "images": [...],
        "structured_data": [...]
    }
}
```

## Whitespace

| Flag                       | Values                 | Default      | Description                                                                   |
| -------------------------- | ---------------------- | ------------ | ----------------------------------------------------------------------------- |
| `--whitespace-mode <MODE>` | `normalized`, `strict` | `normalized` | Whitespace handling. `normalized` collapses spaces; `strict` preserves as-is. |
| `--strip-newlines`         | flag                   | off          | Remove all newlines from input HTML before processing.                        |

## Wrapping

| Flag               | Short | Values | Default | Description                            |
| ------------------ | ----- | ------ | ------- | -------------------------------------- |
| `--wrap`           | `-w`  | flag   | off     | Enable text wrapping.                  |
| `--wrap-width <N>` |       | 20–500 | `80`    | Column width when `--wrap` is enabled. |

## Element Handling

| Flag                     | Values          | Default   | Description                                                                          |
| ------------------------ | --------------- | --------- | ----------------------------------------------------------------------------------- |
| `--convert-as-inline`    | flag            | off       | Treat block elements as inline (no paragraph breaks).                               |
| `--strip-tags <TAGS>`    | comma-separated | none      | HTML tags to strip entirely (output only text content). E.g. `"script,style"`.      |
| `--preserve-tags <TAGS>` | comma-separated | none      | HTML tags to keep verbatim as raw HTML in the output. E.g. `"details,summary"`.     |
| `--skip-images`          | flag            | off       | Omit all `<img>` elements from the output entirely.                                 |
| `--exclude-selectors <SELECTORS>` | comma-separated CSS selectors | none | Remove matching elements and their descendants before conversion. E.g. `".ad,#sidebar"`. |
| `--max-depth <N>`        | integer         | unlimited | Silently truncate subtrees beyond this DOM nesting depth. Omit for unlimited depth. |

## Preprocessing

| Flag                  | Values                              | Default    | Description                                                                     |
| --------------------- | ----------------------------------- | ---------- | ------------------------------------------------------------------------------- |
| `--preprocess` / `-p` | flag                                | off        | Enable HTML preprocessing (removes navigation, ads, forms, etc.).               |
| `--preset <LEVEL>`    | `minimal`, `standard`, `aggressive` | `standard` | Preprocessing aggressiveness. Requires `--preprocess`.                          |
| `--keep-navigation`   | flag                                | off        | Don't remove `<nav>`, menus during preprocessing. Requires `--preprocess`.      |
| `--keep-forms`        | flag                                | off        | Don't remove `<form>`, `<input>` during preprocessing. Requires `--preprocess`. |

## Parsing

| Flag                    | Short | Default | Description                               |
| ----------------------- | ----- | ------- | ----------------------------------------- |
| `--encoding <ENCODING>` | `-e`  | `utf-8` | Input character encoding. E.g. `latin-1`. |
| `--tier-strategy <STRATEGY>` |  | `auto` | Conversion path selection: `auto` picks the fastest tier the input allows, `tier2` always takes the DOM-walk path. Output is identical either way; for benchmarking and bug isolation. |

## Output Format

| Flag                       | Short | Values             | Default    | Description           |
| -------------------------- | ----- | ------------------ | ---------- | --------------------- |
| `--output-format <FORMAT>` | `-f`  | `markdown`, `djot`, `plain` | `markdown` | Output markup format. |

## JSON Output

| Flag                      | Default | Description                                                                                                                                 |
| ------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `--json`                  | off     | Output the full `ConversionResult` as JSON instead of plain Markdown. JSON has keys: `content`, `tables`, `metadata`, `images`, `warnings`. |
| `--include-structure`     | off     | Include document structure tree in JSON output (`document` key). Requires `--json`.                                                         |
| `--extract-inline-images` | off     | Include extracted inline image data in JSON output (`images` key). Requires `--json`.                                                       |
| `--show-warnings`         | off     | Print non-fatal processing warnings to stderr.                                                                                              |
| `--no-content`            | off     | Suppress Markdown text output — only extract metadata/tables/images. Useful with `--json` for extraction-only mode.                         |

When `--json` is used, stdout receives JSON:

```json
{
    "content": "# Title\n\nContent\n",
    "metadata": {
        "document": { "title": "...", "language": "en" },
        "headers": [...],
        "links": [...],
        "images": [...],
        "structured_data": [...]
    },
    "tables": [...],
    "images": [...],
    "warnings": [...]
}
```

## Debugging

| Flag      | Default | Description                                 |
| --------- | ------- | ------------------------------------------- |
| `--debug` | off     | Output diagnostic warnings and information. |

## Meta

| Flag                            | Description                                                                             |
| ------------------------------- | --------------------------------------------------------------------------------------- |
| `--generate-completion <SHELL>` | Generate shell completion script. SHELL: `bash`, `zsh`, `fish`, `powershell`, `elvish`. |
| `--generate-man`                | Generate man page to stdout.                                                            |
| `--version`                     | Print version.                                                                          |
| `--help` / `-h`                 | Print help.                                                                             |

## Examples

```bash
# Basic conversion from stdin
echo '<h1>Title</h1><p>Content</p>' | html-to-markdown

# Convert file to stdout
html-to-markdown input.html

# Convert and save to file
html-to-markdown input.html -o output.md

# Fetch URL and convert
html-to-markdown --url https://example.com > output.md

# Fetch URL with custom user agent
html-to-markdown --url https://example.com --user-agent "MyBot/1.0"

# JSON output (ConversionResult with content, tables, metadata, images, warnings)
html-to-markdown --json input.html

# JSON output with document structure
html-to-markdown --json --include-structure input.html

# JSON output with inline images extracted
html-to-markdown --json --extract-inline-images input.html

# Extraction-only mode (no markdown text, just metadata/tables)
html-to-markdown --json --no-content input.html

# Show warnings to stderr
html-to-markdown --show-warnings input.html

# Full metadata extraction to file (metadata sub-fields populate automatically with --json)
html-to-markdown --json input.html -o output.json

# Web scraping with aggressive preprocessing
html-to-markdown page.html --preprocess --preset aggressive

# Custom heading and list styles
html-to-markdown input.html \
    --heading-style atx \
    --bullets '*' \
    --list-indent-width 2

# Discord/Slack-friendly output (2-space indents, backtick code blocks)
html-to-markdown input.html \
    --list-indent-width 2 \
    --code-block-style backticks

# Djot output format
html-to-markdown input.html --output-format djot

# Generate shell completions
html-to-markdown --generate-completion bash > html-to-markdown.bash
html-to-markdown --generate-completion zsh > _html-to-markdown

# Generate man page
html-to-markdown --generate-man > html-to-markdown.1
```

## Exit Codes

| Code | Meaning                       |
| ---- | ----------------------------- |
| 0    | Success                       |
| 1    | Conversion error or I/O error |
| 2    | Invalid arguments             |
