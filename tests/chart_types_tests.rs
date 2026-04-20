//! Tests for new chart types: Bubble, Waterfall, Funnel, Sunburst, Histogram, BoxWhisker.

use zavora_xlsx::{
    Chart, ChartType, Workbook,
    WaterfallChart, WaterfallPointType, FunnelChart, SunburstChart,
    HistogramChart, BoxWhiskerChart,
};

// ── Task 26.4: Bubble Chart ──

#[test]
fn test_bubble_chart_creation_and_xml() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();

    // Write sample data
    for i in 0..5 {
        ws.write(i, 0, (i + 1) as f64).unwrap();
        ws.write(i, 1, ((i + 1) * 10) as f64).unwrap();
        ws.write(i, 2, ((i + 1) * 5) as f64).unwrap();
    }

    let mut chart = Chart::new(ChartType::Bubble);
    chart.set_title("Bubble Test");
    let series = chart.add_series();
    series
        .set_name("Series 1")
        .set_categories("Data!$A$1:$A$5")
        .set_values("Data!$B$1:$B$5")
        .set_bubble_sizes("Data!$C$1:$C$5");

    ws.insert_chart(6, 0, &chart).unwrap();

    // Save and verify it doesn't error
    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Read back and verify chart type
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();
    let charts = ws2.charts();
    assert_eq!(charts.len(), 1);
    assert_eq!(charts[0].chart_type(), ChartType::Bubble);
    assert_eq!(charts[0].title(), Some("Bubble Test"));
}

#[test]
fn test_bubble_chart_xml_contains_bubble_size() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, 1.0).unwrap();
    ws.write(0, 1, 10.0).unwrap();
    ws.write(0, 2, 5.0).unwrap();

    let mut chart = Chart::new(ChartType::Bubble);
    let series = chart.add_series();
    series
        .set_values("Sheet1!$B$1:$B$1")
        .set_bubble_sizes("Sheet1!$C$1:$C$1");

    ws.insert_chart(2, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    // Verify the file is valid by opening it
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    assert_eq!(wb2.worksheet_ref(0).unwrap().charts().len(), 1);
}

// ── Task 27.4: Waterfall Chart ──

#[test]
fn test_waterfall_chart_creation() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = WaterfallChart::new();
    chart.set_title("Revenue Waterfall");
    chart.set_series_name("Revenue");
    chart.add_point("Starting", 100.0, WaterfallPointType::Total);
    chart.add_point("Product A", 30.0, WaterfallPointType::Increase);
    chart.add_point("Product B", 20.0, WaterfallPointType::Increase);
    chart.add_point("Returns", -15.0, WaterfallPointType::Decrease);
    chart.add_point("Ending", 135.0, WaterfallPointType::Total);

    ws.insert_waterfall(0, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Verify the file can be opened (chartEx parts are present)
    let _wb2 = Workbook::open_from_buffer(&buf).unwrap();
}

#[test]
fn test_waterfall_chart_xml_structure() {
    let mut chart = WaterfallChart::new();
    chart.set_title("Test Waterfall");
    chart.add_point("Start", 50.0, WaterfallPointType::Total);
    chart.add_point("Add", 25.0, WaterfallPointType::Increase);
    chart.add_point("Sub", -10.0, WaterfallPointType::Decrease);
    chart.add_point("End", 65.0, WaterfallPointType::Total);

    let cex = zavora_xlsx::features::chartex::ChartExChart::Waterfall(chart);
    let xml = zavora_xlsx::writer::chartex_writer::write_chartex_generic_xml(&cex, 1);
    let xml_str = String::from_utf8(xml).unwrap();

    assert!(xml_str.contains("cx:chartSpace"));
    assert!(xml_str.contains("layoutId=\"waterfall\""));
    assert!(xml_str.contains("cx:subtotal"));
    assert!(xml_str.contains("Test Waterfall"));
    assert!(xml_str.contains("Start"));
    assert!(xml_str.contains("End"));
}

