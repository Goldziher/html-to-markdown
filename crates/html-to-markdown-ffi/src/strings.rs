//! C string utilities for FFI.
//!
//! This module provides utilities for converting between Rust strings and C strings
//! safely across the FFI boundary.

use std::ffi::CString;

/// Convert a byte vector to a C string.
///
/// # Arguments
///
/// * `bytes` - The byte vector to convert
/// * `context` - Context string for error messages
///
/// # Returns
///
/// `Ok(CString)` if successful, `Err(String)` if the bytes contain interior null bytes
///
/// # Safety
///
/// This function checks for interior null bytes and will fail if any are found.
#[allow(dead_code)]
pub fn bytes_to_c_string(bytes: Vec<u8>, context: &str) -> Result<CString, String> {
    if bytes.contains(&0) {
        return Err(format!("{context} contained an interior null byte"));
    }

    CString::new(bytes).map_err(|_| format!("{context} contained an interior null byte"))
}

/// Convert a Rust String to a C string.
///
/// # Arguments
///
/// * `value` - The String to convert
/// * `context` - Context string for error messages
///
/// # Returns
///
/// `Ok(CString)` if successful, `Err(String)` if the string contains interior null bytes
#[allow(dead_code)]
pub fn string_to_c_string(value: String, context: &str) -> Result<CString, String> {
    bytes_to_c_string(value.into_bytes(), context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_c_string_valid() {
        let result = string_to_c_string("hello world".to_string(), "test");
        assert!(result.is_ok());
        let c_str = result.unwrap();
        assert_eq!(c_str.to_str().unwrap(), "hello world");
    }

    #[test]
    fn test_string_to_c_string_interior_null() {
        let result = string_to_c_string("hello\0world".to_string(), "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("interior null byte"));
    }

    #[test]
    fn test_bytes_to_c_string_valid() {
        let result = bytes_to_c_string(b"test".to_vec(), "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bytes_to_c_string_empty() {
        let result = bytes_to_c_string(vec![], "test");
        assert!(result.is_ok());
        let c_str = result.unwrap();
        assert_eq!(c_str.to_bytes(), b"");
    }
}
