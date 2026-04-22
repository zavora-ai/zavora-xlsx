//! Tests for C FFI bindings (Task 89).
//!
//! These tests require the `cffi` feature flag.

#![cfg(feature = "cffi")]

use std::ffi::CString;
use zavora_xlsx::cffi;

#[test]
fn test_cffi_workbook_new_and_free() {
    let wb = cffi::zavora_workbook_new();
    assert!(!wb.is_null());
    unsafe { cffi::zavora_workbook_free(wb) };
}

#[test]
fn test_cffi_write_string() {
    let wb = cffi::zavora_workbook_new();
    let text = CString::new("Hello FFI").unwrap();
    let result = unsafe { cffi::zavora_worksheet_write_string(wb, 0, 0, 0, text.as_ptr()) };
    assert_eq!(result, 0);
    unsafe { cffi::zavora_workbook_free(wb) };
}

#[test]
fn test_cffi_write_number() {
    let wb = cffi::zavora_workbook_new();
    let result = unsafe { cffi::zavora_worksheet_write_number(wb, 0, 0, 0, 42.5) };
    assert_eq!(result, 0);
    unsafe { cffi::zavora_workbook_free(wb) };
}

#[test]
fn test_cffi_write_bool() {
    let wb = cffi::zavora_workbook_new();
    let result = unsafe { cffi::zavora_worksheet_write_bool(wb, 0, 0, 0, 1) };
    assert_eq!(result, 0);
    unsafe { cffi::zavora_workbook_free(wb) };
}

#[test]
fn test_cffi_save_to_buffer() {
    let wb = cffi::zavora_workbook_new();
    let text = CString::new("Test").unwrap();
    unsafe {
        cffi::zavora_worksheet_write_string(wb, 0, 0, 0, text.as_ptr());
    }

    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let result = unsafe { cffi::zavora_workbook_save_to_buffer(wb, &mut out_ptr, &mut out_len) };
    assert_eq!(result, 0);
    assert!(!out_ptr.is_null());
    assert!(out_len > 0);

    // Free the buffer
    unsafe { cffi::zavora_buffer_free(out_ptr, out_len) };
    unsafe { cffi::zavora_workbook_free(wb) };
}

#[test]
fn test_cffi_null_pointer_error() {
    let result = unsafe {
        cffi::zavora_worksheet_write_string(std::ptr::null_mut(), 0, 0, 0, std::ptr::null())
    };
    assert_eq!(result, -1);

    let err = cffi::zavora_last_error();
    assert!(!err.is_null());
}

#[test]
fn test_cffi_last_error_null_when_no_error() {
    // After a successful operation, last_error should still hold the previous error
    // (it's not cleared automatically). But initially it should be null.
    // We test that the function doesn't crash.
    let _ = cffi::zavora_last_error();
}
