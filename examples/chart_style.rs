//! Example: Apply chart style themes (Task 35).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::path::PathBuf::from("output/chart_style_example.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Styles")?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Product", &hdr)?;
    ws.write_with_format(0, 1, "Sales", &hdr)?;
    let products = ["Widget A", "Widget B", "Widget C", "Widget D"];
    let sales = [340.0, 520.0, 280.0, 610.0];
    for (i, (p, s)) in products.iter().zip(sales.iter()).enumerate() {
        ws.write((i + 1) as u32, 0, *p)?;
        ws.write((i + 1) as u32, 1, *s)?;
    }

    // Style 2 — clean, minimal
    let mut chart1 = Chart::new(ChartType::Column);
    chart1.set_title("Style 2");
    chart1.set_style(2);
    chart1
        .add_series()
        .set_values("Styles!$B$2:$B$5")
        .set_categories("Styles!$A$2:$A$5");
    ws.insert_chart(7, 0, &chart1)?;

    // Style 26 — colorful
    let mut chart2 = Chart::new(ChartType::Column);
    chart2.set_title("Style 26");
    chart2.set_style(26);
    chart2
        .add_series()
        .set_values("Styles!$B$2:$B$5")
        .set_categories("Styles!$A$2:$A$5");
    ws.insert_chart(7, 5, &chart2)?;

    // Style 42 — dark
    let mut chart3 = Chart::new(ChartType::Pie);
    chart3.set_title("Style 42 (Pie)");
    chart3.set_style(42);
    chart3
        .add_series()
        .set_values("Styles!$B$2:$B$5")
        .set_categories("Styles!$A$2:$A$5");
    ws.insert_chart(22, 0, &chart3)?;

    wb.save(&path)?;
    println!("✅ Styled charts saved to {}", path.display());
    Ok(())
}
