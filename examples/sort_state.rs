//! Example: Set sort state on a worksheet (Task 53).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Employees")?;
    ws.set_column_width(0, 15.0)?;
    ws.set_column_width(1, 12.0)?;
    ws.set_column_width(2, 12.0)?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Name", &hdr)?;
    ws.write_with_format(0, 1, "Department", &hdr)?;
    ws.write_with_format(0, 2, "Salary", &hdr)?;

    let data = [
        ("Alice", "Engineering", 95000.0),
        ("Bob", "Marketing", 72000.0),
        ("Charlie", "Engineering", 88000.0),
        ("Diana", "Sales", 68000.0),
        ("Eve", "Marketing", 76000.0),
        ("Frank", "Engineering", 102000.0),
        ("Grace", "Sales", 71000.0),
    ];
    for (i, (name, dept, salary)) in data.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, *name)?;
        ws.write(r, 1, *dept)?;
        ws.write(r, 2, *salary)?;
    }

    // Set autofilter and sort by salary descending
    ws.set_autofilter(0, 0, 7, 2);
    ws.set_sort(2, SortDirection::Descending);

    wb.save("output/sort_state_example.xlsx")?;
    println!("✅ Sort state saved to output/sort_state_example.xlsx");
    println!("   Sorted by Salary (descending)");
    Ok(())
}
