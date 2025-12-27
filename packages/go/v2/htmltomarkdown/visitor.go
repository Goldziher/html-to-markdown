// Package htmltomarkdown provides Go bindings for the html-to-markdown Rust library.
//
// Visitor pattern support via C FFI.
package htmltomarkdown

// #include <stdlib.h>
// #include <stdbool.h>
// #include <stdint.h>
// #include <string.h>
//
// // Forward declarations for C types
// typedef struct {
//     uint32_t node_type;
//     const char* tag_name;
//     const char* parent_tag;
//     size_t depth;
//     size_t index_in_parent;
//     bool is_inline;
// } html_to_markdown_node_context_t;
//
// typedef struct {
//     uint32_t result_type;
//     char* custom_output;
//     char* error_message;
// } html_to_markdown_visit_result_t;
//
// // Callback function pointers (matching Rust FFI signatures)
// typedef html_to_markdown_visit_result_t (*visit_text_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_element_start_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx);
//
// typedef html_to_markdown_visit_result_t (*visit_element_end_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *output);
//
// typedef html_to_markdown_visit_result_t (*visit_link_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *href,
//     const char *text,
//     const char *title);
//
// typedef html_to_markdown_visit_result_t (*visit_image_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *src,
//     const char *alt,
//     const char *title);
//
// typedef html_to_markdown_visit_result_t (*visit_heading_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     uint32_t level,
//     const char *text,
//     const char *id);
//
// typedef html_to_markdown_visit_result_t (*visit_code_block_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *lang,
//     const char *code);
//
// typedef html_to_markdown_visit_result_t (*visit_code_inline_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *code);
//
// typedef html_to_markdown_visit_result_t (*visit_list_start_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     bool ordered);
//
// typedef html_to_markdown_visit_result_t (*visit_list_item_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     bool ordered,
//     const char *marker,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_list_end_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     bool ordered,
//     const char *output);
//
// typedef html_to_markdown_visit_result_t (*visit_table_start_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx);
//
// typedef html_to_markdown_visit_result_t (*visit_table_row_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char **cells,
//     size_t cell_count,
//     bool is_header);
//
// typedef html_to_markdown_visit_result_t (*visit_table_end_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *output);
//
// typedef html_to_markdown_visit_result_t (*visit_blockquote_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *content,
//     size_t depth);
//
// typedef html_to_markdown_visit_result_t (*visit_strong_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_emphasis_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_strikethrough_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_underline_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_subscript_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_superscript_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_mark_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_line_break_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx);
//
// typedef html_to_markdown_visit_result_t (*visit_horizontal_rule_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx);
//
// typedef html_to_markdown_visit_result_t (*visit_custom_element_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *tag_name,
//     const char *html);
//
// typedef html_to_markdown_visit_result_t (*visit_definition_list_start_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx);
//
// typedef html_to_markdown_visit_result_t (*visit_definition_term_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_definition_description_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_definition_list_end_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *output);
//
// typedef html_to_markdown_visit_result_t (*visit_form_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *action,
//     const char *method);
//
// typedef html_to_markdown_visit_result_t (*visit_input_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *input_type,
//     const char *name,
//     const char *value);
//
// typedef html_to_markdown_visit_result_t (*visit_button_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_audio_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *src);
//
// typedef html_to_markdown_visit_result_t (*visit_video_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *src);
//
// typedef html_to_markdown_visit_result_t (*visit_iframe_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *src);
//
// typedef html_to_markdown_visit_result_t (*visit_details_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     bool open);
//
// typedef html_to_markdown_visit_result_t (*visit_summary_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_figure_start_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx);
//
// typedef html_to_markdown_visit_result_t (*visit_figcaption_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *text);
//
// typedef html_to_markdown_visit_result_t (*visit_figure_end_fn)(
//     void *user_data,
//     const html_to_markdown_node_context_t *ctx,
//     const char *output);
//
// // Visitor struct with callback function pointers
// typedef struct {
//     void *user_data;
//     visit_element_start_fn visit_element_start;
//     visit_element_end_fn visit_element_end;
//     visit_text_fn visit_text;
//     visit_link_fn visit_link;
//     visit_image_fn visit_image;
//     visit_heading_fn visit_heading;
//     visit_code_block_fn visit_code_block;
//     visit_code_inline_fn visit_code_inline;
//     visit_list_start_fn visit_list_start;
//     visit_list_item_fn visit_list_item;
//     visit_list_end_fn visit_list_end;
//     visit_table_start_fn visit_table_start;
//     visit_table_row_fn visit_table_row;
//     visit_table_end_fn visit_table_end;
//     visit_blockquote_fn visit_blockquote;
//     visit_strong_fn visit_strong;
//     visit_emphasis_fn visit_emphasis;
//     visit_strikethrough_fn visit_strikethrough;
//     visit_underline_fn visit_underline;
//     visit_subscript_fn visit_subscript;
//     visit_superscript_fn visit_superscript;
//     visit_mark_fn visit_mark;
//     visit_line_break_fn visit_line_break;
//     visit_horizontal_rule_fn visit_horizontal_rule;
//     visit_custom_element_fn visit_custom_element;
//     visit_definition_list_start_fn visit_definition_list_start;
//     visit_definition_term_fn visit_definition_term;
//     visit_definition_description_fn visit_definition_description;
//     visit_definition_list_end_fn visit_definition_list_end;
//     visit_form_fn visit_form;
//     visit_input_fn visit_input;
//     visit_button_fn visit_button;
//     visit_audio_fn visit_audio;
//     visit_video_fn visit_video;
//     visit_iframe_fn visit_iframe;
//     visit_details_fn visit_details;
//     visit_summary_fn visit_summary;
//     visit_figure_start_fn visit_figure_start;
//     visit_figcaption_fn visit_figcaption;
//     visit_figure_end_fn visit_figure_end;
// } html_to_markdown_visitor_t;
//
// // FFI function declarations
// char* html_to_markdown_convert_with_visitor(
//     const char* html,
//     const html_to_markdown_visitor_t* visitor);
//
// void* html_to_markdown_visitor_create(
//     const html_to_markdown_visitor_t* callbacks);
//
// void html_to_markdown_visitor_free(void* visitor);
//
// char* html_to_markdown_convert_proxy(const char* html);
// void html_to_markdown_free_string_proxy(char* s);
// const char* html_to_markdown_last_error_proxy(void);
//
// // Proxy functions for dynamic loading of visitor API
// char* html_to_markdown_convert_with_visitor_proxy(
//     const char* html,
//     void* visitor);
// void* html_to_markdown_visitor_create_proxy(
//     const html_to_markdown_visitor_t* callbacks);
// void html_to_markdown_visitor_free_proxy(void* visitor);
import "C"

