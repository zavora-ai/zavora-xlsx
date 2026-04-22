//! Example: create tables, save, read back, and print table properties.
//!
//! Run with: cargo run --example read_tables

use zavora_xlsx::{Table, TableColumn, TableStyle, Workbook};

fn main() {
    // ── Create a workbook with two tables ──
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // First table: employee data with autofilter
    ws.write(0, 0, "Name").unwrap();
    ws.write(0, 1, "Department").unwrap();
    ws.write(0, 2, "Salary").unwrap();
    ws.write(1, 0, "Alice").unwrap();
    ws.write(1, 1, "Engineering").unwrap();
    ws.write(1, 2, 95000).unwrap();
    ws.write(2, 0, "Bob").unwrap();
    ws.write(2, 1, "Marketing").unwrap();
    ws.write(2, 2, 78000).unwrap();
    ws.write(3, 0, "Carol").unwrap();
    ws.write(3, 1, "Engineering").unwrap();
    ws.write(3, 2, 102000).unwrap();

    let cols1 = vec![
        TableColumn::new("Name"),
        TableColumn::new("Department"),
        TableColumn::new("Salary"),
    ];
    let mut table1 = Table::new();
    table1.set_columns(&cols1);
    table1.set_style(TableStyle::Medium(9));
    table1.set_name("Employees");
    ws.add_table(0, 0, 3, 2, &table1).unwrap();

    // Second table: sales data with total row
    ws.write(0, 4, "Product").unwrap();
    ws.write(0, 5, "Revenue").unwrap();
    ws.write(1, 4, "Widget A").unwrap();
    ws.write(1, 5, 15000).unwrap();
    ws.write(2, 4, "Widget B").unwrap();
    ws.write(2, 5, 23000).unwrap();
    ws.write(3, 4, "Widget C").unwrap();
    ws.write(3, 5, 8500).unwrap();

    let mut col_product = TableColumn::new("Product");
    col_product.set_total_label("Total");
    let mut col_revenue = TableColumn::new("Revenue");
    col_revenue.set_total_function("sum");

    let mut table2 = Table::new();
    table2.set_columns(&[col_product, col_revenue]);
    table2.set_style(TableStyle::Dark(2));
    table2.set_total_row(true);
    table2.set_name("Sales");
    ws.add_table(0, 4, 3, 5, &table2).unwrap();

    // ── Save and read back ──
    let buf = wb.save_to_buffer().unwrap();
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();

    // ── Print table properties ──
    println!("Sheet: {}", ws2.name());
    println!("Tables found: {}\n", ws2.tables().len());

    for (i, table) in ws2.tables().iter().enumerate() {
        println!("── Table {} ──", i + 1);
        println!(
            "  Name:       {}",
            table.table_name().unwrap_or("(unnamed)")
        );
        println!(
            "  Range:      ({},{}) to ({},{})",
            table.first_row(),
            table.first_col(),
            table.last_row(),
            table.last_col()
        );
        println!("  Autofilter: {}", table.autofilter());
        println!("  Total row:  {}", table.total_row());

        if let Some(style) = table.style() {
            println!("  Style:      {}", style.name());
        }

        println!("  Columns:");
        for col in table.columns() {
            print!("    - {}", col.name());
            if let Some(label) = col.total_label() {
                print!("  [total_label: {}]", label);
            }
            if let Some(func) = col.total_function() {
                print!("  [total_function: {}]", func);
            }
            println!();
        }
        println!();
    }
}
