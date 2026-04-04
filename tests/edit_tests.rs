use zavora_xlsx::*;

/// Create a file, reopen in edit mode, modify, save, verify with calamine.
#[test]
fn edit_mode_modify_cell() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("edit_cell.xlsx");

    // Create original
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Original").unwrap();
        ws.write(0, 1, 100.0).unwrap();
        ws.write(1, 0, "Keep").unwrap();
        wb.save(&path).unwrap();
    }

    // Open in edit mode, modify one cell, save
    {
        let mut wb = Workbook::open(&path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        assert_eq!(ws.read_cell(0, 0), CellValue::String("Original".into()));
        ws.write(0, 0, "Modified").unwrap();
        wb.save(&path).unwrap();
    }

    // Verify with calamine
    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let range = workbook.worksheet_range("Sheet1").unwrap();
    let rows: Vec<Vec<_>> = range.rows().map(|r| r.to_vec()).collect();
    assert_eq!(rows[0][0].get_string(), Some("Modified"));
    assert_eq!(rows[0][1].get_float(), Some(100.0));
    assert_eq!(rows[1][0].get_string(), Some("Keep"));
}

/// Test that unmodified sheets pass through as raw bytes.
#[test]
fn edit_mode_raw_passthrough() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("passthrough.xlsx");

    // Create with 2 sheets
    {
        let mut wb = Workbook::new();
        wb.worksheet(0).unwrap().write(0, 0, "Sheet1 data").unwrap();
        let ws2 = wb.add_worksheet_with_name("Sheet2").unwrap();
        ws2.write(0, 0, "Sheet2 data").unwrap();
        wb.save(&path).unwrap();
    }

    // Open, modify only Sheet1, save
    {
        let mut wb = Workbook::open(&path).unwrap();
        assert_eq!(wb.sheet_count(), 2);
        // Only touch Sheet1
        wb.worksheet(0).unwrap().write(0, 0, "Modified").unwrap();
        // Sheet2 is never accessed — should pass through as raw bytes
        wb.save(&path).unwrap();
    }

    // Verify both sheets
    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let r1 = workbook.worksheet_range("Sheet1").unwrap();
    assert_eq!(r1.rows().next().unwrap()[0].get_string(), Some("Modified"));

    let r2 = workbook.worksheet_range("Sheet2").unwrap();
    assert_eq!(r2.rows().next().unwrap()[0].get_string(), Some("Sheet2 data"));
}

/// Test sheet management: add, remove, rename.
#[test]
fn sheet_management() {
    use calamine::{Reader, Xlsx, open_workbook};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sheet_mgmt.xlsx");

    {
        let mut wb = Workbook::new();
        wb.worksheet(0).unwrap().write(0, 0, "A").unwrap();
        wb.add_worksheet_with_name("Second").unwrap().write(0, 0, "B").unwrap();
        wb.add_worksheet_with_name("Third").unwrap().write(0, 0, "C").unwrap();

        // Remove middle sheet
        wb.remove_worksheet(1).unwrap();
        assert_eq!(wb.sheet_names(), vec!["Sheet1", "Third"]);

        // Rename
        wb.rename_worksheet(1, "Last").unwrap();
        assert_eq!(wb.sheet_names(), vec!["Sheet1", "Last"]);

        wb.save(&path).unwrap();
    }

    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let names = workbook.sheet_names().to_vec();
    assert_eq!(names, vec!["Sheet1", "Last"]);
}

/// Test insert rows with formula adjustment.
#[test]
fn insert_rows_shifts_cells_and_formulas() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("insert_rows.xlsx");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Header").unwrap();
        ws.write(1, 0, 10.0).unwrap();
        ws.write(2, 0, 20.0).unwrap();
        ws.write_formula(3, 0, "SUM(A2:A3)").unwrap();

        // Insert 2 rows at row 1 (0-based) — pushes data down
        ws.insert_rows(1, 2).unwrap();

        // Header stays at row 0, data shifts to rows 3,4, formula to row 5
        assert_eq!(ws.read_cell(0, 0), CellValue::String("Header".into()));
        assert_eq!(ws.read_cell(3, 0), CellValue::Number(10.0));
        assert_eq!(ws.read_cell(4, 0), CellValue::Number(20.0));

        // Formula should be adjusted: SUM(A2:A3) → SUM(A4:A5)
        match ws.read_cell(5, 0) {
            CellValue::Formula { formula, .. } => assert_eq!(formula, "SUM(A4:A5)"),
            other => panic!("Expected formula, got {:?}", other),
        }

        wb.save(&path).unwrap();
    }

    // Verify with calamine
    use calamine::{Reader, Xlsx, open_workbook, DataType};
    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let range = workbook.worksheet_range("Sheet1").unwrap();
    assert_eq!(range.get((0, 0)).unwrap().get_string(), Some("Header"));
    assert_eq!(range.get((3, 0)).unwrap().get_float(), Some(10.0));
}

/// Test remove rows.
#[test]
fn remove_rows_shifts_cells() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "A").unwrap();
    ws.write(1, 0, "B").unwrap();
    ws.write(2, 0, "C").unwrap();
    ws.write(3, 0, "D").unwrap();

    // Remove rows 1-2 (0-based), which are "B" and "C"
    ws.remove_rows(1, 2).unwrap();

    assert_eq!(ws.read_cell(0, 0), CellValue::String("A".into()));
    assert_eq!(ws.read_cell(1, 0), CellValue::String("D".into()));
    assert_eq!(ws.read_cell(2, 0), CellValue::Empty);
}