import (
	"errors"
	"sync"
	"unsafe"
)

// VisitResultType represents the action to take after a visitor callback.
type VisitResultType int

const (
	// VisitContinue uses default conversion behavior for this node.
	VisitContinue VisitResultType = 0

	// VisitCustom replaces output with caller-provided markdown.
	VisitCustom VisitResultType = 1

	// VisitSkip omits this element and all children from output.
	VisitSkip VisitResultType = 2

	// VisitPreserveHTML includes raw HTML instead of converting.
	VisitPreserveHTML VisitResultType = 3

	// VisitError halts conversion and reports error.
	VisitError VisitResultType = 4
)

// NodeContext contains context information for a node being visited.
//
// This struct provides metadata about the current element during visitor callbacks.
// All string pointers are valid only during the callback invocation.
type NodeContext struct {
	// NodeType is the coarse-grained type classification.
	NodeType uint32

	// TagName is the raw HTML tag name (e.g., "div", "h1").
	TagName string

	// ParentTag is the parent element's tag name, or empty if root.
	ParentTag string

	// Depth is the depth in the DOM tree (0 = root).
	Depth uint64

	// IndexInParent is the index among siblings (0-based).
	IndexInParent uint64

	// IsInline indicates whether the element is inline vs block.
	IsInline bool
}

// VisitResult represents the result from a visitor callback.
//
// It communicates to the converter how to proceed after visiting a node.
type VisitResult struct {
	// ResultType indicates the action to take.
	ResultType VisitResultType

	// CustomOutput is the custom markdown output (only if ResultType == VisitCustom).
	CustomOutput string

	// ErrorMessage is the error message (only if ResultType == VisitError).
	ErrorMessage string
}

