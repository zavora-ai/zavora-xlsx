use zavora_xlsx::*;
fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A";
    let green = "#0D7C3D";
    let red = "#C00000";
    let blue = "#2B579A";
    let amber = "#E67E22";
    let border = "#D6DCE4";
    let light = "#F5F7FA";
    let ws = wb.worksheet(0)?;
    ws.set_name("Maintenance")?;
    ws.hide_gridlines();
    let cw = [2.0, 12.0, 20.0, 14.0, 12.0, 12.0, 14.0, 12.0, 12.0, 12.0];
    for (c, w) in cw.iter().enumerate() {
        ws.set_column_width(c as u16, *w)?;
    }
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
    let tf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Thin)
        .border_color(border);
    let cf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin)
        .border_color(border);
    let df = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("yyyy-mm-dd")
        .border(BorderStyle::Thin)
        .border_color(border);
    let mf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .num_format("$#,##0")
        .border(BorderStyle::Thin)
        .border_color(border);
    let title = Format::new()
        .bold()
        .font_size(20.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::Bottom);
    let mut r = 0u32;
    ws.set_row_height(r, 6.0)?;
    r += 1;
    ws.write_with_format(r, 1, "🔧 Maintenance Log", &title)?;
    ws.set_row_height(r, 32.0)?;
    r += 1;
    ws.write_with_format(
        r,
        1,
        "Equipment maintenance tracking — 2025",
        &Format::new().font_size(10.0).font_color("#667085").italic(),
    )?;
    r += 1;
    for c in 1..=9u16 {
        ws.write_with_format(r, c, "", &Format::new().background_color(amber))?;
    }
    ws.set_row_height(r, 3.0)?;
    r += 2;
    let h = [
        "Equip ID",
        "Equipment",
        "Type",
        "Date",
        "Cost",
        "Technician",
        "Next Due",
        "Status",
        "Days Until Due",
    ];
    ws.write_with_format(r, 1, h[0], &hdr)?;
    ws.write_with_format(r, 2, h[1], &hl)?;
    for (c, hh) in h[2..].iter().enumerate() {
        ws.write_with_format(r, (c + 3) as u16, *hh, &hdr)?;
    }
    ws.set_row_height(r, 22.0)?;
    r += 1;
    type MaintenanceEntry<'a> = (
        &'a str,
        &'a str,
        &'a str,
        (i32, u32, u32),
        f64,
        &'a str,
        (i32, u32, u32),
    );
    let entries: Vec<MaintenanceEntry<'_>> = vec![
        (
            "EQ-001",
            "CNC Machine #1",
            "Preventive",
            (2025, 3, 15),
            2500.0,
            "John K.",
            (2025, 6, 15),
        ),
        (
            "EQ-002",
            "Forklift A",
            "Preventive",
            (2025, 3, 20),
            800.0,
            "Peter M.",
            (2025, 4, 20),
        ),
        (
            "EQ-003",
            "HVAC System",
            "Corrective",
            (2025, 3, 22),
            3200.0,
            "David O.",
            (2025, 9, 22),
        ),
        (
            "EQ-004",
            "Generator",
            "Preventive",
            (2025, 3, 25),
            1500.0,
            "John K.",
            (2025, 6, 25),
        ),
        (
            "EQ-005",
            "Conveyor Belt",
            "Emergency",
            (2025, 3, 28),
            4500.0,
            "Peter M.",
            (2025, 4, 28),
        ),
        (
            "EQ-006",
            "Compressor",
            "Preventive",
            (2025, 4, 1),
            600.0,
            "Grace N.",
            (2025, 7, 1),
        ),
        (
            "EQ-007",
            "CNC Machine #2",
            "Corrective",
            (2025, 4, 2),
            1800.0,
            "John K.",
            (2025, 7, 2),
        ),
        (
            "EQ-008",
            "Forklift B",
            "Preventive",
            (2025, 4, 3),
            800.0,
            "Peter M.",
            (2025, 5, 3),
        ),
        (
            "EQ-009",
            "Fire Suppression",
            "Preventive",
            (2025, 4, 4),
            950.0,
            "David O.",
            (2025, 10, 4),
        ),
        (
            "EQ-010",
            "Elevator",
            "Emergency",
            (2025, 4, 5),
            6200.0,
            "Grace N.",
            (2025, 4, 12),
        ),
        (
            "EQ-011",
            "Boiler",
            "Preventive",
            (2025, 3, 10),
            1200.0,
            "John K.",
            (2025, 4, 10),
        ),
        (
            "EQ-012",
            "Packaging Machine",
            "Corrective",
            (2025, 4, 1),
            2100.0,
            "Peter M.",
            (2025, 7, 1),
        ),
    ];
    let ds = r;
    for (i, e) in entries.iter().enumerate() {
        let alt = i % 2 == 1;
        let t = if alt {
            &Format::new()
                .font_size(10.0)
                .font_color(navy)
                .align(Align::Left)
                .background_color(light)
                .border(BorderStyle::Thin)
                .border_color(border)
        } else {
            &tf
        };
        let c = if alt {
            &Format::new()
                .font_size(10.0)
                .font_color(navy)
                .align(Align::Center)
                .background_color(light)
                .border(BorderStyle::Thin)
                .border_color(border)
        } else {
            &cf
        };
        let (tc, tb) = match e.2 {
            "Preventive" => (green, "#E8F5E9"),
            "Corrective" => (amber, "#FFF8E1"),
            _ => (red, "#FDE8E8"),
        };
        ws.write_with_format(r, 1, e.0, c)?;
        ws.write_with_format(r, 2, e.1, t)?;
        ws.write_with_format(
            r,
            3,
            e.2,
            &Format::new()
                .font_size(10.0)
                .font_color(tc)
                .bold()
                .align(Align::Center)
                .background_color(tb)
                .border(BorderStyle::Thin)
                .border_color(border),
        )?;
        ws.write_with_format(
            r,
            4,
            ExcelDateTime::from_ymd(e.3.0, e.3.1, e.3.2).unwrap(),
            &df,
        )?;
        ws.write_with_format(r, 5, e.4, &mf)?;
        ws.write_with_format(r, 6, e.5, t)?;
        ws.write_with_format(
            r,
            7,
            ExcelDateTime::from_ymd(e.6.0, e.6.1, e.6.2).unwrap(),
            &df,
        )?;
        ws.write_formula(
            r,
            8,
            &format!(
                "IF(H{r1}<TODAY(),\"🔴 Overdue\",IF(H{r1}-TODAY()<=14,\"🟡 Soon\",\"🟢 OK\"))",
                r1 = r + 1
            ),
        )?;
        ws.set_cell_format(r, 8, c)?;
        ws.write_formula(r, 9, &format!("MAX(0,H{}-TODAY())", r + 1))?;
        ws.set_cell_format(r, 9, c)?;
        r += 1;
    }
    let de = r - 1;
    r += 1;
    let sl = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right);
    ws.write_with_format(r, 4, "Total Cost:", &sl)?;
    ws.write_formula(r, 5, &format!("SUM(F{}:F{})", ds + 1, de + 1))?;
    ws.set_cell_format(
        r,
        5,
        &Format::new()
            .bold()
            .font_size(12.0)
            .font_color(green)
            .align(Align::Center)
            .num_format("$#,##0"),
    )?;
    ws.add_conditional_format(ds, 9, de, 9, ConditionalFormatDataBar::new(blue))?;
    ws.set_autofilter(5, 1, de, 9);
    ws.set_freeze_panes(6, 0)?;
    ws.set_landscape();
    ws.set_fit_to_page(1, 0);
    ws.set_header("&CMaintenance Log");
    wb.save(home("maintenance_log.xlsx"))?;
    println!("✅ Maintenance Log saved");
    Ok(())
}
fn home(n: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("output").join(n)
}
