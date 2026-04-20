use zavora_xlsx::{Table, TableColumn, TableStyle, Workbook};

#[test]
fn round_trip_table_columns_and_style() {
    // Create a workbook with a table
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Write header row and data
    ws.write(0, 0, "Name").unwrap();
    ws.write(0, 1, "Age").unwrap();
    ws.write(0, 2, "Score").unwrap();
    ws.write(1, 0, "Alice").unwrap();
    ws.write(1, 1, 30).unwrap();
    ws.write(1, 2, 95.5).unwrap();
    ws.write(2, 0, "Bob").unwrap();
    ws.write(2, 1, 25).unwrap();
    ws.write(2, 2, 88.0).unwrap();

    let cols = vec![
        TableColumn::new("Name"),
        TableColumn::new("Age"),
        TableColumn::new("Score"),
    ];
    let mut table = Table::new();
    table.set_columns(&cols);
    table.set_style(TableStyle::Medium(9));
    table.set_name("PeopleTable");
    ws.add_table(0, 0, 2, 2, &table).unwrap();

    // Save and read back
    let buf = wb.save_to_buffer().unwrap();
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();

    let tables = ws2.tables();
    assert_eq!(tables.len(), 1, "expected 1 table");

    let t = &tables[0];
    assert_eq!(t.table_name(), Some("PeopleTable"));
    assert_eq!(t.columns().len(), 3);
    assert_eq!(t.columns()[0].name(), "Name");
    assert_eq!(t.columns()[1].name(), "Age");
    assert_eq!(t.columns()[2].name(), "Score");
    assert!(t.autofilter(), "autofilter should be enabled");

    // Verify style
    let style = t.style().expect("table should have a style");
    assert_eq!(style.name(), "TableStyleMedium9");

    // Verify range
    assert_eq!(t.first_row(), 0);
    assert_eq!(t.first_col(), 0);
    assert_eq!(t.last_row(), 2);
    assert_eq!(t.last_col(), 2);
}

#[test]
fn round_trip_table_with_total_row() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "Item").unwrap();
    ws.write(0, 1, "Amount").unwrap();
    ws.write(1, 0, "Widget").unwrap();
    ws.write(1, 1, 100).unwrap();
    ws.write(2, 0, "Gadget").unwrap();
    ws.write(2, 1, 200).unwrap();

    let mut col_item = TableColumn::new("Item");
    col_item.set_total_label("Total");
    let mut col_amount = TableColumn::new("Amount");
    col_amount.set_total_function("sum");

    let mut table = Table::new();
    table.set_columns(&[col_item, col_amount]);
    table.set_style(TableStyle::Light(1));
    table.set_total_row(true);
    table.set_name("SalesTable");
    ws.add_table(0, 0, 2, 1, &table).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();

    let tables = ws2.tables();
    assert_eq!(tables.len(), 1);

    let t = &tables[0];
    assert_eq!(t.table_name(), Some("SalesTable"));
    assert_eq!(t.columns().len(), 2);
    assert_eq!(t.columns()[0].name(), "Item");
    assert_eq!(t.columns()[0].total_label(), Some("Total"));
    assert_eq!(t.columns()[1].name(), "Amount");
    assert_eq!(t.columns()[1].total_function(), Some("sum"));

    let style = t.style().expect("should have style");
    assert_eq!(style.name(), "TableStyleLight1");
}

#[test]
fn round_trip_table_no_autofilter() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    ws.write(0, 0, "X").unwrap();
    ws.write(0, 1, "Y").unwrap();
    ws.write(1, 0, 1).unwrap();
    ws.write(1, 1, 2).unwrap();

    let mut table = Table::new();
    table.set_columns(&[TableColumn::new("X"), TableColumn::new("Y")]);
    table.set_autofilter(false);
    table.set_style(TableStyle::Dark(3));
    ws.add_table(0, 0, 1, 1, &table).unwrap();

    let buf = wb.save_to_buffer().unwrap();
    let wb2 = Workbook::open_readonly_from_buffer(&buf).unwrap();
    let ws2 = wb2.worksheet_ref(0).unwrap();

    let tables = ws2.tables();
    assert_eq!(tables.len(), 1);
    let t = &tables[0];
    assert!(!t.autofilter(), "autofilter should be disabled");

    let style = t.style().expect("should have style");
    assert_eq!(style.name(), "TableStyleDark3");
}
