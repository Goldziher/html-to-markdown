# AI-Rulez

Comprehensive AI-powered development governance and configuration for polyglot Rust-based projects with multi-language bindings. This shared module provides AI agents, development rules, skills, and profiles to accelerate polyglot development workflows across Rust, Python, TypeScript, Ruby, PHP, Java, Go, C#, Elixir, and WebAssembly.

## Overview

AI-Rulez is a reusable configuration module designed for projects that need consistent, AI-assisted development practices across multiple languages. It defines:

- **AI Agents** - Specialized roles for architecture, core development, language binding engineering, documentation, and QA
- **Development Rules** - Language-specific standards, patterns, and quality gates
- **Skills** - Reusable AI capabilities for code generation, testing, documentation, and domain-specific tasks
- **Profiles** - Pre-configured model routing and preset combinations for different development scenarios
- **Domains** - Logical groupings of agents, rules, and skills by functional area

This is a **shared module** meant to be included in other projects via git includes, not a standalone application.

## Quick Start

### Including AI-Rulez in Your Project

Add this repository as an include in your project's `.claude.yaml` or `claude.yaml` configuration:

#### Using HTTPS

```yaml
# claude.yaml (or .claude.yaml)
include:
  - path: https://github.com/kreuzberg-dev/ai-rulez.git
    type: github
    version: main  # or specific tag/commit
```

#### Using SSH

```yaml
# claude.yaml (or .claude.yaml)
include:
  - path: git@github.com:kreuzberg-dev/ai-rulez.git
    type: github
    version: main  # or specific tag/commit
```

#### Manual Installation

Clone the repository as a subdirectory in your project:

```bash
git clone https://github.com/kreuzberg-dev/ai-rulez.git .ai-rulez
```

Then reference it in your configuration:

```yaml
# claude.yaml
include:
  - path: ./.ai-rulez
    type: local
```

## Repository Structure

### Top-Level Configuration

- **`config.yaml`** - Master configuration file with module metadata, presets, model routing, quality gates, and profiles
- **`mcp.yaml`** - Model Context Protocol (MCP) server configurations for GitHub, filesystem, cargo registry, and web search integrations

### Directories

#### `agents/` - AI Agent Definitions

Specialized role definitions for different tasks. Each agent is a markdown file describing responsibilities, expertise, and guiding principles.

**Key agents:**

- `polyglot-architect.md` - System design and multi-language architecture (uses Sonnet)
- `rust-core-engineer.md` - Rust core library development (uses Haiku)
- `python-bindings-engineer.md`, `typescript-bindings-engineer.md`, etc. - Language-specific binding development (use Haiku)
- `code-reviewer.md` - Multi-language code review (uses Haiku)
- `docs-writer.md` - User guide and documentation authoring (uses Haiku)
- `api-doc-writer.md` - API reference documentation (uses Haiku)
- `tutorial-writer.md` - Language-parity tutorial creation (uses Haiku)
- `test-automation-engineer.md` - End-to-end test generation (uses Haiku)

#### `rules/` - Development Rules

Language-specific standards, patterns, and quality requirements. Organized by language and functional domain.

Examples:

- `rust-2024-edition-core-conversion-engine/` - Rust 2024 Edition standards for core development
- `python-310-pyo3-binding-wrappers/` - Python 3.10+ with PyO3 FFI binding standards
- `typescript-5x-napi-rs-bindings-cli/` - TypeScript 5.x with NAPI-RS binding standards
- `polyglot-build-system-distribution/` - Cross-language build and distribution patterns
- `code-quality/` - General code quality standards

#### `context/` - Project Context

Reusable context files describing project structure, architecture patterns, and domain knowledge for AI consumption.

#### `skills/` - AI Skills

Reusable AI capabilities and specialized tasks. Skills are modular, composable units that can be invoked by agents.

Examples:

- Code generation templates
- Documentation generation patterns
- Testing strategies
- Build and deployment automation
- Architecture decision recording (ADR) generation

#### `profiles/` - Development Profiles

Pre-configured combinations of agents, rules, skills, and model routing for specific development scenarios.

**Available profiles:**

- **`full-polyglot.yaml`** - Complete tooling for all languages with full feature parity. Uses Claude Opus/Sonnet for architecture and Haiku for implementation.
- **`rust-first.yaml`** - Optimized for primary Rust development with focus on core library.
- **`python-focused.yaml`** - Tailored for Python-primary projects with Python and Rust support.
- **`typescript-focused.yaml`** - Tailored for TypeScript-primary projects with TypeScript and Rust support.
- **`web-framework.yaml`** - Web framework development profile with frontend and backend optimization.
- **`conversion-library.yaml`** - For data conversion and serialization library projects.
- **`extraction-library.yaml`** - For document and data extraction library projects.

