use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Raw Data (the single source of truth)
    // ═══════════════════════════════════════════════════════════
    let ws = wb.worksheet(0)?;
    ws.set_name("Data")?;

    let headers = [
        "Program",
        "Department",
        "Provider",
        "Month",
        "Day",
        "Duration",
        "Cost",
        "Capacity",
        "Enrolled",
        "Attended",
        "Fill Rate",
        "Rating",
    ];
    let hdr_fmt = Format::new()
        .bold()
        .font_size(10.0)
        .font_color("#FFFFFF")
        .background_color("#C68E17")
        .align(Align::Center)
        .border(BorderStyle::Thin);
    for (c, h) in headers.iter().enumerate() {
        ws.write_with_format(0, c as u16, *h, &hdr_fmt)?;
    }

    let programs: Vec<[&str; 10]> = vec![
        [
            "Advanced People Management",
            "HR",
            "Libero Associates",
            "October",
            "15",
            "1",
            "5000",
            "20",
            "6",
            "3.5",
        ],
        [
            "Advanced IT Techniques",
            "IT",
            "Nulla Eu Inc",
            "November",
            "9",
            "1",
            "5000",
            "30",
            "6",
            "0",
        ],
        [
            "Advanced IT Skills",
            "IT",
            "Vivamus Industries",
            "December",
            "1",
            "1.5",
            "3000",
            "15",
            "6",
            "0",
        ],
        [
            "Advanced Business Techniques",
            "Sales",
            "Eu Dolor Egestas",
            "December",
            "14",
            "1",
            "3000",
            "20",
            "1",
            "0",
        ],
        [
            "Leadership Essentials",
            "Finance",
            "Apex Training",
            "January",
            "20",
            "2",
            "8000",
            "25",
            "22",
            "4.5",
        ],
        [
            "Digital Marketing Masterclass",
            "Marketing",
            "DigiLearn Corp",
            "January",
            "27",
            "1",
            "4500",
            "30",
            "28",
            "4",
        ],
        [
            "Supply Chain Optimization",
            "Operations",
            "LogiTrain Partners",
            "February",
            "10",
            "2",
            "7500",
            "20",
            "18",
            "4.2",
        ],
        [
            "Excel for Finance Pros",
            "Finance",
            "DataSkills Academy",
            "February",
            "24",
            "1",
            "3500",
            "25",
            "25",
            "4.8",
        ],
        [
            "Conflict Resolution Workshop",
            "HR",
            "PeopleSoft Training",
            "March",
            "5",
            "0.5",
            "2500",
            "15",
            "14",
            "3.8",
        ],
        [
            "Cybersecurity Fundamentals",
            "IT",
            "SecureNet Institute",
            "March",
            "17",
            "2",
            "9000",
            "20",
            "20",
            "4.6",
        ],
        [
            "Sales Negotiation Tactics",
            "Sales",
            "CloseRate Academy",
            "March",
            "24",
            "1",
            "5500",
            "25",
            "24",
            "4.3",
        ],
        [
            "Project Management Bootcamp",
            "Operations",
            "PMI Trainers",
            "April",
            "7",
            "3",
            "12000",
            "20",
            "20",
            "4.7",
        ],
    ];

    let txt = Format::new()
        .font_size(10.0)
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");
    let num = Format::new()
        .font_size(10.0)
        .align(Align::Center)
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");
    let money = Format::new()
        .font_size(10.0)
        .align(Align::Center)
        .num_format("$ #,##0")
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");
    let pct = Format::new()
        .font_size(10.0)
        .align(Align::Center)
        .num_format("0%")
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");

    for (i, p) in programs.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write_with_format(r, 0, p[0], &txt)?; // Program
        ws.write_with_format(r, 1, p[1], &txt)?; // Department
        ws.write_with_format(r, 2, p[2], &txt)?; // Provider
        ws.write_with_format(r, 3, p[3], &txt)?; // Month
        ws.write_with_format(r, 4, p[4].parse::<f64>().unwrap(), &num)?; // Day
        ws.write_with_format(r, 5, p[5].parse::<f64>().unwrap(), &num)?; // Duration
        ws.write_with_format(r, 6, p[6].parse::<f64>().unwrap(), &money)?; // Cost
        ws.write_with_format(r, 7, p[7].parse::<f64>().unwrap(), &num)?; // Capacity
        ws.write_with_format(r, 8, p[8].parse::<f64>().unwrap(), &num)?; // Enrolled
        // Attended = formula (placeholder — user fills in)
        ws.write_formula(r, 9, &format!("I{r1}", r1 = r + 1))?;
        ws.set_cell_format(r, 9, &num)?;
        // Fill Rate = Enrolled / Capacity
        ws.write_formula(r, 10, &format!("I{r1}/H{r1}", r1 = r + 1))?;
        ws.set_cell_format(r, 10, &pct)?;
        // Rating
        ws.write_with_format(r, 11, p[9].parse::<f64>().unwrap(), &num)?;
    }

    let n = programs.len();
    let last = n + 1; // 1-indexed last data row

    // Column widths
    let widths = [
        30.0, 14.0, 22.0, 12.0, 6.0, 8.0, 12.0, 10.0, 10.0, 10.0, 10.0, 8.0,
    ];
    for (c, w) in widths.iter().enumerate() {
        ws.set_column_width(c as u16, *w)?;
    }

    // Add table for filtering
    ws.add_table(
        0,
        0,
        last as u32,
        11,
        Table::new().set_style(TableStyle::Medium(4)).set_columns(
            &headers
                .iter()
                .map(|h| TableColumn::new(h))
                .collect::<Vec<_>>(),
        ),
    )?;

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Dashboard (ALL formulas referencing Data sheet)
    // ═══════════════════════════════════════════════════════════
    let ws2 = wb.add_worksheet_with_name("Dashboard")?;
    ws2.hide_gridlines();

    let dw = [2.0, 18.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 2.0];
    for (c, w) in dw.iter().enumerate() {
        ws2.set_column_width(c as u16, *w)?;
    }

    // Styles
    let gold = "#E8A317";
    let dark_gold = "#C68E17";
    let navy = "#1B2A4A";
    let green = "#0D7C3D";

    let title_fmt = Format::new()
        .bold()
        .font_size(22.0)
        .font_color("#FFFFFF")
        .background_color(gold)
        .align(Align::Left)
        .align(Align::VerticalCenter);
    let title_bar = Format::new().background_color(gold);
    let section_green = Format::new()
        .bold()
        .font_size(11.0)
        .font_color("#FFFFFF")
        .background_color(green)
        .align(Align::Center)
        .border(BorderStyle::Thin);
    let section_orange = Format::new()
        .bold()
        .font_size(11.0)
        .font_color("#FFFFFF")
        .background_color(gold)
        .align(Align::Center)
        .border(BorderStyle::Thin);
    let kpi_label = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#666666")
        .align(Align::Center);
    let kpi_val = Format::new()
        .bold()
        .font_size(16.0)
        .font_color(navy)
        .align(Align::Center);
    let kpi_money = Format::new()
        .bold()
        .font_size(16.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("$ #,##0");
    let kpi_pct = Format::new()
        .bold()
        .font_size(16.0)
        .font_color(green)
        .align(Align::Center)
        .num_format("0%");
    let dept_hdr = Format::new()
        .bold()
        .font_size(10.0)
        .font_color("#FFFFFF")
        .background_color(dark_gold)
        .align(Align::Center)
        .border(BorderStyle::Thin);
    let dept_val = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");
    let dept_money = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("$ #,##0")
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");
    let dept_pct = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("0%")
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");
    let dept_name = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");

    let mut r = 0u32;

    // Title
    for c in 0..=8u16 {
        ws2.write_with_format(r, c, "", &title_bar)?;
    }
    ws2.write_with_format(r, 1, "Training Dashboard", &title_fmt)?;
    ws2.set_row_height(r, 44.0)?;
    r += 2;

    // ── KPI Section (all formulas) ──
    ws2.merge_range(r, 1, r, 4, "People", &section_green)?;
    ws2.merge_range(r, 5, r, 8, "Courses", &section_orange)?;
    ws2.set_row_height(r, 24.0)?;
    r += 1;

    // KPI labels
    let labels = [
        "Total Enrolled",
        "Avg per Course",
        "Max Enrolled",
        "Min Enrolled",
        "Total Programs",
        "Total Cost",
        "Total Capacity",
        "Avg Fill Rate",
    ];
    for (c, l) in labels.iter().enumerate() {
        ws2.write_with_format(r, (c + 1) as u16, *l, &kpi_label)?;
    }
    r += 1;

    // KPI formulas — all reference Data sheet
    let dr = format!("Data!I2:I{last}"); // enrolled range
    let cr = format!("Data!G2:G{last}"); // cost range
    let hr = format!("Data!H2:H{last}"); // capacity range
    let kr = format!("Data!K2:K{last}"); // fill rate range

    ws2.write_formula(r, 1, &format!("SUM({dr})"))?;
    ws2.set_cell_format(r, 1, &kpi_val)?;
    ws2.write_formula(r, 2, &format!("AVERAGE({dr})"))?;
    ws2.set_cell_format(r, 2, &kpi_val)?;
    ws2.write_formula(r, 3, &format!("MAX({dr})"))?;
    ws2.set_cell_format(r, 3, &kpi_val)?;
    ws2.write_formula(r, 4, &format!("MIN({dr})"))?;
    ws2.set_cell_format(r, 4, &kpi_val)?;
    ws2.write_formula(r, 5, &format!("COUNTA(Data!A2:A{last})"))?;
    ws2.set_cell_format(r, 5, &kpi_val)?;
    ws2.write_formula(r, 6, &format!("SUM({cr})"))?;
    ws2.set_cell_format(r, 6, &kpi_money)?;
    ws2.write_formula(r, 7, &format!("SUM({hr})"))?;
    ws2.set_cell_format(r, 7, &kpi_val)?;
    ws2.write_formula(r, 8, &format!("AVERAGE({kr})"))?;
    ws2.set_cell_format(r, 8, &kpi_pct)?;
    ws2.set_row_height(r, 30.0)?;
    r += 2;

    // ── Department Breakdown (all COUNTIFS/SUMIFS/AVERAGEIFS) ──
    ws2.merge_range(r, 1, r, 8, "Department Breakdown", &section_orange)?;
    ws2.set_row_height(r, 24.0)?;
    r += 1;

    let dept_headers = [
        "Department",
        "Programs",
        "Total Cost",
        "Avg Cost",
        "Total Enrolled",
        "Avg Fill Rate",
        "Avg Rating",
        "Over-subscribed",
    ];
    for (c, h) in dept_headers.iter().enumerate() {
        ws2.write_with_format(r, (c + 1) as u16, *h, &dept_hdr)?;
    }
    ws2.set_row_height(r, 22.0)?;
    r += 1;

    let departments = ["Finance", "HR", "IT", "Marketing", "Operations", "Sales"];
    let dept_range = format!("Data!B2:B{last}");

    for dept in &departments {
        ws2.write_with_format(r, 1, *dept, &dept_name)?;
        // Programs = COUNTIF
        ws2.write_formula(r, 2, &format!("COUNTIF({dept_range},\"{dept}\")"))?;
        ws2.set_cell_format(r, 2, &dept_val)?;
        // Total Cost = SUMIF
        ws2.write_formula(r, 3, &format!("SUMIF({dept_range},\"{dept}\",{cr})"))?;
        ws2.set_cell_format(r, 3, &dept_money)?;
        // Avg Cost = AVERAGEIF
        ws2.write_formula(r, 4, &format!("AVERAGEIF({dept_range},\"{dept}\",{cr})"))?;
        ws2.set_cell_format(r, 4, &dept_money)?;
        // Total Enrolled = SUMIF
        ws2.write_formula(r, 5, &format!("SUMIF({dept_range},\"{dept}\",{dr})"))?;
        ws2.set_cell_format(r, 5, &dept_val)?;
        // Avg Fill Rate = AVERAGEIF
        ws2.write_formula(r, 6, &format!("AVERAGEIF({dept_range},\"{dept}\",{kr})"))?;
        ws2.set_cell_format(r, 6, &dept_pct)?;
        // Avg Rating = AVERAGEIF (only rated > 0)
        ws2.write_formula(
            r,
            7,
            &format!("AVERAGEIFS(Data!L2:L{last},{dept_range},\"{dept}\",Data!L2:L{last},\">0\")"),
        )?;
        ws2.set_cell_format(r, 7, &dept_val)?;
        // Over-subscribed = COUNTIFS(dept, fill > 100%)
        ws2.write_formula(
            r,
            8,
            &format!("COUNTIFS({dept_range},\"{dept}\",{kr},\">1\")"),
        )?;
        ws2.set_cell_format(r, 8, &dept_val)?;
        ws2.set_row_height(r, 22.0)?;
        r += 1;
    }

    // Totals row
    let dept_start = r - departments.len() as u32;
    let dept_end = r - 1;
    let total_fmt = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .background_color("#FFF8E7")
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");
    let total_money = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("$ #,##0")
        .background_color("#FFF8E7")
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");
    let total_pct = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(green)
        .align(Align::Center)
        .num_format("0%")
        .background_color("#FFF8E7")
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");
    let total_label = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .background_color("#FFF8E7")
        .border(BorderStyle::Thin)
        .border_color("#E0D5B0");

    ws2.write_with_format(r, 1, "TOTAL", &total_label)?;
    ws2.write_formula(r, 2, &format!("SUM(C{}:C{})", dept_start + 1, dept_end + 1))?;
    ws2.set_cell_format(r, 2, &total_fmt)?;
    ws2.write_formula(r, 3, &format!("SUM(D{}:D{})", dept_start + 1, dept_end + 1))?;
    ws2.set_cell_format(r, 3, &total_money)?;
    ws2.write_formula(
        r,
        4,
        &format!("AVERAGE(E{}:E{})", dept_start + 1, dept_end + 1),
    )?;
    ws2.set_cell_format(r, 4, &total_money)?;
    ws2.write_formula(r, 5, &format!("SUM(F{}:F{})", dept_start + 1, dept_end + 1))?;
    ws2.set_cell_format(r, 5, &total_fmt)?;
    ws2.write_formula(
        r,
        6,
        &format!("AVERAGE(G{}:G{})", dept_start + 1, dept_end + 1),
    )?;
    ws2.set_cell_format(r, 6, &total_pct)?;
    ws2.write_formula(
        r,
        7,
        &format!("AVERAGE(H{}:H{})", dept_start + 1, dept_end + 1),
    )?;
    ws2.set_cell_format(r, 7, &total_fmt)?;
    ws2.write_formula(r, 8, &format!("SUM(I{}:I{})", dept_start + 1, dept_end + 1))?;
    ws2.set_cell_format(r, 8, &total_fmt)?;
    r += 2;

    // ── Charts (referencing the formula cells on this sheet) ──
    let chart_row = r;

    // Cost by department
    let mut chart1 = Chart::new(ChartType::Column);
    chart1.set_title("Training Spend by Department");
    chart1.set_width(480);
    chart1.set_height(300);
    chart1.set_y_axis_name("Cost ($)");
    chart1.set_legend_position(LegendPosition::Bottom);
    let s1 = chart1.add_series();
    s1.set_values(&format!(
        "Dashboard!$D${}:$D${}",
        dept_start + 1,
        dept_end + 1
    ));
    s1.set_categories(&format!(
        "Dashboard!$B${}:$B${}",
        dept_start + 1,
        dept_end + 1
    ));
    s1.set_name("Total Cost");
    s1.set_data_labels(true);
    s1.set_color(gold);
    ws2.insert_chart(chart_row, 1, &chart1)?;

    // Fill rate by department
    let mut chart2 = Chart::new(ChartType::Bar);
    chart2.set_title("Avg Fill Rate by Department");
    chart2.set_width(420);
    chart2.set_height(300);
    chart2.set_legend_position(LegendPosition::Bottom);
    let s2 = chart2.add_series();
    s2.set_values(&format!(
        "Dashboard!$G${}:$G${}",
        dept_start + 1,
        dept_end + 1
    ));
    s2.set_categories(&format!(
        "Dashboard!$B${}:$B${}",
        dept_start + 1,
        dept_end + 1
    ));
    s2.set_name("Avg Fill Rate");
    s2.set_data_labels(true);
    s2.set_color(green);
    ws2.insert_chart(chart_row, 5, &chart2)?;

    // Conditional formatting on fill rate
    ws2.add_conditional_format(
        dept_start,
        6,
        dept_end,
        6,
        ConditionalFormatDataBar::new(green),
    )?;

    // Print setup
    ws2.set_landscape();
    ws2.set_fit_to_page(1, 1);
    ws2.set_margins(0.4, 0.4, 0.4, 0.4);
    ws2.set_header("&CTraining Dashboard");
    ws2.set_footer("&CPage &P  |  &D");

    // ── Save ──
    wb.set_active_sheet(1);
    let path = std::path::PathBuf::from("output/training_tracker.xlsx");
    wb.save(&path)?;
    println!("✅ Training Tracker saved to {}", path.display());
    Ok(())
}
