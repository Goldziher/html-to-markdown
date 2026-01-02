______________________________________________________________________

## priority: critical

# Documentation Standards & Language Parity

**CRITICAL: Full language parity required**. ALL documentation, guides, and code examples MUST include snippets for ALL 7 supported languages: Rust, Python, TypeScript, Ruby, Java, Go, C#.

**Code snippet structure**:

- Location: docs/snippets/{language}/{category}/{filename}.{ext}
- Categories: getting-started, config, advanced, ocr, metadata, plugins, api, mcp, cli, utils, cache, docker, benchmarking
- Use MkDocs pymdownx.tabbed extension for multi-language code blocks
- Include snippets with --8\<-- syntax: `--8<-- "docs/snippets/python/config/basic.py"`

**Snippet requirements**:

- Keep concise (10-40 lines per snippet)
- No comments in snippets (documentation provides context)
- Verify API accuracy by reading source code before writing
- Test all snippets for correctness
- Each language must have equivalent functionality shown

**Language-specific inline documentation**:

- **Rust**: /// doc comments on ALL public items, SAFETY comments for unsafe, ~keep suffix for error handling, examples as doctests
- **Python**: NO docstrings in private/test files, public API only, Google style format
- **TypeScript**: JSDoc with @param/@returns/@example on ALL exports
- **Java**: Javadoc on ALL public classes/methods with @param/@return/@throws/@since
- **Go**: Package doc.go files, inline comments following Go conventions
- **Ruby**: YARD documentation with @param/@return tags
- **C#**: XML doc comments with <summary>, <param>, <returns>

**General rules**:

- Code comments explain "why" not "what"
- No proactive README/documentation creation - only when requested
- No AI signatures in any documentation
- Cross-reference related APIs across languages
- Update ALL language snippets when APIs change
