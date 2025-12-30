# WebAssembly Test App for html-to-markdown

Tests the published html-to-markdown-wasm package from npm.

## Setup

```bash
pnpm install
```

## Run Tests

```bash
# Smoke tests (fast, basic functionality)
pnpm test:smoke

# Comprehensive tests (fixture-driven, edge cases)
pnpm test:comprehensive

# All tests
pnpm test
```

## About

This test app validates the WASM package works correctly after publishing to npm. It includes:

- **Smoke tests**: Basic import, conversion, and error handling tests
- **Comprehensive tests**: Fixture-driven tests using shared test data in `../fixtures/`
- **Edge case tests**: Special characters, nested HTML, mixed content, and whitespace handling

The tests run via vitest and validate both the WASM module initialization and the conversion functionality.
