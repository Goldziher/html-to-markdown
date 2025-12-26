# Visitor FFI Dependencies & Key Decisions

## Current FFI Crate Dependencies

### Direct Dependencies
```toml
html-to-markdown-rs = { workspace = true, features = ["inline-images", "metadata"] }
serde_json = "1.0"
pprof = { version = "0.15", features = ["flamegraph"], optional = true }  # Unix/Linux only
```

### Transitive Dependencies (via html-to-markdown-rs)
```toml
html5ever = "0.36"           # HTML5 parsing
markup5ever_rcdom = "0.36"   # DOM representation
regex = "1.12"               # Pattern matching
once_cell = "1.21"           # OnceLock for statics
thiserror = "2.0"            # Error types
base64 = "0.22"              # Base64 encoding
encoding_rs = "0.8"          # Character encoding
serde = { version = "1.0" }  # Serialization framework
```

## NO New Dependencies Required for Visitor FFI

**Good news:** The visitor FFI layer can be implemented using existing dependencies!

### Why No New Dependencies Needed

1. **Trait Objects Already Available**: The `HtmlVisitor` trait is defined in core library
   - Location: `crates/html-to-markdown/src/visitor.rs`
   - Already feature-gated in core
   - Supports RefCell<dyn HtmlVisitor> for interior mutability

2. **Standard Rust Types Sufficient**:
   - `std::cell::RefCell` - Interior mutability (already used for LAST_ERROR)
   - `std::rc::Rc` - Reference counting (standard)
   - `std::ffi::{CStr, CString}` - String marshalling (already used)
   - `std::os::raw::*` - C types (already used)

3. **Panic Safety Available**:
   - `guard_panic()` from `html_to_markdown_rs::safety` (already used)
   - No additional error handling crate needed

4. **Callback Pattern Compatible**:
   - Function pointers (`extern "C"`) are language feature
   - No callback trait library required
   - User data (`void*`) for context is standard C pattern

## Key Architectural Decisions

### Decision 1: Thread-Local Error Storage

**Pattern**: Reuse existing `LAST_ERROR` from error.rs

**Rationale**:
- Consistent with existing FFI functions
- Per-thread error messages (safe for multi-threaded environments)
- No additional synchronization overhead
- Callbacks run in same thread as FFI function call

**Trade-off**: Error pointer invalidated by next FFI call (documented in C API)

---

### Decision 2: Callback Function Pointers vs Trait Objects

**Chosen**: C function pointers + user_data (not Rust trait objects)

**Rationale**:
- Trait objects (`dyn HtmlVisitor`) not C-compatible
- Function pointers are low-level C FFI compatible
- Supports opaque user_data for state
- Callback style familiar to C developers

**Implementation**:
```rust
// C-compatible
pub type VisitTextFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    text: *const c_char,
) -> c_int;

// NOT C-compatible (trait objects)
// pub type VisitTextFn = Box<dyn Fn(&str) -> VisitResult>;
```

---

### Decision 3: Opaque Handle vs Direct Pointer

**Chosen**: Opaque `VisitorHandle` struct wrapping internal state

**Rationale**:
- Prevents C code from inspecting/corrupting Rust data
- Type-safe: Can't pass arbitrary pointers
- Future-proof: Can change internals without breaking API

**Implementation**:
```rust
#[repr(C)]
pub struct VisitorHandle {
    // Private fields - completely opaque to C
}

// Usage:
let handle: *mut VisitorHandle = html_to_markdown_visitor_create(...);
// C code cannot inspect handle.visitor or handle.user_data
```

---

### Decision 4: Single vs Multiple Visitor Instances

**Support**: Multiple concurrent visitors (per thread)

**Rationale**:
- Each thread can use separate visitor
- Thread-local LAST_ERROR already supports this
- No global visitor state in FFI layer
- Language bindings can hold multiple handles

**Limitation**: Callbacks must be thread-safe or use thread-local state

---

### Decision 5: Visitor Result Handling

**Pattern 1 - Continue**: Return 0, no output

**Pattern 2 - Skip**: Return 2, element ignored

**Pattern 3 - PreserveHtml**: Return 3, raw HTML included

**Pattern 4 - Error**: Return 4, conversion stops (error set via LAST_ERROR)

**Limitation**: Custom(String) result deferred (requires additional marshalling)

**Future Enhancement**: Add output buffer parameter for Custom results
```rust
// Phase 2 enhancement:
pub type VisitTextFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    text: *const c_char,
    output: *mut *mut c_char,  // For Custom results
) -> c_int;
```

