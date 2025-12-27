# Visitor Pattern Guide

The visitor pattern enables custom HTML→Markdown conversion logic by providing callbacks for specific HTML elements during traversal. Use visitors to transform content, filter elements, validate structure, or collect analytics.

## Use Cases

**When to use the visitor pattern:**
- **Domain-specific dialects**: Convert to custom Markdown flavors (e.g., Obsidian, Notion)
- **Content filtering**: Remove or transform specific elements (tracking pixels, ads)
- **URL rewriting**: Rewrite CDN URLs, add query parameters, validate links
- **Accessibility validation**: Check alt text, heading hierarchy, link text
- **Analytics**: Track element usage, link destinations, image sources

**When NOT to use it:**
- Basic HTML→Markdown conversion (use `convert()` instead)
- Simple option changes (use `ConversionOptions` instead)
- Metadata extraction only (use `convertWithMetadata()` instead)

---

## Core Concepts

### NodeContext

Every visitor method receives a `NodeContext` object describing the current element:

```typescript
interface NodeContext {
  nodeType: string;         // "element", "text", "comment", etc.
  tagName: string | null;   // "a", "img", "p", null for non-elements
  attributes: Record<string, string>;  // HTML attributes
  depth: number;            // Nesting level in the tree
  indexInParent: number;    // Position among siblings
  parentTag: string | null; // Parent element's tag name
  isInline: boolean;        // Inline vs block element
}
```

**Example:**
```html
<div><p><a href="https://example.com">Link</a></p></div>
```

When visiting the `<a>` tag:
- `nodeType`: "element"
- `tagName`: "a"
- `attributes`: `{ href: "https://example.com" }`
- `depth`: 2 (nested inside `<div>` and `<p>`)
- `indexInParent`: 0 (first child of `<p>`)
- `parentTag`: "p"
- `isInline`: true

### VisitResult

Visitor methods return a `VisitResult` to control conversion:

| Type | Description | Use Case |
|------|-------------|----------|
| `Continue` | Use default Markdown conversion | Accept element as-is |
| `Custom` | Provide custom Markdown output | Transform element differently |
| `Skip` | Skip element entirely | Remove unwanted content |
| `PreserveHtml` | Keep original HTML unchanged | Preserve complex structures |
| `Error` | Stop conversion with error | Fail on invalid content |

**Return value formats:**

**Python:**
```python
# Continue with default conversion
return {"type": "continue"}

# Custom output
return {"type": "custom", "output": "[custom markdown]"}

# Skip element
return {"type": "skip"}

# Preserve HTML
return {"type": "preserve_html"}

# Error
return {"type": "error", "message": "Invalid link"}
```

**TypeScript:**
```typescript
// Continue with default conversion
return { type: 'continue' };

// Custom output
return { type: 'custom', output: '[custom markdown]' };

// Skip element
return { type: 'skip' };

// Preserve HTML
return { type: 'preserve_html' };

// Error
return { type: 'error', message: 'Invalid link' };
```

**Ruby:**
```ruby
# Continue with default conversion
{ type: :continue }

# Custom output
{ type: :custom, output: '[custom markdown]' }

# Skip element
{ type: :skip }

# Preserve HTML
{ type: :preserve_html }

# Error
{ type: :error, message: 'Invalid link' }
```

---

## Supported Visitor Methods

The visitor pattern supports 40+ methods for different HTML elements:

### Text & Inline Elements

| Method | Parameters | Description |
|--------|------------|-------------|
| `visit_text` | `(ctx, text)` | Plain text nodes |
| `visit_strong` | `(ctx, text)` | Bold text `<strong>`, `<b>` |
| `visit_em` | `(ctx, text)` | Italic text `<em>`, `<i>` |
| `visit_code` | `(ctx, code)` | Inline code `<code>` |
| `visit_strikethrough` | `(ctx, text)` | Strikethrough `<s>`, `<del>` |
| `visit_underline` | `(ctx, text)` | Underlined text `<u>` |
| `visit_superscript` | `(ctx, text)` | Superscript `<sup>` |
| `visit_subscript` | `(ctx, text)` | Subscript `<sub>` |

