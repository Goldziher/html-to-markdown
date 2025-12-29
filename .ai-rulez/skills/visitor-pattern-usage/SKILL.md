# Visitor Pattern Usage for html-to-markdown

## Overview

The visitor pattern in html-to-markdown provides extensible hooks into the HTML-to-Markdown conversion pipeline. Custom visitors can inspect, modify, or replace the default conversion behavior for any of the 60+ HTML element types.

## Architecture Philosophy

**Key Principles:**
- **Flexibility over performance**: Give users full control over conversion
- **Zero-cost when unused**: No overhead if visitor feature disabled
- **Comprehensive coverage**: All HTML element types have dedicated visitor methods
- **Pre/post hooks**: Both entry and exit points exposed for elements

## Feature Gates

The visitor pattern is conditionally compiled:

```rust
#[cfg(feature = "visitor")]
pub mod visitor;

#[cfg(feature = "async-visitor")]
pub use visitor_helpers::AsyncVisitorHandle;
```

**In Cargo.toml:**
```toml
[features]
default = ["metadata"]
visitor = []
async-visitor = ["visitor", "dep:async-trait"]
```

## Core Traits and Types

### NodeType Enumeration

Located in `/crates/html-to-markdown/src/visitor.rs`, categorizes all HTML elements:

```rust
pub enum NodeType {
    // Text content
    Text,

    // Block elements
    Heading,
    Paragraph,
    Div,
    Blockquote,
    Pre,
    Hr,

    // Lists
    List,               // ul, ol
    ListItem,           // li
    DefinitionList,     // dl
    DefinitionTerm,     // dt
    DefinitionDescription, // dd

    // Tables
    Table,
    TableRow,
    TableCell,
    TableHeader,
    TableBody,
    TableHead,
    TableFoot,

    // Inline formatting
    Link,
    Image,
    Strong,
    Em,
    Code,
    Strikethrough,
    Mark,
    Sub,
    Sup,
    LineBreak,
    Ruby,

    // Semantic HTML5
    Article,
    Section,
    Nav,
    Aside,
    Header,
    Footer,
    Main,

    // Media
    Audio,
    Video,
    Picture,
    Iframe,
    Svg,

    // Forms
    Input,
    Select,
    Button,
    Textarea,
    Fieldset,

    // Other
    Form,
    Label,
    Span,
    Generic(String),  // Unknown tags
}
```

### VisitResult Enumeration

Specifies what action the conversion should take:

```rust
pub enum VisitResult {
    /// Use default conversion for this element
    Default,

    /// Skip this element entirely (no output)
    Skip,

    /// Custom markdown for this element
    Custom(String),

    /// Process children normally, wrap with custom before/after
    Custom(String),  // Could also support Wrap variant

    /// Replace element content with custom markdown
    Replace(String),
}
```

### NodeContext Structure

Provides context about the current node being visited:

```rust
pub struct NodeContext {
    pub node_type: NodeType,
    pub tag_name: Option<String>,        // Actual HTML tag if element
    pub attributes: BTreeMap<String, String>,  // All HTML attributes
    pub parent_node_type: Option<NodeType>,    // Parent element type
    pub depth: usize,                    // Nesting depth
    pub position_in_parent: usize,       // Sibling index
}
```

## HtmlVisitor Trait

The main visitor trait with methods for each element type:

```rust
pub trait HtmlVisitor {
    // Generic element fallback
    fn visit_element(
        &mut self,
        ctx: &NodeContext,
        tag: &str,
        attributes: &BTreeMap<String, String>,
    ) -> VisitResult;

    // Text content
    fn visit_text(&mut self, ctx: &NodeContext, text: &str) -> VisitResult;

    // Headings
    fn visit_heading(
        &mut self,
        ctx: &NodeContext,
        level: u8,  // 1-6
        text: &str,
    ) -> VisitResult;

    fn visit_paragraph(&mut self, ctx: &NodeContext, text: &str) -> VisitResult;

    // Links and images
    fn visit_link(
        &mut self,
        ctx: &NodeContext,
        href: &str,
        text: &str,
        title: Option<&str>,
    ) -> VisitResult;

    fn visit_image(
        &mut self,
        ctx: &NodeContext,
        src: &str,
        alt: &str,
        title: Option<&str>,
    ) -> VisitResult;

    // Formatting
    fn visit_strong(&mut self, ctx: &NodeContext, text: &str) -> VisitResult;
    fn visit_em(&mut self, ctx: &NodeContext, text: &str) -> VisitResult;
    fn visit_code(&mut self, ctx: &NodeContext, code: &str) -> VisitResult;
    fn visit_code_block(
        &mut self,
        ctx: &NodeContext,
        code: &str,
        language: Option<&str>,
    ) -> VisitResult;
    fn visit_strikethrough(&mut self, ctx: &NodeContext, text: &str) -> VisitResult;

    // Lists
    fn visit_list(
        &mut self,
        ctx: &NodeContext,
        ordered: bool,
        items: &[String],
    ) -> VisitResult;

    fn visit_list_item(
        &mut self,
        ctx: &NodeContext,
        content: &str,
        index: usize,
    ) -> VisitResult;

    // Tables
    fn visit_table(
        &mut self,
        ctx: &NodeContext,
        rows: &[Vec<String>],
    ) -> VisitResult;

    fn visit_table_cell(
        &mut self,
        ctx: &NodeContext,
        content: &str,
        is_header: bool,
    ) -> VisitResult;

    // ... and 40+ more visitor methods
}
```

