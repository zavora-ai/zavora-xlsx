//! Example: Create 3D chart variants with custom view settings.
//!
//! Run with: cargo run --example chart_3d

use zavora_xlsx::{Chart, ChartType, View3D, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Sales").unwrap();

    ws.write(0, 0, "Quarter").unwrap();
    ws.write(0, 1, "Product A").unwrap();
    ws.write(0, 2, "Product B").unwrap();
    let quarters = ["Q1", "Q2", "Q3", "Q4"];
    let prod_a = [120.0, 150.0, 180.0, 200.0];
    let prod_b = [90.0, 110.0, 130.0, 160.0];
    for (i, (q, (a, b))) in quarters.iter().zip(prod_a.iter().zip(prod_b.iter())).enumerate() {
        ws.write(i as u32 + 1, 0, *q).unwrap();
        ws.write(i as u32 + 1, 1, *a).unwrap();
        ws.write(i as u32 + 1, 2, *b).unwrap();
    }

    // 3D Column chart
    let mut chart = Chart::new(ChartType::Column3D);
    chart.set_title("3D Column Chart");
    chart.set_view3d(View3D { rot_x: 20, rot_y: 30, perspective: 25, right_angle_axes: false });
    let s = chart.add_series();
    s.set_categories("Sales!$A$2:$A$5").set_values("Sales!$B$2:$B$5").set_name("Product A");
    let s = chart.add_series();
    s.set_values("Sales!$C$2:$C$5").set_name("Product B");
    ws.insert_chart(6, 0, &chart).unwrap();

    // 3D Pie chart on second sheet
    let ws2 = wb.add_worksheet_with_name("Pie").unwrap();
    ws2.write(0, 0, "Category").unwrap();
    ws2.write(0, 1, "Value").unwrap();
    for (i, (cat, val)) in [("Desktop", 45.0), ("Mobile", 35.0), ("Tablet", 20.0)].iter().enumerate() {
        ws2.write(i as u32 + 1, 0, *cat).unwrap();
        ws2.write(i as u32 + 1, 1, *val).unwrap();
    }
    let mut pie = Chart::new(ChartType::Pie3D);
    pie.set_title("Device Share (3D Pie)");
    pie.set_view3d(View3D { rot_x: 30, rot_y: 0, perspective: 30, right_angle_axes: true });
    let s = pie.add_series();
    s.set_categories("Pie!$A$2:$A$4").set_values("Pie!$B$2:$B$4");
    ws2.insert_chart(5, 0, &pie).unwrap();

    wb.save("chart_3d.xlsx").unwrap();
    println!("Created chart_3d.xlsx");
}
