# AI-Rulez Domains

## Overview

The AI-Rulez domain system organizes agents, skills, and rules into cohesive functional areas. Each domain represents a major aspect of polyglot Rust library development and manages specific responsibilities within the architecture.

## Domain Architecture

```
.ai-rulez/domains/
├── rust-core/              # Rust core library development
├── ffi-bindings/           # C-compatible FFI layer
├── language-bindings/      # Language-specific bindings (Python/TS/Ruby/PHP/Java/Go/C#/Elixir/WASM)
├── build-distribution/     # Build orchestration & packaging
├── quality-verification/   # Testing, CI/CD, code review
├── documentation/          # Docs, API references, tutorials
├── devops-infrastructure/  # CI/CD workflows, deployment
└── organizational/         # Governance, standards, principles
```

## Core Principles

### Rust-First Architecture

All extraction logic lives in the Rust core library. Language bindings are thin wrappers that expose language-idiomatic APIs without duplicating business logic.

### Domain Interactions

Domains collaborate through well-defined interfaces:

- **rust-core** provides the foundation for **ffi-bindings** and **language-bindings**
- **build-distribution** orchestrates compilation across all domains
- **quality-verification** validates outputs from all domains
- **documentation** describes APIs from rust-core and language-bindings
- **devops-infrastructure** deploys artifacts from build-distribution
- **organizational** governs standards across all domains

### Agent Assignment

Each domain references specific agents responsible for its scope:

- Language binding engineers work within **language-bindings** domain
- Rust core engineer works within **rust-core** domain
- Polyglot architect coordinates across **organizational** domain
- Test automation engineer operates in **quality-verification** domain

## Using Domains

### Domain Files

Each domain contains:

- `DOMAIN.md` - Purpose, scope, responsibilities, interaction points
- `agents.yaml` - Referenced agents (not duplicated, just listed)
- `skills.yaml` - Referenced skills (not duplicated, just listed)
- `rules.yaml` - Referenced rules (not duplicated, just listed)

### Profiles

Profiles combine domains for specific project types. See `.ai-rulez/profiles/` for:

- `full-polyglot.yaml` - All domains, all languages
- `rust-library.yaml` - Rust-only projects
- `web-framework.yaml` - HTTP frameworks like Spikard
- `conversion-library.yaml` - Conversion tools like html-to-markdown

## Contributing

When adding new agents, skills, or rules:

1. Identify which domain(s) they belong to
1. Update the relevant domain's YAML files
1. Update interaction points if cross-domain dependencies change
1. Maintain the principle: rust-core is single source of truth
