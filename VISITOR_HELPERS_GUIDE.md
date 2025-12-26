# Visitor Helper Functions Guide

This guide documents the helper functions and macros available in `visitor_helpers.rs` for integrating the visitor pattern into the HTML→Markdown converter.

## Overview

The visitor helper functions provide three main capabilities:

1. **Building NodeContext**: Create visitor context from parsing state
2. **Dispatching Visitors**: Call visitor methods and handle results
3. **Reducing Boilerplate**: Macros for common visitor patterns

## Core Functions

### `build_node_context`

Creates a `NodeContext` from the current parsing state.

**Signature:**
```rust
pub fn build_node_context(
    node_type: NodeType,
    tag_name: &str,
    attributes: &BTreeMap<String, String>,
    depth: usize,
    index_in_parent: usize,
    parent_tag: Option<&str>,
    is_inline: bool,
) -> NodeContext
```

**Parameters:**
- `node_type`: Coarse-grained classification (Link, Image, Heading, etc.)
- `tag_name`: Raw HTML tag name (e.g., "div", "h1")
- `attributes`: All HTML attributes as key-value pairs
- `depth`: Nesting depth in the DOM tree (0 = root)
- `index_in_parent`: Zero-based index among siblings
- `parent_tag`: Parent element's tag name (None if root)
- `is_inline`: Whether this element is treated as inline vs block

**Example:**
```rust
use crate::visitor_helpers::build_node_context;
use crate::visitor::NodeType;

let ctx = build_node_context(
    NodeType::Heading,
    "h1",
    &attributes,
    1,      // depth
    0,      // first child
    Some("body"),
    false,  // block element
);
```

**Performance Notes:**
- Minimal allocations: clones tag_name, parent_tag, and attributes
- For text nodes and simple elements without attributes, allocations are minimal
- Inline-able for hot paths

### `dispatch_visitor`

Dispatches a visitor callback and translates the result into control flow decisions.

**Signature:**
```rust
pub fn dispatch_visitor<F>(
    visitor: &Option<Rc<RefCell<dyn HtmlVisitor>>>,
    callback: F,
) -> Result<VisitorDispatch>
where
    F: FnOnce(&mut dyn HtmlVisitor) -> VisitResult
```

**Parameters:**
- `visitor`: Optional visitor (wrapped in Rc<RefCell<>>)
- `callback`: Closure that invokes the appropriate visitor method

**Returns:**
- `Ok(VisitorDispatch::Continue)`: Proceed with default behavior
- `Ok(VisitorDispatch::Custom(String))`: Replace output with custom markdown
- `Ok(VisitorDispatch::Skip)`: Skip this element entirely
- `Ok(VisitorDispatch::PreserveHtml)`: Preserve original HTML
- `Err(ConversionError::Visitor(msg))`: Stop conversion with error

**Example:**
```rust
use crate::visitor_helpers::dispatch_visitor;

let result = dispatch_visitor(
    &visitor,
    |v| v.visit_heading(&ctx, level, text, id),
)?;

match result {
    VisitorDispatch::Continue => {
        // Use default heading conversion
    }
    VisitorDispatch::Custom(output) => {
        return Ok(output);
    }
    VisitorDispatch::Skip => {
        return Ok(String::new());
    }
    VisitorDispatch::PreserveHtml => {
        // Return raw HTML
    }
}
```

**Performance Notes:**
- Zero-cost when visitor is None (common case)
- Single dynamic dispatch when visitor is present
- No allocations except for error messages

## `VisitorDispatch` Type

An ergonomic wrapper around `VisitResult` with utility methods.

**Methods:**
```rust
impl VisitorDispatch {
    // Check dispatch type
    pub fn is_continue(&self) -> bool;
    pub fn is_custom(&self) -> bool;
    pub fn is_skip(&self) -> bool;
    pub fn is_preserve_html(&self) -> bool;

    // Extract custom output
    pub fn into_custom(self) -> Option<String>;
    pub fn as_custom(&self) -> Option<&str>;
}
```

