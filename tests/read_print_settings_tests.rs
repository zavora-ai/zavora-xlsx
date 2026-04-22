use zavora_xlsx::{Orientation, PrintSettings, Workbook};

/// Round-trip test: set print settings, save, read back, verify margins and orientation.
#[test]
fn test_print_settings_round_trip_margins_and_orientation() {
    let path = "test_print_settings_round_trip.xlsx";

    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Print Settings Test").unwrap();

        let ps = PrintSettings::new()
            .paper_size(9) // A4
            .orientation(Orientation::Landscape)
            .margins(1.0, 1.0, 0.75, 0.75);
        ws.set_print_settings(&ps);

        wb.save(path).unwrap();
    }

    // Read back
    {
        let mut wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let ps = ws
            .print_settings()
            .expect("print_settings should be present");

        // Verify orientation
        assert!(
            matches!(ps.orientation, Some(Orientation::Landscape)),
            "Expected Landscape orientation, got {:?}",
            ps.orientation
        );

        // Verify paper size
        assert_eq!(ps.paper_size, Some(9), "Paper size should be 9 (A4)");

        // Verify margins
        let top = ps.margin_top.expect("margin_top should be set");
        assert!(
            (top - 1.0).abs() < 0.01,
            "Top margin should be 1.0, got {top}"
        );

        let bottom = ps.margin_bottom.expect("margin_bottom should be set");
        assert!(
            (bottom - 1.0).abs() < 0.01,
            "Bottom margin should be 1.0, got {bottom}"
        );

        let left = ps.margin_left.expect("margin_left should be set");
        assert!(
            (left - 0.75).abs() < 0.01,
            "Left margin should be 0.75, got {left}"
        );

        let right = ps.margin_right.expect("margin_right should be set");
        assert!(
            (right - 0.75).abs() < 0.01,
            "Right margin should be 0.75, got {right}"
        );
    }

    std::fs::remove_file(path).ok();
}

/// Round-trip test: set header/footer, save, read back, verify.
#[test]
fn test_print_settings_round_trip_header_footer() {
    let path = "test_print_settings_header_footer.xlsx";

    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Header/Footer Test").unwrap();

        let ps = PrintSettings::new()
            .header("&CConfidential Report")
            .footer("&LPage &P&R&D");
        ws.set_print_settings(&ps);

        wb.save(path).unwrap();
    }

    // Read back
    {
        let mut wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let ps = ws
            .print_settings()
            .expect("print_settings should be present");

        assert_eq!(
            ps.header.as_deref(),
            Some("&CConfidential Report"),
            "Header should match"
        );
        assert_eq!(
            ps.footer.as_deref(),
            Some("&LPage &P&R&D"),
            "Footer should match"
        );
    }

    std::fs::remove_file(path).ok();
}

/// Round-trip test: set page breaks, save, read back, verify.
#[test]
fn test_print_settings_round_trip_page_breaks() {
    let path = "test_print_settings_page_breaks.xlsx";

    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        for r in 0..50 {
            ws.write(r, 0, format!("Row {}", r + 1)).unwrap();
        }
        ws.set_page_breaks(&[9, 19, 29], &[3]);

        wb.save(path).unwrap();
    }

    // Read back
    {
        let mut wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let ps = ws
            .print_settings()
            .expect("print_settings should be present");

        assert_eq!(ps.row_breaks, vec![9, 19, 29], "Row breaks should match");
        assert_eq!(ps.col_breaks, vec![3], "Col breaks should match");
    }

    std::fs::remove_file(path).ok();
}

/// Round-trip test: set repeat rows/columns, save, read back, verify.
#[test]
fn test_print_settings_round_trip_repeat_rows_cols() {
    let path = "test_print_settings_repeat.xlsx";

    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Header Row").unwrap();
        for r in 1..20 {
            ws.write(r, 0, format!("Data {r}")).unwrap();
        }
        ws.set_repeat_rows(0, 1);
        ws.set_repeat_columns(0, 0);

        wb.save(path).unwrap();
    }

    // Read back
    {
        let mut wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let ps = ws
            .print_settings()
            .expect("print_settings should be present");

        assert_eq!(ps.repeat_rows, Some((0, 1)), "Repeat rows should be (0, 1)");
        assert_eq!(ps.repeat_cols, Some((0, 0)), "Repeat cols should be (0, 0)");
    }

    std::fs::remove_file(path).ok();
}

/// Round-trip test: set fit-to-page, save, read back, verify.
#[test]
fn test_print_settings_round_trip_fit_to_page() {
    let path = "test_print_settings_fit_to_page.xlsx";

    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Fit to page test").unwrap();
        ws.set_fit_to_page(1, 2);

        wb.save(path).unwrap();
    }

    // Read back
    {
        let mut wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let ps = ws
            .print_settings()
            .expect("print_settings should be present");

        assert!(ps.fit_to_page, "fit_to_page should be true");
        assert_eq!(ps.fit_to_width, Some(1), "fit_to_width should be 1");
        assert_eq!(ps.fit_to_height, Some(2), "fit_to_height should be 2");
    }

    std::fs::remove_file(path).ok();
}

/// Round-trip test: set scale, save, read back, verify.
#[test]
fn test_print_settings_round_trip_scale() {
    let path = "test_print_settings_scale.xlsx";

    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Scale test").unwrap();
        ws.set_print_scale(75);

        wb.save(path).unwrap();
    }

    // Read back
    {
        let mut wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let ps = ws
            .print_settings()
            .expect("print_settings should be present");

        assert_eq!(ps.scale, Some(75), "Scale should be 75%");
    }

    std::fs::remove_file(path).ok();
}

/// Round-trip test: portrait orientation.
#[test]
fn test_print_settings_round_trip_portrait() {
    let path = "test_print_settings_portrait.xlsx";

    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Portrait test").unwrap();
        ws.set_portrait();

        wb.save(path).unwrap();
    }

    // Read back
    {
        let mut wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let ps = ws
            .print_settings()
            .expect("print_settings should be present");

        assert!(
            matches!(ps.orientation, Some(Orientation::Portrait)),
            "Expected Portrait orientation, got {:?}",
            ps.orientation
        );
    }

    std::fs::remove_file(path).ok();
}

/// Round-trip test: header/footer margins.
#[test]
fn test_print_settings_round_trip_header_footer_margins() {
    let path = "test_print_settings_hf_margins.xlsx";

    // Write
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "HF Margins test").unwrap();

        let mut ps = PrintSettings::new().margins(1.0, 1.0, 0.5, 0.5);
        ps.margin_header = Some(0.5);
        ps.margin_footer = Some(0.5);
        ws.set_print_settings(&ps);

        wb.save(path).unwrap();
    }

    // Read back
    {
        let mut wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        let ps = ws
            .print_settings()
            .expect("print_settings should be present");

        let hdr = ps.margin_header.expect("header margin should be set");
        assert!(
            (hdr - 0.5).abs() < 0.01,
            "Header margin should be 0.5, got {hdr}"
        );

        let ftr = ps.margin_footer.expect("footer margin should be set");
        assert!(
            (ftr - 0.5).abs() < 0.01,
            "Footer margin should be 0.5, got {ftr}"
        );
    }

    std::fs::remove_file(path).ok();
}
