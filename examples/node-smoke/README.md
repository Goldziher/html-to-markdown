# Node Smoke Test

Verifies that the published **`html-to-markdown-node`** package installs cleanly and that
locally-built artifacts still load after rebuilding the crate.

## 1. Test the latest npm release

```bash
cd examples/node-smoke
pnpm install
pnpm run check
```

You should see the rendered Markdown plus a green checkmark.

## 2. Test a local build from this repo

```bash
pnpm --filter html-to-markdown-node run build          # produce fresh .node binaries
cd examples/node-smoke
pnpm install                                           # installs the latest release first
pnpm add html-to-markdown-node@file:../../crates/html-to-markdown-node --save-prod --ignore-workspace-root-check
pnpm run check
```

> `pnpm add` rewrites `package.json`/`pnpm-lock.yaml`. Run
> `git checkout -- package.json pnpm-lock.yaml` inside `examples/node-smoke`
> when you finish testing to restore the tracked versions.
