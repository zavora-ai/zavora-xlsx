use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    let navy = "#1B2A4A"; let accent = "#2B579A"; let border = "#D6DCE4";
    let colors = ["#4472C4","#ED7D31","#70AD47","#FFC000","#5B9BD5","#C00000","#7B1FA2","#00B0F0"];

    // ── Project Data ──
    let phases: Vec<Phase> = vec![
        Phase { name: "Planning", color: 0, tasks: vec![
            Task { name: "Project Kickoff",       start: (2025,4,7),  end: (2025,4,7),  pct: 100, owner: "James K." },
            Task { name: "Requirements Gathering", start: (2025,4,8),  end: (2025,4,14), pct: 100, owner: "Sarah M." },
            Task { name: "Scope Definition",       start: (2025,4,10), end: (2025,4,16), pct: 80,  owner: "James K." },
            Task { name: "Resource Allocation",    start: (2025,4,14), end: (2025,4,18), pct: 50,  owner: "David O." },
        ]},
        Phase { name: "Design", color: 1, tasks: vec![
            Task { name: "System Architecture",    start: (2025,4,21), end: (2025,4,30), pct: 30,  owner: "Grace N." },
            Task { name: "Database Design",        start: (2025,4,23), end: (2025,5,2),  pct: 20,  owner: "Peter L." },
            Task { name: "UI/UX Wireframes",       start: (2025,4,25), end: (2025,5,7),  pct: 10,  owner: "Amina W." },
            Task { name: "Design Review",          start: (2025,5,8),  end: (2025,5,9),  pct: 0,   owner: "James K." },
        ]},
        Phase { name: "Development", color: 2, tasks: vec![
            Task { name: "Backend API",            start: (2025,5,12), end: (2025,6,6),  pct: 0, owner: "Peter L." },
            Task { name: "Frontend UI",            start: (2025,5,14), end: (2025,6,11), pct: 0, owner: "Amina W." },
            Task { name: "Database Implementation", start: (2025,5,12), end: (2025,5,23), pct: 0, owner: "Grace N." },
            Task { name: "Integration",            start: (2025,6,2),  end: (2025,6,13), pct: 0, owner: "Peter L." },
            Task { name: "Code Review",            start: (2025,6,16), end: (2025,6,18), pct: 0, owner: "David O." },
        ]},
        Phase { name: "Testing", color: 3, tasks: vec![
            Task { name: "Unit Testing",           start: (2025,5,19), end: (2025,6,13), pct: 0, owner: "Grace N." },
            Task { name: "Integration Testing",    start: (2025,6,16), end: (2025,6,25), pct: 0, owner: "Sarah M." },
            Task { name: "UAT",                    start: (2025,6,26), end: (2025,7,4),  pct: 0, owner: "James K." },
            Task { name: "Bug Fixes",              start: (2025,7,7),  end: (2025,7,11), pct: 0, owner: "Peter L." },
        ]},
        Phase { name: "Deployment", color: 4, tasks: vec![
            Task { name: "Staging Deploy",         start: (2025,7,14), end: (2025,7,16), pct: 0, owner: "David O." },
            Task { name: "Production Deploy",      start: (2025,7,17), end: (2025,7,18), pct: 0, owner: "David O." },
            Task { name: "Post-Launch Monitoring",  start: (2025,7,21), end: (2025,7,25), pct: 0, owner: "Grace N." },
            Task { name: "Project Closeout",       start: (2025,7,28), end: (2025,7,30), pct: 0, owner: "James K." },
        ]},
    ];

    // Project date range for Gantt columns: Apr 7 – Jul 31 = ~17 weeks
    let proj_start = serial(2025, 4, 7);
    let proj_end = serial(2025, 7, 31);
    let num_weeks = ((proj_end - proj_start) as f64 / 7.0).ceil() as usize;

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Task List + Gantt Chart
    // ═══════════════════════════════════════════════════════════
    let ws = wb.worksheet(0)?;
    ws.set_name("Gantt Chart")?;
    ws.hide_gridlines();

    // Column widths: task info (A-F) + week columns
    ws.set_column_width(0, 2.0)?;  // margin
    ws.set_column_width(1, 28.0)?; // task name
    ws.set_column_width(2, 12.0)?; // owner
    ws.set_column_width(3, 12.0)?; // start
    ws.set_column_width(4, 12.0)?; // end
    ws.set_column_width(5, 8.0)?;  // % complete
    ws.set_column_width(6, 8.0)?;  // duration
    for w in 0..num_weeks { ws.set_column_width((7 + w) as u16, 4.0)?; }

    // Styles
    let title_fmt = Format::new().bold().font_size(20.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let subtitle_fmt = Format::new().font_size(10.0).font_color("#667085").italic();
    let divider_fmt = Format::new().background_color(accent);

    let phase_fmt = |ci: usize| -> Format {
        Format::new().bold().font_size(11.0).font_color("#FFFFFF")
            .background_color(colors[ci % colors.len()])
            .align(Align::Left).align(Align::VerticalCenter)
            .border(BorderStyle::Thin).border_color(colors[ci % colors.len()])
    };
    let hdr_fmt = Format::new().bold().font_size(9.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let hdr_left = Format::new().bold().font_size(9.0).font_color("#FFFFFF")
        .background_color(navy).align(Align::Left).border(BorderStyle::Thin);
    let task_fmt = Format::new().font_size(10.0).font_color(navy)
        .align(Align::Left).border(BorderStyle::Thin).border_color(border);
    let center_fmt = Format::new().font_size(10.0).font_color(navy)
        .align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let date_fmt = Format::new().font_size(9.0).font_color(navy)
        .num_format("m/d").align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let pct_fmt = Format::new().font_size(10.0).font_color(navy)
        .num_format("0%").align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let week_hdr = Format::new().bold().font_size(7.0).font_color(navy)
        .align(Align::Center).background_color("#E8ECF0").border(BorderStyle::Thin).border_color(border);
    let empty_week = Format::new().background_color("#FAFAFA").border(BorderStyle::Thin).border_color("#F0F0F0");

    let mut r = 0u32;

    // Title
    ws.set_row_height(r, 6.0)?; r += 1;
    ws.write_with_format(r, 1, "PROJECT GANTT CHART", &title_fmt)?;
    ws.set_row_height(r, 32.0)?; r += 1;
    ws.write_with_format(r, 1, "ERP Implementation  |  Apr – Jul 2025  |  5 Phases, 21 Tasks", &subtitle_fmt)?;
    r += 1;
    for c in 1..=(6 + num_weeks as u16) { ws.write_with_format(r, c, "", &divider_fmt)?; }
    ws.set_row_height(r, 3.0)?; r += 2;

    // Headers
    let task_headers = ["Task", "Owner", "Start", "End", "% Done", "Days"];
    ws.write_with_format(r, 1, task_headers[0], &hdr_left)?;
    for (c, h) in task_headers[1..].iter().enumerate() {
        ws.write_with_format(r, (c + 2) as u16, *h, &hdr_fmt)?;
    }
    // Week headers
    for w in 0..num_weeks {
        let _week_start = proj_start + (w as f64 * 7.0) as i64;
        let label = format!("W{}", w + 1);
        ws.write_with_format(r, (7 + w) as u16, label.as_str(), &week_hdr)?;
    }
    ws.set_row_height(r, 20.0)?; r += 1;

    // Month sub-headers
    let months = ["Apr", "May", "Jun", "Jul"];
    let month_starts = [serial(2025,4,1), serial(2025,5,1), serial(2025,6,1), serial(2025,7,1)];
    let month_hdr = Format::new().bold().font_size(7.0).font_color(accent)
        .align(Align::Center).background_color("#F5F7FA");
    for w in 0..num_weeks {
        let week_mid = proj_start + (w as f64 * 7.0 + 3.0) as i64;
        for (mi, &ms) in month_starts.iter().enumerate() {
            let me = if mi + 1 < month_starts.len() { month_starts[mi + 1] } else { proj_end + 1 };
            if week_mid >= ms && week_mid < me {
                ws.write_with_format(r, (7 + w) as u16, months[mi], &month_hdr)?;
                break;
            }
        }
    }
    ws.set_row_height(r, 14.0)?; r += 1;

    // ── Tasks with Gantt bars ──
    for phase in &phases {
        // Phase header row
        let pf = phase_fmt(phase.color);
        ws.merge_range(r, 1, r, 6, &format!("  ▸ {}", phase.name), &pf)?;
        for w in 0..num_weeks { ws.write_with_format(r, (7 + w) as u16, "", &pf)?; }
        ws.set_row_height(r, 24.0)?; r += 1;

        for task in &phase.tasks {
            let ts = serial(task.start.0, task.start.1, task.start.2);
            let te = serial(task.end.0, task.end.1, task.end.2);
            let _duration = (te - ts + 1).max(1);

            ws.write_with_format(r, 1, task.name, &task_fmt)?;
            ws.write_with_format(r, 2, task.owner, &center_fmt)?;
            ws.write_with_format(r, 3, ExcelDateTime::from_ymd(task.start.0, task.start.1, task.start.2).unwrap(), &date_fmt)?;
            ws.write_with_format(r, 4, ExcelDateTime::from_ymd(task.end.0, task.end.1, task.end.2).unwrap(), &date_fmt)?;
            ws.write_with_format(r, 5, task.pct as f64 / 100.0, &pct_fmt)?;
            // Duration formula
            ws.write_formula(r, 6, &format!("E{}-D{}+1", r+1, r+1))?;
            ws.set_cell_format(r, 6, &center_fmt)?;

            // Gantt bar — fill week cells that overlap with task
            let bar_color = colors[phase.color % colors.len()];
            let bar_full = Format::new().background_color(bar_color)
                .border(BorderStyle::Thin).border_color(bar_color);
            let bar_progress = Format::new().background_color(navy)
                .border(BorderStyle::Thin).border_color(navy);

            let progress_end = ts + ((te - ts) as f64 * task.pct as f64 / 100.0) as i64;

            for w in 0..num_weeks {
                let ws_start = proj_start + (w as i64 * 7);
                let ws_end = ws_start + 6;
                let col = (7 + w) as u16;

                if ts <= ws_end && te >= ws_start {
                    // This week overlaps with the task
                    if progress_end >= ws_end && task.pct > 0 {
                        // Fully completed portion
                        ws.write_with_format(r, col, "", &bar_progress)?;
                    } else {
                        ws.write_with_format(r, col, "", &bar_full)?;
                    }
                } else {
                    ws.write_with_format(r, col, "", &empty_week)?;
                }
            }
            ws.set_row_height(r, 20.0)?; r += 1;
        }
    }

    // ── Legend ──
    r += 1;
    let legend_label = Format::new().bold().font_size(9.0).font_color(navy);
    ws.write_with_format(r, 1, "Legend:", &legend_label)?;
    for (i, phase) in phases.iter().enumerate() {
        let swatch = Format::new().background_color(colors[phase.color % colors.len()])
            .border(BorderStyle::Thin);
        ws.write_with_format(r, (2 + i * 2) as u16, "", &swatch)?;
        ws.write_with_format(r, (3 + i * 2) as u16, phase.name, &legend_label)?;
    }
    let progress_swatch = Format::new().background_color(navy).border(BorderStyle::Thin);
    let pi = phases.len();
    ws.write_with_format(r, (2 + pi * 2) as u16, "", &progress_swatch)?;
    ws.write_with_format(r, (3 + pi * 2) as u16, "Completed", &legend_label)?;

    // Print setup
    ws.set_landscape();
    ws.set_fit_to_page(1, 1);
    ws.set_margins(0.3, 0.3, 0.3, 0.3);
    ws.set_header("&CProject Gantt Chart — ERP Implementation");
    ws.set_footer("&CPage &P  |  &D");
    ws.set_freeze_panes(7, 7)?;

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Project Summary (formulas)
    // ═══════════════════════════════════════════════════════════
    let ws2 = wb.add_worksheet_with_name("Summary")?;
    ws2.hide_gridlines();

    let sw = [2.0, 20.0, 14.0, 14.0, 14.0, 14.0, 14.0, 2.0];
    for (c, w) in sw.iter().enumerate() { ws2.set_column_width(c as u16, *w)?; }

    let mut r2 = 0u32;
    ws2.set_row_height(r2, 6.0)?; r2 += 1;
    ws2.write_with_format(r2, 1, "PROJECT SUMMARY", &title_fmt)?;
    ws2.set_row_height(r2, 32.0)?; r2 += 1;
    ws2.write_with_format(r2, 1, "ERP Implementation — Phase Progress", &subtitle_fmt)?;
    r2 += 1;
    for c in 1..=6u16 { ws2.write_with_format(r2, c, "", &divider_fmt)?; }
    ws2.set_row_height(r2, 3.0)?; r2 += 2;

    // Phase summary table
    let sh = ["Phase", "Tasks", "Avg % Done", "Earliest Start", "Latest End", "Status"];
    for (c, h) in sh.iter().enumerate() { ws2.write_with_format(r2, (c+1) as u16, *h, &hdr_fmt)?; }
    ws2.write_with_format(r2, 1, sh[0], &hdr_left)?;
    ws2.set_row_height(r2, 22.0)?; r2 += 1;

    let summary_start = r2;
    for phase in &phases {
        let pf = Format::new().bold().font_size(10.0).font_color(colors[phase.color % colors.len()])
            .align(Align::Left).border(BorderStyle::Thin).border_color(border);
        let nf = Format::new().font_size(10.0).font_color(navy)
            .align(Align::Center).border(BorderStyle::Thin).border_color(border);
        let df = Format::new().font_size(10.0).font_color(navy).num_format("m/d/yy")
            .align(Align::Center).border(BorderStyle::Thin).border_color(border);
        let pctf = Format::new().font_size(10.0).font_color(navy).num_format("0%")
            .align(Align::Center).border(BorderStyle::Thin).border_color(border);

        let avg_pct: f64 = phase.tasks.iter().map(|t| t.pct as f64).sum::<f64>() / phase.tasks.len() as f64 / 100.0;
        let earliest = phase.tasks.iter().map(|t| serial(t.start.0, t.start.1, t.start.2)).min().unwrap();
        let latest = phase.tasks.iter().map(|t| serial(t.end.0, t.end.1, t.end.2)).max().unwrap();
        let status = if avg_pct >= 1.0 { "✅ Complete" } else if avg_pct > 0.0 { "🔄 In Progress" } else { "⏳ Not Started" };

        ws2.write_with_format(r2, 1, phase.name, &pf)?;
        ws2.write_with_format(r2, 2, phase.tasks.len() as f64, &nf)?;
        ws2.write_with_format(r2, 3, avg_pct, &pctf)?;
        ws2.write_with_format(r2, 4, earliest as f64, &df)?;
        ws2.write_with_format(r2, 5, latest as f64, &df)?;
        ws2.write_with_format(r2, 6, status, &nf)?;
        ws2.set_row_height(r2, 24.0)?;
        r2 += 1;
    }

    // Data bars on Avg % Done
    ws2.add_conditional_format(summary_start, 3, r2 - 1, 3, ConditionalFormatDataBar::new("#70AD47"))?;

    r2 += 1;

    // Chart: phase progress
    let mut chart = Chart::new(ChartType::Bar);
    chart.set_title("Phase Completion");
    chart.set_width(520); chart.set_height(280);
    chart.set_legend_position(LegendPosition::None);
    let s = chart.add_series();
    s.set_values(&format!("Summary!$D${}:$D${}", summary_start + 1, summary_start + phases.len() as u32));
    s.set_categories(&format!("Summary!$B${}:$B${}", summary_start + 1, summary_start + phases.len() as u32));
    s.set_name("% Complete");
    s.set_data_labels(true);
    s.set_color(accent);
    ws2.insert_chart(r2, 1, &chart)?;

    ws2.set_landscape();
    ws2.set_fit_to_page(1, 1);

    wb.set_active_sheet(0);
    let path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into()))
        .join("Downloads/gantt_chart.xlsx");
    wb.save(&path)?;
    println!("✅ Gantt Chart saved to {}", path.display());
    Ok(())
}

struct Phase { name: &'static str, color: usize, tasks: Vec<Task> }
struct Task { name: &'static str, start: (i32,u32,u32), end: (i32,u32,u32), pct: u32, owner: &'static str }

fn serial(y: i32, m: u32, d: u32) -> i64 {
    // Excel serial date (1900 epoch)
    let month_days: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let mut days = (y as i64 - 1900) * 365 + ((y as i64 - 1900 - 1) / 4) + 1;
    days += month_days[(m - 1) as usize] + d as i64;
    if y > 1900 || (y == 1900 && m > 2) { days += 1; } // Lotus 1-2-3 bug
    days
}
