# WASM Smoke Test

Runs a minimal script against the published **`html-to-markdown-wasm`** bundle and
against a locally-built copy of the WebAssembly artifacts.

## 1. Test the latest npm release

```bash
cd examples/wasm-smoke
pnpm install
pnpm run check
```

## 2. Test a local build

```bash
pnpm --filter html-to-markdown-wasm run build:all     # produces dist/, dist-node/, dist-web/
cd examples/wasm-smoke
pnpm install
pnpm add html-to-markdown-wasm@file:../../crates/html-to-markdown-wasm --save-prod --ignore-workspace-root-check
pnpm run check
```

After running the local test you can reset the tracked files via

```bash
git checkout -- package.json pnpm-lock.yaml
```
