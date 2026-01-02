# Documentation Domain

## Purpose

The documentation domain provides comprehensive, multi-language technical documentation that ensures all users—regardless of their language ecosystem—have equal access to guides, API references, tutorials, and best practices. Documentation maintains full language parity across all supported languages.

## Scope and Responsibilities

- Author user guides, API documentation, tutorials, and troubleshooting resources
- Ensure language parity: every guide and example includes code snippets for ALL supported languages
- Maintain MkDocs documentation infrastructure with pymdownx.tabbed for multi-language code blocks
- Create code snippet library in docs/snippets/{language}/{category}/{filename}.{ext}
- Write inline API documentation (rustdoc, JSDoc, Javadoc, docstrings, YARD, etc.) per language conventions
- Verify API accuracy by reading actual source code before documenting
- Include practical examples, edge cases, error conditions in inline documentation
- Generate API reference pages from inline documentation
- Develop step-by-step learning guides with progressive complexity (basic → intermediate → advanced)
- Create troubleshooting sections and migration guides
- Document repository structure, commands, and common workflows
- Ensure NO proactive documentation creation—only document when requested

## Referenced Agents

- **docs-writer**: Technical documentation author for guides, README, MkDocs content, code snippets in docs/snippets/
- **api-doc-writer**: Inline API documentation (rustdoc, JSDoc, Javadoc, docstrings). API reference pages. Verify against source code.
- **tutorial-writer**: Step-by-step tutorials and learning paths. Progressive complexity. Runnable code examples. Troubleshooting sections.

## Referenced Skills

- **documentation-standards-language-parity**: Full language parity for ALL supported languages. docs/snippets/ structure with categories. MkDocs pymdownx.tabbed. Language-specific inline doc formats.
- **quick-start**: Getting started guides with minimal focused examples. Essential concepts first, advanced patterns later.
- **repository-structure-commands**: Documentation of directory structure, build commands, test commands, development workflows, contribution guidelines

## Referenced Rules

- None currently (documentation is guided by skills-based standards)

## Interaction Points

- **Receives from**: rust-core domain (API changes), language-bindings domain (binding APIs), quality-verification domain (testing documentation accuracy)
- **Provides to**: users and developers (public documentation), organizational domain (standards dissemination)
- **Coordinates with**: organizational domain for language parity consistency, language-bindings for API documentation accuracy

## Critical Files This Domain Manages

- `docs/` (MkDocs documentation root)
- `docs/snippets/` (Code snippet library organized by language/category)
- `docs/index.md`, `docs/getting-started.md`, `docs/api-reference.md`, `docs/tutorials/`
- `README.md` (Project overview and quick start)
- `crates/*/src/` (Inline rustdoc comments on all public items)
- `packages/python/*/` (Python docstrings on public API)
- `packages/typescript/src/` (TypeScript JSDoc on all exports)
- `packages/ruby/lib/` (Ruby YARD documentation)
