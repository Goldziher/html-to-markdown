# .task/ Directory - Modular Task Organization

This directory contains the modular Task configuration for the html-to-markdown project, following the **Kreuzberg pattern** for maintainable, scalable build automation.

## Purpose

The `.task/` directory structure reduces the root `Taskfile.yml` from 838 lines to ~250 lines (66% reduction) by organizing tasks into logical modules. This approach:

- **Improves Maintainability**: Each language/workflow lives in its own file
- **Enables Reusability**: Common patterns defined once, reused everywhere
- **Simplifies Testing**: Test individual modules independently
- **Supports Cross-Platform**: Platform-specific logic isolated in config/
- **Scales Gracefully**: Adding new languages doesn't bloat the root Taskfile

## Directory Structure

```
.task/
├── config/
│   ├── vars.yml           # Global variables, version detection, paths
│   └── platforms.yml      # OS/arch detection, library extensions, target triples
│
├── languages/             # Language-specific task modules (11 total)
│   ├── rust.yml          # Rust core library tasks
│   ├── python.yml        # PyO3 Python bindings
│   ├── node.yml          # NAPI-RS Node.js bindings
│   ├── typescript.yml    # TypeScript wrapper package
│   ├── wasm.yml          # WebAssembly bindings
│   ├── ruby.yml          # Magnus Ruby bindings
│   ├── php.yml           # ext-php-rs PHP extension
│   ├── go.yml            # Go FFI wrapper
│   ├── java.yml          # Java JNI bindings
│   ├── csharp.yml        # C# P/Invoke wrapper
│   └── elixir.yml        # Elixir NIF bindings
│
├── workflows/            # Aggregated workflow tasks (internal)
│   ├── build.yml        # Build all languages with profile support
│   ├── test.yml         # Test all languages (parallel/sequential)
│   ├── lint.yml         # Lint all languages with auto-fix
│   └── benchmark.yml    # Benchmark harness orchestration
│
└── tools/               # Utility and automation tasks
    ├── version-sync.yml # Version synchronization across manifests
    ├── general.yml      # TOML formatting, shell linting
    └── pre-commit.yml   # Prek pre-commit hook management (future)
```

## Configuration Files

### `config/vars.yml`

**Purpose**: Global variables shared across all task modules.

**Key Variables**:
```yaml
VERSION:           # Extracted from Cargo.toml
BUILD_PROFILE:     # dev/release/ci (default: release)
OS:                # darwin/linux/windows
ARCH:              # x86_64/arm64/armv7
NUM_CPUS:          # Detected CPU count for parallel builds
ROOT:              # Project root directory
CRATES_DIR:        # crates/ directory
PACKAGES_DIR:      # packages/ directory
TARGET_DIR:        # target/ directory (Rust build outputs)
```

**Example Usage**:
```yaml
# In any language module:
dir: "{{.PACKAGES_DIR}}/python"
cmds:
  - cargo build --profile {{.BUILD_PROFILE}}
```

### `config/platforms.yml`

**Purpose**: Platform-specific detection and configuration.

**Key Variables**:
```yaml
EXE_EXT:           # .exe on Windows, empty on Unix
LIB_EXT:           # dylib/so/dll based on OS
LIB_PREFIX:        # lib on Unix, empty on Windows
RUST_TARGET:       # Target triple (x86_64-apple-darwin, etc.)
RUBY_FULL_PATH:    # Full path to Ruby binary (handles Homebrew ARM64)
IS_WINDOWS:        # Boolean: true on Windows
IS_MACOS:          # Boolean: true on macOS
IS_LINUX:          # Boolean: true on Linux
```

**Example Usage**:
```yaml
# Cross-platform library path configuration:
env:
  LD_LIBRARY_PATH: '{{if ne .OS "windows"}}{{.TARGET_DIR}}/release{{end}}'
  PATH: '{{if eq .OS "windows"}}{{.TARGET_DIR}}/release;{{end}}{{.PATH}}'
```

## Language Modules

Each language module follows a **consistent pattern**:

### Standard Tasks (All Languages)

