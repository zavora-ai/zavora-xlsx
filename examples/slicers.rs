/// Example: Add slicers to a table and verify file structure.
///
/// Creates a workbook with a data table and two slicers (visual filter controls),
/// then saves to `output/slicers_example.xlsx`.
use zavora_xlsx::{Slicer, Table, TableColumn, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write table data
    ws.write(0, 0, "Region").unwrap();
    ws.write(0, 1, "Product").unwrap();
    ws.write(0, 2, "Sales").unwrap();
    ws.write(0, 3, "Quarter").unwrap();

    let data = [
        ("East", "Widget", 1200, "Q1"),
        ("West", "Widget", 900, "Q1"),
        ("East", "Gadget", 1500, "Q1"),
        ("West", "Gadget", 1100, "Q2"),
        ("East", "Widget", 1300, "Q2"),
        ("West", "Widget", 800, "Q2"),
        ("East", "Gadget", 1600, "Q3"),
        ("West", "Gadget", 1200, "Q3"),
        ("East", "Widget", 1400, "Q4"),
        ("West", "Widget", 950, "Q4"),
    ];

    for (i, (region, product, sales, quarter)) in data.iter().enumerate() {
        let row = (i + 1) as u32;
        ws.write(row, 0, *region).unwrap();
        ws.write(row, 1, *product).unwrap();
        ws.write(row, 2, *sales).unwrap();
        ws.write(row, 3, *quarter).unwrap();
    }

    // Add a table spanning the data
    let mut table = Table::new();
    table.set_columns(&[
        TableColumn::new("Region"),
        TableColumn::new("Product"),
        TableColumn::new("Sales"),
        TableColumn::new("Quarter"),
    ]);
    table.set_name("SalesData");
    ws.add_table(0, 0, data.len() as u32, 3, &table).unwrap();

    // Add slicers for Region and Product columns
    let region_slicer = Slicer::new("Region", "Region")
        .set_caption("Region")
        .set_width(200)
        .set_height(250)
        .set_style("SlicerStyleLight1");

    let product_slicer = Slicer::new("Product", "Product")
        .set_caption("Product")
        .set_width(200)
        .set_height(250)
        .set_style("SlicerStyleLight2");

    ws.add_slicer(0, 5, &region_slicer).unwrap();
    ws.add_slicer(0, 9, &product_slicer).unwrap();

    // Save the workbook
    std::fs::create_dir_all("output").unwrap();
    wb.save("output/slicers_example.xlsx").unwrap();

    println!("Saved output/slicers_example.xlsx");

    // Verify file structure
    let buf = std::fs::read("output/slicers_example.xlsx").unwrap();
    let cursor = std::io::Cursor::new(&buf);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();

    println!("\nVerifying file structure:");

    let has_slicer = archive.by_name("xl/slicers/slicer1.xml").is_ok();
    println!(
        "  xl/slicers/slicer1.xml: {}",
        if has_slicer { "OK" } else { "MISSING" }
    );

    let has_cache1 = archive.by_name("xl/slicerCaches/slicerCache1.xml").is_ok();
    println!(
        "  xl/slicerCaches/slicerCache1.xml: {}",
        if has_cache1 { "OK" } else { "MISSING" }
    );

    let has_cache2 = archive.by_name("xl/slicerCaches/slicerCache2.xml").is_ok();
    println!(
        "  xl/slicerCaches/slicerCache2.xml: {}",
        if has_cache2 { "OK" } else { "MISSING" }
    );

    // Check content types
    let ct_content = {
        let mut entry = archive.by_name("[Content_Types].xml").unwrap();
        let mut s = String::new();
        std::io::Read::read_to_string(&mut entry, &mut s).unwrap();
        s
    };
    let has_slicer_ct = ct_content.contains("application/vnd.ms-excel.slicer+xml");
    let has_cache_ct = ct_content.contains("application/vnd.ms-excel.slicerCache+xml");
    println!(
        "  Slicer content type: {}",
        if has_slicer_ct { "OK" } else { "MISSING" }
    );
    println!(
        "  Slicer cache content type: {}",
        if has_cache_ct { "OK" } else { "MISSING" }
    );

    assert!(
        has_slicer && has_cache1 && has_cache2 && has_slicer_ct && has_cache_ct,
        "File structure verification failed"
    );
    println!("\nAll checks passed!");
}
