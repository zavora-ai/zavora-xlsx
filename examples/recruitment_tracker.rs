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

    let ws = wb.worksheet(0)?;
    ws.set_name("Recruitment")?;
    ws.hide_gridlines();

    let cw = [2.0, 12.0, 20.0, 16.0, 14.0, 12.0, 14.0, 12.0, 14.0, 14.0];
    for (c, w) in cw.iter().enumerate() {
        ws.set_column_width(c as u16, *w)?;
    }

    let title = Format::new()
        .bold()
        .font_size(20.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::Bottom);
    let hdr = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin);
    let hdr_l = Format::new()
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

    let mut r = 0u32;
    ws.set_row_height(r, 6.0)?;
    r += 1;
    ws.write_with_format(r, 1, "🎯 Recruitment Pipeline", &title)?;
    ws.set_row_height(r, 32.0)?;
    r += 1;
    ws.write_with_format(
        r,
        1,
        "Open Positions & Candidate Tracking — Q2 2025",
        &Format::new().font_size(10.0).font_color("#667085").italic(),
    )?;
    r += 1;
    for c in 1..=9u16 {
        ws.write_with_format(r, c, "", &Format::new().background_color(blue))?;
    }
    ws.set_row_height(r, 3.0)?;
    r += 2;

    let headers = [
        "Req ID",
        "Position",
        "Department",
        "Hiring Mgr",
        "Status",
        "Posted",
        "Applicants",
        "Interviews",
        "Target Date",
    ];
    ws.write_with_format(r, 1, headers[0], &hdr)?;
    for (c, h) in headers[1..].iter().enumerate() {
        let f = if c < 3 { &hdr_l } else { &hdr };
        ws.write_with_format(r, (c + 2) as u16, *h, f)?;
    }
    ws.set_row_height(r, 22.0)?;
    r += 1;

    let positions: Vec<(
        &str,
        &str,
        &str,
        &str,
        &str,
        (i32, u32, u32),
        u32,
        u32,
        (i32, u32, u32),
    )> = vec![
        (
            "REQ-101",
            "Senior Developer",
            "Engineering",
            "James M.",
            "Interviewing",
            (2025, 3, 1),
            45,
            8,
            (2025, 5, 15),
        ),
        (
            "REQ-102",
            "Marketing Manager",
            "Marketing",
            "Amina W.",
            "Offer Extended",
            (2025, 2, 15),
            62,
            12,
            (2025, 4, 30),
        ),
        (
            "REQ-103",
            "Financial Analyst",
            "Finance",
            "Sarah K.",
            "Screening",
            (2025, 3, 15),
            38,
            0,
            (2025, 5, 30),
        ),
        (
            "REQ-104",
            "DevOps Engineer",
            "Engineering",
            "James M.",
            "Interviewing",
            (2025, 3, 10),
            52,
            6,
            (2025, 5, 20),
        ),
        (
            "REQ-105",
            "HR Coordinator",
            "HR",
            "Grace N.",
            "Open",
            (2025, 4, 1),
            15,
            0,
            (2025, 6, 15),
        ),
        (
            "REQ-106",
            "Sales Executive",
            "Sales",
            "Brian K.",
            "Interviewing",
            (2025, 2, 20),
            78,
            10,
            (2025, 4, 25),
        ),
        (
            "REQ-107",
            "UX Designer",
            "Engineering",
            "James M.",
            "Screening",
            (2025, 3, 20),
            33,
            0,
            (2025, 6, 1),
        ),
        (
            "REQ-108",
            "Operations Analyst",
            "Operations",
            "Tom N.",
            "Offer Accepted",
            (2025, 1, 15),
            41,
            9,
            (2025, 4, 15),
        ),
        (
            "REQ-109",
            "Content Writer",
            "Marketing",
            "Amina W.",
            "Open",
            (2025, 4, 5),
            8,
            0,
            (2025, 6, 30),
        ),
        (
            "REQ-110",
            "Junior Developer",
            "Engineering",
            "David O.",
            "Interviewing",
            (2025, 3, 25),
            67,
            15,
            (2025, 5, 10),
        ),
    ];

    let data_start = r;
    for (i, p) in positions.iter().enumerate() {
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

        ws.write_with_format(r, 1, p.0, c)?;
        ws.write_with_format(r, 2, p.1, t)?;
        ws.write_with_format(r, 3, p.2, t)?;
        ws.write_with_format(r, 4, p.3, t)?;

        let (status_color, status_bg) = match p.4 {
            "Offer Accepted" => (green, "#E8F5E9"),
            "Offer Extended" => (blue, "#E3F2FD"),
            "Interviewing" => (amber, "#FFF8E1"),
            "Screening" => (navy, light),
            _ => ("#667085", light),
        };
        ws.write_with_format(
            r,
            5,
            p.4,
            &Format::new()
                .font_size(10.0)
                .font_color(status_color)
                .bold()
                .align(Align::Center)
                .background_color(status_bg)
                .border(BorderStyle::Thin)
                .border_color(border),
        )?;

        ws.write_with_format(
            r,
            6,
            ExcelDateTime::from_ymd(p.5.0, p.5.1, p.5.2).unwrap(),
            &df,
        )?;
        ws.write_with_format(r, 7, p.6 as f64, c)?;
        ws.write_with_format(r, 8, p.7 as f64, c)?;
        ws.write_with_format(
            r,
            9,
            ExcelDateTime::from_ymd(p.8.0, p.8.1, p.8.2).unwrap(),
            &df,
        )?;
        r += 1;
    }
    let data_end = r - 1;

    // Summary
    r += 1;
    let sl = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right);
    let sv = Format::new()
        .bold()
        .font_size(12.0)
        .font_color(navy)
        .align(Align::Center);
    ws.write_with_format(r, 5, "Open Positions:", &sl)?;
    ws.write_formula(
        r,
        6,
        &format!("COUNTA(B{}:B{})", data_start + 1, data_end + 1),
    )?;
    ws.set_cell_format(r, 6, &sv)?;
    ws.write_with_format(r, 7, "Total Applicants:", &sl)?;
    ws.write_formula(r, 8, &format!("SUM(H{}:H{})", data_start + 1, data_end + 1))?;
    ws.set_cell_format(r, 8, &sv)?;
    r += 1;
    ws.write_with_format(r, 5, "In Interview:", &sl)?;
    ws.write_formula(
        r,
        6,
        &format!(
            "COUNTIF(F{}:F{},\"Interviewing\")",
            data_start + 1,
            data_end + 1
        ),
    )?;
    ws.set_cell_format(r, 6, &sv)?;
    ws.write_with_format(r, 7, "Avg Applicants:", &sl)?;
    ws.write_formula(
        r,
        8,
        &format!("AVERAGE(H{}:H{})", data_start + 1, data_end + 1),
    )?;
    ws.set_cell_format(
        r,
        8,
        &Format::new()
            .bold()
            .font_size(12.0)
            .font_color(navy)
            .align(Align::Center)
            .num_format("0"),
    )?;

    ws.add_conditional_format(
        data_start,
        7,
        data_end,
        7,
        ConditionalFormatDataBar::new(blue),
    )?;
    ws.set_autofilter(5, 1, data_end, 9);
    ws.set_freeze_panes(6, 0)?;
    ws.set_landscape();
    ws.set_fit_to_page(1, 1);
    ws.set_header("&CRecruitment Pipeline — Q2 2025");
    ws.set_footer("&CHR Department  |  &D");

    let path = home("recruitment_tracker.xlsx");
    wb.save(&path)?;
    println!("✅ Recruitment Tracker saved to {}", path.display());
    Ok(())
}

fn home(n: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("output").join(n)
}