```yaml
install:           # Install dependencies/toolchain
build:             # Build with profile support (uses BUILD_PROFILE)
build:dev:         # Debug build (fast, unoptimized)
build:release:     # Release build (optimized)
build:ci:          # CI build (release + debug symbols)
test:              # Run tests
test:ci:           # Run tests with coverage (CI mode)
coverage:          # Generate coverage reports (lcov format)
lint:              # Lint + auto-fix (format + linters)
lint:check:        # Check-only (no modifications, for CI)
format:            # Format code
format:check:      # Check formatting without changes
update:            # Update dependencies
clean:             # Remove build artifacts
```

### Example: `languages/python.yml`

```yaml
version: "3"
internal: true

includes:
  platforms: ../config/platforms.yml

vars:
  BUILD_PROFILE: "{{.BUILD_PROFILE | default \"release\"}}"
  PYTHON_WORK_DIR: "{{.PACKAGES_DIR}}/python"

tasks:
  install:
    desc: "Install Python dependencies with uv"
    dir: "{{.PYTHON_WORK_DIR}}"
    cmds:
      - uv sync
      - uv pip install -e .

  build:
    desc: "Build Python bindings with maturin ({{.BUILD_PROFILE}} profile)"
    dir: "{{.PYTHON_WORK_DIR}}"
    cmds:
      - maturin develop{{if eq .BUILD_PROFILE "release"}} --release{{end}}

  test:
    desc: "Run Python tests with pytest"
    dir: "{{.PYTHON_WORK_DIR}}"
    cmds:
      - pytest -v tests/

  # ... additional tasks
```

### Cross-Platform Patterns

**DO Use** (Cross-Platform Compatible):
```yaml
# Python for file operations:
- cmd: |
    python -c "
    import shutil, glob
    for d in ['build', 'dist']:
        shutil.rmtree(d, ignore_errors=True)
    "

# Conditional environment variables:
env:
  LD_LIBRARY_PATH: '{{if ne .OS "windows"}}{{.TARGET_DIR}}/release{{end}}'
  PATH: '{{if eq .OS "windows"}}{{.TARGET_DIR}}/release;{{end}}{{.PATH}}'

# Task's built-in ignore_error:
- cmd: some-command-that-might-fail
  ignore_error: true
```

**DON'T Use** (Platform-Specific):
```yaml
# ❌ Unix-only commands:
- rm -rf build/ dist/
- find . -name "*.pyc" -delete

# ❌ Hardcoded paths:
- /opt/homebrew/bin/ruby
- C:\Program Files\Tool\bin

# ❌ Bash-specific syntax:
- cmd: test -d .venv && source .venv/bin/activate
```

## Workflow Modules

Workflow modules aggregate language tasks into unified operations. These are **internal** (not exposed to users directly).

### `workflows/build.yml`

```yaml
version: "3"
internal: true

tasks:
  all:
    desc: "Build all language bindings"
    cmds:
      - task: rust:build
      - task: python:build
      - task: node:build
      # ... (11 languages)

  all:dev:
    desc: "Build all in debug mode"
    cmds:
      - task: rust:build:dev
      - task: python:build:dev
      # ...

  core:
    desc: "Build Rust core only"
    cmds:
      - task: rust:build

  bindings:
    desc: "Build all bindings (skip core)"
    cmds:
      - task: python:build
      - task: node:build
      # ... (exclude rust)
```

### `workflows/test.yml`

```yaml
version: "3"
internal: true

tasks:
  all:
    desc: "Run all tests"
    cmds:
      - task: rust:test
      - task: python:test
      # ... (sequential)

  all:parallel:
    desc: "Run tests in parallel"
    deps:
      - rust:test
      - python:test
      # ... (parallel execution)

  all:ci:
    desc: "Run CI tests with coverage"
    cmds:
      - task: rust:test:ci
      - task: python:test:ci
      # ...
```

## Tools Modules

### `tools/version-sync.yml`

**Purpose**: Synchronize version across all package manifests.

```yaml
version: "3"

tasks:
  sync:
    desc: "Sync version from Cargo.toml to all manifests"
    cmds:
      - python {{.ROOT}}/scripts/sync_versions.py
```