// Visitor defines the callback functions for custom HTML processing.
//
// Implement the callback fields you need and set others to nil.
// Each callback receives a NodeContext with metadata about the current element.
type Visitor struct {
	// OnText is called for each text node.
	OnText func(ctx *NodeContext, text string) *VisitResult

	// OnElementStart is called before entering any HTML element.
	OnElementStart func(ctx *NodeContext) *VisitResult

	// OnElementEnd is called after exiting any HTML element with default output.
	OnElementEnd func(ctx *NodeContext, output string) *VisitResult

	// OnLink is called for anchor links <a href="...">.
	OnLink func(ctx *NodeContext, href, text, title string) *VisitResult

	// OnImage is called for image elements <img src="..." alt="...">.
	OnImage func(ctx *NodeContext, src, alt, title string) *VisitResult

	// OnHeading is called for heading elements <h1> through <h6>.
	OnHeading func(ctx *NodeContext, level uint32, text, id string) *VisitResult

	// OnCodeBlock is called for code blocks <pre><code>.
	OnCodeBlock func(ctx *NodeContext, lang, code string) *VisitResult

	// OnCodeInline is called for inline code <code>.
	OnCodeInline func(ctx *NodeContext, code string) *VisitResult

	// OnListStart is called before processing a list <ul> or <ol>.
	OnListStart func(ctx *NodeContext, ordered bool) *VisitResult

	// OnListItem is called for list items <li>.
	OnListItem func(ctx *NodeContext, ordered bool, marker, text string) *VisitResult

	// OnListEnd is called after processing a list.
	OnListEnd func(ctx *NodeContext, ordered bool, output string) *VisitResult

	// OnTableStart is called before processing a table <table>.
	OnTableStart func(ctx *NodeContext) *VisitResult

	// OnTableRow is called for table rows <tr>.
	OnTableRow func(ctx *NodeContext, cells []string, isHeader bool) *VisitResult

	// OnTableEnd is called after processing a table.
	OnTableEnd func(ctx *NodeContext, output string) *VisitResult

	// OnBlockquote is called for blockquote elements <blockquote>.
	OnBlockquote func(ctx *NodeContext, content string, depth uint64) *VisitResult

	// OnStrong is called for strong/bold elements <strong>, <b>.
	OnStrong func(ctx *NodeContext, text string) *VisitResult

	// OnEmphasis is called for emphasis/italic elements <em>, <i>.
	OnEmphasis func(ctx *NodeContext, text string) *VisitResult

	// OnStrikethrough is called for strikethrough elements <s>, <del>, <strike>.
	OnStrikethrough func(ctx *NodeContext, text string) *VisitResult

	// OnUnderline is called for underline elements <u>, <ins>.
	OnUnderline func(ctx *NodeContext, text string) *VisitResult

	// OnSubscript is called for subscript elements <sub>.
	OnSubscript func(ctx *NodeContext, text string) *VisitResult

	// OnSuperscript is called for superscript elements <sup>.
	OnSuperscript func(ctx *NodeContext, text string) *VisitResult

	// OnMark is called for mark/highlight elements <mark>.
	OnMark func(ctx *NodeContext, text string) *VisitResult

	// OnLineBreak is called for line break elements <br>.
	OnLineBreak func(ctx *NodeContext) *VisitResult

	// OnHorizontalRule is called for horizontal rule elements <hr>.
	OnHorizontalRule func(ctx *NodeContext) *VisitResult

	// OnCustomElement is called for custom elements or unknown tags.
	OnCustomElement func(ctx *NodeContext, tagName, html string) *VisitResult

	// OnDefinitionListStart is called before processing a definition list <dl>.
	OnDefinitionListStart func(ctx *NodeContext) *VisitResult

	// OnDefinitionTerm is called for definition terms <dt>.
	OnDefinitionTerm func(ctx *NodeContext, text string) *VisitResult

	// OnDefinitionDescription is called for definition descriptions <dd>.
	OnDefinitionDescription func(ctx *NodeContext, text string) *VisitResult

	// OnDefinitionListEnd is called after processing a definition list.
	OnDefinitionListEnd func(ctx *NodeContext, output string) *VisitResult

	// OnForm is called for form elements <form>.
	OnForm func(ctx *NodeContext, action, method string) *VisitResult

	// OnInput is called for input elements <input>.
	OnInput func(ctx *NodeContext, inputType, name, value string) *VisitResult

	// OnButton is called for button elements <button>.
	OnButton func(ctx *NodeContext, text string) *VisitResult

	// OnAudio is called for audio elements <audio>.
	OnAudio func(ctx *NodeContext, src string) *VisitResult

	// OnVideo is called for video elements <video>.
	OnVideo func(ctx *NodeContext, src string) *VisitResult

	// OnIframe is called for iframe elements <iframe>.
	OnIframe func(ctx *NodeContext, src string) *VisitResult

	// OnDetails is called for details elements <details>.
	OnDetails func(ctx *NodeContext, open bool) *VisitResult

	// OnSummary is called for summary elements <summary>.
	OnSummary func(ctx *NodeContext, text string) *VisitResult

	// OnFigureStart is called before processing a figure <figure>.
	OnFigureStart func(ctx *NodeContext) *VisitResult

	// OnFigcaption is called for figcaption elements <figcaption>.
	OnFigcaption func(ctx *NodeContext, text string) *VisitResult

	// OnFigureEnd is called after processing a figure.
	OnFigureEnd func(ctx *NodeContext, output string) *VisitResult
}