## Basic Example: Custom Link Converter

Convert all external links to plain text with URLs in parentheses:

```rust
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};

#[derive(Debug)]
struct PlainTextLinkVisitor;

impl HtmlVisitor for PlainTextLinkVisitor {
    fn visit_link(
        &mut self,
        _ctx: &NodeContext,
        href: &str,
        text: &str,
        _title: Option<&str>,
    ) -> VisitResult {
        // Convert all links to plain text with URL
        VisitResult::Custom(format!("{} ({})", text, href))
    }

    // ... implement other visitor methods as Default
}

// Usage
let html = r#"<p>Visit <a href="https://example.com">our site</a></p>"#;
let mut visitor = PlainTextLinkVisitor;
let markdown = convert_with_visitor(html, None, Some(&mut visitor))?;
// Output: Visit our site (https://example.com)
```

## Advanced Example: Custom Code Block Highlighter

Highlight code blocks with language-specific syntax:

```rust
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};

#[derive(Debug)]
struct HighlightingVisitor;

impl HtmlVisitor for HighlightingVisitor {
    fn visit_code_block(
        &mut self,
        _ctx: &NodeContext,
        code: &str,
        language: Option<&str>,
    ) -> VisitResult {
        match language {
            Some("python") => {
                // Custom Python highlighting
                VisitResult::Custom(format!(
                    "```python\n<!-- HIGHLIGHTED -->\n{}\n```",
                    code
                ))
            }
            Some("rust") => {
                // Custom Rust highlighting
                VisitResult::Custom(format!(
                    "```rust\n<!-- WITH SYNTAX HIGHLIGHTING -->\n{}\n```",
                    code
                ))
            }
            _ => VisitResult::Default,  // Use default for other languages
        }
    }

    fn visit_link(
        &mut self,
        _ctx: &NodeContext,
        href: &str,
        text: &str,
        title: Option<&str>,
    ) -> VisitResult {
        // Links in documentation: add reference-style syntax
        VisitResult::Custom(format!("[{}][{}]", text, href))
    }

    fn visit_heading(
        &mut self,
        _ctx: &NodeContext,
        level: u8,
        text: &str,
    ) -> VisitResult {
        // Add anchor links to all headings
        let id = text.to_lowercase().replace(' ', '-');
        VisitResult::Custom(format!(
            "{} {{#{}}}\n",
            "#".repeat(level as usize),
            id
        ))
    }
}

// Usage
let html = r#"
<h1>Documentation</h1>
<p>See <a href="https://docs.rs">our docs</a></p>
<pre><code class="language-rust">fn main() {}</code></pre>
"#;
let mut visitor = HighlightingVisitor;
let markdown = convert_with_visitor(html, None, Some(&mut visitor))?;
```

## Filtering by Node Type

Visit only specific element types:

```rust
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, NodeType};

#[derive(Debug)]
struct ImageOnlyVisitor {
    image_count: usize,
}

impl HtmlVisitor for ImageOnlyVisitor {
    fn visit_image(
        &mut self,
        _ctx: &NodeContext,
        src: &str,
        alt: &str,
        _title: Option<&str>,
    ) -> VisitResult {
        self.image_count += 1;
        println!("Image {}: {} ({})", self.image_count, alt, src);

        // Could extract images to separate directory
        VisitResult::Custom(format!("![{}]({})", alt, src))
    }

    fn visit_text(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Skip  // Skip all text, only output images
    }
}

// Usage
let mut visitor = ImageOnlyVisitor { image_count: 0 };
let markdown = convert_with_visitor(html, None, Some(&mut visitor))?;
println!("Found {} images", visitor.image_count);
```

## Context-Aware Transformations

Use parent context and depth to transform based on structure:

