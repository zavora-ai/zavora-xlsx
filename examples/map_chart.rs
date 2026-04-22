//! Example: Create a map chart showing sales by country (Task 32).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::path::PathBuf::from("output/map_chart_example.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Sales Data")?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Country", &hdr)?;
    ws.write_with_format(0, 1, "Revenue ($K)", &hdr)?;

    let data = [
        ("United States", 450.0),
        ("Canada", 120.0),
        ("Mexico", 95.0),
        ("Brazil", 180.0),
        ("United Kingdom", 210.0),
        ("Germany", 165.0),
        ("France", 140.0),
        ("Japan", 230.0),
    ];
    for (i, (country, revenue)) in data.iter().enumerate() {
        ws.write((i + 1) as u32, 0, *country)?;
        ws.write((i + 1) as u32, 1, *revenue)?;
    }

    let mut chart = MapChart::new();
    chart.set_title("Global Sales by Country");
    chart.set_series_name("Revenue");
    chart.set_map_level(MapLevel::Country);
    for (country, revenue) in &data {
        chart.add_point(country, *revenue);
    }
    ws.insert_map(12, 0, &chart)?;

    wb.save(&path)?;
    println!("✅ Map chart saved to {}", path.display());
    Ok(())
}
