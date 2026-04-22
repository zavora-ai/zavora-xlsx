/// Example: Named Ranges CRUD operations
///
/// Demonstrates creating, updating, and deleting named ranges
/// with both workbook and sheet scopes.
use zavora_xlsx::{DefinedNameScope, Workbook};

fn main() {
    let mut wb = Workbook::new();

    // Add a second worksheet
    wb.add_worksheet_with_name("Sales").unwrap();

    // Write some data
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Tax Rate").unwrap();
    ws.write(0, 1, 0.08).unwrap();
    ws.write(1, 0, "Revenue").unwrap();
    ws.write(1, 1, 50000.0).unwrap();

    let ws = wb.worksheet(1).unwrap();
    ws.write(0, 0, "Q1").unwrap();
    ws.write(0, 1, 12000.0).unwrap();
    ws.write(1, 0, "Q2").unwrap();
    ws.write(1, 1, 15000.0).unwrap();
    ws.write(2, 0, "Q3").unwrap();
    ws.write(2, 1, 18000.0).unwrap();
    ws.write(3, 0, "Q4").unwrap();
    ws.write(3, 1, 20000.0).unwrap();

    // --- CREATE named ranges ---
    println!("Creating named ranges...");

    // Workbook-scoped name (visible everywhere)
    wb.add_named_range("TaxRate", "Sheet1!$B$1", DefinedNameScope::Workbook)
        .unwrap();

    // Another workbook-scoped name
    wb.add_named_range("TotalRevenue", "Sheet1!$B$2", DefinedNameScope::Workbook)
        .unwrap();

    // Sheet-scoped name (only visible in the Sales sheet)
    wb.add_named_range(
        "QuarterlySales",
        "'Sales'!$B$1:$B$4",
        DefinedNameScope::Sheet(1),
    )
    .unwrap();

    // A name we'll delete later
    wb.add_named_range("Temporary", "Sheet1!$A$1", DefinedNameScope::Workbook)
        .unwrap();

    println!("  Added: TaxRate (workbook scope)");
    println!("  Added: TotalRevenue (workbook scope)");
    println!("  Added: QuarterlySales (sheet scope, Sales)");
    println!("  Added: Temporary (workbook scope)");

    // --- UPDATE a named range ---
    println!("\nUpdating named ranges...");

    wb.update_named_range("TotalRevenue", "Sheet1!$B$2:$B$10")
        .unwrap();

    println!("  Updated: TotalRevenue formula -> Sheet1!$B$2:$B$10");

    // --- DELETE a named range ---
    println!("\nDeleting named ranges...");

    wb.remove_named_range("Temporary", &DefinedNameScope::Workbook)
        .unwrap();

    println!("  Removed: Temporary");

    // --- List remaining names ---
    println!("\nFinal defined names:");
    for dn in wb.defined_names_with_scope() {
        let scope_str = match &dn.scope {
            DefinedNameScope::Workbook => "Workbook".to_string(),
            DefinedNameScope::Sheet(idx) => format!("Sheet({})", idx),
        };
        println!("  {} = {} [scope: {}]", dn.name, dn.formula, scope_str);
    }

    // Save the workbook
    wb.save("output/named_ranges_example.xlsx").unwrap();
    println!("\nSaved to output/named_ranges_example.xlsx");
}
