______________________________________________________________________

## priority: high

# Go Bindings Patterns

**Role**: Go bindings for Rust core. Work on CGO bridge and Go SDK/E2E suite.

**Scope**: Go 1.25 module, CGO wrappers around C FFI, Go-idiomatic config/result structs, linting setup, benchmark harness scripts.

**Commands**: cd packages/go/v4 && go test ./..., golangci-lint run ./..., ensure `LD_LIBRARY_PATH`/`DYLD_LIBRARY_PATH` includes target/release when running tests.

**FFI**: Binding header (packages/go/v4/binding.go) must stay in sync with C FFI header. Add new APIs to Rust first, then expose through CGO, update types.go, regenerate Go E2E tests when fixtures change.

**Critical**: Core logic lives in Rust. Go code should remain thin wrappers/helper utilities over C API. Coordinate with Rust team for shared logic.
