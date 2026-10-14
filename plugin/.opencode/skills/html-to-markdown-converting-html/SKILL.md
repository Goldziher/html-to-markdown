---
description: Use when converting HTML to Markdown, Djot, or plain text. Covers output formats, heading and code-block styles, lists, escaping, wrapping, and HTML preprocessing.
name: html-to-markdown-converting-html
---

<!--
AI-RULEZ :: GENERATED FILE — DO NOT EDIT
Content-Hash: blake3:f7de17dbd5516b1e6b26173c3ee40ecea4262a6f5ef168a768eced5bdc7e6126
Source-Hash: blake3:d85ad7bf79fa4cb1ed116e7fd2574bf5e6f64f503331a1bacf3cc3d495e3c873
Schema-Version: v1
-->

# Converting HTML

Use this when the user wants to turn HTML into clean Markdown (or Djot) — a
saved page, an HTML email body, a fragment, or a string. Conversion is driven by
**flags** with a positional `FILE` (stdin is the default input when no file is
given); the only subcommand is `mcp` (run-as-server, see the
`html-to-markdown-using-the-mcp-server` skill).

## Basic conversion

```bash
# File to stdout
html-to-markdown input.html

# File to a file
html-to-markdown input.html -o output.md

# From stdin
cat page.html | html-to-markdown
echo '<h1>Title</h1><p>Body</p>' | html-to-markdown
```

The default (no `--json`) prints plain Markdown text. Use `--json` only when you
need structured metadata/tables (see the `html-to-markdown-extracting-metadata` and
`html-to-markdown-extracting-tables` skills).

## Output format

```bash
html-to-markdown input.html --output-format markdown   # default
html-to-markdown input.html --output-format djot        # Djot markup
html-to-markdown input.html -f djot                      # short form
```

To get *plain text* (no Markdown syntax), strip tags and treat blocks inline:

```bash
html-to-markdown input.html --convert-as-inline --strip-tags "script,style"
```

## Heading and code-block styles

| Flag | Values | Default | Effect |
| ---- | ------ | ------- | ------ |
| `--heading-style` | `atx`, `underlined`, `atx-closed` | `atx` | `# h1` vs `h1\n===` vs `# h1 #` |
| `--code-block-style` | `backticks`, `indented`, `tildes` | `backticks` | Fence style for code blocks |
| `--code-language` / `-l` | string | `""` | Default language for fenced blocks |

```bash
html-to-markdown input.html --heading-style underlined --code-block-style tildes
```

## Lists and text formatting

```bash
# Bullet characters cycle by nesting depth; indent width 1–8
html-to-markdown input.html --bullets '*+-' --list-indent-width 2

# Indent with tabs instead of spaces
html-to-markdown input.html --list-indent-type tabs

# Emphasis symbol and escaping
html-to-markdown input.html --strong-em-symbol _ --escape-misc
```

Escaping flags (`--escape-asterisks`, `--escape-underscores`, `--escape-misc`,
`--escape-ascii`) trade readability for strict CommonMark safety. Use
`--escape-ascii` only when the output must round-trip through a strict parser.

## Wrapping

```bash
html-to-markdown input.html --wrap --wrap-width 100   # off by default; 80 if --wrap with no width
```

## Preprocessing (clean noisy pages)

Strip navigation, ads, and forms before converting — essential for scraped web
pages:

```bash
html-to-markdown input.html --preprocess                     # standard preset
html-to-markdown input.html --preprocess --preset aggressive  # strip more
html-to-markdown input.html --preprocess --keep-navigation    # keep <nav>
html-to-markdown input.html --preprocess --keep-forms         # keep <form>
```

Presets: `minimal`, `standard` (default), `aggressive`. `--preset`,
`--keep-navigation`, and `--keep-forms` all require `--preprocess`.

## Programmatic equivalents

```python
from html_to_markdown import convert, ConversionOptions, PreprocessingOptions

html = "<h1>Title</h1><p>Body text.</p>"
result = convert(
    html,
    ConversionOptions(
        heading_style="atx",
        code_block_style="backticks",
        wrap=True,
        wrap_width=100,
        preprocessing=PreprocessingOptions(enabled=True, preset="aggressive"),
    ),
)
print(result.content)
```

```rust
use html_to_markdown_rs::{convert, ConversionOptions, HeadingStyle};

let options = ConversionOptions::builder().heading_style(HeadingStyle::Atx).wrap(true).build();
let result = convert(html, Some(options))?;
```

See `references/cli-reference.md` for the full flag set and
`references/configuration.md` for every option default.
