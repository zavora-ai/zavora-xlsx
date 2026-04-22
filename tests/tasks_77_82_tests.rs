/// Tests for Tasks 77–82: Security & Compliance + Accessibility
///
/// Task 77: File Encryption (Read) — skeleton behind crypto feature flag
/// Task 78: File Encryption (Write) — skeleton
/// Task 79: Sheet Protection Granularity — builder methods + serialization
/// Task 80: VBA Project Signing — skeleton
/// Task 81: IRM Metadata Preservation — passthrough mechanism test
/// Task 82: Accessibility Metadata — alt_text on Chart, Image, Table
use zavora_xlsx::{Chart, ChartType, Image, SheetProtection, Table, TableColumn, Workbook};

// ── Task 77: File Encryption (Read) ──

#[test]
fn test_ole2_magic_detection() {
    // OLE2 magic bytes
    let ole2 = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00];
    assert!(zavora_xlsx::crypto::is_ole2(&ole2));

    // ZIP magic (PK)
    let zip = vec![0x50, 0x4B, 0x03, 0x04];
    assert!(!zavora_xlsx::crypto::is_ole2(&zip));

    // Empty / too short
    assert!(!zavora_xlsx::crypto::is_ole2(&[]));
    assert!(!zavora_xlsx::crypto::is_ole2(&[0xD0]));
}

#[test]
fn test_open_with_password_returns_unsupported_for_ole2() {
    // Create a fake OLE2 file
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("encrypted.xlsx");
    let ole2_data: Vec<u8> = vec![
        0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];
    std::fs::write(&path, &ole2_data).unwrap();

    let result = Workbook::open_with_password(&path, "secret");
    assert!(result.is_err());
    let err = format!("{}", result.err().unwrap());
    assert!(
        err.contains("encryption") || err.contains("Unsupported"),
        "Error: {err}"
    );
}

#[test]
fn test_open_with_password_falls_through_for_normal_xlsx() {
    // Create a normal xlsx file, then open_with_password should succeed
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("normal.xlsx");
    let mut wb = Workbook::new();
    wb.worksheet(0).unwrap().write(0, 0, "hello").unwrap();
    wb.save(&path).unwrap();

    // open_with_password on a non-encrypted file should fall through to normal open
    let wb2 = Workbook::open_with_password(&path, "anything").unwrap();
    assert_eq!(wb2.sheet_count(), 1);
}

// ── Task 78: File Encryption (Write) ──

#[test]
fn test_save_encrypted_returns_unsupported() {
    let mut wb = Workbook::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("encrypted_out.xlsx");

    let result = wb.save_encrypted(&path, "password123");
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("encryption") || msg.contains("Unsupported"),
        "Error: {msg}"
    );
}

// ── Task 79: Sheet Protection Granularity ──

#[test]
fn test_sheet_protection_builder_methods() {
    let mut prot = SheetProtection::default();

    // Verify defaults
    assert!(prot.sheet);
    assert!(prot.sort);
    assert!(prot.auto_filter);
    assert!(prot.pivot_tables);
    assert!(prot.format_cells);
    assert!(!prot.select_locked_cells);
    assert!(!prot.select_unlocked_cells);

    // Use builder methods to customize
    prot.set_sort(false)
        .set_auto_filter(false)
        .set_pivot_tables(false)
        .set_format_cells(false)
        .set_format_columns(false)
        .set_format_rows(false)
        .set_insert_columns(false)
        .set_insert_rows(false)
        .set_insert_hyperlinks(false)
        .set_delete_columns(false)
        .set_delete_rows(false)
        .set_select_locked_cells(true)
        .set_select_unlocked_cells(true);

    assert!(!prot.sort);
    assert!(!prot.auto_filter);
    assert!(!prot.pivot_tables);
    assert!(!prot.format_cells);
    assert!(!prot.format_columns);
    assert!(!prot.format_rows);
    assert!(!prot.insert_columns);
    assert!(!prot.insert_rows);
    assert!(!prot.insert_hyperlinks);
    assert!(!prot.delete_columns);
    assert!(!prot.delete_rows);
    assert!(prot.select_locked_cells);
    assert!(prot.select_unlocked_cells);
}

