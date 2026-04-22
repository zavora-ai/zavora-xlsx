use zavora_xlsx::{DefinedNameScope, Workbook};

/// Test adding named ranges with different scopes.
#[test]
fn test_add_named_range_workbook_scope() {
    let mut wb = Workbook::new();
    wb.add_named_range("TaxRate", "0.08", DefinedNameScope::Workbook)
        .unwrap();

    let names = wb.defined_names_with_scope();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "TaxRate");
    assert_eq!(names[0].formula, "0.08");
    assert_eq!(names[0].scope, DefinedNameScope::Workbook);

    // Legacy list should also have it
    let legacy = wb.defined_names();
    assert_eq!(legacy.len(), 1);
    assert_eq!(legacy[0], ("TaxRate".to_string(), "0.08".to_string()));
}

#[test]
fn test_add_named_range_sheet_scope() {
    let mut wb = Workbook::new();
    wb.add_named_range("LocalName", "Sheet1!$A$1:$B$10", DefinedNameScope::Sheet(0))
        .unwrap();

    let names = wb.defined_names_with_scope();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "LocalName");
    assert_eq!(names[0].formula, "Sheet1!$A$1:$B$10");
    assert_eq!(names[0].scope, DefinedNameScope::Sheet(0));

    // Legacy list should NOT have sheet-scoped names
    assert!(wb.defined_names().is_empty());
}

#[test]
fn test_add_named_range_duplicate_error() {
    let mut wb = Workbook::new();
    wb.add_named_range("MyName", "Sheet1!$A$1", DefinedNameScope::Workbook)
        .unwrap();

    // Same name and scope should fail
    let result = wb.add_named_range("MyName", "Sheet1!$B$1", DefinedNameScope::Workbook);
    assert!(result.is_err());

    // Same name but different scope should succeed
    wb.add_named_range("MyName", "Sheet1!$C$1", DefinedNameScope::Sheet(0))
        .unwrap();
    assert_eq!(wb.defined_names_with_scope().len(), 2);
}

/// Test updating a named range formula.
#[test]
fn test_update_named_range() {
    let mut wb = Workbook::new();
    wb.add_named_range("Revenue", "Sheet1!$A$1", DefinedNameScope::Workbook)
        .unwrap();

    wb.update_named_range("Revenue", "Sheet1!$A$1:$A$100")
        .unwrap();

    let names = wb.defined_names_with_scope();
    assert_eq!(names[0].formula, "Sheet1!$A$1:$A$100");

    // Legacy list should also be updated
    assert_eq!(wb.defined_names()[0].1, "Sheet1!$A$1:$A$100");
}

#[test]
fn test_update_named_range_not_found() {
    let mut wb = Workbook::new();
    let result = wb.update_named_range("NonExistent", "Sheet1!$A$1");
    assert!(result.is_err());
}

/// Test removing a named range.
#[test]
fn test_remove_named_range_workbook_scope() {
    let mut wb = Workbook::new();
    wb.add_named_range("ToRemove", "Sheet1!$A$1", DefinedNameScope::Workbook)
        .unwrap();
    wb.add_named_range("ToKeep", "Sheet1!$B$1", DefinedNameScope::Workbook)
        .unwrap();

    wb.remove_named_range("ToRemove", &DefinedNameScope::Workbook)
        .unwrap();

    let names = wb.defined_names_with_scope();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "ToKeep");

    // Legacy list should also be updated
    assert_eq!(wb.defined_names().len(), 1);
    assert_eq!(wb.defined_names()[0].0, "ToKeep");
}

#[test]
fn test_remove_named_range_sheet_scope() {
    let mut wb = Workbook::new();
    wb.add_named_range("LocalName", "Sheet1!$A$1", DefinedNameScope::Sheet(0))
        .unwrap();
    wb.add_named_range("LocalName", "Sheet2!$A$1", DefinedNameScope::Sheet(1))
        .unwrap();

    wb.remove_named_range("LocalName", &DefinedNameScope::Sheet(0))
        .unwrap();

    let names = wb.defined_names_with_scope();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].scope, DefinedNameScope::Sheet(1));
}

