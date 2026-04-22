use std::io::{Cursor, Read};
use zavora_xlsx::{Format, GradientStop, Workbook};
use zip::ZipArchive;

#[test]
fn gradient_fill_serializes_correctly() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let fmt = Format::new().gradient_fill(
        90.0,
        vec![
            GradientStop {
                position: 0.0,
                color: [255, 0, 0],
            },
            GradientStop {
                position: 1.0,
                color: [0, 0, 255],
            },
        ],
    );
    ws.write_with_format(0, 0, "Gradient", &fmt).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Extract styles.xml from the xlsx zip
    let cursor = Cursor::new(buf);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut styles_xml = String::new();
    archive
        .by_name("xl/styles.xml")
        .unwrap()
        .read_to_string(&mut styles_xml)
        .unwrap();

    // Verify gradientFill element is present with correct attributes
    assert!(
        styles_xml.contains("<gradientFill"),
        "styles.xml should contain <gradientFill element"
    );
    assert!(
        styles_xml.contains("degree=\"90\""),
        "styles.xml should contain degree=\"90\""
    );
    assert!(
        styles_xml.contains("<stop position=\"0\""),
        "styles.xml should contain stop at position 0"
    );
    assert!(
        styles_xml.contains("<stop position=\"1\""),
        "styles.xml should contain stop at position 1"
    );
    assert!(
        styles_xml.contains("rgb=\"FFFF0000\""),
        "styles.xml should contain red color FF0000"
    );
    assert!(
        styles_xml.contains("rgb=\"FF0000FF\""),
        "styles.xml should contain blue color 0000FF"
    );
}

#[test]
fn gradient_fill_with_multiple_stops() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let fmt = Format::new().gradient_fill(
        45.0,
        vec![
            GradientStop {
                position: 0.0,
                color: [255, 0, 0],
            },
            GradientStop {
                position: 0.5,
                color: [0, 255, 0],
            },
            GradientStop {
                position: 1.0,
                color: [0, 0, 255],
            },
        ],
    );
    ws.write_with_format(0, 0, "Multi-stop", &fmt).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let cursor = Cursor::new(buf);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut styles_xml = String::new();
    archive
        .by_name("xl/styles.xml")
        .unwrap()
        .read_to_string(&mut styles_xml)
        .unwrap();

    assert!(
        styles_xml.contains("degree=\"45\""),
        "styles.xml should contain degree=\"45\""
    );
    assert!(
        styles_xml.contains("<stop position=\"0.5\""),
        "styles.xml should contain stop at position 0.5"
    );
    assert!(
        styles_xml.contains("rgb=\"FF00FF00\""),
        "styles.xml should contain green color 00FF00"
    );
}
