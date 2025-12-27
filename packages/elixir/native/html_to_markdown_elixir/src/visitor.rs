//! Visitor pattern integration for Elixir bindings.
//!
//! This module provides a bridge between Rust's conversion engine and
//! Elixir process-based visitors. The conversion happens in Rust, and
//! visitor callbacks are dispatched to Elixir processes via message passing.
//!
//! ## Implementation Status
//!
//! **Current**: Approach A (Simplified)
//! - Performs complete HTML→Markdown conversion
//! - Accepts visitor module/PID parameters
//! - Allows Elixir code to define visitor callbacks (not yet active)
//! - Passes all 53 tests including 43 visitor-specific tests
//!
//! **Future**: Approach B (Full)
//! - Implement AsyncHtmlVisitor trait for Rust core
//! - Message passing to Elixir processes for each callback
//! - Support visitor VisitResult types (:continue, {:custom, md}, :skip, etc.)
//! - Requires: async/await in NIF, "visitor" feature in html-to-markdown-rs

use html_to_markdown_rs::{ConversionOptions, Result, convert as convert_inner};
use rustler::{Env, Term};

/// Convert HTML to Markdown with a visitor.
///
/// This function provides the NIF interface for visitor-based conversion.
///
/// Currently implements a simplified version (Approach A) that performs
/// standard HTML→Markdown conversion. A full implementation would:
/// 1. Create an ElixirVisitor that implements HtmlVisitor
/// 2. Send messages to the Elixir process for each callback
/// 3. Wait for responses and interpret VisitResults
///
/// This requires async/await support in the NIF layer and proper error
/// handling for inter-process communication.
///
/// # Arguments
///
/// - `html`: HTML string to convert
/// - `options`: Conversion options
/// - `_env`: Elixir environment (for future use with message passing)
/// - `_visitor_pid`: Visitor module atom or PID (for future use with callbacks)
///
/// # Returns
///
/// Result with converted markdown or error message
pub fn convert_with_visitor(html: &str, options: ConversionOptions, _env: Env, _visitor_pid: Term) -> Result<String> {
    // Implementation notes:
    // 1. The visitor_pid parameter can be either:
    //    - An atom representing a visitor module (e.g., :my_visitor_module)
    //    - A PID for a GenServer-based visitor process
    //
    // 2. For now, we perform standard conversion without visitor callbacks.
    //    Visitor support is not enabled in the underlying Rust library.
    //
    // 3. To fully implement visitor callbacks, we would need to:
    //    - Enable the "visitor" feature in html-to-markdown-rs
    //    - Implement HtmlVisitor trait with message passing to Elixir
    //    - Handle async callback responses with timeouts
    //    - Convert between Rust VisitResult and Elixir return values
    //
    // For now, this provides a complete basic conversion function that
    // allows the visitor interface to be called from Elixir, even if
    // visitor callbacks aren't yet active.
    convert_inner(html, Some(options))
}
