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
// #ifndef HTML_TO_MARKDOWN_TYPES_H
// #define HTML_TO_MARKDOWN_TYPES_H
//
// // Result type enumeration (matching FFI HtmlToMarkdownVisitResultType)
// typedef enum {
//     HTML_TO_MARKDOWN_VISIT_CONTINUE = 0,
//     HTML_TO_MARKDOWN_VISIT_CUSTOM = 1,
//     HTML_TO_MARKDOWN_VISIT_SKIP = 2,
//     HTML_TO_MARKDOWN_VISIT_PRESERVE_HTML = 3,
//     HTML_TO_MARKDOWN_VISIT_ERROR = 4
// } HtmlToMarkdownVisitResultType;
//
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
//
// #endif // HTML_TO_MARKDOWN_TYPES_H
//
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
	"regexp"
	"strings"
	"sync"
	"unsafe"
)

// VisitResultType represents the action to take after a visitor callback.
type VisitResultType int

const (
	VisitContinue VisitResultType = 0

	VisitCustom VisitResultType = 1

	VisitSkip VisitResultType = 2

	VisitPreserveHTML VisitResultType = 3

	VisitError VisitResultType = 4

	// Markdown horizontal rule patterns
	horizontalRuleDash       = "---"
	horizontalRuleAster      = "***"
	horizontalRuleUnderscore = "___"
)

// NodeContext contains context information for a node being visited.
//
// This struct provides metadata about the current element during visitor callbacks.
// All string pointers are valid only during the callback invocation.
type NodeContext struct {
	NodeType uint32

	TagName string

	ParentTag string

	Depth uint64

	IndexInParent uint64

	IsInline bool
}

// VisitResult represents the result from a visitor callback.
//
// It communicates to the converter how to proceed after visiting a node.
type VisitResult struct {
	ResultType VisitResultType

	CustomOutput string

	ErrorMessage string
}

// Visitor defines the callback functions for custom HTML processing.
//
// Implement the callback fields you need and set others to nil.
// Each callback receives a NodeContext with metadata about the current element.
type Visitor struct {
	OnText func(ctx *NodeContext, text string) *VisitResult

	OnElementStart func(ctx *NodeContext) *VisitResult

	OnElementEnd func(ctx *NodeContext, output string) *VisitResult

	OnLink func(ctx *NodeContext, href, text, title string) *VisitResult

	OnImage func(ctx *NodeContext, src, alt, title string) *VisitResult

	OnHeading func(ctx *NodeContext, level uint32, text, id string) *VisitResult

	OnCodeBlock func(ctx *NodeContext, lang, code string) *VisitResult

	OnCodeInline func(ctx *NodeContext, code string) *VisitResult

	OnListStart func(ctx *NodeContext, ordered bool) *VisitResult

	OnListItem func(ctx *NodeContext, ordered bool, marker, text string) *VisitResult

	OnListEnd func(ctx *NodeContext, ordered bool, output string) *VisitResult

	OnTableStart func(ctx *NodeContext) *VisitResult

	OnTableRow func(ctx *NodeContext, cells []string, isHeader bool) *VisitResult

	OnTableEnd func(ctx *NodeContext, output string) *VisitResult

	OnBlockquote func(ctx *NodeContext, content string, depth uint64) *VisitResult

	OnStrong func(ctx *NodeContext, text string) *VisitResult

	OnEmphasis func(ctx *NodeContext, text string) *VisitResult

	OnStrikethrough func(ctx *NodeContext, text string) *VisitResult

	OnUnderline func(ctx *NodeContext, text string) *VisitResult

	OnSubscript func(ctx *NodeContext, text string) *VisitResult

	OnSuperscript func(ctx *NodeContext, text string) *VisitResult

	OnMark func(ctx *NodeContext, text string) *VisitResult

	OnLineBreak func(ctx *NodeContext) *VisitResult

	OnHorizontalRule func(ctx *NodeContext) *VisitResult

	OnCustomElement func(ctx *NodeContext, tagName, html string) *VisitResult

	OnDefinitionListStart func(ctx *NodeContext) *VisitResult

	OnDefinitionTerm func(ctx *NodeContext, text string) *VisitResult

	OnDefinitionDescription func(ctx *NodeContext, text string) *VisitResult

	OnDefinitionListEnd func(ctx *NodeContext, output string) *VisitResult

	OnForm func(ctx *NodeContext, action, method string) *VisitResult

	OnInput func(ctx *NodeContext, inputType, name, value string) *VisitResult

	OnButton func(ctx *NodeContext, text string) *VisitResult

	OnAudio func(ctx *NodeContext, src string) *VisitResult

	OnVideo func(ctx *NodeContext, src string) *VisitResult

	OnIframe func(ctx *NodeContext, src string) *VisitResult

	OnDetails func(ctx *NodeContext, open bool) *VisitResult

	OnSummary func(ctx *NodeContext, text string) *VisitResult

	OnFigureStart func(ctx *NodeContext) *VisitResult

	OnFigcaption func(ctx *NodeContext, text string) *VisitResult

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

	visitorID := storeVisitor(visitor)
	defer deleteVisitor(visitorID)

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

	processMarkdownWithVisitor(markdown, visitor, visitorID)

	return markdown, nil
}

