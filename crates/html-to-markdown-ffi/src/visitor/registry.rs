//! Visitor lifecycle management and registry for C FFI.
//!
//! This module provides the complete C API for creating, using, and freeing visitor instances.
//! It bridges the gap between C-style callback functions and Rust's visitor trait system,
//! implementing a registry pattern for tracking visitor lifetime and state.
//!
//! # Architecture
//!
//! The visitor lifecycle follows these stages:
//!
//! 1. **Creation**: `html_to_markdown_visitor_create()` allocates a visitor handle
//! 2. **Usage**: `html_to_markdown_convert_with_visitor()` invokes the visitor during conversion
//! 3. **Cleanup**: `html_to_markdown_visitor_free()` deallocates resources
//!
//! # Memory Safety
//!
//! - Visitor handles are opaque pointers (void*) representing Box<CVisitorWrapper>
//! - All pointer validation uses null checks before dereferencing
//! - Callback results (`VisitResult`) are returned by value; ownership is clear
//! - Strings are allocated by callback and must be freed by the FFI layer
//! - Thread safety: Each thread can have multiple visitor instances; no global state
//!
//! # Thread Safety
//!
//! - Visitor instances are NOT thread-safe by design (single-threaded visitor use)
//! - `LAST_ERROR` is thread-local; safe for concurrent threads with separate visitors
//! - Visitors MUST NOT be shared across threads
//!
//! # Example (C)
//!
//! ```c
//! // 1. Define callbacks
//! html_to_markdown_visit_result_t my_visit_text(
//!     void *user_data,
//!     const html_to_markdown_node_context_t *ctx,
//!     const char *text)
//! {
//!     html_to_markdown_visit_result_t result = {0};
//!     result.result_type = HTML_TO_MARKDOWN_VISIT_CONTINUE;
//!     return result;
//! }
//!
//! // 2. Create visitor
//! html_to_markdown_visitor_callbacks_t callbacks = {
//!     .user_data = NULL,
//!     .visit_text = my_visit_text,
//!     // ... set other callbacks to NULL for defaults
//! };
//! void *visitor = html_to_markdown_visitor_create(&callbacks);
//! if (visitor == NULL) {
//!     const char *err = html_to_markdown_last_error();
//!     fprintf(stderr, "Failed to create visitor: %s\n", err);
//!     return 1;
//! }
//!
//! // 3. Use visitor in conversion
//! size_t out_len = 0;
//! char *markdown = html_to_markdown_convert_with_visitor(
//!     html_input,
//!     visitor,
//!     &out_len);
//! if (markdown == NULL) {
//!     const char *err = html_to_markdown_last_error();
//!     fprintf(stderr, "Conversion failed: %s\n", err);
//!     html_to_markdown_visitor_free(visitor);
//!     return 1;
//! }
//!
//! // 4. Use result
//! printf("Markdown:\n%s\n", markdown);
//! html_to_markdown_free_string(markdown);
//!
//! // 5. Cleanup visitor
//! html_to_markdown_visitor_free(visitor);
//! ```
//!
//! # Error Handling Pattern
//!
//! All functions follow the pattern:
//! - Return NULL or false on error
//! - Call `html_to_markdown_last_error()` to get detailed error message
//! - Error message is thread-local and valid until next FFI call

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::rc::Rc;

use html_to_markdown_rs::convert_with_visitor;
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, NodeType, VisitResult};

use crate::error::{capture_error, set_last_error};
use crate::strings::string_to_c_string;

/// Opaque handle to a visitor instance.
///
/// Returned by `html_to_markdown_visitor_create()` and passed to
/// `html_to_markdown_convert_with_visitor()`. Contains ownership of the
/// underlying `CVisitorWrapper`.
pub type HtmlToMarkdownVisitor = *mut std::ffi::c_void;

/// Result type enumeration for visitor callbacks.
///
/// Indicates what action the conversion engine should take after a callback.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlToMarkdownVisitResultType {
    /// Continue with default conversion behavior
    Continue = 0,
    /// Replace default output with custom markdown (requires `custom_output` set)
    Custom = 1,
    /// Skip this element entirely
    Skip = 2,
    /// Preserve original HTML verbatim
    PreserveHtml = 3,
    /// Stop conversion with error (requires `error_message` set)
    Error = 4,
}

/// Result of a visitor callback (C-compatible).
///
/// Returned by all visitor callback functions. Only certain fields are meaningful
/// depending on `result_type`:
///
/// - **Continue**: No additional fields needed
/// - **Custom**: `custom_output` must point to a malloc'd string (NULL-terminated)
/// - **Skip**: No additional fields needed
/// - **`PreserveHtml`**: No additional fields needed
/// - **Error**: `error_message` must point to a malloc'd string (NULL-terminated)
///
/// # Memory Ownership
///
/// - `custom_output` and `error_message` are OWNED BY THE CALLBACK
/// - The FFI layer will free these strings after processing
/// - MUST be allocated with malloc/calloc; will be freed with `free()`
///
/// # Safety
///
/// The conversion engine guarantees:
/// - This struct is only valid during the callback and for the return statement
/// - Allocated strings are freed immediately after callback returns
/// - No references to callback data persist after return
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HtmlToMarkdownVisitResult {
    /// The action to take (Continue, Custom, Skip, etc.)
    pub result_type: HtmlToMarkdownVisitResultType,

    /// Custom markdown output (only if `result_type` == Custom)
    /// Must be malloc'd NULL-terminated string; ownership transfers to FFI layer
    pub custom_output: *mut c_char,

    /// Error message (only if `result_type` == Error)
    /// Must be malloc'd NULL-terminated string; ownership transfers to FFI layer
    pub error_message: *mut c_char,
}

impl Default for HtmlToMarkdownVisitResult {
    fn default() -> Self {
        Self {
            result_type: HtmlToMarkdownVisitResultType::Continue,
            custom_output: ptr::null_mut(),
            error_message: ptr::null_mut(),
        }
    }
}