### Links & Images

| Method | Parameters | Description |
|--------|------------|-------------|
| `visit_link` | `(ctx, href, text, title?)` | Hyperlinks `<a>` |
| `visit_image` | `(ctx, src, alt?, title?)` | Images `<img>` |

### Headings

| Method | Parameters | Description |
|--------|------------|-------------|
| `visit_heading` | `(ctx, level, text)` | All headings `<h1>` to `<h6>` |
| `visit_h1` | `(ctx, text)` | Level 1 headings |
| `visit_h2` | `(ctx, text)` | Level 2 headings |
| `visit_h3` | `(ctx, text)` | Level 3 headings |
| `visit_h4` | `(ctx, text)` | Level 4 headings |
| `visit_h5` | `(ctx, text)` | Level 5 headings |
| `visit_h6` | `(ctx, text)` | Level 6 headings |

### Lists

| Method | Parameters | Description |
|--------|------------|-------------|
| `visit_list` | `(ctx, is_ordered)` | `<ul>` or `<ol>` containers |
| `visit_list_item` | `(ctx, content)` | List items `<li>` |

### Block Elements

| Method | Parameters | Description |
|--------|------------|-------------|
| `visit_paragraph` | `(ctx, text)` | Paragraphs `<p>` |
| `visit_blockquote` | `(ctx, content)` | Blockquotes `<blockquote>` |
| `visit_code_block` | `(ctx, code, language?)` | Code blocks `<pre><code>` |
| `visit_horizontal_rule` | `(ctx)` | Horizontal rules `<hr>` |
| `visit_div` | `(ctx, content)` | Generic containers `<div>` |

### Tables

| Method | Parameters | Description |
|--------|------------|-------------|
| `visit_table` | `(ctx)` | Table containers `<table>` |
| `visit_table_row` | `(ctx)` | Table rows `<tr>` |
| `visit_table_cell` | `(ctx, content, is_header)` | Table cells `<td>`, `<th>` |

### Other Elements

| Method | Parameters | Description |
|--------|------------|-------------|
| `visit_br` | `(ctx)` | Line breaks `<br>` |
| `visit_input` | `(ctx, input_type)` | Form inputs `<input>` |
| `visit_form` | `(ctx)` | Forms `<form>` |
| `visit_button` | `(ctx, text)` | Buttons `<button>` |

---

## Quick Start by Language

### Python (Synchronous)

```python
from html_to_markdown import convert_with_visitor

class MyVisitor:
    def visit_link(self, ctx, href, text, title):
        # Rewrite CDN URLs
        if href.startswith("https://old-cdn.com"):
            href = href.replace("https://old-cdn.com", "https://new-cdn.com")
        return {"type": "custom", "output": f"[{text}]({href})"}

    def visit_image(self, ctx, src, alt, title):
        # Skip tracking pixels
        if "tracking" in src or "analytics" in src:
            return {"type": "skip"}
        return {"type": "continue"}

html = '<a href="https://old-cdn.com/file.pdf">Download</a>'
markdown = convert_with_visitor(html, visitor=MyVisitor())
# Output: "[Download](https://new-cdn.com/file.pdf)"
```

### Python (Asynchronous)

```python
import asyncio
from html_to_markdown import convert_with_async_visitor

class AsyncVisitor:
    async def visit_link(self, ctx, href, text, title):
        # Validate URLs asynchronously
        is_valid = await validate_url(href)
        if not is_valid:
            return {"type": "error", "message": f"Broken link: {href}"}
        return {"type": "continue"}

    def visit_image(self, ctx, src, alt, title):
        # Sync methods work too
        if not alt:
            return {"type": "error", "message": "Image missing alt text"}
        return {"type": "continue"}

async def validate_url(url):
    # Simulate async URL validation
    await asyncio.sleep(0.1)
    return not url.endswith(".broken")

html = '<a href="https://example.com">Link</a>'
markdown = convert_with_async_visitor(html, visitor=AsyncVisitor())
```

