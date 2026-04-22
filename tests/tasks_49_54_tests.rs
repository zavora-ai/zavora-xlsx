//! Tests for Tasks 49-54: External Data Connections, Power Query, Pivot Table Grouping,
//! Pivot Table Calculated Items, Sort State, and Advanced Autofilter.

use zavora_xlsx::{
    DateGroupLevel, FilterRule, PivotAggregation, PivotTable, SortDirection, Workbook,
};

// ============================================================
// Task 49: External Data Connections Preservation
// ============================================================

#[test]
fn test_connections_preservation_during_edit_mode_save() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Data").unwrap();

    // Add a connection
    wb.add_connection(
        "Provider=SQLOLEDB;Data Source=server;",
        "SELECT * FROM table1",
    );

    let buf = wb.save_to_buffer().unwrap();

    // Re-open and verify the connections.xml is preserved
    let wb2 = Workbook::open_from_buffer(&buf).unwrap();
    let has_connections = wb2
        .passthrough_entries()
        .iter()
        .any(|(name, _)| name == "xl/connections.xml");
    assert!(
        has_connections,
        "xl/connections.xml should be preserved in passthrough"
    );
}

#[test]
fn test_add_connection_method() {
    let mut wb = Workbook::new();
    wb.add_connection(
        "Provider=SQLOLEDB;Data Source=myserver;",
        "SELECT * FROM orders",
    );

    let buf = wb.save_to_buffer().unwrap();

    // Verify the workbook can be saved and re-opened
    let wb2 = Workbook::open_from_buffer(&buf).unwrap();
    let conn_entry = wb2
        .passthrough_entries()
        .iter()
        .find(|(name, _)| name == "xl/connections.xml");
    assert!(conn_entry.is_some(), "connections.xml should exist");

    let conn_xml = String::from_utf8_lossy(&conn_entry.unwrap().1);
    assert!(
        conn_xml.contains("connections"),
        "Should contain connections element"
    );
    assert!(conn_xml.contains("dbPr"), "Should contain dbPr element");
    assert!(
        conn_xml.contains("Provider=SQLOLEDB"),
        "Should contain connection string"
    );
    assert!(
        conn_xml.contains("SELECT * FROM orders"),
        "Should contain command"
    );
}

#[test]
fn test_connections_roundtrip_with_modifications() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Original").unwrap();
    wb.add_connection("DSN=MyDSN;", "EXEC sp_getData");

    let buf1 = wb.save_to_buffer().unwrap();

    // Re-open, modify, save again
    let mut wb2 = Workbook::open_from_buffer(&buf1).unwrap();
    let ws2 = wb2.worksheet(0).unwrap();
    ws2.write(1, 0, "Modified").unwrap();
    let buf2 = wb2.save_to_buffer().unwrap();

    // Verify connections still preserved
    let wb3 = Workbook::open_from_buffer(&buf2).unwrap();
    let conn_entry = wb3
        .passthrough_entries()
        .iter()
        .find(|(name, _)| name == "xl/connections.xml");
    assert!(
        conn_entry.is_some(),
        "connections.xml should survive round-trip"
    );
    let conn_xml = String::from_utf8_lossy(&conn_entry.unwrap().1);
    assert!(
        conn_xml.contains("DSN=MyDSN"),
        "Connection string should be preserved"
    );
}

// ============================================================
// Task 50: Power Query Preservation
// ============================================================

#[test]
fn test_custom_xml_preservation_during_save() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "PQ Data").unwrap();

    // Add customXml parts (simulating Power Query metadata)
    let fake_pq_xml = b"<PowerQueryMetadata><Query Name=\"GetData\"/></PowerQueryMetadata>";
    wb.add_passthrough_entry("customXml/item1.xml", fake_pq_xml.to_vec());
    wb.add_passthrough_entry("customXml/itemProps1.xml", b"<props/>".to_vec());

    let buf = wb.save_to_buffer().unwrap();

    // Re-open and verify customXml parts are preserved
    let wb2 = Workbook::open_from_buffer(&buf).unwrap();
    let has_custom_xml = wb2
        .passthrough_entries()
        .iter()
        .any(|(name, _)| name.starts_with("customXml/"));
    assert!(has_custom_xml, "customXml/ parts should be preserved");

    let item = wb2
        .passthrough_entries()
        .iter()
        .find(|(name, _)| name == "customXml/item1.xml");
    assert!(item.is_some(), "customXml/item1.xml should be preserved");
    let content = String::from_utf8_lossy(&item.unwrap().1);
    assert!(
        content.contains("PowerQueryMetadata"),
        "Power Query metadata should be preserved"
    );
}