#### `domains/` - Functional Domains

Logical groupings of agents, rules, and skills by functional area.

**Available domains:**

- **`rust-core`** - Rust core library development
- **`language-bindings`** - Multi-language FFI binding development
- **`ffi-bindings`** - FFI (Foreign Function Interface) specific patterns and tools
- **`quality-verification`** - Testing, QA, and verification strategies
- **`documentation`** - Documentation generation and content authoring
- **`build-distribution`** - Build systems, packaging, and distribution
- **`devops-infrastructure`** - CI/CD, infrastructure, and deployment automation
- **`organizational`** - Project organization, versioning, and release management

## Configuration Reference

### Model Routing Strategy

The default configuration routes different task types to optimal models:

| Task Type | Model | Roles | Rationale |
|-----------|-------|-------|-----------|
| Architecture | Claude Sonnet 4.5 | polyglot-architect, release-coordinator | Complex polyglot system design |
| Implementation | Claude Haiku 4.5 | binding-engineer-\*, rust-core-engineer | Fast, accurate code generation |
| Code Review | Claude Haiku 4.5 | code-reviewer, quality-assurance | Efficient review and validation |
| Documentation | Claude Haiku 4.5 | documentation-engineer, example-generator | Clear, concise documentation |

### Available Models

**Claude:**

- `claude-opus-4-5` - Complex reasoning for primary architecture
- `claude-sonnet-4-5` - Architecture and system design decisions
- `claude-haiku-4-5` - Fast implementation and code generation

**Gemini:**

- `gemini-2.0-flash` - Supplementary implementation tasks

**OpenAI Codex:**

- `code-davinci-003` - Legacy code completion and generation

### Quality Gates

All projects using this module must meet these quality standards:

- **Rust Coverage Minimum:** 95% code coverage
- **Binding Coverage Minimum:** 80% code coverage (language-specific constraints)
- **Test Types:** Unit, integration, documentation, and end-to-end tests
- **Language Parity:** All supported languages must have feature parity
- **Documentation:** All public APIs must be documented

### Version Synchronization

This module uses a monorepo versioning strategy:

- **Source of Truth:** `Cargo.toml`
- **Sync Targets:** pyproject.toml, package.json, Gemfile, composer.json, pom.xml, go.mod, .csproj, mix.exs

Version changes in Cargo.toml automatically propagate to all language-specific package manifests.

## Using Profiles

Select a profile based on your project's primary language and scope:

### For Full Polyglot Projects

Use `full-polyglot` profile:

```yaml
# config.yaml in your project
extends:
  - path: .ai-rulez
    profile: full-polyglot
```

This activates all domains and agents for comprehensive polyglot development.

### For Rust-First Projects

Use `rust-first` profile:

```yaml
extends:
  - path: .ai-rulez
    profile: rust-first
```

Focuses on Rust core development with Sonnet for architecture decisions.

### For Lightweight/Rapid Iteration

Use `lightweight` profile:

```yaml
extends:
  - path: .ai-rulez
    profile: lightweight
```

Uses Haiku for all tasks, optimizing for speed and cost. Supports Rust, Python, and TypeScript.

## Extension Points

Projects using this module can extend it with custom configurations:

### Custom Agents

Define project-specific agents in `.ai-rulez/custom-agents.yaml`:

```yaml
agents:
  my-custom-agent:
    name: "My Custom Agent"
    description: "Project-specific AI role"
    model: "claude-haiku-4-5"
```

### Custom Rules

Add project-specific rules in `.ai-rulez/custom-rules.yaml`:

```yaml
rules:
  my-project-specific-rule:
    description: "Custom development standard"
```

### Custom Skills

Create `.ai-rulez/custom-skills/` directory for project-specific AI capabilities.

### Project Profiles

Define tailored profiles in `.ai-rulez/project-profiles.yaml`:

```yaml
profiles:
  my-project-profile:
    presets: [claude]
    languages: [rust, python]
    agents:
      - rust-core-engineer
      - python-bindings-engineer
```

## Common Workflows

### Initialize a New Polyglot Project

1. Include this module in your project configuration
1. Select the `full-polyglot` profile
1. Use the `polyglot-architect` agent for initial architecture decisions
1. Invoke language-specific binding engineers for each language target

### Code Review Across Languages

