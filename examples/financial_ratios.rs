use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A";
    let green = "#0D7C3D";
    let red = "#C00000";
    let blue = "#2B579A";
    let border = "#D6DCE4";

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Financial Data (3 years)
    // ═══════════════════════════════════════════════════════════
    {
        let ws = wb.worksheet(0)?;
        ws.set_name("Data")?;
        let hdr = Format::new()
            .bold()
            .font_size(10.0)
            .font_color("#FFFFFF")
            .background_color(navy)
            .align(Align::Center)
            .border(BorderStyle::Thin);
        let hdr_l = Format::new()
            .bold()
            .font_size(10.0)
            .font_color("#FFFFFF")
            .background_color(navy)
            .align(Align::Left)
            .border(BorderStyle::Thin);
        let lbl = Format::new()
            .font_size(10.0)
            .font_color(navy)
            .align(Align::Left)
            .border(BorderStyle::Thin)
            .border_color(border);
        let val = Format::new()
            .font_size(10.0)
            .font_color(navy)
            .align(Align::Right)
            .num_format("#,##0")
            .border(BorderStyle::Thin)
            .border_color(border);
        let sec = Format::new()
            .bold()
            .font_size(10.0)
            .font_color(blue)
            .italic()
            .align(Align::Left)
            .border(BorderStyle::Thin)
            .border_color(border);

        ws.write_with_format(0, 0, "Item", &hdr_l)?;
        ws.write_with_format(0, 1, "FY 2023", &hdr)?;
        ws.write_with_format(0, 2, "FY 2024", &hdr)?;
        ws.write_with_format(0, 3, "FY 2025", &hdr)?;

        let data: Vec<(&str, [f64; 3], bool)> = vec![
            ("Income Statement", [0.0, 0.0, 0.0], true),
            ("Revenue", [1450000.0, 1680000.0, 1920000.0], false),
            ("COGS", [580000.0, 655000.0, 730000.0], false),
            ("Gross Profit", [870000.0, 1025000.0, 1190000.0], false),
            ("Operating Expenses", [520000.0, 580000.0, 640000.0], false),
            ("Net Income", [350000.0, 445000.0, 550000.0], false),
            ("", [0.0, 0.0, 0.0], true),
            ("Balance Sheet", [0.0, 0.0, 0.0], true),
            ("Current Assets", [620000.0, 755000.0, 1020000.0], false),
            ("Total Assets", [1580000.0, 1800000.0, 2190000.0], false),
            ("Current Liabilities", [180000.0, 200000.0, 220000.0], false),
            ("Total Liabilities", [480000.0, 520000.0, 535000.0], false),
            ("Total Equity", [1100000.0, 1280000.0, 1655000.0], false),
        ];

        for (i, (name, vals, is_sec)) in data.iter().enumerate() {
            let r = (i + 1) as u32;
            if *is_sec {
                ws.write_with_format(r, 0, *name, &sec)?;
                for c in 1..=3u16 {
                    ws.write_with_format(r, c, "", &lbl)?;
                }
            } else {
                ws.write_with_format(r, 0, *name, &lbl)?;
                for (c, v) in vals.iter().enumerate() {
                    ws.write_with_format(r, (c + 1) as u16, *v, &val)?;
                }
            }
        }
        ws.set_column_width(0, 22.0)?;
        for c in 1..=3u16 {
            ws.set_column_width(c, 14.0)?;
        }
        ws.set_freeze_panes(1, 0)?;
    }

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Financial Ratios Dashboard
    // ═══════════════════════════════════════════════════════════
    let ws2 = wb.add_worksheet_with_name("Ratios Dashboard")?;
    ws2.hide_gridlines();

    let cw = [
        2.0, 24.0, 14.0, 14.0, 14.0, 4.0, 24.0, 14.0, 14.0, 14.0, 2.0,
    ];
    for (c, w) in cw.iter().enumerate() {
        ws2.set_column_width(c as u16, *w)?;
    }

    let title = Format::new()
        .bold()
        .font_size(20.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::Bottom);
    let divider = Format::new().background_color(blue);
    let section = |c: &str| {
        Format::new()
            .bold()
            .font_size(12.0)
            .font_color("#FFFFFF")
            .background_color(c)
            .align(Align::Left)
            .align(Align::VerticalCenter)
    };
    let hdr = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin);
    let hdr_l = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Thin);
    let ratio_l = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .bold()
        .align(Align::Left)
        .border(BorderStyle::Thin)
        .border_color(border);
    let ratio_pct = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("0.0%")
        .border(BorderStyle::Thin)
        .border_color(border);
    let ratio_x = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("0.00x")
        .border(BorderStyle::Thin)
        .border_color(border);
    let ratio_num = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("0.00")
        .border(BorderStyle::Thin)
        .border_color(border);

    let mut r = 0u32;
    ws2.set_row_height(r, 6.0)?;
    r += 1;
    ws2.write_with_format(r, 1, "📊 Financial Ratios Dashboard", &title)?;
    ws2.set_row_height(r, 32.0)?;
    r += 1;
    ws2.write_with_format(
        r,
        1,
        "3-Year Trend Analysis  |  All ratios calculated from Data sheet",
        &Format::new().font_size(10.0).font_color("#667085").italic(),
    )?;
    r += 1;
    for c in 1..=9u16 {
        ws2.write_with_format(r, c, "", &divider)?;
    }
    ws2.set_row_height(r, 3.0)?;
    r += 2;

    // Data references (1-indexed rows in Data sheet)
    // Rev=2, COGS=3, GP=4, OpEx=5, NI=6, CA=9, TA=10, CL=11, TL=12, Eq=13
    let years = ["FY 2023", "FY 2024", "FY 2025"];
    let yr_cols = ["B", "C", "D"]; // columns in Data sheet

    // ── Left: Profitability Ratios ──
    ws2.merge_range(r, 1, r, 4, "  Profitability Ratios", &section(green))?;
    ws2.set_row_height(r, 26.0)?;
    r += 1;
    ws2.write_with_format(r, 1, "Ratio", &hdr_l)?;
    for (c, y) in years.iter().enumerate() {
        ws2.write_with_format(r, (c + 2) as u16, *y, &hdr)?;
    }
    ws2.set_row_height(r, 20.0)?;
    r += 1;

    let profit_start = r;
    let profit_ratios: Vec<(&str, &str, &str)> = vec![
        ("Gross Margin", "4", "2"),            // GP / Revenue
        ("Net Margin", "6", "2"),              // NI / Revenue
        ("Return on Equity (ROE)", "6", "13"), // NI / Equity
        ("Return on Assets (ROA)", "6", "10"), // NI / Assets
    ];
    for (name, num, den) in &profit_ratios {
        ws2.write_with_format(r, 1, *name, &ratio_l)?;
        for (c, yc) in yr_cols.iter().enumerate() {
            ws2.write_formula(
                r,
                (c + 2) as u16,
                &format!("IFERROR(Data!{yc}{num}/Data!{yc}{den},0)"),
            )?;
            ws2.set_cell_format(r, (c + 2) as u16, &ratio_pct)?;
        }
        ws2.set_row_height(r, 22.0)?;
        r += 1;
    }
    ws2.add_conditional_format(
        profit_start,
        2,
        r - 1,
        4,
        ConditionalFormatDataBar::new(green),
    )?;

    // ── Right: Liquidity Ratios ──
    let liq_row = profit_start - 2;
    ws2.merge_range(liq_row, 6, liq_row, 9, "  Liquidity Ratios", &section(blue))?;
    ws2.write_with_format(liq_row + 1, 6, "Ratio", &hdr_l)?;
    for (c, y) in years.iter().enumerate() {
        ws2.write_with_format(liq_row + 1, (c + 7) as u16, *y, &hdr)?;
    }

    let liq_start = liq_row + 2;
    let _liq_ratios: Vec<(&str, &str, &str)> = vec![
        ("Current Ratio", "9", "11"),        // CA / CL
        ("Working Capital ($k)", "9", "11"), // CA - CL (special)
    ];
    ws2.write_with_format(liq_start, 6, "Current Ratio", &ratio_l)?;
    for (c, yc) in yr_cols.iter().enumerate() {
        ws2.write_formula(
            liq_start,
            (c + 7) as u16,
            &format!("IFERROR(Data!{yc}9/Data!{yc}11,0)"),
        )?;
        ws2.set_cell_format(liq_start, (c + 7) as u16, &ratio_x)?;
    }
    ws2.write_with_format(liq_start + 1, 6, "Working Capital", &ratio_l)?;
    for (c, yc) in yr_cols.iter().enumerate() {
        ws2.write_formula(
            liq_start + 1,
            (c + 7) as u16,
            &format!("(Data!{yc}9-Data!{yc}11)/1000"),
        )?;
        ws2.set_cell_format(
            liq_start + 1,
            (c + 7) as u16,
            &Format::new()
                .font_size(10.0)
                .font_color(navy)
                .align(Align::Center)
                .num_format("$#,##0\"k\"")
                .border(BorderStyle::Thin)
                .border_color(border),
        )?;
    }

    r += 1;

    // ── Leverage Ratios ──
    ws2.merge_range(r, 1, r, 4, "  Leverage Ratios", &section(red))?;
    ws2.set_row_height(r, 26.0)?;
    r += 1;
    ws2.write_with_format(r, 1, "Ratio", &hdr_l)?;
    for (c, y) in years.iter().enumerate() {
        ws2.write_with_format(r, (c + 2) as u16, *y, &hdr)?;
    }
    r += 1;

    let lev_start = r;
    ws2.write_with_format(r, 1, "Debt-to-Equity", &ratio_l)?;
    for (c, yc) in yr_cols.iter().enumerate() {
        ws2.write_formula(
            r,
            (c + 2) as u16,
            &format!("IFERROR(Data!{yc}12/Data!{yc}13,0)"),
        )?;
        ws2.set_cell_format(r, (c + 2) as u16, &ratio_num)?;
    }
    r += 1;
    ws2.write_with_format(r, 1, "Debt Ratio", &ratio_l)?;
    for (c, yc) in yr_cols.iter().enumerate() {
        ws2.write_formula(
            r,
            (c + 2) as u16,
            &format!("IFERROR(Data!{yc}12/Data!{yc}10,0)"),
        )?;
        ws2.set_cell_format(r, (c + 2) as u16, &ratio_pct)?;
    }
    r += 1;
    ws2.write_with_format(r, 1, "Equity Multiplier", &ratio_l)?;
    for (c, yc) in yr_cols.iter().enumerate() {
        ws2.write_formula(
            r,
            (c + 2) as u16,
            &format!("IFERROR(Data!{yc}10/Data!{yc}13,0)"),
        )?;
        ws2.set_cell_format(r, (c + 2) as u16, &ratio_x)?;
    }
    ws2.add_conditional_format(lev_start, 2, r, 4, ConditionalFormatDataBar::new(red))?;
    r += 1;

    // ── Efficiency ──
    ws2.merge_range(
        r + 1,
        6,
        r + 1,
        9,
        "  Efficiency Ratios",
        &section("#8E44AD"),
    )?;
    ws2.write_with_format(r + 2, 6, "Ratio", &hdr_l)?;
    for (c, y) in years.iter().enumerate() {
        ws2.write_with_format(r + 2, (c + 7) as u16, *y, &hdr)?;
    }

    ws2.write_with_format(r + 3, 6, "Asset Turnover", &ratio_l)?;
    for (c, yc) in yr_cols.iter().enumerate() {
        ws2.write_formula(
            r + 3,
            (c + 7) as u16,
            &format!("IFERROR(Data!{yc}2/Data!{yc}10,0)"),
        )?;
        ws2.set_cell_format(r + 3, (c + 7) as u16, &ratio_x)?;
    }
    ws2.write_with_format(r + 4, 6, "Revenue per Employee*", &ratio_l)?;
    for (c, yc) in yr_cols.iter().enumerate() {
        let emps = [45.0, 52.0, 60.0];
        ws2.write_formula(
            r + 4,
            (c + 7) as u16,
            &format!("IFERROR(Data!{yc}2/{},0)", emps[c]),
        )?;
        ws2.set_cell_format(
            r + 4,
            (c + 7) as u16,
            &Format::new()
                .font_size(10.0)
                .font_color(navy)
                .align(Align::Center)
                .num_format("$#,##0")
                .border(BorderStyle::Thin)
                .border_color(border),
        )?;
    }

    r += 2;

    // ── Charts ──
    let chart_row = r + 4;
    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Profitability Trend");
    chart.set_width(480);
    chart.set_height(280);
    chart.set_y_axis_name("%");
    chart.set_legend_position(LegendPosition::Bottom);

    // Gross Margin series
    let gm = chart.add_series();
    gm.set_values(&format!(
        "'Ratios Dashboard'!$C${}:$E${}",
        profit_start + 1,
        profit_start + 1
    ));
    gm.set_categories("'Ratios Dashboard'!$C$6:$E$6");
    gm.set_name("Gross Margin");
    gm.set_color(green);

    let nm = chart.add_series();
    nm.set_values(&format!(
        "'Ratios Dashboard'!$C${}:$E${}",
        profit_start + 2,
        profit_start + 2
    ));
    nm.set_categories("'Ratios Dashboard'!$C$6:$E$6");
    nm.set_name("Net Margin");
    nm.set_color(blue);
    nm.set_data_labels(true);

    let roe = chart.add_series();
    roe.set_values(&format!(
        "'Ratios Dashboard'!$C${}:$E${}",
        profit_start + 3,
        profit_start + 3
    ));
    roe.set_categories("'Ratios Dashboard'!$C$6:$E$6");
    roe.set_name("ROE");
    roe.set_color("#E67E22");

    ws2.insert_chart(chart_row, 1, &chart)?;

    // Revenue growth bar
    let mut bar = Chart::new(ChartType::Column);
    bar.set_title("Revenue & Net Income ($k)");
    bar.set_width(420);
    bar.set_height(280);
    bar.set_legend_position(LegendPosition::Bottom);
    let rs = bar.add_series();
    rs.set_values("Data!$B$2:$D$2");
    rs.set_categories("Data!$B$1:$D$1");
    rs.set_name("Revenue");
    rs.set_color(green);
    rs.set_data_labels(true);
    let ns = bar.add_series();
    ns.set_values("Data!$B$6:$D$6");
    ns.set_categories("Data!$B$1:$D$1");
    ns.set_name("Net Income");
    ns.set_color(blue);
    ns.set_data_labels(true);
    ws2.insert_chart(chart_row, 6, &bar)?;

    ws2.set_landscape();
    ws2.set_fit_to_page(1, 1);
    ws2.set_header("&CFinancial Ratios Dashboard");
    ws2.set_footer("&CConfidential  |  &D");

    wb.set_active_sheet(1);
    let path = home("financial_ratios.xlsx");
    wb.save(&path)?;
    println!("✅ Financial Ratios saved to {}", path.display());
    Ok(())
}

fn home(n: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("output").join(n)
}
