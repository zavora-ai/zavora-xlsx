//! Tests for axis formatting (Task 36): numFmt, txPr, majorTickMark, majorGridlines.

use zavora_xlsx::{AxisFormat, Chart, ChartType, TickMark, Workbook};

/// Helper: create a chart with axis formatting and return the chart XML string.
fn create_chart_with_axis_format(x_fmt: Option<AxisFormat>, y_fmt: Option<AxisFormat>) -> String {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();
    for i in 0..5u32 {
        ws.write(i, 0, format!("Cat{}", i + 1)).unwrap();
        ws.write(i, 1, (i + 1) as f64 * 100.0).unwrap();
    }

    let mut chart = Chart::new(ChartType::Bar);
    chart.set_title("Axis Format Test");
    if let Some(fmt) = x_fmt {
        chart.set_x_axis_format(fmt);
    }
    if let Some(fmt) = y_fmt {
        chart.set_y_axis_format(fmt);
    }
    let s = chart.add_series();
    s.set_categories("Data!$A$1:$A$5")
        .set_values("Data!$B$1:$B$5");
    ws.insert_chart(6, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Extract chart XML from the zip
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    for i in 0..archive.len() {
        let name = archive.name_for_index(i).unwrap().to_string();
        if name.contains("chart")
            && name.ends_with(".xml")
            && !name.contains("chartEx")
            && !name.contains("style")
            && !name.contains("colors")
        {
            let mut file = archive.by_index(i).unwrap();
            let mut contents = String::new();
            std::io::Read::read_to_string(&mut file, &mut contents).unwrap();
            return contents;
        }
    }
    panic!("No chart XML found in archive");
}

#[test]
fn test_axis_num_format_serialization() {
    let mut y_fmt = AxisFormat::new();
    y_fmt.set_num_format("$#,##0.00");

    let xml = create_chart_with_axis_format(None, Some(y_fmt));

    // The value axis should have a custom numFmt with sourceLinked="0"
    assert!(
        xml.contains(r#"<c:numFmt formatCode="$#,##0.00" sourceLinked="0"/>"#),
        "Expected custom numFmt on value axis. XML:\n{xml}"
    );
}

#[test]
fn test_axis_major_tick_mark_serialization() {
    let mut x_fmt = AxisFormat::new();
    x_fmt.set_major_tick_mark(TickMark::Outside);

    let mut y_fmt = AxisFormat::new();
    y_fmt.set_major_tick_mark(TickMark::Cross);

    let xml = create_chart_with_axis_format(Some(x_fmt), Some(y_fmt));

    assert!(
        xml.contains(r#"<c:majorTickMark val="out"/>"#),
        "Expected majorTickMark=out on category axis. XML:\n{xml}"
    );
    assert!(
        xml.contains(r#"<c:majorTickMark val="cross"/>"#),
        "Expected majorTickMark=cross on value axis. XML:\n{xml}"
    );
}

#[test]
fn test_axis_minor_tick_mark_serialization() {
    let mut y_fmt = AxisFormat::new();
    y_fmt.set_minor_tick_mark(TickMark::Inside);

    let xml = create_chart_with_axis_format(None, Some(y_fmt));

    assert!(
        xml.contains(r#"<c:minorTickMark val="in"/>"#),
        "Expected minorTickMark=in on value axis. XML:\n{xml}"
    );
}

#[test]
fn test_axis_major_gridlines_serialization() {
    let mut y_fmt = AxisFormat::new();
    y_fmt.set_major_gridlines(true);

    let xml = create_chart_with_axis_format(None, Some(y_fmt));

    // Should have majorGridlines (empty tag since no color/width specified)
    assert!(
        xml.contains("c:majorGridlines"),
        "Expected c:majorGridlines on value axis. XML:\n{xml}"
    );
}

#[test]
fn test_axis_minor_gridlines_serialization() {
    let mut x_fmt = AxisFormat::new();
    x_fmt.set_minor_gridlines(true);

    let xml = create_chart_with_axis_format(Some(x_fmt), None);

    assert!(
        xml.contains("c:minorGridlines"),
        "Expected c:minorGridlines on category axis. XML:\n{xml}"
    );
}

#[test]
fn test_axis_font_txpr_serialization() {
    let mut x_fmt = AxisFormat::new();
    x_fmt.set_font_name("Arial");
    x_fmt.set_font_size(12.0);
    x_fmt.set_font_bold(true);
    x_fmt.set_font_color([255, 0, 0]);

    let xml = create_chart_with_axis_format(Some(x_fmt), None);

    // Should have txPr element
    assert!(
        xml.contains("c:txPr"),
        "Expected c:txPr element. XML:\n{xml}"
    );
    // Font size in hundredths of a point: 12.0 * 100 = 1200
    assert!(
        xml.contains(r#"sz="1200""#),
        "Expected sz=1200 for 12pt font. XML:\n{xml}"
    );
    // Bold
    assert!(
        xml.contains(r#"b="1""#),
        "Expected b=1 for bold. XML:\n{xml}"
    );
    // Font color
    assert!(
        xml.contains(r#"<a:srgbClr val="FF0000"/>"#),
        "Expected font color FF0000. XML:\n{xml}"
    );
    // Font name
    assert!(
        xml.contains(r#"<a:latin typeface="Arial"/>"#),
        "Expected latin typeface=Arial. XML:\n{xml}"
    );
}

#[test]
fn test_axis_gridline_with_color_and_width() {
    let mut y_fmt = AxisFormat::new();
    y_fmt.set_major_gridlines(true);
    y_fmt.set_gridline_color([128, 128, 128]);
    y_fmt.set_gridline_width(1.5);

    let xml = create_chart_with_axis_format(None, Some(y_fmt));

    // Should have majorGridlines with spPr containing line formatting
    assert!(
        xml.contains("c:majorGridlines"),
        "Expected c:majorGridlines. XML:\n{xml}"
    );
    assert!(
        xml.contains(r#"<a:srgbClr val="808080"/>"#),
        "Expected gridline color 808080. XML:\n{xml}"
    );
    // 1.5pt * 12700 = 19050 EMU
    assert!(
        xml.contains(r#"w="19050""#),
        "Expected line width 19050 EMU. XML:\n{xml}"
    );
}

#[test]
fn test_default_value_axis_has_major_gridlines() {
    // When no axis format is set, value axis should still have default majorGridlines
    let xml = create_chart_with_axis_format(None, None);

    assert!(
        xml.contains("c:majorGridlines"),
        "Expected default c:majorGridlines on value axis. XML:\n{xml}"
    );
}

#[test]
fn test_combined_axis_formatting() {
    let mut x_fmt = AxisFormat::new();
    x_fmt.set_font_name("Calibri");
    x_fmt.set_font_size(10.0);
    x_fmt.set_major_tick_mark(TickMark::Outside);

    let mut y_fmt = AxisFormat::new();
    y_fmt.set_num_format("#,##0");
    y_fmt.set_major_gridlines(true);
    y_fmt.set_major_tick_mark(TickMark::None);
    y_fmt.set_font_size(9.0);

    let xml = create_chart_with_axis_format(Some(x_fmt), Some(y_fmt));

    // X axis checks
    assert!(
        xml.contains(r#"<a:latin typeface="Calibri"/>"#),
        "Expected Calibri font. XML:\n{xml}"
    );
    assert!(
        xml.contains(r#"<c:majorTickMark val="out"/>"#),
        "Expected tick mark out. XML:\n{xml}"
    );

    // Y axis checks
    assert!(
        xml.contains(r##"<c:numFmt formatCode="#,##0" sourceLinked="0"/>"##),
        "Expected numFmt. XML:\n{xml}"
    );
    assert!(
        xml.contains(r#"<c:majorTickMark val="none"/>"#),
        "Expected tick mark none. XML:\n{xml}"
    );
    assert!(
        xml.contains("c:majorGridlines"),
        "Expected gridlines. XML:\n{xml}"
    );
}
