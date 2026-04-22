use zavora_xlsx::{Slicer, Table, TableColumn, Workbook};

#[test]
fn test_slicer_with_table_file_structure() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write table data
    ws.write(0, 0, "Category").unwrap();
    ws.write(0, 1, "Value").unwrap();
    ws.write(1, 0, "A").unwrap();
    ws.write(1, 1, 10).unwrap();
    ws.write(2, 0, "B").unwrap();
    ws.write(2, 1, 20).unwrap();
    ws.write(3, 0, "C").unwrap();
    ws.write(3, 1, 30).unwrap();

    // Add a table
    let mut table = Table::new();
    table.set_columns(&[TableColumn::new("Category"), TableColumn::new("Value")]);
    table.set_name("SalesTable");
    ws.add_table(0, 0, 3, 1, &table).unwrap();

    // Add a slicer linked to the table's "Category" column
    let slicer = Slicer::new("Category", "Category")
        .set_caption("Category Filter")
        .set_width(250)
        .set_height(300)
        .set_style("SlicerStyleLight2");
    ws.add_slicer(0, 3, &slicer).unwrap();

    // Save to buffer
    let buf = wb.save_to_buffer().unwrap();

    // Verify the ZIP contains slicer and slicer cache parts
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();

    // Check slicer XML part exists
    assert!(
        archive.by_name("xl/slicers/slicer1.xml").is_ok(),
        "xl/slicers/slicer1.xml should exist"
    );

    // Check slicer cache XML part exists
    assert!(
        archive.by_name("xl/slicerCaches/slicerCache1.xml").is_ok(),
        "xl/slicerCaches/slicerCache1.xml should exist"
    );

    // Verify content types include slicer entries
    let ct_content = {
        let mut entry = archive.by_name("[Content_Types].xml").unwrap();
        let mut s = String::new();
        std::io::Read::read_to_string(&mut entry, &mut s).unwrap();
        s
    };
    assert!(
        ct_content.contains("application/vnd.ms-excel.slicer+xml"),
        "Content types should include slicer type"
    );
    assert!(
        ct_content.contains("application/vnd.ms-excel.slicerCache+xml"),
        "Content types should include slicer cache type"
    );

    // Verify slicer XML content
    let slicer_xml = {
        let mut entry = archive.by_name("xl/slicers/slicer1.xml").unwrap();
        let mut s = String::new();
        std::io::Read::read_to_string(&mut entry, &mut s).unwrap();
        s
    };
    assert!(
        slicer_xml.contains("name=\"Category\""),
        "Slicer XML should contain slicer name"
    );
    assert!(
        slicer_xml.contains("caption=\"Category Filter\""),
        "Slicer XML should contain caption"
    );
    assert!(
        slicer_xml.contains("style=\"SlicerStyleLight2\""),
        "Slicer XML should contain style"
    );

    // Verify slicer cache XML content
    let cache_xml = {
        let mut entry = archive.by_name("xl/slicerCaches/slicerCache1.xml").unwrap();
        let mut s = String::new();
        std::io::Read::read_to_string(&mut entry, &mut s).unwrap();
        s
    };
    assert!(
        cache_xml.contains("sourceName=\"Category\""),
        "Cache XML should contain source name"
    );
    assert!(
        cache_xml.contains("tableSlicerCache"),
        "Cache XML should contain tableSlicerCache element"
    );
}

#[test]
fn test_multiple_slicers() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write table data
    ws.write(0, 0, "Region").unwrap();
    ws.write(0, 1, "Product").unwrap();
    ws.write(0, 2, "Sales").unwrap();
    ws.write(1, 0, "East").unwrap();
    ws.write(1, 1, "Widget").unwrap();
    ws.write(1, 2, 100).unwrap();

    // Add a table
    let mut table = Table::new();
    table.set_columns(&[
        TableColumn::new("Region"),
        TableColumn::new("Product"),
        TableColumn::new("Sales"),
    ]);
    table.set_name("DataTable");
    ws.add_table(0, 0, 1, 2, &table).unwrap();

    // Add two slicers
    let slicer1 = Slicer::new("Region", "Region");
    let slicer2 = Slicer::new("Product", "Product");
    ws.add_slicer(0, 4, &slicer1).unwrap();
    ws.add_slicer(0, 8, &slicer2).unwrap();

    let buf = wb.save_to_buffer().unwrap();

    // Verify both slicer caches exist
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();

    assert!(archive.by_name("xl/slicers/slicer1.xml").is_ok());
    assert!(archive.by_name("xl/slicerCaches/slicerCache1.xml").is_ok());
    assert!(archive.by_name("xl/slicerCaches/slicerCache2.xml").is_ok());

    // Verify slicer XML contains both slicers
    let slicer_xml = {
        let mut entry = archive.by_name("xl/slicers/slicer1.xml").unwrap();
        let mut s = String::new();
        std::io::Read::read_to_string(&mut entry, &mut s).unwrap();
        s
    };
    assert!(slicer_xml.contains("name=\"Region\""));
    assert!(slicer_xml.contains("name=\"Product\""));
}
