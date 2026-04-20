//! Example: Apply built-in chart style themes.
//!
//! Run with: cargo run --example chart_style

use zavora_xlsx::{Chart, ChartType, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    ws.write(0, 0, "Month").unwrap();
    ws.write(0, 1, "Revenue").unwrap();
    ws.write(0, 2, "Profit").unwrap();
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
    let revenue = [100.0, 120.0, 115.0, 140.0, 155.0, 170.0];
    let profit = [20.0, 25.0, 22.0, 35.0, 40.0, 48.0];
    for (i, (m, (r, p))) in months.iter().zip(revenue.iter().zip(profit.iter())).enumerate() {
        ws.write(i as u32 + 1, 0, *m).unwrap();
        ws.write(i as u32 + 1, 1, *r).unwrap();
        ws.write(i as u32 + 1, 2, *p).unwrap();
    }

    // Chart with style 10
    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Revenue & Profit (Style 10)");
    chart.set_style(10);
    let s = chart.add_series();
    s.set_categories("Data!$A$2:$A$7").set_values("Data!$B$2:$B$7").set_name("Revenue");
    let s = chart.add_series();
    s.set_values("Data!$C$2:$C$7").set_name("Profit");
    ws.insert_chart(8, 0, &chart).unwrap();

    wb.save("chart_style.xlsx").unwrap();
    println!("Created chart_style.xlsx");
}
