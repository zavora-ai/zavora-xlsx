//! Example: Format chart plot area, series styling, and error bars.
//!
//! Run with: cargo run --example plot_area_formatting

use zavora_xlsx::{
    Chart, ChartType, DashStyle, ErrorBar, ErrorBarType, ErrorBarValueType,
    PlotAreaFormat, Workbook,
};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    ws.write(0, 0, "Category").unwrap();
    ws.write(0, 1, "Actual").unwrap();
    ws.write(0, 2, "Target").unwrap();
    let cats = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon"];
    let actual = [85.0, 92.0, 78.0, 95.0, 88.0];
    let target = [80.0, 85.0, 80.0, 90.0, 85.0];
    for (i, (c, (a, t))) in cats.iter().zip(actual.iter().zip(target.iter())).enumerate() {
        ws.write(i as u32 + 1, 0, *c).unwrap();
        ws.write(i as u32 + 1, 1, *a).unwrap();
        ws.write(i as u32 + 1, 2, *t).unwrap();
    }

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Performance vs Target");

    // Plot area with light gray fill
    let mut pf = PlotAreaFormat::new();
    pf.fill = Some([245, 245, 245]);
    pf.border = Some([180, 180, 180]);
    chart.set_plot_area_format(pf);

    // Actual series with gradient and error bars
    let s = chart.add_series();
    s.set_categories("Data!$A$2:$A$6")
     .set_values("Data!$B$2:$B$6")
     .set_name("Actual")
     .set_color((70, 130, 180))
     .set_error_bars(ErrorBar::new(ErrorBarType::Both, ErrorBarValueType::Percentage, 5.0));

    // Target series with dashed outline
    let s = chart.add_series();
    s.set_values("Data!$C$2:$C$6")
     .set_name("Target")
     .set_color((220, 80, 60))
     .set_dash_style(DashStyle::Dash);

    ws.insert_chart(7, 0, &chart).unwrap();
    wb.save("plot_area_formatting.xlsx").unwrap();
    println!("Created plot_area_formatting.xlsx");
}
