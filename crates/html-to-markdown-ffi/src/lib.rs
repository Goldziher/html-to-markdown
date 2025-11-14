//! C FFI bindings for html-to-markdown.
//!
//! Provides a C-compatible API that can be consumed by Java (Panama FFM),
//! Go (cgo), C# (P/Invoke), Zig, and other languages with C FFI support.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use html_to_markdown_rs::convert;

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
        return ptr::null_mut();
    }

    // SAFETY: Caller must ensure html is a valid null-terminated C string
    let html_str = match unsafe { CStr::from_ptr(html) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match convert(html_str, None) {
        Ok(markdown) => match CString::new(markdown) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
    }
}

/// Get the last error message from a failed conversion.
///
/// # Safety
///
/// - Returns a static string that does not need to be freed
/// - May return NULL if no error has occurred
///
/// Note: This is a placeholder. Full error handling will be added in a future version.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_last_error() -> *const c_char {
    // TODO: Implement thread-local error storage
    ptr::null()
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
        // SAFETY: Caller must ensure s was returned by html_to_markdown_convert
        unsafe { drop(CString::from_raw(s)) };
    }
}

/// Get the library version string.
///
/// # Safety
///
/// - Returns a static string that does not need to be freed
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_basic_conversion() {
        unsafe {
            let html = CString::new("<h1>Hello World</h1>").unwrap();
            let result = html_to_markdown_convert(html.as_ptr());
            assert!(!result.is_null());

            let markdown = CStr::from_ptr(result).to_str().unwrap();
            assert!(markdown.contains("Hello World"));

            html_to_markdown_free_string(result);
        }
    }

    #[test]
    fn test_null_html() {
        unsafe {
            let result = html_to_markdown_convert(ptr::null());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_version() {
        unsafe {
            let version = html_to_markdown_version();
            assert!(!version.is_null());
            let version_str = CStr::from_ptr(version).to_str().unwrap();
            assert!(!version_str.is_empty());
        }
    }
}
