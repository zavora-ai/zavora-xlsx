//! Example: Create a funnel chart showing a sales pipeline.
//!
//! Run with: cargo run --example funnel_chart

use zavora_xlsx::{FunnelChart, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = FunnelChart::new();
    chart.set_title("Sales Pipeline");
    chart.set_series_name("Leads");
    chart.add_point("Prospects", 1000.0);
    chart.add_point("Qualified", 600.0);
    chart.add_point("Proposals", 350.0);
    chart.add_point("Negotiations", 180.0);
    chart.add_point("Closed Won", 95.0);

    ws.insert_funnel(0, 0, &chart).unwrap();
    wb.save("funnel_chart.xlsx").unwrap();
    println!("Created funnel_chart.xlsx");
}
