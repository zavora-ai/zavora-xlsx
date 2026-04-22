//! C FFI bindings for zavora-xlsx.
//!
//! Provides an opaque pointer pattern for C interop. All functions use
//! `#[no_mangle] extern "C"` calling convention.
//!
//! Requires the `cffi` feature flag.
//!
//! # Usage from C
//! ```c
//! #include "zavora_xlsx.h"
//!
//! ZavoraWorkbook* wb = zavora_workbook_new();
//! zavora_worksheet_write_string(wb, 0, 0, 0, "Hello");
//! zavora_worksheet_write_number(wb, 0, 0, 1, 42.5);
//! zavora_workbook_save(wb, "output.xlsx");
//! zavora_workbook_free(wb);
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

use crate::Workbook;

/// Thread-local storage for the last error message.
static LAST_ERROR: Mutex<Option<CString>> = Mutex::new(None);

fn set_last_error(msg: &str) {
    if let Ok(mut err) = LAST_ERROR.lock() {
        *err = CString::new(msg).ok();
    }
}

/// Opaque handle to a Workbook.
pub struct ZavoraWorkbook {
    inner: Workbook,
}

/// Create a new empty workbook.
///
/// Returns a pointer to the workbook, or null on failure.
/// The caller must free the workbook with `zavora_workbook_free`.
#[unsafe(no_mangle)]
pub extern "C" fn zavora_workbook_new() -> *mut ZavoraWorkbook {
    Box::into_raw(Box::new(ZavoraWorkbook {
        inner: Workbook::new(),
    }))
}

/// Write a string value to a cell.
///
/// Returns 0 on success, -1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zavora_worksheet_write_string(
    wb: *mut ZavoraWorkbook,
    sheet: u32,
    row: u32,
    col: u16,
    text: *const c_char,
) -> i32 {
    if wb.is_null() || text.is_null() {
        set_last_error("null pointer argument");
        return -1;
    }
    let wb = unsafe { &mut *wb };
    let text = match unsafe { CStr::from_ptr(text) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("invalid UTF-8: {}", e));
            return -1;
        }
    };
    match wb.inner.worksheet(sheet as usize) {
        Ok(ws) => match ws.write(row, col, text) {
            Ok(_) => 0,
            Err(e) => {
                set_last_error(&e.to_string());
                -1
            }
        },
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// Write a numeric value to a cell.
///
/// Returns 0 on success, -1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zavora_worksheet_write_number(
    wb: *mut ZavoraWorkbook,
    sheet: u32,
    row: u32,
    col: u16,
    value: f64,
) -> i32 {
    if wb.is_null() {
        set_last_error("null pointer argument");
        return -1;
    }
    let wb = unsafe { &mut *wb };
    match wb.inner.worksheet(sheet as usize) {
        Ok(ws) => match ws.write(row, col, value) {
            Ok(_) => 0,
            Err(e) => {
                set_last_error(&e.to_string());
                -1
            }
        },
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// Write a boolean value to a cell.
///
/// Returns 0 on success, -1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zavora_worksheet_write_bool(
    wb: *mut ZavoraWorkbook,
    sheet: u32,
    row: u32,
    col: u16,
    value: i32,
) -> i32 {
    if wb.is_null() {
        set_last_error("null pointer argument");
        return -1;
    }
    let wb = unsafe { &mut *wb };
    match wb.inner.worksheet(sheet as usize) {
        Ok(ws) => match ws.write(row, col, value != 0) {
            Ok(_) => 0,
            Err(e) => {
                set_last_error(&e.to_string());
                -1
            }
        },
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// Save the workbook to a file.
///
/// Returns 0 on success, -1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zavora_workbook_save(wb: *mut ZavoraWorkbook, path: *const c_char) -> i32 {
    if wb.is_null() || path.is_null() {
        set_last_error("null pointer argument");
        return -1;
    }
    let wb = unsafe { &mut *wb };
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("invalid UTF-8 path: {}", e));
            return -1;
        }
    };
    match wb.inner.save(path) {
        Ok(_) => 0,
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// Save the workbook to an in-memory buffer.
///
/// On success, writes the buffer pointer to `*out_ptr` and the length to `*out_len`.
/// The caller must free the buffer with `zavora_buffer_free`.
/// Returns 0 on success, -1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zavora_workbook_save_to_buffer(
    wb: *mut ZavoraWorkbook,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    if wb.is_null() || out_ptr.is_null() || out_len.is_null() {
        set_last_error("null pointer argument");
        return -1;
    }
    let wb = unsafe { &mut *wb };
    match wb.inner.save_to_buffer() {
        Ok(buf) => {
            let len = buf.len();
            let ptr = buf.leak().as_mut_ptr();
            unsafe {
                *out_ptr = ptr;
                *out_len = len;
            }
            0
        }
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// Free a buffer allocated by `zavora_workbook_save_to_buffer`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zavora_buffer_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        drop(unsafe { Vec::from_raw_parts(ptr, len, len) });
    }
}

/// Free a workbook handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zavora_workbook_free(wb: *mut ZavoraWorkbook) {
    if !wb.is_null() {
        drop(unsafe { Box::from_raw(wb) });
    }
}

/// Get the last error message.
///
/// Returns a pointer to a null-terminated string, or null if no error.
/// The string is valid until the next FFI call.
#[unsafe(no_mangle)]
pub extern "C" fn zavora_last_error() -> *const c_char {
    match LAST_ERROR.lock() {
        Ok(err) => match err.as_ref() {
            Some(s) => s.as_ptr(),
            None => ptr::null(),
        },
        Err(_) => ptr::null(),
    }
}