---

### Decision 6: Callback Granularity

**Chosen**: 60+ specific visitor methods (one per HTML element type)

**Rationale**:
- Matches core library's HtmlVisitor trait
- Allows fine-grained control
- Zero overhead for unimplemented callbacks (NULL function pointer)
- C code only implements needed callbacks

**Alternative Rejected**: Single generic callback
- Would require C code to dispatch based on tag name
- More boilerplate in C consumer code

---

### Decision 7: Feature Gating

**Approach**: Visitor support behind feature flag

**Cargo.toml**:
```toml
[features]
default = ["metadata", "visitor"]  # Enabled by default
visitor = ["html-to-markdown-rs/visitor"]
```

**Rationale**:
- Core library already feature-gates visitor
- No performance impact if disabled
- Can be disabled for minimal builds
- Consistent with metadata feature pattern

---

### Decision 8: Memory Ownership

**Pattern**: Callbacks don't allocate; adapter allocates

**Rationale**:
- C callbacks return error codes (integers), not owned strings
- String allocation happens in Rust layers only
- Simpler for C consumers
- No double-free or memory leak risks

**Consequence**: Custom output results deferred to Phase 2

---

### Decision 9: NodeContext Marshalling

**Approach**: Pass individual fields to C callback

**Rationale**:
- BTreeMap not C-compatible
- Attributes marshalled to key/value arrays
- Optional fields passed as NULL pointers
- Minimal copying overhead

**Trade-off**: Slightly larger callback signature

---

### Decision 10: Callback Registration Timing

**Pattern**: All callbacks registered at visitor creation time

**Rationale**:
- Simpler API (single creation call)
- No dynamic callback registration
- Callbacks immutable after creation
- Better performance (no runtime checks)

**Alternative Rejected**: Dynamic callback registration
- Would require mutable visitor handle
- More complex API
- Slower (per-callback lookups)

---

## Dependency Chain for Feature

```
html-to-markdown-ffi
  ├─ html-to-markdown-rs (with "visitor" feature)
  │   ├─ html5ever 0.36
  │   ├─ regex 1.12
  │   └─ ... other core deps ...
  ├─ serde_json 1.0          (for metadata)
  └─ pprof 0.15              (optional, Unix/Linux only)
```

**New feature doesn't add external dependencies!**

---

## Core Library Feature Support

### Current State
```rust
// In crates/html-to-markdown/Cargo.toml

[features]
default = ["metadata"]
metadata = ["html-to-markdown-rs/metadata"]
visitor = ["html-to-markdown-rs/visitor"]  # Already exists
inline-images = ["html-to-markdown-rs/inline-images"]
```

**Status**: ✓ Visitor feature already defined in core

### FFI Enablement
```toml
# In crates/html-to-markdown-ffi/Cargo.toml

[dependencies]
html-to-markdown-rs = {
    workspace = true,
    features = ["inline-images", "metadata", "visitor"]  # Just add "visitor"
}

[features]
default = ["metadata", "visitor"]
metadata = ["html-to-markdown-rs/metadata"]
visitor = ["html-to-markdown-rs/visitor"]
profiling = ["dep:pprof"]
```

**Action Required**: Add "visitor" to FFI dependencies

---

## Integration Points with Existing Code

### 1. Error Module (error.rs)
**Reuse**: `set_last_error()`, `last_error_ptr()`, `capture_error()`

```rust
use crate::error::{set_last_error, capture_error};

// In visitor callback:
capture_error(ConversionError::Other("visitor error".to_string()));
```

### 2. String Module (strings.rs)
**Reuse**: `string_to_c_string()`, `bytes_to_c_string()`

```rust
use crate::strings::string_to_c_string;

// In callback marshalling:
match string_to_c_string(rust_str, "context") {
    Ok(c_string) => c_string.as_ptr(),
    Err(err) => {
        capture_error(err);
        std::ptr::null()
    }
}
```

### 3. Profiling Module (profiling.rs)
**Reuse**: `maybe_profile()`

```rust
use crate::profiling;

match guard_panic(|| profiling::maybe_profile(|| convert_with_visitor(...))) {
    Ok(result) => { ... }
    Err(err) => { ... }
}
```

### 4. Core Library (html_to_markdown_rs)
**New**: Requires `convert_with_visitor()` function in core

**Status**: Depends on core library implementation

