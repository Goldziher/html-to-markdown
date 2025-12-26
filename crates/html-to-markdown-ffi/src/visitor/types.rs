//! C-compatible type definitions for the visitor pattern.
//!
//! This module defines all FFI types required to implement custom visitors in C/C++ and
//! other languages via their C FFI bindings. All types use `#[repr(C)]` for layout
//! compatibility across language boundaries.
//!
//! # Memory Ownership Rules
//!
//! - **NodeContext**: Borrowed from Rust for callback duration only; do NOT free
//! - **Strings from NodeContext**: Borrowed; do NOT free (valid only during callback)
//! - **Custom output in VisitResult**: Owned by caller; must free with `html_to_markdown_free_string`
//! - **Error messages in VisitResult**: Owned by caller; must free with `html_to_markdown_free_string`
//! - **Attributes array**: Borrowed; do NOT free (valid only during callback)
//!
//! # String Encoding
//!
//! All `*const c_char` and `*mut c_char` pointers contain UTF-8 encoded strings.
//! NULL pointers are used to represent missing optional values (e.g., title in links).

#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_void};

// =============================================================================
// ENUMS
// =============================================================================

/// Node type enumeration covering all HTML element types.
///
/// Maps directly to the Rust `NodeType` enum. This enum categorizes HTML elements
/// for coarse-grained visitor dispatch.
///
/// # Values
///
/// - Text nodes (most frequent in documents)
/// - Block-level elements (headings, paragraphs, blockquotes, etc.)
/// - Inline formatting (strong, em, code, etc.)
/// - Lists (ol, ul, dl)
/// - Tables and table structures
/// - Forms and form elements
/// - Media elements (audio, video, iframe)
/// - Semantic HTML5 elements
/// - Document structure
/// - Custom/unknown elements (web components)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum html_to_markdown_node_type_t {
    // === Text and Generic ===
    HTML_TO_MARKDOWN_NODE_TEXT = 0,
    HTML_TO_MARKDOWN_NODE_ELEMENT = 1,

    // === Block Elements ===
    HTML_TO_MARKDOWN_NODE_HEADING = 2,
    HTML_TO_MARKDOWN_NODE_PARAGRAPH = 3,
    HTML_TO_MARKDOWN_NODE_DIV = 4,
    HTML_TO_MARKDOWN_NODE_BLOCKQUOTE = 5,
    HTML_TO_MARKDOWN_NODE_PRE = 6,
    HTML_TO_MARKDOWN_NODE_HR = 7,

    // === Lists ===
    HTML_TO_MARKDOWN_NODE_LIST = 8,
    HTML_TO_MARKDOWN_NODE_LIST_ITEM = 9,
    HTML_TO_MARKDOWN_NODE_DEFINITION_LIST = 10,
    HTML_TO_MARKDOWN_NODE_DEFINITION_TERM = 11,
    HTML_TO_MARKDOWN_NODE_DEFINITION_DESCRIPTION = 12,

    // === Tables ===
    HTML_TO_MARKDOWN_NODE_TABLE = 13,
    HTML_TO_MARKDOWN_NODE_TABLE_ROW = 14,
    HTML_TO_MARKDOWN_NODE_TABLE_CELL = 15,
    HTML_TO_MARKDOWN_NODE_TABLE_HEADER = 16,
    HTML_TO_MARKDOWN_NODE_TABLE_BODY = 17,
    HTML_TO_MARKDOWN_NODE_TABLE_HEAD = 18,
    HTML_TO_MARKDOWN_NODE_TABLE_FOOT = 19,

    // === Inline Formatting ===
    HTML_TO_MARKDOWN_NODE_LINK = 20,
    HTML_TO_MARKDOWN_NODE_IMAGE = 21,
    HTML_TO_MARKDOWN_NODE_STRONG = 22,
    HTML_TO_MARKDOWN_NODE_EM = 23,
    HTML_TO_MARKDOWN_NODE_CODE = 24,
    HTML_TO_MARKDOWN_NODE_STRIKETHROUGH = 25,
    HTML_TO_MARKDOWN_NODE_UNDERLINE = 26,
    HTML_TO_MARKDOWN_NODE_SUBSCRIPT = 27,
    HTML_TO_MARKDOWN_NODE_SUPERSCRIPT = 28,
    HTML_TO_MARKDOWN_NODE_MARK = 29,
    HTML_TO_MARKDOWN_NODE_SMALL = 30,
    HTML_TO_MARKDOWN_NODE_BR = 31,
    HTML_TO_MARKDOWN_NODE_SPAN = 32,

    // === Semantic HTML5 ===
    HTML_TO_MARKDOWN_NODE_ARTICLE = 33,
    HTML_TO_MARKDOWN_NODE_SECTION = 34,
    HTML_TO_MARKDOWN_NODE_NAV = 35,
    HTML_TO_MARKDOWN_NODE_ASIDE = 36,
    HTML_TO_MARKDOWN_NODE_HEADER = 37,
    HTML_TO_MARKDOWN_NODE_FOOTER = 38,
    HTML_TO_MARKDOWN_NODE_MAIN = 39,
    HTML_TO_MARKDOWN_NODE_FIGURE = 40,
    HTML_TO_MARKDOWN_NODE_FIGCAPTION = 41,
    HTML_TO_MARKDOWN_NODE_TIME = 42,
    HTML_TO_MARKDOWN_NODE_DETAILS = 43,
    HTML_TO_MARKDOWN_NODE_SUMMARY = 44,

    // === Forms ===
    HTML_TO_MARKDOWN_NODE_FORM = 45,
    HTML_TO_MARKDOWN_NODE_INPUT = 46,
    HTML_TO_MARKDOWN_NODE_SELECT = 47,
    HTML_TO_MARKDOWN_NODE_OPTION = 48,
    HTML_TO_MARKDOWN_NODE_BUTTON = 49,
    HTML_TO_MARKDOWN_NODE_TEXTAREA = 50,
    HTML_TO_MARKDOWN_NODE_LABEL = 51,
    HTML_TO_MARKDOWN_NODE_FIELDSET = 52,
    HTML_TO_MARKDOWN_NODE_LEGEND = 53,

    // === Media ===
    HTML_TO_MARKDOWN_NODE_AUDIO = 54,
    HTML_TO_MARKDOWN_NODE_VIDEO = 55,
    HTML_TO_MARKDOWN_NODE_PICTURE = 56,
    HTML_TO_MARKDOWN_NODE_SOURCE = 57,
    HTML_TO_MARKDOWN_NODE_IFRAME = 58,
    HTML_TO_MARKDOWN_NODE_SVG = 59,
    HTML_TO_MARKDOWN_NODE_CANVAS = 60,

    // === Advanced/Semantic ===
    HTML_TO_MARKDOWN_NODE_RUBY = 61,
    HTML_TO_MARKDOWN_NODE_RT = 62,
    HTML_TO_MARKDOWN_NODE_RP = 63,
    HTML_TO_MARKDOWN_NODE_ABBR = 64,
    HTML_TO_MARKDOWN_NODE_KBD = 65,
    HTML_TO_MARKDOWN_NODE_SAMP = 66,
    HTML_TO_MARKDOWN_NODE_VAR = 67,
    HTML_TO_MARKDOWN_NODE_CITE = 68,
    HTML_TO_MARKDOWN_NODE_Q = 69,
    HTML_TO_MARKDOWN_NODE_DEL = 70,
    HTML_TO_MARKDOWN_NODE_INS = 71,
    HTML_TO_MARKDOWN_NODE_DATA = 72,
    HTML_TO_MARKDOWN_NODE_METER = 73,
    HTML_TO_MARKDOWN_NODE_PROGRESS = 74,
    HTML_TO_MARKDOWN_NODE_OUTPUT = 75,
    HTML_TO_MARKDOWN_NODE_TEMPLATE = 76,
    HTML_TO_MARKDOWN_NODE_SLOT = 77,

    // === Document Structure ===
    HTML_TO_MARKDOWN_NODE_HTML = 78,
    HTML_TO_MARKDOWN_NODE_HEAD = 79,
    HTML_TO_MARKDOWN_NODE_BODY = 80,
    HTML_TO_MARKDOWN_NODE_TITLE = 81,
    HTML_TO_MARKDOWN_NODE_META = 82,
    HTML_TO_MARKDOWN_NODE_LINK_TAG = 83,
    HTML_TO_MARKDOWN_NODE_STYLE = 84,
    HTML_TO_MARKDOWN_NODE_SCRIPT = 85,
    HTML_TO_MARKDOWN_NODE_BASE = 86,

    // === Custom/Unknown ===
    HTML_TO_MARKDOWN_NODE_CUSTOM = 87,
}

