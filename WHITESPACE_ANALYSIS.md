# Whitespace Normalization Analysis

## Problem Statement

The Rust implementation doesn't match Python's whitespace handling, causing test failures:

- Empty inline elements leave extra spaces
- Semantic blocks don't preserve whitespace correctly
- Missing post-processing to collapse multiple spaces

## Python Implementation Behavior

### 1. Whitespace Handler (During Conversion)

The `WhitespaceHandler` (`html_to_markdown/whitespace.py`) processes text nodes:

**Whitespace-only text nodes**:

- Between block elements → `""`
- Contains newline → `""`
- Adjacent to inline elements → `" "`
- Otherwise → `""`

**Text with content**:

- Apply `chomp()`: preserve leading/trailing space as single space
- Normalize unicode spaces to regular space
- Collapse multiple spaces within content to single space

**Special cases**:

- `<pre>`, `<code>`, `<kbd>`, `<samp>`: preserve all whitespace
- `<ruby>`, `<select>`, `<datalist>`: preserve exact spacing

### 2. Empty Inline Elements

Elements like `<abbr>`, `<var>`, `<ins>`, `<dfn>` with no content return `""`:

```python
def _create_inline_converter(markup_prefix: str):
    def implementation(*, tag: Tag, text: str) -> str:
        if not text.strip():
            return ""  # ← Empty elements disappear
        # ... chomp and format ...
```

This causes spaces between empty elements to accumulate:

```html
<p>Empty: <abbr></abbr> <var></var> <ins></ins></p>
```

Processing:

1. "Empty: " → "Empty: " (1 space)
1. `<abbr></abbr>` → ""
1. " " (text node) → " " (1 space)
1. `<var></var>` → ""
1. " " (text node) → " " (1 space)
1. `<ins></ins>` → ""

Result before post-processing: `"Empty:    "` (4 spaces)

### 3. Post-Processing (After Conversion)

Python applies regex post-processing (`processing.py:779-799`):

````python
def normalize_spaces_outside_code(text: str) -> str:
    parts = text.split("```")  # Split on code blocks
    for i in range(0, len(parts), 2):  # Process non-code sections
        lines = parts[i].split("\n")
        for j, line in enumerate(lines):
            def_parts = line.split("``")  # Split on inline code
            for k in range(0, len(def_parts), 2):  # Process non-inline-code
                match = re.match(r"^(\s*)(.*)", def_parts[k])
                if match:
                    leading_spaces, rest = match.groups()
                    rest = re.sub(r" {3,}", " ", rest)  # ← Collapse 3+ spaces to 1
                    def_parts[k] = leading_spaces + rest
            lines[j] = "``".join(def_parts)
        parts[i] = "\n".join(lines)
    return "```".join(parts)

result = normalize_spaces_outside_code(result)

# Also clean spaces around ** markers
result = re.sub(r"\*\* {2,}", "** ", result)
result = re.sub(r" {2,}\*\*", " **", result)
````

This explains:

- `"Empty:    "` (4 spaces) → `"Empty: "` (1 space)
- `"text  **bold**"` → `"text **bold**"`

### 4. Semantic Block Elements

The `_convert_semantic_block` function:

```python
def _convert_semantic_block(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    return f"{text}\n\n" if text.strip() else ""
```

Key behavior:

- Adds `\n\n` suffix if content exists
- Returns `""` if whitespace-only (after stripping)
- Does NOT trim the content itself - preserves leading/trailing spaces

Example:

```html
<section>\n Content with whitespace \n</section>
```

Processing:

1. Text nodes processed by WhitespaceHandler
1. Result: `" Content with whitespace "`
1. `text.strip()` check: `"Content with whitespace"` (not empty)
1. Output: `" Content with whitespace \n\n"` ✓

## Rust Implementation Issues

### Current Problems

1. **No post-processing**: Doesn't collapse 3+ spaces
1. **Semantic blocks trim content**: Uses `trimmed_content` instead of preserving spaces
1. **No chomp for inline elements**: Loses leading/trailing space preservation

### Required Changes

1. **Add post-processing function**:

   ````rust
   fn normalize_spaces(text: &str) -> String {
       // Split on code blocks (```)
       // For non-code sections:
       //   - Split on inline code (``)
       //   - Collapse 3+ spaces to 1
       //   - Clean spaces around **
   }
   ````

1. **Fix semantic blocks**:

   ```rust
   // Currently:
   output.push_str(&content);  // Already includes whitespace

   // Should check if trimmed is empty:
   if content.trim().is_empty() {
       return;  // Skip empty blocks
   }
   // But output the full content with spaces preserved
   ```

1. **Implement chomp**:

   ```rust
   fn chomp(text: &str) -> (&str, &str, &str) {
       let prefix = if text.starts_with(&[' ', '\t'][..]) { " " } else { "" };
       let suffix = if text.ends_with(&[' ', '\t'][..]) { " " } else { "" };
       (prefix, suffix, text.trim())
   }
   ```

## Test Cases

### Test 1: Empty inline elements

```html
<p>Empty elements: <abbr></abbr> <var></var> <ins></ins> <dfn></dfn></p>
```

Expected: `"Empty elements: \n\n"`

- Before post-processing: `"Empty elements:    \n\n"` (4 spaces)
- After post-processing: `"Empty elements: \n\n"` (1 space)

### Test 2: Inline element whitespace

```html
<p>
  Spaces around <var> variable </var> and <abbr title="  title  "> abbr </abbr>
</p>
```

Expected: `"Spaces around  *variable*  and abbr (title)\n\n"`

- `<var>` chomps: prefix=" ", suffix=" ", text="variable"
- `<abbr>` chomps content and title: "abbr" and "title"

### Test 3: Semantic block whitespace

```html
<section>\n Content with whitespace \n</section>
```

Expected: `" Content with whitespace \n\n"`

- WhitespaceHandler processes text nodes
- Preserves leading/trailing spaces
- Adds `\n\n` suffix

## Implementation Plan

1. ✅ Analyze Python behavior (completed)
1. ✅ Document findings (completed)
1. 🔄 Restore Rust changes from stash
1. ⏭️ Add `chomp()` function (already done)
1. ⏭️ Fix semantic block whitespace preservation
1. ⏭️ Add post-processing: `normalize_spaces()`
1. ⏭️ Apply post-processing in `convert()`
1. ⏭️ Test with elements_test.py