**Example:**
```rust
let dispatch = dispatch_visitor(&visitor, |v| v.visit_text(&ctx, "hello"))?;

if dispatch.is_custom() {
    let output = dispatch.into_custom().unwrap();
    return Ok(output);
}
```

## Macros

### `try_visitor!`

Reduces boilerplate when calling visitor methods with early return on Custom/Skip/Error.

**Syntax:**
```rust
try_visitor!(visitor, method_name, &ctx, arg1, arg2, ...);
```

**Behavior:**
- Returns early with custom output if visitor returns `Custom`
- Returns early with empty string if visitor returns `Skip`
- Returns early with error if visitor returns `Error`
- Continues execution if visitor returns `Continue` or is None
- Currently treats `PreserveHtml` as `Continue` (TODO: implement HTML preservation)

**Example:**
```rust
use crate::try_visitor;

fn convert_heading(
    ctx: &Context,
    level: u32,
    text: &str,
    id: Option<&str>,
) -> Result<String> {
    let node_ctx = build_node_context(
        NodeType::Heading,
        "h1",
        &attributes,
        depth,
        index,
        parent_tag,
        false,
    );

    // Try visitor - returns early if custom output or skip
    try_visitor!(&ctx.visitor, visit_heading, &node_ctx, level, text, id);

    // Default heading conversion logic continues here...
    let output = format!("{} {}\n", "#".repeat(level as usize), text);
    Ok(output)
}
```

### `try_visitor_element_start!`

Specialized macro for `visit_element_start` callbacks.

**Syntax:**
```rust
try_visitor_element_start!(visitor, &ctx);
```

**Example:**
```rust
use crate::try_visitor_element_start;

fn process_heading(...) -> Result<String> {
    let ctx = build_node_context(...);
    try_visitor_element_start!(&visitor, &ctx)?;

    // Default heading processing continues here...
}
```

### `try_visitor_element_end!`

Specialized macro for `visit_element_end` callbacks with output inspection.

**Syntax:**
```rust
try_visitor_element_end!(visitor, &ctx, &output);
```

**Example:**
```rust
use crate::try_visitor_element_end;

fn process_heading(...) -> Result<String> {
    let ctx = build_node_context(...);
    let mut output = String::from("# Heading");

    // Allow visitor to inspect/replace output
    try_visitor_element_end!(&visitor, &ctx, &output)?;
    Ok(output)
}
```

## Integration Pattern

Here's the recommended pattern for integrating visitors into converter functions:

```rust
use crate::visitor_helpers::{build_node_context, dispatch_visitor, VisitorDispatch};
use crate::visitor::NodeType;
use crate::try_visitor;

fn convert_element(
    tag: &Tag,
    ctx: &mut Context,
    dom_ctx: &DomContext,
    node_id: u32,
) -> Result<String> {
    // 1. Build visitor context from parsing state
    let node_ctx = build_node_context(
        NodeType::Paragraph,
        tag.name().as_utf8_str(),
        &extract_attributes(tag),
        dom_ctx.get_depth(node_id),
        dom_ctx.get_sibling_index(node_id),
        dom_ctx.get_parent_tag(node_id),
        ctx.convert_as_inline || ctx.in_table_cell,
    );

    // 2. Call element_start visitor
    try_visitor!(&ctx.visitor, visit_element_start, &node_ctx);

    // 3. Perform default conversion logic
    let mut output = String::new();
    // ... conversion code ...

    // 4. Call element_end visitor
    try_visitor!(&ctx.visitor, visit_element_end, &node_ctx, &output);

    Ok(output)
}
```

## Common Patterns

### Pattern 1: Simple Element Conversion

```rust
fn convert_heading(
    level: u32,
    text: &str,
    id: Option<&str>,
    ctx: &Context,
) -> Result<String> {
    let node_ctx = build_node_context(
        NodeType::Heading,
        &format!("h{}", level),
        &BTreeMap::new(),
        ctx.depth,
        0,
        None,
        false,
    );

    try_visitor!(&ctx.visitor, visit_heading, &node_ctx, level, text, id);

    // Default conversion
    let prefix = "#".repeat(level as usize);
    Ok(format!("{} {}\n", prefix, text))
}
```

