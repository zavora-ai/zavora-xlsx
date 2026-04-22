//! Example: Create a bubble chart with sample data.
//!
//! Demonstrates using ChartType::Bubble with bubble_sizes to create
//! a bubble chart where each data point has X, Y, and size dimensions.

use zavora_xlsx::{Chart, ChartType, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Bubble Data").unwrap();

    // Headers
    ws.write(0, 0, "X Values").unwrap();
    ws.write(0, 1, "Y Values").unwrap();
    ws.write(0, 2, "Bubble Sizes").unwrap();

    // Data points
    let x_values = [1.0, 2.5, 3.0, 4.5, 5.0, 6.5];
    let y_values = [10.0, 25.0, 15.0, 30.0, 20.0, 35.0];
    let sizes = [5.0, 10.0, 8.0, 15.0, 12.0, 20.0];

    for (i, ((x, y), s)) in x_values
        .iter()
        .zip(y_values.iter())
        .zip(sizes.iter())
        .enumerate()
    {
        ws.write(i as u32 + 1, 0, *x).unwrap();
        ws.write(i as u32 + 1, 1, *y).unwrap();
        ws.write(i as u32 + 1, 2, *s).unwrap();
    }

    // Create bubble chart
    let mut chart = Chart::new(ChartType::Bubble);
    chart.set_title("Sales Performance");
    chart.set_x_axis_name("Revenue ($M)");
    chart.set_y_axis_name("Growth (%)");

    let series = chart.add_series();
    series
        .set_name("Products")
        .set_categories("'Bubble Data'!$A$2:$A$7")
        .set_values("'Bubble Data'!$B$2:$B$7")
        .set_bubble_sizes("'Bubble Data'!$C$2:$C$7");

    ws.insert_chart(8, 0, &chart).unwrap();

    wb.save("bubble_chart.xlsx").unwrap();
    println!("Created bubble_chart.xlsx with a bubble chart.");
}
