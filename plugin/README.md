# html-to-markdown

Fast, lossless HTML→Markdown conversion with structured metadata, tables, and document-structure extraction — using the local `html-to-markdown` CLI in your agent.

<!-- ~keep TODO: add screenshot -->

## Install

### From the marketplace (recommended)

Pending review for official Claude marketplace.

Self-host:

```text
/plugin marketplace add xberg-io/html-to-markdown
/plugin install html-to-markdown@html-to-markdown
```

### Binary requirement

The bundled MCP launcher (`scripts/mcp-launch.sh`) resolves an
`html-to-markdown` binary automatically on first run: it reuses one already on
`PATH`, then tries `npx`/`uvx`, then Homebrew, then a prebuilt download from the
tool's latest GitHub release. No manual install is required to use the MCP
server.

To install the CLI yourself:

```bash
# (Homebrew 6.0+ requires explicit trust for third-party taps)
brew trust xberg-io/tap
brew install xberg-io/tap/html-to-markdown
# or run it without a persistent install (the CLI proxy package self-installs the binary):
pnpm dlx @xberg-io/html-to-markdown-cli --help
uvx --from html-to-markdown-cli html-to-markdown --help
# or download a prebuilt binary from the latest GitHub release:
#   https://github.com/xberg-io/html-to-markdown/releases/latest
# or build from source:
cargo install html-to-markdown-cli
```

The skills also cover the language SDKs. Install the one you need:

```bash
pip install html-to-markdown                  # Python
pnpm add @xberg-io/html-to-markdown        # TypeScript / Node.js
cargo add html-to-markdown-rs                  # Rust
gem install html-to-markdown                   # Ruby
```

### Pi

From a checkout of this repository:

```bash
pi install ./plugin
```

After the next plugin release, install the same package used by OpenCode:

```bash
pi install npm:@xberg-io/opencode-html-to-markdown
```

Pi loads the bundled skills and prompt templates. Use `/html-to-markdown-convert` or `/html-to-markdown-metadata` with the input as arguments. Install the `html-to-markdown` CLI separately for shell access. This package does not register MCP servers, hooks, or native Pi extensions.

## Skills shipped

| Skill | Trigger |
|-------|---------|
| **html-to-markdown** | Convert HTML to Markdown, Djot, or plain text with structured extraction. Use when writing code that calls html-to-markdown APIs in Rust, Python, TypeScript, Go, Ruby, PHP, Java, C#, Elixir, R, C, or WASM. Covers installation, conversion, configuration, metadata extraction, tables, document structure, and CLI usage. |
| **html-to-markdown-converting-html** | Use when converting HTML to Markdown, Djot, or plain text. Covers output formats, heading and code-block styles, escaping, wrapping, and HTML preprocessing. |
| **html-to-markdown-extracting-metadata** | Use when extracting metadata from HTML — title, description, language, Open Graph, JSON-LD / Microdata / RDFa, headers, links, and images. Covers the `--json` output shape and the `--extract-metadata` flag. |
| **html-to-markdown-extracting-tables** | Use when extracting tabular data from HTML. Covers GFM Markdown tables, the structured `tables` array (grid cells + pre-rendered markdown), and `<br>` handling in cells. |
| **html-to-markdown-fetching-and-converting-urls** | Use when fetching a live URL and converting it to Markdown. Covers `--url`, custom user agents, preprocessing for noisy pages, and the `--json` ConversionResult shape. |
| **html-to-markdown-using-the-mcp-server** | Use when converting HTML and extracting metadata through the MCP server's `convert_html`/`extract_metadata` tools instead of the CLI. Covers the tool surface and the auto-installing launcher. |

**Reference materials** (linked from the `html-to-markdown` skill):

| Reference | Content |
|-----------|---------|
| **CLI Reference** | All flags, output formats, JSON shape, exit codes |
| **Configuration Reference** | All 30+ ConversionOptions fields with defaults |
| **Rust API Reference** | Functions, builder options, feature flags |
| **Python API Reference** | Functions, dataclasses, type hints |
| **TypeScript API Reference** | Functions, interfaces, Buffer support |
| **Other Language Bindings** | Go, Ruby, PHP, Java, C#, Elixir, R, WASM, C FFI |

## MCP server

MCP-capable integrations auto-register a server named `html-to-markdown`, launched via
`scripts/mcp-launch.sh` (which execs `html-to-markdown mcp`). It exposes two
tools — `convert_html` (HTML string → Markdown/Djot/plain text, or the full
`ConversionResult` JSON with `json: true`) and `extract_metadata` (structured
metadata from an HTML string) — so the agent can convert HTML directly without
shelling out to the CLI. The launcher auto-installs a
binary on first run (override with
`HTML_TO_MARKDOWN_LAUNCHER=auto|npx|uvx|brew|download`). The `mcp` subcommand
ships in a recent release of the tool; an older binary on `PATH` may need an
upgrade to expose it. See the **html-to-markdown-using-the-mcp-server** skill for details.

## CLI / SDK usage

The conversion CLI takes **flags only** — `FILE` is positional; omit it (or use `-`) to read HTML from stdin. (The one subcommand is `mcp`, which starts the MCP server.)

```bash
html-to-markdown input.html                    # convert file to stdout
html-to-markdown input.html -o output.md       # convert to a file
cat page.html | html-to-markdown               # read from stdin
html-to-markdown --url https://example.com     # fetch and convert a URL
html-to-markdown --json input.html             # full ConversionResult as JSON
```

The same single entry point exists across the SDKs — `convert()` returns a structured `ConversionResult` (`content`, `metadata`, `tables`, `images`, `warnings`):

