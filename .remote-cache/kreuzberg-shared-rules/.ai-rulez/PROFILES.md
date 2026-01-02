# AI-Rulez Profiles Guide

AI-Rulez provides seven pre-configured profiles that optimize AI assistance for different project types and development contexts. Each profile combines specific domains, agents, and model routing strategies tailored to your needs.

## Available Profiles

### 1. Full Polyglot Library

**When to use:** Complete polyglot libraries with Rust core and bindings for 10 languages.

**Best for:** Projects that need to support all available language ecosystems equally, such as comprehensive data processing libraries, cryptography libraries, or performance-critical utilities that serve multiple language communities.

**Includes:**

- All 8 domains: rust-core, ffi-bindings, language-bindings, build-distribution, quality-verification, documentation, devops-infrastructure, organizational
- 15 specialized agents covering Rust, Python, TypeScript, Ruby, PHP, Java, Go, C#, and Elixir
- Full model routing with Claude Sonnet for architecture and Haiku for implementation
- Bindings for: Python, TypeScript, Ruby, PHP, Java, Go, C#, Elixir, WASM

**Example projects:**

- `kreuzberg` (document extraction library)
- `html-to-markdown` (conversion library)
- High-performance crypto or numerical computation libraries

______________________________________________________________________

### 2. Rust-Library

**When to use:** Pure Rust libraries without language bindings.

**Best for:** Projects focused exclusively on Rust development, such as:

- Core libraries with no need for multi-language support
- Performance-critical algorithms
- Rust ecosystem tools and utilities
- Projects prioritizing zero-copy semantics and memory safety

**Includes:**

- 5 focused domains: rust-core, quality-verification, documentation, devops-infrastructure, organizational
- 5 essential agents: rust-core-engineer, test-automation-engineer, code-reviewer, docs-writer, api-doc-writer
- Haiku-only model routing for fast iteration
- No binding engineers or multi-language agents

**Example projects:**

- Tokio runtime components
- Rust standard library extensions
- Developer tools like `rustfmt` or `clippy`

______________________________________________________________________

### 3. Python-Focused Library

**When to use:** Rust core with Python bindings as the primary interface.

**Best for:** Projects serving the Python ecosystem, particularly:

- Machine learning and data science libraries using PyO3
- Scientific computing with high-performance Rust cores
- Data processing tools where Python is the dominant user interface
- Projects targeting data scientists and ML engineers

**Includes:**

- 7 domains: rust-core, language-bindings, build-distribution, quality-verification, documentation, devops-infrastructure, organizational
- 6 agents: rust-core-engineer, python-bindings-engineer, test-automation-engineer, code-reviewer, docs-writer, api-doc-writer
- Priority rules for PyO3 bindings and pytest integration
- Focuses on Python idioms and documentation (docstrings, guide-style tutorials)

**Example projects:**

- NumPy-like acceleration libraries
- ML inference engines with Python APIs
- Scientific data processors

______________________________________________________________________

### 4. TypeScript-Focused Library

**When to use:** Rust core with TypeScript/Node.js and WebAssembly bindings.

**Best for:** Projects targeting the JavaScript/web ecosystem, including:

- High-performance Node.js libraries using NAPI-RS
- WebAssembly modules for browser-side computation
- Web frameworks and performance-critical tools
- Projects serving JavaScript developers

**Includes:**

- 7 domains: rust-core, language-bindings, build-distribution, quality-verification, documentation, devops-infrastructure, organizational
- 6 agents: rust-core-engineer, typescript-bindings-engineer, test-automation-engineer, code-reviewer, docs-writer, api-doc-writer
- Priority rules for NAPI-RS and wasm-bindgen integration
- Focus on TypeScript idioms, JSDoc, and Vitest integration

**Example projects:**

- High-performance JavaScript libraries
- WebAssembly modules for browsers
- Node.js native add-ons

______________________________________________________________________

### 5. Conversion-Library

**When to use:** Conversion/transformation tools with Rust core and bindings for all 10 languages.

**Best for:** Document processors, format converters, and transformation engines that need widespread language support:

- HTML to Markdown converters (like `html-to-markdown`)
- Document format converters
- Data transformation pipelines
- Any library centered around converting between formats

**Includes:**

