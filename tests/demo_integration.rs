use zavora_xlsx::*;
use std::path::Path;

fn dir() -> std::path::PathBuf {
    let d = std::env::temp_dir().join("zavora_integration");
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn test_create(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "Product")?;
    ws.write(0, 1, "Price")?;
    ws.write(0, 2, "Qty")?;
    ws.write(1, 0, "Widget")?;
    ws.write(1, 1, 9.99)?;
    ws.write(1, 2, 100)?;
    ws.write(2, 0, "Gadget")?;
    ws.write(2, 1, 24.50)?;
    ws.write(2, 2, 50)?;
    wb.save(path)?;
    assert!(path.exists());
    Ok(())
}

fn test_read_back(path: &Path) -> Result<()> {
    let mut wb = Workbook::open_readonly(path)?;
    let ws = wb.worksheet(0)?;
    assert_eq!(ws.read_cell(0, 0), CellValue::String("Product".into()));
    assert_eq!(ws.read_cell(1, 1), CellValue::Number(9.99));
    assert_eq!(ws.read_cell(2, 2), CellValue::Number(50.0));
    assert_eq!(ws.read_cell(99, 99), CellValue::Empty);
    Ok(())
}

fn test_data_types(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "string")?;
    ws.write(1, 0, 42.5_f64)?;
    ws.write(2, 0, -7_i32)?;
    ws.write(3, 0, true)?;
    ws.write(4, 0, false)?;
    ws.write(5, 0, ExcelDateTime::from_ymd(2024, 6, 15).unwrap())?;
    ws.write_formula(6, 0, "1+2")?;
    ws.write(7, 0, "")?; // empty string
    wb.save(path)?;
    Ok(())
}

fn test_read_types(path: &Path) -> Result<()> {
    let mut wb = Workbook::open_readonly(path)?;
    let ws = wb.worksheet(0)?;
    assert_eq!(ws.read_cell(0, 0), CellValue::String("string".into()));
    assert_eq!(ws.read_cell(1, 0), CellValue::Number(42.5));
    assert_eq!(ws.read_cell(2, 0), CellValue::Number(-7.0));
    assert_eq!(ws.read_cell(3, 0), CellValue::Bool(true));
    assert_eq!(ws.read_cell(4, 0), CellValue::Bool(false));
    match ws.read_cell(5, 0) {
        CellValue::DateTime(dt) => {
            let (y, m, d, _, _, _) = dt.to_ymd_hms();
            assert_eq!((y, m, d), (2024, 6, 15));
        }
        other => panic!("Expected DateTime, got {:?}", other),
    }
    match ws.read_cell(6, 0) {
        CellValue::Formula { formula, .. } => assert_eq!(formula, "1+2"),
        other => panic!("Expected Formula, got {:?}", other),
    }
    Ok(())
}

fn test_formatting(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Combined header format: bold + size + color + background + border
    let header = Format::new()
        .bold()
        .font_size(14.0)
        .background_color("#4472C4")
        .font_color("#FFFFFF")
        .border(BorderStyle::Thin)
        .border_color("#000000");
    let currency = Format::new().num_format("$#,##0.00").border(BorderStyle::Thin).border_color("#000000");
    let percent = Format::new().num_format("0.0%").border(BorderStyle::Thin).border_color("#000000");
    let border_fmt = Format::new().border(BorderStyle::Thin).border_color("#000000");

    ws.write_with_format(0, 0, "Item", &header)?;
    ws.write_with_format(0, 1, "Revenue", &header)?;
    ws.write_with_format(0, 2, "Margin", &header)?;

    ws.write_with_format(1, 0, "Product A", &border_fmt)?;
    ws.write_with_format(1, 1, 15000.0, &currency)?;
    ws.write_with_format(1, 2, 0.235, &percent)?;

    ws.write_with_format(2, 0, "Product B", &border_fmt)?;
    ws.write_with_format(2, 1, 8500.0, &currency)?;
    ws.write_with_format(2, 2, 0.18, &percent)?;

    let merge_fmt = Format::new().bold().align(Align::Center);
    ws.merge_range(4, 0, 4, 2, "Summary Section", &merge_fmt)?;

    wb.save(path)?;

    // Verify it reads back
    let mut wb2 = Workbook::open_readonly(path)?;
    let ws2 = wb2.worksheet(0)?;
    assert_eq!(ws2.read_cell(0, 0), CellValue::String("Item".into()));
    assert_eq!(ws2.read_cell(1, 1), CellValue::Number(15000.0));
    Ok(())
}

fn test_multi_sheet(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    wb.worksheet(0)?.set_name("Sales")?;
    wb.worksheet(0)?.write_row(0, 0, ["Region", "Q1", "Q2"])?;
    wb.worksheet(0)?.write_row(1, 0, ["North", "East", "West"])?;

    let ws2 = wb.add_worksheet_with_name("Costs")?;
    ws2.write_row(0, 0, ["Category", "Amount"])?;
    ws2.write(1, 0, "Rent")?;
    ws2.write(1, 1, 5000.0)?;

    let ws3 = wb.add_worksheet_with_name("Notes")?;
    ws3.write(0, 0, "This is a note")?;

    assert_eq!(wb.sheet_count(), 3);
    assert_eq!(wb.sheet_names(), vec!["Sales", "Costs", "Notes"]);
    wb.save(path)?;
    Ok(())
}

fn test_read_multi_sheet(path: &Path) -> Result<()> {
    let mut wb = Workbook::open_readonly(path)?;
    assert_eq!(wb.sheet_names(), vec!["Sales", "Costs", "Notes"]);
    assert_eq!(wb.worksheet(0)?.read_cell(0, 0), CellValue::String("Region".into()));
    assert_eq!(wb.worksheet(1)?.read_cell(1, 1), CellValue::Number(5000.0));
    assert_eq!(wb.worksheet(2)?.read_cell(0, 0), CellValue::String("This is a note".into()));
    Ok(())
}

fn test_edit_mode(path: &Path) -> Result<()> {
    // Create original
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0)?;
        ws.write(0, 0, "Original")?;
        ws.write(1, 0, "Keep this")?;
        ws.write(2, 0, 999.0)?;
        wb.save(path)?;
    }

    // Open, edit, save
    {
        let mut wb = Workbook::open(path)?;
        let ws = wb.worksheet(0)?;

        // Verify we can read existing data
        assert_eq!(ws.read_cell(0, 0), CellValue::String("Original".into()));
        assert_eq!(ws.read_cell(1, 0), CellValue::String("Keep this".into()));

        // Modify
        ws.write(0, 0, "Modified")?;
        ws.write(3, 0, "New row")?;

        wb.save(path)?;
    }

    // Verify
    {
        let mut wb = Workbook::open_readonly(path)?;
        let ws = wb.worksheet(0)?;
        assert_eq!(ws.read_cell(0, 0), CellValue::String("Modified".into()));
        assert_eq!(ws.read_cell(1, 0), CellValue::String("Keep this".into()));
        assert_eq!(ws.read_cell(2, 0), CellValue::Number(999.0));
        assert_eq!(ws.read_cell(3, 0), CellValue::String("New row".into()));
    }
    Ok(())
}

fn test_row_ops() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "A")?;
    ws.write(1, 0, "B")?;
    ws.write(2, 0, "C")?;
    ws.write(3, 0, "D")?;

    // Insert 2 rows at index 1
    ws.insert_rows(1, 2)?;
    assert_eq!(ws.read_cell(0, 0), CellValue::String("A".into()));
    assert_eq!(ws.read_cell(1, 0), CellValue::Empty);
    assert_eq!(ws.read_cell(2, 0), CellValue::Empty);
    assert_eq!(ws.read_cell(3, 0), CellValue::String("B".into()));
    assert_eq!(ws.read_cell(5, 0), CellValue::String("D".into()));

    // Remove those 2 rows
    ws.remove_rows(1, 2)?;
    assert_eq!(ws.read_cell(0, 0), CellValue::String("A".into()));
    assert_eq!(ws.read_cell(1, 0), CellValue::String("B".into()));
    assert_eq!(ws.read_cell(3, 0), CellValue::String("D".into()));
    Ok(())
}

fn test_col_ops() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write_row(0, 0, ["A", "B", "C", "D"])?;

    ws.insert_columns(1, 2)?;
    assert_eq!(ws.read_cell(0, 0), CellValue::String("A".into()));
    assert_eq!(ws.read_cell(0, 1), CellValue::Empty);
    assert_eq!(ws.read_cell(0, 3), CellValue::String("B".into()));
    assert_eq!(ws.read_cell(0, 5), CellValue::String("D".into()));

    ws.remove_columns(1, 2)?;
    assert_eq!(ws.read_cell(0, 0), CellValue::String("A".into()));
    assert_eq!(ws.read_cell(0, 1), CellValue::String("B".into()));
    assert_eq!(ws.read_cell(0, 3), CellValue::String("D".into()));
    Ok(())
}

fn test_sheet_mgmt(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    wb.worksheet(0)?.write(0, 0, "Sheet1")?;
    wb.add_worksheet_with_name("Alpha")?.write(0, 0, "Alpha")?;
    wb.add_worksheet_with_name("Beta")?.write(0, 0, "Beta")?;
    assert_eq!(wb.sheet_names(), vec!["Sheet1", "Alpha", "Beta"]);

    // Rename
    wb.rename_worksheet(0, "Main")?;
    assert_eq!(wb.sheet_names(), vec!["Main", "Alpha", "Beta"]);

    // Move Beta to front
    wb.move_worksheet(2, 0)?;
    assert_eq!(wb.sheet_names(), vec!["Beta", "Main", "Alpha"]);

    // Remove Main
    wb.remove_worksheet(1)?;
    assert_eq!(wb.sheet_names(), vec!["Beta", "Alpha"]);

    // Can't remove last sheet... well we have 2, remove one more
    wb.remove_worksheet(1)?;
    assert_eq!(wb.sheet_names(), vec!["Beta"]);

    // Can't remove the last one
    assert!(wb.remove_worksheet(0).is_err());

    // Can't add duplicate name
    assert!(wb.add_worksheet_with_name("Beta").is_err());

    wb.save(path)?;

    let mut wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.sheet_names(), vec!["Beta"]);
    assert_eq!(wb2.worksheet(0)?.read_cell(0, 0), CellValue::String("Beta".into()));
    Ok(())
}

fn test_formulas(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, 10.0)?;
    ws.write(0, 1, 20.0)?;
    ws.write(0, 2, 30.0)?;
    ws.write_formula(1, 0, "SUM(A1:C1)")?;
    ws.write_formula(2, 0, "A1*2")?;
    wb.save(path)?;

    let mut wb2 = Workbook::open_readonly(path)?;
    let ws2 = wb2.worksheet(0)?;
    match ws2.read_cell(1, 0) {
        CellValue::Formula { formula, .. } => assert_eq!(formula, "SUM(A1:C1)"),
        other => panic!("Expected formula, got {:?}", other),
    }
    match ws2.read_cell(2, 0) {
        CellValue::Formula { formula, .. } => assert_eq!(formula, "A1*2"),
        other => panic!("Expected formula, got {:?}", other),
    }
    Ok(())
}

fn test_layout(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "Wide column")?;
    ws.write(0, 1, "Normal")?;
    ws.set_column_width(0, 30.0)?;
    ws.set_row_height(0, 40.0)?;
    ws.set_freeze_panes(1, 1)?;

    for r in 1..20u32 {
        ws.write(r, 0, r as f64)?;
        ws.write(r, 1, (r * 10) as f64)?;
    }
    wb.save(path)?;

    // Just verify it opens without error
    let mut wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.worksheet(0)?.read_cell(0, 0), CellValue::String("Wide column".into()));
    Ok(())
}

fn test_properties(path: &Path) -> Result<()> {
    {
        let mut wb = Workbook::new();
        wb.worksheet(0)?.write(0, 0, "data")?;
        wb.set_properties(
            DocProperties::new()
                .title("Quarterly Report")
                .author("Finance Team")
                .subject("Q4 2024 Results")
                .company("Acme Corp")
        );
        wb.save(path)?;
    }
    {
        let wb = Workbook::open_readonly(path)?;
        let p = wb.properties();
        assert_eq!(p.title.as_deref(), Some("Quarterly Report"));
        assert_eq!(p.author.as_deref(), Some("Finance Team"));
        assert_eq!(p.subject.as_deref(), Some("Q4 2024 Results"));
    }
    Ok(())
}

fn test_defined_names(path: &Path) -> Result<()> {
    let p = path.with_file_name("10_defined_names.xlsx");
    {
        let mut wb = Workbook::new();
        wb.worksheet(0)?.write(0, 0, 42.0)?;
        wb.define_name("Revenue", "Sheet1!$A$1");
        wb.define_name("_xlnm.Print_Area", "Sheet1!$A$1:$C$10");
        wb.save(&p)?;
    }
    {
        let wb = Workbook::open_readonly(&p)?;
        let names = wb.defined_names();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0].0, "Revenue");
        assert_eq!(names[0].1, "Sheet1!$A$1");
    }
    Ok(())
}

fn test_large(path: &Path) -> Result<()> {
    let rows = 10_000u32;
    let cols = 20u16;

    let start = std::time::Instant::now();
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0)?;
        // Header row
        for c in 0..cols {
            ws.write(0, c, format!("Col_{}", c + 1))?;
        }
        // Data rows
        for r in 1..=rows {
            for c in 0..cols {
                ws.write(r, c, (r as f64) * 100.0 + (c as f64))?;
            }
        }
        wb.save(path)?;
    }
    let write_time = start.elapsed();

    let start = std::time::Instant::now();
    {
        let mut wb = Workbook::open_readonly(path)?;
        let ws = wb.worksheet(0)?;
        // Spot checks
        assert_eq!(ws.read_cell(0, 0), CellValue::String("Col_1".into()));
        assert_eq!(ws.read_cell(1, 0), CellValue::Number(100.0));
        assert_eq!(ws.read_cell(rows, cols - 1), CellValue::Number(rows as f64 * 100.0 + (cols - 1) as f64));
        assert_eq!(ws.read_cell(5000, 10), CellValue::Number(500010.0));
    }
    let read_time = start.elapsed();

    let size = std::fs::metadata(path)?.len();
    println!("    📊 10K×20 = 200K cells | Write: {:?} | Read: {:?} | Size: {:.1} MB",
        write_time, read_time, size as f64 / 1_048_576.0);
    Ok(())
}


// ═══════════════════════════════════════════════════════════════
// Phase 3 feature tests
// ═══════════════════════════════════════════════════════════════

fn test_chart_column(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Sales data
    let header = Format::new().bold().background_color("#4472C4").font_color("#FFFFFF");
    ws.write_with_format(0, 0, "Region", &header)?;
    ws.write_with_format(0, 1, "Q1", &header)?;
    ws.write_with_format(0, 2, "Q2", &header)?;
    ws.write_with_format(0, 3, "Q3", &header)?;
    ws.write_with_format(0, 4, "Q4", &header)?;

    let regions = ["North", "South", "East", "West"];
    let data: [[f64; 4]; 4] = [
        [120.0, 150.0, 180.0, 160.0],
        [90.0, 110.0, 130.0, 140.0],
        [200.0, 180.0, 220.0, 250.0],
        [75.0, 95.0, 85.0, 100.0],
    ];
    for (r, (region, vals)) in regions.iter().zip(data.iter()).enumerate() {
        ws.write((r + 1) as u32, 0, *region)?;
        for (c, &v) in vals.iter().enumerate() {
            ws.write((r + 1) as u32, (c + 1) as u16, v)?;
        }
    }

    // Create chart
    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Quarterly Revenue by Region");
    chart.set_x_axis_name("Quarter");
    chart.set_y_axis_name("Revenue ($K)");
    chart.set_legend_position(LegendPosition::Bottom);
    chart.set_width(640);
    chart.set_height(400);

    for (i, region) in regions.iter().enumerate() {
        let row = i + 2;
        let values = format!("Sheet1!$B${row}:$E${row}");
        chart.add_series()
            .set_values(&values)
            .set_categories("Sheet1!$B$1:$E$1")
            .set_name(region);
    }

    ws.insert_chart(7, 0, &chart)?;
    wb.save(path)?;

    // Verify data survives
    let mut wb2 = Workbook::open_readonly(path)?;
    let ws2 = wb2.worksheet(0)?;
    assert_eq!(ws2.read_cell(0, 0), CellValue::String("Region".into()));
    assert_eq!(ws2.read_cell(1, 1), CellValue::Number(120.0));
    assert_eq!(ws2.read_cell(4, 4), CellValue::Number(100.0));
    Ok(())
}

