______________________________________________________________________

## priority: medium

# Polyglot Binding Architecture

## Language Binding Pattern

Each binding crate (Python, TypeScript, Ruby, PHP, Go, Java, C#) follows:

1. **Minimal wrapper layer**: Call Rust functions directly, no business logic
1. **Type translation**: Convert host language types ↔ Rust types
1. **Error mapping**: Rust errors → language-native exceptions
1. **Documentation**: Link bindings to Rust docs, add language-specific examples
1. **Testing**: Language-native test suite validating binding + integration

## Binding Crates (crates/)

- **html-to-markdown-py**: PyO3 bindings → packages/python distribution
- **html-to-markdown-node**: NAPI-RS bindings → packages/typescript npm package
- **html-to-markdown-rb**: Magnus bindings → packages/ruby gem (Ruby 3.2+)
- **html-to-markdown-php**: ext-php-rs extension → packages/php Composer package
- **html-to-markdown-wasm**: wasm-bindgen → browser + Wasmtime targets
- **html-to-markdown-ffi**: C-compatible FFI library → Go, Java, C# consumers
- **html-to-markdown-cli**: Standalone CLI using core library

## Distribution Packages (packages/)

- **python/**: PyPI package with Python wrappers + tests
- **typescript/**: npm package with TypeScript wrappers + CLI + tests
- **ruby/**: Ruby gem (RBS types in sig/, specs in spec/)
- **php/**: Composer package with PHP wrappers + PHPUnit tests
- **php-ext/**: PIE metadata for ext-php-rs distribution
- **go/**: Go module wrapping FFI library
- **java/**: Maven project wrapping FFI library with JNI
- **csharp/**: .NET project wrapping FFI library with P/Invoke
