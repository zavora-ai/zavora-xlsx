use zavora_xlsx::{Sparkline, SparklineType, Workbook};

#[test]
fn test_roundtrip_sparklines_line() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write some data for the sparkline to reference
    for row in 0..5 {
        ws.write(row, 0, (row + 1) as f64 * 10.0).unwrap();
    }

    // Add a line sparkline
    let sp = Sparkline::new("Sheet1!A1:A5", SparklineType::Line);
    ws.add_sparkline(0, 1, &sp).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Read back and verify
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let sparklines = ws2.sparklines();

    assert_eq!(
        sparklines.len(),
        1,
        "Expected 1 sparkline, got {}",
        sparklines.len()
    );
    assert_eq!(sparklines[0].data_range(), "Sheet1!A1:A5");
    assert_eq!(sparklines[0].row(), 0);
    assert_eq!(sparklines[0].col(), 1);
    assert!(matches!(
        sparklines[0].sparkline_type(),
        SparklineType::Line
    ));
}

#[test]
fn test_roundtrip_sparklines_column() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    for row in 0..5 {
        ws.write(row, 0, (row + 1) as f64 * 5.0).unwrap();
    }

    let sp = Sparkline::new("Sheet1!A1:A5", SparklineType::Column);
    ws.add_sparkline(0, 2, &sp).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let sparklines = ws2.sparklines();

    assert_eq!(sparklines.len(), 1);
    assert!(matches!(
        sparklines[0].sparkline_type(),
        SparklineType::Column
    ));
    assert_eq!(sparklines[0].data_range(), "Sheet1!A1:A5");
    assert_eq!(sparklines[0].col(), 2);
}

#[test]
fn test_roundtrip_sparklines_winloss() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Win/loss data: positive and negative values
    ws.write(0, 0, 1.0).unwrap();
    ws.write(1, 0, -1.0).unwrap();
    ws.write(2, 0, 1.0).unwrap();
    ws.write(3, 0, -1.0).unwrap();
    ws.write(4, 0, 1.0).unwrap();

    let sp = Sparkline::new("Sheet1!A1:A5", SparklineType::WinLoss);
    ws.add_sparkline(0, 3, &sp).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let sparklines = ws2.sparklines();

    assert_eq!(sparklines.len(), 1);
    assert!(matches!(
        sparklines[0].sparkline_type(),
        SparklineType::WinLoss
    ));
    assert_eq!(sparklines[0].data_range(), "Sheet1!A1:A5");
}

#[test]
fn test_roundtrip_sparklines_multiple() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write data in columns A and B
    for row in 0..5 {
        ws.write(row, 0, (row + 1) as f64).unwrap();
        ws.write(row, 1, (row + 1) as f64 * 2.0).unwrap();
    }

    // Add multiple sparklines of different types
    let sp1 = Sparkline::new("Sheet1!A1:A5", SparklineType::Line);
    ws.add_sparkline(0, 2, &sp1).unwrap();

    let sp2 = Sparkline::new("Sheet1!B1:B5", SparklineType::Column);
    ws.add_sparkline(1, 2, &sp2).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let sparklines = ws2.sparklines();

    assert_eq!(
        sparklines.len(),
        2,
        "Expected 2 sparklines, got {}",
        sparklines.len()
    );

    // Sparklines may be in any order, so find by location
    let sp_line = sparklines
        .iter()
        .find(|s| s.row() == 0 && s.col() == 2)
        .expect("Line sparkline not found");
    assert!(matches!(sp_line.sparkline_type(), SparklineType::Line));
    assert_eq!(sp_line.data_range(), "Sheet1!A1:A5");

    let sp_col = sparklines
        .iter()
        .find(|s| s.row() == 1 && s.col() == 2)
        .expect("Column sparkline not found");
    assert!(matches!(sp_col.sparkline_type(), SparklineType::Column));
    assert_eq!(sp_col.data_range(), "Sheet1!B1:B5");
}

#[test]
fn test_roundtrip_sparklines_with_color() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    for row in 0..5 {
        ws.write(row, 0, (row + 1) as f64).unwrap();
    }

    let mut sp = Sparkline::new("Sheet1!A1:A5", SparklineType::Line);
    sp.set_color((255u8, 0u8, 0u8)); // Red
    ws.add_sparkline(0, 1, &sp).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let sparklines = ws2.sparklines();

    assert_eq!(sparklines.len(), 1);
    assert_eq!(sparklines[0].color(), Some([255, 0, 0]));
}

#[test]
fn test_no_sparklines_sheet() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "No sparklines here").unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    assert!(ws2.sparklines().is_empty());
}

#[test]
fn test_roundtrip_sparklines_edit_mode() {
    // Test that sparklines are preserved through edit-mode open
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    for row in 0..5 {
        ws.write(row, 0, (row + 1) as f64).unwrap();
    }

    let sp = Sparkline::new("Sheet1!A1:A5", SparklineType::Column);
    ws.add_sparkline(0, 1, &sp).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Open in edit mode and read sparklines
    let mut wb2 = Workbook::open_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet(0).unwrap();
    let sparklines = ws2.sparklines();

    assert_eq!(sparklines.len(), 1);
    assert!(matches!(
        sparklines[0].sparkline_type(),
        SparklineType::Column
    ));
    assert_eq!(sparklines[0].data_range(), "Sheet1!A1:A5");
}
