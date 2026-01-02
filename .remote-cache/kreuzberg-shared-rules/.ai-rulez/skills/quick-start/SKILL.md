______________________________________________________________________

## priority: medium

# Developer Quick Start

## Prerequisites

- Rust 1.75+ (stable or nightly for WASM)
- Python 3.10+ with uv package manager
- Node.js 18+ with pnpm ≥10.17
- Ruby 3.2+ with rbenv
- PHP 8.2+ with Composer
- Go 1.25+ (optional, for Go binding)
- Java JDK 22+ (optional, for Java binding)
- .NET 8.0+ SDK (optional, for C# binding)
- Task (task runner)
- prek (pre-commit hook manager)

## Quick Setup

```bash
git clone https://github.com/kreuzberg-dev/your-project.git
cd your-project

# Install all dependencies
task setup

# Install pre-commit hooks
task pre-commit:install
```

## Running Tests

```bash
# All languages
task test

# Specific languages
task test:rust
task test:python
task test:ruby
task test:node
task test:ts
task test:js
task test:php
task test:go

# Coverage
task cov:rust
task cov:python
task cov:all
```

## Development Workflow

```bash
# Build everything
task build

# Format code
task format

# Lint everything
task lint

# Run benchmarks
task bench

# Update dependencies
task update
```

## Editing & Committing

1. Edit source files (Rust, Python, TypeScript, Ruby, PHP, etc.)
1. prek will auto-format on commit
1. If hooks reject, fix issues and retry git commit
1. Never use --no-verify; enforce code quality