### TypeScript (Synchronous)

```typescript
import { convertWithVisitor, type Visitor, type NodeContext, type VisitResult } from 'html-to-markdown';

const visitor: Visitor = {
  visitLink(ctx: NodeContext, href: string, text: string, title?: string): VisitResult {
    // Rewrite CDN URLs
    if (href.startsWith('https://old-cdn.com')) {
      href = href.replace('https://old-cdn.com', 'https://new-cdn.com');
    }
    return { type: 'custom', output: `[${text}](${href})` };
  },

  visitImage(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult {
    // Skip tracking pixels
    if (src.includes('tracking') || src.includes('analytics')) {
      return { type: 'skip' };
    }
    return { type: 'continue' };
  },
};

const html = '<a href="https://old-cdn.com/file.pdf">Download</a>';
const markdown = convertWithVisitor(html, { visitor });
// Output: "[Download](https://new-cdn.com/file.pdf)"
```

### TypeScript (Asynchronous)

```typescript
import { convertWithAsyncVisitor, type AsyncVisitor, type NodeContext, type VisitResult } from 'html-to-markdown';

const asyncVisitor: AsyncVisitor = {
  async visitLink(ctx: NodeContext, href: string, text: string, title?: string): Promise<VisitResult> {
    // Validate URLs asynchronously
    const isValid = await validateUrl(href);
    if (!isValid) {
      return { type: 'error', message: `Broken link: ${href}` };
    }
    return { type: 'continue' };
  },

  visitImage(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult {
    // Sync methods work too
    if (!alt) {
      return { type: 'error', message: 'Image missing alt text' };
    }
    return { type: 'continue' };
  },
};

async function validateUrl(url: string): Promise<boolean> {
  // Simulate async URL validation
  await new Promise(resolve => setTimeout(resolve, 100));
  return !url.endsWith('.broken');
}

const html = '<a href="https://example.com">Link</a>';
const markdown = await convertWithAsyncVisitor(html, { visitor: asyncVisitor });
```

### Ruby

```ruby
require 'html_to_markdown'

class MyVisitor
  # Rewrite CDN URLs
  def visit_link(ctx, href, text, title = nil)
    if href.start_with?('https://old-cdn.com')
      href = href.sub('https://old-cdn.com', 'https://new-cdn.com')
    end
    { type: :custom, output: "[#{text}](#{href})" }
  end

  # Skip tracking pixels
  def visit_image(ctx, src, alt = nil, title = nil)
    if src.include?('tracking') || src.include?('analytics')
      { type: :skip }
    else
      { type: :continue }
    end
  end
end

html = '<a href="https://old-cdn.com/file.pdf">Download</a>'
markdown = HtmlToMarkdown.convert_with_visitor(html, visitor: MyVisitor.new)
# Output: "[Download](https://new-cdn.com/file.pdf)"
```

### PHP

```php
<?php
use HtmlToMarkdown\Converter;

readonly class MyVisitor {
    public function visitLink(array $ctx, string $href, string $text, ?string $title): array {
        // Rewrite CDN URLs
        if (str_starts_with($href, 'https://old-cdn.com')) {
            $href = str_replace('https://old-cdn.com', 'https://new-cdn.com', $href);
        }
        return ['type' => 'custom', 'output' => "[{$text}]({$href})"];
    }

    public function visitImage(array $ctx, string $src, ?string $alt, ?string $title): array {
        // Skip tracking pixels
        if (str_contains($src, 'tracking') || str_contains($src, 'analytics')) {
            return ['type' => 'skip'];
        }
        return ['type' => 'continue'];
    }
}

$html = '<a href="https://old-cdn.com/file.pdf">Download</a>';
$markdown = Converter::convertWithVisitor($html, new MyVisitor());
// Output: "[Download](https://new-cdn.com/file.pdf)"
```

---

## Practical Examples

