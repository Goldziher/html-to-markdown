//! Metadata extraction functions for C FFI with feature gate.
//!
//! This module provides HTML to Markdown conversion with metadata extraction,
//! requiring the `metadata` feature to be enabled.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::slice;

use html_to_markdown_rs::metadata::DEFAULT_MAX_STRUCTURED_DATA_SIZE;
use html_to_markdown_rs::safety::guard_panic;
use html_to_markdown_rs::{MetadataConfig, convert_with_metadata};

use crate::error::{capture_error, set_last_error};
use crate::profiling;
use crate::strings::{bytes_to_c_string, string_to_c_string};

/// Convert HTML to Markdown with metadata extraction.
///
/// # Safety
///
/// - `html` must be a valid null-terminated C string
/// - `metadata_json_out` must be a valid pointer to a char pointer
/// - The returned markdown string must be freed with `html_to_markdown_free_string`
/// - The metadata JSON string (written to metadata_json_out) must be freed with `html_to_markdown_free_string`
/// - Returns NULL on error (check error with `html_to_markdown_last_error`)
///
/// # Example (C)
///
/// ```c
/// const char* html = "<html><head><title>Test</title></head><body><h1>Hello</h1></body></html>";
/// char* metadata_json = NULL;
/// char* markdown = html_to_markdown_convert_with_metadata(html, &metadata_json);
/// if (markdown != NULL && metadata_json != NULL) {
///     printf("Markdown: %s\n", markdown);
///     printf("Metadata: %s\n", metadata_json);
///     html_to_markdown_free_string(markdown);
///     html_to_markdown_free_string(metadata_json);
/// }
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert_with_metadata(
    html: *const c_char,
    metadata_json_out: *mut *mut c_char,
) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }

    if metadata_json_out.is_null() {
        set_last_error(Some("metadata_json_out pointer was null".to_string()));
        return ptr::null_mut();
    }

    let html_str = match unsafe { CStr::from_ptr(html) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(Some("html must be valid UTF-8".to_string()));
            return ptr::null_mut();
        }
    };

    let metadata_cfg = MetadataConfig {
        extract_document: true,
        extract_headers: true,
        extract_links: true,
        extract_images: true,
        extract_structured_data: true,
        max_structured_data_size: DEFAULT_MAX_STRUCTURED_DATA_SIZE,
    };

    match guard_panic(|| profiling::maybe_profile(|| convert_with_metadata(html_str, None, metadata_cfg.clone()))) {
        Ok((markdown, metadata)) => {
            set_last_error(None);

            let metadata_json = match serde_json::to_vec(&metadata) {
                Ok(json) => json,
                Err(e) => {
                    set_last_error(Some(format!("failed to serialize metadata to JSON: {}", e)));
                    return ptr::null_mut();
                }
            };

            let metadata_c_string = match bytes_to_c_string(metadata_json, "metadata JSON") {
                Ok(s) => s,
                Err(err) => {
                    set_last_error(Some(format!("failed to build CString for metadata JSON: {err}")));
                    return ptr::null_mut();
                }
            };

            unsafe {
                *metadata_json_out = metadata_c_string.into_raw();
            }

            match string_to_c_string(markdown, "markdown result") {
                Ok(c_string) => c_string.into_raw(),
                Err(err) => {
                    set_last_error(Some(format!("failed to build CString for markdown result: {err}")));
                    unsafe {
                        if !metadata_json_out.is_null() && !(*metadata_json_out).is_null() {
                            drop(CString::from_raw(*metadata_json_out));
                            *metadata_json_out = ptr::null_mut();
                        }
                    }
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

/// Convert HTML to Markdown with metadata extraction, returning output lengths.
///
/// # Safety
///
/// - `html` must be a valid null-terminated C string
/// - `metadata_json_out` must be a valid pointer to a char pointer
/// - `markdown_len_out` and `metadata_len_out` must be valid pointers to size_t
/// - The returned markdown string must be freed with `html_to_markdown_free_string`
/// - The metadata JSON string (written to metadata_json_out) must be freed with `html_to_markdown_free_string`
/// - Returns NULL on error (check error with `html_to_markdown_last_error`)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert_with_metadata_with_len(
    html: *const c_char,
    metadata_json_out: *mut *mut c_char,
    markdown_len_out: *mut usize,
    metadata_len_out: *mut usize,
) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }

    if metadata_json_out.is_null() {
        set_last_error(Some("metadata_json_out pointer was null".to_string()));
        return ptr::null_mut();
    }

    if markdown_len_out.is_null() || metadata_len_out.is_null() {
        set_last_error(Some("length output pointer was null".to_string()));
        return ptr::null_mut();
    }

    let html_str = match unsafe { CStr::from_ptr(html) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(Some("html must be valid UTF-8".to_string()));
            return ptr::null_mut();
        }
    };

    let metadata_cfg = MetadataConfig {
        extract_document: true,
        extract_headers: true,
        extract_links: true,
        extract_images: true,
        extract_structured_data: true,
        max_structured_data_size: DEFAULT_MAX_STRUCTURED_DATA_SIZE,
    };

    match guard_panic(|| profiling::maybe_profile(|| convert_with_metadata(html_str, None, metadata_cfg.clone()))) {
        Ok((markdown, metadata)) => {
            set_last_error(None);

            let metadata_json = match serde_json::to_vec(&metadata) {
                Ok(json) => json,
                Err(e) => {
                    set_last_error(Some(format!("failed to serialize metadata to JSON: {}", e)));
                    return ptr::null_mut();
                }
            };

            let metadata_c_string = match bytes_to_c_string(metadata_json, "metadata JSON") {
                Ok(s) => s,
                Err(err) => {
                    set_last_error(Some(format!("failed to build CString for metadata JSON: {err}")));
                    return ptr::null_mut();
                }
            };

            unsafe {
                *metadata_len_out = metadata_c_string.as_bytes().len();
                *metadata_json_out = metadata_c_string.into_raw();
            }

            match string_to_c_string(markdown, "markdown result") {
                Ok(c_string) => {
                    unsafe {
                        *markdown_len_out = c_string.as_bytes().len();
                    }
                    c_string.into_raw()
                }
                Err(err) => {
                    set_last_error(Some(format!("failed to build CString for markdown result: {err}")));
                    unsafe {
                        if !metadata_json_out.is_null() && !(*metadata_json_out).is_null() {
                            drop(CString::from_raw(*metadata_json_out));
                            *metadata_json_out = ptr::null_mut();
                        }
                        if !metadata_len_out.is_null() {
                            *metadata_len_out = 0;
                        }
                    }
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

/// Convert UTF-8 HTML bytes to Markdown with metadata extraction and return output lengths.
///
/// # Safety
///
/// - `html` must point to `len` bytes of UTF-8 data
/// - `metadata_json_out` must be a valid pointer to a char pointer
/// - `markdown_len_out` and `metadata_len_out` must be valid pointers to size_t
/// - The returned markdown string must be freed with `html_to_markdown_free_string`
/// - The metadata JSON string (written to metadata_json_out) must be freed with `html_to_markdown_free_string`
/// - Returns NULL on error (check error with `html_to_markdown_last_error`)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert_with_metadata_bytes_with_len(
    html: *const u8,
    len: usize,
    metadata_json_out: *mut *mut c_char,
    markdown_len_out: *mut usize,
    metadata_len_out: *mut usize,
) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }

    if metadata_json_out.is_null() {
        set_last_error(Some("metadata_json_out pointer was null".to_string()));
        return ptr::null_mut();
    }

    if markdown_len_out.is_null() || metadata_len_out.is_null() {
        set_last_error(Some("length output pointer was null".to_string()));
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

    let metadata_cfg = MetadataConfig {
        extract_document: true,
        extract_headers: true,
        extract_links: true,
        extract_images: true,
        extract_structured_data: true,
        max_structured_data_size: DEFAULT_MAX_STRUCTURED_DATA_SIZE,
    };

    match guard_panic(|| profiling::maybe_profile(|| convert_with_metadata(html_str, None, metadata_cfg.clone()))) {
        Ok((markdown, metadata)) => {
            set_last_error(None);

            let metadata_json = match serde_json::to_vec(&metadata) {
                Ok(json) => json,
                Err(e) => {
                    set_last_error(Some(format!("failed to serialize metadata to JSON: {}", e)));
                    return ptr::null_mut();
                }
            };

            let metadata_c_string = match bytes_to_c_string(metadata_json, "metadata JSON") {
                Ok(s) => s,
                Err(err) => {
                    set_last_error(Some(format!("failed to build CString for metadata JSON: {err}")));
                    return ptr::null_mut();
                }
            };

            unsafe {
                *metadata_len_out = metadata_c_string.as_bytes().len();
                *metadata_json_out = metadata_c_string.into_raw();
            }

            match string_to_c_string(markdown, "markdown result") {
                Ok(c_string) => {
                    unsafe {
                        *markdown_len_out = c_string.as_bytes().len();
                    }
                    c_string.into_raw()
                }
                Err(err) => {
                    set_last_error(Some(format!("failed to build CString for markdown result: {err}")));
                    unsafe {
                        if !metadata_json_out.is_null() && !(*metadata_json_out).is_null() {
                            drop(CString::from_raw(*metadata_json_out));
                            *metadata_json_out = ptr::null_mut();
                        }
                        if !metadata_len_out.is_null() {
                            *metadata_len_out = 0;
                        }
                    }
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
