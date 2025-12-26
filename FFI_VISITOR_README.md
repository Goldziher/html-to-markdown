# FFI Visitor Adapter Design - Complete Overview

## Document Suite

This design documentation consists of four documents:

### 1. **FFI_VISITOR_ADAPTER_DESIGN.md** (Primary Architecture)
The comprehensive architecture guide covering:
- Problem statement and design principles
- C type definitions and callback signatures
- Rust implementation of FfiVisitorAdapter
- Type conversion functions (Rust ↔ C)
- HtmlVisitor trait implementation examples
- Memory management strategy
- Error handling approach
- Testing strategy
- Safety checklist
- Performance considerations
- Public FFI functions

**Read this first** for a complete understanding of the system.

### 2. **FFI_VISITOR_IMPLEMENTATION_EXAMPLES.md** (Code Ready)
Production-ready code snippets including:
- Complete C header file with all type definitions
- Full FfiVisitorAdapter struct implementation
- 4+ HtmlVisitor method implementations (simple, complex, with arrays)
- Complete marshalling functions
- Integration test example (C code)

**Use this when implementing** the actual `visitor/` module.

### 3. **FFI_VISITOR_TECHNICAL_CHALLENGES.md** (Deep Dive)
Detailed analysis of 8 key technical challenges:
1. BTreeMap to C arrays
2. NodeContext conversion
3. Calling C function pointers from Rust
4. NULL pointer safety
5. Error handling across FFI
6. Lifetime management
7. Ownership across FFI boundary
8. Thread safety & panic propagation

Each challenge includes:
- Problem statement
- Why it's hard
- Solution architecture (with diagrams)
- Critical implementation steps
- Common pitfalls
- Memory lifecycle details

**Read this for understanding** the "why" behind design decisions.

### 4. **FFI_VISITOR_README.md** (This Document)
Overview and navigation guide.

## Quick Start

### For Implementation
1. Read FFI_VISITOR_ADAPTER_DESIGN.md sections 1-4
2. Use FFI_VISITOR_IMPLEMENTATION_EXAMPLES.md as code template
3. Refer to FFI_VISITOR_TECHNICAL_CHALLENGES.md when encountering issues
4. Copy/adapt code snippets as needed

### For Code Review
1. Check safety invariants in FFI_VISITOR_ADAPTER_DESIGN.md section 4.1
2. Verify memory management follows section 5
3. Ensure panic handling per FFI_VISITOR_TECHNICAL_CHALLENGES.md section 3
4. Validate ownership model per section 7