#[test]
fn test_sheet_protection_granular_flags_serialized() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("protection_granular.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Protected").unwrap();

    let mut prot = SheetProtection::default();
    prot.set_sort(false)
        .set_auto_filter(false)
        .set_format_cells(false)
        .set_insert_columns(false)
        .set_delete_rows(false)
        .set_select_locked_cells(true)
        .set_select_unlocked_cells(true);
    ws.protect_with_options(prot);

    wb.save(&path).unwrap();

    // Read back and verify the XML contains the expected attributes
    let data = std::fs::read(&path).unwrap();
    let cursor = std::io::Cursor::new(&data);
    let mut zip = zip::ZipArchive::new(cursor).unwrap();

    let mut sheet_xml = String::new();
    {
        use std::io::Read;
        let mut entry = zip.by_name("xl/worksheets/sheet1.xml").unwrap();
        entry.read_to_string(&mut sheet_xml).unwrap();
    }

    // Verify sheetProtection element exists with expected attributes
    assert!(
        sheet_xml.contains("sheetProtection"),
        "Missing sheetProtection element"
    );
    assert!(sheet_xml.contains("sheet=\"1\""), "Missing sheet=\"1\"");
    // sort=false → sort="0"
    assert!(
        sheet_xml.contains("sort=\"0\""),
        "Missing sort=\"0\" in: {sheet_xml}"
    );
    // autoFilter=false → autoFilter="0"
    assert!(
        sheet_xml.contains("autoFilter=\"0\""),
        "Missing autoFilter=\"0\""
    );
    // formatCells=false → formatCells="0"
    assert!(
        sheet_xml.contains("formatCells=\"0\""),
        "Missing formatCells=\"0\""
    );
    // insertColumns=false → insertColumns="0"
    assert!(
        sheet_xml.contains("insertColumns=\"0\""),
        "Missing insertColumns=\"0\""
    );
    // deleteRows=false → deleteRows="0"
    assert!(
        sheet_xml.contains("deleteRows=\"0\""),
        "Missing deleteRows=\"0\""
    );
    // selectLockedCells=true → selectLockedCells="1"
    assert!(
        sheet_xml.contains("selectLockedCells=\"1\""),
        "Missing selectLockedCells=\"1\""
    );
    // selectUnlockedCells=true → selectUnlockedCells="1"
    assert!(
        sheet_xml.contains("selectUnlockedCells=\"1\""),
        "Missing selectUnlockedCells=\"1\""
    );
}

// ── Task 80: VBA Project Signing ──

#[test]
fn test_sign_vba_returns_unsupported() {
    let mut wb = Workbook::new();
    let fake_cert = b"fake-certificate-data";

    let result = wb.sign_vba(fake_cert);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("VBA project signing") || msg.contains("Unsupported"),
        "Error: {msg}"
    );
}

// ── Task 81: IRM Metadata Preservation ──

#[test]
fn test_irm_metadata_preserved_via_passthrough() {
    // IRM metadata is stored in custom parts like `\006DataSpaces/` or
    // `EncryptedPackage`. The passthrough mechanism preserves unknown ZIP
    // entries during edit-mode open/save. We simulate this by adding
    // IRM-like passthrough entries and verifying they survive a round-trip.

    let dir = tempfile::tempdir().unwrap();
    let path1 = dir.path().join("irm_original.xlsx");
    let path2 = dir.path().join("irm_modified.xlsx");

    // Create a workbook and add fake IRM-like passthrough entries
    let mut wb = Workbook::new();
    wb.worksheet(0).unwrap().write(0, 0, "IRM Test").unwrap();

    let irm_data = b"<irm>rights-management-metadata</irm>".to_vec();
    let irm_data2 = b"<irm>license-info</irm>".to_vec();
    wb.add_passthrough_entry("customXml/irmData.xml", irm_data.clone());
    wb.add_passthrough_entry("customXml/irmLicense.xml", irm_data2.clone());

    wb.save(&path1).unwrap();

    // Verify the passthrough entries are in the saved file
    let file_data = std::fs::read(&path1).unwrap();
    let cursor = std::io::Cursor::new(&file_data);
    let zip = zip::ZipArchive::new(cursor).unwrap();
    let names: Vec<String> = (0..zip.len())
        .map(|i| zip.name_for_index(i).unwrap().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n == "customXml/irmData.xml"),
        "IRM data not found in ZIP: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "customXml/irmLicense.xml"),
        "IRM license not found in ZIP: {names:?}"
    );

    // Open in edit mode and modify
    let mut wb2 = Workbook::open(&path1).unwrap();
    wb2.worksheet(0).unwrap().write(1, 0, "Modified").unwrap();
    wb2.save(&path2).unwrap();

    // Verify IRM entries are preserved after round-trip
    let file_data2 = std::fs::read(&path2).unwrap();
    let cursor2 = std::io::Cursor::new(&file_data2);
    let mut zip2 = zip::ZipArchive::new(cursor2).unwrap();
    let names2: Vec<String> = (0..zip2.len())
        .map(|i| zip2.name_for_index(i).unwrap().to_string())
        .collect();
    assert!(
        names2.iter().any(|n| n == "customXml/irmData.xml"),
        "IRM data lost after round-trip: {names2:?}"
    );
    assert!(
        names2.iter().any(|n| n == "customXml/irmLicense.xml"),
        "IRM license lost after round-trip: {names2:?}"
    );

    // Verify content is preserved
    {
        use std::io::Read;
        let mut entry = zip2.by_name("customXml/irmData.xml").unwrap();
        let mut content = Vec::new();
        entry.read_to_end(&mut content).unwrap();
        assert_eq!(
            content, irm_data,
            "IRM data content changed after round-trip"
        );
    }
}

