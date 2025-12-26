//! Basic HTML to Markdown conversion functions for C FFI.
//!
//! This module provides simple conversion functions without additional features
//! like metadata extraction or callbacks.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::slice;

use html_to_markdown_rs::convert;
use html_to_markdown_rs::safety::guard_panic;

use crate::error::{capture_error, set_last_error};
use crate::profiling;
use crate::strings::string_to_c_string;

/// Convert HTML to Markdown using default options.
///
/// # Safety
///
/// - `html` must be a valid null-terminated C string
/// - The returned string must be freed with `html_to_markdown_free_string`
/// - Returns NULL on error
///
/// # Example (C)
///
/// ```c
/// const char* html = "<h1>Hello</h1>";
/// char* markdown = html_to_markdown_convert(html);
/// if (markdown != NULL) {
///     printf("%s\n", markdown);
///     html_to_markdown_free_string(markdown);
/// }
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert(html: *const c_char) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }

    let html_str = match unsafe { CStr::from_ptr(html) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(Some("html must be valid UTF-8".to_string()));
            return ptr::null_mut();
        }
    };

    match guard_panic(|| profiling::maybe_profile(|| convert(html_str, None))) {
        Ok(markdown) => {
            set_last_error(None);
            match string_to_c_string(markdown, "markdown result") {
                Ok(c_string) => c_string.into_raw(),
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

/// Convert HTML to Markdown using default options, returning the output length.
///
/// # Safety
///
/// - `html` must be a valid null-terminated C string
/// - `len_out` must be a valid pointer to a size_t
/// - The returned string must be freed with `html_to_markdown_free_string`
/// - Returns NULL on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert_with_len(html: *const c_char, len_out: *mut usize) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }

    if len_out.is_null() {
        set_last_error(Some("len_out pointer was null".to_string()));
        return ptr::null_mut();
    }

    let html_str = match unsafe { CStr::from_ptr(html) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(Some("html must be valid UTF-8".to_string()));
            return ptr::null_mut();
        }
    };

    match guard_panic(|| profiling::maybe_profile(|| convert(html_str, None))) {
        Ok(markdown) => {
            set_last_error(None);
            match string_to_c_string(markdown, "markdown result") {
                Ok(c_string) => {
                    unsafe {
                        *len_out = c_string.as_bytes().len();
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

/// Convert UTF-8 HTML bytes to Markdown and return the output length.
///
/// # Safety
///
/// - `html` must point to `len` bytes of UTF-8 data
/// - `len_out` must be a valid pointer to a size_t
/// - The returned string must be freed with `html_to_markdown_free_string`
/// - Returns NULL on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert_bytes_with_len(
    html: *const u8,
    len: usize,
    len_out: *mut usize,
) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }

    if len_out.is_null() {
        set_last_error(Some("len_out pointer was null".to_string()));
        return ptr::null_mut();
    }

    let html_bytes = unsafe { slice::from_raw_parts(html, len) };
    let html_str = match std::str::from_utf8(html_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(Some("html must be valid UTF-8".to_string()));
            return ptr::null_mut();
        }
    };

    match guard_panic(|| profiling::maybe_profile(|| convert(html_str, None))) {
        Ok(markdown) => {
            set_last_error(None);
            match string_to_c_string(markdown, "markdown result") {
                Ok(c_string) => {
                    unsafe {
                        *len_out = c_string.as_bytes().len();
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

/// Free a string returned by html_to_markdown_convert.
///
/// # Safety
///
/// - `s` must be a string previously returned by `html_to_markdown_convert`
/// - `s` must not be NULL
/// - `s` must not be used after this call
///
/// # Example (C)
///
/// ```c
/// char* markdown = html_to_markdown_convert("<p>text</p>");
/// html_to_markdown_free_string(markdown);
/// // markdown is now invalid
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}
