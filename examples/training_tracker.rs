use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    // ── Training Data ──
    let departments = ["Finance", "HR", "IT", "Marketing", "Operations", "Sales"];
    let programs: Vec<Program> = vec![
        Program { name: "Advanced People Management Methods", dept: "HR", month: "October", day: 15,
            duration: "1 day", provider: "Libero At Auctor Associates", cost: 5000.0,
            capacity: 20, enrolled: 6, attended: 6, fill_pct: 1.20, rating: 3.5 },
        Program { name: "Advanced IT Techniques", dept: "IT", month: "November", day: 9,
            duration: "1 day", provider: "Nulla Eu Incorporated", cost: 5000.0,
            capacity: 30, enrolled: 6, attended: 3, fill_pct: 0.20, rating: 0.0 },
        Program { name: "Advanced IT Skills", dept: "IT", month: "December", day: 1,
            duration: "1.5 days", provider: "Vivamus Industries", cost: 3000.0,
            capacity: 15, enrolled: 6, attended: 3, fill_pct: 0.40, rating: 0.0 },
        Program { name: "Advanced Business Techniques", dept: "Sales", month: "December", day: 14,
            duration: "1 day", provider: "Eu Dolor Egestas Inc.", cost: 3000.0,
            capacity: 20, enrolled: 1, attended: 0, fill_pct: 0.05, rating: 0.0 },
        Program { name: "Leadership Essentials", dept: "Finance", month: "January", day: 20,
            duration: "2 days", provider: "Apex Training Group", cost: 8000.0,
            capacity: 25, enrolled: 22, attended: 20, fill_pct: 0.88, rating: 4.5 },
        Program { name: "Digital Marketing Masterclass", dept: "Marketing", month: "January", day: 27,
            duration: "1 day", provider: "DigiLearn Corp", cost: 4500.0,
            capacity: 30, enrolled: 28, attended: 25, fill_pct: 0.93, rating: 4.0 },
        Program { name: "Supply Chain Optimization", dept: "Operations", month: "February", day: 10,
            duration: "2 days", provider: "LogiTrain Partners", cost: 7500.0,
            capacity: 20, enrolled: 18, attended: 16, fill_pct: 0.90, rating: 4.2 },
        Program { name: "Excel for Finance Professionals", dept: "Finance", month: "February", day: 24,
            duration: "1 day", provider: "DataSkills Academy", cost: 3500.0,
            capacity: 25, enrolled: 25, attended: 23, fill_pct: 1.00, rating: 4.8 },
        Program { name: "Conflict Resolution Workshop", dept: "HR", month: "March", day: 5,
            duration: "0.5 day", provider: "PeopleSoft Training", cost: 2500.0,
            capacity: 15, enrolled: 14, attended: 12, fill_pct: 0.93, rating: 3.8 },
        Program { name: "Cybersecurity Fundamentals", dept: "IT", month: "March", day: 17,
            duration: "2 days", provider: "SecureNet Institute", cost: 9000.0,
            capacity: 20, enrolled: 20, attended: 18, fill_pct: 1.00, rating: 4.6 },
        Program { name: "Sales Negotiation Tactics", dept: "Sales", month: "March", day: 24,
            duration: "1 day", provider: "CloseRate Academy", cost: 5500.0,
            capacity: 25, enrolled: 24, attended: 22, fill_pct: 0.96, rating: 4.3 },
        Program { name: "Project Management Bootcamp", dept: "Operations", month: "April", day: 7,
            duration: "3 days", provider: "PMI Certified Trainers", cost: 12000.0,
            capacity: 20, enrolled: 20, attended: 19, fill_pct: 1.00, rating: 4.7 },
    ];

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Dashboard Summary
    // ═══════════════════════════════════════════════════════════
    build_dashboard(&mut wb, &programs, &departments)?;

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Program Details (full table)
    // ═══════════════════════════════════════════════════════════
    build_details(&mut wb, &programs)?;

    // ═══════════════════════════════════════════════════════════
    // Sheet 3: Analytics Charts
    // ═══════════════════════════════════════════════════════════
    build_analytics(&mut wb, &programs, &departments)?;

    wb.set_active_sheet(0);
    let path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into()))
        .join("Downloads/training_tracker.xlsx");
    wb.save(&path)?;
    println!("✅ Training Tracker saved to {}", path.display());
    Ok(())
}

