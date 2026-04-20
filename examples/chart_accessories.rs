//! Example: Chart accessories — drop lines, high-low lines.
//!
//! Run with: cargo run --example chart_accessories

use zavora_xlsx::{Chart, ChartType, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Stock").unwrap();

    ws.write(0, 0, "Day").unwrap();
    ws.write(0, 1, "High").unwrap();
    ws.write(0, 2, "Low").unwrap();
    ws.write(0, 3, "Close").unwrap();
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri"];
    let high = [155.0, 158.0, 160.0, 157.0, 162.0];
    let low = [148.0, 150.0, 153.0, 149.0, 155.0];
    let close = [152.0, 156.0, 155.0, 154.0, 160.0];
    for (i, (d, (h, (l, c)))) in days.iter().zip(high.iter().zip(low.iter().zip(close.iter()))).enumerate() {
        ws.write(i as u32 + 1, 0, *d).unwrap();
        ws.write(i as u32 + 1, 1, *h).unwrap();
        ws.write(i as u32 + 1, 2, *l).unwrap();
        ws.write(i as u32 + 1, 3, *c).unwrap();
    }

    // Line chart with drop lines and high-low lines
    let mut chart = Chart::new(ChartType::Line);
    chart.set_title("Stock Price (with Drop & High-Low Lines)");
    chart.set_drop_lines(true);
    chart.set_high_low_lines(true);

    let s = chart.add_series();
    s.set_categories("Stock!$A$2:$A$6").set_values("Stock!$B$2:$B$6").set_name("High");
    let s = chart.add_series();
    s.set_values("Stock!$C$2:$C$6").set_name("Low");
    let s = chart.add_series();
    s.set_values("Stock!$D$2:$D$6").set_name("Close");

    ws.insert_chart(7, 0, &chart).unwrap();
    wb.save("chart_accessories.xlsx").unwrap();
    println!("Created chart_accessories.xlsx");
}