### Pattern 2: Element with Pre/Post Hooks

```rust
fn convert_blockquote(
    content: &str,
    ctx: &Context,
) -> Result<String> {
    let node_ctx = build_node_context(
        NodeType::Blockquote,
        "blockquote",
        &BTreeMap::new(),
        ctx.blockquote_depth,
        0,
        None,
        false,
    );

    // Pre-hook
    try_visitor_element_start!(&ctx.visitor, &node_ctx)?;

    // Convert content
    let lines: Vec<_> = content.lines().map(|line| format!("> {}", line)).collect();
    let mut output = lines.join("\n");

    // Post-hook with output
    try_visitor_element_end!(&ctx.visitor, &node_ctx, &output)?;

    Ok(output)
}
```

### Pattern 3: Manual Dispatch for Complex Logic

```rust
fn convert_table_row(
    cells: &[String],
    is_header: bool,
    ctx: &Context,
) -> Result<String> {
    let node_ctx = build_node_context(
        NodeType::TableRow,
        "tr",
        &BTreeMap::new(),
        ctx.depth,
        0,
        Some("table"),
        false,
    );

    let dispatch = dispatch_visitor(
        &ctx.visitor,
        |v| v.visit_table_row(&node_ctx, cells, is_header),
    )?;

    match dispatch {
        VisitorDispatch::Continue => {
            // Default table row rendering
            let row = format!("| {} |\n", cells.join(" | "));
            if is_header {
                let separator = format!("|{}|\n", " --- |".repeat(cells.len()));
                Ok(format!("{}{}", row, separator))
            } else {
                Ok(row)
            }
        }
        VisitorDispatch::Custom(output) => Ok(output),
        VisitorDispatch::Skip => Ok(String::new()),
        VisitorDispatch::PreserveHtml => {
            // Reconstruct HTML
            Ok(format!("<tr>{}</tr>", cells.join("")))
        }
    }
}
```

## Error Handling

Visitor errors are wrapped in `ConversionError::Visitor`:

```rust
if let Err(ConversionError::Visitor(msg)) = result {
    eprintln!("Visitor error: {}", msg);
}
```

Visitors can signal errors by returning `VisitResult::Error(String)`:

```rust
impl HtmlVisitor for MyVisitor {
    fn visit_heading(&mut self, ctx: &NodeContext, level: u32, text: &str, id: Option<&str>) -> VisitResult {
        if level > 6 {
            VisitResult::Error("Invalid heading level".to_string())
        } else {
            VisitResult::Continue
        }
    }
}
```

## Performance Considerations

1. **Zero-cost when disabled**: No runtime overhead when visitor feature is disabled
2. **Minimal overhead when None**: Fast path when no visitor is provided
3. **Allocation strategy**:
   - `build_node_context`: Clones tag name, parent tag, and attributes
   - `dispatch_visitor`: No allocations in the happy path
   - Macros: Compile-time code generation, no runtime overhead

4. **Hot path optimization**:
   - Functions marked `#[inline]` where appropriate
   - Early returns to avoid unnecessary work
   - LRU caching for frequently accessed nodes (in converter)

## Testing

All helper functions include comprehensive unit tests:

```bash
cargo test --features visitor --lib visitor_helpers
```

Test coverage includes:
- NodeContext building with various configurations
- Visitor dispatch for all VisitResult variants
- Error handling and conversion
- VisitorDispatch utility methods
- Edge cases (None visitor, empty attributes, etc.)

## Future Enhancements

1. **HTML Preservation**: Implement proper `PreserveHtml` handling
2. **Visitor Chaining**: Support multiple visitors in sequence
3. **Visitor Metrics**: Track visitor invocation counts for debugging
4. **Visitor State**: Helper for maintaining visitor state across calls
5. **Visitor Debugging**: Debug mode with verbose logging

## Related Files

- `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/visitor.rs` - Visitor trait definition
- `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/visitor_helpers.rs` - Helper functions (this module)
- `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/converter.rs` - Main conversion logic
- `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/error.rs` - Error types
