//! Example: Create a sunburst chart with 3-level hierarchy.
//!
//! Run with: cargo run --example sunburst_chart

use zavora_xlsx::{SunburstChart, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = SunburstChart::new();
    chart.set_title("Organization Revenue");
    // Level 1 (innermost): Regions
    chart.add_level(&["Americas", "Americas", "Americas", "EMEA", "EMEA", "APAC", "APAC"]);
    // Level 2: Countries
    chart.add_level(&["USA", "Canada", "Brazil", "UK", "Germany", "Japan", "Australia"]);
    // Level 3 (outermost): Cities
    chart.add_level(&["New York", "Toronto", "São Paulo", "London", "Berlin", "Tokyo", "Sydney"]);
    chart.set_values(&[50.0, 20.0, 15.0, 30.0, 25.0, 35.0, 18.0]);

    ws.insert_sunburst(0, 0, &chart).unwrap();
    wb.save("sunburst_chart.xlsx").unwrap();
    println!("Created sunburst_chart.xlsx");
}
