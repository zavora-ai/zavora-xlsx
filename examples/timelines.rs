/// Example: Create a workbook with a pivot table and a timeline filter.
use zavora_xlsx::{PivotAggregation, PivotTable, Timeline, TimelineLevel, Workbook};

fn main() -> zavora_xlsx::Result<()> {
    let mut wb = Workbook::new();

    // Create source data sheet with date-based data
    let ws = wb.worksheet(0)?;
    ws.set_name("SalesData")?;
    ws.write(0, 0, "Date")?;
    ws.write(0, 1, "Product")?;
    ws.write(0, 2, "Amount")?;

    // Write some sample date data
    let dates = [
        "2024-01-15",
        "2024-02-20",
        "2024-03-10",
        "2024-04-05",
        "2024-05-18",
        "2024-06-22",
        "2024-07-30",
        "2024-08-14",
        "2024-09-25",
        "2024-10-08",
        "2024-11-12",
        "2024-12-01",
    ];
    let products = [
        "Widget", "Gadget", "Widget", "Gadget", "Widget", "Gadget", "Widget", "Gadget", "Widget",
        "Gadget", "Widget", "Gadget",
    ];
    let amounts = [
        100.0, 250.0, 175.0, 300.0, 225.0, 150.0, 275.0, 350.0, 200.0, 125.0, 400.0, 180.0,
    ];

    for (i, (date, (product, amount))) in dates
        .iter()
        .zip(products.iter().zip(amounts.iter()))
        .enumerate()
    {
        let row = (i + 1) as u32;
        ws.write(row, 0, *date)?;
        ws.write(row, 1, *product)?;
        ws.write(row, 2, *amount)?;
    }

    // Create a pivot table on a new sheet
    let pivot_ws = wb.add_worksheet_with_name("PivotSheet")?;

    let pt = PivotTable::new("SalesPivot", "SalesData!$A$1:$C$13")
        .add_row_field("Product")
        .add_value_field("Amount", PivotAggregation::Sum);

    pivot_ws.add_pivot_table(0, 0, &pt)?;

    // Add a timeline filter for the Date field
    let timeline = Timeline::new("DateTimeline", "Date")
        .set_caption("Filter by Date")
        .set_level(TimelineLevel::Months);

    pivot_ws.add_timeline(15, 0, &timeline)?;

    wb.save("output/timelines_example.xlsx")?;
    println!("Created output/timelines_example.xlsx with timeline filter");
    Ok(())
}
