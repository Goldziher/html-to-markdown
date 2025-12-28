---
priority: medium
---

# Code Quality with Prek

**Prek pre-commit hooks · No bare commits without linting/formatting**

- Use prek (NOT pre-commit) for pre-commit hooks
- Configuration: .prek-config.yaml (hooks: ruff, clippy, cargo fmt, mypy, rubocop, etc.)
- Install: prek install && prek install --hook-type commit-msg
- Run: prek run --all-files to check everything before commit
- Rust: cargo fmt --all, clippy with -D warnings
- Python: ruff check, ruff format, mypy --strict
- Ruby: rubocop --config ./.rubocop.yml, rbs validate, steep check
- PHP: phpstan analyse, phpcs/phpcbf
- TypeScript: biome check, biome format
- Never: skip hooks (--no-verify); enforce quality gates
