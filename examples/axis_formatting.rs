//! Example: Format chart axes with custom number formats, fonts, tick marks,
//! and gridlines (Task 36).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::path::PathBuf::from("output/axis_formatting_example.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Revenue")?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Month", &hdr)?;
    ws.write_with_format(0, 1, "Revenue", &hdr)?;

    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
    let revenue = [12500.0, 18200.0, 15800.0, 22100.0, 19400.0, 25600.0];
    for (i, (m, r)) in months.iter().zip(revenue.iter()).enumerate() {
        ws.write((i + 1) as u32, 0, *m)?;
        ws.write((i + 1) as u32, 1, *r)?;
    }

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Monthly Revenue");

    // X axis: bold font, outside tick marks
    let mut x_fmt = AxisFormat::new();
    x_fmt.set_font_name("Arial");
    x_fmt.set_font_size(11.0);
    x_fmt.set_font_bold(true);
    x_fmt.set_font_color([0, 51, 102]);
    x_fmt.set_major_tick_mark(TickMark::Outside);
    chart.set_x_axis_format(x_fmt);

    // Y axis: currency format, gridlines with custom color
    let mut y_fmt = AxisFormat::new();
    y_fmt.set_num_format("$#,##0");
    y_fmt.set_font_size(10.0);
    y_fmt.set_major_gridlines(true);
    y_fmt.set_gridline_color([200, 200, 200]);
    y_fmt.set_gridline_width(0.5);
    y_fmt.set_major_tick_mark(TickMark::None);
    chart.set_y_axis_format(y_fmt);

    chart.set_x_axis_name("Month");
    chart.set_y_axis_name("Revenue ($)");
    chart
        .add_series()
        .set_values("Revenue!$B$2:$B$7")
        .set_categories("Revenue!$A$2:$A$7")
        .set_name("Revenue")
        .set_color((70u8, 130u8, 180u8));

    ws.insert_chart(9, 0, &chart)?;

    wb.save(&path)?;
    println!("✅ Axis-formatted chart saved to {}", path.display());
    Ok(())
}
