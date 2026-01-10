# html-to-markdown-bindings-common

Shared utilities for html-to-markdown language bindings.

This crate provides common functionality used across all language bindings (Python, Node.js, Ruby, PHP, WebAssembly) to reduce code duplication and ensure consistent behavior.

## Features

- **Enum Wrappers**: Serde-compatible enum wrappers for all configuration types
- **Error Mapping**: Unified error mapping patterns for binding-specific error types
- **JSON Parsing**: Reusable JSON parsing helpers for configuration objects
- **Metadata Conversion**: Intermediate representations for metadata types
- **Inline Images**: Intermediate representations for inline image extraction

## Usage

This crate is intended to be used by language binding crates, not directly by end users.

See the main [html-to-markdown](https://github.com/kreuzberg-dev/html-to-markdown) repository for usage examples.
