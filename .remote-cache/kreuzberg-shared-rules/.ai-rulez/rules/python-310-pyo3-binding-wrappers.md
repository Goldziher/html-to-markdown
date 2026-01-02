______________________________________________________________________

## priority: medium

# Python 3.10+ - PyO3 Binding Wrappers

**Python 3.10+ · PyO3 minimal wrappers · Type-safe · pytest · 80%+ coverage**

- Target Python 3.10+; match/case, union types (X | Y), structural pattern matching
- PyO3 bindings minimal wrappers: expose Rust API cleanly without logic duplication
- Type stubs in \_rust.pyi provide type info for Rust bindings
- Full type hints: mypy --strict, no Any types, ParamSpec for decorators
- Testing: pytest in packages/python/tests; 80%+ coverage with pytest-cov
- Package distribution: PyPI via maturin (uv pip install -e packages/python)
- Never: business logic in Python wrappers; that belongs in Rust
- Use Haiku 4.5 for binding engineering and integration issues
