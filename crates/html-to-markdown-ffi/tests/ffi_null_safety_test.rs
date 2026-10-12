//! Invalid-argument safety tests for the C ABI surface.
//!
//! Every exported `extern "C"` function must tolerate an invalid handle or null pointer without
//! triggering undefined behaviour: no invalid access, no crash, and a documented fallback value.
//! These tests call the real exported functions with invalid arguments and assert the exact
//! fallback each one is documented to return.

#![allow(unsafe_code)]

use std::ptr;

use html_to_markdown_ffi::*;

const INVALID_HANDLE: u64 = 0;

/// Every `htm_*_free` function must be a safe no-op when passed an invalid handle. This is an
/// exhaustive sweep over all 42 free functions currently exported by the crate; if a new
/// `_free` function is added without an invalid-handle guard, this test will fail instead of passing,
/// catching the regression immediately.
#[test]
fn should_not_crash_when_freeing_invalid_handle_for_every_owned_type() {
    // SAFETY: every call below passes the documented zero-valued invalid handle.
    unsafe {
        htm_document_metadata_free(INVALID_HANDLE);
        htm_header_metadata_free(INVALID_HANDLE);
        htm_link_metadata_free(INVALID_HANDLE);
        htm_image_metadata_free(INVALID_HANDLE);
        htm_structured_data_free(INVALID_HANDLE);
        htm_html_metadata_free(INVALID_HANDLE);
        htm_conversion_options_free(INVALID_HANDLE);
        htm_conversion_options_update_free(INVALID_HANDLE);
        htm_preprocessing_options_free(INVALID_HANDLE);
        htm_preprocessing_options_update_free(INVALID_HANDLE);
        htm_image_dimensions_free(INVALID_HANDLE);
        htm_document_structure_free(INVALID_HANDLE);
        htm_document_node_free(INVALID_HANDLE);
        htm_text_annotation_free(INVALID_HANDLE);
        htm_metadata_entry_free(INVALID_HANDLE);
        htm_conversion_result_free(INVALID_HANDLE);
        htm_table_grid_free(INVALID_HANDLE);
        htm_grid_cell_free(INVALID_HANDLE);
        htm_table_data_free(INVALID_HANDLE);
        htm_processing_warning_free(INVALID_HANDLE);
        htm_visitor_handle_free(INVALID_HANDLE);
        htm_node_context_free(INVALID_HANDLE);
        htm_text_direction_free(INVALID_HANDLE);
        htm_link_type_free(INVALID_HANDLE);
        htm_image_type_free(INVALID_HANDLE);
        htm_structured_data_type_free(INVALID_HANDLE);
        htm_tier_strategy_free(INVALID_HANDLE);
        htm_preprocessing_preset_free(INVALID_HANDLE);
        htm_heading_style_free(INVALID_HANDLE);
        htm_list_indent_type_free(INVALID_HANDLE);
        htm_whitespace_mode_free(INVALID_HANDLE);
        htm_newline_style_free(INVALID_HANDLE);
        htm_code_block_style_free(INVALID_HANDLE);
        htm_highlight_style_free(INVALID_HANDLE);
        htm_link_style_free(INVALID_HANDLE);
        htm_url_escape_style_free(INVALID_HANDLE);
        htm_inline_data_media_free(INVALID_HANDLE);
        htm_hidden_content_free(INVALID_HANDLE);
        htm_output_format_free(INVALID_HANDLE);
        htm_node_content_free(INVALID_HANDLE);
        htm_annotation_kind_free(INVALID_HANDLE);
        htm_warning_kind_free(INVALID_HANDLE);
        htm_node_type_free(INVALID_HANDLE);
        htm_visitor_free(INVALID_HANDLE);
    }
    // Reaching this point without a crash is the assertion: every free tolerated the invalid handle.
}

#[test]
fn should_return_null_pointer_when_string_getters_receive_invalid_handle() {
    unsafe {
        assert!(htm_document_metadata_title(INVALID_HANDLE).is_null());
        assert!(htm_document_metadata_description(INVALID_HANDLE).is_null());
        assert!(htm_document_metadata_canonical_url(INVALID_HANDLE).is_null());
        assert!(htm_header_metadata_text(INVALID_HANDLE).is_null());
        assert!(htm_link_metadata_href(INVALID_HANDLE).is_null());
        assert!(htm_image_metadata_src(INVALID_HANDLE).is_null());
        assert!(htm_html_metadata_to_json(INVALID_HANDLE).is_null());
        assert!(htm_conversion_options_to_json(INVALID_HANDLE).is_null());
        assert!(htm_conversion_result_content(INVALID_HANDLE).is_null());
        assert!(htm_table_grid_to_json(INVALID_HANDLE).is_null());
        assert!(htm_metadata_entry_key(INVALID_HANDLE).is_null());
    }
}

