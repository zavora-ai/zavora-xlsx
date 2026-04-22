//! Tests for serde integration (Task 83).
//!
//! These tests require the `serde-support` feature flag.

#![cfg(feature = "serde-support")]

use serde::{Deserialize, Serialize};
use zavora_xlsx::Workbook;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SimpleRecord {
    name: String,
    value: f64,
    active: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct OptionalRecord {
    label: String,
    count: f64,
    note: Option<String>,
}

#[test]
fn test_serde_write_rows_creates_headers() {
    let data = vec![SimpleRecord {
        name: "Alice".into(),
        value: 100.0,
        active: true,
    }];
    let mut wb = Workbook::new();
    wb.write_rows(0, &data).unwrap();

    let ws = wb.worksheet(0).unwrap();
    assert_eq!(ws.read_cell(0, 0).as_str(), Some("name"));
    assert_eq!(ws.read_cell(0, 1).as_str(), Some("value"));
    assert_eq!(ws.read_cell(0, 2).as_str(), Some("active"));
}

#[test]
fn test_serde_write_rows_creates_data() {
    let data = vec![
        SimpleRecord {
            name: "Alice".into(),
            value: 100.0,
            active: true,
        },
        SimpleRecord {
            name: "Bob".into(),
            value: 200.0,
            active: false,
        },
    ];
    let mut wb = Workbook::new();
    wb.write_rows(0, &data).unwrap();

    let ws = wb.worksheet(0).unwrap();
    // Row 1 = first data row
    assert_eq!(ws.read_cell(1, 0).as_str(), Some("Alice"));
    assert_eq!(ws.read_cell(1, 1).as_f64(), Some(100.0));
    // Row 2 = second data row
    assert_eq!(ws.read_cell(2, 0).as_str(), Some("Bob"));
    assert_eq!(ws.read_cell(2, 1).as_f64(), Some(200.0));
}

#[test]
fn test_serde_roundtrip() {
    let data = vec![
        SimpleRecord {
            name: "Alice".into(),
            value: 100.0,
            active: true,
        },
        SimpleRecord {
            name: "Bob".into(),
            value: 200.5,
            active: false,
        },
        SimpleRecord {
            name: "Carol".into(),
            value: 300.0,
            active: true,
        },
    ];

    let mut wb = Workbook::new();
    wb.write_rows(0, &data).unwrap();
    let buf = wb.save_to_buffer().unwrap();

    let mut wb2 = Workbook::open_from_buffer(&buf).unwrap();
    let result: Vec<SimpleRecord> = wb2.read_rows(0).unwrap();

    assert_eq!(data.len(), result.len());
    for (orig, read) in data.iter().zip(result.iter()) {
        assert_eq!(orig.name, read.name);
        assert_eq!(orig.value, read.value);
        assert_eq!(orig.active, read.active);
    }
}

#[test]
fn test_serde_empty_data() {
    let data: Vec<SimpleRecord> = vec![];
    let mut wb = Workbook::new();
    wb.write_rows(0, &data).unwrap();

    // Should not have written anything
    let ws = wb.worksheet(0).unwrap();
    assert!(ws.read_cell(0, 0).is_empty());
}

#[test]
fn test_serde_optional_fields_roundtrip() {
    let data = vec![
        OptionalRecord {
            label: "A".into(),
            count: 1.0,
            note: Some("first".into()),
        },
        OptionalRecord {
            label: "B".into(),
            count: 2.0,
            note: None,
        },
    ];

    let mut wb = Workbook::new();
    wb.write_rows(0, &data).unwrap();
    let buf = wb.save_to_buffer().unwrap();

    let mut wb2 = Workbook::open_from_buffer(&buf).unwrap();
    let result: Vec<OptionalRecord> = wb2.read_rows(0).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].label, "A");
    assert_eq!(result[0].note, Some("first".into()));
    assert_eq!(result[1].label, "B");
    // Empty cell deserializes as None for Option<String>
    assert_eq!(result[1].note, None);
}

#[test]
fn test_serde_type_mismatch_error() {
    // Write string data, try to read as numbers
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "name").unwrap();
    ws.write(0, 1, "value").unwrap();
    ws.write(1, 0, "Alice").unwrap();
    ws.write(1, 1, "not_a_number").unwrap();

    #[derive(Debug, Deserialize)]
    struct NumRecord {
        name: String,
        value: f64,
    }

    let buf = wb.save_to_buffer().unwrap();
    let mut wb2 = Workbook::open_from_buffer(&buf).unwrap();
    let result = wb2.read_rows::<NumRecord>(0);
    // Should fail because "not_a_number" can't be parsed as f64
    assert!(result.is_err());
}

#[test]
fn test_serde_buffer_roundtrip_preserves_types() {
    let data = vec![SimpleRecord {
        name: "Test".into(),
        value: 42.5,
        active: true,
    }];

    let mut wb = Workbook::new();
    wb.write_rows(0, &data).unwrap();
    let buf = wb.save_to_buffer().unwrap();

    // Open and re-read
    let mut wb2 = Workbook::open_from_buffer(&buf).unwrap();
    let result: Vec<SimpleRecord> = wb2.read_rows(0).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], data[0]);
}