/// Result type from a visitor callback.
///
/// Controls how the converter proceeds after a visitor callback:
/// - `Continue`: Use default conversion behavior for this node
/// - `Custom`: Replace output with caller-provided markdown
/// - `Skip`: Omit this element and all children from output
/// - `PreserveHtml`: Include raw HTML instead of converting
/// - `Error`: Halt conversion and report error
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum html_to_markdown_visit_result_type_t {
    /// Continue with default conversion behavior.
    HTML_TO_MARKDOWN_VISIT_CONTINUE = 0,

    /// Replace output with custom markdown (from `custom_output` field).
    HTML_TO_MARKDOWN_VISIT_CUSTOM = 1,

    /// Skip this element and all children entirely.
    HTML_TO_MARKDOWN_VISIT_SKIP = 2,

    /// Preserve original HTML instead of converting.
    HTML_TO_MARKDOWN_VISIT_PRESERVE_HTML = 3,

    /// Stop conversion and report error (from `error_message` field).
    HTML_TO_MARKDOWN_VISIT_ERROR = 4,
}

// =============================================================================
// STRUCTS
// =============================================================================

/// Attribute pair in a NULL-terminated array.
///
/// Used in `html_to_markdown_node_context_t::attributes` to represent HTML attributes.
/// Both `key` and `value` are UTF-8 C strings (never NULL, empty string is valid).
///
/// # Memory
///
/// Both pointers are borrowed from Rust; do NOT free. Valid only during callback.
///
/// # Example
///
/// For `<div class="container" id="main">`, the attribute array would be:
/// ```c
/// [
///     { "class", "container" },
///     { "id", "main" },
///     { NULL, NULL }  // Sentinel
/// ]
/// ```
#[repr(C)]
#[derive(Debug, Clone)]
pub struct html_to_markdown_attribute_t {
    /// Attribute name (UTF-8 C string). Never NULL.
    pub key: *const c_char,

