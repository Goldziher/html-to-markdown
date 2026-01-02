# ADR 006: Monorepo Versioning

**Date:** 2024-11-20

**Status:** Accepted

## Context

The ai-rulez project consists of:

- One Rust core library
- 10+ language-specific bindings (Python, JavaScript, Go, Ruby, PHP, Java, C#, Kotlin, Swift, Dart, etc.)
- Supporting infrastructure (CLI, documentation, fixtures, tooling)

All of these are developed in a single monorepo and released together. Each binding is published to its respective package manager:

- **Python:** PyPI (pip)
- **JavaScript:** NPM
- **Go:** pkg.go.dev
- **Ruby:** RubyGems
- **PHP:** Packagist
- **Java:** Maven Central
- **C#:** NuGet
- **Kotlin:** Maven Central
- **Swift:** Swift Package Index
- **Dart:** Pub.dev

The question is: should we version these packages independently or together?

Options considered:

1. **Independent versioning:** Each package has its own version (complexity, inconsistency risk)
1. **Synchronized versioning:** All packages use the same version number (simplicity, consistency)

## Decision

All packages in the monorepo will use the same semantic version across all language ecosystems. A single version number is incremented for the entire project.

The version scheme is: `MAJOR.MINOR.PATCH` (e.g., `0.3.2`)

Release process:

1. Increment the version in the monorepo root
1. Run release automation that builds and publishes all bindings
1. All bindings are released with the same version number simultaneously
1. A single release notes document covers all changes across all bindings

Bindings are published to their respective package managers with the same semantic version.

## Consequences

### Positive

- **Simplified release process:** One release affects all packages simultaneously
- **Clear compatibility:** Version number alone tells users all components are compatible
- **Reduced confusion:** Users don't need to track different version numbers across ecosystems
- **Simpler documentation:** One version number to document features, bug fixes, changes
- **Coordinated updates:** All users can upgrade all bindings in unison
- **Cleaner git history:** Single version per commit, not multiple version tags
- **Easier migration:** Users know all bindings at version X work together

### Negative

- **All-or-nothing releases:** A minor bug fix in one binding forces a release of all bindings
- **Package manager bloat:** Package managers may accumulate many versions due to frequent releases
- **Semantic versioning complications:** What constitutes a MAJOR/MINOR/PATCH change when multiple bindings are affected differently?
- **Language-specific urgent fixes:** Cannot hot-fix one binding without releasing others
- **Larger release surface:** More packages to test and publish in each release
- **Bandwidth on package managers:** More frequent publishing to multiple ecosystems

### Versioning Strategy

**MAJOR version increments:**

- Breaking API changes in Rust core
- Breaking changes to FFI interface
- Minimum dependency version increases that break compatibility

**MINOR version increments:**

- New features added to any binding or core
- Non-breaking API additions
- Dependency updates

**PATCH version increments:**

- Bug fixes
- Documentation improvements
- Internal refactoring (no user-facing changes)

### Release Process

1. **Prepare release branch:** Create `release/v0.X.Y` branch
1. **Update version:** Increment version in:
   - `Cargo.toml` (Rust core)
   - `pyproject.toml` (Python)
   - `package.json` (JavaScript)
   - `go.mod` (Go)
   - `Gemfile` (Ruby)
   - `composer.json` (PHP)
   - `pom.xml` (Java)
   - `*.csproj` (C#)
   - `Package.swift` (Swift)
   - All other binding version files
1. **Run test suite:** Ensure all fixtures pass on all bindings
1. **Generate release notes:** Create consolidated changelog
1. **Tag release:** Create single git tag `v0.X.Y`
1. **Publish:** Automated CI/CD publishes to all package managers simultaneously
1. **Verify:** Confirm all packages appear in their respective package managers

### Exception Process

In cases of critical security vulnerabilities or severe bugs:

1. Create hotfix branch from last release tag
1. Apply minimal fixes only
1. Increment PATCH version
1. Follow standard release process
1. Document what triggered the exception in release notes

### Future Considerations

- Monitor adoption patterns to identify if some bindings need independent versioning
- Evaluate selective release for truly language-specific packages (e.g., CLI tools)
- Consider pre-release versions (0.X.Y-rc1) for testing
- Establish clear SLAs for each language binding's bug fix commitment