/// Node type enumeration (mirrors Rust `NodeType`).
///
/// All HTML element types recognized by the converter are represented here.
/// Used in `NodeContext` to classify elements during visitor callbacks.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlToMarkdownNodeType {
    /// Text node (character data).
    Text = 0,
    /// Generic element node.
    Element = 1,
    /// Heading element (h1-h6).
    Heading = 2,
    /// Paragraph element.
    Paragraph = 3,
    /// Division element.
    Div = 4,
    /// Blockquote element.
    Blockquote = 5,
    /// Preformatted text element.
    Pre = 6,
    /// Horizontal rule element.
    Hr = 7,
    /// Unordered or ordered list element.
    List = 8,
    /// List item element.
    ListItem = 9,
    /// Table element.
    Table = 10,
    /// Table row element.
    TableRow = 11,
    /// Table cell element.
    TableCell = 12,
    /// Anchor/hyperlink element.
    Link = 13,
    /// Image element.
    Image = 14,
    /// Inline code element.
    Code = 15,
    /// Strong/bold element.
    Strong = 16,
    /// Emphasis/italic element.
    Em = 17,
    /// Custom or unknown element type.
    Custom = 255,
}

impl From<NodeType> for HtmlToMarkdownNodeType {
    fn from(nt: NodeType) -> Self {
        use HtmlToMarkdownNodeType::{
            Blockquote, Code, Custom, Div, Element, Em, Heading, Hr, Image, Link, List, ListItem, Paragraph, Pre,
            Strong, Table, TableCell, TableRow, Text,
        };
        match nt {
            NodeType::Text => Text,
            NodeType::Element => Element,
            NodeType::Heading => Heading,
            NodeType::Paragraph => Paragraph,
            NodeType::Div => Div,
            NodeType::Blockquote => Blockquote,
            NodeType::Pre => Pre,
            NodeType::Hr => Hr,
            NodeType::List => List,
            NodeType::ListItem => ListItem,
            NodeType::Table => Table,
            NodeType::TableRow => TableRow,
            NodeType::TableCell => TableCell,
            NodeType::Link => Link,
            NodeType::Image => Image,
            NodeType::Code => Code,
            NodeType::Strong => Strong,
            NodeType::Em => Em,
            _ => Custom,
        }
    }
}

/// Key-value attribute pair (C-compatible).
///
/// Used in attribute arrays passed to callbacks. Both key and value
/// are pointers to NULL-terminated C strings (valid for callback duration only).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HtmlToMarkdownAttribute {
    /// Attribute name (e.g., "href", "class", "id")
    pub key: *const c_char,
    /// Attribute value (e.g., "<https://example.com>", "container", "header-1")
    pub value: *const c_char,
}

/// Node context for visitor callbacks (C-compatible).
///
/// Provides comprehensive metadata about the current HTML node being visited.
/// All string pointers are valid only for the duration of the callback.
///
/// # Memory Safety
///
/// This struct is allocated by the conversion engine and remains valid only
/// during the callback. DO NOT store or dereference pointers after return.
///
/// # String Ownership
///
/// All string fields (`tag_name`, `parent_tag`, attribute keys/values)
/// are owned by the conversion engine and freed after callback returns.
/// Make copies if you need to preserve them.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HtmlToMarkdownNodeContext {
    /// Node type classification (coarse-grained)
    pub node_type: HtmlToMarkdownNodeType,

    /// Raw HTML tag name (e.g., "div", "h1", "custom-element")
    /// NULL-terminated C string
    pub tag_name: *const c_char,

    /// HTML attributes array, NULL-terminated
    /// Access: attributes[0], attributes[1], ..., until key == NULL
    pub attributes: *const HtmlToMarkdownAttribute,

    /// Depth in DOM tree (0 = root)
    pub depth: usize,

    /// Index among siblings (0-based)
    pub index_in_parent: usize,

    /// Parent element's tag name, or NULL if root
    pub parent_tag: *const c_char,

    /// Whether element is inline vs block (true = inline)
    pub is_inline: bool,

    /// RESERVED for future expansion; set to 0
    pub reserved: usize,
}

/// Visitor callback function type for text nodes.
///
/// Called for each text node in the HTML document.
/// This is the most frequently called callback (100+ times per document).
///
/// # Arguments
///
/// - `user_data`: Context pointer passed to `html_to_markdown_visitor_create()`
/// - `ctx`: Node context (valid only for callback duration)
/// - `text`: Text content (NULL-terminated, valid for callback duration)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` with `result_type` and optional custom output.
/// If custom, allocate the output string with `malloc()`.
///
/// # Safety
///
/// - `ctx` is valid only during callback; don't store or dereference after
/// - `text` is valid only during callback; make a copy if needed
/// - Returned `custom_output` MUST be malloc'd; will be freed with `free()`
pub type HtmlToMarkdownVisitTextCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for start of elements.
///
/// Called before entering any HTML element (pre-order traversal).
/// Generic hook for all element types.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context (valid only for callback duration)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action to take.
pub type HtmlToMarkdownVisitElementStartCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for end of elements.
///
/// Called after exiting any HTML element (post-order traversal).
/// Receives the default markdown output for the element.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context (valid only for callback duration)
/// - `output`: Default markdown output (NULL-terminated, valid for callback duration)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` with optional custom replacement output.
pub type HtmlToMarkdownVisitElementEndCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    output: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for links.
///
/// Called for anchor links `<a href="...">`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with link metadata
/// - `href`: Link URL (NULL-terminated, valid for callback duration)
/// - `text`: Link text (already converted to markdown, NULL-terminated)
/// - `title`: Title attribute, or NULL if not present
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitLinkCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    href: *const c_char,
    text: *const c_char,
    title: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for images.
///
/// Called for image elements `<img src="..." alt="...">`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with image metadata
/// - `src`: Image source URL (NULL-terminated)
/// - `alt`: Alt text (NULL-terminated)
/// - `title`: Title attribute, or NULL if not present
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitImageCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    src: *const c_char,
    alt: *const c_char,
    title: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for headings.
///
/// Called for heading elements `<h1>` through `<h6>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with heading metadata
/// - `level`: Heading level (1-6)
/// - `text`: Heading text content (NULL-terminated)
/// - `id`: ID attribute, or NULL if not present
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitHeadingCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    level: u32,
    text: *const c_char,
    id: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for code blocks.
///
/// Called for code block elements `<pre><code>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with code block metadata
/// - `lang`: Optional language specifier (NULL-terminated, or NULL if not present)
/// - `code`: Code content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitCodeBlockCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    lang: *const c_char,
    code: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for inline code.
