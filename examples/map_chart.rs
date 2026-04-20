//! Example: Map chart demonstration.
//!
//! Map charts require Bing Maps geoCache data that Excel generates
//! when you create a map chart interactively. This example creates
//! a workbook with data suitable for a map chart.
//!
//! To create the actual map chart:
//! 1. Open the generated file in Excel
//! 2. Select the data range (A1:B9)
//! 3. Insert → Map Chart → Filled Map
//!
//! Run with: cargo run --example map_chart

use zavora_xlsx::Workbook;

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Map Data").unwrap();

    // Headers
    ws.write(0, 0, "Country").unwrap();
    ws.write(0, 1, "Sales ($M)").unwrap();

    // Data for map chart
    let data = [
        ("United States", 450.0),
        ("Germany", 180.0),
        ("Japan", 220.0),
        ("United Kingdom", 160.0),
        ("France", 140.0),
        ("Brazil", 95.0),
        ("Australia", 85.0),
        ("Canada", 120.0),
    ];

    for (i, (country, sales)) in data.iter().enumerate() {
        ws.write(i as u32 + 1, 0, *country).unwrap();
        ws.write(i as u32 + 1, 1, *sales).unwrap();
    }

    wb.save("map_chart.xlsx").unwrap();
    println!("Created map_chart.xlsx with data for a map chart.");
    println!("Open in Excel, select A1:B9, then Insert → Map Chart → Filled Map");
}
