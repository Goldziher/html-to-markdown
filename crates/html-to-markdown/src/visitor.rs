//! Visitor pattern for HTML to Markdown conversion.
//!
//! This module provides a comprehensive visitor trait that allows users to intervene
//! in the HTML→Markdown transformation logic at any point. Visitors can inspect,
//! modify, or replace the default conversion behavior for any HTML element type.
//!
//! # Design Philosophy
//!
//! - **Flexibility over performance**: Visitors prioritize giving users full control
//! - **Zero-cost when unused**: No performance impact if visitor feature is disabled
//! - **Comprehensive coverage**: All 60+ HTML element types have dedicated visitor methods
//! - **Pre/post hooks**: Both element entry and exit points are exposed
//!
//! # Example
//!
//! ```ignore
//! use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
//!
//! struct CustomVisitor;
//!
//! impl HtmlVisitor for CustomVisitor {
//!     fn visit_link(&mut self, ctx: &NodeContext, href: &str, text: &str, title: Option<&str>) -> VisitResult {
//!         // Convert all links to plain text with URL in parentheses
//!         VisitResult::Custom(format!("{} ({})", text, href))
//!     }
//!
//!     fn visit_image(&mut self, ctx: &NodeContext, src: &str, alt: &str, title: Option<&str>) -> VisitResult {
//!         // Skip all images
//!         VisitResult::Skip
//!     }
//! }
//! ```

use std::collections::BTreeMap;

#[cfg(feature = "async-visitor")]
use async_trait::async_trait;

/// Node type enumeration covering all HTML element types.
///
/// This enum categorizes all HTML elements that the converter recognizes,
/// providing a coarse-grained classification for visitor dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Text node (most frequent - 100+ per document)
    Text,

    /// Generic element node
    Element,

    /// Heading elements (h1-h6)
    Heading,
    /// Paragraph element
    Paragraph,
    /// Generic div container
    Div,
    /// Blockquote element
    Blockquote,
    /// Preformatted text block
    Pre,
    /// Horizontal rule
    Hr,

    /// Ordered or unordered list (ul, ol)
    List,
    /// List item (li)
    ListItem,
    /// Definition list (dl)
    DefinitionList,
    /// Definition term (dt)
    DefinitionTerm,
    /// Definition description (dd)
    DefinitionDescription,

    /// Table element
    Table,
    /// Table row (tr)
    TableRow,
    /// Table cell (td, th)
    TableCell,
    /// Table header cell (th)
    TableHeader,
    /// Table body (tbody)
    TableBody,
    /// Table head (thead)
    TableHead,
    /// Table foot (tfoot)
    TableFoot,

    /// Anchor link (a)
    Link,
    /// Image (img)
    Image,
    /// Strong/bold (strong, b)
    Strong,
    /// Emphasis/italic (em, i)
    Em,
    /// Inline code (code)
    Code,
    /// Strikethrough (s, del, strike)
    Strikethrough,
    /// Underline (u, ins)
    Underline,
    /// Subscript (sub)
    Subscript,
    /// Superscript (sup)
    Superscript,
    /// Mark/highlight (mark)
    Mark,
    /// Small text (small)
    Small,
    /// Line break (br)
    Br,
    /// Span element
    Span,

    /// Article element
    Article,
    /// Section element
    Section,
    /// Navigation element
    Nav,
    /// Aside element
    Aside,
    /// Header element
    Header,
    /// Footer element
    Footer,
    /// Main element
    Main,
    /// Figure element
    Figure,
    /// Figure caption
    Figcaption,
    /// Time element
    Time,
    /// Details element
    Details,
    /// Summary element
    Summary,

    /// Form element
    Form,
    /// Input element
    Input,
    /// Select element
    Select,
    /// Option element
    Option,
    /// Button element
    Button,
    /// Textarea element
    Textarea,
    /// Label element
    Label,
    /// Fieldset element
    Fieldset,
    /// Legend element
    Legend,

    /// Audio element
    Audio,
    /// Video element
    Video,
    /// Picture element
    Picture,
    /// Source element
    Source,
    /// Iframe element
    Iframe,
    /// SVG element
    Svg,
    /// Canvas element
    Canvas,

    /// Ruby annotation
    Ruby,
    /// Ruby text
    Rt,
    /// Ruby parenthesis
    Rp,
    /// Abbreviation
    Abbr,
    /// Keyboard input
    Kbd,
    /// Sample output
    Samp,
    /// Variable
    Var,
    /// Citation
    Cite,
    /// Quote
    Q,
    /// Deleted text
    Del,
    /// Inserted text
    Ins,
    /// Data element
    Data,
    /// Meter element
    Meter,
    /// Progress element
    Progress,
    /// Output element
    Output,
    /// Template element
    Template,
    /// Slot element
    Slot,

    /// HTML root element
    Html,
    /// Head element
    Head,
    /// Body element
    Body,
    /// Title element
    Title,
    /// Meta element
    Meta,
    /// Link element (not anchor)
    LinkTag,
    /// Style element
    Style,
    /// Script element
    Script,
    /// Base element
    Base,

    /// Custom element (web components) or unknown tag
    Custom,
}