///
/// Called for inline code elements `<code>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with code metadata
/// - `code`: Code content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitCodeInlineCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    code: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for list start.
///
/// Called before processing a list element `<ul>` or `<ol>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with list metadata
/// - `ordered`: Whether this is an ordered list (true) or unordered (false)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitListStartCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    ordered: bool,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for list items.
///
/// Called for list item elements `<li>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with list item metadata
/// - `ordered`: Whether this is in an ordered list
/// - `marker`: The list marker string (e.g., "-", "1.", "a)") (NULL-terminated)
/// - `text`: The list item content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitListItemCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    ordered: bool,
    marker: *const c_char,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for list end.
///
/// Called after processing a list element `</ul>` or `</ol>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with list metadata
/// - `ordered`: Whether this is an ordered list
/// - `output`: The default markdown output for the list (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitListEndCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    ordered: bool,
    output: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for table start.
///
/// Called before processing a table element `<table>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with table metadata
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitTableStartCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for table rows.
///
/// Called for table row elements `<tr>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with row metadata
/// - `cells`: Array of cell contents (NULL-terminated array of NULL-terminated strings)
/// - `cell_count`: Number of cells in the row
/// - `is_header`: Whether this row is in a header section (true) or body (false)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitTableRowCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    cells: *const *const c_char,
    cell_count: usize,
    is_header: bool,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for table end.
///
/// Called after processing a table element `</table>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with table metadata
/// - `output`: The default markdown output for the table (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitTableEndCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    output: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for blockquotes.
///
/// Called for blockquote elements `<blockquote>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with blockquote metadata
/// - `content`: The blockquote content (NULL-terminated)
/// - `depth`: Nesting depth for nested blockquotes
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitBlockquoteCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    content: *const c_char,
    depth: usize,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for strong/bold elements.
///
/// Called for `<strong>` and `<b>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with strong element metadata
/// - `text`: The text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitStrongCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for emphasis/italic elements.
///
/// Called for `<em>` and `<i>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with emphasis element metadata
/// - `text`: The text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitEmphasisCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for strikethrough elements.
///
/// Called for `<s>`, `<del>`, and `<strike>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with strikethrough element metadata
/// - `text`: The text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitStrikethroughCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for underline elements.
///
/// Called for `<u>` and `<ins>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with underline element metadata
/// - `text`: The text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitUnderlineCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for subscript elements.
///
/// Called for `<sub>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with subscript element metadata
/// - `text`: The text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitSubscriptCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for superscript elements.
///
/// Called for `<sup>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with superscript element metadata
/// - `text`: The text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitSuperscriptCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for mark/highlight elements.
///
/// Called for `<mark>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with mark element metadata
/// - `text`: The text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitMarkCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for line breaks.
///
/// Called for `<br>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with line break metadata
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitLineBreakCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for horizontal rules.
///
/// Called for `<hr>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with horizontal rule metadata
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitHorizontalRuleCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for custom/unknown elements.
///
/// Called for custom elements (web components) or unknown tags.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with custom element metadata
/// - `tag_name`: The custom element's tag name (NULL-terminated)
/// - `html`: The raw HTML of this element (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitCustomElementCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    tag_name: *const c_char,
    html: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for definition list start.
///
/// Called before processing a definition list `<dl>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with definition list metadata
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitDefinitionListStartCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for definition terms.
///
/// Called for definition term elements `<dt>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with definition term metadata
/// - `text`: The term text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitDefinitionTermCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for definition descriptions.
///
/// Called for definition description elements `<dd>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with definition description metadata
/// - `text`: The description text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitDefinitionDescriptionCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for definition list end.
///
/// Called after processing a definition list `</dl>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with definition list metadata
/// - `output`: The default markdown output for the definition list (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitDefinitionListEndCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    output: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for form elements.
///
/// Called for `<form>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with form metadata
/// - `action`: Form action attribute (NULL-terminated, or NULL if not present)
/// - `method`: Form method attribute (NULL-terminated, or NULL if not present)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitFormCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    action: *const c_char,
    method: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for input elements.
///
/// Called for `<input>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with input metadata
/// - `input_type`: Input type attribute (NULL-terminated)
/// - `name`: Name attribute (NULL-terminated, or NULL if not present)
/// - `value`: Value attribute (NULL-terminated, or NULL if not present)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitInputCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    input_type: *const c_char,
    name: *const c_char,
    value: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for button elements.
///
/// Called for `<button>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with button metadata
/// - `text`: The button text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitButtonCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for audio elements.
///
/// Called for `<audio>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with audio metadata
/// - `src`: Source URL (NULL-terminated, or NULL if not present)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitAudioCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    src: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for video elements.
///
/// Called for `<video>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with video metadata
/// - `src`: Source URL (NULL-terminated, or NULL if not present)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitVideoCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    src: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for iframe elements.
///
/// Called for `<iframe>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with iframe metadata
/// - `src`: Source URL (NULL-terminated, or NULL if not present)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitIframeCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    src: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for details elements.
///
/// Called for `<details>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with details metadata
/// - `open`: Whether the details element is open (true) or closed (false)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitDetailsCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    open: bool,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for summary elements.
///
/// Called for `<summary>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with summary metadata
/// - `text`: The summary text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitSummaryCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for figure start.
///
/// Called before processing a figure element `<figure>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with figure metadata
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitFigureStartCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for figcaption elements.
///
/// Called for `<figcaption>` elements.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with figcaption metadata
/// - `text`: The caption text content (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitFigcaptionCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    text: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Visitor callback function type for figure end.
///
/// Called after processing a figure element `</figure>`.
///
/// # Arguments
///
/// - `user_data`: Context pointer from `html_to_markdown_visitor_create()`
/// - `ctx`: Node context with figure metadata
/// - `output`: The default markdown output for the figure (NULL-terminated)
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` indicating action.
pub type HtmlToMarkdownVisitFigureEndCallback = unsafe extern "C" fn(
    user_data: *mut std::ffi::c_void,
    ctx: *const HtmlToMarkdownNodeContext,
    output: *const c_char,
) -> HtmlToMarkdownVisitResult;

/// Complete callback table for visitor (C-compatible).
///
/// Contains all callback function pointers for visitor events.
/// Set unused callbacks to NULL for default behavior.
///
/// # Example
///
/// ```c
/// html_to_markdown_visitor_callbacks_t callbacks = {
///     .user_data = my_context_ptr,
///     .visit_text = my_visit_text_fn,
///     .visit_link = my_visit_link_fn,
///     // Leave others NULL for defaults
///     .visit_image = NULL,
///     .visit_heading = NULL,
/// };
/// ```
#[repr(C)]
#[derive(Clone)]
pub struct HtmlToMarkdownVisitorCallbacks {
    /// User-provided context pointer passed to all callbacks
    pub user_data: *mut std::ffi::c_void,