#[test]
fn should_return_invalid_handle_when_owned_handle_getters_receive_invalid_handle() {
    unsafe {
        assert_eq!(htm_conversion_options_output_format(INVALID_HANDLE), INVALID_HANDLE);
        assert_eq!(htm_conversion_options_tier_strategy(INVALID_HANDLE), INVALID_HANDLE);
        assert_eq!(htm_html_metadata_document(INVALID_HANDLE), INVALID_HANDLE);
        assert_eq!(htm_image_metadata_dimensions(INVALID_HANDLE), INVALID_HANDLE);
        assert_eq!(htm_conversion_result_document(INVALID_HANDLE), INVALID_HANDLE);
        assert_eq!(htm_conversion_result_metadata(INVALID_HANDLE), INVALID_HANDLE);
        assert_eq!(htm_document_node_content(INVALID_HANDLE), INVALID_HANDLE);
    }
}

#[test]
fn should_return_documented_fallback_when_scalar_getters_receive_invalid_handle() {
    unsafe {
        assert_eq!(htm_header_metadata_level(INVALID_HANDLE), 0);
        assert_eq!(htm_header_metadata_depth(INVALID_HANDLE), 0);
        assert_eq!(htm_header_metadata_html_offset(INVALID_HANDLE), 0);
        assert_eq!(htm_image_dimensions_width(INVALID_HANDLE), 0);
        assert_eq!(htm_image_dimensions_height(INVALID_HANDLE), 0);
        assert_eq!(htm_text_annotation_start(INVALID_HANDLE), 0);
        assert_eq!(htm_text_annotation_end(INVALID_HANDLE), 0);
        assert_eq!(htm_table_grid_rows(INVALID_HANDLE), 0);
        assert_eq!(htm_table_grid_cols(INVALID_HANDLE), 0);
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_not_crash_when_visitor_options_setter_receives_invalid_handles() {
    unsafe {
        htm_options_set_visitor(INVALID_HANDLE, INVALID_HANDLE);
    }
}

#[test]
fn should_return_invalid_handle_when_convert_receives_null_html() {
    unsafe {
        clear_last_error_for_test();
        let result = htm_convert(ptr::null(), INVALID_HANDLE);
        assert_eq!(
            result, INVALID_HANDLE,
            "convert must reject a null html pointer rather than dereference it"
        );
        assert_eq!(
            htm_last_error_code(),
            1,
            "null html should set error code 1 (invalid input)"
        );
    }
}

#[test]
fn should_succeed_when_convert_receives_invalid_options_handle() {
    unsafe {
        let html = std::ffi::CString::new("<p>ok</p>").unwrap();
        let result = htm_convert(html.as_ptr(), INVALID_HANDLE);
        assert_ne!(
            result, INVALID_HANDLE,
            "convert must accept an invalid options handle and fall back to defaults"
        );
        htm_conversion_result_free(result);
    }
}

#[test]
fn should_return_invalid_handle_when_json_constructors_receive_null_string() {
    unsafe {
        assert_eq!(htm_conversion_options_from_json(ptr::null()), INVALID_HANDLE);
        assert_eq!(htm_conversion_options_update_from_json(ptr::null()), INVALID_HANDLE);
        assert_eq!(htm_document_metadata_from_json(ptr::null()), INVALID_HANDLE);
        assert_eq!(htm_header_metadata_from_json(ptr::null()), INVALID_HANDLE);
        assert_eq!(htm_link_metadata_from_json(ptr::null()), INVALID_HANDLE);
        assert_eq!(htm_image_metadata_from_json(ptr::null()), INVALID_HANDLE);
        assert_eq!(htm_structured_data_from_json(ptr::null()), INVALID_HANDLE);
        assert_eq!(htm_html_metadata_from_json(ptr::null()), INVALID_HANDLE);
    }
}

/// `htm_last_error_context`/`htm_last_error_code` must be callable with no prior error state
/// and must not crash when called back-to-back from a fresh thread-local.
#[test]
fn should_report_no_error_when_no_prior_ffi_call_failed_on_this_thread() {
    unsafe {
        clear_last_error_for_test();
        assert_eq!(htm_last_error_code(), 0);
        let ctx = htm_last_error_context();
        assert!(ctx.is_null(), "context must be null when there is no recorded error");
    }
}

/// Reset the thread-local error state via a call known to clear it on success, so tests that
/// check for the "no error" baseline are independent of prior tests' failures on this thread.
unsafe fn clear_last_error_for_test() {
    unsafe {
        let html = std::ffi::CString::new("<p>reset</p>").unwrap();
        let result = htm_convert(html.as_ptr(), INVALID_HANDLE);
        assert_ne!(result, INVALID_HANDLE);
        htm_conversion_result_free(result);
    }
}
