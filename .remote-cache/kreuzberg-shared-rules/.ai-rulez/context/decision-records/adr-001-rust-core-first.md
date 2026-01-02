# ADR 001: Rust Core First

**Date:** 2024-06-15

**Status:** Accepted

## Context

The ai-rulez project is being developed as a polyglot library that needs to support multiple programming languages (Python, JavaScript, Go, Ruby, PHP, Java, C#, Kotlin, Swift, and more). Historically, polyglot projects have suffered from inconsistent behavior across language implementations due to separate code bases and different development teams.

As the project scales to support 10+ languages, maintaining consistency becomes increasingly difficult. Each language might interpret requirements differently, leading to subtle bugs and API parity issues. There is a need for a single source of truth that ensures all bindings behave identically.

Additionally, the business logic for ai-rulez is complex and involves sophisticated rule processing, validation, and transformation. Rewriting this logic for each target language is error-prone, time-consuming, and difficult to maintain.

## Decision

All core business logic will be implemented first and foremost in Rust. Language-specific bindings will be thin wrappers that delegate to the Rust core via Foreign Function Interface (FFI) calls. New features and bug fixes are implemented in Rust before being exposed through language bindings.

This includes:

- Rule parsing and processing
- Validation logic
- Core transformations
- Complex algorithms
- State management

## Consequences

### Positive

- **Single source of truth:** All business logic exists in one place, reducing the surface area for bugs
- **Guaranteed API parity:** All language bindings expose identical behavior by design
- **Faster feature development:** New features only need to be implemented once in Rust
- **Easier maintenance:** Bug fixes in core logic benefit all bindings immediately
- **Performance baseline:** Rust provides excellent performance for computationally intensive operations

### Negative

- **Rust expertise requirement:** The team must maintain strong Rust capabilities
- **FFI complexity:** All bindings must correctly interface with FFI, adding a layer of complexity
- **Rust compilation overhead:** Builds must compile the Rust core for each target platform
- **Bootstrap complexity:** Initial Rust implementation requires careful design to support diverse use cases

### Mitigation Strategies

- Establish clear FFI boundaries and interfaces
- Invest in comprehensive testing of the Rust core
- Document FFI patterns and best practices
- Create reusable FFI templates for new bindings