    /// Called for text nodes (most frequent)
    pub visit_text: Option<HtmlToMarkdownVisitTextCallback>,

    /// Called before entering any element
    pub visit_element_start: Option<HtmlToMarkdownVisitElementStartCallback>,

    /// Called after exiting any element
    pub visit_element_end: Option<HtmlToMarkdownVisitElementEndCallback>,

    /// Called for anchor links
    pub visit_link: Option<HtmlToMarkdownVisitLinkCallback>,

    /// Called for images
    pub visit_image: Option<HtmlToMarkdownVisitImageCallback>,

    /// Called for headings
    pub visit_heading: Option<HtmlToMarkdownVisitHeadingCallback>,

    /// Called for code blocks
    pub visit_code_block: Option<HtmlToMarkdownVisitCodeBlockCallback>,

    /// Called for inline code
    pub visit_code_inline: Option<HtmlToMarkdownVisitCodeInlineCallback>,

    /// Called for list items
    pub visit_list_item: Option<HtmlToMarkdownVisitListItemCallback>,

    /// Called before processing a list
    pub visit_list_start: Option<HtmlToMarkdownVisitListStartCallback>,

    /// Called after processing a list
    pub visit_list_end: Option<HtmlToMarkdownVisitListEndCallback>,

    /// Called before processing a table
    pub visit_table_start: Option<HtmlToMarkdownVisitTableStartCallback>,

    /// Called for table rows
    pub visit_table_row: Option<HtmlToMarkdownVisitTableRowCallback>,

    /// Called after processing a table
    pub visit_table_end: Option<HtmlToMarkdownVisitTableEndCallback>,

    /// Called for blockquotes
    pub visit_blockquote: Option<HtmlToMarkdownVisitBlockquoteCallback>,

    /// Called for strong text
    pub visit_strong: Option<HtmlToMarkdownVisitStrongCallback>,

    /// Called for emphasis text
    pub visit_emphasis: Option<HtmlToMarkdownVisitEmphasisCallback>,

    /// Called for strikethrough text
    pub visit_strikethrough: Option<HtmlToMarkdownVisitStrikethroughCallback>,

    /// Called for underline text
    pub visit_underline: Option<HtmlToMarkdownVisitUnderlineCallback>,

    /// Called for subscript text
    pub visit_subscript: Option<HtmlToMarkdownVisitSubscriptCallback>,

    /// Called for superscript text
    pub visit_superscript: Option<HtmlToMarkdownVisitSuperscriptCallback>,

    /// Called for mark text
    pub visit_mark: Option<HtmlToMarkdownVisitMarkCallback>,

    /// Called for line breaks
    pub visit_line_break: Option<HtmlToMarkdownVisitLineBreakCallback>,

    /// Called for horizontal rules
    pub visit_horizontal_rule: Option<HtmlToMarkdownVisitHorizontalRuleCallback>,

    /// Called for custom elements
    pub visit_custom_element: Option<HtmlToMarkdownVisitCustomElementCallback>,

    /// Called before processing a definition list
    pub visit_definition_list_start: Option<HtmlToMarkdownVisitDefinitionListStartCallback>,

    /// Called for definition terms
    pub visit_definition_term: Option<HtmlToMarkdownVisitDefinitionTermCallback>,

    /// Called for definition descriptions
    pub visit_definition_description: Option<HtmlToMarkdownVisitDefinitionDescriptionCallback>,

    /// Called after processing a definition list
    pub visit_definition_list_end: Option<HtmlToMarkdownVisitDefinitionListEndCallback>,

    /// Called for form elements
    pub visit_form: Option<HtmlToMarkdownVisitFormCallback>,

    /// Called for input elements
    pub visit_input: Option<HtmlToMarkdownVisitInputCallback>,

    /// Called for button elements
    pub visit_button: Option<HtmlToMarkdownVisitButtonCallback>,

    /// Called for audio elements
    pub visit_audio: Option<HtmlToMarkdownVisitAudioCallback>,

    /// Called for video elements
    pub visit_video: Option<HtmlToMarkdownVisitVideoCallback>,

    /// Called for iframe elements
    pub visit_iframe: Option<HtmlToMarkdownVisitIframeCallback>,

    /// Called for details elements
    pub visit_details: Option<HtmlToMarkdownVisitDetailsCallback>,

    /// Called for summary elements
    pub visit_summary: Option<HtmlToMarkdownVisitSummaryCallback>,

    /// Called before processing a figure
    pub visit_figure_start: Option<HtmlToMarkdownVisitFigureStartCallback>,

    /// Called for figcaption elements
    pub visit_figcaption: Option<HtmlToMarkdownVisitFigcaptionCallback>,

    /// Called after processing a figure
    pub visit_figure_end: Option<HtmlToMarkdownVisitFigureEndCallback>,
}

/// Internal wrapper implementing `HtmlVisitor` trait from C callbacks.
///
/// Bridges the gap between:
/// - C callback functions (function pointers with void* `user_data`)
/// - Rust `HtmlVisitor` trait (method receivers with &mut self)
///
/// This wrapper is allocated in a Box and stored as an opaque pointer
/// in the `HtmlToMarkdownVisitor` handle.
///
/// # Design Notes
///
/// - Stores callback function pointers and `user_data` (owned by caller)
/// - Attributes and temporary strings are stored in wrapper, valid only during callback
/// - Callback string parameters are C string pointers (valid only during callback)
/// - Returned custom output/error strings are malloc'd; we take ownership and free them
struct CVisitorWrapper {
    /// C callback table with function pointers and `user_data`
    callbacks: HtmlToMarkdownVisitorCallbacks,
    /// Temporary storage for tag name `CString` (valid only during callback)
    temp_tag_name: RefCell<Option<CString>>,
    /// Temporary storage for parent tag `CString` (valid only during callback)
    temp_parent_tag: RefCell<Option<CString>>,
    /// Temporary storage for attribute array and key/value `CStrings`
    temp_attributes: RefCell<Vec<HtmlToMarkdownAttribute>>,
    /// Temporary storage for attribute key/value `CStrings` (kept alive during callback)
    temp_attribute_strings: RefCell<Vec<CString>>,
}