/// Context information passed to all visitor methods.
///
/// Provides comprehensive metadata about the current node being visited,
/// including its type, attributes, position in the DOM tree, and parent context.
#[derive(Debug, Clone)]
pub struct NodeContext {
    /// Coarse-grained node type classification
    pub node_type: NodeType,

    /// Raw HTML tag name (e.g., "div", "h1", "custom-element")
    pub tag_name: String,

    /// All HTML attributes as key-value pairs
    pub attributes: BTreeMap<String, String>,

    /// Depth in the DOM tree (0 = root)
    pub depth: usize,

    /// Index among siblings (0-based)
    pub index_in_parent: usize,

    /// Parent element's tag name (None if root)
    pub parent_tag: Option<String>,

    /// Whether this element is treated as inline vs block
    pub is_inline: bool,
}

/// Result of a visitor callback.
///
/// Allows visitors to control the conversion flow by either proceeding
/// with default behavior, providing custom output, skipping elements,
/// preserving HTML, or signaling errors.
#[derive(Debug, Clone)]
pub enum VisitResult {
    /// Continue with default conversion behavior
    Continue,

    /// Replace default output with custom markdown
    ///
    /// The visitor takes full responsibility for the markdown output
    /// of this node and its children.
    Custom(String),

    /// Skip this element entirely (don't output anything)
    ///
    /// The element and all its children are ignored in the output.
    Skip,

    /// Preserve original HTML (don't convert to markdown)
    ///
    /// The element's raw HTML is included verbatim in the output.
    PreserveHtml,

    /// Stop conversion with an error
    ///
    /// The conversion process halts and returns this error message.
    Error(String),
}

/// Type alias for a visitor handle (Rc-wrapped RefCell for interior mutability).
///
/// This allows visitors to be passed around and shared while still being mutable.
pub type VisitorHandle = std::rc::Rc<std::cell::RefCell<dyn HtmlVisitor>>;