```python
from html_to_markdown import convert

result = convert("<h1>Hello</h1><p>World</p>")
print(result.content)   # # Hello\n\nWorld
print(result.metadata)  # title, links, headers, …
```

```typescript
import { convert } from "@xberg-io/html-to-markdown";

// Node's convert() returns a ConversionResult object directly.
const result = convert("<h1>Hello</h1><p>World</p>");
console.log(result.content);  // # Hello\n\nWorld
```

Prefer the CLI for one-shot conversions and shell pipelines; prefer the SDKs when embedding conversion in application code.

## Configuration

All conversion behavior is controlled by CLI flags (or the matching `ConversionOptions` fields in the SDKs):

| Flag | Values | Default | Purpose |
|------|--------|---------|---------|
| `--output-format` / `-f` | `markdown`, `djot`, `plain` | `markdown` | Output markup format. |
| `--heading-style` | `atx`, `underlined`, `atx-closed` | `atx` | Heading rendering. |
| `--code-block-style` | `backticks`, `indented`, `tildes` | `backticks` | Code fence style. |
| `--preprocess` / `-p` | flag | off | Strip nav, ads, forms before converting. |
| `--preset` | `minimal`, `standard`, `aggressive` | `standard` | Preprocessing aggressiveness (needs `--preprocess`). |
| `--wrap` / `-w`, `--wrap-width` | flag, 20–500 | off, `80` | Text wrapping. |
| `--json` | flag | off | Emit the full `ConversionResult` JSON. |
| `--include-structure` | flag | off | Add the document-structure tree (needs `--json`). |
| `--no-content` | flag | off | Extraction-only — skip the Markdown text. |

See `skills/html-to-markdown/references/configuration.md` for the full 30+ option table and `skills/html-to-markdown/references/cli-reference.md` for every flag.

## Examples

Convert an HTML file to Markdown:

```text
html-to-markdown article.html -o article.md
```

Scrape a page with aggressive preprocessing:

```text
html-to-markdown --url https://example.com/blog --preprocess --preset aggressive
```

Extract metadata and tables only (no Markdown body):

```text
html-to-markdown --json --no-content page.html | jq '{title: .metadata.document.title, tables: (.tables | length)}'
```

Convert with Djot output and underlined headings:

```text
html-to-markdown input.html --output-format djot --heading-style underlined
```

## Versioning

The plugin version tracks the html-to-markdown core release (`Cargo.toml`). See [CHANGELOG.md](../CHANGELOG.md) for release notes.

## License

MIT. The skill content references the upstream [html-to-markdown](https://github.com/xberg-io/html-to-markdown) repository.

## See also

- **Marketplace**: this repo — `/plugin marketplace add xberg-io/html-to-markdown`
- **Upstream**: [xberg-io/html-to-markdown](https://github.com/xberg-io/html-to-markdown)

## Generated runtime bundles

`plugin/.ai-rulez/config.toml` loads `config.plugins.toml` through the `plugins` layer;
skills, commands and runtime adapters are authored alongside it. AI-Rulez 5.2.1 generates
all eleven supported runtimes: Claude, Cursor, Codex, Gemini, Kimi, OpenCode, Factory,
Hermes, Pi, Agent Plugins and Copilot. Codex ships both manifest layouts; repository-root
Claude and Codex marketplace indexes point to the `plugin` bundle. Cursor also has a marketplace index.

Pi packages skills and prompts without MCP integration. Copilot packages skills and MCP configuration, but does not package slash commands or hooks.
Hermes includes a Python package under `.hermes/package`; OpenCode and Pi share the npm package.
The Agent Plugins manifest uses the stable 1.0 specification. Runtime capabilities differ;
install the CLI before using skills in a runtime without MCP registration.

Run `ai-rulez generate --plugin` from the repository root to regenerate and
`ai-rulez verify --plugin` to compare current source output and provenance.
The release workflow packages every runtime into a deterministic archive, checksums,
and marketplace metadata; npm and PyPI retain their existing publishing jobs.

## Hermes runtime setup

Install the product CLI with MCP support and put it on Hermes' PATH before using the plugin.
The Hermes adapter registers every bundled skill under the product namespace and provides
`html_to_markdown_call`, which calls the selected native MCP tool with `tool_arguments` through a local
stdio session. Select `tool: "list_tools"` first to inspect the server's exact request schemas. Hermes skills are selected explicitly; they are not automatically appended to the
system prompt. Enable the plugin in Hermes configuration; project plugins also require
`HERMES_ENABLE_PROJECT_PLUGINS=true`.

The bridge uses the installed CLI directly. The auto-installing launchers used by other MCP clients
are not included in the Hermes Python wheel. Provider credentials are inherited from the environment,
and are needed for provider-backed requests. Missing CLIs, unavailable tools and model preparation
fail with an actionable request error. CLI result data remains untrusted document content.

## Public Agent Skills

The repository-root `skills/` directory contains the same generated skills and supporting
resources as `plugin/skills/`. Install them independently of a plugin runtime:

```sh
npx skills add xberg-io/html-to-markdown --skill html-to-markdown
```

Edit `plugin/.ai-rulez/skills`, then run `task plugin:generate` to regenerate runtime
bundles and synchronize the public export. `task plugin:verify` checks both projections;
`task plugin:skills:test` verifies resource freshness, ownership and symlink protection.
The public ownership manifest records SHA-256 hashes of every exported file.

Launcher and visual assets are authored in `plugin/.ai-rulez/scripts/` and
`plugin/.ai-rulez/assets/`; `task plugin:generate` copies them into the runtime bundle.