fn test_chart_all_types(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Sample data
    ws.write_row(0, 0, ["A", "B", "C", "D", "E"])?;
    ws.write_row(1, 0, [10.0_f64, 30.0, 20.0, 50.0, 40.0])?;
    ws.write_row(2, 0, [25.0_f64, 15.0, 35.0, 20.0, 30.0])?;

    let types = [
        (ChartType::Bar, "Bar"), (ChartType::Column, "Column"),
        (ChartType::Line, "Line"), (ChartType::Pie, "Pie"),
        (ChartType::Scatter, "Scatter"), (ChartType::Area, "Area"),
        (ChartType::Doughnut, "Doughnut"), (ChartType::Radar, "Radar"),
    ];

    for (i, (ct, name)) in types.iter().enumerate() {
        let mut chart = Chart::new(*ct);
        chart.set_title(name);
        chart.add_series()
            .set_values("Sheet1!$A$2:$E$2")
            .set_categories("Sheet1!$A$1:$E$1")
            .set_name("Series 1");
        let row = 4 + (i as u32) * 16;
        ws.insert_chart(row, 0, &chart)?;
    }

    wb.save(path)?;

    // Verify file structure
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let chart_count = (0..zip.len()).filter(|i| zip.by_index(*i).unwrap().name().starts_with("xl/charts/")).count();
    assert_eq!(chart_count, 8, "Expected 8 chart XML files");
    Ok(())
}

fn test_table(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Employee data
    ws.write_row(0, 0, ["Name", "Department", "Salary", "Start Date"])?;
    ws.write(1, 0, "Alice")?; ws.write(1, 1, "Engineering")?; ws.write(1, 2, 95000.0)?; ws.write(1, 3, "2020-03-15")?;
    ws.write(2, 0, "Bob")?; ws.write(2, 1, "Marketing")?; ws.write(2, 2, 78000.0)?; ws.write(2, 3, "2019-07-01")?;
    ws.write(3, 0, "Charlie")?; ws.write(3, 1, "Engineering")?; ws.write(3, 2, 102000.0)?; ws.write(3, 3, "2018-11-20")?;
    ws.write(4, 0, "Diana")?; ws.write(4, 1, "Sales")?; ws.write(4, 2, 85000.0)?; ws.write(4, 3, "2021-01-10")?;

    let salary_fmt = Format::new().num_format("$#,##0");
    for r in 1..=4u32 { ws.set_cell_format(r, 2, &salary_fmt)?; }

    let mut table = Table::new();
    table.set_columns(&[
        TableColumn::new("Name"),
        TableColumn::new("Department"),
        TableColumn::new("Salary"),
        TableColumn::new("Start Date"),
    ]);
    table.set_style(TableStyle::Medium(9));
    table.set_name("Employees");
    ws.add_table(0, 0, 4, 3, &table)?;

    ws.set_column_width(0, 15.0)?;
    ws.set_column_width(1, 15.0)?;
    ws.set_column_width(2, 12.0)?;
    ws.set_column_width(3, 14.0)?;

    wb.save(path)?;

    // Verify
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let has_table = (0..zip.len()).any(|i| zip.by_index(i).unwrap().name().starts_with("xl/tables/"));
    assert!(has_table, "Expected table XML in zip");

    let mut wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.worksheet(0)?.read_cell(3, 0), CellValue::String("Charlie".into()));
    Ok(())
}

fn test_image(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    ws.write(0, 0, "Company Logo")?;
    ws.write(1, 0, "(image below)")?;

    // Create a minimal PNG in memory
    let png = create_test_png();
    let image = Image::from_buffer(&png)?;

    ws.insert_image(3, 0, &image)?;
    ws.write(20, 0, "Text after image")?;

    wb.save(path)?;

    // Verify media is in the zip
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let media_files: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .filter(|n: &String| n.starts_with("xl/media/"))
        .collect();
    assert_eq!(media_files.len(), 1, "Expected 1 media file, got {:?}", media_files);
    assert!(media_files[0].ends_with(".png"), "Expected PNG extension");

    let has_drawing = (0..zip.len()).any(|i| zip.by_index(i).unwrap().name().starts_with("xl/drawings/"));
    assert!(has_drawing, "Expected drawing XML");

    // Verify data still readable
    let mut wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.worksheet(0)?.read_cell(0, 0), CellValue::String("Company Logo".into()));
    Ok(())
}

fn test_conditional_formatting(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Score data
    let header = Format::new().bold();
    ws.write_with_format(0, 0, "Student", &header)?;
    ws.write_with_format(0, 1, "Score", &header)?;
    ws.write_with_format(0, 2, "Grade", &header)?;
    ws.write_with_format(0, 3, "Trend", &header)?;

    let students = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Hank"];
    let scores = [95.0, 72.0, 88.0, 45.0, 91.0, 63.0, 78.0, 55.0];
    for (i, (name, &score)) in students.iter().zip(scores.iter()).enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, *name)?;
        ws.write(r, 1, score)?;
        ws.write(r, 2, if score >= 90.0 { "A" } else if score >= 80.0 { "B" } else if score >= 70.0 { "C" } else if score >= 60.0 { "D" } else { "F" })?;
        ws.write(r, 3, score / 100.0)?;
    }

    // 1. Cell value rule: highlight scores > 80 (conceptual — format applied via CF)
    let cf_high = ConditionalFormatCell::new(CfOperator::GreaterThan, 80.0);
    ws.add_conditional_format(1, 1, 8, 1, cf_high)?;

    // 2. 3-color scale on scores: red → yellow → green
    let cf_scale = ConditionalFormat3ColorScale::new("#FF0000", "#FFFF00", "#00FF00");
    ws.add_conditional_format(1, 1, 8, 1, cf_scale)?;

    // 3. Data bar on trend column
    let cf_bar = ConditionalFormatDataBar::new("#4472C4");
    ws.add_conditional_format(1, 3, 8, 3, cf_bar)?;

    // 4. Icon set on scores
    let cf_icons = ConditionalFormatIconSet::new(IconSetType::ThreeArrows);
    ws.add_conditional_format(1, 1, 8, 1, cf_icons)?;

    // 5. 2-color scale on trend
    let cf_2scale = ConditionalFormat2ColorScale::new("#FFFFFF", "#0070C0");
    ws.add_conditional_format(1, 3, 8, 3, cf_2scale)?;

    ws.set_column_width(0, 12.0)?;
    ws.set_column_width(1, 10.0)?;
    ws.set_column_width(3, 10.0)?;

    wb.save(path)?;

    // Verify data
    let mut wb2 = Workbook::open_readonly(path)?;
    let ws2 = wb2.worksheet(0)?;
    assert_eq!(ws2.read_cell(1, 1), CellValue::Number(95.0));
    assert_eq!(ws2.read_cell(4, 0), CellValue::String("Diana".into()));

    // Verify CF XML exists in the sheet
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let sheet_xml = {
        let mut entry = zip.by_name("xl/worksheets/sheet1.xml")?;
        let mut s = String::new();
        std::io::Read::read_to_string(&mut entry, &mut s)?;
        s
    };
    assert!(sheet_xml.contains("conditionalFormatting"), "Expected conditionalFormatting in sheet XML");
    assert!(sheet_xml.contains("colorScale"), "Expected colorScale rule");
    assert!(sheet_xml.contains("dataBar"), "Expected dataBar rule");
    assert!(sheet_xml.contains("iconSet"), "Expected iconSet rule");
    Ok(())
}

fn test_data_validation(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let header = Format::new().bold();
    ws.write_with_format(0, 0, "Status", &header)?;
    ws.write_with_format(0, 1, "Priority (1-5)", &header)?;
    ws.write_with_format(0, 2, "Amount", &header)?;
    ws.write_with_format(0, 3, "Notes", &header)?;

    // 1. Dropdown list
    let mut dv_list = DataValidation::new(ValidationRule::List(
        vec!["Open".into(), "In Progress".into(), "Closed".into()]
    ));
    dv_list.set_input_message("Status", "Select task status");
    dv_list.set_error_message(ErrorStyle::Stop, "Invalid", "Choose from the dropdown");
    ws.add_data_validation(1, 0, 20, 0, &dv_list)?;

    // 2. Whole number range
    let mut dv_num = DataValidation::new(ValidationRule::WholeNumber { min: Some(1), max: Some(5) });
    dv_num.set_input_message("Priority", "Enter 1-5");
    ws.add_data_validation(1, 1, 20, 1, &dv_num)?;

    // 3. Decimal range
    let dv_dec = DataValidation::new(ValidationRule::Decimal { min: Some(0.0), max: Some(10000.0) });
    ws.add_data_validation(1, 2, 20, 2, &dv_dec)?;

    // 4. Text length
    let dv_len = DataValidation::new(ValidationRule::TextLength { min: Some(0), max: Some(200) });
    ws.add_data_validation(1, 3, 20, 3, &dv_len)?;

    ws.set_column_width(0, 14.0)?;
    ws.set_column_width(1, 14.0)?;
    ws.set_column_width(3, 30.0)?;

    // Write some sample data
    ws.write(1, 0, "Open")?; ws.write(1, 1, 3)?; ws.write(1, 2, 1500.0)?; ws.write(1, 3, "First task")?;
    ws.write(2, 0, "Closed")?; ws.write(2, 1, 1)?; ws.write(2, 2, 750.0)?; ws.write(2, 3, "Done")?;

    wb.save(path)?;

    // Verify DV XML
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let sheet_xml = {
        let mut entry = zip.by_name("xl/worksheets/sheet1.xml")?;
        let mut s = String::new();
        std::io::Read::read_to_string(&mut entry, &mut s)?;
        s
    };
    assert!(sheet_xml.contains("dataValidation"), "Expected dataValidation in sheet XML");
    assert!(sheet_xml.contains("Open,In Progress,Closed"), "Expected list values in DV");
    Ok(())
}

fn test_sparklines(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let header = Format::new().bold();
    ws.write_with_format(0, 0, "Month", &header)?;
    for (i, m) in ["Jan", "Feb", "Mar", "Apr", "May", "Jun"].iter().enumerate() {
        ws.write(0, (i + 1) as u16, *m)?;
    }
    ws.write_with_format(0, 7, "Trend", &header)?;

    // Revenue data
    ws.write(1, 0, "Product A")?;
    ws.write_row(1, 1, [10.0_f64, 15.0, 12.0, 18.0, 22.0, 20.0])?;
    ws.write(2, 0, "Product B")?;
    ws.write_row(2, 1, [8.0_f64, 6.0, 9.0, 7.0, 11.0, 13.0])?;
    ws.write(3, 0, "Product C")?;
    ws.write_row(3, 1, [5.0_f64, -2.0, 3.0, -1.0, 4.0, 6.0])?;

    // Line sparkline
    let sp1 = Sparkline::new("Sheet1!B2:G2", SparklineType::Line);
    ws.add_sparkline(1, 7, &sp1)?;

    // Column sparkline with color
    let mut sp2 = Sparkline::new("Sheet1!B3:G3", SparklineType::Column);
    sp2.set_color("#FF6600");
    ws.add_sparkline(2, 7, &sp2)?;

    // Win/loss sparkline
    let sp3 = Sparkline::new("Sheet1!B4:G4", SparklineType::WinLoss);
    ws.add_sparkline(3, 7, &sp3)?;

    ws.set_column_width(0, 12.0)?;
    ws.set_column_width(7, 15.0)?;

    wb.save(path)?;

    // Verify sparkline XML
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let sheet_xml = {
        let mut entry = zip.by_name("xl/worksheets/sheet1.xml")?;
        let mut s = String::new();
        std::io::Read::read_to_string(&mut entry, &mut s)?;
        s
    };
    assert!(sheet_xml.contains("sparkline"), "Expected sparkline in sheet XML");
    Ok(())
}

fn test_dashboard(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    wb.worksheet(0)?.set_name("Dashboard")?;
    let ws = wb.worksheet(0)?;

    // ── Header ──
    let title_fmt = Format::new().bold().font_size(16.0).font_color("#1F4E79");
    ws.write_with_format(0, 0, "Sales Dashboard — Q4 2024", &title_fmt)?;
    ws.merge_range(0, 0, 0, 6, "Sales Dashboard — Q4 2024", &title_fmt)?;

    // ── Data table ──
    let hdr = Format::new().bold().background_color("#1F4E79").font_color("#FFFFFF").border(BorderStyle::Thin);
    let headers = ["Product", "Oct", "Nov", "Dec", "Total", "Avg", "Trend"];
    for (c, h) in headers.iter().enumerate() {
        ws.write_with_format(2, c as u16, *h, &hdr)?;
    }

    let products = ["Widget Pro", "Gadget X", "Thingamajig", "Doohickey"];
    let sales: [[f64; 3]; 4] = [
        [12500.0, 15800.0, 18200.0],
        [8900.0, 9200.0, 11500.0],
        [6700.0, 7100.0, 5900.0],
        [3200.0, 4100.0, 3800.0],
    ];

    let money = Format::new().num_format("$#,##0").border(BorderStyle::Thin).border_color("#D9D9D9");
    let border = Format::new().border(BorderStyle::Thin).border_color("#D9D9D9");

    for (i, (prod, vals)) in products.iter().zip(sales.iter()).enumerate() {
        let r = (i + 3) as u32;
        ws.write_with_format(r, 0, *prod, &border)?;
        for (c, &v) in vals.iter().enumerate() {
            ws.write_with_format(r, (c + 1) as u16, v, &money)?;
        }
        let total = vals.iter().sum::<f64>();
        ws.write_with_format(r, 4, total, &money)?;
        ws.write_with_format(r, 5, total / 3.0, &money)?;
    }

    // Table
    let mut table = Table::new();
    table.set_columns(&headers.iter().map(|h| TableColumn::new(h)).collect::<Vec<_>>());
    table.set_style(TableStyle::Medium(2));
    ws.add_table(2, 0, 6, 6, &table)?;

    // Sparklines in Trend column
    for i in 0..4 {
        let r = (i + 3) as u32;
        let range = format!("Dashboard!B{}:D{}", r + 1, r + 1);
        let sp = Sparkline::new(&range, SparklineType::Line);
        ws.add_sparkline(r, 6, &sp)?;
    }

    // Conditional formatting: 3-color scale on monthly data
    let cf = ConditionalFormat3ColorScale::new("#F8696B", "#FFEB84", "#63BE7B");
    ws.add_conditional_format(3, 1, 6, 3, cf)?;

    // Data bar on totals
    let db = ConditionalFormatDataBar::new("#4472C4");
    ws.add_conditional_format(3, 4, 6, 4, db)?;

    // Chart
    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Monthly Sales Comparison");
    chart.set_width(640);
    chart.set_height(360);
    chart.set_legend_position(LegendPosition::Bottom);
    for (i, prod) in products.iter().enumerate() {
        let r = i + 4;
        chart.add_series()
            .set_values(&format!("Dashboard!$B${r}:$D${r}"))
            .set_categories("Dashboard!$B$3:$D$3")
            .set_name(prod);
    }
    ws.insert_chart(8, 0, &chart)?;

    // Data validation on a separate input area
    ws.write(25, 0, "Filter by product:")?;
    let dv = DataValidation::new(ValidationRule::List(
        products.iter().map(|s| s.to_string()).collect()
    ));
    ws.add_data_validation(25, 1, 25, 1, &dv)?;

    // Column widths
    ws.set_column_width(0, 16.0)?;
    for c in 1..=5 { ws.set_column_width(c, 12.0)?; }
    ws.set_column_width(6, 14.0)?;
    ws.set_freeze_panes(3, 1)?;

    wb.save(path)?;

    // Verify everything
    let mut wb2 = Workbook::open_readonly(path)?;
    let ws2 = wb2.worksheet(0)?;
    assert_eq!(ws2.read_cell(3, 0), CellValue::String("Widget Pro".into()));
    assert_eq!(ws2.read_cell(3, 1), CellValue::Number(12500.0));

    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let names: Vec<String> = (0..zip.len()).map(|i| zip.by_index(i).unwrap().name().to_string()).collect();
    assert!(names.iter().any(|n: &String| n.contains("chart")), "Expected chart");
    assert!(names.iter().any(|n: &String| n.contains("table")), "Expected table");
    assert!(names.iter().any(|n: &String| n.contains("drawing")), "Expected drawing");

    let size = std::fs::metadata(path)?.len();
    println!("    📊 Dashboard file size: {:.1} KB", size as f64 / 1024.0);
    Ok(())
}