#[test]
fn test_power_query_roundtrip_with_cell_modifications() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Initial").unwrap();

    // Add Power Query metadata
    wb.add_passthrough_entry("customXml/item1.xml", b"<PQ>query data</PQ>".to_vec());
    wb.add_passthrough_entry("customXml/_rels/item1.xml.rels", b"<rels/>".to_vec());

    let buf1 = wb.save_to_buffer().unwrap();

    // Re-open, modify cells, save
    let mut wb2 = Workbook::open_from_buffer(&buf1).unwrap();
    let ws2 = wb2.worksheet(0).unwrap();
    ws2.write(1, 0, "New data").unwrap();
    let buf2 = wb2.save_to_buffer().unwrap();

    // Verify PQ parts preserved
    let wb3 = Workbook::open_from_buffer(&buf2).unwrap();
    let pq_parts: Vec<_> = wb3
        .passthrough_entries()
        .iter()
        .filter(|(name, _)| name.starts_with("customXml/"))
        .collect();
    assert!(
        pq_parts.len() >= 2,
        "All customXml parts should be preserved, got {}",
        pq_parts.len()
    );
}

// ============================================================
// Task 51: Pivot Table Grouping
// ============================================================

#[test]
fn test_pivot_table_date_grouping() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Date").unwrap();
    ws.write(0, 1, "Amount").unwrap();
    ws.write(1, 0, "2023-01-15").unwrap();
    ws.write(1, 1, 100.0).unwrap();
    ws.write(2, 0, "2023-02-20").unwrap();
    ws.write(2, 1, 200.0).unwrap();
    ws.write(3, 0, "2023-03-10").unwrap();
    ws.write(3, 1, 150.0).unwrap();

    let pt = PivotTable::new("DateGroupPivot", "Sheet1!$A$1:$B$4")
        .add_row_field("Date")
        .add_value_field("Amount", PivotAggregation::Sum)
        .group_by_date("Date", &[DateGroupLevel::Years, DateGroupLevel::Months]);

    ws.add_pivot_table(6, 0, &pt).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    assert!(!buf.is_empty());

    // Re-open to verify round-trip
    let _wb2 = Workbook::open_from_buffer(&buf).unwrap();
}

#[test]
fn test_pivot_table_date_grouping_xml_content() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Date").unwrap();
    ws.write(0, 1, "Sales").unwrap();
    ws.write(1, 0, "Jan").unwrap();
    ws.write(1, 1, 500.0).unwrap();
    ws.write(2, 0, "Feb").unwrap();
    ws.write(2, 1, 600.0).unwrap();

    let pt = PivotTable::new("GroupTest", "Sheet1!$A$1:$B$3")
        .add_row_field("Date")
        .add_value_field("Sales", PivotAggregation::Sum)
        .group_by_date("Date", &[DateGroupLevel::Months]);

    ws.add_pivot_table(5, 0, &pt).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Read the pivot cache definition from the zip
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_field_group = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("pivotCacheDefinition") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("fieldGroup") && content.contains("rangePr") {
                found_field_group = true;
                assert!(content.contains("groupBy"), "Should have groupBy attribute");
            }
        }
    }
    assert!(
        found_field_group,
        "Should have fieldGroup and rangePr in cache definition"
    );
}

#[test]
fn test_pivot_table_range_grouping() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Category").unwrap();
    ws.write(0, 1, "Value").unwrap();
    ws.write(1, 0, "A").unwrap();
    ws.write(1, 1, 10.0).unwrap();
    ws.write(2, 0, "B").unwrap();
    ws.write(2, 1, 25.0).unwrap();
    ws.write(3, 0, "C").unwrap();
    ws.write(3, 1, 50.0).unwrap();

    let pt = PivotTable::new("RangeGroupPivot", "Sheet1!$A$1:$B$4")
        .add_row_field("Category")
        .add_value_field("Value", PivotAggregation::Sum)
        .group_by_range("Value", 0.0, 100.0, 10.0);

    ws.add_pivot_table(6, 0, &pt).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Read the pivot cache definition from the zip
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_range_pr = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("pivotCacheDefinition") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("rangePr") && content.contains("startNum") {
                found_range_pr = true;
                assert!(content.contains("endNum"), "Should have endNum");
                assert!(
                    content.contains("groupInterval"),
                    "Should have groupInterval"
                );
            }
        }
    }
    assert!(
        found_range_pr,
        "Should have rangePr with numeric range in cache definition"
    );
}

// ============================================================
// Task 52: Pivot Table Calculated Items
// ============================================================

