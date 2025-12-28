---
priority: medium
---

# TypeScript 5.x - NAPI-RS Bindings & CLI

**TypeScript 5.x · NAPI-RS native bindings · Strictest typing · vitest · 80%+ coverage**

- Enable ALL strict flags: strict, noUncheckedIndexedAccess, exactOptionalPropertyTypes
- NAPI-RS bindings in crates/html-to-markdown-node, consumed by packages/typescript
- Ban any and object types; use unknown with guards, Record<string, unknown>
- Tests: .spec.ts next to source files (packages/typescript/tests); vitest, 80%+ coverage
- CLI wrapper with TypeScript commands; pnpm ≥10.17, pnpm-lock.yaml committed
- Biome for linting/formatting; import type for types, path aliases (@/lib/*)
- Never: any/object types, non-null assertions !, hardcoded CLI commands
- Use Haiku 4.5 for CLI engineering and TypeScript binding issues
