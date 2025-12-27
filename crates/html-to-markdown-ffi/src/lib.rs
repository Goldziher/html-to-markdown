//! C FFI bindings for html-to-markdown.
//!
//! Provides a C-compatible API that can be consumed by Java (Panama FFM),
//! Go (cgo), C# (P/Invoke), Zig, and other languages with C FFI support.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::slice;

use html_to_markdown_rs::convert;
use html_to_markdown_rs::safety::guard_panic;

#[cfg(feature = "metadata")]
use html_to_markdown_rs::{MetadataConfig, convert_with_metadata, metadata::DEFAULT_MAX_STRUCTURED_DATA_SIZE};
mod error;
mod profiling;
mod strings;
pub mod visitor;

pub use error::html_to_markdown_last_error;
use error::{capture_error, set_last_error};

#[allow(dead_code)]
fn bytes_to_c_string(mut bytes: Vec<u8>, context: &str) -> Result<CString, String> {
    if bytes.contains(&0) {
        return Err(format!("{context} contained an interior null byte"));
    }

    bytes.reserve(1);
    bytes.push(0);

    Ok(unsafe { CString::from_vec_unchecked(bytes) })
}

#[allow(dead_code)]
fn string_to_c_string(value: String, context: &str) -> Result<CString, String> {
    bytes_to_c_string(value.into_bytes(), context)
}

/// Start Rust-side profiling and write a flamegraph to the specified path.
///
/// Returns 1 on success, 0 on failure. Use `html_to_markdown_last_error` to inspect failures.
///
/// # Safety
///
/// - `output` must be a valid, null-terminated UTF-8 C string for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_profile_start(output: *const c_char, frequency: i32) -> bool {
    if output.is_null() {
        set_last_error(Some("output path was null".to_string()));
        return false;
    }

    let output_str = match unsafe { CStr::from_ptr(output) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(Some("output path must be valid UTF-8".to_string()));
            return false;
        }
    };

    match profiling::start(output_str.into(), frequency) {
        Ok(()) => {
            set_last_error(None);
            true
        }
        Err(err) => {
            capture_error(err);
            false
        }
    }
}

/// Stop Rust-side profiling and flush the flamegraph.
///
/// Returns 1 on success, 0 on failure. Use `html_to_markdown_last_error` to inspect failures.
///
/// # Safety
///
/// - This must only be called after a successful `html_to_markdown_profile_start`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_profile_stop() -> bool {
    match profiling::stop() {
        Ok(()) => {
            set_last_error(None);
            true
        }
        Err(err) => {
            capture_error(err);
            false
        }
    }
}

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

