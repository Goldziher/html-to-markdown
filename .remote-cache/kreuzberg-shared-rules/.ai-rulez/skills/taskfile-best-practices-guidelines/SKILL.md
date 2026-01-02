______________________________________________________________________

## priority: high

# Taskfile Best Practices & Guidelines

**Modular Design Principles**:

- Each language gets its own task file in `.task/languages/`
- Workflows (build, test, lint, e2e) are orchestrated internally in `.task/workflows/`
- Configuration separated into `.task/config/` (vars.yml, platforms.yml)
- Tool tasks in `.task/tools/` (version-sync, pdfium, pre-commit, docs, smoke)
- Main Taskfile.yml is minimal - just includes and top-level entry points

**Creating New Task Files**:

1. Create `.task/languages/{language}.yml` for language-specific tasks
1. Include in main Taskfile.yml: `{language}: taskfile: .task/languages/{language}.yml`
1. Use namespace pattern: `task {language}:build`, `task {language}:test`
1. Support BUILD_PROFILE for dev/release/ci variants
1. Include both `.dev` and `.release` variants of build/test tasks

**Variable Management**:

- Global variables in `.task/config/vars.yml` (BUILD_PROFILE, VERSION, paths, OS/ARCH)
- Platform-specific in `.task/config/platforms.yml` (EXE_EXT, LIB_EXT, NUM_CPUS)
- Language-specific vars in language files (e.g., PYTHON_VERSION, RUST_LOG)
- Avoid hardcoding paths; use {{.ROOT}}, {{.CRATES_DIR}}, {{.PACKAGES_DIR}}, {{.TARGET_DIR}}
- Use {{.BUILD_PROFILE}} to determine debug vs. release builds
- Use {{.OS}}, {{.ARCH}}, {{.EXE_EXT}}, {{.LIB_EXT}} for cross-platform support

**Task Naming Convention**:

- Language tasks: `task {language}:{action}` (e.g., rust:build, python:test, node:lint)
- Workflow tasks: `task {workflow}:{scope}` (e.g., build:all, test:all:fast, lint:check)
- Tool tasks: `task {tool}:{action}` (e.g., version:sync, pdfium:install)
- Variants: `:dev`, `:release`, `:ci`, `:fast`, `:check`

**Task Description Standards**:

- Write task `desc` field for every task (used in `task --list`)
- Be descriptive: "Build Rust core in release mode" not just "Build Rust"
- Include BUILD_PROFILE if relevant: "Lint Python (uses ruff, mypy)"

**Cross-Platform Considerations**:

- Test commands on Windows, Linux, macOS before committing
- Use Taskfile path conventions (forward slashes converted automatically)
- Set `platforms: [linux, darwin]` or `platforms: [windows]` if OS-specific
- Use ENV variables for library paths: LD_LIBRARY_PATH (Linux), DYLD_LIBRARY_PATH (macOS)

**Error Handling**:

- Always set `requires: ["task"]` or ensure task is available
- Use `ignore_error: true` sparingly; prefer explicit error handling
- Run pre-flight checks (e.g., version checks) in setup tasks
- Clear error messages for dependencies (e.g., "Install Rust via rustup")

**Caching & Performance**:

- Leverage {{.NUM_CPUS}} for parallel builds: `cargo build -j {{.NUM_CPUS}}`
- Cache dependencies between CI runs (Cargo.lock, npm-lock.yaml, etc.)
- Use `:fast` variants for quick validation; full tests in separate task
- Reuse tasks: `task rust:build` instead of duplicating build commands

**Documentation & Discovery**:

- Run `task --list` to see all available tasks (requires good desc fields)
- Run `task {language}:` to list all tasks in a namespace
- Update README when adding new major workflows
- Link to task files from docs when documenting CLI usage
