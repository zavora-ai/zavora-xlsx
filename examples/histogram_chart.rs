//! Example: Create histogram and Pareto charts.
//!
//! Run with: cargo run --example histogram_chart

use zavora_xlsx::{HistogramChart, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Histogram without custom bin count (let Excel decide)
    let mut hist = HistogramChart::new();
    hist.set_title("Response Time Distribution");
    hist.set_series_name("Latency (ms)");
    hist.set_values(&[
        12.0, 15.0, 18.0, 22.0, 25.0, 28.0, 30.0, 32.0, 35.0, 38.0,
        40.0, 42.0, 45.0, 50.0, 55.0, 60.0, 65.0, 80.0, 95.0, 120.0,
    ]);
    ws.insert_histogram(0, 0, &hist).unwrap();

    // Pareto chart on second sheet
    let ws2 = wb.add_worksheet_with_name("Pareto").unwrap();
    let mut pareto = HistogramChart::pareto();
    pareto.set_title("Defect Pareto Analysis");
    pareto.set_series_name("Defect Count");
    pareto.set_values(&[45.0, 30.0, 20.0, 15.0, 10.0, 8.0, 5.0, 3.0]);
    ws2.insert_histogram(0, 0, &pareto).unwrap();

    wb.save("histogram_chart.xlsx").unwrap();
    println!("Created histogram_chart.xlsx");
}