// ── Task 28.3: Funnel Chart ──

#[test]
fn test_funnel_chart_creation() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = FunnelChart::new();
    chart.set_title("Sales Funnel");
    chart.set_series_name("Leads");
    chart.add_point("Prospects", 1000.0);
    chart.add_point("Qualified", 600.0);
    chart.add_point("Proposals", 300.0);
    chart.add_point("Negotiations", 150.0);
    chart.add_point("Closed", 75.0);

    ws.insert_funnel(0, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());
    let _wb2 = Workbook::open_from_buffer(&buf).unwrap();
}

#[test]
fn test_funnel_chart_xml_structure() {
    let mut chart = FunnelChart::new();
    chart.set_title("Test Funnel");
    chart.add_point("Stage 1", 500.0);
    chart.add_point("Stage 2", 300.0);
    chart.add_point("Stage 3", 100.0);

    let cex = zavora_xlsx::features::chartex::ChartExChart::Funnel(chart);
    let xml = zavora_xlsx::writer::chartex_writer::write_chartex_generic_xml(&cex, 1);
    let xml_str = String::from_utf8(xml).unwrap();

    assert!(xml_str.contains("cx:chartSpace"));
    assert!(xml_str.contains("layoutId=\"funnel\""));
    assert!(xml_str.contains("Test Funnel"));
    assert!(xml_str.contains("Stage 1"));
    assert!(xml_str.contains("500"));
}

// ── Task 29.4: Sunburst Chart ──

#[test]
fn test_sunburst_chart_with_3_levels() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = SunburstChart::new();
    chart.set_title("Organization Sunburst");
    chart.set_series_name("Headcount");

    // 3 levels: Company > Department > Team
    // Level 1 (innermost): Company
    chart.add_level(&["Corp", "Corp", "Corp", "Corp", "Corp", "Corp"]);
    // Level 2: Department
    chart.add_level(&["Engineering", "Engineering", "Engineering", "Sales", "Sales", "Marketing"]);
    // Level 3 (outermost): Team
    chart.add_level(&["Frontend", "Backend", "DevOps", "Enterprise", "SMB", "Digital"]);

    chart.set_values(&[15.0, 20.0, 8.0, 12.0, 10.0, 7.0]);

    ws.insert_sunburst(0, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());
    let _wb2 = Workbook::open_from_buffer(&buf).unwrap();
}

#[test]
fn test_sunburst_chart_xml_structure() {
    let mut chart = SunburstChart::new();
    chart.set_title("Test Sunburst");
    chart.add_level(&["A", "A", "B", "B"]);
    chart.add_level(&["A1", "A2", "B1", "B2"]);
    chart.add_level(&["X", "Y", "Z", "W"]);
    chart.set_values(&[10.0, 20.0, 30.0, 40.0]);

    let cex = zavora_xlsx::features::chartex::ChartExChart::Sunburst(chart);
    let xml = zavora_xlsx::writer::chartex_writer::write_chartex_generic_xml(&cex, 1);
    let xml_str = String::from_utf8(xml).unwrap();

    assert!(xml_str.contains("cx:chartSpace"));
    assert!(xml_str.contains("layoutId=\"sunburst\""));
    assert!(xml_str.contains("Test Sunburst"));
    // Multi-level hierarchy: should have multiple cx:lvl elements
    assert!(xml_str.matches("cx:lvl").count() >= 4); // 3 str levels + 1 num level
}

// ── Task 30.4: Histogram Chart ──

#[test]
fn test_histogram_chart_with_custom_bins() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = HistogramChart::new();
    chart.set_title("Score Distribution");
    chart.set_series_name("Scores");
    chart.set_values(&[55.0, 62.0, 71.0, 78.0, 82.0, 85.0, 88.0, 91.0, 95.0, 99.0]);
    chart.set_bin_count(5);

    ws.insert_histogram(0, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());
    let _wb2 = Workbook::open_from_buffer(&buf).unwrap();
}

