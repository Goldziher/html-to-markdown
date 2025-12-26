# Visitor Pattern Thread Safety Fix - Implementation Status

## Problem
The visitor pattern implementation had a thread safety contradiction:
- `HtmlVisitor` trait required `Send + Sync` bounds
- Stored visitor in `ConversionOptions` as `Rc<RefCell<dyn HtmlVisitor>>`
- This made it impossible to use non-thread-safe visitors (like Ruby objects via Magnus)

## Solution Approach
Remove `Send + Sync` bounds and pass visitor by mutable reference instead of storing in options.

## Changes Completed ✓

### 1. visitor.rs (Line 327)
**Status: COMPLETE**
```rust
// Before:
pub trait HtmlVisitor: Send + Sync + std::fmt::Debug {

// After:
pub trait HtmlVisitor: std::fmt::Debug {
```

### 2. options.rs
**Status: COMPLETE**
- Removed visitor field from `ConversionOptions` struct (line ~252)
- Removed visitor initialization in `Default` impl (line ~328)
- Removed imports: `Rc`, `RefCell`, `HtmlVisitor`

### 3. lib.rs (convert_with_visitor function, lines 523-546)
**Status: COMPLETE**
```rust
// Before:
pub fn convert_with_visitor<V: HtmlVisitor + 'static>(
    html: &str,
    options: Option<ConversionOptions>,
    visitor: V,
) -> Result<String> {
    // ... wrapped visitor in Rc<RefCell<>> ...
}

// After:
pub fn convert_with_visitor<V: HtmlVisitor>(
    html: &str,
    options: Option<ConversionOptions>,
    visitor: &mut V,
) -> Result<String> {
    // ... calls converter::convert_html_with_visitor ...
}
```

### 4. Cargo.toml
**Status: COMPLETE**
Added `visitor = []` to `[features]` section.

## Changes Needed - converter.rs

### Current Status
- File restored to backup (converter.rs)
- Partial implementation saved as converter.rs.broken
- Compiles WITH visitor feature
- Does NOT compile without visitor feature (needs simpler approach)

### Recommended Implementation Strategy

**Simple Approach:**
1. Remove `visitor` field from `Context` struct (line ~570)
2. Add `visitor: Option<&mut dyn HtmlVisitor>` parameter to:
   - `convert_html_impl()`
   - `walk_node()` and ALL its calls (~94 locations)
3. **Key Decision:** Make parameter unconditional (not `#[cfg(feature = "visitor")]`)
   - Type is always `Option<&mut dyn HtmlVisitor>`
   - When feature disabled, always pass `None`
   - Avoids complex conditional compilation

**Steps:**
```rust
// 1. Update convert_html_impl signature
fn convert_html_impl(
    html: &str,
    options: &ConversionOptions,
    inline_collector: Option<InlineCollectorHandle>,
    #[cfg(feature = "metadata")] metadata_collector: Option<...>,
    #[cfg(not(feature = "metadata"))] _metadata_collector: Option<()>,
    visitor: Option<&mut dyn crate::visitor::HtmlVisitor>,  // ALWAYS present
) -> Result<String>

// 2. Add convert_html_with_visitor wrapper
#[cfg(feature = "visitor")]
pub(crate) fn convert_html_with_visitor<V: HtmlVisitor>(
    html: &str,
    options: &ConversionOptions,
    visitor: &mut V,
) -> Result<String> {
    convert_html_impl(html, options, None, None, Some(visitor))
}

// 3. Update walk_node signature
fn walk_node(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
    visitor: &mut Option<&mut dyn crate::visitor::HtmlVisitor>,  // Always present
)

// 4. Update visitor access (4 locations)
// Before:
if let Some(ref visitor) = ctx.visitor {
    let result = visitor.borrow_mut().visit_text(...);
}

// After:
#[cfg(feature = "visitor")]
if let Some(v) = visitor.as_deref_mut() {
    let result = v.visit_text(...);
}

// 5. Update all walk_node calls (~94 locations)
// Use sed or similar:
sed -i 's/walk_node(\([^;]*\), dom_ctx)/walk_node(\1, dom_ctx, visitor)/g'
```

### Exact Line Numbers to Change

**From converter.rs.broken (reference):**
- Line 570: Remove `visitor: Option<Rc<RefCell<...>>>`  from Context
- Lines 3300, 3410, 3777, 3883: Replace `ctx.visitor` with `visitor` parameter
- Lines 3311, 3447, 3816, 3922: Replace `visitor.borrow_mut().visit_*` with `v.visit_*`
- ~94 walk_node calls: Add `, visitor` parameter

## Testing Checklist

### Rust Core
- [ ] `cd crates/html-to-markdown && cargo build --features visitor`
- [ ] `cd crates/html-to-markdown && cargo build` (without visitor)
- [ ] `cd crates/html-to-markdown && cargo test --features visitor --lib`

### Ruby Binding
- [ ] `cd packages/ruby && bundle exec rake compile`
- [ ] `cd packages/ruby && bundle exec rake test`

## Validation Criteria

**Success indicators:**
1. Rust compiles with and without `visitor` feature
2. Ruby binding compiles successfully
3. All existing tests pass
4. No `Send + Sync` bounds on `HtmlVisitor`
5. No `Rc<RefCell<>>` wrapper needed for visitor
6. Visitor passed by `&mut` reference

## Files Changed

1. `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/visitor.rs`
2. `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/options.rs`
3. `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/lib.rs`
4. `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/Cargo.toml`
5. `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/converter.rs` (IN PROGRESS)

## References

- Original issue describes the `Send + Sync` contradiction with Ruby Magnus bindings
- Magnus requires !Send types for Ruby objects
- Solution: Pass visitor by mutable reference instead of storing in options
