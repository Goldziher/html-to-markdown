______________________________________________________________________

## priority: critical

# Modular Taskfile Structure

**Root**: Taskfile.yml (version 3) includes all modular task files from .task/ directory.

**Configuration Files**:

- `.task/config/vars.yml`: Global variables (BUILD_PROFILE, VERSION, PDFIUM_VERSION, ORT_VERSION, GOLANGCI_LINT_VERSION, paths, OS/ARCH detection, CARGO_PROFILE_DIR mapping)
- `.task/config/platforms.yml`: Platform-specific variables (EXE_EXT, LIB_EXT, NUM_CPUS with comprehensive Windows/Linux/macOS support)

**Language Files** (namespaced tasks):

- `.task/languages/rust.yml`: Rust build/test/format/lint tasks
- `.task/languages/python.yml`: Python build/test/format/lint tasks
- `.task/languages/node.yml`: TypeScript/Node.js build/test/format/lint tasks
- `.task/languages/go.yml`: Go build/test/lint tasks
- `.task/languages/java.yml`: Java build/test/lint tasks
- `.task/languages/csharp.yml`: C# build/test tasks
- `.task/languages/wasm.yml`: WebAssembly build/test tasks
- `.task/languages/ruby.yml`: Ruby build/test/lint tasks
- `.task/languages/php.yml`: PHP build/test/lint tasks

**Workflow Orchestration Files** (internal, cross-language):

- `.task/workflows/build.yml`: build, build:all, build:all:dev, build:all:release
- `.task/workflows/test.yml`: test, test:all, test:all:fast
- `.task/workflows/lint.yml`: lint, lint:all, lint:check
- `.task/workflows/e2e.yml`: e2e, e2e:all, e2e:fast

**Tool Task Files**:

- `.task/tools/general.yml`: setup, clean, setup-pre-commit, pre-commit
- `.task/tools/version-sync.yml`: version:sync (sync version across all manifests)
- `.task/tools/pdfium.yml`: pdfium:install, pdfium:setup
- `.task/tools/pre-commit.yml`: pre-commit configuration
- `.task/tools/docs.yml`: Documentation generation tasks
- `.task/tools/smoke.yml`: Smoke tests
- `.task/test-config.yml`: Test configuration

**Namespace Convention**:

- Language tasks: `task rust:build`, `task python:test`, `task node:lint`
- Workflow tasks: `task build:all`, `task test:all`, `task lint:check`
- Tool tasks: `task version:sync`, `task pdfium:install`, `task setup`, `task clean`