    /// Attribute value (UTF-8 C string). Never NULL, may be empty.
    pub value: *const c_char,
}

/// Context information for a node being visited.
///
/// Passed to all visitor callbacks to provide metadata about the current element.
/// All string pointers and the attributes array are borrowed from Rust and valid
/// only during the callback invocation.
///
/// # Memory
///
/// **IMPORTANT**: Do NOT attempt to free any pointers in this struct. All data is
/// borrowed from the Rust converter and becomes invalid after the callback returns.
/// If you need to persist data, copy the string contents immediately.
///
/// # Attributes Iteration
///
/// The `attributes` array is NULL-terminated (sentinel entry with both fields NULL).
/// Always check for the sentinel before dereferencing:
///
/// ```c
/// for (int i = 0; attributes[i].key != NULL; i++) {
///     printf("%s = %s\n", attributes[i].key, attributes[i].value);
/// }
/// ```
#[repr(C)]
#[derive(Debug, Clone)]
pub struct html_to_markdown_node_context_t {
    /// Coarse-grained node type classification.
    pub node_type: html_to_markdown_node_type_t,

    /// Raw HTML tag name (e.g., "div", "h1", "custom-element").
    /// UTF-8 C string. Never NULL, may be empty for text nodes.
    pub tag_name: *const c_char,

    /// Attributes as a NULL-terminated array of pairs.
    ///
    /// The array is terminated by an entry with both `key` and `value` as NULL.
    /// If the element has no attributes, this points to an empty array (just sentinel).
    pub attributes: *const html_to_markdown_attribute_t,

    /// Depth in the DOM tree (0 = root).
    pub depth: usize,

    /// Index among siblings (0-based).
    pub index_in_parent: usize,

    /// Parent element's tag name. NULL if root element.
    /// UTF-8 C string.
    pub parent_tag: *const c_char,

    /// Whether this element is treated as inline vs block.
    pub is_inline: bool,
}

/// Result from a visitor callback.
///
/// Communicates to the converter how to proceed after visiting a node.
/// The converter uses the `result_type` field to determine action:
///
/// - **Continue**: Proceed with default behavior; other fields ignored
/// - **Custom**: Replace element output with `custom_output`; caller must allocate
/// - **Skip**: Discard element and children; other fields ignored
/// - **PreserveHtml**: Keep raw HTML; other fields ignored
/// - **Error**: Halt conversion; return `error_message` to caller
///
/// # Memory Ownership
///
/// The callback is responsible for allocating any returned strings:
/// - `custom_output`: Caller allocates; converter will free with `html_to_markdown_free_string`
/// - `error_message`: Caller allocates; converter will free with `html_to_markdown_free_string`
///
/// The converter does NOT take ownership until the callback returns successfully.
/// If you cannot allocate memory, return `Continue` instead.
///
/// # Example
///
/// ```c
/// html_to_markdown_visit_result_t result = {0};  // Zero-initialize
/// result.result_type = HTML_TO_MARKDOWN_VISIT_CUSTOM;
/// result.custom_output = malloc(16);
/// strcpy(result.custom_output, "**custom**");
/// return result;
/// ```
#[repr(C)]
#[derive(Debug, Clone)]
pub struct html_to_markdown_visit_result_t {
    /// Result type indicating action to take.
    pub result_type: html_to_markdown_visit_result_type_t,