```rust
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, NodeType};

#[derive(Debug)]
struct DepthTrackingVisitor {
    current_depth: usize,
}

impl HtmlVisitor for DepthTrackingVisitor {
    fn visit_paragraph(
        &mut self,
        ctx: &NodeContext,
        text: &str,
    ) -> VisitResult {
        // Different formatting based on depth
        match ctx.depth {
            0 => VisitResult::Custom(format!("**{}**\n", text)),  // Bold at top level
            1 => VisitResult::Custom(format!("*{}*\n", text)),    // Italic nested once
            _ => VisitResult::Default,                            // Normal elsewhere
        }
    }

    fn visit_link(
        &mut self,
        ctx: &NodeContext,
        href: &str,
        text: &str,
        _title: Option<&str>,
    ) -> VisitResult {
        // Links in blockquotes get footnote style
        if let Some(NodeType::Blockquote) = ctx.parent_node_type {
            VisitResult::Custom(format!("{}[^{}]", text, href))
        } else {
            VisitResult::Default
        }
    }
}

// Usage
let mut visitor = DepthTrackingVisitor { current_depth: 0 };
let markdown = convert_with_visitor(html, None, Some(&mut visitor))?;
```

## Attribute-Based Routing

Route handling based on HTML attributes:

```rust
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};

#[derive(Debug)]
struct AttributeRoutingVisitor;

impl HtmlVisitor for AttributeRoutingVisitor {
    fn visit_link(
        &mut self,
        ctx: &NodeContext,
        href: &str,
        text: &str,
        title: Option<&str>,
    ) -> VisitResult {
        // Custom handling for data attributes
        if let Some(tracking_id) = ctx.attributes.get("data-tracking-id") {
            return VisitResult::Custom(format!(
                "[{}]({} \"{}\")",
                text,
                href,
                tracking_id
            ));
        }

        // Skip links marked with data-skip="true"
        if ctx.attributes.get("data-skip").map_or(false, |v| v == "true") {
            return VisitResult::Skip;
        }

        VisitResult::Default
    }

    fn visit_paragraph(
        &mut self,
        ctx: &NodeContext,
        text: &str,
    ) -> VisitResult {
        // Blockquote paragraphs differently
        if ctx.attributes.get("data-featured") == Some(&"true".to_string()) {
            VisitResult::Custom(format!("> {}\n", text))
        } else {
            VisitResult::Default
        }
    }
}

// Usage
let html = r#"
<a href="/page" data-tracking-id="click-001">Track me</a>
<a href="/skip" data-skip="true">Skip me</a>
<p data-featured="true">Important paragraph</p>
"#;
let mut visitor = AttributeRoutingVisitor;
let markdown = convert_with_visitor(html, None, Some(&mut visitor))?;
```

## Async Visitor Support

For languages with native async/await (Python, TypeScript, Elixir):

```rust
#[cfg(feature = "async-visitor")]
pub async fn convert_with_async_visitor(
    html: &str,
    options: Option<ConversionOptions>,
    visitor: Option<AsyncVisitorHandle>,
) -> Result<String> { ... }
```

### Python Async Example (PyO3)

```python
import asyncio
import html_to_markdown

class AsyncSyntaxHighlighter:
    async def visit_code_block(self, ctx, code, language):
        # Call async syntax highlighting service
        highlighted = await highlight_service.highlight(code, language)
        return f"```{language}\n{highlighted}\n```"

    async def visit_link(self, ctx, href, text, title):
        # Check external link status asynchronously
        is_valid = await check_link_validity(href)
        if is_valid:
            return f"[{text}]({href})"
        else:
            return f"~~[{text}]({href})~~ (broken)"

# Usage
markdown = await html_to_markdown.convert_with_async_visitor(
    html,
    None,
    AsyncSyntaxHighlighter()
)
```

### TypeScript Async Example (NAPI-RS)

```typescript
import { convertWithAsyncVisitor } from 'html-to-markdown';

class AsyncContentProcessor {
    async visitLink(ctx, href, text, title) {
        // Fetch metadata for link
        const metadata = await fetch(href).then(r => r.json());
        return `[${text}](${href} "${metadata.title}")`;
    }

    async visitImage(ctx, src, alt, title) {
        // Optimize image
        const optimized = await imageOptimizer.optimize(src);
        return `![${alt}](${optimized})`;
    }
}

const markdown = await convertWithAsyncVisitor(html, undefined, new AsyncContentProcessor());
```

## State Management in Visitors

Maintain state across multiple visits:

```rust
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use std::collections::HashSet;

#[derive(Debug)]
struct LinkCollectorVisitor {
    external_links: HashSet<String>,
    email_links: HashSet<String>,
    internal_links: HashSet<String>,
}

impl HtmlVisitor for LinkCollectorVisitor {
    fn visit_link(
        &mut self,
        _ctx: &NodeContext,
        href: &str,
        _text: &str,
        _title: Option<&str>,
    ) -> VisitResult {
        if href.starts_with("mailto:") {
            self.email_links.insert(href.to_string());
        } else if href.starts_with("http") {
            self.external_links.insert(href.to_string());
        } else {
            self.internal_links.insert(href.to_string());
        }

        VisitResult::Default  // Keep default link formatting
    }
}

// Usage
let mut visitor = LinkCollectorVisitor {
    external_links: HashSet::new(),
    email_links: HashSet::new(),
    internal_links: HashSet::new(),
};

let markdown = convert_with_visitor(html, None, Some(&mut visitor))?;

println!("External: {:?}", visitor.external_links);
println!("Email: {:?}", visitor.email_links);
println!("Internal: {:?}", visitor.internal_links);
```