#[test]
fn test_remove_named_range_not_found() {
    let mut wb = Workbook::new();
    let result = wb.remove_named_range("NonExistent", &DefinedNameScope::Workbook);
    assert!(result.is_err());
}

/// Test that CRUD operations serialize correctly to XML (round-trip).
#[test]
fn test_named_ranges_crud_roundtrip() {
    let path = "output/test_named_ranges_crud.xlsx";

    // Create workbook, add names, update one, remove one, save
    {
        let mut wb = Workbook::new();
        wb.add_worksheet_with_name("Data").unwrap();

        wb.add_named_range(
            "SalesTotal",
            "Sheet1!$B$2:$B$10",
            DefinedNameScope::Workbook,
        )
        .unwrap();
        wb.add_named_range("LocalName", "'Data'!$A$1", DefinedNameScope::Sheet(1))
            .unwrap();
        wb.add_named_range("ToDelete", "Sheet1!$C$1", DefinedNameScope::Workbook)
            .unwrap();

        // Update SalesTotal
        wb.update_named_range("SalesTotal", "Sheet1!$B$2:$B$100")
            .unwrap();

        // Remove ToDelete
        wb.remove_named_range("ToDelete", &DefinedNameScope::Workbook)
            .unwrap();

        wb.save(path).unwrap();
    }

    // Read back and verify
    {
        let wb = Workbook::open_readonly(path).unwrap();
        let names = wb.defined_names_with_scope();

        assert_eq!(
            names.len(),
            2,
            "Expected 2 defined names after CRUD, got {}",
            names.len()
        );

        let sales = names
            .iter()
            .find(|dn| dn.name == "SalesTotal")
            .expect("SalesTotal not found");
        assert_eq!(sales.formula, "Sheet1!$B$2:$B$100");
        assert_eq!(sales.scope, DefinedNameScope::Workbook);

        let local = names
            .iter()
            .find(|dn| dn.name == "LocalName")
            .expect("LocalName not found");
        assert_eq!(local.formula, "'Data'!$A$1");
        assert_eq!(local.scope, DefinedNameScope::Sheet(1));

        // ToDelete should not be present
        assert!(names.iter().find(|dn| dn.name == "ToDelete").is_none());
    }

    std::fs::remove_file(path).ok();
}

/// Test that the XML output contains correct definedName elements with localSheetId.
#[test]
fn test_named_ranges_xml_serialization() {
    let path = "output/test_named_ranges_xml.xlsx";

    let mut wb = Workbook::new();
    wb.add_named_range("GlobalName", "Sheet1!$A$1", DefinedNameScope::Workbook)
        .unwrap();
    wb.add_named_range("SheetLocal", "Sheet1!$B$1", DefinedNameScope::Sheet(0))
        .unwrap();
    wb.save(path).unwrap();

    // Read the raw workbook.xml from the zip to verify XML structure
    let file = std::fs::File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut workbook_xml = String::new();
    {
        use std::io::Read;
        let mut entry = archive.by_name("xl/workbook.xml").unwrap();
        entry.read_to_string(&mut workbook_xml).unwrap();
    }

    // Should contain definedNames section
    assert!(
        workbook_xml.contains("<definedNames>"),
        "XML should contain <definedNames>"
    );
    assert!(
        workbook_xml.contains("</definedNames>"),
        "XML should contain </definedNames>"
    );

    // Global name without localSheetId
    assert!(
        workbook_xml.contains(r#"<definedName name="GlobalName">Sheet1!$A$1</definedName>"#),
        "XML should contain global defined name, got: {}",
        workbook_xml
    );

    // Sheet-scoped name with localSheetId="0"
    assert!(
        workbook_xml.contains(
            r#"<definedName name="SheetLocal" localSheetId="0">Sheet1!$B$1</definedName>"#
        ),
        "XML should contain sheet-scoped defined name, got: {}",
        workbook_xml
    );

    std::fs::remove_file(path).ok();
}
