//! Example: Pivot table with calculated items (Task 52).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Products")?;

    ws.write(0, 0, "Product")?;
    ws.write(0, 1, "Revenue")?;
    ws.write(0, 2, "Cost")?;

    let data = [
        ("Widget", 5000.0, 2000.0),
        ("Gadget", 8000.0, 3500.0),
        ("Doohickey", 3000.0, 1200.0),
        ("Thingamajig", 6500.0, 2800.0),
    ];
    for (i, (product, revenue, cost)) in data.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, *product)?;
        ws.write(r, 1, *revenue)?;
        ws.write(r, 2, *cost)?;
    }

    let pivot_ws = wb.add_worksheet_with_name("PivotCalc")?;
    let pt = PivotTable::new("CalcPivot", "Products!$A$1:$C$5")
        .add_row_field("Product")
        .add_value_field("Revenue", PivotAggregation::Sum)
        .add_calculated_item("Profit", "Revenue - Cost")
        .add_calculated_item("Margin", "(Revenue - Cost) / Revenue");
    pivot_ws.add_pivot_table(0, 0, &pt)?;

    wb.save("output/pivot_calculated_example.xlsx")?;
    println!("✅ Pivot calculated items saved to output/pivot_calculated_example.xlsx");
    Ok(())
}
