use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Sales Funnel")?;
    ws.hide_gridlines();

    // ── Funnel Data ──
    let stages: Vec<(&str, f64, f64)> = vec![
        // (stage, value_in_thousands, conversion_%)
        ("Initial Contact", 13_210.0, 1.00),
        ("Application of Initial Fit Criteria", 8_250.0, 0.62),
        ("Sales Lead", 7_400.0, 0.90),
        ("Need Identification", 6_200.0, 0.84),
        ("Qualified Prospect", 4_847.0, 0.78),
        ("Proposal", 3_215.0, 0.66),
        ("Negotiation", 2_020.0, 0.63),
        ("Closing", 963.0, 0.48),
        ("Deal Transaction", 904.0, 0.94),
    ];

    // ── Column widths ──
    ws.set_column_width(0, 34.0)?; // Funnel Stage
    ws.set_column_width(1, 18.0)?; // Value
    ws.set_column_width(2, 40.0)?; // Funnel Visualization (wide for data bars)
    ws.set_column_width(3, 10.0)?; // Conv %

    // ── Colors ──
    let header_bg = "#4E5B31"; // olive/dark green like the image
    let bar_color = "#8C7853"; // warm brown/tan like the image bars
    let alt_bg = "#F9F9F9";
    let border_clr = "#E0E0E0";

    // ── Styles ──
    let header_fmt = Format::new()
        .bold()
        .font_size(10.0)
        .font_color("#FFFFFF")
        .background_color(header_bg)
        .align(Align::Left)
        .align(Align::VerticalCenter)
        .border(BorderStyle::Thin)
        .border_color(header_bg);
    let header_center = Format::new()
        .bold()
        .font_size(10.0)
        .font_color("#FFFFFF")
        .background_color(header_bg)
        .align(Align::Center)
        .align(Align::VerticalCenter)
        .border(BorderStyle::Thin)
        .border_color(header_bg);

    let stage_fmt = |alt: bool| -> Format {
        Format::new()
            .font_size(10.0)
            .font_color("#2B579A")
            .align(Align::Left)
            .align(Align::VerticalCenter)
            .background_color(if alt { alt_bg } else { "#FFFFFF" })
            .border(BorderStyle::Thin)
            .border_color(border_clr)
    };
    let value_fmt = |alt: bool| -> Format {
        Format::new()
            .font_size(10.0)
            .font_color("#333333")
            .align(Align::Right)
            .align(Align::VerticalCenter)
            .num_format("$ #,##0")
            .background_color(if alt { alt_bg } else { "#FFFFFF" })
            .border(BorderStyle::Thin)
            .border_color(border_clr)
    };
    let bar_cell_fmt = |alt: bool| -> Format {
        Format::new()
            .font_size(11.0)
            .font_color(bar_color)
            .align(Align::Center)
            .align(Align::VerticalCenter)
            .font_name("Courier New") // monospace for even block widths
            .background_color(if alt { alt_bg } else { "#FFFFFF" })
            .border(BorderStyle::Thin)
            .border_color(border_clr)
    };
    let conv_fmt = |alt: bool, is_high: bool| -> Format {
        Format::new()
            .font_size(10.0)
            .font_color(if is_high { "#0D7C3D" } else { "#C00000" })
            .bold()
            .align(Align::Center)
            .align(Align::VerticalCenter)
            .num_format("0%")
            .background_color(if alt { alt_bg } else { "#FFFFFF" })
            .border(BorderStyle::Thin)
            .border_color(border_clr)
    };

    // ── Headers ──
    ws.write_with_format(0, 0, "Funnel Stage", &header_fmt)?;
    ws.write_with_format(0, 1, "Value (in $ 000s)", &header_center)?;
    ws.write_with_format(0, 2, "Funnel Visualization", &header_center)?;
    ws.write_with_format(0, 3, "Conv %", &header_center)?;
    ws.set_row_height(0, 22.0)?;

    // ── Data Rows ──
    let max_val = stages.iter().map(|s| s.1).fold(0.0f64, f64::max);
    let max_blocks = 36;
    for (i, (stage, value, conv)) in stages.iter().enumerate() {
        let r = (i + 1) as u32;
        let alt = i % 2 == 1;
        let is_high = *conv >= 0.70;

        ws.write_with_format(r, 0, *stage, &stage_fmt(alt))?;
        ws.write_with_format(r, 1, *value, &value_fmt(alt))?;
        // Centered block bar — creates funnel shape
        let n = ((value / max_val) * max_blocks as f64).round() as usize;
        let bar: String = "█".repeat(n);
        ws.write_with_format(r, 2, bar.as_str(), &bar_cell_fmt(alt))?;
        ws.write_with_format(r, 3, *conv, &conv_fmt(alt, is_high))?;
        ws.set_row_height(r, 24.0)?;
    }

    let last_row = stages.len() as u32;

    // ── Summary Section ──
    let sr = last_row + 2;
    let summary_label = Format::new()
        .bold()
        .font_size(10.0)
        .font_color("#333333")
        .align(Align::Right);
    let summary_value = Format::new()
        .bold()
        .font_size(12.0)
        .font_color("#0D7C3D")
        .align(Align::Left);
    let summary_pct = Format::new()
        .bold()
        .font_size(12.0)
        .font_color("#0D7C3D")
        .align(Align::Left)
        .num_format("0.0%");

    ws.write_with_format(sr, 0, "Total Pipeline:", &summary_label)?;
    ws.write_with_format(sr, 1, "$ 13,210k", &summary_value)?;
    ws.write_with_format(sr + 1, 0, "Overall Win Rate:", &summary_label)?;
    ws.write_with_format(sr + 1, 1, 904.0 / 13210.0, &summary_pct)?;
    ws.write_with_format(sr + 2, 0, "Avg Deal Size:", &summary_label)?;
    ws.write_with_format(sr + 2, 1, "$ 904k", &summary_value)?;

    // ── Horizontal Bar Chart (for copy-paste to PPT/Word) ──
    let chart_row = sr + 4;
    let mut chart = Chart::new(ChartType::Bar);
    chart.set_title("Sales Funnel — Pipeline by Stage");
    chart.set_width(640);
    chart.set_height(340);
    chart.set_legend_position(LegendPosition::Bottom);

    let s = chart.add_series();
    s.set_values(&format!("'Sales Funnel'!$B$2:$B${}", last_row + 1));
    s.set_categories(&format!("'Sales Funnel'!$A$2:$A${}", last_row + 1));
    s.set_name("Pipeline Value ($000s)");
    s.set_color(bar_color);
    s.set_data_labels(true);

    ws.insert_chart(chart_row, 0, &chart)?;

    // ── Print Setup ──
    ws.set_landscape();
    ws.set_paper_size(1);
    ws.set_margins(0.5, 0.5, 0.5, 0.5);
    ws.set_fit_to_page(1, 1);
    ws.set_header("&CSales Funnel Report");
    ws.set_footer("&CPage &P  |  &D");

    // ── Save ──
    let path = std::path::PathBuf::from("output/sales_funnel_incell.xlsx");
    wb.save(&path)?;
    println!("✅ Sales Funnel saved to {}", path.display());

    Ok(())
}
