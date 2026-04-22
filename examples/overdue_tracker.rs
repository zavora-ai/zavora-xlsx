use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    // ── Colors ──
    let red = "#C00000";
    let green = "#0D7C3D";
    let amber = "#E8A317";
    let navy = "#1B2A4A";
    let light_red = "#FDE8E8";
    let light_green = "#E8F5E9";
    let light_amber = "#FFF8E1";
    let border = "#D6DCE4";

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Invoice Data (source of truth)
    // ═══════════════════════════════════════════════════════════
    let last;
    {
        let ws = wb.worksheet(0)?;
        ws.set_name("Invoices")?;

        let headers = [
            "Invoice #",
            "Client",
            "Amount",
            "Issue Date",
            "Due Date",
            "Paid Date",
            "Days Overdue",
            "Status",
            "Amount Overdue",
        ];
        let hdr = Format::new()
            .bold()
            .font_size(10.0)
            .font_color("#FFFFFF")
            .background_color(navy)
            .align(Align::Center)
            .border(BorderStyle::Thin);
        for (c, h) in headers.iter().enumerate() {
            ws.write_with_format(0, c as u16, *h, &hdr)?;
        }

        // Sample invoices — mix of overdue, due soon, and paid
        let invoices: Vec<(
            &str,
            &str,
            f64,
            (i32, u32, u32),
            (i32, u32, u32),
            Option<(i32, u32, u32)>,
        )> = vec![
            (
                "INV-001",
                "Acme Corp",
                15000.0,
                (2025, 1, 5),
                (2025, 2, 4),
                Some((2025, 2, 1)),
            ),
            (
                "INV-002",
                "Global Traders",
                8500.0,
                (2025, 1, 15),
                (2025, 2, 14),
                Some((2025, 2, 20)),
            ),
            (
                "INV-003",
                "Zenith Ltd",
                22000.0,
                (2025, 1, 20),
                (2025, 2, 19),
                None,
            ),
            (
                "INV-004",
                "Apex Industries",
                4200.0,
                (2025, 2, 1),
                (2025, 3, 3),
                Some((2025, 3, 1)),
            ),
            (
                "INV-005",
                "Summit Partners",
                31500.0,
                (2025, 2, 10),
                (2025, 3, 12),
                None,
            ),
            (
                "INV-006",
                "Horizon Group",
                12800.0,
                (2025, 2, 15),
                (2025, 3, 17),
                None,
            ),
            (
                "INV-007",
                "Pinnacle Inc",
                6700.0,
                (2025, 3, 1),
                (2025, 3, 31),
                Some((2025, 3, 28)),
            ),
            (
                "INV-008",
                "Vertex Solutions",
                19400.0,
                (2025, 3, 5),
                (2025, 4, 4),
                None,
            ),
            (
                "INV-009",
                "Crest Consulting",
                8900.0,
                (2025, 3, 10),
                (2025, 4, 9),
                Some((2025, 4, 15)),
            ),
            (
                "INV-010",
                "Atlas Corp",
                45000.0,
                (2025, 3, 15),
                (2025, 4, 14),
                None,
            ),
            (
                "INV-011",
                "Nova Enterprises",
                3200.0,
                (2025, 3, 20),
                (2025, 4, 19),
                Some((2025, 4, 10)),
            ),
            (
                "INV-012",
                "Orbit Tech",
                27600.0,
                (2025, 3, 25),
                (2025, 4, 24),
                None,
            ),
            (
                "INV-013",
                "Stellar Group",
                11200.0,
                (2025, 4, 1),
                (2025, 5, 1),
                None,
            ),
            (
                "INV-014",
                "Quantum Ltd",
                5800.0,
                (2025, 4, 3),
                (2025, 5, 3),
                None,
            ),
            (
                "INV-015",
                "Fusion Partners",
                16500.0,
                (2025, 4, 5),
                (2025, 5, 5),
                None,
            ),
        ];

        let date_fmt = Format::new()
            .num_format("yyyy-mm-dd")
            .align(Align::Center)
            .border(BorderStyle::Thin)
            .border_color(border);
        let money_fmt = Format::new()
            .num_format("$#,##0")
            .align(Align::Right)
            .border(BorderStyle::Thin)
            .border_color(border);
        let txt_fmt = Format::new()
            .align(Align::Left)
            .border(BorderStyle::Thin)
            .border_color(border);
        let center_fmt = Format::new()
            .align(Align::Center)
            .border(BorderStyle::Thin)
            .border_color(border);

        for (i, inv) in invoices.iter().enumerate() {
            let r = (i + 1) as u32;
            ws.write_with_format(r, 0, inv.0, &txt_fmt)?;
            ws.write_with_format(r, 1, inv.1, &txt_fmt)?;
            ws.write_with_format(r, 2, inv.2, &money_fmt)?;
            ws.write_with_format(
                r,
                3,
                ExcelDateTime::from_ymd(inv.3.0, inv.3.1, inv.3.2).unwrap(),
                &date_fmt,
            )?;
            ws.write_with_format(
                r,
                4,
                ExcelDateTime::from_ymd(inv.4.0, inv.4.1, inv.4.2).unwrap(),
                &date_fmt,
            )?;
            if let Some(pd) = inv.5 {
                ws.write_with_format(
                    r,
                    5,
                    ExcelDateTime::from_ymd(pd.0, pd.1, pd.2).unwrap(),
                    &date_fmt,
                )?;
            } else {
                ws.write_with_format(r, 5, "", &center_fmt)?;
            }
            // Days Overdue = IF(paid, paid-due, TODAY()-due) — only if overdue
            ws.write_formula(
                r,
                6,
                &format!(
                    "IF(F{r1}=\"\",MAX(0,TODAY()-E{r1}),MAX(0,F{r1}-E{r1}))",
                    r1 = r + 1
                ),
            )?;
            ws.set_cell_format(r, 6, &center_fmt)?;
            // Status = formula
            ws.write_formula(r, 7, &format!(
            "IF(F{r1}<>\"\",IF(F{r1}>E{r1},\"Paid Late\",\"Paid\"),IF(TODAY()>E{r1},\"OVERDUE\",IF(E{r1}-TODAY()<=7,\"Due Soon\",\"On Time\")))",
            r1 = r + 1
        ))?;
            ws.set_cell_format(r, 7, &center_fmt)?;
            // Amount Overdue = IF overdue, show amount, else 0
            ws.write_formula(
                r,
                8,
                &format!("IF(AND(F{r1}=\"\",TODAY()>E{r1}),C{r1},0)", r1 = r + 1),
            )?;
            ws.set_cell_format(r, 8, &money_fmt)?;
        }

        last = invoices.len() as u32;

        // Column widths
        let widths = [12.0, 20.0, 12.0, 14.0, 14.0, 14.0, 14.0, 12.0, 14.0];
        for (c, w) in widths.iter().enumerate() {
            ws.set_column_width(c as u16, *w)?;
        }

        // Conditional formatting on Status column
        ws.add_conditional_format(
            1,
            7,
            last,
            7,
            ConditionalFormatCell::new(CfOperator::EqualTo, 0.0),
        )?; // placeholder — text CF not supported, use data bars on overdue amount instead

        // Data bars on Days Overdue
        ws.add_conditional_format(1, 6, last, 6, ConditionalFormatDataBar::new(red))?;
        // Data bars on Amount Overdue
        ws.add_conditional_format(1, 8, last, 8, ConditionalFormatDataBar::new(amber))?;

        ws.set_autofilter(0, 0, last, 8);
        ws.set_freeze_panes(1, 0)?;
        ws.set_landscape();
        ws.set_fit_to_page(1, 1);
        ws.set_header("&COverdue Items Tracker");
        ws.set_footer("&CPage &P  |  &D");
    } // end sheet 1 scope

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Dashboard (all formulas)
    // ═══════════════════════════════════════════════════════════
    let ws2 = wb.add_worksheet_with_name("Dashboard")?;
    ws2.hide_gridlines();

    let dw = [2.0, 18.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 2.0];
    for (c, w) in dw.iter().enumerate() {
        ws2.set_column_width(c as u16, *w)?;
    }

    let title = Format::new()
        .bold()
        .font_size(22.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::Bottom);
    let subtitle = Format::new().font_size(10.0).font_color("#667085").italic();
    let divider = Format::new().background_color(red);
    let section = Format::new()
        .bold()
        .font_size(13.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Left)
        .align(Align::VerticalCenter);
    let kpi_label = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#667085")
        .align(Align::Center);
    let kpi_val = Format::new()
        .bold()
        .font_size(18.0)
        .font_color(navy)
        .align(Align::Center);
    let kpi_money = Format::new()
        .bold()
        .font_size(18.0)
        .font_color(red)
        .align(Align::Center)
        .num_format("$#,##0");
    let kpi_pct = Format::new()
        .bold()
        .font_size(18.0)
        .font_color(green)
        .align(Align::Center)
        .num_format("0%");
    let kpi_green = Format::new()
        .bold()
        .font_size(18.0)
        .font_color(green)
        .align(Align::Center);

    let mut r = 0u32;
    ws2.set_row_height(r, 6.0)?;
    r += 1;
    ws2.write_with_format(r, 1, "OVERDUE ITEMS TRACKER", &title)?;
    ws2.set_row_height(r, 36.0)?;
    r += 1;
    ws2.write_with_format(
        r,
        1,
        "Invoice aging dashboard  |  Auto-updates from Invoices sheet",
        &subtitle,
    )?;
    r += 1;
    for c in 1..=7u16 {
        ws2.write_with_format(r, c, "", &divider)?;
    }
    ws2.set_row_height(r, 3.0)?;
    r += 2;

    // ── KPI Row ──
    let labels = [
        "Total Invoices",
        "Total Amount",
        "Overdue Count",
        "Overdue Amount",
        "On Time %",
        "Avg Days Late",
        "Paid Count",
    ];
    for (c, l) in labels.iter().enumerate() {
        ws2.write_with_format(r, (c + 1) as u16, *l, &kpi_label)?;
    }
    r += 1;

    let ir = format!("Invoices!H2:H{}", last + 1); // status range
    let dr = format!("Invoices!G2:G{}", last + 1); // days overdue range
    let ar = format!("Invoices!C2:C{}", last + 1); // amount range
    let or = format!("Invoices!I2:I{}", last + 1); // overdue amount range

    // Total Invoices
    ws2.write_formula(r, 1, &format!("COUNTA(Invoices!A2:A{})", last + 1))?;
    ws2.set_cell_format(r, 1, &kpi_val)?;
    // Total Amount
    ws2.write_formula(r, 2, &format!("SUM({ar})"))?;
    ws2.set_cell_format(r, 2, &kpi_money)?;
    // Overdue Count
    ws2.write_formula(r, 3, &format!("COUNTIF({ir},\"OVERDUE\")"))?;
    ws2.set_cell_format(r, 3, &kpi_val)?;
    // Overdue Amount
    ws2.write_formula(r, 4, &format!("SUM({or})"))?;
    ws2.set_cell_format(r, 4, &kpi_money)?;
    // On Time %
    ws2.write_formula(
        r,
        5,
        &format!("COUNTIF({ir},\"Paid\")/COUNTA(Invoices!A2:A{})", last + 1),
    )?;
    ws2.set_cell_format(r, 5, &kpi_pct)?;
    // Avg Days Late
    ws2.write_formula(r, 6, &format!("AVERAGEIF({dr},\">0\")"))?;
    ws2.set_cell_format(r, 6, &kpi_val)?;
    // Paid Count
    ws2.write_formula(
        r,
        7,
        &format!("COUNTIF({ir},\"Paid\")+COUNTIF({ir},\"Paid Late\")"),
    )?;
    ws2.set_cell_format(r, 7, &kpi_green)?;
    ws2.set_row_height(r, 32.0)?;
    r += 2;

    // ── Status Breakdown ──
    ws2.merge_range(r, 1, r, 7, "  STATUS BREAKDOWN", &section)?;
    ws2.set_row_height(r, 28.0)?;
    r += 1;

    let status_hdr = Format::new()
        .bold()
        .font_size(10.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin);
    let sh = [
        "Status",
        "Count",
        "Amount",
        "% of Total",
        "Avg Days",
        "",
        "",
    ];
    for (c, h) in sh.iter().enumerate() {
        ws2.write_with_format(r, (c + 1) as u16, *h, &status_hdr)?;
    }
    r += 1;

    let statuses = [
        ("OVERDUE", red, light_red),
        ("Due Soon", amber, light_amber),
        ("On Time", green, light_green),
        ("Paid", green, light_green),
        ("Paid Late", amber, light_amber),
    ];

    let status_start = r;
    for &(status, color, bg) in &statuses {
        let s_fmt = Format::new()
            .bold()
            .font_size(10.0)
            .font_color(color)
            .background_color(bg)
            .align(Align::Left)
            .border(BorderStyle::Thin)
            .border_color(border);
        let n_fmt = Format::new()
            .font_size(10.0)
            .font_color(navy)
            .background_color(bg)
            .align(Align::Center)
            .border(BorderStyle::Thin)
            .border_color(border);
        let m_fmt = Format::new()
            .font_size(10.0)
            .font_color(navy)
            .background_color(bg)
            .align(Align::Center)
            .num_format("$#,##0")
            .border(BorderStyle::Thin)
            .border_color(border);
        let p_fmt = Format::new()
            .font_size(10.0)
            .font_color(navy)
            .background_color(bg)
            .align(Align::Center)
            .num_format("0%")
            .border(BorderStyle::Thin)
            .border_color(border);

        ws2.write_with_format(r, 1, status, &s_fmt)?;
        ws2.write_formula(r, 2, &format!("COUNTIF({ir},B{})", r + 1))?;
        ws2.set_cell_format(r, 2, &n_fmt)?;
        ws2.write_formula(r, 3, &format!("SUMIF({ir},B{},{ar})", r + 1))?;
        ws2.set_cell_format(r, 3, &m_fmt)?;
        ws2.write_formula(r, 4, &format!("IFERROR(C{}/SUM({ar}),0)", r + 1))?;
        ws2.set_cell_format(r, 4, &p_fmt)?;
        ws2.write_formula(
            r,
            5,
            &format!("IFERROR(AVERAGEIFS({dr},{ir},B{}),0)", r + 1),
        )?;
        ws2.set_cell_format(r, 5, &n_fmt)?;
        ws2.set_row_height(r, 24.0)?;
        r += 1;
    }

    r += 1;

    // ── Charts ──
    let chart_row = r;

    // Pie chart: status distribution
    let mut pie = Chart::new(ChartType::Pie);
    pie.set_title("Invoice Status Distribution");
    pie.set_width(420);
    pie.set_height(300);
    let ps = pie.add_series();
    ps.set_values(&format!(
        "Dashboard!$C${}:$C${}",
        status_start + 1,
        status_start + statuses.len() as u32
    ));
    ps.set_categories(&format!(
        "Dashboard!$B${}:$B${}",
        status_start + 1,
        status_start + statuses.len() as u32
    ));
    ps.set_name("Count");
    ps.set_data_labels(true);
    ws2.insert_chart(chart_row, 1, &pie)?;

    // Bar chart: overdue amounts
    let mut bar = Chart::new(ChartType::Column);
    bar.set_title("Amount by Status");
    bar.set_width(420);
    bar.set_height(300);
    bar.set_y_axis_name("Amount ($)");
    bar.set_legend_position(LegendPosition::None);
    let bs = bar.add_series();
    bs.set_values(&format!(
        "Dashboard!$D${}:$D${}",
        status_start + 1,
        status_start + statuses.len() as u32
    ));
    bs.set_categories(&format!(
        "Dashboard!$B${}:$B${}",
        status_start + 1,
        status_start + statuses.len() as u32
    ));
    bs.set_name("Amount");
    bs.set_data_labels(true);
    bs.set_color(navy);
    ws2.insert_chart(chart_row, 5, &bar)?;

    ws2.set_landscape();
    ws2.set_fit_to_page(1, 1);

    // ── Save ──
    wb.set_active_sheet(1);
    let path = std::path::PathBuf::from("output/overdue_tracker.xlsx");
    wb.save(&path)?;
    println!("✅ Overdue Tracker saved to {}", path.display());
    Ok(())
}
