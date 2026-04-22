use zavora_xlsx::Workbook;

#[test]
fn test_protection_roundtrip_no_password() {
    let path = "target/test_protection_roundtrip_no_pw.xlsx";
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Protected").unwrap();
        ws.protect();
        wb.save(path).unwrap();
    }
    let wb = Workbook::open_readonly(path).unwrap();
    let ws = wb.worksheet_ref(0).unwrap();
    let prot = ws.protection().expect("protection should be present");
    assert!(prot.sheet, "sheet flag should be true");
    assert!(prot.objects, "objects flag should be true");
    assert!(prot.scenarios, "scenarios flag should be true");
    assert!(prot.password_hash().is_none(), "no password hash expected");
    // Default flags: most operations are protected (true)
    assert!(prot.format_cells);
    assert!(prot.format_columns);
    assert!(prot.format_rows);
    assert!(prot.insert_columns);
    assert!(prot.insert_rows);
    assert!(prot.insert_hyperlinks);
    assert!(prot.delete_columns);
    assert!(prot.delete_rows);
    assert!(prot.sort);
    assert!(prot.auto_filter);
    assert!(prot.pivot_tables);
    // These default to false (allowed)
    assert!(!prot.select_locked_cells);
    assert!(!prot.select_unlocked_cells);
    std::fs::remove_file(path).ok();
}

#[test]
fn test_protection_roundtrip_with_password() {
    let path = "target/test_protection_roundtrip_pw.xlsx";
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Secret").unwrap();
        ws.protect_with_password("hello");
        wb.save(path).unwrap();
    }
    let wb = Workbook::open_readonly(path).unwrap();
    let ws = wb.worksheet_ref(0).unwrap();
    let prot = ws.protection().expect("protection should be present");
    assert!(prot.sheet);
    assert!(
        prot.password_hash().is_some(),
        "password hash should be present"
    );
    // The hash should be a 4-character hex string
    let hash = prot.password_hash().unwrap();
    assert_eq!(hash.len(), 4, "legacy hash should be 4 hex chars");
    std::fs::remove_file(path).ok();
}

#[test]
fn test_no_protection_returns_none() {
    let path = "target/test_no_protection.xlsx";
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Unprotected").unwrap();
        wb.save(path).unwrap();
    }
    let wb = Workbook::open_readonly(path).unwrap();
    let ws = wb.worksheet_ref(0).unwrap();
    assert!(
        ws.protection().is_none(),
        "unprotected sheet should return None"
    );
    std::fs::remove_file(path).ok();
}

#[test]
fn test_protection_roundtrip_edit_mode() {
    let path = "target/test_protection_edit_mode.xlsx";
    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Edit mode test").unwrap();
        ws.protect_with_password("test123");
        wb.save(path).unwrap();
    }
    // Open in edit mode, modify a cell, save, then read back
    let path2 = "target/test_protection_edit_mode2.xlsx";
    {
        let mut wb = Workbook::open(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        ws.write(1, 0, "Added row").unwrap();
        wb.save(path2).unwrap();
    }
    let wb = Workbook::open_readonly(path2).unwrap();
    let ws = wb.worksheet_ref(0).unwrap();
    let prot = ws
        .protection()
        .expect("protection should survive edit mode");
    assert!(prot.sheet);
    assert!(prot.password_hash().is_some());
    std::fs::remove_file(path).ok();
    std::fs::remove_file(path2).ok();
}