/// Get the library version string.
///
/// # Safety
///
/// - Returns a static string that does not need to be freed
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

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
#[cfg(feature = "metadata")]
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
                            html_to_markdown_free_string(*metadata_json_out);
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
#[cfg(feature = "metadata")]
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
                            html_to_markdown_free_string(*metadata_json_out);
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
#[cfg(feature = "metadata")]
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
                            html_to_markdown_free_string(*metadata_json_out);
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
            let err = html_to_markdown_last_error();
            assert!(!err.is_null());
            let msg = CStr::from_ptr(err).to_str().unwrap();
            assert_eq!(msg, "html pointer was null");
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

    #[test]
    fn test_last_error_clears_after_success() {
        unsafe {
            let _ = html_to_markdown_convert(ptr::null());
            let err = html_to_markdown_last_error();
            assert!(!err.is_null());

            let html = CString::new("<p>ok</p>").unwrap();
            let result = html_to_markdown_convert(html.as_ptr());
            assert!(!result.is_null());
            html_to_markdown_free_string(result);

            let cleared = html_to_markdown_last_error();
            assert!(cleared.is_null());
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_basic() {
        unsafe {
            let html =
                CString::new("<html><head><title>Test Page</title></head><body><h1>Hello World</h1></body></html>")
                    .unwrap();
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let result = html_to_markdown_convert_with_metadata(html.as_ptr(), &mut metadata_json);

            assert!(!result.is_null(), "Markdown result should not be null");
            assert!(!metadata_json.is_null(), "Metadata JSON should not be null");

            let markdown = CStr::from_ptr(result).to_str().unwrap();
            assert!(markdown.contains("Hello World"));

            let metadata = CStr::from_ptr(metadata_json).to_str().unwrap();
            assert!(metadata.contains("\"title\""));
            assert!(metadata.contains("Test Page"));

            html_to_markdown_free_string(result);
            html_to_markdown_free_string(metadata_json);
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_json_structure() {
        unsafe {
            let html = CString::new("<html lang=\"en\"><head><title>Test</title></head><body><h1 id=\"heading\">Title</h1><a href=\"https://example.com\">Link</a></body></html>").unwrap();
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let result = html_to_markdown_convert_with_metadata(html.as_ptr(), &mut metadata_json);

            assert!(!result.is_null());
            assert!(!metadata_json.is_null());

            let metadata_str = CStr::from_ptr(metadata_json).to_str().unwrap();

            assert!(metadata_str.contains("\"document\""), "Should have document field");
            assert!(metadata_str.contains("\"headers\""), "Should have headers field");
            assert!(metadata_str.contains("\"links\""), "Should have links field");
            assert!(metadata_str.contains("\"language\":\"en\""), "Should have language");
            assert!(metadata_str.contains("\"level\":1"), "Should have header level");

            html_to_markdown_free_string(result);
            html_to_markdown_free_string(metadata_json);
        }
    }

    #[test]
    fn test_convert_with_len_reports_length() {
        unsafe {
            let html = CString::new("<p>hello</p>").unwrap();
            let mut len: usize = 0;
            let result = html_to_markdown_convert_with_len(html.as_ptr(), &mut len);
            assert!(!result.is_null());
            assert!(len > 0);
            html_to_markdown_free_string(result);
        }
    }

    #[test]
    fn test_convert_bytes_with_len_reports_length() {
        unsafe {
            let html = b"<p>hello</p>";
            let mut len: usize = 0;
            let result = html_to_markdown_convert_bytes_with_len(html.as_ptr(), html.len(), &mut len);
            assert!(!result.is_null());
            assert!(len > 0);
            html_to_markdown_free_string(result);
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_null_html() {
        unsafe {
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let result = html_to_markdown_convert_with_metadata(ptr::null(), &mut metadata_json);

            assert!(result.is_null());
            let err = html_to_markdown_last_error();
            assert!(!err.is_null());
            let msg = CStr::from_ptr(err).to_str().unwrap();
            assert_eq!(msg, "html pointer was null");
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_null_output_ptr() {
        unsafe {
            let html = CString::new("<p>test</p>").unwrap();
            let result = html_to_markdown_convert_with_metadata(html.as_ptr(), ptr::null_mut());

            assert!(result.is_null());
            let err = html_to_markdown_last_error();
            assert!(!err.is_null());
            let msg = CStr::from_ptr(err).to_str().unwrap();
            assert_eq!(msg, "metadata_json_out pointer was null");
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_empty_html() {
        unsafe {
            let html = CString::new("").unwrap();
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let result = html_to_markdown_convert_with_metadata(html.as_ptr(), &mut metadata_json);

            assert!(!result.is_null());
            assert!(!metadata_json.is_null());

            html_to_markdown_free_string(result);
            html_to_markdown_free_string(metadata_json);
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_complex_document() {
        unsafe {
            let html = CString::new(
                r#"
                <html>
                <head>
                    <title>Complex Document</title>
                    <meta name="description" content="Test description">
                    <meta name="author" content="Test Author">
                </head>
                <body>
                    <h1>Main Title</h1>
                    <h2>Subtitle</h2>
                    <p>Content with <a href="https://example.com">external link</a></p>
                    <p>And <a href="/internal">internal link</a></p>
                    <img src="https://example.com/image.png" alt="Test Image">
                </body>
                </html>
            "#,
            )
            .unwrap();
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let result = html_to_markdown_convert_with_metadata(html.as_ptr(), &mut metadata_json);

            assert!(!result.is_null());
            assert!(!metadata_json.is_null());

            let metadata_str = CStr::from_ptr(metadata_json).to_str().unwrap();

            assert!(metadata_str.contains("Complex Document"));
            assert!(metadata_str.contains("Test description"));
            assert!(metadata_str.contains("Test Author"));
            assert!(metadata_str.contains("Main Title"));
            assert!(metadata_str.contains("Subtitle"));
            assert!(metadata_str.contains("https://example.com"));
            assert!(metadata_str.contains("Test Image"));

            html_to_markdown_free_string(result);
            html_to_markdown_free_string(metadata_json);
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_error_clears_both_pointers() {
        unsafe {
            let mut metadata_json: *mut c_char = ptr::null_mut();

            let result = html_to_markdown_convert_with_metadata(ptr::null(), &mut metadata_json);
            assert!(result.is_null(), "Markdown pointer should be null on error");
            assert!(metadata_json.is_null(), "Metadata pointer should remain null on error");

            let html = CString::new("<p>test</p>").unwrap();
            let result2 = html_to_markdown_convert_with_metadata(html.as_ptr(), ptr::null_mut());
            assert!(result2.is_null(), "Markdown pointer should be null on error");
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_ensures_both_null_on_error() {
        unsafe {
            let mut metadata_json: *mut c_char = ptr::null_mut();

            let result = html_to_markdown_convert_with_metadata(ptr::null(), &mut metadata_json);

            assert!(result.is_null(), "markdown should be null on error");
            assert!(
                metadata_json.is_null(),
                "metadata should be null on error (not partially written)"
            );

            let err = html_to_markdown_last_error();
            assert!(!err.is_null(), "error should be set");
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_null_pointer_safety() {
        unsafe {
            let html = CString::new("<html><head><title>Test</title></head></html>").unwrap();
            let result = html_to_markdown_convert_with_metadata(html.as_ptr(), ptr::null_mut());

            assert!(result.is_null(), "markdown should be null when output ptr is null");
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_valid_json_output() {
        unsafe {
            let html = CString::new("<html><head><title>JSON Output Test</title></head></html>").unwrap();
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let result = html_to_markdown_convert_with_metadata(html.as_ptr(), &mut metadata_json);

            assert!(!result.is_null(), "markdown should not be null");
            assert!(!metadata_json.is_null(), "metadata should not be null");

            let metadata_str = CStr::from_ptr(metadata_json).to_str().unwrap();

            assert!(metadata_str.contains("{"), "Should contain JSON object");
            assert!(metadata_str.contains("}"), "Should contain JSON closing brace");
            assert!(metadata_str.contains("\""), "Should contain JSON quotes");

            html_to_markdown_free_string(result);
            html_to_markdown_free_string(metadata_json);
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_no_memory_leak_on_success() {
        unsafe {
            let html = CString::new("<html><head><title>Memory Test</title></head><body><h1>Header</h1></body></html>")
                .unwrap();
            let mut metadata_json: *mut c_char = ptr::null_mut();

            for _ in 0..10 {
                let result = html_to_markdown_convert_with_metadata(html.as_ptr(), &mut metadata_json);

                assert!(!result.is_null());
                assert!(!metadata_json.is_null());

                html_to_markdown_free_string(result);
                html_to_markdown_free_string(metadata_json);
                metadata_json = ptr::null_mut();
            }
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_with_len_reports_lengths() {
        unsafe {
            let html = CString::new("<html><body><p>hello</p></body></html>").unwrap();
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let mut markdown_len: usize = 0;
            let mut metadata_len: usize = 0;
            let result = html_to_markdown_convert_with_metadata_with_len(
                html.as_ptr(),
                &mut metadata_json,
                &mut markdown_len,
                &mut metadata_len,
            );
            assert!(!result.is_null());
            assert!(!metadata_json.is_null());
            assert!(markdown_len > 0);
            assert!(metadata_len > 0);
            html_to_markdown_free_string(result);
            html_to_markdown_free_string(metadata_json);
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_bytes_with_len_reports_lengths() {
        unsafe {
            let html = b"<html><body><p>hello</p></body></html>";
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let mut markdown_len: usize = 0;
            let mut metadata_len: usize = 0;
            let result = html_to_markdown_convert_with_metadata_bytes_with_len(
                html.as_ptr(),
                html.len(),
                &mut metadata_json,
                &mut markdown_len,
                &mut metadata_len,
            );
            assert!(!result.is_null());
            assert!(!metadata_json.is_null());
            assert!(markdown_len > 0);
            assert!(metadata_len > 0);
            html_to_markdown_free_string(result);
            html_to_markdown_free_string(metadata_json);
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_unicode_handling() {
        unsafe {
            let html = CString::new("<html><head><title>Unicode: 你好 мир 🦀</title></head></html>").unwrap();
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let result = html_to_markdown_convert_with_metadata(html.as_ptr(), &mut metadata_json);

            assert!(!result.is_null());
            assert!(!metadata_json.is_null());

            let metadata_str = CStr::from_ptr(metadata_json).to_str().unwrap();
            assert!(
                metadata_str.contains("你好") || metadata_str.contains("\\u"),
                "Should handle Unicode properly"
            );

            html_to_markdown_free_string(result);
            html_to_markdown_free_string(metadata_json);
        }
    }

    #[cfg(feature = "metadata")]
    #[test]
    fn test_convert_with_metadata_all_fields_present() {
        unsafe {
            let html = CString::new(
                r#"<html lang="en">
                <head>
                    <title>All Fields</title>
                    <meta name="description" content="Test description">
                </head>
                <body>
                    <h1>Header</h1>
                    <a href="https://example.com">Link</a>
                    <img src="test.jpg" alt="Image">
                </body>
            </html>"#,
            )
            .unwrap();
            let mut metadata_json: *mut c_char = ptr::null_mut();
            let result = html_to_markdown_convert_with_metadata(html.as_ptr(), &mut metadata_json);

            assert!(!result.is_null());
            assert!(!metadata_json.is_null());

            let metadata_str = CStr::from_ptr(metadata_json).to_str().unwrap();

            assert!(metadata_str.contains("document"), "Should have document section");
            assert!(metadata_str.contains("headers"), "Should have headers section");
            assert!(metadata_str.contains("links"), "Should have links section");
            assert!(metadata_str.contains("images"), "Should have images section");

            html_to_markdown_free_string(result);
            html_to_markdown_free_string(metadata_json);
        }
    }
}
