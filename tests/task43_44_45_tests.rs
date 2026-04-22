// Tests for Task 43 (Custom Table Styles), Task 44 (Phonetic Text), Task 45 (Text Effects)

use zavora_xlsx::{
    CustomTableStyle, Format, PhoneticRun, Table, TableColumn, TableStyleElementFormat, Workbook,
};

/// Task 43.4: Custom table style, verify XML
#[test]
fn test_custom_table_style_xml() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write some data for the table
    ws.write(0, 0, "Name").unwrap();
    ws.write(0, 1, "Value").unwrap();
    ws.write(1, 0, "A").unwrap();
    ws.write(1, 1, 1).unwrap();
    ws.write(2, 0, "B").unwrap();
    ws.write(2, 1, 2).unwrap();

    // Create a custom table style
    let custom_style = CustomTableStyle::new("MyCustomStyle")
        .first_row_stripe_size(2)
        .second_row_stripe_size(1)
        .header_row(TableStyleElementFormat::new().bg_color([0, 0, 255]).bold())
        .first_row_stripe(TableStyleElementFormat::new().bg_color([200, 200, 200]))
        .second_row_stripe(TableStyleElementFormat::new().bg_color([255, 255, 255]));

    let mut table = Table::new();
    table.set_columns(&[TableColumn::new("Name"), TableColumn::new("Value")]);
    table.set_custom_style(custom_style);

    ws.add_table(0, 0, 2, 1, &table).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Verify the table XML references the custom style name
    let table_xml = extract_file_from_xlsx(&buf, "xl/tables/table1.xml");
    let table_str = String::from_utf8(table_xml).unwrap();
    assert!(
        table_str.contains("name=\"MyCustomStyle\""),
        "Table XML should reference custom style name. Got: {}",
        table_str
    );

    // Verify the styles XML contains the tableStyles element
    let styles_xml = extract_file_from_xlsx(&buf, "xl/styles.xml");
    let styles_str = String::from_utf8(styles_xml).unwrap();
    assert!(
        styles_str.contains("<tableStyles"),
        "Styles XML should contain <tableStyles>. Got: {}",
        styles_str
    );
    assert!(
        styles_str.contains("name=\"MyCustomStyle\""),
        "Styles XML should contain custom style name"
    );
    assert!(
        styles_str.contains("type=\"headerRow\""),
        "Styles XML should contain headerRow element"
    );
    assert!(
        styles_str.contains("type=\"firstRowStripe\""),
        "Styles XML should contain firstRowStripe element"
    );
    assert!(
        styles_str.contains("size=\"2\""),
        "Styles XML should contain stripe size 2"
    );
    assert!(
        styles_str.contains("type=\"secondRowStripe\""),
        "Styles XML should contain secondRowStripe element"
    );
}

/// Task 44.4: Add phonetic text, verify XML elements
#[test]
fn test_phonetic_text_xml() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write a Japanese word
    ws.write(0, 0, "漢字").unwrap();

    // Add phonetic (furigana) runs
    let runs = vec![PhoneticRun::new(0, 1, "かん"), PhoneticRun::new(1, 2, "じ")];
    ws.set_phonetic(0, 0, runs).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Extract the sheet XML
    let sheet_xml = extract_file_from_xlsx(&buf, "xl/worksheets/sheet1.xml");
    let sheet_str = String::from_utf8(sheet_xml).unwrap();

    // Verify rPh elements
    assert!(
        sheet_str.contains("<rPh"),
        "Sheet XML should contain <rPh> elements. Got: {}",
        sheet_str
    );
    assert!(
        sheet_str.contains("sb=\"0\""),
        "Should have sb=\"0\" for first phonetic run"
    );
    assert!(
        sheet_str.contains("eb=\"1\""),
        "Should have eb=\"1\" for first phonetic run"
    );
    assert!(
        sheet_str.contains("sb=\"1\""),
        "Should have sb=\"1\" for second phonetic run"
    );
    assert!(
        sheet_str.contains("eb=\"2\""),
        "Should have eb=\"2\" for second phonetic run"
    );
    assert!(
        sheet_str.contains("かん"),
        "Should contain phonetic text かん"
    );
    assert!(sheet_str.contains("じ"), "Should contain phonetic text じ");

    // Verify phoneticPr element
    assert!(
        sheet_str.contains("<phoneticPr"),
        "Sheet XML should contain <phoneticPr> element"
    );
    assert!(
        sheet_str.contains("fontId=\"0\""),
        "phoneticPr should have fontId attribute"
    );
}

/// Task 45.4: Apply text effects, verify font XML
#[test]
fn test_text_effects_xml() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Apply text effects
    let fmt = Format::new().shadow().outline().emboss().engrave();
    ws.write_with_format(0, 0, "Effects", &fmt).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Extract the styles XML
    let styles_xml = extract_file_from_xlsx(&buf, "xl/styles.xml");
    let styles_str = String::from_utf8(styles_xml).unwrap();

    // Verify text effect elements in font
    assert!(
        styles_str.contains("<shadow/>"),
        "Styles XML should contain <shadow/>. Got: {}",
        styles_str
    );
    assert!(
        styles_str.contains("<outline/>"),
        "Styles XML should contain <outline/>"
    );
    assert!(
        styles_str.contains("<emboss/>"),
        "Styles XML should contain <emboss/>"
    );
    assert!(
        styles_str.contains("<engrave/>"),
        "Styles XML should contain <engrave/>"
    );
}

/// Test individual text effects
#[test]
fn test_individual_text_effects() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let fmt_shadow = Format::new().shadow();
    ws.write_with_format(0, 0, "Shadow", &fmt_shadow).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let styles_xml = extract_file_from_xlsx(&buf, "xl/styles.xml");
    let styles_str = String::from_utf8(styles_xml).unwrap();

    assert!(styles_str.contains("<shadow/>"), "Should contain <shadow/>");
    // Should NOT contain other effects
    assert!(
        !styles_str.contains("<outline/>"),
        "Should not contain <outline/>"
    );
    assert!(
        !styles_str.contains("<emboss/>"),
        "Should not contain <emboss/>"
    );
    assert!(
        !styles_str.contains("<engrave/>"),
        "Should not contain <engrave/>"
    );
}

// Helper function to extract a file from an xlsx (zip) buffer
fn extract_file_from_xlsx(buf: &[u8], path: &str) -> Vec<u8> {
    use std::io::{Cursor, Read};
    let reader = Cursor::new(buf);
    let mut archive = zip::ZipArchive::new(reader).unwrap();
    let mut contents = Vec::new();
    archive
        .by_name(path)
        .unwrap_or_else(|e| panic!("File '{}' not found in archive: {}", path, e))
        .read_to_end(&mut contents)
        .unwrap();
    contents
}
