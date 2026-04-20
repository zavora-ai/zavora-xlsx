//! Example: Create a map chart with geographic data (ChartEx format).
//!
//! Map charts use the cx: namespace and Bing Maps for rendering.
//! Excel will fetch geography data from Bing when the file is opened
//! (requires internet connection).
//!
//! Run with: cargo run --example map_chart

use zavora_xlsx::{MapChart, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = MapChart::new();
    chart.set_title("Global Sales by Country");
    chart.set_series_name("Sales ($M)");
    chart.add_point("United States", 450.0);
    chart.add_point("Germany", 180.0);
    chart.add_point("Japan", 220.0);
    chart.add_point("United Kingdom", 160.0);
    chart.add_point("France", 140.0);
    chart.add_point("Brazil", 95.0);
    chart.add_point("Australia", 85.0);
    chart.add_point("Canada", 120.0);

    ws.insert_map(0, 0, &chart).unwrap();
    wb.save("map_chart.xlsx").unwrap();
    println!("Created map_chart.xlsx");
}
