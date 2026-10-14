---
type: Decision
title: Typescript Conventions
x-ai-rulez:
  kind: rule
  id: typescript-conventions
  metadata:
    priority: high
---

- `strict: true` + `noUncheckedIndexedAccess` in tsconfig, never `any` — use `unknown` with type guards.
- ESM imports only, `const` over `let`, `as const` for literals, `interface` over `type` for objects.
- `import type` for type-only imports to avoid runtime overhead. Discriminated unions for type-safe state.
- Formatting/linting: `oxfmt` + `oxlint`. Type checking: `tsc --noEmit` in CI.
- Testing: `vitest` (80%+ coverage). Runtime validation at system boundaries with `zod`.
- Error handling: discriminated unions for expected errors, throw only for unexpected.
- Package manager: `pnpm` with `pnpm-lock.yaml` committed, build: `tsup` or `esbuild`.
- Monorepo: workspace protocol (`workspace:*`), shared tsconfig base, `pnpm-workspace.yaml`.
- Node.js: `node:` prefix for core modules, `fetch` over `axios`.
- Security: `pnpm audit` for dependency CVE scanning. Zero tolerance for critical/high vulnerabilities.
- Anti-patterns: non-null assertions (`!`), type assertions (`as`), `enum` (use unions), `@ts-ignore`.
