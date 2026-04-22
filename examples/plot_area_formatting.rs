//! Example: Format plot area, series lines, and add error bars (Task 37).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::path::PathBuf::from("output/plot_area_formatting_example.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Data")?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Trial", &hdr)?;
    ws.write_with_format(0, 1, "Measurement A", &hdr)?;
    ws.write_with_format(0, 2, "Measurement B", &hdr)?;

    let trials = ["T1", "T2", "T3", "T4", "T5", "T6"];
    let a = [23.5, 28.1, 25.7, 31.2, 27.8, 33.4];
    let b = [18.2, 22.6, 20.1, 26.3, 24.5, 29.0];
    for (i, (t, (va, vb))) in trials.iter().zip(a.iter().zip(b.iter())).enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, *t)?;
        ws.write(r, 1, *va)?;
        ws.write(r, 2, *vb)?;
    }

    let mut chart = Chart::new(ChartType::Line);
    chart.set_title("Measurements with Error Bars");

    // Light blue plot area with gray border
    let mut pf = PlotAreaFormat::new();
    pf.set_fill([240, 248, 255]);
    pf.set_border([180, 180, 180]);
    chart.set_plot_area_format(pf);

    // Series A: thick dashed line with percentage error bars
    chart
        .add_series()
        .set_values("Data!$B$2:$B$7")
        .set_categories("Data!$A$2:$A$7")
        .set_name("Measurement A")
        .set_color((220u8, 50u8, 50u8))
        .set_line_width(2.5)
        .set_dash_style(DashStyle::Dash)
        .set_error_bars(ErrorBar::new(
            ErrorBarType::Both,
            ErrorBarValueType::Percentage,
            10.0,
        ));

    // Series B: solid line with fixed-value error bars
    chart
        .add_series()
        .set_values("Data!$C$2:$C$7")
        .set_categories("Data!$A$2:$A$7")
        .set_name("Measurement B")
        .set_color((50u8, 120u8, 200u8))
        .set_line_width(1.5)
        .set_error_bars(ErrorBar::new(
            ErrorBarType::Both,
            ErrorBarValueType::FixedValue,
            2.0,
        ));

    ws.insert_chart(9, 0, &chart)?;

    wb.save(&path)?;
    println!(
        "✅ Formatted chart with error bars saved to {}",
        path.display()
    );
    Ok(())
}
