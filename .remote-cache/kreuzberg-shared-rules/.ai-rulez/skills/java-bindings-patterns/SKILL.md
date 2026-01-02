______________________________________________________________________

## priority: high

# Java Bindings Patterns

**Role**: Java bindings for Rust core. Work on C FFI bridge and Java wrapper packages.

**Scope**: Java 25 Foreign Function & Memory API (FFM/Panama), Java-idiomatic API, JUnit 5 tests, Javadoc.

**Architecture**: Java FFM API → C FFI library → Rust core. No JNI, modern Foreign Function API.

**Commands**: cd packages/java && mvn clean compile test, mvn checkstyle:check, mvn spotless:apply.

**E2E Tests**: Auto-generated from fixtures via e2e-generator. Regenerate tests when fixtures change.

**Critical**: Core logic lives in Rust. Java only for FFI bindings/wrappers. If core logic needed, coordinate with Rust team.

**Key files**: Core.java (high-level API), CoreFFI.java (FFI bindings), config/\* (builder pattern), Exception.java (exception hierarchy).

**Code quality**: Zero Checkstyle/PMD warnings, use mvn checkstyle:check and mvn spotless:apply.
