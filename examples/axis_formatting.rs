//! Example: Format chart axes with number formats, fonts, tick marks, and gridlines.
//!
//! Run with: cargo run --example axis_formatting

use zavora_xlsx::{AxisFormat, Chart, ChartType, GridlineStyle, TickMark, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    ws.write(0, 0, "Date").unwrap();
    ws.write(0, 1, "Value").unwrap();
    let dates = ["2024-01", "2024-02", "2024-03", "2024-04", "2024-05", "2024-06"];
    let values = [1250.50, 1480.75, 1320.00, 1590.25, 1710.50, 1850.00];
    for (i, (d, v)) in dates.iter().zip(values.iter()).enumerate() {
        ws.write(i as u32 + 1, 0, *d).unwrap();
        ws.write(i as u32 + 1, 1, *v).unwrap();
    }

    let mut chart = Chart::new(ChartType::Line);
    chart.set_title("Monthly Revenue with Formatted Axes");

    // X axis: custom font
    let mut x_fmt = AxisFormat::new();
    x_fmt.font = Some("Calibri".into());
    x_fmt.font_size = Some(10.0);
    x_fmt.tick_marks = Some(TickMark::Outside);
    chart.set_x_axis_format(x_fmt);
    chart.set_x_axis_name("Month");

    // Y axis: currency format, gridlines
    let mut y_fmt = AxisFormat::new();
    y_fmt.num_format = Some("$#,##0.00".into());
    y_fmt.font_size = Some(9.0);
    y_fmt.tick_marks = Some(TickMark::Cross);
    y_fmt.gridline_style = Some(GridlineStyle::Dash);
    chart.set_y_axis_format(y_fmt);
    chart.set_y_axis_name("Revenue ($)");

    let s = chart.add_series();
    s.set_categories("Data!$A$2:$A$7").set_values("Data!$B$2:$B$7").set_name("Revenue");

    ws.insert_chart(8, 0, &chart).unwrap();
    wb.save("axis_formatting.xlsx").unwrap();
    println!("Created axis_formatting.xlsx");
}
