______________________________________________________________________

## priority: critical

# Multi-Platform Release Coordination

**Simultaneous releases · Coordinated publishing · Registry-specific procedures · Rollback strategy**

## Core Principle

All package releases across all language ecosystems (Rust, Python, JavaScript, Ruby, PHP, Go, Java, C#, Elixir) must happen on the same calendar date with the same version number. No staggered releases, no ecosystem-specific versioning—one unified release event.

## Simultaneous Release Requirement

### Version Unification

- **All packages released at same version**: e.g., v1.2.3 across all ecosystems
- **Same release date**: All packages published within 2-hour window (UTC)
- **Coordinated timing**: Release window determined by registry APIs availability
- **No pre-releases across ecosystems**: If v1.2.3-rc1 released, all ecosystems release -rc1

### Release Rationale

1. **User clarity**: Single version number across ecosystem stack
1. **Dependency simplification**: Intercomponent versioning straightforward
1. **Security parity**: Security fixes apply uniformly across releases
1. **Change tracking**: Changelog entries consolidated by release date
1. **Documentation**: Single release announcement covers all ecosystems

### Monorepo Release Model

- **Workspace releases**: Single Cargo.toml version drives all packages
- **No independent releases**: Language-specific sub-packages never released independently
- **Lock-step versioning**: All crates, wheels, npm packages, gems released together
- **Uniform changelog**: Single CHANGELOG.md documents all ecosystem changes

## Release Checklist

### Pre-Release Validation (T-7 days)

Perform these checks before opening release PR:

- [ ] All tests passing on main branch

  - Rust: `cargo test --workspace --all-features`
  - Python: `pytest packages/python tests/`
  - JavaScript: `pnpm run test` in packages/typescript
  - Ruby: `rspec spec/` in packages/ruby
  - PHP: `phpunit tests/` in packages/php-ext
  - Go: `go test ./...` if applicable
  - Java: `mvn test` if applicable
  - C#: `dotnet test` if applicable
  - Elixir: `mix test` if applicable

- [ ] All linters passing (via `task lint`)

  - Cargo clippy: zero warnings
  - Python ruff: zero issues
  - TypeScript biome: zero issues
  - Ruby rubocop: zero issues
  - PHP phpstan: zero issues

- [ ] Code coverage thresholds met

  - Rust: ≥ 95% coverage
  - Python: ≥ 85% coverage
  - TypeScript: ≥ 80% coverage
  - Ruby: ≥ 80% coverage

- [ ] Security audit passed (polyglot-security-hardening rule)

  - `cargo audit --deny warnings` passes
  - `pip-audit --desc` clean
  - `npm audit` no critical issues
  - All SAFETY comments present in unsafe code

- [ ] Version consistency verified (cross-ecosystem-version-synchronization rule)

  - Cargo.toml workspace.package.version correct
  - `task sync-versions --dry-run` shows all files would update correctly
  - All language manifests align

- [ ] Documentation updated

  - README.md reflects new features/changes
  - API docs generated for all languages
  - CHANGELOG.md entries drafted for all ecosystems
  - Migration guides written (if breaking changes)

- [ ] Changelog generated for all ecosystems

  - Rust: Features, bug fixes, deprecations
  - Python: Bindings updates, behavioral changes
  - JavaScript: API changes, performance improvements
  - Ruby: New methods, bug fixes
  - PHP: Extension updates, compatibility notes
  - Go: Module updates (if applicable)
  - Java: Package updates (if applicable)
  - C#: NuGet updates (if applicable)
  - Elixir: Hex package updates (if applicable)

### Release Day Execution (T day)

1. **Prepare release branch** (T-0, morning)

   ```bash
   git checkout -b release/v0.8.3
   ```

1. **Update version in Cargo.toml only**

   ```toml
   [workspace.package]
   version = "0.8.3"
   ```

1. **Synchronize versions across ecosystems**

   ```bash
   task sync-versions
   ```

1. **Update CHANGELOG.md** for all ecosystems

   - Consolidated entry documenting all changes
   - Breaking changes highlighted
   - Migration instructions provided
   - Contributors credited

1. **Create release commit**

   ```bash
   git add Cargo.toml pyproject.toml package.json composer.json Gemfile \
     go.mod pom.xml *.csproj mix.exs CHANGELOG.md
   git commit -m "release: v0.8.3 - [Release Title]"
   ```

1. **Push release branch and open PR**

   ```bash
   git push origin release/v0.8.3
   gh pr create --title "Release v0.8.3" --body "..."
   ```

1. **Wait for CI approval** (1-2 hours)

   - All tests pass
   - All security checks pass
   - Version consistency verified
   - Coverage thresholds met

1. **Merge release PR** (T-2 hours before publishing)

   - Squash and merge to main
   - Tag commit: `git tag -a v0.8.3 -m "Release v0.8.3"`
   - Push tag: `git push origin v0.8.3`

## Publishing Order and Registry-Specific Requirements

### 1. Rust Ecosystem (Crates.io)

**Timing**: T+0 hours

Requirements:

- All crates in workspace must have matching version
- Cargo.toml dependencies referencing internal crates use exact versions
- Documentation built successfully: `cargo doc --open`
- Examples compile without warnings

Publishing:

```bash
# Verify package contents
cargo package -p html-to-markdown --allow-dirty

# Dry run publish
cargo publish -p html-to-markdown --dry-run

# Publish to crates.io (requires API token)
cargo publish -p html-to-markdown
cargo publish -p html-to-markdown-core
cargo publish -p html-to-markdown-bindings
# ... publish all workspace crates in dependency order
```

Registry verification:

- [ ] crates.io shows all published crates
- [ ] Documentation rendered correctly
- [ ] Yanked old versions (if applicable)

### 2. Python Ecosystem (PyPI)

**Timing**: T+15 minutes (after Crates.io available)

Requirements:

- Python version from sync-versions matches Cargo
- Maturin build completes for all platforms
- Wheel files for: Linux x86_64, Linux aarch64, macOS x86_64, macOS arm64, Windows x86_64
- Source distribution (.tar.gz) includes all Python code

Building wheels:

```bash
cd packages/python
maturin build --release -o dist/
maturin build --release --target aarch64-unknown-linux-gnu -o dist/
maturin build --release --target x86_64-pc-windows-msvc -o dist/
```

Publishing:

```bash
# Build and verify
python -m twine check dist/*

# Upload to PyPI (requires API token)
python -m twine upload dist/* --skip-existing
```

Registry verification:

- [ ] PyPI shows new release
- [ ] All wheel files available for download
- [ ] Source distribution includes Python bindings
- [ ] Previous version not affected

### 3. JavaScript/npm Ecosystem (npm Registry)

**Timing**: T+30 minutes (after PyPI available)

Requirements:

- All packages in packages/typescript use same version
- TypeScript builds without errors: `pnpm run build`
- No console warnings or deprecation notices
- Typings generated and included in distribution

Publishing:

```bash
cd packages/typescript

# Verify package contents
npm pack

# Set npm registry (if using private registry)
npm config set registry https://registry.npmjs.org/

# Dry run publish
npm publish --dry-run

# Publish to npm
npm publish --access public
```

Registry verification:

- [ ] npm shows package at correct version
- [ ] TypeScript typings available via `@types/package`
- [ ] Source maps included for debugging
- [ ] Monorepo packages all published

### 4. Ruby Ecosystem (Rubygems)

**Timing**: T+45 minutes (after npm available)

Requirements:

- Gemspec version matches release version
- Native extension compiles: `bundle exec rake compile`
- Documentation generated: `bundle exec rake yard`
- Test suite passes on Ruby 3.2+

Building gem:

```bash
cd packages/ruby
gem build kreuzberg.gemspec
```

Publishing:

```bash
# Verify gem integrity
gem check kreuzberg-0.8.3.gem

# Push to Rubygems (requires API key)
gem push kreuzberg-0.8.3.gem
```

Registry verification:

- [ ] Rubygems.org shows new version
- [ ] Native extension available for Ruby 3.2+
- [ ] Documentation rendered on rdoc.info
- [ ] Previous versions still available

### 5. PHP Ecosystem (Packagist)

**Timing**: T+60 minutes (after Ruby available)

Requirements:

- composer.json version synchronized
- PHP 8.2+ compatibility verified
- Extension builds for Linux/macOS
- No deprecation warnings in bindings

Publishing:

```bash
cd packages/php-ext

# Composer validates composer.json
composer validate

# Tag and push (Packagist auto-updates from GitHub tags)
git tag php-v0.8.3
git push origin php-v0.8.3
```

Registry verification:

- [ ] Packagist shows new version
- [ ] composer require kreuzberg:^0.8.3 works
- [ ] Extension loads without warnings
- [ ] GitHub webhook triggered Packagist update

### 6. Go Ecosystem (pkg.go.dev)

**Timing**: T+75 minutes (optional, after PHP)

Requirements:

- go.mod module version correct
- go.mod dependencies resolved
- Examples compile and run

Publishing:

```bash
# Tag in format vX.Y.Z (with 'v' prefix)
git tag go-v0.8.3
git push origin go-v0.8.3
```

Registry verification:

- [ ] pkg.go.dev indexes new release
- [ ] `go get` resolves correct version
- [ ] Module documentation available

### 7. Java/Maven Ecosystem (Maven Central)

**Timing**: T+90 minutes (optional, after Go)

Requirements:

- pom.xml version synchronized
- Maven build completes: `mvn clean install`
- Javadoc generated
- GPG signing configured

Publishing:

```bash
# Deploy to Maven Central
mvn clean deploy -P release
```

Registry verification:

- [ ] Maven Central shows release
- [ ] Javadoc accessible online
- [ ] Artifact coordinates resolvable

### 8. .NET/C# Ecosystem (NuGet)

**Timing**: T+105 minutes (optional, after Maven)

Requirements:

- \*.csproj version synchronized
- NuGet package builds: `dotnet pack -c Release`
- XML documentation included
- Assembly signing configured

Publishing:

```bash
# Pack NuGet package
dotnet pack -c Release

# Push to NuGet (requires API key)
dotnet nuget push bin/Release/*.nupkg --api-key $NUGET_API_KEY --source https://api.nuget.org/v3/index.json
```

Registry verification:

- [ ] NuGet.org shows new package version
- [ ] Package metadata complete
- [ ] Binaries included in package

### 9. Elixir Ecosystem (Hex.pm)

**Timing**: T+120 minutes (optional, after NuGet)

Requirements:

- mix.exs version synchronized
- Elixir build succeeds: `mix compile`
- Hex package built: `mix hex.build`
- Documentation generated

Publishing:

```bash
# Publish to Hex
mix hex.publish
```

Registry verification:

- [ ] Hex.pm shows new release
- [ ] hexdocs.pm documentation available
- [ ] Dependencies resolved correctly

## GitHub Release Creation

After all registries publish successfully:

1. **Draft GitHub Release**

   ```bash
   gh release create v0.8.3 \
     --title "Release v0.8.3: [Release Title]" \
     --draft \
     --notes "See CHANGELOG.md"
   ```

1. **Attach Release Artifacts** (if applicable)

   - Compiled binaries (CLI, extensions)
   - Checksums file (SHA256)
   - GPG signature files
   - Documentation bundle

1. **Publishing GitHub Release**

   - Use web UI to review before publish
   - Publish release
   - Cross-link from security advisories (if CVEs)

## Rollback Procedures

### Pre-Registry Rollback (Before Any Publication)

If release validation fails before first registry publish:

1. **Don't merge release PR** to main
1. **Fix issues** in release branch
1. **Update release commit** with fixes
1. **Re-run tests** and security checks
1. **Merge when ready**

### Post-Partial Publish Rollback

If first registries publish but later registries fail:

1. **Halt all further publication** immediately
1. **Document failure reason** in issue
1. **Prepare patch release** with fix
1. **Yank problematic version** from all published registries

**Yanking procedures**:

- Crates.io: `cargo yank --vers 0.8.3`
- PyPI: Use web UI to yank version
- npm: `npm unpublish @pkg/name@0.8.3` (within 72 hours)
- Rubygems: `gem yank kreuzberg -v 0.8.3`
- Packagist: Web UI to mark as abandoned
- Other registries: Version-specific UI controls

### Post-Full Publish Rollback

If all registries publish but critical issue discovered:

1. **Security advisory issued** immediately
1. **Patch release prepared** within 24 hours
1. **Versions yanked** from registries
1. **Users notified** to upgrade to patched version
1. **Post-mortem** scheduled to prevent recurrence

## Release Monitoring

### 24-Hour Post-Release Monitoring

- [ ] Download and install packages from each registry
- [ ] Run basic functionality tests
- [ ] Check for installation errors or warnings
- [ ] Monitor issue tracker for user reports
- [ ] Verify documentation accessible
- [ ] Check for package manager cache updates

### Weekly Post-Release Monitoring (First Month)

- [ ] Monitor GitHub issues and discussions
- [ ] Check downstream project compatibility
- [ ] Verify security scanning tools recognize update
- [ ] Track download/installation metrics
- [ ] Review user feedback
- [ ] Prepare patch release if critical issues found

## Agent Coordination

The **release-versioning-coordinator** agent is responsible for:

- Orchestrating release timeline across ecosystems
- Managing registry authentication and credentials
- Verifying version consistency before publication
- Coordinating with dependency-management-coordinator agent
- Handling rollback decisions
- Publishing release announcements
- Monitoring post-release stability

## Release Communication

### Release Announcement Contents

- Release version and date
- Summary of major features
- Breaking changes (if any)
- Security fixes (if applicable)
- Known issues (if any)
- Upgrade path for each ecosystem
- Links to registry pages
- Credit contributors

### Timing of Announcements

- **GitHub release**: Published at end of release day
- **Email notification**: Sent 24 hours post-release
- **Blog post**: Published 1 week post-release (if significant)
- **Security advisory**: Same-day publication (if CVE)

## Never

- Release different versions across ecosystems
- Publish to registries on different days
- Skip security checks before release
- Release with failing tests or coverage below threshold
- Yank versions without notification
- Modify published packages after registry acceptance
- Release before all registries confirmed operational
- Merge release PR without all CI checks passing
- Publish pre-release and stable versions simultaneously
- Release without updated CHANGELOG.md
