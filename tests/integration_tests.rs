use zavora_xlsx::*;

#[test]
fn create_save_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_basic.xlsx");

    // Create and write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Name").unwrap();
        ws.write(0, 1, "Age").unwrap();
        ws.write(0, 2, "Active").unwrap();
        ws.write(1, 0, "Alice").unwrap();
        ws.write(1, 1, 30.0).unwrap();
        ws.write(1, 2, true).unwrap();
        ws.write(2, 0, "Bob").unwrap();
        ws.write(2, 1, 25.0).unwrap();
        ws.write(2, 2, false).unwrap();
        wb.save(&path).unwrap();
    }

    // Read back with our own reader
    {
        let mut wb = Workbook::open_readonly(&path).unwrap();
        assert_eq!(wb.sheet_names(), vec!["Sheet1"]);
        let ws = wb.worksheet(0).unwrap();

        assert_eq!(ws.read_cell(0, 0), CellValue::String("Name".into()));
        assert_eq!(ws.read_cell(0, 1), CellValue::String("Age".into()));
        assert_eq!(ws.read_cell(1, 0), CellValue::String("Alice".into()));
        assert_eq!(ws.read_cell(1, 1), CellValue::Number(30.0));
        assert_eq!(ws.read_cell(1, 2), CellValue::Bool(true));
        assert_eq!(ws.read_cell(2, 1), CellValue::Number(25.0));
        assert_eq!(ws.read_cell(2, 2), CellValue::Bool(false));
        assert_eq!(ws.read_cell(99, 99), CellValue::Empty);
    }
}

#[test]
fn verify_with_calamine() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_calamine.xlsx");

    // Create with zavora
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Hello").unwrap();
        ws.write(0, 1, 42.5).unwrap();
        ws.write(0, 2, true).unwrap();
        ws.write(1, 0, "World").unwrap();
        ws.write(1, 1, -7.0).unwrap();
        ws.write(1, 2, false).unwrap();
        wb.save(&path).unwrap();
    }

    // Verify with calamine
    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let names = workbook.sheet_names().to_vec();
    assert_eq!(names, vec!["Sheet1"]);

    let range = workbook.worksheet_range("Sheet1").unwrap();
    let rows: Vec<Vec<_>> = range.rows().map(|r| r.to_vec()).collect();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0].get_string(), Some("Hello"));
    assert_eq!(rows[0][1].get_float(), Some(42.5));
    assert_eq!(rows[0][2].get_bool(), Some(true));
    assert_eq!(rows[1][0].get_string(), Some("World"));
    assert_eq!(rows[1][1].get_float(), Some(-7.0));
    assert_eq!(rows[1][2].get_bool(), Some(false));
}

#[test]
fn multiple_sheets() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_multi.xlsx");

    {
        let mut wb = Workbook::new();
        wb.worksheet(0).unwrap().set_name("Data").unwrap();
        wb.worksheet(0).unwrap().write(0, 0, "Sheet1 data").unwrap();

        let ws2 = wb.add_worksheet_with_name("Summary").unwrap();
        ws2.write(0, 0, "Sheet2 data").unwrap();

        wb.save(&path).unwrap();
    }

    {
        let mut wb = Workbook::open_readonly(&path).unwrap();
        assert_eq!(wb.sheet_names(), vec!["Data", "Summary"]);
        assert_eq!(wb.worksheet(0).unwrap().read_cell(0, 0), CellValue::String("Sheet1 data".into()));
        assert_eq!(wb.worksheet(1).unwrap().read_cell(0, 0), CellValue::String("Sheet2 data".into()));
    }
}

#[test]
fn write_row_and_column() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_row_col.xlsx");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write_row(0, 0, ["A", "B", "C"]).unwrap();
        ws.write_column(1, 0, [1.0_f64, 2.0, 3.0]).unwrap();
        wb.save(&path).unwrap();
    }

    {
        let mut wb = Workbook::open_readonly(&path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        assert_eq!(ws.read_cell(0, 0), CellValue::String("A".into()));
        assert_eq!(ws.read_cell(0, 2), CellValue::String("C".into()));
        assert_eq!(ws.read_cell(1, 0), CellValue::Number(1.0));
        assert_eq!(ws.read_cell(3, 0), CellValue::Number(3.0));
    }
}

