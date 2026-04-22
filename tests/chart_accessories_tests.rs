use zavora_xlsx::{Chart, ChartType, Workbook};

/// Test that drop lines and hi-low lines are serialized in line chart XML.
#[test]
fn test_drop_lines_and_hi_low_lines_in_line_chart() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    // Write some data for the chart
    ws.write(0, 0, "Q1").unwrap();
    ws.write(0, 1, "Q2").unwrap();
    ws.write(0, 2, "Q3").unwrap();
    ws.write(1, 0, 10.0).unwrap();
    ws.write(1, 1, 20.0).unwrap();
    ws.write(1, 2, 30.0).unwrap();

    let mut chart = Chart::new(ChartType::Line);
    chart.set_title("Line with Accessories");
    chart.set_drop_lines(true);
    chart.set_high_low_lines(true);
    chart
        .add_series()
        .set_values("Sheet1!$A$2:$C$2")
        .set_categories("Sheet1!$A$1:$C$1");
    ws.insert_chart(3, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Read the chart XML from the ZIP
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let chart_xml = {
        use std::io::Read;
        let mut f = archive.by_name("xl/charts/chart1.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };

    // Verify drop lines and hi-low lines are present
    assert!(
        chart_xml.contains("<c:dropLines/>"),
        "Expected <c:dropLines/> in chart XML"
    );
    assert!(
        chart_xml.contains("<c:hiLowLines/>"),
        "Expected <c:hiLowLines/> in chart XML"
    );
}

/// Test that drop lines and hi-low lines are NOT serialized for non-line chart types.
#[test]
fn test_drop_lines_not_in_bar_chart() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "A").unwrap();
    ws.write(1, 0, 10.0).unwrap();

    let mut chart = Chart::new(ChartType::Bar);
    chart.set_drop_lines(true);
    chart.set_high_low_lines(true);
    chart.add_series().set_values("Sheet1!$A$2");
    ws.insert_chart(3, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let chart_xml = {
        use std::io::Read;
        let mut f = archive.by_name("xl/charts/chart1.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };

    // Drop lines and hi-low lines should NOT appear for bar charts
    assert!(
        !chart_xml.contains("<c:dropLines"),
        "Drop lines should not appear in bar chart"
    );
    assert!(
        !chart_xml.contains("<c:hiLowLines"),
        "Hi-low lines should not appear in bar chart"
    );
}

/// Test chart sheet creation and ZIP file structure.
#[test]
fn test_chart_sheet_creation() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Category").unwrap();
    ws.write(1, 0, "A").unwrap();
    ws.write(2, 0, "B").unwrap();
    ws.write(0, 1, "Values").unwrap();
    ws.write(1, 1, 100.0).unwrap();
    ws.write(2, 1, 200.0).unwrap();

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Chart Sheet Test");
    chart
        .add_series()
        .set_values("Sheet1!$B$2:$B$3")
        .set_categories("Sheet1!$A$2:$A$3");

    wb.add_chart_sheet("MyChart", chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Verify the ZIP structure
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();

    // Check chartsheet XML exists
    let chartsheet_xml = {
        use std::io::Read;
        let mut f = archive.by_name("xl/chartsheets/sheet1.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };
    assert!(
        chartsheet_xml.contains("<chartsheet"),
        "Expected <chartsheet> element"
    );
    assert!(
        chartsheet_xml.contains("r:id=\"rId1\""),
        "Expected drawing relationship"
    );
    assert!(
        chartsheet_xml.contains("<sheetView"),
        "Expected sheetView element"
    );

    // Check chartsheet rels
    let cs_rels = {
        use std::io::Read;
        let mut f = archive
            .by_name("xl/chartsheets/_rels/sheet1.xml.rels")
            .unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };
    assert!(
        cs_rels.contains("drawing"),
        "Expected drawing relationship in chartsheet rels"
    );

    // Check content types include chartsheet
    let content_types = {
        use std::io::Read;
        let mut f = archive.by_name("[Content_Types].xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };
    assert!(
        content_types.contains("chartsheet+xml"),
        "Expected chartsheet content type"
    );

    // Check workbook.xml includes the chart sheet
    let workbook_xml = {
        use std::io::Read;
        let mut f = archive.by_name("xl/workbook.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };
    assert!(
        workbook_xml.contains("MyChart"),
        "Expected chart sheet name in workbook.xml"
    );

    // Check workbook rels include chartsheet relationship
    let wb_rels = {
        use std::io::Read;
        let mut f = archive.by_name("xl/_rels/workbook.xml.rels").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };
    assert!(
        wb_rels.contains("chartsheet"),
        "Expected chartsheet relationship type in workbook rels"
    );
    assert!(
        wb_rels.contains("chartsheets/sheet1.xml"),
        "Expected chartsheet target in workbook rels"
    );

    // Check the chart XML for the chart sheet exists
    // The chart for the chart sheet should be chart2 (chart1 is not used since no worksheet charts)
    // Actually, total_charts from worksheets is 0, so chart_idx = 0 + 1 = 1
    let chart_xml = {
        use std::io::Read;
        let mut f = archive.by_name("xl/charts/chart1.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };
    assert!(
        chart_xml.contains("Chart Sheet Test"),
        "Expected chart title in chart XML"
    );
}

/// Test that duplicate chart sheet names are rejected.
#[test]
fn test_chart_sheet_duplicate_name_rejected() {
    let mut wb = Workbook::new();
    let chart = Chart::new(ChartType::Line);
    // "Sheet1" already exists as a worksheet
    let result = wb.add_chart_sheet("Sheet1", chart);
    assert!(result.is_err());
}
