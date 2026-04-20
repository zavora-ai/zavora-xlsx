//! Example: Create a waterfall chart showing revenue breakdown.
//!
//! Run with: cargo run --example waterfall_chart

use zavora_xlsx::{WaterfallChart, WaterfallPointType, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = WaterfallChart::new();
    chart.set_title("Q4 Revenue Breakdown");
    chart.set_series_name("Revenue");
    chart.add_point("Product Sales", 120000.0, WaterfallPointType::Increase);
    chart.add_point("Services", 45000.0, WaterfallPointType::Increase);
    chart.add_point("Returns", -8000.0, WaterfallPointType::Decrease);
    chart.add_point("Discounts", -12000.0, WaterfallPointType::Decrease);
    chart.add_point("Gross Revenue", 145000.0, WaterfallPointType::Total);
    chart.add_point("Operating Costs", -60000.0, WaterfallPointType::Decrease);
    chart.add_point("Net Revenue", 85000.0, WaterfallPointType::Total);

    ws.insert_waterfall(0, 0, &chart).unwrap();
    wb.save("waterfall_chart.xlsx").unwrap();
    println!("Created waterfall_chart.xlsx");
}
