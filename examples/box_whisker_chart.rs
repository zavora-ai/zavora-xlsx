//! Example: Create a box and whisker chart comparing school test scores.
//!
//! Run with: cargo run --example box_whisker_chart

use zavora_xlsx::{BoxWhiskerChart, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write the data table (matching the reference)
    ws.write(0, 0, "Course").unwrap();
    ws.write(0, 1, "School A").unwrap();
    ws.write(0, 2, "School B").unwrap();
    ws.write(0, 3, "School C").unwrap();

    let data: &[(&str, f64, f64, f64)] = &[
        ("English", 63.0, 53.0, 45.0),
        ("Physics", 61.0, 55.0, 65.0),
        ("English", 63.0, 50.0, 65.0),
        ("Math",    62.0, 51.0, 64.0),
        ("English", 46.0, 53.0, 66.0),
        ("English", 58.0, 56.0, 67.0),
        ("Math",    60.0, 51.0, 67.0),
        ("Math",    62.0, 53.0, 66.0),
        ("English", 63.0, 54.0, 64.0),
        ("English", 63.0, 52.0, 67.0),
        ("Physics", 60.0, 56.0, 64.0),
        ("English", 60.0, 56.0, 67.0),
        ("Math",    61.0, 56.0, 45.0),
        ("Math",    63.0, 58.0, 64.0),
        ("English", 59.0, 54.0, 65.0),
    ];

    for (i, (course, a, b, c)) in data.iter().enumerate() {
        let row = i as u32 + 1;
        ws.write(row, 0, *course).unwrap();
        ws.write(row, 1, *a).unwrap();
        ws.write(row, 2, *b).unwrap();
        ws.write(row, 3, *c).unwrap();
    }

    // Create box & whisker chart
    let school_a: Vec<f64> = data.iter().map(|d| d.1).collect();
    let school_b: Vec<f64> = data.iter().map(|d| d.2).collect();
    let school_c: Vec<f64> = data.iter().map(|d| d.3).collect();

    let mut chart = BoxWhiskerChart::new();
    chart.set_title("Test Score Distribution by School");
    chart.set_show_outliers(true);
    chart.set_show_mean_markers(true);
    chart.add_data_set("School A", &school_a);
    chart.add_data_set("School B", &school_b);
    chart.add_data_set("School C", &school_c);

    ws.insert_box_whisker(17, 0, &chart).unwrap();
    wb.save("box_whisker_chart.xlsx").unwrap();
    println!("Created box_whisker_chart.xlsx");
}
