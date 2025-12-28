// Package htmltomarkdown provides Go bindings for the html-to-markdown Rust library.
//
// This package uses cgo to call the C FFI interface exposed by the Rust library.
// It provides a simple, idiomatic Go API for converting HTML to Markdown.
//
// Example usage:
//
//	import "github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown"
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

// #include <stdlib.h>
// #include <stdbool.h>
// #include <stdint.h>
//
// char* html_to_markdown_convert_proxy(const char* html);
// void html_to_markdown_free_string_proxy(char* s);
// const char* html_to_markdown_version_proxy(void);
// const char* html_to_markdown_last_error_proxy(void);
// char* html_to_markdown_convert_with_metadata_proxy(const char* html, char** metadata_json);
// bool html_to_markdown_profile_start_proxy(const char* output, int32_t frequency);
// bool html_to_markdown_profile_stop_proxy(void);
import "C"
import (
	"encoding/json"
	"errors"
	"unsafe"
)

const unknownValue = "unknown"

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
	if err := ensureFFILoaded(); err != nil {
		return "", err
	}

	cHTML := C.CString(html)
	defer C.free(unsafe.Pointer(cHTML))

	result := C.html_to_markdown_convert_proxy(cHTML)
	if result == nil {
		errMsg := C.html_to_markdown_last_error_proxy()
		if errMsg != nil {
			return "", errors.New(C.GoString(errMsg))
		}
		return "", errors.New("html to markdown conversion failed")
	}
	defer C.html_to_markdown_free_string_proxy(result)

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
	if err := ensureFFILoaded(); err != nil {
		return unknownValue
	}
	cVersion := C.html_to_markdown_version_proxy()
	if cVersion == nil {
		return unknownValue
	}
	return C.GoString(cVersion)
}

// StartProfiling begins Rust-side profiling and writes a flamegraph to outputPath.
func StartProfiling(outputPath string, frequency int) error {
	if outputPath == "" {
		return errors.New("output path is required")
	}
	if err := ensureFFILoaded(); err != nil {
		return err
	}
	if frequency <= 0 {
		frequency = 1000
	}
	cOutput := C.CString(outputPath)
	defer C.free(unsafe.Pointer(cOutput))

	ok := C.html_to_markdown_profile_start_proxy(cOutput, C.int32_t(frequency))
	if !bool(ok) {
		errMsg := C.html_to_markdown_last_error_proxy()
		if errMsg != nil {
			return errors.New(C.GoString(errMsg))
		}
		return errors.New("profiling start failed")
	}
	return nil
}

// StopProfiling stops Rust-side profiling and flushes the flamegraph.
func StopProfiling() error {
	if err := ensureFFILoaded(); err != nil {
		return err
	}
	ok := C.html_to_markdown_profile_stop_proxy()
	if !bool(ok) {
		errMsg := C.html_to_markdown_last_error_proxy()
		if errMsg != nil {
			return errors.New(C.GoString(errMsg))
		}
		return errors.New("profiling stop failed")
	}
	return nil
}

// TextDirection represents the directionality of text content.
//
// This enum is used to indicate whether text flows left-to-right (as in English)
// or right-to-left (as in Arabic, Hebrew, etc.).
type TextDirection string

const (
	TextDirectionLTR  TextDirection = "ltr"
	TextDirectionRTL  TextDirection = "rtl"
	TextDirectionAuto TextDirection = "auto"
)

// LinkType represents the classification of a hyperlink.
//
// Links are categorized based on their href value and document context,
// useful for filtering and analysis.
type LinkType string

const (
	LinkTypeAnchor   LinkType = "anchor"
	LinkTypeInternal LinkType = "internal"
	LinkTypeExternal LinkType = "external"
	LinkTypeEmail    LinkType = "email"
	LinkTypePhone    LinkType = "phone"
	LinkTypeOther    LinkType = "other"
)

// ImageType represents the classification of an image source.
//
// Images are categorized based on their source to determine how they should
// be handled and processed.
type ImageType string

const (
	ImageTypeDataURI   ImageType = "data_uri"
	ImageTypeInlineSVG ImageType = "inline_svg"
	ImageTypeExternal  ImageType = "external"
	ImageTypeRelative  ImageType = "relative"
)

// StructuredDataType represents the format of structured data markup.
//
// This identifies which schema/format is used for machine-readable structured data.
type StructuredDataType string

const (
	StructuredDataTypeJSONLD    StructuredDataType = "json_ld"
	StructuredDataTypeMicrodata StructuredDataType = "microdata"
	StructuredDataTypeRDFa      StructuredDataType = "rdfa"
)

// DocumentMetadata contains document-level metadata extracted from head and top-level elements.
//
// This includes metadata typically used by search engines, social media platforms,
// and browsers for document indexing and presentation.
type DocumentMetadata struct {
	Title *string `json:"title,omitempty"`

	Description *string `json:"description,omitempty"`

	Keywords []string `json:"keywords,omitempty"`

	Author *string `json:"author,omitempty"`

	CanonicalURL *string `json:"canonical_url,omitempty"`

	BaseHref *string `json:"base_href,omitempty"`

	Language *string `json:"language,omitempty"`

	TextDirection *TextDirection `json:"text_direction,omitempty"`

	OpenGraph map[string]string `json:"open_graph,omitempty"`

	TwitterCard map[string]string `json:"twitter_card,omitempty"`

	MetaTags map[string]string `json:"meta_tags,omitempty"`
}

// HeaderMetadata contains header element metadata with hierarchy tracking.
//
// Captures heading elements (h1-h6) with their text content, identifiers,
// and position in the document structure.
type HeaderMetadata struct {
	Level uint8 `json:"level"`

	Text string `json:"text"`

	ID *string `json:"id,omitempty"`

	Depth uint32 `json:"depth"`

	HTMLOffset uint32 `json:"html_offset"`
}

