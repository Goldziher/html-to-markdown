______________________________________________________________________

## priority: high

# Test Apps - Published Package Validation

**Purpose**: tests/test_apps validates PUBLISHED/RELEASED packages from npm, PyPI, RubyGems, Maven Central, Docker Hub, and Homebrew. NOT for local development testing.

**Location**: tests/test_apps/{python,node,wasm,ruby,go,java,csharp,rust,docker,homebrew,browser-vite-svelte}

**Version sync**: Included in `task sync-versions` - automatically updates all test app manifests to match Cargo.toml version. Supports all package formats: pyproject.toml, package.json, Gemfile, go.mod, pom.xml, .csproj, Cargo.toml.

**Linting**: Pre-commit hooks (biome, shellcheck, ruff) apply to test_apps. CPD checks excluded (intentional duplication for testing).

**Usage**: Each test app installs the published package and runs comprehensive tests. Run from kreuzberg repo root: `cd tests/test_apps/{language} && {language-specific-test-command}`

**Documentation**: tests/test_apps/README.md contains full details on running tests for each language.

**Critical**: Test apps are for validating releases, not development. Use e2e/ tests for development validation.
