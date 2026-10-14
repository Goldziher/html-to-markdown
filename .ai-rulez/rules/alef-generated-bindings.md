---
type: Decision
title: Alef Generated Bindings
x-ai-rulez:
  kind: rule
  id: alef-generated-bindings
  metadata:
    priority: critical
---

- Files in `packages/*/` and binding crates are generated or managed by Alef — check `alef.toml` before editing
- `alef.toml` defines: output paths, module names, rename mappings, e2e call overrides, README templates
- Run `task alef:generate` after changing `alef.toml`; it runs `alef all --clean`
- Formatting is handled solely by poly (`task format` / `poly fmt --fix .`); there is no separate Alef formatting task
- Use `task build:bindings` or `task build:all` explicitly when bindings must be built
- Never hand-edit generated files; modify `alef.toml` or the Rust source instead
- Fixtures under `fixtures/` are the input to `alef e2e generate` (`task e2e:generate`), which writes `e2e/` — never add tests to `e2e/` directly. There is no `tools/e2e-generator`.
- Canonical e2e tasks are `task e2e:generate`, `task e2e:build`, `task e2e:test`, and `task e2e:all`; do not add legacy aliases
