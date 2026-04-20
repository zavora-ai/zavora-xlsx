//! Example: Create surface and wireframe surface charts.
//!
//! Run with: cargo run --example surface_chart

use zavora_xlsx::{Chart, ChartType, View3D, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Surface").unwrap();

    // Generate a 5x5 grid of Z values: z = sin(x) * cos(y)
    ws.write(0, 0, "").unwrap();
    for c in 0..5u16 {
        ws.write(0, c + 1, format!("Y={}", c)).unwrap();
    }
    for r in 0..5u32 {
        ws.write(r + 1, 0, format!("X={}", r)).unwrap();
        for c in 0..5u16 {
            let x = r as f64 * 0.5;
            let y = c as f64 * 0.5;
            let z = (x.sin() * y.cos() * 100.0).round() / 100.0;
            ws.write(r + 1, c + 1, z).unwrap();
        }
    }

    let mut chart = Chart::new(ChartType::Surface);
    chart.set_title("Surface Chart: sin(x)*cos(y)");
    chart.set_view3d(View3D { rot_x: 25, rot_y: 30, perspective: 20, right_angle_axes: false });
    for c in 0..5 {
        let col_letter = (b'B' + c as u8) as char;
        let s = chart.add_series();
        s.set_categories("Surface!$A$2:$A$6")
         .set_values(&format!("Surface!${col_letter}$2:${col_letter}$6"))
         .set_name(&format!("Y={c}"));
    }
    ws.insert_chart(7, 0, &chart).unwrap();

    // Wireframe variant
    let ws2 = wb.add_worksheet_with_name("Wireframe").unwrap();
    for r in 0..5u32 {
        for c in 0..5u16 {
            let x = r as f64 * 0.5;
            let y = c as f64 * 0.5;
            ws2.write(r, c, (x * y).sqrt()).unwrap();
        }
    }
    let mut wire = Chart::new(ChartType::WireframeSurface);
    wire.set_title("Wireframe Surface");
    for c in 0..5 {
        let col_letter = (b'A' + c as u8) as char;
        let s = wire.add_series();
        s.set_values(&format!("Wireframe!${col_letter}$1:${col_letter}$5"));
    }
    ws2.insert_chart(6, 0, &wire).unwrap();

    wb.save("surface_chart.xlsx").unwrap();
    println!("Created surface_chart.xlsx");
}
