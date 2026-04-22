//! Demonstrates the `#[derive(ExcelRow)]` proc macro for typed row mapping.
//!
//! Run with: `cargo run --example derive_macro_demo`

use zavora_xlsx::*;
use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow, Debug, PartialEq)]
struct Employee {
    #[excel(header = "Employee Name")]
    name: String,
    #[excel(header = "Department")]
    department: String,
    #[excel(header = "Salary", format = "#,##0.00")]
    salary: f64,
    #[excel(header = "Years")]
    years: u32,
    #[excel(header = "Active")]
    active: bool,
    #[excel(header = "Notes")]
    notes: Option<String>,
    #[excel(skip)]
    internal_id: String,
}

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let employees = vec![
        Employee {
            name: "Alice Johnson".into(),
            department: "Engineering".into(),
            salary: 95000.0,
            years: 5,
            active: true,
            notes: Some("Team lead".into()),
            internal_id: "EMP-001".into(),
        },
        Employee {
            name: "Bob Smith".into(),
            department: "Marketing".into(),
            salary: 72000.0,
            years: 3,
            active: true,
            notes: None,
            internal_id: "EMP-002".into(),
        },
        Employee {
            name: "Carol Davis".into(),
            department: "Finance".into(),
            salary: 88000.0,
            years: 8,
            active: false,
            notes: Some("On leave".into()),
            internal_id: "EMP-003".into(),
        },
        Employee {
            name: "David Lee".into(),
            department: "Engineering".into(),
            salary: 105000.0,
            years: 10,
            active: true,
            notes: Some("Senior architect".into()),
            internal_id: "EMP-004".into(),
        },
    ];

    // Write header row
    employees[0].write_header(ws, 0)?;

    // Write data rows
    for (i, emp) in employees.iter().enumerate() {
        emp.write_row(ws, (i + 1) as u32)?;
    }

    // Style the header row
    let header_fmt = Format::new()
        .bold()
        .font_color("#FFFFFF")
        .background_color("#2B579A")
        .border(BorderStyle::Thin);
    for col in 0..6u16 {
        ws.set_cell_format(0, col, &header_fmt)?;
    }

    ws.autofit()?;
    ws.set_freeze_panes(1, 0)?;

    // Read back and verify
    let headers: Vec<String> = vec![
        "Employee Name".into(),
        "Department".into(),
        "Salary".into(),
        "Years".into(),
        "Active".into(),
        "Notes".into(),
    ];

    println!("Written {} employees. Reading back...\n", employees.len());
    for row in 1..=employees.len() as u32 {
        let emp = Employee::read_row(ws, row, &headers)?;
        println!(
            "  Row {}: {} ({}) - ${:.2} - {} years{}",
            row,
            emp.name,
            emp.department,
            emp.salary,
            emp.years,
            emp.notes
                .as_ref()
                .map(|n| format!(" [{}]", n))
                .unwrap_or_default()
        );
        // internal_id should be default (empty) since it's skipped
        assert_eq!(emp.internal_id, "");
    }

    let path = std::path::PathBuf::from("output").join("derive_macro_demo.xlsx");
    std::fs::create_dir_all("output").ok();
    wb.save(&path)?;
    println!("\n✅ Saved to {}", path.display());

    Ok(())
}
