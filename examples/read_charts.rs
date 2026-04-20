//! Example: Write a workbook with various chart types, then read it back
//! and print the parsed chart properties.
//!
//! This validates Task 4 (Read Charts) end-to-end.

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::env::temp_dir().join("read_charts_example.xlsx");

    // ── Step 1: Create a workbook with charts ──
    println!("📝 Creating workbook with multiple chart types...\n");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0)?;
        ws.set_name("Chart Demo")?;

        // Write sample data
        let headers = ["Month", "Sales", "Expenses", "Profit"];
        let data: &[(&str, f64, f64, f64)] = &[
            ("Jan", 12000.0, 8000.0, 4000.0),
            ("Feb", 15000.0, 9500.0, 5500.0),
            ("Mar", 18000.0, 11000.0, 7000.0),
            ("Apr", 14000.0, 8500.0, 5500.0),
            ("May", 21000.0, 12000.0, 9000.0),
            ("Jun", 19000.0, 10500.0, 8500.0),
        ];

        let hdr_fmt = Format::new().bold();
        for (c, h) in headers.iter().enumerate() {
            ws.write_with_format(0, c as u16, *h, &hdr_fmt)?;
        }
        for (r, (month, sales, expenses, profit)) in data.iter().enumerate() {
            let row = (r + 1) as u32;
            ws.write(row, 0, *month)?;
            ws.write(row, 1, *sales)?;
            ws.write(row, 2, *expenses)?;
            ws.write(row, 3, *profit)?;
        }

        // Chart 1: Bar chart — Sales vs Expenses
        let mut bar_chart = Chart::new(ChartType::Bar);
        bar_chart.set_title("Sales vs Expenses");
        bar_chart.set_x_axis_name("Month");
        bar_chart.set_y_axis_name("Amount ($)");
        bar_chart.set_legend_position(LegendPosition::Bottom);
        bar_chart.add_series()
            .set_values("'Chart Demo'!$B$2:$B$7")
            .set_categories("'Chart Demo'!$A$2:$A$7")
            .set_name("Sales");
        bar_chart.add_series()
            .set_values("'Chart Demo'!$C$2:$C$7")
            .set_categories("'Chart Demo'!$A$2:$A$7")
            .set_name("Expenses");
        ws.insert_chart(9, 0, &bar_chart)?;

        // Chart 2: Line chart — Profit trend
        let mut line_chart = Chart::new(ChartType::Line);
        line_chart.set_title("Profit Trend");
        line_chart.set_legend_position(LegendPosition::Top);
        line_chart.add_series()
            .set_values("'Chart Demo'!$D$2:$D$7")
            .set_categories("'Chart Demo'!$A$2:$A$7")
            .set_name("Profit");
        ws.insert_chart(9, 5, &line_chart)?;

        // Chart 3: Pie chart — Sales distribution
        let mut pie_chart = Chart::new(ChartType::Pie);
        pie_chart.set_title("Sales by Month");
        pie_chart.add_series()
            .set_values("'Chart Demo'!$B$2:$B$7")
            .set_categories("'Chart Demo'!$A$2:$A$7")
            .set_name("Sales");
        ws.insert_chart(25, 0, &pie_chart)?;

        // Chart 4: Scatter chart
        let mut scatter_chart = Chart::new(ChartType::Scatter);
        scatter_chart.set_title("Sales vs Profit Correlation");
        scatter_chart.add_series()
            .set_values("'Chart Demo'!$D$2:$D$7")
            .set_categories("'Chart Demo'!$B$2:$B$7")
            .set_name("Correlation");
        ws.insert_chart(25, 5, &scatter_chart)?;

        wb.save(&path)?;
    }

    println!("💾 Saved to: {}\n", path.display());

    // ── Step 2: Read the workbook back and inspect charts ──
    println!("📖 Reading workbook back and inspecting charts...\n");
    println!("{:-<70}", "");

    let wb = Workbook::open_readonly(&path)?;
    let ws = wb.worksheet_ref(0).unwrap();
    let charts = ws.charts();

    println!("Found {} chart(s):\n", charts.len());

    for (i, chart) in charts.iter().enumerate() {
        println!("  Chart {} — {:?}", i + 1, chart.chart_type());

        if let Some(title) = chart.title() {
            println!("    Title: \"{}\"", title);
        }

        if let Some(x) = chart.x_axis_name() {
            println!("    X Axis: \"{}\"", x);
        }
        if let Some(y) = chart.y_axis_name() {
            println!("    Y Axis: \"{}\"", y);
        }

        println!("    Legend: {:?}", chart.legend_position());
        println!("    Series ({}):", chart.series().len());

        for (j, series) in chart.series().iter().enumerate() {
            println!("      [{}] name={:?}", j, series.name());
            println!("          values={}", series.values());
            if let Some(cats) = series.categories() {
                println!("          categories={}", cats);
            }
        }

        println!();
    }

    println!("{:-<70}", "");
    println!("✅ Read Charts validation complete!");

    Ok(())
}
