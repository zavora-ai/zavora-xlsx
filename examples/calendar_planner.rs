use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    // ── Events & Important Dates ──
    let events: Vec<(u32, u32, u32, &str, &str)> = vec![
        // (month, day, 0=event/1=anniversary/2=important, label, color_hint)
        (1, 1, 2, "New Year's Day", "holiday"),
        (1, 20, 2, "MLK Jr. Day", "holiday"),
        (2, 14, 1, "Valentine's Day", "anniversary"),
        (3, 17, 0, "St. Patrick's Day", "event"),
        (4, 20, 2, "Easter Sunday", "holiday"),
        (5, 11, 1, "Mother's Day", "anniversary"),
        (5, 26, 2, "Memorial Day", "holiday"),
        (6, 15, 1, "Wedding Anniversary", "anniversary"),
        (6, 19, 2, "Juneteenth", "holiday"),
        (7, 4, 2, "Independence Day", "holiday"),
        (9, 1, 2, "Labor Day", "holiday"),
        (10, 31, 0, "Halloween", "event"),
        (11, 27, 2, "Thanksgiving", "holiday"),
        (12, 25, 2, "Christmas Day", "holiday"),
        (12, 31, 0, "New Year's Eve", "event"),
        // Birthdays
        (3, 8, 1, "Mom's Birthday 🎂", "anniversary"),
        (7, 22, 1, "Dad's Birthday 🎂", "anniversary"),
        (9, 15, 1, "Partner's Birthday 🎂", "anniversary"),
        // Work deadlines
        (1, 15, 0, "Q4 Report Due", "event"),
        (4, 15, 2, "Tax Day", "important"),
        (6, 30, 0, "Mid-Year Review", "event"),
        (10, 15, 0, "Q3 Report Due", "event"),
    ];

    let months = ["January", "February", "March", "April", "May", "June",
                   "July", "August", "September", "October", "November", "December"];
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]; // 2025 not leap

    // ── Styles ──
    let title_fmt = Format::new()
        .bold().font_size(20.0).font_color(NamedColor::White)
        .background_color("#2B579A").align(Align::Center).align(Align::VerticalCenter);
    let month_fmt = Format::new()
        .bold().font_size(16.0).font_color(NamedColor::White)
        .background_color("#2B579A").align(Align::Center).align(Align::VerticalCenter);
    let day_header_fmt = Format::new()
        .bold().font_size(11.0).font_color(NamedColor::White)
        .background_color("#4472C4").align(Align::Center)
        .border(BorderStyle::Thin).border_color("#2B579A");
    let day_cell_fmt = Format::new()
        .font_size(10.0).align(Align::Right).align(Align::Top)
        .border(BorderStyle::Thin).border_color("#D6DCE4");
    let today_fmt = Format::new()
        .font_size(10.0).bold().align(Align::Right).align(Align::Top)
        .background_color("#E2EFDA")
        .border(BorderStyle::Thin).border_color("#A9D18E");
    let weekend_fmt = Format::new()
        .font_size(10.0).align(Align::Right).align(Align::Top)
        .background_color("#F2F2F2")
        .border(BorderStyle::Thin).border_color("#D6DCE4");
    let empty_cell_fmt = Format::new()
        .background_color("#F8F8F8")
        .border(BorderStyle::Thin).border_color("#E8E8E8");
    let event_fmt = Format::new()
        .font_size(8.0).font_color("#2B579A").italic()
        .align(Align::Left).align(Align::Top)
        .border(BorderStyle::Thin).border_color("#D6DCE4");
    let holiday_fmt = Format::new()
        .font_size(10.0).bold().font_color("#C00000").align(Align::Right).align(Align::Top)
        .background_color("#FCE4EC")
        .border(BorderStyle::Thin).border_color("#E57373");
    let anniversary_fmt = Format::new()
        .font_size(10.0).bold().font_color("#7B1FA2").align(Align::Right).align(Align::Top)
        .background_color("#F3E5F5")
        .border(BorderStyle::Thin).border_color("#CE93D8");

    // Planner styles
    let planner_header_fmt = Format::new()
        .bold().font_size(11.0).font_color(NamedColor::White)
        .background_color("#2B579A").align(Align::Center)
        .border(BorderStyle::Thin);
    let time_fmt = Format::new()
        .font_size(10.0).bold().align(Align::Center).align(Align::VerticalCenter)
        .background_color("#D9E2F3").border(BorderStyle::Thin).border_color("#B4C6E7");
    let slot_fmt = Format::new()
        .font_size(10.0).align(Align::Left).align(Align::VerticalCenter)
        .border(BorderStyle::Thin).border_color("#D6DCE4");
    let legend_label = Format::new().font_size(9.0).bold().align(Align::Left);
    let legend_holiday = Format::new().font_size(9.0).background_color("#FCE4EC").border(BorderStyle::Thin);
    let legend_anniv = Format::new().font_size(9.0).background_color("#F3E5F5").border(BorderStyle::Thin);
    let legend_event = Format::new().font_size(9.0).font_color("#2B579A").italic().border(BorderStyle::Thin);

    // ── Sheet 1: Year-at-a-Glance Calendar ──
    let ws = wb.worksheet(0)?;
    ws.set_name("2025 Calendar")?;

    // Title row
    ws.merge_range(0, 0, 0, 6, "2025 CALENDAR & PLANNER", &title_fmt)?;
    ws.set_row_height(0, 40.0)?;

    // Legend
    ws.write_with_format(1, 0, "Legend:", &legend_label)?;
    ws.write_with_format(1, 1, " Holiday ", &legend_holiday)?;
    ws.write_with_format(1, 2, " Anniversary ", &legend_anniv)?;
    ws.write_with_format(1, 3, " Event ", &legend_event)?;
    ws.set_row_height(1, 20.0)?;

    // Column widths for calendar grid
    for c in 0..7u16 { ws.set_column_width(c, 16.0)?; }

    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let mut row = 3u32;

    for m in 0..12 {
        // Month header
        ws.merge_range(row, 0, row, 6, months[m], &month_fmt)?;
        ws.set_row_height(row, 28.0)?;
        row += 1;

        // Day headers
        for (c, name) in day_names.iter().enumerate() {
            ws.write_with_format(row, c as u16, *name, &day_header_fmt)?;
        }
        ws.set_row_height(row, 22.0)?;
        row += 1;

        // Calculate first day of month (Zeller-like for 2025)
        let first_dow = day_of_week(2025, (m + 1) as u32, 1); // 0=Mon
        let num_days = days_in_month[m];

        let mut day = 1u32;
        let mut col = first_dow as u16;

        // Fill empty cells before first day
        for c in 0..col { ws.write_with_format(row, c, "", &empty_cell_fmt)?; }

        while day <= num_days {
            if col == 0 && day > 1 { row += 1; }

            // Check for events on this day
            let month_events: Vec<&str> = events.iter()
                .filter(|e| e.0 == (m as u32 + 1) && e.1 == day)
                .map(|e| e.3)
                .collect();
            let event_type = events.iter()
                .find(|e| e.0 == (m as u32 + 1) && e.1 == day)
                .map(|e| e.4);

            let cell_text = if month_events.is_empty() {
                format!("{day}")
            } else {
                format!("{day}\n{}", month_events.join("\n"))
            };

            let fmt = match event_type {
                Some("holiday") | Some("important") => &holiday_fmt,
                Some("anniversary") => &anniversary_fmt,
                Some("event") => &event_fmt,
                _ if col >= 5 => &weekend_fmt,
                _ => &day_cell_fmt,
            };

            // Use event format even for weekends if there's an event
            ws.write_with_format(row, col, cell_text.as_str(), fmt)?;
            ws.set_row_height(row, if month_events.is_empty() { 32.0 } else { 48.0 })?;

            day += 1;
            col += 1;
            if col > 6 { col = 0; }
        }

        // Fill remaining cells in last week
        while col > 0 && col <= 6 {
            ws.write_with_format(row, col, "", &empty_cell_fmt)?;
            col += 1;
        }

        row += 2; // gap between months
    }

    // Print setup
    ws.set_portrait();
    ws.set_paper_size(1);
    ws.set_margins(0.5, 0.5, 0.5, 0.5);
    ws.set_fit_to_page(1, 0);
    ws.set_header("&C&\"Calibri,Bold\"&122025 Calendar");
    ws.set_footer("&CPage &P of &N");

    // ── Sheet 2: Daily Planner ──
    let ws2 = wb.add_worksheet_with_name("Daily Planner")?;

    for c in 0..8u16 { ws2.set_column_width(c, if c == 0 { 12.0 } else { 18.0 })?; }

    // Title
    ws2.merge_range(0, 0, 0, 7, "DAILY PLANNER — 2025", &title_fmt)?;
    ws2.set_row_height(0, 36.0)?;

    // Headers: Time | Mon | Tue | Wed | Thu | Fri | Sat | Sun
    ws2.write_with_format(1, 0, "Time", &planner_header_fmt)?;
    for (c, name) in day_names.iter().enumerate() {
        ws2.write_with_format(1, (c + 1) as u16, *name, &planner_header_fmt)?;
    }
    ws2.set_row_height(1, 24.0)?;

    // Time slots: 6:00 AM to 9:00 PM
    let times = [
        "6:00 AM", "7:00 AM", "8:00 AM", "9:00 AM", "10:00 AM", "11:00 AM",
        "12:00 PM", "1:00 PM", "2:00 PM", "3:00 PM", "4:00 PM", "5:00 PM",
        "6:00 PM", "7:00 PM", "8:00 PM", "9:00 PM",
    ];

    for (i, time) in times.iter().enumerate() {
        let r = (i as u32) + 2;
        ws2.write_with_format(r, 0, *time, &time_fmt)?;
        for c in 1..=7u16 {
            ws2.write_with_format(r, c, "", &slot_fmt)?;
        }
        ws2.set_row_height(r, 28.0)?;
    }

    // Notes section
    let notes_row = times.len() as u32 + 3;
    ws2.merge_range(notes_row, 0, notes_row, 7, "NOTES & REMINDERS", &month_fmt)?;
    ws2.set_row_height(notes_row, 28.0)?;
    for r in 1..=6u32 {
        let nr = notes_row + r;
        ws2.merge_range(nr, 0, nr, 7, "", &slot_fmt)?;
        ws2.set_row_height(nr, 24.0)?;
    }

    // Print setup
    ws2.set_landscape();
    ws2.set_paper_size(1);
    ws2.set_margins(0.5, 0.5, 0.5, 0.5);
    ws2.set_fit_to_page(1, 1);
    ws2.set_header("&C&\"Calibri,Bold\"&12Daily Planner");

    // ── Sheet 3: Important Dates Summary ──
    let ws3 = wb.add_worksheet_with_name("Important Dates")?;

    ws3.set_column_width(0, 14.0)?;
    ws3.set_column_width(1, 30.0)?;
    ws3.set_column_width(2, 16.0)?;

    ws3.merge_range(0, 0, 0, 2, "IMPORTANT DATES — 2025", &title_fmt)?;
    ws3.set_row_height(0, 36.0)?;

    let col_headers = ["Date", "Event", "Type"];
    for (c, h) in col_headers.iter().enumerate() {
        ws3.write_with_format(1, c as u16, *h, &planner_header_fmt)?;
    }

    let date_fmt_cell = Format::new().num_format("mmmm d, yyyy").align(Align::Left)
        .border(BorderStyle::Thin).border_color("#D6DCE4");
    let event_name_fmt = Format::new().font_size(10.0).align(Align::Left)
        .border(BorderStyle::Thin).border_color("#D6DCE4");
    let type_holiday = Format::new().font_size(10.0).bold().font_color("#C00000")
        .background_color("#FCE4EC").align(Align::Center)
        .border(BorderStyle::Thin).border_color("#D6DCE4");
    let type_anniv = Format::new().font_size(10.0).bold().font_color("#7B1FA2")
        .background_color("#F3E5F5").align(Align::Center)
        .border(BorderStyle::Thin).border_color("#D6DCE4");
    let type_event = Format::new().font_size(10.0).italic().font_color("#2B579A")
        .align(Align::Center).border(BorderStyle::Thin).border_color("#D6DCE4");

    let mut sorted_events = events.clone();
    sorted_events.sort_by_key(|e| (e.0, e.1));

    for (i, ev) in sorted_events.iter().enumerate() {
        let r = (i as u32) + 2;
        let dt = ExcelDateTime::from_ymd(2025, ev.0, ev.1).unwrap();
        ws3.write_with_format(r, 0, dt, &date_fmt_cell)?;
        ws3.write_with_format(r, 1, ev.3, &event_name_fmt)?;
        let (type_label, type_f) = match ev.4 {
            "holiday" | "important" => ("Holiday", &type_holiday),
            "anniversary" => ("Anniversary", &type_anniv),
            _ => ("Event", &type_event),
        };
        ws3.write_with_format(r, 2, type_label, type_f)?;
    }

    // Table for the dates list
    let last_row = sorted_events.len() as u32 + 1;
    ws3.add_table(1, 0, last_row, 2, &Table::new()
        .set_style(TableStyle::Medium(2))
        .set_columns(&[TableColumn::new("Date"), TableColumn::new("Event"), TableColumn::new("Type")]))?;

    ws3.set_landscape();
    ws3.set_fit_to_page(1, 1);
    ws3.set_header("&CImportant Dates 2025");
    ws3.set_footer("&CPage &P of &N");

    // ── Save ──
    wb.set_active_sheet(0);
    let path = std::env::temp_dir().join("2025_calendar_planner.xlsx");
    wb.save(&path)?;
    println!("✅ Calendar saved to {}", path.display());

    // Copy to Downloads
    let dest = dirs_or_home().join("2025_calendar_planner.xlsx");
    std::fs::copy(&path, &dest).ok();
    println!("📋 Copied to {}", dest.display());

    Ok(())
}

/// Day of week for a given date (0=Monday, 6=Sunday). Tomohiko Sakamoto's algorithm.
fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    let dow = (y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day as i32) % 7;
    // Convert: 0=Sun → 6, 1=Mon → 0, etc.
    ((dow + 6) % 7) as u32
}

fn dirs_or_home() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())).join("Downloads")
}
