# Quality Verification Domain

## Purpose

The quality-verification domain ensures code quality, correctness, and reliability across all Rust and language-binding implementations through comprehensive testing, continuous integration, and automated code review standards.

## Scope and Responsibilities

- Establish and enforce 95% test coverage for Rust core library (cargo-llvm-cov enforcement)
- Mandate 80%+ test coverage for all language bindings (pytest-cov, vitest, rspec, phpunit, etc.)
- Implement dual-tier testing strategy: Rust tests for core (95%), language-specific tests for bindings (80%+)
- Design fixture-driven test generation system (JSON/YAML in examples/fixtures/ with schemas)
- Auto-generate E2E tests across languages via tools/e2e-generator
- Maintain E2E test suites in e2e/{rust,python,typescript,ruby,java,go,csharp,elixir}/ directories
- Execute multi-platform CI/CD testing across Linux (amd64/arm64), macOS (Intel/ARM), Windows
- Implement quality gates: zero clippy warnings, test pass requirements, coverage threshold enforcement
- Perform code review verification for FFI safety, error handling, SAFETY comments, memory correctness
- Verify rule adherence (no unwrap/Any/class tests), security patterns, performance optimizations
- Coordinate test failure analysis and regression detection

## Referenced Agents

- **test-automation-engineer**: Test infrastructure and auto-generation. Rust cargo test with #[tokio::test], pytest, pnpm test, rspec, mvn test, go test. E2E generator coordination.
- **code-reviewer**: Quality and compliance review. Implementation gaps, redundancies, rule adherence, security, FFI safety, cross-language consistency.

## Referenced Skills

- **testing-philosophy-coverage**: Three-tier testing (unit 80-95%, integration with real infrastructure, E2E smoke/full). Rust 95%, Python/TypeScript/Ruby/Java 80%+ coverage minimums. Real objects over mocks.
- **cicd-pipeline-standards**: GitHub Actions multi-platform testing. Stages: Validate → Build → Test → Deploy. Quality gates enforcement. Task command usage. BUILD_PROFILE=ci configuration.
- **test-apps-published-package-validation**: Package validation harness for testing distribution artifacts pre-release

## Referenced Rules

- **dual-testing-strategy-core-bindings**: Rust core 95% coverage via cargo-llvm-cov, bindings 80%+ in language-native frameworks
- **continuous-integration-coverage**: GitHub Actions matrix (Python 3.10/3.12/3.14-dev, PHP 8.2+, Rust 1.75+). OS matrix Linux/macOS/Windows. Artifacts: rust-coverage.lcov, coverage.lcov.
- **code-quality**: Coding standards for descriptive names, complex logic comments, unit tests, \<50 line functions, explicit error handling
- **code-quality-with-prek**: Pre-commit hooks with prek/lefthook/husky for linting, formatting, test execution

## Interaction Points

- **Receives from**: rust-core domain (test requirements), language-bindings domain (binding test suites), build-distribution domain (artifacts to test)
- **Provides to**: devops-infrastructure domain (CI/CD configuration), organizational domain (standards enforcement)
- **Coordinates with**: rust-core and language-bindings domains for coverage targets

## Critical Files This Domain Manages

- `crates/*/tests/` (Rust integration test fixtures and harnesses)
- `packages/python/tests/` (Python test suites with pytest)
- `packages/typescript/tests/` (TypeScript test suites with vitest)
- `packages/ruby/spec/` (Ruby test suites with rspec)
- `packages/php-ext/tests/` (PHP test suites with phpunit)
- `packages/go/v*/` (Go test suites with go test)
- `packages/java/src/test/java/` (Java test suites with JUnit5)
- `packages/csharp/*.Tests/` (C# test suites)
- `packages/elixir/test/` (Elixir test suites with ExUnit)
- `e2e/` (Auto-generated E2E test suites)
- `examples/fixtures/` (JSON/YAML test fixtures with schemas)
- `.github/workflows/ci-*.yaml` (CI/CD workflow definitions)
