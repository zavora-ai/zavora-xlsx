use std::io::Read;
use zavora_xlsx::{Format, ThemeColorIndex, Workbook};

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
fn test_theme_color_serialization_accent1() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Use Accent1 theme color with no tint
    let fmt = Format::new().theme_color(ThemeColorIndex::Accent1, 0.0);
    ws.write_with_format(0, 0, "Accent1", &fmt).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let styles_xml = extract_xml_from_buffer(&buf, "xl/styles.xml").unwrap();

    // Should contain theme="4" (Accent1 = index 4)
    assert!(
        styles_xml.contains("theme=\"4\""),
        "Expected theme=\"4\" in styles.xml for Accent1, got:\n{}",
        styles_xml
    );
    // Should NOT contain tint when tint is 0
    // The color element for this font should just be <color theme="4"/>
    assert!(
        styles_xml.contains("<color theme=\"4\"/>"),
        "Expected <color theme=\"4\"/> without tint in styles.xml, got:\n{}",
        styles_xml
    );
}

#[test]
fn test_theme_color_serialization_with_tint() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Use Dark1 theme color with positive tint (lighten)
    let fmt = Format::new().theme_color(ThemeColorIndex::Dark1, 0.5);
    ws.write_with_format(0, 0, "Dark1 lightened", &fmt).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let styles_xml = extract_xml_from_buffer(&buf, "xl/styles.xml").unwrap();

    // Should contain theme="0" (Dark1 = index 0) and tint="0.5"
    assert!(
        styles_xml.contains("theme=\"0\""),
        "Expected theme=\"0\" in styles.xml for Dark1, got:\n{}",
        styles_xml
    );
    assert!(
        styles_xml.contains("tint=\"0.5\""),
        "Expected tint=\"0.5\" in styles.xml, got:\n{}",
        styles_xml
    );
}

#[test]
fn test_theme_color_serialization_accent6_negative_tint() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Use Accent6 theme color with negative tint (darken)
    let fmt = Format::new().theme_color(ThemeColorIndex::Accent6, -0.25);
    ws.write_with_format(0, 0, "Accent6 darkened", &fmt)
        .unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let styles_xml = extract_xml_from_buffer(&buf, "xl/styles.xml").unwrap();

    // Should contain theme="9" (Accent6 = index 9) and tint="-0.25"
    assert!(
        styles_xml.contains("theme=\"9\""),
        "Expected theme=\"9\" in styles.xml for Accent6, got:\n{}",
        styles_xml
    );
    assert!(
        styles_xml.contains("tint=\"-0.25\""),
        "Expected tint=\"-0.25\" in styles.xml, got:\n{}",
        styles_xml
    );
}

#[test]
fn test_theme_color_index_from_index() {
    assert_eq!(ThemeColorIndex::from_index(0), Some(ThemeColorIndex::Dark1));
    assert_eq!(
        ThemeColorIndex::from_index(1),
        Some(ThemeColorIndex::Light1)
    );
    assert_eq!(
        ThemeColorIndex::from_index(4),
        Some(ThemeColorIndex::Accent1)
    );
    assert_eq!(
        ThemeColorIndex::from_index(9),
        Some(ThemeColorIndex::Accent6)
    );
    assert_eq!(ThemeColorIndex::from_index(10), None);
    assert_eq!(ThemeColorIndex::from_index(255), None);
}

#[test]
fn test_theme_color_default_rgb_values() {
    assert_eq!(ThemeColorIndex::Dark1.default_rgb(), [0x00, 0x00, 0x00]);
    assert_eq!(ThemeColorIndex::Light1.default_rgb(), [0xFF, 0xFF, 0xFF]);
    assert_eq!(ThemeColorIndex::Dark2.default_rgb(), [0x44, 0x54, 0x6A]);
    assert_eq!(ThemeColorIndex::Light2.default_rgb(), [0xE7, 0xE6, 0xE6]);
    assert_eq!(ThemeColorIndex::Accent1.default_rgb(), [0x44, 0x72, 0xC4]);
    assert_eq!(ThemeColorIndex::Accent2.default_rgb(), [0xED, 0x7D, 0x31]);
    assert_eq!(ThemeColorIndex::Accent3.default_rgb(), [0xA5, 0xA5, 0xA5]);
    assert_eq!(ThemeColorIndex::Accent4.default_rgb(), [0xFF, 0xC0, 0x00]);
    assert_eq!(ThemeColorIndex::Accent5.default_rgb(), [0x5B, 0x9B, 0xD5]);
    assert_eq!(ThemeColorIndex::Accent6.default_rgb(), [0x70, 0xAD, 0x47]);
}

#[test]
fn test_apply_tint_zero() {
    use zavora_xlsx::format::apply_tint;
    let rgb = [0x44, 0x72, 0xC4];
    assert_eq!(apply_tint(rgb, 0.0), rgb);
}

#[test]
fn test_apply_tint_positive_lightens() {
    use zavora_xlsx::format::apply_tint;
    // Black with tint 0.5 should become gray (127, 127, 127) approximately
    let result = apply_tint([0, 0, 0], 0.5);
    assert_eq!(result, [128, 128, 128]); // 0 + (255-0)*0.5 = 127.5 → 128

    // Black with tint 1.0 should become white
    let result = apply_tint([0, 0, 0], 1.0);
    assert_eq!(result, [255, 255, 255]);
}

#[test]
fn test_apply_tint_negative_darkens() {
    use zavora_xlsx::format::apply_tint;
    // White with tint -0.5 should become gray
    let result = apply_tint([255, 255, 255], -0.5);
    assert_eq!(result, [128, 128, 128]); // 255 * (1 + (-0.5)) = 127.5 → 128

    // White with tint -1.0 should become black
    let result = apply_tint([255, 255, 255], -1.0);
    assert_eq!(result, [0, 0, 0]);
}