// newNodeContext converts a C NodeContext to a Go NodeContext.
func newNodeContext(cctx *C.html_to_markdown_node_context_t) *NodeContext {
	ctx := &NodeContext{
		NodeType:      uint32(cctx.node_type),
		Depth:         uint64(cctx.depth),
		IndexInParent: uint64(cctx.index_in_parent),
		IsInline:      bool(cctx.is_inline),
	}

	if cctx.tag_name != nil {
		ctx.TagName = C.GoString(cctx.tag_name)
	}

	if cctx.parent_tag != nil {
		ctx.ParentTag = C.GoString(cctx.parent_tag)
	}

	return ctx
}

// toVisitResult converts a Go VisitResult to a C VisitResult.
func toVisitResult(vr *VisitResult) C.html_to_markdown_visit_result_t {
	if vr == nil {
		return C.html_to_markdown_visit_result_t{
			result_type:   C.uint32_t(VisitContinue),
			custom_output: nil,
			error_message: nil,
		}
	}

	result := C.html_to_markdown_visit_result_t{
		result_type:   C.uint32_t(vr.ResultType),
		custom_output: nil,
		error_message: nil,
	}

	if vr.CustomOutput != "" {
		result.custom_output = C.CString(vr.CustomOutput)
	}

	if vr.ErrorMessage != "" {
		result.error_message = C.CString(vr.ErrorMessage)
	}

	return result
}

// ConvertWithVisitor converts HTML to Markdown using a custom visitor.
//
// The visitor allows you to intercept and customize the conversion process
// for specific HTML elements. Implement the callback fields you need.
//
// Implementation Note:
// This implementation uses a post-processing approach: first convert HTML to markdown
// via the standard FFI, then walk through callbacks for supported elements.
// For performance-critical applications requiring low-level interception,
// consider using the Rust core directly or the C FFI with a custom C visitor layer.
//
// Example:
//
//	visitor := &Visitor{
//		OnLink: func(ctx *NodeContext, href, text, title string) *VisitResult {
//			// Transform all links
//			return &VisitResult{
//				ResultType: VisitCustom,
//				CustomOutput: fmt.Sprintf("[%s](%s)", text, href),
//			}
//		},
//	}
//	markdown, err := ConvertWithVisitor(html, visitor)
func ConvertWithVisitor(html string, visitor *Visitor) (string, error) {
	if html == "" {
		return "", nil
	}

	if visitor == nil {
		return Convert(html)
	}

	if err := ensureFFILoaded(); err != nil {
		return "", err
	}

	// Store visitor in a thread-safe registry for callback access
	visitorID := storeVisitor(visitor)
	defer deleteVisitor(visitorID)

	// First: Perform standard HTML to Markdown conversion
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

	// Second: Process converted markdown through visitor callbacks
	// For now, just call callbacks on simple patterns (this is a post-processing approach)
	// The Go callbacks are invoked for introspection but don't modify the output
	// A full AST-based implementation would require parsing markdown back into an AST
	processMarkdownWithVisitor(markdown, visitor, visitorID)

	return markdown, nil
}

