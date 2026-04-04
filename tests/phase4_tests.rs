use zavora_xlsx::*;

#[test]
fn test_autofit() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Short").unwrap();
    ws.write(0, 1, "This is a much longer string value").unwrap();
    ws.write(0, 2, 12345.6789).unwrap();
    ws.autofit().unwrap();
    let buf = wb.save_to_buffer().unwrap();
    let xml = extract_sheet_xml(&buf, 1);
    assert!(xml.contains("customWidth"), "Should contain customWidth columns");
    assert!(xml.contains("<cols>"), "Should contain cols element");
}

#[test]
fn test_sheet_protection() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Protected").unwrap();
    ws.protect_with_password("secret");
    let buf = wb.save_to_buffer().unwrap();
    let xml = extract_sheet_xml(&buf, 1);
    assert!(xml.contains("sheetProtection"), "Should contain sheetProtection element");
    assert!(xml.contains("password="), "Should contain password hash");
}

#[test]
fn test_sheet_protection_no_password() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Protected").unwrap();
    ws.protect();
    let buf = wb.save_to_buffer().unwrap();
    let xml = extract_sheet_xml(&buf, 1);
    assert!(xml.contains("sheetProtection"), "Should contain sheetProtection element");
    assert!(!xml.contains("password="), "Should not contain password when none set");
}

#[test]
fn test_workbook_protection() {
    let mut wb = Workbook::new();
    wb.worksheet(0).unwrap().write(0, 0, "Data").unwrap();
    wb.protect_with_password("admin");
    let buf = wb.save_to_buffer().unwrap();
    let xml = extract_xml(&buf, "xl/workbook.xml");
    assert!(xml.contains("workbookProtection"), "Should contain workbookProtection");
    assert!(xml.contains("lockStructure"), "Should lock structure");
}

#[test]
fn test_print_settings() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Print me").unwrap();
    let ps = PrintSettings::new()
        .paper_size(1)
        .orientation(Orientation::Landscape)
        .margins(1.0, 1.0, 0.75, 0.75)
        .header("&CPage &P")
        .footer("&LConfidential");
    ws.set_print_settings(&ps);
    ws.set_page_breaks(&[10, 20], &[5]);
    let buf = wb.save_to_buffer().unwrap();
    let xml = extract_sheet_xml(&buf, 1);
    assert!(xml.contains("pageMargins"), "Should contain pageMargins");
    assert!(xml.contains("pageSetup"), "Should contain pageSetup");
    assert!(xml.contains("landscape"), "Should be landscape");
    assert!(xml.contains("oddHeader"), "Should contain header");
    assert!(xml.contains("oddFooter"), "Should contain footer");
    assert!(xml.contains("rowBreaks"), "Should contain row breaks");
    assert!(xml.contains("colBreaks"), "Should contain col breaks");
}

#[test]
fn test_rich_text() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    let rt = RichText::new()
        .add_run("Normal ")
        .add_bold("Bold ")
        .add_italic("Italic")
        .add_styled("Red", RichTextRun::new().color("FF0000").font_size(16.0));
    ws.write_rich_text(0, 0, &rt).unwrap();
    assert_eq!(rt.plain_text(), "Normal Bold ItalicRed");
    let buf = wb.save_to_buffer().unwrap();
    let xml = extract_sheet_xml(&buf, 1);
    assert!(xml.contains("<r>"), "Should contain rich text runs");
    assert!(xml.contains("<b/>"), "Should contain bold tag");
    assert!(xml.contains("<i/>"), "Should contain italic tag");
    assert!(xml.contains("FFFF0000"), "Should contain color");
}

#[test]
fn test_streaming_workbook() {
    let mut wb = StreamingWorkbook::new();
    for row in 0..100u32 {
        wb.write_string(row, 0, &format!("Row {row}")).unwrap();
        wb.write_number(row, 1, row as f64).unwrap();
    }
    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());
    // Verify it's a valid xlsx by checking zip structure
    let cursor = std::io::Cursor::new(&buf);
    let archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<_> = (0..archive.len()).map(|i| archive.name_for_index(i).unwrap().to_string()).collect();
    assert!(names.iter().any(|n| n.contains("sheet1.xml")));
    assert!(names.iter().any(|n| n.contains("workbook.xml")));
}

#[test]
fn test_streaming_row_order_enforced() {
    let mut wb = StreamingWorkbook::new();
    wb.write_string(5, 0, "Row 5").unwrap();
    let result = wb.write_string(3, 0, "Row 3");
    assert!(result.is_err(), "Should error on non-ascending row order");
}

#[test]
fn test_streaming_multi_sheet() {
    let mut wb = StreamingWorkbook::new();
    wb.write_string(0, 0, "Sheet1 data").unwrap();
    wb.add_worksheet("Second");
    wb.write_string(0, 0, "Sheet2 data").unwrap();
    let buf = wb.save_to_buffer().unwrap();
    let cursor = std::io::Cursor::new(&buf);
    let archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<_> = (0..archive.len()).map(|i| archive.name_for_index(i).unwrap().to_string()).collect();
    assert!(names.iter().any(|n| n.contains("sheet2.xml")));
}

#[test]
fn test_parallel_assembly_multi_sheet() {
    let mut wb = Workbook::new();
    wb.worksheet(0).unwrap().write(0, 0, "Sheet1").unwrap();
    wb.add_worksheet_with_name("Data").unwrap().write(0, 0, "Sheet2").unwrap();
    wb.add_worksheet_with_name("Summary").unwrap().write(0, 0, "Sheet3").unwrap();
    // This exercises the parallel assembly path in save_to_buffer
    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());
    // Verify all 3 sheets are present
    let cursor = std::io::Cursor::new(&buf);
    let archive = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<_> = (0..archive.len()).map(|i| archive.name_for_index(i).unwrap().to_string()).collect();
    assert!(names.iter().any(|n| n.contains("sheet3.xml")));
}

#[test]
fn test_password_hash_deterministic() {
    // Test indirectly: two workbooks with same password should produce same protection XML
    let mut wb1 = Workbook::new();
    wb1.worksheet(0).unwrap().write(0, 0, "A").unwrap();
    wb1.worksheet(0).unwrap().protect_with_password("test");
    let buf1 = wb1.save_to_buffer().unwrap();
    let xml1 = extract_sheet_xml(&buf1, 1);

    let mut wb2 = Workbook::new();
    wb2.worksheet(0).unwrap().write(0, 0, "A").unwrap();
    wb2.worksheet(0).unwrap().protect_with_password("test");
    let buf2 = wb2.save_to_buffer().unwrap();
    let xml2 = extract_sheet_xml(&buf2, 1);

    // Same password should produce same hash
    assert_eq!(xml1, xml2);
}

// Helper: extract sheet XML from xlsx buffer
fn extract_sheet_xml(buf: &[u8], sheet_num: usize) -> String {
    let cursor = std::io::Cursor::new(buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let path = format!("xl/worksheets/sheet{sheet_num}.xml");
    let mut entry = archive.by_name(&path).unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut entry, &mut xml).unwrap();
    xml
}

fn extract_xml(buf: &[u8], path: &str) -> String {
    let cursor = std::io::Cursor::new(buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut entry = archive.by_name(path).unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut entry, &mut xml).unwrap();
    xml
}
