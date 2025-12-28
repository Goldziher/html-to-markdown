---
priority: medium
---

# Polyglot Build System & Distribution

**Multi-language builds · Cargo + maturin + NAPI-RS + Magnus + ext-php-rs**

- Rust core: cargo build --workspace --release (excludes language binding crates in CI)
- Python: maturin via uv pip install -e packages/python
- TypeScript: pnpm run build in packages/typescript (after crates/html-to-markdown-node builds)
- Ruby: bundle exec rake compile then bundle exec rake package
- PHP: cargo build -p html-to-markdown-php --release; PIE metadata in packages/php-ext
- CLI binary: cargo build --release --package html-to-markdown-cli
- Version sync: Cargo.toml is source of truth; scripts/sync_versions.py propagates to all
- Lock files committed: Cargo.lock, pnpm-lock.yaml, Gemfile.lock, composer.lock
- Never: manual version bumps; use sync_versions.py