// ── Task 82: Accessibility Metadata ──

#[test]
fn test_chart_alt_text() {
    let mut chart = Chart::new(ChartType::Bar);
    chart.set_title("Sales");
    chart.set_alt_text("Sales Chart", "Bar chart showing quarterly sales figures");

    let (title, descr) = chart.alt_text().unwrap();
    assert_eq!(title, "Sales Chart");
    assert_eq!(descr, "Bar chart showing quarterly sales figures");
}

#[test]
fn test_image_alt_text() {
    // Create a minimal valid PNG (1x1 pixel)
    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // width = 1
        0x00, 0x00, 0x00, 0x01, // height = 1
        0x08, 0x02, 0x00, 0x00, 0x00, // bit depth, color type, etc.
        0x90, 0x77, 0x53, 0xDE, // CRC
        0x00, 0x00, 0x00, 0x0C, // IDAT length
        0x49, 0x44, 0x41, 0x54, // IDAT
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, // data
        0xE2, 0x21, 0xBC, 0x33, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ];
    let mut img = Image::from_buffer(&png_data).unwrap();
    img.set_alt_text("Company Logo", "The company logo displayed in the header");

    let (title, descr) = img.alt_text().unwrap();
    assert_eq!(title, "Company Logo");
    assert_eq!(descr, "The company logo displayed in the header");
}

#[test]
fn test_table_alt_text() {
    let mut table = Table::new();
    table.set_columns(&[TableColumn::new("Name"), TableColumn::new("Value")]);
    table.set_alt_text("Data Table", "Table containing name-value pairs");

    let (title, descr) = table.alt_text().unwrap();
    assert_eq!(title, "Data Table");
    assert_eq!(descr, "Table containing name-value pairs");
}

#[test]
fn test_chart_alt_text_serialized_in_drawing_xml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("chart_alt_text.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Q1").unwrap();
    ws.write(0, 1, 100.0).unwrap();
    ws.write(1, 0, "Q2").unwrap();
    ws.write(1, 1, 200.0).unwrap();

    let mut chart = Chart::new(ChartType::Bar);
    chart.set_alt_text("Sales Chart", "Quarterly sales data");
    let series = chart.add_series();
    series.set_values("Sheet1!$B$1:$B$2");
    series.set_categories("Sheet1!$A$1:$A$2");
    ws.insert_chart(0, 3, &chart).unwrap();

    wb.save(&path).unwrap();

    // Read the drawing XML and verify alt text attributes
    let data = std::fs::read(&path).unwrap();
    let cursor = std::io::Cursor::new(&data);
    let mut zip = zip::ZipArchive::new(cursor).unwrap();

    let mut drawing_xml = String::new();
    {
        use std::io::Read;
        let mut entry = zip.by_name("xl/drawings/drawing1.xml").unwrap();
        entry.read_to_string(&mut drawing_xml).unwrap();
    }

    assert!(
        drawing_xml.contains("title=\"Sales Chart\""),
        "Missing title attribute in drawing XML: {drawing_xml}"
    );
    assert!(
        drawing_xml.contains("descr=\"Quarterly sales data\""),
        "Missing descr attribute in drawing XML: {drawing_xml}"
    );
}

#[test]
fn test_image_alt_text_serialized_in_drawing_xml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("image_alt_text.xlsx");

    // Create a minimal valid PNG
    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC, 0x33, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Image test").unwrap();

    let mut img = Image::from_buffer(&png_data).unwrap();
    img.set_alt_text("Logo", "Company logo image");
    ws.insert_image(2, 2, &img).unwrap();

    wb.save(&path).unwrap();

    // Read the drawing XML and verify alt text attributes
    let data = std::fs::read(&path).unwrap();
    let cursor = std::io::Cursor::new(&data);
    let mut zip = zip::ZipArchive::new(cursor).unwrap();

    let mut drawing_xml = String::new();
    {
        use std::io::Read;
        let mut entry = zip.by_name("xl/drawings/drawing1.xml").unwrap();
        entry.read_to_string(&mut drawing_xml).unwrap();
    }

    assert!(
        drawing_xml.contains("title=\"Logo\""),
        "Missing title attribute in drawing XML: {drawing_xml}"
    );
    assert!(
        drawing_xml.contains("descr=\"Company logo image\""),
        "Missing descr attribute in drawing XML: {drawing_xml}"
    );
}
