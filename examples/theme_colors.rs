//! Example: Apply theme colors with tint adjustments (Task 40).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Theme Colors")?;
    ws.set_column_width(0, 16.0)?;
    ws.set_column_width(1, 10.0)?;
    ws.set_column_width(2, 22.0)?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Theme", &hdr)?;
    ws.write_with_format(0, 1, "Tint", &hdr)?;
    ws.write_with_format(0, 2, "Sample", &hdr)?;

    let colors = [
        ("Accent1", ThemeColorIndex::Accent1),
        ("Accent2", ThemeColorIndex::Accent2),
        ("Accent3", ThemeColorIndex::Accent3),
        ("Accent4", ThemeColorIndex::Accent4),
        ("Accent5", ThemeColorIndex::Accent5),
        ("Accent6", ThemeColorIndex::Accent6),
        ("Dark1", ThemeColorIndex::Dark1),
        ("Dark2", ThemeColorIndex::Dark2),
    ];

    let mut row = 1u32;
    for (name, idx) in &colors {
        for &tint in &[0.0, 0.4, -0.25] {
            let fmt = Format::new().bold().font_size(12.0).theme_color(*idx, tint);
            ws.write(row, 0, *name)?;
            ws.write(row, 1, tint)?;
            ws.write_with_format(row, 2, format!("{} tint={}", name, tint), &fmt)?;
            row += 1;
        }
    }

    wb.save("output/theme_colors_example.xlsx")?;
    println!("✅ Theme colors saved to output/theme_colors_example.xlsx");
    Ok(())
}