/// Test insert columns.
#[test]
fn insert_columns_shifts_cells() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "A").unwrap();
    ws.write(0, 1, "B").unwrap();
    ws.write(0, 2, "C").unwrap();

    // Insert 1 column at col 1 (B)
    ws.insert_columns(1, 1).unwrap();

    assert_eq!(ws.read_cell(0, 0), CellValue::String("A".into()));
    assert_eq!(ws.read_cell(0, 1), CellValue::Empty); // new empty column
    assert_eq!(ws.read_cell(0, 2), CellValue::String("B".into()));
    assert_eq!(ws.read_cell(0, 3), CellValue::String("C".into()));
}

/// Test remove columns.
#[test]
fn remove_columns_shifts_cells() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "A").unwrap();
    ws.write(0, 1, "B").unwrap();
    ws.write(0, 2, "C").unwrap();
    ws.write(0, 3, "D").unwrap();

    // Remove column 1 (B)
    ws.remove_columns(1, 1).unwrap();

    assert_eq!(ws.read_cell(0, 0), CellValue::String("A".into()));
    assert_eq!(ws.read_cell(0, 1), CellValue::String("C".into()));
    assert_eq!(ws.read_cell(0, 2), CellValue::String("D".into()));
}

/// Test defined names roundtrip.
#[test]
fn defined_names_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("defined_names.xlsx");

    {
        let mut wb = Workbook::new();
        wb.worksheet(0).unwrap().write(0, 0, 42.0).unwrap();
        wb.define_name("MyRange", "Sheet1!$A$1");
        wb.save(&path).unwrap();
    }

    {
        let wb = Workbook::open_readonly(&path).unwrap();
        let names = wb.defined_names();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0].0, "MyRange");
        assert_eq!(names[0].1, "Sheet1!$A$1");
    }
}

/// Test document properties roundtrip.
#[test]
fn doc_properties_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("props.xlsx");

    {
        let mut wb = Workbook::new();
        wb.worksheet(0).unwrap().write(0, 0, "data").unwrap();
        wb.set_properties(DocProperties::new().title("My Report").author("Test User"));
        wb.save(&path).unwrap();
    }

    {
        let wb = Workbook::open_readonly(&path).unwrap();
        assert_eq!(wb.properties().title.as_deref(), Some("My Report"));
        assert_eq!(wb.properties().author.as_deref(), Some("Test User"));
    }
}

/// Test move worksheet reorder.
#[test]
fn move_worksheet_reorder() {
    use calamine::{Reader, Xlsx, open_workbook};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reorder.xlsx");

    {
        let mut wb = Workbook::new();
        wb.worksheet(0).unwrap().set_name("A").unwrap();
        wb.add_worksheet_with_name("B").unwrap();
        wb.add_worksheet_with_name("C").unwrap();

        // Move C (index 2) to front (index 0)
        wb.move_worksheet(2, 0).unwrap();
        assert_eq!(wb.sheet_names(), vec!["C", "A", "B"]);
        wb.save(&path).unwrap();
    }

    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    assert_eq!(workbook.sheet_names(), &["C", "A", "B"]);
}

/// Full edit workflow: create → save → open edit → add sheet → modify → save → verify.
#[test]
fn full_edit_workflow() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("full_edit.xlsx");

    // Step 1: Create
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write_row(0, 0, ["Name", "Score"]).unwrap();
        ws.write(1, 0, "Alice").unwrap();
        ws.write(1, 1, 95.0).unwrap();
        ws.write(2, 0, "Bob").unwrap();
        ws.write(2, 1, 87.0).unwrap();
        wb.save(&path).unwrap();
    }

    // Step 2: Edit — add a row, add a sheet
    {
        let mut wb = Workbook::open(&path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        // Insert a row at position 2 to add Charlie between Alice and Bob
        ws.insert_rows(2, 1).unwrap();
        ws.write(2, 0, "Charlie").unwrap();
        ws.write(2, 1, 91.0).unwrap();

        // Add summary sheet
        let ws2 = wb.add_worksheet_with_name("Summary").unwrap();
        ws2.write(0, 0, "Total students: 3").unwrap();

        wb.save(&path).unwrap();
    }

    // Step 3: Verify
    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    assert_eq!(workbook.sheet_names(), &["Sheet1", "Summary"]);

    let range = workbook.worksheet_range("Sheet1").unwrap();
    let rows: Vec<Vec<_>> = range.rows().map(|r| r.to_vec()).collect();
    assert_eq!(rows[0][0].get_string(), Some("Name"));
    assert_eq!(rows[1][0].get_string(), Some("Alice"));
    assert_eq!(rows[2][0].get_string(), Some("Charlie"));
    assert_eq!(rows[2][1].get_float(), Some(91.0));
    assert_eq!(rows[3][0].get_string(), Some("Bob"));

    let summary = workbook.worksheet_range("Summary").unwrap();
    assert_eq!(summary.rows().next().unwrap()[0].get_string(), Some("Total students: 3"));
}
