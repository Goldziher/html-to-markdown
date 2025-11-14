// Package htmltomarkdown provides Go bindings for the html-to-markdown Rust library.
//
// This package uses cgo to call the C FFI interface exposed by the Rust library.
// It provides a simple, idiomatic Go API for converting HTML to Markdown.
//
// Example usage:
//
//	import "github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown"
//
//	func main() {
//	    html := "<h1>Hello World</h1><p>This is a paragraph.</p>"
//	    markdown, err := htmltomarkdown.Convert(html)
//	    if err != nil {
//	        log.Fatal(err)
//	    }
//	    fmt.Println(markdown)
//	}
package htmltomarkdown

// #cgo LDFLAGS: -lhtml_to_markdown_ffi
// #include <stdlib.h>
//
// extern char* html_to_markdown_convert(const char* html);
// extern void html_to_markdown_free_string(char* s);
// extern const char* html_to_markdown_version();
// extern const char* html_to_markdown_last_error();
import "C"
import (
	"errors"
	"unsafe"
)

// Convert converts HTML to Markdown using default options.
//
// It returns the converted Markdown string or an error if the conversion fails.
// The function handles memory management automatically using defer.
//
// Example:
//
//	markdown, err := htmltomarkdown.Convert("<h1>Title</h1>")
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Println(markdown)
func Convert(html string) (string, error) {
	if html == "" {
		return "", nil
	}

	// Convert Go string to C string
	cHTML := C.CString(html)
	defer C.free(unsafe.Pointer(cHTML))

	// Call the native conversion function
	result := C.html_to_markdown_convert(cHTML)
	if result == nil {
		// Conversion failed - try to get error message
		errMsg := C.html_to_markdown_last_error()
		if errMsg != nil {
			return "", errors.New(C.GoString(errMsg))
		}
		return "", errors.New("html to markdown conversion failed")
	}
	defer C.html_to_markdown_free_string(result)

	// Convert C string back to Go string
	markdown := C.GoString(result)
	return markdown, nil
}

// MustConvert is like Convert but panics if an error occurs.
//
// This is useful in situations where conversion errors are unexpected
// and should cause the program to terminate.
//
// Example:
//
//	markdown := htmltomarkdown.MustConvert("<h1>Title</h1>")
//	fmt.Println(markdown)
func MustConvert(html string) string {
	markdown, err := Convert(html)
	if err != nil {
		panic(err)
	}
	return markdown
}

// Version returns the version string of the underlying html-to-markdown library.
//
// Example:
//
//	version := htmltomarkdown.Version()
//	fmt.Printf("Using html-to-markdown version: %s\n", version)
func Version() string {
	cVersion := C.html_to_markdown_version()
	if cVersion == nil {
		return "unknown"
	}
	return C.GoString(cVersion)
}