#[test]
fn formatting_roundtrip_calamine() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_format.xlsx");

    {
        let mut wb = Workbook::new();
        let bold = Format::new().bold();
        let red_bg = Format::new().background_color("#FF0000");

        let ws = wb.worksheet(0).unwrap();
        ws.write_with_format(0, 0, "Bold text", &bold).unwrap();
        ws.write_with_format(0, 1, 100.0, &red_bg).unwrap();
        ws.write(1, 0, "Normal text").unwrap();
        wb.save(&path).unwrap();
    }

    // Verify values survive (formatting is in styles.xml, calamine reads values)
    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let range = workbook.worksheet_range("Sheet1").unwrap();
    let rows: Vec<Vec<_>> = range.rows().map(|r| r.to_vec()).collect();
    assert_eq!(rows[0][0].get_string(), Some("Bold text"));
    assert_eq!(rows[0][1].get_float(), Some(100.0));
    assert_eq!(rows[1][0].get_string(), Some("Normal text"));
}

#[test]
fn formula_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_formula.xlsx");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, 10.0).unwrap();
        ws.write(0, 1, 20.0).unwrap();
        ws.write(0, 2, "=A1+B1").unwrap(); // formula via = prefix
        wb.save(&path).unwrap();
    }

    {
        let mut wb = Workbook::open_readonly(&path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        assert_eq!(ws.read_cell(0, 0), CellValue::Number(10.0));
        match ws.read_cell(0, 2) {
            CellValue::Formula { formula, .. } => assert_eq!(formula, "A1+B1"),
            other => panic!("Expected formula, got {:?}", other),
        }
    }
}

#[test]
fn datetime_roundtrip() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_datetime.xlsx");

    let dt = ExcelDateTime::from_ymd(2024, 1, 15).unwrap();

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, dt).unwrap();
        wb.save(&path).unwrap();
    }

    // Verify with calamine — should see the serial date number
    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let range = workbook.worksheet_range("Sheet1").unwrap();
    let val = &range.rows().next().unwrap()[0];
    // calamine detects this as DateTime due to the yyyy-mm-dd number format — that's correct!
    // Extract the serial value from whichever variant calamine returns
    let serial = match val {
        calamine::Data::Float(f) => *f,
        calamine::Data::DateTime(edt) => edt.as_f64(),
        other => panic!("Unexpected calamine type: {:?}", other),
    };
    assert!((serial - dt.serial()).abs() < 0.001, "Serial date mismatch: {} vs {}", serial, dt.serial());
}

#[test]
fn used_range_tracking() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    assert_eq!(ws.used_range(), None);

    ws.write(2, 3, "hello").unwrap();
    ws.write(5, 1, 42.0).unwrap();
    assert_eq!(ws.used_range(), Some((2, 1, 5, 3)));
}

#[test]
fn large_dataset_calamine_verify() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_large.xlsx");

    let rows: u32 = 1000;
    let cols: u32 = 10;

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        for r in 0..rows {
            for c in 0..cols {
                ws.write(r, c as u16, (r * cols + c) as f64).unwrap();
            }
        }
        wb.save(&path).unwrap();
    }

    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let range = workbook.worksheet_range("Sheet1").unwrap();
    let (h, w) = range.get_size();
    assert_eq!(h, rows as usize);
    assert_eq!(w, cols as usize);

    // Spot check
    for r in [0u32, 100, 500, 999] {
        for c in [0u32, 5, 9] {
            let expected = (r * cols + c) as f64;
            let actual = range.get((r as usize, c as usize)).unwrap().as_f64().unwrap();
            assert_eq!(actual, expected, "Mismatch at ({r}, {c})");
        }
    }
}