impl CVisitorWrapper {
    /// Create a new visitor wrapper from C callbacks.
    const fn new(callbacks: HtmlToMarkdownVisitorCallbacks) -> Self {
        Self {
            callbacks,
            temp_tag_name: RefCell::new(None),
            temp_parent_tag: RefCell::new(None),
            temp_attributes: RefCell::new(Vec::new()),
            temp_attribute_strings: RefCell::new(Vec::new()),
        }
    }

    /// Build a C `NodeContext` from a Rust `NodeContext`.
    ///
    /// Note: The returned context contains pointers to temporary strings stored in `RefCell` fields.
    /// These strings remain valid only during the callback execution. After the callback returns,
    /// the strings are cleared and the pointers become invalid.
    fn build_node_context(&self, ctx: &NodeContext) -> HtmlToMarkdownNodeContext {
        let c_tag_name = CString::new(ctx.tag_name.clone()).unwrap_or_else(|_| CString::new("").unwrap());
        let tag_name_ptr = c_tag_name.as_ptr();
        *self.temp_tag_name.borrow_mut() = Some(c_tag_name);

        let parent_tag_ptr = if let Some(parent) = &ctx.parent_tag {
            let c_parent = CString::new(parent.clone()).unwrap_or_else(|_| CString::new("").unwrap());
            let ptr = c_parent.as_ptr();
            *self.temp_parent_tag.borrow_mut() = Some(c_parent);
            ptr
        } else {
            ptr::null()
        };

        let mut attrs = Vec::new();
        let mut attr_strings = Vec::new();

        for (key, value) in &ctx.attributes {
            let c_key = CString::new(key.clone()).unwrap_or_default();
            let c_value = CString::new(value.clone()).unwrap_or_default();
            let key_ptr = c_key.as_ptr();
            let value_ptr = c_value.as_ptr();

            attrs.push(HtmlToMarkdownAttribute {
                key: key_ptr,
                value: value_ptr,
            });

            attr_strings.push(c_key);
            attr_strings.push(c_value);
        }

        attrs.push(HtmlToMarkdownAttribute {
            key: ptr::null(),
            value: ptr::null(),
        });

        *self.temp_attribute_strings.borrow_mut() = attr_strings;
        let attributes_ptr = if attrs.is_empty() {
            ptr::null()
        } else {
            *self.temp_attributes.borrow_mut() = attrs;
            self.temp_attributes.borrow().as_ptr()
        };

        HtmlToMarkdownNodeContext {
            node_type: ctx.node_type.into(),
            tag_name: tag_name_ptr,
            attributes: attributes_ptr,
            depth: ctx.depth,
            index_in_parent: ctx.index_in_parent,
            parent_tag: parent_tag_ptr,
            is_inline: ctx.is_inline,
            reserved: 0,
        }
    }

    /// Clear temporary strings after callback execution.
    ///
    /// This should be called after each callback to ensure memory is not leaked.
    fn clear_temp_strings(&self) {
        *self.temp_tag_name.borrow_mut() = None;
        *self.temp_parent_tag.borrow_mut() = None;
        *self.temp_attributes.borrow_mut() = Vec::new();
        *self.temp_attribute_strings.borrow_mut() = Vec::new();
    }

    /// Process a `VisitResult` from a C callback, handling memory cleanup.
    fn process_result(&self, result: HtmlToMarkdownVisitResult) -> VisitResult {
        match result.result_type {
            HtmlToMarkdownVisitResultType::Continue => VisitResult::Continue,
            HtmlToMarkdownVisitResultType::Custom => {
                if result.custom_output.is_null() {
                    VisitResult::Continue
                } else {
                    let output = unsafe { CStr::from_ptr(result.custom_output).to_string_lossy().into_owned() };
                    unsafe { libc::free(result.custom_output.cast::<std::ffi::c_void>()) };
                    VisitResult::Custom(output)
                }
            }
            HtmlToMarkdownVisitResultType::Skip => VisitResult::Skip,
            HtmlToMarkdownVisitResultType::PreserveHtml => VisitResult::PreserveHtml,
            HtmlToMarkdownVisitResultType::Error => {
                if result.error_message.is_null() {
                    VisitResult::Error("unknown error".to_string())
                } else {
                    let msg = unsafe { CStr::from_ptr(result.error_message).to_string_lossy().into_owned() };
                    unsafe { libc::free(result.error_message.cast::<std::ffi::c_void>()) };
                    VisitResult::Error(msg)
                }
            }
        }
    }
}

impl std::fmt::Debug for CVisitorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CVisitorWrapper")
            .field("user_data", &(self.callbacks.user_data as usize))
            .finish()
    }
}

