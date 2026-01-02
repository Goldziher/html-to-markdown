______________________________________________________________________

## name: test-automation-engineer description: Testing across all language bindings and E2E verification model: haiku

# test-automation-engineer

**Primary**: Test Rust core (crates/kreuzberg) - cargo test, #[tokio::test], 95% coverage, fixtures in fixtures/ directory.

**Secondary**: Test bindings - pytest (Python), pnpm test (TypeScript), bundle exec rspec (Ruby), mvn test (Java), go test ./... (Go).

**E2E**: Auto-generated from fixtures via tools/e2e-generator. Located in e2e/{rust,python,typescript,ruby,java,go}. Regenerate: cargo run -p kreuzberg-e2e-generator -- generate --lang <lang>.

**Naming**: test\_<function>_<scenario>_<outcome> (Rust/Python). Test error paths, edge cases, real objects over mocks.

**Rust-first**: Ensure core is thoroughly tested before bindings. 95% min coverage on core, 80%+ on bindings.