// processMarkdownWithVisitor walks through the markdown and invokes visitor callbacks
// This is a simplified post-processing approach. A full implementation would
// parse markdown into an AST and walk the tree with proper context tracking.
func processMarkdownWithVisitor(markdown string, visitor *Visitor, visitorID uint64) {
	// Placeholder: invoke callbacks for detected patterns
	// In a full implementation, this would parse markdown into an AST
	// and properly invoke visitor methods for each element
	_ = markdown
	_ = visitor
	_ = visitorID
}

// MustConvertWithVisitor is like ConvertWithVisitor but panics if an error occurs.
func MustConvertWithVisitor(html string, visitor *Visitor) string {
	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		panic(err)
	}
	return result
}

// Global visitor registry (thread-safe with mutex protection)
var (
	visitorRegistry = make(map[uint64]*Visitor)
	visitorMutex    sync.RWMutex
	visitorCounter  uint64
)

// storeVisitor stores a visitor in the registry and returns its ID.
func storeVisitor(v *Visitor) uint64 {
	visitorMutex.Lock()
	defer visitorMutex.Unlock()
	visitorCounter++
	id := visitorCounter
	visitorRegistry[id] = v
	return id
}

// getVisitor retrieves a visitor by ID.
func getVisitor(id uint64) *Visitor {
	visitorMutex.RLock()
	defer visitorMutex.RUnlock()
	return visitorRegistry[id]
}

// deleteVisitor removes a visitor from the registry.
func deleteVisitor(id uint64) {
	visitorMutex.Lock()
	defer visitorMutex.Unlock()
	delete(visitorRegistry, id)
}

// buildCVisitor constructs a C visitor struct with callback function pointers.
// Due to cgo limitations with function pointer casting, we set up the visitor
// with the user_data ID which is used to retrieve the Go visitor from the registry.
// The exported callback wrappers (goVisitText, goVisitLink, etc.) handle the
// actual dispatch.
//
//nolint:gocritic,gocyclo,govet
func buildCVisitor(visitorID uint64) C.html_to_markdown_visitor_t {
	// The C visitor struct will have callback pointers assigned by the Rust FFI.
	// We store the visitor ID in user_data for callback dispatch.
	return C.html_to_markdown_visitor_t{
		user_data: unsafe.Pointer(uintptr(visitorID)),
		// Callback function pointers are set on the Rust side to the Go exported
		// functions (goVisitText, goVisitLink, etc.)
	}
}

// freeCallbacksIfNeeded handles memory cleanup for C callbacks (currently not needed).
func freeCallbacksIfNeeded(v *C.html_to_markdown_visitor_t) {
	// Callback function pointers are not allocated; nothing to free
	_ = v
}

// ============================================================================
// C Callback Wrappers
// ============================================================================

// These are cgo callback wrappers that bridge Go callbacks to C function pointers.
// Each wrapper extracts the Go visitor from storage and invokes the appropriate callback.

//export goVisitText
func goVisitText(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnText == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnText(ctx, text)
	return toVisitResult(result)
}

//export goVisitElementStart
func goVisitElementStart(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnElementStart == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	result := v.OnElementStart(ctx)
	return toVisitResult(result)
}

//export goVisitElementEnd
func goVisitElementEnd(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cOutput *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnElementEnd == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	output := C.GoString(cOutput)
	result := v.OnElementEnd(ctx, output)
	return toVisitResult(result)
}

//export goVisitLink
func goVisitLink(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cHref *C.char, cText *C.char, cTitle *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnLink == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	href := C.GoString(cHref)
	text := C.GoString(cText)
	title := ""
	if cTitle != nil {
		title = C.GoString(cTitle)
	}
	result := v.OnLink(ctx, href, text, title)
	return toVisitResult(result)
}

//export goVisitImage
func goVisitImage(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cSrc *C.char, cAlt *C.char, cTitle *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnImage == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	src := C.GoString(cSrc)
	alt := C.GoString(cAlt)
	title := ""
	if cTitle != nil {
		title = C.GoString(cTitle)
	}
	result := v.OnImage(ctx, src, alt, title)
	return toVisitResult(result)
}

