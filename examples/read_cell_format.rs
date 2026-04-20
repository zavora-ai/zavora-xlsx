//! Example: Write a workbook with various cell formats, then read it back
//! and print the resolved format properties for each cell.
//!
//! This validates Task 1 (Read Cell Formatting) end-to-end.

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::env::temp_dir().join("read_cell_format_example.xlsx");

    // ── Step 1: Create a workbook with formatted cells ──
    println!("📝 Creating workbook with formatted cells...\n");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0)?;
        ws.set_name("Format Demo")?;
        ws.set_column_width(0, 25.0)?;
        ws.set_column_width(1, 30.0)?;

        // Row 0: Bold + Italic
        let fmt_bold_italic = Format::new().bold().italic().font_size(14.0);
        ws.write_with_format(0, 0, "Bold + Italic", &fmt_bold_italic)?;
        ws.write_with_format(0, 1, "Header text", &fmt_bold_italic)?;

        // Row 1: Font color (red)
        let fmt_red = Format::new().font_color((255u8, 0u8, 0u8));
        ws.write_with_format(1, 0, "Red font", &fmt_red)?;
        ws.write_with_format(1, 1, "This text is red", &fmt_red)?;

        // Row 2: Background color (yellow)
        let fmt_yellow_bg = Format::new().background_color((255u8, 255u8, 0u8));
        ws.write_with_format(2, 0, "Yellow background", &fmt_yellow_bg)?;
        ws.write_with_format(2, 1, "Highlighted cell", &fmt_yellow_bg)?;

        // Row 3: Thin borders on all sides
        let fmt_bordered = Format::new()
            .border(BorderStyle::Thin)
            .border_color((0u8, 0u8, 0u8));
        ws.write_with_format(3, 0, "Thin borders", &fmt_bordered)?;
        ws.write_with_format(3, 1, "Boxed cell", &fmt_bordered)?;

        // Row 4: Number format
        let fmt_currency = Format::new().num_format("#,##0.00");
        ws.write_with_format(4, 0, "Currency format", &Format::new())?;
        ws.write_with_format(4, 1, 12345.678, &fmt_currency)?;

        // Row 5: Center alignment + wrap text
        let fmt_centered = Format::new()
            .align(Align::Center)
            .align(Align::VerticalCenter)
            .text_wrap();
        ws.write_with_format(5, 0, "Centered + wrap", &fmt_centered)?;
        ws.write_with_format(5, 1, "This is\nwrapped text", &fmt_centered)?;

        // Row 6: Combined formatting
        let fmt_combined = Format::new()
            .bold()
            .font_size(12.0)
            .font_color((0u8, 100u8, 0u8))
            .background_color((220u8, 255u8, 220u8))
            .border(BorderStyle::Medium)
            .border_color((0u8, 100u8, 0u8))
            .align(Align::Center)
            .num_format("0.00%");
        ws.write_with_format(6, 0, "Combined", &Format::new().bold())?;
        ws.write_with_format(6, 1, 0.9525, &fmt_combined)?;

        // Row 7: Plain cell (no format)
        ws.write(7, 0, "Plain cell")?;
        ws.write(7, 1, "No formatting applied")?;

        wb.save(&path)?;
    }

    println!("💾 Saved to: {}\n", path.display());

    // ── Step 2: Read the workbook back and inspect formats ──
    println!("📖 Reading workbook back and inspecting cell formats...\n");
    println!("{:-<70}", "");

    let mut wb = Workbook::open_readonly(&path)?;
    let ws = wb.worksheet(0)?;

    let descriptions = [
        "Bold + Italic",
        "Red font",
        "Yellow background",
        "Thin borders",
        "Currency format",
        "Centered + wrap",
        "Combined",
        "Plain cell",
    ];

    for row in 0..8u32 {
        println!("Row {} — {}:", row, descriptions[row as usize]);

        match ws.cell_format(row, 1) {
            Some(fmt) => {
                // Font properties
                if fmt.is_bold() || fmt.is_italic() {
                    print!("  Font: ");
                    if fmt.is_bold() { print!("bold "); }
                    if fmt.is_italic() { print!("italic "); }
                    println!("size={}", fmt.get_font_size());
                }

                // Font color
                if let Some(color) = fmt.get_font_color() {
                    println!("  Font color: rgb({}, {}, {})", color[0], color[1], color[2]);
                }

                // Fill
                if let Some(fg) = fmt.get_fg_color() {
                    println!("  Fill fg: rgb({}, {}, {})", fg[0], fg[1], fg[2]);
                }
                if let Some(bg) = fmt.get_bg_color() {
                    println!("  Fill bg: rgb({}, {}, {})", bg[0], bg[1], bg[2]);
                }

                // Borders
                let bl = fmt.get_border_left();
                let br = fmt.get_border_right();
                let bt = fmt.get_border_top();
                let bb = fmt.get_border_bottom();
                if bl != BorderStyle::None || br != BorderStyle::None
                    || bt != BorderStyle::None || bb != BorderStyle::None
                {
                    println!("  Borders: left={:?} right={:?} top={:?} bottom={:?}", bl, br, bt, bb);
                }

                // Number format
                let nf = fmt.get_num_format();
                if !nf.is_empty() && nf != "General" {
                    println!("  Number format: \"{}\"", nf);
                }

                // Alignment
                let ha = fmt.get_h_align();
                let va = fmt.get_v_align();
                if ha != 0 || va != 0 || fmt.is_wrap_text() {
                    let h_name = match ha { 1 => "left", 2 => "center", 3 => "right", _ => "general" };
                    let v_name = match va { 1 => "top", 2 => "center", 3 => "bottom", _ => "default" };
                    print!("  Alignment: h={} v={}", h_name, v_name);
                    if fmt.is_wrap_text() { print!(" wrap=true"); }
                    println!();
                }
            }
            None => {
                println!("  (no format — default style)");
            }
        }
        println!();
    }

    println!("{:-<70}", "");
    println!("✅ Read Cell Formatting validation complete!");

    Ok(())
}
