//! Demonstrates `Workbook::recalculate()` — the formula evaluation engine.
//!
//! Run with: `cargo run --example formula_recalculate`

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // ── Build a small financial model ──

    let header = Format::new()
        .bold()
        .font_color("#FFFFFF")
        .background_color("#2B579A")
        .border(BorderStyle::Thin);
    let currency = Format::new().num_format("$#,##0.00");
    let pct = Format::new().num_format("0.0%");

    // Headers
    ws.write_with_format(0, 0, "Item", &header)?;
    ws.write_with_format(0, 1, "Q1", &header)?;
    ws.write_with_format(0, 2, "Q2", &header)?;
    ws.write_with_format(0, 3, "Q3", &header)?;
    ws.write_with_format(0, 4, "Q4", &header)?;
    ws.write_with_format(0, 5, "Total", &header)?;
    ws.write_with_format(0, 6, "Avg", &header)?;

    // Revenue data
    ws.write(1, 0, "Revenue")?;
    ws.write_with_format(1, 1, 50000.0, &currency)?;
    ws.write_with_format(1, 2, 62000.0, &currency)?;
    ws.write_with_format(1, 3, 58000.0, &currency)?;
    ws.write_with_format(1, 4, 71000.0, &currency)?;

    // Cost data
    ws.write(2, 0, "Costs")?;
    ws.write_with_format(2, 1, 30000.0, &currency)?;
    ws.write_with_format(2, 2, 35000.0, &currency)?;
    ws.write_with_format(2, 3, 32000.0, &currency)?;
    ws.write_with_format(2, 4, 38000.0, &currency)?;

    // Profit = Revenue - Costs (formulas)
    ws.write(3, 0, "Profit")?;
    ws.write_formula(3, 1, "B2-B3")?;
    ws.write_formula(3, 2, "C2-C3")?;
    ws.write_formula(3, 3, "D2-D3")?;
    ws.write_formula(3, 4, "E2-E3")?;

    // Totals (SUM formulas)
    ws.write_formula(1, 5, "SUM(B2:E2)")?;
    ws.write_formula(2, 5, "SUM(B3:E3)")?;
    ws.write_formula(3, 5, "SUM(B4:E4)")?;

    // Averages (AVERAGE formulas)
    ws.write_formula(1, 6, "AVERAGE(B2:E2)")?;
    ws.write_formula(2, 6, "AVERAGE(B3:E3)")?;
    ws.write_formula(3, 6, "AVERAGE(B4:E4)")?;

    // Margin % = Profit / Revenue
    ws.write(5, 0, "Margin %")?;
    ws.write_formula(5, 1, "B4/B2")?;
    ws.write_formula(5, 2, "C4/C2")?;
    ws.write_formula(5, 3, "D4/D2")?;
    ws.write_formula(5, 4, "E4/E2")?;
    ws.write_formula(5, 5, "F4/F2")?;

    // Apply formats
    for col in 1..=6u16 {
        ws.set_cell_format(3, col, &currency)?;
    }
    for col in 1..=5u16 {
        ws.set_cell_format(5, col, &pct)?;
    }

    ws.autofit()?;
    ws.set_freeze_panes(1, 1)?;

    // ── Recalculate all formulas ──

    println!("Recalculating formulas...");
    let count = wb.recalculate()?;
    println!("Evaluated {} formula cells.\n", count);

    // ── Read back and display results ──

    let ws = wb.worksheet(0)?;

    println!("Financial Model Results:");
    println!("{:-<60}", "");

    let items = ["Revenue", "Costs", "Profit"];
    let rows = [1u32, 2, 3];
    for (item, &row) in items.iter().zip(rows.iter()) {
        print!("{:<10}", item);
        for col in 1..=6u16 {
            let val = ws.read_cell(row, col);
            match val {
                CellValue::Number(n) => print!("  {:>10.0}", n),
                CellValue::Formula { cached_value, .. } => {
                    if let CellValue::Number(n) = *cached_value {
                        print!("  {:>10.0}", n);
                    } else {
                        print!("  {:>10}", "?");
                    }
                }
                _ => print!("  {:>10}", "-"),
            }
        }
        println!();
    }

    println!("{:-<60}", "");
    print!("{:<10}", "Margin %");
    for col in 1..=5u16 {
        let val = ws.read_cell(5, col);
        match val {
            CellValue::Formula { cached_value, .. } => {
                if let CellValue::Number(n) = *cached_value {
                    print!("  {:>9.1}%", n * 100.0);
                } else {
                    print!("  {:>10}", "?");
                }
            }
            _ => print!("  {:>10}", "-"),
        }
    }
    println!("\n");

    // Verify some values
    let profit_q1 = ws.read_cell(3, 1);
    if let CellValue::Formula { cached_value, .. } = &profit_q1
        && let CellValue::Number(n) = **cached_value
    {
        assert_eq!(n, 20000.0, "Q1 Profit should be 50000 - 30000 = 20000");
        println!("✓ Q1 Profit = ${:.0} (correct)", n);
    }

    let total_revenue = ws.read_cell(1, 5);
    if let CellValue::Formula { cached_value, .. } = &total_revenue
        && let CellValue::Number(n) = **cached_value
    {
        assert_eq!(
            n, 241000.0,
            "Total Revenue should be 50000+62000+58000+71000"
        );
        println!("✓ Total Revenue = ${:.0} (correct)", n);
    }

    let avg_cost = ws.read_cell(2, 6);
    if let CellValue::Formula { cached_value, .. } = &avg_cost
        && let CellValue::Number(n) = **cached_value
    {
        assert_eq!(n, 33750.0, "Avg Cost should be (30000+35000+32000+38000)/4");
        println!("✓ Average Cost = ${:.0} (correct)", n);
    }

    // Save
    let path = std::path::PathBuf::from("output").join("formula_recalculate.xlsx");
    std::fs::create_dir_all("output").ok();
    wb.save(&path)?;
    println!("\n✅ Saved to {}", path.display());

    Ok(())
}