## Performance Considerations

### Impact on Conversion Speed

- Visitor trait calls add function dispatch overhead
- For large documents (10K+ elements), overhead ~5-10%
- Use feature gate to eliminate entirely if not needed

### Optimization Strategies

1. **Fast path for most elements:**
   ```rust
   fn visit_text(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
       VisitResult::Default  // Quick return for most text nodes
   }
   ```

2. **Only override when needed:**
   ```rust
   // Only override link handling
   // All other methods inherit Default implementation
   ```

3. **Avoid allocations in hot path:**
   ```rust
   // Bad: allocate string for every node
   VisitResult::Custom(format!(">{}<", text))

   // Better: pre-allocate or use Cow
   let mut result = String::with_capacity(text.len() + 2);
   result.push('>');
   result.push_str(text);
   result.push('<');
   VisitResult::Custom(result)
   ```

## Integration with Conversion Options

Visitors work alongside `ConversionOptions`:

```rust
use html_to_markdown_rs::{ConversionOptions, HeadingStyle};

let options = ConversionOptions {
    heading_style: HeadingStyle::AtxClosed,  // User preference
    wrap: true,
    wrap_width: 80,
    ..Default::default()
};

// Visitor can override specific behaviors
let mut visitor = CustomVisitor;
let markdown = convert_with_visitor(html, Some(options), Some(&mut visitor))?;
```

**Priority:** Visitor always takes precedence. If visitor returns `Custom` or `Skip`, conversion options are bypassed for that element.

## Error Handling in Visitors

The visitor pattern doesn't support errors directly. Return `Default` or `Skip` instead:

```rust
impl HtmlVisitor for SafeVisitor {
    fn visit_link(
        &mut self,
        _ctx: &NodeContext,
        href: &str,
        text: &str,
        title: Option<&str>,
    ) -> VisitResult {
        // Can't return error, so validate and fallback
        if href.is_empty() {
            return VisitResult::Custom(text.to_string());  // Fallback to text
        }

        VisitResult::Default
    }
}
```

## Testing Visitors

Located in binding test suites (Python, TypeScript, Ruby, PHP):

```bash
# Test visitor feature
task rust:test  # Includes visitor tests

# Binding-specific visitor tests
task python:test  # tests/test_visitor.py
task typescript:test  # packages/typescript/tests/visitor.spec.ts
task ruby:test  # packages/ruby/spec/visitor_spec.rb
```

## Implementation Location

**Core Files:**
- `/crates/html-to-markdown/src/visitor.rs` - Trait definitions and NodeType enum
- `/crates/html-to-markdown/src/visitor_helpers.rs` - VisitorHandle and async support
- `/crates/html-to-markdown/src/converter.rs` - Integration with conversion pipeline

**Binding Examples:**
- `/crates/html-to-markdown-py/src/lib.rs` - PyO3 visitor wrapping
- `/crates/html-to-markdown-node/src/lib.rs` - NAPI-RS visitor support
- `/packages/ruby/lib/visitor.rb` - Ruby visitor interface
- `/packages/php/src/Visitor.php` - PHP visitor base class

## API Pattern

```rust
// Simple visitor (sync)
pub fn convert_with_visitor(
    html: &str,
    options: Option<ConversionOptions>,
    visitor: Option<visitor::VisitorHandle>,
) -> Result<String>

// Async visitor (for languages with native async)
#[cfg(feature = "async-visitor")]
pub async fn convert_with_async_visitor(
    html: &str,
    options: Option<ConversionOptions>,
    visitor: Option<AsyncVisitorHandle>,
) -> Result<String>

// Combined with metadata (future enhancement)
// pub fn convert_with_metadata_and_visitor(...) -> Result<(String, ExtendedMetadata)>
```

## Quick Reference: Common Visitor Patterns

| Use Case | Implementation |
|----------|----------------|
| Skip certain elements | Return `VisitResult::Skip` |
| Modify element output | Return `VisitResult::Custom(new_markdown)` |
| Track state | Use `&mut self` fields to accumulate data |
| Conditional routing | Use `ctx` fields (parent, depth, attributes) |
| Preserve default | Return `VisitResult::Default` |
| Context-aware | Match on `ctx.parent_node_type`, `ctx.depth` |
| Attribute-based | Read from `ctx.attributes` map |
| Stateless transformation | Implement stateless visitor struct |
