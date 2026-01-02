______________________________________________________________________

## priority: critical

# Universal Anti-Patterns

**Never use:**

- Any type (Python, TypeScript) - use Unknown/generics
- Class-based tests (Python) - function-based only
- Mocking internal services (any language) - use real objects
- Manual dependency management - use lock files
- Blocking I/O in async code (Python/TypeScript) - fully async paths
- Bare exception handlers - catch specific types only
- Magic numbers - extract to named constants
- Inheritance for code reuse - prefer composition
- Global state - dependency injection
- f-strings in logging - structured key=value logging