struct Program {
    name: &'static str, dept: &'static str, month: &'static str, day: u32,
    duration: &'static str, provider: &'static str, cost: f64,
    capacity: u32, enrolled: u32, attended: u32, fill_pct: f64, rating: f64,
}

fn stars(rating: f64) -> String {
    let full = rating.floor() as usize;
    let half = if rating - rating.floor() >= 0.5 { 1 } else { 0 };
    let empty = 5 - full - half;
    format!("{}{}{}", "★".repeat(full), if half > 0 { "½" } else { "" }, "☆".repeat(empty))
}

// ── Colors ──
const GOLD: &str = "#E8A317";
const DARK_GOLD: &str = "#C68E17";
const GREEN: &str = "#0D7C3D";
const ORANGE: &str = "#E8A317";
const RED: &str = "#C00000";
const NAVY: &str = "#1B2A4A";
const LIGHT_BG: &str = "#FFF8E7";
const WHITE: &str = "#FFFFFF";
const BORDER: &str = "#E0D5B0";

fn build_dashboard(wb: &mut Workbook, programs: &[Program], departments: &[&str]) -> Result<()> {
    let ws = wb.worksheet(0)?;
    ws.set_name("Training Dashboard")?;
    ws.hide_gridlines();

    let widths = [2.0, 16.0, 12.0, 12.0, 12.0, 14.0, 12.0, 12.0, 12.0, 12.0, 2.0];
    for (c, w) in widths.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }

    let title_fmt = Format::new().bold().font_size(22.0).font_color(WHITE)
        .background_color(GOLD).align(Align::Left).align(Align::VerticalCenter);
    let title_bar = Format::new().background_color(GOLD);
    let subtitle_fmt = Format::new().font_size(9.0).font_color("#666666").italic();

    let section_people = Format::new().bold().font_size(11.0).font_color(WHITE)
        .background_color(GREEN).align(Align::Center).border(BorderStyle::Thin).border_color(GREEN);
    let section_courses = Format::new().bold().font_size(11.0).font_color(WHITE)
        .background_color(ORANGE).align(Align::Center).border(BorderStyle::Thin).border_color(ORANGE);

    let hdr = Format::new().bold().font_size(9.0).font_color("#666666").align(Align::Center);
    let val_big = Format::new().bold().font_size(14.0).font_color(NAVY).align(Align::Center);
    let val_pct = Format::new().bold().font_size(14.0).font_color(NAVY).align(Align::Center).num_format("0%");
    let val_money = Format::new().bold().font_size(14.0).font_color(NAVY).align(Align::Center).num_format("$ #,##0");
    let val_green = Format::new().bold().font_size(12.0).font_color(GREEN).align(Align::Center);
    let label_fmt = Format::new().bold().font_size(11.0).font_color(NAVY).align(Align::Left);

    let mut r = 0u32;

    // Title bar
    for c in 0..=9u16 { ws.write_with_format(r, c, "", &title_bar)?; }
    ws.write_with_format(r, 1, "Training - Calendar View", &title_fmt)?;
    ws.set_row_height(r, 44.0)?; r += 1;

    // Department tags
    let dept_tag = Format::new().bold().font_size(9.0).font_color(WHITE)
        .background_color("#4CAF50").align(Align::Center).border(BorderStyle::Thin);
    let dept_tag_orange = Format::new().bold().font_size(9.0).font_color(WHITE)
        .background_color(ORANGE).align(Align::Center).border(BorderStyle::Thin);
    ws.write_with_format(r, 1, "Select a department:", &subtitle_fmt)?;
    for (i, dept) in departments.iter().enumerate() {
        let f = if i % 2 == 0 { &dept_tag } else { &dept_tag_orange };
        ws.write_with_format(r, (i + 2) as u16, *dept, f)?;
    }
    ws.set_row_height(r, 20.0)?; r += 1;

    ws.write_with_format(r, 1, "Selected dept values are shown in green or (brackets)", &subtitle_fmt)?;
    r += 2;

    // ── Summary KPIs ──
    // People section
    ws.merge_range(r, 1, r, 3, "People", &section_people)?;
    ws.merge_range(r, 5, r, 9, "Courses", &section_courses)?;
    ws.set_row_height(r, 24.0)?; r += 1;

    // People headers
    ws.write_with_format(r, 2, "Count", &hdr)?;
    ws.write_with_format(r, 3, "1 Course", &hdr)?;
    ws.write_with_format(r, 4, ">1 Course", &hdr)?;
    // Course headers
    ws.write_with_format(r, 5, "Count", &hdr)?;
    ws.write_with_format(r, 6, "Cost", &hdr)?;
    ws.write_with_format(r, 7, "Cap.", &hdr)?;
    ws.write_with_format(r, 8, "Att.", &hdr)?;
    ws.write_with_format(r, 9, "Fill Rate", &hdr)?;
    r += 1;

    // All Depts row
    let total_people = 100u32;
    let one_course = 63u32;
    let multi_course = 38u32;
    let total_courses = programs.len() as u32;
    let total_cost: f64 = programs.iter().map(|p| p.cost).sum();
    let total_cap: u32 = programs.iter().map(|p| p.capacity).sum();
    let total_att: u32 = programs.iter().map(|p| p.attended).sum();
    let avg_fill: f64 = programs.iter().map(|p| p.fill_pct).sum::<f64>() / programs.len() as f64;

    ws.write_with_format(r, 1, "All Depts", &label_fmt)?;
    ws.write_with_format(r, 2, total_people as f64, &val_big)?;
    ws.write_with_format(r, 3, one_course as f64, &val_big)?;
    ws.write_with_format(r, 4, multi_course as f64, &val_big)?;
    ws.write_with_format(r, 5, total_courses as f64, &val_big)?;
    ws.write_with_format(r, 6, total_cost, &val_money)?;
    ws.write_with_format(r, 7, total_cap as f64, &val_big)?;
    ws.write_with_format(r, 8, total_att as f64, &val_big)?;
    ws.write_with_format(r, 9, avg_fill, &val_pct)?;
    ws.set_row_height(r, 28.0)?; r += 2;

    // ── Calendar Cards ──
    let card_title = Format::new().bold().font_size(10.0).font_color(WHITE)
        .background_color(DARK_GOLD).align(Align::Left);
    let card_month = Format::new().bold().font_size(11.0).font_color(RED)
        .align(Align::Left);
    let card_day = Format::new().bold().font_size(22.0).font_color(DARK_GOLD)
        .align(Align::Center).align(Align::VerticalCenter)
        .background_color(LIGHT_BG).border(BorderStyle::Thin).border_color(BORDER);
    let card_name = Format::new().bold().font_size(11.0).font_color(NAVY).italic()
        .align(Align::Left).align(Align::VerticalCenter);
    let card_detail = Format::new().font_size(8.0).font_color("#666666").align(Align::Left);
    let card_stats = Format::new().font_size(8.0).font_color(GREEN).align(Align::Left);

    // Show programs as calendar cards (3 per row)
    let mut pi = 0;
    while pi < programs.len() {
        // Month headers
        let cols_per_card = 3u16;
        for ci in 0..3 {
            if pi + ci >= programs.len() { break; }
            let p = &programs[pi + ci];
            let base_col = 1 + (ci as u16) * cols_per_card;
            ws.merge_range(r, base_col, r, base_col + cols_per_card - 1,
                &format!("{}, 2025", p.month), &card_month)?;
        }
        ws.set_row_height(r, 18.0)?; r += 1;

        // Day + Name
        for ci in 0..3 {
            if pi + ci >= programs.len() { break; }
            let p = &programs[pi + ci];
            let base_col = 1 + (ci as u16) * cols_per_card;
            ws.write_with_format(r, base_col, p.day as f64, &card_day)?;
            ws.merge_range(r, base_col + 1, r, base_col + cols_per_card - 1, p.name, &card_name)?;
        }
        ws.set_row_height(r, 36.0)?; r += 1;

        // Provider + cost
        for ci in 0..3 {
            if pi + ci >= programs.len() { break; }
            let p = &programs[pi + ci];
            let base_col = 1 + (ci as u16) * cols_per_card;
            let detail = format!("{} - $ {}k", p.provider, p.cost / 1000.0);
            ws.write_with_format(r, base_col, p.duration, &card_detail)?;
            ws.merge_range(r, base_col + 1, r, base_col + cols_per_card - 1, detail.as_str(), &card_detail)?;
        }
        r += 1;

        // Attendance + fill + rating
        for ci in 0..3 {
            if pi + ci >= programs.len() { break; }
            let p = &programs[pi + ci];
            let base_col = 1 + (ci as u16) * cols_per_card;
            let stats = format!("Att: {} ({}) Fill:{:.0}%", p.enrolled, p.attended, p.fill_pct * 100.0);
            ws.write_with_format(r, base_col, stats.as_str(), &card_stats)?;
            if p.rating > 0.0 {
                ws.write_with_format(r, base_col + 2, stars(p.rating).as_str(), &card_stats)?;
            }
        }
        ws.set_row_height(r, 16.0)?; r += 2;

        pi += 3;
    }

    // Print setup
    ws.set_landscape();
    ws.set_paper_size(1);
    ws.set_fit_to_page(1, 1);
    ws.set_margins(0.4, 0.4, 0.4, 0.4);
    ws.set_header("&CTraining Calendar View");
    Ok(())
}

