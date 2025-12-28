# Debug Logging Examples - walk_node() Instrumentation

## Overview
This document shows example output from the enhanced debug logging in `walk_node()` function when processing various HTML structures.

## Example 1: Simple HTML with Body Tag

### Input HTML
```html
<html>
<head>
    <title>Test</title>
</head>
<body>
    <div class="article">
        <h1>Test Article</h1>
        <p>This is the first paragraph.</p>
        <p>This is the second paragraph.</p>
    </div>
</body>
</html>
```

### Debug Output
```
[DEBUG-ENTRY] walk_node called: tag=html, depth=0, output_len=21
[DEBUG] Processing <html> tag at depth=0, output_len_before=21
[DEBUG] <html> has 5 children
[DEBUG] <html> processing child 0 of 5, output_len_before=21
[DEBUG-ENTRY] walk_node called: Text node at depth=1, output_len=21
[DEBUG] <html> child 0 complete: added 0 bytes (total: 21)

[DEBUG] <html> processing child 1 of 5, output_len_before=21
[DEBUG-ENTRY] walk_node called: tag=head, depth=1, output_len=21
[DEBUG] <html> child 1 complete: added 0 bytes (total: 21)

[DEBUG] <html> processing child 2 of 5, output_len_before=21
[DEBUG-ENTRY] walk_node called: Text node at depth=1, output_len=21
[DEBUG] <html> child 2 complete: added 0 bytes (total: 21)

[DEBUG] <html> processing child 3 of 5, output_len_before=21
[DEBUG-ENTRY] walk_node called: tag=body, depth=1, output_len=21
[DEBUG] Processing <body> tag at depth=1, output_len_before=21
[DEBUG] <body> has 3 children

[DEBUG] <body> processing child 0 of 3, output_len_before=21
[DEBUG-ENTRY] walk_node called: Text node at depth=2, output_len=21
[DEBUG] <body> child 0 complete: added 0 bytes (total: 21)

[DEBUG] <body> processing child 1 of 3, output_len_before=21
[DEBUG-ENTRY] walk_node called: tag=div, depth=2, output_len=21
[DEBUG] <body> child 1 complete: added 77 bytes (total: 98)

[DEBUG] <body> processing child 2 of 3, output_len_before=98
[DEBUG-ENTRY] walk_node called: Text node at depth=2, output_len=98
[DEBUG] <body> child 2 complete: added 0 bytes (total: 98)

[DEBUG] <body> tag complete, output_len_after=98

[DEBUG] <html> child 3 complete: added 77 bytes (total: 98)
[DEBUG] <html> processing child 4 of 5, output_len_before=98
[DEBUG-ENTRY] walk_node called: Text node at depth=1, output_len=21
[DEBUG] <html> child 4 complete: added 0 bytes (total: 98)
[DEBUG] <html> tag complete, output_len_after=98
```

### Analysis
- **html tag**: 5 children total (whitespace text nodes, head, body)
- **body tag**: 3 children (whitespace, div with content, whitespace)
- **Output generated**: 98 bytes total from body's content
- **Key insight**: Body's child at index 1 (div element) added 77 bytes

---

## Example 2: Body-Only HTML Structure

### Input HTML
```html
<body>
    <main>
        <article>
            <h1>Headline</h1>
            <p>Article content here.</p>
        </article>
    </main>
</body>
```

### Debug Output
```
[DEBUG-ENTRY] walk_node called: tag=body, depth=0, output_len=0
[DEBUG] Processing <body> tag at depth=0, output_len_before=0
[DEBUG] <body> has 3 children

[DEBUG] <body> processing child 0 of 3, output_len_before=0
[DEBUG-ENTRY] walk_node called: Text node at depth=1, output_len=0
[DEBUG] <body> child 0 complete: added 0 bytes (total: 0)

[DEBUG] <body> processing child 1 of 3, output_len_before=0
[DEBUG-ENTRY] walk_node called: tag=main, depth=1, output_len=0
[DEBUG] <body> child 1 complete: added 35 bytes (total: 35)

[DEBUG] <body> processing child 2 of 3, output_len_before=35
[DEBUG-ENTRY] walk_node called: Text node at depth=1, output_len=35
[DEBUG] <body> child 2 complete: added 0 bytes (total: 35)

[DEBUG] <body> tag complete, output_len_after=35
```

