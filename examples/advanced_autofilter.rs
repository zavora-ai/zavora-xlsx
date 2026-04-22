//! Example: Advanced autofilter with Top10, date, and custom filters (Task 54).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    // Sheet 1: Top 10 filter
    let ws = wb.worksheet(0)?;
    ws.set_name("Top10")?;
    ws.set_column_width(0, 12.0)?;
    ws.set_column_width(1, 12.0)?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Student", &hdr)?;
    ws.write_with_format(0, 1, "Score", &hdr)?;

    for i in 1..=20u32 {
        ws.write(i, 0, format!("Student {}", i))?;
        ws.write(i, 1, (i as f64) * 4.5 + 10.0)?;
    }

    ws.set_autofilter(0, 0, 20, 1);
    ws.filter_column_advanced(
        1,
        FilterRule::Top10 {
            top: true,
            percent: false,
            val: 5.0,
        },
    );

    // Sheet 2: Date filter
    let ws2 = wb.add_worksheet_with_name("DateFilter")?;
    ws2.set_column_width(0, 14.0)?;
    ws2.set_column_width(1, 12.0)?;

    ws2.write_with_format(0, 0, "Date", &hdr)?;
    ws2.write_with_format(0, 1, "Amount", &hdr)?;

    let dates = [
        "2024-01-15",
        "2024-03-20",
        "2024-06-10",
        "2024-09-05",
        "2024-12-18",
    ];
    let amounts = [100.0, 250.0, 175.0, 300.0, 225.0];
    for (i, (date, amount)) in dates.iter().zip(amounts.iter()).enumerate() {
        let r = (i + 1) as u32;
        ws2.write(r, 0, *date)?;
        ws2.write(r, 1, *amount)?;
    }

    ws2.set_autofilter(0, 0, 5, 1);
    ws2.filter_column_advanced(
        0,
        FilterRule::DateFilter {
            year: 2024,
            month: Some(6),
            day: None,
        },
    );

    // Sheet 3: Custom AND filter
    let ws3 = wb.add_worksheet_with_name("CustomFilter")?;
    ws3.set_column_width(0, 12.0)?;

    ws3.write_with_format(0, 0, "Value", &hdr)?;
    for i in 1..=10u32 {
        ws3.write(i, 0, (i as f64) * 10.0)?;
    }

    ws3.set_autofilter(0, 0, 10, 0);
    ws3.filter_column_advanced(
        0,
        FilterRule::CustomFilter {
            and: true,
            conditions: vec![
                ("greaterThan".into(), "30".into()),
                ("lessThan".into(), "80".into()),
            ],
        },
    );

    wb.save("output/advanced_autofilter_example.xlsx")?;
    println!("✅ Advanced autofilter saved to output/advanced_autofilter_example.xlsx");
    println!("   Sheet 1: Top 5 scores");
    println!("   Sheet 2: June 2024 dates");
    println!("   Sheet 3: Values between 30 and 80");
    Ok(())
}
