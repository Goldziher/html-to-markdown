# Conversion with Library Information

Check the library version and handle errors properly.

## Example

```go
package main

import (
    "fmt"
    "log"

    "github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown"
)

func main() {
    // Check library version
    version := htmltomarkdown.Version()
    fmt.Printf("html-to-markdown version: %s\n", version)

    html := "<h1>Hello</h1><p>Welcome</p>"

    // Convert with error handling
    markdown, err := htmltomarkdown.Convert(html)
    if err != nil {
        log.Fatalf("Conversion failed: %v", err)
    }

    fmt.Println(markdown)

    // Alternative: Use MustConvert for panicking on error
    // Useful when you're certain conversion won't fail
    anotherMarkdown := htmltomarkdown.MustConvert("<p>Safe HTML</p>")
    fmt.Println(anotherMarkdown)
}
```

## Notes

- **Version()**: Returns the underlying Rust library version string.
- **Convert()**: Standard error handling with Go's error interface. Recommended for production code.
- **MustConvert()**: Panics on error. Use only when you're certain the HTML is valid and conversion won't fail.
- The Go binding automatically downloads and caches the FFI library on first use. See environment variables in the README for customization (HTML_TO_MARKDOWN_FFI_PATH, HTML_TO_MARKDOWN_FFI_CACHE_DIR, etc.).

## Advanced Customization

For more advanced use cases, see the Go binding documentation for:
- **ConvertWithMetadata()** - Extract document metadata (titles, links, headers, etc.)
- **ConvertWithVisitor()** - Customize conversion with visitor callbacks
