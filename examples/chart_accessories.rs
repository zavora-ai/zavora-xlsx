//! Example: Drop lines, high-low lines, and chart sheets (Task 38).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::path::PathBuf::from("output/chart_accessories_example.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Stock Data")?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Day", &hdr)?;
    ws.write_with_format(0, 1, "High", &hdr)?;
    ws.write_with_format(0, 2, "Low", &hdr)?;
    ws.write_with_format(0, 3, "Close", &hdr)?;

    let days = ["Mon", "Tue", "Wed", "Thu", "Fri"];
    let high = [152.0, 158.0, 155.0, 162.0, 160.0];
    let low = [145.0, 148.0, 146.0, 153.0, 155.0];
    let close = [150.0, 153.0, 148.0, 159.0, 158.0];
    for (i, (d, (h, (l, c)))) in days
        .iter()
        .zip(high.iter().zip(low.iter().zip(close.iter())))
        .enumerate()
    {
        let r = (i + 1) as u32;
        ws.write(r, 0, *d)?;
        ws.write(r, 1, *h)?;
        ws.write(r, 2, *l)?;
        ws.write(r, 3, *c)?;
    }

    // Line chart with drop lines and high-low lines
    let mut chart = Chart::new(ChartType::Line);
    chart.set_title("Stock Price — High/Low/Close");
    chart.set_drop_lines(true);
    chart.set_high_low_lines(true);
    chart
        .add_series()
        .set_values("'Stock Data'!$B$2:$B$6")
        .set_categories("'Stock Data'!$A$2:$A$6")
        .set_name("High");
    chart
        .add_series()
        .set_values("'Stock Data'!$C$2:$C$6")
        .set_categories("'Stock Data'!$A$2:$A$6")
        .set_name("Low");
    chart
        .add_series()
        .set_values("'Stock Data'!$D$2:$D$6")
        .set_categories("'Stock Data'!$A$2:$A$6")
        .set_name("Close");
    ws.insert_chart(8, 0, &chart)?;

    // Chart sheet — a dedicated sheet with just a chart
    let mut cs_chart = Chart::new(ChartType::Column);
    cs_chart.set_title("Weekly Close Prices");
    cs_chart.set_style(26);
    cs_chart
        .add_series()
        .set_values("'Stock Data'!$D$2:$D$6")
        .set_categories("'Stock Data'!$A$2:$A$6")
        .set_name("Close");
    wb.add_chart_sheet("Close Chart", cs_chart)?;

    wb.save(&path)?;
    println!("✅ Chart accessories example saved to {}", path.display());
    println!("   Sheet 1: Line chart with drop lines + high-low lines");
    println!("   Sheet 2: Dedicated chart sheet");
    Ok(())
}
