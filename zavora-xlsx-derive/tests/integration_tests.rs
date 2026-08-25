use zavora_xlsx::{ExcelRowReader, ExcelRowWriter, Workbook};
use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow, Debug, PartialEq)]
struct SimpleRecord {
    name: String,
    value: f64,
    active: bool,
}

#[derive(ExcelRow, Debug, PartialEq)]
struct WithOptions {
    #[excel(header = "Full Name")]
    name: String,
    #[excel(header = "Amount", format = "#,##0.00")]
    amount: f64,
    notes: Option<String>,
    #[excel(skip)]
    internal: String,
}

#[test]
fn test_simple_round_trip() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let record = SimpleRecord {
        name: "Alice".into(),
        value: 42.5,
        active: true,
    };
    record.write_header(ws, 0).unwrap();
    record.write_row(ws, 1).unwrap();

    let headers: Vec<String> = vec!["name".into(), "value".into(), "active".into()];
    let read_back = SimpleRecord::read_row(ws, 1, &headers).unwrap();

    assert_eq!(read_back.name, "Alice");
    assert_eq!(read_back.value, 42.5);
    assert!(read_back.active);
}

#[test]
fn test_custom_headers() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let record = WithOptions {
        name: "Bob".into(),
        amount: 1234.56,
        notes: Some("test".into()),
        internal: "secret".into(),
    };
    record.write_header(ws, 0).unwrap();
    record.write_row(ws, 1).unwrap();

    let headers: Vec<String> = vec!["Full Name".into(), "Amount".into(), "notes".into()];
    let read_back = WithOptions::read_row(ws, 1, &headers).unwrap();

    assert_eq!(read_back.name, "Bob");
    assert_eq!(read_back.amount, 1234.56);
    assert_eq!(read_back.notes, Some("test".into()));
    assert_eq!(read_back.internal, String::new()); // Default for skipped field
}

#[test]
fn test_option_none() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let record = WithOptions {
        name: "Carol".into(),
        amount: 0.0,
        notes: None,
        internal: "x".into(),
    };
    record.write_header(ws, 0).unwrap();
    record.write_row(ws, 1).unwrap();

    let headers: Vec<String> = vec!["Full Name".into(), "Amount".into(), "notes".into()];
    let read_back = WithOptions::read_row(ws, 1, &headers).unwrap();

    assert_eq!(read_back.notes, None);
}

#[test]
fn test_missing_header_error() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let record = SimpleRecord {
        name: "Test".into(),
        value: 1.0,
        active: false,
    };
    record.write_row(ws, 0).unwrap();

    let headers: Vec<String> = vec!["name".into(), "active".into()]; // missing "value"
    let result = SimpleRecord::read_row(ws, 0, &headers);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("value"),
        "Error should mention missing field: {}",
        err_msg
    );
}
