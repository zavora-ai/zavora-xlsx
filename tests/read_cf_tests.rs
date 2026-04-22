use zavora_xlsx::*;

/// Helper: create a workbook, save it, reopen readonly, and return the first worksheet.
fn roundtrip_workbook(setup: impl FnOnce(&mut Worksheet)) -> Workbook {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cf_roundtrip.xlsx");

    {
        let mut wb = Workbook::new();
        let ws = wb.worksheet(0).unwrap();
        setup(ws);
        wb.save(&path).unwrap();
    }

    Workbook::open_readonly(&path).unwrap()
}

#[test]
fn test_cell_value_cf_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let bold_fmt = Format::new().bold();
        let mut rule = ConditionalFormatCell::new(CfOperator::GreaterThan, 50.0);
        rule.set_format(&bold_fmt);
        ws.add_conditional_format(0, 0, 9, 0, rule).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let cfs = ws.conditional_formats();

    assert_eq!(cfs.len(), 1, "expected 1 CF rule");
    assert_eq!(cfs[0].range, (0, 0, 9, 0), "range should be A1:A10");
    assert_eq!(
        cfs[0].rule.cf_type(),
        "cellIs",
        "rule type should be cellIs"
    );
    assert!(
        cfs[0].dxf_id.is_some(),
        "dxf_id should be Some for a rule with format"
    );
    assert_eq!(cfs[0].dxf_id, Some(0), "dxf_id should be 0");
}

#[test]
fn test_2color_scale_cf_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let rule = ConditionalFormat2ColorScale::new(
            (255u8, 0u8, 0u8), // red min
            (0u8, 255u8, 0u8), // green max
        );
        ws.add_conditional_format(0, 1, 9, 1, rule).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let cfs = ws.conditional_formats();

    assert_eq!(cfs.len(), 1, "expected 1 CF rule");
    assert_eq!(cfs[0].range, (0, 1, 9, 1), "range should be B1:B10");
    assert_eq!(
        cfs[0].rule.cf_type(),
        "colorScale",
        "rule type should be colorScale"
    );
}

#[test]
fn test_data_bar_cf_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let rule = ConditionalFormatDataBar::new((0u8, 0u8, 255u8)); // blue
        ws.add_conditional_format(0, 2, 9, 2, rule).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let cfs = ws.conditional_formats();

    assert_eq!(cfs.len(), 1, "expected 1 CF rule");
    assert_eq!(cfs[0].range, (0, 2, 9, 2), "range should be C1:C10");
    assert_eq!(
        cfs[0].rule.cf_type(),
        "dataBar",
        "rule type should be dataBar"
    );
}

#[test]
fn test_duplicate_values_cf_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        let bold_fmt = Format::new().bold();
        let mut rule = ConditionalFormatDuplicate::new();
        rule.set_format(&bold_fmt);
        ws.add_conditional_format(0, 3, 9, 3, rule).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let cfs = ws.conditional_formats();

    assert_eq!(cfs.len(), 1, "expected 1 CF rule");
    assert_eq!(cfs[0].range, (0, 3, 9, 3), "range should be D1:D10");
    assert_eq!(
        cfs[0].rule.cf_type(),
        "duplicateValues",
        "rule type should be duplicateValues"
    );
}

#[test]
fn test_multiple_cf_rules_roundtrip() {
    let mut wb = roundtrip_workbook(|ws| {
        // Rule 1: cell value on A1:A10
        let mut cell_rule = ConditionalFormatCell::new(CfOperator::LessThan, 10.0);
        cell_rule.set_format(&Format::new().italic());
        ws.add_conditional_format(0, 0, 9, 0, cell_rule).unwrap();

        // Rule 2: 2-color scale on B1:B20
        let scale_rule = ConditionalFormat2ColorScale::new(
            (255u8, 255u8, 0u8), // yellow min
            (0u8, 128u8, 0u8),   // dark green max
        );
        ws.add_conditional_format(0, 1, 19, 1, scale_rule).unwrap();

        // Rule 3: data bar on C1:C5
        let bar_rule = ConditionalFormatDataBar::new((128u8, 0u8, 128u8)); // purple
        ws.add_conditional_format(0, 2, 4, 2, bar_rule).unwrap();
    });

    let ws = wb.worksheet(0).unwrap();
    let cfs = ws.conditional_formats();

    assert_eq!(cfs.len(), 3, "expected 3 CF rules");

    // Verify each rule's range and type
    assert_eq!(cfs[0].range, (0, 0, 9, 0), "rule 1 range should be A1:A10");
    assert_eq!(
        cfs[0].rule.cf_type(),
        "cellIs",
        "rule 1 type should be cellIs"
    );

    assert_eq!(cfs[1].range, (0, 1, 19, 1), "rule 2 range should be B1:B20");
    assert_eq!(
        cfs[1].rule.cf_type(),
        "colorScale",
        "rule 2 type should be colorScale"
    );

    assert_eq!(cfs[2].range, (0, 2, 4, 2), "rule 3 range should be C1:C5");
    assert_eq!(
        cfs[2].rule.cf_type(),
        "dataBar",
        "rule 3 type should be dataBar"
    );
}
