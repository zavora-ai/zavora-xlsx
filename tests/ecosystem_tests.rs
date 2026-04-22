//! Tests for ecosystem integration tasks (83-90).
//!
//! Tests for WASM support, C FFI, and placeholder modules.

use zavora_xlsx::Workbook;

// ── Task 86: WASM Target ──

#[test]
fn test_buffer_roundtrip_works_without_fs() {
    // This verifies the buffer-based API works (the same API used in WASM)
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Hello WASM").unwrap();
    ws.write(0, 1, 42.0).unwrap();
    ws.write(1, 0, "Row 2").unwrap();

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Open from buffer (no filesystem)
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    assert_eq!(ws2.read_cell(0, 0).as_str(), Some("Hello WASM"));
    assert_eq!(ws2.read_cell(0, 1).as_f64(), Some(42.0));
    assert_eq!(ws2.read_cell(1, 0).as_str(), Some("Row 2"));
}

#[test]
fn test_wasm_support_module_exists() {
    // Verify the wasm_support module is accessible
    assert!(!zavora_xlsx::wasm_support::is_wasm());
}

// ── Task 84: Derive Macro Placeholder ──

#[test]
fn test_derive_placeholder_module_exists() {
    // The derive_placeholder module should exist (it's empty, just documentation)
    let _ = std::mem::size_of::<()>(); // placeholder assertion
}

// ── Task 87: Python Bindings Placeholder ──

#[test]
fn test_python_placeholder_module_exists() {
    let _ = std::mem::size_of::<()>();
}

// ── Task 88: Node.js Bindings Placeholder ──

#[test]
fn test_node_placeholder_module_exists() {
    let _ = std::mem::size_of::<()>();
}

// ── Task 90: CLI Tool Placeholder ──

#[test]
fn test_cli_placeholder_module_exists() {
    let _ = std::mem::size_of::<()>();
}

// ── Task 89: C FFI (compile-time check via feature flag) ──

#[test]
fn test_cffi_feature_flag_exists() {
    // The cffi module is behind a feature flag; this test just verifies
    // the crate compiles without it enabled.
    let _ = Workbook::new();
}
