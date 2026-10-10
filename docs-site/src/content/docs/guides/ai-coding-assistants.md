---
title: AI Coding Assistants
---

Give your coding agent working knowledge of html-to-markdown so it writes correct `convert()` calls, picks the right options, and reaches for the right binding — without you pasting docs into the chat.

## What this plugin does

The plugin packages html-to-markdown's usage patterns, options, and per-language APIs as agent skills. Once installed, your assistant can answer html-to-markdown questions and generate accurate code straight from your editor or terminal. It installs from this repo's own [`xberg-io/html-to-markdown`](https://github.com/xberg-io/html-to-markdown) marketplace and works with every major coding agent — pick yours below.

## Install as a plugin

<details open>
<summary><strong>Claude Code</strong></summary>

```text
/plugin marketplace add xberg-io/html-to-markdown
/plugin install html-to-markdown@html-to-markdown
```

</details>

<details>
<summary><strong>Codex CLI</strong></summary>

```text
/plugins add https://github.com/xberg-io/html-to-markdown
```

Then search for `html-to-markdown` and select **Install Plugin**.
</details>

<details>
<summary><strong>Cursor</strong></summary>

Settings → Plugins → Add from URL → `https://github.com/xberg-io/html-to-markdown`, then select **html-to-markdown**.
</details>

<details>
<summary><strong>Gemini CLI</strong></summary>

```text
gemini extensions install https://github.com/xberg-io/html-to-markdown
```

</details>

<details>
<summary><strong>Factory Droid</strong></summary>

```text
droid plugin marketplace add https://github.com/xberg-io/html-to-markdown
droid plugin install html-to-markdown@html-to-markdown
```

</details>

<details>
<summary><strong>GitHub Copilot CLI</strong></summary>

```text
copilot plugin marketplace add https://github.com/xberg-io/html-to-markdown
copilot plugin install html-to-markdown@html-to-markdown
```

</details>

<details>
<summary><strong>Pi</strong></summary>

From a checkout of this repository:

```bash
pi install ./plugin
```

After the next plugin release, install the same package used by OpenCode:

```bash
pi install npm:@xberg-io/opencode-html-to-markdown
```

Pi loads the bundled skills and prompt templates. Use `/html-to-markdown-convert` or `/html-to-markdown-metadata` with the input as arguments. Install the `html-to-markdown` CLI separately for shell access. This package does not register MCP servers, hooks, or native Pi extensions.

</details>

<details>
<summary><strong>opencode</strong></summary>

Add the package to `opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "plugin": ["@xberg-io/opencode-html-to-markdown"]
}
```

</details>

## Install as an MCP server

The MCP-capable integrations above bundle the html-to-markdown MCP server and register it for you. Pi uses skills and CLI access instead. To wire the same server into any other MCP client directly, point it at the published CLI — no plugin required:

```json
{
  "mcpServers": {
    "html-to-markdown": {
      "command": "npx",
      "args": ["-y", "@xberg-io/html-to-markdown-cli@latest", "mcp"]
    }
  }
}
```

Python users can launch it through `uvx` instead:

```json
{
  "mcpServers": {
    "html-to-markdown": {
      "command": "uvx",
      "args": ["--from", "html-to-markdown-cli", "html-to-markdown", "mcp"]
    }
  }
}
```

The server exposes `convert_html` and `extract_metadata` — see the [MCP Reference](/reference/mcp/) for the full tool and parameter schema.

For the Hermes Python agent framework, install the plugin as a package instead:

```bash
pip install html-to-markdown-hermes-plugin
```
