# Rust Smoke Test

Minimal binary to confirm the `html-to-markdown-rs` crate works from crates.io
and from the local workspace.

## 1. Test crates.io

```bash
cargo run --manifest-path examples/rust-smoke/Cargo.toml
```

## 2. Test the local crate

```bash
cp examples/rust-smoke/.cargo/config.local-template.toml examples/rust-smoke/.cargo/config.toml
cargo run --manifest-path examples/rust-smoke/Cargo.toml
rm examples/rust-smoke/.cargo/config.toml
```

The temporary config patches crates.io to use `../../crates/html-to-markdown`.
The `.gitignore` already excludes `examples/**/.cargo/config.toml` so nothing
accidentally lands in a commit.