Use the `code-reviewer` agent with the full-polyglot profile to review code in any supported language. The agent applies language-specific rules and quality standards.

### Generate Documentation

Use the `docs-writer` or `api-doc-writer` agents to generate user guides and API documentation respectively. The configuration ensures documentation covers all supported languages with code examples.

### Create Language-Parity Tutorials

Use the `tutorial-writer` agent to generate tutorials that cover all supported languages. The agent ensures equivalent examples and explanations across languages.

### Manage Multi-Language Releases

Use the `release-versioning-coordinator` agent to:

1. Update version in Cargo.toml (source of truth)
1. Automatically sync versions to all language-specific manifests
1. Generate release notes covering all language bindings
1. Coordinate release across all platforms

## MCP Server Integrations

This module is configured with Model Context Protocol (MCP) servers for enhanced capabilities:

### GitHub Integration

Enable PR reviews, issue tracking, and release management through the `code-reviewer` agent.

**Setup:**

```bash
export GITHUB_TOKEN=your_github_token
```

### Filesystem Access

Provide AI agents with code reading and analysis capabilities for architecture exploration and binding verification.

### Cargo Registry Integration

Enable dependency auditing and security scanning for Rust dependencies.

### Web Search

Support for researching FFI patterns, language binding best practices, and Rust ecosystem standards.

## Documentation Files

- **`CUSTOMIZATION.md`** - Detailed guide for customizing agents, rules, skills, and profiles
- **`PROFILES.md`** - In-depth profile documentation with examples and use cases

## Supported Languages

**Required:**

- Rust (core library)

**Optional (with full feature parity):**

- Python 3.10+
- TypeScript 5.x
- Ruby 3.2+
- PHP 8.2+
- Java 17+
- Go 1.21+
- C# (.NET 8+)
- Elixir 1.14+
- WebAssembly

## Version Information

- **Config Version:** 3.0
- **Module Version:** 1.0.0
- **Scope:** polyglot-rust
- **Type:** shared

## Schema Validation

The configuration files use JSON schemas for IDE validation:

- Config schema: `https://raw.githubusercontent.com/Goldziher/ai-rulez/main/schema/ai-rules-v3.schema.json`
- MCP schema: `https://raw.githubusercontent.com/Goldziher/ai-rulez/main/schema/ai-rules-v3-mcp.schema.json`

## Best Practices

### Maintain Feature Parity

Ensure all supported languages have equivalent APIs and behavior. Use the `polyglot-architect` agent to design FFI boundaries that work well across all target languages.

### Respect Rust as Source of Truth

All core functionality lives in Rust. Language bindings are thin wrappers that expose language-idiomatic APIs. New features are implemented in Rust first, then exposed through bindings.

### Run Quality Gates

Before releases, verify:

- Rust code meets 95% coverage
- Binding code meets 80% coverage
- All test types pass (unit, integration, doc, e2e)
- All public APIs are documented
- All supported languages have feature parity

### Use Model Routing Strategically

Use Sonnet for complex architecture decisions and Haiku for implementation. This balance optimizes for quality and cost.

### Version Synchronization

When updating versions:

1. Update `Cargo.toml` (source of truth)
1. Verify all sync targets are updated
1. Use the `release-versioning-coordinator` agent to coordinate releases across languages

## Contributing

To extend or customize AI-Rulez for your project:

1. Review `CUSTOMIZATION.md` for detailed guidance
1. Create custom agents, rules, and skills in `.ai-rulez/custom-*` directories
1. Define project-specific profiles in `.ai-rulez/project-profiles.yaml`
1. Test with your AI tooling to ensure proper integration

## License

See the LICENSE file in this repository for licensing information.

## Additional Resources

- [CUSTOMIZATION.md](/Users/naamanhirschfeld/workspace/kreuzberg-dev/ai-rulez/CUSTOMIZATION.md) - Detailed customization guide
- [PROFILES.md](/Users/naamanhirschfeld/workspace/kreuzberg-dev/ai-rulez/PROFILES.md) - Complete profile documentation
- [config.yaml](/Users/naamanhirschfeld/workspace/kreuzberg-dev/ai-rulez/config.yaml) - Master configuration reference
- [mcp.yaml](/Users/naamanhirschfeld/workspace/kreuzberg-dev/ai-rulez/mcp.yaml) - MCP server configuration

## Support

For questions or issues with AI-Rulez, please refer to the documentation files or create an issue in the repository.

______________________________________________________________________

**Built for Kreuzberg polyglot projects.** Use this module to bring consistent AI-assisted development practices to your multi-language codebase.
