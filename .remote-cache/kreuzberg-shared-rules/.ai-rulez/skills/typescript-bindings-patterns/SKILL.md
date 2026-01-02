______________________________________________________________________

## priority: critical

# TypeScript Bindings Patterns

**Role**: TypeScript bindings for Rust core. Work on NAPI-RS bridge and TypeScript SDK packages.

**Scope**: NAPI-RS FFI, TypeScript-idiomatic API, type definitions, JSDoc for all exports with @param/@returns/@example.

**Commands**: pnpm install/build/test/lint.

**Critical**: Core logic lives in Rust. TypeScript only for bindings/wrappers. If core logic needed, coordinate with Rust team.
