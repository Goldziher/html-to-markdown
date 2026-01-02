# ADR 003: FFI Error Boundaries

**Date:** 2024-08-10

**Status:** Accepted

## Context

Different programming languages have fundamentally different error handling models:

- **Rust:** Result\<T, E> with typed errors and exhaustive pattern matching
- **Python:** Exceptions with inheritance hierarchies
- **JavaScript:** Exceptions, callbacks, and Promise rejections
- **Go:** Explicit return values with error as second return
- **Java:** Checked and unchecked exceptions
- **C#:** Try-catch with custom exception hierarchies

When the Rust core encounters an error, the language binding must translate that error into the appropriate model for the target language. Having this translation happen in the Rust core would couple the core to all language-specific error models, violating separation of concerns.

Previous attempts at polyglot libraries that tried to unify error handling across languages resulted in awkward, unidiomatic APIs that felt foreign to users of each language.

## Decision

Error conversion will occur at the FFI boundary, not in the Rust core. The process works as follows:

1. **Rust core returns structured errors:** The Rust core returns detailed, typed error information using standard Rust error types
1. **FFI layer receives raw error data:** The FFI boundary receives the error information from Rust
1. **Binding converts to language idioms:** Each language binding converts the error data into the appropriate language-native error representation
1. **Users receive familiar error patterns:** Application code receives errors in the familiar patterns of their language

For example:

- Rust: `Result<Rules, ParseError>` -> Python: `raise ParseError(...)`
- Rust: `Result<Rules, ParseError>` -> JavaScript: `reject(new ParseError(...))`
- Rust: `Result<Rules, ParseError>` -> Go: `return nil, ParseError(...)`

## Consequences

### Positive

- **Language-idiomatic error handling:** Users of each language work with familiar error patterns
- **Clean Rust core:** Core logic is not polluted with language-specific error adaptation code
- **Flexible error transformation:** Each binding can enrich errors with language-specific context
- **Better debugging:** Errors feel native to each language, making debugging easier
- **Type safety in each language:** Languages with strong typing can map to their error types

### Negative

- **Duplicate error definition:** Error types must be defined in both Rust and each binding
- **Synchronization burden:** Changes to error types in Rust must be reflected in all bindings
- **Potential inconsistencies:** If error mapping is implemented inconsistently, behaviors may diverge
- **More FFI complexity:** Bindings must implement error marshaling logic
- **Testing overhead:** Each binding must test error conversion scenarios

### Error Categories

We will standardize on these core error categories in Rust:

- `ValidationError` - Input validation failed
- `ParseError` - Parsing/deserialization failed
- `ExecutionError` - Execution/processing failed
- `ConfigError` - Configuration is invalid
- `NotFoundError` - Requested resource not found
- `InternalError` - Internal consistency error (should be reported as bug)

### Implementation Standards

1. All Rust errors must include error codes and descriptions
1. Error codes are stable across versions
1. Bindings must implement error constructors for all error categories
1. Error conversion functions should be reusable across multiple operations
1. Documentation must map Rust errors to language-specific implementations