// ── Helper: create minimal valid PNG ──

fn create_test_png() -> Vec<u8> {
    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    write_png_chunk(&mut png, b"IHDR", &[0,0,0,1, 0,0,0,1, 8, 2, 0,0,0]);
    write_png_chunk(&mut png, b"IDAT", &[0x78,0x01,0x62,0xF8,0xCF,0xC0,0x00,0x00,0x00,0x04,0x00,0x01]);
    write_png_chunk(&mut png, b"IEND", &[]);
    png
}

fn write_png_chunk(buf: &mut Vec<u8>, ct: &[u8; 4], data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(ct);
    buf.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(ct);
    crc_input.extend_from_slice(data);
    let mut crc: u32 = 0xFFFFFFFF;
    for &b in &crc_input {
        crc ^= b as u32;
        for _ in 0..8 { if crc & 1 != 0 { crc = (crc >> 1) ^ 0xEDB88320; } else { crc >>= 1; } }
    }
    buf.extend_from_slice(&(!crc).to_be_bytes());
}

// ─── Phase 4 Tests ─────────────────────────────────────────

fn test_autofit(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "ID")?;
    ws.write(0, 1, "Product Name")?;
    ws.write(0, 2, "Description")?;
    ws.write(0, 3, "Price")?;
    ws.write(1, 0, 1)?;
    ws.write(1, 1, "Widget Pro Max Ultra")?;
    ws.write(1, 2, "A very long description that should make the column wider than default")?;
    ws.write(1, 3, 1299.99)?;
    ws.write(2, 0, 2)?;
    ws.write(2, 1, "Gadget")?;
    ws.write(2, 2, "Short")?;
    ws.write(2, 3, 49.99)?;
    ws.autofit()?;
    wb.save(path)?;
    // Verify the file has cols with customWidth
    let xml = read_sheet_xml(path, 1);
    assert!(xml.contains("customWidth"), "autofit should set customWidth");
    Ok(())
}

fn test_protection(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "This sheet is protected")?;
    ws.write(1, 0, "Password: demo123")?;
    ws.protect_with_password("demo123");
    wb.protect_with_password("admin");
    wb.save(path)?;
    let sheet_xml = read_sheet_xml(path, 1);
    assert!(sheet_xml.contains("sheetProtection"), "should have sheetProtection");
    assert!(sheet_xml.contains("password="), "should have password hash");
    let wb_xml = read_zip_entry(path, "xl/workbook.xml");
    assert!(wb_xml.contains("workbookProtection"), "should have workbookProtection");
    Ok(())
}

fn test_print_settings(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    for r in 0..50u32 {
        ws.write(r, 0, format!("Row {}", r + 1))?;
        ws.write(r, 1, r as f64 * 10.0)?;
    }
    let ps = PrintSettings::new()
        .paper_size(1)
        .orientation(Orientation::Landscape)
        .margins(1.0, 1.0, 0.75, 0.75)
        .header("&C&\"Arial,Bold\"Monthly Report")
        .footer("&LPage &P of &N&RConfidential");
    ws.set_print_settings(&ps);
    ws.set_page_breaks(&[24], &[]);
    wb.save(path)?;
    let xml = read_sheet_xml(path, 1);
    assert!(xml.contains("pageMargins"), "should have pageMargins");
    assert!(xml.contains("pageSetup"), "should have pageSetup");
    assert!(xml.contains("landscape"), "should be landscape");
    assert!(xml.contains("oddHeader"), "should have header");
    assert!(xml.contains("rowBreaks"), "should have page breaks");
    Ok(())
}

fn test_rich_text(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_column_width(0, 50.0)?;
    ws.set_row_height(0, 30.0)?;

    let rt = RichText::new()
        .add_run("Normal text, ")
        .add_bold("bold text, ")
        .add_italic("italic text, ")
        .add_styled("red & big", RichTextRun::new().color("FF0000").font_size(18.0).bold());
    ws.write_rich_text(0, 0, &rt)?;

    let rt2 = RichText::new()
        .add_styled("Calibri ", RichTextRun::new().font_name("Calibri").font_size(11.0))
        .add_styled("Courier ", RichTextRun::new().font_name("Courier New").font_size(11.0))
        .add_styled("Arial", RichTextRun::new().font_name("Arial").font_size(11.0));
    ws.write_rich_text(1, 0, &rt2)?;

    wb.save(path)?;
    let xml = read_sheet_xml(path, 1);
    assert!(xml.contains("<r>"), "should have rich text runs");
    assert!(xml.contains("<b/>"), "should have bold");
    assert!(xml.contains("<i/>"), "should have italic");
    assert!(xml.contains("FFFF0000"), "should have red color");
    assert!(xml.contains("rFont"), "should have font names");
    Ok(())
}

fn test_streaming(path: &Path) -> Result<()> {
    let mut wb = StreamingWorkbook::new();
    wb.set_column_width(0, 12.0);
    wb.set_column_width(1, 15.0);
    wb.set_column_width(2, 10.0);

    let header_fmt = Format::new().bold();
    wb.write_string_with_format(0, 0, "Row", &header_fmt)?;
    wb.write_string_with_format(0, 1, "Value", &header_fmt)?;
    wb.write_string_with_format(0, 2, "Status", &header_fmt)?;

    for row in 1..=100_000u32 {
        wb.write_number(row, 0, row as f64)?;
        wb.write_number(row, 1, row as f64 * 3.14)?;
        wb.write_string(row, 2, if row % 2 == 0 { "Even" } else { "Odd" })?;
    }
    wb.save(path)?;
    let size = std::fs::metadata(path)?.len();
    assert!(size > 100_000, "100K row file should be substantial: {} bytes", size);
    // Verify it's a valid zip
    let file = std::fs::File::open(path)?;
    let archive = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    assert!(archive.len() > 3, "should have multiple zip entries");
    Ok(())
}

fn test_phase4_combined(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();

    // Sheet 1: Rich text + autofit + protection
    let ws = wb.worksheet(0)?;
    ws.set_name("Rich & Protected")?;
    let rt = RichText::new()
        .add_bold("Q1 Revenue: ")
        .add_styled("$1.2M", RichTextRun::new().color("008000").font_size(14.0));
    ws.write_rich_text(0, 0, &rt)?;
    ws.write(1, 0, "This sheet is protected")?;
    ws.write(1, 1, 1_200_000.0)?;
    ws.autofit()?;
    ws.protect_with_password("secret");

    // Sheet 2: Print settings + data
    let ws2 = wb.add_worksheet_with_name("Print Ready")?;
    let ps = PrintSettings::new()
        .orientation(Orientation::Landscape)
        .margins(0.5, 0.5, 0.5, 0.5)
        .header("&CPhase 4 Demo")
        .footer("&RPage &P");
    ws2.set_print_settings(&ps);
    for r in 0..30u32 {
        ws2.write(r, 0, format!("Item {}", r + 1))?;
        ws2.write(r, 1, (r + 1) as f64 * 99.99)?;
    }
    ws2.set_page_breaks(&[14], &[]);

    // Workbook protection
    wb.protect_with_password("admin");
    wb.save(path)?;

    // Validate
    let s1 = read_sheet_xml(path, 1);
    assert!(s1.contains("<r>"), "sheet1 should have rich text");
    assert!(s1.contains("sheetProtection"), "sheet1 should be protected");
    let s2 = read_sheet_xml(path, 2);
    assert!(s2.contains("pageSetup"), "sheet2 should have print settings");
    assert!(s2.contains("rowBreaks"), "sheet2 should have page breaks");
    let wbxml = read_zip_entry(path, "xl/workbook.xml");
    assert!(wbxml.contains("workbookProtection"), "workbook should be protected");
    Ok(())
}

// ─── Helpers ───────────────────────────────────────────────

fn read_sheet_xml(path: &Path, sheet_num: usize) -> String {
    read_zip_entry(path, &format!("xl/worksheets/sheet{sheet_num}.xml"))
}

fn read_zip_entry(path: &Path, entry_name: &str) -> String {
    let file = std::fs::File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file)).unwrap();
    let mut entry = archive.by_name(entry_name).unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut entry, &mut xml).unwrap();
    xml
}

// ─── 25. Acme Corp Financial Model ─────────────────────────

