use std::io::Read;
use zavora_xlsx::{Format, Workbook};

/// Helper: save workbook to buffer and extract a specific XML file from the zip.
fn extract_xml_from_buffer(buf: &[u8], entry_name: &str) -> Option<String> {
    let cursor = std::io::Cursor::new(buf.to_vec());
    let mut archive = zip::ZipArchive::new(cursor).ok()?;
    let mut file = archive.by_name(entry_name).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    Some(contents)
}

#[test]
fn test_cell_style_heading1_serialization() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let heading_fmt = Format::new().bold().font_size(16.0).cell_style("Heading 1");
    ws.write_with_format(0, 0, "Title", &heading_fmt).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let styles_xml = extract_xml_from_buffer(&buf, "xl/styles.xml").unwrap();

    // Should have cellStyleXfs with count > 1 (Normal + Heading 1)
    assert!(
        styles_xml.contains("<cellStyleXfs count=\"2\""),
        "Expected 2 cellStyleXfs entries, got:\n{}",
        styles_xml
    );

    // Should have a cellStyle element for "Heading 1" with builtinId="16"
    assert!(
        styles_xml.contains("name=\"Heading 1\""),
        "Expected cellStyle name=\"Heading 1\" in styles.xml, got:\n{}",
        styles_xml
    );
    assert!(
        styles_xml.contains("builtinId=\"16\""),
        "Expected builtinId=\"16\" for Heading 1 in styles.xml, got:\n{}",
        styles_xml
    );
}

#[test]
fn test_cell_style_multiple_styles() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let heading1 = Format::new().bold().font_size(16.0).cell_style("Heading 1");
    let currency = Format::new().num_format("$#,##0.00").cell_style("Currency");
    let percent = Format::new().num_format("0%").cell_style("Percent");

    ws.write_with_format(0, 0, "Title", &heading1).unwrap();
    ws.write_with_format(1, 0, 1234.56, &currency).unwrap();
    ws.write_with_format(2, 0, 0.75, &percent).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let styles_xml = extract_xml_from_buffer(&buf, "xl/styles.xml").unwrap();

    // Should have 4 cellStyleXfs entries (Normal + Heading 1 + Currency + Percent)
    assert!(
        styles_xml.contains("<cellStyleXfs count=\"4\""),
        "Expected 4 cellStyleXfs entries, got:\n{}",
        styles_xml
    );

    // Should have cellStyles count="4"
    assert!(
        styles_xml.contains("<cellStyles count=\"4\""),
        "Expected 4 cellStyles entries, got:\n{}",
        styles_xml
    );

    // Verify each named style is present
    assert!(
        styles_xml.contains("name=\"Heading 1\""),
        "Missing Heading 1 cellStyle"
    );
    assert!(
        styles_xml.contains("builtinId=\"16\""),
        "Missing builtinId=16 for Heading 1"
    );
    assert!(
        styles_xml.contains("name=\"Currency\""),
        "Missing Currency cellStyle"
    );
    assert!(
        styles_xml.contains("builtinId=\"4\""),
        "Missing builtinId=4 for Currency"
    );
    assert!(
        styles_xml.contains("name=\"Percent\""),
        "Missing Percent cellStyle"
    );
    assert!(
        styles_xml.contains("builtinId=\"5\""),
        "Missing builtinId=5 for Percent"
    );
}

#[test]
fn test_cell_style_xf_id_reference() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let heading_fmt = Format::new().bold().cell_style("Title");
    ws.write_with_format(0, 0, "My Title", &heading_fmt)
        .unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let styles_xml = extract_xml_from_buffer(&buf, "xl/styles.xml").unwrap();

    // The cellXfs should have an xf with xfId="1" (referencing the Title cellStyleXfs entry)
    assert!(
        styles_xml.contains("xfId=\"1\""),
        "Expected xfId=\"1\" in cellXfs for Title style, got:\n{}",
        styles_xml
    );

    // Verify Title style with builtinId=15
    assert!(
        styles_xml.contains("name=\"Title\""),
        "Missing Title cellStyle"
    );
    assert!(
        styles_xml.contains("builtinId=\"15\""),
        "Missing builtinId=15 for Title"
    );
}

#[test]
fn test_cell_style_builder_method() {
    let fmt = Format::new().cell_style("Good");
    assert_eq!(fmt.get_cell_style(), Some("Good"));

    let fmt_none = Format::new();
    assert_eq!(fmt_none.get_cell_style(), None);
}