- All 8 domains: rust-core, ffi-bindings, language-bindings, build-distribution, quality-verification, documentation, devops-infrastructure, organizational
- 12 agents including polyglot-architect, rust-core-engineer, binding engineers for all languages, code-reviewer, and documentation specialists
- Emphasis on conversion-specific architecture and API parity across languages
- End-to-end conversion testing across all language bindings

**Example projects:**

- `html-to-markdown`: Convert HTML to Markdown
- Format converters (JSON, XML, YAML, etc.)
- Data transformation libraries

______________________________________________________________________

### 6. Extraction-Library

**When to use:** Complex document extraction systems with plugin architecture and bindings for all 10 languages.

**Best for:** Advanced data extraction platforms with extensibility requirements:

- Document processors with OCR, ML, and metadata extraction (`kreuzberg`)
- Pluggable extraction engines
- Complex document analysis systems
- Projects requiring plugin safety and FFI boundaries

**Includes:**

- All 8 domains with emphasis on ffi-bindings and language-bindings
- 12 agents specialized for extraction architecture and plugin systems
- Focus on plugin API safety and FFI boundary design
- Fixture-driven testing for complex extraction scenarios
- Specialized code review for plugin safety considerations

**Example projects:**

- `kreuzberg`: Advanced document extraction with PDF, Office, and OCR support
- Pluggable data extraction platforms
- Document analysis systems with ML capabilities

______________________________________________________________________

### 7. Web-Framework

**When to use:** HTTP frameworks with Rust core and bindings for web ecosystem languages.

**Best for:** Web frameworks and HTTP libraries targeting multiple languages:

- HTTP server frameworks (like `spikard`)
- Web framework libraries
- Cross-language web tooling
- Projects integrating with popular web frameworks (Django, FastAPI, Express, Rails, Laravel)

**Includes:**

- All 8 domains: rust-core, ffi-bindings, language-bindings, build-distribution, quality-verification, documentation, devops-infrastructure, organizational
- 11 agents: polyglot-architect, rust-core-engineer, binding engineers for Python/TypeScript/Ruby/PHP, test-automation-engineer, code-reviewer, and documentation specialists
- Emphasis on framework architecture and integration patterns
- End-to-end HTTP testing across all language bindings

**Example projects:**

- `spikard`: HTTP framework with polyglot support
- Web server performance accelerators
- Cross-language HTTP middleware libraries

______________________________________________________________________

## Profile Selection Quick Reference

| Profile | Best For | Core Languages | Binding Languages |
|---------|----------|-----------------|------------------|
| Full Polyglot | Universal support | Rust | Python, TS, Ruby, PHP, Java, Go, C#, Elixir, WASM |
| Rust-Library | Rust-only | Rust | None |
| Python-Focused | ML/Data Science | Rust | Python |
| TypeScript-Focused | Web/Node.js | Rust | TypeScript, WASM |
| Conversion-Library | Format Conversion | Rust | Python, TS, Ruby, PHP, Java, Go, C#, Elixir, WASM |
| Extraction-Library | Document Extraction | Rust | Python, TS, Ruby, PHP, Java, Go, C#, Elixir, WASM |
| Web-Framework | Web Frameworks | Rust | Python, TypeScript, Ruby, PHP |

## How to Use Profiles

### 1. Set Default Profile

Edit `config.yaml`:

```yaml
default_profile: "python-focused"
```

### 2. Create Project-Specific Profiles

Create `.ai-rulez/project-profiles.yaml` in your consuming project:

```yaml
profiles:
  my-custom-profile:
    extends: "conversion-library"
    agents:
      - my-custom-agent
    priority_rules:
      - my-custom-rule
```

### 3. Switch Profiles at Runtime

When using Claude Code with ai-rulez integration, specify the profile:

```bash
claude-code --profile=python-focused
```

## Model Routing Within Profiles

Each profile configures how different types of tasks are routed to different Claude models:

- **Architecture decisions** → Claude Sonnet 4.5 (complex reasoning)
- **Core implementation** → Claude Haiku 4.5 (fast, accurate coding)
- **Testing** → Claude Haiku 4.5 (efficient test generation)
- **Code review** → Claude Haiku 4.5 (quick quality checks)
- **Documentation** → Claude Haiku 4.5 (clear, concise writing)

This ensures optimal cost, speed, and quality for each type of work.

## Customizing Profiles

See [CUSTOMIZATION.md](./CUSTOMIZATION.md) for guidance on extending and customizing profiles for your specific project needs.