fn test_acme_financial_model(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();

    // ── Formats ──
    let title_fmt = Format::new().bold().font_size(18.0).font_color("#1F4E79");
    let header_fmt = Format::new().bold().font_size(11.0).background_color("#1F4E79").font_color("#FFFFFF")
        .border(BorderStyle::Thin).border_color("#000000").text_wrap();
    let currency_fmt = Format::new().num_format("$#,##0").border(BorderStyle::Thin);
    let pct_fmt = Format::new().num_format("0.0%").border(BorderStyle::Thin);
    let date_fmt = Format::new().num_format("yyyy-mm-dd").border(BorderStyle::Thin);
    let total_fmt = Format::new().bold().num_format("$#,##0").border(BorderStyle::Medium)
        .background_color("#D6E4F0");
    let input_fmt = Format::new().unlocked().background_color("#FFFFCC").border(BorderStyle::Thin);
    let cell_fmt = Format::new().border(BorderStyle::Thin);
    let indent_fmt = Format::new().border(BorderStyle::Thin).indent(1);  // Indented sub-items
    let subtotal_fmt = Format::new().bold().num_format("$#,##0").border(BorderStyle::Thin)
        .background_color("#E2EFDA");

    // ════════════════════════════════════════════════════════════
    // Sheet 1: P&L Statement
    // ════════════════════════════════════════════════════════════
    let ws = wb.worksheet(0)?;
    ws.set_name("P&L")?;
    ws.set_tab_color("#1F4E79");  // Blue tab for financials
    ws.set_print_scale(85);       // Scale to 85% for printing

    // Title
    let rt = RichText::new()
        .add_styled("Acme Corp ", RichTextRun::new().bold().font_size(16.0).color("1F4E79"))
        .add_styled("— Q1 Income Statement", RichTextRun::new().font_size(14.0).color("4472C4"));
    ws.write_rich_text(0, 0, &rt)?;
    ws.set_row_height(0, 28.0)?;

    // Column headers
    let headers = ["Line Item", "Jan", "Feb", "Mar", "Q1 Total", "% of Revenue"];
    for (c, h) in headers.iter().enumerate() {
        ws.write_with_format(2, c as u16, *h, &header_fmt)?;
    }
    ws.set_column_width(0, 22.0)?;
    for c in 1..=5u16 { ws.set_column_width(c, 14.0)?; }

    // Revenue section
    let rev_items = [
        ("Product Sales", [120000.0, 135000.0, 142000.0]),
        ("Service Revenue", [45000.0, 48000.0, 52000.0]),
        ("Licensing", [15000.0, 15000.0, 18000.0]),
    ];
    let mut row = 3u32;
    ws.write_with_format(row, 0, "Revenue", &Format::new().bold().border(BorderStyle::Thin))?;
    row += 1;
    for (name, months) in &rev_items {
        ws.write_with_format(row, 0, *name, &indent_fmt)?;
        for (c, &val) in months.iter().enumerate() {
            ws.write_with_format(row, (c + 1) as u16, val, &currency_fmt)?;
        }
        // Q1 Total formula
        ws.write_formula(row, 4, &format!("SUM(B{r}:D{r})", r = row + 1))?;
        ws.set_cell_format(row, 4, &currency_fmt)?;
        // % of Revenue formula (will reference total row)
        ws.write_formula(row, 5, &format!("E{r}/E8", r = row + 1))?;
        ws.set_cell_format(row, 5, &pct_fmt)?;
        row += 1;
    }
    // Group detail rows under Revenue
    ws.group_rows(4, 6, 1);

    // Revenue Total
    ws.write_with_format(7, 0, "Total Revenue", &total_fmt)?;
    for c in 1..=4u16 {
        let col_letter = col_letter(c);
        ws.write_formula(7, c, &format!("SUM({cl}5:{cl}7)", cl = col_letter))?;
        ws.set_cell_format(7, c, &total_fmt)?;
    }
    ws.write_formula(7, 5, "1")?;
    ws.set_cell_format(7, 5, &Format::new().bold().num_format("0.0%").border(BorderStyle::Medium).background_color("#D6E4F0"))?;

    // Expenses section
    let exp_items = [
        ("COGS", [54000.0, 60750.0, 63900.0]),
        ("Salaries", [85000.0, 85000.0, 87000.0]),
        ("Marketing", [12000.0, 15000.0, 18000.0]),
        ("Rent & Utilities", [8000.0, 8000.0, 8000.0]),
        ("Other", [5000.0, 6000.0, 5500.0]),
    ];
    row = 9;
    ws.write_with_format(row, 0, "Expenses", &Format::new().bold().border(BorderStyle::Thin))?;
    row += 1;
    for (name, months) in &exp_items {
        ws.write_with_format(row, 0, *name, &indent_fmt)?;
        for (c, &val) in months.iter().enumerate() {
            ws.write_with_format(row, (c + 1) as u16, val, &currency_fmt)?;
        }
        ws.write_formula(row, 4, &format!("SUM(B{r}:D{r})", r = row + 1))?;
        ws.set_cell_format(row, 4, &currency_fmt)?;
        ws.write_formula(row, 5, &format!("E{r}/E8", r = row + 1))?;
        ws.set_cell_format(row, 5, &pct_fmt)?;
        row += 1;
    }
    ws.group_rows(10, 14, 1);

    // Expense Total
    ws.write_with_format(15, 0, "Total Expenses", &total_fmt)?;
    for c in 1..=4u16 {
        let cl = col_letter(c);
        ws.write_formula(15, c, &format!("SUM({cl}11:{cl}15)", cl = cl))?;
        ws.set_cell_format(15, c, &total_fmt)?;
    }
    ws.write_formula(15, 5, "E16/E8")?;
    ws.set_cell_format(15, 5, &Format::new().bold().num_format("0.0%").border(BorderStyle::Medium).background_color("#D6E4F0"))?;

    // Net Income
    ws.write_with_format(17, 0, "Net Income", &Format::new().bold().font_size(12.0).border(BorderStyle::Double).background_color("#C6EFCE"))?;
    for c in 1..=4u16 {
        let cl = col_letter(c);
        ws.write_formula(17, c, &format!("{cl}8-{cl}16"))?;
        ws.set_cell_format(17, c, &Format::new().bold().num_format("$#,##0").border(BorderStyle::Double).background_color("#C6EFCE"))?;
    }
    ws.write_formula(17, 5, "E18/E8")?;
    ws.set_cell_format(17, 5, &Format::new().bold().num_format("0.0%").border(BorderStyle::Double).background_color("#C6EFCE"))?;

    // Conditional formatting: red for negative net income
    ws.add_conditional_format(17, 1, 17, 4, {
        let mut cf = ConditionalFormatCell::new(CfOperator::LessThan, 0.0);
        cf.set_format(&Format::new().font_color("#FF0000").bold());
        cf
    })?;

    // Sprint 2: Formula-based CF — highlight expense items where Q1 total > $200K
    let mut cf_formula = ConditionalFormatFormula::new("$E11>200000");
    cf_formula.set_format(&Format::new().background_color("#FFC7CE").font_color("#9C0006"));
    ws.add_conditional_format(10, 0, 14, 5, cf_formula)?;

    // Sprint 2: Above-average CF — highlight above-average expense line items
    let mut cf_avg = ConditionalFormatAverage::new(AverageType::Above);
    cf_avg.set_format(&Format::new().bold().background_color("#FFEB9C").font_color("#9C5700"));
    ws.add_conditional_format(10, 4, 14, 4, cf_avg)?;

    // Freeze header row
    ws.set_freeze_panes(3, 1)?;
    ws.set_print_area(0, 0, 17, 5);
    ws.set_repeat_rows(2, 2);

    // Comment on an estimated value
    ws.add_comment(6, 1, "Licensing revenue estimated based on contract pipeline");

    // ════════════════════════════════════════════════════════════
    // Sheet 2: Dashboard
    // ════════════════════════════════════════════════════════════
    {
    let ws2 = wb.add_worksheet_with_name("Dashboard")?;
    ws2.set_tab_color("#4472C4");  // Blue tab
    ws2.hide_gridlines();          // Clean dashboard look
    ws2.set_zoom(90);              // Slightly zoomed out

    ws2.write_with_format(0, 0, "Acme Corp — Q1 Dashboard", &title_fmt)?;
    ws2.set_row_height(0, 28.0)?;

    // Monthly revenue data for chart
    ws2.write_with_format(2, 0, "Month", &header_fmt)?;
    ws2.write_with_format(2, 1, "Revenue", &header_fmt)?;
    ws2.write_with_format(2, 2, "Expenses", &header_fmt)?;
    ws2.write_with_format(2, 3, "Profit", &header_fmt)?;
    let months = ["Jan", "Feb", "Mar"];
    let revenues = [180000.0, 198000.0, 212000.0];
    let expenses = [164000.0, 174750.0, 182400.0];
    for i in 0..3u32 {
        ws2.write_with_format(3 + i, 0, months[i as usize], &cell_fmt)?;
        ws2.write_with_format(3 + i, 1, revenues[i as usize], &currency_fmt)?;
        ws2.write_with_format(3 + i, 2, expenses[i as usize], &currency_fmt)?;
        ws2.write_with_format(3 + i, 3, revenues[i as usize] - expenses[i as usize], &currency_fmt)?;
    }
    ws2.set_column_width(0, 10.0)?;
    for c in 1..=3u16 { ws2.set_column_width(c, 14.0)?; }

    // Revenue trend chart
    let mut chart = Chart::new(ChartType::Column);
    chart.add_series()
        .set_categories("Dashboard!$A$4:$A$6")
        .set_values("Dashboard!$B$4:$B$6")
        .set_name("Revenue")
        .set_data_labels(true);
    chart.add_series()
        .set_categories("Dashboard!$A$4:$A$6")
        .set_values("Dashboard!$C$4:$C$6")
        .set_name("Expenses")
        .set_data_labels(true);
    chart.set_title("Q1 Revenue vs Expenses");
    ws2.insert_chart(7, 0, &chart)?;

    // Sprint 3: Combo chart — revenue bars + profit margin line on secondary axis
    ws2.write_with_format(2, 5, "Margin %", &header_fmt)?;
    ws2.set_column_width(5, 12.0)?;
    let margins = [8.9, 11.7, 13.9]; // profit margin %
    for i in 0..3u32 {
        ws2.write_with_format(3 + i, 5, margins[i as usize] / 100.0, &pct_fmt)?;
    }

    let mut combo = Chart::new(ChartType::Column);
    combo.set_title("Revenue & Profit Margin");
    combo.set_y_axis_name("Revenue ($)");
    combo.set_y2_axis_name("Margin %");
    combo.set_width(640);
    combo.set_height(360);
    combo.add_series()
        .set_categories("Dashboard!$A$4:$A$6")
        .set_values("Dashboard!$B$4:$B$6")
        .set_name("Revenue")
        .set_data_labels(true)
        .set_trendline(TrendlineType::Linear);
    combo.add_series()
        .set_categories("Dashboard!$A$4:$A$6")
        .set_values("Dashboard!$F$4:$F$6")
        .set_name("Margin %")
        .set_chart_type(ChartType::Line)
        .set_secondary_axis(true)
        .set_data_labels(true);
    ws2.insert_chart(22, 0, &combo)?;

    // Sparklines for monthly trends
    ws2.add_sparkline(3, 4, &Sparkline::new("Dashboard!$B$4:$B$6", SparklineType::Line))?;
    ws2.add_sparkline(4, 4, &Sparkline::new("Dashboard!$C$4:$C$6", SparklineType::Line))?;
    ws2.add_sparkline(5, 4, &Sparkline::new("Dashboard!$D$4:$D$6", SparklineType::Column))?;
    ws2.write_with_format(2, 4, "Trend", &Format::new().bold().rotation(90).align(Align::Center))?;

    // Conditional formatting: color scale on profit
    ws2.add_conditional_format(3, 3, 5, 3, ConditionalFormat2ColorScale::new("#FF0000", "#00FF00"))?;

    // Sprint 2: Top 1 profit month — bold green highlight
    let mut cf_top = ConditionalFormatTopBottom::new(TopBottomType::Top, 1);
    cf_top.set_format(&Format::new().bold().font_color("#006100").background_color("#C6EFCE"));
    ws2.add_conditional_format(3, 3, 5, 3, cf_top)?;
    }

    // ════════════════════════════════════════════════════════════
    // Sheet 3: Assumptions (protected with unlocked input cells)
    // ════════════════════════════════════════════════════════════
    {
    let ws3 = wb.add_worksheet_with_name("Assumptions")?;
    ws3.set_tab_color("#FFC000");  // Gold tab for inputs

    ws3.write_with_format(0, 0, "Model Assumptions", &title_fmt)?;
    ws3.set_row_height(0, 28.0)?;
    ws3.set_column_width(0, 25.0)?;
    ws3.set_column_width(1, 18.0)?;

    let assumptions = [
        ("Growth Rate", "8%"),
        ("Tax Rate", "21%"),
        ("Discount Rate", "10%"),
        ("Headcount", "45"),
        ("Avg Salary", "$85,000"),
    ];
    ws3.write_with_format(2, 0, "Parameter", &header_fmt)?;
    ws3.write_with_format(2, 1, "Value", &header_fmt)?;
    for (i, (name, val)) in assumptions.iter().enumerate() {
        let r = 3 + i as u32;
        ws3.write_with_format(r, 0, *name, &cell_fmt)?;
        ws3.write_with_format(r, 1, *val, &input_fmt)?;
        ws3.add_comment(r, 1, &format!("Input: {name} — editable when sheet is protected"));
    }

    // Data validation on Growth Rate
    let mut dv = DataValidation::new(ValidationRule::Decimal { min: Some(0.0), max: Some(0.5) });
    dv.set_input_message("Growth Rate", "Enter 0-50%");
    ws3.add_data_validation(3, 1, 3, 1, &dv)?;

    // Deprecated assumption (strikethrough)
    let strike_fmt = Format::new().strikethrough().font_color("#999999").border(BorderStyle::Thin);
    let r = 3 + assumptions.len() as u32;
    ws3.write_with_format(r, 0, "Old Tax Rate (deprecated)", &strike_fmt)?;
    ws3.write_with_format(r, 1, "25%", &strike_fmt)?;

    ws3.protect_with_password("acme2024");
    }

    // ════════════════════════════════════════════════════════════
    // Sheet 4: Transaction Data (auto-filter, hidden cols)
    // ════════════════════════════════════════════════════════════
    {
    let ws4 = wb.add_worksheet_with_name("Data")?;
    ws4.set_tab_color("#70AD47");  // Green tab for data
    ws4.set_repeat_columns(0, 0); // Repeat Date column when printing

    let data_headers = ["Date", "Description", "Category", "Amount", "Helper"];
    for (c, h) in data_headers.iter().enumerate() {
        ws4.write_with_format(0, c as u16, *h, &header_fmt)?;
    }
    ws4.set_column_width(0, 12.0)?;
    ws4.set_column_width(1, 25.0)?;
    ws4.set_column_width(2, 15.0)?;
    ws4.set_column_width(3, 14.0)?;

    let categories = ["Sales", "Marketing", "Operations", "HR", "IT"];
    let descs = ["Client payment", "Ad campaign", "Office supplies", "Recruiting fee", "Software license",
                 "Consulting revenue", "Trade show", "Equipment", "Training", "Cloud hosting"];
    for r in 1..=500u32 {
        let dt = ExcelDateTime::from_ymd(2026, 1 + ((r - 1) % 3) as u32, 1 + ((r - 1) % 28) as u32).unwrap();
        ws4.write_with_format(r, 0, dt, &date_fmt)?;
        ws4.write_with_format(r, 1, descs[((r - 1) % 10) as usize], &cell_fmt)?;
        ws4.write_with_format(r, 2, categories[((r - 1) % 5) as usize], &cell_fmt)?;
        let amount = 1000.0 + (r as f64 * 73.0) % 50000.0;
        ws4.write_with_format(r, 3, amount, &currency_fmt)?;
        // Hidden helper column
        ws4.write_formula(r, 4, &format!("D{r}*1.1", r = r + 1))?;
    }

    // Auto-filter on data range
    ws4.set_autofilter(0, 0, 500, 3);
    // Hide helper column
    ws4.set_column_hidden(4, true);
    // Freeze header
    ws4.set_freeze_panes(1, 0)?;

    // ── Sprint 2: New CF types on Data sheet ──

    // Text contains: highlight "Sales" category in gold (first 20 rows — visible on open)
    let mut cf_text = ConditionalFormatText::new(TextOperator::Contains, "Sales");
    cf_text.set_format(&Format::new().background_color("#FFF2CC"));
    ws4.add_conditional_format(1, 2, 20, 2, cf_text)?;

    // Duplicate values: highlight duplicate descriptions (first 20 rows)
    let mut cf_dup = ConditionalFormatDuplicate::new();
    cf_dup.set_format(&Format::new().font_color("#9C5700").background_color("#FFEB9C"));
    ws4.add_conditional_format(1, 1, 20, 1, cf_dup)?;

    // Date occurring: highlight "this month" dates (first 20 rows)
    let mut cf_date = ConditionalFormatDate::new(DateOccurring::ThisMonth);
    cf_date.set_format(&Format::new().bold().font_color("#1F4E79"));
    ws4.add_conditional_format(1, 0, 20, 0, cf_date)?;
    }

    // Hyperlink back to P&L from Dashboard
    wb.worksheet(1)?.write_internal_link(20, 0, "'P&L'!A1", "← Back to P&L")?;

    // Workbook protection
    wb.protect_with_password("acme");
    wb.set_calc_mode(CalcMode::Auto);

    // Named range for revenue total
    wb.define_name("Q1_Revenue", "'P&L'!$E$8");

    wb.save(path)?;

    // ── Validate ──
    let s1 = read_sheet_xml(path, 1);
    assert!(s1.contains("dimension"), "P&L should have dimension");
    assert!(s1.contains("sheetViews"), "P&L should have sheetViews");
    assert!(s1.contains("sheetFormatPr"), "P&L should have sheetFormatPr");
    assert!(s1.contains("<f>"), "P&L should have formulas");
    assert!(s1.contains("conditionalFormatting"), "P&L should have CF");
    assert!(s1.contains("outlineLevel"), "P&L should have row grouping");

    let s3 = read_sheet_xml(path, 3);
    assert!(s3.contains("sheetProtection"), "Assumptions should be protected");
    assert!(s3.contains("dataValidation"), "Assumptions should have DV");

    let s4 = read_sheet_xml(path, 4);
    assert!(s4.contains("autoFilter"), "Data should have autoFilter");
    assert!(s4.contains("hidden=\"1\""), "Data should have hidden column");

    let wb_xml = read_zip_entry(path, "xl/workbook.xml");
    assert!(wb_xml.contains("workbookProtection"), "Workbook should be protected");
    assert!(wb_xml.contains("Q1_Revenue"), "Should have named range");
    assert!(wb_xml.contains("calcPr"), "Should have calcPr");

    // Phase 6 Sprint 1 validations
    let s2 = read_sheet_xml(path, 2);
    assert!(s2.contains("showGridLines=\"0\""), "Dashboard should hide gridlines");
    assert!(s2.contains("zoomScale=\"90\""), "Dashboard should have zoom 90");
    assert!(s2.contains("tabColor"), "Dashboard should have tab color");
    assert!(s1.contains("tabColor"), "P&L should have tab color");
    assert!(s3.contains("tabColor"), "Assumptions should have tab color");

    // Sprint 2: new CF types
    assert!(s1.contains("type=\"expression\""), "P&L should have formula-based CF");
    assert!(s1.contains("type=\"aboveAverage\""), "P&L should have above-average CF");
    assert!(s2.contains("type=\"top10\""), "Dashboard should have top/bottom CF");
    assert!(s4.contains("type=\"containsText\""), "Data should have text-contains CF");
    assert!(s4.contains("type=\"duplicateValues\""), "Data should have duplicate CF");
    assert!(s4.contains("type=\"timePeriod\""), "Data should have date CF");

    let styles_xml = read_zip_entry(path, "xl/styles.xml");
    assert!(styles_xml.contains("indent"), "Styles should have indented alignment");
    assert!(styles_xml.contains("strike"), "Styles should have strikethrough font");
    // Sprint 2: dxf support
    assert!(styles_xml.contains("<dxfs"), "Styles should have dxfs section");
    assert!(styles_xml.contains("<dxf>"), "Styles should have dxf entries");

    // Check comments exist
    let entries = list_zip_entries(path);
    assert!(entries.iter().any(|e| e.contains("comments")), "Should have comments file");

    // Sprint 3: chart enhancements — verify combo chart XML
    let chart_entries: Vec<String> = entries.iter().filter(|e| e.starts_with("xl/charts/")).cloned().collect();
    assert!(chart_entries.len() >= 2, "Should have at least 2 charts (original + combo)");
    let combo_xml = read_zip_entry(path, chart_entries.last().unwrap());
    assert!(combo_xml.contains("c:lineChart"), "Combo chart should have lineChart block");
    assert!(combo_xml.contains("c:barChart"), "Combo chart should have barChart block");
    assert!(combo_xml.contains("c:dLbls"), "Combo chart should have data labels");
    assert!(combo_xml.contains("c:trendline"), "Combo chart should have trendline");
    assert!(combo_xml.contains("axId") && combo_xml.contains("val=\"444444444\""), "Combo chart should have secondary axis");

    let size = std::fs::metadata(path)?.len();
    println!("    📊 Acme Financial Model: {:.1} KB, {} sheets, {} features validated",
        size as f64 / 1024.0, 4, 35);

    Ok(())
}

// ─── 26. Phase 6 Feature Showcase ───────────────────────

