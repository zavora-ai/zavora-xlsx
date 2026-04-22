//! Serde round-trip example: serialize structs to xlsx, deserialize back, print results.
//!
//! Run with: `cargo run --example serde_roundtrip --features serde-support`

#[cfg(feature = "serde-support")]
fn main() {
    use serde::{Deserialize, Serialize};
    use zavora_xlsx::Workbook;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Employee {
        name: String,
        department: String,
        salary: f64,
        active: bool,
    }

    let data = vec![
        Employee {
            name: "Alice Johnson".into(),
            department: "Engineering".into(),
            salary: 95000.0,
            active: true,
        },
        Employee {
            name: "Bob Smith".into(),
            department: "Marketing".into(),
            salary: 72000.0,
            active: true,
        },
        Employee {
            name: "Carol Williams".into(),
            department: "Engineering".into(),
            salary: 105000.0,
            active: false,
        },
    ];

    // Serialize to xlsx
    let mut wb = Workbook::new();
    wb.write_rows(0, &data).unwrap();
    let buf = wb.save_to_buffer().unwrap();
    std::fs::write("output/serde_roundtrip.xlsx", &buf).unwrap();
    println!(
        "Wrote {} employees to output/serde_roundtrip.xlsx",
        data.len()
    );

    // Deserialize back
    let mut wb2 = Workbook::open_from_buffer(&buf).unwrap();
    let employees: Vec<Employee> = wb2.read_rows(0).unwrap();

    println!("\nDeserialized {} employees:", employees.len());
    for emp in &employees {
        println!(
            "  {} | {} | ${:.0} | active={}",
            emp.name, emp.department, emp.salary, emp.active
        );
    }

    // Verify round-trip
    assert_eq!(data.len(), employees.len());
    for (orig, read) in data.iter().zip(employees.iter()) {
        assert_eq!(orig.name, read.name);
        assert_eq!(orig.department, read.department);
        assert_eq!(orig.salary, read.salary);
        assert_eq!(orig.active, read.active);
    }
    println!("\nRound-trip verification passed!");
}

#[cfg(not(feature = "serde-support"))]
fn main() {
    eprintln!("This example requires the `serde-support` feature flag.");
    eprintln!("Run with: cargo run --example serde_roundtrip --features serde-support");
}
