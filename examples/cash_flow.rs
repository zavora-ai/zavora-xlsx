use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A";
    let green = "#0D7C3D";
    let _red = "#C00000";
    let blue = "#2B579A";
    let amber = "#E67E22";
    let border = "#D6DCE4";
    let light = "#F5F7FA";
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let ws = wb.worksheet(0)?;
    ws.set_name("Cash Flow")?;
    ws.hide_gridlines();

    ws.set_column_width(0, 2.0)?;
    ws.set_column_width(1, 28.0)?;
    for c in 2..=13u16 {
        ws.set_column_width(c, 11.0)?;
    }
    ws.set_column_width(14, 13.0)?;

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
    let item_f = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Thin)
        .border_color(border);
    let val_f = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .num_format("#,##0")
        .border(BorderStyle::Thin)
        .border_color(border);
    let val_a = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .num_format("#,##0")
        .background_color(light)
        .border(BorderStyle::Thin)
        .border_color(border);
    let stl = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .background_color("#E8E8E8")
        .border(BorderStyle::Thin);
    let stv = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .num_format("#,##0")
        .background_color("#E8E8E8")
        .border(BorderStyle::Thin);
    let title = Format::new()
        .bold()
        .font_size(20.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::Bottom);
    let divider = Format::new().background_color(blue);

    macro_rules! sec {
        ($r:ident, $name:expr, $color:expr) => {{
            let sf = Format::new()
                .bold()
                .font_size(11.0)
                .font_color("#FFFFFF")
                .background_color($color)
                .align(Align::Left)
                .border(BorderStyle::Thin);
            let sv = Format::new()
                .bold()
                .background_color($color)
                .border(BorderStyle::Thin);
            ws.write_with_format($r, 1, $name, &sf).unwrap();
            for c in 2..=14u16 {
                ws.write_with_format($r, c, "", &sv).unwrap();
            }
            ws.set_row_height($r, 24.0).unwrap();
            $r += 1;
        }};
    }
    macro_rules! row {
        ($r:ident, $name:expr, $vals:expr, $alt:expr) => {{
            let vf = if $alt { &val_a } else { &val_f };
            ws.write_with_format($r, 1, $name, &item_f).unwrap();
            for (c, v) in $vals.iter().enumerate() {
                ws.write_with_format($r, (c + 2) as u16, *v, vf).unwrap();
            }
            ws.write_formula($r, 14, &format!("SUM(C{}:N{})", $r + 1, $r + 1))
                .unwrap();
            ws.set_cell_format($r, 14, &stv).unwrap();
            $r += 1;
        }};
    }

    let mut r = 0u32;
    ws.set_row_height(r, 6.0)?;
    r += 1;
    ws.write_with_format(r, 1, "Cash Flow Statement", &title)?;
    ws.set_row_height(r, 32.0)?;
    r += 1;
    ws.write_with_format(
        r,
        1,
        "Fiscal Year 2025  |  All formulas auto-calculate",
        &Format::new().font_size(10.0).font_color("#667085").italic(),
    )?;
    r += 1;
    for c in 1..=14u16 {
        ws.write_with_format(r, c, "", &divider)?;
    }
    ws.set_row_height(r, 3.0)?;
    r += 2;

    ws.write_with_format(r, 1, "Account", &hdr_l)?;
    for (c, m) in months.iter().enumerate() {
        ws.write_with_format(r, (c + 2) as u16, *m, &hdr)?;
    }
    ws.write_with_format(
        r,
        14,
        "TOTAL",
        &Format::new()
            .bold()
            .font_size(10.0)
            .font_color("#FFFFFF")
            .background_color(blue)
            .align(Align::Center)
            .border(BorderStyle::Thin),
    )?;
    ws.set_row_height(r, 22.0)?;
    r += 1;

    // OPERATING
    sec!(r, "OPERATING ACTIVITIES", green);
    let op_s = r;
    row!(
        r,
        "  Net Income",
        &[
            45000.0, 48000.0, 42000.0, 52000.0, 55000.0, 50000.0, 58000.0, 60000.0, 56000.0,
            62000.0, 65000.0, 72000.0
        ],
        false
    );
    row!(
        r,
        "  Depreciation",
        &[
            8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8500.0, 8500.0, 8500.0, 8500.0, 8500.0,
            8500.0
        ],
        true
    );
    row!(
        r,
        "  Change in AR",
        &[
            -5000.0, 3000.0, -8000.0, 2000.0, -4000.0, 6000.0, -3000.0, -7000.0, 5000.0, -2000.0,
            4000.0, -10000.0
        ],
        false
    );
    row!(
        r,
        "  Change in Inventory",
        &[
            -3000.0, -2000.0, 4000.0, -5000.0, 1000.0, -3000.0, 2000.0, -4000.0, 3000.0, -1000.0,
            -2000.0, 5000.0
        ],
        true
    );
    row!(
        r,
        "  Change in AP",
        &[
            4000.0, -2000.0, 5000.0, 3000.0, -1000.0, 4000.0, -2000.0, 6000.0, -3000.0, 2000.0,
            5000.0, -4000.0
        ],
        false
    );
    let op_e = r - 1;
    let op_t = r;
    ws.write_with_format(r, 1, "Net Cash from Operations", &stl)?;
    for c in 2..=14u16 {
        ws.write_formula(
            r,
            c,
            &format!("SUM({}{}:{}{})", col(c), op_s + 1, col(c), op_e + 1),
        )?;
        ws.set_cell_format(r, c, &stv)?;
    }
    ws.set_row_height(r, 24.0)?;
    r += 2;

    // INVESTING
    sec!(r, "INVESTING ACTIVITIES", blue);
    let inv_s = r;
    row!(
        r,
        "  Capital Expenditures",
        &[
            -15000.0, -8000.0, -12000.0, -20000.0, -10000.0, -5000.0, -25000.0, -8000.0, -15000.0,
            -10000.0, -18000.0, -30000.0
        ],
        false
    );
    row!(
        r,
        "  Asset Sales",
        &[
            0.0, 5000.0, 0.0, 0.0, 12000.0, 0.0, 0.0, 0.0, 8000.0, 0.0, 0.0, 15000.0
        ],
        true
    );
    row!(
        r,
        "  Investments",
        &[
            -10000.0, 0.0, -5000.0, 0.0, -10000.0, 0.0, -15000.0, 0.0, -5000.0, 0.0, -10000.0, 0.0
        ],
        false
    );
    let inv_e = r - 1;
    let inv_t = r;
    ws.write_with_format(r, 1, "Net Cash from Investing", &stl)?;
    for c in 2..=14u16 {
        ws.write_formula(
            r,
            c,
            &format!("SUM({}{}:{}{})", col(c), inv_s + 1, col(c), inv_e + 1),
        )?;
        ws.set_cell_format(r, c, &stv)?;
    }
    ws.set_row_height(r, 24.0)?;
    r += 2;

    // FINANCING
    sec!(r, "FINANCING ACTIVITIES", amber);
    let fin_s = r;
    row!(
        r,
        "  Loan Proceeds",
        &[
            0.0, 0.0, 50000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        ],
        false
    );
    row!(
        r,
        "  Loan Repayments",
        &[
            -5000.0, -5000.0, -5000.0, -5000.0, -5000.0, -5000.0, -5000.0, -5000.0, -5000.0,
            -5000.0, -5000.0, -5000.0
        ],
        true
    );
    row!(
        r,
        "  Dividends Paid",
        &[
            0.0, 0.0, 0.0, -20000.0, 0.0, 0.0, 0.0, -20000.0, 0.0, 0.0, 0.0, -25000.0
        ],
        false
    );
    row!(
        r,
        "  Equity Issued",
        &[
            0.0, 0.0, 0.0, 0.0, 0.0, 100000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        ],
        true
    );
    let fin_e = r - 1;
    let fin_t = r;
    ws.write_with_format(r, 1, "Net Cash from Financing", &stl)?;
    for c in 2..=14u16 {
        ws.write_formula(
            r,
            c,
            &format!("SUM({}{}:{}{})", col(c), fin_s + 1, col(c), fin_e + 1),
        )?;
        ws.set_cell_format(r, c, &stv)?;
    }
    ws.set_row_height(r, 24.0)?;
    r += 2;

    // NET CHANGE + BALANCES
    let nc = r;
    let gl = Format::new()
        .bold()
        .font_size(11.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Medium);
    let gv = Format::new()
        .bold()
        .font_size(11.0)
        .font_color("#FFFFFF")
        .num_format("#,##0")
        .background_color(navy)
        .align(Align::Right)
        .border(BorderStyle::Medium);
    ws.write_with_format(r, 1, "NET CHANGE IN CASH", &gl)?;
    for c in 2..=14u16 {
        ws.write_formula(
            r,
            c,
            &format!(
                "{}{}+{}{}+{}{}",
                col(c),
                op_t + 1,
                col(c),
                inv_t + 1,
                col(c),
                fin_t + 1
            ),
        )?;
        ws.set_cell_format(r, c, &gv)?;
    }
    ws.set_row_height(r, 28.0)?;
    r += 1;

    // Opening balance
    let ob = r;
    ws.write_with_format(r, 1, "Opening Cash Balance", &item_f)?;
    ws.write_with_format(r, 2, 250000.0, &val_f)?; // Jan opening
    for c in 3..=14u16 {
        ws.write_formula(r, c, &format!("{}{}", col(c - 1), r + 2))?;
        ws.set_cell_format(r, c, &val_f)?;
    } // opening = prev closing
    r += 1;

    // Closing balance
    let cb = r;
    let cb_fmt = Format::new()
        .bold()
        .font_size(11.0)
        .font_color(green)
        .align(Align::Right)
        .num_format("#,##0")
        .background_color("#E8F5E9")
        .border(BorderStyle::Medium)
        .border_color(green);
    ws.write_with_format(
        r,
        1,
        "Closing Cash Balance",
        &Format::new()
            .bold()
            .font_size(11.0)
            .font_color(green)
            .align(Align::Left)
            .background_color("#E8F5E9")
            .border(BorderStyle::Medium)
            .border_color(green),
    )?;
    for c in 2..=14u16 {
        ws.write_formula(r, c, &format!("{}{}+{}{}", col(c), ob + 1, col(c), nc + 1))?;
        ws.set_cell_format(r, c, &cb_fmt)?;
    }
    ws.set_row_height(r, 26.0)?;
    r += 2;

    // Chart
    let cat = "'Cash Flow'!$C$6:$N$6";
    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Monthly Cash Flow");
    chart.set_width(720);
    chart.set_height(320);
    chart.set_y_axis_name("USD");
    chart.set_legend_position(LegendPosition::Bottom);

    let s1 = chart.add_series();
    s1.set_values(&format!("'Cash Flow'!$C${}:$N${}", op_t + 1, op_t + 1));
    s1.set_categories(cat);
    s1.set_name("Operating");
    s1.set_color(green);

    let s2 = chart.add_series();
    s2.set_values(&format!("'Cash Flow'!$C${}:$N${}", inv_t + 1, inv_t + 1));
    s2.set_categories(cat);
    s2.set_name("Investing");
    s2.set_color(blue);

    let s3 = chart.add_series();
    s3.set_values(&format!("'Cash Flow'!$C${}:$N${}", fin_t + 1, fin_t + 1));
    s3.set_categories(cat);
    s3.set_name("Financing");
    s3.set_color(amber);

    let s4 = chart.add_series();
    s4.set_values(&format!("'Cash Flow'!$C${}:$N${}", cb + 1, cb + 1));
    s4.set_categories(cat);
    s4.set_name("Cash Balance");
    s4.set_color(navy);
    s4.set_chart_type(ChartType::Line);
    s4.set_marker(MarkerType::Circle);
    s4.set_data_labels(true);

    ws.insert_chart(r, 1, &chart)?;

    ws.set_freeze_panes(6, 2)?;
    ws.set_landscape();
    ws.set_fit_to_page(1, 1);
    ws.set_header("&CCash Flow Statement — FY2025");
    ws.set_footer("&CConfidential  |  &D");

    let path = home("cash_flow_statement.xlsx");
    wb.save(&path)?;
    println!("✅ Cash Flow Statement saved to {}", path.display());
    Ok(())
}

fn col(c: u16) -> String {
    zavora_xlsx::utility::col_to_letter(c)
}
fn home(n: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("output").join(n)
}
