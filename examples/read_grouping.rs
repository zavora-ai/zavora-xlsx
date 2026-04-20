/// Example: Group rows and columns, save, read back, and print outline levels.
///
/// Demonstrates the round-trip of row/column grouping (outline levels) in zavora-xlsx.
use zavora_xlsx::Workbook;

fn main() {
    let path = "read_grouping_example.xlsx";

    // ── Write a workbook with grouped rows and columns ──
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();

        // Header row
        ws.write(0, 0, "Region").unwrap();
        ws.write(0, 1, "Q1").unwrap();
        ws.write(0, 2, "Q2").unwrap();
        ws.write(0, 3, "Q3").unwrap();
        ws.write(0, 4, "Q4").unwrap();
        ws.write(0, 5, "Total").unwrap();

        // Data rows
        ws.write(1, 0, "North").unwrap();
        ws.write(2, 0, "South").unwrap();
        ws.write(3, 0, "East").unwrap();
        ws.write(4, 0, "West").unwrap();
        ws.write(5, 0, "Summary").unwrap();

        for r in 1..=4 {
            for c in 1..=4 {
                ws.write(r, c, (r * 10 + c as u32) as f64 * 100.0).unwrap();
            }
        }

        // Group detail rows (rows 1-4) at level 1
        ws.group_rows(1, 4, 1);

        // Group quarterly columns (Q1-Q4, cols 1-4) at level 1
        ws.group_columns(1, 4, 1);

        wb.save(path).unwrap();
        println!("Saved workbook with grouped rows and columns to {path}");
    }

    // ── Read back and print outline levels ──
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet_ref(0).unwrap();

        println!("\n── Row Outline Levels ──");
        let row_levels = ws.row_outline_levels();
        if row_levels.is_empty() {
            println!("  (no row grouping)");
        } else {
            for (row, level) in row_levels {
                println!("  Row {row}: outline level {level}");
            }
        }

        println!("\n── Column Outline Levels ──");
        let col_levels = ws.col_outline_levels();
        if col_levels.is_empty() {
            println!("  (no column grouping)");
        } else {
            for (col, level) in col_levels {
                let letter = (b'A' + *col as u8) as char;
                println!("  Column {letter} ({col}): outline level {level}");
            }
        }
    }

    std::fs::remove_file(path).ok();
}
