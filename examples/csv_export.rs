//! Example: Export a worksheet to CSV and TSV formats.

use zavora_xlsx::{CsvOptions, Workbook};

fn main() -> zavora_xlsx::Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Write some sample data
    ws.write(0, 0, "Name")?;
    ws.write(0, 1, "Age")?;
    ws.write(0, 2, "City")?;
    ws.write(1, 0, "Alice")?;
    ws.write(1, 1, 30)?;
    ws.write(1, 2, "New York")?;
    ws.write(2, 0, "Bob")?;
    ws.write(2, 1, 25)?;
    ws.write(2, 2, "San Francisco")?;
    ws.write(3, 0, "Charlie, Jr.")?; // Contains comma — will be quoted
    ws.write(3, 1, 35)?;
    ws.write(3, 2, "Chicago")?;

    // Export as CSV with default options
    let csv_opts = CsvOptions::default();
    let csv_output = ws.to_csv_string(&csv_opts);
    println!("=== CSV Output ===");
    print!("{csv_output}");

    // Export as TSV
    let tsv_opts = CsvOptions::tsv();
    let tsv_output = ws.to_csv_string(&tsv_opts);
    println!("=== TSV Output ===");
    print!("{tsv_output}");

    // Export to file
    ws.to_csv_file("output/export.csv", &csv_opts)?;
    println!("\nSaved CSV to output/export.csv");

    // Also save the workbook
    wb.save("output/csv_export_source.xlsx")?;

    Ok(())
}
