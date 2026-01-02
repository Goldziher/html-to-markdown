# ADR 002: Thin Binding Wrappers

**Date:** 2024-07-22

**Status:** Accepted

## Context

Following the decision to implement all core logic in Rust (ADR-001), we face the challenge of supporting language bindings for 10+ different ecosystems. The maintenance burden of duplicating business logic across all these languages would be enormous.

Initial analysis shows that supporting full, independent implementations would require:

- 10 separate code bases with independent logic
- 10 separate test suites
- 10 separate bug fix cycles
- Potential for 10x more bugs due to logic divergence

This approach is unsustainable. Instead, the bindings should act as lightweight adapters that translate language-specific patterns into Rust core calls and vice versa.

## Decision

All language-specific bindings will be implemented as thin, idiomatic wrappers around the Rust core. The primary responsibilities of each binding are:

1. **FFI translation:** Marshal function calls to the Rust core and unmarshal results
1. **Idiomatic APIs:** Present language-native APIs that feel natural to users of that language
1. **Type conversion:** Convert between language-native types and Rust types via FFI boundaries
1. **Memory management:** Handle language-specific memory semantics (garbage collection, reference counting, etc.)
1. **Error mapping:** Convert Rust errors into language-native error types

Bindings should NOT implement business logic, validation rules, or core algorithms. These belong in Rust.

## Consequences

### Positive

- **Dramatically reduced maintenance burden:** Logic exists in one place, reducing complexity by 10x
- **Easier updates:** Features and bug fixes need to be applied only once
- **Consistent behavior:** All bindings automatically have identical semantics
- **Faster time to market:** New language support can be added more quickly
- **Clear separation of concerns:** Each binding has a well-defined, limited scope

### Negative

- **FFI overhead:** Every operation incurs FFI marshaling costs, potentially impacting performance
- **Binding complexity:** Each binding must correctly handle FFI interactions and memory safety
- **Language-specific limitations:** Some advanced Rust features may be difficult to expose to certain languages
- **Limited language-specific optimizations:** Bindings cannot implement language-specific performance optimizations
- **Debugging challenges:** Errors may originate in either the binding layer or Rust core

### Performance Considerations

- FFI calls add latency; batching operations where possible is recommended
- Zero-copy interfaces should be used for large data transfers
- Async operations should minimize FFI round-trips

### Guidelines for Implementation

- Each binding should have \<2000 lines of source code (excluding tests)
- Bindings should be generated or templated where possible
- Documentation should be generated from a single source
