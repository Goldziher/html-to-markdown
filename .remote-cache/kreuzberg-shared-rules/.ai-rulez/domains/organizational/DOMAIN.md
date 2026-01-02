# Organizational Domain

## Purpose

The organizational domain establishes governance, architectural principles, and cross-cutting standards for the entire polyglot project. It ensures consistency, clarity, and alignment across language ecosystems, tool choices, and development practices.

## Scope and Responsibilities

- Define core architectural principles: Rust-first design, language-idiomatic bindings, no logic duplication across languages
- Establish polyglot multi-language FFI strategy: Rust core is single source of truth; all bindings are thin wrappers
- Coordinate cross-language design decisions and architectural tradeoffs
- Define agent selection guidelines for appropriate expert assignment by domain/language
- Establish model routing rules (Sonnet 4.5 for strategic architectural decisions, Haiku 4.5 for all engineering tasks)
- Enforce universal anti-patterns and language-specific anti-patterns across all code
- Establish git and commit standards (Conventional Commits 1.0.0, branch protection, pre-commit hooks)
- Maintain code quality standards applicable across all languages
- Document language parity requirements (10 supported languages: Rust, Python, TypeScript, Ruby, Java, Go, C#, Elixir, PHP, WASM)
- Establish dependency minimization principles and audit requirements
- Define common task commands and workflows for developer consistency
- Coordinate version synchronization across all language package managers
- Manage documentation language parity enforcement

## Referenced Agents

- **polyglot-architect**: System design and multi-language architecture decisions. Expertise in FFI design, version syncing, E2E test generation, cross-platform compatibility. Uses Sonnet 4.5 for strategic decisions.

## Referenced Skills

- **agent-selection-usage-guidelines**: Rules for assigning agents (rust-core-engineer for Rust, language-bindings engineers for language-specific work, etc.)
- **model-routing**: Sonnet 4.5 for architectural decisions, Haiku 4.5 for implementation and integration issues
- **git-commit-standards**: Conventional Commits 1.0.0 (feat/fix/docs/refactor/test/chore with scope), NO AI signatures, branch protection with status checks
- **git-standards**: Pre-commit hooks (prek/lefthook/husky), linting, formatting, test execution before commits
- **universal-anti-patterns**: Patterns to avoid across all languages
- **anti-patterns**: Language-specific pitfalls and design anti-patterns
- **core-principles**: Do only what's asked, minimize file creation, prefer editing, no proactive documentation, Rust core for all logic, bindings for language-idiomatic APIs only
- **common-task-commands**: Standard task commands (task setup, task build, task test, task lint, task format, task cov:all)

## Referenced Rules

- **code-quality**: Descriptive variable names, complex logic comments, unit tests, \<50 line functions, explicit error handling

## Interaction Points

- **Provides to**: All domains (governance, standards, architectural guidance)
- **Receives from**: All domains (feedback, architectural evolution, standards updates)
- **Coordinates with**: All domains for standards compliance and consistency

## Critical Files This Domain Manages

- `.ai-rulez/agents/` (Agent role definitions and expertise models)
- `.ai-rulez/skills/` (Domain expertise and capability definitions)
- `.ai-rulez/rules/` (Project-wide rules and standards)
- `.ai-rulez/context/architecture.md` (High-level architecture documentation)
- `.ai-rulez/domains/` (Domain definitions and interactions)
- `.git/hooks/` or `lefthook.yaml` (Git pre-commit hook configuration)
- `scripts/sync_versions.py` (Cross-language version synchronization)