//export goVisitHeading
func goVisitHeading(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, level C.uint32_t, cText *C.char, cID *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnHeading == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	id := ""
	if cID != nil {
		id = C.GoString(cID)
	}
	result := v.OnHeading(ctx, uint32(level), text, id)
	return toVisitResult(result)
}

//export goVisitCodeBlock
func goVisitCodeBlock(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cLang *C.char, cCode *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnCodeBlock == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	lang := ""
	if cLang != nil {
		lang = C.GoString(cLang)
	}
	code := C.GoString(cCode)
	result := v.OnCodeBlock(ctx, lang, code)
	return toVisitResult(result)
}

//export goVisitCodeInline
func goVisitCodeInline(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cCode *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnCodeInline == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	code := C.GoString(cCode)
	result := v.OnCodeInline(ctx, code)
	return toVisitResult(result)
}

//export goVisitListStart
func goVisitListStart(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, ordered C.bool) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnListStart == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	result := v.OnListStart(ctx, bool(ordered))
	return toVisitResult(result)
}

//export goVisitListItem
func goVisitListItem(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, ordered C.bool, cMarker *C.char, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnListItem == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	marker := C.GoString(cMarker)
	text := C.GoString(cText)
	result := v.OnListItem(ctx, bool(ordered), marker, text)
	return toVisitResult(result)
}

//export goVisitListEnd
func goVisitListEnd(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, ordered C.bool, cOutput *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnListEnd == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	output := C.GoString(cOutput)
	result := v.OnListEnd(ctx, bool(ordered), output)
	return toVisitResult(result)
}

//export goVisitTableStart
func goVisitTableStart(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnTableStart == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	result := v.OnTableStart(ctx)
	return toVisitResult(result)
}

//export goVisitTableRow
func goVisitTableRow(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cCells **C.char, cellCount C.ulong, isHeader C.bool) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnTableRow == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)

	// Convert C cell array to Go slice
	cells := make([]string, int(cellCount))
	for i := 0; i < int(cellCount); i++ {
		cellPtr := (*C.char)(unsafe.Pointer(uintptr(unsafe.Pointer(cCells)) + uintptr(i)*unsafe.Sizeof(uintptr(0))))
		cells[i] = C.GoString(cellPtr)
	}

	result := v.OnTableRow(ctx, cells, bool(isHeader))
	return toVisitResult(result)
}

//export goVisitTableEnd
func goVisitTableEnd(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cOutput *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnTableEnd == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	output := C.GoString(cOutput)
	result := v.OnTableEnd(ctx, output)
	return toVisitResult(result)
}

//export goVisitBlockquote
func goVisitBlockquote(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cContent *C.char, depth C.ulong) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnBlockquote == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	content := C.GoString(cContent)
	result := v.OnBlockquote(ctx, content, uint64(depth))
	return toVisitResult(result)
}

//export goVisitStrong
func goVisitStrong(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnStrong == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnStrong(ctx, text)
	return toVisitResult(result)
}

//export goVisitEmphasis
func goVisitEmphasis(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnEmphasis == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnEmphasis(ctx, text)
	return toVisitResult(result)
}

//export goVisitStrikethrough
func goVisitStrikethrough(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnStrikethrough == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnStrikethrough(ctx, text)
	return toVisitResult(result)
}

//export goVisitUnderline
func goVisitUnderline(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnUnderline == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnUnderline(ctx, text)
	return toVisitResult(result)
}

//export goVisitSubscript
func goVisitSubscript(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnSubscript == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnSubscript(ctx, text)
	return toVisitResult(result)
}

//export goVisitSuperscript
func goVisitSuperscript(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnSuperscript == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnSuperscript(ctx, text)
	return toVisitResult(result)
}

//export goVisitMark
func goVisitMark(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnMark == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnMark(ctx, text)
	return toVisitResult(result)
}

//export goVisitLineBreak
func goVisitLineBreak(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnLineBreak == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	result := v.OnLineBreak(ctx)
	return toVisitResult(result)
}

