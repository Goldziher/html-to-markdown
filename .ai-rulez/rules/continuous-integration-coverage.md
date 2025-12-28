---
priority: medium
---

# Continuous Integration & Coverage

**GitHub Actions · Multi-platform · Coverage enforcement**

- CI split by domain (Kreuzberg pattern): ci-rust, ci-python, ci-node, ci-wasm, ci-ruby, ci-php, ci-go, ci-java, ci-elixir, ci-validate
- Linting & formatting: ruff, clippy, rubocop, biome, phpstan, golangci-lint
- Test matrix: Python (3.10, 3.12, 3.14-dev), PHP (8.2+), Rust (stable 1.75+)
- OS matrix: Linux (amd64, arm64), macOS, Windows (where applicable)
- Artifacts: Rust coverage → rust-coverage.lcov, Python → coverage.lcov
- Quality gates: zero warnings, tests pass, coverage thresholds (Rust 95%, others 80%)
- Wheel builds: separate test-wheels.yaml for PyPI distribution testing
- Version-gated: tag-based releases trigger multi-platform builds
