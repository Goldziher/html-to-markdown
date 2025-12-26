# FFI Visitor Adapter - Architecture Diagrams & Flow Charts

## System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         C Language Bindings                             │
│                      (Go, Java, C#, Elixir, etc.)                       │
└────────────────────────────┬────────────────────────────────────────────┘
                             │
                    C FFI Layer (extern "C")
                             │
┌────────────────────────────▼────────────────────────────────────────────┐
│            html-to-markdown-ffi (C-compatible exports)                   │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────────────── Public API ──────────────────────────────┐    │
│  │                                                                   │    │
│  │  html_to_markdown_visitor_new()                                 │    │
│  │  html_to_markdown_visitor_free()                                │    │
│  │  html_to_markdown_convert_with_visitor(html, visitor)           │    │
│  │                                                                   │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                          │                                              │
│      ┌───────────────────▼──────────────────────┐                      │
│      │    FfiVisitorAdapter (Rust struct)       │                      │
│      │                                          │                      │
│      │  - visit_element_start_fn: Option<...>  │                      │
│      │  - visit_element_end_fn: Option<...>    │                      │
│      │  - visit_text_fn: Option<...>           │                      │
│      │  - visit_link_fn: Option<...>           │                      │
│      │  - ... (30+ callback pointers)          │                      │
│      │  - user_data: *mut c_void               │                      │
│      │  - panic_occurred: bool                 │                      │
│      │                                          │                      │
│      │  Methods:                                │                      │
│      │  - new(callbacks, user_data)            │                      │
│      │  - call_c_function<F>(...) → Result     │                      │
│      └───────────────────────────────────────┘
│             │
│             │ implements
│             ▼
│      ┌─────────────────────────────────────────┐
│      │   HtmlVisitor (Rust trait)              │
│      │                                         │
│      │  fn visit_element_start(...)            │
│      │  fn visit_element_end(...)              │
│      │  fn visit_text(...)                     │
│      │  fn visit_link(...)                     │
│      │  ... (30+ methods)                      │
│      └─────────────────────────────────────────┘
│             │
│             │ dispatches to
│             ▼
│      ┌─────────────────────────────────────────┐
│      │  C Callback Functions                   │
│      │  (provided by caller)                   │
│      │                                         │
│      │  ffi_visit_element_start(user_data,    │
│      │                           ctx)          │
│      │  ffi_visit_text(user_data, ctx, text) │
│      │  ffi_visit_link(user_data, ctx,        │
│      │                 href, text, title)    │
│      │  ... (30+ function pointers)            │
│      └─────────────────────────────────────────┘
│
└──────────────────────────────────────────────────────────────────────────┘
                            │
                            │ uses
                            ▼
        ┌──────────────────────────────────────────┐
        │  html-to-markdown-rs (Rust core)         │
        │                                          │
        │  - HTML5 Parser (html5ever)              │
        │  - DOM Walker                            │
        │  - Markdown Converter                    │
        │  - Element-specific logic                │
        └──────────────────────────────────────────┘
```

## Data Flow: Single Element Conversion

```
Input HTML:
  <a href="https://example.com">Click here</a>

         │
         ▼
  ┌────────────────────┐
  │ Parse HTML         │ (html5ever)
  └────────────────────┘
         │
         ▼
  ┌─────────────────────────────┐
  │ Create NodeContext          │
  │ - node_type: Link           │
  │ - tag_name: "a"             │
  │ - attributes: {href: "..."}  │
  │ - depth: 1                  │
  │ - is_inline: true           │
  └─────────────────────────────┘
         │
         ▼
  ┌─────────────────────────────────────────┐
  │ Call visitor.visit_link()               │
  └─────────────────────────────────────────┘
         │
         ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ FfiVisitorAdapter.visit_link()                              │
  │                                                             │
  │ 1. Convert NodeContext → CNodeContext                       │
  │    - Clone tag_name → C string                              │
  │    - Clone attributes → C array                             │
  │                                                             │
  │ 2. Convert strings: href, text, title → C strings           │
  │                                                             │
  │ 3. Call C callback with panic guard:                        │
  │    callback(user_data, &c_ctx, href_c, text_c, title_c)    │
  │       │                                                     │
  │       ▼ (user's C code)                                     │
  │    // Custom C logic                                        │
  │    // return 1 for success, 0 for error                     │
  │                                                             │
  │ 4. Cleanup (LIFO):                                          │
  │    - drop(title_c)                                          │
  │    - drop(text_c)                                           │
  │    - drop(href_c)                                           │
  │    - free_cnode_context()                                   │
  │                                                             │
  │ 5. Return VisitResult                                       │
  │    - If callback succeeded: Continue                        │
  │    - If callback panicked: Error("panicked")                │
  │    - If callback returned 0: Error("returned 0")            │
  └─────────────────────────────────────────────────────────────┘
         │
         ▼
  ┌──────────────────────────────────────────┐
  │ Check VisitResult                        │
  ├──────────────────────────────────────────┤
  │ Continue    → Use default markdown       │
  │ Custom      → Use visitor's markdown     │
  │ Skip        → Omit element               │
  │ PreserveHtml→ Keep raw HTML              │
  │ Error       → Halt conversion            │
  └──────────────────────────────────────────┘
         │
         ▼
  Output Markdown:
  [Click here](https://example.com)
```

## Memory Allocation Timeline

```
Time ─────────────────────────────────────────────────────────────────→

      START
      │
      ├─ 1. CString::new("a")          [Allocate tag_name]
      │
      ├─ 2. btreemap_to_c_array()      [Allocate array + 2 strings]
      │      ├─ CString::new("href")
      │      └─ CString::new("https://...")
      │
      ├─ 3. CString::new("https://...")  [Allocate href_c]
      │
      ├─ 4. CString::new("Click here")   [Allocate text_c]
      │
      ├─ 5. Some(CString::new(""))        [Allocate title_c if Some]
      │
      │  ┌─────────────────────────────────────────┐
      │  │ LIVE: tag_name, parent_tag, attrs,      │
      │  │       href_c, text_c, title_c           │
      │  │                                         │
      │  │ ┌─────────────────────────────┐         │
      │  │ │ CALLBACK INVOKED            │         │
      │  │ │ (all pointers valid here)   │         │
      │  │ └─────────────────────────────┘         │
      │  └─────────────────────────────────────────┘
      │
      ├─ 6. drop(title_c)                [Deallocate title_c]
      │
      ├─ 7. drop(text_c)                 [Deallocate text_c]
      │
      ├─ 8. drop(href_c)                 [Deallocate href_c]
      │
      ├─ 9. free_c_attributes_array()    [Deallocate attr strings + array]
      │
      ├─ 10. free_cnode_context()        [Deallocate tag_name, parent_tag]
      │
      END
      └─ All pointers now invalid
```

## Error Handling Flow

```
                        visit_link() called
                                │
                    ┌───────────┴──────────┐
                    ▼                      ▼
              Some(callback)          None (no callback)
                    │                      │
                    ├─ Allocate ctx       └─→ Return Continue
                    │  (if error)
                    ├─ Return Error
                    │
                    ├─ Allocate href_c
                    │  (if error)
                    │  └─→ Free ctx, Return Error
                    │
                    ├─ Allocate text_c
                    │  (if error)
                    │  └─→ Free ctx, Return Error
                    │
                    ├─ Call C callback
                    │  (with catch_unwind)
                    │
                    ├─┬─ If panic caught:
                    │ │  ├─ Set panic_occurred = true
                    │ │  └─→ Return Error("panicked")
                    │ │
                    │ ├─ If return code == 0:
                    │ │  └─→ Return Error("returned 0")
                    │ │
                    │ └─ If return code != 0:
                    │    └─→ Continue
                    │
                    ├─ Free title_c
                    ├─ Free text_c
                    ├─ Free href_c
                    ├─ Free ctx
                    │
                    └─→ Return VisitResult
```

## Panic Safety Flow

```
FfiVisitorAdapter State:
  panic_occurred: bool = false

Visit sequence:

  visit_element_start()
    │
    ├─ panic_occurred == false? Yes
    │  ├─ Call C callback
    │  └─ Callback succeeds ✓
    │
    ├─ visit_text()
    │  ├─ panic_occurred == false? Yes
    │  ├─ Call C callback
    │  └─ Callback PANICS! 🔥
    │     ├─ catch_unwind catches panic
    │     ├─ Set panic_occurred = true
    │     └─ Return Error("panicked")
    │
    ├─ visit_element_end()
    │  ├─ panic_occurred == false? No! ❌
    │  ├─ skip callback entirely
    │  └─ Return Error("visitor disabled")
    │
    └─ RESULT: Remaining callbacks disabled, no cascading failures ✓
```

## Type Conversion Pipeline

```
Rust NodeContext              BTreeMap iter()           C Array
┌──────────────────┐         ┌───────────┐         ┌──────────────┐
│ node_type        │         │ "class"   │ clone   │ ptr[0].key   │──→"class"
│ tag_name         │ ─────→  │ "active"  │ ──────→ │ ptr[0].value │──→"active"
│ attributes       │         ├───────────┤ ↓       ├──────────────┤
│ depth            │         │ "id"      │ clone   │ ptr[1].key   │──→"id"
│ index_in_parent  │         │ "header"  │ ──────→ │ ptr[1].value │──→"header"
│ parent_tag       │         └───────────┘         └──────────────┘
│ is_inline        │
│                  │         CString::new()
└──────────────────┘         + into_raw()
                             + forget()

         │                          │                    │
         ▼                          ▼                    ▼
      Rust Owned              C Callback Receives    Rust Still Owns
      (on stack)              (read-only pointers)   (cleanup later)
```

## Ownership Boundary Diagram

```
╔════════════════════════════════════════════════════════════════════╗
║                      RUST MEMORY SPACE                              ║
║                                                                      ║
║  ┌──────────────────────────────────────────────────────────────┐  ║
║  │ HtmlVisitor Trait Implementation                            │  ║
║  │ (FfiVisitorAdapter)                                         │  ║
║  └──────────────────────────────────────────────────────────────┘  ║
║         │                                                            ║
║         │ Owns and manages:                                         ║
║         │                                                            ║
║  ┌──────▼──────────────────────────────────────────────────────┐  ║
║  │  • CNodeContext                      [Rust-allocated]       │  ║
║  │  • tag_name string                   [Rust-allocated]       │  ║
║  │  • parent_tag string                 [Rust-allocated]       │  ║
║  │  • attributes array                  [Rust-allocated]       │  ║
║  │  • Each attribute key/value          [Rust-allocated]       │  ║
║  │                                                              │  ║
║  │  ╔═════════════════════════════════════════════════════╗   │  ║
║  │  ║  FFI BOUNDARY                                       ║   │  ║
║  │  ║  ════════════════════════════════════════════════════   │  ║
║  │  ║  Pointers passed to C callback (READ-ONLY)         ║   │  ║
║  │  ║  - Must not modify data                             ║   │  ║
║  │  ║  - Must not free memory                             ║   │  ║
║  │  ║  - Valid only during callback execution             ║   │  ║
║  │  ╚═════════════════════════════════════════════════════╝   │  ║
║  │                                                              │  ║
║  │  • Cleanup after callback returns                  [RAII]   │  ║
║  │  • Panic guard around callback              [catch_unwind]  │  ║
║  └──────────────────────────────────────────────────────────────┘  ║
║                                                                      ║
╚════════════════════════════════════════════════════════════════════╝
           │
           │ Calls (via extern "C")
           │
╔════════════════════════════════════════════════════════════════════╗
║                       C LANGUAGE SPACE                               ║
║                                                                      ║
║  User-provided C callback function:                                 ║
║                                                                      ║
║  int ffi_visit_link(void* user_data,                               ║
║                     const CNodeContext* ctx,                       ║
║                     const char* href,                              ║
║                     const char* text,                              ║
║                     const char* title)                             ║
║  {                                                                   ║
║      // CAN:                                                        ║
║      // - Read any pointed-to data                                 ║
║      // - Access user_data                                         ║
║      // - Modify local variables                                   ║
║      //                                                             ║
║      // CANNOT:                                                     ║
║      // - Modify *ctx, *href, *text, *title                        ║
║      // - Free ctx, href, text, title                              ║
║      // - Keep pointers after return                               ║
║      //                                                             ║
║      // MUST:                                                       ║
║      // - Return 1 (success) or 0 (error)                          ║
║      // - Not hold locks that caller holds                         ║
║      // - Complete quickly (non-blocking)                          ║
║  }                                                                   ║
║                                                                      ║
╚════════════════════════════════════════════════════════════════════╝
```

## Call Stack During Callback

```
C Caller (main thread)
    │
    ├─ html_to_markdown_convert_with_visitor()      [C FFI function]
    │  └─ convert(&html, Some(visitor))             [Rust function]
    │     └─ visitor.visit_link(...)                [HtmlVisitor trait]
    │        └─ FfiVisitorAdapter.visit_link()      [Impl. in FfiVisitorAdapter]
    │           ├─ Convert inputs (NodeContext → CNodeContext)
    │           ├─ call_c_function(|| callback(...))
    │           │  └─ catch_unwind(|| ...)
    │           │     └─ CALLBACK INVOKED ──→ ┐
    │           │                             │
    │           │                             ├─→ C Callback Function
    │           │                             │   (user-provided)
    │           │                             │
    │           │                             ├─→ May call back to Rust?
    │           │                             │   (NOT RECOMMENDED)
    │           │                             │
    │           │                             ├─→ May block? (YES!)
    │           │                             │
    │           │                             ├─→ May panic? (YES! caught)
    │           │                             │
    │           │        ◄──────────────────────
    │           │
    │           └─ Cleanup (RAII + manual)
    │
    └─ Returns markdown string

    Stack depth: 4-6 levels deep depending on DOM nesting
```

## Thread Safety Model

```
Valid Usage:

Thread A:                          Thread B:
    │                                  │
    ├─ new(&callbacks, user_data_a)   ├─ new(&callbacks, user_data_b)
    │  → visitor_a                      │  → visitor_b
    │                                   │
    ├─ convert_with_visitor(..., a)    ├─ convert_with_visitor(..., b)
    │  │                                │  │
    │  ├─ callback A1                   │  └─ callback B1
    │  ├─ callback A2                   │
    │  └─ callback A3                   │  (independent execution)
    │                                   │
    └─ free(visitor_a)                 └─ free(visitor_b)


INVALID Usage (UNDEFINED BEHAVIOR):

Thread A:                          Thread B:
    │                                  │
    ├─ new(&callbacks, user_data)     │
    │  → visitor                        │
    │                                   │
    ├─ convert(..., visitor)  ─────────┼─→ ALSO converts(..., visitor)!
    │  🔥 RACE CONDITION 🔥             │
    │     panic_occurred flag           │
    │     might be read/written         │
    │     by both threads               │
    │                                   │
    └─ free(visitor)                   └─ free(visitor)
                                        🔥 DOUBLE-FREE 🔥
```

## Performance Overhead Per Callback

```
Default behavior (no visitor):
  1. Check: if let Some(callback)  = fn_ptr
     └─ O(1)

With visitor callback:
  1. Convert context (NodeContext → CNodeContext)
     └─ O(n) where n = number of attributes (typically 1-5)
  2. Call C function
     └─ O(1) [FFI call cost ~10-100 ns]
  3. Cleanup (RAII + manual)
     └─ O(n) where n = number of attributes
  4. Process result
     └─ O(m) where m = output string length (for Custom variant)

Total: O(n + m) per callback
Typical: 1-5 µs per callback on modern hardware

Breakdown for <div class="main" id="header">:
  Allocations:  ~200 ns
  FFI call:     ~50 ns
  Cleanup:      ~200 ns
  ────────────────
  Total:        ~500 ns (0.5 µs)

With ~1000 elements: 0.5 ms overhead (acceptable)
```

## Integration: Language Binding Flow

```
Go Binding Layer:
  ┌────────────────────────────────┐
  │ type Visitor struct {          │
  │   callbacks *C.CVisitorCallbacks │
  │   userData unsafe.Pointer      │
  │   handle C.html_to_markdown... │
  │ }                              │
  │                                │
  │ func NewVisitor(cbs ...) *V {  │
  │   h := C.html_to_markdown_...  │
  │   return &Visitor{..., h}      │
  │ }                              │
  │                                │
  │ func (v *Visitor) ConvertHTML()│
  │   return C.html_to_markdown... │
  │ }                              │
  └────────────────────────────────┘
         │
         │ cgo calls
         │
  ┌────────────────────────────────┐
  │ C FFI Layer (html-to-markdown) │
  │ (html_to_markdown_visitor_new) │
  │ (html_to_markdown_convert...)  │
  └────────────────────────────────┘
         │
         │ Rust implementation
         │
  ┌────────────────────────────────┐
  │ FfiVisitorAdapter              │
  │ (calls C callbacks provided)   │
  │ (Go closure → C callback)      │
  └────────────────────────────────┘
         │
         │ Rust core
         │
  ┌────────────────────────────────┐
  │ html-to-markdown-rs core       │
  │ (HTML parsing + conversion)    │
  └────────────────────────────────┘
```

---

This visual overview complements the detailed text documentation. Use these diagrams to:
- Understand overall architecture
- Trace data flow through the system
- Identify memory ownership boundaries
- Understand error handling paths
- Verify thread safety constraints
