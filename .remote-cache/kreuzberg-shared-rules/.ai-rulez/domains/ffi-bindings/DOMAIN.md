# FFI Bindings Layer Domain

## Purpose

The FFI bindings layer provides C-compatible foreign function interfaces for languages that require direct FFI rather than managed binding frameworks. This domain manages low-level pointer marshaling, memory safety guarantees, and cross-platform shared library distribution for Java, Go, C#, and other languages using C-compatible FFI.

## Scope and Responsibilities

- Design and implement crates/\*-ffi for C-compatible exports
- Maintain pointer validation, SAFETY documentation, and error conversion at FFI boundaries
- Ensure platform-specific library builds (.dll, .dylib, .so) for Windows, macOS, Linux
- Provide isolated FFI modules with explicit error handling and no memory leaks
- Support Java JNI integration through FFI (packages/java with cargo build -p \*-ffi)
- Support Go cgo binding integration (packages/go using C bindings)
- Support C# P/Invoke bindings (packages/csharp using \*-ffi)
- Manage cross-platform compilation targets and build artifact distribution
- Document C function signatures, pointer requirements, and lifetime guarantees

## Referenced Agents

- None currently (FFI layer is library code managed by rust-core-engineer and language binding engineers)

## Referenced Skills

- **platform-support-cross-platform-compatibility**: Windows/Linux/macOS detection, EXE_EXT/LIB_EXT variables, LD_LIBRARY_PATH/DYLD_LIBRARY_PATH configuration, cross-platform build strategies

## Referenced Rules

- **polyglot-build-system-distribution**: Cargo FFI build for Java/Go/C#, platform-specific library builds, version synchronization across manifests

## Interaction Points

- **Receives from**: rust-core domain (provides Rust implementation via FFI)
- **Provides to**: language-bindings domain (Java, Go, C# use FFI layer), build-distribution domain (artifact publishing)
- **Coordinates with**: quality-verification for FFI safety testing, organizational for cross-language standards

## Critical Files This Domain Manages

- `crates/*-ffi/src/lib.rs` (C-compatible FFI layer)
- `crates/*-ffi/src/error.rs` (Error conversion at FFI boundary)
- `crates/*-ffi/include/*.h` (C header definitions)
- `packages/java/build.gradle.kts` (JNI Cargo integration)
- `packages/go/v*/cgo_bindings.go` (Go cgo wrapper)
- `packages/csharp/PInvokeBindings.cs` (C# P/Invoke definitions)