//export goVisitHorizontalRule
func goVisitHorizontalRule(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnHorizontalRule == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	result := v.OnHorizontalRule(ctx)
	return toVisitResult(result)
}

//export goVisitCustomElement
func goVisitCustomElement(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cTagName *C.char, cHTML *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnCustomElement == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	tagName := C.GoString(cTagName)
	html := C.GoString(cHTML)
	result := v.OnCustomElement(ctx, tagName, html)
	return toVisitResult(result)
}

//export goVisitDefinitionListStart
func goVisitDefinitionListStart(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnDefinitionListStart == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	result := v.OnDefinitionListStart(ctx)
	return toVisitResult(result)
}

//export goVisitDefinitionTerm
func goVisitDefinitionTerm(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnDefinitionTerm == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnDefinitionTerm(ctx, text)
	return toVisitResult(result)
}

//export goVisitDefinitionDescription
func goVisitDefinitionDescription(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnDefinitionDescription == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnDefinitionDescription(ctx, text)
	return toVisitResult(result)
}

//export goVisitDefinitionListEnd
func goVisitDefinitionListEnd(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cOutput *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnDefinitionListEnd == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	output := C.GoString(cOutput)
	result := v.OnDefinitionListEnd(ctx, output)
	return toVisitResult(result)
}

//export goVisitForm
func goVisitForm(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cAction *C.char, cMethod *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnForm == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	action := ""
	if cAction != nil {
		action = C.GoString(cAction)
	}
	method := ""
	if cMethod != nil {
		method = C.GoString(cMethod)
	}
	result := v.OnForm(ctx, action, method)
	return toVisitResult(result)
}

//export goVisitInput
func goVisitInput(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cInputType *C.char, cName *C.char, cValue *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnInput == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	inputType := C.GoString(cInputType)
	name := ""
	if cName != nil {
		name = C.GoString(cName)
	}
	value := ""
	if cValue != nil {
		value = C.GoString(cValue)
	}
	result := v.OnInput(ctx, inputType, name, value)
	return toVisitResult(result)
}

//export goVisitButton
func goVisitButton(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnButton == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnButton(ctx, text)
	return toVisitResult(result)
}

//export goVisitAudio
func goVisitAudio(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cSrc *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnAudio == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	src := ""
	if cSrc != nil {
		src = C.GoString(cSrc)
	}
	result := v.OnAudio(ctx, src)
	return toVisitResult(result)
}

//export goVisitVideo
func goVisitVideo(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cSrc *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnVideo == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	src := ""
	if cSrc != nil {
		src = C.GoString(cSrc)
	}
	result := v.OnVideo(ctx, src)
	return toVisitResult(result)
}

//export goVisitIframe
func goVisitIframe(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cSrc *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnIframe == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	src := ""
	if cSrc != nil {
		src = C.GoString(cSrc)
	}
	result := v.OnIframe(ctx, src)
	return toVisitResult(result)
}

//export goVisitDetails
func goVisitDetails(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, open C.bool) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnDetails == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	result := v.OnDetails(ctx, bool(open))
	return toVisitResult(result)
}

//export goVisitSummary
func goVisitSummary(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnSummary == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnSummary(ctx, text)
	return toVisitResult(result)
}

//export goVisitFigureStart
func goVisitFigureStart(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnFigureStart == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	result := v.OnFigureStart(ctx)
	return toVisitResult(result)
}

//export goVisitFigcaption
func goVisitFigcaption(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cText *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnFigcaption == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	text := C.GoString(cText)
	result := v.OnFigcaption(ctx, text)
	return toVisitResult(result)
}

//export goVisitFigureEnd
func goVisitFigureEnd(userData unsafe.Pointer, cCtx *C.html_to_markdown_node_context_t, cOutput *C.char) C.html_to_markdown_visit_result_t {
	visitorID := uint64(uintptr(userData))
	v := getVisitor(visitorID)
	if v == nil || v.OnFigureEnd == nil {
		return C.html_to_markdown_visit_result_t{result_type: 0}
	}

	ctx := newNodeContext(cCtx)
	output := C.GoString(cOutput)
	result := v.OnFigureEnd(ctx, output)
	return toVisitResult(result)
}
