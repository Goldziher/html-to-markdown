# Build & Distribution Domain

## Purpose

The build-distribution domain orchestrates multi-language compilation, packaging, and distribution across Rust, Python, TypeScript, Ruby, PHP, Go, Java, C#, Elixir, and WebAssembly ecosystems. It maintains modular Taskfiles for coordinated builds, version synchronization across all languages, and standardized distribution through package registries.

## Scope and Responsibilities

- Design and maintain modular Taskfile.yaml structure with `.task/` subdirectory organization
- Orchestrate Rust core builds (cargo build --workspace --release with language binding exclusions)
- Manage Python maturin builds and PyPI distribution (uv pip install -e packages/python)
- Coordinate TypeScript pnpm builds post-NAPI-RS compilation
- Control Ruby gem compilation via bundle exec rake compile and package
- Build PHP extension via cargo build -p \*-php --release
- Compile Java bindings via Maven/Gradle with FFI integration
- Build Go bindings with cgo
- Compile C# bindings with dotnet build
- Build Elixir bindings with mix compile
- Generate WebAssembly bindings with wasm-pack
- Synchronize versions across Cargo.toml, package.json, pyproject.toml, Gemfile, composer.json, go.mod, \*.csproj, mix.exs
- Manage lock files (Cargo.lock, pnpm-lock.yaml, Gemfile.lock, composer.lock, go.sum) as version pinning
- Configure BUILD_PROFILE for development, CI, and release optimization levels
- Implement workspace structure across language monorepo with proper dependency resolution

## Referenced Agents

- None currently (orchestration through Taskfile automation, coordinated by polyglot-architect oversight)

## Referenced Skills

- **task-automation-build**: Taskfile.yaml for all workflows (setup, dev, lint, format, test, build), dependency minimization, version sync
- **modular-taskfile-structure**: Root Taskfile.yaml includes `.task/config/vars.yml`, `.task/languages/*.yml`, `.task/workflows/*.yml`, `.task/tools/*.yml`
- **taskfile-best-practices-guidelines**: Task naming conventions, platform guards, conditional commands, variable usage
- **build-profiles**: Development, CI, and release profile configuration with DEBUG/RELEASE optimization mappings
- **workspace-structure-project-organization**: Monorepo organization with crates/, packages/, tools/, e2e/, docs/ directories

## Referenced Rules

- **polyglot-build-system-distribution**: Cargo + maturin + NAPI-RS + Magnus + ext-php-rs multi-language build coordination
- **task-automation-workflow**: Taskfile.yaml primary interface for all development, setup → build → test → lint → format workflow
- **continuous-integration-coverage**: CI workflows use task commands, BUILD_PROFILE=ci for release-optimized binaries with debug symbols

## Interaction Points

- **Receives from**: rust-core domain (Cargo core compilation), language-bindings domain (binding-specific builds)
- **Provides to**: quality-verification domain (build artifacts for testing), devops-infrastructure (CI/CD integration), organizational domain (version coordination)
- **Coordinates with**: quality-verification for coverage verification during builds

## Critical Files This Domain Manages

- `Taskfile.yaml` (root task orchestration file)
- `.task/config/vars.yml` (global variables, VERSION, paths)
- `.task/config/platforms.yml` (Windows/Linux/macOS detection, EXE_EXT, LIB_EXT, NUM_CPUS)
- `.task/languages/rust.yml` (Rust build/test/format/lint tasks)
- `.task/languages/python.yml` (Python environment and build tasks)
- `.task/languages/node.yml` (TypeScript/Node.js build tasks)
- `.task/workflows/build.yml` (build, build:all, build:all:dev, build:all:release tasks)
- `scripts/sync_versions.py` (automatic version synchronization across manifests)
