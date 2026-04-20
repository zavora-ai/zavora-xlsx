//! Round-trip tests for chart reading: write charts, save, read back, verify.

use zavora_xlsx::{Chart, ChartType, LegendPosition, Workbook};

/// Write a bar chart with 2 series, read back, verify chart_type and series count.
#[test]
fn round_trip_bar_chart_two_series() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write some data
    ws.write(0, 0, "Category").unwrap();
    ws.write(1, 0, "A").unwrap();
    ws.write(2, 0, "B").unwrap();
    ws.write(3, 0, "C").unwrap();
    ws.write(0, 1, "Series 1").unwrap();
    ws.write(1, 1, 10).unwrap();
    ws.write(2, 1, 20).unwrap();
    ws.write(3, 1, 30).unwrap();
    ws.write(0, 2, "Series 2").unwrap();
    ws.write(1, 2, 15).unwrap();
    ws.write(2, 2, 25).unwrap();
    ws.write(3, 2, 35).unwrap();

    let mut chart = Chart::new(ChartType::Bar);
    chart.add_series()
        .set_values("Sheet1!$B$2:$B$4")
        .set_categories("Sheet1!$A$2:$A$4")
        .set_name("Series 1");
    chart.add_series()
        .set_values("Sheet1!$C$2:$C$4")
        .set_categories("Sheet1!$A$2:$A$4")
        .set_name("Series 2");
    ws.insert_chart(5, 0, &chart).unwrap();

    // Save to buffer
    let buf = wb.save_to_buffer().unwrap();

    // Read back
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let charts = ws2.charts();

    assert_eq!(charts.len(), 1, "Expected 1 chart");
    let c = &charts[0];
    assert_eq!(c.chart_type(), ChartType::Bar);
    assert_eq!(c.series().len(), 2, "Expected 2 series");
    assert_eq!(c.series()[0].name(), Some("Series 1"));
    assert_eq!(c.series()[1].name(), Some("Series 2"));
    assert_eq!(c.series()[0].values(), "Sheet1!$B$2:$B$4");
    assert_eq!(c.series()[0].categories(), Some("Sheet1!$A$2:$A$4"));
}

/// Write a line chart with title, read back, verify title.
#[test]
fn round_trip_line_chart_with_title() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "X").unwrap();
    ws.write(1, 0, 1).unwrap();
    ws.write(2, 0, 2).unwrap();
    ws.write(0, 1, "Y").unwrap();
    ws.write(1, 1, 10).unwrap();
    ws.write(2, 1, 20).unwrap();

    let mut chart = Chart::new(ChartType::Line);
    chart.set_title("My Line Chart");
    chart.set_x_axis_name("X Axis");
    chart.set_y_axis_name("Y Axis");
    chart.set_legend_position(LegendPosition::Top);
    chart.add_series()
        .set_values("Sheet1!$B$2:$B$3")
        .set_categories("Sheet1!$A$2:$A$3")
        .set_name("Data");
    ws.insert_chart(5, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let charts = ws2.charts();

    assert_eq!(charts.len(), 1);
    let c = &charts[0];
    assert_eq!(c.chart_type(), ChartType::Line);
    assert_eq!(c.title(), Some("My Line Chart"));
    assert_eq!(c.x_axis_name(), Some("X Axis"));
    assert_eq!(c.y_axis_name(), Some("Y Axis"));
    assert!(matches!(c.legend_position(), LegendPosition::Top));
    assert_eq!(c.series().len(), 1);
    assert_eq!(c.series()[0].name(), Some("Data"));
}

/// Write multiple charts on one sheet, read back, verify count.
#[test]
fn round_trip_multiple_charts_on_one_sheet() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Cat").unwrap();
    ws.write(1, 0, "A").unwrap();
    ws.write(2, 0, "B").unwrap();
    ws.write(0, 1, "Val").unwrap();
    ws.write(1, 1, 100).unwrap();
    ws.write(2, 1, 200).unwrap();

    // Chart 1: Pie
    let mut chart1 = Chart::new(ChartType::Pie);
    chart1.set_title("Pie Chart");
    chart1.add_series()
        .set_values("Sheet1!$B$2:$B$3")
        .set_categories("Sheet1!$A$2:$A$3");
    ws.insert_chart(5, 0, &chart1).unwrap();

    // Chart 2: Column
    let mut chart2 = Chart::new(ChartType::Column);
    chart2.set_title("Column Chart");
    chart2.add_series()
        .set_values("Sheet1!$B$2:$B$3")
        .set_categories("Sheet1!$A$2:$A$3");
    ws.insert_chart(20, 0, &chart2).unwrap();

    // Chart 3: Scatter
    let mut chart3 = Chart::new(ChartType::Scatter);
    chart3.add_series()
        .set_values("Sheet1!$B$2:$B$3")
        .set_categories("Sheet1!$A$2:$A$3");
    ws.insert_chart(35, 0, &chart3).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let charts = ws2.charts();

    assert_eq!(charts.len(), 3, "Expected 3 charts on the sheet");
    assert_eq!(charts[0].chart_type(), ChartType::Pie);
    assert_eq!(charts[0].title(), Some("Pie Chart"));
    assert_eq!(charts[1].chart_type(), ChartType::Column);
    assert_eq!(charts[1].title(), Some("Column Chart"));
    assert_eq!(charts[2].chart_type(), ChartType::Scatter);
}