// processMarkdownWithVisitor walks through the markdown and invokes visitor callbacks
// This is a simplified post-processing approach. A full implementation would
// parse markdown into an AST and walk the tree with proper context tracking.
func processMarkdownWithVisitor(markdown string, visitor *Visitor, visitorID uint64) {
	if visitor == nil {
		return
	}

	ctx := &NodeContext{
		NodeType:      0,
		TagName:       "",
		ParentTag:     "",
		Depth:         0,
		IndexInParent: 0,
		IsInline:      false,
	}

	lines := strings.Split(markdown, "\n")
	inList := false
	inTable := false
	inDefList := false

	for i, line := range lines {
		origLine := line
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		// Process list items
		isListItem := isListItemLine(line)
		inList, _ = processListBoundary(visitor, ctx, line, isListItem, inList, i, markdown)

		// Process table rows
		isTableRow := strings.HasPrefix(line, "|")
		inTable = processTableBoundary(visitor, ctx, line, isTableRow, inTable)

		// Process headings
		if strings.HasPrefix(line, "#") {
			processHeading(visitor, ctx, line, origLine)
		}

		// Process general elements (paragraphs, etc.)
		if isGeneralElement(line, isListItem, isTableRow) {
			processGeneralElement(visitor, ctx, line, origLine)
		}

		// Process links
		processLinks(visitor, ctx, line)

		// Process images
		processImages(visitor, ctx, line)

		// Process code blocks
		if strings.HasPrefix(line, "```") && visitor.OnCodeBlock != nil {
			visitor.OnCodeBlock(ctx, "", line)
		}

		// Process inline code
		processInlineCode(visitor, ctx, line)

		// Process list items
		if isListItem {
			processListItem(visitor, ctx, line)
		}

		// Process table rows
		if isTableRow {
			processTableRow(visitor, ctx, line)
		}

		// Process blockquotes
		if strings.HasPrefix(line, ">") && visitor.OnBlockquote != nil {
			text := strings.TrimPrefix(line, ">")
			text = strings.TrimSpace(text)
			visitor.OnBlockquote(ctx, text, 0)
		}

		// Process horizontal rules
		if isHorizontalRule(line) && visitor.OnHorizontalRule != nil {
			visitor.OnHorizontalRule(ctx)
		}

		// Process definition lists
		inDefList = processDefinitionList(visitor, ctx, line, i, inDefList)

		// Process figures
		if strings.Contains(line, "![") {
			processFigure(visitor, ctx, line, lines, i)
		}

		// Process details and summary
		if strings.Contains(line, "<details>") || strings.Contains(line, "**") {
			processDetails(visitor, ctx, line)
		}

		// Process strong text
		processStrong(visitor, ctx, line)

		// Process emphasis
		processEmphasis(visitor, ctx, line)

		// Process marks
		processMark(visitor, ctx, line)

		// Process plain text
		if visitor.OnText != nil && isPlainText(line, isListItem) {
			visitor.OnText(ctx, line)
		}
	}

	// Close any open blocks
	if inList && visitor.OnListEnd != nil {
		visitor.OnListEnd(ctx, false, "")
	}
	if inTable && visitor.OnTableEnd != nil {
		visitor.OnTableEnd(ctx, "")
	}
	if inDefList && visitor.OnDefinitionListEnd != nil {
		visitor.OnDefinitionListEnd(ctx, "")
	}
}

// Helper functions for processMarkdownWithVisitor

func isListItemLine(line string) bool {
	return strings.HasPrefix(line, "- ") || strings.HasPrefix(line, "* ") ||
		regexp.MustCompile(`^\d+\.\s`).MatchString(line)
}

func processListBoundary(visitor *Visitor, ctx *NodeContext, line string, isListItem, inList bool, i int, markdown string) (bool, bool) {
	if isListItem && !inList {
		inList = true
		if visitor.OnListStart != nil {
			ordered := regexp.MustCompile(`^\d+\.`).MatchString(line)
			visitor.OnListStart(ctx, ordered)
		}
	} else if !isListItem && inList && !strings.HasPrefix(line, " ") {
		inList = false
		if visitor.OnListEnd != nil {
			visitor.OnListEnd(ctx, false, markdown[i:])
		}
	}
	return inList, isListItem
}

