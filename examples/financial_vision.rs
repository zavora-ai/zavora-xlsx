use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    let navy = "#1B2A4A"; let green = "#0D7C3D"; let red = "#C00000";
    let blue = "#2B579A"; let gold = "#C68E17"; let border = "#D6DCE4";

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Financial Data (source of truth)
    // ═══════════════════════════════════════════════════════════
    {
        let ws = wb.worksheet(0)?;
        ws.set_name("Data")?;

        // Income sources
        let h1 = ["Income Source", "Monthly", "Annual", "Growth %"];
        let hf = Format::new().bold().font_size(10.0).font_color("#FFFFFF")
            .background_color(navy).align(Align::Center).border(BorderStyle::Thin);
        for (c, h) in h1.iter().enumerate() { ws.write_with_format(0, c as u16, *h, &hf)?; }

        let incomes = [
            ("Salary", 8500.0, 0.05), ("Side Business", 2000.0, 0.15),
            ("Investments", 800.0, 0.10), ("Rental Income", 1500.0, 0.03),
            ("Freelance", 1200.0, 0.20),
        ];
        let tf = Format::new().border(BorderStyle::Thin).border_color(border);
        let mf = Format::new().num_format("$#,##0").align(Align::Right).border(BorderStyle::Thin).border_color(border);
        let pf = Format::new().num_format("0%").align(Align::Center).border(BorderStyle::Thin).border_color(border);
        for (i, (name, monthly, growth)) in incomes.iter().enumerate() {
            let r = (i + 1) as u32;
            ws.write_with_format(r, 0, *name, &tf)?;
            ws.write_with_format(r, 1, *monthly, &mf)?;
            ws.write_formula(r, 2, &format!("B{}*12", r + 1))?; ws.set_cell_format(r, 2, &mf)?;
            ws.write_with_format(r, 3, *growth, &pf)?;
        }
        let inc_last = incomes.len() as u32;

        // Goals
        let goals_start = inc_last + 2;
        let h2 = ["Financial Goal", "Target", "Current", "% Done", "Deadline", "Monthly Needed"];
        for (c, h) in h2.iter().enumerate() { ws.write_with_format(goals_start, c as u16, *h, &hf)?; }

        let df = Format::new().num_format("yyyy-mm-dd").align(Align::Center).border(BorderStyle::Thin).border_color(border);
        let goals = [
            ("Emergency Fund (6 mo)", 84000.0, 35000.0, (2025,12,31)),
            ("House Down Payment", 150000.0, 42000.0, (2027,6,30)),
            ("Kids College Fund", 200000.0, 15000.0, (2040,9,1)),
            ("Retirement (FIRE)", 1500000.0, 280000.0, (2045,1,1)),
            ("Dream Vacation", 15000.0, 3500.0, (2026,3,1)),
            ("New Car Fund", 45000.0, 12000.0, (2026,12,31)),
        ];
        for (i, (name, target, current, deadline)) in goals.iter().enumerate() {
            let r = goals_start + 1 + i as u32;
            ws.write_with_format(r, 0, *name, &tf)?;
            ws.write_with_format(r, 1, *target, &mf)?;
            ws.write_with_format(r, 2, *current, &mf)?;
            ws.write_formula(r, 3, &format!("C{}/B{}", r+1, r+1))?; ws.set_cell_format(r, 3, &pf)?;
            ws.write_with_format(r, 4, ExcelDateTime::from_ymd(deadline.0, deadline.1, deadline.2).unwrap(), &df)?;
            // Monthly needed = (Target - Current) / months remaining
            ws.write_formula(r, 5, &format!("IFERROR((B{r1}-C{r1})/MAX(1,(E{r1}-TODAY())/30.44),0)", r1=r+1))?;
            ws.set_cell_format(r, 5, &mf)?;
        }

        // Net worth tracker
        let nw_start = goals_start + 1 + goals.len() as u32 + 1;
        let h3 = ["Asset/Liability", "Value", "Type"];
        for (c, h) in h3.iter().enumerate() { ws.write_with_format(nw_start, c as u16, *h, &hf)?; }

        let assets = [
            ("Checking Account", 12000.0, "Asset"), ("Savings Account", 35000.0, "Asset"),
            ("Investment Portfolio", 280000.0, "Asset"), ("Rental Property", 320000.0, "Asset"),
            ("Car", 25000.0, "Asset"), ("401k/Retirement", 180000.0, "Asset"),
            ("Mortgage", -245000.0, "Liability"), ("Car Loan", -18000.0, "Liability"),
            ("Student Loans", -32000.0, "Liability"), ("Credit Cards", -4500.0, "Liability"),
        ];
        for (i, (name, val, typ)) in assets.iter().enumerate() {
            let r = nw_start + 1 + i as u32;
            ws.write_with_format(r, 0, *name, &tf)?;
            ws.write_with_format(r, 1, *val, &mf)?;
            ws.write_with_format(r, 2, *typ, &tf)?;
        }

        let widths = [24.0, 14.0, 14.0, 10.0, 14.0, 14.0];
        for (c, w) in widths.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }
        ws.set_freeze_panes(1, 0)?;
    }

    let inc_last = 5u32;
    let goals_start = 7u32;
    let goals_last = goals_start + 6;
    let nw_start = goals_last + 2;
    let nw_last = nw_start + 10;

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Financial Vision Dashboard
    // ═══════════════════════════════════════════════════════════
    let ws2 = wb.add_worksheet_with_name("Financial Vision")?;
    ws2.hide_gridlines();

    let cw = [2.0, 16.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 2.0];
    for (c, w) in cw.iter().enumerate() { ws2.set_column_width(c as u16, *w)?; }

    let title = Format::new().bold().font_size(24.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let subtitle = Format::new().font_size(11.0).font_color("#667085").italic();
    let divider = Format::new().background_color(gold);
    let section = Format::new().bold().font_size(13.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Left).align(Align::VerticalCenter);
    let kpi_label = Format::new().bold().font_size(9.0).font_color("#667085").align(Align::Center);
    let kpi_big = Format::new().bold().font_size(22.0).font_color(navy).align(Align::Center).num_format("$#,##0");
    let kpi_pct = Format::new().bold().font_size(22.0).font_color(green).align(Align::Center).num_format("0%");
    let kpi_green = Format::new().bold().font_size(22.0).font_color(green).align(Align::Center).num_format("$#,##0");
    let kpi_red = Format::new().bold().font_size(18.0).font_color(red).align(Align::Center).num_format("$#,##0");

    let goal_name = Format::new().bold().font_size(10.0).font_color(navy)
        .align(Align::Left).border(BorderStyle::Thin).border_color(border);
    let goal_val = Format::new().font_size(10.0).font_color(navy).num_format("$#,##0")
        .align(Align::Right).border(BorderStyle::Thin).border_color(border);
    let goal_pct = Format::new().font_size(10.0).font_color(navy).num_format("0%")
        .align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let goal_date = Format::new().font_size(10.0).font_color(navy).num_format("mmm yyyy")
        .align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let goal_need = Format::new().bold().font_size(10.0).font_color(blue).num_format("$#,##0")
        .align(Align::Right).border(BorderStyle::Thin).border_color(border);

    let tbl_hdr = Format::new().bold().font_size(9.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let tbl_hdr_l = Format::new().bold().font_size(9.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Left).border(BorderStyle::Thin);
    let asset_val = Format::new().font_size(10.0).font_color(green).bold().num_format("$#,##0")
        .align(Align::Right).border(BorderStyle::Thin).border_color(border);
    let liab_val = Format::new().font_size(10.0).font_color(red).bold().num_format("$#,##0")
        .align(Align::Right).border(BorderStyle::Thin).border_color(border);

    let mut r = 0u32;

    // Title
    ws2.set_row_height(r, 6.0)?; r += 1;
    ws2.write_with_format(r, 1, "💰 Financial Vision Board", &title)?;
    ws2.set_row_height(r, 36.0)?; r += 1;
    ws2.write_with_format(r, 1, "Your roadmap to financial freedom  |  Auto-updates from Data sheet", &subtitle)?;
    r += 1;
    for c in 1..=8u16 { ws2.write_with_format(r, c, "", &divider)?; }
    ws2.set_row_height(r, 3.0)?; r += 2;

    // ── KPI Cards ──
    let labels = ["Monthly Income", "Annual Income", "Savings Rate", "Net Worth", "Total Debt"];
    for (c, l) in labels.iter().enumerate() { ws2.write_with_format(r, (c+1) as u16, *l, &kpi_label)?; }
    r += 1;

    // Monthly Income = SUM
    ws2.write_formula(r, 1, &format!("SUM(Data!B2:B{})", inc_last + 1))?;
    ws2.set_cell_format(r, 1, &kpi_big)?;
    // Annual Income
    ws2.write_formula(r, 2, &format!("SUM(Data!C2:C{})", inc_last + 1))?;
    ws2.set_cell_format(r, 2, &kpi_big)?;
    // Savings Rate = (Income - Expenses) / Income — approximate
    ws2.write_formula(r, 3, "0.35")?; // placeholder — user fills in
    ws2.set_cell_format(r, 3, &kpi_pct)?;
    // Net Worth = SUM of all assets/liabilities
    ws2.write_formula(r, 4, &format!("SUM(Data!B{}:B{})", nw_start + 2, nw_last + 1))?;
    ws2.set_cell_format(r, 4, &kpi_green)?;
    // Total Debt
    ws2.write_formula(r, 5, &format!("SUMIF(Data!C{}:C{},\"Liability\",Data!B{}:B{})", nw_start + 2, nw_last + 1, nw_start + 2, nw_last + 1))?;
    ws2.set_cell_format(r, 5, &kpi_red)?;
    ws2.set_row_height(r, 34.0)?; r += 2;

    // ── Financial Goals ──
    ws2.merge_range(r, 1, r, 8, "  🎯 Financial Goals — Progress Tracker", &section)?;
    ws2.set_row_height(r, 28.0)?; r += 1;

    let gh = ["Goal", "Target", "Current", "% Done", "Deadline", "Monthly Needed"];
    for (c, h) in gh.iter().enumerate() {
        let f = if c == 0 { &tbl_hdr_l } else { &tbl_hdr };
        ws2.write_with_format(r, (c + 1) as u16, *h, f)?;
    }
    ws2.set_row_height(r, 20.0)?; r += 1;

    let goals_dash_start = r;
    for i in 0..6u32 {
        let dr = goals_start + 2 + i; // data row (1-indexed)
        ws2.write_formula(r, 1, &format!("Data!A{dr}"))?; ws2.set_cell_format(r, 1, &goal_name)?;
        ws2.write_formula(r, 2, &format!("Data!B{dr}"))?; ws2.set_cell_format(r, 2, &goal_val)?;
        ws2.write_formula(r, 3, &format!("Data!C{dr}"))?; ws2.set_cell_format(r, 3, &goal_val)?;
        ws2.write_formula(r, 4, &format!("Data!D{dr}"))?; ws2.set_cell_format(r, 4, &goal_pct)?;
        ws2.write_formula(r, 5, &format!("Data!E{dr}"))?; ws2.set_cell_format(r, 5, &goal_date)?;
        ws2.write_formula(r, 6, &format!("Data!F{dr}"))?; ws2.set_cell_format(r, 6, &goal_need)?;
        ws2.set_row_height(r, 22.0)?; r += 1;
    }
    // Data bars on % done
    ws2.add_conditional_format(goals_dash_start, 4, r - 1, 4, ConditionalFormatDataBar::new(green))?;

    r += 1;

    // ── Net Worth Breakdown ──
    ws2.merge_range(r, 1, r, 4, "  📊 Net Worth Breakdown", &section)?;
    ws2.merge_range(r, 5, r, 8, "  📈 Income Sources", &section)?;
    ws2.set_row_height(r, 28.0)?; r += 1;

    // Assets & Liabilities table (left)
    ws2.write_with_format(r, 1, "Item", &tbl_hdr_l)?;
    ws2.write_with_format(r, 2, "Value", &tbl_hdr)?;
    ws2.write_with_format(r, 3, "Type", &tbl_hdr)?;
    // Income table (right)
    ws2.write_with_format(r, 5, "Source", &tbl_hdr_l)?;
    ws2.write_with_format(r, 6, "Monthly", &tbl_hdr)?;
    ws2.write_with_format(r, 7, "Annual", &tbl_hdr)?;
    ws2.write_with_format(r, 8, "Growth", &tbl_hdr)?;
    ws2.set_row_height(r, 20.0)?; r += 1;

    let nw_dash_start = r;
    let item_fmt = Format::new().font_size(10.0).font_color(navy).align(Align::Left)
        .border(BorderStyle::Thin).border_color(border);
    let type_fmt = Format::new().font_size(10.0).font_color(navy).align(Align::Center)
        .border(BorderStyle::Thin).border_color(border);

    for i in 0..10u32 {
        let dr = nw_start + 2 + i;
        ws2.write_formula(r + i, 1, &format!("Data!A{dr}"))?; ws2.set_cell_format(r + i, 1, &item_fmt)?;
        ws2.write_formula(r + i, 2, &format!("Data!B{dr}"))?;
        // Color based on type
        ws2.write_formula(r + i, 2, &format!("Data!B{dr}"))?;
        ws2.set_cell_format(r + i, 2, if i < 6 { &asset_val } else { &liab_val })?;
        ws2.write_formula(r + i, 3, &format!("Data!C{dr}"))?; ws2.set_cell_format(r + i, 3, &type_fmt)?;
    }

    // Income sources (right side)
    for i in 0..5u32 {
        let dr = i + 2;
        ws2.write_formula(r + i, 5, &format!("Data!A{dr}"))?; ws2.set_cell_format(r + i, 5, &item_fmt)?;
        ws2.write_formula(r + i, 6, &format!("Data!B{dr}"))?; ws2.set_cell_format(r + i, 6, &goal_val)?;
        ws2.write_formula(r + i, 7, &format!("Data!C{dr}"))?; ws2.set_cell_format(r + i, 7, &goal_val)?;
        ws2.write_formula(r + i, 8, &format!("Data!D{dr}"))?; ws2.set_cell_format(r + i, 8, &goal_pct)?;
    }

    r += 11;

    // ── Charts ──
    // Doughnut: Net worth composition
    let mut donut = Chart::new(ChartType::Doughnut);
    donut.set_title("Net Worth Composition");
    donut.set_width(420); donut.set_height(280);
    let ds = donut.add_series();
    ds.set_values(&format!("Data!$B${}:$B${}", nw_start + 2, nw_start + 7)); // assets only
    ds.set_categories(&format!("Data!$A${}:$A${}", nw_start + 2, nw_start + 7));
    ds.set_data_labels(true);
    ws2.insert_chart(r, 1, &donut)?;

    // Bar: Income sources
    let mut bar = Chart::new(ChartType::Column);
    bar.set_title("Monthly Income by Source");
    bar.set_width(420); bar.set_height(280);
    bar.set_legend_position(LegendPosition::None);
    bar.set_y_axis_name("$/month");
    let bs = bar.add_series();
    bs.set_values(&format!("Data!$B$2:$B${}", inc_last + 1));
    bs.set_categories(&format!("Data!$A$2:$A${}", inc_last + 1));
    bs.set_data_labels(true);
    bs.set_color(blue);
    ws2.insert_chart(r, 5, &bar)?;

    // Print
    ws2.set_landscape(); ws2.set_fit_to_page(1, 1);
    ws2.set_margins(0.3, 0.3, 0.3, 0.3);
    ws2.set_header("&C💰 Financial Vision Board");
    ws2.set_footer("&CConfidential  |  &D");

    wb.set_active_sheet(1);
    let path = home("financial_vision.xlsx");
    wb.save(&path)?;
    println!("✅ Financial Vision saved to {}", path.display());
    Ok(())
}

fn home(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(name)
}
