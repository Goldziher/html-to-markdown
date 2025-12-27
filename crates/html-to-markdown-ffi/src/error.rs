//! Error handling for C FFI.
//!
//! This module provides thread-local error storage and utilities for capturing
//! and reporting errors across the FFI boundary.

use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use html_to_markdown_rs::ConversionError;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

/// Set the thread-local last error message.
///
/// # Arguments
///
/// * `message` - Optional error message. If `None`, clears the error.
pub(crate) fn set_last_error(message: Option<String>) {
    LAST_ERROR.with(|cell| {
        let mut slot = cell.borrow_mut();
        *slot = message.and_then(|msg| CString::new(msg).ok());
    });
}

/// Get a pointer to the last error message.
///
/// # Safety
///
/// Returns a pointer to thread-local storage. The pointer is valid only until
/// the next call to any function that modifies `LAST_ERROR`.
pub(crate) fn last_error_ptr() -> *const c_char {
    LAST_ERROR.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|cstr| cstr.as_ptr() as *const c_char)
            .unwrap_or(ptr::null())
    })
}

/// Capture a `ConversionError` and store it in thread-local storage.
///
/// # Arguments
///
/// * `err` - The conversion error to capture
pub(crate) fn capture_error(err: ConversionError) {
    set_last_error(Some(err.to_string()));
}

/// Get the last error message from a failed conversion.
///
/// # Safety
///
/// - Returns a pointer to a thread-local buffer; copy it immediately if needed
/// - Pointer is invalidated by the next call to any `html_to_markdown_*` function
/// - May return NULL if no error has occurred in this thread
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_last_error() -> *const c_char {
    last_error_ptr()
}
