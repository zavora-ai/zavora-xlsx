//! Integration tests for Tasks 62-68: File format variants.

use zavora_xlsx::{CsvOptions, Workbook};

// ── Task 62: XLSM Write ──

#[test]
fn test_save_as_xlsm_with_vba() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "XLSM Test").unwrap();
    ws.write(1, 0, 42.0).unwrap();

    let fake_vba = b"FAKE_VBA_PROJECT_DATA";
    let path = "output/test_xlsm_write.xlsm";
    wb.save_as_xlsm(path, fake_vba).unwrap();

    // Verify the file was created and is a valid ZIP
    let data = std::fs::read(path).unwrap();
    assert!(data.len() > 100, "XLSM file should not be empty");

    // Verify content types contain macro-enabled type
    let cursor = std::io::Cursor::new(&data);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut ct_xml = String::new();
    {
        use std::io::Read;
        let mut ct_file = archive.by_name("[Content_Types].xml").unwrap();
        ct_file.read_to_string(&mut ct_xml).unwrap();
    }
    assert!(
        ct_xml.contains("macroEnabled"),
        "Content types should contain macroEnabled: {ct_xml}"
    );
    assert!(
        ct_xml.contains("vnd.ms-excel.sheet.macroEnabled.main+xml"),
        "Should have XLSM content type"
    );

    // Verify vbaProject.bin is in the ZIP
    assert!(
        archive.by_name("xl/vbaProject.bin").is_ok(),
        "ZIP should contain xl/vbaProject.bin"
    );

    std::fs::remove_file(path).ok();
}

#[test]
fn test_save_as_xlsm_empty_vba_returns_error() {
    let mut wb = Workbook::new();
    let result = wb.save_as_xlsm("output/should_not_exist.xlsm", &[]);
    assert!(result.is_err(), "Empty VBA project should return error");
}

// ── Task 63: Template Formats ──

#[test]
fn test_save_as_template_xltx() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Template Test").unwrap();

    let path = "output/test_template.xltx";
    wb.save_as_template(path).unwrap();

    // Verify content types contain template type
    let data = std::fs::read(path).unwrap();
    let cursor = std::io::Cursor::new(&data);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut ct_xml = String::new();
    {
        use std::io::Read;
        let mut ct_file = archive.by_name("[Content_Types].xml").unwrap();
        ct_file.read_to_string(&mut ct_xml).unwrap();
    }
    assert!(
        ct_xml.contains("spreadsheetml.template.main+xml"),
        "Should have XLTX content type: {ct_xml}"
    );

    std::fs::remove_file(path).ok();
}

#[test]
fn test_save_as_template_macro_xltm() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Macro Template Test").unwrap();

    let fake_vba = b"FAKE_VBA_FOR_TEMPLATE";
    let path = "output/test_template_macro.xltm";
    wb.save_as_template_macro(path, fake_vba).unwrap();

    // Verify content types contain macro-enabled template type
    let data = std::fs::read(path).unwrap();
    let cursor = std::io::Cursor::new(&data);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut ct_xml = String::new();
    {
        use std::io::Read;
        let mut ct_file = archive.by_name("[Content_Types].xml").unwrap();
        ct_file.read_to_string(&mut ct_xml).unwrap();
    }
    assert!(
        ct_xml.contains("template.macroEnabled.main+xml"),
        "Should have XLTM content type: {ct_xml}"
    );

    // Verify vbaProject.bin is in the ZIP
    assert!(
        archive.by_name("xl/vbaProject.bin").is_ok(),
        "ZIP should contain xl/vbaProject.bin"
    );

    std::fs::remove_file(path).ok();
}

// ── Task 66: CSV/TSV Export ──

#[test]
fn test_csv_export_basic() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Name").unwrap();
    ws.write(0, 1, "Age").unwrap();
    ws.write(1, 0, "Alice").unwrap();
    ws.write(1, 1, 30).unwrap();

    let opts = CsvOptions::default();
    let csv = ws.to_csv_string(&opts);

    assert!(csv.contains("Name,Age"));
    assert!(csv.contains("Alice,30"));
    assert!(csv.ends_with("\r\n"));
}

#[test]
fn test_csv_export_with_special_chars() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Hello, World").unwrap(); // Contains comma
    ws.write(0, 1, "Say \"hi\"").unwrap(); // Contains quotes
    ws.write(1, 0, "Line1\nLine2").unwrap(); // Contains newline
    ws.write(1, 1, "Normal").unwrap();

    let opts = CsvOptions::default();
    let csv = ws.to_csv_string(&opts);

    // Fields with commas should be quoted
    assert!(csv.contains("\"Hello, World\""));
    // Fields with quotes should be escaped
    assert!(csv.contains("\"Say \"\"hi\"\"\""));
    // Fields with newlines should be quoted
    assert!(csv.contains("\"Line1\nLine2\""));
}

#[test]
fn test_csv_export_tsv() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "A").unwrap();
    ws.write(0, 1, "B").unwrap();
    ws.write(1, 0, "1").unwrap();
    ws.write(1, 1, "2").unwrap();

    let opts = CsvOptions::tsv();
    let tsv = ws.to_csv_string(&opts);

    assert!(tsv.contains("A\tB"));
    assert!(tsv.contains("1\t2"));
}

#[test]
fn test_csv_export_to_file() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Test").unwrap();
    ws.write(0, 1, 123).unwrap();

    let path = "output/test_csv_export.csv";
    let opts = CsvOptions::default();
    ws.to_csv_file(path, &opts).unwrap();

    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("Test,123"));

    std::fs::remove_file(path).ok();
}

#[test]
fn test_csv_export_empty_sheet() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let opts = CsvOptions::default();
    let csv = ws.to_csv_string(&opts);
    assert!(csv.is_empty(), "Empty sheet should produce empty CSV");
}

#[test]
fn test_csv_export_bool_values() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, true).unwrap();
    ws.write(0, 1, false).unwrap();

    let opts = CsvOptions::default();
    let csv = ws.to_csv_string(&opts);
    assert!(csv.contains("TRUE,FALSE"));
}

// ── Task 68: Strict OOXML ──

#[test]
fn test_strict_ooxml_detection() {
    use zavora_xlsx::strict_ooxml;

    let strict = br#"<worksheet xmlns="http://purl.oclc.org/ooxml/spreadsheetml/main"><sheetData/></worksheet>"#;
    assert!(strict_ooxml::is_strict_ooxml(strict));

    let transitional = br#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData/></worksheet>"#;
    assert!(!strict_ooxml::is_strict_ooxml(transitional));
}

#[test]
fn test_strict_ooxml_conversion() {
    use zavora_xlsx::strict_ooxml;

    let strict = br#"<worksheet xmlns="http://purl.oclc.org/ooxml/spreadsheetml/main"><sheetData/></worksheet>"#;
    let converted = strict_ooxml::convert_strict_to_transitional(strict);
    let s = String::from_utf8(converted).unwrap();
    assert!(s.contains("schemas.openxmlformats.org/spreadsheetml/2006/main"));
    assert!(!s.contains("purl.oclc.org"));
}
