use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A"; let green = "#0D7C3D"; let _blue = "#2B579A"; let border = "#D6DCE4"; let light = "#F5F7FA";

    let ws = wb.worksheet(0)?;
    ws.set_name("Payroll")?; ws.hide_gridlines();

    let cw = [2.0, 18.0, 14.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 14.0];
    for (c, w) in cw.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }

    let title = Format::new().bold().font_size(20.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let hdr = Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let hdr_l = Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Left).border(BorderStyle::Thin);
    let tf = Format::new().font_size(10.0).font_color(navy).align(Align::Left).border(BorderStyle::Thin).border_color(border);
    let mf = Format::new().font_size(10.0).font_color(navy).align(Align::Right).num_format("$#,##0").border(BorderStyle::Thin).border_color(border);
    let _pf = Format::new().font_size(10.0).font_color(navy).align(Align::Center).num_format("0%").border(BorderStyle::Thin).border_color(border);
    let stl = Format::new().bold().font_size(10.0).font_color(navy).align(Align::Left).background_color("#E8E8E8").border(BorderStyle::Thin);
    let stv = Format::new().bold().font_size(10.0).font_color(navy).align(Align::Right).num_format("$#,##0").background_color("#E8E8E8").border(BorderStyle::Thin);
    let net_f = Format::new().bold().font_size(10.0).font_color(green).align(Align::Right).num_format("$#,##0").background_color("#E8F5E9").border(BorderStyle::Medium).border_color(green);

    let mut r = 0u32;
    ws.set_row_height(r, 6.0)?; r += 1;
    ws.write_with_format(r, 1, "💰 Payroll Calculator", &title)?; ws.set_row_height(r, 32.0)?; r += 1;
    ws.write_with_format(r, 1, "Monthly Payroll — April 2025  |  All deductions calculated by formula", &Format::new().font_size(10.0).font_color("#667085").italic())?; r += 1;
    for c in 1..=9u16 { ws.write_with_format(r, c, "", &Format::new().background_color(green))?; } ws.set_row_height(r, 3.0)?; r += 2;

    let headers = ["Employee","Department","Gross Pay","Tax (30%)","NSSF","NHIF","Pension (5%)","Deductions","Net Pay"];
    ws.write_with_format(r, 1, headers[0], &hdr_l)?; ws.write_with_format(r, 2, headers[1], &hdr_l)?;
    for (c, h) in headers[2..].iter().enumerate() { ws.write_with_format(r, (c+3) as u16, *h, &hdr)?; }
    ws.set_row_height(r, 22.0)?; r += 1;

    let staff: Vec<(&str, &str, f64)> = vec![
        ("Sarah Kimani","Finance",185000.0), ("James Mwangi","Engineering",165000.0),
        ("Amina Wanjiku","Marketing",140000.0), ("David Ochieng","Engineering",120000.0),
        ("Grace Njeri","HR",110000.0), ("Peter Lumumba","Engineering",95000.0),
        ("Faith Akinyi","Finance",85000.0), ("Brian Kipchoge","Sales",125000.0),
        ("Lucy Wambui","Marketing",88000.0), ("Kevin Otieno","Engineering",72000.0),
        ("Mary Auma","HR",58000.0), ("John Kamau","Sales",68000.0),
    ];

    let data_start = r;
    for (i, (name, dept, gross)) in staff.iter().enumerate() {
        let alt = i % 2 == 1;
        let t = if alt { &Format::new().font_size(10.0).font_color(navy).align(Align::Left).background_color(light).border(BorderStyle::Thin).border_color(border) } else { &tf };
        ws.write_with_format(r, 1, *name, t)?;
        ws.write_with_format(r, 2, *dept, t)?;
        ws.write_with_format(r, 3, *gross / 12.0, &mf)?; // monthly
        // Tax = 30% of gross
        ws.write_formula(r, 4, &format!("D{}*0.30", r+1))?; ws.set_cell_format(r, 4, &mf)?;
        // NSSF = fixed 2160
        ws.write_with_format(r, 5, 2160.0, &mf)?;
        // NHIF = tiered (simplified: 1700 for >100k, 1200 for >50k, 600 otherwise)
        ws.write_formula(r, 6, &format!("IF(D{r1}>12000,1700,IF(D{r1}>5000,1200,600))", r1=r+1))?; ws.set_cell_format(r, 6, &mf)?;
        // Pension = 5%
        ws.write_formula(r, 7, &format!("D{}*0.05", r+1))?; ws.set_cell_format(r, 7, &mf)?;
        // Total Deductions
        ws.write_formula(r, 8, &format!("E{}+F{}+G{}+H{}", r+1,r+1,r+1,r+1))?; ws.set_cell_format(r, 8, &mf)?;
        // Net Pay
        ws.write_formula(r, 9, &format!("D{}-I{}", r+1, r+1))?; ws.set_cell_format(r, 9, &net_f)?;
        r += 1;
    }
    let data_end = r - 1;

    // Totals
    r += 1;
    ws.write_with_format(r, 2, "TOTALS", &stl)?;
    for c in 3..=9u16 {
        ws.write_formula(r, c, &format!("SUM({}{}:{}{})", col(c), data_start+1, col(c), data_end+1))?;
        ws.set_cell_format(r, c, if c == 9 { &net_f } else { &stv })?;
    }

    ws.add_conditional_format(data_start, 9, data_end, 9, ConditionalFormatDataBar::new(green))?;
    ws.set_autofilter(5, 1, data_end, 9);
    ws.set_freeze_panes(6, 0)?;
    ws.set_landscape(); ws.set_fit_to_page(1, 1);
    ws.set_header("&CPayroll — April 2025  |  Confidential"); ws.set_footer("&C&D  |  Page &P");

    let path = home("payroll_calculator.xlsx");
    wb.save(&path)?;
    println!("✅ Payroll Calculator saved to {}", path.display());
    Ok(())
}

fn col(c: u16) -> String { zavora_xlsx::utility::col_to_letter(c) }
fn home(n: &str) -> std::path::PathBuf { std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(n) }