fn test_phase6_showcase(path: &Path) -> Result<()> {
    let mut wb = Workbook::new();

    let title = Format::new().bold().font_size(14.0).font_color("#1F4E79");
    let hdr = Format::new().bold().background_color("#4472C4").font_color("#FFFFFF").border(BorderStyle::Thin);
    let cell = Format::new().border(BorderStyle::Thin);
    let money = Format::new().num_format("$#,##0").border(BorderStyle::Thin);
    let pct = Format::new().num_format("0.0%").border(BorderStyle::Thin);

    // ═══════════════════════════════════════════════════════════
    // Sheet 1: Formula-Based CF — alternating row shading
    // ═══════════════════════════════════════════════════════════
    {
    let ws = wb.worksheet(0)?;
    ws.set_name("Formula CF")?;
    ws.set_tab_color("#4472C4");
    ws.write_with_format(0, 0, "Formula-Based CF: Alternating Row Shading", &title)?;
    for (c, h) in ["Employee", "Department", "Salary"].iter().enumerate() {
        ws.write_with_format(2, c as u16, *h, &hdr)?;
    }
    let data = [
        ("Alice", "Engineering", 95000.0), ("Bob", "Marketing", 78000.0),
        ("Charlie", "Engineering", 102000.0), ("Diana", "Sales", 85000.0),
        ("Eve", "Finance", 91000.0), ("Frank", "Marketing", 63000.0),
        ("Grace", "Engineering", 88000.0), ("Hank", "Sales", 72000.0),
    ];
    for (i, (name, dept, sal)) in data.iter().enumerate() {
        let r = 3 + i as u32;
        ws.write_with_format(r, 0, *name, &cell)?;
        ws.write_with_format(r, 1, *dept, &cell)?;
        ws.write_with_format(r, 2, *sal, &money)?;
    }
    // Formula CF: even rows get light blue
    let mut cf = ConditionalFormatFormula::new("MOD(ROW(),2)=0");
    cf.set_format(&Format::new().background_color("#D6E4F0"));
    ws.add_conditional_format(3, 0, 10, 2, cf)?;
    ws.set_column_width(0, 14.0)?;
    ws.set_column_width(1, 14.0)?;
    ws.set_column_width(2, 12.0)?;
    }

    // ═══════════════════════════════════════════════════════════
    // Sheet 2: Above/Below Average CF
    // ═══════════════════════════════════════════════════════════
    {
    let ws = wb.add_worksheet_with_name("Average CF")?;
    ws.set_tab_color("#70AD47");
    ws.write_with_format(0, 0, "Above/Below Average CF: Sales Performance", &title)?;
    for (c, h) in ["Rep", "Q1 Sales"].iter().enumerate() {
        ws.write_with_format(2, c as u16, *h, &hdr)?;
    }
    let reps = [("Alice", 45000.0), ("Bob", 32000.0), ("Charlie", 58000.0),
        ("Diana", 27000.0), ("Eve", 51000.0), ("Frank", 39000.0)];
    for (i, (name, val)) in reps.iter().enumerate() {
        let r = 3 + i as u32;
        ws.write_with_format(r, 0, *name, &cell)?;
        ws.write_with_format(r, 1, *val, &money)?;
    }
    // Above average = green
    let mut cf_above = ConditionalFormatAverage::new(AverageType::Above);
    cf_above.set_format(&Format::new().font_color("#006100").background_color("#C6EFCE"));
    ws.add_conditional_format(3, 1, 8, 1, cf_above)?;
    // Below average = red
    let mut cf_below = ConditionalFormatAverage::new(AverageType::Below);
    cf_below.set_format(&Format::new().font_color("#9C0006").background_color("#FFC7CE"));
    ws.add_conditional_format(3, 1, 8, 1, cf_below)?;
    ws.set_column_width(0, 12.0)?;
    ws.set_column_width(1, 14.0)?;
    }

    // ═══════════════════════════════════════════════════════════
    // Sheet 3: Top/Bottom N CF
    // ═══════════════════════════════════════════════════════════
    {
    let ws = wb.add_worksheet_with_name("Top Bottom CF")?;
    ws.set_tab_color("#FFC000");
    ws.write_with_format(0, 0, "Top/Bottom CF: Top 3 & Bottom 2 Scores", &title)?;
    for (c, h) in ["Student", "Score"].iter().enumerate() {
        ws.write_with_format(2, c as u16, *h, &hdr)?;
    }
    let students = [("Alice", 95.0), ("Bob", 72.0), ("Charlie", 88.0), ("Diana", 45.0),
        ("Eve", 91.0), ("Frank", 63.0), ("Grace", 78.0), ("Hank", 55.0)];
    for (i, (name, score)) in students.iter().enumerate() {
        let r = 3 + i as u32;
        ws.write_with_format(r, 0, *name, &cell)?;
        ws.write_with_format(r, 1, *score, &cell)?;
    }
    // Top 3 = bold green
    let mut cf_top = ConditionalFormatTopBottom::new(TopBottomType::Top, 3);
    cf_top.set_format(&Format::new().bold().font_color("#006100").background_color("#C6EFCE"));
    ws.add_conditional_format(3, 1, 10, 1, cf_top)?;
    // Bottom 2 = bold red
    let mut cf_bot = ConditionalFormatTopBottom::new(TopBottomType::Bottom, 2);
    cf_bot.set_format(&Format::new().bold().font_color("#9C0006").background_color("#FFC7CE"));
    ws.add_conditional_format(3, 1, 10, 1, cf_bot)?;
    ws.set_column_width(0, 12.0)?;
    ws.set_column_width(1, 10.0)?;
    }

    // ═══════════════════════════════════════════════════════════
    // Sheet 4: Text Contains CF
    // ═══════════════════════════════════════════════════════════
    {
    let ws = wb.add_worksheet_with_name("Text CF")?;
    ws.set_tab_color("#ED7D31");
    ws.write_with_format(0, 0, "Text Contains CF: Highlight 'Engineering' rows", &title)?;
    for (c, h) in ["Name", "Department", "Status"].iter().enumerate() {
        ws.write_with_format(2, c as u16, *h, &hdr)?;
    }
    let rows = [
        ("Alice", "Engineering", "Active"), ("Bob", "Marketing", "Active"),
        ("Charlie", "Engineering", "On Leave"), ("Diana", "Sales", "Active"),
        ("Eve", "Engineering", "Active"), ("Frank", "HR", "Active"),
    ];
    for (i, (n, d, s)) in rows.iter().enumerate() {
        let r = 3 + i as u32;
        ws.write_with_format(r, 0, *n, &cell)?;
        ws.write_with_format(r, 1, *d, &cell)?;
        ws.write_with_format(r, 2, *s, &cell)?;
    }
    let mut cf = ConditionalFormatText::new(TextOperator::Contains, "Engineering");
    cf.set_format(&Format::new().background_color("#D6E4F0").bold());
    ws.add_conditional_format(3, 1, 8, 1, cf)?;
    ws.set_column_width(0, 12.0)?;
    ws.set_column_width(1, 14.0)?;
    ws.set_column_width(2, 10.0)?;
    }

    // ═══════════════════════════════════════════════════════════
    // Sheet 5: Duplicate Values CF
    // ═══════════════════════════════════════════════════════════
    {
    let ws = wb.add_worksheet_with_name("Duplicates CF")?;
    ws.set_tab_color("#A5A5A5");
    ws.write_with_format(0, 0, "Duplicate Values CF: Spot repeated entries", &title)?;
    ws.write_with_format(2, 0, "Invoice #", &hdr)?;
    let invoices = ["INV-001", "INV-002", "INV-003", "INV-001", "INV-004", "INV-002", "INV-005", "INV-003"];
    for (i, inv) in invoices.iter().enumerate() {
        ws.write_with_format(3 + i as u32, 0, *inv, &cell)?;
    }
    let mut cf = ConditionalFormatDuplicate::new();
    cf.set_format(&Format::new().font_color("#9C5700").background_color("#FFEB9C").bold());
    ws.add_conditional_format(3, 0, 10, 0, cf)?;
    ws.set_column_width(0, 14.0)?;
    }

    // ═══════════════════════════════════════════════════════════
    // Sheet 6: Date Occurring CF
    // ═══════════════════════════════════════════════════════════
    {
    let ws = wb.add_worksheet_with_name("Date CF")?;
    ws.set_tab_color("#5B9BD5");
    ws.write_with_format(0, 0, "Date Occurring CF: Highlights 'Today' dates", &title)?;
    let date_fmt = Format::new().num_format("yyyy-mm-dd").border(BorderStyle::Thin);
    ws.write_with_format(2, 0, "Due Date", &hdr)?;
    // Write a range of dates around today
    for i in 0..10u32 {
        let day = 1 + i;
        let dt = ExcelDateTime::from_ymd(2026, 4, day).unwrap();
        ws.write_with_format(3 + i, 0, dt, &date_fmt)?;
    }
    let mut cf = ConditionalFormatDate::new(DateOccurring::Today);
    cf.set_format(&Format::new().bold().font_color("#FFFFFF").background_color("#4472C4"));
    ws.add_conditional_format(3, 0, 12, 0, cf)?;
    ws.set_column_width(0, 14.0)?;
    }

    // ═══════════════════════════════════════════════════════════
    // Sheet 7: Combo Chart — Revenue bars + Margin % line
    // ═══════════════════════════════════════════════════════════
    {
    let ws = wb.add_worksheet_with_name("Combo Chart")?;
    ws.set_tab_color("#1F4E79");
    ws.write_with_format(0, 0, "Combo Chart: Revenue (bars) + Margin % (line, secondary axis)", &title)?;

    for (c, h) in ["Quarter", "Revenue", "Margin %"].iter().enumerate() {
        ws.write_with_format(2, c as u16, *h, &hdr)?;
    }
    let quarters = [("Q1", 180000.0, 0.089), ("Q2", 210000.0, 0.117),
        ("Q3", 245000.0, 0.139), ("Q4", 270000.0, 0.155)];
    for (i, (q, rev, margin)) in quarters.iter().enumerate() {
        let r = 3 + i as u32;
        ws.write_with_format(r, 0, *q, &cell)?;
        ws.write_with_format(r, 1, *rev, &money)?;
        ws.write_with_format(r, 2, *margin, &pct)?;
    }
    ws.set_column_width(0, 10.0)?;
    ws.set_column_width(1, 14.0)?;
    ws.set_column_width(2, 12.0)?;

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Revenue & Profit Margin");
    chart.set_y_axis_name("Revenue ($)");
    chart.set_y2_axis_name("Margin %");
    chart.set_width(640);
    chart.set_height(400);
    chart.add_series()
        .set_categories("'Combo Chart'!$A$4:$A$7")
        .set_values("'Combo Chart'!$B$4:$B$7")
        .set_name("Revenue")
        .set_data_labels(true)
        .set_trendline(TrendlineType::Linear);
    chart.add_series()
        .set_categories("'Combo Chart'!$A$4:$A$7")
        .set_values("'Combo Chart'!$C$4:$C$7")
        .set_name("Margin %")
        .set_chart_type(ChartType::Line)
        .set_secondary_axis(true)
        .set_data_labels(true);
    ws.insert_chart(8, 0, &chart)?;
    }

    // ═══════════════════════════════════════════════════════════
    // Sheet 8: Data Labels + Trendline
    // ═══════════════════════════════════════════════════════════
    {
    let ws = wb.add_worksheet_with_name("Trendline")?;
    ws.set_tab_color("#70AD47");
    ws.write_with_format(0, 0, "Chart with Data Labels + Linear Trendline", &title)?;

    for (c, h) in ["Month", "Users"].iter().enumerate() {
        ws.write_with_format(2, c as u16, *h, &hdr)?;
    }
    let months = [("Jan", 1200.0), ("Feb", 1450.0), ("Mar", 1380.0), ("Apr", 1620.0),
        ("May", 1800.0), ("Jun", 2100.0)];
    for (i, (m, v)) in months.iter().enumerate() {
        let r = 3 + i as u32;
        ws.write_with_format(r, 0, *m, &cell)?;
        ws.write_with_format(r, 1, *v, &cell)?;
    }
    ws.set_column_width(0, 10.0)?;
    ws.set_column_width(1, 10.0)?;

    let mut chart = Chart::new(ChartType::Line);
    chart.set_title("Monthly Active Users");
    chart.set_width(640);
    chart.set_height(400);
    chart.add_series()
        .set_categories("Trendline!$A$4:$A$9")
        .set_values("Trendline!$B$4:$B$9")
        .set_name("Users")
        .set_data_labels(true)
        .set_trendline(TrendlineType::Linear);
    ws.insert_chart(10, 0, &chart)?;
    }

    wb.save(path)?;

    // ── Validate ──
    let s1 = read_sheet_xml(path, 1);
    assert!(s1.contains("type=\"expression\""), "Sheet 1: formula CF");
    let s2 = read_sheet_xml(path, 2);
    assert!(s2.contains("type=\"aboveAverage\""), "Sheet 2: above-average CF");
    let s3 = read_sheet_xml(path, 3);
    assert!(s3.contains("type=\"top10\""), "Sheet 3: top/bottom CF");
    let s4 = read_sheet_xml(path, 4);
    assert!(s4.contains("type=\"containsText\""), "Sheet 4: text CF");
    let s5 = read_sheet_xml(path, 5);
    assert!(s5.contains("type=\"duplicateValues\""), "Sheet 5: duplicate CF");
    let s6 = read_sheet_xml(path, 6);
    assert!(s6.contains("type=\"timePeriod\""), "Sheet 6: date CF");

    let entries = list_zip_entries(path);
    let charts: Vec<&String> = entries.iter().filter(|e| e.starts_with("xl/charts/")).collect();
    assert!(charts.len() >= 2, "Should have 2+ charts");
    let combo_xml = read_zip_entry(path, charts[0]);
    assert!(combo_xml.contains("c:lineChart") && combo_xml.contains("c:barChart"), "Combo chart");
    assert!(combo_xml.contains("c:dLbls"), "Data labels");
    assert!(combo_xml.contains("c:trendline"), "Trendline");

    let styles = read_zip_entry(path, "xl/styles.xml");
    assert!(styles.contains("<dxfs"), "dxf support");

    let size = std::fs::metadata(path)?.len();
    println!("    📊 Phase 6 Showcase: {:.1} KB, 8 sheets, 6 CF types + combo chart + trendline",
        size as f64 / 1024.0);
    Ok(())
}

fn col_letter(col: u16) -> String {
    let mut result = String::new();
    let mut c = col;
    loop {
        result.insert(0, (b'A' + (c % 26) as u8) as char);
        if c < 26 { break; }
        c = c / 26 - 1;
    }
    result
}

fn list_zip_entries(path: &Path) -> Vec<String> {
    let file = std::fs::File::open(path).unwrap();
    let archive = zip::ZipArchive::new(std::io::BufReader::new(file)).unwrap();
    (0..archive.len()).map(|i| archive.name_for_index(i).unwrap().to_string()).collect()
}

