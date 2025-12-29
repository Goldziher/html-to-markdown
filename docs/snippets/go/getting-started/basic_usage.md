# Basic HTML to Markdown Conversion

Convert simple HTML to Markdown with the `Convert` function.

## Example

```go
package main

import (
    "fmt"
    "log"

    "github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown"
)

func main() {
    html := "<h1>Hello World</h1><p>This is a paragraph.</p>"

    markdown, err := htmltomarkdown.Convert(html)
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println(markdown)
}
```

## Output

```
# Hello World

This is a paragraph.
```

## Notes

- The `Convert` function takes a string of HTML and returns the converted Markdown as a string, or an error.
- Use `Convert` for error handling with the standard Go error interface.
- For panicking on error (when you're certain conversion won't fail), use `MustConvert(html)` instead.
