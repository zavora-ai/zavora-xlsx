//! Example: Create a box and whisker chart comparing distributions.
//!
//! Run with: cargo run --example box_whisker_chart

use zavora_xlsx::{BoxWhiskerChart, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = BoxWhiskerChart::new();
    chart.set_title("Test Score Distribution by Class");
    chart.set_show_outliers(true);
    chart.set_show_mean_markers(true);
    chart.set_show_inner_points(false);
    chart.add_data_set("Class A", &[72.0, 85.0, 90.0, 68.0, 95.0, 78.0, 88.0, 92.0, 45.0, 82.0]);
    chart.add_data_set("Class B", &[65.0, 70.0, 75.0, 80.0, 85.0, 60.0, 72.0, 78.0, 82.0, 88.0]);
    chart.add_data_set("Class C", &[55.0, 60.0, 65.0, 70.0, 90.0, 95.0, 50.0, 75.0, 80.0, 85.0]);

    ws.insert_box_whisker(0, 0, &chart).unwrap();
    wb.save("box_whisker_chart.xlsx").unwrap();
    println!("Created box_whisker_chart.xlsx");
}
