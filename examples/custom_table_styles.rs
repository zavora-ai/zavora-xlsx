//! Example: Create a table with a custom style (Task 43).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Custom Table")?;

    // Write data
    ws.write(0, 0, "Employee")?;
    ws.write(0, 1, "Department")?;
    ws.write(0, 2, "Salary")?;

    let data = [
        ("Alice", "Engineering", 95000.0),
        ("Bob", "Marketing", 72000.0),
        ("Carol", "Engineering", 88000.0),
        ("Dave", "Sales", 68000.0),
        ("Eve", "Marketing", 76000.0),
        ("Frank", "Engineering", 102000.0),
    ];
    for (i, (name, dept, salary)) in data.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, *name)?;
        ws.write(r, 1, *dept)?;
        ws.write(r, 2, *salary)?;
    }

    // Custom table style with blue header and alternating gray/white stripes
    let custom_style = CustomTableStyle::new("CompanyBlue")
        .header_row(
            TableStyleElementFormat::new()
                .bg_color([30, 60, 120])
                .font_color([255, 255, 255])
                .bold(),
        )
        .first_row_stripe(TableStyleElementFormat::new().bg_color([230, 235, 245]))
        .second_row_stripe(TableStyleElementFormat::new().bg_color([255, 255, 255]))
        .first_row_stripe_size(1)
        .second_row_stripe_size(1);

    let mut table = Table::new();
    table.set_columns(&[
        TableColumn::new("Employee"),
        TableColumn::new("Department"),
        TableColumn::new("Salary"),
    ]);
    table.set_custom_style(custom_style);
    ws.add_table(0, 0, 6, 2, &table)?;

    ws.set_column_width(0, 15.0)?;
    ws.set_column_width(1, 15.0)?;
    ws.set_column_width(2, 12.0)?;

    wb.save("output/custom_table_styles_example.xlsx")?;
    println!("✅ Custom table style saved to output/custom_table_styles_example.xlsx");
    Ok(())
}