fn test_sprint7_showcase(path: &Path) -> Result<()> {
    use zavora_xlsx::*;

    let mut wb = Workbook::new();

    // Sheet 1: Active Sheet + Selection + Top-Left Cell
    let ws = wb.worksheet(0)?;
    ws.set_name("Dashboard")?;
    ws.write(0, 0, "This sheet opens first with cursor at B5")?;
    ws.write(4, 1, "← Cursor here")?;
    ws.set_selection(4, 1);
    ws.set_top_left_cell(0, 0);

    // Sheet 2: Write Blank + Clear Cell
    let ws2 = wb.add_worksheet_with_name("Blank & Clear")?;
    ws2.write(0, 0, "Header")?;
    let border_fmt = Format::new().border(BorderStyle::Thin).background_color("#E2EFDA");
    ws2.write_blank(1, 0, &border_fmt)?;
    ws2.write_blank(1, 1, &border_fmt)?;
    ws2.write_blank(1, 2, &border_fmt)?;
    ws2.write(2, 0, "This row has data")?;
    ws2.write(2, 1, "To be cleared")?;
    ws2.clear_cell(2, 1); // removes "To be cleared"
    ws2.write(3, 0, "Row 3 col B should be empty above")?;

    // Sheet 3: Column & Row Format
    let ws3 = wb.add_worksheet_with_name("Col & Row Fmt")?;
    let currency_fmt = Format::new().num_format("$#,##0.00");
    let header_fmt = Format::new().bold().background_color("#4472C4").font_color("#FFFFFF");
    ws3.set_column_format(1, &currency_fmt);
    ws3.set_row_format(0, &header_fmt);
    ws3.write(0, 0, "Item")?;
    ws3.write(0, 1, "Amount")?;
    ws3.write(1, 0, "Revenue")?;
    ws3.write(1, 1, 50000.0)?;
    ws3.write(2, 0, "Expenses")?;
    ws3.write(2, 1, 32000.0)?;
    ws3.set_column_width(0, 15.0)?;
    ws3.set_column_width(1, 15.0)?;

    // Sheet 4: Default Row Height
    let ws4 = wb.add_worksheet_with_name("Row Height")?;
    ws4.set_default_row_height(24.0);
    ws4.write(0, 0, "All rows are 24pt tall")?;
    ws4.write(1, 0, "Row 2")?;
    ws4.write(2, 0, "Row 3")?;

    // Sheet 5: Hidden + Very Hidden
    let ws5 = wb.add_worksheet_with_name("Hidden Sheet")?;
    ws5.write(0, 0, "This sheet is hidden")?;
    ws5.set_hidden();

    let ws6 = wb.add_worksheet_with_name("VeryHidden")?;
    ws6.write(0, 0, "This sheet is very hidden")?;
    ws6.set_very_hidden();

    // Sheet 7: Ignore Errors
    let ws7 = wb.add_worksheet_with_name("Ignore Errors")?;
    ws7.write(0, 0, "ID")?;
    ws7.write(1, 0, "001")?;
    ws7.write(2, 0, "002")?;
    ws7.write(3, 0, "003")?;
    ws7.ignore_error("numberStoredAsText", "A2:A4");
    ws7.write(0, 1, "Note: no green triangles on IDs")?;

    // Sheet 8: Filter Column
    let ws8 = wb.add_worksheet_with_name("Filtered")?;
    ws8.write(0, 0, "Name")?;
    ws8.write(0, 1, "Status")?;
    ws8.write(0, 2, "Amount")?;
    for (i, (name, status, amt)) in [("Alice", "Active", 100.0), ("Bob", "Pending", 200.0),
        ("Carol", "Active", 150.0), ("Dave", "Closed", 300.0)].iter().enumerate() {
        ws8.write(i as u32 + 1, 0, *name)?;
        ws8.write(i as u32 + 1, 1, *status)?;
        ws8.write(i as u32 + 1, 2, *amt)?;
    }
    ws8.set_autofilter(0, 0, 4, 2);
    ws8.filter_column(1, &["Active"]);

    // Set Dashboard as active sheet (index 0)
    wb.set_active_sheet(0);

    wb.save(path)?;

    // Validate: check file opens and has correct sheet count
    let wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.sheet_count(), 8);
    assert_eq!(wb2.sheet_names()[0], "Dashboard");
    assert_eq!(wb2.sheet_names()[4], "Hidden Sheet");

    Ok(())
}

fn test_sprint8_showcase(path: &Path) -> Result<()> {
    use zavora_xlsx::*;

    let mut wb = Workbook::new();

    // Sheet 1: Array Formulas
    let ws = wb.worksheet(0)?;
    ws.set_name("Array Formulas")?;
    ws.write(0, 0, "Matrix A")?;
    ws.write(1, 0, 1.0)?; ws.write(1, 1, 2.0)?;
    ws.write(2, 0, 3.0)?; ws.write(2, 1, 4.0)?;
    ws.write(0, 3, "Matrix B")?;
    ws.write(1, 3, 5.0)?; ws.write(1, 4, 6.0)?;
    ws.write(2, 3, 7.0)?; ws.write(2, 4, 8.0)?;
    ws.write(0, 6, "A × B (array formula)")?;
    ws.write_array_formula(1, 6, 2, 7, "MMULT(A2:B3,D2:E3)")?;
    ws.set_column_width(0, 10.0)?;
    ws.set_column_width(6, 22.0)?;

    // Sheet 2: Dynamic Formulas
    let ws2 = wb.add_worksheet_with_name("Dynamic Formulas")?;
    ws2.write(0, 0, "Name")?;
    for (i, name) in ["Alice", "Bob", "Alice", "Carol", "Bob", "Alice"].iter().enumerate() {
        ws2.write(i as u32 + 1, 0, *name)?;
    }
    ws2.write(0, 2, "Unique Names (spill)")?;
    ws2.write_dynamic_formula(1, 2, "_xlfn.UNIQUE(A2:A7)")?;
    ws2.write(0, 4, "Sorted (spill)")?;
    ws2.write_dynamic_formula(1, 4, "_xlfn._xlws.SORT(A2:A7)")?;
    ws2.set_column_width(2, 20.0)?;
    ws2.set_column_width(4, 18.0)?;

    // Sheet 3: Formula with Cached Result
    let ws3 = wb.add_worksheet_with_name("Cached Results")?;
    ws3.write(0, 0, "Value")?; ws3.write(0, 1, "Formula")?; ws3.write(0, 2, "Result")?;
    ws3.write(1, 0, 100.0)?;
    ws3.write(2, 0, 200.0)?;
    ws3.write(3, 0, 300.0)?;
    ws3.write_formula_with_result(1, 1, "A2*2", 200.0)?;
    ws3.write_formula_with_result(2, 1, "A3*2", 400.0)?;
    ws3.write_formula_with_result(3, 1, "SUM(A2:A4)", 600.0)?;
    ws3.write(4, 0, "↑ Shows values immediately without recalc")?;

    // Sheet 4: Print Options
    let ws4 = wb.add_worksheet_with_name("Print Options")?;
    ws4.write(0, 0, "This sheet has print gridlines, headings, and centering")?;
    for r in 1..20u32 {
        ws4.write(r, 0, format!("Row {r}"))?;
        ws4.write(r, 1, r as f64 * 10.0)?;
    }
    let ps = PrintSettings::new()
        .print_gridlines(true)
        .print_headings(true)
        .center_horizontally(true)
        .orientation(Orientation::Landscape)
        .first_page_number(5);
    ws4.set_print_settings(&ps);

    // Sheet 5: Unprotect Range
    let ws5 = wb.add_worksheet_with_name("Protected + Input")?;
    ws5.write(0, 0, "This sheet is protected but B2:B5 is editable")?;
    ws5.write(1, 0, "Name")?; ws5.write(1, 1, "Enter here →")?;
    ws5.write(2, 0, "Email")?; ws5.write(2, 1, "")?;
    ws5.write(3, 0, "Phone")?; ws5.write(3, 1, "")?;
    ws5.write(4, 0, "Notes")?; ws5.write(4, 1, "")?;
    ws5.protect_with_password("test");
    ws5.unprotect_range("Inputs", "B2:B5");
    ws5.set_column_width(0, 12.0)?;
    ws5.set_column_width(1, 25.0)?;

    // Sheet 6: Open from Buffer test
    let ws6 = wb.add_worksheet_with_name("Buffer Test")?;
    ws6.write(0, 0, "This workbook was also tested via open_from_buffer")?;

    wb.save(path)?;

    // Validate: open from buffer
    let bytes = std::fs::read(path)?;
    let wb2 = Workbook::open_readonly_from_buffer(&bytes)?;
    assert_eq!(wb2.sheet_count(), 6);
    assert_eq!(wb2.sheet_names()[0], "Array Formulas");

    Ok(())
}

fn test_sprint9_showcase(path: &Path) -> Result<()> {
    use zavora_xlsx::*;

    let mut wb = Workbook::new();

    // Sheet 1: Diagonal Borders
    let ws = wb.worksheet(0)?;
    ws.set_name("Diagonal Borders")?;
    ws.write(0, 0, "Up diagonal")?;
    ws.write(0, 1, "Down diagonal")?;
    ws.write(0, 2, "Both")?;
    let fmt_up = Format::new().diagonal_border(BorderStyle::Thin, DiagonalType::Up).border(BorderStyle::Thin);
    let fmt_down = Format::new().diagonal_border(BorderStyle::Medium, DiagonalType::Down).border(BorderStyle::Thin);
    let fmt_both = Format::new().diagonal_border(BorderStyle::Thick, DiagonalType::Both).border(BorderStyle::Thin);
    ws.write_blank(1, 0, &fmt_up)?;
    ws.write_blank(1, 1, &fmt_down)?;
    ws.write_blank(1, 2, &fmt_both)?;
    ws.set_column_width(0, 18.0)?;
    ws.set_column_width(1, 18.0)?;
    ws.set_column_width(2, 18.0)?;

    // Sheet 2: Pattern Fills
    let ws2 = wb.add_worksheet_with_name("Pattern Fills")?;
    let patterns = [
        ("Solid", Pattern::Solid), ("MediumGray", Pattern::MediumGray),
        ("DarkGray", Pattern::DarkGray), ("LightGray", Pattern::LightGray),
        ("DarkHorizontal", Pattern::DarkHorizontal), ("DarkVertical", Pattern::DarkVertical),
        ("DarkDown", Pattern::DarkDown), ("DarkUp", Pattern::DarkUp),
        ("LightHorizontal", Pattern::LightHorizontal), ("LightVertical", Pattern::LightVertical),
    ];
    ws2.write(0, 0, "Pattern")?;
    ws2.write(0, 1, "Sample")?;
    for (i, (name, pat)) in patterns.iter().enumerate() {
        let r = i as u32 + 1;
        ws2.write(r, 0, *name)?;
        let fmt = Format::new().pattern_fill(*pat).foreground_color("#4472C4").background_color("#D9E2F3");
        ws2.write_blank(r, 1, &fmt)?;
    }
    ws2.set_column_width(0, 20.0)?;
    ws2.set_column_width(1, 20.0)?;
    ws2.set_row_height(0, 20.0)?;

    // Sheet 3: Superscript / Subscript
    let ws3 = wb.add_worksheet_with_name("Super & Subscript")?;
    let rt1 = RichText::new()
        .add_styled("E = mc", RichTextRun::new())
        .add_styled("2", RichTextRun::new().superscript());
    ws3.write_rich_text(0, 0, &rt1)?;

    let rt2 = RichText::new()
        .add_styled("H", RichTextRun::new())
        .add_styled("2", RichTextRun::new().subscript())
        .add_styled("O", RichTextRun::new());
    ws3.write_rich_text(1, 0, &rt2)?;

    let rt3 = RichText::new()
        .add_styled("Footnote", RichTextRun::new())
        .add_styled("1", RichTextRun::new().superscript().font_size(8.0));
    ws3.write_rich_text(2, 0, &rt3)?;
    ws3.set_column_width(0, 25.0)?;

    // Sheet 4: Quote Prefix
    let ws4 = wb.add_worksheet_with_name("Quote Prefix")?;
    ws4.write(0, 0, "Normal")?;
    ws4.write(0, 1, "Quote Prefix")?;
    ws4.write(1, 0, "001234")?;
    let qp = Format::new().quote_prefix();
    ws4.write_with_format(1, 1, "001234", &qp)?;
    ws4.write(2, 0, "Note: Quote prefix forces text display")?;
    ws4.set_column_width(0, 18.0)?;
    ws4.set_column_width(1, 18.0)?;

    wb.save(path)?;

    let wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.sheet_count(), 4);
    Ok(())
}

fn test_sprint10_showcase(path: &Path) -> Result<()> {
    use zavora_xlsx::*;

    // Create a file with various features to read back
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0)?;
        ws.set_name("ReadTest")?;
        ws.write(0, 0, "Header A")?;
        ws.write(0, 1, "Header B")?;
        ws.write(1, 0, "Data 1")?;
        ws.write(1, 1, 100.0)?;
        ws.merge_range(3, 0, 3, 2, "Merged", &Format::new().bold())?;
        ws.set_column_width(0, 25.0)?;
        ws.set_row_height(0, 30.0)?;
        ws.set_freeze_panes(1, 0)?;

        let ws2 = wb.add_worksheet_with_name("Hidden")?;
        ws2.write(0, 0, "Hidden sheet")?;
        ws2.set_hidden();

        let ws3 = wb.add_worksheet_with_name("VeryHidden")?;
        ws3.write(0, 0, "Very hidden")?;
        ws3.set_very_hidden();

        wb.save(path)?;
    }

    // Read back and verify all metadata
    let mut wb = Workbook::open_readonly(path)?;
    assert_eq!(wb.sheet_count(), 3);

    let ws = wb.worksheet(0)?;
    assert_eq!(ws.name(), "ReadTest");

    // Verify merge ranges read back
    assert_eq!(ws.merge_ranges().len(), 1);
    assert_eq!(ws.merge_ranges()[0], (3, 0, 3, 2));

    // Verify column width read back
    let w = ws.column_width(0);
    assert!(w.is_some(), "Column width should be read back");
    assert!((w.unwrap() - 25.0).abs() < 0.5, "Column width should be ~25, got {:?}", w);

    // Verify row height read back
    let h = ws.row_height(0);
    assert!(h.is_some(), "Row height should be read back");
    assert!((h.unwrap() - 30.0).abs() < 0.5, "Row height should be ~30, got {:?}", h);

    // Verify visibility
    let ws2 = wb.worksheet(1)?;
    assert!(ws2.is_hidden(), "Sheet 2 should be hidden");

    let ws3 = wb.worksheet(2)?;
    assert!(ws3.is_very_hidden(), "Sheet 3 should be very hidden");

    // Verify open from buffer works with metadata
    let bytes = std::fs::read(path)?;
    let mut wb2 = Workbook::open_readonly_from_buffer(&bytes)?;
    let ws_buf = wb2.worksheet(0)?;
    assert_eq!(ws_buf.merge_ranges().len(), 1);

    Ok(())
}