// LinkMetadata contains hyperlink metadata with categorization and attributes.
//
// Represents <a> elements with parsed href values, text content, and link type classification.
type LinkMetadata struct {
	Href string `json:"href"`

	Text string `json:"text"`

	Title *string `json:"title,omitempty"`

	LinkType LinkType `json:"link_type"`

	Rel []string `json:"rel,omitempty"`

	Attributes map[string]string `json:"attributes,omitempty"`
}

// ImageMetadata contains image metadata with source and dimensions.
//
// Captures <img> elements and inline <svg> elements with metadata
// for image analysis and optimization.
type ImageMetadata struct {
	Src string `json:"src"`

	Alt *string `json:"alt,omitempty"`

	Title *string `json:"title,omitempty"`

	Dimensions *[2]uint32 `json:"dimensions,omitempty"`

	ImageType ImageType `json:"image_type"`

	Attributes map[string]string `json:"attributes,omitempty"`
}

// StructuredData represents a structured data block (JSON-LD, Microdata, or RDFa).
//
// Represents machine-readable structured data found in the document.
// JSON-LD blocks are collected as raw JSON strings for flexibility.
type StructuredData struct {
	DataType StructuredDataType `json:"data_type"`

	RawJSON string `json:"raw_json"`

	SchemaType *string `json:"schema_type,omitempty"`
}

// ExtendedMetadata is the comprehensive metadata extraction result from an HTML document.
//
// Contains all extracted metadata types in a single structure,
// suitable for serialization and transmission across language boundaries.
type ExtendedMetadata struct {
	Document DocumentMetadata `json:"document"`

	Headers []HeaderMetadata `json:"headers,omitempty"`

	Links []LinkMetadata `json:"links,omitempty"`

	Images []ImageMetadata `json:"images,omitempty"`

	StructuredData []StructuredData `json:"structured_data,omitempty"`
}

// MetadataExtraction contains the conversion result with metadata.
//
// This structure combines the converted markdown with its associated metadata.
type MetadataExtraction struct {
	Markdown string

	Metadata ExtendedMetadata
}

// ConvertWithMetadata converts HTML to Markdown and extracts comprehensive metadata.
//
// This function calls the underlying FFI layer to perform conversion and metadata
// extraction in a single pass. It returns the markdown along with structured metadata
// including document information, headers, links, images, and structured data.
//
// The metadata extraction includes:
// - Document metadata: title, description, keywords, author, canonical URL, language, etc.
// - Headers: h1-h6 elements with hierarchy tracking
// - Links: anchor elements with type classification and attributes
// - Images: img and svg elements with source and dimensions
// - Structured data: JSON-LD, Microdata, and RDFa blocks
//
// Example:
//
//	html := `<html>
//	  <head>
//	    <title>My Article</title>
//	    <meta name="description" content="A great article">
//	  </head>
//	  <body>
//	    <h1>Main Title</h1>
//	    <p>Content with <a href="https://example.com">a link</a></p>
//	    <img src="image.jpg" alt="An image">
//	  </body>
//	</html>`
//	result, err := ConvertWithMetadata(html)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Println(result.Markdown)
//	fmt.Printf("Title: %s\n", *result.Metadata.Document.Title)
//	fmt.Printf("Headers: %d\n", len(result.Metadata.Headers))
//	fmt.Printf("Links: %d\n", len(result.Metadata.Links))
//	fmt.Printf("Images: %d\n", len(result.Metadata.Images))
func ConvertWithMetadata(html string) (MetadataExtraction, error) {
	if html == "" {
		return MetadataExtraction{
			Markdown: "",
			Metadata: ExtendedMetadata{},
		}, nil
	}
	if err := ensureFFILoaded(); err != nil {
		return MetadataExtraction{}, err
	}

	cHTML := C.CString(html)
	defer C.free(unsafe.Pointer(cHTML))

	// Allocate output pointer for metadata JSON
	var metadataPtr *C.char

	result := C.html_to_markdown_convert_with_metadata_proxy(cHTML, &metadataPtr) // nolint:gocritic
	if result == nil {
		errMsg := C.html_to_markdown_last_error_proxy()
		if errMsg != nil {
			return MetadataExtraction{}, errors.New(C.GoString(errMsg))
		}
		return MetadataExtraction{}, errors.New("html to markdown conversion with metadata failed")
	}

	defer C.html_to_markdown_free_string_proxy(result)

	if metadataPtr != nil {
		defer C.html_to_markdown_free_string_proxy(metadataPtr)
	}

	markdown := C.GoString(result)

	// Parse metadata JSON if available
	var metadata ExtendedMetadata
	if metadataPtr != nil {
		metadataJSON := C.GoString(metadataPtr)
		if err := json.Unmarshal([]byte(metadataJSON), &metadata); err != nil {
			return MetadataExtraction{}, errors.New("failed to parse metadata JSON: " + err.Error())
		}
	}

	return MetadataExtraction{
		Markdown: markdown,
		Metadata: metadata,
	}, nil
}

// MustConvertWithMetadata is like ConvertWithMetadata but panics if an error occurs.
//
// This is useful in situations where metadata extraction errors are unexpected
// and should cause the program to terminate.
//
// Example:
//
//	result := htmltomarkdown.MustConvertWithMetadata("<h1>Title</h1>")
//	fmt.Println(result.Markdown)
func MustConvertWithMetadata(html string) MetadataExtraction {
	result, err := ConvertWithMetadata(html)
	if err != nil {
		panic(err)
	}
	return result
}
