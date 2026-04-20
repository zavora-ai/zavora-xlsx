use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A"; let green = "#0D7C3D"; let _red = "#C00000"; let blue = "#2B579A"; let _amber = "#E67E22"; let border = "#D6DCE4"; let light = "#F5F7FA";

    // Sheet 1: Review Data
    {
        let ws = wb.worksheet(0)?;
        ws.set_name("Reviews")?;
        ws.hide_gridlines();

        let cw = [2.0, 18.0, 14.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 14.0];
        for (c, w) in cw.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }

        let title = Format::new().bold().font_size(20.0).font_color(navy).align(Align::Left).align(Align::Bottom);
        let divider = Format::new().background_color(blue);
        let hdr = Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Center).border(BorderStyle::Thin);
        let hdr_l = Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Left).border(BorderStyle::Thin);
        let tf = Format::new().font_size(10.0).font_color(navy).align(Align::Left).border(BorderStyle::Thin).border_color(border);
        let nf = Format::new().font_size(10.0).font_color(navy).align(Align::Center).border(BorderStyle::Thin).border_color(border);
        let sf = Format::new().font_size(10.0).font_color(navy).align(Align::Center).num_format("0.0").border(BorderStyle::Thin).border_color(border);

        let mut r = 0u32;
        ws.set_row_height(r, 6.0)?; r += 1;
        ws.write_with_format(r, 1, "⭐ Performance Reviews", &title)?; ws.set_row_height(r, 32.0)?; r += 1;
        ws.write_with_format(r, 1, "Annual Review Cycle — Q1 2025", &Format::new().font_size(10.0).font_color("#667085").italic())?; r += 1;
        for c in 1..=9u16 { ws.write_with_format(r, c, "", &divider)?; } ws.set_row_height(r, 3.0)?; r += 2;

        let headers = ["Employee","Department","Technical","Communication","Leadership","Teamwork","Overall","Rating"];
        ws.write_with_format(r, 1, headers[0], &hdr_l)?; ws.write_with_format(r, 2, headers[1], &hdr_l)?;
        for (c, h) in headers[2..].iter().enumerate() { ws.write_with_format(r, (c+3) as u16, *h, &hdr)?; }
        ws.set_row_height(r, 22.0)?; r += 1;

        let reviews: Vec<(&str, &str, f64, f64, f64, f64)> = vec![
            ("Sarah Kimani", "Finance", 4.5, 4.8, 4.7, 4.6),
            ("James Mwangi", "Engineering", 4.8, 4.2, 4.5, 4.3),
            ("Amina Wanjiku", "Marketing", 4.2, 4.9, 4.0, 4.5),
            ("David Ochieng", "Engineering", 4.6, 3.8, 3.5, 4.2),
            ("Grace Njeri", "HR", 4.0, 4.7, 4.3, 4.8),
            ("Peter Lumumba", "Engineering", 4.3, 3.5, 3.0, 3.8),
            ("Faith Akinyi", "Finance", 4.1, 4.0, 3.8, 4.1),
            ("Brian Kipchoge", "Sales", 3.8, 4.6, 4.2, 4.4),
            ("Lucy Wambui", "Marketing", 3.9, 4.3, 3.5, 4.0),
            ("Kevin Otieno", "Engineering", 3.5, 3.2, 2.8, 3.5),
            ("Mary Auma", "HR", 3.8, 4.1, 3.2, 4.3),
            ("John Kamau", "Sales", 3.6, 4.4, 3.0, 3.9),
        ];

        let data_start = r;
        for (i, (name, dept, tech, comm, lead, team)) in reviews.iter().enumerate() {
            let alt = i % 2 == 1;
            let t = if alt { &Format::new().font_size(10.0).font_color(navy).align(Align::Left).background_color(light).border(BorderStyle::Thin).border_color(border) } else { &tf };
            ws.write_with_format(r, 1, *name, t)?;
            ws.write_with_format(r, 2, *dept, t)?;
            ws.write_with_format(r, 3, *tech, &sf)?;
            ws.write_with_format(r, 4, *comm, &sf)?;
            ws.write_with_format(r, 5, *lead, &sf)?;
            ws.write_with_format(r, 6, *team, &sf)?;
            // Overall = AVERAGE
            ws.write_formula(r, 7, &format!("AVERAGE(D{}:G{})", r+1, r+1))?;
            ws.set_cell_format(r, 7, &Format::new().bold().font_size(10.0).font_color(navy).align(Align::Center).num_format("0.0").border(BorderStyle::Thin).border_color(border))?;
            // Rating text
            ws.write_formula(r, 8, &format!("IF(H{r1}>=4.5,\"⭐ Exceptional\",IF(H{r1}>=4,\"✅ Exceeds\",IF(H{r1}>=3.5,\"👍 Meets\",\"⚠ Needs Improvement\")))", r1=r+1))?;
            ws.set_cell_format(r, 8, &nf)?;
            r += 1;
        }
        let data_end = r - 1;

        ws.add_conditional_format(data_start, 7, data_end, 7, ConditionalFormatDataBar::new(green))?;
        ws.set_autofilter(5, 1, data_end, 8);
        ws.set_freeze_panes(6, 0)?;
        ws.set_landscape(); ws.set_fit_to_page(1, 1);
    }

    // Sheet 2: Summary
    let ws2 = wb.add_worksheet_with_name("Summary")?;
    ws2.hide_gridlines();
    let cw2 = [2.0, 18.0, 14.0, 14.0, 14.0, 14.0, 14.0];
    for (c, w) in cw2.iter().enumerate() { ws2.set_column_width(c as u16, *w)?; }

    let title2 = Format::new().bold().font_size(18.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let sec = Format::new().bold().font_size(12.0).font_color("#FFFFFF").background_color(navy).align(Align::Left).align(Align::VerticalCenter);
    let hdr2 = Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let hdr2_l = Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Left).border(BorderStyle::Thin);
    let val2 = Format::new().font_size(10.0).font_color(navy).align(Align::Center).num_format("0.0").border(BorderStyle::Thin).border_color(border);
    let lbl2 = Format::new().bold().font_size(10.0).font_color(navy).align(Align::Left).border(BorderStyle::Thin).border_color(border);

    let mut r2 = 1u32;
    ws2.write_with_format(r2, 1, "Performance Summary", &title2)?; r2 += 3;

    ws2.merge_range(r2, 1, r2, 6, "  Department Averages", &sec)?; ws2.set_row_height(r2, 26.0)?; r2 += 1;
    let dh = ["Department", "Technical", "Communication", "Leadership", "Teamwork", "Overall"];
    ws2.write_with_format(r2, 1, dh[0], &hdr2_l)?;
    for (c, h) in dh[1..].iter().enumerate() { ws2.write_with_format(r2, (c+2) as u16, *h, &hdr2)?; }
    r2 += 1;

    let depts = ["Engineering", "Finance", "HR", "Marketing", "Sales"];
    let dept_start = r2;
    for dept in &depts {
        ws2.write_with_format(r2, 1, *dept, &lbl2)?;
        for c in 0..4u16 {
            let col_letter = ["D","E","F","G"][c as usize];
            ws2.write_formula(r2, c + 2, &format!("AVERAGEIF(Reviews!C7:C18,B{},Reviews!{}7:{}18)", r2+1, col_letter, col_letter))?;
            ws2.set_cell_format(r2, c + 2, &val2)?;
        }
        ws2.write_formula(r2, 6, &format!("AVERAGE(C{}:F{})", r2+1, r2+1))?;
        ws2.set_cell_format(r2, 6, &Format::new().bold().font_size(10.0).font_color(navy).align(Align::Center).num_format("0.0").border(BorderStyle::Thin).border_color(border))?;
        r2 += 1;
    }
    ws2.add_conditional_format(dept_start, 6, r2-1, 6, ConditionalFormatDataBar::new(green))?;

    r2 += 1;
    let mut chart = Chart::new(ChartType::Bar);
    chart.set_title("Department Performance"); chart.set_width(520); chart.set_height(280);
    chart.set_legend_position(LegendPosition::None);
    let s = chart.add_series();
    s.set_values(&format!("Summary!$G${}:$G${}", dept_start+1, dept_start+depts.len() as u32));
    s.set_categories(&format!("Summary!$B${}:$B${}", dept_start+1, dept_start+depts.len() as u32));
    s.set_name("Overall Score"); s.set_data_labels(true); s.set_color(blue);
    ws2.insert_chart(r2, 1, &chart)?;

    ws2.set_landscape(); ws2.set_fit_to_page(1, 1);

    wb.set_active_sheet(0);
    let path = home("performance_reviews.xlsx");
    wb.save(&path)?;
    println!("✅ Performance Reviews saved to {}", path.display());
    Ok(())
}

fn home(n: &str) -> std::path::PathBuf { std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(n) }
