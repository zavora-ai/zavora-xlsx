//! Example: Gradient vs solid data bars in conditional formatting (Task 41).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Data Bars")?;
    ws.set_column_width(0, 20.0)?;
    ws.set_column_width(1, 20.0)?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Gradient Bar", &hdr)?;
    ws.write_with_format(0, 1, "Solid Bar", &hdr)?;

    let values = [15.0, 35.0, 60.0, 80.0, 45.0, 95.0, 25.0, 70.0];
    for (i, &v) in values.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, v)?;
        ws.write(r, 1, v)?;
    }

    // Gradient data bar (blue)
    let mut gradient_bar = ConditionalFormatDataBar::new((70u8, 130u8, 180u8));
    gradient_bar.set_gradient(true);
    ws.add_conditional_format(1, 0, 8, 0, gradient_bar)?;

    // Solid data bar (green) — default is solid
    let solid_bar = ConditionalFormatDataBar::new((34u8, 139u8, 34u8));
    ws.add_conditional_format(1, 1, 8, 1, solid_bar)?;

    wb.save("output/gradient_data_bars_example.xlsx")?;
    println!("✅ Gradient data bars saved to output/gradient_data_bars_example.xlsx");
    Ok(())
}
