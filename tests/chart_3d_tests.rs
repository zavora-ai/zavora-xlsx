//! Tests for 3D chart variants (Task 33.4).
//! Verifies that Column3D, Bar3D, Line3D, Pie3D, and Area3D charts
//! serialize correctly with view3D settings and proper chart element names.

use zavora_xlsx::{Chart, ChartType, View3D, Workbook};

/// Helper: create a workbook with a 3D chart, save to buffer, and return the chart XML as a string.
fn create_3d_chart_xml(chart_type: ChartType, view3d: Option<View3D>) -> String {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    // Write sample data
    for i in 0..5u32 {
        ws.write(i, 0, format!("Cat{}", i + 1)).unwrap();
        ws.write(i, 1, ((i + 1) * 10) as f64).unwrap();
    }

    let mut chart = Chart::new(chart_type);
    chart.set_title("3D Chart Test");
    if let Some(v) = view3d {
        chart.set_view3d(v);
    }
    let series = chart.add_series();
    series
        .set_name("Series 1")
        .set_categories("Data!$A$1:$A$5")
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
fn test_3d_column_chart_serialization() {
    let xml = create_3d_chart_xml(ChartType::Column3D, None);

    // Should use bar3DChart element with col direction
    assert!(
        xml.contains("c:bar3DChart"),
        "Expected c:bar3DChart element"
    );
    assert!(
        xml.contains("<c:barDir val=\"col\""),
        "Expected barDir=col for Column3D"
    );

    // Should have default view3D since none was explicitly set
    assert!(xml.contains("c:view3D"), "Expected c:view3D element");
    assert!(
        xml.contains("<c:rotX val=\"15\""),
        "Expected default rotX=15"
    );
    assert!(
        xml.contains("<c:rotY val=\"20\""),
        "Expected default rotY=20"
    );
    assert!(
        xml.contains("<c:perspective val=\"30\""),
        "Expected default perspective=30"
    );
    assert!(
        xml.contains("<c:rAngAx val=\"1\""),
        "Expected default rAngAx=1"
    );
}

#[test]
fn test_3d_bar_chart_serialization() {
    let xml = create_3d_chart_xml(ChartType::Bar3D, None);

    // Should use bar3DChart element with bar direction
    assert!(
        xml.contains("c:bar3DChart"),
        "Expected c:bar3DChart element"
    );
    assert!(
        xml.contains("<c:barDir val=\"bar\""),
        "Expected barDir=bar for Bar3D"
    );

    // Should have default view3D
    assert!(xml.contains("c:view3D"), "Expected c:view3D element");
}

#[test]
fn test_3d_line_chart_serialization() {
    let xml = create_3d_chart_xml(ChartType::Line3D, None);

    // Should use line3DChart element
    assert!(
        xml.contains("c:line3DChart"),
        "Expected c:line3DChart element"
    );

    // Should have default view3D
    assert!(xml.contains("c:view3D"), "Expected c:view3D element");
    assert!(
        xml.contains("<c:rotX val=\"15\""),
        "Expected default rotX=15"
    );
}

#[test]
fn test_3d_pie_chart_serialization() {
    let xml = create_3d_chart_xml(ChartType::Pie3D, None);

    // Should use pie3DChart element
    assert!(
        xml.contains("c:pie3DChart"),
        "Expected c:pie3DChart element"
    );

    // Should have default view3D
    assert!(xml.contains("c:view3D"), "Expected c:view3D element");
}

#[test]
fn test_3d_area_chart_serialization() {
    let xml = create_3d_chart_xml(ChartType::Area3D, None);

    // Should use area3DChart element
    assert!(
        xml.contains("c:area3DChart"),
        "Expected c:area3DChart element"
    );

    // Should have default view3D
    assert!(xml.contains("c:view3D"), "Expected c:view3D element");
}

#[test]
fn test_3d_chart_custom_view3d() {
    let view = View3D {
        rot_x: -30,
        rot_y: 45,
        perspective: 100,
        right_angle_axes: false,
    };
    let xml = create_3d_chart_xml(ChartType::Column3D, Some(view));

    // Should use custom view3D values
    assert!(
        xml.contains("<c:rotX val=\"-30\""),
        "Expected custom rotX=-30"
    );
    assert!(
        xml.contains("<c:rotY val=\"45\""),
        "Expected custom rotY=45"
    );
    assert!(
        xml.contains("<c:perspective val=\"100\""),
        "Expected custom perspective=100"
    );
    assert!(
        xml.contains("<c:rAngAx val=\"0\""),
        "Expected custom rAngAx=0"
    );
}

#[test]
fn test_3d_chart_roundtrip_preserves_type() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    for i in 0..3u32 {
        ws.write(i, 0, format!("Cat{}", i + 1)).unwrap();
        ws.write(i, 1, ((i + 1) * 10) as f64).unwrap();
    }

    let mut chart = Chart::new(ChartType::Column3D);
    chart.set_title("3D Column Roundtrip");
    let series = chart.add_series();
    series
        .set_name("S1")
        .set_categories("Data!$A$1:$A$3")
        .set_values("Data!$B$1:$B$3");

    ws.insert_chart(4, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Read back and verify chart type is preserved
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let charts = ws2.charts();
    assert_eq!(charts.len(), 1);
    assert_eq!(charts[0].chart_type(), ChartType::Column3D);
    assert_eq!(charts[0].title(), Some("3D Column Roundtrip"));
}

#[test]
fn test_non_3d_chart_has_no_view3d() {
    let xml = create_3d_chart_xml(ChartType::Column, None);

    // Regular 2D chart should NOT have view3D
    assert!(
        !xml.contains("c:view3D"),
        "2D chart should not have c:view3D"
    );
    // Should use regular barChart, not bar3DChart
    assert!(
        xml.contains("c:barChart"),
        "Expected c:barChart for 2D Column"
    );
    assert!(
        !xml.contains("c:bar3DChart"),
        "2D Column should not use bar3DChart"
    );
}

// ── Task 35.4: Chart Style Themes ──

#[test]
fn test_chart_style_serialization() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    for i in 0..5u32 {
        ws.write(i, 0, format!("Cat{}", i + 1)).unwrap();
        ws.write(i, 1, ((i + 1) * 10) as f64).unwrap();
    }

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Styled Chart");
    chart.set_style(26);
    let series = chart.add_series();
    series
        .set_name("Series 1")
        .set_categories("Data!$A$1:$A$5")
        .set_values("Data!$B$1:$B$5");

    ws.insert_chart(6, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Extract chart XML from the zip
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut chart_xml = String::new();
    for i in 0..archive.len() {
        let name = archive.name_for_index(i).unwrap().to_string();
        if name.contains("chart")
            && name.ends_with(".xml")
            && !name.contains("chartEx")
            && !name.contains("style")
            && !name.contains("colors")
        {
            let mut file = archive.by_index(i).unwrap();
            std::io::Read::read_to_string(&mut file, &mut chart_xml).unwrap();
            break;
        }
    }
    assert!(!chart_xml.is_empty(), "No chart XML found in archive");

    // Verify the style element is present with the correct value
    assert!(
        chart_xml.contains("<c:style val=\"26\""),
        "Expected <c:style val=\"26\"/> in chart XML, got:\n{}",
        chart_xml
    );
}

#[test]
fn test_chart_style_not_emitted_when_none() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    for i in 0..3u32 {
        ws.write(i, 0, format!("Cat{}", i + 1)).unwrap();
        ws.write(i, 1, ((i + 1) * 10) as f64).unwrap();
    }

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("No Style");
    let series = chart.add_series();
    series
        .set_categories("Data!$A$1:$A$3")
        .set_values("Data!$B$1:$B$3");

    ws.insert_chart(4, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Extract chart XML
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut chart_xml = String::new();
    for i in 0..archive.len() {
        let name = archive.name_for_index(i).unwrap().to_string();
        if name.contains("chart")
            && name.ends_with(".xml")
            && !name.contains("chartEx")
            && !name.contains("style")
            && !name.contains("colors")
        {
            let mut file = archive.by_index(i).unwrap();
            std::io::Read::read_to_string(&mut file, &mut chart_xml).unwrap();
            break;
        }
    }
    assert!(!chart_xml.is_empty(), "No chart XML found in archive");

    // Verify no style element when not set
    assert!(
        !chart_xml.contains("c:style"),
        "Should not contain c:style when style is None"
    );
}

#[test]
fn test_chart_style_roundtrip() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    for i in 0..3u32 {
        ws.write(i, 0, format!("Cat{}", i + 1)).unwrap();
        ws.write(i, 1, ((i + 1) * 10) as f64).unwrap();
    }

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Style Roundtrip");
    chart.set_style(26);
    let series = chart.add_series();
    series
        .set_categories("Data!$A$1:$A$3")
        .set_values("Data!$B$1:$B$3");

    ws.insert_chart(4, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Read back and verify style is preserved
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let charts = ws2.charts();
    assert_eq!(charts.len(), 1);
    assert_eq!(
        charts[0].style(),
        Some(26),
        "Chart style should be preserved on roundtrip"
    );
}
