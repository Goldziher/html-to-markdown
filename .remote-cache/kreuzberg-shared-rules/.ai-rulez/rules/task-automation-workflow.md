______________________________________________________________________

## priority: medium

# Task Automation & Workflow

**Taskfile.yaml for all workflows · setup → build → test → lint**

- Taskfile.yaml primary interface for all development tasks
- task setup: install all language deps (Python, Ruby, PHP, JS, Go, Java, C#, Elixir)
- task build: compile Rust core + JavaScript bindings
- task test: run all language test suites (pytest, cargo test, rspec, vitest, phpunit)
- task lint: ruff + clippy + phpstan + rubocop + golangci-lint + more
- task format: ruff fix + cargo fmt + rubocop --autocorrect + biome + phpcbf
- task cov:all: generate Rust + Python coverage reports (lcov format)
- task bench: benchmark harness via tools/benchmark-harness
- Environment variables in Taskfile (RUST_LOG, RUBY_BIN, BUNDLER_VERSION)
- Never: manual commands instead of task tasks
