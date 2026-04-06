use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    let navy = "#1B2A4A"; let blue = "#2B579A"; let green = "#0D7C3D";
    let red = "#C00000"; let gold = "#C68E17"; let border = "#D6DCE4";

    let ws = wb.worksheet(0)?;
    ws.set_name("Mortgage Calculator")?;
    ws.hide_gridlines();

    let cw = [2.0, 22.0, 16.0, 4.0, 10.0, 14.0, 14.0, 14.0, 14.0, 2.0];
    for (c, w) in cw.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }

    // Styles
    let title = Format::new().bold().font_size(22.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let sub = Format::new().font_size(10.0).font_color("#667085").italic();
    let divider = Format::new().background_color(blue);
    let label = Format::new().bold().font_size(11.0).font_color(navy).align(Align::Right)
        .border(BorderStyle::Thin).border_color(border);
    let input_money = Format::new().font_size(12.0).font_color(blue).bold()
        .num_format("$#,##0").align(Align::Center).background_color("#EBF5FB")
        .border(BorderStyle::Medium).border_color(blue);
    let input_pct = Format::new().font_size(12.0).font_color(blue).bold()
        .num_format("0.00%").align(Align::Center).background_color("#EBF5FB")
        .border(BorderStyle::Medium).border_color(blue);
    let input_num = Format::new().font_size(12.0).font_color(blue).bold()
        .align(Align::Center).background_color("#EBF5FB")
        .border(BorderStyle::Medium).border_color(blue);
    let result_label = Format::new().bold().font_size(11.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Right).border(BorderStyle::Thin);
    let result_val = Format::new().bold().font_size(14.0).font_color("#FFFFFF")
        .num_format("$#,##0.00").background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let section = Format::new().bold().font_size(13.0).font_color("#FFFFFF")
        .background_color(blue).align(Align::Left).align(Align::VerticalCenter);
    let tbl_hdr = Format::new().bold().font_size(9.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let cell_n = Format::new().font_size(9.0).font_color(navy).align(Align::Center)
        .border(BorderStyle::Thin).border_color(border);
    let cell_m = Format::new().font_size(9.0).font_color(navy).align(Align::Right)
        .num_format("$#,##0.00").border(BorderStyle::Thin).border_color(border);
    let cell_green = Format::new().font_size(9.0).font_color(green).bold().align(Align::Right)
        .num_format("$#,##0.00").border(BorderStyle::Thin).border_color(border);
    let cell_red = Format::new().font_size(9.0).font_color(red).align(Align::Right)
        .num_format("$#,##0.00").border(BorderStyle::Thin).border_color(border);

    let mut r = 0u32;

    // Title
    ws.set_row_height(r, 6.0)?; r += 1;
    ws.write_with_format(r, 1, "🏠 Mortgage Payment Calculator", &title)?;
    ws.set_row_height(r, 36.0)?; r += 1;
    ws.write_with_format(r, 1, "Enter your loan details below — all results update automatically", &sub)?;
    r += 1;
    for c in 1..=8u16 { ws.write_with_format(r, c, "", &divider)?; }
    ws.set_row_height(r, 3.0)?; r += 2;

    // ── Input Section ──
    // Loan Amount (C6)
    ws.write_with_format(r, 1, "Home Price", &label)?;
    ws.write_with_format(r, 2, 450000.0, &input_money)?;
    r += 1;
    ws.write_with_format(r, 1, "Down Payment", &label)?;
    ws.write_with_format(r, 2, 90000.0, &input_money)?;
    r += 1;
    ws.write_with_format(r, 1, "Loan Amount", &result_label)?;
    ws.write_formula(r, 2, "C6-C7")?; ws.set_cell_format(r, 2, &result_val)?;
    r += 1;
    ws.write_with_format(r, 1, "Annual Interest Rate", &label)?;
    ws.write_with_format(r, 2, 0.0675, &input_pct)?;
    r += 1;
    ws.write_with_format(r, 1, "Loan Term (years)", &label)?;
    ws.write_with_format(r, 2, 30.0, &input_num)?;
    r += 1;
    ws.write_with_format(r, 1, "Start Date", &label)?;
    ws.write_with_format(r, 2, ExcelDateTime::from_ymd(2025, 5, 1).unwrap(),
        &Format::new().font_size(12.0).font_color(blue).bold().num_format("mmmm yyyy")
            .align(Align::Center).background_color("#EBF5FB").border(BorderStyle::Medium).border_color(blue))?;
    r += 2;

    // ── Results Section ──
    // Monthly Payment = PMT formula
    ws.write_with_format(r, 1, "Monthly Payment", &result_label)?;
    ws.write_formula(r, 2, "-PMT(C9/12,C10*12,C8)")?;
    ws.set_cell_format(r, 2, &Format::new().bold().font_size(18.0).font_color("#FFFFFF")
        .num_format("$#,##0.00").background_color(green).align(Align::Center).border(BorderStyle::Thin))?;
    let pmt_row = r;
    r += 1;

    ws.write_with_format(r, 1, "Total Payments", &result_label)?;
    ws.write_formula(r, 2, &format!("C{}*C10*12", pmt_row + 1))?;
    ws.set_cell_format(r, 2, &result_val)?;
    r += 1;

    ws.write_with_format(r, 1, "Total Interest", &result_label)?;
    ws.write_formula(r, 2, &format!("C{}-C8", r))?;
    ws.set_cell_format(r, 2, &Format::new().bold().font_size(14.0).font_color("#FFFFFF")
        .num_format("$#,##0.00").background_color(red).align(Align::Center).border(BorderStyle::Thin))?;
    r += 1;

    ws.write_with_format(r, 1, "Down Payment %", &result_label)?;
    ws.write_formula(r, 2, "C7/C6")?;
    ws.set_cell_format(r, 2, &Format::new().bold().font_size(14.0).font_color("#FFFFFF")
        .num_format("0.0%").background_color(navy).align(Align::Center).border(BorderStyle::Thin))?;
    r += 2;

    // ── Amortization Schedule ──
    ws.merge_range(r, 1, r, 8, "  📋 Amortization Schedule", &section)?;
    ws.set_row_height(r, 28.0)?; r += 1;

    let headers = ["Month", "Payment", "Principal", "Interest", "Balance", "Cum. Interest"];
    let hcols = [4u16, 5, 6, 7, 8, 8]; // skip — use E-I
    ws.write_with_format(r, 4, headers[0], &tbl_hdr)?;
    ws.write_with_format(r, 5, headers[1], &tbl_hdr)?;
    ws.write_with_format(r, 6, headers[2], &tbl_hdr)?;
    ws.write_with_format(r, 7, headers[3], &tbl_hdr)?;
    ws.write_with_format(r, 8, headers[4], &tbl_hdr)?;
    ws.set_row_height(r, 20.0)?; r += 1;

    let sched_start = r;
    // Generate 360 months (30 years) of amortization with formulas
    let months = 360u32;
    for m in 1..=months.min(360) {
        let mr = sched_start + m - 1;
        // Month number
        ws.write_with_format(mr, 4, m as f64, &cell_n)?;
        // Payment = fixed monthly payment
        ws.write_formula(mr, 5, &format!("C{}", pmt_row + 1))?;
        ws.set_cell_format(mr, 5, &cell_m)?;
        // Interest = previous balance * monthly rate
        if m == 1 {
            ws.write_formula(mr, 7, &format!("C8*C9/12"))?;
        } else {
            ws.write_formula(mr, 7, &format!("I{}*C9/12", mr))?;
        }
        ws.set_cell_format(mr, 7, &cell_red)?;
        // Principal = Payment - Interest
        ws.write_formula(mr, 6, &format!("F{}-H{}", mr + 1, mr + 1))?;
        ws.set_cell_format(mr, 6, &cell_green)?;
        // Balance = previous balance - principal
        if m == 1 {
            ws.write_formula(mr, 8, &format!("C8-G{}", mr + 1))?;
        } else {
            ws.write_formula(mr, 8, &format!("I{}-G{}", mr, mr + 1))?;
        }
        ws.set_cell_format(mr, 8, &cell_m)?;
    }

    // Data bars on principal (green) and interest (red)
    let sched_end = sched_start + months.min(60) - 1; // first 5 years for data bars
    ws.add_conditional_format(sched_start, 6, sched_end, 6, ConditionalFormatDataBar::new(green))?;
    ws.add_conditional_format(sched_start, 7, sched_end, 7, ConditionalFormatDataBar::new(red))?;

    // ── Chart: Principal vs Interest over time ──
    let chart_row = 13u32;
    let mut chart = Chart::new(ChartType::Area);
    chart.set_title("Principal vs Interest Over Time");
    chart.set_width(480); chart.set_height(280);
    chart.set_y_axis_name("$ per payment");
    chart.set_legend_position(LegendPosition::Bottom);

    // Use first 60 months for chart readability
    let cs = chart.add_series();
    cs.set_values(&format!("'Mortgage Calculator'!$G${}:$G${}", sched_start + 1, sched_start + 60));
    cs.set_name("Principal");
    cs.set_color(green);

    let is = chart.add_series();
    is.set_values(&format!("'Mortgage Calculator'!$H${}:$H${}", sched_start + 1, sched_start + 60));
    is.set_name("Interest");
    is.set_color(red);

    ws.insert_chart(chart_row, 4, &chart)?;

    // Freeze panes
    ws.set_freeze_panes(sched_start, 0)?;

    // Print
    ws.set_landscape(); ws.set_fit_to_page(1, 0);
    ws.set_margins(0.3, 0.3, 0.3, 0.3);
    ws.set_header("&CMortgage Payment Calculator");

    let path = home("mortgage_calculator.xlsx");
    wb.save(&path)?;
    println!("✅ Mortgage Calculator saved to {}", path.display());
    Ok(())
}

fn home(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(name)
}
