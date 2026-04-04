use zavora_xlsx::*;

#[test]
fn chart_column_basic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("chart_col.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write_row(0, 0, ["Category", "Value"]).unwrap();
    ws.write(1, 0, "A").unwrap(); ws.write(1, 1, 10.0).unwrap();
    ws.write(2, 0, "B").unwrap(); ws.write(2, 1, 20.0).unwrap();
    ws.write(3, 0, "C").unwrap(); ws.write(3, 1, 30.0).unwrap();

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Sales");
    chart.add_series()
        .set_values("Sheet1!$B$2:$B$4")
        .set_categories("Sheet1!$A$2:$A$4")
        .set_name("Revenue");
    chart.set_x_axis_name("Category");
    chart.set_y_axis_name("Amount");
    ws.insert_chart(5, 0, &chart).unwrap();

    wb.save(&path).unwrap();

    // Verify file is valid xlsx that calamine can open
    use calamine::{Reader, Xlsx, open_workbook, DataType};
    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let range = workbook.worksheet_range("Sheet1").unwrap();
    assert_eq!(range.get((0, 0)).unwrap().get_string(), Some("Category"));
    assert_eq!(range.get((1, 1)).unwrap().get_float(), Some(10.0));
}

#[test]
fn chart_all_types() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("chart_types.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    for r in 0..5u32 { ws.write(r, 0, (r + 1) as f64).unwrap(); }

    for chart_type in [ChartType::Bar, ChartType::Column, ChartType::Line, ChartType::Pie,
                       ChartType::Scatter, ChartType::Area, ChartType::Doughnut, ChartType::Radar] {
        let mut chart = Chart::new(chart_type);
        chart.add_series().set_values("Sheet1!$A$1:$A$5");
        ws.insert_chart(6, 0, &chart).unwrap();
    }

    wb.save(&path).unwrap();
    assert!(path.exists());
    assert!(std::fs::metadata(&path).unwrap().len() > 0);
}

#[test]
fn table_basic() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("table.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write_row(0, 0, ["Name", "Score", "Grade"]).unwrap();
    ws.write(1, 0, "Alice").unwrap(); ws.write(1, 1, 95.0).unwrap(); ws.write(1, 2, "A").unwrap();
    ws.write(2, 0, "Bob").unwrap(); ws.write(2, 1, 87.0).unwrap(); ws.write(2, 2, "B").unwrap();

    let mut table = Table::new();
    table.set_columns(&[
        TableColumn::new("Name"),
        TableColumn::new("Score"),
        TableColumn::new("Grade"),
    ]);
    table.set_style(TableStyle::Medium(2));
    ws.add_table(0, 0, 2, 2, &table).unwrap();

    wb.save(&path).unwrap();

    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let range = workbook.worksheet_range("Sheet1").unwrap();
    assert_eq!(range.get((1, 0)).unwrap().get_string(), Some("Alice"));
}

#[test]
fn image_png_embed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("image.xlsx");

    // Create a minimal valid 1x1 PNG
    let png_data = create_minimal_png();
    let image = Image::from_buffer(&png_data).unwrap();

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Image below:").unwrap();
    ws.insert_image(2, 0, &image).unwrap();

    wb.save(&path).unwrap();

    // Verify the file contains the image in media/
    let file = std::fs::File::open(&path).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).unwrap();
    let has_media = (0..zip.len()).any(|i| zip.by_index(i).unwrap().name().starts_with("xl/media/"));
    assert!(has_media, "Expected image in xl/media/");
}

#[test]
fn conditional_format_cell_value() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cf.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    for r in 0..10u32 { ws.write(r, 0, (r * 10) as f64).unwrap(); }

    let cf = ConditionalFormatCell::new(CfOperator::GreaterThan, 50.0);
    ws.add_conditional_format(0, 0, 9, 0, cf).unwrap();

    let cf2 = ConditionalFormat2ColorScale::new("#FFFFFF", "#FF0000");
    ws.add_conditional_format(0, 0, 9, 0, cf2).unwrap();

    let cf3 = ConditionalFormatDataBar::new("#4472C4");
    ws.add_conditional_format(0, 0, 9, 0, cf3).unwrap();

    let cf4 = ConditionalFormatIconSet::new(IconSetType::ThreeArrows);
    ws.add_conditional_format(0, 0, 9, 0, cf4).unwrap();

    wb.save(&path).unwrap();

    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let range = workbook.worksheet_range("Sheet1").unwrap();
    assert_eq!(range.get((5, 0)).unwrap().get_float(), Some(50.0));
}