    /// Custom markdown output (only used if `result_type == VISIT_CUSTOM`).
    ///
    /// Caller must allocate as UTF-8 C string; converter takes ownership and
    /// frees with `html_to_markdown_free_string`.
    /// Ignored for other result types.
    pub custom_output: *mut c_char,

    /// Error message (only used if `result_type == VISIT_ERROR`).
    ///
    /// Caller must allocate as UTF-8 C string; converter takes ownership and
    /// frees with `html_to_markdown_free_string`.
    /// Ignored for other result types.
    pub error_message: *mut c_char,
}

/// Visitor callbacks struct.
///
/// Container for all visitor callback function pointers. The converter invokes
/// these callbacks at appropriate points during traversal. Each callback receives
/// the `user_data` pointer to allow stateful visitors.
///
/// # Optional Callbacks
///
/// Set unused callbacks to NULL; the converter will skip them. Only implement
/// the callbacks you need.
///
/// # Callback Signatures
///
/// All callbacks receive:
/// 1. `user_data`: Opaque pointer passed to converter (for state)
/// 2. `ctx`: Node context (borrowed, valid only during callback)
/// 3. Element-specific parameters (strings, counts, etc.)
///
/// Callbacks return `html_to_markdown_visit_result_t`.
///
/// # Text Node Handling
///
/// `visit_text` is called frequently (100+ times per document). For performance,
/// return `Continue` quickly for unmodified text.
///
/// # Pre/Post Order Traversal
///
/// For element `<div><p>text</p></div>`:
/// 1. `visit_element_start` for `<div>`
/// 2. `visit_element_start` for `<p>`
/// 3. `visit_text` for "text"
/// 4. `visit_element_end` for `<p>` with default output
/// 5. `visit_element_end` for `</div>` with accumulated output
///
/// # Memory Safety
///
/// - Callbacks must be thread-safe if visitor is shared across threads
/// - String pointers in callbacks are borrowed; do NOT free
/// - Return allocated strings only in `custom_output` or `error_message`
/// - Avoid storing borrowed pointers; copy strings immediately
#[repr(C)]
#[derive(Debug, Clone)]
pub struct html_to_markdown_visitor_t {
    /// User-defined data pointer passed to all callbacks.
    ///
    /// Use this to maintain visitor state (e.g., counters, context stacks).
    pub user_data: *mut c_void,

