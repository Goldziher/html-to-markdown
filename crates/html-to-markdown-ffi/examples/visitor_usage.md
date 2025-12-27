# Visitor Pattern FFI Usage Examples

This document provides comprehensive examples of implementing custom visitors using the C FFI types defined in `crates/html-to-markdown-ffi/src/visitor/`.

## Overview

The visitor pattern allows language bindings (Go, Java, C#, etc.) to intercept and customize HTML element processing during conversion. All FFI types are defined as `#[repr(C)]` structs and enums for maximum compatibility.

## Core Types

### Node Type Enumeration

Covers all 88 HTML element types:
- **Text/Generic**: TEXT, ELEMENT
- **Block Elements**: HEADING, PARAGRAPH, DIV, BLOCKQUOTE, PRE, HR
- **Lists**: LIST, LIST_ITEM, DEFINITION_LIST, DEFINITION_TERM, DEFINITION_DESCRIPTION
- **Tables**: TABLE, TABLE_ROW, TABLE_CELL, TABLE_HEADER, TABLE_BODY, TABLE_HEAD, TABLE_FOOT
- **Inline Formatting**: LINK, IMAGE, STRONG, EM, CODE, STRIKETHROUGH, UNDERLINE, SUBSCRIPT, SUPERSCRIPT, MARK, SMALL, BR, SPAN
- **Semantic HTML5**: ARTICLE, SECTION, NAV, ASIDE, HEADER, FOOTER, MAIN, FIGURE, FIGCAPTION, TIME, DETAILS, SUMMARY
- **Forms**: FORM, INPUT, SELECT, OPTION, BUTTON, TEXTAREA, LABEL, FIELDSET, LEGEND
- **Media**: AUDIO, VIDEO, PICTURE, SOURCE, IFRAME, SVG, CANVAS
- **Advanced**: RUBY, RT, RP, ABBR, KBD, SAMP, VAR, CITE, Q, DEL, INS, DATA, METER, PROGRESS, OUTPUT, TEMPLATE, SLOT
- **Document**: HTML, HEAD, BODY, TITLE, META, LINK_TAG, STYLE, SCRIPT, BASE
- **Custom**: CUSTOM (web components)

### Visit Result Type

Controls conversion flow:
```c
typedef enum {
    HTML_TO_MARKDOWN_VISIT_CONTINUE = 0,      // Use default behavior
    HTML_TO_MARKDOWN_VISIT_CUSTOM = 1,        // Replace with custom output
    HTML_TO_MARKDOWN_VISIT_SKIP = 2,          // Omit element entirely
    HTML_TO_MARKDOWN_VISIT_PRESERVE_HTML = 3, // Keep raw HTML
    HTML_TO_MARKDOWN_VISIT_ERROR = 4          // Halt conversion
} html_to_markdown_visit_result_type_t;
```

## Memory Management

### Rules

- **Borrowed data**: NodeContext, tag_name, attributes, text - all borrowed from Rust
  - Do NOT free these pointers
  - Valid only during callback execution
  - Copy immediately if you need to persist

- **Allocated data**: custom_output, error_message in VisitResult
  - Caller must allocate (malloc/new)
  - Converter takes ownership and frees
  - Must be UTF-8 C strings

### Attributes Iteration

```c
// Iterate over NULL-terminated attributes array
for (size_t i = 0; attributes[i].key != NULL; i++) {
    printf("%s = %s\n", attributes[i].key, attributes[i].value);
}
```

## Language-Specific Examples

### C

Complete example of filtering out all link elements:

```c
#include <stdlib.h>
#include <string.h>
#include "html_to_markdown.h"

typedef struct {
    int link_count;
    int image_count;
} MyVisitorState;

html_to_markdown_visit_result_t my_visit_link(
    void *user_data,
    const html_to_markdown_node_context_t *ctx,
    const char *href,
    const char *text,
    const char *title)
{
    MyVisitorState *state = (MyVisitorState *)user_data;
    state->link_count++;

    html_to_markdown_visit_result_t result = {0};
    result.result_type = HTML_TO_MARKDOWN_VISIT_SKIP;  // Skip all links
    return result;
}

html_to_markdown_visit_result_t my_visit_image(
    void *user_data,
    const html_to_markdown_node_context_t *ctx,
    const char *src,
    const char *alt,
    const char *title)
{
    MyVisitorState *state = (MyVisitorState *)user_data;
    state->image_count++;

    html_to_markdown_visit_result_t result = {0};
    result.result_type = HTML_TO_MARKDOWN_VISIT_SKIP;  // Skip all images
    return result;
}

int main() {
    MyVisitorState state = {0};

    html_to_markdown_visitor_t visitor = {
        .user_data = &state,
        .visit_link = my_visit_link,
        .visit_image = my_visit_image,
        // ... other callbacks (set to NULL if unused)
    };

    const char *html = "<a href='https://example.com'>Link</a><img src='test.jpg'>";

    // Note: actual conversion with visitor would be implemented in FFI layer
    // char *markdown = html_to_markdown_convert_with_visitor(html, &visitor);

    // After conversion:
    printf("Found %d links and %d images\n", state.link_count, state.image_count);

    return 0;
}
```

### Go (cgo)

Using the FFI with Go's cgo:

```go
package main

/*
#include "html_to_markdown.h"
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"unsafe"
)

type VisitorState struct {
	LinkCount  int32
	ImageCount int32
}

//export visitLink
func visitLink(userData unsafe.Pointer, ctx *C.html_to_markdown_node_context_t,
	href, text, title *C.char) C.html_to_markdown_visit_result_t {

	state := (*VisitorState)(userData)
	state.LinkCount++

	result := C.html_to_markdown_visit_result_t{
		result_type: C.HTML_TO_MARKDOWN_VISIT_SKIP,
	}
	return result
}

//export visitImage
func visitImage(userData unsafe.Pointer, ctx *C.html_to_markdown_node_context_t,
	src, alt, title *C.char) C.html_to_markdown_visit_result_t {

	state := (*VisitorState)(userData)
	state.ImageCount++

	result := C.html_to_markdown_visit_result_t{
		result_type: C.HTML_TO_MARKDOWN_VISIT_SKIP,
	}
	return result
}

func main() {
	state := VisitorState{}

	visitor := C.html_to_markdown_visitor_t{
		user_data:   unsafe.Pointer(&state),
		visit_link:  (C.html_to_markdown_visit_result_t)(unsafe.Pointer(uintptr(0))),
		visit_image: (C.html_to_markdown_visit_result_t)(unsafe.Pointer(uintptr(0))),
	}

	html := C.CString("<a href='https://example.com'>Link</a><img src='test.jpg'>")
	defer C.free(unsafe.Pointer(html))

	// Would call html_to_markdown_convert_with_visitor here

	fmt.Printf("Found %d links and %d images\n", state.LinkCount, state.ImageCount)
}
```

### Java (Panama FFM)

Using Java 21+ Panama FFM bindings:

```java
import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;

public class VisitorExample {
    static class VisitorState {
        int linkCount = 0;
        int imageCount = 0;
    }

    // FFI bindings for callback pointers
    interface LinkCallback {
        int visitLink(MemorySegment userData, MemorySegment ctx,
                      MemorySegment href, MemorySegment text, MemorySegment title);
    }

    static LinkCallback createLinkVisitor() {
        return (userData, ctx, href, text, title) -> {
            VisitorState state = (VisitorState) null; // Unwrap from userData
            state.linkCount++;
            // Return HTML_TO_MARKDOWN_VISIT_SKIP (2)
            return 2;
        };
    }

    public static void main(String[] args) {
        try (Arena arena = Arena.ofConfined()) {
            VisitorState state = new VisitorState();

            // Create visitor struct with callbacks
            MemorySegment visitor = arena.allocateArray(
                ValueLayout.JAVA_LONG, 100); // Rough size estimate

            String html = "<a href='https://example.com'>Link</a><img src='test.jpg'>";
            MemorySegment htmlSegment = arena.allocateUtf8String(html);

            // Would call native FFI function here
            // MemorySegment markdown = htmlToMarkdownConvertWithVisitor(
            //     htmlSegment, visitor);

            System.out.printf("Found %d links and %d images%n",
                state.linkCount, state.imageCount);
        }
    }
}
```

### C# (P/Invoke)

Using C# P/Invoke with the FFI library:

```csharp
using System;
using System.Runtime.InteropServices;

public class HtmlToMarkdownVisitor {
    // FFI type definitions
    [StructLayout(LayoutKind.Sequential)]
    public struct VisitResult {
        public int ResultType;
        public IntPtr CustomOutput;
        public IntPtr ErrorMessage;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct Attribute {
        public IntPtr Key;
        public IntPtr Value;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct NodeContext {
        public int NodeType;
        public IntPtr TagName;
        public IntPtr Attributes;
        public UIntPtr Depth;
        public UIntPtr IndexInParent;
        public IntPtr ParentTag;
        public bool IsInline;
    }

    // Callback delegate types
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate VisitResult VisitLinkCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr href,
        IntPtr text,
        IntPtr title);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate VisitResult VisitImageCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr src,
        IntPtr alt,
        IntPtr title);

    [StructLayout(LayoutKind.Sequential)]
    public struct Visitor {
        public IntPtr UserData;
        public IntPtr VisitElementStart;
        public IntPtr VisitElementEnd;
        public IntPtr VisitText;
        public VisitLinkCallback VisitLink;
        // ... other callbacks
        public VisitImageCallback VisitImage;
        // ... rest of callbacks
    }

    // Native FFI functions
    [DllImport("html_to_markdown", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr html_to_markdown_convert(string html);

    [DllImport("html_to_markdown", CallingConvention = CallingConvention.Cdecl)]
    private static extern void html_to_markdown_free_string(IntPtr str);

    public class VisitorState {
        public int LinkCount { get; set; }
        public int ImageCount { get; set; }
    }

    static VisitResult MyVisitLink(IntPtr userData, IntPtr ctx,
        IntPtr href, IntPtr text, IntPtr title) {
        // Extract state from userData (using GCHandle)
        var handle = GCHandle.FromIntPtr(userData);
        var state = (VisitorState)handle.Target;
        state.LinkCount++;

        return new VisitResult {
            ResultType = 2  // HTML_TO_MARKDOWN_VISIT_SKIP
        };
    }

    static void Main() {
        var state = new VisitorState();
        var stateHandle = GCHandle.Alloc(state);

        var visitor = new Visitor {
            UserData = GCHandle.ToIntPtr(stateHandle),
            VisitLink = MyVisitLink,
        };

        string html = "<a href='https://example.com'>Link</a><img src='test.jpg'>";

        // Would call html_to_markdown_convert_with_visitor here

        Console.WriteLine($"Found {state.LinkCount} links and {state.ImageCount} images");

        stateHandle.Free();
    }
}
```

## Common Patterns

### 1. Counting Element Types

```c
typedef struct {
    int heading_count;
    int link_count;
    int image_count;
} ElementCounter;

html_to_markdown_visit_result_t count_heading(
    void *user_data,
    const html_to_markdown_node_context_t *ctx,
    uint32_t level,
    const char *text,
    const char *id)
{
    ((ElementCounter *)user_data)->heading_count++;
    return (html_to_markdown_visit_result_t){
        .result_type = HTML_TO_MARKDOWN_VISIT_CONTINUE
    };
}
```

### 2. Skipping Specific Elements

```c
html_to_markdown_visit_result_t skip_images(
    void *user_data,
    const html_to_markdown_node_context_t *ctx,
    const char *src,
    const char *alt,
    const char *title)
{
    return (html_to_markdown_visit_result_t){
        .result_type = HTML_TO_MARKDOWN_VISIT_SKIP
    };
}
```

### 3. Custom Element Processing

```c
html_to_markdown_visit_result_t custom_link(
    void *user_data,
    const html_to_markdown_node_context_t *ctx,
    const char *href,
    const char *text,
    const char *title)
{
    html_to_markdown_visit_result_t result = {0};
    result.result_type = HTML_TO_MARKDOWN_VISIT_CUSTOM;

    // Allocate custom markdown output
    size_t len = strlen(text) + strlen(href) + 10;
    result.custom_output = malloc(len);
    snprintf(result.custom_output, len, "[%s](%s)", text, href);

    return result;
}
```

### 4. Reporting Errors

```c
html_to_markdown_visit_result_t validate_link(
    void *user_data,
    const html_to_markdown_node_context_t *ctx,
    const char *href,
    const char *text,
    const char *title)
{
    if (href == NULL || strlen(href) == 0) {
        html_to_markdown_visit_result_t result = {0};
        result.result_type = HTML_TO_MARKDOWN_VISIT_ERROR;
        result.error_message = malloc(100);
        strcpy(result.error_message, "Link missing href attribute");
        return result;
    }

    return (html_to_markdown_visit_result_t){
        .result_type = HTML_TO_MARKDOWN_VISIT_CONTINUE
    };
}
```

### 5. Attribute Inspection

```c
html_to_markdown_visit_result_t inspect_attributes(
    void *user_data,
    const html_to_markdown_node_context_t *ctx)
{
    // Iterate attributes (NULL-terminated array)
    const html_to_markdown_attribute_t *attrs = ctx->attributes;
    for (int i = 0; attrs[i].key != NULL; i++) {
        printf("  %s = %s\n", attrs[i].key, attrs[i].value);
    }

    return (html_to_markdown_visit_result_t){
        .result_type = HTML_TO_MARKDOWN_VISIT_CONTINUE
    };
}
```

## Performance Considerations

1. **Return early**: Return `CONTINUE` quickly for unmodified elements
2. **Avoid allocations**: Use stack-allocated strings when possible
3. **Cache frequently accessed data**: Store in visitor user_data
4. **Text node performance**: `visit_text` is called 100+ times per document
   - Keep text handling very fast
   - Avoid string operations in hot path

## Safety Guidelines

1. **NULL checks**: Always verify pointers before dereferencing
   - `ctx` is never NULL
   - `href`, `src`, `text` may be NULL (check before use)
   - `title` may be NULL for optional attributes

2. **Memory ownership**:
   - Never free borrowed pointers from context
   - Always free allocated output strings

3. **String encoding**: All strings are UTF-8
   - Handle multi-byte characters correctly
   - Use proper string length calculations

4. **Callback safety**:
   - Callbacks may be invoked recursively
   - Do not modify visitor struct during callback
   - Do not store callback pointers (valid only during conversion)

## Integration with Language Bindings

Each language binding implements:

1. **Type translation layer**: Maps native types to C FFI types
2. **Callback wrappers**: Converts native callbacks to C function pointers
3. **Memory management**: Handles allocation/deallocation across boundaries
4. **Error handling**: Maps C errors to language-native exceptions

Bindings are generated for:
- **Go**: crates/go/ (FFI wrapper)
- **Java**: crates/java/ (JNI + Panama FFM)
- **C#**: crates/csharp/ (P/Invoke)
- **Python**: crates/html-to-markdown-py/ (via CPython C API)
- **Ruby**: crates/html-to-markdown-rb/ (Magnus bindings)
- **TypeScript/Node**: crates/html-to-markdown-node/ (NAPI)
