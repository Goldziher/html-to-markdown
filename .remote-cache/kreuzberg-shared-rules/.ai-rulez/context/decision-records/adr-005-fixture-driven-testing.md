# ADR 005: Fixture-Driven Testing

**Date:** 2024-10-12

**Status:** Accepted

## Context

Given that ai-rulez supports 10+ language bindings that all delegate to a shared Rust core, we need a robust way to verify that the API behaves consistently across all implementations.

The challenge is significant:

- Manual testing each binding separately is time-consuming and error-prone
- Different test frameworks across languages make unified testing difficult
- Edge cases might be missed in some bindings but not others
- Regression testing becomes exponentially complex with N languages

Previous polyglot projects have struggled with this problem, leading to subtle bugs where behavior differs across languages. For example, one language binding might handle error cases differently, or return slightly different data structures.

We need a mechanism that ensures API parity across all 10+ bindings while minimizing test maintenance burden.

## Decision

We will implement a fixture-driven testing strategy where:

1. **Shared fixtures:** Test fixtures (input data, expected outputs, error conditions) are defined in a language-neutral format (JSON/YAML)
1. **Fixture generation:** A fixture generator reads these definitions and produces language-specific test cases
1. **Identical test scenarios:** Every binding runs the exact same test cases, just expressed in the native test framework
1. **Automated verification:** CI/CD automatically runs all bindings against all fixtures, ensuring API parity

Fixtures will cover:

- **Happy path tests:** Valid inputs with expected outputs
- **Error scenarios:** Invalid inputs, boundary conditions, error messages
- **Edge cases:** Large inputs, special characters, Unicode, empty values
- **Performance tests:** Baseline performance expectations across bindings
- **State management:** Multi-step operations, state transitions

Example fixture structure:

```yaml
- name: "parse_simple_rule"
  description: "Parse a basic rule definition"
  input:
    rule_string: "IF condition THEN action"
  expected_output:
    type: "rule"
    condition: "condition"
    action: "action"
  language_overrides:
    java: {expected_class: "Rule"}
```

## Consequences

### Positive

- **Guaranteed API parity:** All bindings verify identical behavior against the same test suite
- **Reduced test maintenance:** Tests are defined once, used across all bindings
- **Comprehensive coverage:** Shared fixtures can be more thorough than language-specific tests
- **Regression prevention:** Catch inconsistencies immediately in CI/CD
- **Documentation as tests:** Fixtures serve as executable documentation of API behavior
- **Scalable:** Adding new bindings is easier; they inherit the entire test suite

### Negative

- **Fixture generation overhead:** Building the fixture generation infrastructure requires effort
- **Language-specific customization:** Some tests may need language-specific assertions
- **Fixture maintenance:** Fixtures must be kept in sync with API changes
- **Learning curve:** Teams must learn the fixture format and generation process
- **Debugging complexity:** Failures might require understanding the fixture generator and the binding implementation

### Fixture Format

Fixtures are stored in `/fixtures/` with this structure:

```
fixtures/
├── parsing/
│   ├── basic_rules.yaml
│   ├── complex_rules.yaml
│   └── error_cases.yaml
├── execution/
│   ├── rule_evaluation.yaml
│   └── state_management.yaml
└── integration/
    └── end_to_end.yaml
```

### Fixture Generator

The fixture generator (written in Rust) will:

1. Read fixture definitions from YAML/JSON
1. Generate language-specific test code for each binding
1. Support custom assertions for language-specific types
1. Generate CI/CD test matrices
1. Produce test reports showing parity across bindings

### Language-Specific Assertions

Each binding implements a fixtures helper that knows how to:

- Construct expected values using language idioms
- Compare results with language-native equality
- Format error messages appropriately
- Handle type conversions

### Continuous Verification

- All fixtures run on every commit
- CI/CD maintains a parity matrix showing which bindings pass which fixtures
- Failed fixtures block merging to main branch
- Monthly comprehensive fixture test runs across all platforms
