---
priority: medium
---

# Universal Anti-Patterns

**Cross-language patterns to NEVER use:**
- Any type (Python, TypeScript, Rust unknown) without exhaustive matching
- Class-based tests (Python) – use function-based with pytest fixtures
- Unwrap/panic in production code (Rust) – use Result<T, E>
- Mocking internal services – use real objects/fixtures
- Manual dependency management – use lock files (Cargo.lock, pnpm-lock.yaml, etc.)
- Blocking I/O in async code (Python/TypeScript) – fully async paths
- Bare exception handlers – catch specific types only
- Magic numbers – extract to named constants
- Inheritance for code reuse – prefer composition
- Global state – dependency injection instead
- f-strings in logging – structured key=value logging
- Duplication across bindings – core logic ALWAYS in Rust
