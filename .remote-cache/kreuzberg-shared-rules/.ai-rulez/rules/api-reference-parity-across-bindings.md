______________________________________________________________________

## priority: high

# API Reference Parity Across Bindings

**Documentation parity enforcement · 10 language examples · rustdoc + Sphinx + TypeDoc + YARD · API mapping YAML**

- Documentation parity requirement: Every public API documented in all supported bindings (Rust, Python, TypeScript, Ruby, PHP, Go, Java, C#, Elixir, WASM)
- Documentation tools: rustdoc (Rust), Sphinx (Python), TypeDoc (TypeScript), YARD (Ruby), PHPDoc/Sphinx (PHP), godoc (Go), Javadoc (Java), XML docs (C#), ExDoc (Elixir), JSDoc (WASM)
- Code examples: Minimum 2 examples per API in each supported language; examples include error handling and edge cases
- API mapping file: docs/api-mapping.yaml defines canonical API surface with language-specific signatures and parameter mappings
- Language-specific paths: Docs generated to docs/{language}/ with consistent structure across all bindings
- Cross-reference links: All binding docs cross-link to canonical Rust API definition; mapping file provides translation
- Breaking change docs: docs/MIGRATIONS.md specifies version-to-version API changes with language-specific migration steps
- CI verification: task verify:docs checks all binding docs exist, contain examples, match API mapping, reference canonical docs
- Example testing: All code examples in docs are runnable tests; ci-docs-examples validates snippets compile/run
- Migration guides: Mandatory for breaking changes; include before/after snippets for all 10 languages
- Agent references: docs-writer owns documentation consistency; polyglot-architect owns API mapping and binding parity
- Documentation review: All API additions require docs-writer approval; binding docs must be completed before merge
- Never: APIs in binding without corresponding docs, undocumented parameters, missing migration guides, language-specific docs diverging from canonical source
