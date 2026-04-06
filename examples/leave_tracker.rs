use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A"; let green = "#0D7C3D"; let red = "#C00000"; let blue = "#2B579A"; let border = "#D6DCE4"; let light = "#F5F7FA";

    let ws = wb.worksheet(0)?;
    ws.set_name("Leave Tracker")?;
    ws.hide_gridlines();

    let cw = [2.0, 18.0, 14.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0];
    for (c, w) in cw.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }

    let title = Format::new().bold().font_size(20.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let divider = Format::new().background_color(green);
    let hdr = Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let hdr_l = Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Left).border(BorderStyle::Thin);
    let tf = Format::new().font_size(10.0).font_color(navy).align(Align::Left).border(BorderStyle::Thin).border_color(border);
    let nf = Format::new().font_size(10.0).font_color(navy).align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let gf = Format::new().font_size(10.0).font_color(green).bold().align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let rf = Format::new().font_size(10.0).font_color(red).bold().align(Align::Center).border(BorderStyle::Thin).border_color(border);

    let mut r = 0u32;
    ws.set_row_height(r, 6.0)?; r += 1;
    ws.write_with_format(r, 1, "🏖️ Leave Tracker", &title)?; ws.set_row_height(r, 32.0)?; r += 1;
    ws.write_with_format(r, 1, "Annual Leave Management — 2025", &Format::new().font_size(10.0).font_color("#667085").italic())?; r += 1;
    for c in 1..=10u16 { ws.write_with_format(r, c, "", &divider)?; } ws.set_row_height(r, 3.0)?; r += 2;

    let headers = ["Employee","Department","Annual","Sick","Personal","Used","Pending","Remaining","% Used"];
    ws.write_with_format(r, 1, headers[0], &hdr_l)?; ws.write_with_format(r, 2, headers[1], &hdr_l)?;
    for (c, h) in headers[2..].iter().enumerate() { ws.write_with_format(r, (c+3) as u16, *h, &hdr)?; }
    ws.set_row_height(r, 22.0)?; r += 1;

    let staff: Vec<(&str, &str, f64, f64, f64, f64, f64)> = vec![
        ("Sarah Kimani", "Finance", 21.0, 10.0, 5.0, 12.0, 3.0),
        ("James Mwangi", "Engineering", 21.0, 10.0, 5.0, 8.0, 2.0),
        ("Amina Wanjiku", "Marketing", 21.0, 10.0, 5.0, 18.0, 0.0),
        ("David Ochieng", "Engineering", 21.0, 10.0, 5.0, 5.0, 5.0),
        ("Grace Njeri", "HR", 21.0, 10.0, 5.0, 15.0, 1.0),
        ("Peter Lumumba", "Engineering", 21.0, 10.0, 5.0, 3.0, 0.0),
        ("Faith Akinyi", "Finance", 21.0, 10.0, 5.0, 20.0, 2.0),
        ("Brian Kipchoge", "Sales", 21.0, 10.0, 5.0, 10.0, 4.0),
        ("Lucy Wambui", "Marketing", 21.0, 10.0, 5.0, 22.0, 0.0),
        ("Kevin Otieno", "Engineering", 15.0, 10.0, 3.0, 2.0, 1.0),
        ("Mary Auma", "HR", 15.0, 10.0, 3.0, 7.0, 2.0),
        ("John Kamau", "Sales", 21.0, 10.0, 5.0, 14.0, 3.0),
    ];

    let data_start = r;
    for (i, (name, dept, annual, sick, personal, used, pending)) in staff.iter().enumerate() {
        let alt = i % 2 == 1;
        let t = if alt { &Format::new().font_size(10.0).font_color(navy).align(Align::Left).background_color(light).border(BorderStyle::Thin).border_color(border) } else { &tf };
        let n = if alt { &Format::new().font_size(10.0).font_color(navy).align(Align::Center).background_color(light).border(BorderStyle::Thin).border_color(border) } else { &nf };
        ws.write_with_format(r, 1, *name, t)?;
        ws.write_with_format(r, 2, *dept, t)?;
        ws.write_with_format(r, 3, *annual, n)?;
        ws.write_with_format(r, 4, *sick, n)?;
        ws.write_with_format(r, 5, *personal, n)?;
        ws.write_with_format(r, 6, *used, n)?;
        ws.write_with_format(r, 7, *pending, n)?;
        // Remaining = (Annual+Sick+Personal) - Used - Pending
        ws.write_formula(r, 8, &format!("(D{}+E{}+F{})-G{}-H{}", r+1,r+1,r+1,r+1,r+1))?;
        let rem = (annual + sick + personal) - used - pending;
        ws.set_cell_format(r, 8, if rem >= 5.0 { &gf } else { &rf })?;
        // % Used
        ws.write_formula(r, 9, &format!("IFERROR(G{}/(D{}+E{}+F{}),0)", r+1,r+1,r+1,r+1))?;
        ws.set_cell_format(r, 9, &Format::new().font_size(10.0).font_color(navy).align(Align::Center).num_format("0%").border(BorderStyle::Thin).border_color(border))?;
        r += 1;
    }
    let data_end = r - 1;

    ws.add_conditional_format(data_start, 9, data_end, 9, ConditionalFormatDataBar::new(blue))?;

    // Summary
    r += 1;
    let sl = Format::new().bold().font_size(10.0).font_color(navy).align(Align::Right);
    let sv = Format::new().bold().font_size(12.0).font_color(navy).align(Align::Center);
    ws.write_with_format(r, 5, "Total Used:", &sl)?;
    ws.write_formula(r, 6, &format!("SUM(G{}:G{})", data_start+1, data_end+1))?; ws.set_cell_format(r, 6, &sv)?;
    ws.write_with_format(r, 7, "Total Pending:", &sl)?;
    ws.write_formula(r, 8, &format!("SUM(H{}:H{})", data_start+1, data_end+1))?; ws.set_cell_format(r, 8, &sv)?;
    r += 1;
    ws.write_with_format(r, 5, "Avg % Used:", &sl)?;
    ws.write_formula(r, 6, &format!("AVERAGE(J{}:J{})", data_start+1, data_end+1))?;
    ws.set_cell_format(r, 6, &Format::new().bold().font_size(12.0).font_color(navy).align(Align::Center).num_format("0%"))?;

    ws.set_autofilter(5, 1, data_end, 9);
    ws.set_freeze_panes(6, 0)?;
    ws.set_landscape(); ws.set_fit_to_page(1, 1);
    ws.set_header("&CLeave Tracker — 2025"); ws.set_footer("&CHR Department  |  &D");

    let path = home("leave_tracker.xlsx");
    wb.save(&path)?;
    println!("✅ Leave Tracker saved to {}", path.display());
    Ok(())
}

fn home(n: &str) -> std::path::PathBuf { std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(n) }
