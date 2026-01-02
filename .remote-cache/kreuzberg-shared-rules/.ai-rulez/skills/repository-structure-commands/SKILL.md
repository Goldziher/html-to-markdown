______________________________________________________________________

## priority: high

# Repository Structure & Commands

**Core**: crates/kreuzberg (Rust 2024, standalone library, Edition 2024)
**Bindings**: Python (PyO3/maturin), TypeScript (NAPI-RS/pnpm), Ruby (Magnus/rake), Java (FFM API/Maven), Go (cgo/go), Elixir (Rustler/mix), C# (FFI/dotnet), PHP (FFI/composer)
**Task Commands**: task setup, task build, task test, task lint, task format
**All E2E generated**: cargo run -p kreuzberg-e2e-generator -- generate --lang \<rust|python|typescript|ruby|java|go|elixir>
**Version sync**: task sync-versions (syncs Cargo.toml → all manifests)