#[test]
fn data_validation_list() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dv.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Status:").unwrap();

    let mut dv = DataValidation::new(ValidationRule::List(vec!["Yes".into(), "No".into(), "Maybe".into()]));
    dv.set_input_message("Choose", "Select a value from the list");
    dv.set_error_message(ErrorStyle::Stop, "Invalid", "Please select from the list");
    ws.add_data_validation(1, 0, 10, 0, &dv).unwrap();

    let mut dv2 = DataValidation::new(ValidationRule::WholeNumber { min: Some(1), max: Some(100) });
    ws.add_data_validation(1, 1, 10, 1, &dv2).unwrap();

    wb.save(&path).unwrap();
    assert!(path.exists());
}

#[test]
fn sparkline_basic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sparkline.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write_row(0, 0, [1.0_f64, 3.0, 2.0, 5.0, 4.0]).unwrap();
    ws.write_row(1, 0, [5.0_f64, 2.0, 4.0, 1.0, 3.0]).unwrap();

    let sp1 = Sparkline::new("Sheet1!A1:E1", SparklineType::Line);
    ws.add_sparkline(0, 6, &sp1).unwrap();

    let mut sp2 = Sparkline::new("Sheet1!A2:E2", SparklineType::Column);
    sp2.set_color("#FF0000");
    ws.add_sparkline(1, 6, &sp2).unwrap();

    wb.save(&path).unwrap();
    assert!(path.exists());
}

#[test]
fn all_features_combined() {
    use calamine::{Reader, Xlsx, open_workbook, DataType};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("all_features.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Data
    ws.write_row(0, 0, ["Product", "Q1", "Q2", "Q3", "Q4"]).unwrap();
    ws.write(1, 0, "Widget").unwrap();
    ws.write_row(1, 1, [100.0_f64, 150.0, 200.0, 180.0]).unwrap();
    ws.write(2, 0, "Gadget").unwrap();
    ws.write_row(2, 1, [80.0_f64, 90.0, 120.0, 110.0]).unwrap();

    // Table
    let mut table = Table::new();
    table.set_columns(&[
        TableColumn::new("Product"), TableColumn::new("Q1"),
        TableColumn::new("Q2"), TableColumn::new("Q3"), TableColumn::new("Q4"),
    ]);
    table.set_style(TableStyle::Medium(9));
    ws.add_table(0, 0, 2, 4, &table).unwrap();

    // Chart
    let mut chart = Chart::new(ChartType::Line);
    chart.set_title("Quarterly Sales");
    chart.add_series()
        .set_values("Sheet1!$B$2:$E$2")
        .set_categories("Sheet1!$B$1:$E$1")
        .set_name("Widget");
    chart.add_series()
        .set_values("Sheet1!$B$3:$E$3")
        .set_categories("Sheet1!$B$1:$E$1")
        .set_name("Gadget");
    ws.insert_chart(5, 0, &chart).unwrap();

    // Conditional formatting on Q1-Q4
    let cf = ConditionalFormat3ColorScale::new("#FF0000", "#FFFF00", "#00FF00");
    ws.add_conditional_format(1, 1, 2, 4, cf).unwrap();

    // Data validation
    let dv = DataValidation::new(ValidationRule::Decimal { min: Some(0.0), max: Some(1000.0) });
    ws.add_data_validation(1, 1, 2, 4, &dv).unwrap();

    // Sparklines
    let sp = Sparkline::new("Sheet1!B2:E2", SparklineType::Line);
    ws.add_sparkline(1, 5, &sp).unwrap();

    wb.save(&path).unwrap();

    // Verify
    let mut workbook: Xlsx<_> = open_workbook(&path).unwrap();
    let range = workbook.worksheet_range("Sheet1").unwrap();
    assert_eq!(range.get((0, 0)).unwrap().get_string(), Some("Product"));
    assert_eq!(range.get((1, 1)).unwrap().get_float(), Some(100.0));
    assert_eq!(range.get((2, 4)).unwrap().get_float(), Some(110.0));
}

/// Create a minimal valid 1x1 white PNG (67 bytes).
fn create_minimal_png() -> Vec<u8> {
    let mut png = Vec::new();
    // PNG signature
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    // IHDR chunk
    let ihdr_data = [
        0, 0, 0, 1, // width = 1
        0, 0, 0, 1, // height = 1
        8,           // bit depth
        2,           // color type (RGB)
        0, 0, 0,     // compression, filter, interlace
    ];
    write_png_chunk(&mut png, b"IHDR", &ihdr_data);
    // IDAT chunk (zlib-compressed single white pixel)
    let idat_data = [0x78, 0x01, 0x62, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x04, 0x00, 0x01];
    write_png_chunk(&mut png, b"IDAT", &idat_data);
    // IEND chunk
    write_png_chunk(&mut png, b"IEND", &[]);
    png
}

fn write_png_chunk(buf: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(chunk_type);
    buf.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    let crc = crc32(&crc_data);
    buf.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 { crc = (crc >> 1) ^ 0xEDB88320; }
            else { crc >>= 1; }
        }
    }
    !crc
}
