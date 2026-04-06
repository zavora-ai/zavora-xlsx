use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    let orange = "#E8611A"; let dark = "#333333"; let gray = "#7F8C8D";
    let green_bar = "#27AE60"; let blue_bar = "#2980B9"; let amber_bar = "#F39C12";
    let red_bar = "#E74C3C"; let purple_bar = "#8E44AD"; let teal_bar = "#1ABC9C";
    let border = "#D5D8DC"; let light = "#F8F9FA";

    let ws = wb.worksheet(0)?;
    ws.set_name("Home Budget")?;
    ws.hide_gridlines();

    // Column widths: A(margin) B-D(earnings) E(gap) F-H(everyday) I-K(living) L-N(regular)
    let cw = [2.0, 22.0, 10.0, 10.0, 3.0, 22.0, 10.0, 10.0, 22.0, 10.0, 10.0, 22.0, 10.0, 10.0];
    for (c, w) in cw.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }

    // ── Styles ──
    let title = Format::new().bold().font_size(22.0).font_color("#FFFFFF")
        .background_color(orange).align(Align::Left).align(Align::VerticalCenter);
    let title_bar = Format::new().background_color(orange);

    let section_hdr = |color: &str| -> Format {
        Format::new().bold().font_size(12.0).font_color(dark).align(Align::Left)
            .align(Align::Bottom)
    };
    let section_total = Format::new().bold().font_size(12.0).font_color(dark)
        .align(Align::Right).num_format("$#,##0");
    let section_freq = Format::new().font_size(10.0).font_color(orange).italic().align(Align::Left);
    let color_bar = |color: &str| -> Format { Format::new().background_color(color) };

    let item = Format::new().font_size(10.0).font_color(dark).align(Align::Left)
        .border(BorderStyle::Thin).border_color(border);
    let amount = Format::new().font_size(10.0).font_color(dark).align(Align::Right)
        .num_format("#,##0").border(BorderStyle::Thin).border_color(border);
    let freq = Format::new().font_size(9.0).font_color(gray).italic().align(Align::Left)
        .border(BorderStyle::Thin).border_color(border);
    let empty_row = Format::new().border(BorderStyle::Thin).border_color(border);

    let balance_label = Format::new().bold().font_size(11.0).font_color(dark)
        .background_color("#E8E8E8").align(Align::Left).border(BorderStyle::Thin).border_color(border);
    let balance_val = Format::new().bold().font_size(11.0).font_color(dark)
        .background_color("#E8E8E8").align(Align::Right).num_format("$#,##0")
        .border(BorderStyle::Thin).border_color(border);
    let balance_freq = Format::new().bold().font_size(10.0).font_color(orange)
        .background_color("#E8E8E8").italic().align(Align::Left)
        .border(BorderStyle::Thin).border_color(border);

    let mut r = 0u32;

    // ── Title ──
    for c in 0..=13u16 { ws.write_with_format(r, c, "", &title_bar)?; }
    ws.write_with_format(r, 1, "Simple Home Budget", &title)?;
    ws.set_row_height(r, 36.0)?; r += 2;

    // ════════════════════════════════════════════════════════════
    // LEFT COLUMN: Earnings & Savings (B-D)
    // ════════════════════════════════════════════════════════════
    let earn_row = r;

    // Section: Your Earnings & Savings
    ws.write_with_format(r, 1, "Your Earnings & Savings", &Format::new().bold().font_size(11.0)
        .font_color("#FFFFFF").background_color("#5D6D7E").align(Align::Left))?;
    ws.write_with_format(r, 2, "", &Format::new().background_color("#5D6D7E"))?;
    ws.write_with_format(r, 3, "", &Format::new().background_color("#5D6D7E"))?;
    ws.set_row_height(r, 22.0)?; r += 1;

    ws.write_with_format(r, 1, "Income after taxes", &Format::new().bold().font_size(11.0).font_color(dark)
        .border(BorderStyle::Thin).border_color(border))?;
    ws.write_with_format(r, 2, 8000.0, &Format::new().bold().font_size(11.0).font_color(dark)
        .num_format("$#,##0").align(Align::Right).border(BorderStyle::Thin).border_color(border))?;
    ws.write_with_format(r, 3, "Monthly", &freq)?;
    r += 2;

    // Savings section
    ws.write_with_format(r, 1, "Savings", &Format::new().bold().font_size(11.0).font_color(dark)
        .background_color(light).border(BorderStyle::Thin).border_color(border))?;
    // Savings total = formula summing savings items
    ws.write_formula(r, 2, &format!("SUM(C{}:C{})", r + 3, r + 7))?;
    ws.set_cell_format(r, 2, &Format::new().bold().font_size(11.0).font_color(dark)
        .num_format("$#,##0").align(Align::Right).background_color(light)
        .border(BorderStyle::Thin).border_color(border))?;
    ws.write_with_format(r, 3, "Monthly", &Format::new().font_size(10.0).font_color(orange).italic()
        .background_color(light).border(BorderStyle::Thin).border_color(border))?;
    r += 1;

    let savings_start = r;
    let savings: Vec<(&str, f64, &str)> = vec![
        ("Retirement Savings", 1500.0, "Monthly"),
        ("Long-term Savings", 100.0, "Monthly"),
        ("Emergency Funds", 50.0, "Monthly"),
        ("Kids / Education Savings", 150.0, "Monthly"),
        ("Travel Savings", 0.0, ""),
        ("Other Savings", 0.0, ""),
    ];
    for (name, val, f) in &savings {
        ws.write_with_format(r, 1, *name, &item)?;
        if *val > 0.0 { ws.write_with_format(r, 2, *val, &amount)?; }
        else { ws.write_with_format(r, 2, "", &empty_row)?; }
        ws.write_with_format(r, 3, *f, &freq)?;
        r += 1;
    }

    r += 1;
    // Balance = Income - Savings - All Expenses
    let balance_row = r;
    ws.write_with_format(r, 1, "BALANCE", &balance_label)?;
    // Will set formula after we know expense totals
    ws.write_with_format(r, 3, "Monthly", &balance_freq)?;

    // ════════════════════════════════════════════════════════════
    // EXPENSE CATEGORIES (columns F-N)
    // ════════════════════════════════════════════════════════════
    let exp_start = earn_row;

    // Helper to write a category block
    let categories: Vec<Cat> = vec![
        Cat { name: "Everyday", col: 5, color: green_bar, items: vec![
            ("Public Transport", 16.0, "Weekly"), ("Fuel", 50.0, "Monthly"),
            ("Bought Lunches", 10.0, "Fortnightly"), ("Groceries", 150.0, "Weekly"),
            ("Takeaways", 30.0, "Monthly"), ("Dining Out", 70.0, "Monthly"),
            ("Entertainment", 60.0, "Monthly"),
        ]},
        Cat { name: "Living", col: 8, color: blue_bar, items: vec![
            ("Mortgage", 1380.0, "Fortnightly"), ("Power", 50.0, "Monthly"),
            ("Gas / Wood", 50.0, "Monthly"), ("Water", 0.0, "Monthly"),
            ("Internet", 90.0, "Monthly"), ("Cell phone", 50.0, "Monthly"),
            ("Property Tax", 3800.0, "Yearly"), ("Garden Maintenance", 20.0, "Monthly"),
            ("House Maintenance", 50.0, "Monthly"), ("Cleaning", 10.0, "Monthly"),
            ("Laundry", 5.0, "Monthly"),
        ]},
        Cat { name: "Regular", col: 11, color: red_bar, items: vec![
            ("Child Support", 0.0, ""), ("Elder Care", 0.0, ""),
            ("Paid TV", 12.0, "Monthly"), ("Bank Fees", 20.0, "Yearly"),
            ("Subscriptions / Magazines", 8.0, "Monthly"),
            ("Clubs & Memberships", 25.0, "Monthly"),
            ("Personal loans", 0.0, ""), ("Credit card debt", 0.0, ""),
            ("Student loans", 0.0, ""),
        ]},
    ];

    let mut cat_total_cells: Vec<String> = Vec::new();

    for cat in &categories {
        let c = cat.col as u16;
        let mut cr = exp_start;

        // Header: "Your Expenditure" label on first category
        if cat.col == 5 {
            ws.write_with_format(cr, c, "Your Expenditure", &Format::new().bold().font_size(11.0)
                .font_color("#5D6D7E"))?;
        }
        cr += 1;

        // Category name + total + color bar
        ws.write_with_format(cr, c, cat.name, &Format::new().bold().font_size(12.0).font_color(dark))?;
        let total_cell = format!("{}{}",col_letter(c + 1), cr + 1);
        // Total = SUM of items (monthly equivalent)
        let items_start = cr + 3;
        let items_end = items_start + cat.items.len() as u32 - 1;
        ws.write_formula(cr, c + 1, &format!("SUM({}{}:{}{})", col_letter(c+1), items_start+1, col_letter(c+1), items_end+1))?;
        ws.set_cell_format(cr, c + 1, &section_total)?;
        ws.write_with_format(cr, c + 2, "Monthly", &section_freq)?;
        cat_total_cells.push(total_cell);
        cr += 1;

        // Color bar
        for ci in 0..3u16 { ws.write_with_format(cr, c + ci, "", &color_bar(cat.color))?; }
        ws.set_row_height(cr, 3.0)?;
        cr += 1;

        // Items
        for (name, val, f) in &cat.items {
            ws.write_with_format(cr, c, *name, &item)?;
            if *val > 0.0 { ws.write_with_format(cr, c + 1, *val, &amount)?; }
            else { ws.write_with_format(cr, c + 1, "", &empty_row)?; }
            ws.write_with_format(cr, c + 2, *f, &freq)?;
            cr += 1;
        }
        // Empty rows for user to add more
        for _ in 0..3 {
            ws.write_with_format(cr, c, "", &empty_row)?;
            ws.write_with_format(cr, c + 1, "", &empty_row)?;
            ws.write_with_format(cr, c + 2, "", &empty_row)?;
            cr += 1;
        }
    }

    // Second row of categories
    let row2_start = exp_start + 20;
    let categories2: Vec<Cat> = vec![
        Cat { name: "Irregular", col: 5, color: amber_bar, items: vec![
            ("Home Insurance", 1200.0, "Yearly"), ("Body corp. fees", 0.0, ""),
            ("Vehicle insurance", 9.0, "Monthly"), ("Vehicle maintenance", 400.0, "Yearly"),
            ("Registration / licensing", 200.0, "Yearly"), ("Warrant of Fitness", 60.0, "Yearly"),
            ("Breakdown cover", 0.0, ""), ("Contents insurance", 0.0, ""),
            ("Doctors", 50.0, "Monthly"), ("Health products", 20.0, "Monthly"),
            ("Gifts / Christmas", 30.0, "Monthly"), ("Vacation", 4000.0, "Yearly"),
        ]},
        Cat { name: "Personal", col: 8, color: purple_bar, items: vec![
            ("Personal cash", 25.0, "Monthly"), ("Clothing / shoes", 200.0, "Yearly"),
            ("Hair & beauty", 20.0, "Monthly"), ("Charity", 20.0, "Monthly"),
            ("Beverages", 20.0, "Monthly"),
        ]},
        Cat { name: "Kids", col: 11, color: teal_bar, items: vec![
            ("After school care", 0.0, ""), ("Pocket money", 30.0, "Monthly"),
            ("School trips", 0.0, ""), ("School fees", 500.0, "Yearly"),
            ("Uniforms", 250.0, "Yearly"), ("Activities / learning", 500.0, "Yearly"),
            ("Parties / gifts", 250.0, "Yearly"),
        ]},
    ];

    for cat in &categories2 {
        let c = cat.col as u16;
        let mut cr = row2_start;

        ws.write_with_format(cr, c, cat.name, &Format::new().bold().font_size(12.0).font_color(dark))?;
        let items_start = cr + 3;
        let items_end = items_start + cat.items.len() as u32 - 1;
        ws.write_formula(cr, c + 1, &format!("SUM({}{}:{}{})", col_letter(c+1), items_start+1, col_letter(c+1), items_end+1))?;
        ws.set_cell_format(cr, c + 1, &section_total)?;
        ws.write_with_format(cr, c + 2, "Monthly", &section_freq)?;
        let total_cell = format!("{}{}", col_letter(c+1), cr + 1);
        cat_total_cells.push(total_cell);
        cr += 1;

        for ci in 0..3u16 { ws.write_with_format(cr, c + ci, "", &color_bar(cat.color))?; }
        ws.set_row_height(cr, 3.0)?;
        cr += 1;

        for (name, val, f) in &cat.items {
            ws.write_with_format(cr, c, *name, &item)?;
            if *val > 0.0 { ws.write_with_format(cr, c + 1, *val, &amount)?; }
            else { ws.write_with_format(cr, c + 1, "", &empty_row)?; }
            ws.write_with_format(cr, c + 2, *f, &freq)?;
            cr += 1;
        }
    }

    // Balance formula: Income - Savings - All expense totals
    let savings_total = format!("C{}", earn_row + 4); // savings total cell
    let income_cell = format!("C{}", earn_row + 2);
    let expense_sum = cat_total_cells.join("+");
    ws.write_formula(balance_row, 2, &format!("{income_cell}-{savings_total}-{expense_sum}"))?;
    ws.set_cell_format(balance_row, 2, &balance_val)?;

    // ── Chart: Where your money goes ──
    let chart_row = balance_row + 2;
    ws.write_with_format(chart_row, 1, "Where your money goes...", &Format::new().font_size(14.0)
        .font_color(dark).italic())?;

    // Write chart source data in a hidden area
    let src = 50u32;
    let all_cats = ["Savings","Everyday","Living","Regular","Irregular","Personal","Kids"];
    let all_colors = ["#2ECC71", green_bar, blue_bar, red_bar, amber_bar, purple_bar, teal_bar];
    for (i, name) in all_cats.iter().enumerate() {
        ws.write(src + i as u32, 1, *name)?;
    }
    // Savings amount
    ws.write_formula(src, 2, &format!("{savings_total}"))?;
    // Expense category amounts
    for (i, tc) in cat_total_cells.iter().enumerate() {
        ws.write_formula(src + 1 + i as u32, 2, tc)?;
    }

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Monthly Spending Breakdown");
    chart.set_width(560); chart.set_height(320);
    chart.set_y_axis_name("$ per month");
    chart.set_legend_position(LegendPosition::Bottom);
    let s = chart.add_series();
    s.set_values(&format!("'Home Budget'!$C${}:$C${}", src + 1, src + all_cats.len() as u32));
    s.set_categories(&format!("'Home Budget'!$B${}:$B${}", src + 1, src + all_cats.len() as u32));
    s.set_name("Monthly Amount");
    s.set_data_labels(true);
    for (i, c) in all_colors.iter().enumerate() { s.set_point_color(i, *c); }
    ws.insert_chart(chart_row + 1, 1, &chart)?;

    // Print
    ws.set_landscape(); ws.set_fit_to_page(1, 1);
    ws.set_margins(0.3, 0.3, 0.3, 0.3);
    ws.set_header("&CSimple Home Budget");
    ws.set_footer("&CPage &P  |  &D");

    wb.set_active_sheet(0);
    let path = home("personal_budget.xlsx");
    wb.save(&path)?;
    println!("✅ Personal Budget saved to {}", path.display());
    Ok(())
}

struct Cat { name: &'static str, col: usize, color: &'static str, items: Vec<(&'static str, f64, &'static str)> }

fn col_letter(col: u16) -> String { zavora_xlsx::utility::col_to_letter(col) }

fn home(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(name)
}
