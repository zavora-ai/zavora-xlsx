use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    let navy = "#1B3A5C"; let blue = "#2B7BB9"; let _light_blue = "#D6EAF8";
    let green = "#27AE60"; let red = "#C0392B"; let amber = "#F39C12";
    let dark = "#2C3E50"; let border = "#BDC3C7"; let white = "#FFFFFF";
    let light = "#ECF0F1";

    // ── Project Data ──
    let projects: Vec<Proj> = vec![
        Proj { name: "ABC Integration", budget: 484.0, spent: 396.0, pct: 84, months: 16,
            people: 20, activities: 40, issues: 6, risks: 1, status: "on_track" },
        Proj { name: "Morlort Highway", budget: 382.0, spent: 347.0, pct: 91, months: 13,
            people: 15, activities: 40, issues: 1, risks: 0, status: "on_track" },
        Proj { name: "Riggs-A-lot Decommissioning", budget: 377.0, spent: 188.0, pct: 50, months: 3,
            people: 35, activities: 25, issues: 4, risks: 1, status: "at_risk" },
        Proj { name: "12th Annual Customer Convention", budget: 490.0, spent: 343.0, pct: 70, months: 11,
            people: 40, activities: 30, issues: 2, risks: 0, status: "on_track" },
        Proj { name: "Six Sigma Assessment", budget: 445.0, spent: 383.0, pct: 86, months: 4,
            people: 35, activities: 40, issues: 3, risks: 0, status: "on_track" },
        Proj { name: "Cloud Migration Phase 2", budget: 620.0, spent: 186.0, pct: 30, months: 8,
            people: 25, activities: 35, issues: 8, risks: 3, status: "critical" },
        Proj { name: "ERP Upgrade", budget: 510.0, spent: 459.0, pct: 90, months: 14,
            people: 18, activities: 28, issues: 1, risks: 0, status: "on_track" },
        Proj { name: "New Office Buildout", budget: 290.0, spent: 232.0, pct: 80, months: 6,
            people: 12, activities: 22, issues: 2, risks: 1, status: "on_track" },
    ];

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Data
    // ═══════════════════════════════════════════════════════════
    {
        let ws = wb.worksheet(0)?;
        ws.set_name("Data")?;
        let h = ["Project","Budget ($k)","Spent ($k)","% Complete","Duration (mo)","People","Activities","Issues","Risks","Status"];
        let hf = Format::new().bold().font_size(10.0).font_color(white).background_color(navy).align(Align::Center).border(BorderStyle::Thin);
        for (c, hdr) in h.iter().enumerate() { ws.write_with_format(0, c as u16, *hdr, &hf)?; }

        let tf = Format::new().align(Align::Left).border(BorderStyle::Thin).border_color(border);
        let nf = Format::new().align(Align::Center).border(BorderStyle::Thin).border_color(border);
        let mf = Format::new().num_format("$#,##0").align(Align::Center).border(BorderStyle::Thin).border_color(border);
        let pf = Format::new().num_format("0%").align(Align::Center).border(BorderStyle::Thin).border_color(border);

        for (i, p) in projects.iter().enumerate() {
            let r = (i + 1) as u32;
            ws.write_with_format(r, 0, p.name, &tf)?;
            ws.write_with_format(r, 1, p.budget, &mf)?;
            ws.write_with_format(r, 2, p.spent, &mf)?;
            ws.write_with_format(r, 3, p.pct as f64 / 100.0, &pf)?;
            ws.write_with_format(r, 4, p.months as f64, &nf)?;
            ws.write_with_format(r, 5, p.people as f64, &nf)?;
            ws.write_with_format(r, 6, p.activities as f64, &nf)?;
            ws.write_with_format(r, 7, p.issues as f64, &nf)?;
            ws.write_with_format(r, 8, p.risks as f64, &nf)?;
            ws.write_with_format(r, 9, p.status, &tf)?;
        }
        let widths = [30.0,12.0,12.0,12.0,12.0,10.0,12.0,10.0,10.0,12.0];
        for (c, w) in widths.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }
        ws.set_autofilter(0, 0, projects.len() as u32, 9);
        ws.set_freeze_panes(1, 0)?;
    }

    let n = projects.len() as u32;

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Portfolio Dashboard
    // ═══════════════════════════════════════════════════════════
    let ws2 = wb.add_worksheet_with_name("Portfolio Dashboard")?;
    ws2.hide_gridlines();

    let cw = [2.0, 4.0, 22.0, 12.0, 10.0, 10.0, 10.0, 10.0, 4.0, 14.0, 14.0, 14.0, 14.0, 14.0, 2.0];
    for (c, w) in cw.iter().enumerate() { ws2.set_column_width(c as u16, *w)?; }

    // Styles
    let title = Format::new().bold().font_size(20.0).font_color(white)
        .background_color(blue).align(Align::Left).align(Align::VerticalCenter);
    let title_bar = Format::new().background_color(blue);

    let _kpi_num = Format::new().bold().font_size(22.0).font_color(dark).align(Align::Center).align(Align::VerticalCenter);
    let _kpi_money = Format::new().bold().font_size(18.0).font_color(dark).align(Align::Center).align(Align::VerticalCenter).num_format("$#,##0k");
    let _kpi_sub = Format::new().font_size(8.0).font_color("#7F8C8D").align(Align::Center);
    let kpi_label = Format::new().bold().font_size(9.0).font_color(white).background_color(navy).align(Align::Center);
    let _kpi_box_border = Format::new().border(BorderStyle::Thin).border_color(border);

    let proj_name = Format::new().bold().font_size(11.0).font_color(dark).align(Align::Left).align(Align::VerticalCenter)
        .background_color(light).border(BorderStyle::Thin).border_color(border);
    let proj_pct_green = Format::new().bold().font_size(14.0).font_color(green).align(Align::Center).align(Align::VerticalCenter)
        .num_format("0%").background_color(light).border(BorderStyle::Thin).border_color(border);
    let proj_pct_amber = Format::new().bold().font_size(14.0).font_color(amber).align(Align::Center).align(Align::VerticalCenter)
        .num_format("0%").background_color(light).border(BorderStyle::Thin).border_color(border);
    let proj_pct_red = Format::new().bold().font_size(14.0).font_color(red).align(Align::Center).align(Align::VerticalCenter)
        .num_format("0%").background_color(light).border(BorderStyle::Thin).border_color(border);
    let proj_detail = Format::new().font_size(9.0).font_color("#7F8C8D").align(Align::Center)
        .background_color(light).border(BorderStyle::Thin).border_color(border);
    let proj_budget = Format::new().bold().font_size(10.0).font_color(dark).align(Align::Center)
        .num_format("$#,##0k").background_color(light).border(BorderStyle::Thin).border_color(border);
    let proj_months = Format::new().font_size(10.0).font_color(dark).align(Align::Center)
        .background_color(light).border(BorderStyle::Thin).border_color(border);

    let _stat_label = Format::new().font_size(8.0).font_color("#7F8C8D").align(Align::Center);
    let stat_icon_green = Format::new().font_size(10.0).font_color(green).align(Align::Center);
    let stat_icon_red = Format::new().font_size(10.0).font_color(red).align(Align::Center);
    let stat_icon_amber = Format::new().font_size(10.0).font_color(amber).align(Align::Center);

    let _section = Format::new().bold().font_size(12.0).font_color(white)
        .background_color(navy).align(Align::Left).align(Align::VerticalCenter);

    let mut r = 0u32;

    // ── Title Bar ──
    for c in 0..=14u16 { ws2.write_with_format(r, c, "", &title_bar)?; }
    ws2.write_with_format(r, 1, "Project Portfolio Dashboard", &title)?;
    ws2.set_row_height(r, 36.0)?; r += 2;

    // ── KPI Cards Row (all formulas) ──
    let dr = format!("Data!D2:D{}", n + 1); // % complete
    let br = format!("Data!B2:B{}", n + 1); // budget
    let sr = format!("Data!C2:C{}", n + 1); // spent
    let pr = format!("Data!F2:F{}", n + 1); // people
    let rr = format!("Data!I2:I{}", n + 1); // risks
    let ir = format!("Data!H2:H{}", n + 1); // issues

    // Projects count + avg %
    ws2.write_with_format(r, 1, "PROJECTS", &kpi_label)?;
    ws2.write_formula(r + 1, 1, &format!("COUNTA(Data!A2:A{})", n + 1))?;
    ws2.set_cell_format(r + 1, 1, &Format::new().bold().font_size(22.0).font_color(dark).align(Align::Center)
        .num_format("0\" Projects\""))?;
    ws2.write_formula(r + 2, 1, &format!("AVERAGE({dr})"))?;
    ws2.set_cell_format(r + 2, 1, &Format::new().font_size(10.0).font_color(green).align(Align::Center).num_format("0%\" done\""))?;

    // Budget
    ws2.write_with_format(r, 3, "BUDGETS", &kpi_label)?;
    ws2.write_formula(r + 1, 3, &format!("SUM({br})/1000"))?;
    ws2.set_cell_format(r + 1, 3, &Format::new().bold().font_size(18.0).font_color(dark).align(Align::Center).num_format("$0.0\" mn\""))?;
    ws2.write_formula(r + 2, 3, &format!("SUM({sr})/1000"))?;
    ws2.set_cell_format(r + 2, 3, &Format::new().font_size(10.0).font_color("#7F8C8D").align(Align::Center).num_format("$0.0\" mn spent\""))?;

    // Team
    ws2.write_with_format(r, 5, "TEAM", &kpi_label)?;
    ws2.write_formula(r + 1, 5, &format!("SUM({pr})"))?;
    ws2.set_cell_format(r + 1, 5, &Format::new().bold().font_size(22.0).font_color(dark).align(Align::Center).num_format("0\" members\""))?;

    // Risks
    ws2.write_with_format(r, 9, "RISKS", &kpi_label)?;
    ws2.write_formula(r + 1, 9, &format!("SUM({rr})"))?;
    ws2.set_cell_format(r + 1, 9, &Format::new().bold().font_size(22.0).font_color(red).align(Align::Center).num_format("0\" Risks\""))?;

    // Issues
    ws2.write_with_format(r, 11, "ISSUES", &kpi_label)?;
    ws2.write_formula(r + 1, 11, &format!("SUM({ir})"))?;
    ws2.set_cell_format(r + 1, 11, &Format::new().bold().font_size(22.0).font_color(amber).align(Align::Center).num_format("0\" Open\""))?;

    ws2.set_row_height(r, 18.0)?;
    ws2.set_row_height(r + 1, 32.0)?;
    ws2.set_row_height(r + 2, 18.0)?;
    r += 4;

    // ── Project Cards (left side) ──
    let cards_start = r;
    for (i, p) in projects.iter().enumerate() {
        let pct_fmt = if p.pct >= 80 { &proj_pct_green } else if p.pct >= 50 { &proj_pct_amber } else { &proj_pct_red };
        let status_icon = match p.status {
            "on_track" => ("●", &stat_icon_green),
            "at_risk" => ("▲", &stat_icon_amber),
            _ => ("✖", &stat_icon_red),
        };

        // Project name
        ws2.merge_range(r, 2, r, 5, p.name, &proj_name)?;
        ws2.set_row_height(r, 22.0)?; r += 1;

        // Status dot + % + budget + duration
        ws2.write_with_format(r, 1, status_icon.0, status_icon.1)?;
        // % from formula
        ws2.write_formula(r, 2, &format!("Data!D{}", i + 2))?;
        ws2.set_cell_format(r, 2, pct_fmt)?;
        ws2.write_formula(r, 3, &format!("Data!B{}", i + 2))?;
        ws2.set_cell_format(r, 3, &proj_budget)?;
        let dur = format!("{} months", p.months);
        ws2.write_with_format(r, 4, dur.as_str(), &proj_months)?;
        ws2.set_row_height(r, 20.0)?; r += 1;

        // People / Activities / Issues / Risks counts
        let icons = [("👥", p.people), ("📋", p.activities), ("⚠", p.issues), ("🔴", p.risks)];
        for (ci, (icon, val)) in icons.iter().enumerate() {
            let label = format!("{icon} {val}");
            ws2.write_with_format(r, (2 + ci) as u16, label.as_str(), &proj_detail)?;
        }
        ws2.set_row_height(r, 16.0)?; r += 1;

        // Spacer between cards
        ws2.set_row_height(r, 4.0)?; r += 1;
    }

    // ── Right side: Summary Charts ──
    let chart_row = cards_start;

    // Doughnut: budget allocation
    let mut donut = Chart::new(ChartType::Doughnut);
    donut.set_title("Budget Allocation");
    donut.set_width(420); donut.set_height(280);
    let ds = donut.add_series();
    ds.set_values(&format!("Data!$B$2:$B${}", n + 1));
    ds.set_categories(&format!("Data!$A$2:$A${}", n + 1));
    ds.set_data_labels(true);
    let chart_colors = [blue, "#ED7D31", amber, green, "#5B9BD5", red, "#7B1FA2", "#00B0F0"];
    for (i, c) in chart_colors.iter().enumerate() { if i < projects.len() { ds.set_point_color(i, *c); } }
    ws2.insert_chart(chart_row, 9, &donut)?;

    // Bar: completion by project
    let bar_row = chart_row + 16;
    let mut bar = Chart::new(ChartType::Bar);
    bar.set_title("Completion by Project");
    bar.set_width(420); bar.set_height(280);
    bar.set_legend_position(LegendPosition::None);
    let bs = bar.add_series();
    bs.set_values(&format!("Data!$D$2:$D${}", n + 1));
    bs.set_categories(&format!("Data!$A$2:$A${}", n + 1));
    bs.set_name("% Complete");
    bs.set_data_labels(true);
    bs.set_color(blue);
    ws2.insert_chart(bar_row, 9, &bar)?;

    // ── Legend ──
    let legend_row = r + 1;
    let legend_fmt = Format::new().font_size(8.0).font_color("#7F8C8D");
    ws2.write_with_format(legend_row, 1, "LEGEND:", &Format::new().bold().font_size(8.0).font_color(dark))?;
    ws2.write_with_format(legend_row, 2, "● On Track", &Format::new().font_size(8.0).font_color(green))?;
    ws2.write_with_format(legend_row, 3, "▲ At Risk", &Format::new().font_size(8.0).font_color(amber))?;
    ws2.write_with_format(legend_row, 4, "✖ Critical", &Format::new().font_size(8.0).font_color(red))?;
    ws2.write_with_format(legend_row + 1, 1, "👥 People  📋 Activities  ⚠ Issues  🔴 Risks", &legend_fmt)?;

    // Print
    ws2.set_landscape(); ws2.set_fit_to_page(1, 1);
    ws2.set_margins(0.3, 0.3, 0.3, 0.3);
    ws2.set_header("&CProject Portfolio Dashboard");
    ws2.set_footer("&CPrepared on &D  |  Page &P");

    wb.set_active_sheet(1);
    let path = home("portfolio_dashboard.xlsx");
    wb.save(&path)?;
    println!("✅ Portfolio Dashboard saved to {}", path.display());
    Ok(())
}

struct Proj { name: &'static str, budget: f64, spent: f64, pct: u32, months: u32,
    people: u32, activities: u32, issues: u32, risks: u32, status: &'static str }

fn home(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(name)
}