### For Integration
1. Use public FFI functions from section 9 of main design doc
2. Implement C header from FFI_VISITOR_IMPLEMENTATION_EXAMPLES.md
3. Bind to language runtime (Go, Java, C#)
4. Write integration tests per FFI_VISITOR_ADAPTER_DESIGN.md section 10

## Key Design Decisions

### Memory Ownership Model
```
Input (Rust → C):
  All input pointers are Rust-owned
  C callbacks must not modify or free
  Lifetime: duration of callback only

Output (C → Rust):
  Custom/Error result strings are C-allocated
  Rust takes ownership after callback returns
  Rust deallocates via CString::from_raw + drop
```

### Error Handling
- C callbacks return: 1 = success, 0 = error
- Panics caught via catch_unwind
- Error messages stored in thread-local LAST_ERROR
- VisitResult encoded as (kind: u8, value: *mut c_char)

### Thread Safety
- FfiVisitorAdapter marked !Send + !Sync
- Each thread creates own visitor instance
- No synchronization (caller responsible)
- Panic in callback disables future callbacks

### Performance
- Zero-cost when no callbacks registered (Option is None)
- Minimal copying (mostly pointer passing)
- RAII ensures cleanup even on error
- Estimated 1-5 µs overhead per callback

## Implementation Checklist

### Phase 1: Type Definitions
- [ ] Define CNodeContext struct with #[repr(C)]
- [ ] Define CAttributePair struct
- [ ] Define CVisitResult struct
- [ ] Define CVisitorCallbacks struct with function pointer fields
- [ ] Generate C header via cbindgen

### Phase 2: Core Adapter
- [ ] Implement FfiVisitorAdapter struct
- [ ] Implement FfiVisitorAdapter::new()
- [ ] Implement call_c_function() with panic guard
- [ ] Add !Send + !Sync marker

### Phase 3: Marshalling
- [ ] Implement node_context_to_c()
- [ ] Implement free_cnode_context()
- [ ] Implement btreemap_to_c_array()
- [ ] Implement free_c_attributes_array()
- [ ] Implement c_visit_result_to_rust()

### Phase 4: Trait Implementation
- [ ] Implement HtmlVisitor for FfiVisitorAdapter
- [ ] Implement all 30+ visitor methods
- [ ] Test each method with mock callbacks
- [ ] Verify cleanup on error paths

### Phase 5: Public API
- [ ] Implement html_to_markdown_visitor_new()
- [ ] Implement html_to_markdown_visitor_free()
- [ ] Implement html_to_markdown_convert_with_visitor()
- [ ] Generate C header

### Phase 6: Testing
- [ ] Unit tests for marshalling functions
- [ ] Unit tests for panic handling
- [ ] Unit tests for NULL pointer handling
- [ ] Integration tests with C callbacks
- [ ] Memory leak detection (valgrind)
- [ ] Thread safety verification

### Phase 7: Documentation
- [ ] Document ownership model in header
- [ ] Document thread safety constraints
- [ ] Document callback error handling
- [ ] Add examples for each language binding
- [ ] Create troubleshooting guide

## File Structure

```
crates/html-to-markdown-ffi/src/
├── lib.rs                    # Existing FFI entry points
├── conversion.rs             # Existing conversion functions
├── error.rs                  # Existing error handling
├── strings.rs                # Existing string utilities
├── profiling.rs              # Existing profiling
│
└── visitor/                  # NEW MODULE
    ├── mod.rs                # Module exports
    ├── ffi.rs                # FfiVisitorAdapter implementation
    ├── marshalling.rs        # Type conversions (Rust ↔ C)
    ├── callbacks.rs          # C callback type definitions
    └── callbacks.h           # C header (generated by cbindgen)
```

## Integration Points

### With html-to-markdown-rs (Core)
- Uses HtmlVisitor trait from visitor.rs
- Uses NodeContext, VisitResult types
- Uses NodeType enum (convert to u8)

### With FFI Layer
- Adds new public C functions:
  - html_to_markdown_visitor_new()
  - html_to_markdown_visitor_free()
  - html_to_markdown_convert_with_visitor()

### With Language Bindings
- Go: Wraps C functions, passes function pointers
- Java: JNI bridge to C functions
- C#: P/Invoke to C functions
- Ruby: FFI gem to C functions

## Testing Strategy

### Unit Tests (Rust)
```rust
#[test]
fn test_node_context_conversion() { ... }
#[test]
fn test_btreemap_empty() { ... }
#[test]
fn test_btreemap_multiple() { ... }
#[test]
fn test_panic_disables_callbacks() { ... }
#[test]
fn test_null_callback() { ... }
```

### Integration Tests (C)
```c
void test_visitor_callback_invoked() { ... }
void test_visitor_attributes() { ... }
void test_visitor_panic_isolation() { ... }
void test_visitor_error_handling() { ... }
```

### Memory Tests
```bash
valgrind --leak-check=full ./test_visitor_ffi
```

## Safety Analysis

### Unsafe Blocks
Every unsafe block must have a SAFETY comment explaining:
1. Why unsafe is necessary
2. What invariants must hold
3. What callers must ensure

Example:
```rust
unsafe {
    // SAFETY: callbacks must be non-null (validated in new())
    // and must point to valid C functions. Free is called only
    // after callback returns, so pointers remain valid.
    free_cnode_context(&c_ctx);
}
```

### Soundness Checklist
- [ ] No buffer overflows (CString validates null bytes)
- [ ] No use-after-free (RAII cleanup)
- [ ] No double-free (ownership clearly defined)
- [ ] No memory leaks (every allocation paired with deallocation)
- [ ] No data races (!Send + !Sync enforced)
- [ ] No undefined behavior (catch_unwind guards panics)

## Performance Considerations

### Benchmarks to Measure
1. **Conversion without visitor**: baseline
2. **Conversion with no-op visitor**: overhead of visitor dispatch
3. **Conversion with callback visitor**: overhead + callback time
4. **Large attributes**: BTreeMap → C array scaling
5. **Deep DOM tree**: recursion overhead with callbacks

### Optimization Opportunities
1. **Batch Attributes**: If document has many elements with attributes
2. **String Interning**: Cache tag names (common in documents)
3. **Fast Path**: Skip all checks if all callbacks are None
4. **Lazy Conversion**: Only convert fields that callback needs

## Known Limitations

1. **Not Thread-Safe**: Each thread needs own visitor instance
2. **Panic Isolation**: If callback panics, visitor is disabled
3. **Synchronous**: C callbacks block conversion thread
4. **Single Visitor**: Cannot chain/compose multiple visitors
5. **Memory Overhead**: Each conversion allocates new CNodeContext structs
6. **No Custom Allocators**: Uses system malloc for C strings

## Future Enhancements

### Short Term
1. [ ] Performance benchmarking
2. [ ] Language-specific binding examples
3. [ ] Documentation translations
4. [ ] FAQ for common issues

### Medium Term
1. [ ] Thread-safe visitor (with Arc<Mutex<>>)
2. [ ] Visitor composition (multiple visitors)
3. [ ] Async callback support
4. [ ] Custom allocator interface

### Long Term
1. [ ] Streaming mode (parse + convert on-the-fly)
2. [ ] Parallel conversion (multiple threads)
3. [ ] GPU acceleration for large documents
4. [ ] WebAssembly visitor callbacks

## Related Documentation

- **html-to-markdown-rs visitor.rs**: Core trait definition
- **CLAUDE.md**: AI assistant guidelines
- **FFI Security**: https://doc.rust-lang.org/nomicon/ffi.html
- **Rust Panic Behavior**: https://doc.rust-lang.org/nomicon/panic-safety.html

## Questions & Support

### Common Questions
**Q: Why not use async/await for callbacks?**
A: C doesn't have async support. Callbacks must be synchronous.

**Q: Can I share a visitor across threads?**
A: No. Mark as !Send + !Sync. Create one per thread.

**Q: What if my callback allocates a string?**
A: C callback owns the allocation. Rust takes ownership when callback returns.

**Q: What if my callback never returns?**
A: Conversion blocks forever. Document that callbacks must be non-blocking.

**Q: Can I call convert_with_visitor from my callback?**
A: No. This causes reentrancy issues and potential deadlock.

## Document Versions

- **Version**: 1.0
- **Created**: 2025-12-26
- **Last Updated**: 2025-12-26
- **Status**: Design Complete (implementation pending)
- **Author**: Claude Code (Anthropic)

## License

This design documentation is provided as part of the html-to-markdown project.
See main repository LICENSE file for details.

---

## Next Steps

1. **Review** FFI_VISITOR_ADAPTER_DESIGN.md
2. **Study** FFI_VISITOR_TECHNICAL_CHALLENGES.md
3. **Use** FFI_VISITOR_IMPLEMENTATION_EXAMPLES.md as coding reference
4. **Implement** according to implementation checklist above
5. **Test** with unit + integration tests
6. **Benchmark** performance impact
7. **Release** to language bindings (Go, Java, C#, etc.)
