//! Tests for Surface chart variants (Task 34.3).
//! Verifies that Surface and WireframeSurface charts serialize correctly
//! with 3D view, series axis, band formats, and proper XML structure.

use zavora_xlsx::{Chart, ChartType, Workbook};

/// Helper: create a workbook with a surface chart, save to buffer, and return the chart XML.
fn create_surface_chart_xml(chart_type: ChartType) -> String {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    // Write a grid of data suitable for surface charts
    // Row headers (series names)
    ws.write(0, 0, "").unwrap();
    ws.write(0, 1, "X1").unwrap();
    ws.write(0, 2, "X2").unwrap();
    ws.write(0, 3, "X3").unwrap();

    // Data rows
    for row in 1..=3u32 {
        ws.write(row, 0, format!("Y{}", row)).unwrap();
        for col in 1..=3u16 {
            ws.write(row, col, (row * col as u32) as f64).unwrap();
        }
    }

    let mut chart = Chart::new(chart_type);
    chart.set_title("Surface Chart Test");

    // Add multiple series (typical for surface charts)
    for i in 1..=3u32 {
        let series = chart.add_series();
        series
            .set_name(&format!("Series {}", i))
            .set_values(&format!("Data!$B${}:$D${}", i + 1, i + 1));
    }

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
fn test_surface_chart_uses_surface3d_element() {
    let xml = create_surface_chart_xml(ChartType::Surface);

    // Should use surface3DChart element
    assert!(
        xml.contains("c:surface3DChart"),
        "Expected c:surface3DChart element"
    );
    // Wireframe should be 0 for solid surface
    assert!(
        xml.contains("<c:wireframe val=\"0\""),
        "Expected wireframe=0 for Surface"
    );
}

#[test]
fn test_wireframe_surface_chart_uses_wireframe_attribute() {
    let xml = create_surface_chart_xml(ChartType::WireframeSurface);

    // Should use surface3DChart element
    assert!(
        xml.contains("c:surface3DChart"),
        "Expected c:surface3DChart element"
    );
    // Wireframe should be 1 for wireframe surface
    assert!(
        xml.contains("<c:wireframe val=\"1\""),
        "Expected wireframe=1 for WireframeSurface"
    );
}

#[test]
fn test_surface_chart_has_view3d() {
    let xml = create_surface_chart_xml(ChartType::Surface);

    // Surface charts should have a 3D view
    assert!(
        xml.contains("c:view3D"),
        "Expected c:view3D element for surface chart"
    );
    assert!(
        xml.contains("<c:rotX val=\"15\""),
        "Expected default rotX=15"
    );
    assert!(
        xml.contains("<c:rotY val=\"20\""),
        "Expected default rotY=20"
    );
    // Surface charts use rAngAx=0 (perspective projection)
    assert!(
        xml.contains("<c:rAngAx val=\"0\""),
        "Expected rAngAx=0 for surface chart"
    );
}

#[test]
fn test_surface_chart_has_three_axes() {
    let xml = create_surface_chart_xml(ChartType::Surface);

    // Surface charts require 3 axis IDs in the chart type block
    // catAx, valAx, and serAx
    assert!(xml.contains("c:catAx"), "Expected c:catAx element");
    assert!(xml.contains("c:valAx"), "Expected c:valAx element");
    assert!(
        xml.contains("c:serAx"),
        "Expected c:serAx (series axis) element"
    );
}

#[test]
fn test_surface_chart_has_band_formats() {
    let xml = create_surface_chart_xml(ChartType::Surface);

    // Surface charts should have band formats for color bands
    assert!(xml.contains("c:bandFmts"), "Expected c:bandFmts element");
    assert!(xml.contains("c:bandFmt"), "Expected c:bandFmt elements");
}

#[test]
fn test_surface_chart_has_floor_and_walls() {
    let xml = create_surface_chart_xml(ChartType::Surface);

    // Surface charts should have floor, side wall, and back wall
    assert!(xml.contains("c:floor"), "Expected c:floor element");
    assert!(xml.contains("c:sideWall"), "Expected c:sideWall element");
    assert!(xml.contains("c:backWall"), "Expected c:backWall element");
}

#[test]
fn test_surface_chart_series_has_num_cache() {
    let xml = create_surface_chart_xml(ChartType::Surface);

    // Surface chart series should include numCache to avoid Excel crashes
    assert!(
        xml.contains("c:numCache"),
        "Expected c:numCache in surface chart series"
    );
    assert!(
        xml.contains("<c:formatCode>General</c:formatCode>"),
        "Expected formatCode in numCache"
    );
}

#[test]
fn test_surface_chart_roundtrip_preserves_type() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    for i in 0..3u32 {
        ws.write(i, 0, format!("Cat{}", i + 1)).unwrap();
        ws.write(i, 1, ((i + 1) * 10) as f64).unwrap();
    }

    let mut chart = Chart::new(ChartType::Surface);
    chart.set_title("Surface Roundtrip");
    let series = chart.add_series();
    series.set_name("S1").set_values("Data!$B$1:$B$3");

    ws.insert_chart(4, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Read back and verify chart type is preserved
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let charts = ws2.charts();
    assert_eq!(charts.len(), 1);
    assert_eq!(charts[0].chart_type(), ChartType::Surface);
    assert_eq!(charts[0].title(), Some("Surface Roundtrip"));
}
