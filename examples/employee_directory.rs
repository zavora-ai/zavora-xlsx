use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A";
    let green = "#0D7C3D";
    let _red = "#C00000";
    let blue = "#2B579A";
    let border = "#D6DCE4";
    let light = "#F5F7FA";

    let ws = wb.worksheet(0)?;
    ws.set_name("Employee Directory")?;
    ws.hide_gridlines();

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
    let tf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Thin)
        .border_color(border);
    let tf_a = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .background_color(light)
        .border(BorderStyle::Thin)
        .border_color(border);
    let cf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin)
        .border_color(border);
    let mf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .num_format("$#,##0")
        .border(BorderStyle::Thin)
        .border_color(border);
    let df = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("yyyy-mm-dd")
        .border(BorderStyle::Thin)
        .border_color(border);
    let title = Format::new()
        .bold()
        .font_size(20.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::Bottom);
    let divider = Format::new().background_color(blue);

    let headers = [
        "Emp ID",
        "Name",
        "Department",
        "Title",
        "Start Date",
        "Salary",
        "Status",
        "Email",
        "Phone",
        "Manager",
    ];
    let widths = [
        2.0, 10.0, 20.0, 16.0, 22.0, 14.0, 12.0, 10.0, 26.0, 16.0, 18.0,
    ];
    for (c, w) in widths.iter().enumerate() {
        ws.set_column_width(c as u16, *w)?;
    }

    let mut r = 0u32;
    ws.set_row_height(r, 6.0)?;
    r += 1;
    ws.write_with_format(r, 1, "👥 Employee Directory", &title)?;
    ws.set_row_height(r, 32.0)?;
    r += 1;
    ws.write_with_format(
        r,
        1,
        "HR Department  |  Confidential",
        &Format::new().font_size(10.0).font_color("#667085").italic(),
    )?;
    r += 1;
    for c in 1..=10u16 {
        ws.write_with_format(r, c, "", &divider)?;
    }
    ws.set_row_height(r, 3.0)?;
    r += 2;

    ws.write_with_format(r, 1, headers[0], &hdr)?;
    for (c, h) in headers[1..].iter().enumerate() {
        let f = if c == 0 { &hdr_l } else { &hdr };
        ws.write_with_format(r, (c + 2) as u16, *h, f)?;
    }
    ws.set_row_height(r, 22.0)?;
    r += 1;

    let employees: Vec<(
        &str,
        &str,
        &str,
        &str,
        (i32, u32, u32),
        f64,
        &str,
        &str,
        &str,
        &str,
    )> = vec![
        (
            "E001",
            "Sarah Kimani",
            "Finance",
            "CFO",
            (2018, 3, 15),
            185000.0,
            "Active",
            "s.kimani@co.ke",
            "+254 700 111 001",
            "CEO",
        ),
        (
            "E002",
            "James Mwangi",
            "Engineering",
            "VP Engineering",
            (2019, 6, 1),
            165000.0,
            "Active",
            "j.mwangi@co.ke",
            "+254 700 111 002",
            "CEO",
        ),
        (
            "E003",
            "Amina Wanjiku",
            "Marketing",
            "Marketing Director",
            (2020, 1, 10),
            140000.0,
            "Active",
            "a.wanjiku@co.ke",
            "+254 700 111 003",
            "CEO",
        ),
        (
            "E004",
            "David Ochieng",
            "Engineering",
            "Senior Developer",
            (2020, 4, 20),
            120000.0,
            "Active",
            "d.ochieng@co.ke",
            "+254 700 111 004",
            "James Mwangi",
        ),
        (
            "E005",
            "Grace Njeri",
            "HR",
            "HR Manager",
            (2019, 9, 5),
            110000.0,
            "Active",
            "g.njeri@co.ke",
            "+254 700 111 005",
            "CEO",
        ),
        (
            "E006",
            "Peter Lumumba",
            "Engineering",
            "Developer",
            (2021, 2, 14),
            95000.0,
            "Active",
            "p.lumumba@co.ke",
            "+254 700 111 006",
            "James Mwangi",
        ),
        (
            "E007",
            "Faith Akinyi",
            "Finance",
            "Accountant",
            (2021, 7, 1),
            85000.0,
            "Active",
            "f.akinyi@co.ke",
            "+254 700 111 007",
            "Sarah Kimani",
        ),
        (
            "E008",
            "Brian Kipchoge",
            "Sales",
            "Sales Manager",
            (2020, 11, 15),
            125000.0,
            "Active",
            "b.kipchoge@co.ke",
            "+254 700 111 008",
            "CEO",
        ),
        (
            "E009",
            "Lucy Wambui",
            "Marketing",
            "Content Lead",
            (2022, 3, 1),
            88000.0,
            "Active",
            "l.wambui@co.ke",
            "+254 700 111 009",
            "Amina Wanjiku",
        ),
        (
            "E010",
            "Kevin Otieno",
            "Engineering",
            "Junior Developer",
            (2023, 1, 15),
            72000.0,
            "Active",
            "k.otieno@co.ke",
            "+254 700 111 010",
            "David Ochieng",
        ),
        (
            "E011",
            "Mary Auma",
            "HR",
            "HR Assistant",
            (2023, 6, 1),
            58000.0,
            "Active",
            "m.auma@co.ke",
            "+254 700 111 011",
            "Grace Njeri",
        ),
        (
            "E012",
            "John Kamau",
            "Sales",
            "Sales Rep",
            (2022, 8, 10),
            68000.0,
            "Active",
            "j.kamau@co.ke",
            "+254 700 111 012",
            "Brian Kipchoge",
        ),
        (
            "E013",
            "Rose Chebet",
            "Finance",
            "Finance Analyst",
            (2023, 9, 1),
            78000.0,
            "Active",
            "r.chebet@co.ke",
            "+254 700 111 013",
            "Sarah Kimani",
        ),
        (
            "E014",
            "Tom Nyong'o",
            "Operations",
            "Operations Lead",
            (2021, 5, 20),
            105000.0,
            "Active",
            "t.nyongo@co.ke",
            "+254 700 111 014",
            "CEO",
        ),
        (
            "E015",
            "Alice Muthoni",
            "Engineering",
            "QA Engineer",
            (2022, 11, 1),
            90000.0,
            "On Leave",
            "a.muthoni@co.ke",
            "+254 700 111 015",
            "James Mwangi",
        ),
    ];

    for (i, e) in employees.iter().enumerate() {
        let alt = i % 2 == 1;
        let t = if alt { &tf_a } else { &tf };
        ws.write_with_format(r, 1, e.0, &cf)?;
        ws.write_with_format(r, 2, e.1, t)?;
        ws.write_with_format(r, 3, e.2, t)?;
        ws.write_with_format(r, 4, e.3, t)?;
        ws.write_with_format(
            r,
            5,
            ExcelDateTime::from_ymd(e.4.0, e.4.1, e.4.2).unwrap(),
            &df,
        )?;
        ws.write_with_format(r, 6, e.5, &mf)?;
        let sf = if e.6 == "Active" {
            Format::new()
                .font_size(10.0)
                .font_color(green)
                .bold()
                .align(Align::Center)
                .border(BorderStyle::Thin)
                .border_color(border)
        } else {
            Format::new()
                .font_size(10.0)
                .font_color("#E67E22")
                .bold()
                .align(Align::Center)
                .border(BorderStyle::Thin)
                .border_color(border)
        };
        ws.write_with_format(r, 7, e.6, &sf)?;
        ws.write_with_format(r, 8, e.7, t)?;
        ws.write_with_format(r, 9, e.8, t)?;
        ws.write_with_format(r, 10, e.9, t)?;
        r += 1;
    }
    let last = r - 1;

    // Summary formulas
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
    ws.write_with_format(r, 4, "Total Employees:", &sl)?;
    ws.write_formula(r, 5, &format!("COUNTA(B7:B{})", last + 1))?;
    ws.set_cell_format(r, 5, &sv)?;
    ws.write_with_format(r, 6, "Avg Salary:", &sl)?;
    ws.write_formula(r, 7, &format!("AVERAGE(G7:G{})", last + 1))?;
    ws.set_cell_format(
        r,
        7,
        &Format::new()
            .bold()
            .font_size(12.0)
            .font_color(navy)
            .align(Align::Center)
            .num_format("$#,##0"),
    )?;
    r += 1;
    ws.write_with_format(r, 4, "Active:", &sl)?;
    ws.write_formula(r, 5, &format!("COUNTIF(H7:H{},\"Active\")", last + 1))?;
    ws.set_cell_format(r, 5, &sv)?;
    ws.write_with_format(r, 6, "Total Payroll:", &sl)?;
    ws.write_formula(r, 7, &format!("SUM(G7:G{})", last + 1))?;
    ws.set_cell_format(
        r,
        7,
        &Format::new()
            .bold()
            .font_size(12.0)
            .font_color(green)
            .align(Align::Center)
            .num_format("$#,##0"),
    )?;

    ws.add_conditional_format(6, 6, last, 6, ConditionalFormatDataBar::new(blue))?;
    ws.set_autofilter(5, 1, last, 10);
    ws.set_freeze_panes(6, 0)?;
    ws.set_landscape();
    ws.set_fit_to_page(1, 0);
    ws.set_header("&CEmployee Directory — Confidential");
    ws.set_footer("&C&D  |  Page &P");

    let path = home("employee_directory.xlsx");
    wb.save(&path)?;
    println!("✅ Employee Directory saved to {}", path.display());
    Ok(())
}

fn home(n: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("output").join(n)
}