    // === Generic Hooks ===
    /// Called before entering any HTML element.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx) -> VisitResult`
    pub visit_element_start: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Called after exiting any HTML element with the default markdown output.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *output) -> VisitResult`
    pub visit_element_end: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            output: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Text Nodes ===
    /// Visit text nodes (most frequent callback - 100+ per document).
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_text: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Links and Images ===
    /// Visit anchor links `<a href="...">`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *href, const char *text, const char *title) -> VisitResult`
    /// `title` may be NULL.
    pub visit_link: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            href: *const c_char,
            text: *const c_char,
            title: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit images `<img src="...">`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *src, const char *alt, const char *title) -> VisitResult`
    /// `title` may be NULL.
    pub visit_image: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            src: *const c_char,
            alt: *const c_char,
            title: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Headings ===
    /// Visit heading elements `<h1>` through `<h6>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, uint32_t level, const char *text, const char *id) -> VisitResult`
    /// `id` may be NULL.
    pub visit_heading: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            level: u32,
            text: *const c_char,
            id: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Code ===
    /// Visit code blocks `<pre><code>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *lang, const char *code) -> VisitResult`
    /// `lang` may be NULL.
    pub visit_code_block: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            lang: *const c_char,
            code: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit inline code `<code>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *code) -> VisitResult`
    pub visit_code_inline: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            code: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Lists ===
    /// Visit list items `<li>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, bool ordered, const char *marker, const char *text) -> VisitResult`
    pub visit_list_item: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            ordered: bool,
            marker: *const c_char,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Called before processing a list `<ul>` or `<ol>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, bool ordered) -> VisitResult`
    pub visit_list_start: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            ordered: bool,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Called after processing a list `</ul>` or `</ol>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, bool ordered, const char *output) -> VisitResult`
    pub visit_list_end: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            ordered: bool,
            output: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Tables ===
    /// Called before processing a table `<table>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx) -> VisitResult`
    pub visit_table_start: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit table rows `<tr>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char **cells, size_t cell_count, bool is_header) -> VisitResult`
    /// `cells` is a NULL-terminated array of cell contents.
    pub visit_table_row: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            cells: *const *const c_char,
            cell_count: usize,
            is_header: bool,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Called after processing a table `</table>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *output) -> VisitResult`
    pub visit_table_end: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            output: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Blockquotes ===
    /// Visit blockquote elements `<blockquote>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *content, size_t depth) -> VisitResult`
    pub visit_blockquote: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            content: *const c_char,
            depth: usize,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Inline Formatting ===
    /// Visit strong/bold elements `<strong>`, `<b>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_strong: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit emphasis/italic elements `<em>`, `<i>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_emphasis: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit strikethrough elements `<s>`, `<del>`, `<strike>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_strikethrough: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit underline elements `<u>`, `<ins>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_underline: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit subscript elements `<sub>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_subscript: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit superscript elements `<sup>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_superscript: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit mark/highlight elements `<mark>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_mark: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Breaks ===
    /// Visit line break elements `<br>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx) -> VisitResult`
    pub visit_line_break: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit horizontal rule elements `<hr>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx) -> VisitResult`
    pub visit_horizontal_rule: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Custom/Unknown Elements ===
    /// Visit custom elements (web components) or unknown tags.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *tag_name, const char *html) -> VisitResult`
    pub visit_custom_element: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            tag_name: *const c_char,
            html: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Definition Lists ===
    /// Visit definition list `<dl>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx) -> VisitResult`
    pub visit_definition_list_start: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit definition term `<dt>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_definition_term: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit definition description `<dd>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_definition_description: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Called after processing a definition list `</dl>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *output) -> VisitResult`
    pub visit_definition_list_end: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            output: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Forms ===
    /// Visit form elements `<form>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *action, const char *method) -> VisitResult`
    /// `action` and `method` may be NULL.
    pub visit_form: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            action: *const c_char,
            method: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit input elements `<input>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *input_type, const char *name, const char *value) -> VisitResult`
    /// `name` and `value` may be NULL.
    pub visit_input: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            input_type: *const c_char,
            name: *const c_char,
            value: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit button elements `<button>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_button: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Media ===
    /// Visit audio elements `<audio>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *src) -> VisitResult`
    /// `src` may be NULL.
    pub visit_audio: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            src: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit video elements `<video>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *src) -> VisitResult`
    /// `src` may be NULL.
    pub visit_video: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            src: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit iframe elements `<iframe>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *src) -> VisitResult`
    /// `src` may be NULL.
    pub visit_iframe: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            src: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    // === Semantic HTML5 ===
    /// Visit details elements `<details>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, bool open) -> VisitResult`
    pub visit_details: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            open: bool,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit summary elements `<summary>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_summary: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit figure elements `<figure>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx) -> VisitResult`
    pub visit_figure_start: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Visit figcaption elements `<figcaption>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *text) -> VisitResult`
    pub visit_figcaption: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            text: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,

    /// Called after processing a figure `</figure>`.
    ///
    /// Signature: `(void *user_data, const NodeContext *ctx, const char *output) -> VisitResult`
    pub visit_figure_end: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            ctx: *const html_to_markdown_node_context_t,
            output: *const c_char,
        ) -> html_to_markdown_visit_result_t,
    >,
}

// =============================================================================
// TYPE ALIASES for convenience
// =============================================================================

/// Convenience alias for visitor node type enumeration.
pub type CNodeType = html_to_markdown_node_type_t;

/// Convenience alias for visitor result type enumeration.
pub type CVisitResultType = html_to_markdown_visit_result_type_t;

/// Convenience alias for attribute struct.
pub type CAttribute = html_to_markdown_attribute_t;

/// Convenience alias for node context struct.
pub type CNodeContext = html_to_markdown_node_context_t;

/// Convenience alias for visitor result struct.
pub type CVisitResult = html_to_markdown_visit_result_t;

/// Convenience alias for visitor callbacks struct.
pub type CVisitor = html_to_markdown_visitor_t;
