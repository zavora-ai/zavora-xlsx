use zavora_xlsx::{DefinedName, DefinedNameScope, Workbook};

/// Round-trip test: create workbook with both global and sheet-scoped defined names,
/// save, read back, and verify scope information is preserved.
#[test]
fn test_defined_names_with_scope_roundtrip() {
    let path = "test_defined_names_scope.xlsx";

    // Create workbook with multiple sheets and defined names
    {
        let mut wb = Workbook::new();
        wb.add_worksheet_with_name("Sales").unwrap();
        wb.add_worksheet_with_name("Budget").unwrap();

        // Global (workbook-scoped) defined name
        wb.define_name("TaxRate", "0.08");

        // Sheet-scoped defined names
        wb.define_name_scoped("DataRange", "'Sales'!$A$1:$D$100", 1);
        wb.define_name_scoped("DataRange", "'Budget'!$A$1:$C$50", 2);

        wb.save(path).unwrap();
    }

    // Read back and verify
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let scoped = wb.defined_names_with_scope();

        assert_eq!(scoped.len(), 3, "Expected 3 defined names, got {}", scoped.len());

        // Verify global name
        let tax = scoped.iter().find(|dn| dn.name == "TaxRate").expect("TaxRate not found");
        assert_eq!(tax.formula, "0.08");
        assert_eq!(tax.scope, DefinedNameScope::Workbook);

        // Verify sheet-scoped names (both named "DataRange" but different scopes)
        let data_ranges: Vec<&DefinedName> = scoped.iter().filter(|dn| dn.name == "DataRange").collect();
        assert_eq!(data_ranges.len(), 2, "Expected 2 DataRange names");

        let sales_range = data_ranges.iter().find(|dn| dn.scope == DefinedNameScope::Sheet(1))
            .expect("DataRange scoped to sheet 1 not found");
        assert_eq!(sales_range.formula, "'Sales'!$A$1:$D$100");

        let budget_range = data_ranges.iter().find(|dn| dn.scope == DefinedNameScope::Sheet(2))
            .expect("DataRange scoped to sheet 2 not found");
        assert_eq!(budget_range.formula, "'Budget'!$A$1:$C$50");

        // Verify backward compatibility: legacy defined_names() still works
        let legacy = wb.defined_names();
        assert_eq!(legacy.len(), 3);
        assert!(legacy.iter().any(|(n, _)| n == "TaxRate"));
    }

    std::fs::remove_file(path).ok();
}

/// Test that workbook-only defined names (no sheet scope) round-trip correctly.
#[test]
fn test_global_only_defined_names() {
    let path = "test_global_defined_names.xlsx";

    {
        let mut wb = Workbook::new();
        wb.define_name("Revenue", "Sheet1!$A$1");
        wb.define_name("Expenses", "Sheet1!$B$1");
        wb.save(path).unwrap();
    }

    {
        let wb = Workbook::open_readonly(path).unwrap();
        let scoped = wb.defined_names_with_scope();

        assert_eq!(scoped.len(), 2);
        for dn in scoped {
            assert_eq!(dn.scope, DefinedNameScope::Workbook);
        }
    }

    std::fs::remove_file(path).ok();
}

/// Test that a workbook with no defined names returns empty slices.
#[test]
fn test_no_defined_names() {
    let path = "test_no_defined_names.xlsx";

    {
        let mut wb = Workbook::new();
        wb.save(path).unwrap();
    }

    {
        let wb = Workbook::open_readonly(path).unwrap();
        assert!(wb.defined_names_with_scope().is_empty());
        assert!(wb.defined_names().is_empty());
    }

    std::fs::remove_file(path).ok();
}

/// Test edit mode preserves scoped defined names.
#[test]
fn test_edit_mode_preserves_scoped_names() {
    let path = "test_edit_scoped_names.xlsx";
    let path2 = "test_edit_scoped_names_resaved.xlsx";

    // Create with scoped names
    {
        let mut wb = Workbook::new();
        wb.add_worksheet_with_name("Data").unwrap();
        wb.define_name("GlobalName", "Sheet1!$A$1");
        wb.define_name_scoped("LocalName", "'Data'!$B$1:$B$50", 1);
        wb.save(path).unwrap();
    }

    // Open in edit mode, modify something, re-save
    {
        let mut wb = Workbook::open(path).unwrap();
        let ws = wb.worksheet(0).unwrap();
        ws.write(0, 0, "Updated").unwrap();
        wb.save(path2).unwrap();
    }

    // Read back and verify names are preserved
    {
        let wb = Workbook::open_readonly(path2).unwrap();
        let scoped = wb.defined_names_with_scope();

        assert_eq!(scoped.len(), 2);

        let global = scoped.iter().find(|dn| dn.name == "GlobalName").unwrap();
        assert_eq!(global.scope, DefinedNameScope::Workbook);

        let local = scoped.iter().find(|dn| dn.name == "LocalName").unwrap();
        assert_eq!(local.scope, DefinedNameScope::Sheet(1));
        assert_eq!(local.formula, "'Data'!$B$1:$B$50");
    }

    std::fs::remove_file(path).ok();
    std::fs::remove_file(path2).ok();
}
