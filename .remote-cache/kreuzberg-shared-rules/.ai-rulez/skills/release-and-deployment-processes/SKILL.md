______________________________________________________________________

## priority: critical

# Release & Deployment Processes

## Semantic Versioning Strategy

- **Format**: MAJOR.MINOR.PATCH (e.g., 1.2.3)
  - MAJOR: Breaking changes to public API
  - MINOR: New features, backward-compatible
  - PATCH: Bug fixes, no new features
- **Pre-releases**: Use -alpha, -beta, -rc suffixes (e.g., 1.0.0-beta.1)
- **Build metadata**: +build.123 for CI build info (not part of precedence)
- **Commit tags**: v1.2.3 format, signed with GPG when possible
- **Version consistency**: Single source of truth in Cargo.toml, then sync to package.json, pyproject.toml, version.txt

## Coordinated Multi-Platform Release

- **Release branch**: Create `release/v1.2.3` from main with final version bumps
- **Single coordinated release**: All language bindings (Python, Node.js, Ruby, Java, Go, C#) released same day
- **Binaries**: Pre-build and attach to GitHub Release for macOS (arm64, x86_64), Linux, Windows
- **Language-specific releases**:
  - **Rust**: cargo publish to crates.io after GitHub Release
  - **Python**: twine upload to PyPI (requires PyPI token in secrets)
  - **Node.js**: npm publish to npm registry (requires npm token)
  - **Ruby**: gem push to RubyGems.org
  - **Java**: Maven Central via Sonatype (GPG-signed)
  - **Go**: Tag commit, go proxy handles publishing
  - **C#**: NuGet.org via dotnet nuget push

## Release Automation with GitHub Actions

Example workflow for coordinated release:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  publish-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish --token ${{ secrets.CARGO_TOKEN }}

  publish-python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      - run: |
          pip install twine wheel
          cd python && python -m build
          twine upload dist/* -u __token__ -p ${{ secrets.PYPI_TOKEN }}

  publish-npm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      - run: npm ci && npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  create-github-release:
    runs-on: ubuntu-latest
    needs: [publish-rust, publish-python, publish-npm]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
      - uses: softprops/action-gh-release@v1
        with:
          files: artifacts/**/*
          draft: false
```

## Changelog Generation with git-cliff

- **Tool**: git-cliff for automated changelog from commits
- **Config file**: cliff.toml in repository root
- **Commit-driven**: Parses conventional commits (feat:, fix:, etc.) from merge commits
- **Template**: Organize by type (Breaking Changes, Features, Bug Fixes, Documentation)

Example cliff.toml:

```toml
[changelog]
title = "# Changelog"
body = """
{% for release in releases -%}
## [{{ release.version }}] - {{ release.date }}
{% if release.breaking %}
### Breaking Changes
{% for breaking in release.breaking -%}
- {{ breaking.message }}
{% endfor %}
{% endif %}
{% if release.features %}
### Features
{% for feature in release.features -%}
- {{ feature.message }}
{% endfor %}
{% endif %}
{% if release.fixes %}
### Bug Fixes
{% for fix in release.fixes -%}
- {{ fix.message }}
{% endfor %}
{% endif %}
{% endfor %}
"""
commit_parsers = [
  { message = "^feat", group = "features" },
  { message = "^fix", group = "fixes" },
  { message = "^doc", group = "documentation" },
  { message = "^perf", group = "performance" },
  { message = "^refactor", group = "refactoring" },
  { message = "^test", skip = true },
]
```

Usage:

```bash
git cliff --output CHANGELOG.md
git cliff v1.0.0..v1.1.0  # Specific version range
```

## Publishing to Registries

### Crates.io (Rust)

- Requires ownership verification and token
- Use `cargo publish` from clean workspace
- Prevent yank of versions with active dependents
- Validate with `cargo package --allow-dirty` before publishing

```bash
cargo login  # Stores token in ~/.cargo/credentials.toml
cargo publish --token $CARGO_TOKEN
```

### PyPI (Python)

- Build via setuptools/build module
- Use `twine` for secure uploads with API tokens
- Separate PyPI token per project recommended
- No token in repository secrets (use GitHub encrypted secrets)

```bash
python -m build  # Creates dist/ with wheels and sdist
twine upload dist/* --username __token__ --password $PYPI_TOKEN
```

### npm (Node.js/TypeScript)

- Publish from root or scoped package directory
- Ensure package.json version matches git tag
- Use two-factor authentication if enabled on npm account
- Provenance attestation via OIDC (GitHub)

```bash
npm publish --access public  # or --access restricted
npm info @myorg/pkg  # Verify published version
```

### RubyGems

- Build gem via `gem build gemspec.gemspec`
- Requires account and API key
- Yank unsecure versions quickly if needed

```bash
gem build html-to-markdown.gemspec
gem push html-to-markdown-1.0.0.gem
```

### Maven Central (Java)

- Requires Sonatype account and GPG key setup
- Sign artifacts before upload
- Use gradle-nexus-publish-plugin for automation
- Can take hours for artifacts to sync

```gradle
plugins {
    id "io.github.gradle-nexus.publish-plugin" version "1.3.0"
}

nexusPublishing {
    repositories {
        sonatype {
            nexusUrl.set(uri("https://s01.oss.sonatype.org/service/local/"))
            snapshotRepositoryUrl.set(uri("https://s01.oss.sonatype.org/content/repositories/snapshots/"))
            username = System.getenv("MAVEN_USERNAME")
            password = System.getenv("MAVEN_PASSWORD")
        }
    }
}
```

### Go Module Publishing

- No central registry; uses version control tags
- Go proxy (`proxy.golang.org`) automatically caches modules
- Tag must be on commit: `git tag v1.2.3 && git push --tags`
- Minimal go.mod, go.sum management required

```bash
git tag v1.2.3
git push origin v1.2.3
# Go proxy caches within 24h, can purge with go get -u
```

### NuGet.org (C#/.NET)

- Package via `dotnet pack`
- Sign packages with strong name or authenticode
- API key per package recommended
- Push via dotnet CLI or web portal

```bash
dotnet pack --configuration Release
dotnet nuget push bin/Release/MyPackage.1.0.0.nupkg \
  --api-key $NUGET_API_KEY \
  --source https://api.nuget.org/v3/index.json
```

## Release Checklist

1. **Pre-release**:

   - [ ] All tests passing on main
   - [ ] Code review complete
   - [ ] Security scanning clean (no high/critical CVEs)
   - [ ] Performance benchmarks stable

1. **Version bump**:

   - [ ] Update Cargo.toml version
   - [ ] Update package.json version
   - [ ] Update pyproject.toml version
   - [ ] Sync version across all language bindings
   - [ ] Commit with message: "chore: bump version to 1.2.3"

1. **Changelog**:

   - [ ] Generate via git-cliff
   - [ ] Review for accuracy and completeness
   - [ ] Add to CHANGELOG.md

1. **Git tag**:

   - [ ] Create annotated tag: `git tag -a v1.2.3 -m "Release 1.2.3"`
   - [ ] Sign tag: `git tag -s v1.2.3 -m "Release 1.2.3"` (requires GPG)
   - [ ] Push: `git push origin v1.2.3`

1. **Registry publishing**:

   - [ ] Publish to crates.io
   - [ ] Publish to PyPI
   - [ ] Publish to npm
   - [ ] Publish to RubyGems
   - [ ] Publish to Maven Central
   - [ ] Create GitHub Release with artifacts

1. **Post-release**:

   - [ ] Verify installations work: `pip install`, `npm install`, `cargo add`
   - [ ] Update documentation with version
   - [ ] Announce release in issue/discussion
   - [ ] Monitor for adoption and early bug reports

## Anti-Patterns

- **Manual registry uploads**: Always automate with CI/CD
- **Version drift**: Keep single source of truth (Cargo.toml), sync others in CI
- **Releasing without tests**: Run full test suite including cross-platform matrix
- **Skipping CHANGELOG**: Git-cliff automates this; always include
- **Pre-release as production**: Clearly mark alpha/beta/rc tags
- **No artifact signature**: Sign binaries for security-conscious users
- **Releasing to one registry only**: Publish to ALL official registries simultaneously
- **Manual dependency updates**: Let dependents auto-discover via lock files