#[test]
fn test_theme_color_read_back() {
    // Create a workbook with theme colors, save, and read back
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let fmt = Format::new().theme_color(ThemeColorIndex::Accent1, 0.4);
    ws.write_with_format(0, 0, "Theme colored", &fmt).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Write to temp file and read back
    let tmp_path = std::env::temp_dir().join("theme_color_test_readback.xlsx");
    std::fs::write(&tmp_path, &buf).unwrap();

    let mut wb2 = Workbook::open(&tmp_path).unwrap();
    let ws2 = wb2.worksheet(0).unwrap();

    // The reader resolves theme colors to RGB, so we should get a font color
    if let Some(fmt) = ws2.cell_format(0, 0) {
        // Accent1 base is [0x44, 0x72, 0xC4], with tint 0.4 it should lighten
        if let Some(color) = fmt.get_font_color() {
            // Verify it's a lightened version of Accent1
            // 0x44 + (0xFF - 0x44) * 0.4 = 68 + 187*0.4 = 68 + 74.8 = 142.8 → 143
            // 0x72 + (0xFF - 0x72) * 0.4 = 114 + 141*0.4 = 114 + 56.4 = 170.4 → 170
            // 0xC4 + (0xFF - 0xC4) * 0.4 = 196 + 59*0.4 = 196 + 23.6 = 219.6 → 220
            assert_eq!(color[0], 143, "Red channel mismatch");
            assert_eq!(color[1], 170, "Green channel mismatch");
            assert_eq!(color[2], 220, "Blue channel mismatch");
        }
    }

    // Clean up
    let _ = std::fs::remove_file(&tmp_path);
}

#[test]
fn test_theme_color_example_output() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write header
    let header_fmt = Format::new().bold();
    ws.write_with_format(0, 0, "Theme Color", &header_fmt)
        .unwrap();
    ws.write_with_format(0, 1, "Tint", &header_fmt).unwrap();
    ws.write_with_format(0, 2, "Sample Text", &header_fmt)
        .unwrap();

    // Write theme color samples
    let colors = [
        ("Dark1", ThemeColorIndex::Dark1),
        ("Light1", ThemeColorIndex::Light1),
        ("Dark2", ThemeColorIndex::Dark2),
        ("Light2", ThemeColorIndex::Light2),
        ("Accent1", ThemeColorIndex::Accent1),
        ("Accent2", ThemeColorIndex::Accent2),
        ("Accent3", ThemeColorIndex::Accent3),
        ("Accent4", ThemeColorIndex::Accent4),
        ("Accent5", ThemeColorIndex::Accent5),
        ("Accent6", ThemeColorIndex::Accent6),
    ];

    let mut row = 1u32;
    for (name, idx) in &colors {
        // No tint
        let fmt = Format::new().theme_color(*idx, 0.0);
        ws.write(row, 0, *name).unwrap();
        ws.write(row, 1, "0.0").unwrap();
        ws.write_with_format(row, 2, "Sample", &fmt).unwrap();
        row += 1;

        // Positive tint (lighten)
        let fmt = Format::new().theme_color(*idx, 0.4);
        ws.write(row, 0, *name).unwrap();
        ws.write(row, 1, "0.4").unwrap();
        ws.write_with_format(row, 2, "Lightened", &fmt).unwrap();
        row += 1;

        // Negative tint (darken)
        let fmt = Format::new().theme_color(*idx, -0.25);
        ws.write(row, 0, *name).unwrap();
        ws.write(row, 1, "-0.25").unwrap();
        ws.write_with_format(row, 2, "Darkened", &fmt).unwrap();
        row += 1;
    }

    // Set column widths
    let _ = ws.set_column_width(0, 15.0);
    let _ = ws.set_column_width(1, 10.0);
    let _ = ws.set_column_width(2, 20.0);

    wb.save("output/theme_colors_example.xlsx").unwrap();
}

#[test]
fn test_multiple_theme_colors_in_same_workbook() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let fmt1 = Format::new().theme_color(ThemeColorIndex::Accent1, 0.0);
    let fmt2 = Format::new().theme_color(ThemeColorIndex::Accent2, 0.0);
    let fmt3 = Format::new().theme_color(ThemeColorIndex::Dark2, 0.5);

    ws.write_with_format(0, 0, "Accent1", &fmt1).unwrap();
    ws.write_with_format(1, 0, "Accent2", &fmt2).unwrap();
    ws.write_with_format(2, 0, "Dark2 light", &fmt3).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let styles_xml = extract_xml_from_buffer(&buf, "xl/styles.xml").unwrap();

    // All three theme references should be present
    assert!(
        styles_xml.contains("theme=\"4\""),
        "Missing Accent1 theme ref"
    );
    assert!(
        styles_xml.contains("theme=\"5\""),
        "Missing Accent2 theme ref"
    );
    assert!(
        styles_xml.contains("theme=\"2\""),
        "Missing Dark2 theme ref"
    );
}

#[test]
fn test_theme_color_with_other_formatting() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Combine theme color with bold and font size
    let fmt = Format::new()
        .bold()
        .font_size(14.0)
        .theme_color(ThemeColorIndex::Accent3, 0.0);
    ws.write_with_format(0, 0, "Bold Accent3", &fmt).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let styles_xml = extract_xml_from_buffer(&buf, "xl/styles.xml").unwrap();

    assert!(
        styles_xml.contains("theme=\"6\""),
        "Missing Accent3 theme ref"
    );
    assert!(styles_xml.contains("<b/>"), "Missing bold tag");
    assert!(styles_xml.contains("val=\"14.0\""), "Missing font size");
}
