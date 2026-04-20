/// Example: create hyperlinks, read them back, and print link targets.
///
/// Demonstrates round-trip hyperlink reading for both external URLs
/// and internal sheet references.
use zavora_xlsx::Workbook;

fn main() {
    let path = "example_read_hyperlinks.xlsx";

    // ── Write a workbook with various hyperlinks ──
    {
        let mut wb = Workbook::new();
        wb.add_worksheet_with_name("Details").unwrap();

        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Link Target").unwrap();
        ws.write(0, 1, "Type").unwrap();

        // External URLs
        ws.write_url(1, 0, "https://www.rust-lang.org", "Rust Homepage")
            .unwrap();
        ws.write(1, 1, "External").unwrap();

        ws.write_url(2, 0, "https://docs.rs/zavora-xlsx", "API Docs")
            .unwrap();
        ws.write(2, 1, "External").unwrap();

        ws.write_url(3, 0, "https://github.com", "GitHub").unwrap();
        ws.write(3, 1, "External").unwrap();

        // Internal link to another sheet
        ws.write_internal_link(4, 0, "Details!A1", "Go to Details")
            .unwrap();
        ws.write(4, 1, "Internal").unwrap();

        wb.save(path).unwrap();
        println!("Wrote {path} with 4 hyperlinks.\n");
    }

    // ── Read the workbook back and inspect hyperlinks ──
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet_ref(0).unwrap();
        let links = ws.hyperlinks();

        println!(
            "Sheet '{}' contains {} hyperlink(s):\n",
            ws.name(),
            links.len()
        );

        for (i, hl) in links.iter().enumerate() {
            let kind = if hl.location.is_some() {
                "internal"
            } else {
                "external"
            };
            println!("  [{i}] row={}, col={}, type={kind}", hl.row, hl.col);
            if !hl.url.is_empty() {
                println!("       url = {}", hl.url);
            }
            if let Some(ref loc) = hl.location {
                println!("       location = {loc}");
            }
            if let Some(ref tip) = hl.tooltip {
                println!("       tooltip = {tip}");
            }
            println!();
        }
    }

    let _ = std::fs::remove_file(path);
}
