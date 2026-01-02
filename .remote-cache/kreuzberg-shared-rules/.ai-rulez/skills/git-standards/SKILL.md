______________________________________________________________________

## priority: medium

# Git Commit Standards

**Conventional Commits 1.0.0 · Pre-commit hooks enforce quality**

- Commit message format: feat/fix/docs/refactor/test/chore(scope): description
- Scopes: rust-core, py-binding, ts-binding, rb-binding, php-binding, build, ci, docs
- Example: fix(rust-core): handle nested lists in markdown output
- Example: feat(py-binding): expose sanitization options to Python API
- Example: test(ts-binding): add parametrized tests for edge cases
- Commits MUST pass prek hooks: fmt, lint, tests (at min)
- NEVER include AI signatures in commits; commits authored by humans
- NEVER force push to main/development; require PR reviews
