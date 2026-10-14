---
description: Use when converting HTML to Markdown or extracting metadata and tables through the html-to-markdown MCP server's tools, rather than shelling out to the CLI. Covers the tool surface, the auto-installing launcher, and when MCP beats the CLI or SDK.
name: html-to-markdown-using-the-mcp-server
---

<!--
AI-RULEZ :: GENERATED FILE — DO NOT EDIT
Content-Hash: blake3:4b986de78ce3caa3794cba251d5bec59ce5847d6a0c537d792db0d3c7e956204
Source-Hash: blake3:5dafffdc4edfda5a2230fc2decd518b71dd77f8ff3e161ef4f9bd0fa25b00f1f
Schema-Version: v1
-->

# Using the MCP Server

The `html-to-markdown` MCP server exposes the converter's conversion and
metadata-extraction capabilities as MCP tools, so an MCP-compatible client (Claude
Code, Claude Desktop) can convert HTML and pull structured metadata directly,
with no CLI invocation or glue code.

## Standalone skill installation

Installing this skill alone does not register an MCP server or include the plugin launcher.
Install the CLI (for example, `brew install xberg-io/tap/html-to-markdown`), then configure your MCP client with
`html-to-markdown mcp` as its stdio command. The full product plugin supplies registration where
the runtime supports MCP.

## How it runs in this plugin

The plugin auto-registers the server. Its command is the bundled launcher
`scripts/mcp-launch.sh`, which execs `html-to-markdown mcp`. On first run the
launcher resolves an `html-to-markdown` binary: it reuses one already on `PATH`
or cached in the plugin's `bin/`, then tries `npx`/`uvx`, then Homebrew, then
downloads a prebuilt from the tool's latest GitHub release. Override the channel
with `HTML_TO_MARKDOWN_LAUNCHER=auto|npx|uvx|brew|download`.

The `mcp` subcommand ships in a recent release of the tool. If an older
`html-to-markdown` is already on `PATH`, it may not expose `mcp` yet — upgrade
the binary (`brew upgrade`, re-download, or rebuild) to pick it up.

To run it manually:

```bash
html-to-markdown mcp        # for Claude Code / Claude Desktop (stdio)
```

## The tools

- **convert_html** — convert an HTML string to Markdown, Djot, or plain text.
  Takes `html` (the HTML string), an optional `config` object mirroring
  `ConversionOptions`, and an optional `json` flag. With `json: true` it returns
  the full `ConversionResult` (content, tables, metadata, document structure,
  inline images, warnings) instead of the bare converted text.
- **extract_metadata** — pull structured metadata (document info, Open
  Graph / Twitter / JSON-LD / Microdata, headers, links, images) from an HTML
  string. Takes only `html`.

Both tools accept an HTML **string** — the MCP server does not fetch URLs or read
files (that is CLI-only). The exact argument schemas come from the running
server; ask the client to list tools to see the live surface.

## When to prefer MCP over the CLI or SDK

- **Prefer MCP** inside an agent session: the agent calls the converter directly
  as a tool, with no shell-out and no process management.
- **Prefer the CLI** for one-shot conversions and shell pipelines (`… | jq`).
- **Prefer the SDK** when embedding conversion in application code.
