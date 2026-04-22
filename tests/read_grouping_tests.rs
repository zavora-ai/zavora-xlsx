use zavora_xlsx::Workbook;

/// Round-trip test: group rows and columns, save, read back, verify outline levels.
#[test]
fn test_row_grouping_roundtrip() {
    let path = "test_row_grouping_roundtrip.xlsx";

    // Write: group rows 1-3 at level 1, rows 4-5 at level 2
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Header").unwrap();
        for r in 1..=5 {
            ws.write(r, 0, format!("Row {r}")).unwrap();
        }
        ws.group_rows(1, 3, 1);
        ws.group_rows(4, 5, 2);
        wb.save(path).unwrap();
    }

    // Read back and verify
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet_ref(0).unwrap();
        let row_levels = ws.row_outline_levels();

        // Rows 1-3 should be level 1
        assert_eq!(
            row_levels.get(&1),
            Some(&1),
            "Row 1 should have outline level 1"
        );
        assert_eq!(
            row_levels.get(&2),
            Some(&1),
            "Row 2 should have outline level 1"
        );
        assert_eq!(
            row_levels.get(&3),
            Some(&1),
            "Row 3 should have outline level 1"
        );

        // Rows 4-5 should be level 2
        assert_eq!(
            row_levels.get(&4),
            Some(&2),
            "Row 4 should have outline level 2"
        );
        assert_eq!(
            row_levels.get(&5),
            Some(&2),
            "Row 5 should have outline level 2"
        );

        // Row 0 should have no outline level
        assert_eq!(
            row_levels.get(&0),
            None,
            "Row 0 should have no outline level"
        );
    }

    std::fs::remove_file(path).ok();
}

#[test]
fn test_col_grouping_roundtrip() {
    let path = "test_col_grouping_roundtrip.xlsx";

    // Write: group columns B-D (1-3) at level 1, columns E-F (4-5) at level 2
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "A").unwrap();
        ws.write(0, 1, "B").unwrap();
        ws.write(0, 2, "C").unwrap();
        ws.write(0, 3, "D").unwrap();
        ws.write(0, 4, "E").unwrap();
        ws.write(0, 5, "F").unwrap();
        ws.group_columns(1, 3, 1);
        ws.group_columns(4, 5, 2);
        wb.save(path).unwrap();
    }

    // Read back and verify
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet_ref(0).unwrap();
        let col_levels = ws.col_outline_levels();

        // Columns 1-3 should be level 1
        assert_eq!(
            col_levels.get(&1),
            Some(&1),
            "Col B should have outline level 1"
        );
        assert_eq!(
            col_levels.get(&2),
            Some(&1),
            "Col C should have outline level 1"
        );
        assert_eq!(
            col_levels.get(&3),
            Some(&1),
            "Col D should have outline level 1"
        );

        // Columns 4-5 should be level 2
        assert_eq!(
            col_levels.get(&4),
            Some(&2),
            "Col E should have outline level 2"
        );
        assert_eq!(
            col_levels.get(&5),
            Some(&2),
            "Col F should have outline level 2"
        );

        // Column 0 should have no outline level
        assert_eq!(
            col_levels.get(&0),
            None,
            "Col A should have no outline level"
        );
    }

    std::fs::remove_file(path).ok();
}

#[test]
fn test_mixed_row_col_grouping_roundtrip() {
    let path = "test_mixed_grouping_roundtrip.xlsx";

    // Write: group both rows and columns
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        for r in 0..6 {
            for c in 0..6 {
                ws.write(r, c, format!("R{r}C{c}")).unwrap();
            }
        }
        ws.group_rows(1, 3, 1);
        ws.group_columns(2, 4, 1);
        wb.save(path).unwrap();
    }

    // Read back and verify both
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let ws = wb.worksheet_ref(0).unwrap();

        let row_levels = ws.row_outline_levels();
        assert_eq!(row_levels.len(), 3);
        assert_eq!(row_levels.get(&1), Some(&1));
        assert_eq!(row_levels.get(&2), Some(&1));
        assert_eq!(row_levels.get(&3), Some(&1));

        let col_levels = ws.col_outline_levels();
        assert_eq!(col_levels.len(), 3);
        assert_eq!(col_levels.get(&2), Some(&1));
        assert_eq!(col_levels.get(&3), Some(&1));
        assert_eq!(col_levels.get(&4), Some(&1));
    }

    std::fs::remove_file(path).ok();
}

#[test]
fn test_grouping_edit_mode_roundtrip() {
    let path = "test_grouping_edit_mode.xlsx";

    // Write initial file with grouping
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Header").unwrap();
        ws.write(1, 0, "Detail 1").unwrap();
        ws.write(2, 0, "Detail 2").unwrap();
        ws.write(0, 1, "Col B").unwrap();
        ws.write(0, 2, "Col C").unwrap();
        ws.group_rows(1, 2, 1);
        ws.group_columns(1, 2, 1);
        wb.save(path).unwrap();
    }

    // Open in edit mode, verify grouping is accessible
    {
        let mut wb = Workbook::open(path).unwrap();
        let ws = wb.worksheet(0).unwrap();

        let row_levels = ws.row_outline_levels();
        assert_eq!(row_levels.get(&1), Some(&1));
        assert_eq!(row_levels.get(&2), Some(&1));

        let col_levels = ws.col_outline_levels();
        assert_eq!(col_levels.get(&1), Some(&1));
        assert_eq!(col_levels.get(&2), Some(&1));
    }

    std::fs::remove_file(path).ok();
}
