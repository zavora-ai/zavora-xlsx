//! Example: Stream-read an xlsx file and print row count and sample data.
//!
//! Usage: cargo run --example streaming_read

use zavora_xlsx::{StreamingReader, StreamingWorkbook};

fn main() {
    // First, create a sample file to read
    let mut wb = StreamingWorkbook::new();
    wb.write_string(0, 0, "Name").unwrap();
    wb.write_string(0, 1, "Score").unwrap();
    wb.write_string(0, 2, "Pass").unwrap();

    let names = ["Alice", "Bob", "Charlie", "Diana", "Eve"];
    let scores = [95.0, 82.0, 78.0, 91.0, 88.0];

    for (i, (name, score)) in names.iter().zip(scores.iter()).enumerate() {
        let row = (i + 1) as u32;
        wb.write_string(row, 0, name).unwrap();
        wb.write_number(row, 1, *score).unwrap();
        wb.write_boolean(row, 2, *score >= 80.0).unwrap();
    }

    wb.save("output/streaming_read_sample.xlsx").unwrap();
    println!("Created sample file: output/streaming_read_sample.xlsx");

    // Now stream-read it
    let reader = StreamingReader::open("output/streaming_read_sample.xlsx").unwrap();
    println!("\nSheets: {:?}", reader.sheet_names());
    println!("Sheet count: {}", reader.sheet_count());

    for sheet_idx in 0..reader.sheet_count() {
        let sheet_name = &reader.sheet_names()[sheet_idx];
        let rows = reader.sheet_rows(sheet_idx).unwrap();
        let row_count = rows.row_count();
        println!("\n--- Sheet '{}': {} rows ---", sheet_name, row_count);

        for row in reader.sheet_rows(sheet_idx).unwrap() {
            print!("  Row {}: ", row.row_index);
            for cell in &row.cells {
                print!(
                    "[col={} val={:?} xf={}] ",
                    cell.col, cell.value, cell.xf_index
                );
            }
            println!();
        }
    }
}
