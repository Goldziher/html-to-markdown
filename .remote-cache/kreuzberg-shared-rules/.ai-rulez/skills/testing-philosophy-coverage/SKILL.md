______________________________________________________________________

## priority: critical

# Testing Philosophy & Coverage

**Three-tier**: unit (80-95%), integration (real DB/services), E2E (smoke/full)

**Real infrastructure in tests**: PostgreSQL, Redis, not mocks

**Rust**: cargo test, #[tokio::test], 95% coverage (tarpaulin), test error paths, edge cases, no panics

**Python**: Function-based tests only (\*_test.py), pytest fixtures, 95% coverage. Structure: tests/{core,features,integration,interfaces,extractors,ocr,utils,e2e}. Test async+sync, error paths. Naming: test_<function>_<scenario>_<outcome>. NEVER mock anyio/asyncio, prefer real objects (tmp_path).

**TypeScript**: .spec.ts next to source files, vitest, 80%+ coverage

**Go**: \*\_test.go with \_test package suffix (black-box), table-driven with t.Run(), 80%+ business logic, go test -race

**Ruby**: RSpec, describe/context/it blocks, 80%+ coverage

**Java**: JUnit 5, @Test methods, AssertJ, 80%+ coverage. E2E auto-generated from fixtures.
