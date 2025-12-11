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
// extern char* html_to_markdown_convert_with_metadata(const char* html, char** metadata_json);
import "C"
import (
	"encoding/json"
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

// TextDirection represents the directionality of text content.
//
// This enum is used to indicate whether text flows left-to-right (as in English)
// or right-to-left (as in Arabic, Hebrew, etc.).
type TextDirection string

const (
	// TextDirectionLTR represents left-to-right text flow.
	TextDirectionLTR TextDirection = "ltr"
	// TextDirectionRTL represents right-to-left text flow.
	TextDirectionRTL TextDirection = "rtl"
	// TextDirectionAuto represents automatic directionality detection.
	TextDirectionAuto TextDirection = "auto"
)

// LinkType represents the classification of a hyperlink.
//
// Links are categorized based on their href value and document context,
// useful for filtering and analysis.
type LinkType string

const (
	// LinkTypeAnchor represents anchor links within the same document (href starts with #).
	LinkTypeAnchor LinkType = "anchor"
	// LinkTypeInternal represents internal links within the same domain.
	LinkTypeInternal LinkType = "internal"
	// LinkTypeExternal represents external links to different domains.
	LinkTypeExternal LinkType = "external"
	// LinkTypeEmail represents email links (mailto:).
	LinkTypeEmail LinkType = "email"
	// LinkTypePhone represents phone links (tel:).
	LinkTypePhone LinkType = "phone"
	// LinkTypeOther represents other protocol or unclassifiable links.
	LinkTypeOther LinkType = "other"
)

// ImageType represents the classification of an image source.
//
// Images are categorized based on their source to determine how they should
// be handled and processed.
type ImageType string

const (
	// ImageTypeDataURI represents embedded images using data URIs (base64 or other encoding).
	ImageTypeDataURI ImageType = "data_uri"
	// ImageTypeInlineSVG represents inline SVG elements.
	ImageTypeInlineSVG ImageType = "inline_svg"
	// ImageTypeExternal represents external image URLs (http/https).
	ImageTypeExternal ImageType = "external"
	// ImageTypeRelative represents relative image paths.
	ImageTypeRelative ImageType = "relative"
)

// StructuredDataType represents the format of structured data markup.
//
// This identifies which schema/format is used for machine-readable structured data.
type StructuredDataType string

const (
	// StructuredDataTypeJSONLD represents JSON-LD (JSON for Linking Data) blocks.
	StructuredDataTypeJSONLD StructuredDataType = "json_ld"
	// StructuredDataTypeMicrodata represents HTML5 Microdata attributes.
	StructuredDataTypeMicrodata StructuredDataType = "microdata"
	// StructuredDataTypeRDFa represents RDF in Attributes (RDFa) markup.
	StructuredDataTypeRDFa StructuredDataType = "rdfa"
)

// DocumentMetadata contains document-level metadata extracted from head and top-level elements.
//
// This includes metadata typically used by search engines, social media platforms,
// and browsers for document indexing and presentation.
type DocumentMetadata struct {
	// Title is the document title from the <title> tag.
	Title *string `json:"title,omitempty"`

	// Description is the document description from the <meta name="description"> tag.
	Description *string `json:"description,omitempty"`

	// Keywords are document keywords from the <meta name="keywords"> tag, split on commas.
	Keywords []string `json:"keywords,omitempty"`

	// Author is the document author from the <meta name="author"> tag.
	Author *string `json:"author,omitempty"`

	// CanonicalURL is the canonical URL from the <link rel="canonical"> tag.
	CanonicalURL *string `json:"canonical_url,omitempty"`

	// BaseHref is the base URL from the <base href=""> tag for resolving relative URLs.
	BaseHref *string `json:"base_href,omitempty"`

	// Language is the document language from the lang attribute.
	Language *string `json:"language,omitempty"`

	// TextDirection is the document text direction from the dir attribute.
	TextDirection *TextDirection `json:"text_direction,omitempty"`

	// OpenGraph contains Open Graph metadata (og:* properties) for social media.
	// Keys like "title", "description", "image", "url", etc.
	OpenGraph map[string]string `json:"open_graph,omitempty"`

	// TwitterCard contains Twitter Card metadata (twitter:* properties).
	// Keys like "card", "site", "creator", "title", "description", "image", etc.
	TwitterCard map[string]string `json:"twitter_card,omitempty"`

	// MetaTags contains additional meta tags not covered by specific fields.
	// Keys are meta name/property attributes, values are content.
	MetaTags map[string]string `json:"meta_tags,omitempty"`
}

// HeaderMetadata contains header element metadata with hierarchy tracking.
//
// Captures heading elements (h1-h6) with their text content, identifiers,
// and position in the document structure.
type HeaderMetadata struct {
	// Level is the header level (1 for h1, 6 for h6).
	Level uint8 `json:"level"`

	// Text is the normalized text content of the header.
	Text string `json:"text"`

	// ID is the HTML id attribute if present.
	ID *string `json:"id,omitempty"`

	// Depth is the document tree depth at the header element.
	Depth uint32 `json:"depth"`

	// HTMLOffset is the byte offset in the original HTML document.
	HTMLOffset uint32 `json:"html_offset"`
}

// LinkMetadata contains hyperlink metadata with categorization and attributes.
//
// Represents <a> elements with parsed href values, text content, and link type classification.
type LinkMetadata struct {
	// Href is the URL value from the href attribute.
	Href string `json:"href"`

	// Text is the link text content (normalized, concatenated if mixed with elements).
	Text string `json:"text"`

	// Title is the optional title attribute (often shown as tooltip).
	Title *string `json:"title,omitempty"`

	// LinkType is the link type classification.
	LinkType LinkType `json:"link_type"`

	// Rel contains rel attribute values (e.g., "nofollow", "stylesheet", "canonical").
	Rel []string `json:"rel,omitempty"`

	// Attributes contains additional HTML attributes.
	Attributes map[string]string `json:"attributes,omitempty"`
}

// ImageMetadata contains image metadata with source and dimensions.
//
// Captures <img> elements and inline <svg> elements with metadata
// for image analysis and optimization.
type ImageMetadata struct {
	// Src is the image source (URL, data URI, or SVG content identifier).
	Src string `json:"src"`

	// Alt is the alternative text from the alt attribute (for accessibility).
	Alt *string `json:"alt,omitempty"`

	// Title is the title attribute (often shown as tooltip).
	Title *string `json:"title,omitempty"`

	// Dimensions are the image dimensions as [width, height] if available.
	Dimensions *[2]uint32 `json:"dimensions,omitempty"`

	// ImageType is the image type classification.
	ImageType ImageType `json:"image_type"`

	// Attributes contains additional HTML attributes.
	Attributes map[string]string `json:"attributes,omitempty"`
}

// StructuredData represents a structured data block (JSON-LD, Microdata, or RDFa).
//
// Represents machine-readable structured data found in the document.
// JSON-LD blocks are collected as raw JSON strings for flexibility.
type StructuredData struct {
	// DataType is the type of structured data (JSON-LD, Microdata, RDFa).
	DataType StructuredDataType `json:"data_type"`

	// RawJSON is the raw JSON string (for JSON-LD) or serialized representation.
	RawJSON string `json:"raw_json"`

	// SchemaType is the schema type if detectable (e.g., "Article", "Event", "Product").
	SchemaType *string `json:"schema_type,omitempty"`
}

// ExtendedMetadata is the comprehensive metadata extraction result from an HTML document.
//
// Contains all extracted metadata types in a single structure,
// suitable for serialization and transmission across language boundaries.
type ExtendedMetadata struct {
	// Document contains document-level metadata (title, description, canonical, etc.).
	Document DocumentMetadata `json:"document"`

	// Headers contains extracted header elements with hierarchy.
	Headers []HeaderMetadata `json:"headers,omitempty"`

	// Links contains extracted hyperlinks with type classification.
	Links []LinkMetadata `json:"links,omitempty"`

	// Images contains extracted images with source and dimensions.
	Images []ImageMetadata `json:"images,omitempty"`

	// StructuredData contains extracted structured data blocks.
	StructuredData []StructuredData `json:"structured_data,omitempty"`
}

// MetadataExtraction contains the conversion result with metadata.
//
// This structure combines the converted markdown with its associated metadata.
type MetadataExtraction struct {
	// Markdown is the converted markdown string.
	Markdown string

	// Metadata contains the extracted metadata.
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

	// Convert Go string to C string
	cHTML := C.CString(html)
	defer C.free(unsafe.Pointer(cHTML))

	// Allocate output pointer for metadata JSON
	var metadataPtr *C.char

	// Call the native conversion with metadata function
	result := C.html_to_markdown_convert_with_metadata(cHTML, &metadataPtr)  // nolint:gocritic
	if result == nil {
		// Conversion failed - try to get error message
		errMsg := C.html_to_markdown_last_error()
		if errMsg != nil {
			return MetadataExtraction{}, errors.New(C.GoString(errMsg))
		}
		return MetadataExtraction{}, errors.New("html to markdown conversion with metadata failed")
	}

	// Ensure markdown string is freed
	defer C.html_to_markdown_free_string(result)

	// Ensure metadata JSON string is freed
	if metadataPtr != nil {
		defer C.html_to_markdown_free_string(metadataPtr)
	}

	// Convert C strings back to Go strings
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
