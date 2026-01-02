______________________________________________________________________

## priority: critical

# CI/CD Pipeline Standards

**Architecture**: Stages: Validate (lint/format) → Build → Test (unit/integration) → Deploy. Quality gates: zero warnings, tests pass, coverage thresholds met.

**Multi-platform Testing**: linux/amd64, linux/arm64, macOS (Intel/ARM). Docker: multi-stage builds, minimal base images (alpine/distroless).

**Artifact Management**: Cache dependencies (Cargo, npm, Maven, Go modules). Publish packages (PyPI/npm/crates.io/Maven Central).

**CI Workflows Use Task Commands**:

- Workflows now ALWAYS use `task` commands, never direct script calls
- All CI workflows automatically set `BUILD_PROFILE=ci`
- Example workflow structure:
  ```yaml
  - name: Setup
    run: task setup
  - name: Lint
    run: task lint:check
  - name: Build
    run: BUILD_PROFILE=ci task build:all
  - name: Test
    run: BUILD_PROFILE=ci task test:all
  - name: E2E
    run: BUILD_PROFILE=ci task e2e:all
  ```

**BUILD_PROFILE in CI**: Always set `BUILD_PROFILE=ci` in GitHub Actions workflows:

- Provides release-optimized binaries
- Includes debug symbols for troubleshooting
- Consistent with local development workflow
- Use `task lint:check` for CI-specific linting (fails on issues vs. warnings)

**Pre-commit Hooks in CI**: GitHub Actions runs `task pre-commit` in validate stage to catch linting/formatting issues early