/// Visitor trait for HTML→Markdown conversion.
///
/// Implement this trait to customize the conversion behavior for any HTML element type.
/// All methods have default implementations that return `VisitResult::Continue`, allowing
/// selective override of only the elements you care about.
///
/// # Method Naming Convention
///
/// - `visit_*_start`: Called before entering an element (pre-order traversal)
/// - `visit_*_end`: Called after exiting an element (post-order traversal)
/// - `visit_*`: Called for specific element types (e.g., `visit_link`, `visit_image`)
///
/// # Execution Order
///
/// For a typical element like `<div><p>text</p></div>`:
/// 1. `visit_element_start` for `<div>`
/// 2. `visit_element_start` for `<p>`
/// 3. `visit_text` for "text"
/// 4. `visit_element_end` for `<p>`
/// 5. `visit_element_end` for `</div>`
///
/// # Performance Notes
///
/// - `visit_text` is the most frequently called method (~100+ times per document)
/// - Return `VisitResult::Continue` quickly for elements you don't need to customize
/// - Avoid heavy computation in visitor methods; consider caching if needed
pub trait HtmlVisitor: std::fmt::Debug {
    /// Called before entering any element.
    ///
    /// This is the first callback invoked for every HTML element, allowing
    /// visitors to implement generic element handling before tag-specific logic.
    fn visit_element_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after exiting any element.
    ///
    /// Receives the default markdown output that would be generated.
    /// Visitors can inspect or replace this output.
    fn visit_element_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit text nodes (most frequent callback - ~100+ per document).
    ///
    /// # Arguments
    /// - `ctx`: Node context (will have `node_type: NodeType::Text`)
    /// - `text`: The raw text content (HTML entities already decoded)
    fn visit_text(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit anchor links `<a href="...">`.
    ///
    /// # Arguments
    /// - `ctx`: Node context with link element metadata
    /// - `href`: The link URL (from `href` attribute)
    /// - `text`: The link text content (already converted to markdown)
    /// - `title`: Optional title attribute
    fn visit_link(&mut self, _ctx: &NodeContext, _href: &str, _text: &str, _title: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit images `<img src="...">`.
    ///
    /// # Arguments
    /// - `ctx`: Node context with image element metadata
    /// - `src`: The image source URL
    /// - `alt`: The alt text
    /// - `title`: Optional title attribute
    fn visit_image(&mut self, _ctx: &NodeContext, _src: &str, _alt: &str, _title: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit heading elements `<h1>` through `<h6>`.
    ///
    /// # Arguments
    /// - `ctx`: Node context with heading metadata
    /// - `level`: Heading level (1-6)
    /// - `text`: The heading text content
    /// - `id`: Optional id attribute (for anchor links)
    fn visit_heading(&mut self, _ctx: &NodeContext, _level: u32, _text: &str, _id: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit code blocks `<pre><code>`.
    ///
    /// # Arguments
    /// - `ctx`: Node context
    /// - `lang`: Optional language specifier (from class attribute)
    /// - `code`: The code content
    fn visit_code_block(&mut self, _ctx: &NodeContext, _lang: Option<&str>, _code: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit inline code `<code>`.
    ///
    /// # Arguments
    /// - `ctx`: Node context
    /// - `code`: The code content
    fn visit_code_inline(&mut self, _ctx: &NodeContext, _code: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit list items `<li>`.
    ///
    /// # Arguments
    /// - `ctx`: Node context
    /// - `ordered`: Whether this is an ordered list item
    /// - `marker`: The list marker (e.g., "-", "1.", "a)")
    /// - `text`: The list item content (already converted)
    fn visit_list_item(&mut self, _ctx: &NodeContext, _ordered: bool, _marker: &str, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Called before processing a list `<ul>` or `<ol>`.
    fn visit_list_start(&mut self, _ctx: &NodeContext, _ordered: bool) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after processing a list `</ul>` or `</ol>`.
    fn visit_list_end(&mut self, _ctx: &NodeContext, _ordered: bool, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Called before processing a table `<table>`.
    fn visit_table_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit table rows `<tr>`.
    ///
    /// # Arguments
    /// - `ctx`: Node context
    /// - `cells`: Cell contents (already converted to markdown)
    /// - `is_header`: Whether this row is in `<thead>`
    fn visit_table_row(&mut self, _ctx: &NodeContext, _cells: &[String], _is_header: bool) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after processing a table `</table>`.
    fn visit_table_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit blockquote elements `<blockquote>`.
    ///
    /// # Arguments
    /// - `ctx`: Node context
    /// - `content`: The blockquote content (already converted)
    /// - `depth`: Nesting depth (for nested blockquotes)
    fn visit_blockquote(&mut self, _ctx: &NodeContext, _content: &str, _depth: usize) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit strong/bold elements `<strong>`, `<b>`.
    fn visit_strong(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit emphasis/italic elements `<em>`, `<i>`.
    fn visit_emphasis(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit strikethrough elements `<s>`, `<del>`, `<strike>`.
    fn visit_strikethrough(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit underline elements `<u>`, `<ins>`.
    fn visit_underline(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit subscript elements `<sub>`.
    fn visit_subscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit superscript elements `<sup>`.
    fn visit_superscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit mark/highlight elements `<mark>`.
    fn visit_mark(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit line break elements `<br>`.
    fn visit_line_break(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit horizontal rule elements `<hr>`.
    fn visit_horizontal_rule(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit custom elements (web components) or unknown tags.
    ///
    /// # Arguments
    /// - `ctx`: Node context
    /// - `tag_name`: The custom element's tag name
    /// - `html`: The raw HTML of this element
    fn visit_custom_element(&mut self, _ctx: &NodeContext, _tag_name: &str, _html: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit definition list `<dl>`.
    fn visit_definition_list_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit definition term `<dt>`.
    fn visit_definition_term(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit definition description `<dd>`.
    fn visit_definition_description(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after processing a definition list `</dl>`.
    fn visit_definition_list_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit form elements `<form>`.
    fn visit_form(&mut self, _ctx: &NodeContext, _action: Option<&str>, _method: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit input elements `<input>`.
    fn visit_input(
        &mut self,
        _ctx: &NodeContext,
        _input_type: &str,
        _name: Option<&str>,
        _value: Option<&str>,
    ) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit button elements `<button>`.
    fn visit_button(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit audio elements `<audio>`.
    fn visit_audio(&mut self, _ctx: &NodeContext, _src: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit video elements `<video>`.
    fn visit_video(&mut self, _ctx: &NodeContext, _src: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit iframe elements `<iframe>`.
    fn visit_iframe(&mut self, _ctx: &NodeContext, _src: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit details elements `<details>`.
    fn visit_details(&mut self, _ctx: &NodeContext, _open: bool) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit summary elements `<summary>`.
    fn visit_summary(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit figure elements `<figure>`.
    fn visit_figure_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit figcaption elements `<figcaption>`.
    fn visit_figcaption(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after processing a figure `</figure>`.
    fn visit_figure_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }
}

/// Async visitor trait for HTML→Markdown conversion.
///
/// This trait is identical to `HtmlVisitor` but all methods are async. Use this for languages
/// with native async/await support:
/// - Python (with `async def` and `asyncio`)
/// - TypeScript/JavaScript (with `Promise`-based callbacks)
/// - Elixir (with message-passing processes)
///
/// For synchronous languages (Ruby, PHP, Go, Java, C#), use the sync `HtmlVisitor` trait.
///
/// # Example (Python-like)
///
/// ```ignore
/// use html_to_markdown_rs::visitor::{AsyncHtmlVisitor, NodeContext, VisitResult};
///
/// struct CustomAsyncVisitor;
///
/// #[async_trait::async_trait]
/// impl AsyncHtmlVisitor for CustomAsyncVisitor {
///     async fn visit_link(
///         &mut self,
///         ctx: &NodeContext,
///         href: &str,
///         text: &str,
///         title: Option<&str>,
///     ) -> VisitResult {
///         // Can await async operations here
///         VisitResult::Custom(format!("{} ({})", text, href))
///     }
/// }
/// ```
#[cfg(feature = "async-visitor")]
#[async_trait]
pub trait AsyncHtmlVisitor: std::fmt::Debug + Send + Sync {
    /// Called before entering any element (async version).
    async fn visit_element_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after exiting any element (async version).
    async fn visit_element_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit text nodes (async version - most frequent callback - ~100+ per document).
    async fn visit_text(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit anchor links `<a href="...">` (async version).
    async fn visit_link(&mut self, _ctx: &NodeContext, _href: &str, _text: &str, _title: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit images `<img src="...">` (async version).
    async fn visit_image(&mut self, _ctx: &NodeContext, _src: &str, _alt: &str, _title: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit heading elements `<h1>` through `<h6>` (async version).
    async fn visit_heading(&mut self, _ctx: &NodeContext, _level: u32, _text: &str, _id: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit code blocks `<pre><code>` (async version).
    async fn visit_code_block(&mut self, _ctx: &NodeContext, _lang: Option<&str>, _code: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit inline code `<code>` (async version).
    async fn visit_code_inline(&mut self, _ctx: &NodeContext, _code: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit list items `<li>` (async version).
    async fn visit_list_item(&mut self, _ctx: &NodeContext, _ordered: bool, _marker: &str, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Called before processing a list `<ul>` or `<ol>` (async version).
    async fn visit_list_start(&mut self, _ctx: &NodeContext, _ordered: bool) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after processing a list `</ul>` or `</ol>` (async version).
    async fn visit_list_end(&mut self, _ctx: &NodeContext, _ordered: bool, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Called before processing a table `<table>` (async version).
    async fn visit_table_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit table rows `<tr>` (async version).
    async fn visit_table_row(&mut self, _ctx: &NodeContext, _cells: &[String], _is_header: bool) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after processing a table `</table>` (async version).
    async fn visit_table_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit blockquote elements `<blockquote>` (async version).
    async fn visit_blockquote(&mut self, _ctx: &NodeContext, _content: &str, _depth: usize) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit strong/bold elements `<strong>`, `<b>` (async version).
    async fn visit_strong(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit emphasis/italic elements `<em>`, `<i>` (async version).
    async fn visit_emphasis(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit strikethrough elements `<s>`, `<del>`, `<strike>` (async version).
    async fn visit_strikethrough(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit underline elements `<u>`, `<ins>` (async version).
    async fn visit_underline(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit subscript elements `<sub>` (async version).
    async fn visit_subscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit superscript elements `<sup>` (async version).
    async fn visit_superscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit mark/highlight elements `<mark>` (async version).
    async fn visit_mark(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit line break elements `<br>` (async version).
    async fn visit_line_break(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit horizontal rule elements `<hr>` (async version).
    async fn visit_horizontal_rule(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit custom elements (web components) or unknown tags (async version).
    async fn visit_custom_element(&mut self, _ctx: &NodeContext, _tag_name: &str, _html: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit definition list `<dl>` (async version).
    async fn visit_definition_list_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit definition term `<dt>` (async version).
    async fn visit_definition_term(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit definition description `<dd>` (async version).
    async fn visit_definition_description(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after processing a definition list `</dl>` (async version).
    async fn visit_definition_list_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit form elements `<form>` (async version).
    async fn visit_form(&mut self, _ctx: &NodeContext, _action: Option<&str>, _method: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit input elements `<input>` (async version).
    async fn visit_input(
        &mut self,
        _ctx: &NodeContext,
        _input_type: &str,
        _name: Option<&str>,
        _value: Option<&str>,
    ) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit button elements `<button>` (async version).
    async fn visit_button(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit audio elements `<audio>` (async version).
    async fn visit_audio(&mut self, _ctx: &NodeContext, _src: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit video elements `<video>` (async version).
    async fn visit_video(&mut self, _ctx: &NodeContext, _src: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit iframe elements `<iframe>` (async version).
    async fn visit_iframe(&mut self, _ctx: &NodeContext, _src: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit details elements `<details>` (async version).
    async fn visit_details(&mut self, _ctx: &NodeContext, _open: bool) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit summary elements `<summary>` (async version).
    async fn visit_summary(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit figure elements `<figure>` (async version).
    async fn visit_figure_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit figcaption elements `<figcaption>` (async version).
    async fn visit_figcaption(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    /// Called after processing a figure `</figure>` (async version).
    async fn visit_figure_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_equality() {
        assert_eq!(NodeType::Text, NodeType::Text);
        assert_ne!(NodeType::Text, NodeType::Element);
        assert_eq!(NodeType::Heading, NodeType::Heading);
    }

    #[test]
    fn test_node_context_creation() {
        let ctx = NodeContext {
            node_type: NodeType::Heading,
            tag_name: "h1".to_string(),
            attributes: BTreeMap::new(),
            depth: 1,
            index_in_parent: 0,
            parent_tag: Some("body".to_string()),
            is_inline: false,
        };

        assert_eq!(ctx.node_type, NodeType::Heading);
        assert_eq!(ctx.tag_name, "h1");
        assert_eq!(ctx.depth, 1);
        assert!(!ctx.is_inline);
    }

    #[test]
    fn test_visit_result_variants() {
        let continue_result = VisitResult::Continue;
        matches!(continue_result, VisitResult::Continue);

        let custom_result = VisitResult::Custom("# Custom Output".to_string());
        if let VisitResult::Custom(output) = custom_result {
            assert_eq!(output, "# Custom Output");
        }

        let skip_result = VisitResult::Skip;
        matches!(skip_result, VisitResult::Skip);

        let preserve_result = VisitResult::PreserveHtml;
        matches!(preserve_result, VisitResult::PreserveHtml);

        let error_result = VisitResult::Error("Test error".to_string());
        if let VisitResult::Error(msg) = error_result {
            assert_eq!(msg, "Test error");
        }
    }

    #[derive(Debug)]
    struct NoOpVisitor;

    impl HtmlVisitor for NoOpVisitor {}

    #[test]
    fn test_default_visitor_implementation() {
        let mut visitor = NoOpVisitor;

        let ctx = NodeContext {
            node_type: NodeType::Text,
            tag_name: String::new(),
            attributes: BTreeMap::new(),
            depth: 0,
            index_in_parent: 0,
            parent_tag: None,
            is_inline: true,
        };

        matches!(visitor.visit_element_start(&ctx), VisitResult::Continue);
        matches!(visitor.visit_element_end(&ctx, "output"), VisitResult::Continue);
        matches!(visitor.visit_text(&ctx, "text"), VisitResult::Continue);
        matches!(visitor.visit_link(&ctx, "href", "text", None), VisitResult::Continue);
        matches!(visitor.visit_image(&ctx, "src", "alt", None), VisitResult::Continue);
        matches!(visitor.visit_heading(&ctx, 1, "text", None), VisitResult::Continue);
        matches!(visitor.visit_code_block(&ctx, None, "code"), VisitResult::Continue);
        matches!(visitor.visit_code_inline(&ctx, "code"), VisitResult::Continue);
    }

    #[derive(Debug)]
    struct CustomLinkVisitor;

    impl HtmlVisitor for CustomLinkVisitor {
        fn visit_link(&mut self, _ctx: &NodeContext, href: &str, text: &str, _title: Option<&str>) -> VisitResult {
            VisitResult::Custom(format!("{} ({})", text, href))
        }

        fn visit_image(&mut self, _ctx: &NodeContext, _src: &str, _alt: &str, _title: Option<&str>) -> VisitResult {
            VisitResult::Skip
        }
    }

    #[test]
    fn test_custom_visitor_implementation() {
        let mut visitor = CustomLinkVisitor;

        let ctx = NodeContext {
            node_type: NodeType::Link,
            tag_name: "a".to_string(),
            attributes: BTreeMap::new(),
            depth: 1,
            index_in_parent: 0,
            parent_tag: Some("p".to_string()),
            is_inline: true,
        };

        let result = visitor.visit_link(&ctx, "https://example.com", "Example", None);
        if let VisitResult::Custom(output) = result {
            assert_eq!(output, "Example (https://example.com)");
        } else {
            panic!("Expected Custom result");
        }

        let img_result = visitor.visit_image(&ctx, "image.jpg", "Alt text", None);
        matches!(img_result, VisitResult::Skip);
    }
}