#[test]
fn test_pivot_table_calculated_item() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Product").unwrap();
    ws.write(0, 1, "Revenue").unwrap();
    ws.write(0, 2, "Cost").unwrap();
    ws.write(1, 0, "Widget").unwrap();
    ws.write(1, 1, 1000.0).unwrap();
    ws.write(1, 2, 400.0).unwrap();
    ws.write(2, 0, "Gadget").unwrap();
    ws.write(2, 1, 2000.0).unwrap();
    ws.write(2, 2, 800.0).unwrap();

    let pt = PivotTable::new("CalcItemPivot", "Sheet1!$A$1:$C$3")
        .add_row_field("Product")
        .add_value_field("Revenue", PivotAggregation::Sum)
        .add_calculated_item("Profit", "Revenue - Cost");

    ws.add_pivot_table(5, 0, &pt).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Read the pivot table definition from the zip
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_calculated_item = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("pivotTable") && file.name().ends_with(".xml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("calculatedItem") {
                found_calculated_item = true;
                assert!(
                    content.contains("Profit"),
                    "Should contain calculated item name"
                );
                assert!(content.contains("Revenue - Cost"), "Should contain formula");
            }
        }
    }
    assert!(
        found_calculated_item,
        "Should have calculatedItem in pivot table definition"
    );
}

#[test]
fn test_pivot_table_multiple_calculated_items() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Item").unwrap();
    ws.write(0, 1, "Sales").unwrap();
    ws.write(1, 0, "A").unwrap();
    ws.write(1, 1, 100.0).unwrap();

    let pt = PivotTable::new("MultiCalcPivot", "Sheet1!$A$1:$B$2")
        .add_row_field("Item")
        .add_value_field("Sales", PivotAggregation::Sum)
        .add_calculated_item("Double", "Sales * 2")
        .add_calculated_item("Triple", "Sales * 3");

    ws.add_pivot_table(4, 0, &pt).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_items = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("pivotTable") && file.name().ends_with(".xml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("calculatedItems") {
                found_items = true;
                assert!(
                    content.contains("Double"),
                    "Should contain first calculated item"
                );
                assert!(
                    content.contains("Triple"),
                    "Should contain second calculated item"
                );
            }
        }
    }
    assert!(
        found_items,
        "Should have calculatedItems in pivot table definition"
    );
}

// ============================================================
// Task 53: Sort State
// ============================================================

#[test]
fn test_sort_state_serialization() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Name").unwrap();
    ws.write(0, 1, "Score").unwrap();
    ws.write(1, 0, "Alice").unwrap();
    ws.write(1, 1, 95.0).unwrap();
    ws.write(2, 0, "Bob").unwrap();
    ws.write(2, 1, 87.0).unwrap();
    ws.write(3, 0, "Charlie").unwrap();
    ws.write(3, 1, 92.0).unwrap();

    ws.set_autofilter(0, 0, 3, 1);
    ws.set_sort(1, SortDirection::Descending);

    let buf = wb.save_to_buffer().unwrap();

    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_sort_state = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("sheet1.xml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("sortState") {
                found_sort_state = true;
                assert!(
                    content.contains("sortCondition"),
                    "Should have sortCondition"
                );
                assert!(content.contains("descending=\"1\""), "Should be descending");
            }
        }
    }
    assert!(found_sort_state, "Should have sortState in sheet XML");
}

#[test]
fn test_sort_state_ascending() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Name").unwrap();
    ws.write(1, 0, "Zoe").unwrap();
    ws.write(2, 0, "Alice").unwrap();

    ws.set_autofilter(0, 0, 2, 0);
    ws.set_sort(0, SortDirection::Ascending);

    let buf = wb.save_to_buffer().unwrap();

    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_sort = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("sheet1.xml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("sortState") {
                found_sort = true;
                assert!(
                    content.contains("sortCondition"),
                    "Should have sortCondition"
                );
                // Ascending doesn't need descending="1"
                assert!(
                    !content.contains("descending"),
                    "Ascending sort should not have descending attribute"
                );
            }
        }
    }
    assert!(found_sort, "Should have sortState in sheet XML");
}

#[test]
fn test_sort_state_roundtrip_preservation() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();
    ws.write(0, 0, "Col1").unwrap();
    ws.write(1, 0, "B").unwrap();
    ws.write(2, 0, "A").unwrap();
    ws.set_autofilter(0, 0, 2, 0);
    ws.set_sort(0, SortDirection::Ascending);

    let buf1 = wb.save_to_buffer().unwrap();

    // Verify the file is valid
    let _wb2 = Workbook::open_from_buffer(&buf1).unwrap();
    assert!(!buf1.is_empty());
}

// ============================================================
// Task 54: Advanced Autofilter
// ============================================================

