use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Budget Data (source of truth)
    // ═══════════════════════════════════════════════════════════
    let ws = wb.worksheet(0)?;
    ws.set_name("Budget Data")?;

    let hdr = Format::new().bold().font_size(10.0).font_color("#FFFFFF")
        .background_color("#4472C4").align(Align::Center).border(BorderStyle::Thin);
    let hdr_l = Format::new().bold().font_size(10.0).font_color("#FFFFFF")
        .background_color("#4472C4").align(Align::Left).border(BorderStyle::Thin);

    // ── Cost items ──
    ws.write_with_format(0, 0, "Cost Item", &hdr_l)?;
    ws.write_with_format(0, 1, "Budget", &hdr)?;
    ws.write_with_format(0, 2, "Actual", &hdr)?;

    let costs: Vec<(&str, f64, f64)> = vec![
        ("Wages",            9250.0,  8500.0),
        ("Payroll Taxes",    2100.0,  2050.0),
        ("Bank & Card Fees",  612.0,   580.0),
        ("Mortgage or Rent",  550.0,   550.0),
        ("Equipment Lease",   450.0,   434.0),
        ("Utilities",         400.0,   420.0),
        ("Insurance",         380.0,   380.0),
        ("Office Supplies",   350.0,   310.0),
        ("Marketing",         633.0,   590.0),
        ("Miscellaneous",    1000.0,   800.0),
    ];

    let txt = Format::new().font_size(10.0).border(BorderStyle::Thin).border_color("#D6DCE4");
    let money = Format::new().font_size(10.0).align(Align::Right).num_format("$#,##0")
        .border(BorderStyle::Thin).border_color("#D6DCE4");

    for (i, (item, budget, actual)) in costs.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write_with_format(r, 0, *item, &txt)?;
        ws.write_with_format(r, 1, *budget, &money)?;
        ws.write_with_format(r, 2, *actual, &money)?;
    }
    let cost_last = costs.len() as u32;

    // ── Revenue items ──
    let rev_start = cost_last + 2;
    ws.write_with_format(rev_start, 0, "Revenue Item", &hdr_l)?;
    ws.write_with_format(rev_start, 1, "Budget", &hdr)?;
    ws.write_with_format(rev_start, 2, "Actual", &hdr)?;

    let revenues: Vec<(&str, f64, f64)> = vec![
        ("Regular Appointments", 10800.0, 12000.0),
        ("Special Appointments",  3700.0,  7200.0),
        ("Product Sales",         1500.0,  1500.0),
        ("House Calls",            500.0,   500.0),
        ("Consulting",            1000.0,  1200.0),
        ("Training Workshops",    1000.0,   600.0),
    ];

    for (i, (item, budget, actual)) in revenues.iter().enumerate() {
        let r = rev_start + 1 + i as u32;
        ws.write_with_format(r, 0, *item, &txt)?;
        ws.write_with_format(r, 1, *budget, &money)?;
        ws.write_with_format(r, 2, *actual, &money)?;
    }
    let rev_last = rev_start + revenues.len() as u32;

    ws.set_column_width(0, 24.0)?;
    ws.set_column_width(1, 14.0)?;
    ws.set_column_width(2, 14.0)?;

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Budget Summary (all formulas)
    // ═══════════════════════════════════════════════════════════
    let ws2 = wb.add_worksheet_with_name("Budget Summary")?;
    ws2.hide_gridlines();

    let sw = [2.0, 16.0, 14.0, 20.0, 4.0, 24.0, 12.0, 10.0, 2.0];
    for (c, w) in sw.iter().enumerate() { ws2.set_column_width(c as u16, *w)?; }

    // Colors
    let orange = "#ED7D31";
    let dark_orange = "#C55A11";
    let green = "#70AD47";
    let dark_green = "#548235";
    let navy = "#2B579A";
    let dark_navy = "#1B2A4A";
    let gray = "#808080";
    let light_gray = "#F2F2F2";
    let border = "#D6DCE4";
    let red_text = "#C00000";
    let green_text = "#0D7C3D";

    // Styles
    let title = Format::new().bold().font_size(26.0).font_color(dark_navy).align(Align::Left).align(Align::Bottom);
    let title_line = Format::new().background_color(orange);

    let section_title = |color: &str| -> Format {
        Format::new().bold().font_size(20.0).font_color(color).align(Align::Left).align(Align::Bottom)
    };
    let top5_title = Format::new().bold().font_size(14.0).font_color(gray).align(Align::Left).align(Align::Bottom);

    let lbl = Format::new().bold().font_size(11.0).font_color(dark_navy).align(Align::Left)
        .border(BorderStyle::Thin).border_color(border);
    let val_money = Format::new().bold().font_size(11.0).font_color(dark_navy).align(Align::Right)
        .num_format("$#,##0").border(BorderStyle::Thin).border_color(border);
    let var_money = Format::new().bold().font_size(11.0).align(Align::Right)
        .num_format("($#,##0);($#,##0)").border(BorderStyle::Thin).border_color(border);
    let var_pct = Format::new().bold().font_size(11.0).align(Align::Right)
        .num_format("0%").border(BorderStyle::Thin).border_color(border);

    let top5_item = Format::new().font_size(10.0).font_color(dark_navy).align(Align::Left)
        .border(BorderStyle::Thin).border_color(border);
    let top5_money = Format::new().font_size(10.0).font_color(dark_navy).align(Align::Right)
        .num_format("$#,##0").border(BorderStyle::Thin).border_color(border);
    let top5_pct = Format::new().font_size(10.0).font_color(gray).align(Align::Right)
        .num_format("0%").border(BorderStyle::Thin).border_color(border);

    let bar_cell = |color: &str| -> Format {
        Format::new().font_size(9.0).font_color(color).background_color(color)
            .align(Align::Left).border(BorderStyle::Thin).border_color(border)
    };

    // ── Title ──
    let mut r = 0u32;
    ws2.write_with_format(r, 1, "Budget Summary", &title)?;
    ws2.set_row_height(r, 40.0)?; r += 1;
    for c in 1..=7u16 { ws2.write_with_format(r, c, "", &title_line)?; }
    ws2.set_row_height(r, 3.0)?; r += 2;

    // ════════════════════════════════════════
    // COSTS SECTION
    // ════════════════════════════════════════
    let costs_section = r;
    ws2.write_with_format(r, 1, "Costs", &section_title(dark_orange))?;
    ws2.write_with_format(r, 5, "Top 5 Costs", &top5_title)?;
    ws2.set_row_height(r, 30.0)?; r += 1;

    // Budget row
    ws2.write_with_format(r, 1, "Budget", &lbl)?;
    // =SUM('Budget Data'!B2:B11)
    ws2.write_formula(r, 2, &format!("SUM('Budget Data'!B2:B{})", cost_last + 1))?;
    ws2.set_cell_format(r, 2, &val_money)?;
    // Visual bar
    ws2.write_with_format(r, 3, "████████████", &bar_cell(orange))?;

    // Top 5 costs — LARGE + INDEX/MATCH formulas
    for rank in 1..=5u32 {
        let tr = costs_section + rank;
        // Item name = INDEX(A2:A11, MATCH(LARGE(C2:C11,rank), C2:C11, 0))
        ws2.write_formula(tr, 5, &format!(
            "INDEX('Budget Data'!A2:A{cl},MATCH(LARGE('Budget Data'!C2:C{cl},{rank}),'Budget Data'!C2:C{cl},0))",
            cl = cost_last + 1
        ))?;
        ws2.set_cell_format(tr, 5, &top5_item)?;
        // Amount = LARGE(C2:C11, rank)
        ws2.write_formula(tr, 6, &format!("LARGE('Budget Data'!C2:C{},{rank})", cost_last + 1))?;
        ws2.set_cell_format(tr, 6, &top5_money)?;
        // % of total = amount / total actual costs
        ws2.write_formula(tr, 7, &format!("G{}/SUM('Budget Data'!C2:C{})", tr + 1, cost_last + 1))?;
        ws2.set_cell_format(tr, 7, &top5_pct)?;
    }

    r += 1;

    // Actual row
    ws2.write_with_format(r, 1, "Actual", &lbl)?;
    ws2.write_formula(r, 2, &format!("SUM('Budget Data'!C2:C{})", cost_last + 1))?;
    ws2.set_cell_format(r, 2, &val_money)?;
    ws2.write_with_format(r, 3, "███████████", &bar_cell(dark_orange))?;
    r += 2;

    // Variance = Actual - Budget (for costs, negative = good)
    let budget_cell = format!("C{}", costs_section + 2);
    let actual_cell = format!("C{}", costs_section + 3);
    ws2.write_with_format(r, 1, "Variance", &lbl)?;
    ws2.write_formula(r, 2, &format!("{actual_cell}-{budget_cell}"))?;
    ws2.set_cell_format(r, 2, &var_money)?;
    r += 1;

    ws2.write_with_format(r, 1, "Variance %", &lbl)?;
    ws2.write_formula(r, 2, &format!("({actual_cell}-{budget_cell})/{budget_cell}"))?;
    ws2.set_cell_format(r, 2, &var_pct)?;
    // Arrow indicator: IF negative = ▼ (good for costs), positive = ▲ (bad)
    ws2.write_formula(r, 3, &format!("IF({actual_cell}<{budget_cell},\"▼\",\"▲\")"))?;
    r += 2;

    // ════════════════════════════════════════
    // REVENUES SECTION
    // ════════════════════════════════════════
    let rev_section = r;
    ws2.write_with_format(r, 1, "Revenues", &section_title(dark_green))?;
    ws2.write_with_format(r, 5, "Top 5 Revenues", &top5_title)?;
    ws2.set_row_height(r, 30.0)?; r += 1;

    // Budget
    ws2.write_with_format(r, 1, "Budget", &lbl)?;
    ws2.write_formula(r, 2, &format!("SUM('Budget Data'!B{}:B{})", rev_start + 2, rev_last + 1))?;
    ws2.set_cell_format(r, 2, &val_money)?;
    ws2.write_with_format(r, 3, "████████████", &bar_cell(green))?;

    // Top 5 revenues
    for rank in 1..=5u32 {
        let tr = rev_section + rank;
        ws2.write_formula(tr, 5, &format!(
            "INDEX('Budget Data'!A{}:A{rl},MATCH(LARGE('Budget Data'!C{}:C{rl},{rank}),'Budget Data'!C{}:C{rl},0))",
            rev_start + 2, rev_start + 2, rev_start + 2, rl = rev_last + 1
        ))?;
        ws2.set_cell_format(tr, 5, &top5_item)?;
        ws2.write_formula(tr, 6, &format!("LARGE('Budget Data'!C{}:C{},{rank})", rev_start + 2, rev_last + 1))?;
        ws2.set_cell_format(tr, 6, &top5_money)?;
        ws2.write_formula(tr, 7, &format!("G{}/SUM('Budget Data'!C{}:C{})", tr + 1, rev_start + 2, rev_last + 1))?;
        ws2.set_cell_format(tr, 7, &top5_pct)?;
    }

    r += 1;

    // Actual
    ws2.write_with_format(r, 1, "Actual", &lbl)?;
    ws2.write_formula(r, 2, &format!("SUM('Budget Data'!C{}:C{})", rev_start + 2, rev_last + 1))?;
    ws2.set_cell_format(r, 2, &val_money)?;
    ws2.write_with_format(r, 3, "██████████████", &bar_cell(dark_green))?;
    r += 2;

    let rev_budget = format!("C{}", rev_section + 2);
    let rev_actual = format!("C{}", rev_section + 3);
    ws2.write_with_format(r, 1, "Variance", &lbl)?;
    ws2.write_formula(r, 2, &format!("{rev_actual}-{rev_budget}"))?;
    ws2.set_cell_format(r, 2, &var_money)?;
    r += 1;

    ws2.write_with_format(r, 1, "Variance %", &lbl)?;
    ws2.write_formula(r, 2, &format!("({rev_actual}-{rev_budget})/{rev_budget}"))?;
    ws2.set_cell_format(r, 2, &var_pct)?;
    ws2.write_formula(r, 3, &format!("IF({rev_actual}>{rev_budget},\"▲\",\"▼\")"))?;
    r += 2;

    // ════════════════════════════════════════
    // PROFITS SECTION
    // ════════════════════════════════════════
    let profit_section = r;
    ws2.write_with_format(r, 1, "Profits", &section_title(dark_navy))?;
    ws2.set_row_height(r, 30.0)?; r += 1;

    // Profit Budget = Revenue Budget - Cost Budget
    ws2.write_with_format(r, 1, "Budget", &lbl)?;
    ws2.write_formula(r, 2, &format!("{rev_budget}-{budget_cell}"))?;
    ws2.set_cell_format(r, 2, &val_money)?;
    ws2.write_with_format(r, 3, "████", &bar_cell(navy))?;
    r += 1;

    // Profit Actual = Revenue Actual - Cost Actual
    ws2.write_with_format(r, 1, "Actual", &lbl)?;
    ws2.write_formula(r, 2, &format!("{rev_actual}-{actual_cell}"))?;
    ws2.set_cell_format(r, 2, &val_money)?;
    ws2.write_with_format(r, 3, "████████████", &bar_cell(dark_navy))?;
    r += 2;

    let profit_budget = format!("C{}", profit_section + 2);
    let profit_actual = format!("C{}", profit_section + 3);
    ws2.write_with_format(r, 1, "Variance", &lbl)?;
    ws2.write_formula(r, 2, &format!("{profit_actual}-{profit_budget}"))?;
    ws2.set_cell_format(r, 2, &var_money)?;
    r += 1;

    ws2.write_with_format(r, 1, "Variance %", &lbl)?;
    ws2.write_formula(r, 2, &format!("({profit_actual}-{profit_budget})/{profit_budget}"))?;
    ws2.set_cell_format(r, 2, &var_pct)?;
    ws2.write_formula(r, 3, &format!("IF({profit_actual}>{profit_budget},\"▲\",\"▼\")"))?;

    // Print setup
    ws2.set_portrait();
    ws2.set_paper_size(1);
    ws2.set_fit_to_page(1, 1);
    ws2.set_margins(0.5, 0.5, 0.5, 0.5);
    ws2.set_header("&CBudget Summary");
    ws2.set_footer("&CPage &P  |  &D");

    // ── Save ──
    wb.set_active_sheet(1);
    let path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into()))
        .join("Downloads/budget_summary.xlsx");
    wb.save(&path)?;
    println!("✅ Budget Summary saved to {}", path.display());
    Ok(())
}