**Expected Signature**:
```rust
pub fn convert_with_visitor(
    html: &str,
    options: Option<&ConversionOptions>,
    visitor: Option<&mut dyn HtmlVisitor>,
) -> Result<String>
```

---

## Build Impact

### Build Time
- **No measurable increase** (no new dependencies)
- cbindgen re-run on changes to visitor.rs
- Same compilation speed as existing FFI functions

### Binary Size
- Minimal increase (visitor code only compiled with feature)
- With feature disabled: Zero bloat
- Typical increase with feature: <2% (rough estimate)

### Dependencies on Platform
```
Unix/Linux/macOS:
  html5ever → all platforms
  pprof → only if profiling feature enabled

Windows:
  html5ever → all platforms
  pprof → not compiled (not supported on Windows)
  profiling feature disabled
```

---

## Testing Dependencies

### Required for FFI Tests
- Standard Rust `#[test]`
- No additional test framework needed
- Can use `std::sync::Arc<Mutex<T>>` for thread-safe callback state tracking

### Example Test Structure
```rust
#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::ffi::CString;

    struct CallTracker {
        calls: Vec<String>,
    }

    #[test]
    fn test_visitor_callback() {
        let tracker = Arc::new(Mutex::new(CallTracker { calls: Vec::new() }));
        let tracker_ptr = Box::into_raw(Box::new(tracker.clone()));

        // Create visitor with callback that records calls
        // Convert HTML
        // Verify tracker.calls contains expected values
    }
}
```

---

## Performance Characteristics

### Callback Overhead
- **Per callback**: ~5-10 C↔Rust boundary crossings
  - C function pointer dereference
  - CString creation/drop
  - Integer comparisons for result codes

- **Typical HTML**: 100-500 callbacks per document
  - Negligible if callbacks do minimal work
  - Acceptable for filtering/inspection use cases

### Optimization Opportunities
1. **Batch Marshalling**: Marshal multiple attributes once
2. **String Interning**: Cache CString for repeated text
3. **Callback Result Caching**: Cache visitor decisions
4. **Lazy Attribute Conversion**: Only marshal accessed attributes

**Priority**: Monitor performance with real-world usage

---

## Version Compatibility

### Rust Version
- **Minimum**: 1.85 (workspace requirement)
- **Target**: 2024 edition (workspace uses)
- **No compatibility issue**: Standard Rust + std library only

### Core Library Version Tie
- FFI version must match core library version
- Workspace version management: Single source of truth
- Cargo.lock prevents version mismatches

### ABI Stability
- C ABI stable across Rust versions (extern "C" guarantees)
- Binary compatibility: Consumers don't recompile Rust
- FFI upgrades: Need recompile of language bindings

---

## Summary Table

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **Dependencies** | None new | Existing Rust std lib + core lib sufficient |
| **Error Handling** | Reuse LAST_ERROR | Thread-safe, consistent with existing FFI |
| **Callbacks** | C function pointers | FFI-compatible, familiar to C developers |
| **Handles** | Opaque struct | Type-safe, prevents misuse |
| **Concurrency** | Per-thread visitors | Multi-threaded safe, thread-local errors |
| **Results** | Int discriminants (Phase 1) | Simple, extensible (Phase 2: output buffers) |
| **Granularity** | 60+ methods | Matches core library, zero overhead |
| **Features** | Behind flag | Optional, no performance impact when disabled |
| **Ownership** | Rust allocates | Simpler for C, fewer memory bugs |
| **Registration** | Creation-time | Simpler API, better performance |

---

## Implementation Priority

1. **High Priority** (Core FFI layer)
   - Visitor types & callbacks
   - Create/free functions
   - Element callbacks (most common: element_start, text, link, image, heading)

2. **Medium Priority** (Extended support)
   - All 60+ visitor methods
   - Metadata extraction with visitor
   - Comprehensive tests

3. **Low Priority** (Enhancements)
   - Custom result string output
   - Callback result caching
   - Performance optimizations

---

## References

**Files Providing Dependencies**:
- `/Users/naamanhirschfeld/workspace/html-to-markdown/Cargo.toml` - Workspace versions
- `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown-ffi/Cargo.toml` - FFI deps
- `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/visitor.rs` - Trait definition

**Documentation**:
- `FFI_ARCHITECTURE_REPORT.md` - Full architecture analysis
- `VISITOR_FFI_IMPLEMENTATION_GUIDE.md` - Step-by-step implementation
- `FFI_QUICK_REFERENCE.md` - Code patterns
