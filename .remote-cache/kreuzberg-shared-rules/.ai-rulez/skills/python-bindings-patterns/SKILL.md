______________________________________________________________________

## priority: critical

# Python Bindings Patterns

**Role**: Python bindings for Rust core. Work on PyO3 bridge and Python wrapper packages.

**Scope**: PyO3 FFI, Python-idiomatic API, Python-specific extensions, postprocessors.

**Commands**: maturin develop, pytest, ruff format/check.

**Critical**: Core logic lives in Rust. Only Python code for bindings, Python-specific extensions, or API wrappers. If core logic needed, coordinate with Rust team.

**Principles**: Function-based tests only, 95% coverage, builtin imports at top, no docstrings in private/test files.