**Updates**:
- Cargo workspace members (crates/*/Cargo.toml)
- Python (packages/python/pyproject.toml)
- Node.js (packages/typescript/package.json)
- Ruby (packages/ruby/lib/html_to_markdown/version.rb)
- PHP (packages/php/composer.json)
- Go (packages/go/v2/version.go)
- Java (packages/java/pom.xml)
- C# (packages/csharp/HtmlToMarkdown.csproj)
- Elixir (packages/elixir/mix.exs)
- **test_apps manifests** (tests/test_apps/*/pyproject.toml, package.json, etc.)

### `tools/general.yml`

**Purpose**: General-purpose linting and validation tasks.

```yaml
version: "3"

tasks:
  toml:format:
    desc: "Format TOML files"
    cmds:
      - taplo format **/*.toml

  toml:format:check:
    desc: "Check TOML formatting"
    cmds:
      - taplo format --check **/*.toml
```

## How to Add a New Language

Let's add **Swift** as an example:

### Step 1: Create Language Module

**File**: `.task/languages/swift.yml`

```yaml
version: "3"
internal: true

includes:
  platforms: ../config/platforms.yml

vars:
  BUILD_PROFILE: "{{.BUILD_PROFILE | default \"release\"}}"
  SWIFT_WORK_DIR: "{{.PACKAGES_DIR}}/swift"

tasks:
  install:
    desc: "Install Swift dependencies"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swift package resolve

  build:
    desc: "Build Swift package ({{.BUILD_PROFILE}} profile)"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - cmd: swift build{{if eq .BUILD_PROFILE "release"}} -c release{{else}} -c debug{{end}}
        ignore_error: false

  build:dev:
    desc: "Build Swift package in debug mode"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swift build -c debug

  build:release:
    desc: "Build Swift package in release mode"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swift build -c release

  test:
    desc: "Run Swift tests"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swift test

  test:ci:
    desc: "Run Swift tests with coverage (CI mode)"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swift test --enable-code-coverage

  lint:
    desc: "Lint Swift code with auto-fix"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swiftlint --fix
      - swiftformat .

  lint:check:
    desc: "Lint Swift code without auto-fix"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swiftlint
      - swiftformat --lint .

  format:
    desc: "Format Swift code"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swiftformat .

  format:check:
    desc: "Check Swift formatting"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swiftformat --lint .

  update:
    desc: "Update Swift dependencies"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swift package update

  clean:
    desc: "Clean Swift build artifacts"
    dir: "{{.SWIFT_WORK_DIR}}"
    cmds:
      - swift package clean
```

### Step 2: Include in Root Taskfile

**File**: `Taskfile.yml`

```yaml
includes:
  # ... existing includes
  swift:
    taskfile: .task/languages/swift.yml
```

### Step 3: Add to Workflow Aggregators

**File**: `.task/workflows/build.yml`

```yaml
tasks:
  all:
    cmds:
      - task: rust:build
      # ... existing languages
      - task: swift:build  # ADD THIS
```

**File**: `.task/workflows/test.yml`

```yaml
tasks:
  all:
    cmds:
      - task: rust:test
      # ... existing languages
      - task: swift:test  # ADD THIS
```

**File**: `.task/workflows/lint.yml`

```yaml
tasks:
  all:
    cmds:
      - task: rust:lint
      # ... existing languages
      - task: swift:lint  # ADD THIS
```

### Step 4: Update Root Taskfile Aggregates

**File**: `Taskfile.yml`

```yaml
tasks:
  setup:
    cmds:
      - task: rust:install
      # ... existing installs
      - task: swift:install  # ADD THIS
```

Now users can run:
```bash
task swift:build
task swift:test
task swift:lint
```

## Internal vs Public Tasks

### Internal Tasks

Defined with `internal: true` at the file level:
```yaml
version: "3"
internal: true  # This file's tasks are not listed in `task --list`
```

**Characteristics**:
- Not visible in `task --list`
- Only callable from other tasks
- Used for: config files, workflow aggregators

**Examples**:
- `.task/config/vars.yml` (internal)
- `.task/workflows/build.yml` (internal)
- `.task/workflows/test.yml` (internal)