fn test_sprint11_showcase(path: &Path) -> Result<()> {
    use zavora_xlsx::*;

    let mut wb = Workbook::new();

    // Sheet 1: Chart with pixel offset + data table
    let ws = wb.worksheet(0)?;
    ws.set_name("Chart Offset + Table")?;
    ws.write(0, 0, "Quarter")?;
    ws.write(0, 1, "Revenue")?;
    ws.write(0, 2, "Profit")?;
    for (i, (q, r, p)) in [("Q1", 100.0, 20.0), ("Q2", 150.0, 35.0), ("Q3", 130.0, 28.0), ("Q4", 180.0, 45.0)].iter().enumerate() {
        ws.write(i as u32 + 1, 0, *q)?;
        ws.write(i as u32 + 1, 1, *r)?;
        ws.write(i as u32 + 1, 2, *p)?;
    }
    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Revenue & Profit");
    chart.add_series().set_values("'Chart Offset + Table'!$B$2:$B$5").set_categories("'Chart Offset + Table'!$A$2:$A$5").set_name("Revenue");
    chart.add_series().set_values("'Chart Offset + Table'!$C$2:$C$5").set_name("Profit");
    chart.show_data_table(true);
    chart.set_width(600);
    chart.set_height(400);
    ws.insert_chart_with_offset(6, 0, &chart, 20, 10)?;

    // Sheet 2: Image scale
    let ws2 = wb.add_worksheet_with_name("Image Scale")?;
    ws2.write(0, 0, "Original size and 50% scaled PNG")?;
    let png = std::fs::read("/tmp/test_logo.png").unwrap_or_else(|_| create_test_png());
    let mut img = Image::from_buffer(&png)?;
    ws2.insert_image(2, 0, &img)?;
    img.set_scale_width(0.5);
    img.set_scale_height(0.5);
    ws2.insert_image(12, 0, &img)?;
    ws2.write(1, 0, "Original:")?;
    ws2.write(11, 0, "50% scaled:")?;

    // Sheet 3: Stock chart (High-Low-Close)
    let ws3 = wb.add_worksheet_with_name("Stock Chart")?;
    ws3.write(0, 0, "Date")?;
    ws3.write(0, 1, "High")?;
    ws3.write(0, 2, "Low")?;
    ws3.write(0, 3, "Close")?;
    let stock_data = [
        ("Mon", 45.0, 38.0, 42.0), ("Tue", 48.0, 40.0, 44.0),
        ("Wed", 46.0, 39.0, 41.0), ("Thu", 50.0, 42.0, 48.0), ("Fri", 52.0, 44.0, 50.0),
    ];
    for (i, (d, h, l, c)) in stock_data.iter().enumerate() {
        let r = i as u32 + 1;
        ws3.write(r, 0, *d)?;
        ws3.write(r, 1, *h)?;
        ws3.write(r, 2, *l)?;
        ws3.write(r, 3, *c)?;
    }
    let mut stock = Chart::new(ChartType::Stock);
    stock.set_title("Stock Price (HLC)");
    stock.add_series().set_values("'Stock Chart'!$B$2:$B$6").set_categories("'Stock Chart'!$A$2:$A$6").set_name("High");
    stock.add_series().set_values("'Stock Chart'!$C$2:$C$6").set_name("Low");
    stock.add_series().set_values("'Stock Chart'!$D$2:$D$6").set_name("Close");
    ws3.insert_chart(7, 0, &stock)?;

    wb.save(path)?;

    let wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.sheet_count(), 3);
    Ok(())
}


fn test_sprint12_showcase(path: &Path) -> Result<()> {
    use zavora_xlsx::*;

    let tmp = path.with_extension("tmp.xlsx");

    // Step 1: Create a file with charts, merges, widths, freeze, comments
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0)?;
        ws.set_name("Data")?;
        ws.write(0, 0, "Quarter")?;
        ws.write(0, 1, "Revenue")?;
        ws.write(1, 0, "Q1")?; ws.write(1, 1, 100.0)?;
        ws.write(2, 0, "Q2")?; ws.write(2, 1, 150.0)?;
        ws.write(3, 0, "Q3")?; ws.write(3, 1, 130.0)?;
        ws.set_column_width(0, 20.0)?;
        ws.set_column_width(1, 15.0)?;
        ws.set_row_height(0, 25.0)?;
        ws.set_freeze_panes(1, 0)?;
        ws.merge_range(5, 0, 5, 1, "Merged Cell", &Format::new().bold())?;
        ws.add_comment(1, 1, "Check this value");

        let mut chart = Chart::new(ChartType::Column);
        chart.set_title("Revenue");
        chart.add_series().set_values("Data!$B$2:$B$4").set_categories("Data!$A$2:$A$4").set_name("Rev");
        ws.insert_chart(7, 0, &chart)?;

        wb.save(&tmp)?;
    }

    // Step 2: Open in edit mode, modify a cell, save
    {
        let mut wb = Workbook::open(&tmp)?;
        let ws = wb.worksheet(0)?;

        // Verify metadata was read back
        assert_eq!(ws.merge_ranges().len(), 1, "Merge should be read back");
        assert!(ws.column_width(0).is_some(), "Col width should be read back");

        // Modify a cell (makes sheet dirty)
        ws.write(1, 1, 999.0)?;
        ws.write(4, 0, "Added in edit mode")?;

        wb.save(path)?;
    }

    // Step 3: Verify the saved file preserves everything
    let mut wb = Workbook::open_readonly(path)?;
    let ws = wb.worksheet(0)?;

    // Cell data preserved
    assert_eq!(ws.read_cell(1, 1), CellValue::Number(999.0));
    assert_eq!(ws.read_cell(2, 0), CellValue::String("Q2".into()));
    assert_eq!(ws.read_cell(4, 0), CellValue::String("Added in edit mode".into()));

    // Metadata preserved on dirty sheet
    assert_eq!(ws.merge_ranges().len(), 1, "Merge should survive edit");
    assert!(ws.column_width(0).is_some(), "Col width should survive edit");

    // Clean up temp
    let _ = std::fs::remove_file(&tmp);

    Ok(())
}

fn test_border_colors(path: &Path) -> Result<()> {
    use zavora_xlsx::*;
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Border Colors")?;

    ws.write(0, 0, "Per-side border colors")?;
    ws.set_column_width(0, 25.0)?;
    ws.set_column_width(1, 25.0)?;
    ws.set_column_width(2, 25.0)?;

    // Red top, blue bottom, green left, orange right
    let fmt1 = Format::new()
        .border(BorderStyle::Medium)
        .border_top_color("#FF0000")
        .border_bottom_color("#0000FF")
        .border_left_color("#00AA00")
        .border_right_color("#FF8800");
    ws.write_with_format(2, 0, "4 different colors", &fmt1)?;

    // Thick red top only
    let fmt2 = Format::new()
        .border_top(BorderStyle::Thick)
        .border_top_color("#FF0000");
    ws.write_with_format(2, 1, "Red top only", &fmt2)?;

    // All blue thin
    let fmt3 = Format::new()
        .border(BorderStyle::Thin)
        .border_color("#4472C4");
    ws.write_with_format(2, 2, "All blue (global)", &fmt3)?;

    // Double bottom with color
    let fmt4 = Format::new()
        .border_bottom(BorderStyle::Double)
        .border_bottom_color("#FF0000");
    ws.write_with_format(4, 0, "Double red bottom", &fmt4)?;

    ws.set_row_height(2, 30.0)?;
    ws.set_row_height(4, 30.0)?;
    wb.save(path)?;
    Ok(())
}

fn test_chart_advanced(path: &Path) -> Result<()> {
    use zavora_xlsx::*;
    let mut wb = Workbook::new();

    // Sheet 1: Axis control + markers
    let ws = wb.worksheet(0)?;
    ws.set_name("Axis & Markers")?;
    ws.write(0, 0, "X")?; ws.write(0, 1, "Y")?;
    for i in 1..=8u32 {
        ws.write(i, 0, format!("P{i}"))?;
        ws.write(i, 1, (i as f64) * (i as f64))?;
    }

    let mut chart = Chart::new(ChartType::Line);
    chart.set_title("Axis Control: min=0, max=80");
    chart.add_series()
        .set_values("'Axis & Markers'!$B$2:$B$9")
        .set_categories("'Axis & Markers'!$A$2:$A$9")
        .set_name("Squared")
        .set_marker(MarkerType::Diamond)
        .set_marker_size(8)
        .set_color("#FF6600");
    chart.set_y_axis_min(0.0);
    chart.set_y_axis_max(80.0);
    chart.set_y_axis_name("Value");
    ws.insert_chart(10, 0, &chart)?;

    // Sheet 2: Point colors + trendline R²
    let ws2 = wb.add_worksheet_with_name("Colors & R²")?;
    ws2.write(0, 0, "Category")?; ws2.write(0, 1, "Value")?;
    let data = [("A", 10.0), ("B", 25.0), ("C", 18.0), ("D", 40.0), ("E", 35.0)];
    for (i, (cat, val)) in data.iter().enumerate() {
        ws2.write(i as u32 + 1, 0, *cat)?;
        ws2.write(i as u32 + 1, 1, *val)?;
    }

    let mut chart2 = Chart::new(ChartType::Column);
    chart2.set_title("Point Colors + Trendline R²");
    chart2.add_series()
        .set_values("'Colors & R²'!$B$2:$B$6")
        .set_categories("'Colors & R²'!$A$2:$A$6")
        .set_name("Sales")
        .set_point_color(0, "#4472C4")
        .set_point_color(1, "#ED7D31")
        .set_point_color(2, "#A5A5A5")
        .set_point_color(3, "#FFC000")
        .set_point_color(4, "#5B9BD5")
        .set_trendline(TrendlineType::Linear)
        .set_trendline_display_rsquared(true)
        .set_trendline_display_equation(true);
    ws2.insert_chart(7, 0, &chart2)?;

    // Sheet 3: Reversed axis
    let ws3 = wb.add_worksheet_with_name("Reversed Axis")?;
    ws3.write(0, 0, "Item")?; ws3.write(0, 1, "Score")?;
    for (i, (item, score)) in [("Alpha", 90.0), ("Beta", 75.0), ("Gamma", 60.0), ("Delta", 85.0)].iter().enumerate() {
        ws3.write(i as u32 + 1, 0, *item)?;
        ws3.write(i as u32 + 1, 1, *score)?;
    }
    let mut chart3 = Chart::new(ChartType::Bar);
    chart3.set_title("Reversed Y Axis");
    chart3.add_series()
        .set_values("'Reversed Axis'!$B$2:$B$5")
        .set_categories("'Reversed Axis'!$A$2:$A$5")
        .set_name("Score");
    chart3.set_y_axis_reverse();
    ws3.insert_chart(6, 0, &chart3)?;

    wb.save(path)?;
    let wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.sheet_count(), 3);
    Ok(())
}

fn test_all_patterns(path: &Path) -> Result<()> {
    use zavora_xlsx::*;
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Patterns & Diagonals")?;
    ws.set_column_width(0, 22.0)?;
    ws.set_column_width(1, 22.0)?;
    ws.set_column_width(2, 22.0)?;

    // All 18 patterns
    let patterns: Vec<(&str, Pattern)> = vec![
        ("Solid", Pattern::Solid), ("MediumGray", Pattern::MediumGray),
        ("DarkGray", Pattern::DarkGray), ("LightGray", Pattern::LightGray),
        ("DarkHorizontal", Pattern::DarkHorizontal), ("DarkVertical", Pattern::DarkVertical),
        ("DarkDown", Pattern::DarkDown), ("DarkUp", Pattern::DarkUp),
        ("DarkGrid", Pattern::DarkGrid), ("DarkTrellis", Pattern::DarkTrellis),
        ("LightHorizontal", Pattern::LightHorizontal), ("LightVertical", Pattern::LightVertical),
        ("LightDown", Pattern::LightDown), ("LightUp", Pattern::LightUp),
        ("LightGrid", Pattern::LightGrid), ("LightTrellis", Pattern::LightTrellis),
        ("Gray125", Pattern::Gray125),
    ];

    ws.write(0, 0, "Pattern")?;
    ws.write(0, 1, "Blue/White")?;
    ws.write(0, 2, "Red/Yellow")?;
    let header = Format::new().bold();
    ws.set_cell_format(0, 0, &header)?;
    ws.set_cell_format(0, 1, &header)?;
    ws.set_cell_format(0, 2, &header)?;

    for (i, (name, pat)) in patterns.iter().enumerate() {
        let r = i as u32 + 1;
        ws.write(r, 0, *name)?;
        let f1 = Format::new().pattern_fill(*pat).foreground_color("#4472C4").background_color("#FFFFFF");
        ws.write_blank(r, 1, &f1)?;
        let f2 = Format::new().pattern_fill(*pat).foreground_color("#FF0000").background_color("#FFFF00");
        ws.write_blank(r, 2, &f2)?;
        ws.set_row_height(r, 22.0)?;
    }

    // Diagonal borders section
    let r = patterns.len() as u32 + 2;
    ws.write(r, 0, "Diagonal Up")?;
    ws.write(r, 1, "Diagonal Down")?;
    ws.write(r, 2, "Diagonal Both")?;
    let d1 = Format::new().diagonal_border(BorderStyle::Thin, DiagonalType::Up).border(BorderStyle::Thin);
    let d2 = Format::new().diagonal_border(BorderStyle::Medium, DiagonalType::Down).border(BorderStyle::Thin);
    let d3 = Format::new().diagonal_border(BorderStyle::Thick, DiagonalType::Both).border(BorderStyle::Thin);
    ws.write_blank(r + 1, 0, &d1)?;
    ws.write_blank(r + 1, 1, &d2)?;
    ws.write_blank(r + 1, 2, &d3)?;
    ws.set_row_height(r + 1, 30.0)?;

    wb.save(path)?;
    Ok(())
}

fn test_formulas_complete(path: &Path) -> Result<()> {
    use zavora_xlsx::*;
    let mut wb = Workbook::new();

    // Sheet 1: All formula types
    let ws = wb.worksheet(0)?;
    ws.set_name("Formulas")?;
    ws.set_column_width(0, 20.0)?;
    ws.set_column_width(1, 15.0)?;
    ws.set_column_width(2, 25.0)?;

    ws.write(0, 0, "Type")?;
    ws.write(0, 1, "Result")?;
    ws.write(0, 2, "Notes")?;

    // Regular formula
    ws.write(1, 0, "Regular")?;
    ws.write_formula(1, 1, "1+2+3")?;
    ws.write(1, 2, "Should show 6")?;

    // Cached result
    ws.write(2, 0, "Cached")?;
    ws.write_formula_with_result(2, 1, "100*2.5", 250.0)?;
    ws.write(2, 2, "Shows 250 immediately")?;

    // Array formula
    ws.write(4, 0, "Matrix A")?;
    ws.write(5, 0, 1.0)?; ws.write(5, 1, 2.0)?;
    ws.write(6, 0, 3.0)?; ws.write(6, 1, 4.0)?;
    ws.write(4, 3, "A×A (array)")?;
    ws.write_array_formula(5, 3, 6, 4, "MMULT(A6:B7,A6:B7)")?;

    // Dynamic formula
    ws.write(8, 0, "Names")?;
    for (i, n) in ["Alice", "Bob", "Alice", "Carol"].iter().enumerate() {
        ws.write(9 + i as u32, 0, *n)?;
    }
    ws.write(8, 2, "Unique (spill)")?;
    ws.write_dynamic_formula(9, 2, "_xlfn.UNIQUE(A10:A13)")?;

    // Blank cells with formatting
    ws.write(14, 0, "Formatted blanks →")?;
    let border_fmt = Format::new().border(BorderStyle::Thin).background_color("#E2EFDA");
    ws.write_blank(14, 1, &border_fmt)?;
    ws.write_blank(14, 2, &border_fmt)?;

    // Superscript/subscript
    ws.write(16, 0, "Rich text:")?;
    let rt = RichText::new()
        .add_styled("x", RichTextRun::new().italic())
        .add_styled("2", RichTextRun::new().superscript().font_size(8.0))
        .add_styled(" + y", RichTextRun::new().italic())
        .add_styled("2", RichTextRun::new().superscript().font_size(8.0));
    ws.write_rich_text(16, 1, &rt)?;

    wb.save(path)?;
    let mut wb2 = Workbook::open_readonly(path)?;
    let ws2 = wb2.worksheet(0)?;
    assert_eq!(ws2.read_cell(2, 1), CellValue::Formula { formula: "100*2.5".into(), cached_value: Box::new(CellValue::Number(250.0)) });
    Ok(())
}

