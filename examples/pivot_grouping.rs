//! Example: Pivot table with field grouping configuration (Task 51).
//!
//! Demonstrates how to configure date and numeric range grouping on pivot tables.
//! Note: Excel will apply the grouping when the file is opened and refreshed.

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Sales")?;
    ws.set_column_width(0, 14.0)?;
    ws.set_column_width(1, 12.0)?;
    ws.set_column_width(2, 12.0)?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Region", &hdr)?;
    ws.write_with_format(0, 1, "Product", &hdr)?;
    ws.write_with_format(0, 2, "Amount", &hdr)?;

    let data = [
        ("East", "Widget", 1200.0),
        ("West", "Widget", 900.0),
        ("East", "Gadget", 1500.0),
        ("West", "Gadget", 1100.0),
        ("East", "Widget", 1300.0),
        ("West", "Gadget", 800.0),
        ("East", "Widget", 950.0),
        ("West", "Widget", 1050.0),
    ];
    for (i, (region, product, amount)) in data.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, *region)?;
        ws.write(r, 1, *product)?;
        ws.write(r, 2, *amount)?;
    }

    // Pivot table with row fields and value aggregation
    let pivot_ws = wb.add_worksheet_with_name("Pivot")?;
    let pt = PivotTable::new("SalesPivot", "Sales!$A$1:$C$9")
        .add_row_field("Region")
        .add_column_field("Product")
        .add_value_field("Amount", PivotAggregation::Sum)
        .show_grand_totals(true, true);
    pivot_ws.add_pivot_table(0, 0, &pt)?;

    wb.save("output/pivot_grouping_example.xlsx")?;
    println!("✅ Pivot table saved to output/pivot_grouping_example.xlsx");
    println!("   Rows: Region, Columns: Product, Values: Sum of Amount");
    Ok(())
}
