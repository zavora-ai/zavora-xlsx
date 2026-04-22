//! Example: Apply named cell styles like Heading 1, Currency, Percent (Task 42).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Cell Styles")?;
    ws.set_column_width(0, 25.0)?;
    ws.set_column_width(1, 20.0)?;

    // Title style
    let title = Format::new().bold().font_size(18.0).cell_style("Title");
    ws.write_with_format(0, 0, "Financial Report", &title)?;

    // Heading styles
    let h1 = Format::new().bold().font_size(14.0).cell_style("Heading 1");
    ws.write_with_format(2, 0, "Revenue", &h1)?;

    let h2 = Format::new().bold().font_size(12.0).cell_style("Heading 2");
    ws.write_with_format(3, 0, "Q1 Sales", &h2)?;

    // Currency style
    let currency = Format::new().num_format("$#,##0.00").cell_style("Currency");
    ws.write_with_format(4, 0, "Amount", &Format::new())?;
    ws.write_with_format(4, 1, 45678.90, &currency)?;

    // Percent style
    let percent = Format::new().num_format("0.0%").cell_style("Percent");
    ws.write_with_format(5, 0, "Growth", &Format::new())?;
    ws.write_with_format(5, 1, 0.125, &percent)?;

    // Good / Bad / Neutral
    let good = Format::new()
        .font_color((0u8, 128u8, 0u8))
        .cell_style("Good");
    let bad = Format::new()
        .font_color((255u8, 0u8, 0u8))
        .cell_style("Bad");
    let neutral = Format::new().cell_style("Neutral");

    ws.write_with_format(7, 0, "Status: Good", &good)?;
    ws.write_with_format(8, 0, "Status: Bad", &bad)?;
    ws.write_with_format(9, 0, "Status: Neutral", &neutral)?;

    // Total style
    let total = Format::new().bold().cell_style("Total");
    ws.write_with_format(11, 0, "Total", &total)?;
    ws.write_with_format(
        11,
        1,
        45678.90,
        &Format::new()
            .bold()
            .num_format("$#,##0.00")
            .cell_style("Total"),
    )?;

    wb.save("output/cell_styles_example.xlsx")?;
    println!("✅ Cell styles saved to output/cell_styles_example.xlsx");
    Ok(())
}
