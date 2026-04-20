//! Example: Write a workbook with various conditional formatting rules,
//! then read it back and print the parsed CF rules.
//!
//! This validates Task 2 (Read Conditional Formatting) end-to-end.

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::env::temp_dir().join("read_cf_example.xlsx");

    // ── Step 1: Create a workbook with conditional formatting ──
    println!("📝 Creating workbook with conditional formatting rules...\n");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0)?;
        ws.set_name("CF Demo")?;
        ws.set_column_width(0, 20.0)?;
        ws.set_column_width(1, 15.0)?;

        // Write some sample data
        let values = [95.0, 42.0, 78.0, 15.0, 88.0, 55.0, 30.0, 67.0, 12.0, 99.0];
        for (i, &v) in values.iter().enumerate() {
            ws.write(i as u32, 0, v)?;
            ws.write(i as u32, 1, v * 1.1)?;
        }

        // Rule 1: Cell value > 80 → bold red font
        let mut rule1 = ConditionalFormatCell::new(CfOperator::GreaterThan, 80.0);
        rule1.set_format(&Format::new().bold().font_color((255u8, 0u8, 0u8)));
        ws.add_conditional_format(0, 0, 9, 0, rule1)?;

        // Rule 2: Cell value < 20 → italic blue font
        let mut rule2 = ConditionalFormatCell::new(CfOperator::LessThan, 20.0);
        rule2.set_format(&Format::new().italic().font_color((0u8, 0u8, 255u8)));
        ws.add_conditional_format(0, 0, 9, 0, rule2)?;

        // Rule 3: 2-color scale (red → green) on column B
        let rule3 = ConditionalFormat2ColorScale::new(
            (255u8, 99u8, 71u8),  // tomato red (min)
            (34u8, 139u8, 34u8),  // forest green (max)
        );
        ws.add_conditional_format(0, 1, 9, 1, rule3)?;

        // Rule 4: Data bar (blue) on column A
        let rule4 = ConditionalFormatDataBar::new((70u8, 130u8, 180u8)); // steel blue
        ws.add_conditional_format(0, 0, 9, 0, rule4)?;

        // Rule 5: Duplicate values → yellow background
        let mut rule5 = ConditionalFormatDuplicate::new();
        rule5.set_format(&Format::new().background_color((255u8, 255u8, 0u8)));
        ws.add_conditional_format(0, 0, 9, 0, rule5)?;

        // Rule 6: Top 3 values → green background
        let mut rule6 = ConditionalFormatTopBottom::new(TopBottomType::Top, 3);
        rule6.set_format(&Format::new().background_color((144u8, 238u8, 144u8)));
        ws.add_conditional_format(0, 1, 9, 1, rule6)?;

        // Rule 7: Icon set (3 traffic lights) on column B
        let rule7 = ConditionalFormatIconSet::new(IconSetType::ThreeTrafficLights);
        ws.add_conditional_format(0, 1, 9, 1, rule7)?;

        wb.save(&path)?;
    }

    println!("💾 Saved to: {}\n", path.display());

    // ── Step 2: Read the workbook back and inspect CF rules ──
    println!("📖 Reading workbook back and inspecting conditional formats...\n");
    println!("{:-<70}", "");

    let mut wb = Workbook::open_readonly(&path)?;
    let ws = wb.worksheet(0)?;
    let cfs = ws.conditional_formats();

    println!("Found {} conditional formatting rule(s):\n", cfs.len());

    for (i, cf) in cfs.iter().enumerate() {
        let (r1, c1, r2, c2) = cf.range;
        let range_str = format!(
            "{}{}:{}{}",
            col_letter(c1), r1 + 1,
            col_letter(c2), r2 + 1
        );

        println!(
            "  Rule {}: type=\"{}\" range={} dxf_id={:?}",
            i + 1,
            cf.rule.cf_type(),
            range_str,
            cf.dxf_id
        );
    }

    println!("\n{:-<70}", "");
    println!("✅ Read Conditional Formatting validation complete!");

    Ok(())
}

/// Convert a 0-based column number to a letter (A, B, C, ...).
fn col_letter(col: u16) -> String {
    let mut result = String::new();
    let mut c = col as u32;
    loop {
        result.insert(0, (b'A' + (c % 26) as u8) as char);
        if c < 26 { break; }
        c = c / 26 - 1;
    }
    result
}
