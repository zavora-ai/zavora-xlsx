//! Example: Apply text effects — shadow, outline, emboss, engrave (Task 45).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Text Effects")?;
    ws.set_column_width(0, 30.0)?;

    let hdr = Format::new().bold().font_size(14.0);
    ws.write_with_format(0, 0, "Text Effects Demo", &hdr)?;

    // Shadow
    let shadow_fmt = Format::new().font_size(14.0).shadow();
    ws.write_with_format(2, 0, "Shadow text", &shadow_fmt)?;

    // Outline
    let outline_fmt = Format::new().font_size(14.0).outline();
    ws.write_with_format(3, 0, "Outline text", &outline_fmt)?;

    // Emboss
    let emboss_fmt = Format::new().font_size(14.0).emboss();
    ws.write_with_format(4, 0, "Emboss text", &emboss_fmt)?;

    // Engrave
    let engrave_fmt = Format::new().font_size(14.0).engrave();
    ws.write_with_format(5, 0, "Engrave text", &engrave_fmt)?;

    // Combined: bold + shadow + outline
    let combined = Format::new()
        .bold()
        .font_size(16.0)
        .font_color((0u8, 0u8, 128u8))
        .shadow()
        .outline();
    ws.write_with_format(7, 0, "Bold + Shadow + Outline", &combined)?;

    // All effects at once
    let all = Format::new()
        .font_size(14.0)
        .shadow()
        .outline()
        .emboss()
        .engrave();
    ws.write_with_format(9, 0, "All effects combined", &all)?;

    wb.save("output/text_effects_example.xlsx")?;
    println!("✅ Text effects saved to output/text_effects_example.xlsx");
    Ok(())
}