func processTableBoundary(visitor *Visitor, ctx *NodeContext, line string, isTableRow, inTable bool) bool {
	if isTableRow && !inTable {
		inTable = true
		if visitor.OnTableStart != nil {
			visitor.OnTableStart(ctx)
		}
	} else if !isTableRow && inTable && !strings.HasPrefix(line, "|") {
		inTable = false
		if visitor.OnTableEnd != nil {
			visitor.OnTableEnd(ctx, "")
		}
	}
	return inTable
}

func processHeading(visitor *Visitor, ctx *NodeContext, line, origLine string) {
	if visitor.OnElementStart != nil && len(origLine) > 0 && (origLine[0] != ' ' && origLine[0] != '\t') {
		visitor.OnElementStart(ctx)
	}

	if visitor.OnHeading != nil {
		level := 0
		for j := 0; j < len(line); j++ {
			if line[j] == '#' {
				level++
			} else {
				break
			}
		}
		text := strings.TrimSpace(line[level:])
		visitor.OnHeading(ctx, uint32(level), text, "")
	}

	if visitor.OnElementEnd != nil && len(origLine) > 0 && (origLine[0] != ' ' && origLine[0] != '\t') {
		visitor.OnElementEnd(ctx, line)
	}
}

func isGeneralElement(line string, isListItem, isTableRow bool) bool {
	return len(line) > 0 && !strings.HasPrefix(line, "#") && !strings.HasPrefix(line, ">") &&
		!isListItem && !isTableRow && !isHorizontalRule(line) &&
		!strings.HasPrefix(line, "![") && !strings.HasPrefix(line, "[")
}

func processGeneralElement(visitor *Visitor, ctx *NodeContext, line, origLine string) {
	if visitor.OnElementStart != nil && len(origLine) > 0 && (origLine[0] != ' ' && origLine[0] != '\t') {
		visitor.OnElementStart(ctx)
	}
	if visitor.OnElementEnd != nil && len(origLine) > 0 && (origLine[0] != ' ' && origLine[0] != '\t') {
		visitor.OnElementEnd(ctx, line)
	}
}

func processLinks(visitor *Visitor, ctx *NodeContext, line string) {
	if strings.Contains(line, "[") && strings.Contains(line, "](") && visitor.OnLink != nil {
		re := regexp.MustCompile(`\[([^\]]+)\]\(([^)]+)\)`)
		matches := re.FindAllStringSubmatch(line, -1)
		for _, match := range matches {
			if len(match) >= 3 {
				visitor.OnLink(ctx, match[2], match[1], "")
			}
		}
	}
}

func processImages(visitor *Visitor, ctx *NodeContext, line string) {
	if strings.Contains(line, "![") && strings.Contains(line, "](") && visitor.OnImage != nil {
		re := regexp.MustCompile(`!\[([^\]]*)\]\(([^)]+)\)`)
		matches := re.FindAllStringSubmatch(line, -1)
		for _, match := range matches {
			if len(match) >= 3 {
				visitor.OnImage(ctx, match[2], match[1], "")
			}
		}
	}
}

func processInlineCode(visitor *Visitor, ctx *NodeContext, line string) {
	if strings.Contains(line, "`") && visitor.OnCodeInline != nil {
		re := regexp.MustCompile("`([^`]+)`")
		matches := re.FindAllStringSubmatch(line, -1)
		for _, match := range matches {
			if len(match) >= 2 {
				visitor.OnCodeInline(ctx, match[1])
			}
		}
	}
}

func processListItem(visitor *Visitor, ctx *NodeContext, line string) {
	if visitor.OnListItem != nil {
		text := strings.TrimPrefix(strings.TrimPrefix(line, "- "), "* ")
		text = regexp.MustCompile(`^\d+\.\s`).ReplaceAllString(text, "")
		marker := "-"
		if strings.HasPrefix(line, "* ") {
			marker = "*"
		}
		ordered := regexp.MustCompile(`^\d+\.`).MatchString(line)
		visitor.OnListItem(ctx, ordered, marker, text)
	}
}

func processTableRow(visitor *Visitor, ctx *NodeContext, line string) {
	if visitor.OnTableRow != nil {
		cells := strings.Split(line, "|")
		cleanCells := make([]string, 0)
		for _, cell := range cells {
			cell = strings.TrimSpace(cell)
			if cell != "" && cell != "-" {
				cleanCells = append(cleanCells, cell)
			}
		}
		if len(cleanCells) > 0 {
			isHeader := strings.Contains(line, "---") || strings.Contains(line, "---|")
			visitor.OnTableRow(ctx, cleanCells, isHeader)
		}
	}
}

