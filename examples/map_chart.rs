//! Example: Create a map chart with geographic data.
//!
//! Run with: cargo run --example map_chart

use zavora_xlsx::{Chart, ChartType, MapLevel, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Map Data").unwrap();

    ws.write(0, 0, "Country").unwrap();
    ws.write(0, 1, "Sales ($M)").unwrap();
    let data = [
        ("United States", 450.0), ("Germany", 180.0), ("Japan", 220.0),
        ("United Kingdom", 160.0), ("France", 140.0), ("Brazil", 95.0),
        ("Australia", 85.0), ("Canada", 120.0),
    ];
    for (i, (country, sales)) in data.iter().enumerate() {
        ws.write(i as u32 + 1, 0, *country).unwrap();
        ws.write(i as u32 + 1, 1, *sales).unwrap();
    }

    let mut chart = Chart::new(ChartType::Map);
    chart.set_title("Global Sales by Country");
    chart.set_map_level(MapLevel::Country);
    let series = chart.add_series();
    series
        .set_categories("'Map Data'!$A$2:$A$9")
        .set_values("'Map Data'!$B$2:$B$9")
        .set_name("Sales");

    ws.insert_chart(10, 0, &chart).unwrap();
    wb.save("map_chart.xlsx").unwrap();
    println!("Created map_chart.xlsx");
}
