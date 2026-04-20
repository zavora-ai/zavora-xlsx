/// Example: Create named ranges with different scopes, read them back, and print scope info.
///
/// Demonstrates:
/// - Creating workbook-scoped (global) defined names
/// - Creating sheet-scoped (local) defined names
/// - Reading back defined names with scope information
/// - Using the legacy `defined_names()` API for backward compatibility

use zavora_xlsx::{DefinedNameScope, Workbook};

fn main() -> zavora_xlsx::Result<()> {
    let path = "defined_names_example.xlsx";

    // ── Create a workbook with various defined names ──
    let mut wb = Workbook::new();
    wb.add_worksheet_with_name("Sales")?;
    wb.add_worksheet_with_name("Budget")?;

    // Write some data so the ranges reference real cells
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "Sheet1 data")?;

    let ws = wb.worksheet(1)?;
    ws.write(0, 0, "Q1")?;
    ws.write(0, 1, 10000)?;

    let ws = wb.worksheet(2)?;
    ws.write(0, 0, "Category")?;
    ws.write(0, 1, 5000)?;

    // Global defined names (visible everywhere)
    wb.define_name("TaxRate", "0.08");
    wb.define_name("CompanyName", "\"Acme Corp\"");

    // Sheet-scoped defined names (same name, different sheets)
    wb.define_name_scoped("DataRange", "'Sales'!$A$1:$B$100", 1);
    wb.define_name_scoped("DataRange", "'Budget'!$A$1:$B$50", 2);

    wb.save(path)?;
    println!("Saved workbook to {path}\n");

    // ── Read back and inspect defined names ──
    let wb = Workbook::open_readonly(path)?;

    println!("=== Defined Names with Scope ===");
    for dn in wb.defined_names_with_scope() {
        let scope_str = match &dn.scope {
            DefinedNameScope::Workbook => "Workbook (global)".to_string(),
            DefinedNameScope::Sheet(idx) => {
                let sheet_name = wb.sheet_names().get(*idx).map(|s| *s).unwrap_or("?");
                format!("Sheet {} (\"{}\")", idx, sheet_name)
            }
        };
        println!("  Name: {:<15} Formula: {:<30} Scope: {}", dn.name, dn.formula, scope_str);
    }

    println!("\n=== Legacy API (name, formula pairs) ===");
    for (name, formula) in wb.defined_names() {
        println!("  {name} = {formula}");
    }

    // Clean up
    std::fs::remove_file(path).ok();

    Ok(())
}
