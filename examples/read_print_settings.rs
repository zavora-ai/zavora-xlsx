/// Example: set print settings on a worksheet, save, read back, and print parsed values.
use zavora_xlsx::{Orientation, PrintSettings, Workbook};

fn main() -> zavora_xlsx::Result<()> {
    let path = "read_print_settings_example.xlsx";

    // ── Step 1: Create a workbook with print settings ──
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0)?;

        // Write some data
        ws.write(0, 0, "Department")?;
        ws.write(0, 1, "Q1")?;
        ws.write(0, 2, "Q2")?;
        ws.write(0, 3, "Q3")?;
        ws.write(0, 4, "Q4")?;
        for r in 1..=20 {
            ws.write(r, 0, format!("Dept {r}"))?;
            ws.write(r, 1, r as f64 * 1000.0)?;
            ws.write(r, 2, r as f64 * 1100.0)?;
            ws.write(r, 3, r as f64 * 1200.0)?;
            ws.write(r, 4, r as f64 * 1300.0)?;
        }

        // Configure print settings
        let ps = PrintSettings::new()
            .paper_size(9) // A4
            .orientation(Orientation::Landscape)
            .margins(1.0, 1.0, 0.75, 0.75)
            .header("&CQuarterly Report")
            .footer("&LConfidential&RPage &P of &N");
        ws.set_print_settings(&ps);

        // Set repeat rows (header row repeats on each printed page)
        ws.set_repeat_rows(0, 0);

        // Set page breaks
        ws.set_page_breaks(&[10], &[]);

        wb.save(path)?;
        println!("Created {path} with print settings.");
    }

    // ── Step 2: Read back and display parsed print settings ──
    {
        let mut wb = Workbook::open_readonly(path)?;
        let ws = wb.worksheet(0)?;

        println!("\n── Print Settings for '{}' ──", ws.name());

        match ws.print_settings() {
            Some(ps) => {
                // Orientation
                if let Some(ref orient) = ps.orientation {
                    println!("  Orientation: {:?}", orient);
                }

                // Paper size
                if let Some(size) = ps.paper_size {
                    let name = match size {
                        1 => "Letter",
                        9 => "A4",
                        5 => "Legal",
                        _ => "Other",
                    };
                    println!("  Paper size:  {} ({})", size, name);
                }

                // Margins
                println!("  Margins:");
                println!("    Top:    {:.2}", ps.margin_top.unwrap_or(0.75));
                println!("    Bottom: {:.2}", ps.margin_bottom.unwrap_or(0.75));
                println!("    Left:   {:.2}", ps.margin_left.unwrap_or(0.7));
                println!("    Right:  {:.2}", ps.margin_right.unwrap_or(0.7));
                println!("    Header: {:.2}", ps.margin_header.unwrap_or(0.3));
                println!("    Footer: {:.2}", ps.margin_footer.unwrap_or(0.3));

                // Header/Footer
                if let Some(ref h) = ps.header {
                    println!("  Header: {h}");
                }
                if let Some(ref f) = ps.footer {
                    println!("  Footer: {f}");
                }

                // Scale
                if let Some(scale) = ps.scale {
                    println!("  Scale: {scale}%");
                }

                // Fit to page
                if ps.fit_to_page {
                    println!(
                        "  Fit to page: {}w x {}h",
                        ps.fit_to_width.unwrap_or(1),
                        ps.fit_to_height.unwrap_or(1)
                    );
                }

                // Page breaks
                if !ps.row_breaks.is_empty() {
                    println!("  Row breaks: {:?}", ps.row_breaks);
                }
                if !ps.col_breaks.is_empty() {
                    println!("  Col breaks: {:?}", ps.col_breaks);
                }

                // Repeat rows/columns
                if let Some((first, last)) = ps.repeat_rows {
                    println!("  Repeat rows: {} to {}", first + 1, last + 1);
                }
                if let Some((first, last)) = ps.repeat_cols {
                    println!(
                        "  Repeat cols: {} to {}",
                        (b'A' + first as u8) as char,
                        (b'A' + last as u8) as char
                    );
                }
            }
            None => {
                println!("  No print settings found.");
            }
        }
    }

    // Clean up
    std::fs::remove_file(path).ok();
    println!("\nDone.");
    Ok(())
}