#[test]
fn test_histogram_chart_xml_structure() {
    let mut chart = HistogramChart::new();
    chart.set_title("Test Histogram");
    chart.set_values(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    chart.set_bin_count(3);

    let cex = zavora_xlsx::features::chartex::ChartExChart::Histogram(chart);
    let xml = zavora_xlsx::writer::chartex_writer::write_chartex_generic_xml(&cex, 1);
    let xml_str = String::from_utf8(xml).unwrap();

    assert!(xml_str.contains("cx:chartSpace"));
    assert!(xml_str.contains("layoutId=\"clusteredColumn\""));
    assert!(xml_str.contains("cx:binning"));
    assert!(xml_str.contains("binCount=\"3\""));
    assert!(xml_str.contains("Test Histogram"));
}

#[test]
fn test_pareto_chart_xml_structure() {
    let mut chart = HistogramChart::pareto();
    chart.set_title("Pareto Analysis");
    chart.set_values(&[10.0, 20.0, 30.0, 40.0, 50.0]);
    chart.set_bin_width(10.0);

    let cex = zavora_xlsx::features::chartex::ChartExChart::Histogram(chart);
    let xml = zavora_xlsx::writer::chartex_writer::write_chartex_generic_xml(&cex, 1);
    let xml_str = String::from_utf8(xml).unwrap();

    assert!(xml_str.contains("layoutId=\"paretoLine\""));
    assert!(xml_str.contains("binWidth=\"10\""));
}

// ── Task 31.4: Box & Whisker Chart ──

#[test]
fn test_box_whisker_chart_creation() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    let mut chart = BoxWhiskerChart::new();
    chart.set_title("Test Scores by Class");
    chart.set_series_name("Scores");
    chart.add_data_set("Class A", &[72.0, 85.0, 90.0, 68.0, 95.0, 78.0, 82.0]);
    chart.add_data_set("Class B", &[65.0, 70.0, 88.0, 92.0, 75.0, 80.0, 85.0]);
    chart.add_data_set("Class C", &[55.0, 60.0, 78.0, 82.0, 70.0, 75.0, 90.0]);
    chart.set_show_outliers(true);
    chart.set_show_mean_markers(true);
    chart.set_show_inner_points(false);

    ws.insert_box_whisker(0, 0, &chart).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());
    let _wb2 = Workbook::open_from_buffer(&buf).unwrap();
}

#[test]
fn test_box_whisker_chart_xml_structure() {
    let mut chart = BoxWhiskerChart::new();
    chart.set_title("Test BoxWhisker");
    chart.add_data_set("Group 1", &[10.0, 20.0, 30.0, 40.0, 50.0]);
    chart.add_data_set("Group 2", &[15.0, 25.0, 35.0, 45.0, 55.0]);
    chart.set_show_outliers(true);
    chart.set_show_mean_markers(false);
    chart.set_show_inner_points(true);

    let cex = zavora_xlsx::features::chartex::ChartExChart::BoxWhisker(chart);
    let xml = zavora_xlsx::writer::chartex_writer::write_chartex_generic_xml(&cex, 1);
    let xml_str = String::from_utf8(xml).unwrap();

    assert!(xml_str.contains("cx:chartSpace"));
    assert!(xml_str.contains("layoutId=\"boxWhisker\""));
    assert!(xml_str.contains("cx:statistics"));
    assert!(xml_str.contains("quartileMethod=\"exclusive\""));
    assert!(xml_str.contains("Group 1"));
    assert!(xml_str.contains("Group 2"));
    // Two separate data blocks
    assert!(xml_str.contains("<cx:data id=\"0\">"));
    assert!(xml_str.contains("<cx:data id=\"1\">"));
    // Two separate series
    assert_eq!(xml_str.matches("layoutId=\"boxWhisker\"").count(), 2);
}
