---
title: AI Coding Assistants
---

Give your coding agent working knowledge of html-to-markdown so it writes correct `convert()` calls, picks the right options, and reaches for the right binding — without you pasting docs into the chat.

## What this plugin does

The plugin packages html-to-markdown's usage patterns, options, and per-language APIs as agent skills. Once installed, your assistant can answer html-to-markdown questions and generate accurate code straight from your editor or terminal. It installs from this repo's own [`xberg-io/html-to-markdown`](https://github.com/xberg-io/html-to-markdown) marketplace ; choose a runtime below.

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

From a checkout of this repository:

```sh
gemini extensions install ./plugin
```

The manifest is inside `plugin/`. See the [Gemini extension reference](https://geminicli.com/docs/extensions/reference/)
for local installation and updates.

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

## Runtime resources and prerequisites

The generated bundle targets Claude Code, Codex, Cursor, Gemini CLI, Kimi, OpenCode,
Factory Droid, Hermes, Pi, Agent Plugins and GitHub Copilot CLI. Use
[the plugin installation guide](https://github.com/xberg-io/html-to-markdown/blob/main/plugin/README.md)
for the runtime's installation method and configuration.

| Runtime | Integration |
| --- | --- |
| Claude, Codex, Cursor, Factory | Native plugin manifests, skills and MCP configuration |
| Gemini CLI | Extension manifest, command templates and MCP configuration |
| Kimi | Plugin manifest and session instructions |
| OpenCode | npm adapter, instructions and MCP configuration |
| Hermes | Python plugin, explicitly selected skills and a local stdio MCP bridge |
| Pi | Skills and prompt templates; execute operations through the CLI |
| Agent Plugins, Copilot | Agent Plugins 1.0 manifest, skills and MCP configuration |

Install the product CLI before using Hermes or Pi. Hermes starts the installed CLI directly;
its wheel does not include the auto-installing launcher used by other MCP clients.
Select `list_tools` through the product's Hermes bridge to inspect the supported tools and
request schemas. Select skills explicitly when needed; they are not appended automatically
to every conversation.

From a checkout, install the generated Hermes package into Hermes' Python environment:

```sh
pip install ./plugin/.hermes/package
```

Enable the plugin in Hermes configuration. Loading project plugins also requires
`HERMES_ENABLE_PROJECT_PLUGINS=true`. After a release publishes the updated package, the
registry install is `pip install html-to-markdown-hermes-plugin`.

Pi resources can be installed from the checkout with `pi install ./plugin`.
After a plugin release includes them, install `npm:@xberg-io/opencode-html-to-markdown` through Pi.
Pi does not register an MCP server, hooks or executable extensions from this package.

## Standalone public skills

The [public skills directory](https://github.com/xberg-io/html-to-markdown/tree/main/skills) contains
instructions you can install independently of the runtime plugin. Use the
[skills CLI](https://github.com/vercel-labs/skills) to list and select them:

```sh
npx skills add xberg-io/html-to-markdown --list
npx skills add xberg-io/html-to-markdown
```

Choose the target agents interactively. For a local checkout, use `npx skills add ./skills`.
This installs instructions; it does not install the product CLI, enable MCP, download models
or set provider credentials.
