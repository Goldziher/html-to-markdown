______________________________________________________________________

## priority: critical

# Agent Selection & Usage Guidelines

**When to use agents**: Use the Task tool to spawn specialized agents for focused, language-specific work. Agents provide domain expertise and follow language-specific conventions.

**Agent selection rules**:

- **Rust core work** → rust-core-engineer (core library, extraction logic, plugins)
- **Python bindings** → python-bindings-engineer (PyO3 FFI, Python wrappers, EasyOCR/PaddleOCR)
- **TypeScript bindings** → typescript-bindings-engineer (NAPI-RS FFI, TS SDK)
- **Ruby bindings** → ruby-bindings-engineer (Magnus FFI, Ruby gem)
- **Java bindings** → java-bindings-engineer (FFM API, Java wrappers)
- **Go bindings** → go-bindings-engineer (cgo FFI, Go SDK)
- **Testing tasks** → test-automation-engineer (unit/integration/E2E across all languages)
- **Code review** → code-reviewer (quality, security, compliance checks)
- **Architecture/planning** → polyglot-architect (FFI design, multi-language coordination)
- **User guides/tutorials** → docs-writer (user-facing documentation, multi-language examples)
- **API documentation** → api-doc-writer (inline docs, API reference pages)
- **Learning materials** → tutorial-writer (step-by-step guides, getting started content)

**Multi-language tasks**: If work spans multiple languages (e.g., Rust core + bindings), spawn multiple agents in parallel when possible.

**Performance**: All implementation agents use haiku (fast, cost-effective). Only polyglot-architect uses sonnet for strategic planning.

**Agent coordination**: Rust-first principle - core logic goes in rust-core-engineer first, then binding engineers expose through language-idiomatic APIs. Binding engineers should coordinate with rust-core-engineer for shared logic.

**When NOT to use agents**: Simple edits, single-file changes, or tasks you can complete directly without specialized domain knowledge.
