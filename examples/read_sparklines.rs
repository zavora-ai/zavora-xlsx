//! Example: create sparklines, save, read back, and print sparkline properties.
//!
//! Run with: cargo run --example read_sparklines

use zavora_xlsx::{Sparkline, SparklineType, Workbook};

fn main() {
    // ── Create a workbook with sparklines ──
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write sample data
    let data = [10.0, 25.0, 15.0, 30.0, 20.0, 35.0];
    for (row, &val) in data.iter().enumerate() {
        ws.write(row as u32, 0, val).unwrap();
    }

    // Add a line sparkline in B1
    let sp_line = Sparkline::new("Sheet1!A1:A6", SparklineType::Line);
    ws.add_sparkline(0, 1, &sp_line).unwrap();

    // Add a column sparkline in B2 with a custom color
    let mut sp_col = Sparkline::new("Sheet1!A1:A6", SparklineType::Column);
    sp_col.set_color((0u8, 112u8, 192u8)); // Blue
    ws.add_sparkline(1, 1, &sp_col).unwrap();

    // Add a win/loss sparkline in B3
    let sp_wl = Sparkline::new("Sheet1!A1:A6", SparklineType::WinLoss);
    ws.add_sparkline(2, 1, &sp_wl).unwrap();

    // Save to buffer
    let buf = wb.save_to_buffer().unwrap();
    println!("Created workbook with 3 sparklines ({} bytes)", buf.len());

    // ── Read back and print sparkline properties ──
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let sparklines = ws2.sparklines();

    println!("\nFound {} sparkline(s):", sparklines.len());
    for (i, sp) in sparklines.iter().enumerate() {
        println!("\n  Sparkline {}:", i + 1);
        println!("    Type:       {:?}", sp.sparkline_type());
        println!("    Data range: {}", sp.data_range());
        println!("    Location:   row={}, col={}", sp.row(), sp.col());
        if let Some(color) = sp.color() {
            println!("    Color:      #{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
        } else {
            println!("    Color:      (default)");
        }
    }
}
