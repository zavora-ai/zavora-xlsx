use zavora_xlsx::*;
fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A";
    let green = "#0D7C3D";
    let red = "#C00000";
    let blue = "#2B579A";
    let border = "#D6DCE4";
    let _light = "#F5F7FA";
    // Sheet 1: Zone Data
    {
        let ws = wb.worksheet(0)?;
        ws.set_name("Zones")?;
        let hdr = Format::new()
            .bold()
            .font_size(10.0)
            .font_color("#FFFFFF")
            .background_color(navy)
            .align(Align::Center)
            .border(BorderStyle::Thin);
        let hl = Format::new()
            .bold()
            .font_size(10.0)
            .font_color("#FFFFFF")
            .background_color(navy)
            .align(Align::Left)
            .border(BorderStyle::Thin);
        let nf = Format::new()
            .font_size(10.0)
            .font_color(navy)
            .align(Align::Center)
            .border(BorderStyle::Thin)
            .border_color(border);
        let tf = Format::new()
            .font_size(10.0)
            .font_color(navy)
            .align(Align::Left)
            .border(BorderStyle::Thin)
            .border_color(border);
        let h = [
            "Zone",
            "Type",
            "Capacity",
            "Utilized",
            "Items",
            "Picks/Day",
            "Errors/Day",
        ];
        ws.write_with_format(0, 0, h[0], &hl)?;
        ws.write_with_format(0, 1, h[1], &hl)?;
        for (c, hh) in h[2..].iter().enumerate() {
            ws.write_with_format(0, (c + 2) as u16, *hh, &hdr)?;
        }
        let zones: Vec<(&str, &str, f64, f64, f64, f64, f64)> = vec![
            (
                "Zone A - Receiving",
                "Inbound",
                5000.0,
                4200.0,
                3800.0,
                450.0,
                12.0,
            ),
            (
                "Zone B - Bulk Storage",
                "Storage",
                15000.0,
                12500.0,
                8200.0,
                280.0,
                8.0,
            ),
            (
                "Zone C - Pick & Pack",
                "Fulfillment",
                8000.0,
                7100.0,
                12500.0,
                820.0,
                25.0,
            ),
            (
                "Zone D - Cold Storage",
                "Refrigerated",
                3000.0,
                2400.0,
                1500.0,
                150.0,
                5.0,
            ),
            (
                "Zone E - Hazmat",
                "Specialized",
                2000.0,
                800.0,
                400.0,
                45.0,
                2.0,
            ),
            (
                "Zone F - Returns",
                "Processing",
                4000.0,
                3500.0,
                2800.0,
                320.0,
                18.0,
            ),
            (
                "Zone G - Shipping",
                "Outbound",
                6000.0,
                5200.0,
                4500.0,
                680.0,
                15.0,
            ),
            (
                "Zone H - Overflow",
                "Temporary",
                5000.0,
                1800.0,
                900.0,
                120.0,
                4.0,
            ),
        ];
        for (i, z) in zones.iter().enumerate() {
            let r = (i + 1) as u32;
            ws.write_with_format(r, 0, z.0, &tf)?;
            ws.write_with_format(r, 1, z.1, &tf)?;
            ws.write_with_format(r, 2, z.2, &nf)?;
            ws.write_with_format(r, 3, z.3, &nf)?;
            ws.write_with_format(r, 4, z.4, &nf)?;
            ws.write_with_format(r, 5, z.5, &nf)?;
            ws.write_with_format(r, 6, z.6, &nf)?;
        }
        let widths = [22.0, 14.0, 12.0, 12.0, 12.0, 12.0, 12.0];
        for (c, w) in widths.iter().enumerate() {
            ws.set_column_width(c as u16, *w)?;
        }
        ws.set_freeze_panes(1, 0)?;
    }
    let n = 8u32;
    // Sheet 2: Dashboard
    let ws2 = wb.add_worksheet_with_name("Dashboard")?;
    ws2.hide_gridlines();
    let cw = [2.0, 18.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 2.0];
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
    let kpi_l = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#667085")
        .align(Align::Center);
    let kpi_v = Format::new()
        .bold()
        .font_size(22.0)
        .font_color(navy)
        .align(Align::Center);
    let kpi_p = Format::new()
        .bold()
        .font_size(22.0)
        .font_color(green)
        .align(Align::Center)
        .num_format("0%");
    let kpi_r = Format::new()
        .bold()
        .font_size(18.0)
        .font_color(red)
        .align(Align::Center)
        .num_format("0.00%");
    let sec = Format::new()
        .bold()
        .font_size(12.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Left)
        .align(Align::VerticalCenter);
    let hdr = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin);
    let hl = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Thin);
    let zf = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Thin)
        .border_color(border);
    let nf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin)
        .border_color(border);
    let pf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("0%")
        .border(BorderStyle::Thin)
        .border_color(border);
    let ef = Format::new()
        .font_size(10.0)
        .font_color(red)
        .bold()
        .align(Align::Center)
        .num_format("0.00%")
        .border(BorderStyle::Thin)
        .border_color(border);
    let mut r = 0u32;
    ws2.set_row_height(r, 6.0)?;
    r += 1;
    ws2.write_with_format(r, 1, "🏭 Warehouse Dashboard", &title)?;
    ws2.set_row_height(r, 32.0)?;
    r += 1;
    ws2.write_with_format(
        r,
        1,
        "Real-time KPIs — All formulas reference Zones sheet",
        &Format::new().font_size(10.0).font_color("#667085").italic(),
    )?;
    r += 1;
    for c in 1..=7u16 {
        ws2.write_with_format(r, c, "", &divider)?;
    }
    ws2.set_row_height(r, 3.0)?;
    r += 2;
    // KPIs
    let labels = [
        "Total Capacity",
        "Avg Utilization",
        "Total Items",
        "Total Picks/Day",
        "Error Rate",
    ];
    for (c, l) in labels.iter().enumerate() {
        ws2.write_with_format(r, (c + 1) as u16, *l, &kpi_l)?;
    }
    r += 1;
    ws2.write_formula(r, 1, &format!("SUM(Zones!C2:C{})", n + 1))?;
    ws2.set_cell_format(r, 1, &kpi_v)?;
    ws2.write_formula(
        r,
        2,
        &format!("AVERAGE(Zones!D2:D{}/Zones!C2:C{})", n + 1, n + 1),
    )?;
    ws2.set_cell_format(r, 2, &kpi_p)?;
    ws2.write_formula(r, 3, &format!("SUM(Zones!E2:E{})", n + 1))?;
    ws2.set_cell_format(r, 3, &kpi_v)?;
    ws2.write_formula(r, 4, &format!("SUM(Zones!F2:F{})", n + 1))?;
    ws2.set_cell_format(r, 4, &kpi_v)?;
    ws2.write_formula(
        r,
        5,
        &format!(
            "IFERROR(SUM(Zones!G2:G{})/SUM(Zones!F2:F{}),0)",
            n + 1,
            n + 1
        ),
    )?;
    ws2.set_cell_format(r, 5, &kpi_r)?;
    ws2.set_row_height(r, 34.0)?;
    r += 2;
    // Zone table
    ws2.merge_range(r, 1, r, 7, "  Zone Performance", &sec)?;
    ws2.set_row_height(r, 26.0)?;
    r += 1;
    let zh = [
        "Zone",
        "Capacity",
        "Utilized",
        "Utilization %",
        "Picks/Day",
        "Errors",
        "Error Rate",
    ];
    ws2.write_with_format(r, 1, zh[0], &hl)?;
    for (c, h) in zh[1..].iter().enumerate() {
        ws2.write_with_format(r, (c + 2) as u16, *h, &hdr)?;
    }
    ws2.set_row_height(r, 20.0)?;
    r += 1;
    let zs = r;
    for i in 0..n {
        let dr = i + 2;
        ws2.write_formula(r, 1, &format!("Zones!A{dr}"))?;
        ws2.set_cell_format(r, 1, &zf)?;
        ws2.write_formula(r, 2, &format!("Zones!C{dr}"))?;
        ws2.set_cell_format(r, 2, &nf)?;
        ws2.write_formula(r, 3, &format!("Zones!D{dr}"))?;
        ws2.set_cell_format(r, 3, &nf)?;
        ws2.write_formula(r, 4, &format!("IFERROR(Zones!D{dr}/Zones!C{dr},0)"))?;
        ws2.set_cell_format(r, 4, &pf)?;
        ws2.write_formula(r, 5, &format!("Zones!F{dr}"))?;
        ws2.set_cell_format(r, 5, &nf)?;
        ws2.write_formula(r, 6, &format!("Zones!G{dr}"))?;
        ws2.set_cell_format(r, 6, &nf)?;
        ws2.write_formula(r, 7, &format!("IFERROR(Zones!G{dr}/Zones!F{dr},0)"))?;
        ws2.set_cell_format(r, 7, &ef)?;
        r += 1;
    }
    ws2.add_conditional_format(zs, 4, r - 1, 4, ConditionalFormatDataBar::new(blue))?;
    ws2.add_conditional_format(zs, 7, r - 1, 7, ConditionalFormatDataBar::new(red))?;
    r += 1;
    // Charts
    let mut donut = Chart::new(ChartType::Doughnut);
    donut.set_title("Capacity by Zone");
    donut.set_width(400);
    donut.set_height(260);
    let ds = donut.add_series();
    ds.set_values(&format!("Dashboard!$C${}:$C${}", zs + 1, zs + n));
    ds.set_categories(&format!("Dashboard!$B${}:$B${}", zs + 1, zs + n));
    ds.set_data_labels(true);
    ws2.insert_chart(r, 1, &donut)?;
    let mut bar = Chart::new(ChartType::Column);
    bar.set_title("Picks/Day by Zone");
    bar.set_width(400);
    bar.set_height(260);
    bar.set_legend_position(LegendPosition::None);
    let bs = bar.add_series();
    bs.set_values(&format!("Dashboard!$F${}:$F${}", zs + 1, zs + n));
    bs.set_categories(&format!("Dashboard!$B${}:$B${}", zs + 1, zs + n));
    bs.set_data_labels(true);
    bs.set_color(blue);
    ws2.insert_chart(r, 5, &bar)?;
    ws2.set_landscape();
    ws2.set_fit_to_page(1, 1);
    ws2.set_header("&CWarehouse Dashboard");
    wb.set_active_sheet(1);
    wb.save(home("warehouse_dashboard.xlsx"))?;
    println!("✅ Warehouse Dashboard saved");
    Ok(())
}
fn home(n: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("output").join(n)
}
