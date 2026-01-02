# DevOps & Infrastructure Domain

## Purpose

The devops-infrastructure domain manages continuous integration, continuous deployment, and infrastructure-as-code to ensure reliable, automated testing and distribution across multiple platforms, languages, and packaging ecosystems.

## Scope and Responsibilities

- Design and maintain GitHub Actions workflows for multi-platform CI/CD
- Implement split CI workflows by domain (ci-rust, ci-python, ci-node, ci-wasm, ci-ruby, ci-php, ci-go, ci-java, ci-validate)
- Enforce linting and formatting checks (ruff, clippy, rubocop, biome, phpstan, golangci-lint)
- Configure test matrix across language versions and OS platforms
- Manage multi-OS testing (Linux amd64/arm64, macOS Intel/ARM, Windows where applicable)
- Generate and publish coverage reports (Rust coverage.lcov, language-specific coverage)
- Enforce quality gates: zero warnings, passing tests, coverage threshold compliance
- Configure artifact builds for distribution validation
- Implement tag-based release triggers for multi-platform builds
- Use task command interface in all CI workflows (never direct script calls)
- Set BUILD_PROFILE=ci for release-optimized binaries with debug symbols
- Manage artifact caching (Cargo, npm, Maven, Go modules)
- Publish packages to registries (PyPI, npm, crates.io, Maven Central, RubyGems, Packagist, Go modules, NuGet)
- Implement pre-commit hooks integration (prek/lefthook/husky) in CI validation stage

## Referenced Agents

- None currently (CI/CD is infrastructure managed by quality-verification and build-distribution domains)

## Referenced Skills

- **cicd-pipeline-standards**: Multi-stage architecture (Validate → Build → Test → Deploy). Multi-platform testing. Artifact management. Task command usage. BUILD_PROFILE=ci configuration. Pre-commit hook integration.

## Referenced Rules

- **continuous-integration-coverage**: CI split by domain, linting/formatting checks, test matrix, OS matrix, artifact generation, quality gates, distribution validation, version-gated releases
- **task-automation-workflow**: Taskfile.yaml primary interface for CI workflows

## Interaction Points

- **Receives from**: build-distribution domain (build artifacts), quality-verification domain (test requirements)
- **Provides to**: users and developers (published packages, CI status, release artifacts)
- **Coordinates with**: quality-verification for test execution, build-distribution for artifact publishing

## Critical Files This Domain Manages

- `.github/workflows/ci-rust.yaml` (Rust core CI)
- `.github/workflows/ci-python.yaml` (Python bindings CI)
- `.github/workflows/ci-node.yaml` (TypeScript/Node.js CI)
- `.github/workflows/ci-wasm.yaml` (WebAssembly CI)
- `.github/workflows/ci-ruby.yaml` (Ruby bindings CI)
- `.github/workflows/ci-php.yaml` (PHP extension CI)
- `.github/workflows/ci-go.yaml` (Go bindings CI)
- `.github/workflows/ci-java.yaml` (Java bindings CI)
- `.github/workflows/ci-validate.yaml` (Linting and format validation)
- `.github/workflows/release.yaml` (Tag-based multi-platform releases)
- `.github/workflows/pre-commit.yaml` (Pre-commit hook validation)