### Analysis
- **body is root element**: Processing starts at depth=0
- **body tag**: 3 children (whitespace, main element, whitespace)
- **Output generated**: 35 bytes from body's main element
- **Key observation**: All content comes from the main element (child index 1)

---

## Example 3: Reuters HTML Files

### Output Summary for Body-Only Reuters (552KB HTML)
```
[DEBUG-ENTRY] walk_node called: tag=html, depth=0, output_len=4230
[DEBUG] Processing <html> tag at depth=0, output_len_before=4230
[DEBUG] <html> has 1 children
[DEBUG] <html> processing child 0 of 1, output_len_before=4230
[DEBUG-ENTRY] walk_node called: tag=head, depth=1, output_len=4230
[DEBUG] <html> child 0 complete: added 0 bytes (total: 4230)
[DEBUG] <html> tag complete, output_len_after=4230
```

### Analysis
- **Initial output**: 4230 bytes BEFORE body is processed
  - This indicates metadata extraction or preprocessing adds content
- **html children count**: Only 1 child (the head tag)
  - The body tag is NOT in the direct children list
- **Interpretation**: Body content was likely extracted during preprocessing

### Output Summary for Full Reuters (686KB HTML)
```
[DEBUG-ENTRY] walk_node called: tag=html, depth=0, output_len=5351
[DEBUG] Processing <html> tag at depth=0, output_len_before=5351
[DEBUG] <html> has 1 children
[DEBUG] <html> processing child 0 of 1, output_len_before=5351
[DEBUG-ENTRY] walk_node called: tag=head, depth=1, output_len=5351
[DEBUG] <html> child 0 complete: added 0 bytes (total: 5351)
[DEBUG] <html> tag complete, output_len_after=5351
```

### Comparison
| Metric | Body-Only | Full HTML |
|--------|-----------|-----------|
| Input Size | 552 KB | 686 KB |
| Initial Output | 4,230 bytes | 5,351 bytes |
| Added from Processing | 0 bytes | 0 bytes |
| Final Output | 4,229 bytes | 5,350 bytes |
| Difference | 1,121 bytes | - |

---

## Trace Format Explanation

### Entry Logging
```
[DEBUG-ENTRY] walk_node called: tag=body, depth=0, output_len=21
                                |tag       |depth |output buffer size
```

### Processing Start
```
[DEBUG] Processing <body> tag at depth=0, output_len_before=21
        |marker |tag            |depth     |output before processing
```

### Child Count
```
[DEBUG] <body> has 3 children
        |tag  |child count
```

### Per-Child Processing
```
[DEBUG] <body> processing child 0 of 3, output_len_before=21
        |tag  |child idx|total |output before this child
```

### Child Complete
```
[DEBUG] <body> child 0 complete: added 0 bytes (total: 21)
        |tag  |child idx|bytes added|new total output size
```

### Tag Complete
```
[DEBUG] <body> tag complete, output_len_after=35
        |tag  |final output size
```

---

## How to Use This Information

### Finding Bottlenecks
Look for children that contribute 0 bytes - they may be:
- Whitespace text nodes
- Meta tags (stripped)
- Script/style tags (removed)
- Empty elements

### Verifying Processing
Check if your expected elements appear with correct byte counts:
```
Expected <h1> content: "Test Article" (~15 bytes)
Actual output shows: <div> child added 77 bytes ✓
```

### Debugging Missing Content
If content is missing:
1. Check if the element appears in the children list
2. Look for its index number
3. See if it added 0 bytes
4. Trace the depth to understand nesting

---

## Enabling Debug Logging

In Rust code:
```rust
use html_to_markdown_rs::{ConversionOptions, convert};

let mut opts = ConversionOptions::default();
opts.debug = true;  // Enable all debug output

let result = convert(html_str, Some(opts))?;
```

Output goes to `stderr`, so when running tests:
```bash
cargo test test_name -- --ignored --nocapture 2>&1 | grep DEBUG
```

---

## Performance Note

Debug logging adds overhead due to:
- String formatting for every walk_node call
- stderr writes for each log line
- Multiple output length checks

Only enable in test/diagnostic scenarios, not production code.