fn build_details(wb: &mut Workbook, programs: &[Program]) -> Result<()> {
    let ws = wb.add_worksheet_with_name("Program Details")?;
    ws.hide_gridlines();

    let widths = [30.0, 14.0, 14.0, 10.0, 14.0, 10.0, 10.0, 10.0, 10.0, 10.0];
    for (c, w) in widths.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }

    let title = Format::new().bold().font_size(18.0).font_color(WHITE)
        .background_color(GOLD).align(Align::Left).align(Align::VerticalCenter);
    let title_bar = Format::new().background_color(GOLD);
    ws.write_with_format(0, 0, "Training Programs — Full Details", &title)?;
    for c in 1..=9u16 { ws.write_with_format(0, c, "", &title_bar)?; }
    ws.set_row_height(0, 36.0)?;

    let hdr = Format::new().bold().font_size(10.0).font_color(WHITE)
        .background_color(DARK_GOLD).align(Align::Center).border(BorderStyle::Thin).border_color(DARK_GOLD);
    let hdr_left = Format::new().bold().font_size(10.0).font_color(WHITE)
        .background_color(DARK_GOLD).align(Align::Left).border(BorderStyle::Thin).border_color(DARK_GOLD);

    let headers = ["Program", "Department", "Provider", "Month", "Duration", "Cost", "Capacity", "Enrolled", "Attended", "Fill Rate"];
    for (c, h) in headers.iter().enumerate() {
        let f = if c <= 3 { &hdr_left } else { &hdr };
        ws.write_with_format(1, c as u16, *h, f)?;
    }
    ws.set_row_height(1, 24.0)?;

    for (i, p) in programs.iter().enumerate() {
        let r = (i + 2) as u32;
        let alt = i % 2 == 1;
        let bg = if alt { LIGHT_BG } else { WHITE };

        let txt = Format::new().font_size(10.0).font_color(NAVY).background_color(bg)
            .align(Align::Left).border(BorderStyle::Thin).border_color(BORDER);
        let num = Format::new().font_size(10.0).font_color(NAVY).background_color(bg)
            .align(Align::Center).border(BorderStyle::Thin).border_color(BORDER);
        let money = Format::new().font_size(10.0).font_color(NAVY).background_color(bg)
            .align(Align::Center).num_format("$ #,##0").border(BorderStyle::Thin).border_color(BORDER);
        let fill_color = if p.fill_pct >= 0.90 { GREEN } else if p.fill_pct >= 0.50 { ORANGE } else { RED };
        let pct = Format::new().font_size(10.0).bold().font_color(fill_color).background_color(bg)
            .align(Align::Center).num_format("0%").border(BorderStyle::Thin).border_color(BORDER);

        ws.write_with_format(r, 0, p.name, &txt)?;
        ws.write_with_format(r, 1, p.dept, &txt)?;
        ws.write_with_format(r, 2, p.provider, &txt)?;
        ws.write_with_format(r, 3, p.month, &txt)?;
        ws.write_with_format(r, 4, p.duration, &num)?;
        ws.write_with_format(r, 5, p.cost, &money)?;
        ws.write_with_format(r, 6, p.capacity as f64, &num)?;
        ws.write_with_format(r, 7, p.enrolled as f64, &num)?;
        ws.write_with_format(r, 8, p.attended as f64, &num)?;
        ws.write_with_format(r, 9, p.fill_pct, &pct)?;
        ws.set_row_height(r, 22.0)?;
    }

    // Fill rate data bars
    let last = programs.len() as u32 + 1;
    ws.add_conditional_format(2, 9, last, 9, ConditionalFormatDataBar::new(GOLD))?;

    ws.set_landscape();
    ws.set_fit_to_page(1, 1);
    Ok(())
}