### 1. CDN Image URL Rewriting

Rewrite all image sources to use a new CDN:

```python
class CdnRewriter:
    def __init__(self, old_cdn, new_cdn):
        self.old_cdn = old_cdn
        self.new_cdn = new_cdn

    def visit_image(self, ctx, src, alt, title):
        if src.startswith(self.old_cdn):
            src = src.replace(self.old_cdn, self.new_cdn)
        return {"type": "custom", "output": f"![{alt or ''}]({src})"}

visitor = CdnRewriter("https://old.cdn.com", "https://new.cdn.com")
markdown = convert_with_visitor(html, visitor=visitor)
```

See: [`cdn-rewrite.py`](./cdn-rewrite.py), [`cdn-rewrite.ts`](./cdn-rewrite.ts), [`cdn-rewrite.rb`](./cdn-rewrite.rb)

### 2. Content Filtering

Remove unwanted elements like ads, tracking pixels, or specific classes:

```python
class ContentFilter:
    def visit_div(self, ctx, content):
        # Remove divs with class="ad" or class="tracking"
        classes = ctx.attributes.get("class", "")
        if "ad" in classes or "tracking" in classes:
            return {"type": "skip"}
        return {"type": "continue"}

    def visit_script(self, ctx):
        # Always remove scripts
        return {"type": "skip"}

    def visit_image(self, ctx, src, alt, title):
        # Remove tracking pixels (1x1 images)
        width = ctx.attributes.get("width")
        height = ctx.attributes.get("height")
        if width == "1" and height == "1":
            return {"type": "skip"}
        return {"type": "continue"}

markdown = convert_with_visitor(html, visitor=ContentFilter())
```

See: [`content-filter.py`](./content-filter.py), [`content-filter.ts`](./content-filter.ts)

### 3. Link Footnote References

Convert links to footnote-style references:

```python
class FootnoteVisitor:
    def __init__(self):
        self.footnotes = []

    def visit_link(self, ctx, href, text, title):
        # Add to footnote list
        footnote_num = len(self.footnotes) + 1
        self.footnotes.append((footnote_num, href, title))

        # Return text with footnote reference
        return {"type": "custom", "output": f"{text}[^{footnote_num}]"}

    def get_footnotes(self):
        # Generate footnote section
        lines = ["\n\n---\n"]
        for num, href, title in self.footnotes:
            title_text = f' "{title}"' if title else ""
            lines.append(f"[^{num}]: {href}{title_text}")
        return "\n".join(lines)

visitor = FootnoteVisitor()
markdown = convert_with_visitor(html, visitor=visitor)
markdown += visitor.get_footnotes()
```

### 4. Accessibility Validation

Check for common accessibility issues and fail conversion if found:

```python
class AccessibilityChecker:
    def visit_image(self, ctx, src, alt, title):
        # Fail if images are missing alt text
        if not alt or not alt.strip():
            return {"type": "error", "message": f"Image missing alt text: {src}"}
        return {"type": "continue"}

    def visit_link(self, ctx, href, text, title):
        # Fail if links have no text
        if not text or not text.strip():
            return {"type": "error", "message": f"Link missing text: {href}"}
        return {"type": "continue"}

    def visit_heading(self, ctx, level, text):
        # Check heading hierarchy (no skipping levels)
        if not hasattr(self, "last_heading_level"):
            self.last_heading_level = 0

        if level > self.last_heading_level + 1:
            return {"type": "error", "message": f"Heading skips level: h{self.last_heading_level} → h{level}"}

        self.last_heading_level = level
        return {"type": "continue"}

markdown = convert_with_visitor(html, visitor=AccessibilityChecker())
```

See: [`accessibility-check.py`](./accessibility-check.py)

### 5. Async URL Validation

Validate all URLs asynchronously and fail on broken links:

