use zavora_xlsx::{PivotAggregation, PivotTable, Timeline, TimelineLevel, Workbook};

#[test]
fn test_add_timeline_to_pivot_table() {
    let mut wb = Workbook::new();

    // Create source data
    let ws = wb.worksheet(0).unwrap();
    ws.set_name("Data").unwrap();
    ws.write(0, 0, "Date").unwrap();
    ws.write(0, 1, "Product").unwrap();
    ws.write(0, 2, "Amount").unwrap();
    ws.write(1, 0, "2024-01-15").unwrap();
    ws.write(1, 1, "Widget").unwrap();
    ws.write(1, 2, 100.0).unwrap();
    ws.write(2, 0, "2024-02-20").unwrap();
    ws.write(2, 1, "Gadget").unwrap();
    ws.write(2, 2, 250.0).unwrap();

    // Create pivot table on a new sheet
    let pivot_ws = wb.add_worksheet_with_name("PivotSheet").unwrap();
    let pt = PivotTable::new("TestPivot", "Data!$A$1:$C$3")
        .add_row_field("Product")
        .add_value_field("Amount", PivotAggregation::Sum);
    pivot_ws.add_pivot_table(0, 0, &pt).unwrap();

    // Add timeline
    let timeline = Timeline::new("DateFilter", "Date")
        .set_caption("Date Timeline")
        .set_level(TimelineLevel::Quarters);
    pivot_ws.add_timeline(10, 0, &timeline).unwrap();

    // Save and verify the file can be written
    let buffer = wb.save_to_buffer().unwrap();
    assert!(!buffer.is_empty());

    // Verify the zip contains timeline parts
    let cursor = std::io::Cursor::new(&buffer);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();

    // Check timeline XML part exists
    assert!(
        archive.by_name("xl/timelines/timeline1.xml").is_ok(),
        "Timeline XML part should exist"
    );

    // Check timeline cache XML part exists
    assert!(
        archive
            .by_name("xl/timelineCaches/timelineCache1.xml")
            .is_ok(),
        "Timeline cache XML part should exist"
    );

    // Check content types includes timeline entries
    let mut ct_content = String::new();
    {
        let mut ct_entry = archive.by_name("[Content_Types].xml").unwrap();
        std::io::Read::read_to_string(&mut ct_entry, &mut ct_content).unwrap();
    }
    assert!(
        ct_content.contains("application/vnd.ms-excel.timeline+xml"),
        "Content types should include timeline type"
    );
    assert!(
        ct_content.contains("application/vnd.ms-excel.timelineCache+xml"),
        "Content types should include timeline cache type"
    );
}

#[test]
fn test_timeline_levels() {
    let t_years = Timeline::new("T1", "Date").set_level(TimelineLevel::Years);
    assert_eq!(t_years.level, TimelineLevel::Years);

    let t_quarters = Timeline::new("T2", "Date").set_level(TimelineLevel::Quarters);
    assert_eq!(t_quarters.level, TimelineLevel::Quarters);

    let t_months = Timeline::new("T3", "Date").set_level(TimelineLevel::Months);
    assert_eq!(t_months.level, TimelineLevel::Months);

    let t_days = Timeline::new("T4", "Date").set_level(TimelineLevel::Days);
    assert_eq!(t_days.level, TimelineLevel::Days);
}

#[test]
fn test_timeline_default_level_is_months() {
    let t = Timeline::new("Test", "DateField");
    assert_eq!(t.level, TimelineLevel::Months);
    assert_eq!(t.name, "Test");
    assert_eq!(t.source_name, "DateField");
    assert_eq!(t.caption, "Test");
}

#[test]
fn test_timeline_set_caption() {
    let t = Timeline::new("MyTimeline", "OrderDate").set_caption("Filter Orders by Date");
    assert_eq!(t.caption, "Filter Orders by Date");
}

#[test]
fn test_timeline_xml_level_values() {
    assert_eq!(TimelineLevel::Years.to_xml_value(), 0);
    assert_eq!(TimelineLevel::Quarters.to_xml_value(), 1);
    assert_eq!(TimelineLevel::Months.to_xml_value(), 2);
    assert_eq!(TimelineLevel::Days.to_xml_value(), 3);
}

#[test]
fn test_timeline_writer_output() {
    use zavora_xlsx::writer::timeline_writer;

    let timeline = Timeline::new("SalesDate", "Date")
        .set_caption("Sales by Date")
        .set_level(TimelineLevel::Days);

    let xml_bytes = timeline_writer::write_timeline_xml(&[timeline]);
    let xml_str = String::from_utf8(xml_bytes).unwrap();

    assert!(xml_str.contains("timelines"));
    assert!(xml_str.contains(r#"name="SalesDate""#));
    assert!(xml_str.contains(r#"cache="NativeTimeline_SalesDate""#));
    assert!(xml_str.contains(r#"caption="Sales by Date""#));
    assert!(xml_str.contains(r#"level="3""#)); // Days = 3
}

#[test]
fn test_timeline_cache_writer_output() {
    use zavora_xlsx::writer::timeline_writer;

    let timeline = Timeline::new("OrderDate", "Date").set_level(TimelineLevel::Quarters);

    let xml_bytes = timeline_writer::write_timeline_cache_xml(&timeline, 1);
    let xml_str = String::from_utf8(xml_bytes).unwrap();

    assert!(xml_str.contains("timelineCacheDefinition"));
    assert!(xml_str.contains(r#"name="NativeTimeline_OrderDate""#));
    assert!(xml_str.contains(r#"sourceName="Date""#));
    assert!(xml_str.contains("pivotTables"));
    assert!(xml_str.contains("state"));
}

#[test]
fn test_worksheet_timelines_accessor() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Initially empty
    assert!(ws.timelines().is_empty());

    // Add a timeline
    let timeline = Timeline::new("T1", "Date");
    ws.add_timeline(5, 3, &timeline).unwrap();

    assert_eq!(ws.timelines().len(), 1);
    assert_eq!(ws.timelines()[0].name, "T1");
}
