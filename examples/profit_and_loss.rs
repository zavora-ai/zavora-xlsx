use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    let navy = "#1B2A4A"; let green = "#0D7C3D"; let red = "#C00000";
    let blue = "#2B579A"; let border = "#D6DCE4"; let light = "#F5F7FA";

    let ws = wb.worksheet(0)?;
    ws.set_name("P&L Statement")?;
    ws.hide_gridlines();

    // Columns: A(margin) B(labels) C-N(Jan-Dec) O(Total)
    ws.set_column_width(0, 2.0)?;
    ws.set_column_width(1, 30.0)?;
    for c in 2..=13u16 { ws.set_column_width(c, 11.0)?; }
    ws.set_column_width(14, 13.0)?;

    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

    // Styles
    let title = Format::new().bold().font_size(20.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let sub = Format::new().font_size(10.0).font_color("#667085").italic();
    let divider = Format::new().background_color(blue);

    let hdr = Format::new().bold().font_size(10.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let hdr_l = Format::new().bold().font_size(10.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Left).border(BorderStyle::Thin);
    let hdr_total = Format::new().bold().font_size(10.0).font_color("#FFFFFF")
        .background_color(blue).align(Align::Center).border(BorderStyle::Thin);

    let section_fmt = |color: &str| -> Format {
        Format::new().bold().font_size(11.0).font_color("#FFFFFF")
            .background_color(color).align(Align::Left).border(BorderStyle::Thin)
    };
    let section_val = |color: &str| -> Format {
        Format::new().bold().font_size(10.0).font_color("#FFFFFF")
            .background_color(color).align(Align::Right).num_format("#,##0")
            .border(BorderStyle::Thin)
    };

    let item_fmt = Format::new().font_size(10.0).font_color(navy).align(Align::Left)
        .border(BorderStyle::Thin).border_color(border);
    let val_fmt = Format::new().font_size(10.0).font_color(navy).align(Align::Right)
        .num_format("#,##0").border(BorderStyle::Thin).border_color(border);
    let val_alt = Format::new().font_size(10.0).font_color(navy).align(Align::Right)
        .num_format("#,##0").background_color(light).border(BorderStyle::Thin).border_color(border);
    let total_label = Format::new().bold().font_size(10.0).font_color(navy)
        .align(Align::Left).background_color("#E8E8E8").border(BorderStyle::Thin).border_color(border);
    let total_val = Format::new().bold().font_size(10.0).font_color(navy)
        .num_format("#,##0").align(Align::Right).background_color("#E8E8E8")
        .border(BorderStyle::Thin).border_color(border);
    let profit_label = Format::new().bold().font_size(11.0).font_color("#FFFFFF")
        .background_color(green).align(Align::Left).border(BorderStyle::Medium);
    let profit_val = Format::new().bold().font_size(11.0).font_color("#FFFFFF")
        .num_format("#,##0").background_color(green).align(Align::Right).border(BorderStyle::Medium);
    let margin_fmt = Format::new().bold().font_size(10.0).font_color(navy)
        .num_format("0.0%").align(Align::Right).background_color("#E8F5E9")
        .border(BorderStyle::Thin).border_color(border);

    let mut r = 0u32;

    // Title
    ws.set_row_height(r, 6.0)?; r += 1;
    ws.write_with_format(r, 1, "Profit & Loss Statement", &title)?;
    ws.set_row_height(r, 32.0)?; r += 1;
    ws.write_with_format(r, 1, "Fiscal Year 2025  |  All amounts in USD  |  Formulas auto-calculate", &sub)?;
    r += 1;
    for c in 1..=14u16 { ws.write_with_format(r, c, "", &divider)?; }
    ws.set_row_height(r, 3.0)?; r += 2;

    // Headers
    ws.write_with_format(r, 1, "Account", &hdr_l)?;
    for (c, m) in months.iter().enumerate() { ws.write_with_format(r, (c + 2) as u16, *m, &hdr)?; }
    ws.write_with_format(r, 14, "TOTAL", &hdr_total)?;
    ws.set_row_height(r, 22.0)?; r += 1;

    // ── REVENUE ──
    let rev_section = r;
    ws.write_with_format(r, 1, "REVENUE", &section_fmt(green))?;
    for c in 2..=14u16 { ws.write_with_format(r, c, "", &section_val(green))?; }
    ws.set_row_height(r, 24.0)?; r += 1;

    let revenue_items: Vec<(&str, [f64; 12])> = vec![
        ("Product Sales",     [85000.0,92000.0,88000.0,95000.0,102000.0,98000.0,105000.0,110000.0,108000.0,115000.0,120000.0,135000.0]),
        ("Service Revenue",   [42000.0,45000.0,43000.0,48000.0,50000.0,52000.0,55000.0,53000.0,56000.0,58000.0,60000.0,65000.0]),
        ("Subscription Fees", [18000.0,18500.0,19000.0,19500.0,20000.0,20500.0,21000.0,21500.0,22000.0,22500.0,23000.0,24000.0]),
        ("Consulting",        [12000.0,8000.0,15000.0,10000.0,13000.0,11000.0,14000.0,9000.0,16000.0,12000.0,10000.0,18000.0]),
        ("Other Income",      [3000.0,2500.0,4000.0,3500.0,2000.0,5000.0,3000.0,4500.0,2500.0,3000.0,6000.0,4000.0]),
    ];

    let rev_start = r;
    for (i, (name, vals)) in revenue_items.iter().enumerate() {
        let alt = i % 2 == 1;
        let vf = if alt { &val_alt } else { &val_fmt };
        ws.write_with_format(r, 1, *name, &item_fmt)?;
        for (c, v) in vals.iter().enumerate() { ws.write_with_format(r, (c + 2) as u16, *v, vf)?; }
        // Total = SUM(C:N)
        ws.write_formula(r, 14, &format!("SUM(C{}:N{})", r+1, r+1))?;
        ws.set_cell_format(r, 14, &total_val)?;
        r += 1;
    }
    let rev_end = r - 1;

    // Total Revenue row
    let total_rev_row = r;
    ws.write_with_format(r, 1, "Total Revenue", &total_label)?;
    for c in 2..=14u16 {
        ws.write_formula(r, c, &format!("SUM({}{}:{}{})", col(c), rev_start+1, col(c), rev_end+1))?;
        ws.set_cell_format(r, c, &total_val)?;
    }
    ws.set_row_height(r, 24.0)?; r += 2;

    // ── COST OF GOODS SOLD ──
    ws.write_with_format(r, 1, "COST OF GOODS SOLD", &section_fmt(red))?;
    for c in 2..=14u16 { ws.write_with_format(r, c, "", &section_val(red))?; }
    ws.set_row_height(r, 24.0)?; r += 1;

    let cogs_items: Vec<(&str, [f64; 12])> = vec![
        ("Materials & Supplies", [28000.0,30000.0,29000.0,31000.0,33000.0,32000.0,34000.0,36000.0,35000.0,37000.0,39000.0,42000.0]),
        ("Direct Labor",         [22000.0,22000.0,23000.0,23000.0,24000.0,24000.0,25000.0,25000.0,26000.0,26000.0,27000.0,28000.0]),
        ("Manufacturing OH",     [8000.0,8000.0,8500.0,8500.0,9000.0,9000.0,9500.0,9500.0,10000.0,10000.0,10500.0,11000.0]),
        ("Shipping & Freight",   [4500.0,5000.0,4800.0,5200.0,5500.0,5300.0,5800.0,6000.0,5700.0,6200.0,6500.0,7000.0]),
    ];

    let cogs_start = r;
    for (i, (name, vals)) in cogs_items.iter().enumerate() {
        let alt = i % 2 == 1;
        let vf = if alt { &val_alt } else { &val_fmt };
        ws.write_with_format(r, 1, *name, &item_fmt)?;
        for (c, v) in vals.iter().enumerate() { ws.write_with_format(r, (c + 2) as u16, *v, vf)?; }
        ws.write_formula(r, 14, &format!("SUM(C{}:N{})", r+1, r+1))?;
        ws.set_cell_format(r, 14, &total_val)?;
        r += 1;
    }
    let cogs_end = r - 1;

    let total_cogs_row = r;
    ws.write_with_format(r, 1, "Total COGS", &total_label)?;
    for c in 2..=14u16 {
        ws.write_formula(r, c, &format!("SUM({}{}:{}{})", col(c), cogs_start+1, col(c), cogs_end+1))?;
        ws.set_cell_format(r, c, &total_val)?;
    }
    ws.set_row_height(r, 24.0)?; r += 1;

    // GROSS PROFIT
    let gp_row = r;
    ws.write_with_format(r, 1, "GROSS PROFIT", &profit_label)?;
    for c in 2..=14u16 {
        ws.write_formula(r, c, &format!("{}{}-{}{}", col(c), total_rev_row+1, col(c), total_cogs_row+1))?;
        ws.set_cell_format(r, c, &profit_val)?;
    }
    ws.set_row_height(r, 26.0)?; r += 1;

    // Gross Margin %
    ws.write_with_format(r, 1, "Gross Margin %", &Format::new().italic().font_size(10.0).font_color("#667085")
        .align(Align::Left).border(BorderStyle::Thin).border_color(border))?;
    for c in 2..=14u16 {
        ws.write_formula(r, c, &format!("IFERROR({}{}/{}{},0)", col(c), gp_row+1, col(c), total_rev_row+1))?;
        ws.set_cell_format(r, c, &margin_fmt)?;
    }
    r += 2;

    // ── OPERATING EXPENSES ──
    ws.write_with_format(r, 1, "OPERATING EXPENSES", &section_fmt("#E67E22"))?;
    for c in 2..=14u16 { ws.write_with_format(r, c, "", &section_val("#E67E22"))?; }
    ws.set_row_height(r, 24.0)?; r += 1;

    let opex_items: Vec<(&str, [f64; 12])> = vec![
        ("Salaries & Wages",   [35000.0,35000.0,35000.0,36000.0,36000.0,36000.0,37000.0,37000.0,37000.0,38000.0,38000.0,40000.0]),
        ("Rent & Utilities",   [8000.0,8000.0,8000.0,8000.0,8500.0,8500.0,8500.0,8500.0,9000.0,9000.0,9000.0,9000.0]),
        ("Marketing & Ads",    [12000.0,15000.0,10000.0,18000.0,14000.0,16000.0,20000.0,13000.0,17000.0,15000.0,22000.0,25000.0]),
        ("Software & Tech",    [5000.0,5000.0,5500.0,5500.0,6000.0,6000.0,6500.0,6500.0,7000.0,7000.0,7500.0,8000.0]),
        ("Insurance",          [3000.0,3000.0,3000.0,3000.0,3000.0,3000.0,3200.0,3200.0,3200.0,3200.0,3200.0,3200.0]),
        ("Professional Fees",  [4000.0,2000.0,5000.0,3000.0,4500.0,2500.0,6000.0,3500.0,4000.0,2000.0,5500.0,8000.0]),
        ("Travel & Meals",     [3000.0,2500.0,4000.0,3500.0,5000.0,4000.0,3000.0,6000.0,4500.0,3000.0,5000.0,7000.0]),
        ("Depreciation",       [2500.0,2500.0,2500.0,2500.0,2500.0,2500.0,2500.0,2500.0,2500.0,2500.0,2500.0,2500.0]),
        ("Other Expenses",     [1500.0,2000.0,1000.0,2500.0,1800.0,1200.0,3000.0,1500.0,2000.0,1000.0,2500.0,3500.0]),
    ];

    let opex_start = r;
    for (i, (name, vals)) in opex_items.iter().enumerate() {
        let alt = i % 2 == 1;
        let vf = if alt { &val_alt } else { &val_fmt };
        ws.write_with_format(r, 1, *name, &item_fmt)?;
        for (c, v) in vals.iter().enumerate() { ws.write_with_format(r, (c + 2) as u16, *v, vf)?; }
        ws.write_formula(r, 14, &format!("SUM(C{}:N{})", r+1, r+1))?;
        ws.set_cell_format(r, 14, &total_val)?;
        r += 1;
    }
    let opex_end = r - 1;

    let total_opex_row = r;
    ws.write_with_format(r, 1, "Total Operating Expenses", &total_label)?;
    for c in 2..=14u16 {
        ws.write_formula(r, c, &format!("SUM({}{}:{}{})", col(c), opex_start+1, col(c), opex_end+1))?;
        ws.set_cell_format(r, c, &total_val)?;
    }
    ws.set_row_height(r, 24.0)?; r += 2;

    // ── NET PROFIT ──
    let np_row = r;
    let np_label = Format::new().bold().font_size(12.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Left).border(BorderStyle::Medium).border_color(navy);
    let np_val = Format::new().bold().font_size(12.0).font_color("#FFFFFF")
        .num_format("#,##0").background_color(navy).align(Align::Right).border(BorderStyle::Medium).border_color(navy);
    ws.write_with_format(r, 1, "NET PROFIT / (LOSS)", &np_label)?;
    for c in 2..=14u16 {
        ws.write_formula(r, c, &format!("{}{}-{}{}", col(c), gp_row+1, col(c), total_opex_row+1))?;
        ws.set_cell_format(r, c, &np_val)?;
    }
    ws.set_row_height(r, 28.0)?; r += 1;

    // Net Margin %
    ws.write_with_format(r, 1, "Net Margin %", &Format::new().italic().font_size(10.0).font_color("#667085")
        .align(Align::Left).border(BorderStyle::Thin).border_color(border))?;
    for c in 2..=14u16 {
        ws.write_formula(r, c, &format!("IFERROR({}{}/{}{},0)", col(c), np_row+1, col(c), total_rev_row+1))?;
        ws.set_cell_format(r, c, &margin_fmt)?;
    }
    r += 2;

    // ── Chart ──
    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Monthly P&L Summary");
    chart.set_width(720); chart.set_height(320);
    chart.set_y_axis_name("USD");
    chart.set_legend_position(LegendPosition::Bottom);

    let cat = format!("'P&L Statement'!$C$6:$N$6"); // month headers

    let rs = chart.add_series();
    rs.set_values(&format!("'P&L Statement'!$C${}:$N${}", total_rev_row+1, total_rev_row+1));
    rs.set_categories(&cat);
    rs.set_name("Revenue");
    rs.set_color(green);

    let es = chart.add_series();
    es.set_values(&format!("'P&L Statement'!$C${}:$N${}", total_opex_row+1, total_opex_row+1));
    es.set_categories(&cat);
    es.set_name("Expenses");
    es.set_color(red);

    let ns = chart.add_series();
    ns.set_values(&format!("'P&L Statement'!$C${}:$N${}", np_row+1, np_row+1));
    ns.set_categories(&cat);
    ns.set_name("Net Profit");
    ns.set_color(blue);
    ns.set_chart_type(ChartType::Line);
    ns.set_marker(MarkerType::Circle);
    ns.set_data_labels(true);

    ws.insert_chart(r, 1, &chart)?;

    // Print & freeze
    ws.set_freeze_panes(6, 2)?;
    ws.set_landscape(); ws.set_fit_to_page(1, 1);
    ws.set_header("&CProfit & Loss Statement — FY2025");
    ws.set_footer("&CConfidential  |  Page &P  |  &D");

    let path = home("profit_and_loss.xlsx");
    wb.save(&path)?;
    println!("✅ P&L Statement saved to {}", path.display());
    Ok(())
}

fn col(c: u16) -> String { zavora_xlsx::utility::col_to_letter(c) }

fn home(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(name)
}