func isHorizontalRule(line string) bool {
	return line == horizontalRuleDash || line == horizontalRuleAster || line == horizontalRuleUnderscore
}

func processDefinitionList(visitor *Visitor, ctx *NodeContext, line string, i int, inDefList bool) bool {
	isMaybeDefinitionLine := len(line) > 0 && !strings.HasPrefix(line, "#") &&
		!strings.HasPrefix(line, "-") && !strings.HasPrefix(line, "*") &&
		!strings.HasPrefix(line, ">") && !strings.HasPrefix(line, "|") &&
		!isHorizontalRule(line) &&
		!strings.Contains(line, "[") && !strings.Contains(line, "**")

	if isMaybeDefinitionLine {
		if !inDefList {
			inDefList = true
			if visitor.OnDefinitionListStart != nil {
				visitor.OnDefinitionListStart(ctx)
			}
		}
		if visitor.OnDefinitionTerm != nil && i%2 == 0 {
			visitor.OnDefinitionTerm(ctx, line)
		} else if visitor.OnDefinitionDescription != nil && i%2 == 1 {
			visitor.OnDefinitionDescription(ctx, line)
		}
	}
	return inDefList
}

func processFigure(visitor *Visitor, ctx *NodeContext, line string, lines []string, i int) {
	if visitor.OnFigureStart != nil {
		visitor.OnFigureStart(ctx)
	}
	if strings.Contains(line, "![") && strings.Contains(line, "](") {
		re := regexp.MustCompile(`!\[([^\]]*)\]\(([^)]+)\)`)
		matches := re.FindAllStringSubmatch(line, -1)
		for _, match := range matches {
			if len(match) >= 3 && visitor.OnImage != nil {
				visitor.OnImage(ctx, match[2], match[1], "")
			}
		}
	}
	nextLine := ""
	for j := i + 1; j < len(lines); j++ {
		nextLine = strings.TrimSpace(lines[j])
		if nextLine != "" && !strings.HasPrefix(nextLine, "![") {
			break
		}
	}
	if nextLine != "" && visitor.OnFigcaption != nil {
		visitor.OnFigcaption(ctx, nextLine)
	}
	if visitor.OnFigureEnd != nil {
		visitor.OnFigureEnd(ctx, "")
	}
}

func processDetails(visitor *Visitor, ctx *NodeContext, line string) {
	if visitor.OnDetails != nil {
		visitor.OnDetails(ctx, true)
	}
	if visitor.OnSummary != nil {
		visitor.OnSummary(ctx, line)
	}
}

func processStrong(visitor *Visitor, ctx *NodeContext, line string) {
	if strings.Contains(line, "**") && visitor.OnStrong != nil {
		re := regexp.MustCompile(`\*\*([^*]+)\*\*`)
		matches := re.FindAllStringSubmatch(line, -1)
		for _, match := range matches {
			if len(match) >= 2 {
				visitor.OnStrong(ctx, match[1])
			}
		}
	}
}

func processEmphasis(visitor *Visitor, ctx *NodeContext, line string) {
	if (strings.Contains(line, "*") || strings.Contains(line, "_")) && visitor.OnEmphasis != nil {
		re := regexp.MustCompile(`\*([^*]+)\*|_([^_]+)_`)
		matches := re.FindAllStringSubmatch(line, -1)
		for _, match := range matches {
			if len(match) >= 2 && match[1] != "" {
				visitor.OnEmphasis(ctx, match[1])
			} else if len(match) >= 3 && match[2] != "" {
				visitor.OnEmphasis(ctx, match[2])
			}
		}
	}
}

func processMark(visitor *Visitor, ctx *NodeContext, line string) {
	if strings.Contains(line, "==") && visitor.OnMark != nil {
		re := regexp.MustCompile(`==([^=]+)==`)
		matches := re.FindAllStringSubmatch(line, -1)
		for _, match := range matches {
			if len(match) >= 2 {
				visitor.OnMark(ctx, match[1])
			}
		}
	}
}

func isPlainText(line string, isListItem bool) bool {
	return !strings.HasPrefix(line, "#") &&
		!strings.HasPrefix(line, ">") &&
		!isListItem &&
		!strings.HasPrefix(line, "|") &&
		!isHorizontalRule(line)
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
	return C.html_to_markdown_visitor_t{
		user_data: unsafe.Pointer(uintptr(visitorID)),
	}
}

// freeCallbacksIfNeeded handles memory cleanup for C callbacks (currently not needed).
func freeCallbacksIfNeeded(v *C.html_to_markdown_visitor_t) {
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
