use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    let navy = "#1B2A4A"; let green = "#0D7C3D"; let red = "#C00000";
    let amber = "#E8A317"; let blue = "#2B579A"; let light = "#F5F7FA"; let border = "#D6DCE4";
    let cat_colors: [(&str, &str); 5] = [
        ("Deployment","#4472C4"), ("Design","#ED7D31"), ("Requirements","#70AD47"),
        ("Development","#FFC000"), ("Testing","#C00000"),
    ];

    // ── Raw Data ──
    let activities: Vec<Act> = vec![
        Act{id:"Act 001",cat:"Design",      person:"Barbara G.", start:(2025,3,1), end:(2025,3,4),  pct:100,issues:0},
        Act{id:"Act 002",cat:"Deployment",   person:"Harley Q.",  start:(2025,3,1), end:(2025,3,11), pct:100,issues:0},
        Act{id:"Act 005",cat:"Design",       person:"Richard G.", start:(2025,3,5), end:(2025,3,12), pct:100,issues:0},
        Act{id:"Act 007",cat:"Development",  person:"Barbara G.", start:(2025,3,7), end:(2025,3,15), pct:100,issues:1},
        Act{id:"Act 012",cat:"Deployment",   person:"Joker",      start:(2025,3,8), end:(2025,3,14), pct:100,issues:0},
        Act{id:"Act 013",cat:"Design",       person:"Lucius F.",  start:(2025,3,9), end:(2025,3,13), pct:100,issues:0},
        Act{id:"Act 017",cat:"Development",  person:"Barbara G.", start:(2025,3,15),end:(2025,3,19), pct:100,issues:0},
        Act{id:"Act 019",cat:"Deployment",   person:"Joker",      start:(2025,3,17),end:(2025,3,22), pct:80, issues:0},
        Act{id:"Act 032",cat:"Deployment",   person:"Joker",      start:(2025,3,29),end:(2025,4,2),  pct:60, issues:1},
        Act{id:"Act 039",cat:"Design",       person:"Lucius F.",  start:(2025,4,3), end:(2025,4,9),  pct:70, issues:0},
        Act{id:"Act 041",cat:"Design",       person:"Richard G.", start:(2025,4,3), end:(2025,4,6),  pct:50, issues:1},
        Act{id:"Act 044",cat:"Deployment",   person:"Richard G.", start:(2025,4,1), end:(2025,4,15), pct:30, issues:2},
        Act{id:"Act 049",cat:"Deployment",   person:"Barbara G.", start:(2025,4,13),end:(2025,4,19), pct:20, issues:0},
        Act{id:"Act 050",cat:"Design",       person:"Joker",      start:(2025,4,14),end:(2025,4,23), pct:10, issues:0},
        Act{id:"Act 054",cat:"Development",  person:"Barbara G.", start:(2025,4,21),end:(2025,4,27), pct:0,  issues:1},
        Act{id:"Act 060",cat:"Deployment",   person:"Richard G.", start:(2025,4,26),end:(2025,5,6),  pct:0,  issues:0},
        Act{id:"Act 063",cat:"Development",  person:"Lucius F.",  start:(2025,4,30),end:(2025,5,3),  pct:0,  issues:0},
        Act{id:"Act 068",cat:"Testing",      person:"Joker",      start:(2025,5,4), end:(2025,5,13), pct:0,  issues:2},
        Act{id:"Act 069",cat:"Testing",      person:"Joker",      start:(2025,5,5), end:(2025,5,12), pct:0,  issues:1},
        Act{id:"Act 079",cat:"Requirements", person:"Barbara G.", start:(2025,5,14),end:(2025,5,19), pct:0,  issues:0},
        Act{id:"Act 083",cat:"Development",  person:"Barbara G.", start:(2025,5,17),end:(2025,5,25), pct:0,  issues:0},
        Act{id:"Act 096",cat:"Deployment",   person:"Barbara G.", start:(2025,6,1), end:(2025,6,4),  pct:0,  issues:0},
    ];

    let people = ["Barbara G.","Bruce W.","Harley Q.","James G.","Joker","Lucius F.","Richard G."];

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Data (source of truth)
    // ═══════════════════════════════════════════════════════════
    {
        let ws = wb.worksheet(0)?;
        ws.set_name("Data")?;
        let h = ["Activity","Category","Person","Start","End","% Complete","Issues"];
        let hf = Format::new().bold().font_size(10.0).font_color("#FFFFFF")
            .background_color(navy).align(Align::Center).border(BorderStyle::Thin);
        for (c, hdr) in h.iter().enumerate() { ws.write_with_format(0, c as u16, *hdr, &hf)?; }

        let df = Format::new().num_format("m/d/yyyy").align(Align::Center).border(BorderStyle::Thin).border_color(border);
        let tf = Format::new().align(Align::Left).border(BorderStyle::Thin).border_color(border);
        let nf = Format::new().align(Align::Center).border(BorderStyle::Thin).border_color(border);
        let pf = Format::new().num_format("0%").align(Align::Center).border(BorderStyle::Thin).border_color(border);

        for (i, a) in activities.iter().enumerate() {
            let r = (i + 1) as u32;
            ws.write_with_format(r, 0, a.id, &tf)?;
            ws.write_with_format(r, 1, a.cat, &tf)?;
            ws.write_with_format(r, 2, a.person, &tf)?;
            ws.write_with_format(r, 3, ExcelDateTime::from_ymd(a.start.0,a.start.1,a.start.2).unwrap(), &df)?;
            ws.write_with_format(r, 4, ExcelDateTime::from_ymd(a.end.0,a.end.1,a.end.2).unwrap(), &df)?;
            ws.write_with_format(r, 5, a.pct as f64 / 100.0, &pf)?;
            ws.write_with_format(r, 6, a.issues as f64, &nf)?;
        }
        let widths = [10.0, 14.0, 14.0, 12.0, 12.0, 12.0, 8.0];
        for (c, w) in widths.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }
        ws.set_autofilter(0, 0, activities.len() as u32, 6);
        ws.set_freeze_panes(1, 0)?;
    }

    let n = activities.len() as u32;

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Project Dashboard (all formulas)
    // ═══════════════════════════════════════════════════════════
    let ws2 = wb.add_worksheet_with_name("Project Dashboard")?;
    ws2.hide_gridlines();

    let cw = [2.0,14.0,16.0,14.0,12.0,12.0,10.0,4.0,14.0,14.0,14.0,14.0,2.0];
    for (c, w) in cw.iter().enumerate() { ws2.set_column_width(c as u16, *w)?; }

    // Styles
    let title = Format::new().bold().font_size(22.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let sub = Format::new().bold().font_size(14.0).font_color(navy).italic().align(Align::Left).align(Align::Bottom);
    let divider = Format::new().background_color(green);

    let kpi_box = |bg: &str| -> Format {
        Format::new().bold().font_size(28.0).font_color("#FFFFFF")
            .background_color(bg).align(Align::Center).align(Align::VerticalCenter)
            .border(BorderStyle::Thin).border_color(bg)
    };
    let kpi_label = Format::new().font_size(9.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Center).border(BorderStyle::Thin).border_color(navy);

    let section = Format::new().bold().font_size(13.0).font_color("#FFFFFF")
        .background_color(green).align(Align::Left).align(Align::VerticalCenter);
    let tbl_hdr = Format::new().bold().font_size(9.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let tbl_hdr_l = Format::new().bold().font_size(9.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Left).border(BorderStyle::Thin);
    let cell_l = Format::new().font_size(10.0).font_color(navy)
        .align(Align::Left).border(BorderStyle::Thin).border_color(border);
    let cell_c = Format::new().font_size(10.0).font_color(navy)
        .align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let cell_pct = Format::new().font_size(10.0).font_color(navy).num_format("0%")
        .align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let cell_red = Format::new().font_size(10.0).bold().font_color(red)
        .align(Align::Center).border(BorderStyle::Thin).border_color(border);

    let mut r = 0u32;

    // ── Title ──
    ws2.set_row_height(r, 6.0)?; r += 1;
    ws2.write_with_format(r, 1, "Project Mega Something", &title)?;
    ws2.set_row_height(r, 28.0)?; r += 1;
    ws2.write_with_format(r, 1, "Status Dashboard", &sub)?;
    ws2.set_row_height(r, 22.0)?; r += 1;

    // ── KPI Cards (formulas) ──
    let dr = format!("Data!F2:F{}", n + 1);
    let ir = format!("Data!G2:G{}", n + 1);

    // Status (overall %)
    ws2.write_formula(r, 1, &format!("AVERAGE({dr})"))?;
    ws2.set_cell_format(r, 1, &Format::new().bold().font_size(28.0).font_color("#FFFFFF")
        .background_color(green).align(Align::Center).align(Align::VerticalCenter)
        .num_format("0%").border(BorderStyle::Thin).border_color(green))?;
    ws2.write_with_format(r + 1, 1, "status", &kpi_label)?;

    // Days to go
    ws2.write_formula(r, 3, &format!("MAX(Data!E2:E{})-TODAY()", n + 1))?;
    ws2.set_cell_format(r, 3, &kpi_box(navy))?;
    ws2.write_with_format(r + 1, 3, "days to go", &kpi_label)?;

    // Activities on-going
    ws2.write_formula(r, 5, &format!("COUNTIFS({dr},\">0\",{dr},\"<1\")"))?;
    ws2.set_cell_format(r, 5, &kpi_box(blue))?;
    ws2.write_with_format(r + 1, 5, "activities on-going", &kpi_label)?;

    // Active issues
    ws2.write_formula(r, 8, &format!("SUM({ir})"))?;
    ws2.set_cell_format(r, 8, &kpi_box(amber))?;
    ws2.write_with_format(r + 1, 8, "active issues", &kpi_label)?;

    // Not started
    ws2.write_formula(r, 10, &format!("COUNTIF({dr},0)"))?;
    ws2.set_cell_format(r, 10, &kpi_box(red))?;
    ws2.write_with_format(r + 1, 10, "not started", &kpi_label)?;

    ws2.set_row_height(r, 48.0)?;
    ws2.set_row_height(r + 1, 16.0)?;
    r += 3;

    for c in 1..=10u16 { ws2.write_with_format(r, c, "", &divider)?; }
    ws2.set_row_height(r, 3.0)?; r += 2;

    // ── Left: Our People (formulas) ──
    let people_row = r;
    ws2.merge_range(r, 1, r, 5, "  Our People", &section)?;
    ws2.set_row_height(r, 26.0)?; r += 1;

    let ph = ["Person", "Issues", "Activities", "Avg %", "Status"];
    ws2.write_with_format(r, 1, ph[0], &tbl_hdr_l)?;
    for (c, h) in ph[1..].iter().enumerate() { ws2.write_with_format(r, (c+2) as u16, *h, &tbl_hdr)?; }
    ws2.set_row_height(r, 20.0)?; r += 1;

    let pr = format!("Data!C2:C{}", n + 1);
    let people_start = r;
    for person in &people {
        ws2.write_with_format(r, 1, *person, &cell_l)?;
        // Issues = SUMIF
        ws2.write_formula(r, 2, &format!("SUMIF({pr},B{},{ir})", r + 1))?;
        ws2.set_cell_format(r, 2, &cell_red)?;
        // Activities = COUNTIF
        ws2.write_formula(r, 3, &format!("COUNTIF({pr},B{})", r + 1))?;
        ws2.set_cell_format(r, 3, &cell_c)?;
        // Avg % = AVERAGEIF
        ws2.write_formula(r, 4, &format!("IFERROR(AVERAGEIF({pr},B{},{dr}),0)", r + 1))?;
        ws2.set_cell_format(r, 4, &cell_pct)?;
        // Status
        ws2.write_formula(r, 5, &format!("IF(E{r1}>=1,\"✅\",IF(E{r1}>0,\"🔄\",\"⏳\"))", r1 = r + 1))?;
        ws2.set_cell_format(r, 5, &cell_c)?;
        ws2.set_row_height(r, 22.0)?;
        r += 1;
    }

    // Data bars on issues
    ws2.add_conditional_format(people_start, 2, r - 1, 2, ConditionalFormatDataBar::new(red))?;
    // Data bars on avg %
    ws2.add_conditional_format(people_start, 4, r - 1, 4, ConditionalFormatDataBar::new(green))?;

    // ── Right: Category Breakdown (formulas) ──
    let cat_row = people_row;
    let cr = format!("Data!B2:B{}", n + 1);

    ws2.merge_range(cat_row, 8, cat_row, 10, "  By Category", &section)?;
    ws2.set_row_height(cat_row, 26.0)?;

    let ch = ["Category", "Count", "Avg %"];
    ws2.write_with_format(cat_row + 1, 8, ch[0], &tbl_hdr_l)?;
    ws2.write_with_format(cat_row + 1, 9, ch[1], &tbl_hdr)?;
    ws2.write_with_format(cat_row + 1, 10, ch[2], &tbl_hdr)?;

    let cat_start = cat_row + 2;
    for (ci, &(cat, color)) in cat_colors.iter().enumerate() {
        let cr2 = cat_start + ci as u32;
        let cf = Format::new().bold().font_size(10.0).font_color(color)
            .align(Align::Left).border(BorderStyle::Thin).border_color(border);
        ws2.write_with_format(cr2, 8, cat, &cf)?;
        ws2.write_formula(cr2, 9, &format!("COUNTIF({cr},I{})", cr2 + 1))?;
        ws2.set_cell_format(cr2, 9, &cell_c)?;
        ws2.write_formula(cr2, 10, &format!("IFERROR(AVERAGEIF({cr},I{},{dr}),0)", cr2 + 1))?;
        ws2.set_cell_format(cr2, 10, &cell_pct)?;
        ws2.set_row_height(cr2, 22.0)?;
    }
    let cat_end = cat_start + cat_colors.len() as u32 - 1;
    ws2.add_conditional_format(cat_start, 10, cat_end, 10, ConditionalFormatDataBar::new(blue))?;

    // ── Charts ──
    let chart_row = r.max(cat_end + 1) + 2;

    // Pie: category distribution
    let mut pie = Chart::new(ChartType::Pie);
    pie.set_title("Activities by Category");
    pie.set_width(400); pie.set_height(280);
    let ps = pie.add_series();
    ps.set_values(&format!("'Project Dashboard'!$J${}:$J${}", cat_start + 1, cat_end + 1));
    ps.set_categories(&format!("'Project Dashboard'!$I${}:$I${}", cat_start + 1, cat_end + 1));
    ps.set_data_labels(true);
    for (i, &(_, color)) in cat_colors.iter().enumerate() { ps.set_point_color(i, color); }
    ws2.insert_chart(chart_row, 1, &pie)?;

    // Bar: people workload
    let mut bar = Chart::new(ChartType::Bar);
    bar.set_title("Activities per Person");
    bar.set_width(400); bar.set_height(280);
    bar.set_legend_position(LegendPosition::None);
    let bs = bar.add_series();
    bs.set_values(&format!("'Project Dashboard'!$D${}:$D${}", people_start + 1, people_start + people.len() as u32));
    bs.set_categories(&format!("'Project Dashboard'!$B${}:$B${}", people_start + 1, people_start + people.len() as u32));
    bs.set_name("Activities");
    bs.set_data_labels(true);
    bs.set_color(blue);
    ws2.insert_chart(chart_row, 6, &bar)?;

    // Print
    ws2.set_landscape(); ws2.set_fit_to_page(1, 1);
    ws2.set_margins(0.3, 0.3, 0.3, 0.3);
    ws2.set_header("&CProject Dashboard");

    wb.set_active_sheet(1);
    let path = home_downloads("project_dashboard.xlsx");
    wb.save(&path)?;
    println!("✅ Project Dashboard saved to {}", path.display());
    Ok(())
}

struct Act { id: &'static str, cat: &'static str, person: &'static str, start: (i32,u32,u32), end: (i32,u32,u32), pct: u32, issues: u32 }

fn home_downloads(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(name)
}