```python
import asyncio
import aiohttp

class AsyncUrlValidator:
    def __init__(self):
        self.session = None

    async def visit_link(self, ctx, href, text, title):
        # Skip anchor links
        if href.startswith("#"):
            return {"type": "continue"}

        # Validate external URLs
        if self.session is None:
            self.session = aiohttp.ClientSession()

        try:
            async with self.session.head(href, timeout=5) as response:
                if response.status >= 400:
                    return {"type": "error", "message": f"Broken link ({response.status}): {href}"}
        except Exception as e:
            return {"type": "error", "message": f"Failed to validate {href}: {str(e)}"}

        return {"type": "continue"}

    async def visit_image(self, ctx, src, alt, title):
        # Validate image URLs
        if not src.startswith(("http://", "https://")):
            return {"type": "continue"}

        if self.session is None:
            self.session = aiohttp.ClientSession()

        try:
            async with self.session.head(src, timeout=5) as response:
                if response.status >= 400:
                    return {"type": "error", "message": f"Broken image ({response.status}): {src}"}
        except Exception as e:
            return {"type": "error", "message": f"Failed to validate {src}: {str(e)}"}

        return {"type": "continue"}

visitor = AsyncUrlValidator()
try:
    markdown = await convert_with_async_visitor(html, visitor=visitor)
finally:
    if visitor.session:
        await visitor.session.close()
```

See: [`async-validation.py`](./async-validation.py), [`async-validation.ts`](./async-validation.ts)

---

## Performance Considerations

### Minimal Overhead

- **Single-pass traversal**: Visitor callbacks execute during tree traversal (no second pass)
- **Zero-cost when disabled**: No overhead if visitor pattern is not used
- **Compiled callbacks**: Visitor methods are called directly (no reflection or dynamic dispatch)

### Stateful Visitors

Visitors can maintain state across method calls:

```python
class StatefulVisitor:
    def __init__(self):
        self.link_count = 0
        self.image_count = 0

    def visit_link(self, ctx, href, text, title):
        self.link_count += 1
        return {"type": "continue"}

    def visit_image(self, ctx, src, alt, title):
        self.image_count += 1
        return {"type": "continue"}
```

**Important**: Visitor instances are NOT thread-safe. Create a new instance per conversion when processing multiple documents concurrently.

### Async Method Overhead

- **Python**: `convert_with_async_visitor()` uses asyncio event loop
- **TypeScript**: `convertWithAsyncVisitor()` returns Promise
- **Recommendation**: Use async only when performing I/O (network requests, file reads)

### Error Handling

Returning `{"type": "error"}` immediately stops conversion and raises an exception. For non-critical issues, consider:
- Logging warnings and returning `{"type": "continue"}`
- Collecting issues in visitor state for post-processing
- Using `{"type": "skip"}` to silently remove invalid elements

---

## Working Examples

This directory contains executable examples demonstrating the visitor pattern:

- **`cdn-rewrite.py`**, **`cdn-rewrite.ts`**, **`cdn-rewrite.rb`** - CDN URL rewriting
- **`content-filter.py`**, **`content-filter.ts`** - Content filtering and element removal
- **`accessibility-check.py`** - Accessibility validation (alt text, heading hierarchy)
- **`async-validation.py`**, **`async-validation.ts`** - Asynchronous URL validation

Run the Python examples:
```bash
cd examples/visitor-pattern
pip install -r requirements.txt
python cdn-rewrite.py
python async-validation.py
```

Run the TypeScript examples:
```bash
cd examples/visitor-pattern
npm install
npx tsx cdn-rewrite.ts
npx tsx async-validation.ts
```

Run the Ruby examples:
```bash
cd examples/visitor-pattern
bundle install
ruby cdn-rewrite.rb
```

---

## See Also

- **[Metadata Extraction](../metadata-extraction/)** - Extract document metadata during conversion
- **[Performance Guide](../performance/)** - Benchmarking and optimization
- **Package READMEs** - Language-specific API documentation
  - [Python](../../packages/python/)
  - [TypeScript](../../packages/typescript/)
  - [Ruby](../../packages/ruby/)
  - [PHP](../../packages/php/)