#[test]
fn test_top10_filter() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Name").unwrap();
    ws.write(0, 1, "Score").unwrap();
    for i in 1u32..=20 {
        ws.write(i, 0, format!("Student{i}")).unwrap();
        ws.write(i, 1, (i as f64) * 5.0).unwrap();
    }

    ws.set_autofilter(0, 0, 20, 1);
    ws.filter_column_advanced(
        1,
        FilterRule::Top10 {
            top: true,
            percent: false,
            val: 10.0,
        },
    );

    let buf = wb.save_to_buffer().unwrap();

    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_top10 = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("sheet1.xml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("top10") {
                found_top10 = true;
                assert!(content.contains("top=\"1\""), "Should be top filter");
                assert!(content.contains("val=\"10\""), "Should have val=10");
            }
        }
    }
    assert!(found_top10, "Should have top10 element in sheet XML");
}

#[test]
fn test_date_filter() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Date").unwrap();
    ws.write(0, 1, "Value").unwrap();
    ws.write(1, 0, "2023-01-15").unwrap();
    ws.write(1, 1, 100.0).unwrap();
    ws.write(2, 0, "2023-06-20").unwrap();
    ws.write(2, 1, 200.0).unwrap();

    ws.set_autofilter(0, 0, 2, 1);
    ws.filter_column_advanced(
        0,
        FilterRule::DateFilter {
            year: 2023,
            month: Some(1),
            day: None,
        },
    );

    let buf = wb.save_to_buffer().unwrap();

    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_date_filter = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("sheet1.xml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("dateGroupItem") {
                found_date_filter = true;
                assert!(content.contains("year=\"2023\""), "Should have year=2023");
                assert!(content.contains("month=\"1\""), "Should have month=1");
            }
        }
    }
    assert!(found_date_filter, "Should have dateGroupItem in sheet XML");
}

#[test]
fn test_custom_filter_and() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Value").unwrap();
    ws.write(1, 0, 10.0).unwrap();
    ws.write(2, 0, 50.0).unwrap();
    ws.write(3, 0, 90.0).unwrap();

    ws.set_autofilter(0, 0, 3, 0);
    ws.filter_column_advanced(
        0,
        FilterRule::CustomFilter {
            and: true,
            conditions: vec![
                ("greaterThan".to_string(), "20".to_string()),
                ("lessThan".to_string(), "80".to_string()),
            ],
        },
    );

    let buf = wb.save_to_buffer().unwrap();

    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_custom = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("sheet1.xml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("customFilters") {
                found_custom = true;
                assert!(content.contains("and=\"1\""), "Should be AND filter");
                assert!(
                    content.contains("greaterThan"),
                    "Should have greaterThan operator"
                );
                assert!(
                    content.contains("lessThan"),
                    "Should have lessThan operator"
                );
            }
        }
    }
    assert!(found_custom, "Should have customFilters in sheet XML");
}

#[test]
fn test_custom_filter_or() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Status").unwrap();
    ws.write(1, 0, "Active").unwrap();
    ws.write(2, 0, "Inactive").unwrap();
    ws.write(3, 0, "Pending").unwrap();

    ws.set_autofilter(0, 0, 3, 0);
    ws.filter_column_advanced(
        0,
        FilterRule::CustomFilter {
            and: false,
            conditions: vec![
                ("equal".to_string(), "Active".to_string()),
                ("equal".to_string(), "Pending".to_string()),
            ],
        },
    );

    let buf = wb.save_to_buffer().unwrap();

    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_custom = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("sheet1.xml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("customFilters") {
                found_custom = true;
                assert!(content.contains("and=\"0\""), "Should be OR filter (and=0)");
                assert!(content.contains("Active"), "Should have Active value");
                assert!(content.contains("Pending"), "Should have Pending value");
            }
        }
    }
    assert!(
        found_custom,
        "Should have customFilters with OR logic in sheet XML"
    );
}

#[test]
fn test_bottom_10_percent_filter() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Score").unwrap();
    for i in 1u32..=10 {
        ws.write(i, 0, (i as f64) * 10.0).unwrap();
    }

    ws.set_autofilter(0, 0, 10, 0);
    ws.filter_column_advanced(
        0,
        FilterRule::Top10 {
            top: false,
            percent: true,
            val: 25.0,
        },
    );

    let buf = wb.save_to_buffer().unwrap();

    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut found_top10 = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name().contains("sheet1.xml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("top10") {
                found_top10 = true;
                assert!(content.contains("top=\"0\""), "Should be bottom filter");
                assert!(
                    content.contains("percent=\"1\""),
                    "Should be percent filter"
                );
            }
        }
    }
    assert!(
        found_top10,
        "Should have top10 element for bottom percent filter"
    );
}
