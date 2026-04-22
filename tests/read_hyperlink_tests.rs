use zavora_xlsx::Workbook;

#[test]
fn round_trip_external_hyperlink() {
    let path = "test_output_read_hyperlink_external.xlsx";
    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write_url(0, 0, "https://example.com", "Example")
            .unwrap();
        ws.write_url(1, 0, "https://rust-lang.org", "").unwrap();
        wb.save(path).unwrap();
    }
    // Read back
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet_ref(0).unwrap();
        let links = ws.hyperlinks();
        assert_eq!(links.len(), 2, "expected 2 hyperlinks, got {}", links.len());

        assert_eq!(links[0].row, 0);
        assert_eq!(links[0].col, 0);
        assert_eq!(links[0].url, "https://example.com");
        assert!(links[0].location.is_none());

        assert_eq!(links[1].row, 1);
        assert_eq!(links[1].col, 0);
        assert_eq!(links[1].url, "https://rust-lang.org");
        assert!(links[1].location.is_none());
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn round_trip_internal_hyperlink() {
    let path = "test_output_read_hyperlink_internal.xlsx";
    // Write
    {
        let mut wb = Workbook::new();
        wb.add_worksheet();
        let ws = wb.worksheet(0).unwrap();
        ws.write_internal_link(0, 0, "Sheet2!A1", "Go to Sheet2")
            .unwrap();
        wb.save(path).unwrap();
    }
    // Read back
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet_ref(0).unwrap();
        let links = ws.hyperlinks();
        assert_eq!(links.len(), 1, "expected 1 hyperlink, got {}", links.len());

        assert_eq!(links[0].row, 0);
        assert_eq!(links[0].col, 0);
        assert!(links[0].url.is_empty());
        assert_eq!(links[0].location.as_deref(), Some("Sheet2!A1"));
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn round_trip_mixed_hyperlinks() {
    let path = "test_output_read_hyperlink_mixed.xlsx";
    // Write
    {
        let mut wb = Workbook::new();
        wb.add_worksheet();
        let ws = wb.worksheet(0).unwrap();
        ws.write_url(0, 0, "https://github.com", "GitHub").unwrap();
        ws.write_internal_link(1, 0, "Sheet2!B5", "Jump").unwrap();
        ws.write_url(2, 0, "https://docs.rs", "Docs").unwrap();
        wb.save(path).unwrap();
    }
    // Read back
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet_ref(0).unwrap();
        let links = ws.hyperlinks();
        assert_eq!(links.len(), 3);

        // External
        assert_eq!(links[0].url, "https://github.com");
        assert!(links[0].location.is_none());

        // Internal
        assert!(links[1].url.is_empty());
        assert_eq!(links[1].location.as_deref(), Some("Sheet2!B5"));

        // External
        assert_eq!(links[2].url, "https://docs.rs");
        assert!(links[2].location.is_none());
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn round_trip_hyperlinks_edit_mode() {
    let path = "test_output_read_hyperlink_edit.xlsx";
    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write_url(0, 0, "https://example.com", "Example")
            .unwrap();
        ws.write_internal_link(1, 0, "Sheet1!C3", "Self ref")
            .unwrap();
        wb.save(path).unwrap();
    }
    // Open in edit mode and read hyperlinks
    {
        let mut wb = Workbook::open(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let links = ws.hyperlinks();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://example.com");
        assert_eq!(links[1].location.as_deref(), Some("Sheet1!C3"));
    }
    let _ = std::fs::remove_file(path);
}