fn build_analytics(wb: &mut Workbook, programs: &[Program], departments: &[&str]) -> Result<()> {
    let ws = wb.add_worksheet_with_name("Analytics")?;
    ws.hide_gridlines();

    let widths = [2.0, 16.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 2.0];
    for (c, w) in widths.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }

    let title = Format::new().bold().font_size(18.0).font_color(WHITE)
        .background_color(GOLD).align(Align::Left).align(Align::VerticalCenter);
    let title_bar = Format::new().background_color(GOLD);
    ws.write_with_format(0, 1, "Training Analytics", &title)?;
    for c in 2..=7u16 { ws.write_with_format(0, c, "", &title_bar)?; }
    ws.set_row_height(0, 36.0)?;

    // Dept summary data for charts
    let data_row = 2u32;
    let hdr = Format::new().bold().font_size(9.0).font_color("#666666").align(Align::Center);
    ws.write_with_format(data_row, 1, "Department", &hdr)?;
    ws.write_with_format(data_row, 2, "Programs", &hdr)?;
    ws.write_with_format(data_row, 3, "Total Cost", &hdr)?;
    ws.write_with_format(data_row, 4, "Avg Fill %", &hdr)?;
    ws.write_with_format(data_row, 5, "Avg Rating", &hdr)?;

    let val = Format::new().font_size(10.0).font_color(NAVY).align(Align::Center)
        .border(BorderStyle::Thin).border_color(BORDER);
    let val_m = Format::new().font_size(10.0).font_color(NAVY).align(Align::Center)
        .num_format("$ #,##0").border(BorderStyle::Thin).border_color(BORDER);
    let val_p = Format::new().font_size(10.0).font_color(NAVY).align(Align::Center)
        .num_format("0%").border(BorderStyle::Thin).border_color(BORDER);

    for (i, dept) in departments.iter().enumerate() {
        let r = data_row + 1 + i as u32;
        let dept_programs: Vec<&Program> = programs.iter().filter(|p| p.dept == *dept).collect();
        let count = dept_programs.len() as f64;
        let cost: f64 = dept_programs.iter().map(|p| p.cost).sum();
        let avg_fill = if count > 0.0 { dept_programs.iter().map(|p| p.fill_pct).sum::<f64>() / count } else { 0.0 };
        let rated: Vec<&&Program> = dept_programs.iter().filter(|p| p.rating > 0.0).collect();
        let avg_rating = if !rated.is_empty() { rated.iter().map(|p| p.rating).sum::<f64>() / rated.len() as f64 } else { 0.0 };

        ws.write_with_format(r, 1, *dept, &val)?;
        ws.write_with_format(r, 2, count, &val)?;
        ws.write_with_format(r, 3, cost, &val_m)?;
        ws.write_with_format(r, 4, avg_fill, &val_p)?;
        ws.write_with_format(r, 5, avg_rating, &val)?;
    }

    let last_data = data_row + departments.len() as u32;

    // ── Column chart: Cost by department ──
    let chart_row = last_data + 2;
    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Training Spend by Department");
    chart.set_width(520);
    chart.set_height(320);
    chart.set_y_axis_name("Cost ($)");
    chart.set_legend_position(LegendPosition::Bottom);

    let s = chart.add_series();
    s.set_values(&format!("Analytics!$D${}:$D${}", data_row + 2, last_data + 1));
    s.set_categories(&format!("Analytics!$B${}:$B${}", data_row + 2, last_data + 1));
    s.set_name("Total Cost");
    s.set_data_labels(true);
    s.set_color(GOLD);
    ws.insert_chart(chart_row, 1, &chart)?;

    // ── Bar chart: Fill rate by department ──
    let mut bar = Chart::new(ChartType::Bar);
    bar.set_title("Avg Fill Rate by Department");
    bar.set_width(420);
    bar.set_height(320);
    bar.set_legend_position(LegendPosition::Bottom);

    let bs = bar.add_series();
    bs.set_values(&format!("Analytics!$E${}:$E${}", data_row + 2, last_data + 1));
    bs.set_categories(&format!("Analytics!$B${}:$B${}", data_row + 2, last_data + 1));
    bs.set_name("Avg Fill Rate");
    bs.set_data_labels(true);
    bs.set_color(GREEN);
    ws.insert_chart(chart_row, 5, &bar)?;

    ws.set_landscape();
    ws.set_fit_to_page(1, 1);
    ws.set_header("&CTraining Analytics");
    Ok(())
}