impl HtmlVisitor for CVisitorWrapper {
    fn visit_text(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_text {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);

            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_element_start {
            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_element_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_element_end {
            let c_output_string =
                std::ffi::CString::new(output).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_output = c_output_string.as_ptr();
            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_output) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_link(&mut self, ctx: &NodeContext, href: &str, text: &str, title: Option<&str>) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_link {
            let c_href_string = std::ffi::CString::new(href).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_title_string = title.and_then(|t| std::ffi::CString::new(t).ok());

            let c_href = c_href_string.as_ptr();
            let c_text = c_text_string.as_ptr();
            let c_title = c_title_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_href, c_text, c_title) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_image(&mut self, ctx: &NodeContext, src: &str, alt: &str, title: Option<&str>) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_image {
            let c_src_string = std::ffi::CString::new(src).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_alt_string = std::ffi::CString::new(alt).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_title_string = title.and_then(|t| std::ffi::CString::new(t).ok());

            let c_src = c_src_string.as_ptr();
            let c_alt = c_alt_string.as_ptr();
            let c_title = c_title_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_src, c_alt, c_title) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_heading(&mut self, ctx: &NodeContext, level: u32, text: &str, id: Option<&str>) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_heading {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_id_string = id.and_then(|i| std::ffi::CString::new(i).ok());

            let c_text = c_text_string.as_ptr();
            let c_id = c_id_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, level, c_text, c_id) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_code_block(&mut self, ctx: &NodeContext, lang: Option<&str>, code: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_code_block {
            let c_lang_string = lang.and_then(|l| std::ffi::CString::new(l).ok());
            let c_code_string = std::ffi::CString::new(code).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());

            let c_lang = c_lang_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());
            let c_code = c_code_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_lang, c_code) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_code_inline(&mut self, ctx: &NodeContext, code: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_code_inline {
            let c_code_string = std::ffi::CString::new(code).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_code = c_code_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_code) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_list_item(&mut self, ctx: &NodeContext, ordered: bool, marker: &str, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_list_item {
            let c_marker_string =
                std::ffi::CString::new(marker).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());

            let c_marker = c_marker_string.as_ptr();
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, ordered, c_marker, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_list_start(&mut self, ctx: &NodeContext, ordered: bool) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_list_start {
            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, ordered) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_list_end(&mut self, ctx: &NodeContext, ordered: bool, output: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_list_end {
            let c_output_string =
                std::ffi::CString::new(output).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_output = c_output_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, ordered, c_output) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_table_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_table_start {
            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_table_row(&mut self, ctx: &NodeContext, cells: &[String], is_header: bool) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_table_row {
            let c_cells: Vec<std::ffi::CString> = cells
                .iter()
                .map(|cell| {
                    std::ffi::CString::new(cell.clone()).unwrap_or_else(|_| std::ffi::CString::new("").unwrap())
                })
                .collect();

            let c_cell_ptrs: Vec<*const c_char> = c_cells.iter().map(|s| s.as_ptr()).collect();
            let c_cells_ptr = c_cell_ptrs.as_ptr();
            let cells_len = c_cell_ptrs.len();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe {
                callback(
                    self.callbacks.user_data,
                    &raw const c_ctx,
                    c_cells_ptr,
                    cells_len,
                    is_header,
                )
            };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_table_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_table_end {
            let c_output_string =
                std::ffi::CString::new(output).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_output = c_output_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_output) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_blockquote(&mut self, ctx: &NodeContext, content: &str, depth: usize) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_blockquote {
            let c_content_string =
                std::ffi::CString::new(content).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_content = c_content_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_content, depth) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_strong(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_strong {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_emphasis(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_emphasis {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_strikethrough(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_strikethrough {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_underline(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_underline {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_subscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_subscript {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_superscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_superscript {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_mark(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_mark {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_line_break(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_line_break {
            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_horizontal_rule(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_horizontal_rule {
            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_custom_element(&mut self, ctx: &NodeContext, tag_name: &str, html: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_custom_element {
            let c_tag_name_string =
                std::ffi::CString::new(tag_name).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_html_string = std::ffi::CString::new(html).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());

            let c_tag_name = c_tag_name_string.as_ptr();
            let c_html = c_html_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_tag_name, c_html) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_definition_list_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_definition_list_start {
            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_definition_term(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_definition_term {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_definition_description(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_definition_description {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_definition_list_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_definition_list_end {
            let c_output_string =
                std::ffi::CString::new(output).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_output = c_output_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_output) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_form(&mut self, ctx: &NodeContext, action: Option<&str>, method: Option<&str>) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_form {
            let c_action_string = action.and_then(|a| std::ffi::CString::new(a).ok());
            let c_method_string = method.and_then(|m| std::ffi::CString::new(m).ok());

            let c_action = c_action_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());
            let c_method = c_method_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_action, c_method) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_input(
        &mut self,
        ctx: &NodeContext,
        input_type: &str,
        name: Option<&str>,
        value: Option<&str>,
    ) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_input {
            let c_input_type_string =
                std::ffi::CString::new(input_type).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_name_string = name.and_then(|n| std::ffi::CString::new(n).ok());
            let c_value_string = value.and_then(|v| std::ffi::CString::new(v).ok());

            let c_input_type = c_input_type_string.as_ptr();
            let c_name = c_name_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());
            let c_value = c_value_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe {
                callback(
                    self.callbacks.user_data,
                    &raw const c_ctx,
                    c_input_type,
                    c_name,
                    c_value,
                )
            };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_button(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_button {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_audio(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_audio {
            let c_src_string = src.and_then(|s| std::ffi::CString::new(s).ok());
            let c_src = c_src_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_src) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_video(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_video {
            let c_src_string = src.and_then(|s| std::ffi::CString::new(s).ok());
            let c_src = c_src_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_src) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_iframe(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_iframe {
            let c_src_string = src.and_then(|s| std::ffi::CString::new(s).ok());
            let c_src = c_src_string.as_ref().map_or(ptr::null(), |s| s.as_ptr());

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_src) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_details(&mut self, ctx: &NodeContext, open: bool) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_details {
            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, open) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_summary(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_summary {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_figure_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_figure_start {
            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_figcaption(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_figcaption {
            let c_text_string = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_text = c_text_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_text) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }

    fn visit_figure_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.visit_figure_end {
            let c_output_string =
                std::ffi::CString::new(output).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
            let c_output = c_output_string.as_ptr();

            let c_ctx = self.build_node_context(ctx);
            let result = unsafe { callback(self.callbacks.user_data, &raw const c_ctx, c_output) };
            let visit_result = self.process_result(result);
            self.clear_temp_strings();
            visit_result
        } else {
            VisitResult::Continue
        }
    }
}

/// Create a new visitor instance from a callback table.
///
/// Allocates a visitor handle that can be used with `html_to_markdown_convert_with_visitor()`.
/// The visitor is NOT thread-safe; each thread must create its own visitor.
///
/// # Arguments
///
/// - `callbacks`: Pointer to callback table. Must be valid for the entire lifetime
///   of the returned visitor handle.
///
/// # Returns
///
/// - Non-NULL: Opaque visitor handle (pass to convert functions)
/// - NULL: Failed to create visitor; call `html_to_markdown_last_error()` for details
///
/// # Safety
///
/// - `callbacks` must point to a valid `HtmlToMarkdownVisitorCallbacks` struct
/// - `callbacks` must remain valid until visitor is freed
/// - Returned handle must be freed with `html_to_markdown_visitor_free()`
/// - Returned handle is NOT thread-safe; don't share across threads
///
/// # Example
///
/// ```c
/// html_to_markdown_visitor_callbacks_t callbacks = {0};
/// callbacks.visit_text = my_visit_text;
/// void *visitor = html_to_markdown_visitor_create(&callbacks);
/// if (visitor == NULL) {
///     fprintf(stderr, "Failed: %s\n", html_to_markdown_last_error());
///     return 1;
/// }
/// // Use visitor...
/// html_to_markdown_visitor_free(visitor);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_visitor_create(
    callbacks: *const HtmlToMarkdownVisitorCallbacks,
) -> HtmlToMarkdownVisitor {
    if callbacks.is_null() {
        set_last_error(Some("callbacks pointer was null".to_string()));
        return ptr::null_mut();
    }

    let callbacks = unsafe { (*callbacks).clone() };
    let wrapper = CVisitorWrapper::new(callbacks);
    let handle = Rc::new(RefCell::new(wrapper));
    set_last_error(None);
    Box::into_raw(Box::new(handle)) as HtmlToMarkdownVisitor
}

/// Free a visitor instance created by `html_to_markdown_visitor_create()`.
///
/// Deallocates all resources associated with the visitor.
/// After this call, the visitor handle is invalid and must not be used.
///
/// # Arguments
///
/// - `visitor`: Visitor handle from `html_to_markdown_visitor_create()`.
///   NULL pointers are safe (no-op).
///
/// # Safety
///
/// - `visitor` must be a handle previously returned by `html_to_markdown_visitor_create()`
/// - `visitor` must not be used after this call
/// - Calling with NULL is safe but unnecessary
///
/// # Example
///
/// ```c
/// void *visitor = html_to_markdown_visitor_create(&callbacks);
/// // Use visitor in conversions...
/// html_to_markdown_visitor_free(visitor);
/// visitor = NULL; // Good practice to avoid use-after-free
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_visitor_free(visitor: HtmlToMarkdownVisitor) {
    if visitor.is_null() {
        return;
    }
    let _handle = unsafe { Box::from_raw(visitor.cast::<Rc<RefCell<CVisitorWrapper>>>()) };
}

/// Convert HTML to Markdown using a custom visitor.
///
/// Performs HTML→Markdown conversion with visitor callbacks invoked at each element.
/// Returns the length of the output markdown.
///
/// # Arguments
///
/// - `html`: Null-terminated UTF-8 C string containing HTML
/// - `visitor`: Visitor handle from `html_to_markdown_visitor_create()`
/// - `len_out`: Pointer to `size_t` where output length will be written.
///   Can be NULL if length is not needed.
///
/// # Returns
///
/// - Non-NULL: Pointer to malloc'd markdown string (NULL-terminated).
///   Length written to *`len_out` if `len_out` is not NULL.
///   Must be freed with `html_to_markdown_free_string()`.
/// - NULL: Conversion failed; call `html_to_markdown_last_error()` for details
///
/// # Safety
///
/// - `html` must be a valid null-terminated UTF-8 C string
/// - `visitor` must be a handle from `html_to_markdown_visitor_create()`
/// - `len_out` (if not NULL) must be a valid pointer to `size_t`
/// - Returned string must be freed with `html_to_markdown_free_string()`
/// - Visitor callbacks are invoked on the calling thread (must not block or panic)
///
/// # Errors
///
/// - HTML parsing errors (malformed HTML)
/// - Visitor callback returning Error result
/// - Memory allocation failures
/// - UTF-8 encoding errors
///
/// # Example
///
/// ```c
/// const char *html = "<h1>Title</h1><p>Content</p>";
/// void *visitor = html_to_markdown_visitor_create(&callbacks);
/// size_t out_len = 0;
/// char *md = html_to_markdown_convert_with_visitor(html, visitor, &out_len);
/// if (md == NULL) {
///     fprintf(stderr, "Failed: %s\n", html_to_markdown_last_error());
/// } else {
///     printf("Output length: %zu\n%s\n", out_len, md);
///     html_to_markdown_free_string(md);
/// }
/// html_to_markdown_visitor_free(visitor);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert_with_visitor(
    html: *const c_char,
    visitor: HtmlToMarkdownVisitor,
    len_out: *mut usize,
) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }

    if visitor.is_null() {
        set_last_error(Some("visitor handle was null".to_string()));
        return ptr::null_mut();
    }

    let html_str = if let Ok(s) = unsafe { CStr::from_ptr(html).to_str() } {
        s
    } else {
        set_last_error(Some("html must be valid UTF-8".to_string()));
        return ptr::null_mut();
    };

    let handle = unsafe { &*(visitor as *const Rc<RefCell<CVisitorWrapper>>) };

    match convert_with_visitor(html_str, None, Some(handle.clone())) {
        Ok(markdown) => {
            set_last_error(None);
            match string_to_c_string(markdown.clone(), "markdown result") {
                Ok(c_string) => {
                    if !len_out.is_null() {
                        unsafe { *len_out = markdown.len() };
                    }
                    c_string.into_raw()
                }
                Err(err) => {
                    set_last_error(Some(format!("failed to build CString for markdown result: {err}")));
                    ptr::null_mut()
                }
            }
        }
        Err(err) => {
            capture_error(err);
            ptr::null_mut()
        }
    }
}

/// Convert UTF-8 HTML bytes to Markdown using a custom visitor.
///
/// Variant of `html_to_markdown_convert_with_visitor()` that accepts raw byte pointers
/// instead of null-terminated C strings. Useful for data with embedded nulls or
/// when length is already known.
///
/// # Arguments
///
/// - `html`: Pointer to UTF-8 bytes (NOT null-terminated)
/// - `len`: Number of bytes pointed to by html
/// - `visitor`: Visitor handle from `html_to_markdown_visitor_create()`
/// - `len_out`: Pointer to `size_t` where output length will be written (can be NULL)
///
/// # Returns
///
/// - Non-NULL: Pointer to malloc'd markdown string (NULL-terminated).
///   Must be freed with `html_to_markdown_free_string()`.
/// - NULL: Conversion failed; call `html_to_markdown_last_error()` for details
///
/// # Safety
///
/// - `html` must point to at least `len` bytes of valid data
/// - Data must be valid UTF-8
/// - `len` must be accurate (not larger than allocated buffer)
/// - `visitor` must be a handle from `html_to_markdown_visitor_create()`
/// - `len_out` (if not NULL) must point to a valid `size_t`
///
/// # Example
///
/// ```c
/// const uint8_t *html_bytes = (const uint8_t *)input_data;
/// size_t html_len = 1024;
/// void *visitor = html_to_markdown_visitor_create(&callbacks);
/// size_t out_len = 0;
/// char *md = html_to_markdown_convert_bytes_with_visitor(html_bytes, html_len, visitor, &out_len);
/// if (md != NULL) {
///     printf("%s\n", md);
///     html_to_markdown_free_string(md);
/// }
/// html_to_markdown_visitor_free(visitor);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert_bytes_with_visitor(
    html: *const u8,
    len: usize,
    visitor: HtmlToMarkdownVisitor,
    len_out: *mut usize,
) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }

    if visitor.is_null() {
        set_last_error(Some("visitor handle was null".to_string()));
        return ptr::null_mut();
    }

    let html_bytes = unsafe { std::slice::from_raw_parts(html, len) };
    let html_str = if let Ok(s) = std::str::from_utf8(html_bytes) {
        s
    } else {
        set_last_error(Some("html must be valid UTF-8".to_string()));
        return ptr::null_mut();
    };

    let handle = unsafe { &*(visitor as *const Rc<RefCell<CVisitorWrapper>>) };

    match convert_with_visitor(html_str, None, Some(handle.clone())) {
        Ok(markdown) => {
            set_last_error(None);
            match string_to_c_string(markdown.clone(), "markdown result") {
                Ok(c_string) => {
                    if !len_out.is_null() {
                        unsafe { *len_out = markdown.len() };
                    }
                    c_string.into_raw()
                }
                Err(err) => {
                    set_last_error(Some(format!("failed to build CString for markdown result: {err}")));
                    ptr::null_mut()
                }
            }
        }
        Err(err) => {
            capture_error(err);
            ptr::null_mut()
        }
    }
}

/// Create a `VisitResult` with Continue action.
///
/// Helper function to construct a Continue result without custom output.
/// Equivalent to: `result = {0}; result.result_type = HTML_TO_MARKDOWN_VISIT_CONTINUE;`
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` with Continue action.
///
/// # Example
///
/// ```c
/// HtmlToMarkdownVisitResult result = html_to_markdown_visit_result_continue();
/// return result;
/// ```
#[unsafe(no_mangle)]
pub const extern "C" fn html_to_markdown_visit_result_continue() -> HtmlToMarkdownVisitResult {
    HtmlToMarkdownVisitResult {
        result_type: HtmlToMarkdownVisitResultType::Continue,
        custom_output: ptr::null_mut(),
        error_message: ptr::null_mut(),
    }
}

/// Create a `VisitResult` with Custom action.
///
/// Helper function to construct a Custom result. The `output` string should be
/// allocated with `malloc()` and will be freed by the FFI layer after use.
///
/// # Arguments
///
/// - `output`: malloc'd null-terminated C string with custom markdown
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` with Custom action.
///
/// # Safety
///
/// - `output` must be malloc'd (will be freed with `free()`)
/// - `output` must be NULL-terminated
///
/// # Example
///
/// ```c
/// char *custom = (char *)malloc(100);
/// snprintf(custom, 100, "Custom markdown here");
/// HtmlToMarkdownVisitResult result = html_to_markdown_visit_result_custom(custom);
/// return result;
/// ```
#[unsafe(no_mangle)]
pub const extern "C" fn html_to_markdown_visit_result_custom(output: *mut c_char) -> HtmlToMarkdownVisitResult {
    HtmlToMarkdownVisitResult {
        result_type: HtmlToMarkdownVisitResultType::Custom,
        custom_output: output,
        error_message: ptr::null_mut(),
    }
}

/// Create a `VisitResult` with Skip action.
///
/// Helper function to construct a Skip result (element and children omitted from output).
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` with Skip action.
///
/// # Example
///
/// ```c
/// HtmlToMarkdownVisitResult result = html_to_markdown_visit_result_skip();
/// return result;
/// ```
#[unsafe(no_mangle)]
pub const extern "C" fn html_to_markdown_visit_result_skip() -> HtmlToMarkdownVisitResult {
    HtmlToMarkdownVisitResult {
        result_type: HtmlToMarkdownVisitResultType::Skip,
        custom_output: ptr::null_mut(),
        error_message: ptr::null_mut(),
    }
}

/// Create a `VisitResult` with `PreserveHtml` action.
///
/// Helper function to construct a `PreserveHtml` result (raw HTML included verbatim).
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` with `PreserveHtml` action.
///
/// # Example
///
/// ```c
/// HtmlToMarkdownVisitResult result = html_to_markdown_visit_result_preserve_html();
/// return result;
/// ```
#[unsafe(no_mangle)]
pub const extern "C" fn html_to_markdown_visit_result_preserve_html() -> HtmlToMarkdownVisitResult {
    HtmlToMarkdownVisitResult {
        result_type: HtmlToMarkdownVisitResultType::PreserveHtml,
        custom_output: ptr::null_mut(),
        error_message: ptr::null_mut(),
    }
}

/// Create a `VisitResult` with Error action.
///
/// Helper function to construct an Error result. The `message` string should be
/// allocated with `malloc()` and will be freed by the FFI layer after use.
///
/// # Arguments
///
/// - `message`: malloc'd null-terminated C string with error message
///
/// # Returns
///
/// `HtmlToMarkdownVisitResult` with Error action.
///
/// # Safety
///
/// - `message` must be malloc'd (will be freed with `free()`)
/// - `message` must be NULL-terminated
///
/// # Example
///
/// ```c
/// char *error_msg = (char *)malloc(100);
/// snprintf(error_msg, 100, "Unexpected element type");
/// HtmlToMarkdownVisitResult result = html_to_markdown_visit_result_error(error_msg);
/// return result;
/// ```
#[unsafe(no_mangle)]
pub const extern "C" fn html_to_markdown_visit_result_error(message: *mut c_char) -> HtmlToMarkdownVisitResult {
    HtmlToMarkdownVisitResult {
        result_type: HtmlToMarkdownVisitResultType::Error,
        custom_output: ptr::null_mut(),
        error_message: message,
    }
}