fn test_column_range(path: &Path) -> Result<()> {
    use zavora_xlsx::*;
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Column Range")?;

    // Headers
    ws.write(0, 0, "ID")?;
    ws.write(0, 1, "Jan")?;
    ws.write(0, 2, "Feb")?;
    ws.write(0, 3, "Mar")?;
    ws.write(0, 4, "Q1 Total")?;
    ws.write(0, 5, "Hidden1")?;
    ws.write(0, 6, "Hidden2")?;

    // Data
    for r in 1..=5u32 {
        ws.write(r, 0, format!("R{r}"))?;
        ws.write(r, 1, r as f64 * 100.0)?;
        ws.write(r, 2, r as f64 * 110.0)?;
        ws.write(r, 3, r as f64 * 120.0)?;
        ws.write_formula(r, 4, &format!("SUM(B{}:D{})", r + 1, r + 1))?;
        ws.write(r, 5, "hidden")?;
        ws.write(r, 6, "hidden")?;
    }

    // Column range width: B:D = 12
    ws.set_column_range_width(1, 3, 12.0);
    ws.set_column_width(0, 8.0)?;
    ws.set_column_width(4, 14.0)?;

    // Column range hidden: F:G
    ws.set_column_range_hidden(5, 6);

    // Row format: header row bold
    let bold_fmt = Format::new().bold().background_color("#4472C4").font_color("#FFFFFF");
    ws.set_row_format(0, &bold_fmt);

    // Column format: currency on E
    let currency = Format::new().num_format("$#,##0");
    ws.set_column_format(4, &currency);

    // Ignore errors on ID column
    ws.ignore_error("numberStoredAsText", "A2:A6");

    wb.save(path)?;
    Ok(())
}

fn test_pivot_table(path: &Path) -> Result<()> {
    use zavora_xlsx::*;

    let mut wb = Workbook::new();

    // Source data sheet
    let ws = wb.worksheet(0)?;
    ws.set_name("Sales Data")?;
    ws.write_row(0, 0, ["Region", "Product", "Quarter", "Revenue", "Units"])?;
    let data: Vec<(&str, &str, &str, f64, f64)> = vec![
        ("East", "Widget", "Q1", 10000.0, 150.0),
        ("East", "Widget", "Q2", 12000.0, 180.0),
        ("East", "Gadget", "Q1", 8000.0, 100.0),
        ("East", "Gadget", "Q2", 9500.0, 120.0),
        ("West", "Widget", "Q1", 15000.0, 200.0),
        ("West", "Widget", "Q2", 14000.0, 190.0),
        ("West", "Gadget", "Q1", 11000.0, 140.0),
        ("West", "Gadget", "Q2", 13000.0, 170.0),
        ("North", "Widget", "Q1", 7000.0, 90.0),
        ("North", "Widget", "Q2", 8500.0, 110.0),
        ("North", "Gadget", "Q1", 6000.0, 80.0),
        ("North", "Gadget", "Q2", 7500.0, 95.0),
    ];
    for (i, (region, product, quarter, revenue, units)) in data.iter().enumerate() {
        let r = i as u32 + 1;
        ws.write(r, 0, *region)?;
        ws.write(r, 1, *product)?;
        ws.write(r, 2, *quarter)?;
        ws.write(r, 3, *revenue)?;
        ws.write(r, 4, *units)?;
    }
    ws.set_column_width(0, 12.0)?;
    ws.set_column_width(1, 12.0)?;
    ws.set_column_width(3, 12.0)?;

    // Pivot table on second sheet
    let ws2 = wb.add_worksheet_with_name("Pivot Analysis")?;

    let pivot = PivotTable::new("SalesPivot", "'Sales Data'!$A$1:$E$13")
        .add_row_field("Region")
        .add_row_field("Product")
        .add_column_field("Quarter")
        .add_value_field("Revenue", PivotAggregation::Sum)
        .add_value_field("Units", PivotAggregation::Sum)
        .set_style_name("PivotStyleMedium9")
        .show_row_stripes(true)
        .show_grand_totals(true, true)
        .set_layout(PivotLayout::Tabular);

    ws2.add_pivot_table(0, 0, &pivot)?;

    // Pivot chart — linked to the pivot table on the same sheet
    let mut chart = Chart::new(ChartType::Column);
    chart.set_pivot_source("SalesPivot", "Pivot Analysis");
    chart.set_title("Revenue by Region & Quarter");
    chart.set_y_axis_name("Amount");
    chart.set_legend_position(LegendPosition::Bottom);
    chart.set_width(720);
    chart.set_height(400);
    // Series reference the pivot table output range (Excel rebuilds on refresh)
    chart.add_series()
        .set_name("Sum of Revenue")
        .set_values("'Pivot Analysis'!$C$2:$C$4")
        .set_categories("'Pivot Analysis'!$A$2:$A$4");
    ws2.insert_chart(2, 4, &chart)?;

    wb.set_active_sheet(1);
    wb.save(path)?;

    let wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.sheet_count(), 2);
    Ok(())
}

fn test_feature_parity(path: &Path) -> Result<()> {
    use zavora_xlsx::*;

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Financial Report")?;

    // DateTime
    let date_fmt = Format::new().num_format("yyyy-mm-dd");
    let datetime_fmt = Format::new().num_format("yyyy-mm-dd hh:mm:ss");
    ws.write(0, 0, "Date")?;
    ws.write(0, 1, "DateTime")?;
    ws.write(0, 2, "Revenue")?;
    ws.write(0, 3, "Region")?;

    ws.write_with_format(1, 0, ExcelDateTime::from_ymd(2024, 1, 15).unwrap(), &date_fmt)?;
    ws.write_with_format(1, 1, ExcelDateTime::from_ymd_hms(2024, 1, 15, 9, 30, 0).unwrap(), &datetime_fmt)?;
    ws.write(1, 2, 45000.0)?;
    ws.write(1, 3, "East")?;

    ws.write_with_format(2, 0, ExcelDateTime::from_ymd(2024, 2, 20).unwrap(), &date_fmt)?;
    ws.write_with_format(2, 1, ExcelDateTime::from_ymd_hms(2024, 2, 20, 14, 0, 0).unwrap(), &datetime_fmt)?;
    ws.write(2, 2, 52000.0)?;
    ws.write(2, 3, "West")?;

    ws.write_with_format(3, 0, ExcelDateTime::from_ymd(2024, 3, 10).unwrap(), &date_fmt)?;
    ws.write_with_format(3, 1, ExcelDateTime::from_ymd_hms(2024, 3, 10, 16, 45, 0).unwrap(), &datetime_fmt)?;
    ws.write(3, 2, 38000.0)?;
    ws.write(3, 3, "North")?;

    // Page setup
    ws.set_landscape();
    ws.set_paper_size(1); // Letter
    ws.set_margins(1.0, 1.0, 0.75, 0.75);
    ws.set_fit_to_page(1, 0);
    ws.set_print_area(0, 0, 3, 3);
    ws.set_repeat_rows(0, 0);

    // Header/footer
    ws.set_header("&LCONFIDENTIAL&CFinancial Report&R&D");
    ws.set_footer("&CPage &P of &N");

    // Row grouping
    ws.group_rows(1, 3, 1);
    ws.set_column_width(0, 14.0)?;
    ws.set_column_width(1, 22.0)?;
    ws.set_column_width(2, 14.0)?;
    ws.set_column_width(3, 12.0)?;

    // Column grouping
    ws.group_columns(1, 2, 1);

    // Autofilter
    ws.set_autofilter(0, 0, 3, 3);

    // Comments
    let ws2 = wb.add_worksheet_with_name("Notes")?;
    ws2.write(0, 0, "See comment")?;
    ws2.add_comment_with_author(0, 0, "Review this figure", "Analyst");

    // Workbook protection
    wb.protect_with_password("secret123");

    wb.save(path)?;

    // Read back and verify
    let wb2 = Workbook::open_readonly(path)?;
    assert_eq!(wb2.sheet_count(), 2);
    let ws_r = wb2.worksheet_ref(0)?;
    // Verify dates read back as numbers (serial dates)
    match ws_r.read_cell(1, 0) {
        CellValue::Number(_) | CellValue::DateTime(_) => {}
        other => panic!("Expected date, got {:?}", other),
    }
    // Verify comments read back
    let ws2_r = wb2.worksheet_ref(1)?;
    assert!(!ws2_r.comments().is_empty(), "Comments should be read back");
    assert_eq!(ws2_r.comments()[0].text, "Review this figure");

    // Verify defined names (print area, repeat rows)
    let names = wb2.defined_names();
    assert!(names.iter().any(|(n, _)| n == "_xlnm.Print_Area"), "Print area defined name missing");

    // CSV export
    let mut csv_buf = Vec::new();
    ws_r.to_csv(&mut csv_buf, b',')?;
    let csv = String::from_utf8(csv_buf).unwrap();
    assert!(csv.contains("Revenue"), "CSV should contain header");
    assert!(csv.contains("45000"), "CSV should contain data");

    Ok(())
}

// ── Integration test wrappers ──
#[test]
fn test_create_integration() {
    let d = dir();
    let p = d.join("test_create.xlsx");
    test_create(&p).unwrap();
}

#[test]
fn test_read_back_integration() {
    let d = dir();
    let p = d.join("test_read_back.xlsx");
    test_create(&p).unwrap();
    test_read_back(&p).unwrap();
}

#[test]
fn test_data_types_integration() {
    let d = dir();
    let p = d.join("test_data_types.xlsx");
    test_data_types(&p).unwrap();
}

#[test]
fn test_read_types_integration() {
    let d = dir();
    let p = d.join("test_read_types.xlsx");
    test_data_types(&p).unwrap();
    test_read_types(&p).unwrap();
}

#[test]
fn test_formatting_integration() {
    let d = dir();
    let p = d.join("test_formatting.xlsx");
    test_formatting(&p).unwrap();
}

#[test]
fn test_multi_sheet_integration() {
    let d = dir();
    let p = d.join("test_multi_sheet.xlsx");
    test_multi_sheet(&p).unwrap();
}

#[test]
fn test_read_multi_sheet_integration() {
    let d = dir();
    let p = d.join("test_read_multi_sheet.xlsx");
    test_multi_sheet(&p).unwrap();
    test_read_multi_sheet(&p).unwrap();
}

#[test]
fn test_edit_mode_integration() {
    let d = dir();
    let p = d.join("test_edit_mode.xlsx");
    test_edit_mode(&p).unwrap();
}

#[test]
fn test_row_ops_integration() {
    test_row_ops().unwrap();
}

#[test]
fn test_col_ops_integration() {
    test_col_ops().unwrap();
}

#[test]
fn test_sheet_mgmt_integration() {
    let d = dir();
    let p = d.join("test_sheet_mgmt.xlsx");
    test_sheet_mgmt(&p).unwrap();
}

#[test]
fn test_formulas_integration() {
    let d = dir();
    let p = d.join("test_formulas.xlsx");
    test_formulas(&p).unwrap();
}

#[test]
fn test_layout_integration() {
    let d = dir();
    let p = d.join("test_layout.xlsx");
    test_layout(&p).unwrap();
}

#[test]
fn test_properties_integration() {
    let d = dir();
    let p = d.join("test_properties.xlsx");
    test_properties(&p).unwrap();
}

#[test]
fn test_defined_names_integration() {
    let d = dir();
    let p = d.join("test_defined_names.xlsx");
    test_defined_names(&p).unwrap();
}

#[test]
fn test_large_integration() {
    let d = dir();
    let p = d.join("test_large.xlsx");
    test_large(&p).unwrap();
}

#[test]
fn test_chart_column_integration() {
    let d = dir();
    let p = d.join("test_chart_column.xlsx");
    test_chart_column(&p).unwrap();
}

#[test]
fn test_chart_all_types_integration() {
    let d = dir();
    let p = d.join("test_chart_all_types.xlsx");
    test_chart_all_types(&p).unwrap();
}

#[test]
fn test_table_integration() {
    let d = dir();
    let p = d.join("test_table.xlsx");
    test_table(&p).unwrap();
}

#[test]
fn test_image_integration() {
    let d = dir();
    let p = d.join("test_image.xlsx");
    test_image(&p).unwrap();
}

#[test]
fn test_conditional_formatting_integration() {
    let d = dir();
    let p = d.join("test_conditional_formatting.xlsx");
    test_conditional_formatting(&p).unwrap();
}

#[test]
fn test_data_validation_integration() {
    let d = dir();
    let p = d.join("test_data_validation.xlsx");
    test_data_validation(&p).unwrap();
}

#[test]
fn test_sparklines_integration() {
    let d = dir();
    let p = d.join("test_sparklines.xlsx");
    test_sparklines(&p).unwrap();
}

#[test]
fn test_dashboard_integration() {
    let d = dir();
    let p = d.join("test_dashboard.xlsx");
    test_dashboard(&p).unwrap();
}

#[test]
fn test_autofit_integration() {
    let d = dir();
    let p = d.join("test_autofit.xlsx");
    test_autofit(&p).unwrap();
}

#[test]
fn test_protection_integration() {
    let d = dir();
    let p = d.join("test_protection.xlsx");
    test_protection(&p).unwrap();
}

#[test]
fn test_print_settings_integration() {
    let d = dir();
    let p = d.join("test_print_settings.xlsx");
    test_print_settings(&p).unwrap();
}

#[test]
fn test_rich_text_integration() {
    let d = dir();
    let p = d.join("test_rich_text.xlsx");
    test_rich_text(&p).unwrap();
}

#[test]
fn test_streaming_integration() {
    let d = dir();
    let p = d.join("test_streaming.xlsx");
    test_streaming(&p).unwrap();
}

#[test]
fn test_phase4_combined_integration() {
    let d = dir();
    let p = d.join("test_phase4_combined.xlsx");
    test_phase4_combined(&p).unwrap();
}

#[test]
fn test_acme_financial_model_integration() {
    let d = dir();
    let p = d.join("test_acme_financial_model.xlsx");
    test_acme_financial_model(&p).unwrap();
}

#[test]
fn test_phase6_showcase_integration() {
    let d = dir();
    let p = d.join("test_phase6_showcase.xlsx");
    test_phase6_showcase(&p).unwrap();
}

#[test]
fn test_sprint7_showcase_integration() {
    let d = dir();
    let p = d.join("test_sprint7_showcase.xlsx");
    test_sprint7_showcase(&p).unwrap();
}

#[test]
fn test_sprint8_showcase_integration() {
    let d = dir();
    let p = d.join("test_sprint8_showcase.xlsx");
    test_sprint8_showcase(&p).unwrap();
}

#[test]
fn test_sprint9_showcase_integration() {
    let d = dir();
    let p = d.join("test_sprint9_showcase.xlsx");
    test_sprint9_showcase(&p).unwrap();
}

#[test]
fn test_sprint10_showcase_integration() {
    let d = dir();
    let p = d.join("test_sprint10_showcase.xlsx");
    test_sprint10_showcase(&p).unwrap();
}

#[test]
fn test_sprint11_showcase_integration() {
    let d = dir();
    let p = d.join("test_sprint11_showcase.xlsx");
    test_sprint11_showcase(&p).unwrap();
}

#[test]
fn test_sprint12_showcase_integration() {
    let d = dir();
    let p = d.join("test_sprint12_showcase.xlsx");
    test_sprint12_showcase(&p).unwrap();
}

#[test]
fn test_border_colors_integration() {
    let d = dir();
    let p = d.join("test_border_colors.xlsx");
    test_border_colors(&p).unwrap();
}

#[test]
fn test_chart_advanced_integration() {
    let d = dir();
    let p = d.join("test_chart_advanced.xlsx");
    test_chart_advanced(&p).unwrap();
}

#[test]
fn test_all_patterns_integration() {
    let d = dir();
    let p = d.join("test_all_patterns.xlsx");
    test_all_patterns(&p).unwrap();
}

#[test]
fn test_formulas_complete_integration() {
    let d = dir();
    let p = d.join("test_formulas_complete.xlsx");
    test_formulas_complete(&p).unwrap();
}

#[test]
fn test_column_range_integration() {
    let d = dir();
    let p = d.join("test_column_range.xlsx");
    test_column_range(&p).unwrap();
}

#[test]
fn test_pivot_table_integration() {
    let d = dir();
    let p = d.join("test_pivot_table.xlsx");
    test_pivot_table(&p).unwrap();
}

#[test]
fn test_feature_parity_integration() {
    let d = dir();
    let p = d.join("test_feature_parity.xlsx");
    test_feature_parity(&p).unwrap();
}

