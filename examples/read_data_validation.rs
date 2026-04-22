//! Example: Write a workbook with various data validation rules,
//! then read it back and print the parsed validation rules.
//!
//! This validates Task 3 (Read Data Validation) end-to-end.

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::path::PathBuf::from("output/read_dv_example.xlsx");

    // ── Step 1: Create a workbook with data validations ──
    println!("📝 Creating workbook with data validation rules...\n");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0)?;
        ws.set_name("Validation Demo")?;
        ws.set_column_width(0, 18.0)?;
        ws.set_column_width(1, 18.0)?;
        ws.set_column_width(2, 18.0)?;
        ws.set_column_width(3, 18.0)?;
        ws.set_column_width(4, 18.0)?;

        // Headers
        let hdr = Format::new().bold();
        ws.write_with_format(0, 0, "Dropdown", &hdr)?;
        ws.write_with_format(0, 1, "Integer (1-100)", &hdr)?;
        ws.write_with_format(0, 2, "Decimal (0-99.99)", &hdr)?;
        ws.write_with_format(0, 3, "Max 50 chars", &hdr)?;
        ws.write_with_format(0, 4, "Custom formula", &hdr)?;

        // Rule 1: List validation on column A
        let mut dv1 = DataValidation::new(ValidationRule::List(vec![
            "Apple".into(),
            "Banana".into(),
            "Cherry".into(),
            "Date".into(),
        ]));
        dv1.set_input_message("Fruit", "Pick a fruit from the list");
        dv1.set_error_message(
            ErrorStyle::Stop,
            "Invalid!",
            "Please select from the dropdown",
        );
        ws.add_data_validation(1, 0, 10, 0, &dv1)?;

        // Rule 2: Whole number between 1 and 100 on column B
        let mut dv2 = DataValidation::new(ValidationRule::WholeNumber {
            min: Some(1),
            max: Some(100),
        });
        dv2.set_input_message("Number", "Enter a whole number 1-100");
        dv2.set_error_message(ErrorStyle::Warning, "Out of range", "Value should be 1-100");
        ws.add_data_validation(1, 1, 10, 1, &dv2)?;

        // Rule 3: Decimal between 0 and 99.99 on column C
        let dv3 = DataValidation::new(ValidationRule::Decimal {
            min: Some(0.0),
            max: Some(99.99),
        });
        ws.add_data_validation(1, 2, 10, 2, &dv3)?;

        // Rule 4: Text length max 50 on column D
        let dv4 = DataValidation::new(ValidationRule::TextLength {
            min: Some(1),
            max: Some(50),
        });
        ws.add_data_validation(1, 3, 10, 3, &dv4)?;

        // Rule 5: Custom formula on column E
        let mut dv5 = DataValidation::new(ValidationRule::Custom("AND(E2>0,E2<1000)".into()));
        dv5.set_error_message(
            ErrorStyle::Information,
            "Note",
            "Value should be between 0 and 1000",
        );
        ws.add_data_validation(1, 4, 10, 4, &dv5)?;

        wb.save(&path)?;
    }

    println!("💾 Saved to: {}\n", path.display());

    // ── Step 2: Read the workbook back and inspect validations ──
    println!("📖 Reading workbook back and inspecting data validations...\n");
    println!("{:-<70}", "");

    let mut wb = Workbook::open_readonly(&path)?;
    let ws = wb.worksheet(0)?;
    let dvs = ws.validations();

    println!("Found {} data validation rule(s):\n", dvs.len());

    for (i, dv) in dvs.iter().enumerate() {
        let range_str = format!(
            "{}{}:{}{}",
            col_letter(dv.first_col()),
            dv.first_row() + 1,
            col_letter(dv.last_col()),
            dv.last_row() + 1
        );

        println!("  Rule {} — range={}", i + 1, range_str);

        // Print rule type and details
        match dv.rule() {
            ValidationRule::List(items) => {
                println!("    Type: List [{}]", items.join(", "));
            }
            ValidationRule::ListRange(range) => {
                println!("    Type: ListRange ({})", range);
            }
            ValidationRule::WholeNumber { min, max } => {
                println!("    Type: WholeNumber min={:?} max={:?}", min, max);
            }
            ValidationRule::Decimal { min, max } => {
                println!("    Type: Decimal min={:?} max={:?}", min, max);
            }
            ValidationRule::DateRange { min, max } => {
                println!("    Type: DateRange min={:?} max={:?}", min, max);
            }
            ValidationRule::TextLength { min, max } => {
                println!("    Type: TextLength min={:?} max={:?}", min, max);
            }
            ValidationRule::Custom(formula) => {
                println!("    Type: Custom formula=\"{}\"", formula);
            }
        }

        // Print messages if present
        if let Some(title) = dv.input_title() {
            println!(
                "    Input: title=\"{}\" msg=\"{}\"",
                title,
                dv.input_message().unwrap_or("")
            );
        }
        if let Some(title) = dv.error_title() {
            println!(
                "    Error: style={:?} title=\"{}\" msg=\"{}\"",
                dv.error_style(),
                title,
                dv.error_message().unwrap_or("")
            );
        }

        println!();
    }

    println!("{:-<70}", "");
    println!("✅ Read Data Validation validation complete!");

    Ok(())
}

/// Convert a 0-based column number to a letter (A, B, C, ...).
fn col_letter(col: u16) -> String {
    let mut result = String::new();
    let mut c = col as u32;
    loop {
        result.insert(0, (b'A' + (c % 26) as u8) as char);
        if c < 26 {
            break;
        }
        c = c / 26 - 1;
    }
    result
}