### Public Tasks

Included without `internal: true` or via root Taskfile:
```yaml
includes:
  rust:
    taskfile: .task/languages/rust.yml
    # No internal flag = public
```

**Characteristics**:
- Visible in `task --list`
- Directly callable by users
- Used for: language modules, tool modules

**Examples**:
- `rust:build` (public)
- `python:test` (public)
- `version:sync` (public)

## Best Practices

### 1. Always Use Template Variables

```yaml
# ✅ Good:
dir: "{{.PACKAGES_DIR}}/python"
cmds:
  - cargo build --profile {{.BUILD_PROFILE}}

# ❌ Bad:
dir: "packages/python"
cmds:
  - cargo build --release
```

### 2. Support All Build Profiles

```yaml
# ✅ Good: Profile-aware command
- cmd: maturin develop{{if eq .BUILD_PROFILE "release"}} --release{{end}}

# ❌ Bad: Hardcoded profile
- cmd: maturin develop --release
```

### 3. Use Cross-Platform Commands

```yaml
# ✅ Good: Python for file operations
- cmd: |
    python -c "
    import shutil
    shutil.rmtree('build', ignore_errors=True)
    "

# ❌ Bad: Unix-only command
- cmd: rm -rf build/
```

### 4. Include Platform Config

```yaml
# ✅ Good: Include platforms for cross-platform logic
includes:
  platforms: ../config/platforms.yml

env:
  LD_LIBRARY_PATH: '{{if ne .OS "windows"}}{{.TARGET_DIR}}/release{{end}}'

# ❌ Bad: Hardcoded Unix assumption
env:
  LD_LIBRARY_PATH: "{{.TARGET_DIR}}/release"
```

### 5. Consistent Task Naming

```yaml
# ✅ Good: Consistent naming with colons
install:
build:
build:dev:
build:release:
build:ci:
test:
test:ci:
lint:
lint:check:
format:
format:check:

# ❌ Bad: Inconsistent naming
install_deps:
make_build:
run_tests:
check-format:
```

### 6. Document Descriptions

```yaml
# ✅ Good: Clear, actionable description
install:
  desc: "Install Python dependencies with uv"

# ❌ Bad: Vague or missing description
install:
  desc: "Install stuff"
```

### 7. Error Handling

```yaml
# ✅ Good: Explicit error handling
- cmd: pytest -v tests/
  ignore_error: false  # Fail on errors

- cmd: rm -rf .cache/
  ignore_error: true   # OK to fail (file may not exist)

# ❌ Bad: Implicit behavior
- cmd: pytest -v tests/
```

## Troubleshooting

### Task Not Found

**Error**: `Task "foo:bar" not found`

**Solution**: Ensure the include is in root `Taskfile.yml`:
```yaml
includes:
  foo:
    taskfile: .task/languages/foo.yml
```

### Variable Not Defined

**Error**: `template: :1:2: executing "" at <.SOME_VAR>: map has no entry for key "SOME_VAR"`

**Solution**: Define variable in `.task/config/vars.yml` or include platforms:
```yaml
includes:
  platforms: ../config/platforms.yml
```

### Cross-Platform Failures

**Error**: Task works on macOS but fails on Windows

**Solution**: Use conditional environment variables and cross-platform commands:
```yaml
env:
  PATH: '{{if eq .OS "windows"}}{{.TARGET_DIR}}/release;{{end}}{{.PATH}}'
cmds:
  - cmd: |
      python -c "import shutil; shutil.rmtree('build', ignore_errors=True)"
```

### Circular Dependencies

**Error**: `task: import cycle not allowed`

**Solution**: Avoid including files that include each other. Use internal workflow aggregators instead.

## References

- **Task Documentation**: https://taskfile.dev
- **Kreuzberg Pattern**: ../kreuzberg/ (sibling project)
- **Root Taskfile**: ../Taskfile.yml
- **Platform Config**: config/platforms.yml
- **Global Variables**: config/vars.yml

---

**Last Updated**: 2025-12-28
**Maintainers**: html-to-markdown contributors
